//! Bounded session controllers composed from canonical Lifetime and Progression.
use super::{Lifetime, Progression, TaskContext, identity, runtime};
use crate::{
    IntoCommands,
    entity::{EntityKind, PlayerKind},
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack, LifecycleContribution},
};
use sand_commands::{CommandResult, ObjectiveName};
use std::sync::Arc;

/// One loaded-tick session controller per owner, with ordered objective phases.
///
/// A player can own a personal session; attach State to a dedicated entity host
/// to represent a shared match and use two hosts for two independent matches.
/// Participant/team/spectator assignment is authored separately in canonical
/// State. This type does not infer membership from distance or selectors.
///
/// Register only the Session: it includes its Progression and Lifetime.
/// Starting cancels this owner's previous controller and descendants, preserving
/// progress. Restart additionally resets progress before admission; reset cancels
/// without starting. A rejected admission after restart leaves progress at zero.
/// Rewards are committed once per explicit reset. Completion stops the Lifetime;
/// all stop reasons share its callback and owned-State/entity cleanup.
///
/// Existing Lifetime tick/start callbacks are preserved. Its stop callback runs
/// with session controls locked, so synchronous start/restart/reset/cancel calls
/// from that callback are no-ops. Controls are also rejected during rewards.
/// Use a subsequent event/tick for a new round. Loaded work and descendants follow
/// Lifetime's unload/owner-loss/reload rules; canonical State persists only as its
/// own schema specifies. Reload invalidates controllers and never restores tasks.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session", module = "sand::systems",
 summary = "Bounded owner-scoped session and round lifecycle.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],

 example = "use sand::systems::Session;")]
pub struct Session<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    progression: Progression<K>,
}
impl<K: EntityKind> Session<K> {
    /// Compose an ownership deadline/capacity/cleanup policy and ordered phases.
    /// The Lifetime owner must own the Progression's attached State. Distinct
    /// registered identities are supported; both definitions are included here.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session::new", module = "sand::systems",
 summary = "Compose session ownership and canonical phase progression.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],
 params(lifetime = "Bounded ownership, callbacks, required/owned State and cleanup.", progression = "Ordered owner-State objectives; included in this registration."),
 returns = "The session definition or retained owner-context lifecycle actions.", example = "use sand::systems::Session;")]
    pub fn new(lifetime: Lifetime<K>, progression: Progression<K>) -> Self {
        Self {
            lifetime,
            progression,
        }
    }
    /// Replace this owner's controller and descendants, retaining saved progress.
    /// Completed or missing State rejects admission. This does not reset rewards.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session::start", module = "sand::systems",
 summary = "Start or resume a session while preserving progress.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],

 returns = "The session definition or retained owner-context lifecycle actions.", example = "use sand::systems::Session;")]
    pub fn start(&self) -> Actions {
        identity::call(&self.lifetime.id, "session_start")
    }
    /// Cancel prior owned work, reset phase progress and attempt a fresh session.
    /// Authors reset additional gameplay State in the Lifetime stop/start callbacks.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session::restart", module = "sand::systems",
 summary = "Reset progress and start a fresh bounded session.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],

 returns = "The session definition or retained owner-context lifecycle actions.", example = "use sand::systems::Session;")]
    pub fn restart(&self) -> Actions {
        identity::call(&self.lifetime.id, "session_restart")
    }
    /// Cancel prior owned work and write progress zero without starting new work.
    /// Missing progress State still allows cancellation and skips the State write.
    /// Other author State follows the configured Lifetime cleanup policy.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session::reset", module = "sand::systems",
 summary = "Cancel and reset progress without starting new work.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],

 returns = "The session definition or retained owner-context lifecycle actions.", example = "use sand::systems::Session;")]
    pub fn reset(&self) -> Actions {
        identity::call(&self.lifetime.id, "session_reset")
    }
    /// Cancel this owner's controller and descendants while preserving progress.
    /// Cancellation remains available after progress State is detached.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Session::cancel", module = "sand::systems",
 summary = "Cancel owned session work while preserving progress.", context = "One bounded Lifetime controller per owner checks ordered canonical State objectives on loaded ticks. Start preserves progress; restart and reset explicitly enable replay. Cleanup cancels owned descendants.",
 minecraft = "Composes canonical Lifetime identities, descendant cleanup, State guards and Progression rewards; no independent gameplay state or task restoration.",
 use_when = ["Composing reusable owner-scoped match or round phases"], avoid_when = ["Automatic persistent sessions or unbounded shared matchmaking"],

 returns = "The session definition or retained owner-context lifecycle actions.", example = "use sand::systems::Session;")]
    pub fn cancel(&self) -> Actions {
        identity::call(&self.lifetime.id, "session_cancel")
    }
    fn lock(&self) -> CommandResult<ObjectiveName> {
        let id = crate::function::resolve_export_location(&self.lifetime.id)?;
        let lock = ObjectiveName::logical(format!("sand session {id} controls lock"));
        crate::function::request_numeric_objectives([lock.clone()])?;
        Ok(lock)
    }
    fn build(&self, part: &str) -> CommandResult<Actions> {
        let lock = self.lock()?;
        let progress_lock = self.progression.lock()?;
        let ctx = TaskContext::new(self.lifetime.id.clone());
        if part == "load" {
            return Ok(format!("scoreboard players set #busy {lock} 0").into_commands());
        }
        if part == "step" {
            return Ok(format!(
                "function {} with entity @s data",
                identity::resource(&self.lifetime.id, "session_advance")
            )
            .into_commands());
        }
        if part == "advance" {
            let mut actions = ctx.owner(|_| self.progression.advance());
            actions.extend([runtime::resolve(&self.lifetime.id, "session_after")?]);
            return Ok(actions);
        }
        if part == "after" {
            let mut actions = ctx
                .owner_matches(|_| self.progression.attached())
                .guard_actions(
                    true,
                    &[],
                    Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                        prefix: "sand/session/missing_state".into(),
                        body: ctx.stop(),
                    }))
                    .into_commands(),
                );
            actions.extend([ctx
                .owner_matches(|_| self.progression.completed())
                .guard_actions(false, &[], ctx.stop())]);
            return Ok(actions);
        }
        if part == "stopped_body" {
            return Ok(self
                .lifetime
                .stop
                .as_ref()
                .map_or_else(Actions::default, |callback| {
                    runtime::isolated(callback(ctx))
                }));
        }
        if part == "stopped" {
            let call = identity::call(&self.lifetime.id, "session_stopped_body");
            let mut actions = format!(
                "execute if score #busy {lock} matches 1 run return run function {}",
                identity::resource(&self.lifetime.id, "session_stopped_body")
            )
            .into_commands();
            actions.extend([
                format!("scoreboard players set #busy {lock} 1").into_commands(),
                call,
                format!("scoreboard players set #busy {lock} 0").into_commands(),
            ]);
            return Ok(actions);
        }
        let mut actions = Actions::default();
        actions.extend([
            format!("execute if score #busy {lock} matches 1 run return 0"),
            format!("execute if score #busy {progress_lock} matches 1 run return 0"),
        ]);
        if matches!(part, "start" | "restart") {
            actions.extend([self.progression.ready()?]);
        }
        if part == "start" {
            actions.extend([self.progression.completed().guard_actions(
                false,
                &[],
                Cmd::Return(0).into_commands(),
            )]);
        }
        actions.extend([
            format!("scoreboard players set #busy {lock} 1").into_commands(),
            format!("scoreboard players set #busy {progress_lock} 1").into_commands(),
            runtime::isolated(self.lifetime.cancel()),
        ]);
        if matches!(part, "reset" | "restart") {
            let mut reset = self.progression.ready()?;
            reset.extend([self.progression.reset_write()?]);
            actions.extend([runtime::isolated(reset)]);
        }
        // Stop callbacks can detach or modify State; recheck readiness before admission.
        if matches!(part, "start" | "restart") {
            let mut admission = self.progression.ready()?;
            admission.extend([self.lifetime.start()]);
            actions.extend([runtime::isolated(admission)]);
        }
        actions.extend([
            format!("scoreboard players set #busy {progress_lock} 0"),
            format!("scoreboard players set #busy {lock} 0"),
        ]);
        Ok(actions)
    }
}
impl<K: EntityKind> IntoDatapack for Session<K> {
    fn into_datapack(self) -> DatapackRegistration {
        let mut lifetime = self.lifetime.clone();
        lifetime.step = Some("session_step");
        lifetime.stop = Some(Arc::new(|ctx| {
            identity::call(&ctx.scope, "session_stopped")
        }));
        let mut registration = lifetime
            .into_datapack()
            .merge(self.progression.clone())
            .lifecycle(LifecycleContribution::load(format!(
                "function {}",
                identity::resource(&self.lifetime.id, "session_load")
            )));
        for part in [
            "load",
            "start",
            "restart",
            "reset",
            "cancel",
            "step",
            "advance",
            "after",
            "stopped",
            "stopped_body",
        ] {
            let definition = self.clone();
            registration = registration.component(runtime::Function::new(
                &self.lifetime.id,
                &format!("session_{part}"),
                move || definition.build(part),
            ));
        }
        registration
    }
}
