//! Vector intrinsics operate on ordinary typed values and writable addresses.
use super::*;
use pc::DataModifyOperation;
use sand::data::NbtValue;

impl Cx {
    pub(super) fn emit_list_method(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        ty: Ty,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        self.list_method(body, Some((dest, ty)), expr, ops)
    }

    pub(super) fn lower_list_statement(
        &mut self,
        body: &mut BodyCx,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) {
        let _ = self.list_method(body, None, expr, ops);
    }

    fn list_method(
        &mut self,
        body: &mut BodyCx,
        result: Option<(&pc::FieldReference, Ty)>,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let Expr::Method {
            receiver,
            name,
            args,
            span,
        } = expr
        else {
            unreachable!()
        };
        let list_ty = self.type_of(body, receiver);
        let element = self.list_element(list_ty)?;
        let Some(builtin) = crate::builtins::list_method(&name.name) else {
            self.error(
                "method",
                body.file,
                name.span,
                format!("vectors have no method `{}`", name.name),
            );
            return None;
        };
        self.references_builtin(body.file, name.span, builtin);
        let bound = self.bind_args(body, builtin, args, *span)?;
        let result_ty = match name.name.as_str() {
            "len" | "extend" => Ty::Number,
            "is_empty" | "contains" | "push" | "insert" => Ty::Bool,
            "get_or" | "pop_or" | "remove_or" => element,
            _ => Ty::Unknown,
        };
        if let Some((_, expected)) = result {
            if builtin.kind == crate::builtins::BuiltinKind::Statement {
                self.error(
                    "type",
                    body.file,
                    *span,
                    "this vector method does not return a value",
                );
                return None;
            }
            if !self.types_agree(body.file, *span, expected, result_ty) {
                return None;
            }
        }
        let depth = body.temp_depth;
        let mutable = matches!(
            name.name.as_str(),
            "push" | "pop_or" | "remove_or" | "insert" | "extend" | "clear" | "truncate"
        );
        let address = if mutable {
            Some(self.value_address(body, receiver, ops)?)
        } else {
            None
        };
        let source = self.temp(body);
        if let Some(address) = &address {
            self.read_address(body, &source, address, *span, ops)?;
        } else {
            self.emit(body, &source, list_ty, receiver, ops)?;
        }
        let mut values = BTreeMap::new();
        for argument in args {
            let (&parameter, _) = bound
                .iter()
                .find(|(_, value)| std::ptr::eq(**value, argument))?;
            let ArgValue::Expr(value) = &argument.value else {
                self.error(
                    "argument",
                    body.file,
                    argument.span,
                    "this method argument needs a value expression",
                );
                return None;
            };
            let slot = self.temp(body);
            let ty = match parameter {
                "index" | "length" => Ty::Number,
                "values" => list_ty,
                _ => element,
            };
            self.emit(body, &slot, ty, value, ops)?;
            values.insert(parameter, slot);
        }
        let ignored = self.temp(body);
        let dest = result.map_or(&ignored, |(dest, _)| dest);
        let file = body.file;
        let at = |action| op_at(action, file, *span);
        if result_ty != Ty::Unknown {
            let defaults = self.record_constant(body.module, file, None, result_ty)?;
            for ((score, _), value) in self.value_slots(dest, result_ty).into_iter().zip(defaults) {
                ops.push(at(Self::initialize_leaf(score, value)));
            }
        }
        let invocation_start = ops.len();
        let length = self.temp(body);
        ops.push(at(pc::Action::DataRead {
            score: length.clone(),
            source: Self::data_reference(&source),
        }));
        match name.name.as_str() {
            "len" => self.copy_value(dest, &length, Ty::Number, file, *span, ops),
            "is_empty" => ops.push(at(pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: length,
                    min: Some(0),
                    max: Some(0),
                }),
                then: vec![at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 1,
                })],
                otherwise: Vec::new(),
            })),
            "get_or" | "pop_or" | "remove_or" => {
                self.copy_value(dest, &values["fallback"], element, file, *span, ops);
                let selected = if name.name == "pop_or" {
                    let selected = self.temp(body);
                    self.copy_value(&selected, &length, Ty::Number, file, *span, ops);
                    ops.push(at(pc::Action::ScoreAdd {
                        score: selected.clone(),
                        value: -1,
                    }));
                    selected
                } else {
                    values["index"].clone()
                };
                let valid = self.storage_index_valid(
                    body,
                    Self::data_reference(&source),
                    &selected,
                    *span,
                    ops,
                );
                let selected_value = Self::stored_element(&source, &selected);
                let mut then = Vec::new();
                self.unpack_value(
                    body,
                    dest,
                    selected_value.clone(),
                    element,
                    *span,
                    &mut then,
                );
                if mutable {
                    then.push(at(pc::Action::DataRemove {
                        target: selected_value,
                    }));
                }
                ops.push(at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: valid,
                        min: Some(1),
                        max: Some(1),
                    }),
                    then,
                    otherwise: Vec::new(),
                }));
                if let Some(address) = &address {
                    self.write_address(body, address, &source, *span, ops);
                }
            }
            "contains" => {
                let needle = self.temp(body);
                self.register_data_slot(&needle, NbtValue::List(Vec::new()));
                self.pack_value(
                    body,
                    Self::data_reference(&needle),
                    &values["value"],
                    element,
                    *span,
                    ops,
                );
                let counter = self.temp(body);
                let equal = self.temp(body);
                ops.push(at(pc::Action::ScoreSet {
                    score: counter.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::While {
                    condition: pc::Condition::Relation(pc::ScoreRelation {
                        left: counter.clone(),
                        relation: pc::Relation::Lt,
                        right: length.clone(),
                    }),
                    body: vec![
                        at(pc::Action::DataEqual {
                            score: equal.clone(),
                            left: Self::stored_element(&source, &counter),
                            right: Self::data_reference(&needle),
                        }),
                        at(pc::Action::Branch {
                            condition: pc::Condition::Range(pc::ScoreComparison {
                                score: equal,
                                min: Some(1),
                                max: Some(1),
                            }),
                            then: vec![
                                at(pc::Action::ScoreSet {
                                    score: dest.clone(),
                                    value: 1,
                                }),
                                at(pc::Action::ScoreOperation {
                                    score: counter.clone(),
                                    operator: pc::ScoreOperator::Set,
                                    operand: pc::ScoreOperand::Field(length),
                                }),
                            ],
                            otherwise: vec![at(pc::Action::ScoreAdd {
                                score: counter,
                                value: 1,
                            })],
                        }),
                    ],
                }));
            }
            "push" => {
                let packed = self.temp(body);
                self.register_data_slot(&packed, NbtValue::List(Vec::new()));
                let mut then = Vec::new();
                self.pack_value(
                    body,
                    Self::data_reference(&packed),
                    &values["value"],
                    element,
                    *span,
                    &mut then,
                );
                then.push(at(pc::Action::DataModify {
                    target: Self::data_reference(&source),
                    operator: DataModifyOperation::Append,
                    source: pc::DataOperand::Reference {
                        value: Self::data_reference(&packed),
                    },
                }));
                self.write_address(body, address.as_ref()?, &source, *span, &mut then);
                then.push(at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 1,
                }));
                ops.push(at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: length,
                        min: Some(0),
                        max: Some(super::sequences::LIST_LIMIT as i32 - 1),
                    }),
                    then,
                    otherwise: Vec::new(),
                }));
            }
            "clear" => {
                ops.push(at(Self::initialize_leaf(
                    source.clone(),
                    text::LeafDefault::Data(NbtValue::List(Vec::new())),
                )));
                self.write_address(body, address.as_ref()?, &source, *span, ops);
            }
            "truncate" => {
                let limit = &values["length"];
                ops.push(at(pc::Action::ScoreOperation {
                    score: limit.clone(),
                    operator: pc::ScoreOperator::Max,
                    operand: pc::ScoreOperand::Value(0),
                }));
                let last = self.temp(body);
                ops.push(at(pc::Action::While {
                    condition: pc::Condition::Relation(pc::ScoreRelation {
                        left: length.clone(),
                        relation: pc::Relation::Gt,
                        right: limit.clone(),
                    }),
                    body: vec![
                        at(pc::Action::ScoreOperation {
                            score: last.clone(),
                            operator: pc::ScoreOperator::Set,
                            operand: pc::ScoreOperand::Field(length.clone()),
                        }),
                        at(pc::Action::ScoreAdd {
                            score: last.clone(),
                            value: -1,
                        }),
                        at(pc::Action::DataRemove {
                            target: Self::stored_element(&source, &last),
                        }),
                        at(pc::Action::ScoreAdd {
                            score: length,
                            value: -1,
                        }),
                    ],
                }));
                self.write_address(body, address.as_ref()?, &source, *span, ops);
            }
            "extend" => {
                let other = &values["values"];
                let limit = self.temp(body);
                ops.push(at(pc::Action::ScoreSet {
                    score: limit.clone(),
                    value: super::sequences::LIST_LIMIT as i32,
                }));
                ops.push(at(pc::Action::ScoreOperation {
                    score: limit.clone(),
                    operator: pc::ScoreOperator::Sub,
                    operand: pc::ScoreOperand::Field(length),
                }));
                let other_length = self.temp(body);
                ops.push(at(pc::Action::DataRead {
                    score: other_length.clone(),
                    source: Self::data_reference(other),
                }));
                ops.push(at(pc::Action::ScoreOperation {
                    score: limit.clone(),
                    operator: pc::ScoreOperator::Min,
                    operand: pc::ScoreOperand::Field(other_length),
                }));
                ops.push(at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::While {
                    condition: pc::Condition::Relation(pc::ScoreRelation {
                        left: dest.clone(),
                        relation: pc::Relation::Lt,
                        right: limit,
                    }),
                    body: vec![
                        at(pc::Action::DataModify {
                            target: Self::data_reference(&source),
                            operator: DataModifyOperation::Append,
                            source: pc::DataOperand::Reference {
                                value: Self::stored_element(other, dest),
                            },
                        }),
                        at(pc::Action::ScoreAdd {
                            score: dest.clone(),
                            value: 1,
                        }),
                    ],
                }));
                self.write_address(body, address.as_ref()?, &source, *span, ops);
            }
            "insert" => {
                let packed = self.temp(body);
                self.register_data_slot(&packed, NbtValue::List(Vec::new()));
                let output = self.temp(body);
                self.register_data_slot(&output, NbtValue::List(Vec::new()));
                let counter = self.temp(body);
                let index = &values["index"];
                let mut then = Vec::new();
                self.pack_value(
                    body,
                    Self::data_reference(&packed),
                    &values["value"],
                    element,
                    *span,
                    &mut then,
                );
                then.push(at(Self::initialize_leaf(
                    output.clone(),
                    text::LeafDefault::Data(NbtValue::List(Vec::new())),
                )));
                then.push(at(pc::Action::ScoreSet {
                    score: counter.clone(),
                    value: 0,
                }));
                then.push(at(pc::Action::While {
                    condition: pc::Condition::Relation(pc::ScoreRelation {
                        left: counter.clone(),
                        relation: pc::Relation::Le,
                        right: length.clone(),
                    }),
                    body: vec![
                        at(pc::Action::Branch {
                            condition: pc::Condition::Relation(pc::ScoreRelation {
                                left: counter.clone(),
                                relation: pc::Relation::Eq,
                                right: index.clone(),
                            }),
                            then: vec![at(pc::Action::DataModify {
                                target: Self::data_reference(&output),
                                operator: DataModifyOperation::Append,
                                source: pc::DataOperand::Reference {
                                    value: Self::data_reference(&packed),
                                },
                            })],
                            otherwise: Vec::new(),
                        }),
                        at(pc::Action::Branch {
                            condition: pc::Condition::Relation(pc::ScoreRelation {
                                left: counter.clone(),
                                relation: pc::Relation::Lt,
                                right: length.clone(),
                            }),
                            then: vec![at(pc::Action::DataModify {
                                target: Self::data_reference(&output),
                                operator: DataModifyOperation::Append,
                                source: pc::DataOperand::Reference {
                                    value: Self::stored_element(&source, &counter),
                                },
                            })],
                            otherwise: Vec::new(),
                        }),
                        at(pc::Action::ScoreAdd {
                            score: counter,
                            value: 1,
                        }),
                    ],
                }));
                self.write_address(body, address.as_ref()?, &output, *span, &mut then);
                then.push(at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 1,
                }));
                let room = pc::Condition::Range(pc::ScoreComparison {
                    score: length.clone(),
                    min: Some(0),
                    max: Some(super::sequences::LIST_LIMIT as i32 - 1),
                });
                let nonnegative = pc::Condition::Range(pc::ScoreComparison {
                    score: index.clone(),
                    min: Some(0),
                    max: None,
                });
                let within = pc::Condition::Relation(pc::ScoreRelation {
                    left: index.clone(),
                    relation: pc::Relation::Le,
                    right: length,
                });
                ops.push(at(pc::Action::Branch {
                    condition: room,
                    then: vec![at(pc::Action::Branch {
                        condition: nonnegative,
                        then: vec![at(pc::Action::Branch {
                            condition: within,
                            then,
                            otherwise: Vec::new(),
                        })],
                        otherwise: Vec::new(),
                    })],
                    otherwise: Vec::new(),
                }));
            }
            _ => unreachable!("documented vector methods have lowering"),
        }
        if let Some(address) = &address
            && self.address_is_guarded(address)
        {
            let invocation = ops.split_off(invocation_start);
            ops.extend(self.guard_address(body, address, *span, invocation));
        }
        body.temp_depth = depth;
        Some(())
    }
}
