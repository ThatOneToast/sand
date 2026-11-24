//! Growable collections use the same owned NBT leaves as runtime text.
use super::*;
use pc::DataModifyOperation;
use sand::data::{NbtCompound, NbtValue};

pub(super) const LIST_LIMIT: usize = 1024;

impl Cx {
    pub(super) fn storage_index_valid(
        &mut self,
        body: &mut BodyCx,
        source: pc::DataReference,
        index: &pc::FieldReference,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) -> pc::FieldReference {
        let file = body.file;
        let at = |action| op_at(action, file, span);
        let length = self.temp(body);
        let valid = self.temp(body);
        ops.push(at(pc::Action::DataRead {
            score: length.clone(),
            source,
        }));
        ops.push(at(pc::Action::ScoreSet {
            score: valid.clone(),
            value: 0,
        }));
        ops.push(at(pc::Action::Branch {
            condition: pc::Condition::Range(pc::ScoreComparison {
                score: index.clone(),
                min: Some(0),
                max: None,
            }),
            then: vec![at(pc::Action::Branch {
                condition: pc::Condition::Relation(pc::ScoreRelation {
                    left: index.clone(),
                    relation: pc::Relation::Lt,
                    right: length,
                }),
                then: vec![at(pc::Action::ScoreSet {
                    score: valid.clone(),
                    value: 1,
                })],
                otherwise: Vec::new(),
            })],
            otherwise: Vec::new(),
        }));
        valid
    }

    pub(super) fn emit_list_index(
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
        let list_ty = self.type_of(body, value);
        let element = self.list_element(list_ty)?;
        if !self.types_agree(body.file, *span, ty, element) {
            return None;
        }
        let depth = body.temp_depth;
        let list = self.temp(body);
        self.emit(body, &list, list_ty, value, ops)?;
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
        let defaults = self.record_constant(body.module, body.file, None, element)?;
        for ((score, _), value) in self.value_slots(dest, element).into_iter().zip(defaults) {
            ops.push(op_at(Self::initialize_leaf(score, value), body.file, *span));
        }
        let mut source = Self::data_reference(&list);
        let valid = self.storage_index_valid(body, source.clone(), &selected, *span, ops);
        source.path.push(pc::DataProjection::Index {
            index: pc::ScoreOperand::Field(selected),
        });
        let mut then = Vec::new();
        self.unpack_value(body, dest, source, element, *span, &mut then);
        ops.push(op_at(
            pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: valid,
                    min: Some(1),
                    max: Some(1),
                }),
                then,
                otherwise: Vec::new(),
            },
            body.file,
            *span,
        ));
        body.temp_depth = depth;
        Some(())
    }

    pub(super) fn list_type(&mut self, module: ModuleId, element: Ty) -> Ty {
        if let Some(def) = self.list_types.get(&element) {
            return Ty::List(*def);
        }
        let index = self.defs.len();
        let def = self.push_def(Def {
            kind: DefKind::Struct,
            name: format!("Vec<{}>", self.ty_name(element)),
            module,
            file: self.package.module(module).file,
            name_span: Span::default(),
            span: Span::default(),
            docs: Vec::new(),
            is_pub: false,
            parent: Some(DefId(index)),
            info: DefInfo::List { element },
        });
        self.list_types.insert(element, def);
        Ty::List(def)
    }

    pub(super) fn list_element(&self, ty: Ty) -> Option<Ty> {
        let Ty::List(def) = ty else {
            return None;
        };
        let DefInfo::List { element } = self.defs[def.0].info else {
            return None;
        };
        Some(element)
    }

    pub(super) fn list_constant(
        &mut self,
        module: ModuleId,
        file: FileId,
        def: DefId,
        value: Option<&Expr>,
    ) -> Option<NbtValue> {
        let DefInfo::List { element } = self.defs[def.0].info else {
            unreachable!()
        };
        match value {
            None => Some(NbtValue::List(Vec::new())),
            Some(Expr::Array(values, span)) => {
                if values.len() > LIST_LIMIT {
                    self.error(
                        "const",
                        file,
                        *span,
                        "vectors support at most 1024 elements",
                    );
                    return None;
                }
                let mut result = Vec::new();
                let mut bytes = 2usize;
                for value in values {
                    let value = self.constant_nbt(module, file, element, Some(value))?;
                    bytes = bytes.saturating_add(value.to_string().len().saturating_add(1));
                    if !self.default_size(file, *span, bytes) {
                        return None;
                    }
                    result.push(value);
                }
                Some(NbtValue::List(result))
            }
            Some(Expr::ArrayRepeat {
                value,
                length,
                span,
            }) => {
                let count = self.eval_i32(module, file, length, Some(Ty::Number), None)?;
                if !(0..=LIST_LIMIT as i64).contains(&count) {
                    self.error(
                        "const",
                        file,
                        *span,
                        "vectors support at most 1024 elements",
                    );
                    return None;
                }
                let value = self.constant_nbt(module, file, element, Some(value))?;
                let bytes = value
                    .to_string()
                    .len()
                    .saturating_add(1)
                    .saturating_mul(count as usize)
                    .saturating_add(2);
                if !self.default_size(file, *span, bytes) {
                    return None;
                }
                Some(NbtValue::List(vec![value; count as usize]))
            }
            Some(value) => {
                self.error(
                    "const",
                    file,
                    value.span(),
                    "expected a constant vector literal",
                );
                None
            }
        }
    }

    fn constant_nbt(
        &mut self,
        module: ModuleId,
        file: FileId,
        ty: Ty,
        value: Option<&Expr>,
    ) -> Option<NbtValue> {
        let defaults = self.record_constant(module, file, value, ty)?;
        let result = self.nested_default(ty, &mut defaults.into_iter());
        if !self.default_size(
            file,
            value.map(Expr::span).unwrap_or_default(),
            result.to_string().len(),
        ) {
            return None;
        }
        Some(result)
    }

    fn nested_default(
        &self,
        ty: Ty,
        leaves: &mut impl Iterator<Item = text::LeafDefault>,
    ) -> NbtValue {
        if let Some((element, length)) = self.array_shape(ty) {
            return NbtValue::List(
                (0..length)
                    .map(|_| self.nested_default(element, leaves))
                    .collect(),
            );
        }
        if let Ty::Struct(def) = ty {
            let mut result = NbtCompound::new();
            for field in self.struct_fields(def) {
                result = result.field(
                    &self.defs[field.0].name,
                    self.nested_default(self.field_ty(field), leaves),
                );
            }
            return NbtValue::Compound(result);
        }
        match leaves.next().expect("validated value layout") {
            text::LeafDefault::Score(value) => NbtValue::Int(value),
            text::LeafDefault::Data(value) => value,
        }
    }

    /// Pack a typed score/aggregate value into its canonical nested NBT shape.
    pub(super) fn pack_value(
        &mut self,
        body: &mut BodyCx,
        target: pc::DataReference,
        source: &pc::FieldReference,
        ty: Ty,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let at = |action| op_at(action, file, span);
        self.value_slots(source, ty);
        if let Some((element, length)) = self.array_shape(ty) {
            ops.push(at(pc::Action::DataModify {
                target: target.clone(),
                operator: DataModifyOperation::Set,
                source: pc::DataOperand::Value {
                    value: NbtValue::List(Vec::new()),
                },
            }));
            for index in 0..length {
                let depth = body.temp_depth;
                let packed = self.temp(body);
                self.register_data_slot(&packed, NbtValue::List(Vec::new()));
                self.pack_value(
                    body,
                    Self::data_reference(&packed),
                    &super::values::child(source, &format!("i{index}")),
                    element,
                    span,
                    ops,
                );
                ops.push(at(pc::Action::DataModify {
                    target: target.clone(),
                    operator: DataModifyOperation::Append,
                    source: pc::DataOperand::Reference {
                        value: Self::data_reference(&packed),
                    },
                }));
                body.temp_depth = depth;
            }
        } else if let Ty::Struct(def) = ty {
            ops.push(at(pc::Action::DataModify {
                target: target.clone(),
                operator: DataModifyOperation::Set,
                source: pc::DataOperand::Value {
                    value: NbtValue::Compound(NbtCompound::new()),
                },
            }));
            for field in self.struct_fields(def) {
                let name = self.defs[field.0].name.clone();
                let mut target = target.clone();
                target
                    .path
                    .push(pc::DataProjection::Member { name: name.clone() });
                self.pack_value(
                    body,
                    target,
                    &super::values::child(source, &name),
                    self.field_ty(field),
                    span,
                    ops,
                );
            }
        } else {
            let source = if matches!(ty, Ty::String | Ty::List(_) | Ty::Callable(_)) {
                pc::DataOperand::Reference {
                    value: Self::data_reference(source),
                }
            } else {
                pc::DataOperand::Score {
                    value: pc::ScoreOperand::Field(source.clone()),
                }
            };
            ops.push(at(pc::Action::DataModify {
                target,
                operator: DataModifyOperation::Set,
                source,
            }));
        }
    }

    /// Restore the typed leaf layout from a stored element.
    pub(super) fn unpack_value(
        &mut self,
        body: &mut BodyCx,
        target: &pc::FieldReference,
        source: pc::DataReference,
        ty: Ty,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        self.value_slots(target, ty);
        if let Some((element, length)) = self.array_shape(ty) {
            for index in 0..length {
                let mut source = source.clone();
                source.path.push(pc::DataProjection::Index {
                    index: pc::ScoreOperand::Value(index as i32),
                });
                self.unpack_value(
                    body,
                    &super::values::child(target, &format!("i{index}")),
                    source,
                    element,
                    span,
                    ops,
                );
            }
        } else if let Ty::Struct(def) = ty {
            for field in self.struct_fields(def) {
                let name = self.defs[field.0].name.clone();
                let mut source = source.clone();
                source
                    .path
                    .push(pc::DataProjection::Member { name: name.clone() });
                self.unpack_value(
                    body,
                    &super::values::child(target, &name),
                    source,
                    self.field_ty(field),
                    span,
                    ops,
                );
            }
        } else {
            let action = if matches!(ty, Ty::String | Ty::List(_) | Ty::Callable(_)) {
                pc::Action::DataModify {
                    target: Self::data_reference(target),
                    operator: DataModifyOperation::Set,
                    source: pc::DataOperand::Reference { value: source },
                }
            } else {
                pc::Action::DataRead {
                    score: target.clone(),
                    source,
                }
            };
            ops.push(op_at(action, file, span));
        }
    }

    pub(super) fn emit_list(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let element = self.list_element(ty)?;
        self.value_slots(dest, ty);
        let file = body.file;
        let span = expr.span();
        let at = |action| op_at(action, file, span);
        match expr {
            Expr::Array(values, _) => {
                if values.len() > LIST_LIMIT {
                    self.error("type", file, span, "vectors support at most 1024 elements");
                    return None;
                }
                // Stage the full literal before committing the destination.
                let depth = body.temp_depth;
                let output = self.temp(body);
                self.register_data_slot(&output, NbtValue::List(Vec::new()));
                ops.push(at(Self::initialize_leaf(
                    output.clone(),
                    text::LeafDefault::Data(NbtValue::List(Vec::new())),
                )));
                for value in values {
                    let element_depth = body.temp_depth;
                    let logical = self.temp(body);
                    self.emit(body, &logical, element, value, ops)?;
                    let packed = self.temp(body);
                    self.register_data_slot(&packed, NbtValue::List(Vec::new()));
                    self.pack_value(
                        body,
                        Self::data_reference(&packed),
                        &logical,
                        element,
                        span,
                        ops,
                    );
                    ops.push(at(pc::Action::DataModify {
                        target: Self::data_reference(&output),
                        operator: DataModifyOperation::Append,
                        source: pc::DataOperand::Reference {
                            value: Self::data_reference(&packed),
                        },
                    }));
                    body.temp_depth = element_depth;
                }
                self.copy_value(dest, &output, ty, file, span, ops);
                body.temp_depth = depth;
            }
            Expr::ArrayRepeat { value, length, .. } => {
                let count = self.eval_i32(body.module, file, length, Some(Ty::Number), None)?;
                if !(0..=LIST_LIMIT as i64).contains(&count) {
                    self.error("type", file, span, "vectors support at most 1024 elements");
                    return None;
                }
                let depth = body.temp_depth;
                let logical = self.temp(body);
                self.emit(body, &logical, element, value, ops)?;
                let packed = self.temp(body);
                self.register_data_slot(&packed, NbtValue::List(Vec::new()));
                self.pack_value(
                    body,
                    Self::data_reference(&packed),
                    &logical,
                    element,
                    span,
                    ops,
                );
                ops.push(at(Self::initialize_leaf(
                    dest.clone(),
                    text::LeafDefault::Data(NbtValue::List(Vec::new())),
                )));
                for _ in 0..count {
                    ops.push(at(pc::Action::DataModify {
                        target: Self::data_reference(dest),
                        operator: DataModifyOperation::Append,
                        source: pc::DataOperand::Reference {
                            value: Self::data_reference(&packed),
                        },
                    }));
                }
                body.temp_depth = depth;
            }
            Expr::Path(path) => {
                let local = Self::local_of(body, expr).or_else(|| {
                    self.error("resolve", file, span, "expected a vector value");
                    None
                })?;
                if !self.types_agree(file, span, ty, self.local_ty(local)) {
                    return None;
                }
                body.used.insert(local);
                self.references.push(Reference {
                    file,
                    span: path.last().span,
                    def: local,
                });
                let source = self.local_ref(local);
                self.copy_value(dest, &source, ty, file, span, ops);
            }
            Expr::Place(place) => {
                let source = self.place_value(body, place, false)?;
                if !self.types_agree(file, span, ty, source.ty) {
                    return None;
                }
                self.copy_value(dest, &source.reference, ty, file, span, ops);
            }
            Expr::Call { path, args, .. } => {
                return self.emit_call(body, dest, ty, path, args, span, ops);
            }
            Expr::Project(value, field, _) => {
                return self.emit_projection(body, dest, ty, value, field, ops);
            }
            Expr::Method {
                receiver,
                name,
                type_arguments,
                args,
                ..
            } if name.name == "to_vec"
                && matches!(self.type_of(body, receiver), Ty::Struct(def) if matches!(self.defs[def.0].info, DefInfo::Array { .. })) =>
            {
                let builtin = crate::builtins::array_method("to_vec")?;
                self.references_builtin(file, name.span, builtin);
                self.bind_args(body, builtin, args, span)?;
                let array_ty = self.type_of(body, receiver);
                let (actual, _) = self.array_shape(array_ty)?;
                if !self.types_agree(file, span, element, actual) {
                    return None;
                }
                let depth = body.temp_depth;
                let source = self.temp(body);
                self.emit(body, &source, array_ty, receiver, ops)?;
                self.pack_value(
                    body,
                    Self::data_reference(dest),
                    &source,
                    array_ty,
                    span,
                    ops,
                );
                body.temp_depth = depth;
            }
            Expr::Method { receiver, .. }
                if matches!(self.type_of(body, receiver), Ty::List(_)) =>
            {
                return self.emit_list_method(body, dest, ty, expr, ops);
            }
            Expr::Method {
                receiver,
                name,
                type_arguments,
                args,
                ..
            } => {
                return self.emit_method(
                    body,
                    dest,
                    ty,
                    receiver,
                    name,
                    type_arguments,
                    args,
                    span,
                    ops,
                );
            }
            Expr::Index(..) => return self.emit_index(body, dest, ty, expr, ops),
            _ => {
                self.error("type", file, span, "expected a vector value");
                return None;
            }
        }
        Some(())
    }
}
