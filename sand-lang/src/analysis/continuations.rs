//! Retain the invocation's locals at each top-level wait boundary.
use super::*;
use crate::builtins::{self, BuiltinKind};
use std::collections::BTreeSet;

pub(super) struct WaitSite {
    ticks: u32,
    fields: Vec<pc::FieldReference>,
    span: Span,
}

impl WaitSite {
    pub(super) fn suspend(self, file: FileId, body: Vec<pc::Operation>) -> pc::Operation {
        op_at(
            pc::Action::Suspend {
                ticks: self.ticks,
                fields: self.fields,
                body,
            },
            file,
            self.span,
        )
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
        let ticks = match (bound.get("ticks"), bound.get("seconds")) {
            (Some(arg), None) => self.arg_int(body, arg, 1, i32::MAX as i64)? as u32,
            (None, Some(arg)) => 20 * self.arg_int(body, arg, 1, i32::MAX as i64 / 20)? as u32,
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
            ticks,
            fields,
            span,
        })
    }
}
