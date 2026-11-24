//! Retain each invocation across wait boundaries and join conditional paths.
use super::*;
use crate::builtins::{self, BuiltinKind};
use std::collections::BTreeSet;

pub(super) struct WaitSite {
    setup: Vec<pc::Operation>,
    ticks: pc::ScoreOperand,
    fields: Vec<pc::FieldReference>,
    group: Option<String>,
    span: Span,
}

impl WaitSite {
    pub(super) fn suspend(mut self, file: FileId, body: Vec<pc::Operation>) -> Vec<pc::Operation> {
        self.setup.push(op_at(
            pc::Action::Suspend {
                ticks: self.ticks,
                fields: self.fields,
                body,
                group: self.group,
            },
            file,
            self.span,
        ));
        self.setup
    }
}

impl Cx {
    pub(super) fn is_wait(&self, body: &BodyCx, path: &Path) -> bool {
        path.segments.len() == 1
            && path.last().name == "wait"
            && Self::lookup_local(body, "wait").is_none()
            && !self.scopes[body.module.0].contains_key("wait")
    }

    pub(super) fn lower_wait(
        &mut self,
        body: &mut BodyCx,
        path: &Path,
        args: &[Arg],
        span: Span,
    ) -> Option<WaitSite> {
        let file = body.file;
        let builtin = builtins::find("wait", BuiltinKind::Statement).expect("registered builtin");
        self.references_builtin(file, path.span, builtin);
        if body.owner.is_none()
            || body.wait_block_depth != body.block_ends.len()
            || body.ret.is_some()
            || body.receiver_out.is_some()
        {
            self.error("wait", file, span, "use `wait` directly in a function body or an if/else branch, match arm, or local loop, with no return value or mutable borrowed receiver");
            return None;
        }
        let bound = self.bind_args(body, builtin, args, span)?;
        let group = self.wait_group(file, bound.get("group").copied())?;
        let (duration, scale) = match (bound.get("ticks"), bound.get("seconds")) {
            (Some(arg), None) => (*arg, 1),
            (None, Some(arg)) => (*arg, 20),
            _ => {
                self.error(
                    "wait",
                    file,
                    span,
                    "provide exactly one of `ticks` or `seconds`",
                );
                return None;
            }
        };
        let ArgValue::Expr(expr) = &duration.value else {
            self.error(
                "type",
                file,
                duration.span,
                "expected a number duration, found a range",
            );
            return None;
        };
        let mut setup = Vec::new();
        let max = i32::MAX / scale;
        let ticks = if Self::is_const_in(body, expr) {
            pc::ScoreOperand::Value(self.arg_int(body, duration, 1, max.into())? as i32 * scale)
        } else {
            let found = self.type_of(body, expr);
            if !self.types_agree(file, expr.span(), Ty::Number, found) {
                return None;
            }
            let depth = body.temp_depth;
            let score = self.temp(body);
            let result = self.emit(body, &score, Ty::Number, expr, &mut setup);
            body.temp_depth = depth;
            result?;
            if scale != 1 {
                setup.push(op_at(
                    pc::Action::Branch {
                        condition: pc::Condition::Range(pc::ScoreComparison {
                            score: score.clone(),
                            min: Some(1),
                            max: Some(max),
                        }),
                        then: vec![op_at(
                            pc::Action::ScoreOperation {
                                score: score.clone(),
                                operator: pc::ScoreOperator::Mul,
                                operand: pc::ScoreOperand::Value(scale),
                            },
                            file,
                            span,
                        )],
                        otherwise: vec![op_at(
                            pc::Action::ScoreSet {
                                score: score.clone(),
                                value: 0,
                            },
                            file,
                            span,
                        )],
                    },
                    file,
                    span,
                ));
            }
            pc::ScoreOperand::Field(score)
        };
        body.note_executor(span, "retains the executing entity across a wait".into());

        // An outer binding hidden by a branch local becomes visible again
        // after the branch joins. A later declaration in the same scope
        // permanently hides the earlier one.
        let mut seen = BTreeSet::new();
        let mut definitions = BTreeSet::new();
        for scope in body.scopes.iter().rev() {
            if body.block_ends.len() > 1 {
                seen.clear();
            }
            definitions.extend(
                scope
                    .iter()
                    .rev()
                    .filter_map(|(name, def)| seen.insert(name).then_some(*def)),
            );
        }
        let mut fields = body.wait_slots.clone();
        for def in definitions {
            let root = self.local_ref(def);
            let ty = self.local_ty(def);
            fields.extend(self.retained_slots(&root, ty));
        }
        if fields.len() > 64 {
            self.error(
                "wait",
                file,
                span,
                "a wait can retain at most 64 local fields, including fields of records",
            );
            return None;
        }
        Some(WaitSite {
            setup,
            ticks,
            fields,
            group,
            span,
        })
    }

    pub(super) fn wait_group(
        &mut self,
        file: FileId,
        group: Option<&Arg>,
    ) -> Option<Option<String>> {
        match group {
            Some(group) => self
                .arg_string(file, group, "a wait group name")
                .map(|(name, _, _)| Some(name)),
            None => Some(None),
        }
    }

    pub(super) fn retained_slots(
        &mut self,
        root: &pc::FieldReference,
        ty: Ty,
    ) -> Vec<pc::FieldReference> {
        self.value_slots(root, ty)
            .into_iter()
            .map(|(field, ty)| {
                if matches!(ty, Ty::String | Ty::List(_) | Ty::Callable(_)) {
                    Self::data_reference(&field).field
                } else {
                    field
                }
            })
            .collect()
    }
}

/// Whether control flow in this function frame can suspend. Selector bodies
/// and value blocks are separate frames.
pub(super) fn may_suspend(operations: &[pc::Operation]) -> bool {
    operations.iter().any(|operation| match &operation.action {
        pc::Action::Suspend { .. } => true,
        pc::Action::While { body, .. } => may_suspend(body),
        pc::Action::Branch {
            then, otherwise, ..
        } => may_suspend(then) || may_suspend(otherwise),
        _ => false,
    })
}

/// Outline shared suffixes once. Reaching the end of either branch calls its
/// suffix, and suspending retains the suffix call for the resumed path.
pub(super) fn join_wait_paths(function: &mut pc::Function) -> Vec<pc::Function> {
    if !may_suspend(&function.body) {
        return Vec::new();
    }
    let mut joins = WaitJoins {
        id: &function.id,
        context: function.context,
        functions: Vec::new(),
    };
    joins.rewrite(&mut function.body, None);
    joins.functions
}

struct WaitJoins<'a> {
    id: &'a sand::ResourceLocation,
    context: pc::ExecutionContext,
    functions: Vec<pc::Function>,
}

impl WaitJoins<'_> {
    fn rewrite(&mut self, operations: &mut Vec<pc::Operation>, fallthrough: Option<pc::Operation>) {
        if !may_suspend(operations) {
            if !matches!(
                operations.last().map(|op| &op.action),
                Some(pc::Action::Return)
            ) {
                operations.extend(fallthrough);
            }
            return;
        }
        let mut suffix: Vec<_> = fallthrough.into_iter().collect();
        for mut operation in std::mem::take(operations).into_iter().rev() {
            match &mut operation.action {
                pc::Action::Suspend { body, .. } => {
                    let fallthrough = self.take_suffix(&mut suffix);
                    self.rewrite(body, fallthrough);
                }
                pc::Action::While { condition, body } if may_suspend(body) => {
                    let fallthrough = self.take_suffix(&mut suffix);
                    let index = self.outline(Vec::new());
                    let repeat = self.call(index, operation.origin.clone());
                    self.rewrite(body, Some(repeat.clone()));
                    self.functions[index].body = vec![pc::Operation {
                        action: pc::Action::Branch {
                            condition: condition.clone(),
                            then: std::mem::take(body),
                            otherwise: fallthrough.into_iter().collect(),
                        },
                        origin: operation.origin,
                    }];
                    operation = repeat;
                }
                pc::Action::Branch {
                    then, otherwise, ..
                } if may_suspend(then) || may_suspend(otherwise) => {
                    let fallthrough = self.take_suffix(&mut suffix);
                    self.rewrite(then, fallthrough.clone());
                    self.rewrite(otherwise, fallthrough);
                }
                _ => {}
            }
            suffix.push(operation);
        }
        suffix.reverse();
        *operations = suffix;
    }

    fn take_suffix(&mut self, reversed: &mut Vec<pc::Operation>) -> Option<pc::Operation> {
        match reversed.len() {
            0 => None,
            1 if matches!(reversed[0].action, pc::Action::Call { .. }) => reversed.pop(),
            _ => {
                reversed.reverse();
                let origin = reversed
                    .first()
                    .and_then(|operation| operation.origin.clone());
                let index = self.outline(std::mem::take(reversed));
                Some(self.call(index, origin))
            }
        }
    }

    fn outline(&mut self, body: Vec<pc::Operation>) -> usize {
        let index = self.functions.len();
        let id = sand::ResourceLocation::new(
            self.id.namespace(),
            format!("__wait/{index}/{}", self.id.path()),
        )
        .expect("validated continuation identity");
        self.functions.push(pc::Function {
            id,
            context: self.context,
            body,
        });
        index
    }

    fn call(&self, index: usize, origin: Option<serde_json::Value>) -> pc::Operation {
        pc::Operation {
            action: pc::Action::Call {
                function: pc::FunctionReference::Internal {
                    id: self.functions[index].id.clone(),
                },
            },
            origin,
        }
    }
}
