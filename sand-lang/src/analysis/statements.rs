//! Lowering for assignments, builtin statements, and text macros.

use super::*;
use crate::builtins::{self, Builtin, BuiltinKind};
use crate::properties::{Property, PropertyKind, PropertyType};
use std::collections::BTreeSet;

/// A value that needs no computation: a constant or one score read.
#[derive(Debug, Clone)]
pub(super) enum Value {
    /// A folded constant.
    Const(i64),
    /// A State field or local slot.
    Read(pc::FieldReference),
}

/// Where an assignment writes.
struct Dest {
    reference: pc::FieldReference,
    ty: Ty,
    bounds: Option<(i32, i32)>,
    name: String,
}

struct LoweredMatchArm {
    values: Vec<i32>,
    catch_all: bool,
    body: Vec<pc::Operation>,
    span: Span,
}

/// One local and its path inside an already evaluated pattern subject.
#[derive(Clone)]
pub(super) struct PatternLocal {
    name: Ident,
    def: DefId,
    ty: Ty,
    projection: Vec<String>,
}

/// What a `while`-shaped loop repeats on.
pub(super) enum Repeat<'a> {
    /// `while cond { }`.
    While(&'a Cond),
    /// `loop { }`.
    Forever,
    /// `for binding in range { }`.
    Count {
        binding: &'a Ident,
        range: &'a Range,
        reverse: bool,
        step: Option<&'a Expr>,
        /// Array snapshot supplying the loop binding instead of the counter.
        collection: Option<&'a Expr>,
    },
}

/// Whether the statement can `break` (or `continue`, when `is_break` is
/// false) out of the innermost enclosing loop, without entering a nested one.
pub(super) fn jumps_here(stmt: &Stmt, is_break: bool) -> bool {
    match stmt {
        Stmt::Break(_) => is_break,
        Stmt::Continue(_) => !is_break,
        Stmt::If(if_stmt) => if_jumps(if_stmt, is_break),
        Stmt::Match { arms, .. } => arms
            .iter()
            .any(|arm| arm.body.stmts.iter().any(|s| jumps_here(s, is_break))),
        _ => false,
    }
}

fn if_jumps(if_stmt: &IfStmt, is_break: bool) -> bool {
    if_stmt.then.stmts.iter().any(|s| jumps_here(s, is_break))
        || match &if_stmt.otherwise {
            None => false,
            Some(Else::Block(block)) => block.stmts.iter().any(|s| jumps_here(s, is_break)),
            Some(Else::If(nested)) => if_jumps(nested, is_break),
        }
}

/// Whether a `return` appears anywhere in the block.
fn block_mentions_return(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| match stmt {
        Stmt::Return { .. } => true,
        Stmt::If(if_stmt) => if_mentions_return(if_stmt),
        Stmt::Match { arms, .. } => arms.iter().any(|arm| block_mentions_return(&arm.body)),
        Stmt::While { body, .. }
        | Stmt::Loop { body, .. }
        | Stmt::ForRange { body, .. }
        | Stmt::ForValues { body, .. } => block_mentions_return(body),
        _ => false,
    })
}

fn if_mentions_return(if_stmt: &IfStmt) -> bool {
    block_mentions_return(&if_stmt.then)
        || match &if_stmt.otherwise {
            None => false,
            Some(Else::Block(block)) => block_mentions_return(block),
            Some(Else::If(nested)) => if_mentions_return(nested),
        }
}

/// Placeholder flag for loop tests built before the flag is allocated.
fn keep_marker() -> pc::FieldReference {
    pc::FieldReference {
        state: "sand:keep".parse().expect("valid placeholder"),
        field: "__keep".into(),
    }
}

/// Point every placeholder flag write at `keep`.
fn retarget_marker(mut ops: Vec<pc::Operation>, keep: &pc::FieldReference) -> Vec<pc::Operation> {
    for op in &mut ops {
        match &mut op.action {
            pc::Action::ScoreSet { score, .. } if score.field == "__keep" => *score = keep.clone(),
            pc::Action::Branch {
                then, otherwise, ..
            } => {
                *then = retarget_marker(std::mem::take(then), keep);
                *otherwise = retarget_marker(std::mem::take(otherwise), keep);
            }
            _ => {}
        }
    }
    ops
}

/// Field-name prefix of an unresolved frame placeholder; `__` cannot start a
/// slot name, so it never collides with a real slot.
pub(super) const FRAME_MARKER: &str = "__frame";

/// Whether evaluating the expression calls a function.
pub(super) fn contains_call(expr: &Expr) -> bool {
    match expr {
        Expr::Match { value, arms, .. } => {
            contains_call(value)
                || arms.iter().any(|arm| {
                    contains_call(&arm.value) || arm.guard.as_ref().is_some_and(cond_contains_call)
                })
        }
        // A value block can contain arbitrary statements before its tail.
        Expr::Block { .. } => true,
        Expr::Call { .. } => true,
        Expr::Neg(inner, _) | Expr::Cast(inner, _, _) => contains_call(inner),
        Expr::Binary(lhs, _, rhs, _) => contains_call(lhs) || contains_call(rhs),
        Expr::Cond(cond, _) => cond_contains_call(cond),
        Expr::Struct { fields, .. } => fields.iter().any(|(_, value)| contains_call(value)),
        Expr::Method { .. } => true,
        Expr::Project(value, ..) => contains_call(value),
        Expr::Array(values, _) => values.iter().any(contains_call),
        Expr::ArrayRepeat { value, .. } => contains_call(value),
        Expr::Index(value, index, _) => contains_call(value) || contains_call(index),
        _ => false,
    }
}

fn cond_contains_call(cond: &Cond) -> bool {
    match cond {
        Cond::Or(a, b) | Cond::And(a, b) => cond_contains_call(a) || cond_contains_call(b),
        Cond::Not(inner, _) => cond_contains_call(inner),
        Cond::Compare { lhs, rhs, .. } => contains_call(lhs) || contains_call(rhs),
        Cond::Value(value) | Cond::InRange { value, .. } => contains_call(value),
        // Builtin predicates call nothing, but a `bool` function does.
        Cond::Call { .. } => true,
    }
}

impl Cx {
    fn locals_state(&self) -> String {
        format!("{}:sand.locals", self.namespace())
    }

    /// The slot reference for a compiler-owned locals field.
    pub(super) fn slot(&mut self, field: String) -> pc::FieldReference {
        if !self.local_fields.contains(&field) {
            self.local_fields.push(field.clone());
        }
        pc::FieldReference {
            state: self.locals_state().parse().expect("namespace validated"),
            field,
        }
    }

    fn local_slot_name(&self, def: DefId) -> String {
        format!("v{}_{}", def.0, self.defs[def.0].name)
    }

    pub(super) fn local_ref(&mut self, def: DefId) -> pc::FieldReference {
        let name = self.local_slot_name(def);
        self.slot(name)
    }

    pub(super) fn lookup_local(body: &BodyCx, name: &str) -> Option<DefId> {
        body.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|(n, _)| n == name).map(|(_, d)| *d))
    }

    pub(super) fn local_of(body: &BodyCx, expr: &Expr) -> Option<DefId> {
        match expr {
            Expr::Path(path) if path.segments.len() == 1 && path.arguments().is_empty() => {
                Self::lookup_local(body, &path.segments[0].name)
            }
            _ => None,
        }
    }

    pub(super) fn local_ty(&self, def: DefId) -> Ty {
        match self.defs[def.0].info {
            DefInfo::Local { ty, .. } => ty,
            _ => Ty::Unknown,
        }
    }

    pub(super) fn is_const_in(body: &BodyCx, expr: &Expr) -> bool {
        match expr {
            Expr::Int(..) => true,
            Expr::Path(_) => Self::local_of(body, expr).is_none(),
            Expr::Neg(inner, _) | Expr::Cast(inner, _, _) => Self::is_const_in(body, inner),
            Expr::Binary(a, _, b, _) => Self::is_const_in(body, a) && Self::is_const_in(body, b),
            Expr::Place(_)
            | Expr::Call { .. }
            | Expr::Cond(..)
            | Expr::Str { .. }
            | Expr::Struct { .. }
            | Expr::Method { .. }
            | Expr::Project(..)
            | Expr::Array(..)
            | Expr::ArrayRepeat { .. }
            | Expr::Index(..)
            | Expr::Match { .. }
            | Expr::Block { .. }
            | Expr::Closure(_) => false,
            Expr::Float(..) | Expr::Bool(..) => true,
        }
    }

    /// The type an expression produces, without reporting errors.
    pub(super) fn type_of(&mut self, body: &BodyCx, expr: &Expr) -> Ty {
        match expr {
            Expr::Closure(closure) => self.closure_type(body.module, closure),
            Expr::Block {
                body: block, value, ..
            } => self.block_tail_type(body, block, value),
            Expr::Match { value, arms, .. } => {
                let Some(arm) = arms.first() else {
                    return Ty::Unknown;
                };
                let MatchPattern::Binding(pattern) = &arm.pattern else {
                    return self.type_of(body, &arm.value);
                };
                let ty = self.type_of(body, value);
                let bindings = self.pattern_locals(body, pattern, ty, arm.span);
                let mut scoped = body.clone();
                scoped.scopes.push(
                    bindings
                        .iter()
                        .map(|binding| (binding.name.name.clone(), binding.def))
                        .collect(),
                );
                self.type_of(&scoped, &arm.value)
            }
            Expr::Path(path) => {
                if let Some(local) = Self::local_of(body, expr) {
                    return self.local_ty(local);
                }
                match self.resolve_path(body.module, path) {
                    Ok((def, _)) => match self.defs[def.0].kind {
                        DefKind::Variant => {
                            self.defs[def.0].parent.map(Ty::Enum).unwrap_or(Ty::Number)
                        }
                        DefKind::Function => self.named_function_type(body.module, def),
                        DefKind::Const => {
                            self.const_src.get(&def).map(|s| s.ty).unwrap_or(Ty::Number)
                        }
                        _ => Ty::Number,
                    },
                    Err(_) => Ty::Unknown,
                }
            }
            Expr::Place(place) if self.is_handle(body, place) => {
                match crate::properties::find(&place.field.name).map(|p| p.ty) {
                    Some(PropertyType::Number) => Ty::Number,
                    Some(PropertyType::Float) => Ty::Float,
                    Some(PropertyType::Bool) => Ty::Bool,
                    None => Ty::Unknown,
                }
            }
            Expr::ArrayRepeat { value, length, .. } => {
                let element = self.type_of(body, value);
                let Some(length) =
                    self.eval_i32(body.module, body.file, length, Some(Ty::Number), None)
                else {
                    return Ty::Unknown;
                };
                if !(0..=1024).contains(&length) {
                    self.error(
                        "type",
                        body.file,
                        expr.span(),
                        "array length must be between 0 and 1024",
                    );
                    return Ty::Unknown;
                }
                self.array_type(body.module, element, length as usize)
            }
            Expr::Array(values, _) => {
                let Some(first) = values.first() else {
                    return Ty::Unknown;
                };
                let element = self.type_of(body, first);
                self.array_type(body.module, element, values.len())
            }
            Expr::Index(value, _, _) => {
                let ty = self.type_of(body, value);
                if ty == Ty::String {
                    return Ty::String;
                }
                if let Some(element) = self.list_element(ty) {
                    return element;
                }
                self.array_shape(ty)
                    .map_or(Ty::Unknown, |(element, _)| element)
            }
            Expr::Place(place) => self.place_ty(body, place),
            Expr::Struct { path, fields, .. } => self
                .record_literal_type(
                    Some(body),
                    body.module,
                    body.file,
                    path,
                    fields,
                    None,
                    false,
                )
                .unwrap_or(Ty::Unknown),
            Expr::Project(value, field, _) => {
                let ty = self.type_of(body, value);
                self.record_member(ty, &field.name)
                    .map_or(Ty::Unknown, |field| self.field_ty(field))
            }
            Expr::Method {
                receiver,
                name,
                type_arguments,
                args,
                ..
            } => {
                let receiver_ty = self.type_of(body, receiver);
                if name.name == "to_string"
                    && matches!(receiver_ty, Ty::Number | Ty::Float | Ty::Bool | Ty::Enum(_))
                {
                    return Ty::String;
                }
                if let Some(element) = self.list_element(receiver_ty) {
                    return match name.name.as_str() {
                        "len" | "extend" => Ty::Number,
                        "is_empty" | "contains" | "push" | "insert" => Ty::Bool,
                        "get_or" | "pop_or" | "remove_or" => element,
                        _ => Ty::Unknown,
                    };
                }
                if receiver_ty == Ty::String {
                    return match name.name.as_str() {
                        "len" => Ty::Number,
                        "slice" => Ty::String,
                        "is_empty" | "contains" | "starts_with" | "ends_with" => Ty::Bool,
                        _ => Ty::Unknown,
                    };
                }
                if let Some((element, _)) = self.array_shape(receiver_ty) {
                    if name.name == "to_vec" {
                        return self.list_type(body.module, element);
                    }
                    return match name.name.as_str() {
                        "len" => Ty::Number,
                        "is_empty" | "contains" => Ty::Bool,
                        "get_or" => element,
                        _ => Ty::Unknown,
                    };
                }
                let target = self.method_target(body, receiver).or_else(|| {
                    match self.type_of(body, receiver) {
                        Ty::Struct(def) => Some(def),
                        _ => None,
                    }
                });
                let Some(def) = target
                    .and_then(|target| self.members.get(&target)?.get(&name.name))
                    .copied()
                else {
                    return Ty::Unknown;
                };
                let mut arguments = args.clone();
                if matches!(receiver_ty, Ty::Struct(_)) {
                    arguments.insert(
                        0,
                        Arg {
                            name: None,
                            value: ArgValue::Expr((**receiver).clone()),
                            span: receiver.span(),
                        },
                    );
                }
                let path = Path {
                    segments: vec![name.clone()],
                    applications: if type_arguments.is_empty() {
                        Vec::new()
                    } else {
                        vec![PathApplication {
                            segment: 0,
                            arguments: type_arguments.clone(),
                        }]
                    },
                    span: name.span,
                };
                self.specialize_call(body, def, &path, &arguments, None, false)
                    .and_then(|def| self.fn_slots.get(&def)?.ret.as_ref())
                    .map_or(Ty::Unknown, |(_, ty)| *ty)
            }
            Expr::Float(..) => Ty::Float,
            Expr::Bool(..) | Expr::Cond(..) => Ty::Bool,
            Expr::Str { .. } => Ty::String,
            Expr::Neg(inner, _) => self.type_of(body, inner),
            Expr::Cast(_, ty, _) => self.cast_target_quiet(body.module, ty),
            Expr::Binary(lhs, _, rhs, _) => {
                let (left, right) = (self.type_of(body, lhs), self.type_of(body, rhs));
                if left == Ty::Float || right == Ty::Float {
                    Ty::Float
                } else {
                    left
                }
            }
            Expr::Call { path, .. }
                if path.segments.len() == 1
                    && let Some(local) = Self::lookup_local(body, &path.last().name) =>
            {
                self.callable_signature(self.local_ty(local))
                    .and_then(|signature| signature.returns)
                    .unwrap_or(Ty::Unknown)
            }
            Expr::Call { path, .. }
                if path.segments.len() == 1
                    && Self::lookup_local(body, &path.last().name).is_none()
                    && !self.scopes[body.module.0].contains_key(&path.last().name)
                    && builtins::find(&path.last().name, BuiltinKind::Condition).is_some() =>
            {
                Ty::Bool
            }
            Expr::Call { path, .. }
                if path.segments.len() == 1
                    && path.last().name == "entity_ref"
                    && !self.scopes[body.module.0].contains_key("entity_ref") =>
            {
                Ty::EntityRef
            }
            Expr::Call { path, .. }
                if path.segments.len() == 1
                    && path.last().name == "block_data_number"
                    && !self.scopes[body.module.0].contains_key("block_data_number") =>
            {
                Ty::Number
            }
            Expr::Call { path, args, .. } => self
                .resolve_call(body, path, args, None, false)
                .and_then(|def| self.fn_slots.get(&def))
                .and_then(|slots| slots.ret.as_ref().map(|(_, ty)| *ty))
                .unwrap_or(Ty::Unknown),
            _ => Ty::Number,
        }
    }

    /// A constant or single read, or `None` when the expression needs work.
    pub(super) fn simple(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        ty: Ty,
    ) -> Result<Option<Value>, ()> {
        if matches!(
            ty,
            Ty::Struct(_) | Ty::String | Ty::List(_) | Ty::Callable(_)
        ) {
            return Ok(None);
        }
        let (module, file) = (body.module, body.file);
        if Self::is_const_in(body, expr) {
            return self
                .eval_i32(module, file, expr, Some(ty), None)
                .map(|v| Some(Value::Const(v)))
                .ok_or(());
        }
        if let Some(local) = Self::local_of(body, expr) {
            body.used.insert(local);
            if let Expr::Path(path) = expr {
                // The identifier's own span: path spans can include trailing whitespace.
                self.references.push(Reference {
                    file,
                    span: path.segments[0].span,
                    def: local,
                });
            }
            let local_ty = self.local_ty(local);
            if !self.types_agree(file, expr.span(), ty, local_ty) {
                return Err(());
            }
            return Ok(Some(Value::Read(self.local_ref(local))));
        }
        if let Expr::Place(place) = expr {
            if self.is_handle(body, place) || self.entity_place(body, place) {
                return Ok(None);
            }
            let value = self.place_value(body, place, false).ok_or(())?;
            if !self.types_agree(file, place.span, ty, value.ty) {
                return Err(());
            }
            return Ok(Some(Value::Read(value.reference)));
        }
        Ok(None)
    }

    pub(super) fn types_agree(
        &mut self,
        file: FileId,
        span: Span,
        expected: Ty,
        found: Ty,
    ) -> bool {
        if expected == Ty::Unknown || found == Ty::Unknown || expected == found {
            return true;
        }
        let message = format!(
            "mismatched types: expected `{}`, found `{}`",
            self.ty_name(expected),
            self.ty_name(found)
        );
        let mut diagnostic = Diagnostic::error("type", file, span, message);
        match (expected, found) {
            (Ty::Float, Ty::Number) => {
                diagnostic = diagnostic.with_help("convert it with `as float`")
            }
            (Ty::Number, Ty::Float) => {
                diagnostic = diagnostic.with_help("convert it with `as number`")
            }
            (Ty::Number, Ty::Enum(_)) => {
                diagnostic = diagnostic.with_help("convert the enum with `as number`")
            }
            _ => {}
        }
        self.diagnostics.push(diagnostic);
        false
    }

    pub(super) fn temp(&mut self, body: &mut BodyCx) -> pc::FieldReference {
        let name = format!("{}_t{}", body.slot, body.temp_depth);
        body.temp_depth += 1;
        self.slot(name)
    }

    fn arithmetic(
        &mut self,
        file: FileId,
        dest: &pc::FieldReference,
        op: BinOp,
        value: Value,
        span: Span,
    ) -> Option<pc::Action> {
        Some(match (op, value) {
            (BinOp::Add, Value::Const(v)) => pc::Action::ScoreAdd {
                score: dest.clone(),
                value: v as i32,
            },
            (BinOp::Sub, Value::Const(v)) => {
                let Some(value) = (v as i32).checked_neg() else {
                    self.error("range", file, span, "subtracting this overflows i32");
                    return None;
                };
                pc::Action::ScoreAdd {
                    score: dest.clone(),
                    value,
                }
            }
            (BinOp::Div | BinOp::Rem, Value::Const(0)) => {
                self.error("const", file, span, "division by zero");
                return None;
            }
            (op, value) => pc::Action::ScoreOperation {
                score: dest.clone(),
                operator: match op {
                    BinOp::Add => pc::ScoreOperator::Add,
                    BinOp::Sub => pc::ScoreOperator::Sub,
                    BinOp::Mul => pc::ScoreOperator::Mul,
                    BinOp::Div => pc::ScoreOperator::Div,
                    BinOp::Rem => pc::ScoreOperator::Mod,
                },
                operand: match value {
                    Value::Const(v) => pc::ScoreOperand::Value(v as i32),
                    Value::Read(reference) => pc::ScoreOperand::Field(reference),
                },
            },
        })
    }

    pub(super) fn set_from(dest: &pc::FieldReference, value: Value) -> Option<pc::Action> {
        match value {
            Value::Const(v) => Some(pc::Action::ScoreSet {
                score: dest.clone(),
                value: v as i32,
            }),
            Value::Read(source) if source.state == dest.state && source.field == dest.field => None,
            Value::Read(source) => Some(pc::Action::ScoreOperation {
                score: dest.clone(),
                operator: pc::ScoreOperator::Set,
                operand: pc::ScoreOperand::Field(source),
            }),
        }
    }

    /// Compute `expr` into `dest`, using temporaries for nested parts.
    pub(super) fn emit(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        if !self.validate_method_type_arguments(body, expr) {
            return None;
        }
        let file = body.file;
        let span = expr.span();
        self.value_slots(dest, ty);
        if let Expr::Block {
            body: block, value, ..
        } = expr
        {
            let depth = body.temp_depth;
            body.value_blocks.push((body.loops.len(), body.jumps.len()));
            ops.extend(self.lower_block_with_tail(body, block, Some((dest, ty, value))));
            body.value_blocks.pop();
            body.temp_depth = depth;
            return Some(());
        }
        if let Expr::Match { value, arms, span } = expr {
            return self.emit_match(body, dest, ty, value, arms, *span, ops);
        }
        if let Ty::List(_) = ty {
            return self.emit_list(body, dest, ty, expr, ops);
        }
        if ty == Ty::String {
            return self.emit_text(body, dest, expr, ops);
        }
        if matches!(ty, Ty::Struct(_)) {
            return self.emit_record(body, dest, ty, expr, ops);
        }
        if matches!(ty, Ty::Callable(_)) {
            return self.emit_callable(body, dest, ty, expr, ops);
        }
        if let Some(value) = self.simple(body, expr, ty).ok()? {
            if let Some(action) = Self::set_from(dest, value) {
                ops.push(op_at(action, file, span));
            }
            return Some(());
        }
        match expr {
            Expr::Index(..) => self.emit_index(body, dest, ty, expr, ops),
            Expr::Method {
                receiver,
                name,
                type_arguments,
                args,
                span,
            } => {
                let receiver_ty = self.type_of(body, receiver);
                if self.list_element(receiver_ty).is_some() {
                    return self.emit_list_method(body, dest, ty, expr, ops);
                }
                if receiver_ty == Ty::String {
                    return self.emit_text_method(body, dest, ty, expr, ops);
                }
                if self.array_shape(receiver_ty).is_some() {
                    return self.emit_array_method(body, dest, ty, expr, ops);
                }
                self.emit_method(
                    body,
                    dest,
                    ty,
                    receiver,
                    name,
                    type_arguments,
                    args,
                    *span,
                    ops,
                )
            }
            Expr::Project(value, field, _) => {
                self.emit_projection(body, dest, ty, value, field, ops)
            }
            Expr::Call { path, args, span } => {
                self.emit_call(body, dest, ty, path, args, *span, ops)
            }
            Expr::Cast(inner, cast_ty, cast_span) => {
                let to = self.cast_target(body.module, file, cast_ty)?;
                if !self.types_agree(file, *cast_span, ty, to) {
                    return None;
                }
                let from = self.type_of(body, inner);
                let step = match self.cast_step(from, to) {
                    Ok(step) => step,
                    Err((message, help)) => {
                        self.diagnostics.push(
                            Diagnostic::error("type", file, *cast_span, message).with_help(help),
                        );
                        return None;
                    }
                };
                self.emit(body, dest, from, inner, ops)?;
                match step {
                    super::CastStep::Same => {}
                    super::CastStep::ToFloat | super::CastStep::ToNumber => {
                        let op = if step == super::CastStep::ToFloat {
                            BinOp::Mul
                        } else {
                            BinOp::Div
                        };
                        let action =
                            self.arithmetic(file, dest, op, Value::Const(FLOAT_SCALE), *cast_span)?;
                        ops.push(op_at(action, file, *cast_span));
                    }
                    super::CastStep::ToBool => ops.push(op_at(
                        pc::Action::Branch {
                            condition: pc::Condition::Range(pc::ScoreComparison {
                                score: dest.clone(),
                                min: Some(0),
                                max: Some(0),
                            }),
                            then: Vec::new(),
                            otherwise: vec![op_at(
                                pc::Action::ScoreSet {
                                    score: dest.clone(),
                                    value: 1,
                                },
                                file,
                                *cast_span,
                            )],
                        },
                        file,
                        *cast_span,
                    )),
                }
                Some(())
            }
            Expr::Cond(cond, cond_span) => {
                if !self.types_agree(file, *cond_span, ty, Ty::Bool) {
                    return None;
                }
                // A temporary, so `flag = !flag` reads the old value.
                let depth = body.temp_depth;
                let temp = self.temp(body);
                let set = |value| pc::Action::ScoreSet {
                    score: temp.clone(),
                    value,
                };
                ops.push(op_at(set(0), file, *cond_span));
                let then = vec![op_at(set(1), file, *cond_span)];
                let branch = self.lower_cond(body, cond, then, Vec::new())?;
                ops.extend(branch);
                body.temp_depth = depth;
                if let Some(action) = Self::set_from(dest, Value::Read(temp.clone())) {
                    ops.push(op_at(action, file, *cond_span));
                }
                Some(())
            }
            Expr::Neg(inner, neg_span) => {
                self.expect_numeric(file, *neg_span, ty)?;
                let inner_ty = if ty == Ty::Unknown {
                    self.type_of(body, inner)
                } else {
                    ty
                };
                self.emit(body, dest, inner_ty, inner, ops)?;
                let action =
                    self.arithmetic(file, dest, BinOp::Mul, Value::Const(-1), *neg_span)?;
                ops.push(op_at(action, file, *neg_span));
                Some(())
            }
            Expr::Binary(lhs, op, rhs, bin_span) => {
                self.expect_numeric(file, *bin_span, ty)?;
                let ty = if ty == Ty::Unknown {
                    self.type_of(body, expr)
                } else {
                    ty
                };
                self.emit(body, dest, ty, lhs, ops)?;
                let value = match self.simple(body, rhs, ty).ok()? {
                    Some(value) => value,
                    None => {
                        let depth = body.temp_depth;
                        let temp = self.temp(body);
                        self.emit(body, &temp, ty, rhs, ops)?;
                        body.temp_depth = depth;
                        Value::Read(temp)
                    }
                };
                self.apply(file, dest, ty, *op, value, rhs.span(), ops)
            }
            Expr::Place(place) if !self.is_handle(body, place) => {
                let value = self.place_value(body, place, false)?;
                if !self.types_agree(file, place.span, ty, value.ty) {
                    return None;
                }
                self.copy_value(dest, &value.reference, ty, file, place.span, ops);
                Some(())
            }
            Expr::Place(place) => {
                let property = self.entity_property(body, place)?;
                let found = match property.ty {
                    PropertyType::Number => Ty::Number,
                    PropertyType::Float => Ty::Float,
                    PropertyType::Bool => Ty::Bool,
                };
                if !self.types_agree(file, place.span, ty, found) {
                    return None;
                }
                let scale = if found == Ty::Float {
                    FLOAT_SCALE as f64
                } else {
                    1.0
                };
                let action = match property.kind {
                    PropertyKind::Attribute(attribute) => pc::Action::AttributeGet {
                        score: dest.clone(),
                        attribute,
                        scale,
                    },
                    PropertyKind::Data(value) => pc::Action::DataGet {
                        score: dest.clone(),
                        value,
                        scale,
                    },
                };
                ops.push(op_at(action, file, place.span));
                Some(())
            }
            Expr::Str { span: s, .. } => {
                self.error("type", file, *s, "expected a value, found text");
                None
            }
            _ => {
                self.error("type", file, span, "expected a value");
                None
            }
        }
    }

    /// One arithmetic step. Decimal multiply and divide rescale, because a
    /// `float` score already holds `value * 1000`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply(
        &mut self,
        file: FileId,
        dest: &pc::FieldReference,
        ty: Ty,
        op: BinOp,
        value: Value,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let mut step = |cx: &mut Self, op: BinOp, value: Value| -> Option<()> {
            let action = cx.arithmetic(file, dest, op, value, span)?;
            ops.push(op_at(action, file, span));
            Some(())
        };
        if ty != Ty::Float || !matches!(op, BinOp::Mul | BinOp::Div) {
            return step(self, op, value);
        }
        let (mul, div) = match (op, value) {
            (_, Value::Const(0)) if op == BinOp::Div => {
                self.error("const", file, span, "division by zero");
                return None;
            }
            // Reduce the fraction so constant factors overflow later.
            (BinOp::Mul, Value::Const(c)) => {
                let g = gcd(c.unsigned_abs(), FLOAT_SCALE as u64) as i64;
                (Value::Const(c / g), Value::Const(FLOAT_SCALE / g))
            }
            (_, Value::Const(c)) => {
                let g = gcd(c.unsigned_abs(), FLOAT_SCALE as u64) as i64;
                (Value::Const(FLOAT_SCALE / g), Value::Const(c / g))
            }
            (BinOp::Mul, read) => (read, Value::Const(FLOAT_SCALE)),
            (_, read) => (Value::Const(FLOAT_SCALE), read),
        };
        if !matches!(mul, Value::Const(1)) {
            step(self, BinOp::Mul, mul)?;
        }
        if !matches!(div, Value::Const(1)) {
            step(self, BinOp::Div, div)?;
        }
        Some(())
    }

    fn expect_numeric(&mut self, file: FileId, span: Span, ty: Ty) -> Option<()> {
        if matches!(
            ty,
            Ty::Enum(_) | Ty::Bool | Ty::Struct(_) | Ty::EntityRef | Ty::Callable(_)
        ) {
            self.diagnostics.push(
                Diagnostic::error(
                    "type",
                    file,
                    span,
                    "arithmetic needs `number` or `float` values",
                )
                .with_help("`bool` and enum values can only be assigned and compared"),
            );
            return None;
        }
        Some(())
    }

    /// A call used as a value: `random(..)`, `count_entities(..)`, or a function
    /// with a return type.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_call(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let (module, file) = (body.module, body.file);
        if path.segments.len() == 1
            && let Some(local) = Self::lookup_local(body, &path.last().name)
        {
            let (def, actual) = self.indirect_call(body, path, local, args)?;
            return self.emit_user_result(body, dest, ty, def, path, &actual, span, ops);
        }
        let name = &path.last().name;
        if !path.arguments().is_empty()
            && path.segments.len() == 1
            && !self.scopes[module.0].contains_key(name)
        {
            self.error(
                "generic",
                file,
                path.span,
                "builtins do not take type arguments",
            );
            return None;
        }

        if path.segments.len() == 1
            && !self.scopes[module.0].contains_key(name)
            && Self::lookup_local(body, name).is_none()
            && builtins::find(name, BuiltinKind::Condition).is_some()
        {
            let value = Expr::Cond(
                Box::new(Cond::Call {
                    path: path.clone(),
                    args: args.to_vec(),
                    span,
                }),
                span,
            );
            return self.emit(body, dest, ty, &value, ops);
        }
        let builtin = (path.segments.len() == 1
            && !self.scopes[module.0].contains_key(name)
            && Self::lookup_local(body, name).is_none())
        .then(|| builtins::find(name, BuiltinKind::Expression))
        .flatten();
        if let Some(builtin) = builtin {
            self.references_builtin(file, path.span, builtin);
            if builtin.name != "entity_ref" {
                self.expect_numeric(file, span, ty)?;
            }
            let result_ty = if builtin.name == "entity_ref" {
                Ty::EntityRef
            } else {
                Ty::Number
            };
            if !self.types_agree(file, span, ty, result_ty) {
                return None;
            }
            let bound = self.bind_args(body, builtin, args, span)?;
            let action = match builtin.name {
                "random" => {
                    let range = match &bound["range"].value {
                        ArgValue::Range(range) => range.clone(),
                        ArgValue::Expr(expr) => {
                            self.error(
                                "call",
                                file,
                                expr.span(),
                                "expected a range such as `1..=6`",
                            );
                            return None;
                        }
                    };
                    let (Some(start), Some(end)) = (&range.start, &range.end) else {
                        self.error(
                            "call",
                            file,
                            range.span,
                            "`random` needs both ends, like `1..=6`",
                        );
                        return None;
                    };
                    let min = self.eval_i32(module, file, start, Some(Ty::Number), None)?;
                    let max = self.eval_i32(module, file, end, Some(Ty::Number), None)?;
                    let max = if range.inclusive { max } else { max - 1 };
                    if min > max {
                        self.error("range", file, range.span, "random range is empty");
                        return None;
                    }
                    pc::Action::ScoreRandom {
                        score: dest.clone(),
                        min: min as i32,
                        max: max as i32,
                    }
                }
                "entity_ref" => {
                    let relation = if let Some(arg) = bound.get("relation") {
                        let keyword = self.arg_id(body, arg, "Relation")?;
                        match serde_json::from_value(serde_json::Value::String(keyword)) {
                            Ok(relation) => Some(relation),
                            Err(_) => {
                                self.error(
                                    "type",
                                    file,
                                    arg.span,
                                    "expected a `Relation::...` value",
                                );
                                return None;
                            }
                        }
                    } else {
                        None
                    };
                    body.note_executor(span, "retains an entity identity".into());
                    pc::Action::CaptureEntity {
                        score: dest.clone(),
                        relation,
                    }
                }
                "block_data_number" => pc::Action::ReadBlockData {
                    target: self.arg_block_data(body, bound["at"], bound["path"])?,
                    score: dest.clone(),
                },
                "count_entities" => pc::Action::CountEntities {
                    score: dest.clone(),
                    query: self.query_from(body, &bound)?,
                },
                "entities" => {
                    self.diagnostics.push(
                        Diagnostic::error("call", file, span, "`entities(..)` is the source of a `for` loop")
                            .with_help("write `for entity in entities(..) { ... }`, or count them with `count_entities(..)`"),
                    );
                    return None;
                }
                other => unreachable!("expression builtin `{other}`"),
            };
            ops.push(op_at(action, file, span));
            return Some(());
        }
        let def = self.resolve_call(body, path, args, Some(ty), true)?;
        self.emit_user_result(body, dest, ty, def, path, args, span, ops)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_user_result(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        def: DefId,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let file = body.file;
        let Some(slots) = self.fn_slots.get(&def).cloned() else {
            self.error(
                "call",
                file,
                path.span,
                "this function does not return a value",
            );
            return None;
        };
        let Some((ret_field, ret_ty)) = slots.ret.clone() else {
            let d = &self.defs[def.0];
            let (df, ds, dname) = (d.file, d.name_span, d.name.clone());
            self.diagnostics.push(
                Diagnostic::error(
                    "call",
                    file,
                    path.span,
                    format!("`{dname}` does not return a value"),
                )
                .with_note(df, ds, "declared without a return type")
                .with_help(format!("declare one: `fn {dname}(...) -> number`")),
            );
            return None;
        };
        if !self.types_agree(file, span, ty, ret_ty) {
            return None;
        }
        self.call_user(body, def, &slots, path, args, span, ops)?;
        let ret = self.slot(ret_field);
        self.copy_value(dest, &ret, ty, file, span, ops);
        Some(())
    }

    /// Pass arguments into the callee's parameter slots, then call it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn call_user(
        &mut self,
        body: &mut BodyCx,
        def: DefId,
        slots: &FnSlots,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let file = body.file;
        if let Some(caller) = body.owner.and_then(|owner| self.methods.get(&owner))
            && let Some(callee) = self.methods.get(&def)
            && caller.state_receiver
            && caller.target == callee.target
            && caller.receiver.as_ref().is_some_and(|r| !r.mutable)
            && callee.receiver.as_ref().is_some_and(|r| r.mutable)
        {
            self.error(
                "mutability",
                file,
                span,
                "this State receiver is read-only; calling a mutable method requires `&mut self`",
            );
            return None;
        }
        if args.len() != slots.params.len() || args.iter().any(|a| a.name.is_some()) {
            let name = self.defs[def.0].name.clone();
            let expected = slots.params.len();
            let message = if args.iter().any(|a| a.name.is_some()) {
                "named arguments are only for builtins; pass arguments in order".to_string()
            } else {
                format!(
                    "`{name}` takes {expected} argument{}, but {} {} given",
                    if expected == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" }
                )
            };
            let signature = match &self.defs[def.0].info {
                DefInfo::Function { signature, .. } => signature.clone(),
                _ => String::new(),
            };
            self.diagnostics
                .push(
                    Diagnostic::error("call", file, span, message).with_help(format!(
                        "signature: {} {name}{signature}",
                        if self.defs[def.0].kind == DefKind::Event {
                            "event"
                        } else {
                            "fn"
                        }
                    )),
                );
            return None;
        }
        let mut exprs = Vec::with_capacity(args.len());
        for arg in args {
            let ArgValue::Expr(expr) = &arg.value else {
                self.error("type", file, arg.span, "expected a value, found a range");
                return None;
            };
            exprs.push(expr);
        }
        let depth = body.temp_depth;
        let receiver_dest = if slots.receiver_out.is_some() {
            Some(self.value_address(body, exprs.first().copied()?, ops)?)
        } else {
            None
        };
        // A self-call reads the parameters it is about to overwrite, and a
        // nested call may overwrite parameters already passed, so those
        // arguments are computed into temporaries before any is passed.
        let staged = body.owner == Some(def)
            || slots
                .params
                .iter()
                .any(|(_, _, ty)| matches!(ty, Ty::Struct(_)))
            || exprs.iter().any(|expr| contains_call(expr));
        let mut values = Vec::with_capacity(exprs.len());
        if staged {
            for (index, ((_, _, param_ty), expr)) in slots.params.iter().zip(&exprs).enumerate() {
                if index == 0
                    && let Some(address) = &receiver_dest
                {
                    let temp = self.temp(body);
                    self.read_address(body, &temp, address, span, ops)?;
                    values.push(Value::Read(temp));
                    continue;
                }
                let value = match self.simple(body, expr, *param_ty).ok()? {
                    Some(Value::Const(value)) => Value::Const(value),
                    _ => {
                        let temp = self.temp(body);
                        self.emit(body, &temp, *param_ty, expr, ops)?;
                        Value::Read(temp)
                    }
                };
                values.push(value);
            }
        }
        let invocation_start = ops.len();
        let marker = self.frame_marker(body.owner, def);
        ops.push(op_at(
            pc::Action::PushFrame {
                fields: vec![marker.clone()],
            },
            file,
            span,
        ));
        for (index, ((field, _, param_ty), expr)) in slots.params.iter().zip(&exprs).enumerate() {
            let slot = self.slot(field.clone());
            if staged {
                match values[index].clone() {
                    Value::Read(source) => {
                        self.copy_value(&slot, &source, *param_ty, file, expr.span(), ops)
                    }
                    value => {
                        if let Some(action) = Self::set_from(&slot, value) {
                            ops.push(op_at(action, file, expr.span()));
                        }
                    }
                }
            } else {
                self.emit(body, &slot, *param_ty, expr, ops)?;
            }
        }
        body.temp_depth = depth;
        body.all_calls.push((def, path.span));
        if !body.in_players {
            body.calls.push((def, path.span));
        }
        if body.loops.last() == Some(&super::LoopKind::Entities) {
            body.entity_loop_calls.push((def, path.span));
        }
        let function = self.function_reference(def)?;
        ops.push(op_at(pc::Action::Call { function }, file, span));
        ops.push(op_at(
            pc::Action::PopFrame {
                fields: vec![marker],
            },
            file,
            span,
        ));
        if let (Some(dest), Some((field, _))) = (&receiver_dest, &slots.receiver_out) {
            let output = self.slot(field.clone());
            self.write_address(body, dest, &output, span, ops);
        }
        if let Some(address) = &receiver_dest
            && self.address_is_guarded(address)
        {
            let invocation = ops.split_off(invocation_start);
            if let Some((field, ty)) = &slots.ret {
                let reference = self.slot(field.clone());
                let defaults = self.record_constant(body.module, file, None, *ty)?;
                for ((score, _), value) in
                    self.value_slots(&reference, *ty).into_iter().zip(defaults)
                {
                    ops.push(op_at(Self::initialize_leaf(score, value), file, span));
                }
            }
            ops.extend(self.guard_address(body, address, span, invocation));
        }
        Some(())
    }

    /// A placeholder frame field for one call, resolved once the whole call
    /// graph is known: a call that can re-enter its caller saves the caller's
    /// variables around it, and every other call drops the placeholder.
    pub(super) fn frame_marker(
        &mut self,
        caller: Option<DefId>,
        callee: DefId,
    ) -> pc::FieldReference {
        let index = self.call_frames.len();
        self.call_frames.push((caller, callee));
        pc::FieldReference {
            state: self.locals_state().parse().expect("namespace validated"),
            field: format!("{FRAME_MARKER}{index}"),
        }
    }

    fn resolve_target(&mut self, body: &mut BodyCx, target: &AssignTarget) -> Option<Dest> {
        let (module, file) = (body.module, body.file);
        match target {
            AssignTarget::Value(_) => None,
            AssignTarget::Place(place) => {
                let value = self.place_value(body, place, true)?;
                let bounds = match self.defs[value.field.0].info {
                    DefInfo::Field { bounds, .. } => bounds,
                    _ => None,
                };
                Some(Dest {
                    reference: value.reference,
                    ty: value.ty,
                    bounds,
                    name: self.defs[value.field.0].name.clone(),
                })
            }
            AssignTarget::Local(ident) => {
                let Some(local) = Self::lookup_local(body, &ident.name) else {
                    let mut diagnostic = Diagnostic::error(
                        "resolve",
                        file,
                        ident.span,
                        format!("cannot find variable `{}` in this scope", ident.name),
                    );
                    if self.scopes[module.0].contains_key(&ident.name) {
                        diagnostic = diagnostic.with_help(
                            "only `let` variables and State fields (`State.field`) can be assigned",
                        );
                    } else {
                        diagnostic = diagnostic
                            .with_help(format!("declare it first: `let mut {} = 0;`", ident.name));
                    }
                    self.diagnostics.push(diagnostic);
                    return None;
                };
                self.references.push(Reference {
                    file,
                    span: ident.span,
                    def: local,
                });
                let DefInfo::Local {
                    mutable, ty, param, ..
                } = self.defs[local.0].info
                else {
                    return None;
                };
                if !mutable {
                    let d = &self.defs[local.0];
                    let (df, ds) = (d.file, d.name_span);
                    let help = if param {
                        format!("make the parameter mutable: `mut {0}: ...`", ident.name)
                    } else {
                        format!("make it mutable: `let mut {}`", ident.name)
                    };
                    self.diagnostics.push(
                        Diagnostic::error(
                            "mutability",
                            file,
                            ident.span,
                            format!("cannot assign twice to immutable variable `{}`", ident.name),
                        )
                        .with_note(df, ds, "first assigned here")
                        .with_help(help),
                    );
                    return None;
                }
                body.assigned.insert(local);
                Some(Dest {
                    reference: self.local_ref(local),
                    ty,
                    bounds: None,
                    name: ident.name.clone(),
                })
            }
        }
    }

    pub(super) fn lower_assign(
        &mut self,
        body: &mut BodyCx,
        target: &AssignTarget,
        op: AssignOp,
        value: &Expr,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        if let AssignTarget::Value(target) = target {
            self.lower_value_assignment(body, target, op, value, ops);
            return;
        }
        let Some(dest) = self.resolve_target(body, target) else {
            return;
        };
        let compound = match op {
            AssignOp::Set => None,
            AssignOp::Add => Some(BinOp::Add),
            AssignOp::Sub => Some(BinOp::Sub),
            AssignOp::Mul => Some(BinOp::Mul),
            AssignOp::Div => Some(BinOp::Div),
            AssignOp::Rem => Some(BinOp::Rem),
        };
        if dest.ty == Ty::String && compound.is_some() {
            if op != AssignOp::Add {
                self.error(
                    "type",
                    file,
                    span,
                    "strings support `=` and `+=` assignments",
                );
                return;
            }
            let depth = body.temp_depth;
            let rhs = self.temp(body);
            if self.emit(body, &rhs, Ty::String, value, ops).is_some() {
                ops.push(op_at(Self::append_text(&dest.reference, &rhs), file, span));
            }
            body.temp_depth = depth;
            return;
        }
        if compound.is_some() && matches!(dest.ty, Ty::List(_)) {
            self.error(
                "type",
                file,
                span,
                "vectors use `=` for assignment; use collection methods to change their contents",
            );
            return;
        }
        if let Some(bin) = compound {
            if matches!(dest.ty, Ty::Struct(_)) {
                self.error(
                    "type",
                    file,
                    span,
                    "compound arithmetic needs scalar fields, not a struct value",
                );
                return;
            }
            if matches!(
                dest.ty,
                Ty::Enum(_) | Ty::Bool | Ty::EntityRef | Ty::Callable(_)
            ) {
                self.error(
                    "type",
                    file,
                    span,
                    "`bool` and enum values can only be assigned with `=`",
                );
                return;
            }
            let depth = body.temp_depth;
            let value_ = match self.simple(body, value, dest.ty) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    let temp = self.temp(body);
                    if self.emit(body, &temp, dest.ty, value, ops).is_none() {
                        return;
                    }
                    Value::Read(temp)
                }
                Err(()) => return,
            };
            // Evaluate the RHS first, then snapshot the compound-assignment
            // value. Rescaling must not clamp intermediate score writes.
            let rescale = dest.bounds.is_some()
                && dest.ty == Ty::Float
                && matches!(bin, BinOp::Mul | BinOp::Div);
            let target = if rescale {
                let temp = self.temp(body);
                if let Some(action) = Self::set_from(&temp, Value::Read(dest.reference.clone())) {
                    ops.push(op_at(action, file, span));
                }
                temp
            } else {
                dest.reference.clone()
            };
            let applied = self.apply(file, &target, dest.ty, bin, value_, value.span(), ops);
            if applied.is_some()
                && rescale
                && let Some(action) = Self::set_from(&dest.reference, Value::Read(target))
            {
                ops.push(op_at(action, file, span));
            }
            body.temp_depth = depth;
            return;
        }
        // Constant writes outside a clamp range are worth flagging.
        if Self::is_const_in(body, value)
            && let Some((min, max)) = dest.bounds
            && let Some(v) = self.eval_i32(body.module, file, value, Some(dest.ty), None)
        {
            if v < min as i64 || v > max as i64 {
                self.diagnostics.push(Diagnostic::warning(
                    "clamp",
                    file,
                    value.span(),
                    format!(
                        "{v} is outside `{}`'s clamp range {min}..={max} and will be clamped",
                        dest.name
                    ),
                ));
            }
            ops.push(op_at(
                pc::Action::ScoreSet {
                    score: dest.reference,
                    value: v as i32,
                },
                file,
                span,
            ));
            return;
        }
        // Calls and entity-property reads already write their result once.
        if dest.bounds.is_none()
            && !matches!(dest.ty, Ty::Struct(_))
            && (matches!(value, Expr::Call { .. } | Expr::Method { .. })
                || matches!(value, Expr::Place(place) if self.is_handle(body, place)))
        {
            let _ = self.emit(body, &dest.reference, dest.ty, value, ops);
            return;
        }
        // A simple read/constant commits once. Every computed value uses
        // scratch storage: calls and conditions may read or mutate the target,
        // and casts must finish in their source units before bounds apply.
        match self.simple(body, value, dest.ty) {
            Ok(Some(value)) => {
                if let Some(action) = Self::set_from(&dest.reference, value) {
                    ops.push(op_at(action, file, span));
                }
            }
            Ok(None) => {
                let depth = body.temp_depth;
                let temp = self.temp(body);
                let computed = self.emit(body, &temp, dest.ty, value, ops);
                body.temp_depth = depth;
                if computed.is_some() {
                    self.copy_value(&dest.reference, &temp, dest.ty, file, span, ops);
                }
            }
            Err(()) => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_let(
        &mut self,
        body: &mut BodyCx,
        mutable: bool,
        name: &Ident,
        ty: Option<&Type>,
        value: Option<&Expr>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (module, file) = (body.module, body.file);
        let key = (body.file, name.span, body.slot.clone());
        let local = match self.inferred_block_locals.get(&key) {
            Some(local) => *local,
            None => {
                let ty = self.local_declaration_type(body, ty, value);
                self.new_local(body, name, ty, mutable, span)
            }
        };
        let ty = self.local_ty(local);
        let reference = self.local_ref(local);
        match value {
            Some(value) => {
                let _ = self.emit(body, &reference, ty, value, ops);
            }
            None => {
                let defaults = self
                    .record_constant(module, file, None, ty)
                    .unwrap_or_default();
                for ((score, _), default) in
                    self.value_slots(&reference, ty).into_iter().zip(defaults)
                {
                    ops.push(op_at(Self::initialize_leaf(score, default), file, span));
                }
            }
        }
        // Visible from after the statement to the end of the block.
        let end = body.block_ends.last().copied().unwrap_or(span.end);
        self.bring_into_scope(body, name, local, Span::new(span.end, end));
    }

    pub(super) fn lower_destructure(
        &mut self,
        body: &mut BodyCx,
        pattern: &Pattern,
        annotation: Option<&Type>,
        value: &Expr,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let depth = body.temp_depth;
        let ty = match annotation {
            Some(ty) => self.resolve_type(body.module, ty),
            None => self.type_of(body, value),
        };
        let snapshot = self.temp(body);
        if self.emit(body, &snapshot, ty, value, ops).is_some() {
            let bindings = self.pattern_locals(body, pattern, ty, span);
            let end = body.block_ends.last().copied().unwrap_or(span.end);
            self.apply_pattern(body, &bindings, &snapshot, Span::new(span.end, end), ops);
        }
        body.temp_depth = depth;
    }

    #[allow(clippy::too_many_arguments)]
    fn plan_pattern(
        &mut self,
        body: &BodyCx,
        pattern: &Pattern,
        ty: Ty,
        span: Span,
        projection: &mut Vec<String>,
        bindings: &mut Vec<PatternLocal>,
    ) {
        match pattern {
            Pattern::Ignore(_) => {}
            Pattern::Bind { name, mutable } => {
                if bindings
                    .iter()
                    .any(|binding| binding.name.name == name.name)
                {
                    self.error(
                        "pattern",
                        body.file,
                        name.span,
                        format!("duplicate binding `{}` in pattern", name.name),
                    );
                    return;
                }
                bindings.push(PatternLocal {
                    name: name.clone(),
                    def: self.new_local(body, name, ty, *mutable, span),
                    ty,
                    projection: projection.clone(),
                });
            }
            Pattern::Array(patterns, location) => {
                let Some((element, length)) = self.array_shape(ty) else {
                    self.error(
                        "pattern",
                        body.file,
                        *location,
                        "an array pattern requires a fixed array value",
                    );
                    return;
                };
                if patterns.len() != length {
                    self.error(
                        "pattern",
                        body.file,
                        *location,
                        format!("expected {length} array patterns, found {}", patterns.len()),
                    );
                    return;
                }
                for (index, pattern) in patterns.iter().enumerate() {
                    projection.push(format!("i{index}"));
                    self.plan_pattern(body, pattern, element, span, projection, bindings);
                    projection.pop();
                }
            }
            Pattern::Record {
                path,
                fields,
                rest,
                span: location,
            } => {
                let Some(def) = self.expect_path(body.module, path, &[DefKind::Struct], "a struct")
                else {
                    return;
                };
                let expected = if self.generics.records.contains_key(&def)
                    && path.arguments().is_empty()
                    && matches!(ty, Ty::Struct(instance) if self.generics.record_arguments.get(&instance).is_some_and(|(template, _)| *template == def))
                {
                    ty
                } else if !path.arguments().is_empty() {
                    let types = path
                        .arguments()
                        .iter()
                        .map(|ty| self.resolve_type(body.module, ty))
                        .collect();
                    self.instantiate_record(def, types, body.file, *location)
                        .unwrap_or(Ty::Unknown)
                } else {
                    Ty::Struct(def)
                };
                if !self.types_agree(body.file, *location, expected, ty) {
                    return;
                }
                let Ty::Struct(def) = ty else {
                    return;
                };
                let mut seen = HashSet::new();
                for (name, pattern) in fields {
                    if !seen.insert(name.name.clone()) {
                        self.error(
                            "pattern",
                            body.file,
                            name.span,
                            "duplicate field in record pattern",
                        );
                        continue;
                    }
                    let Some(field) = self.record_member(ty, &name.name) else {
                        self.error(
                            "pattern",
                            body.file,
                            name.span,
                            format!("unknown field `{}`", name.name),
                        );
                        continue;
                    };
                    if !self.visible(
                        body.module,
                        self.defs[field.0].module,
                        self.defs[field.0].is_pub,
                    ) {
                        self.error(
                            "privacy",
                            body.file,
                            name.span,
                            format!("field `{}` is private", name.name),
                        );
                        continue;
                    }
                    self.references.push(Reference {
                        file: body.file,
                        span: name.span,
                        def: field,
                    });
                    projection.push(name.name.clone());
                    self.plan_pattern(
                        body,
                        pattern,
                        self.field_ty(field),
                        span,
                        projection,
                        bindings,
                    );
                    projection.pop();
                }
                if !rest {
                    for field in self.struct_fields(def) {
                        let name = self.defs[field.0].name.clone();
                        if !seen.contains(&name) {
                            self.error("pattern", body.file, *location, format!("missing record field `{name}`; add `..` to ignore remaining fields"));
                        }
                    }
                }
            }
        }
    }

    fn apply_pattern(
        &mut self,
        body: &mut BodyCx,
        bindings: &[PatternLocal],
        snapshot: &pc::FieldReference,
        visibility: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        for binding in bindings {
            let source = binding
                .projection
                .iter()
                .fold(snapshot.clone(), |part, field| {
                    super::values::child(&part, field)
                });
            let dest = self.local_ref(binding.def);
            self.copy_value(
                &dest,
                &source,
                binding.ty,
                body.file,
                binding.name.span,
                ops,
            );
            self.bring_into_scope(body, &binding.name, binding.def, visibility);
        }
    }

    fn pattern_locals(
        &mut self,
        body: &BodyCx,
        pattern: &Pattern,
        ty: Ty,
        statement_span: Span,
    ) -> Vec<PatternLocal> {
        let key = (body.file, pattern.span(), body.slot.clone());
        if let Some(bindings) = self.pattern_bindings.get(&key) {
            return bindings.clone();
        }
        let mut bindings = Vec::new();
        // Keep definitions allocated during type inference so lowering and
        // editor references resolve each name to the same binding.
        self.plan_pattern(
            body,
            pattern,
            ty,
            statement_span,
            &mut Vec::new(),
            &mut bindings,
        );
        self.pattern_bindings.insert(key, bindings.clone());
        bindings
    }

    fn local_declaration_type(
        &mut self,
        body: &BodyCx,
        ty: Option<&Type>,
        value: Option<&Expr>,
    ) -> Ty {
        match (ty, value) {
            (Some(ty), _) => self.resolve_type(body.module, ty),
            (None, Some(value)) => self.type_of(body, value),
            (None, None) => Ty::Number,
        }
    }

    /// Infer the value with the declarations in its lexical scope. Other
    /// statements can change values and control flow but cannot declare locals
    /// visible to the tail.
    fn block_tail_type(&mut self, body: &BodyCx, block: &Block, tail: &Expr) -> Ty {
        let mut scoped = body.clone();
        scoped.scopes.push(Vec::new());
        for statement in &block.stmts {
            match statement {
                Stmt::Let {
                    mutable,
                    name,
                    ty,
                    value,
                    span,
                } => {
                    let key = (scoped.file, name.span, scoped.slot.clone());
                    let local = match self.inferred_block_locals.get(&key) {
                        Some(local) => *local,
                        None => {
                            let ty =
                                self.local_declaration_type(&scoped, ty.as_ref(), value.as_ref());
                            let local = self.new_local(&scoped, name, ty, *mutable, *span);
                            self.inferred_block_locals.insert(key, local);
                            local
                        }
                    };
                    scoped
                        .scopes
                        .last_mut()
                        .expect("value block scope")
                        .push((name.name.clone(), local));
                }
                Stmt::Destructure {
                    pattern,
                    ty,
                    value,
                    span,
                } => {
                    let ty = self.local_declaration_type(&scoped, ty.as_ref(), Some(value));
                    let bindings = self.pattern_locals(&scoped, pattern, ty, *span);
                    scoped.scopes.last_mut().expect("value block scope").extend(
                        bindings
                            .into_iter()
                            .map(|binding| (binding.name.name, binding.def)),
                    );
                }
                _ => {}
            }
        }
        self.type_of(&scoped, tail)
    }

    /// A new local owned by the function being lowered; not yet visible.
    fn new_local(
        &mut self,
        body: &BodyCx,
        name: &Ident,
        ty: Ty,
        mutable: bool,
        span: Span,
    ) -> DefId {
        self.push_def(Def {
            kind: DefKind::Local,
            name: name.name.clone(),
            module: body.module,
            file: body.file,
            name_span: name.span,
            span,
            docs: Vec::new(),
            is_pub: false,
            parent: None,
            info: DefInfo::Local {
                ty,
                mutable,
                param: false,
                owner: body.owner,
            },
        })
    }

    /// Make a local visible by name in the innermost scope, over `visible`.
    fn bring_into_scope(&mut self, body: &mut BodyCx, name: &Ident, local: DefId, visible: Span) {
        self.local_scopes.push((body.file, visible, local));
        body.scopes
            .last_mut()
            .expect("block scope")
            .push((name.name.clone(), local));
        body.locals.push(local);
    }

    pub(super) fn lower_return(
        &mut self,
        body: &mut BodyCx,
        value: Option<&Expr>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        if !body.value_blocks.is_empty() {
            self.error("control", body.file, span, "use a tail expression for the value of a block; `return` cannot leave a value block");
            return;
        }
        let file = body.file;
        if !body.loops.is_empty() {
            self.diagnostics.push(
                Diagnostic::error(
                    "control",
                    file,
                    span,
                    "`return` inside a `for` loop would only end this iteration",
                )
                .with_help("use `continue` to skip to the next one, or return after the loop"),
            );
            return;
        }
        match (&body.ret.clone(), value) {
            (Some((slot, ty)), Some(value)) => {
                // A recursive call would overwrite a partly computed return
                // slot, so such a value is computed in a saved temporary.
                if contains_call(value) && !matches!(value, Expr::Call { .. }) {
                    let depth = body.temp_depth;
                    let temp = self.temp(body);
                    if self.emit(body, &temp, *ty, value, ops).is_none() {
                        return;
                    }
                    body.temp_depth = depth;
                    self.copy_value(slot, &temp, *ty, file, value.span(), ops);
                } else if self.emit(body, slot, *ty, value, ops).is_none() {
                    return;
                }
            }
            (Some((_, ty)), None) => {
                let ty = self.ty_name(*ty);
                self.error(
                    "return",
                    file,
                    span,
                    format!("this function returns `{ty}`; write `return <value>;`"),
                );
                return;
            }
            (None, Some(value)) => {
                self.diagnostics.push(
                    Diagnostic::error(
                        "return",
                        file,
                        value.span(),
                        "this function does not return a value",
                    )
                    .with_help("declare a return type: `fn name() -> number`"),
                );
                return;
            }
            (None, None) => {}
        }
        self.finish_receiver(body, span, ops);
        ops.push(op_at(pc::Action::Return, file, span));
    }

    /// Match a snapshot of a value against typed constant alternatives.
    pub(super) fn lower_match(
        &mut self,
        body: &mut BodyCx,
        value: &Expr,
        arms: &[MatchArm],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let depth = body.temp_depth;
        let ty = self.type_of(body, value);
        let snapshot = self.temp(body);
        if self.emit(body, &snapshot, ty, value, ops).is_none() {
            body.temp_depth = depth;
            return;
        }
        // Reserve the dispatch flag before lowering arm scratch storage.
        let matched = self.temp(body);
        let patterns: Vec<_> = arms
            .iter()
            .map(|arm| (&arm.pattern, arm.guard.is_some(), arm.span))
            .collect();
        let values = self.match_patterns(body, ty, &patterns, span);
        let lowered = arms
            .iter()
            .zip(values)
            .filter_map(|(arm, values)| {
                let branch = self.lower_match_arm(
                    body,
                    &snapshot,
                    ty,
                    &matched,
                    &arm.pattern,
                    arm.guard.as_ref(),
                    arm.span,
                    |this, body| Some(this.lower_block(body, &arm.body)),
                )?;
                Some(LoweredMatchArm {
                    values,
                    catch_all: !matches!(&arm.pattern, MatchPattern::Constants(_)),
                    body: branch,
                    span: arm.span,
                })
            })
            .collect();
        self.match_dispatch(body, &snapshot, &matched, lowered, span, ops);
        body.temp_depth = depth;
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_match_arm(
        &mut self,
        body: &mut BodyCx,
        snapshot: &pc::FieldReference,
        ty: Ty,
        matched: &pc::FieldReference,
        pattern: &MatchPattern,
        guard: Option<&Cond>,
        span: Span,
        lower: impl FnOnce(&mut Self, &mut BodyCx) -> Option<Vec<pc::Operation>>,
    ) -> Option<Vec<pc::Operation>> {
        body.scopes.push(Vec::new());
        let mut prelude = Vec::new();
        if let MatchPattern::Binding(pattern) = pattern {
            let bindings = self.pattern_locals(body, pattern, ty, span);
            self.apply_pattern(
                body,
                &bindings,
                snapshot,
                Span::new(pattern.span().end, span.end),
                &mut prelude,
            );
        }

        let branch = lower(self, body).and_then(|mut branch| {
            branch.insert(
                0,
                op_at(
                    pc::Action::ScoreSet {
                        score: matched.clone(),
                        value: 1,
                    },
                    body.file,
                    span,
                ),
            );
            match guard {
                Some(guard) => self.lower_cond(body, guard, branch, Vec::new()),
                None => Some(branch),
            }
        });
        body.scopes.pop();
        prelude.extend(branch?);
        Some(prelude)
    }

    /// Validate shared statement/value-match patterns and exhaustiveness.
    fn match_patterns(
        &mut self,
        body: &BodyCx,
        ty: Ty,
        arms: &[(&MatchPattern, bool, Span)],
        span: Span,
    ) -> Vec<Vec<i32>> {
        let file = body.file;
        let mut covered = BTreeSet::new();
        let mut catch_all = None;
        let mut lowered = Vec::new();
        for (pattern, guarded, arm_span) in arms {
            if let Some(message) = catch_all {
                self.error("match", file, *arm_span, message);
            }
            let mut values = Vec::new();
            let mut alternatives = BTreeSet::new();
            let patterns: &[Expr] = match pattern {
                MatchPattern::Constants(patterns) => patterns,
                MatchPattern::Wildcard | MatchPattern::Binding(_) => {
                    if !guarded && catch_all.is_none() {
                        catch_all = Some(if matches!(pattern, MatchPattern::Wildcard) {
                            "unreachable match arm after `_`"
                        } else {
                            "unreachable match arm after an unguarded binding pattern"
                        });
                    }
                    &[]
                }
            };
            for pattern in patterns {
                if !Self::is_const_in(body, pattern) {
                    self.error(
                        "match",
                        file,
                        pattern.span(),
                        "match patterns must be constants or enum variants",
                    );
                    continue;
                }
                if let Some(value) = self.eval_i32(body.module, file, pattern, Some(ty), None) {
                    let value = value as i32;
                    if covered.contains(&value) || !alternatives.insert(value) {
                        self.error(
                            "match",
                            file,
                            pattern.span(),
                            "this value is already covered by a previous pattern",
                        );
                    }
                    if !guarded {
                        covered.insert(value);
                    }
                    values.push(value);
                }
            }
            lowered.push(values);
        }
        let exhaustive = catch_all.is_some()
            || match ty {
                Ty::Bool => covered.contains(&0) && covered.contains(&1),
                Ty::Enum(def) => match &self.defs[def.0].info {
                    DefInfo::Enum { variants } => {
                        variants
                            .iter()
                            .all(|variant| match self.defs[variant.0].info {
                                DefInfo::Variant { value: Some(value) } => {
                                    covered.contains(&(value as i32))
                                }
                                _ => false,
                            })
                    }
                    _ => false,
                },
                _ => false,
            };
        if !exhaustive && ty != Ty::Unknown {
            self.diagnostics.push(
                Diagnostic::error(
                    "match",
                    file,
                    span,
                    "non-exhaustive match: some values are not covered",
                )
                .with_help("cover every variant, both boolean values, or add `_ => { ... }`"),
            );
        }
        lowered
    }

    /// Flat dispatch keeps the operation depth independent of the number of arms.
    #[allow(clippy::too_many_arguments)]
    fn match_dispatch(
        &mut self,
        body: &mut BodyCx,
        snapshot: &pc::FieldReference,
        matched: &pc::FieldReference,
        arms: Vec<LoweredMatchArm>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        ops.push(op_at(
            pc::Action::ScoreSet {
                score: matched.clone(),
                value: 0,
            },
            body.file,
            span,
        ));
        for arm in arms {
            let arm_span = arm.span;
            let branch = arm.body;
            let selected = if arm.catch_all {
                branch
            } else {
                arm.values
                    .into_iter()
                    .map(|value| {
                        op_at(
                            pc::Action::Branch {
                                condition: pc::Condition::Range(pc::ScoreComparison {
                                    score: snapshot.clone(),
                                    min: Some(value),
                                    max: Some(value),
                                }),
                                then: branch.clone(),
                                otherwise: Vec::new(),
                            },
                            body.file,
                            arm_span,
                        )
                    })
                    .collect()
            };
            ops.push(op_at(
                pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: matched.clone(),
                        min: Some(0),
                        max: Some(0),
                    }),
                    then: selected,
                    otherwise: Vec::new(),
                },
                body.file,
                arm_span,
            ));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_match(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        value: &Expr,
        arms: &[MatchValueArm],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let depth = body.temp_depth;
        let scrutinee_ty = self.type_of(body, value);
        let snapshot = self.temp(body);
        self.emit(body, &snapshot, scrutinee_ty, value, ops)?;
        // Reserve the dispatch flag before lowering arm scratch storage.
        let matched = self.temp(body);
        let patterns: Vec<_> = arms
            .iter()
            .map(|arm| (&arm.pattern, arm.guard.is_some(), arm.span))
            .collect();
        let values = self.match_patterns(body, scrutinee_ty, &patterns, span);
        let mut lowered = Vec::new();
        for (arm, values) in arms.iter().zip(values) {
            let branch = self.lower_match_arm(
                body,
                &snapshot,
                scrutinee_ty,
                &matched,
                &arm.pattern,
                arm.guard.as_ref(),
                arm.span,
                |this, body| {
                    let mut branch = Vec::new();
                    this.emit(body, dest, ty, &arm.value, &mut branch)?;
                    Some(branch)
                },
            )?;
            lowered.push(LoweredMatchArm {
                values,
                catch_all: !matches!(&arm.pattern, MatchPattern::Constants(_)),
                body: branch,
                span: arm.span,
            });
        }
        self.match_dispatch(body, &snapshot, &matched, lowered, span, ops);
        body.temp_depth = depth;
        Some(())
    }

    /// Lower `while`, `loop`, and counting `for` loops.
    ///
    /// Each becomes one backend `While` whose helper re-checks the test after
    /// every pass. A test that is a single plain comparison is checked
    /// directly; any other test (`!=`, `!`, `&&`, `||`, calculations, calls)
    /// is computed into a saved flag before the first pass and again at the
    /// end of each. `break` and `continue` set a jump flag that skips the rest
    /// of the pass; `break` also clears the loop flag.
    pub(super) fn lower_repeat(
        &mut self,
        body: &mut BodyCx,
        kind: Repeat<'_>,
        block: &Block,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let depth = body.temp_depth;
        let count_guard = match &kind {
            Repeat::Count {
                reverse: false,
                step: None,
                range,
                ..
            } => !matches!(range.end, Some(Expr::Int(n, _)) if n < i64::from(i32::MAX)),
            Repeat::Count { .. } => true,
            _ => false,
        };
        let has_break = count_guard || block.stmts.iter().any(|stmt| jumps_here(stmt, true));
        let has_jump = has_break || block.stmts.iter().any(|stmt| jumps_here(stmt, false));
        if matches!(kind, Repeat::Forever) && !has_break && !block_mentions_return(block) {
            self.diagnostics.push(
                Diagnostic::warning("loop", file, span, "this `loop` never ends").with_help(
                    "leave it with `break;` or `return`; Minecraft stops runaway command chains",
                ),
            );
        }
        let jump = has_jump.then(|| self.temp(body));
        // Reserve the loop flag's temporary before the test uses any.
        let keep_index = body.temp_depth;
        body.temp_depth += 1;
        let mut setup = Vec::new();
        let mut step = Vec::new();
        let mut pre_body = Vec::new();
        body.scopes.push(Vec::new());
        // The plain condition, when the test is a single comparison.
        let mut direct = None;
        // Operations that compute the test into `keep`.
        let mut test = Vec::new();
        let keep_needed;
        match kind {
            Repeat::While(cond) => {
                let mut pre = Vec::new();
                let simple_shape = matches!(
                    cond,
                    Cond::Compare { .. }
                        | Cond::InRange { .. }
                        | Cond::Call { .. }
                        | Cond::Value(_)
                );
                let atom = if simple_shape {
                    let Some(atom) = self.atom(body, cond, &mut pre) else {
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    };
                    Some(atom)
                } else {
                    None
                };
                match atom {
                    Some((condition, false)) if pre.is_empty() => direct = Some(condition),
                    Some((condition, negated)) => {
                        test = pre;
                        test.push(Self::flag_branch(condition, negated, file, cond.span()));
                    }
                    None => {
                        let Some(lowered) = self.lower_cond(
                            body,
                            cond,
                            vec![Self::marker_set(1, file, span)],
                            Vec::new(),
                        ) else {
                            body.scopes.pop();
                            body.temp_depth = depth;
                            return;
                        };
                        test.push(Self::marker_set(0, file, span));
                        test.extend(lowered);
                    }
                }
                keep_needed = direct.is_none() || has_break;
            }
            Repeat::Forever => {
                test.push(Self::marker_set(1, file, span));
                keep_needed = true;
            }
            Repeat::Count {
                binding,
                range,
                reverse,
                step: increment,
                collection,
            } => {
                let (Some(start), Some(end)) = (&range.start, &range.end) else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "for",
                            file,
                            range.span,
                            "a counting `for` needs both ends",
                        )
                        .with_help("write `for i in 0..10 { ... }`; `..=` includes the end"),
                    );
                    body.scopes.pop();
                    return;
                };
                for bound in [start, end] {
                    let ty = self.type_of(body, bound);
                    if !matches!(ty, Ty::Number | Ty::Unknown) {
                        let found = self.ty_name(ty);
                        self.diagnostics.push(
                            Diagnostic::error(
                                "type",
                                file,
                                bound.span(),
                                format!("a counting `for` counts `number`s, but this is `{found}`"),
                            )
                            .with_help(if ty == Ty::Float {
                                "convert it with `as number`"
                            } else {
                                "use whole numbers for the range ends"
                            }),
                        );
                        body.scopes.pop();
                        return;
                    }
                }
                let magnitude = if let Some(increment) = increment {
                    if !Self::is_const_in(body, increment) {
                        self.error(
                            "for",
                            file,
                            increment.span(),
                            "`step_by` needs a positive constant number",
                        );
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    }
                    let Some(value) =
                        self.eval_i32(body.module, file, increment, Some(Ty::Number), None)
                    else {
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    };
                    if value <= 0 {
                        self.error(
                            "for",
                            file,
                            increment.span(),
                            "`step_by` needs a positive constant number",
                        );
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    }
                    value as i32
                } else {
                    1
                };
                let snapshot = if let Some(value) = collection {
                    let ty = self.type_of(body, value);
                    let (element, length) = if let Some((element, length)) = self.array_shape(ty) {
                        (element, Some(length))
                    } else if let Some(element) = self.list_element(ty) {
                        (element, None)
                    } else if ty == Ty::String {
                        (Ty::String, None)
                    } else {
                        self.error(
                            "for",
                            file,
                            value.span(),
                            "this loop needs an array, vector or string value",
                        );
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    };
                    let array = self.temp(body);
                    if self.emit(body, &array, ty, value, &mut setup).is_none() {
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    }
                    Some((array, element, length, ty == Ty::String))
                } else {
                    None
                };
                // Save the first bound before evaluating the second one: either
                // expression may call a function that changes the other's State.
                let first = self.temp(body);
                if self
                    .emit(body, &first, Ty::Number, start, &mut setup)
                    .is_none()
                {
                    body.scopes.pop();
                    body.temp_depth = depth;
                    return;
                }
                let binding_ty = snapshot
                    .as_ref()
                    .map_or(Ty::Number, |(_, element, _, _)| *element);
                let local = self.new_local(body, binding, binding_ty, false, binding.span);
                let binding_slot = self.local_ref(local);
                self.value_slots(&binding_slot, binding_ty);
                let counter = if snapshot.is_some() {
                    self.temp(body)
                } else {
                    binding_slot.clone()
                };
                let mut dynamic_length = None;
                if let Some((array, element, length, characters)) = snapshot {
                    if let Some(length) = length {
                        for offset in 0..length {
                            let mut then = Vec::new();
                            self.copy_value(
                                &binding_slot,
                                &super::values::child(&array, &format!("i{offset}")),
                                element,
                                file,
                                span,
                                &mut then,
                            );
                            pre_body.push(op_at(
                                pc::Action::Branch {
                                    condition: pc::Condition::Range(pc::ScoreComparison {
                                        score: counter.clone(),
                                        min: Some(offset as i32),
                                        max: Some(offset as i32),
                                    }),
                                    then,
                                    otherwise: Vec::new(),
                                },
                                file,
                                span,
                            ));
                        }
                    } else {
                        let length = self.temp(body);
                        setup.push(op_at(
                            pc::Action::DataRead {
                                score: length.clone(),
                                source: Self::data_reference(&array),
                            },
                            file,
                            span,
                        ));
                        dynamic_length = Some(length);
                        let source = Self::stored_element(&array, &counter);
                        if characters {
                            pre_body.push(op_at(
                                Self::initialize_leaf(
                                    binding_slot.clone(),
                                    super::text::LeafDefault::Data(super::text::text_value("")),
                                ),
                                file,
                                span,
                            ));
                            pre_body.push(op_at(
                                pc::Action::DataModify {
                                    target: Self::data_reference(&binding_slot),
                                    operator: pc::DataModifyOperation::Append,
                                    source: pc::DataOperand::Reference { value: source },
                                },
                                file,
                                span,
                            ));
                        } else {
                            self.unpack_value(
                                body,
                                &binding_slot,
                                source,
                                element,
                                span,
                                &mut pre_body,
                            );
                        }
                    }
                }
                let limit = if reverse {
                    if self
                        .emit(body, &counter, Ty::Number, end, &mut setup)
                        .is_none()
                    {
                        body.scopes.pop();
                        body.temp_depth = depth;
                        return;
                    }
                    Value::Read(first)
                } else {
                    setup.push(op_at(
                        Self::set_from(&counter, Value::Read(first)).expect("score copy"),
                        file,
                        start.span(),
                    ));
                    if let Some(length) = dynamic_length {
                        Value::Read(length)
                    } else {
                        match self.simple(body, end, Ty::Number) {
                            Ok(Some(Value::Const(value)))
                                if range.inclusive || value > i64::from(i32::MIN) =>
                            {
                                Value::Const(value)
                            }
                            Ok(_) => {
                                let temp = self.temp(body);
                                if self
                                    .emit(body, &temp, Ty::Number, end, &mut setup)
                                    .is_none()
                                {
                                    body.scopes.pop();
                                    body.temp_depth = depth;
                                    return;
                                }
                                Value::Read(temp)
                            }
                            Err(()) => {
                                body.scopes.pop();
                                body.temp_depth = depth;
                                return;
                            }
                        }
                    }
                };
                self.bring_into_scope(
                    body,
                    binding,
                    local,
                    Span::new(block.span.start, block.span.end),
                );
                let condition = match limit {
                    Value::Const(value) => {
                        let max = if range.inclusive { value } else { value - 1 };
                        let Ok(max) = i32::try_from(max) else {
                            self.error(
                                "range",
                                file,
                                end.span(),
                                "this range end overflows `number`",
                            );
                            body.scopes.pop();
                            body.temp_depth = depth;
                            return;
                        };
                        pc::Condition::Range(pc::ScoreComparison {
                            score: counter.clone(),
                            min: None,
                            max: Some(max),
                        })
                    }
                    Value::Read(limit) => pc::Condition::Relation(pc::ScoreRelation {
                        left: counter.clone(),
                        relation: match (reverse, range.inclusive) {
                            (true, true) => pc::Relation::Ge,
                            (true, false) => pc::Relation::Gt,
                            (false, true) => pc::Relation::Le,
                            (false, false) => pc::Relation::Lt,
                        },
                        right: limit,
                    }),
                };
                // A reversed exclusive range decrements before the body;
                // its strict lower-bound test proves this cannot underflow.
                if reverse && !range.inclusive {
                    pre_body.push(op_at(
                        pc::Action::ScoreAdd {
                            score: counter.clone(),
                            value: -1,
                        },
                        file,
                        range.span,
                    ));
                }
                let delta = if reverse {
                    -magnitude + i32::from(!range.inclusive)
                } else {
                    magnitude
                };
                if delta != 0 {
                    let advance = op_at(
                        pc::Action::ScoreAdd {
                            score: counter.clone(),
                            value: delta,
                        },
                        file,
                        range.span,
                    );
                    if count_guard {
                        // Do not wrap at the scoreboard's endpoints and loop
                        // forever. A synthetic break shares the normal jump flag.
                        let jump = jump.as_ref().expect("guarded counting loop");
                        step.push(op_at(
                            pc::Action::Branch {
                                condition: pc::Condition::Range(pc::ScoreComparison {
                                    score: counter,
                                    min: (delta < 0).then(|| i32::MIN - delta),
                                    max: (delta > 0).then(|| i32::MAX - delta),
                                }),
                                then: vec![advance],
                                otherwise: vec![op_at(
                                    pc::Action::ScoreSet {
                                        score: jump.clone(),
                                        value: 2,
                                    },
                                    file,
                                    span,
                                )],
                            },
                            file,
                            span,
                        ));
                    } else {
                        step.push(advance);
                    }
                }
                direct = Some(condition);
                keep_needed = has_break;
            }
        }
        let keep = keep_needed.then(|| self.slot(format!("{}_t{keep_index}", body.slot)));
        // `test` was built against a placeholder flag; point it at `keep`.
        let test: Vec<pc::Operation> = match (&keep, &direct) {
            (Some(keep), Some(condition)) => vec![
                op_at(
                    pc::Action::ScoreSet {
                        score: keep.clone(),
                        value: 0,
                    },
                    file,
                    span,
                ),
                op_at(
                    pc::Action::Branch {
                        condition: condition.clone(),
                        then: vec![op_at(
                            pc::Action::ScoreSet {
                                score: keep.clone(),
                                value: 1,
                            },
                            file,
                            span,
                        )],
                        otherwise: Vec::new(),
                    },
                    file,
                    span,
                ),
            ],
            (Some(keep), None) => retarget_marker(test, keep),
            (None, _) => Vec::new(),
        };
        body.jumps.push(super::Jump {
            flag: jump.clone(),
            executor_loops: body.loops.len(),
        });
        let lowered = self.lower_block(body, block);
        body.jumps.pop();
        body.scopes.pop();
        let mut inner = Vec::new();
        if let Some(jump) = &jump {
            inner.push(op_at(
                pc::Action::ScoreSet {
                    score: jump.clone(),
                    value: 0,
                },
                file,
                span,
            ));
        }
        inner.extend(pre_body);
        inner.extend(lowered);
        inner.extend(step);
        if let Some(keep) = &keep {
            match (&jump, has_break) {
                (Some(jump), true) => inner.push(op_at(
                    pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: jump.clone(),
                            min: Some(2),
                            max: Some(2),
                        }),
                        then: vec![op_at(
                            pc::Action::ScoreSet {
                                score: keep.clone(),
                                value: 0,
                            },
                            file,
                            span,
                        )],
                        otherwise: test.clone(),
                    },
                    file,
                    span,
                )),
                _ => inner.extend(test.clone()),
            }
        }
        body.temp_depth = depth;
        ops.extend(setup);
        let condition = match (keep, direct) {
            (Some(keep), _) => {
                ops.extend(test);
                pc::Condition::Range(pc::ScoreComparison {
                    score: keep,
                    min: Some(1),
                    max: Some(1),
                })
            }
            (None, Some(condition)) => condition,
            (None, None) => unreachable!("a loop without a direct test keeps a flag"),
        };
        ops.push(op_at(
            pc::Action::While {
                condition,
                body: inner,
            },
            file,
            span,
        ));
    }

    /// `keep = 1` (or 0) against the placeholder flag, retargeted later.
    fn marker_set(value: i32, file: FileId, span: Span) -> pc::Operation {
        op_at(
            pc::Action::ScoreSet {
                score: keep_marker(),
                value,
            },
            file,
            span,
        )
    }

    /// Set the placeholder flag to whether `condition` holds (or fails).
    fn flag_branch(
        condition: pc::Condition,
        negated: bool,
        file: FileId,
        span: Span,
    ) -> pc::Operation {
        op_at(
            pc::Action::Branch {
                condition,
                then: vec![Self::marker_set(i32::from(!negated), file, span)],
                otherwise: vec![Self::marker_set(i32::from(negated), file, span)],
            },
            file,
            span,
        )
    }

    /// `break;` and `continue;`.
    pub(super) fn lower_jump(
        &mut self,
        body: &mut BodyCx,
        is_break: bool,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        if body.value_blocks.last() == Some(&(body.loops.len(), body.jumps.len())) {
            self.error(
                "control",
                body.file,
                span,
                "a loop jump cannot leave a value block",
            );
            return;
        }
        let file = body.file;
        let word = if is_break { "break" } else { "continue" };
        let repeat = body
            .jumps
            .last()
            .filter(|jump| jump.executor_loops == body.loops.len());
        if let Some(jump) = repeat {
            let flag = jump
                .flag
                .clone()
                .expect("a loop with jumps keeps a jump flag");
            ops.push(op_at(
                pc::Action::ScoreSet {
                    score: flag,
                    value: if is_break { 2 } else { 1 },
                },
                file,
                span,
            ));
        } else if !body.loops.is_empty() {
            if is_break {
                self.diagnostics.push(
                    Diagnostic::error(
                        "control",
                        file,
                        span,
                        "`break` cannot stop a `players()` or `entities(..)` loop",
                    )
                    .with_help(
                        "every match runs once; `continue` skips the rest of this one's turn",
                    ),
                );
            } else {
                ops.push(op_at(pc::Action::Return, file, span));
            }
        } else {
            self.diagnostics.push(
                Diagnostic::error("control", file, span, format!("`{word}` only works inside a loop"))
                    .with_help("loops are `while`, `loop`, `for i in a..b`, `for player in players()`, and `for entity in entities(..)`"),
            );
        }
    }

    /// Bind call arguments to a builtin's parameters by position and name.
    pub(super) fn bind_args<'a>(
        &mut self,
        body: &BodyCx,
        builtin: &Builtin,
        args: &'a [Arg],
        span: Span,
    ) -> Option<BTreeMap<&'static str, &'a Arg>> {
        let file = body.file;
        let mut bound = BTreeMap::new();
        let mut ok = true;
        let mut position = 0;
        for arg in args {
            let param = match &arg.name {
                Some(name) => builtin.params.iter().find(|p| p.name == name.name),
                None => {
                    let param = builtin.params.get(position);
                    position += 1;
                    param
                }
            };
            let Some(param) = param else {
                let message = match &arg.name {
                    Some(name) => format!(
                        "`{}` has no argument named `{}`; expected {}",
                        builtin.name,
                        name.name,
                        param_list(builtin)
                    ),
                    None => format!(
                        "too many arguments: `{}` takes {}",
                        builtin.name,
                        param_list(builtin)
                    ),
                };
                self.diagnostics.push(
                    Diagnostic::error("call", file, arg.span, message)
                        .with_help(format!("usage: {}", builtin.signature)),
                );
                ok = false;
                continue;
            };
            if bound.insert(param.name, arg).is_some() {
                self.error(
                    "call",
                    file,
                    arg.span,
                    format!("argument `{}` is given more than once", param.name),
                );
                ok = false;
            }
        }
        for param in builtin.params.iter().filter(|p| p.required) {
            if !bound.contains_key(param.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        "call",
                        file,
                        span,
                        format!("`{}` needs `{}`: {}", builtin.name, param.name, param.doc),
                    )
                    .with_help(format!("usage: {}", builtin.signature)),
                );
                ok = false;
            }
        }
        ok.then_some(bound)
    }

    fn arg_string(
        &mut self,
        file: FileId,
        arg: &Arg,
        what: &str,
    ) -> Option<(String, String, Span)> {
        match &arg.value {
            ArgValue::Expr(Expr::Str { value, raw, span }) => {
                Some((value.clone(), raw.clone(), *span))
            }
            other => {
                self.error(
                    "type",
                    file,
                    other.span(),
                    format!("expected {what} in quotes"),
                );
                None
            }
        }
    }

    fn arg_id(&mut self, body: &BodyCx, arg: &Arg, registry: &'static str) -> Option<String> {
        let ArgValue::Expr(expr) = &arg.value else {
            self.error(
                "type",
                body.file,
                arg.value.span(),
                format!("expected {} `{registry}` value", super::article(registry)),
            );
            return None;
        };
        self.vanilla_value(body.module, body.file, expr, registry)
    }

    fn arg_equipment_slot(
        &mut self,
        body: &BodyCx,
        arg: &Arg,
    ) -> Option<sand::component::EquipmentSlot> {
        let value = self.arg_id(body, arg, "EquipmentSlot")?;
        match serde_json::from_value(serde_json::Value::String(value)) {
            Ok(slot) => Some(slot),
            Err(_) => {
                self.error(
                    "type",
                    body.file,
                    arg.span,
                    "expected an `EquipmentSlot::...` value",
                );
                None
            }
        }
    }

    pub(super) fn arg_int(&mut self, body: &BodyCx, arg: &Arg, min: i64, max: i64) -> Option<i64> {
        let ArgValue::Expr(expr) = &arg.value else {
            self.error("type", body.file, arg.value.span(), "expected an integer");
            return None;
        };
        if !Self::is_const_in(body, expr) {
            self.diagnostics.push(
                Diagnostic::error("type", body.file, expr.span(), "expected a constant number")
                    .with_help("this argument is written into the command when the pack is built"),
            );
            return None;
        }
        let value = self.eval_i32(body.module, body.file, expr, Some(Ty::Number), None)?;
        if value < min || value > max {
            self.error(
                "range",
                body.file,
                expr.span(),
                format!("expected a value from {min} to {max}, found {value}"),
            );
            return None;
        }
        Some(value)
    }

    fn arg_number(&mut self, body: &BodyCx, arg: &Arg, min: f64, max: f64) -> Option<f32> {
        let value = match &arg.value {
            ArgValue::Expr(Expr::Float(value, _)) => *value,
            ArgValue::Expr(expr) if Self::is_const_in(body, expr) => {
                self.eval_i32(body.module, body.file, expr, Some(Ty::Number), None)? as f64
            }
            other => {
                self.error(
                    "type",
                    body.file,
                    other.span(),
                    "expected a number such as `0.5`",
                );
                return None;
            }
        };
        if !(min..=max).contains(&value) {
            self.error(
                "range",
                body.file,
                arg.value.span(),
                format!("expected a number from {min} to {max}"),
            );
            return None;
        }
        Some(value as f32)
    }

    fn arg_bool(&mut self, file: FileId, arg: &Arg) -> Option<bool> {
        match &arg.value {
            ArgValue::Expr(Expr::Bool(value, _)) => Some(*value),
            other => {
                self.error("type", file, other.span(), "expected `true` or `false`");
                None
            }
        }
    }

    pub(super) fn lower_call(
        &mut self,
        body: &mut BodyCx,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (module, file) = (body.module, body.file);
        if path.segments.len() == 1
            && let Some(local) = Self::lookup_local(body, &path.last().name)
        {
            if let Some((def, actual)) = self.indirect_call(body, path, local, args) {
                let slots = self.fn_slots[&def].clone();
                self.call_user(body, def, &slots, path, &actual, span, ops);
            }
            return;
        }
        let name = &path.last().name;
        if !path.arguments().is_empty()
            && path.segments.len() == 1
            && !self.scopes[module.0].contains_key(name)
        {
            self.error(
                "generic",
                file,
                path.span,
                "builtins do not take type arguments",
            );
            return;
        }

        let is_builtin_name = path.segments.len() == 1 && !self.scopes[module.0].contains_key(name);
        if is_builtin_name && let Some(builtin) = builtins::find(name, BuiltinKind::Statement) {
            self.lower_builtin(body, builtin, path, args, span, ops);
            return;
        }
        if is_builtin_name && builtins::find(name, BuiltinKind::Macro).is_some() {
            self.diagnostics.push(
                Diagnostic::error("call", file, path.span, format!("`{name}` is a macro"))
                    .with_help(format!("write `{name}!(...)`")),
            );
            return;
        }
        let Some(def) = self.resolve_call(body, path, args, None, true) else {
            return;
        };
        if let Some(slots) = self.fn_slots.get(&def).cloned() {
            let _ = self.call_user(body, def, &slots, path, args, span, ops);
            return;
        }
        if let Some(first) = args.first() {
            self.error(
                "call",
                file,
                first.span.to(args.last().expect("non-empty").span),
                "external functions take no arguments",
            );
        }
        body.all_calls.push((def, path.span));
        if !body.in_players {
            body.calls.push((def, path.span));
        }
        if body.loops.last() == Some(&super::LoopKind::Entities) {
            body.entity_loop_calls.push((def, path.span));
        }
        if let Some(function) = self.function_reference(def) {
            ops.push(op_at(pc::Action::Call { function }, file, span));
        }
    }

    /// `emit(Event, args...)`: pass the payload to the event's dispatcher.
    /// `emit Event(args);`: evaluate the payload once into the event's
    /// parameters and run its dispatcher.
    pub(super) fn lower_emit(
        &mut self,
        body: &mut BodyCx,
        event: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (module, file) = (body.module, body.file);
        let Some(def) = self.expect_path(module, event, &[DefKind::Event], "an event") else {
            return;
        };
        if !self.event_handlers.contains_key(&def) {
            let name = self.defs[def.0].name.clone();
            self.diagnostics.push(
                Diagnostic::warning(
                    "event",
                    file,
                    event.span,
                    format!("`{name}` has no handlers, so this does nothing"),
                )
                .with_help(format!("mark a function with `#[on({name})]`")),
            );
        }
        let Some(slots) = self.fn_slots.get(&def).cloned() else {
            return;
        };
        let _ = self.call_user(body, def, &slots, event, args, span, ops);
    }

    fn lower_builtin(
        &mut self,
        body: &mut BodyCx,
        builtin: &'static Builtin,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        self.references_builtin(file, path.span, builtin);
        let Some(bound) = self.bind_args(body, builtin, args, span) else {
            return;
        };
        if builtin.needs_player {
            body.note_player(
                path.span,
                format!("uses `{}`, which needs a player", builtin.name),
            );
        }
        if builtins::EXECUTOR_BUILTINS.contains(&builtin.name) {
            body.note_executor(
                path.span,
                format!(
                    "uses `{}`, which acts on the executing entity",
                    builtin.name
                ),
            );
        }
        if builtin.name == "start_timer" {
            self.lower_start_timer(body, &bound, span, ops);
            return;
        }
        if builtin.name == "in_dimension" {
            let Some(dimension) = self
                .arg_id(body, bound["dimension"], "Dimension")
                .and_then(|id| id.parse().ok())
            else {
                return;
            };
            let ArgValue::Expr(Expr::Path(function)) = &bound["function"].value else {
                self.error(
                    "type",
                    file,
                    bound["function"].span,
                    "expected a parameterless function name",
                );
                return;
            };
            if !self
                .resolve_path(body.module, function)
                .is_ok_and(|(def, _)| {
                    matches!(
                        self.defs[def.0].kind,
                        DefKind::Function | DefKind::ExternFunction
                    )
                })
            {
                self.error(
                    "resolve",
                    file,
                    function.span,
                    "expected a parameterless function name",
                );
                return;
            }
            let mut nested = Vec::new();
            self.lower_call(body, function, &[], function.span, &mut nested);
            if !nested.is_empty() {
                ops.push(op_at(
                    pc::Action::InDimension {
                        dimension,
                        body: nested,
                    },
                    file,
                    span,
                ));
            }
            return;
        }
        if matches!(builtin.name, "set_bossbar" | "set_bossbar_max") {
            self.lower_set_bossbar(body, builtin, &bound, span, ops);
            return;
        }
        if matches!(
            builtin.name,
            "give_item"
                | "damage"
                | "mount"
                | "set_block_data"
                | "teleport"
                | "teleport_by"
                | "set_velocity"
                | "play_sound"
                | "add_experience"
        ) {
            let depth = body.temp_depth;
            let action = self.numeric_builtin_action(body, builtin.name, args, &bound, span, ops);
            body.temp_depth = depth;
            if let Some(action) = action {
                ops.push(op_at(action, file, span));
            }
            return;
        }
        if let Some(action) = self.builtin_action(body, builtin, &bound, span) {
            ops.push(op_at(action, file, span));
        }
    }

    // Numeric command arguments are frozen in authored argument order. A later
    // argument may call a function that changes the State read by an earlier one.
    fn numeric_builtin_action(
        &mut self,
        body: &mut BodyCx,
        name: &str,
        args: &[Arg],
        bound: &BTreeMap<&str, &Arg>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<pc::Action> {
        match name {
            "give_item" => {
                let item = self.arg_id(body, bound["item"], "Item")?;
                let count = match bound.get("count") {
                    None => pc::ScoreOperand::Value(1),
                    Some(arg) => {
                        let ArgValue::Expr(expr) = &arg.value else {
                            self.error(
                                "type",
                                body.file,
                                arg.span,
                                "expected a number value, found a range",
                            );
                            return None;
                        };
                        if Self::is_const_in(body, expr) {
                            pc::ScoreOperand::Value(self.arg_int(body, arg, 1, 6400)? as i32)
                        } else {
                            let found = self.type_of(body, expr);
                            if !self.types_agree(body.file, expr.span(), Ty::Number, found) {
                                return None;
                            }
                            let score = self.temp(body);
                            self.emit(body, &score, Ty::Number, expr, ops)?;
                            pc::ScoreOperand::Field(score)
                        }
                    }
                };
                Some(pc::Action::GiveItem { item, count })
            }
            "play_sound" => {
                let sound = self.arg_id(body, bound["sound"], "Sound")?;
                let mut volume = pc::Amount::Value(1.0);
                let mut pitch = pc::Amount::Value(1.0);
                for arg in args {
                    let (key, _) = bound.iter().find(|(_, value)| std::ptr::eq(**value, arg))?;
                    match *key {
                        "volume" => volume = self.runtime_amount(body, arg, ops)?,
                        "pitch" => pitch = self.runtime_amount(body, arg, ops)?,
                        _ => {}
                    }
                }
                for (key, amount, max) in
                    [("volume", &volume, f32::MAX as f64), ("pitch", &pitch, 2.0)]
                {
                    if matches!(amount, pc::Amount::Value(value) if !(0.0..=max).contains(value)) {
                        self.error(
                            "range",
                            body.file,
                            bound.get(key).map_or(span, |arg| arg.span),
                            format!("sound {key} must be from 0 to {max}"),
                        );
                        return None;
                    }
                }
                Some(pc::Action::Sound {
                    sound,
                    volume,
                    pitch,
                })
            }
            "add_experience" => {
                let (arg, levels) = match (bound.get("points"), bound.get("levels")) {
                    (Some(arg), None) => (*arg, false),
                    (None, Some(arg)) => (*arg, true),
                    _ => {
                        self.error(
                            "call",
                            body.file,
                            span,
                            "`add_experience` needs exactly one of `points` or `levels`",
                        );
                        return None;
                    }
                };
                let ArgValue::Expr(expr) = &arg.value else {
                    self.error(
                        "type",
                        body.file,
                        arg.span,
                        "expected a number value, found a range",
                    );
                    return None;
                };
                let found = self.type_of(body, expr);
                if !self.types_agree(body.file, expr.span(), Ty::Number, found) {
                    return None;
                }
                let amount = if Self::is_const_in(body, expr) {
                    pc::ScoreOperand::Value(self.arg_int(
                        body,
                        arg,
                        i32::MIN as i64,
                        i32::MAX as i64,
                    )? as i32)
                } else {
                    let score = self.temp(body);
                    self.emit(body, &score, Ty::Number, expr, ops)?;
                    pc::ScoreOperand::Field(score)
                };
                Some(pc::Action::AddExperience { amount, levels })
            }
            "damage" => {
                let mut amount = None;
                let mut by = None;
                for arg in args {
                    if std::ptr::eq(arg, bound["amount"]) {
                        amount = Some(self.runtime_amount(body, arg, ops)?);
                    } else if bound.get("by").is_some_and(|by| std::ptr::eq(arg, *by)) {
                        let ArgValue::Expr(expr) = &arg.value else {
                            return None;
                        };
                        let score = self.temp(body);
                        self.emit(body, &score, Ty::EntityRef, expr, ops)?;
                        by = Some(score);
                    }
                }
                let amount = amount?;
                if let pc::Amount::Value(value) = amount
                    && !(0.01..=1_000_000.0).contains(&value)
                {
                    self.error(
                        "range",
                        body.file,
                        bound["amount"].span,
                        "damage must be from 0.01 to 1000000",
                    );
                    return None;
                }
                let damage_type = match bound.get("type") {
                    Some(arg) => Some(self.arg_id(body, arg, "DamageType")?.parse().ok()?),
                    None => None,
                };
                Some(pc::Action::Damage {
                    amount,
                    by,
                    damage_type,
                })
            }
            "set_block_data" => {
                let target = self.arg_block_data(body, bound["at"], bound["path"])?;
                let arg = bound["value"];
                let ArgValue::Expr(expr) = &arg.value else {
                    self.error("type", body.file, arg.span, "expected an integer value");
                    return None;
                };
                let value = if matches!(
                    expr,
                    Expr::Struct { .. } | Expr::Array(..) | Expr::Str { .. }
                ) || Self::is_const_in(body, expr)
                {
                    pc::DataOperand::Value {
                        value: self.const_nbt_value(body, expr, 0)?,
                    }
                } else {
                    let value = match self.simple(body, expr, Ty::Number).ok()? {
                        Some(Value::Const(value)) => pc::ScoreOperand::Value(value as i32),
                        Some(Value::Read(score)) => pc::ScoreOperand::Field(score),
                        None => {
                            let score = self.temp(body);
                            self.emit(body, &score, Ty::Number, expr, ops)?;
                            pc::ScoreOperand::Field(score)
                        }
                    };
                    pc::DataOperand::Score { value }
                };
                Some(pc::Action::WriteBlockData {
                    target,
                    value: Some(value),
                })
            }
            "mount" => {
                let arg = bound["vehicle"];
                let ArgValue::Expr(expr) = &arg.value else {
                    self.error("type", body.file, arg.span, "expected an `EntityRef` value");
                    return None;
                };
                let vehicle = self.temp(body);
                self.emit(body, &vehicle, Ty::EntityRef, expr, ops)?;
                Some(pc::Action::Ride {
                    vehicle: Some(vehicle),
                })
            }
            "teleport" | "teleport_by" | "set_velocity" => {
                let keys = if name == "teleport_by" {
                    ["dx", "dy", "dz"]
                } else {
                    ["x", "y", "z"]
                };
                let mut values = BTreeMap::new();
                for arg in args {
                    let (key, _) = bound.iter().find(|(_, value)| std::ptr::eq(**value, arg))?;
                    if keys.contains(key) {
                        let amount = self.runtime_amount(body, arg, ops)?;
                        if name == "set_velocity"
                            && matches!(amount, pc::Amount::Value(value) if !(-10.0..=10.0).contains(&value))
                        {
                            self.error(
                                "range",
                                body.file,
                                arg.span,
                                "velocity must be from -10 to 10 blocks per tick",
                            );
                            return None;
                        }
                        values.insert(*key, amount);
                    }
                }
                let x = values.remove(keys[0])?;
                let y = values.remove(keys[1])?;
                let z = values.remove(keys[2])?;
                Some(match name {
                    "set_velocity" => pc::Action::SetVelocity { x, y, z },
                    "teleport" => pc::Action::Teleport {
                        destination: pc::TeleportDestination::Position { x, y, z },
                    },
                    _ => pc::Action::Teleport {
                        destination: pc::TeleportDestination::Offset { x, y, z },
                    },
                })
            }
            _ => unreachable!("numeric builtin"),
        }
    }

    fn lower_start_timer(
        &mut self,
        body: &mut BodyCx,
        bound: &BTreeMap<&str, &Arg>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let Some(timer) = self.arg_timer(body, bound["timer"], "starts") else {
            return;
        };
        let (arg, scale) = match (bound.get("ticks"), bound.get("seconds")) {
            (Some(arg), None) => (*arg, 1),
            (None, Some(arg)) => (*arg, 20),
            _ => {
                self.error(
                    "call",
                    body.file,
                    span,
                    "`start_timer` needs exactly one of `ticks` or `seconds`",
                );
                return;
            }
        };
        let max = i32::MAX / scale;
        let ArgValue::Expr(expr) = &arg.value else {
            self.error(
                "type",
                body.file,
                arg.span,
                "expected a number duration, found a range",
            );
            return;
        };
        if Self::is_const_in(body, expr) {
            if let Some(value) = self.arg_int(body, arg, 1, max as i64) {
                ops.push(op_at(
                    pc::Action::ScoreSet {
                        score: timer,
                        value: value as i32 * scale,
                    },
                    body.file,
                    span,
                ));
            }
            return;
        }
        let found = self.type_of(body, expr);
        if !self.types_agree(body.file, expr.span(), Ty::Number, found) {
            return;
        }
        let depth = body.temp_depth;
        let temp = self.temp(body);
        let computed = self.emit(body, &temp, Ty::Number, expr, ops);
        body.temp_depth = depth;
        if computed.is_none() {
            return;
        }
        let mut then = Vec::new();
        if scale != 1 {
            then.push(op_at(
                pc::Action::ScoreOperation {
                    score: temp.clone(),
                    operator: pc::ScoreOperator::Mul,
                    operand: pc::ScoreOperand::Value(scale),
                },
                body.file,
                span,
            ));
        }
        if let Some(action) = Self::set_from(&timer, Value::Read(temp.clone())) {
            then.push(op_at(action, body.file, span));
        }
        ops.push(op_at(
            pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: temp,
                    min: Some(1),
                    max: Some(max),
                }),
                then,
                otherwise: Vec::new(),
            },
            body.file,
            span,
        ));
    }

    fn runtime_amount(
        &mut self,
        body: &mut BodyCx,
        arg: &Arg,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<pc::Amount> {
        let ArgValue::Expr(expr) = &arg.value else {
            self.error("type", body.file, arg.span, "expected a numeric value");
            return None;
        };
        let ty = self.type_of(body, expr);
        self.expect_numeric(body.file, expr.span(), ty)?;
        if Self::is_const_in(body, expr) {
            let value = if matches!(expr, Expr::Float(..))
                || matches!(expr, Expr::Neg(inner, _) if matches!(**inner, Expr::Float(..)))
            {
                self.arg_coordinate(body, arg)?
            } else {
                self.eval_i32(body.module, body.file, expr, Some(ty), None)? as f64
                    / if ty == Ty::Float {
                        FLOAT_SCALE as f64
                    } else {
                        1.0
                    }
            };
            return Some(pc::Amount::Value(value));
        }
        let score = self.temp(body);
        self.emit(body, &score, ty, expr, ops)?;
        Some(pc::Amount::Field(pc::ScaledField {
            score,
            scale: if ty == Ty::Float {
                FLOAT_SCALE as f64
            } else {
                1.0
            },
        }))
    }

    /// A constant coordinate, in full double precision.
    fn arg_coordinate(&mut self, body: &BodyCx, arg: &Arg) -> Option<f64> {
        let file = body.file;
        let value = match &arg.value {
            ArgValue::Expr(Expr::Float(value, _)) => *value,
            ArgValue::Expr(Expr::Neg(inner, _)) if matches!(**inner, Expr::Float(..)) => {
                let Expr::Float(value, _) = **inner else {
                    unreachable!()
                };
                -value
            }
            ArgValue::Expr(expr) if Self::is_const_in(body, expr) => {
                self.eval_i32(body.module, file, expr, Some(Ty::Number), None)? as f64
            }
            other => {
                self.diagnostics.push(
                    Diagnostic::error("type", file, other.span(), "coordinates must be constants")
                        .with_help("use `teleport_by` to move relative to the entity"),
                );
                return None;
            }
        };
        Some(value)
    }

    /// An argument naming a declared `entity state`.
    fn arg_attachable_state(&mut self, body: &BodyCx, arg: &Arg) -> Option<(DefId, String)> {
        let (module, file) = (body.module, body.file);
        let ArgValue::Expr(Expr::Path(path)) = &arg.value else {
            self.error(
                "type",
                file,
                arg.value.span(),
                "expected an entity state name",
            );
            return None;
        };
        let def = self.expect_path(module, path, &[DefKind::State], "an entity state")?;
        let DefInfo::State { scope, id, .. } = &self.defs[def.0].info else {
            return None;
        };
        if *scope != Scope::Entity {
            let name = self.defs[def.0].name.clone();
            self.diagnostics.push(
                Diagnostic::error(
                    "type",
                    file,
                    path.span,
                    format!("`{name}` is not an entity state"),
                )
                .with_help("only `entity state` declarations are attached to entities"),
            );
            return None;
        }
        Some((def, id.clone()))
    }

    /// A `State.field` argument naming a `timer` field.
    fn arg_timer(
        &mut self,
        body: &mut BodyCx,
        arg: &Arg,
        verb: &str,
    ) -> Option<pc::FieldReference> {
        let (module, file) = (body.module, body.file);
        let ArgValue::Expr(Expr::Place(place)) = &arg.value else {
            self.diagnostics.push(
                Diagnostic::error("type", file, arg.value.span(), "expected a `timer` field")
                    .with_help("write it as `State.field`"),
            );
            return None;
        };
        let (state, field, _) = self.resolve_place(module, file, place)?;
        if verb != "tests" && !self.check_state_write(body, state, place.span) {
            return None;
        }
        if !matches!(self.defs[field.0].info, DefInfo::Field { timer: true, .. }) {
            self.diagnostics.push(
                Diagnostic::error(
                    "type",
                    file,
                    place.field.span,
                    format!("`{}` is not a `timer` field", place.field.name),
                )
                .with_help("declare it as `name: timer`"),
            );
            return None;
        }
        body.note_state(self, state, place, verb);
        self.field_reference(state, field)
    }

    /// `pos(x, y, z)` or `offset(dx, dy, dz)`.
    fn arg_position(&mut self, body: &mut BodyCx, arg: &Arg) -> Option<pc::BlockPosition> {
        let file = body.file;
        let call = match &arg.value {
            ArgValue::Expr(Expr::Call { path, args, .. }) if path.segments.len() == 1 => {
                Some((path, args))
            }
            _ => None,
        };
        let Some((path, args)) = call.filter(|(path, args)| {
            matches!(path.segments[0].name.as_str(), "pos" | "offset") && args.len() == 3
        }) else {
            self.diagnostics.push(
                Diagnostic::error("type", file, arg.value.span(), "expected a block position")
                    .with_help("write `pos(x, y, z)` or `offset(dx, dy, dz)`"),
            );
            return None;
        };
        let min = i32::MIN as i64;
        let max = i32::MAX as i64;
        if let Some(arg) = args.iter().find(
            |arg| !matches!(&arg.value, ArgValue::Expr(expr) if Self::is_const_in(body, expr)),
        ) {
            self.diagnostics.push(
                Diagnostic::error("type", file, arg.span, "block positions must be constants")
                    .with_help("positions are written into the command when the pack is built"),
            );
            return None;
        }
        let x = self.arg_int(body, &args[0], min, max);
        let y = self.arg_int(body, &args[1], min, max);
        let z = self.arg_int(body, &args[2], min, max);
        let (x, y, z) = (x? as i32, y? as i32, z? as i32);
        Some(if path.segments[0].name == "offset" {
            body.note_executor(
                path.span,
                "uses `offset`, which is relative to the executing entity".into(),
            );
            pc::BlockPosition::Relative { x, y, z }
        } else {
            pc::BlockPosition::Absolute { x, y, z }
        })
    }

    /// An argument naming a declared boss bar or loot table.
    fn arg_declared(
        &mut self,
        body: &BodyCx,
        arg: &Arg,
        kind: DefKind,
        what: &str,
    ) -> Option<sand::ResourceLocation> {
        let ArgValue::Expr(Expr::Path(path)) = &arg.value else {
            self.error(
                "type",
                body.file,
                arg.value.span(),
                format!("expected {what}"),
            );
            return None;
        };
        let def = self.expect_path(body.module, path, &[kind], what)?;
        let DefInfo::Data { id, .. } = &self.defs[def.0].info else {
            return None;
        };
        id.parse().ok()
    }

    fn arg_block_data(
        &mut self,
        body: &mut BodyCx,
        position: &Arg,
        path: &Arg,
    ) -> Option<pc::BlockDataAddress> {
        let at = self.arg_position(body, position)?;
        let (path, _, _) = self.arg_string(body.file, path, "an NBT path")?;
        Some(pc::BlockDataAddress { at, path })
    }

    /// A vanilla block, or `#tag` for a declared `Block` tag.
    fn arg_block_test(&mut self, body: &BodyCx, arg: &Arg) -> Option<String> {
        if let ArgValue::Expr(expr) = &arg.value
            && let Some((def, id)) = self.declared(body.module, body.file, expr, DefKind::Tag)
        {
            if matches!(
                self.defs[def.0].info,
                DefInfo::Data {
                    tag_type: Some("Block"),
                    ..
                }
            ) {
                return Some(format!("#{id}"));
            }
            self.error(
                "type",
                body.file,
                expr.span(),
                format!("`{}` is not a `Block` tag", self.defs[def.0].name),
            );
            return None;
        }
        self.arg_id(body, arg, "Block")
    }

    fn lower_set_bossbar(
        &mut self,
        body: &mut BodyCx,
        builtin: &'static Builtin,
        bound: &BTreeMap<&'static str, &Arg>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let Some(bar) = self.arg_declared(body, bound["bar"], DefKind::BossBar, "a boss bar")
        else {
            return;
        };
        let max = builtin.name == "set_bossbar_max";
        let value_arg = bound[if max { "max" } else { "value" }];
        let ArgValue::Expr(expr) = &value_arg.value else {
            self.error("type", file, value_arg.span, "expected a value");
            return;
        };
        let ty = self.type_of(body, expr);
        if !self.types_agree(file, expr.span(), Ty::Number, ty) {
            return;
        }
        let value = if Self::is_const_in(body, expr) {
            let Some(value) =
                self.arg_int(body, value_arg, if max { 1 } else { 0 }, i32::MAX as i64)
            else {
                return;
            };
            pc::ScoreOperand::Value(value as i32)
        } else {
            match self.simple(body, expr, ty) {
                Ok(Some(Value::Read(reference))) => pc::ScoreOperand::Field(reference),
                Ok(_) => {
                    let depth = body.temp_depth;
                    let temp = self.temp(body);
                    if self.emit(body, &temp, ty, expr, ops).is_none() {
                        return;
                    }
                    body.temp_depth = depth;
                    pc::ScoreOperand::Field(temp)
                }
                Err(()) => return,
            }
        };
        ops.push(op_at(
            pc::Action::SetBossBar { bar, value, max },
            file,
            span,
        ));
    }

    /// A predicate call in a condition, such as `has_state(Mana)`.
    pub(super) fn predicate(
        &mut self,
        body: &mut BodyCx,
        path: &Path,
        args: &[Arg],
        span: Span,
    ) -> Option<pc::Condition> {
        if !path.arguments().is_empty() {
            self.error(
                "generic",
                body.file,
                path.span,
                "builtins do not take type arguments",
            );
            return None;
        }
        let file = body.file;
        let name = &path.last().name;
        let Some(builtin) = (path.segments.len() == 1)
            .then(|| builtins::find(name, BuiltinKind::Condition))
            .flatten()
        else {
            self.diagnostics.push(
                Diagnostic::error("condition", file, path.span, format!("`{}` is not a condition", path.display()))
                    .with_help("compare a value, like `price(2) > 5`, or use a predicate such as `has_state(...)`"),
            );
            return None;
        };
        self.references_builtin(file, path.span, builtin);
        let bound = self.bind_args(body, builtin, args, span)?;
        if matches!(builtin.name, "has_state")
            || builtins::EXECUTOR_BUILTINS.contains(&builtin.name)
        {
            body.note_executor(
                path.span,
                format!("uses `{name}`, which tests the executing entity"),
            );
        }
        let flag = |flag| Some(pc::Condition::Flag(pc::FlagSet { flag }));
        match builtin.name {
            "has_state" => {
                let (_, id) = self.arg_attachable_state(body, bound["state"])?;
                Some(pc::Condition::Attached(pc::StateAttached {
                    attached: id.parse().ok()?,
                }))
            }
            "any_entities" => Some(pc::Condition::Exists(pc::EntitiesExist {
                exists: self.query_from(body, &bound)?,
            })),
            "block_is" => {
                let block = self.arg_block_test(body, bound["block"]);
                let at = self.arg_position(body, bound["at"]);
                Some(pc::Condition::Block(pc::BlockMatches {
                    block: block?,
                    at: at?,
                }))
            }
            "biome_is" => Some(pc::Condition::Biome(pc::BiomeMatches {
                biome: self.arg_id(body, bound["biome"], "Biome")?,
                at: self.arg_position(body, bound["at"])?,
            })),
            "predicate_matches" => {
                let arg = bound["predicate"];
                let ArgValue::Expr(expr) = &arg.value else {
                    self.error("type", file, arg.span, "expected a predicate resource");
                    return None;
                };
                Some(pc::Condition::Predicate {
                    predicate: self.predicate_id(body.module, file, expr)?,
                })
            }
            "dimension_is" => Some(pc::Condition::Dimension(pc::DimensionMatches {
                dimension: self
                    .arg_id(body, bound["dimension"], "Dimension")?
                    .parse()
                    .ok()?,
            })),
            "holding" => {
                let holding = self.arg_id(body, bound["item"], "Item")?;
                let offhand = match bound.get("offhand") {
                    Some(arg) => self.arg_bool(file, arg)?,
                    None => false,
                };
                let slot = match bound.get("slot") {
                    Some(arg) if bound.contains_key("offhand") => {
                        self.error("type", file, arg.span, "choose either slot or offhand");
                        return None;
                    }
                    Some(arg) => Some(self.arg_equipment_slot(body, arg)?),
                    None => None,
                };
                Some(pc::Condition::Holding(pc::ItemHeld {
                    holding,
                    offhand,
                    slot,
                }))
            }
            "is_sneaking" => flag(pc::EntityFlag::Sneaking),
            "is_sprinting" => flag(pc::EntityFlag::Sprinting),
            "is_swimming" => flag(pc::EntityFlag::Swimming),
            "is_on_fire" => flag(pc::EntityFlag::OnFire),
            "timer_running" | "timer_done" => {
                let score = self.arg_timer(body, bound["timer"], "tests")?;
                let (min, max) = if builtin.name == "timer_running" {
                    (Some(1), None)
                } else {
                    (Some(0), Some(0))
                };
                Some(pc::Condition::Range(pc::ScoreComparison {
                    score,
                    min,
                    max,
                }))
            }
            other => unreachable!("condition builtin `{other}` is not lowered"),
        }
    }

    /// Parse the filters of `entities(..)` in a `for` loop.
    pub(super) fn entity_query(
        &mut self,
        body: &mut BodyCx,
        builtin: &'static Builtin,
        args: &[Arg],
        span: Span,
    ) -> Option<pc::EntityQuery> {
        let bound = self.bind_args(body, builtin, args, span)?;
        self.query_from(body, &bound)
    }

    fn query_from(
        &mut self,
        body: &mut BodyCx,
        bound: &BTreeMap<&'static str, &Arg>,
    ) -> Option<pc::EntityQuery> {
        let file = body.file;
        let mut query = pc::EntityQuery::default();
        let mut ok = true;
        if let Some(arg) = bound.get("type") {
            match self.arg_id(body, arg, "Entity") {
                Some(id) => query.entity_type = Some(id),
                None => ok = false,
            }
        }
        if let Some(arg) = bound.get("tag") {
            match self.arg_string(file, arg, "a tag name") {
                Some((tag, _, _)) if valid_tag(&tag) => query.tag = Some(tag),
                Some((tag, _, tag_span)) => {
                    self.error(
                        "id",
                        file,
                        tag_span,
                        format!("`{tag}` is not a valid tag; use letters, digits, and `_ . + -`"),
                    );
                    ok = false;
                }
                None => ok = false,
            }
        }
        if let Some(arg) = bound.get("distance") {
            match &arg.value {
                ArgValue::Range(range) => {
                    let mut bound_of = |expr: &Option<Expr>| -> Option<Option<f64>> {
                        match expr {
                            None => Some(None),
                            Some(Expr::Float(v, _)) => Some(Some(*v)),
                            Some(e) => self
                                .eval_i32(body.module, file, e, Some(Ty::Number), None)
                                .map(|v| Some(v as f64)),
                        }
                    };
                    match (bound_of(&range.start), bound_of(&range.end)) {
                        (Some(min), Some(max)) => {
                            query.min_distance = min;
                            query.max_distance = max;
                        }
                        _ => ok = false,
                    }
                }
                ArgValue::Expr(expr) => {
                    self.diagnostics.push(
                        Diagnostic::error("type", file, expr.span(), "expected a distance range")
                            .with_help("for example `distance = ..=16` or `distance = 4..=10`"),
                    );
                    ok = false;
                }
            }
        }
        if let Some(arg) = bound.get("with") {
            match self
                .arg_attachable_state(body, arg)
                .and_then(|(_, id)| id.parse().ok())
            {
                Some(state) => query.with_state = Some(state),
                None => ok = false,
            }
        }
        if let Some(arg) = bound.get("limit") {
            match self.arg_int(body, arg, 1, i32::MAX as i64) {
                Some(limit) => query.limit = Some(limit as u32),
                None => ok = false,
            }
        }
        if let Some(arg) = bound.get("sort") {
            match self.arg_id(body, arg, "Sort").as_deref() {
                Some("nearest") => query.sort = Some(pc::EntitySort::Nearest),
                Some("furthest") => query.sort = Some(pc::EntitySort::Furthest),
                Some("random") => query.sort = Some(pc::EntitySort::Random),
                Some(_) => query.sort = Some(pc::EntitySort::Arbitrary),
                None => ok = false,
            }
        }
        query.score = match (bound.get("score"), bound.get("score_range")) {
            (Some(field), Some(range)) => match self.query_score(body, field, range) {
                Some(filter) => Some(filter),
                None => {
                    ok = false;
                    None
                }
            },
            (Some(arg), None) | (None, Some(arg)) => {
                self.error("type", file, arg.span, "supply both score and score_range");
                ok = false;
                None
            }
            (None, None) => None,
        };
        if let Some(arg) = bound.get("nbt") {
            if let ArgValue::Expr(Expr::Struct { path, fields, .. }) = &arg.value
                && path.segments.len() == 1
                && path.applications.is_empty()
                && path.last().name == "Nbt"
            {
                query.nbt = self.const_nbt_compound(body, fields, 0);
                ok &= query.nbt.is_some();
            } else {
                self.error(
                    "type",
                    file,
                    arg.span,
                    "expected a typed compound such as `Nbt { Silent: true }`",
                );
                ok = false;
            }
        }
        ok.then_some(query)
    }

    fn query_score(
        &mut self,
        body: &BodyCx,
        field: &Arg,
        range: &Arg,
    ) -> Option<pc::ScoreComparison> {
        let file = body.file;
        let ArgValue::Expr(Expr::Place(place)) = &field.value else {
            self.error(
                "type",
                file,
                field.span,
                "expected a player or entity `State.field`",
            );
            return None;
        };
        let (state, field, ty) = self.resolve_place(body.module, file, place)?;
        if ty != Ty::Number
            || !matches!(
                self.defs[state.0].info,
                DefInfo::State {
                    scope: Scope::Entity | Scope::Player,
                    ..
                }
            )
        {
            self.error(
                "type",
                file,
                place.span,
                "query scores require a number field on player or entity State",
            );
            return None;
        }
        let mut evaluate = |expr: &Expr| {
            self.eval_i32(body.module, file, expr, Some(Ty::Number), None)
                .map(|number| number as i32)
        };
        let (min, max) = match &range.value {
            ArgValue::Expr(expr) => {
                let exact = evaluate(expr)?;
                (Some(exact), Some(exact))
            }
            ArgValue::Range(range) => {
                let min = match &range.start {
                    Some(expr) => Some(evaluate(expr)?),
                    None => None,
                };
                let max = match &range.end {
                    Some(expr) => {
                        let end = evaluate(expr)?;
                        if range.inclusive {
                            Some(end)
                        } else {
                            match end.checked_sub(1) {
                                Some(end) => Some(end),
                                None => {
                                    self.error(
                                        "range",
                                        file,
                                        expr.span(),
                                        "exclusive score bound leaves no possible value",
                                    );
                                    return None;
                                }
                            }
                        }
                    }
                    None => None,
                };
                (min, max)
            }
        };
        if (min.is_none() && max.is_none()) || min.zip(max).is_some_and(|(min, max)| min > max) {
            self.error(
                "range",
                file,
                range.span,
                "query score bounds must be ordered with at least one bound",
            );
            return None;
        }
        Some(pc::ScoreComparison {
            score: self.field_reference(state, field)?,
            min,
            max,
        })
    }

    fn const_nbt_compound(
        &mut self,
        body: &BodyCx,
        fields: &[(Ident, Expr)],
        depth: usize,
    ) -> Option<sand::data::NbtCompound> {
        let mut nbt = sand::data::NbtCompound::new();
        let mut seen = std::collections::BTreeSet::new();
        for (field, expr) in fields {
            if !seen.insert(&field.name) {
                self.error(
                    "duplicate",
                    body.file,
                    field.span,
                    format!("duplicate NBT field `{}`", field.name),
                );
                return None;
            }
            nbt = nbt.field(&field.name, self.const_nbt_value(body, expr, depth + 1)?);
        }
        Some(nbt)
    }

    fn const_nbt_value(
        &mut self,
        body: &BodyCx,
        expr: &Expr,
        depth: usize,
    ) -> Option<sand::data::NbtValue> {
        use sand::data::NbtValue;
        let file = body.file;
        if depth > 32 {
            self.error(
                "limit",
                file,
                expr.span(),
                "typed NBT values may be nested at most 32 levels",
            );
            return None;
        }
        Some(match expr {
            Expr::Str { value, .. } => NbtValue::String(value.clone()),
            Expr::Float(value, _) => NbtValue::Double(*value),
            Expr::Neg(inner, _) if matches!(inner.as_ref(), Expr::Float(_, _)) => {
                let Expr::Float(value, _) = inner.as_ref() else {
                    unreachable!()
                };
                NbtValue::Double(-value)
            }
            Expr::Array(values, _) => NbtValue::List(
                values
                    .iter()
                    .map(|element| self.const_nbt_value(body, element, depth + 1))
                    .collect::<Option<_>>()?,
            ),
            Expr::Struct { path, fields, .. }
                if path.segments.len() == 1
                    && path.applications.is_empty()
                    && path.last().name == "Nbt" =>
            {
                NbtValue::Compound(self.const_nbt_compound(body, fields, depth)?)
            }
            _ => {
                let (value, ty) = self.eval(body.module, file, expr, None)?;
                match ty {
                    Ty::Bool => NbtValue::Bool(value != 0),
                    Ty::Float => NbtValue::Double(value as f64 / super::FLOAT_SCALE as f64),
                    Ty::Number => match i32::try_from(value) {
                        Ok(value) => NbtValue::Int(value),
                        Err(_) => {
                            self.error(
                                "range",
                                file,
                                expr.span(),
                                "NBT integer must fit in a number",
                            );
                            return None;
                        }
                    },
                    _ => {
                        self.error(
                            "type",
                            file,
                            expr.span(),
                            "expected a constant NBT scalar, list, or compound",
                        );
                        return None;
                    }
                }
            }
        })
    }

    /// Whether `Handle.field` names an entity handle rather than a State:
    /// `Player` (when no item of that name is in scope) or the binding of an
    /// enclosing `players()` or `entities(..)` loop.
    pub(super) fn is_handle(&self, body: &BodyCx, place: &Place) -> bool {
        if !place.state.applications.is_empty() {
            return false;
        }
        let [segment] = place.state.segments.as_slice() else {
            return false;
        };
        let name = segment.name.as_str();
        body.loop_bindings.iter().any(|b| b == name)
            || (name == "Player" && !self.scopes[body.module.0].contains_key(name))
    }

    /// Resolve a property of a handle, checking that the handle is the
    /// executor and that the property exists on it.
    fn entity_property(&mut self, body: &mut BodyCx, place: &Place) -> Option<Property> {
        let file = body.file;
        let handle = place.state.segments[0].name.clone();
        let name = &place.field.name;
        let Some(property) = crate::properties::find(name) else {
            let suggestions = crate::properties::suggest(name);
            let mut diagnostic = Diagnostic::error(
                "resolve",
                file,
                place.field.span,
                format!("`{handle}` has no property `{name}`"),
            );
            diagnostic = if suggestions.is_empty() {
                diagnostic.with_help("properties are entity data such as `health`, `food`, and `on_ground`, and every attribute, such as `max_health` or `step_height`")
            } else {
                diagnostic.with_help(format!(
                    "did you mean {}?",
                    suggestions
                        .iter()
                        .map(|s| format!("`{s}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            };
            self.diagnostics.push(diagnostic);
            return None;
        };
        self.property_uses.push(super::PropertyUse {
            file,
            span: place.field.span,
            handle: handle.clone(),
            property: property.clone(),
        });
        if handle == "Player" && !body.loop_bindings.iter().any(|b| b == "Player") {
            body.note_player(
                place.span,
                format!("reads `Player.{name}`, which needs a player"),
            );
            return Some(property);
        }
        let innermost = body.loop_bindings.last().map(String::as_str);
        if innermost != Some(handle.as_str()) {
            let current = innermost.unwrap_or_default().to_string();
            self.diagnostics.push(
                Diagnostic::error(
                    "context",
                    file,
                    place.state.span,
                    format!("`{handle}` is not the executing entity here"),
                )
                .with_help(format!(
                    "only the innermost loop's `{current}` can be read or changed; copy `{handle}.{name}` into a `let` before the inner loop"
                )),
            );
            return None;
        }
        if property.player_only && body.loops.last() == Some(&super::LoopKind::Entities) {
            self.diagnostics.push(
                Diagnostic::error(
                    "context",
                    file,
                    place.field.span,
                    format!("only players have `{name}`, but `entities(..)` may match other entities"),
                )
                .with_help("read it in `for player in players()`, or as `Player.{name}` in a player function"),
            );
            return None;
        }
        Some(property)
    }

    /// `Player.max_health = 40.0;`, `zombie.scale *= 2;`, and the other
    /// assignments to a handle's attribute.
    pub(super) fn lower_property_assign(
        &mut self,
        body: &mut BodyCx,
        place: &Place,
        op: AssignOp,
        value: &Expr,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let Some(property) = self.entity_property(body, place) else {
            return;
        };
        let PropertyKind::Attribute(attribute) = property.kind.clone() else {
            let help = match property.name.as_str() {
                "health" => {
                    "change health with `damage(...)` or a healing effect such as `add_effect(Effect::InstantHealth, ...)`"
                }
                "food" | "saturation" => {
                    "feed the player with an effect such as `add_effect(Effect::Saturation, ...)`"
                }
                "xp_level" | "xp_total" | "xp_progress" => {
                    "give experience with `add_experience(...)`"
                }
                "x" | "y" | "z" => "move it with `teleport(...)` or `teleport_by(...)`",
                _ => "entity data can be read but not written",
            };
            self.diagnostics.push(
                Diagnostic::error(
                    "assign",
                    file,
                    place.field.span,
                    format!("`{}.{}` is read-only", place.state.display(), property.name),
                )
                .with_help(help),
            );
            return;
        };
        let compound = match op {
            AssignOp::Set => None,
            AssignOp::Add => Some(BinOp::Add),
            AssignOp::Sub => Some(BinOp::Sub),
            AssignOp::Mul => Some(BinOp::Mul),
            AssignOp::Div => Some(BinOp::Div),
            AssignOp::Rem => Some(BinOp::Rem),
        };
        let amount = match compound {
            None if Self::is_const_in(body, value) => {
                let Some(v) = self.eval_i32(body.module, file, value, Some(Ty::Float), None) else {
                    return;
                };
                pc::Amount::Value(v as f64 / FLOAT_SCALE as f64)
            }
            None => {
                let depth = body.temp_depth;
                let temp = self.temp(body);
                if self.emit(body, &temp, Ty::Float, value, ops).is_none() {
                    return;
                }
                body.temp_depth = depth;
                pc::Amount::Field(pc::ScaledField {
                    score: temp,
                    scale: FLOAT_SCALE as f64,
                })
            }
            Some(bin) => {
                // Read the current base value, combine, and write it back.
                let depth = body.temp_depth;
                let temp = self.temp(body);
                ops.push(op_at(
                    pc::Action::AttributeGet {
                        score: temp.clone(),
                        attribute: attribute.clone(),
                        scale: FLOAT_SCALE as f64,
                    },
                    file,
                    place.span,
                ));
                let operand = match self.simple(body, value, Ty::Float) {
                    Ok(Some(operand)) => operand,
                    Ok(None) => {
                        let rhs = self.temp(body);
                        if self.emit(body, &rhs, Ty::Float, value, ops).is_none() {
                            return;
                        }
                        Value::Read(rhs)
                    }
                    Err(()) => return,
                };
                if self
                    .apply(file, &temp, Ty::Float, bin, operand, value.span(), ops)
                    .is_none()
                {
                    return;
                }
                body.temp_depth = depth;
                pc::Amount::Field(pc::ScaledField {
                    score: temp,
                    scale: FLOAT_SCALE as f64,
                })
            }
        };
        ops.push(op_at(
            pc::Action::AttributeSet {
                attribute,
                value: amount,
            },
            file,
            span,
        ));
    }

    fn builtin_action(
        &mut self,
        body: &mut BodyCx,
        builtin: &'static Builtin,
        bound: &BTreeMap<&'static str, &Arg>,
        span: Span,
    ) -> Option<pc::Action> {
        let file = body.file;
        Some(match builtin.name {
            "equip" | "clear_equipment" => {
                let slot = self.arg_equipment_slot(body, bound["slot"])?;
                let item = match bound.get("item") {
                    Some(arg) => Some(self.arg_id(body, arg, "Item")?),
                    None => None,
                };
                let count = match bound.get("count") {
                    Some(arg) => Some(self.arg_int(body, arg, 1, 99)? as u32),
                    None => None,
                };
                pc::Action::SetEquipment { slot, item, count }
            }
            "spawn_particle" => {
                let particle = self.arg_id(body, bound["particle"], "Particle");
                let count = match bound.get("count") {
                    Some(arg) => self.arg_int(body, arg, 0, 100_000),
                    None => Some(1),
                };
                let spread = match bound.get("spread") {
                    Some(arg) => self.arg_number(body, arg, 0.0, 1000.0),
                    None => Some(0.0),
                };
                let speed = match bound.get("speed") {
                    Some(arg) => self.arg_number(body, arg, 0.0, 1000.0),
                    None => Some(0.0),
                };
                pc::Action::Particle {
                    particle: particle?,
                    count: count? as u32,
                    spread: spread?,
                    speed: speed?,
                }
            }
            "add_effect" => {
                let effect = self.arg_id(body, bound["effect"], "Effect");
                let seconds = match bound.get("seconds") {
                    Some(arg) => self.arg_int(body, arg, 1, 1_000_000).map(Some),
                    None => Some(None),
                };
                let level = match bound.get("level") {
                    Some(arg) => self.arg_int(body, arg, 1, 256),
                    None => Some(1),
                };
                let hide = match bound.get("hide_particles") {
                    Some(arg) => self.arg_bool(file, arg),
                    None => Some(false),
                };
                pc::Action::Effect {
                    effect: effect?,
                    seconds: seconds?.map(|s| s as u32),
                    amplifier: (level? - 1) as u8,
                    hide_particles: hide?,
                }
            }
            "clear_effect" => {
                let effect = match bound.get("effect") {
                    Some(arg) => Some(self.arg_id(body, arg, "Effect")?),
                    None => None,
                };
                pc::Action::ClearEffect { effect }
            }
            "clear_item" => {
                let item = self.arg_id(body, bound["item"], "Item");
                let count = match bound.get("count") {
                    Some(arg) => self.arg_int(body, arg, 0, i32::MAX as i64).map(Some),
                    None => Some(None),
                };
                pc::Action::ClearItem {
                    item: Some(item?),
                    count: count?.map(|c| c as u32),
                }
            }
            "clear_inventory" => pc::Action::ClearItem {
                item: None,
                count: None,
            },
            "schedule" => {
                let target = match &bound["function"].value {
                    ArgValue::Expr(Expr::Path(target)) => self.expect_path(
                        body.module,
                        target,
                        &[DefKind::Function, DefKind::ExternFunction],
                        "a function",
                    ),
                    other => {
                        self.error("type", file, other.span(), "expected a function name");
                        None
                    }
                };
                let delay = match (bound.get("ticks"), bound.get("seconds")) {
                    (Some(arg), None) => self.arg_int(body, arg, 1, i32::MAX as i64),
                    (None, Some(arg)) => self
                        .arg_int(body, arg, 1, i32::MAX as i64 / 20)
                        .map(|s| s * 20),
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "call",
                                file,
                                span,
                                "`schedule` needs exactly one of `ticks` or `seconds`",
                            )
                            .with_help("for example `schedule(reset, seconds = 5);`"),
                        );
                        None
                    }
                };
                let target = target?;
                if self
                    .fn_slots
                    .get(&target)
                    .is_some_and(|s| !s.params.is_empty())
                {
                    self.error(
                        "params",
                        file,
                        bound["function"].value.span(),
                        "scheduled functions run later without arguments; remove the parameters",
                    );
                    return None;
                }
                let function = self.function_reference(target)?;
                body.schedules
                    .push((target, bound["function"].value.span()));
                pc::Action::Schedule {
                    function,
                    ticks: delay? as u32,
                }
            }
            "attach_state" | "detach_state" => {
                let (def, id) = self.arg_attachable_state(body, bound["state"])?;
                if !self.check_state_write(body, def, bound["state"].span) {
                    return None;
                }
                let state: sand::ResourceLocation = id.parse().ok()?;
                if builtin.name == "attach_state" {
                    pc::Action::AttachState { state }
                } else {
                    pc::Action::DetachState { state }
                }
            }
            "teleport_to" => pc::Action::Teleport {
                destination: pc::TeleportDestination::Entity {
                    query: self.query_from(body, bound)?,
                },
            },
            "kill" => pc::Action::Kill,
            "dismount" => pc::Action::Ride { vehicle: None },
            "cancel_waits" => pc::Action::CancelWaits,
            "add_tag" | "remove_tag" => {
                let (tag, _, tag_span) = self.arg_string(file, bound["tag"], "a tag name")?;
                if !valid_tag(&tag) {
                    self.error(
                        "id",
                        file,
                        tag_span,
                        format!("`{tag}` is not a valid tag; use letters, digits, and `_ . + -`"),
                    );
                    return None;
                }
                pc::Action::EntityTag {
                    tag,
                    add: builtin.name == "add_tag",
                }
            }
            "summon" => {
                let entity_type = self.arg_id(body, bound["entity"], "Entity");
                let tag = match bound.get("tag") {
                    Some(arg) => {
                        let (tag, _, tag_span) = self.arg_string(file, arg, "a tag name")?;
                        if !valid_tag(&tag) {
                            self.error(
                                "id",
                                file,
                                tag_span,
                                format!(
                                    "`{tag}` is not a valid tag; use letters, digits, and `_ . + -`"
                                ),
                            );
                            return None;
                        }
                        Some(tag)
                    }
                    None => None,
                };
                pc::Action::Summon {
                    entity_type: entity_type?,
                    tag,
                }
            }
            "stop_timer" => pc::Action::ScoreSet {
                score: self.arg_timer(body, bound["timer"], "stops")?,
                value: 0,
            },
            "set_block" => {
                let block = self.arg_id(body, bound["block"], "Block");
                let at = self.arg_position(body, bound["at"]);
                pc::Action::SetBlock {
                    block: block?,
                    at: at?,
                }
            }
            "copy_block_data" => pc::Action::WriteBlockData {
                target: self.arg_block_data(body, bound["to"], bound["destination_path"])?,
                value: Some(pc::DataOperand::Block {
                    value: self.arg_block_data(body, bound["from"], bound["source_path"])?,
                }),
            },
            "remove_block_data" => pc::Action::WriteBlockData {
                target: self.arg_block_data(body, bound["at"], bound["path"])?,
                value: None,
            },
            "clone_blocks" => {
                let masked = match bound.get("masked") {
                    Some(arg) => self.arg_bool(file, arg)?,
                    None => false,
                };
                let filter = match bound.get("filter") {
                    Some(arg) => Some(self.arg_block_test(body, arg)?),
                    None => None,
                };
                if masked && filter.is_some() {
                    self.error(
                        "call",
                        file,
                        span,
                        "choose either a clone filter or a masked clone",
                    );
                    return None;
                }
                let mode = match bound.get("mode") {
                    Some(arg) => {
                        let value = self.arg_id(body, arg, "CloneMode")?;
                        match serde_json::from_value(serde_json::Value::String(value)) {
                            Ok(mode) => mode,
                            Err(_) => {
                                self.error(
                                    "type",
                                    file,
                                    arg.span,
                                    "expected a `CloneMode::...` value",
                                );
                                return None;
                            }
                        }
                    }
                    None => sand::command::CloneMode::Normal,
                };
                pc::Action::CloneBlocks {
                    from: self.arg_position(body, bound["from"])?,
                    to: self.arg_position(body, bound["to"])?,
                    destination: self.arg_position(body, bound["destination"])?,
                    masked,
                    filter,
                    mode,
                }
            }
            "fill_blocks" => {
                let block = self.arg_id(body, bound["block"], "Block");
                let from = self.arg_position(body, bound["from"]);
                let to = self.arg_position(body, bound["to"]);
                pc::Action::FillBlocks {
                    block: block?,
                    from: from?,
                    to: to?,
                }
            }
            "set_gamemode" => pc::Action::SetGameMode {
                mode: match self.arg_id(body, bound["mode"], "GameMode")?.as_str() {
                    "survival" => pc::PlayerGameMode::Survival,
                    "creative" => pc::PlayerGameMode::Creative,
                    "adventure" => pc::PlayerGameMode::Adventure,
                    _ => pc::PlayerGameMode::Spectator,
                },
            },
            "set_time" => pc::Action::SetTime {
                ticks: self.arg_int(body, bound["ticks"], 0, i32::MAX as i64)? as u32,
            },
            "set_weather" => {
                let weather = self.arg_id(body, bound["weather"], "Weather");
                let seconds = match bound.get("seconds") {
                    Some(arg) => self.arg_int(body, arg, 1, 1_000_000).map(Some),
                    None => Some(None),
                };
                pc::Action::SetWeather {
                    weather: match weather?.as_str() {
                        "clear" => pc::Weather::Clear,
                        "rain" => pc::Weather::Rain,
                        _ => pc::Weather::Thunder,
                    },
                    seconds: seconds?.map(|s| s as u32),
                }
            }
            "show_bossbar" | "hide_bossbar" | "show_bossbar_all" | "hide_bossbar_all" => {
                pc::Action::BossBarAudience {
                    bar: self.arg_declared(body, bound["bar"], DefKind::BossBar, "a boss bar")?,
                    audience: if builtin.name.ends_with("_all") {
                        pc::Audience::Everyone
                    } else {
                        pc::Audience::Player
                    },
                    visible: builtin.name.starts_with("show"),
                }
            }
            "give_loot" | "drop_loot" => {
                let table =
                    self.arg_declared(body, bound["table"], DefKind::Loot, "a loot table")?;
                if builtin.name == "give_loot" {
                    pc::Action::GiveLoot { table }
                } else {
                    pc::Action::DropLoot { table }
                }
            }
            other => unreachable!("statement builtin `{other}` is not lowered"),
        })
    }

    pub(super) fn lower_macro(
        &mut self,
        body: &mut BodyCx,
        name: &Ident,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let Some(builtin) = builtins::find(&name.name, BuiltinKind::Macro) else {
            let known = builtins::BUILTINS
                .iter()
                .filter(|b| b.kind == BuiltinKind::Macro)
                .map(|b| format!("`{}!`", b.name))
                .collect::<Vec<_>>()
                .join(", ");
            self.diagnostics.push(
                Diagnostic::error(
                    "macro",
                    file,
                    name.span,
                    format!("unknown macro `{}!`", name.name),
                )
                .with_help(format!("available macros: {known}")),
            );
            return;
        };
        self.references_builtin(file, name.span, builtin);
        let Some(bound) = self.bind_args(body, builtin, args, span) else {
            return;
        };
        if builtin.name == "assert" {
            let Some(owner) = body.owner else {
                self.error(
                    "test",
                    file,
                    span,
                    "`assert!` is only available inside a Sand test",
                );
                return;
            };
            if !self.test_functions.contains(&owner) {
                self.error(
                    "test",
                    file,
                    span,
                    "`assert!` is only available inside a Sand test",
                );
                return;
            }
            let ArgValue::Expr(condition) = &bound["condition"].value else {
                self.error("test", file, span, "`assert!` needs a boolean condition");
                return;
            };
            let depth = body.temp_depth;
            let score = self.temp(body);
            if self.emit(body, &score, Ty::Bool, condition, ops).is_some() {
                let mut fail = vec![op_at(
                    pc::Action::Raw {
                        command: format!("say __SAND_ASSERT_{}_{}__", file.0, span.start),
                    },
                    file,
                    span,
                )];
                self.lower_return(body, Some(&Expr::Bool(false, span)), span, &mut fail);
                body.raw = true;
                ops.push(op_at(
                    pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score,
                            min: Some(1),
                            max: Some(1),
                        }),
                        then: Vec::new(),
                        otherwise: fail,
                    },
                    file,
                    span,
                ));
            }
            body.temp_depth = depth;
            return;
        }
        if builtin.name == "raw" {
            let Some((command, _, string_span)) =
                self.arg_string(file, bound["command"], "a command")
            else {
                return;
            };
            if command.trim().is_empty() {
                self.error("raw", file, string_span, "raw command is empty");
                return;
            }
            body.raw = true;
            ops.push(op_at(pc::Action::Raw { command }, file, span));
            return;
        }
        if builtin.needs_player {
            body.note_player(
                name.span,
                format!(
                    "uses `{}!`, which shows text to the executing player",
                    builtin.name
                ),
            );
        }
        let Some(text) = self.presentation_text(body, bound["text"], Some(ops), 0) else {
            return;
        };
        let (audience, channel) = match builtin.name {
            "chat" => (pc::Audience::Player, pc::MessageChannel::Chat),
            "broadcast" => (pc::Audience::Everyone, pc::MessageChannel::Chat),
            "actionbar" => (pc::Audience::Player, pc::MessageChannel::Actionbar),
            "title" => (pc::Audience::Player, pc::MessageChannel::Title),
            _ => (pc::Audience::Player, pc::MessageChannel::Subtitle),
        };
        ops.push(op_at(
            pc::Action::Message {
                audience,
                channel,
                text,
            },
            file,
            span,
        ));
    }

    fn float_in_text(&mut self, file: FileId, span: Span) {
        self.diagnostics.push(
            Diagnostic::error("text", file, span, "text can only show whole numbers").with_help(
                "store a copy first: `let shown = value as number;`, then write `{shown}`",
            ),
        );
    }

    pub(super) fn references_builtin(
        &mut self,
        file: FileId,
        span: Span,
        builtin: &'static Builtin,
    ) {
        self.builtin_uses
            .push((file, span, builtin.name, builtin.kind));
    }

    /// Build a runtime message or a static resource's component text.
    pub(super) fn presentation_text(
        &mut self,
        body: &mut BodyCx,
        arg: &Arg,
        mut ops: Option<&mut Vec<pc::Operation>>,
        depth: usize,
    ) -> Option<Vec<pc::TextPart>> {
        if let ArgValue::Expr(Expr::Call { path, args, span }) = &arg.value
            && path.applications.is_empty()
            && path.segments.len() == 2
            && path.segments[0].name == "Text"
            && path.last().name == "translate"
        {
            if depth >= pc::TextPart::MAX_TRANSLATION_DEPTH {
                self.error(
                    "text",
                    body.file,
                    *span,
                    "text exceeds the translation nesting limit",
                );
                return None;
            }
            let Some((key, arguments)) = args.split_first() else {
                self.error(
                    "text",
                    body.file,
                    *span,
                    "`Text::translate` needs a quoted translation key",
                );
                return None;
            };
            if let Some(named) = args.iter().find_map(|arg| arg.name.as_ref()) {
                self.error(
                    "text",
                    body.file,
                    named.span,
                    "translation arguments are positional",
                );
                return None;
            }
            let (key, _, key_span) = self.arg_string(body.file, key, "a translation key")?;
            if key.trim().is_empty() || key.chars().any(char::is_control) {
                self.error(
                    "text",
                    body.file,
                    key_span,
                    "translation keys must be nonblank and contain no control characters",
                );
                return None;
            }
            let mut rendered = Vec::with_capacity(arguments.len());
            for argument in arguments {
                rendered.push(self.presentation_text(
                    body,
                    argument,
                    ops.as_deref_mut(),
                    depth + 1,
                )?);
            }
            return Some(vec![pc::TextPart::Translation {
                key,
                arguments: rendered,
                style: pc::TextStyle::default(),
            }]);
        }
        if let ArgValue::Expr(value) = &arg.value
            && !matches!(value, Expr::Str { .. })
            && let Some(ops) = ops
        {
            let slot = self.temp(body);
            self.emit(body, &slot, Ty::String, value, ops)?;
            return Some(vec![pc::TextPart::Stored {
                value: Self::data_field(&slot),
                style: pc::TextStyle::default(),
            }]);
        }
        self.format_text(body, arg)
    }

    /// Parse a format string into styled text parts, resolving `{State.field}`.
    pub(super) fn format_text(
        &mut self,
        body: &mut BodyCx,
        arg: &Arg,
    ) -> Option<Vec<pc::TextPart>> {
        let file = body.file;
        let (_, raw, string_span) = self.arg_string(file, arg, "text")?;
        let base = string_span.start + 1;
        let bytes = raw.as_bytes();
        let mut parts = Vec::new();
        let mut style = pc::TextStyle::default();
        let mut literal = String::new();
        let mut ok = true;
        let mut i = 0;
        let flush = |literal: &mut String, style: &pc::TextStyle, parts: &mut Vec<pc::TextPart>| {
            if !literal.is_empty() {
                parts.push(pc::TextPart::Text {
                    text: crate::parser::unescape(literal),
                    style: style.clone(),
                });
                literal.clear();
            }
        };
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => {
                    let start = i;
                    i += 1;
                    if bytes.get(i) == Some(&b'u') && bytes.get(i + 1) == Some(&b'{') {
                        i = raw[i..].find('}').map_or(raw.len(), |end| i + end + 1);
                    } else if let Some(character) = raw[i..].chars().next() {
                        i += character.len_utf8();
                    }
                    literal.push_str(&raw[start..i]);
                }
                b'{' if bytes.get(i + 1) == Some(&b'{') => {
                    literal.push('{');
                    i += 2;
                }
                b'}' if bytes.get(i + 1) == Some(&b'}') => {
                    literal.push('}');
                    i += 2;
                }
                b'}' => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "text",
                            file,
                            Span::new(base + i, base + i + 1),
                            "unmatched `}` in text",
                        )
                        .with_help("write `}}` for a literal brace"),
                    );
                    ok = false;
                    i += 1;
                }
                b'{' => {
                    let Some(close) = raw[i + 1..].find('}') else {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "text",
                                file,
                                Span::new(base + i, base + raw.len()),
                                "unclosed `{` in text",
                            )
                            .with_help("close it with `}`, or write `{{` for a literal brace"),
                        );
                        ok = false;
                        break;
                    };
                    let inner = &raw[i + 1..i + 1 + close];
                    let token = inner.trim();
                    let token_start = base + i + 1 + (inner.len() - inner.trim_start().len());
                    let token_span = Span::new(token_start, token_start + token.len());
                    flush(&mut literal, &style, &mut parts);
                    if !self.text_token(body, token, token_span, &mut style, &mut parts) {
                        ok = false;
                    }
                    i += close + 2;
                }
                _ => {
                    let ch = raw[i..].chars().next().expect("in bounds");
                    literal.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
        flush(&mut literal, &style, &mut parts);
        if ok && parts.is_empty() {
            self.error("text", file, string_span, "message text is empty");
            return None;
        }
        ok.then_some(parts)
    }

    fn text_token(
        &mut self,
        body: &mut BodyCx,
        token: &str,
        span: Span,
        style: &mut pc::TextStyle,
        parts: &mut Vec<pc::TextPart>,
    ) -> bool {
        let file = body.file;
        let color = |name: &str| {
            Some(match name {
                "black" => pc::TextColor::Black,
                "dark_blue" => pc::TextColor::DarkBlue,
                "dark_green" => pc::TextColor::DarkGreen,
                "dark_aqua" => pc::TextColor::DarkAqua,
                "dark_red" => pc::TextColor::DarkRed,
                "dark_purple" => pc::TextColor::DarkPurple,
                "gold" => pc::TextColor::Gold,
                "gray" => pc::TextColor::Gray,
                "dark_gray" => pc::TextColor::DarkGray,
                "blue" => pc::TextColor::Blue,
                "green" => pc::TextColor::Green,
                "aqua" => pc::TextColor::Aqua,
                "red" => pc::TextColor::Red,
                "light_purple" => pc::TextColor::LightPurple,
                "yellow" => pc::TextColor::Yellow,
                "white" => pc::TextColor::White,
                _ => return None,
            })
        };
        if let Some(local) = Self::lookup_local(body, token) {
            if self.local_ty(local) == Ty::Float {
                self.float_in_text(file, span);
                return false;
            }
            body.used.insert(local);
            self.references.push(Reference {
                file,
                span,
                def: local,
            });
            let score = self.local_ref(local);
            parts.push(if self.local_ty(local) == Ty::String {
                pc::TextPart::Stored {
                    value: Self::data_field(&score),
                    style: style.clone(),
                }
            } else {
                pc::TextPart::Score {
                    score,
                    style: style.clone(),
                }
            });
            return true;
        }
        match token {
            "player" => {
                body.note_player(span, "shows `{player}`, the executing player's name".into());
                parts.push(pc::TextPart::PlayerName {
                    style: style.clone(),
                });
                true
            }
            "bold" => {
                style.bold = true;
                true
            }
            "italic" => {
                style.italic = true;
                true
            }
            "underlined" => {
                style.underlined = true;
                true
            }
            "reset" => {
                *style = pc::TextStyle::default();
                true
            }
            name if color(name).is_some() => {
                style.color = color(name);
                true
            }
            path_text if path_text.contains('.') => {
                let (state_text, rest) = path_text.split_once('.').expect("contains a dot");
                let mut offset = span.start;
                let segments: Vec<_> = state_text
                    .split("::")
                    .map(|name| {
                        let ident = Ident {
                            name: name.into(),
                            span: Span::new(offset, offset + name.len()),
                        };
                        offset += name.len() + 2;
                        ident
                    })
                    .collect();
                offset = span.start + state_text.len() + 1;
                let mut projections: Vec<_> = rest
                    .split('.')
                    .map(|name| {
                        let ident = Ident {
                            name: name.into(),
                            span: Span::new(offset, offset + name.len()),
                        };
                        offset += name.len() + 1;
                        ident
                    })
                    .collect();
                if !segments.iter().chain(&projections).all(|part| {
                    part.name.chars().next().is_some_and(|character| {
                        character.is_ascii_alphabetic() || character == '_'
                    }) && part
                        .name
                        .bytes()
                        .all(|character| character.is_ascii_alphanumeric() || character == b'_')
                }) {
                    self.error(
                        "text",
                        file,
                        span,
                        "expected a local or State field path in text",
                    );
                    return false;
                }
                let field = projections.remove(0);
                let place = Place {
                    state: Path {
                        applications: Vec::new(),
                        segments,
                        span: Span::new(span.start, span.start + state_text.len()),
                    },
                    field,
                    projections,
                    span,
                };
                let Some(value) = self.place_value(body, &place, false) else {
                    return false;
                };
                if value.ty == Ty::Float {
                    self.float_in_text(file, span);
                    return false;
                }
                if matches!(value.ty, Ty::Struct(_) | Ty::EntityRef) {
                    self.error(
                        "text",
                        file,
                        span,
                        "text interpolation needs a scalar or string value",
                    );
                    return false;
                }
                parts.push(if value.ty == Ty::String {
                    pc::TextPart::Stored {
                        value: Self::data_field(&value.reference),
                        style: style.clone(),
                    }
                } else {
                    pc::TextPart::Score {
                        score: value.reference,
                        style: style.clone(),
                    }
                });
                true
            }
            other => {
                self.diagnostics.push(
                    Diagnostic::error("text", file, span, format!("unknown `{{{other}}}` in text"))
                        .with_help(
                            "use a variable such as `{price}`, `{State.field}` for a live score, `{player}` for the player's name, or a style such as `{gold}`, `{bold}`, `{reset}`",
                        ),
                );
                false
            }
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.+-".contains(&b))
}

fn param_list(builtin: &Builtin) -> String {
    if builtin.params.is_empty() {
        return "no arguments".into();
    }
    builtin
        .params
        .iter()
        .map(|p| format!("`{}`", p.name))
        .collect::<Vec<_>>()
        .join(", ")
}
