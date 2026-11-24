//! Retain the invocation's locals at each top-level wait boundary.
use super::*;
use crate::builtins::{self, BuiltinKind};
use std::collections::BTreeSet;

pub(super) struct WaitSite {
    setup: Vec<pc::Operation>,
    ticks: pc::ScoreOperand,
    fields: Vec<pc::FieldReference>,
    span: Span,
}

impl WaitSite {
    pub(super) fn suspend(mut self, file: FileId, body: Vec<pc::Operation>) -> Vec<pc::Operation> {
        self.setup.push(op_at(
            pc::Action::Suspend {
                ticks: self.ticks,
                fields: self.fields,
                body,
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
            || body.block_ends.len() != 1
            || body.ret.is_some()
            || body.receiver_out.is_some()
        {
            self.error("wait", file, span, "use `wait` directly in a function body that returns no value and has no mutable borrowed receiver");
            return None;
        }
        let bound = self.bind_args(body, builtin, args, span)?;
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

        // Retain the live lexical slots, including parameters and destructured
        // locals. Reused command scratch slots never escape the expression that
        // owns them. Shadowed bindings will stay unreachable on resume.
        let mut seen = BTreeSet::new();
        let definitions: BTreeSet<_> = body
            .scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.iter().rev())
            .filter_map(|(name, def)| seen.insert(name).then_some(*def))
            .collect();
        let mut fields = Vec::new();
        for def in definitions {
            let root = self.local_ref(def);
            let ty = self.local_ty(def);
            fields.extend(self.value_slots(&root, ty).into_iter().map(|(field, ty)| {
                if matches!(ty, Ty::String | Ty::List(_) | Ty::Callable(_)) {
                    Self::data_reference(&field).field
                } else {
                    field
                }
            }));
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
            span,
        })
    }
}
