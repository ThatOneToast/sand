//! Homogeneous collections reuse the canonical aggregate leaf/call model.
use super::*;

impl Cx {
    pub(super) fn array_type(&mut self, module: ModuleId, element: Ty, length: usize) -> Ty {
        if length > 1024 {
            self.error(
                "type",
                self.package.module(module).file,
                Span::default(),
                "arrays support at most 1024 elements",
            );
            return Ty::Unknown;
        }
        if let Some(def) = self.array_types.get(&(element, length)) {
            return Ty::Struct(*def);
        }
        let file = self.package.module(module).file;
        let def = self.push_def(Def {
            kind: DefKind::Struct,
            name: format!("[{}; {length}]", self.ty_name(element)),
            module,
            file,
            name_span: Span::default(),
            span: Span::default(),
            docs: Vec::new(),
            is_pub: false,
            parent: Some(DefId(self.defs.len())),
            info: DefInfo::Array {
                element,
                fields: Vec::new(),
            },
        });
        let mut fields = Vec::with_capacity(length);
        for index in 0..length {
            fields.push(self.push_def(Def {
                kind: DefKind::Field,
                name: format!("i{index}"),
                module,
                file,
                name_span: Span::default(),
                span: Span::default(),
                docs: Vec::new(),
                is_pub: false,
                parent: Some(def),
                info: DefInfo::Field {
                    ty: element,
                    default: None,
                    bounds: None,
                    timer: false,
                },
            }));
        }
        self.defs[def.0].info = DefInfo::Array { element, fields };
        if self.layout_leaves(Ty::Struct(def)).is_none() {
            self.error(
                "type",
                file,
                Span::default(),
                "array layouts must be finite, at most 32 levels deep and 1024 scalar leaves",
            );
            self.defs[def.0].info = DefInfo::Array {
                element,
                fields: Vec::new(),
            };
            return Ty::Unknown;
        }
        self.array_types.insert((element, length), def);
        Ty::Struct(def)
    }

    pub(super) fn array_shape(&self, ty: Ty) -> Option<(Ty, usize)> {
        let Ty::Struct(def) = ty else {
            return None;
        };
        match &self.defs[def.0].info {
            DefInfo::Array { element, fields } => Some((*element, fields.len())),
            _ => None,
        }
    }
}

impl Cx {
    pub(super) fn emit_array(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        if let Expr::ArrayRepeat {
            value,
            length: count,
            span,
        } = expr
        {
            let (element, length) = self.array_shape(ty)?;
            let count = self.eval_i32(body.module, body.file, count, Some(Ty::Number), None)?;
            if count != length as i64 {
                self.error(
                    "type",
                    body.file,
                    *span,
                    format!("expected {length} array elements, found {count}"),
                );
                return None;
            }
            let depth = body.temp_depth;
            let repeated = self.temp(body);
            self.emit(body, &repeated, element, value, ops)?;
            self.value_slots(dest, ty);
            for index in 0..length {
                self.copy_value(
                    &super::values::child(dest, &format!("i{index}")),
                    &repeated,
                    element,
                    body.file,
                    *span,
                    ops,
                );
            }
            body.temp_depth = depth;
            return Some(());
        }
        let Expr::Array(values, location) = expr else {
            unreachable!()
        };
        let span = *location;
        let Some((element, length)) = self.array_shape(ty) else {
            self.error(
                "type",
                body.file,
                span,
                "an array literal needs an array type",
            );
            return None;
        };
        if values.len() != length {
            self.error(
                "type",
                body.file,
                span,
                format!("expected {length} array elements, found {}", values.len()),
            );
            return None;
        }
        self.value_slots(dest, ty);
        for (index, value) in values.iter().enumerate() {
            let field = super::values::child(dest, &format!("i{index}"));
            self.emit(body, &field, element, value, ops)?;
        }
        Some(())
    }

    pub(super) fn emit_index(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let Expr::Index(value, index, span) = expr else {
            unreachable!()
        };
        let array_ty = self.type_of(body, value);
        if self.list_element(array_ty).is_some() {
            return self.emit_list_index(body, dest, ty, expr, ops);
        }
        let Some((element, length)) = self.array_shape(array_ty) else {
            self.error(
                "type",
                body.file,
                value.span(),
                "indexing needs an array value",
            );
            return None;
        };
        if !self.types_agree(body.file, *span, ty, element) {
            return None;
        }
        let depth = body.temp_depth;
        let array = self.temp(body);
        self.emit(body, &array, array_ty, value, ops)?;
        if Self::is_const_in(body, index) {
            let offset = self.eval_i32(body.module, body.file, index, Some(Ty::Number), None)?;
            if offset < 0 || offset as usize >= length {
                self.error(
                    "index",
                    body.file,
                    index.span(),
                    format!("index {offset} is outside an array of length {length}"),
                );
                body.temp_depth = depth;
                return None;
            }
            self.copy_value(
                dest,
                &super::values::child(&array, &format!("i{offset}")),
                element,
                body.file,
                *span,
                ops,
            );
        } else {
            let selected = self.temp(body);
            self.emit(body, &selected, Ty::Number, index, ops)?;
            let defaults = self.record_constant(body.module, body.file, None, element)?;
            for ((score, _), value) in self.value_slots(dest, element).into_iter().zip(defaults) {
                ops.push(op_at(Self::initialize_leaf(score, value), body.file, *span));
            }
            for offset in 0..length {
                let mut then = Vec::new();
                self.copy_value(
                    dest,
                    &super::values::child(&array, &format!("i{offset}")),
                    element,
                    body.file,
                    *span,
                    &mut then,
                );
                ops.push(op_at(
                    pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: selected.clone(),
                            min: Some(offset as i32),
                            max: Some(offset as i32),
                        }),
                        then,
                        otherwise: Vec::new(),
                    },
                    body.file,
                    *span,
                ));
            }
        }
        body.temp_depth = depth;
        Some(())
    }
}

/// An assignable aggregate or leaf, with frozen indexes for indirect elements.
pub(super) struct ValueAddress {
    ty: Ty,
    cells: Vec<AddressCell>,
    valid: Vec<pc::ScoreComparison>,
    possible: bool,
}
#[derive(Clone)]
struct AddressCell {
    target: AddressTarget,
    guards: Vec<pc::ScoreComparison>,
}
#[derive(Clone)]
enum AddressTarget {
    Slots(pc::FieldReference),
    Data(pc::DataReference),
}

impl Cx {
    pub(super) fn value_address(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<ValueAddress> {
        self.value_address_access(body, expr, true, ops)
    }

    pub(super) fn value_address_access(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        write: bool,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<ValueAddress> {
        match expr {
            Expr::Path(_) | Expr::Place(_) => {
                let reference = if write {
                    self.receiver_destination(body, expr)?
                } else {
                    match expr {
                        Expr::Path(path)
                            if path.segments.len() == 1 && path.applications.is_empty() =>
                        {
                            let local = Self::lookup_local(body, &path.last().name)?;
                            body.used.insert(local);
                            self.references.push(Reference {
                                file: body.file,
                                span: path.last().span,
                                def: local,
                            });
                            self.local_ref(local)
                        }
                        Expr::Place(place) => self.place_value(body, place, false)?.reference,
                        _ => return None,
                    }
                };
                let ty = self.type_of(body, expr);
                Some(ValueAddress {
                    ty,
                    cells: vec![AddressCell {
                        target: AddressTarget::Slots(reference),
                        guards: Vec::new(),
                    }],
                    valid: Vec::new(),
                    possible: true,
                })
            }
            Expr::Project(value, field, _) => {
                let mut address = self.value_address_access(body, value, write, ops)?;
                let Some(member) = self.record_member(address.ty, &field.name) else {
                    self.error(
                        "field",
                        body.file,
                        field.span,
                        format!("no field `{}` on this value", field.name),
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
                self.references.push(Reference {
                    file: body.file,
                    span: field.span,
                    def: member,
                });
                for cell in &mut address.cells {
                    cell.target = match &cell.target {
                        AddressTarget::Slots(reference) => {
                            AddressTarget::Slots(super::values::child(reference, &field.name))
                        }
                        AddressTarget::Data(reference) => {
                            let mut reference = reference.clone();
                            reference.path.push(pc::DataProjection::Member {
                                name: field.name.clone(),
                            });
                            AddressTarget::Data(reference)
                        }
                    };
                }
                address.ty = self.field_ty(member);
                Some(address)
            }
            Expr::Index(value, index, _) => self.index_address(body, value, index, write, ops),
            _ => {
                self.error(
                    "mutability",
                    body.file,
                    expr.span(),
                    "this expression is not writable; use a mutable variable or State field",
                );
                None
            }
        }
    }

    fn index_address(
        &mut self,
        body: &mut BodyCx,
        value: &Expr,
        index: &Expr,
        write: bool,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<ValueAddress> {
        let address = self.value_address_access(body, value, write, ops)?;
        if let Some(element) = self.list_element(address.ty) {
            if Self::is_const_in(body, index)
                && self.eval_i32(body.module, body.file, index, Some(Ty::Number), None)? < 0
            {
                self.error(
                    "index",
                    body.file,
                    index.span(),
                    "vector indices must be nonnegative",
                );
                return None;
            }
            let selected = self.temp(body);
            self.emit(body, &selected, Ty::Number, index, ops)?;
            let any_valid = self.temp(body);
            ops.push(op_at(
                pc::Action::ScoreSet {
                    score: any_valid.clone(),
                    value: 0,
                },
                body.file,
                index.span(),
            ));
            let mut cells = Vec::new();
            for mut parent in address.cells {
                let mut source = match &parent.target {
                    AddressTarget::Slots(reference) => Self::data_reference(reference),
                    AddressTarget::Data(reference) => reference.clone(),
                };
                let valid =
                    self.storage_index_valid(body, source.clone(), &selected, index.span(), ops);
                let guard = pc::ScoreComparison {
                    score: valid,
                    min: Some(1),
                    max: Some(1),
                };
                parent.guards.push(guard);
                let mark = vec![op_at(
                    pc::Action::ScoreSet {
                        score: any_valid.clone(),
                        value: 1,
                    },
                    body.file,
                    index.span(),
                )];
                ops.extend(self.guard_cell(body, &parent, index.span(), mark));
                source.path.push(pc::DataProjection::Index {
                    index: pc::ScoreOperand::Field(selected.clone()),
                });
                cells.push(AddressCell {
                    target: AddressTarget::Data(source),
                    guards: parent.guards,
                });
            }
            let mut valid = address.valid;
            valid.push(pc::ScoreComparison {
                score: any_valid,
                min: Some(1),
                max: Some(1),
            });
            return Some(ValueAddress {
                ty: element,
                cells,
                valid,
                possible: address.possible,
            });
        }
        let Some((element, length)) = self.array_shape(address.ty) else {
            self.error(
                "type",
                body.file,
                value.span(),
                "mutable indexing needs an array or vector value",
            );
            return None;
        };
        let offset = if Self::is_const_in(body, index) {
            let offset = self.eval_i32(body.module, body.file, index, Some(Ty::Number), None)?;
            if offset < 0 || offset as usize >= length {
                self.error(
                    "index",
                    body.file,
                    index.span(),
                    format!("index {offset} is outside an array of length {length}"),
                );
                return None;
            }
            Some(offset as usize)
        } else {
            None
        };
        let selected = if offset.is_none() {
            let selected = self.temp(body);
            self.emit(body, &selected, Ty::Number, index, ops)?;
            Some(selected)
        } else {
            None
        };
        let mut valid = address.valid;
        let possible = address.possible && length > 0;
        if let Some(selected) = &selected
            && length > 0
        {
            valid.push(pc::ScoreComparison {
                score: selected.clone(),
                min: Some(0),
                max: Some(length as i32 - 1),
            });
        }
        let mut cells = Vec::new();
        for parent in address.cells {
            if let AddressTarget::Data(reference) = &parent.target {
                let mut reference = reference.clone();
                reference.path.push(pc::DataProjection::Index {
                    index: selected.clone().map_or(
                        pc::ScoreOperand::Value(offset.unwrap_or(0) as i32),
                        pc::ScoreOperand::Field,
                    ),
                });
                cells.push(AddressCell {
                    target: AddressTarget::Data(reference),
                    guards: parent.guards,
                });
                continue;
            }
            let AddressTarget::Slots(reference) = &parent.target else {
                unreachable!()
            };
            for offset in offset.map_or(0..length, |offset| offset..offset + 1) {
                let mut guards = parent.guards.clone();
                if let Some(selected) = &selected {
                    guards.push(pc::ScoreComparison {
                        score: selected.clone(),
                        min: Some(offset as i32),
                        max: Some(offset as i32),
                    });
                }
                cells.push(AddressCell {
                    target: AddressTarget::Slots(super::values::child(
                        reference,
                        &format!("i{offset}"),
                    )),
                    guards,
                });
            }
        }
        Some(ValueAddress {
            ty: element,
            cells,
            valid,
            possible,
        })
    }

    pub(super) fn address_is_guarded(&self, address: &ValueAddress) -> bool {
        !address.possible || !address.valid.is_empty()
    }

    pub(super) fn guard_address(
        &self,
        body: &BodyCx,
        address: &ValueAddress,
        span: Span,
        mut inner: Vec<pc::Operation>,
    ) -> Vec<pc::Operation> {
        if !address.possible {
            return Vec::new();
        }
        for guard in address.valid.iter().rev() {
            inner = vec![op_at(
                pc::Action::Branch {
                    condition: pc::Condition::Range(guard.clone()),
                    then: inner,
                    otherwise: Vec::new(),
                },
                body.file,
                span,
            )];
        }
        inner
    }

    fn guard_cell(
        &self,
        body: &BodyCx,
        cell: &AddressCell,
        span: Span,
        mut inner: Vec<pc::Operation>,
    ) -> Vec<pc::Operation> {
        for guard in cell.guards.iter().rev() {
            inner = vec![op_at(
                pc::Action::Branch {
                    condition: pc::Condition::Range(guard.clone()),
                    then: inner,
                    otherwise: Vec::new(),
                },
                body.file,
                span,
            )];
        }
        inner
    }

    pub(super) fn read_address(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        address: &ValueAddress,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        if self.address_is_guarded(address)
            || address.cells.len() != 1
            || !address.cells[0].guards.is_empty()
        {
            let defaults = self.record_constant(body.module, body.file, None, address.ty)?;
            for ((score, _), value) in self.value_slots(dest, address.ty).into_iter().zip(defaults)
            {
                ops.push(op_at(Self::initialize_leaf(score, value), body.file, span));
            }
        }
        for cell in &address.cells {
            let mut inner = Vec::new();
            match &cell.target {
                AddressTarget::Slots(reference) => {
                    self.copy_value(dest, reference, address.ty, body.file, span, &mut inner)
                }
                AddressTarget::Data(reference) => {
                    self.unpack_value(body, dest, reference.clone(), address.ty, span, &mut inner)
                }
            }
            let inner = self.guard_cell(body, cell, span, inner);
            ops.extend(self.guard_address(body, address, span, inner));
        }
        Some(())
    }

    pub(super) fn write_address(
        &mut self,
        body: &mut BodyCx,
        address: &ValueAddress,
        source: &pc::FieldReference,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        for cell in &address.cells {
            let mut inner = Vec::new();
            match &cell.target {
                AddressTarget::Slots(reference) => {
                    self.copy_value(reference, source, address.ty, body.file, span, &mut inner)
                }
                AddressTarget::Data(reference) => self.pack_value(
                    body,
                    reference.clone(),
                    source,
                    address.ty,
                    span,
                    &mut inner,
                ),
            }
            let inner = self.guard_cell(body, cell, span, inner);
            ops.extend(self.guard_address(body, address, span, inner));
        }
    }

    pub(super) fn lower_value_assignment(
        &mut self,
        body: &mut BodyCx,
        target: &Expr,
        operator: AssignOp,
        value: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) {
        let depth = body.temp_depth;
        let span = target.span().to(value.span());
        let Some(address) = self.value_address(body, target, ops) else {
            body.temp_depth = depth;
            return;
        };
        let rhs = self.temp(body);
        if self.emit(body, &rhs, address.ty, value, ops).is_none() {
            body.temp_depth = depth;
            return;
        }
        let result = if operator == AssignOp::Set {
            rhs
        } else if address.ty == Ty::String && operator == AssignOp::Add {
            let current = self.temp(body);
            if self
                .read_address(body, &current, &address, span, ops)
                .is_none()
            {
                body.temp_depth = depth;
                return;
            }
            ops.push(op_at(Self::append_text(&current, &rhs), body.file, span));
            current
        } else {
            if !matches!(address.ty, Ty::Number | Ty::Float) {
                self.error(
                    "type",
                    body.file,
                    span,
                    "compound assignment needs a numeric element",
                );
                body.temp_depth = depth;
                return;
            }
            let current = self.temp(body);
            if self
                .read_address(body, &current, &address, span, ops)
                .is_none()
            {
                body.temp_depth = depth;
                return;
            }
            let bin = match operator {
                AssignOp::Add => BinOp::Add,
                AssignOp::Sub => BinOp::Sub,
                AssignOp::Mul => BinOp::Mul,
                AssignOp::Div => BinOp::Div,
                AssignOp::Rem => BinOp::Rem,
                AssignOp::Set => unreachable!(),
            };
            if self
                .apply(
                    body.file,
                    &current,
                    address.ty,
                    bin,
                    super::statements::Value::Read(rhs),
                    span,
                    ops,
                )
                .is_none()
            {
                body.temp_depth = depth;
                return;
            }
            current
        };
        self.write_address(body, &address, &result, span, ops);
        body.temp_depth = depth;
    }
}

impl Cx {
    pub(super) fn emit_array_method(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let Expr::Method {
            receiver,
            name,
            args,
            span,
            ..
        } = expr
        else {
            unreachable!()
        };
        let array_ty = self.type_of(body, receiver);
        let (element, length) = self.array_shape(array_ty)?;
        let Some(builtin) = crate::builtins::array_method(&name.name) else {
            self.error(
                "method",
                body.file,
                name.span,
                format!("arrays have no method `{}`", name.name),
            );
            return None;
        };
        if builtin.kind != crate::builtins::BuiltinKind::Expression {
            self.error(
                "type",
                body.file,
                *span,
                "this array method does not return a value",
            );
            return None;
        }
        self.references_builtin(body.file, name.span, builtin);
        let bound = self.bind_args(body, builtin, args, *span)?;
        let result_ty = match name.name.as_str() {
            "len" => Ty::Number,
            "get_or" => element,
            _ => Ty::Bool,
        };
        if !self.types_agree(body.file, *span, ty, result_ty) {
            return None;
        }
        let depth = body.temp_depth;
        let array = self.temp(body);
        self.emit(body, &array, array_ty, receiver, ops)?;
        match name.name.as_str() {
            "len" | "is_empty" => ops.push(op_at(
                pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: if name.name == "len" {
                        length as i32
                    } else {
                        i32::from(length == 0)
                    },
                },
                body.file,
                *span,
            )),
            "contains" => {
                let ArgValue::Expr(value) = &bound["value"].value else {
                    return None;
                };
                let needle = self.temp(body);
                self.emit(body, &needle, element, value, ops)?;
                ops.push(op_at(
                    pc::Action::ScoreSet {
                        score: dest.clone(),
                        value: 0,
                    },
                    body.file,
                    *span,
                ));
                let equal = self.temp(body);
                for index in 0..length {
                    let source = super::values::child(&array, &format!("i{index}"));
                    ops.push(op_at(
                        pc::Action::ScoreSet {
                            score: equal.clone(),
                            value: 1,
                        },
                        body.file,
                        *span,
                    ));
                    for ((left, leaf_ty), (right, _)) in self
                        .value_slots(&source, element)
                        .into_iter()
                        .zip(self.value_slots(&needle, element))
                    {
                        let condition = if leaf_ty == Ty::String {
                            let matched = self.temp(body);
                            ops.push(op_at(
                                pc::Action::DataEqual {
                                    score: matched.clone(),
                                    left: Self::data_reference(&left),
                                    right: Self::data_reference(&right),
                                },
                                body.file,
                                *span,
                            ));
                            pc::Condition::Range(pc::ScoreComparison {
                                score: matched,
                                min: Some(1),
                                max: Some(1),
                            })
                        } else {
                            pc::Condition::Relation(pc::ScoreRelation {
                                left,
                                relation: pc::Relation::Eq,
                                right,
                            })
                        };
                        ops.push(op_at(
                            pc::Action::Branch {
                                condition,
                                then: Vec::new(),
                                otherwise: vec![op_at(
                                    pc::Action::ScoreSet {
                                        score: equal.clone(),
                                        value: 0,
                                    },
                                    body.file,
                                    *span,
                                )],
                            },
                            body.file,
                            *span,
                        ));
                    }
                    ops.push(op_at(
                        pc::Action::Branch {
                            condition: pc::Condition::Range(pc::ScoreComparison {
                                score: equal.clone(),
                                min: Some(1),
                                max: Some(1),
                            }),
                            then: vec![op_at(
                                pc::Action::ScoreSet {
                                    score: dest.clone(),
                                    value: 1,
                                },
                                body.file,
                                *span,
                            )],
                            otherwise: Vec::new(),
                        },
                        body.file,
                        *span,
                    ));
                }
            }
            "get_or" => {
                let mut index = None;
                let mut fallback = None;
                for arg in args {
                    let ArgValue::Expr(value) = &arg.value else {
                        return None;
                    };
                    let temp = self.temp(body);
                    if std::ptr::eq(arg, bound["index"]) {
                        self.emit(body, &temp, Ty::Number, value, ops)?;
                        index = Some(temp);
                    } else {
                        self.emit(body, &temp, element, value, ops)?;
                        fallback = Some(temp);
                    }
                }
                self.copy_value(dest, &fallback?, element, body.file, *span, ops);
                let index = index?;
                for offset in 0..length {
                    let mut then = Vec::new();
                    self.copy_value(
                        dest,
                        &super::values::child(&array, &format!("i{offset}")),
                        element,
                        body.file,
                        *span,
                        &mut then,
                    );
                    ops.push(op_at(
                        pc::Action::Branch {
                            condition: pc::Condition::Range(pc::ScoreComparison {
                                score: index.clone(),
                                min: Some(offset as i32),
                                max: Some(offset as i32),
                            }),
                            then,
                            otherwise: Vec::new(),
                        },
                        body.file,
                        *span,
                    ));
                }
            }
            _ => unreachable!(),
        }
        body.temp_depth = depth;
        Some(())
    }

    pub(super) fn lower_array_statement(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) {
        let Expr::Method {
            receiver,
            name,
            args,
            span,
            ..
        } = expr
        else {
            unreachable!()
        };
        let Some(builtin) = crate::builtins::array_method(&name.name) else {
            self.error(
                "method",
                body.file,
                name.span,
                format!("arrays have no method `{}`", name.name),
            );
            return;
        };
        if name.name != "fill" {
            let ty = self.type_of(body, expr);
            let depth = body.temp_depth;
            let ignored = self.temp(body);
            self.emit(body, &ignored, ty, expr, ops);
            body.temp_depth = depth;
            return;
        }
        self.references_builtin(body.file, name.span, builtin);
        let Some(bound) = self.bind_args(body, builtin, args, *span) else {
            return;
        };
        let depth = body.temp_depth;
        let Some(address) = self.value_address(body, receiver, ops) else {
            return;
        };
        let Some((element, length)) = self.array_shape(address.ty) else {
            return;
        };
        let ArgValue::Expr(value) = &bound["value"].value else {
            self.error(
                "argument",
                body.file,
                bound["value"].span,
                "array fill needs a value expression",
            );
            return;
        };
        let replacement = self.temp(body);
        if self.emit(body, &replacement, element, value, ops).is_none() {
            body.temp_depth = depth;
            return;
        }
        let array = self.temp(body);
        self.value_slots(&array, address.ty);
        for index in 0..length {
            self.copy_value(
                &super::values::child(&array, &format!("i{index}")),
                &replacement,
                element,
                body.file,
                *span,
                ops,
            );
        }
        self.write_address(body, &address, &array, *span, ops);
        body.temp_depth = depth;
    }
}
