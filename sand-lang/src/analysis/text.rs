//! Runtime text uses owned NBT values, sharing aggregate copies and call frames.
use super::*;
use pc::DataModifyOperation;
use sand::data::NbtValue;

pub(super) const TEXT_LITERAL_BYTES: usize = 65_536;
pub(super) const DEFAULT_BYTES: usize = 1_048_576;

pub(super) fn defaults_cost(values: &[LeafDefault]) -> usize {
    values.iter().fold(0usize, |total, value| {
        total.saturating_add(match value {
            LeafDefault::Score(value) => value.to_string().len(),
            LeafDefault::Data(value) => value.to_string().len(),
        })
    })
}

#[derive(Clone)]
pub(super) enum LeafDefault {
    Score(i32),
    Data(NbtValue),
}

pub(super) fn text_value(value: &str) -> NbtValue {
    NbtValue::List(
        value
            .chars()
            .map(|character| NbtValue::String(character.to_string()))
            .collect(),
    )
}

impl Cx {
    /// Format in the negative domain so i32::MIN never needs an overflowing
    /// absolute value. Correct floor division to truncation for each digit.
    fn integer_text(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        source: &pc::FieldReference,
        signed: bool,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        let at = |action| op_at(action, file, span);
        self.register_data_slot(dest, text_value(""));
        let number = self.temp(body);
        let remainder = self.temp(body);
        let digit = self.temp(body);
        let digits = self.temp(body);
        self.register_data_slot(&digits, text_value(""));
        ops.push(at(Self::initialize_leaf(
            digits.clone(),
            LeafDefault::Data(text_value("0123456789")),
        )));
        ops.push(at(Self::initialize_leaf(
            dest.clone(),
            LeafDefault::Data(text_value("")),
        )));
        ops.push(at(pc::Action::ScoreOperation {
            score: number.clone(),
            operator: pc::ScoreOperator::Set,
            operand: pc::ScoreOperand::Field(source.clone()),
        }));
        ops.push(at(pc::Action::Branch {
            condition: pc::Condition::Range(pc::ScoreComparison {
                score: number.clone(),
                min: Some(1),
                max: None,
            }),
            then: vec![at(pc::Action::ScoreOperation {
                score: number.clone(),
                operator: pc::ScoreOperator::Mul,
                operand: pc::ScoreOperand::Value(-1),
            })],
            otherwise: Vec::new(),
        }));
        let iteration = vec![
            at(pc::Action::ScoreOperation {
                score: remainder.clone(),
                operator: pc::ScoreOperator::Set,
                operand: pc::ScoreOperand::Field(number.clone()),
            }),
            at(pc::Action::ScoreOperation {
                score: remainder.clone(),
                operator: pc::ScoreOperator::Mod,
                operand: pc::ScoreOperand::Value(10),
            }),
            at(pc::Action::ScoreOperation {
                score: number.clone(),
                operator: pc::ScoreOperator::Div,
                operand: pc::ScoreOperand::Value(10),
            }),
            at(pc::Action::ScoreSet {
                score: digit.clone(),
                value: 0,
            }),
            at(pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: remainder.clone(),
                    min: Some(1),
                    max: None,
                }),
                then: vec![
                    at(pc::Action::ScoreSet {
                        score: digit.clone(),
                        value: 10,
                    }),
                    at(pc::Action::ScoreOperation {
                        score: digit.clone(),
                        operator: pc::ScoreOperator::Sub,
                        operand: pc::ScoreOperand::Field(remainder),
                    }),
                    at(pc::Action::ScoreAdd {
                        score: number.clone(),
                        value: 1,
                    }),
                ],
                otherwise: Vec::new(),
            }),
            at(pc::Action::DataModify {
                target: Self::data_reference(dest),
                operator: DataModifyOperation::Prepend,
                source: pc::DataOperand::Reference {
                    value: Self::stored_element(&digits, &digit),
                },
            }),
        ];
        ops.push(at(pc::Action::While {
            condition: pc::Condition::Range(pc::ScoreComparison {
                score: number,
                min: None,
                max: Some(-1),
            }),
            body: iteration,
        }));
        ops.push(at(pc::Action::Branch {
            condition: pc::Condition::Range(pc::ScoreComparison {
                score: source.clone(),
                min: Some(0),
                max: Some(0),
            }),
            then: vec![at(Self::initialize_leaf(
                dest.clone(),
                LeafDefault::Data(text_value("0")),
            ))],
            otherwise: Vec::new(),
        }));
        if signed {
            ops.push(at(pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: source.clone(),
                    min: None,
                    max: Some(-1),
                }),
                then: vec![at(pc::Action::DataModify {
                    target: Self::data_reference(dest),
                    operator: DataModifyOperation::Prepend,
                    source: pc::DataOperand::Value {
                        value: NbtValue::String("-".into()),
                    },
                })],
                otherwise: Vec::new(),
            }));
        }
    }

    fn emit_to_string(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
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
        let builtin = crate::builtins::value_method(&name.name)?;
        self.references_builtin(body.file, name.span, builtin);
        self.bind_args(body, builtin, args, *span)?;
        let depth = body.temp_depth;
        let ty = self.type_of(body, receiver);
        let source = self.temp(body);
        self.emit(body, &source, ty, receiver, ops)?;
        let file = body.file;
        let at = |action| op_at(action, file, *span);
        match ty {
            Ty::Number => self.integer_text(body, dest, &source, true, *span, ops),
            Ty::Bool => ops.push(at(pc::Action::Branch {
                condition: pc::Condition::Range(pc::ScoreComparison {
                    score: source,
                    min: Some(0),
                    max: Some(0),
                }),
                then: vec![at(Self::initialize_leaf(
                    dest.clone(),
                    LeafDefault::Data(text_value("false")),
                ))],
                otherwise: vec![at(Self::initialize_leaf(
                    dest.clone(),
                    LeafDefault::Data(text_value("true")),
                ))],
            })),
            Ty::Enum(def) => {
                self.integer_text(body, dest, &source, true, *span, ops);
                let DefInfo::Enum { variants } = self.defs[def.0].info.clone() else {
                    unreachable!()
                };
                for variant in variants {
                    let DefInfo::Variant { value: Some(value) } = self.defs[variant.0].info else {
                        continue;
                    };
                    ops.push(at(pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: source.clone(),
                            min: Some(value as i32),
                            max: Some(value as i32),
                        }),
                        then: vec![at(Self::initialize_leaf(
                            dest.clone(),
                            LeafDefault::Data(text_value(&self.defs[variant.0].name)),
                        ))],
                        otherwise: Vec::new(),
                    }));
                }
            }
            Ty::Float => {
                let digits = self.temp(body);
                self.integer_text(body, &digits, &source, false, *span, ops);
                let length = self.temp(body);
                ops.push(at(Self::text_length(&digits, &length)));
                ops.push(at(pc::Action::While {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: length.clone(),
                        min: None,
                        max: Some(3),
                    }),
                    body: vec![
                        at(pc::Action::DataModify {
                            target: Self::data_reference(&digits),
                            operator: DataModifyOperation::Prepend,
                            source: pc::DataOperand::Value {
                                value: NbtValue::String("0".into()),
                            },
                        }),
                        at(pc::Action::ScoreAdd {
                            score: length.clone(),
                            value: 1,
                        }),
                    ],
                }));
                let zero = self.temp(body);
                ops.push(at(pc::Action::ScoreSet {
                    score: zero.clone(),
                    value: 0,
                }));
                let point = self.temp(body);
                ops.push(at(pc::Action::ScoreOperation {
                    score: point.clone(),
                    operator: pc::ScoreOperator::Set,
                    operand: pc::ScoreOperand::Field(length.clone()),
                }));
                ops.push(at(pc::Action::ScoreAdd {
                    score: point.clone(),
                    value: -3,
                }));
                self.slice_text(body, dest, &digits, (&zero, &point), *span, ops);
                ops.push(at(pc::Action::DataModify {
                    target: Self::data_reference(dest),
                    operator: DataModifyOperation::Append,
                    source: pc::DataOperand::Value {
                        value: NbtValue::String(".".into()),
                    },
                }));
                let fractional = self.temp(body);
                self.slice_text(body, &fractional, &digits, (&point, &length), *span, ops);
                ops.push(at(Self::append_text(dest, &fractional)));
                ops.push(at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: source,
                        min: None,
                        max: Some(-1),
                    }),
                    then: vec![at(pc::Action::DataModify {
                        target: Self::data_reference(dest),
                        operator: DataModifyOperation::Prepend,
                        source: pc::DataOperand::Value {
                            value: NbtValue::String("-".into()),
                        },
                    })],
                    otherwise: Vec::new(),
                }));
            }
            _ => unreachable!("scalar formatting is type checked"),
        }
        body.temp_depth = depth;
        Some(())
    }

    fn text_length(source: &pc::FieldReference, score: &pc::FieldReference) -> pc::Action {
        pc::Action::DataRead {
            score: score.clone(),
            source: Self::data_reference(source),
        }
    }

    pub(super) fn stored_element(
        source: &pc::FieldReference,
        index: &pc::FieldReference,
    ) -> pc::DataReference {
        let mut source = Self::data_reference(source);
        source.path.push(pc::DataProjection::Index {
            index: pc::ScoreOperand::Field(index.clone()),
        });
        source
    }

    pub(super) fn emit_text_index(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let Expr::Index(value, index, span) = expr else {
            unreachable!()
        };
        let depth = body.temp_depth;
        let source = self.temp(body);
        self.emit(body, &source, Ty::String, value, ops)?;
        let selected = self.temp(body);
        if Self::is_const_in(body, index)
            && self.eval_i32(body.module, body.file, index, Some(Ty::Number), None)? < 0
        {
            self.error(
                "index",
                body.file,
                index.span(),
                "string indices must be nonnegative",
            );
            return None;
        }
        self.emit(body, &selected, Ty::Number, index, ops)?;
        ops.push(op_at(
            Self::initialize_leaf(dest.clone(), LeafDefault::Data(text_value(""))),
            body.file,
            *span,
        ));
        ops.push(op_at(
            pc::Action::DataModify {
                target: Self::data_reference(dest),
                operator: DataModifyOperation::Append,
                source: pc::DataOperand::Reference {
                    value: Self::stored_element(&source, &selected),
                },
            },
            body.file,
            *span,
        ));
        body.temp_depth = depth;
        Some(())
    }

    /// Copy a clamped scalar-character range. Its source and bounds are already
    /// evaluated, and no user code runs during the copy.
    fn slice_text(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        source: &pc::FieldReference,
        bounds: (&pc::FieldReference, &pc::FieldReference),
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let (start, end) = bounds;
        let file = body.file;
        let at = |action| op_at(action, file, span);
        self.register_data_slot(dest, text_value(""));
        let counter = self.temp(body);
        let limit = self.temp(body);
        let length = self.temp(body);
        ops.push(at(Self::text_length(source, &length)));
        ops.push(at(pc::Action::ScoreOperation {
            score: counter.clone(),
            operator: pc::ScoreOperator::Set,
            operand: pc::ScoreOperand::Field(start.clone()),
        }));
        ops.push(at(pc::Action::ScoreOperation {
            score: counter.clone(),
            operator: pc::ScoreOperator::Max,
            operand: pc::ScoreOperand::Value(0),
        }));
        ops.push(at(pc::Action::ScoreOperation {
            score: limit.clone(),
            operator: pc::ScoreOperator::Set,
            operand: pc::ScoreOperand::Field(end.clone()),
        }));
        ops.push(at(pc::Action::ScoreOperation {
            score: limit.clone(),
            operator: pc::ScoreOperator::Min,
            operand: pc::ScoreOperand::Field(length),
        }));
        ops.push(at(Self::initialize_leaf(
            dest.clone(),
            LeafDefault::Data(text_value("")),
        )));
        ops.push(at(pc::Action::While {
            condition: pc::Condition::Relation(pc::ScoreRelation {
                left: counter.clone(),
                relation: pc::Relation::Lt,
                right: limit,
            }),
            body: vec![
                at(pc::Action::DataModify {
                    target: Self::data_reference(dest),
                    operator: DataModifyOperation::Append,
                    source: pc::DataOperand::Reference {
                        value: Self::stored_element(source, &counter),
                    },
                }),
                at(pc::Action::ScoreAdd {
                    score: counter,
                    value: 1,
                }),
            ],
        }));
    }

    pub(super) fn emit_text_method(
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
        } = expr
        else {
            unreachable!()
        };
        let Some(builtin) = crate::builtins::string_method(&name.name) else {
            self.error(
                "method",
                body.file,
                name.span,
                format!("strings have no method `{}`", name.name),
            );
            return None;
        };
        self.references_builtin(body.file, name.span, builtin);
        let bound = self.bind_args(body, builtin, args, *span)?;
        let result_ty = match name.name.as_str() {
            "len" => Ty::Number,
            "slice" => Ty::String,
            _ => Ty::Bool,
        };
        if !self.types_agree(body.file, *span, ty, result_ty) {
            return None;
        }
        let depth = body.temp_depth;
        let source = self.temp(body);
        self.emit(body, &source, Ty::String, receiver, ops)?;
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
            self.emit(
                body,
                &slot,
                if parameter == "value" {
                    Ty::String
                } else {
                    Ty::Number
                },
                value,
                ops,
            )?;
            values.insert(parameter, slot);
        }
        let file = body.file;
        let at = |action| op_at(action, file, *span);
        let length = self.temp(body);
        ops.push(at(Self::text_length(&source, &length)));
        match name.name.as_str() {
            "len" => ops.push(at(pc::Action::ScoreOperation {
                score: dest.clone(),
                operator: pc::ScoreOperator::Set,
                operand: pc::ScoreOperand::Field(length),
            })),
            "is_empty" => {
                ops.push(at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::Branch {
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
                }));
            }
            "slice" => self.slice_text(
                body,
                dest,
                &source,
                (&values["start"], &values["end"]),
                *span,
                ops,
            ),
            "starts_with" | "ends_with" => {
                let needle = &values["value"];
                let needle_length = self.temp(body);
                ops.push(at(Self::text_length(needle, &needle_length)));
                let start = self.temp(body);
                let end = self.temp(body);
                if name.name == "starts_with" {
                    ops.push(at(pc::Action::ScoreSet {
                        score: start.clone(),
                        value: 0,
                    }));
                    ops.push(at(pc::Action::ScoreOperation {
                        score: end.clone(),
                        operator: pc::ScoreOperator::Set,
                        operand: pc::ScoreOperand::Field(needle_length),
                    }));
                } else {
                    ops.push(at(pc::Action::ScoreOperation {
                        score: start.clone(),
                        operator: pc::ScoreOperator::Set,
                        operand: pc::ScoreOperand::Field(length.clone()),
                    }));
                    ops.push(at(pc::Action::ScoreOperation {
                        score: start.clone(),
                        operator: pc::ScoreOperator::Sub,
                        operand: pc::ScoreOperand::Field(needle_length),
                    }));
                    ops.push(at(pc::Action::ScoreOperation {
                        score: end.clone(),
                        operator: pc::ScoreOperator::Set,
                        operand: pc::ScoreOperand::Field(length),
                    }));
                }
                let part = self.temp(body);
                self.slice_text(body, &part, &source, (&start, &end), *span, ops);
                ops.push(at(pc::Action::DataEqual {
                    score: dest.clone(),
                    left: Self::data_reference(&part),
                    right: Self::data_reference(needle),
                }));
            }
            "contains" => {
                let needle = &values["value"];
                let needle_length = self.temp(body);
                ops.push(at(Self::text_length(needle, &needle_length)));
                let last = self.temp(body);
                ops.push(at(pc::Action::ScoreOperation {
                    score: last.clone(),
                    operator: pc::ScoreOperator::Set,
                    operand: pc::ScoreOperand::Field(length),
                }));
                ops.push(at(pc::Action::ScoreOperation {
                    score: last.clone(),
                    operator: pc::ScoreOperator::Sub,
                    operand: pc::ScoreOperand::Field(needle_length.clone()),
                }));
                let start = self.temp(body);
                let end = self.temp(body);
                let part = self.temp(body);
                let matched = self.temp(body);
                let running = self.temp(body);
                ops.push(at(pc::Action::ScoreSet {
                    score: dest.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::ScoreSet {
                    score: start.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::ScoreSet {
                    score: running.clone(),
                    value: 0,
                }));
                ops.push(at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: last.clone(),
                        min: Some(0),
                        max: None,
                    }),
                    then: vec![at(pc::Action::ScoreSet {
                        score: running.clone(),
                        value: 1,
                    })],
                    otherwise: Vec::new(),
                }));
                let mut iteration = vec![
                    at(pc::Action::ScoreOperation {
                        score: end.clone(),
                        operator: pc::ScoreOperator::Set,
                        operand: pc::ScoreOperand::Field(start.clone()),
                    }),
                    at(pc::Action::ScoreOperation {
                        score: end.clone(),
                        operator: pc::ScoreOperator::Add,
                        operand: pc::ScoreOperand::Field(needle_length),
                    }),
                ];
                self.slice_text(body, &part, &source, (&start, &end), *span, &mut iteration);
                iteration.push(at(pc::Action::DataEqual {
                    score: matched.clone(),
                    left: Self::data_reference(&part),
                    right: Self::data_reference(needle),
                }));
                iteration.push(at(pc::Action::Branch {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: matched,
                        min: Some(1),
                        max: Some(1),
                    }),
                    then: vec![
                        at(pc::Action::ScoreSet {
                            score: dest.clone(),
                            value: 1,
                        }),
                        at(pc::Action::ScoreSet {
                            score: running.clone(),
                            value: 0,
                        }),
                    ],
                    otherwise: Vec::new(),
                }));
                // Check the last index before incrementing, avoiding wrap at the
                // scoreboard boundary even for an extremely large stored list.
                iteration.push(at(pc::Action::Branch {
                    condition: pc::Condition::Relation(pc::ScoreRelation {
                        left: start.clone(),
                        relation: pc::Relation::Ge,
                        right: last,
                    }),
                    then: vec![at(pc::Action::ScoreSet {
                        score: running.clone(),
                        value: 0,
                    })],
                    otherwise: vec![at(pc::Action::ScoreAdd {
                        score: start,
                        value: 1,
                    })],
                }));
                ops.push(at(pc::Action::While {
                    condition: pc::Condition::Range(pc::ScoreComparison {
                        score: running,
                        min: Some(1),
                        max: Some(1),
                    }),
                    body: iteration,
                }));
            }
            _ => unreachable!(),
        }
        body.temp_depth = depth;
        Some(())
    }

    pub(super) fn data_field(base: &pc::FieldReference) -> pc::FieldReference {
        pc::FieldReference {
            state: base.state.clone(),
            field: format!("__sand_data_{}", base.field),
        }
    }

    pub(super) fn data_reference(base: &pc::FieldReference) -> pc::DataReference {
        pc::DataReference {
            field: Self::data_field(base),
            path: Vec::new(),
        }
    }

    pub(super) fn register_data_slot(&mut self, base: &pc::FieldReference, default: NbtValue) {
        if base.state.to_string() == self.locals_state_id() {
            self.local_data_fields
                .entry(Self::data_field(base).field)
                .or_insert(default);
        }
    }

    pub(super) fn initialize_leaf(score: pc::FieldReference, value: LeafDefault) -> pc::Action {
        match value {
            LeafDefault::Score(value) => pc::Action::ScoreSet { score, value },
            LeafDefault::Data(value) => pc::Action::DataModify {
                target: Self::data_reference(&score),
                operator: DataModifyOperation::Set,
                source: pc::DataOperand::Value { value },
            },
        }
    }

    pub(super) fn copy_data(dest: &pc::FieldReference, source: &pc::FieldReference) -> pc::Action {
        pc::Action::DataModify {
            target: Self::data_reference(dest),
            operator: DataModifyOperation::Set,
            source: pc::DataOperand::Reference {
                value: Self::data_reference(source),
            },
        }
    }

    pub(super) fn append_text(
        dest: &pc::FieldReference,
        source: &pc::FieldReference,
    ) -> pc::Action {
        let mut value = Self::data_reference(source);
        value.path.push(pc::DataProjection::Each);
        pc::Action::DataModify {
            target: Self::data_reference(dest),
            operator: DataModifyOperation::Append,
            source: pc::DataOperand::Reference { value },
        }
    }

    pub(super) fn constant_text(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        stack: &mut Vec<DefId>,
    ) -> Option<String> {
        self.constant_text_depth(module, file, expr, stack, 0)
    }

    fn constant_text_depth(
        &mut self,
        module: ModuleId,
        file: FileId,
        expr: &Expr,
        stack: &mut Vec<DefId>,
        depth: usize,
    ) -> Option<String> {
        if depth > 64 {
            self.error(
                "const",
                file,
                expr.span(),
                "constant string expansion exceeds 64 levels",
            );
            return None;
        }
        match expr {
            Expr::Str { value, .. } => {
                if !self.text_size(file, expr.span(), value.len()) {
                    return None;
                }
                Some(value.clone())
            }
            Expr::Binary(left, BinOp::Add, right, _) => {
                let mut value = self.constant_text_depth(module, file, left, stack, depth + 1)?;
                let right = self.constant_text_depth(module, file, right, stack, depth + 1)?;
                if !self.text_size(file, expr.span(), value.len().saturating_add(right.len())) {
                    return None;
                }
                value.push_str(&right);
                Some(value)
            }
            Expr::Path(path) => {
                let def = self.expect_path(module, path, &[DefKind::Const], "a string constant")?;
                if stack.contains(&def) {
                    self.error(
                        "cycle",
                        file,
                        path.span,
                        "cycle detected while evaluating a string constant",
                    );
                    return None;
                }
                if let Some(value) = self.text_constants.get(&def) {
                    return Some(value.clone());
                }
                let source = &self.const_src[&def];
                if source.ty != Ty::String {
                    self.error("type", file, path.span, "expected a string constant");
                    return None;
                }
                let (module, file, expr) = (source.module, source.file, source.expr.clone());
                stack.push(def);
                let value = self.constant_text_depth(module, file, &expr, stack, depth + 1);
                stack.pop();
                value
            }
            _ => {
                self.error(
                    "const",
                    file,
                    expr.span(),
                    "expected a constant string expression",
                );
                None
            }
        }
    }

    fn text_size(&mut self, file: FileId, span: Span, bytes: usize) -> bool {
        if bytes > TEXT_LITERAL_BYTES {
            self.error(
                "const",
                file,
                span,
                "text literals and expanded constants support at most 65536 UTF-8 bytes",
            );
            return false;
        }
        true
    }

    pub(super) fn default_size(&mut self, file: FileId, span: Span, bytes: usize) -> bool {
        if bytes > DEFAULT_BYTES {
            self.error(
                "const",
                file,
                span,
                "expanded State default exceeds 1048576 bytes; build larger values at runtime",
            );
            return false;
        }
        true
    }

    pub(super) fn emit_text(
        &mut self,
        body: &mut BodyCx,
        dest: &pc::FieldReference,
        expr: &Expr,
        ops: &mut Vec<pc::Operation>,
    ) -> Option<()> {
        let span = expr.span();
        let found = self.type_of(body, expr);
        if !self.types_agree(body.file, span, Ty::String, found) {
            return None;
        }
        self.register_data_slot(dest, text_value(""));
        match expr {
            Expr::Str { value, .. } => {
                if !self.text_size(body.file, span, value.len()) {
                    return None;
                }
                ops.push(op_at(
                    Self::initialize_leaf(dest.clone(), LeafDefault::Data(text_value(value))),
                    body.file,
                    span,
                ));
            }
            Expr::Binary(left, BinOp::Add, right, _) => {
                let depth = body.temp_depth;
                let lhs = self.temp(body);
                self.emit(body, &lhs, Ty::String, left, ops)?;
                let rhs = self.temp(body);
                self.emit(body, &rhs, Ty::String, right, ops)?;
                ops.push(op_at(Self::append_text(&lhs, &rhs), body.file, span));
                self.copy_value(dest, &lhs, Ty::String, body.file, span, ops);
                body.temp_depth = depth;
            }
            Expr::Path(path) => {
                if let Some(local) = Self::local_of(body, expr) {
                    body.used.insert(local);
                    self.references.push(Reference {
                        file: body.file,
                        span: path.last().span,
                        def: local,
                    });
                    let source = self.local_ref(local);
                    self.copy_value(dest, &source, Ty::String, body.file, span, ops);
                } else {
                    let value =
                        self.constant_text(body.module, body.file, expr, &mut Vec::new())?;
                    ops.push(op_at(
                        Self::initialize_leaf(dest.clone(), LeafDefault::Data(text_value(&value))),
                        body.file,
                        span,
                    ));
                }
            }
            Expr::Place(place) => {
                let source = self.place_value(body, place, false)?;
                self.copy_value(dest, &source.reference, Ty::String, body.file, span, ops);
            }
            Expr::Project(value, field, _) => {
                return self.emit_projection(body, dest, Ty::String, value, field, ops);
            }
            Expr::Index(value, _, _) if self.type_of(body, value) != Ty::String => {
                return self.emit_index(body, dest, Ty::String, expr, ops);
            }
            Expr::Index(..) => return self.emit_text_index(body, dest, expr, ops),
            Expr::Method { receiver, .. }
                if matches!(self.type_of(body, receiver), Ty::List(_)) =>
            {
                return self.emit_list_method(body, dest, Ty::String, expr, ops);
            }
            Expr::Method { receiver, .. } if self.type_of(body, receiver) == Ty::String => {
                return self.emit_text_method(body, dest, Ty::String, expr, ops);
            }
            Expr::Call { path, args, .. } => {
                return self.emit_call(body, dest, Ty::String, path, args, span, ops);
            }
            Expr::Method { receiver, name, .. }
                if name.name == "to_string"
                    && matches!(
                        self.type_of(body, receiver),
                        Ty::Number | Ty::Float | Ty::Bool | Ty::Enum(_)
                    ) =>
            {
                return self.emit_to_string(body, dest, expr, ops);
            }
            Expr::Method {
                receiver,
                name,
                args,
                ..
            } if self.type_of(body, receiver) != Ty::String => {
                let (path, args) = self.method_call(body, receiver, name, args)?;
                return self.emit_call(body, dest, Ty::String, &path, &args, span, ops);
            }
            _ => {
                self.error("type", body.file, span, "expected a string expression");
                return None;
            }
        }
        Some(())
    }
}
