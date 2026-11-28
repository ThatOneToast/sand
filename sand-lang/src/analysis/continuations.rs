//! Retain each invocation across wait boundaries and join conditional paths.
use super::*;
use crate::builtins::{self, BuiltinKind};
use std::collections::BTreeSet;

const CALL_MARKER: &str = "__sand_wait_call/";
// Specialisation must be bounded before the portable backend can enforce its
// wait-site limit. A branching chain of returning functions can grow quickly.
pub(super) const MAX_COMPOSED_CALLS: usize = 256;

/// One call whose result or receiver output may be produced after a wait.
pub(super) struct WaitCall {
    target: DefId,
    original: pc::FunctionReference,
    caller: Option<DefId>,
    locals: Vec<DefId>,
    slots: Vec<pc::FieldReference>,
    temp_prefix: String,
    temp_depth: usize,
    shared_receiver: bool,
    file: FileId,
    span: Span,
}

#[derive(Default)]
pub(super) struct Composition {
    calls: BTreeMap<usize, ComposedCall>,
    bodies: BTreeMap<DefId, Vec<pc::Operation>>,
}

struct ComposedCall {
    target: DefId,
    original: pc::FunctionReference,
    fields: Vec<pc::FieldReference>,
}

fn call_marker(action: &pc::Action) -> Option<usize> {
    let pc::Action::Call {
        function: pc::FunctionReference::Internal { id },
    } = action
    else {
        return None;
    };
    id.path().strip_prefix(CALL_MARKER)?.parse().ok()
}

fn lexical_locals(body: &BodyCx) -> Vec<DefId> {
    // An outer binding hidden by a branch local becomes visible again after
    // the branch joins. A declaration in the same scope permanently hides it.
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
    let borrowed_sources: Vec<_> = definitions
        .iter()
        .filter_map(|local| body.borrowed_slices.get(local).map(|view| view.source))
        .collect();
    definitions.extend(borrowed_sources);
    definitions.extend(body.initializing_locals.iter().copied());
    definitions.into_iter().collect()
}

fn children(action: &mut pc::Action) -> impl Iterator<Item = &mut Vec<pc::Operation>> {
    let (first, second) = match action {
        pc::Action::Branch {
            then, otherwise, ..
        } => (Some(then), Some(otherwise)),
        pc::Action::While { body, .. }
        | pc::Action::Players { body }
        | pc::Action::Entities { body, .. }
        | pc::Action::WithEntity { body, .. }
        | pc::Action::InDimension { body, .. }
        | pc::Action::Suspend { body, .. } => (Some(body), None),
        _ => (None, None),
    };
    first.into_iter().chain(second)
}

fn restore_calls(operations: &mut [pc::Operation], calls: &[WaitCall], composed: &Composition) {
    for operation in operations {
        if let Some(site) = call_marker(&operation.action)
            && !composed.calls.contains_key(&site)
        {
            operation.action = pc::Action::Call {
                function: calls[site].original.clone(),
            };
        }
        for body in children(&mut operation.action) {
            restore_calls(body, calls, composed);
        }
    }
}

pub(super) struct WaitSite {
    setup: Vec<pc::Operation>,
    ticks: pc::ScoreOperand,
    fields: Vec<pc::FieldReference>,
    group: Option<String>,
    server: bool,
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
                server: self.server,
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
        if body.owner.is_none() || body.wait_block_depth != body.block_ends.len() {
            self.error("wait", file, span, "use `wait` directly in a function body or an if/else branch, match arm, or local loop");
            return None;
        }
        let bound = self.bind_args(body, builtin, args, span)?;
        let server = self.wait_is_server(file, bound.get("server").copied())?;
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
        self.capture_wait(
            body,
            WaitSite {
                setup,
                ticks,
                fields: Vec::new(),
                group,
                server,
                span,
            },
        )
    }

    fn capture_wait(&mut self, body: &mut BodyCx, mut wait: WaitSite) -> Option<WaitSite> {
        if !wait.server {
            body.note_executor(
                wait.span,
                "retains the executing entity across a wait".into(),
            );
        }
        wait.fields = body.wait_slots.clone();
        for def in lexical_locals(body) {
            let root = self.local_ref(def);
            let ty = self.local_ty(def);
            wait.fields.extend(self.retained_slots(&root, ty));
        }
        if wait.fields.len() > 64 {
            self.error(
                "wait",
                body.file,
                wait.span,
                "a wait can retain at most 64 local fields, including fields of records",
            );
            return None;
        }
        Some(wait)
    }

    pub(super) fn breakpoint_wait(
        &mut self,
        body: &mut BodyCx,
        server: bool,
        group: String,
        span: Span,
    ) -> Vec<pc::Operation> {
        self.capture_wait(
            body,
            WaitSite {
                setup: Vec::new(),
                ticks: pc::ScoreOperand::Value(1),
                fields: Vec::new(),
                group: Some(group),
                server,
                span,
            },
        )
        .map_or_else(Vec::new, |wait| wait.suspend(body.file, Vec::new()))
    }

    pub(super) fn lower_breakpoint(
        &mut self,
        body: &mut BodyCx,
        args: &BTreeMap<&'static str, &Arg>,
        span: Span,
        ops: &mut Vec<pc::Operation>,
    ) {
        let file = body.file;
        if body.owner.is_none() || body.wait_block_depth != body.block_ends.len() {
            self.error(
                "breakpoint",
                file,
                span,
                "use a breakpoint at a statement where the invocation can wait",
            );
            return;
        }
        let Some((label, _, _)) = self.arg_string(file, args["label"], "a breakpoint label") else {
            return;
        };
        if !(1..=16).contains(&label.len())
            || !label.as_bytes()[0].is_ascii_lowercase()
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || matches!(
                label.as_str(),
                "presence" | "suppressed" | "numeric_scratch"
            )
        {
            self.error(
                "breakpoint",
                file,
                span,
                "use a 1–16 character lowercase breakpoint label starting with a letter",
            );
            return;
        }
        let Some(server) = self.wait_is_server(file, args.get("server").copied()) else {
            return;
        };
        if let Some(&(first_file, first_span)) = self.breakpoints.get(&label)
            && (first_file, first_span) != (file, span)
        {
            self.diagnostics.push(
                Diagnostic::error(
                    "breakpoint",
                    file,
                    span,
                    format!("duplicate breakpoint label `{label}`"),
                )
                .with_note(first_file, first_span, "first declaration"),
            );
            return;
        }
        self.breakpoints.insert(label.clone(), (file, span));
        let score = pc::FieldReference {
            state: format!(
                "{}:sand.breakpoints",
                self.pack.as_ref().expect("pack checked").namespace
            )
            .parse()
            .expect("valid breakpoint State identity"),
            field: label.clone(),
        };
        let block = Block {
            stmts: Vec::new(),
            span,
        };
        self.lower_repeat(
            body,
            super::statements::Repeat::Breakpoint {
                score,
                server,
                group: format!("breakpoint_{label}"),
            },
            &block,
            span,
            ops,
        );
    }

    pub(super) fn wait_is_server(&mut self, file: FileId, arg: Option<&Arg>) -> Option<bool> {
        match arg {
            Some(arg) => self.arg_bool(file, arg),
            None => Some(false),
        }
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn mark_wait_call(
        &mut self,
        body: &BodyCx,
        target: DefId,
        temp_depth: usize,
        shared_receiver: bool,
        span: Span,
        original: pc::FunctionReference,
    ) -> pc::FunctionReference {
        let site = self.wait_calls.len();
        self.wait_calls.push(WaitCall {
            target,
            original,
            caller: body.owner,
            locals: lexical_locals(body),
            slots: body.wait_slots.clone(),
            temp_prefix: format!("{}_t", body.slot),
            temp_depth,
            shared_receiver,
            file: body.file,
            span,
        });
        pc::FunctionReference::Internal {
            id: sand::ResourceLocation::new(self.namespace(), format!("{CALL_MARKER}{site}"))
                .expect("valid call marker"),
        }
    }

    pub(super) fn prepare_wait_calls(&mut self, lowerer: &mut Lowering) -> Composition {
        let calls = std::mem::take(&mut self.wait_calls);
        let mut suspending = BTreeSet::new();
        loop {
            let previous_size = suspending.len();
            for function in &lowerer.functions {
                if let Some(def) = function.def
                    && !suspending.contains(&def)
                    && frame_suspends(&function.body, &|site| {
                        suspending.contains(&calls[site].target)
                    })
                {
                    suspending.insert(def);
                }
            }
            if suspending.len() == previous_size {
                break;
            }
        }
        let mut composed = Composition::default();
        let state: sand::ResourceLocation =
            self.locals_state_id().parse().expect("namespace validated");
        for (site, call) in calls.iter().enumerate() {
            if !suspending.contains(&call.target) {
                continue;
            }
            if let Some(caller) = call.caller
                && lowerer.reenters(call.target, caller)
            {
                self.error(
                    "wait",
                    call.file,
                    call.span,
                    "a call that suspends its caller cannot be recursive",
                );
                continue;
            }
            if call.shared_receiver {
                self.error("wait", call.file, call.span, "a mutable receiver held across a wait must be local; copy the State value into a local and assign it back explicitly");
                continue;
            }
            let mut fields = call.slots.clone();
            for local in &call.locals {
                let root = self.local_ref(*local);
                fields.extend(self.retained_slots(&root, self.local_ty(*local)));
            }
            for name in self
                .local_fields
                .iter()
                .chain(self.local_data_fields.keys())
            {
                let base = name.strip_prefix("__sand_data_").unwrap_or(name);
                if base
                    .strip_prefix(&call.temp_prefix)
                    .and_then(|suffix| suffix.split('_').next()?.parse::<usize>().ok())
                    .is_some_and(|index| index < call.temp_depth)
                {
                    fields.push(pc::FieldReference {
                        state: state.clone(),
                        field: name.clone(),
                    });
                }
            }
            composed.calls.insert(
                site,
                ComposedCall {
                    target: call.target,
                    original: call.original.clone(),
                    fields,
                },
            );
        }
        for function in &mut lowerer.functions {
            restore_calls(&mut function.body, &calls, &composed);
            if let Some(def) = function.def
                && suspending.contains(&def)
            {
                composed.bodies.insert(def, function.body.clone());
            }
        }
        composed
    }
}

/// Whether copying these operations could duplicate a wait site.
/// Executor-changing bodies have independent continuations, but their
/// waits still count toward the same pack limit.
pub(super) fn contains_wait_site(operations: &[pc::Operation]) -> bool {
    operations.iter().any(|operation| match &operation.action {
        pc::Action::Suspend { .. } => true,
        pc::Action::Branch {
            then, otherwise, ..
        } => contains_wait_site(then) || contains_wait_site(otherwise),
        pc::Action::While { body, .. }
        | pc::Action::Players { body }
        | pc::Action::Entities { body, .. }
        | pc::Action::WithEntity { body, .. }
        | pc::Action::InDimension { body, .. } => contains_wait_site(body),
        _ => call_marker(&operation.action).is_some(),
    })
}

fn frame_suspends(operations: &[pc::Operation], composed: &impl Fn(usize) -> bool) -> bool {
    operations.iter().any(|operation| match &operation.action {
        pc::Action::Suspend { .. } => true,
        pc::Action::While { body, .. } => frame_suspends(body, composed),
        pc::Action::Branch {
            then, otherwise, ..
        } => frame_suspends(then, composed) || frame_suspends(otherwise, composed),
        _ => call_marker(&operation.action).is_some_and(composed),
    })
}

/// Outline shared suffixes once. Reaching the end of either branch calls its
/// suffix, and suspending retains the suffix call for the resumed path.
pub(super) fn join_wait_paths(
    function: &mut pc::Function,
    composition: &Composition,
    remaining: &mut usize,
) -> Option<Vec<pc::Function>> {
    let mut joins = WaitJoins {
        id: &function.id,
        context: function.context,
        functions: Vec::new(),
        composition,
        retained: Vec::new(),
        remaining,
        exhausted: false,
    };
    joins.rewrite_nested_frames(&mut function.body);
    joins.rewrite(&mut function.body, None);
    (!joins.exhausted).then_some(joins.functions)
}

struct WaitJoins<'a> {
    id: &'a sand::ResourceLocation,
    context: pc::ExecutionContext,
    functions: Vec<pc::Function>,
    composition: &'a Composition,
    retained: Vec<pc::FieldReference>,
    remaining: &'a mut usize,
    exhausted: bool,
}

impl WaitJoins<'_> {
    fn suspends(&self, operations: &[pc::Operation]) -> bool {
        frame_suspends(operations, &|site| {
            self.composition.calls.contains_key(&site)
        })
    }

    fn rewrite_nested_frames(&mut self, operations: &mut [pc::Operation]) {
        for operation in operations {
            let (body, context) = match &mut operation.action {
                pc::Action::Players { body } => (body, pc::ExecutionContext::Player),
                pc::Action::Entities { body, .. } => (body, pc::ExecutionContext::Entity),
                pc::Action::WithEntity { body, players, .. } => {
                    let context = if *players {
                        pc::ExecutionContext::Player
                    } else {
                        pc::ExecutionContext::Entity
                    };
                    (body, context)
                }
                pc::Action::InDimension { body, .. } => (body, self.context),
                pc::Action::Suspend { body, .. } | pc::Action::While { body, .. } => {
                    self.rewrite_nested_frames(body);
                    continue;
                }
                pc::Action::Branch {
                    then, otherwise, ..
                } => {
                    self.rewrite_nested_frames(then);
                    self.rewrite_nested_frames(otherwise);
                    continue;
                }
                _ => continue,
            };
            let outer = std::mem::replace(&mut self.context, context);
            let retained = std::mem::take(&mut self.retained);
            self.rewrite_nested_frames(body);
            self.rewrite(body, None);
            self.retained = retained;
            self.context = outer;
        }
    }

    fn rewrite(&mut self, operations: &mut Vec<pc::Operation>, fallthrough: Option<pc::Operation>) {
        if !self.suspends(operations) {
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
                pc::Action::Suspend { body, fields, .. } => {
                    for field in &self.retained {
                        if !fields.contains(field) {
                            fields.push(field.clone());
                        }
                    }
                    let fallthrough = self.take_suffix(&mut suffix);
                    self.rewrite(body, fallthrough);
                }
                pc::Action::While { condition, body } if self.suspends(body) => {
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
                } if self.suspends(then) || self.suspends(otherwise) => {
                    let fallthrough = self.take_suffix(&mut suffix);
                    self.rewrite(then, fallthrough.clone());
                    self.rewrite(otherwise, fallthrough);
                }
                action if let Some(site) = call_marker(action) => {
                    let fallthrough = self.take_suffix(&mut suffix);
                    operation = self.compose_call(site, operation.origin, fallthrough);
                }
                _ => {}
            }
            suffix.push(operation);
        }
        suffix.reverse();
        *operations = suffix;
    }

    fn compose_call(
        &mut self,
        site: usize,
        origin: Option<serde_json::Value>,
        fallthrough: Option<pc::Operation>,
    ) -> pc::Operation {
        let call = &self.composition.calls[&site];
        if fallthrough.is_some() && *self.remaining == 0 {
            self.exhausted = true;
        }
        let Some(completion) = fallthrough.filter(|_| !self.exhausted) else {
            return pc::Operation {
                action: pc::Action::Call {
                    function: call.original.clone(),
                },
                origin,
            };
        };
        *self.remaining -= 1;
        let mut body = self.composition.bodies[&call.target].clone();
        Self::complete_returns(&mut body, &completion);
        let retained = self.retained.len();
        self.retained.extend_from_slice(&call.fields);
        self.rewrite_nested_frames(&mut body);
        self.rewrite(&mut body, Some(completion));
        self.retained.truncate(retained);
        let entry = self.outline(body);
        self.call(entry, origin)
    }

    /// A return finishes only the callee. Run its caller's suffix and end
    /// the callee branch even if the suffix itself suspends.
    fn complete_returns(operations: &mut Vec<pc::Operation>, completion: &pc::Operation) {
        for index in (0..operations.len()).rev() {
            match &mut operations[index].action {
                pc::Action::Return => operations.insert(index, completion.clone()),
                pc::Action::Branch {
                    then, otherwise, ..
                } => {
                    Self::complete_returns(then, completion);
                    Self::complete_returns(otherwise, completion);
                }
                pc::Action::While { body, .. } | pc::Action::Suspend { body, .. } => {
                    Self::complete_returns(body, completion);
                }
                _ => {}
            }
        }
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
