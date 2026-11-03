//! Owned timelines compiled through the canonical indexed job runtime.
use super::{Job, Lifetime, TaskContext, identity, job, lifetime::Callback, runtime};
use crate::{
    IntoCommands,
    entity::{EntityKind, PlayerKind},
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack},
};
use sand_commands::{CommandError, CommandResult};
use sand_components::effect::Ticks;
use std::sync::Arc;

/// A finite, one-shot timeline owned by a Lifetime.
///
/// Actions before the first wait run on the first loaded tick, not during start.
/// `wait(n)` advances the authored timeline by n loaded ticks. Adjacent actions
/// run in insertion order on the same tick. A trailing wait delays completion.
/// Cancellation, expiry and owner loss discard remaining actions and completion.
/// Use ordinary conditions inside actions for branches and State for progress.
/// This is not a parallel task graph or a wall-clock scheduler.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence", module = "sand::systems",
summary = "A finite owned sequence of actions and loaded-tick waits.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
 example = "use sand::systems::Sequence;")]
pub struct Sequence<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    elapsed: u64,
    actions: Vec<(u64, Callback<K>)>,
    complete: Option<Callback<K>>,
    gates: Vec<Gate<K>>,
}
#[derive(Clone)]
struct Gate<K: EntityKind> {
    at: u64,
    timeout: Ticks,
    condition: Arc<dyn Fn(TaskContext<K>) -> crate::condition::Condition>,
}
impl<K: EntityKind> Sequence<K> {
    /// Create an empty timeline. Add at least one action before export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::new", module = "sand::systems",
summary = "Create an owned timeline.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
params(lifetime = "Ownership, capacity, callbacks and hard deadline."), returns = "The configured sequence.", example = "use sand::systems::{Sequence, Lifetime}; fn demo(lifetime: Lifetime) { let _ = Sequence::new(lifetime); }")]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            elapsed: 0,
            actions: Vec::new(),
            complete: None,
            gates: Vec::new(),
        }
    }
    /// Append an action at the current timeline position. Up to 256 actions are
    /// accepted. Each callback runs in its own function, so an early return does
    /// not skip later actions; TaskContext::stop explicitly cancels the sequence.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::then", module = "sand::systems",
summary = "Append an action at the current timeline position.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
params(body = "Authors an action in instance context; use TaskContext to reach the owner."), returns = "The configured sequence.", example = "use sand::systems::Sequence; fn demo(sequence: Sequence) { let _ = sequence.then(|ctx| ctx.stop()); }")]
    pub fn then<R: IntoCommands>(mut self, body: impl Fn(TaskContext<K>) -> R + 'static) -> Self {
        self.actions
            .push((self.elapsed, Arc::new(move |ctx| body(ctx).into_commands())));
        self
    }
    /// Advance the timeline. Zero is allowed and preserves insertion order.
    /// The total duration must be at most i32::MAX - 1 ticks. Allow at least
    /// total waits + 1 loaded ticks in the lifetime for completion to occur.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::wait", module = "sand::systems",
summary = "Advance the timeline by loaded ticks.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
params(duration = "Delay before subsequently appended actions or completion."), returns = "The configured sequence.", example = "use sand::systems::Sequence; use sand::prelude::Ticks; fn demo(sequence: Sequence) { let _ = sequence.wait(Ticks::new(20)); }")]
    pub fn wait(mut self, duration: Ticks) -> Self {
        self.elapsed = self.elapsed.saturating_add(u64::from(duration.get()));
        self
    }
    /// Wait for a runtime condition, aborting the sequence if it remains false
    /// for `timeout` loaded ticks. Starts checking on the tick after preceding
    /// actions, even when the condition is already true. Later waits are relative
    /// to release. Timeout skips completion and runs normal lifetime cleanup.
    /// The lifetime's own deadline continues counting while blocked.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::wait_until", module = "sand::systems",
summary = "Pause a timeline until a condition holds, with a bounded timeout.", context = "Checks begin on the next loaded tick. Timeout cancels remaining actions and completion; Lifetime cleanup always runs. The hard lifetime deadline continues while waiting.",
minecraft = "Gates the existing Job cursor and keeps a private per-instance wait counter. The predicate executes once per loaded tick while blocked.",
use_when = ["Waiting for a phase condition or player readiness"], avoid_when = ["Unbounded or wall-clock waits are required"],
params(condition = "Authors a condition in instance context; owner_matches can query the owner.", timeout = "Positive maximum number of failed loaded-tick checks."), returns = "The timeline with a bounded condition wait.",
example = "use sand::systems::Sequence; use sand::prelude::{Ticks, Condition}; fn demo(sequence: Sequence) { let _ = sequence.wait_until(|_| Condition::all([]), Ticks::new(40)); }")]
    pub fn wait_until(
        mut self,
        condition: impl Fn(TaskContext<K>) -> crate::condition::Condition + 'static,
        timeout: Ticks,
    ) -> Self {
        self.elapsed = self.elapsed.saturating_add(1);
        self.gates.push(Gate {
            at: self.elapsed,
            timeout,
            condition: Arc::new(condition),
        });
        self
    }
    /// Run once at the end, before lifetime cleanup. Cancellation never invokes it.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::on_complete", module = "sand::systems",
summary = "Run once after the final action and trailing wait.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
params(body = "Completion action in instance context."), returns = "The configured sequence.", example = "use sand::systems::Sequence; fn demo(sequence: Sequence) { let _ = sequence.on_complete(|ctx| ctx.stop()); }")]
    pub fn on_complete<R: IntoCommands>(
        mut self,
        body: impl Fn(TaskContext<K>) -> R + 'static,
    ) -> Self {
        self.complete = Some(Arc::new(move |ctx| body(ctx).into_commands()));
        self
    }
    /// Admit an independent sequence for the current owner using Lifetime capacity.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::start", module = "sand::systems",
summary = "Start an independent sequence for the current owner.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
 returns = "Retained start actions.", example = "use sand::systems::Sequence; fn demo(sequence: Sequence) { let _ = sequence.start(); }")]
    pub fn start(&self) -> Actions {
        self.lifetime.start()
    }
    /// Cancel every instance of this definition owned by the current executor.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Sequence::cancel", module = "sand::systems",
summary = "Cancel this owner’s instances of the sequence.", context = "Register the definition with datapack_component. Uses the existing Lifetime and Job runtime; gameplay state remains ordinary State.",
minecraft = "Runs on loaded ticks with independent ownership, cancellation and cleanup. Callback continuations resolve the live instance after dimension changes.",
use_when = ["Composing timed encounter actions"], avoid_when = ["Work must run in unloaded chunks"],
 returns = "Retained cancellation actions.", example = "use sand::systems::Sequence; fn demo(sequence: Sequence) { let _ = sequence.cancel(); }")]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
    fn validate(&self) -> CommandResult<()> {
        if self.actions.is_empty() || self.actions.len() > 256 {
            return Err(CommandError::new(
                "sequence",
                "actions",
                "requires 1..=256 actions",
            ));
        }
        if self.gates.len() > 256
            || self
                .gates
                .iter()
                .any(|g| g.timeout.get() == 0 || g.timeout.get() > i32::MAX as u32)
        {
            return Err(CommandError::new(
                "sequence",
                "wait_until",
                "at most 256 gates with timeout in 1..=2147483647 are supported",
            ));
        }
        if self.elapsed >= i32::MAX as u64 {
            return Err(CommandError::new(
                "sequence",
                "duration",
                "total waits must be at most 2147483646 ticks",
            ));
        }
        Ok(())
    }
    fn tick(&self) -> CommandResult<Actions> {
        self.validate()?;
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let cursor = job::cursor(&id);
        Ok(self
            .actions
            .iter()
            .enumerate()
            .map(|(i, (at, _))| {
                format!(
                    "execute if score @s {cursor} matches {at} run function {} with entity @s data",
                    identity::resource(&id, &format!("sequence_dispatch_{i}"))
                )
            })
            .collect::<Vec<_>>()
            .into_commands())
    }
}
fn gate_counter(id: &sand_components::ResourceLocation) -> sand_commands::ObjectiveName {
    sand_commands::ObjectiveName::logical(format!("sand sequence {id} wait"))
}
impl<K: EntityKind> Sequence<K> {
    fn ready(&self) -> CommandResult<Actions> {
        self.validate()?;
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let cursor = job::cursor(&id);
        let mut lines = self
            .gates
            .iter()
            .enumerate()
            .map(|(i, g)| {
                format!(
                    "execute if score @s {cursor} matches {} run return run function {}",
                    g.at,
                    identity::resource(&id, &format!("sequence_gate_{i}"))
                )
            })
            .collect::<Vec<_>>();
        lines.push("return 1".into());
        Ok(lines.into_commands())
    }
    fn gate(&self, index: usize) -> CommandResult<Actions> {
        self.validate()?;
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let counter = gate_counter(&id);
        crate::function::request_numeric_objectives([counter.clone()])?;
        let gate = &self.gates[index];
        let mut actions = (gate.condition)(runtime::context(&self.lifetime)).guard_actions(
            false,
            &[],
            Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                prefix: "sand/sequence/ready".into(),
                body: vec![
                    format!("scoreboard players set @s {counter} 0"),
                    "return 1".into(),
                ]
                .into_commands(),
            }))
            .into_commands(),
        );
        actions.extend([vec![
            format!("scoreboard players add @s {counter} 1"),
            format!(
                "execute if score @s {counter} matches {}.. run function {}",
                gate.timeout.get(),
                identity::resource(&id, "stop")
            ),
            "return 0".into(),
        ]
        .into_commands()]);
        Ok(actions)
    }
}
impl<K: EntityKind> IntoDatapack for Sequence<K> {
    fn into_datapack(self) -> DatapackRegistration {
        let scope = self.lifetime.id.clone();
        let mut job = Job::new(
            self.lifetime.clone(),
            self.elapsed.min(i32::MAX as u64 - 1) as u32 + 1,
            move |_, _| identity::call(&scope, "sequence_tick"),
        );
        if !self.gates.is_empty() {
            let id = self.lifetime.id.clone();
            job.ready = Some(Arc::new(move |_| {
                Cmd::ReturnRun(Box::new(Cmd::Function(
                    identity::resource(&id, "sequence_ready").to_string(),
                )))
                .into_commands()
            }));
            let old_stop = job.lifetime.stop.clone();
            let scope = self.lifetime.id.clone();
            job.lifetime.stop = Some(Arc::new(move |ctx| {
                let mut result = old_stop
                    .as_ref()
                    .map(|f| runtime::isolated(f(ctx)))
                    .unwrap_or_default();
                result.extend([identity::call(&scope, "sequence_gate_cleanup")]);
                result
            }));
        }
        if let Some(complete) = self.complete.clone() {
            job = job.on_complete(move |ctx| complete(ctx));
        }
        let definition = self.clone();
        let mut registration = job.into_datapack().component(runtime::Function::new(
            &self.lifetime.id,
            "sequence_tick",
            move || definition.tick(),
        ));
        if !self.gates.is_empty() {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                "sequence_ready",
                move || definition.ready(),
            ));
            let scope = self.lifetime.id.clone();
            registration =
                registration.component(runtime::Function::new(
                    &self.lifetime.id,
                    "sequence_gate_cleanup",
                    move || {
                        let id = crate::function::resolve_export_location(&scope)?;
                        Ok(format!("scoreboard players reset @s {}", gate_counter(&id))
                            .into_commands())
                    },
                ));
            for i in 0..self.gates.len() {
                let definition = self.clone();
                registration = registration.component(runtime::Function::new(
                    &self.lifetime.id,
                    &format!("sequence_gate_{i}"),
                    move || definition.gate(i),
                ));
            }
        }
        for (i, (_, callback)) in self.actions.iter().enumerate() {
            let id = self.lifetime.id.clone();
            let destination = format!("sequence_action_{i}");
            let callback = callback.clone();
            let lifetime = self.lifetime.clone();
            registration =
                registration.component(runtime::Function::new(&id, &destination, move || {
                    Ok(callback(runtime::context(&lifetime)))
                }));
            let scope = id.clone();
            registration = registration.component(runtime::Function::new(
                &id,
                &format!("sequence_dispatch_{i}"),
                move || {
                    runtime::resolve(
                        &crate::function::resolve_export_location(&scope)?,
                        &destination,
                    )
                },
            ));
        }
        registration
    }
}
impl<K: EntityKind> AsRef<Lifetime<K>> for Sequence<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sequence() -> Sequence {
        Sequence::new(Lifetime::new(
            "test:sequence".parse().unwrap(),
            Ticks::new(20),
        ))
    }
    #[test]
    fn rejects_empty_excessive_and_overflowed_timelines() {
        assert_eq!(sequence().validate().unwrap_err().field, "actions");
        let mut many = sequence();
        for _ in 0..257 {
            many = many.then(|_| Actions::default());
        }
        assert_eq!(many.validate().unwrap_err().field, "actions");
        assert_eq!(
            sequence()
                .then(|_| Actions::default())
                .wait(Ticks::new(u32::MAX))
                .validate()
                .unwrap_err()
                .field,
            "duration"
        );
    }
    #[test]
    fn condition_waits_validate_timeout_and_advance_to_a_new_tick() {
        let invalid = sequence()
            .then(|_| Actions::default())
            .wait_until(|_| crate::condition::Condition::all([]), Ticks::new(0));
        assert_eq!(invalid.validate().unwrap_err().field, "wait_until");
        let valid = sequence()
            .then(|_| Actions::default())
            .wait_until(|_| crate::condition::Condition::all([]), Ticks::new(2))
            .then(|_| Actions::default());
        assert_eq!(valid.gates[0].at, 1);
        assert_eq!(valid.actions[1].0, 1);
        assert!(valid.validate().is_ok());
    }
    #[test]
    fn preserves_insertion_order_and_trailing_waits() {
        let value = sequence()
            .then(|_| Actions::default())
            .wait(Ticks::new(2))
            .then(|_| Actions::default())
            .then(|_| Actions::default())
            .wait(Ticks::new(3));
        assert_eq!(
            value.actions.iter().map(|(at, _)| *at).collect::<Vec<_>>(),
            [0, 2, 2]
        );
        assert_eq!(value.elapsed, 5);
        assert!(value.validate().is_ok());
    }
}
