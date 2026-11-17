//! Record layouts and inherent methods reuse ordinary score slots and calls.
use super::statements::Value;
use super::*;
use crate::builtins::{self, BuiltinKind};

#[derive(Clone)]
pub(super) struct Method {
    pub target: DefId,
    pub receiver: Option<Receiver>,
    pub state_receiver: bool,
}

pub(super) struct PlaceValue {
    pub reference: pc::FieldReference,
    pub ty: Ty,
    pub field: DefId,
}

pub(super) fn child(base: &pc::FieldReference, name: &str) -> pc::FieldReference {
    pc::FieldReference {
        state: base.state.clone(),
        field: format!("{}_f{}_{name}", base.field, name.len()),
    }
}

impl Cx {
    pub(super) fn collect_struct(
        &mut self,
        module: ModuleId,
        file: FileId,
        item: &Item,
        record: &StructItem,
    ) -> DefId {
        self.reject_attrs(file, item);
        let id = self.push_def(Def {
            kind: DefKind::Struct,
            name: record.name.name.clone(),
            module,
            file,
            name_span: record.name.span,
            span: item.span,
            docs: item.docs.clone(),
            is_pub: item.is_pub,
            parent: None,
            info: DefInfo::Struct { fields: Vec::new() },
        });
        let mut fields = Vec::new();
        let mut members = BTreeMap::new();
        for field in &record.fields {
            let def = self.push_def(Def {
                kind: DefKind::Field,
                name: field.name.name.clone(),
                module,
                file,
                name_span: field.name.span,
                span: field.span,
                docs: field.docs.clone(),
                is_pub: field.is_pub,
                parent: Some(id),
                info: DefInfo::Field {
                    ty: Ty::Unknown,
                    default: None,
                    bounds: None,
                    timer: false,
                },
            });
            if members.insert(field.name.name.clone(), def).is_some() {
                self.error(
                    "duplicate",
                    file,
                    field.name.span,
                    format!("duplicate field `{}`", field.name.name),
                );
            }
            fields.push(def);
        }
        self.defs[id.0].info = DefInfo::Struct { fields };
        self.members.insert(id, members);
        id
    }

    pub(super) fn collect_impls(&mut self) {
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            let items = self.package.module(module).items.clone();
            for item in items {
                let ItemKind::Impl(implementation) = item.kind else {
                    continue;
                };
                if item.is_pub || !item.attrs.is_empty() {
                    self.error(
                        "impl",
                        file,
                        item.span,
                        "visibility and attributes belong on impl methods, not the impl block",
                    );
                }
                let Some(target) = self.expect_path(
                    module,
                    &implementation.target,
                    &[DefKind::State, DefKind::Struct],
                    "a State or struct",
                ) else {
                    continue;
                };
                let state = self.defs[target.0].kind == DefKind::State;
                for mut method in implementation.methods {
                    let ItemKind::Fn(function) = &mut method.kind else {
                        self.error("impl", file, method.span, "impl blocks contain functions");
                        continue;
                    };
                    let receiver = function.receiver.take();
                    if !state && let Some(receiver) = &receiver {
                        let self_span =
                            Span::new(receiver.span.end.saturating_sub(4), receiver.span.end);
                        function.params.insert(
                            0,
                            Param {
                                mutable: receiver.mutable,
                                name: Ident {
                                    name: "self".into(),
                                    span: self_span,
                                },
                                ty: Type::named(Path {
                                    segments: vec![Ident {
                                        name: "Self".into(),
                                        span: self_span,
                                    }],
                                    span: self_span,
                                }),
                                span: receiver.span,
                            },
                        );
                    }
                    let name = function.name.clone();
                    if self.members[&target].contains_key(&name.name) {
                        self.error(
                            "duplicate",
                            file,
                            name.span,
                            format!(
                                "`{}` already has a member named `{}`",
                                self.defs[target.0].name, name.name
                            ),
                        );
                        continue;
                    }
                    let id = format!(
                        "{}.method/{}",
                        self.item_id(module, &self.defs[target.0].name),
                        name.name
                    );
                    let def = self.push_def(Def {
                        kind: DefKind::Function,
                        name: name.name.clone(),
                        module,
                        file,
                        name_span: name.span,
                        span: method.span,
                        docs: method.docs.clone(),
                        is_pub: method.is_pub,
                        parent: Some(target),
                        info: DefInfo::Function {
                            id,
                            context: Context::Server,
                            player_reason: None,
                            entry: Vec::new(),
                            signature: "()".into(),
                        },
                    });
                    self.methods.insert(
                        def,
                        Method {
                            target,
                            state_receiver: state && receiver.is_some(),
                            receiver,
                        },
                    );
                    self.members
                        .get_mut(&target)
                        .expect("type members")
                        .insert(name.name, def);
                    self.package.modules[index].items.push(method);
                }
            }
        }
    }

    pub(super) fn evaluate_structs(&mut self) {
        for index in 0..self.package.modules.len() {
            let module = ModuleId(index);
            let file = self.package.module(module).file;
            for item in self.package.module(module).items.clone() {
                let ItemKind::Struct(record) = &item.kind else {
                    continue;
                };
                let Some(def) = self.def_by_span(module, record.name.span) else {
                    continue;
                };
                let fields = self.struct_fields(def);
                if fields.is_empty() {
                    self.error(
                        "struct",
                        file,
                        record.name.span,
                        "a struct needs at least one field",
                    );
                }
                for (source, field) in record.fields.iter().zip(fields) {
                    if !source.attrs.is_empty() || source.default.is_some() {
                        self.error("struct", file, source.span, "struct fields declare types; defaults and clamps belong on State fields");
                    }
                    let ty = self.resolve_type(module, &source.ty);
                    self.defs[field.0].info = DefInfo::Field {
                        ty,
                        default: None,
                        bounds: None,
                        timer: false,
                    };
                }
            }
        }
        let structs: Vec<_> = self
            .defs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == DefKind::Struct)
            .map(|(i, _)| DefId(i))
            .collect();
        for def in structs {
            if self.layout_leaves(Ty::Struct(def)).is_none() {
                let d = self.defs[def.0].clone();
                self.error(
                    "struct",
                    d.file,
                    d.name_span,
                    "struct layouts must be finite, at most 32 levels deep and 1024 scalar fields",
                );
                self.defs[def.0].info = DefInfo::Struct { fields: Vec::new() };
            }
        }
    }

    /// Validate one aggregate layout, including inferred arrays created while
    /// lowering bodies, before any recursive slot expansion can run.
    pub(super) fn layout_leaves(&self, ty: Ty) -> Option<usize> {
        fn visit(cx: &Cx, ty: Ty, stack: &mut Vec<DefId>) -> Option<usize> {
            let Ty::Struct(def) = ty else {
                return Some(1);
            };
            if stack.len() >= 32 || stack.contains(&def) {
                return None;
            }
            stack.push(def);
            let mut leaves = 0usize;
            for field in cx.struct_fields(def) {
                leaves = leaves.checked_add(visit(cx, cx.field_ty(field), stack)?)?;
                if leaves > 1024 {
                    return None;
                }
            }
            stack.pop();
            Some(leaves)
        }
        visit(self, ty, &mut Vec::new())
    }

    pub(super) fn struct_fields(&self, def: DefId) -> Vec<DefId> {
        match &self.defs[def.0].info {
            DefInfo::Struct { fields } | DefInfo::Array { fields, .. } => fields.clone(),
            _ => Vec::new(),
        }
    }
    pub(super) fn field_ty(&self, field: DefId) -> Ty {
        match self.defs[field.0].info {
            DefInfo::Field { ty, .. } => ty,
            _ => Ty::Unknown,
        }
    }
    pub(super) fn record_member(&self, ty: Ty, name: &str) -> Option<DefId> {
        let Ty::Struct(def) = ty else { return None };
        self.members
            .get(&def)?
            .get(name)
            .copied()
            .filter(|field| self.defs[field.0].kind == DefKind::Field)
    }
    pub(super) fn value_slots(
        &mut self,
        base: &pc::FieldReference,
        ty: Ty,
    ) -> Vec<(pc::FieldReference, Ty)> {
        if let Ty::Struct(def) = ty {
            self.struct_fields(def)
                .into_iter()
                .flat_map(|field| {
                    self.value_slots(
                        &child(base, &self.defs[field.0].name.clone()),
                        self.field_ty(field),
                    )
                })
                .collect()
        } else {
            if ty == Ty::String {
                self.register_data_slot(base, text::text_value(""));
            } else if base.state.to_string() == self.locals_state_id() {
                self.slot(base.field.clone());
            }
            vec![(base.clone(), ty)]
        }
    }
    pub(super) fn copy_value(
        &mut self,
        dest: &pc::FieldReference,
        source: &pc::FieldReference,
        ty: Ty,
        file: FileId,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        for ((dest, leaf_ty), (source, _)) in self
            .value_slots(dest, ty)
            .into_iter()
            .zip(self.value_slots(source, ty))
        {
            if leaf_ty == Ty::String {
                ops.push(op_at(Self::copy_data(&dest, &source), file, span));
            } else if let Some(action) = Self::set_from(&dest, Value::Read(source)) {
                ops.push(op_at(action, file, span));
            }
        }
    }
    pub(super) fn place_ty(&self, body: &BodyCx, place: &Place) -> Ty {
        let local = (place.state.segments.len() == 1)
            .then(|| Self::lookup_local(body, &place.state.last().name))
            .flatten();
        let (mut ty, projections) = if let Some(local) = local {
            (
                self.local_ty(local),
                std::iter::once(&place.field)
                    .chain(&place.projections)
                    .collect::<Vec<_>>(),
            )
        } else {
            let Ok((state, _)) = self.resolve_path(body.module, &place.state) else {
                return Ty::Unknown;
            };
            let Some(field) = self
                .members
                .get(&state)
                .and_then(|m| m.get(&place.field.name))
            else {
                return Ty::Unknown;
            };
            (
                self.field_ty(*field),
                place.projections.iter().collect::<Vec<_>>(),
            )
        };
        for field in projections {
            let Some(member) = self.record_member(ty, &field.name) else {
                return Ty::Unknown;
            };
            ty = self.field_ty(member);
        }
        ty
    }
    pub(super) fn place_value(
        &mut self,
        body: &mut BodyCx,
        place: &Place,
        write: bool,
    ) -> Option<PlaceValue> {
        let local = (place.state.segments.len() == 1)
            .then(|| Self::lookup_local(body, &place.state.last().name))
            .flatten();
        let (mut reference, mut ty, mut field, projections) = if let Some(local) = local {
            let DefInfo::Local { ty, mutable, .. } = self.defs[local.0].info else {
                return None;
            };
            if write && !mutable {
                self.error(
                    "mutability",
                    body.file,
                    place.state.span,
                    "cannot change a field of an immutable value; use `let mut` or `mut self`",
                );
                return None;
            }
            body.used.insert(local);
            if write {
                body.assigned.insert(local);
            }
            self.references.push(Reference {
                file: body.file,
                span: place.state.span,
                def: local,
            });
            let projections = std::iter::once(place.field.clone())
                .chain(place.projections.clone())
                .collect::<Vec<_>>();
            (self.local_ref(local), ty, local, projections)
        } else {
            let (state, field, ty) = self.resolve_place(body.module, body.file, place)?;
            if write && !self.check_state_write(body, state, place.span) {
                return None;
            }
            body.note_state(self, state, place, if write { "writes" } else { "reads" });
            (
                self.field_reference(state, field)?,
                ty,
                field,
                place.projections.clone(),
            )
        };
        for projection in projections {
            let Some(member) = self.record_member(ty, &projection.name) else {
                self.error(
                    "field",
                    body.file,
                    projection.span,
                    format!("`{}` has no field `{}`", self.ty_name(ty), projection.name),
                );
                return None;
            };
            if !self.visible(
                body.module,
                self.defs[member.0].module,
                self.defs[member.0].is_pub,
            ) {
                self.error(
                    "privacy",
                    body.file,
                    projection.span,
                    format!("field `{}` is private", projection.name),
                );
                return None;
            }
            reference = child(&reference, &projection.name);
            self.references.push(Reference {
                file: body.file,
                span: projection.span,
                def: member,
            });
            ty = self.field_ty(member);
            field = member;
        }
        self.value_slots(&reference, ty);
        Some(PlaceValue {
            reference,
            ty,
            field,
        })
    }

    pub(super) fn check_state_write(&mut self, body: &BodyCx, state: DefId, span: Span) -> bool {
        if let Some(method) = body.owner.and_then(|owner| self.methods.get(&owner))
            && method.target == state
            && method.receiver.as_ref().is_some_and(|r| !r.mutable)
        {
            self.error(
                "mutability",
                body.file,
                span,
                "this State receiver is read-only; use `&mut self`",
            );
            return false;
        }
        true
    }

    pub(super) fn lower_reference_loop(
        &mut self,
        body: &mut BodyCx,
        stmt: &Stmt,
        ops: &mut Vec<pc::Operation>,
    ) {
        let Stmt::For {
            binding,
            source,
            args,
            body: block,
            span,
        } = stmt
        else {
            unreachable!()
        };
        let players = source.last().name == "resolve_player";
        let builtin = builtins::find(&source.last().name, BuiltinKind::Expression)
            .expect("reference builtin");
        self.references_builtin(body.file, source.span, builtin);
        let Some(bound) = self.bind_args(body, builtin, args, *span) else {
            return;
        };
        let ArgValue::Expr(expr) = &bound["reference"].value else {
            return;
        };
        let depth = body.temp_depth;
        let reference = self.temp(body);
        if self
            .emit(body, &reference, Ty::EntityRef, expr, ops)
            .is_none()
        {
            body.temp_depth = depth;
            return;
        }
        let was = body.in_players;
        body.in_players = true;
        body.loops.push(if players {
            LoopKind::Players
        } else {
            LoopKind::Entities
        });
        body.loop_bindings.push(binding.name.clone());
        if players {
            body.player_loops += 1;
        }
        self.handle_scopes
            .push((body.file, block.span, binding.name.clone(), players));
        let inner = self.lower_block(body, block);
        if players {
            body.player_loops -= 1;
        }
        body.loops.pop();
        body.loop_bindings.pop();
        body.in_players = was;
        body.temp_depth = depth;
        ops.push(op_at(
            pc::Action::WithEntity {
                reference,
                players,
                body: inner,
            },
            body.file,
            *span,
        ));
    }

    /// Resolve a mutable receiver to existing writable storage. A method call
    /// commits its receiver result after restoring the caller's recursive frame.
    pub(super) fn receiver_destination(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
    ) -> Option<pc::FieldReference> {
        match expr {
            Expr::Path(path) if path.segments.len() == 1 => {
                let local = Self::lookup_local(body, &path.last().name)?;
                if !matches!(
                    self.defs[local.0].info,
                    DefInfo::Local { mutable: true, .. }
                ) {
                    self.error(
                        "mutability",
                        body.file,
                        path.span,
                        "a mutable method needs a mutable receiver; declare it with `let mut`",
                    );
                    return None;
                }
                body.assigned.insert(local);
                body.used.insert(local);
                self.references.push(Reference {
                    file: body.file,
                    span: path.span,
                    def: local,
                });
                Some(self.local_ref(local))
            }
            Expr::Place(place) => self
                .place_value(body, place, true)
                .map(|value| value.reference),
            _ => {
                self.error("receiver", body.file, expr.span(), "a mutable method needs a variable or State field; store this value in `let mut` first");
                None
            }
        }
    }

    pub(super) fn finish_receiver(
        &mut self,
        body: &BodyCx,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        if let Some((source, dest, ty)) = &body.receiver_out {
            self.copy_value(dest, source, *ty, body.file, span, ops);
        }
    }

    pub(super) fn record_constant(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: Option<&Expr>,
        ty: Ty,
    ) -> Option<Vec<text::LeafDefault>> {
        if ty == Ty::String {
            let value = match expr {
                Some(expr) => self.constant_text(module, file, expr, &mut Vec::new())?,
                None => String::new(),
            };
            return Some(vec![text::LeafDefault::Data(text::text_value(&value))]);
        }
        if let Some((element, length)) = self.array_shape(ty) {
            if let Some(Expr::ArrayRepeat {
                value,
                length: count,
                span,
            }) = expr
            {
                let count = self.eval_i32(module, file, count, Some(Ty::Number), None)?;
                if count != length as i64 {
                    self.error(
                        "const",
                        file,
                        *span,
                        format!("expected {length} array elements, found {count}"),
                    );
                    return None;
                }
                let repeated = self.record_constant(module, file, Some(value), element)?;
                return Some((0..length).flat_map(|_| repeated.clone()).collect());
            }
            let values = match expr {
                Some(Expr::Array(values, _)) if values.len() == length => Some(values),
                Some(value) => {
                    self.error(
                        "const",
                        file,
                        value.span(),
                        format!("expected an array literal with {length} elements"),
                    );
                    return None;
                }
                None => None,
            };
            let mut leaves = Vec::new();
            for index in 0..length {
                leaves.extend(self.record_constant(
                    module,
                    file,
                    values.map(|v| &v[index]),
                    element,
                )?);
            }
            return Some(leaves);
        }
        if let Ty::Struct(def) = ty {
            let mut supplied = BTreeMap::new();
            if let Some(expr) = expr {
                let Expr::Struct { path, fields, .. } = expr else {
                    self.error(
                        "const",
                        file,
                        expr.span(),
                        "a struct State default must be a record literal",
                    );
                    return None;
                };
                let found = self.expect_path(module, path, &[DefKind::Struct], "a struct")?;
                if found != def {
                    self.error(
                        "type",
                        file,
                        path.span,
                        "record default has the wrong struct type",
                    );
                    return None;
                }
                for (name, value) in fields {
                    if let Some(member) = self.record_member(ty, &name.name) {
                        if !self.visible(
                            module,
                            self.defs[member.0].module,
                            self.defs[member.0].is_pub,
                        ) {
                            self.error(
                                "privacy",
                                file,
                                name.span,
                                format!("field `{}` is private", name.name),
                            );
                            return None;
                        }
                        self.references.push(Reference {
                            file,
                            span: name.span,
                            def: member,
                        });
                    }
                    if supplied.insert(name.name.clone(), value).is_some() {
                        self.error("duplicate", file, name.span, "duplicate record field");
                    }
                }
            }
            let fields = self.struct_fields(def);
            for name in supplied.keys() {
                if !fields.iter().any(|f| &self.defs[f.0].name == name) {
                    self.error(
                        "field",
                        file,
                        expr.expect("literal").span(),
                        format!("unknown record field `{name}`"),
                    );
                }
            }
            let mut values = Vec::new();
            for field in fields {
                let name = self.defs[field.0].name.clone();
                let value = supplied.remove(&name);
                if let Some(expr) = expr
                    && value.is_none()
                {
                    self.error(
                        "field",
                        file,
                        expr.span(),
                        format!("missing record field `{name}`"),
                    );
                    return None;
                }
                values.extend(self.record_constant(module, file, value, self.field_ty(field))?);
            }
            Some(values)
        } else {
            let value = match expr {
                Some(expr) => self.eval_i32(module, file, expr, Some(ty), None)?,
                None => match ty {
                    Ty::Enum(def) => match &self.defs[def.0].info {
                        DefInfo::Enum { variants } => variants
                            .first()
                            .and_then(|v| match self.defs[v.0].info {
                                DefInfo::Variant { value } => value,
                                _ => None,
                            })
                            .unwrap_or(0),
                        _ => 0,
                    },
                    _ => 0,
                },
            };
            Some(vec![text::LeafDefault::Score(value as i32)])
        }
    }

    pub(super) fn emit_record(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        if matches!(expr, Expr::Array(..) | Expr::ArrayRepeat { .. }) {
            return self.emit_array(body, dest, ty, expr, ops);
        }
        self.value_slots(dest, ty);
        let found = self.type_of(body, expr);
        if !self.types_agree(body.file, expr.span(), ty, found) {
            return None;
        }
        match expr {
            Expr::Struct { path, fields, span } => {
                let def = self.expect_path(body.module, path, &[DefKind::Struct], "a struct")?;
                let expected = self.struct_fields(def);
                let mut seen = HashSet::new();
                for (name, value) in fields {
                    if !seen.insert(name.name.clone()) {
                        self.error("duplicate", body.file, name.span, "duplicate record field");
                        return None;
                    }
                    let Some(field) = self.record_member(Ty::Struct(def), &name.name) else {
                        self.error(
                            "field",
                            body.file,
                            name.span,
                            format!("unknown field `{}`", name.name),
                        );
                        return None;
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
                        return None;
                    }
                    self.references.push(Reference {
                        file: body.file,
                        span: name.span,
                        def: field,
                    });
                    self.emit(
                        body,
                        &child(dest, &name.name),
                        self.field_ty(field),
                        value,
                        ops,
                    )?;
                }
                for field in expected {
                    if !seen.contains(&self.defs[field.0].name) {
                        self.error(
                            "field",
                            body.file,
                            *span,
                            format!("missing record field `{}`", self.defs[field.0].name),
                        );
                        return None;
                    }
                }
                Some(())
            }
            Expr::Path(path) => {
                let Some(local) = Self::lookup_local(body, &path.last().name) else {
                    self.error(
                        "resolve",
                        body.file,
                        path.span,
                        format!("cannot find struct value `{}`", path.display()),
                    );
                    return None;
                };
                body.used.insert(local);
                self.references.push(Reference {
                    file: body.file,
                    span: path.span,
                    def: local,
                });
                let source = self.local_ref(local);
                self.copy_value(dest, &source, ty, body.file, expr.span(), ops);
                Some(())
            }
            Expr::Place(place) => {
                let value = self.place_value(body, place, false)?;
                self.copy_value(dest, &value.reference, ty, body.file, place.span, ops);
                Some(())
            }
            Expr::Call { path, args, span } => {
                self.emit_call(body, dest, ty, path, args, *span, ops)
            }
            Expr::Method {
                receiver,
                name,
                args,
                span,
            } => {
                let receiver_ty = self.type_of(body, receiver);
                if self.array_shape(receiver_ty).is_some() {
                    return self.emit_array_method(body, dest, ty, expr, ops);
                }
                let (path, args) = self.method_call(body, receiver, name, args)?;
                self.emit_call(body, dest, ty, &path, &args, *span, ops)
            }
            Expr::Index(..) => self.emit_index(body, dest, ty, expr, ops),
            Expr::Project(value, field, _) => {
                self.emit_projection(body, dest, ty, value, field, ops)
            }
            _ => {
                self.error("type", body.file, expr.span(), "expected a struct value");
                None
            }
        }
    }

    pub(super) fn emit_projection(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        value: &Expr,
        field: &Ident,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let span = Span::new(value.span().start, field.span.end);
        let parent_ty = self.type_of(body, value);
        let Some(member) = self.record_member(parent_ty, &field.name) else {
            self.error(
                "field",
                body.file,
                field.span,
                format!(
                    "`{}` has no field `{}`",
                    self.ty_name(parent_ty),
                    field.name
                ),
            );
            return None;
        };
        if !self.visible(
            body.module,
            self.defs[member.0].module,
            self.defs[member.0].is_pub,
        ) {
            self.error(
                "privacy",
                body.file,
                field.span,
                format!("field `{}` is private", field.name),
            );
            return None;
        }
        let found = self.field_ty(member);
        if !self.types_agree(body.file, span, ty, found) {
            return None;
        }
        let depth = body.temp_depth;
        let temp = self.temp(body);
        self.emit(body, &temp, parent_ty, value, ops)?;
        self.references.push(Reference {
            file: body.file,
            span: field.span,
            def: member,
        });
        self.copy_value(dest, &child(&temp, &field.name), ty, body.file, span, ops);
        body.temp_depth = depth;
        Some(())
    }

    pub(super) fn method_target(&self, body: &BodyCx, receiver: &Expr) -> Option<DefId> {
        if let Expr::Path(path) = receiver
            && Self::lookup_local(body, &path.last().name).is_none()
            && let Ok((def, _)) = self.resolve_path(body.module, path)
            && self.defs[def.0].kind == DefKind::State
        {
            return Some(def);
        }
        None
    }
    pub(super) fn method_call(
        &mut self,
        body: &mut BodyCx,
        receiver: &Expr,
        name: &Ident,
        args: &[Arg],
    ) -> Option<(Path, Vec<Arg>)> {
        let target =
            self.method_target(body, receiver)
                .or_else(|| match self.type_of(body, receiver) {
                    Ty::Struct(def) => Some(def),
                    _ => None,
                });
        let Some(target) = target else {
            self.error(
                "method",
                body.file,
                name.span,
                "methods need a struct value or a State receiver",
            );
            return None;
        };
        let Some(def) = self
            .members
            .get(&target)
            .and_then(|m| m.get(&name.name))
            .copied()
            .filter(|d| self.methods.contains_key(d))
        else {
            self.error(
                "method",
                body.file,
                name.span,
                format!(
                    "`{}` has no method `{}`",
                    self.defs[target.0].name, name.name
                ),
            );
            return None;
        };
        let method = self.methods[&def].clone();
        let mut arguments = args.to_vec();
        if self.defs[target.0].kind == DefKind::Struct {
            if method.receiver.is_none() {
                self.error(
                    "method",
                    body.file,
                    name.span,
                    "call associated functions with `Type::function(...)`",
                );
                return None;
            }
            arguments.insert(
                0,
                Arg {
                    name: None,
                    value: ArgValue::Expr(receiver.clone()),
                    span: receiver.span(),
                },
            );
        } else if let Expr::Path(path) = receiver {
            let _ = self.expect_path(body.module, path, &[DefKind::State], "a State");
        }
        let mut segments = vec![Ident {
            name: "crate".into(),
            span: Span::default(),
        }];
        segments.extend(
            self.package
                .module(self.defs[target.0].module)
                .path
                .iter()
                .map(|name| Ident {
                    name: name.clone(),
                    span: Span::default(),
                }),
        );
        segments.push(Ident {
            name: self.defs[target.0].name.clone(),
            span: Span::default(),
        });
        segments.push(name.clone());
        Some((
            Path {
                segments,
                span: name.span,
            },
            arguments,
        ))
    }
    pub(super) fn lower_value_statement(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        if let Expr::Method {
            receiver,
            name,
            args,
            ..
        } = expr
        {
            let receiver_ty = self.type_of(body, receiver);
            if self.array_shape(receiver_ty).is_some() {
                self.lower_array_statement(body, expr, ops);
                return;
            }
            if let Some((path, args)) = self.method_call(body, receiver, name, args) {
                self.lower_call(body, &path, &args, span, ops);
            }
        } else {
            self.error(
                "statement",
                body.file,
                span,
                "only function and method calls can be used as statements",
            );
        }
    }
}
