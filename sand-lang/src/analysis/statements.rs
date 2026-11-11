//! Lowering for assignments, builtin statements, and text macros.

use super::*;
use crate::builtins::{self, Builtin, BuiltinKind};

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
    /// Source text of the target (`Hero.gold` or `x`) for hazard checks.
    key: String,
    name: String,
}

fn place_key(place: &Place) -> String {
    format!("{}.{}", place.state.display(), place.field.name)
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

    fn lookup_local(body: &BodyCx, name: &str) -> Option<DefId> {
        body.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|(n, _)| n == name).map(|(_, d)| *d))
    }

    fn local_of(body: &BodyCx, expr: &Expr) -> Option<DefId> {
        match expr {
            Expr::Path(path) if path.segments.len() == 1 => {
                Self::lookup_local(body, &path.segments[0].name)
            }
            _ => None,
        }
    }

    fn local_ty(&self, def: DefId) -> Ty {
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
            Expr::Place(_) | Expr::Call { .. } | Expr::Cond(..) | Expr::Str { .. } => false,
            Expr::Float(..) | Expr::Bool(..) => true,
        }
    }

    /// The type an expression produces, without reporting errors.
    pub(super) fn type_of(&mut self, body: &BodyCx, expr: &Expr) -> Ty {
        match expr {
            Expr::Path(path) => {
                if let Some(local) = Self::local_of(body, expr) {
                    return self.local_ty(local);
                }
                match self.resolve_path(body.module, path) {
                    Ok((def, _)) => match self.defs[def.0].kind {
                        DefKind::Variant => {
                            self.defs[def.0].parent.map(Ty::Enum).unwrap_or(Ty::Number)
                        }
                        DefKind::Const => {
                            self.const_src.get(&def).map(|s| s.ty).unwrap_or(Ty::Number)
                        }
                        _ => Ty::Number,
                    },
                    Err(_) => Ty::Unknown,
                }
            }
            Expr::Place(place) => {
                let Ok((state, _)) = self.resolve_path(body.module, &place.state) else {
                    return Ty::Unknown;
                };
                self.members
                    .get(&state)
                    .and_then(|m| m.get(&place.field.name))
                    .map(|field| match self.defs[field.0].info {
                        DefInfo::Field { ty, .. } => ty,
                        _ => Ty::Unknown,
                    })
                    .unwrap_or(Ty::Unknown)
            }
            Expr::Float(..) => Ty::Float,
            Expr::Bool(..) | Expr::Cond(..) => Ty::Bool,
            Expr::Str { .. } => Ty::Unknown,
            Expr::Neg(inner, _) => self.type_of(body, inner),
            Expr::Cast(_, ty, _) => match ty.segments.first().map(|s| s.name.as_str()) {
                Some("float") => Ty::Float,
                _ => Ty::Number,
            },
            Expr::Binary(lhs, _, rhs, _) => {
                let (left, right) = (self.type_of(body, lhs), self.type_of(body, rhs));
                if left == Ty::Float || right == Ty::Float {
                    Ty::Float
                } else {
                    left
                }
            }
            Expr::Call { path, .. } => match self.resolve_path(body.module, path) {
                Ok((def, _)) => self
                    .fn_slots
                    .get(&def)
                    .and_then(|slots| slots.ret.as_ref().map(|(_, ty)| *ty))
                    .unwrap_or(Ty::Number),
                Err(_) => Ty::Number,
            },
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
            let (state, field, field_ty) = self.resolve_place(module, file, place).ok_or(())?;
            body.note_state(self, state, place, "reads");
            if !self.types_agree(file, place.span, ty, field_ty) {
                return Err(());
            }
            return Ok(Some(Value::Read(
                self.field_reference(state, field).ok_or(())?,
            )));
        }
        Ok(None)
    }

    fn types_agree(&mut self, file: FileId, span: Span, expected: Ty, found: Ty) -> bool {
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

    fn set_from(dest: &pc::FieldReference, value: Value) -> Option<pc::Action> {
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
        let file = body.file;
        let span = expr.span();
        if let Some(value) = self.simple(body, expr, ty).ok()? {
            if let Some(action) = Self::set_from(dest, value) {
                ops.push(op_at(action, file, span));
            }
            return Some(());
        }
        match expr {
            Expr::Call { path, args, span } => {
                self.emit_call(body, dest, ty, path, args, *span, ops)
            }
            Expr::Cast(inner, cast_ty, cast_span) => {
                let target = cast_ty.segments.first().map(|s| s.name.as_str());
                if !matches!(target, Some("number" | "float")) || cast_ty.segments.len() != 1 {
                    self.error(
                        "type",
                        file,
                        *cast_span,
                        "only `as number` and `as float` casts are supported",
                    );
                    return None;
                }
                let from = self.type_of(body, inner);
                self.emit(body, dest, from, inner, ops)?;
                let step = match (from, target) {
                    (Ty::Float, Some("number")) => Some(BinOp::Div),
                    (Ty::Number, Some("float")) => Some(BinOp::Mul),
                    (Ty::Bool | Ty::Enum(_), Some("float")) => {
                        self.error(
                            "type",
                            file,
                            *cast_span,
                            "only `number` values can become `float`",
                        );
                        return None;
                    }
                    _ => None,
                };
                if let Some(step) = step {
                    let action =
                        self.arithmetic(file, dest, step, Value::Const(FLOAT_SCALE), *cast_span)?;
                    ops.push(op_at(action, file, *cast_span));
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
    fn apply(
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
        if matches!(ty, Ty::Enum(_) | Ty::Bool) {
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

    /// A call used as a value: `random(..)`, `get_attribute(..)`, or a function
    /// with a return type.
    #[allow(clippy::too_many_arguments)]
    fn emit_call(
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
        let name = &path.last().name;
        let builtin = (path.segments.len() == 1
            && !self.scopes[module.0].contains_key(name)
            && Self::lookup_local(body, name).is_none())
        .then(|| builtins::find(name, BuiltinKind::Expression))
        .flatten();
        if let Some(builtin) = builtin {
            self.references_builtin(file, path.span, builtin);
            self.expect_numeric(file, span, ty)?;
            if builtin.name != "get_attribute" && !self.types_agree(file, span, ty, Ty::Number) {
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
                "get_attribute" => {
                    body.note_player(
                        path.span,
                        "reads an attribute of the executing player".into(),
                    );
                    let attribute = self.arg_id(body, bound["attribute"], "Attribute")?;
                    let scale = match bound.get("scale") {
                        Some(arg) => self.arg_number(body, arg, 0.0001, 1_000_000.0)?,
                        None => 1.0,
                    };
                    // A `float` destination keeps three decimals.
                    let scale = f64::from(scale)
                        * if ty == Ty::Float {
                            FLOAT_SCALE as f64
                        } else {
                            1.0
                        };
                    pc::Action::AttributeGet {
                        score: dest.clone(),
                        attribute,
                        scale,
                    }
                }
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
        let def = self.expect_path(
            module,
            path,
            &[DefKind::Function],
            "a function that returns a value",
        )?;
        let slots = self.fn_slots.get(&def).cloned()?;
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
                .with_help(format!("declare one: `fn {dname}(...) -> i32`")),
            );
            return None;
        };
        if !self.types_agree(file, span, ty, ret_ty) {
            return None;
        }
        self.call_user(body, def, &slots, path, args, span, ops)?;
        let ret = self.slot(ret_field);
        if let Some(action) = Self::set_from(dest, Value::Read(ret)) {
            ops.push(op_at(action, file, span));
        }
        Some(())
    }

    /// Pass arguments into the callee's parameter slots, then call it.
    #[allow(clippy::too_many_arguments)]
    fn call_user(
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
        for ((field, _, param_ty), arg) in slots.params.iter().zip(args) {
            let ArgValue::Expr(expr) = &arg.value else {
                self.error("type", file, arg.span, "expected a value, found a range");
                return None;
            };
            let slot = self.slot(field.clone());
            self.emit(body, &slot, *param_ty, expr, ops)?;
        }
        body.all_calls.push((def, path.span));
        if !body.in_players {
            body.calls.push((def, path.span));
        }
        if body.loops.last() == Some(&super::LoopKind::Entities) {
            body.entity_loop_calls.push((def, path.span));
        }
        let function = self.function_reference(def)?;
        ops.push(op_at(pc::Action::Call { function }, file, span));
        Some(())
    }

    fn resolve_target(&mut self, body: &mut BodyCx, target: &AssignTarget) -> Option<Dest> {
        let (module, file) = (body.module, body.file);
        match target {
            AssignTarget::Place(place) => {
                let (state, field, ty) = self.resolve_place(module, file, place)?;
                body.note_state(self, state, place, "writes");
                let bounds = match self.defs[field.0].info {
                    DefInfo::Field { bounds, .. } => bounds,
                    _ => None,
                };
                Some(Dest {
                    reference: self.field_reference(state, field)?,
                    ty,
                    bounds,
                    key: place_key(place),
                    name: place.field.name.clone(),
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
                        format!(
                            "copy it into a mutable local: `let mut {0} = {0};`",
                            ident.name
                        )
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
                    key: ident.name.clone(),
                    name: ident.name.clone(),
                })
            }
        }
    }

    /// Whether `expr` reads `key` anywhere except as its leftmost operand.
    fn reads_target(expr: &Expr, key: &str, leftmost: bool) -> bool {
        match expr {
            Expr::Place(place) => !leftmost && place_key(place) == key,
            Expr::Path(path) => {
                !leftmost && path.segments.len() == 1 && path.segments[0].name == key
            }
            Expr::Binary(lhs, _, rhs, _) => {
                Self::reads_target(lhs, key, leftmost) || Self::reads_target(rhs, key, false)
            }
            Expr::Neg(inner, _) | Expr::Cast(inner, _, _) => {
                Self::reads_target(inner, key, leftmost)
            }
            Expr::Call { args, .. } => args.iter().any(|arg| match &arg.value {
                ArgValue::Expr(e) => Self::reads_target(e, key, false),
                ArgValue::Range(_) => false,
            }),
            _ => false,
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
        if let Some(bin) = compound {
            if matches!(dest.ty, Ty::Enum(_) | Ty::Bool) {
                self.error(
                    "type",
                    file,
                    span,
                    "`bool` and enum values can only be assigned with `=`",
                );
                return;
            }
            let value_ = match self.simple(body, value, dest.ty) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    let depth = body.temp_depth;
                    let temp = self.temp(body);
                    if self.emit(body, &temp, dest.ty, value, ops).is_none() {
                        return;
                    }
                    body.temp_depth = depth;
                    Value::Read(temp)
                }
                Err(()) => return,
            };
            let _ = self.apply(
                file,
                &dest.reference,
                dest.ty,
                bin,
                value_,
                value.span(),
                ops,
            );
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
        // Compute through a temporary when writing in place would be wrong:
        // a clamped target would clamp every intermediate step, and reading
        // the target after overwriting it would observe partial results.
        let multi_step = matches!(value, Expr::Binary(..) | Expr::Neg(..));
        let via_temp =
            (dest.bounds.is_some() && multi_step) || Self::reads_target(value, &dest.key, true);
        if via_temp {
            let depth = body.temp_depth;
            let temp = self.temp(body);
            if self.emit(body, &temp, dest.ty, value, ops).is_none() {
                return;
            }
            body.temp_depth = depth;
            if let Some(action) = Self::set_from(&dest.reference, Value::Read(temp)) {
                ops.push(op_at(action, file, span));
            }
        } else {
            let _ = self.emit(body, &dest.reference, dest.ty, value, ops);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn lower_let(
        &mut self,
        body: &mut BodyCx,
        mutable: bool,
        name: &Ident,
        ty: Option<&Path>,
        value: Option<&Expr>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (module, file) = (body.module, body.file);
        let ty = match (ty, value) {
            (Some(ty), _) => self.resolve_type(module, ty),
            (None, Some(value)) => self.type_of(body, value),
            (None, None) => Ty::Number,
        };
        let local = self.push_def(Def {
            kind: DefKind::Local,
            name: name.name.clone(),
            module,
            file,
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
        });
        let reference = self.local_ref(local);
        match value {
            Some(value) => {
                let _ = self.emit(body, &reference, ty, value, ops);
            }
            None => ops.push(op_at(
                pc::Action::ScoreSet {
                    score: reference,
                    value: 0,
                },
                file,
                span,
            )),
        }
        // Visible from after the statement to the end of the block.
        let end = body.block_ends.last().copied().unwrap_or(span.end);
        self.local_scopes
            .push((file, Span::new(span.end, end), local));
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
                if self.emit(body, slot, *ty, value, ops).is_none() {
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
                    .with_help("declare a return type: `fn name() -> i32`"),
                );
                return;
            }
            (None, None) => {}
        }
        ops.push(op_at(pc::Action::Return, file, span));
    }

    pub(super) fn lower_while(
        &mut self,
        body: &mut BodyCx,
        cond: &Cond,
        block: &Block,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        if !matches!(
            cond,
            Cond::Compare { .. } | Cond::InRange { .. } | Cond::Call { .. }
        ) {
            self.diagnostics.push(
                Diagnostic::error(
                    "while",
                    file,
                    cond.span(),
                    "a `while` condition must be a single comparison",
                )
                .with_help("compute the condition into a `let mut` variable, e.g. `while keep_going == 1 { ... }`"),
            );
            return;
        }
        let mut pre = Vec::new();
        let Some((condition, negated)) = self.atom(body, cond, &mut pre) else {
            return;
        };
        if negated || !pre.is_empty() {
            self.diagnostics.push(
                Diagnostic::error(
                    "while",
                    file,
                    cond.span(),
                    "a `while` condition must compare fields, variables, or constants without `!=` or calculations",
                )
                .with_help("compute it into a `let mut` variable before the loop and update it inside"),
            );
            return;
        }
        let inner = self.lower_block(body, block);
        ops.push(op_at(
            pc::Action::While {
                condition,
                body: inner,
            },
            file,
            span,
        ));
    }

    /// Bind call arguments to a builtin's parameters by position and name.
    fn bind_args<'a>(
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

    fn arg_int(&mut self, body: &BodyCx, arg: &Arg, min: i64, max: i64) -> Option<i64> {
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
        let name = &path.last().name;
        let is_builtin_name = path.segments.len() == 1 && !self.scopes[module.0].contains_key(name);
        if is_builtin_name && let Some(builtin) = builtins::find(name, BuiltinKind::Statement) {
            if builtin.name == "emit" {
                self.lower_emit(body, builtin, path, args, span, ops);
                return;
            }
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
        let Some(def) = self.expect_path(
            module,
            path,
            &[DefKind::Function, DefKind::ExternFunction],
            "a function",
        ) else {
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
    fn lower_emit(
        &mut self,
        body: &mut BodyCx,
        builtin: &'static Builtin,
        path: &Path,
        args: &[Arg],
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (module, file) = (body.module, body.file);
        self.references_builtin(file, path.span, builtin);
        let Some(Arg {
            name: None,
            value: ArgValue::Expr(Expr::Path(event)),
            ..
        }) = args.first()
        else {
            self.diagnostics.push(
                Diagnostic::error("call", file, span, "`emit` needs an event first")
                    .with_help("declare `event Name;` and write `emit(Name);`"),
            );
            return;
        };
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
        let _ = self.call_user(body, def, &slots, event, &args[1..], span, ops);
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
        if builtin.name == "set_attribute" {
            self.lower_set_attribute(body, &bound, span, ops);
            return;
        }
        if matches!(builtin.name, "set_bossbar" | "set_bossbar_max") {
            self.lower_set_bossbar(body, builtin, &bound, span, ops);
            return;
        }
        if let Some(action) = self.builtin_action(body, builtin, &bound, span) {
            ops.push(op_at(action, file, span));
        }
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
            "holding" => {
                let holding = self.arg_id(body, bound["item"], "Item");
                let offhand = match bound.get("offhand") {
                    Some(arg) => self.arg_bool(file, arg),
                    None => Some(false),
                };
                Some(pc::Condition::Holding(pc::ItemHeld {
                    holding: holding?,
                    offhand: offhand?,
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
        ok.then_some(query)
    }

    fn lower_set_attribute(
        &mut self,
        body: &mut BodyCx,
        bound: &BTreeMap<&'static str, &Arg>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let Some(attribute) = self.arg_id(body, bound["attribute"], "Attribute") else {
            return;
        };
        let scale = match bound.get("scale") {
            Some(arg) => match self.arg_number(body, arg, 0.0001, 1_000_000.0) {
                Some(scale) => f64::from(scale),
                None => return,
            },
            None => 1.0,
        };
        let ArgValue::Expr(expr) = &bound["value"].value else {
            self.error("type", file, bound["value"].span, "expected a value");
            return;
        };
        let value = match expr {
            Expr::Float(value, _) => pc::Amount::Value(*value / scale),
            Expr::Neg(inner, _) if matches!(**inner, Expr::Float(..)) => {
                let Expr::Float(value, _) = **inner else {
                    unreachable!()
                };
                pc::Amount::Value(-value / scale)
            }
            expr if Self::is_const_in(body, expr) => {
                let ty = self.type_of(body, expr);
                let Some(v) = self.eval_i32(body.module, file, expr, Some(ty), None) else {
                    return;
                };
                let divisor = if ty == Ty::Float {
                    FLOAT_SCALE as f64 * scale
                } else {
                    scale
                };
                pc::Amount::Value(v as f64 / divisor)
            }
            expr => {
                let ty = self.type_of(body, expr);
                let reference = match self.simple(body, expr, ty) {
                    Ok(Some(Value::Read(reference))) => reference,
                    Ok(_) => {
                        let depth = body.temp_depth;
                        let temp = self.temp(body);
                        if self.emit(body, &temp, ty, expr, ops).is_none() {
                            return;
                        }
                        body.temp_depth = depth;
                        temp
                    }
                    Err(()) => return,
                };
                pc::Amount::Field(pc::ScaledField {
                    score: reference,
                    scale: if ty == Ty::Float {
                        FLOAT_SCALE as f64 * scale
                    } else {
                        scale
                    },
                })
            }
        };
        ops.push(op_at(
            pc::Action::AttributeSet { attribute, value },
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
            "play_sound" => {
                let sound = self.arg_id(body, bound["sound"], "Sound")?;
                let volume = match bound.get("volume") {
                    Some(arg) => self.arg_number(body, arg, 0.0, 1000.0),
                    None => Some(1.0),
                };
                let pitch = match bound.get("pitch") {
                    Some(arg) => self.arg_number(body, arg, 0.0, 2.0),
                    None => Some(1.0),
                };
                pc::Action::Sound {
                    sound,
                    volume: volume?,
                    pitch: pitch?,
                }
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
            "give_item" => {
                let item = self.arg_id(body, bound["item"], "Item");
                let count = match bound.get("count") {
                    Some(arg) => self.arg_int(body, arg, 1, 6400),
                    None => Some(1),
                };
                pc::Action::GiveItem {
                    item: item?,
                    count: count? as u32,
                }
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
                let (_, id) = self.arg_attachable_state(body, bound["state"])?;
                let state: sand::ResourceLocation = id.parse().ok()?;
                if builtin.name == "attach_state" {
                    pc::Action::AttachState { state }
                } else {
                    pc::Action::DetachState { state }
                }
            }
            "teleport" | "teleport_by" => {
                let (a, b, c) = if builtin.name == "teleport" {
                    ("x", "y", "z")
                } else {
                    ("dx", "dy", "dz")
                };
                let (x, y, z) = (
                    self.arg_coordinate(body, bound[a]),
                    self.arg_coordinate(body, bound[b]),
                    self.arg_coordinate(body, bound[c]),
                );
                let (x, y, z) = (x?, y?, z?);
                pc::Action::Teleport {
                    destination: if builtin.name == "teleport" {
                        pc::TeleportDestination::Position { x, y, z }
                    } else {
                        pc::TeleportDestination::Offset { x, y, z }
                    },
                }
            }
            "teleport_to" => pc::Action::Teleport {
                destination: pc::TeleportDestination::Entity {
                    query: self.query_from(body, bound)?,
                },
            },
            "kill" => pc::Action::Kill,
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
            "damage" => pc::Action::Damage {
                amount: self.arg_number(body, bound["amount"], 0.01, 1_000_000.0)?,
            },
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
            "start_timer" => {
                let score = self.arg_timer(body, bound["timer"], "starts");
                let ticks = match (bound.get("ticks"), bound.get("seconds")) {
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
                                "`start_timer` needs exactly one of `ticks` or `seconds`",
                            )
                            .with_help("for example `start_timer(Ability.dash, seconds = 5);`"),
                        );
                        None
                    }
                };
                pc::Action::ScoreSet {
                    score: score?,
                    value: ticks? as i32,
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
            "add_experience" => {
                let (amount, levels) = match (bound.get("points"), bound.get("levels")) {
                    (Some(arg), None) => (arg, false),
                    (None, Some(arg)) => (arg, true),
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "call",
                                file,
                                span,
                                "`add_experience` needs exactly one of `points` or `levels`",
                            )
                            .with_help("for example `add_experience(points = 10);`"),
                        );
                        return None;
                    }
                };
                pc::Action::AddExperience {
                    amount: self.arg_int(body, amount, i32::MIN as i64, i32::MAX as i64)? as i32,
                    levels,
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
        let Some(text) = self.format_text(body, bound["text"]) else {
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

    fn references_builtin(&mut self, file: FileId, span: Span, builtin: &'static Builtin) {
        self.builtin_uses
            .push((file, span, builtin.name, builtin.kind));
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
            parts.push(pc::TextPart::Score {
                score,
                style: style.clone(),
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
                let (state_text, field_text) = path_text.rsplit_once('.').expect("contains a dot");
                let mut segments = Vec::new();
                let mut offset = span.start;
                for segment in state_text.split("::") {
                    segments.push(Ident {
                        name: segment.to_string(),
                        span: Span::new(offset, offset + segment.len()),
                    });
                    offset += segment.len() + 2;
                }
                let field_start = span.start + state_text.len() + 1;
                let valid = segments
                    .iter()
                    .chain(std::iter::once(&Ident {
                        name: field_text.to_string(),
                        span: Span::default(),
                    }))
                    .all(|s| {
                        s.name
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                            && s.name
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    });
                if !valid {
                    self.error(
                        "text",
                        file,
                        span,
                        format!(
                            "`{token_text}` is not a `State.field` path",
                            token_text = path_text
                        ),
                    );
                    return false;
                }
                let place = Place {
                    state: Path {
                        segments,
                        span: Span::new(span.start, span.start + state_text.len()),
                    },
                    field: Ident {
                        name: field_text.to_string(),
                        span: Span::new(field_start, span.end),
                    },
                    span,
                };
                let Some((state, field, field_ty)) = self.resolve_place(body.module, file, &place)
                else {
                    return false;
                };
                if field_ty == Ty::Float {
                    self.float_in_text(file, span);
                    return false;
                }
                body.note_state(self, state, &place, "shows");
                let Some(score) = self.field_reference(state, field) else {
                    return false;
                };
                parts.push(pc::TextPart::Score {
                    score,
                    style: style.clone(),
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
