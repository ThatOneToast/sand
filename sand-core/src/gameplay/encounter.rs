//! Priority encounter behavior over canonical owned child tasks.
use super::{Lifetime, TaskContext, runtime};
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{EntityContext, EntityKind, PlayerKind},
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack},
};
use std::sync::Arc;

/// A priority controller that runs one registered child attack at a time.
///
/// Register child definitions separately. Each loaded tick, the controller first
/// checks its active condition, then waits for all direct children to settle.
/// It starts the first eligible attack in insertion order. Include recovery in
/// the child Sequence; no eligible attack leaves the encounter idle. A rejected
/// admission retries the selected attack on the next tick, without falling
/// through to a lower-priority attack. Conditions can read canonical State for
/// phase thresholds, target availability and alternating attack selection.
///
/// Starting restarts this owner's encounter and cancels its descendants. Other
/// owners remain independent. Cancellation, expiry, owner loss and an inactive
/// condition run the Lifetime stop callback exactly once. Reset gameplay State
/// there. Unloaded children remain outstanding until their normal cleanup.
/// This controller does not provide navigation, a threat table, or native AI.
#[derive(Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter", module = "sand::systems",
 summary = "Priority encounter behavior built from owned child tasks.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
   example = "use sand::systems::Encounter;")]
pub struct Encounter<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    active: Option<OwnerCondition<K>>,
    attacks: Vec<(OwnerCondition<K>, Lifetime<K>)>,
}
type OwnerCondition<K> = Arc<dyn Fn(EntityContext<K>) -> Condition>;

impl<K: EntityKind> Encounter<K> {
    /// Create a controller with a bounded ownership lifetime.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter::new", module = "sand::systems",
 summary = "Create a controller with a bounded ownership lifetime.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
 params(lifetime = "Capacity, hard duration, required State, and start/stop callbacks."), returns = "The configured encounter.", example = "use sand::systems::Encounter;")]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            active: None,
            attacks: vec![],
        }
    }
    /// Stop and clean up when this owner-context condition becomes false.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter::while_active", module = "sand::systems",
 summary = "Stop and clean up when this owner-context condition becomes false.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
 params(condition = "Owner condition checked every loaded tick, including during attacks."), returns = "The configured encounter.", example = "use sand::systems::Encounter;")]
    pub fn while_active(
        mut self,
        condition: impl Fn(EntityContext<K>) -> Condition + 'static,
    ) -> Self {
        self.active = Some(Arc::new(condition));
        self
    }
    /// Append a registered child attack in priority order.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter::attack", module = "sand::systems",
 summary = "Append a registered child attack in priority order.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
 params(condition = "Owner-context eligibility condition.", task = "Registered child definition; includes its recovery and cleanup."), returns = "The configured encounter.", example = "use sand::systems::Encounter;")]
    pub fn attack(
        mut self,
        condition: impl Fn(EntityContext<K>) -> Condition + 'static,
        task: impl AsRef<Lifetime<K>>,
    ) -> Self {
        self.attacks
            .push((Arc::new(condition), task.as_ref().clone()));
        self
    }
    /// Cancel this owner’s previous encounter and start a fresh one.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter::start", module = "sand::systems",
 summary = "Cancel this owner’s previous encounter and start a fresh one.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
  returns = "Retained owner-context actions.", example = "use sand::systems::Encounter;")]
    pub fn start(&self) -> Actions {
        let mut actions = self.lifetime.cancel();
        actions.extend([self.lifetime.start()]);
        actions
    }
    /// Cancel this owner’s encounter and all its descendants.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::systems::Encounter::cancel", module = "sand::systems",
 summary = "Cancel this owner’s encounter and all its descendants.", context = "An encounter selects the first eligible registered child attack only when its previous children have settled. Lifetime owns expiry, owner loss, reset and descendant cleanup. Attack conditions execute in owner context.",
 minecraft = "Uses the canonical Lifetime child ledger and isolated first-match selection; selection runs on loaded ticks.",
 use_when = ["Composing reusable boss or wave controllers"], avoid_when = ["Pathfinding or automatic interception of native combat"],
  returns = "Retained owner-context actions.", example = "use sand::systems::Encounter;")]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> AsRef<Lifetime<K>> for Encounter<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}
impl<K: EntityKind> IntoDatapack for Encounter<K> {
    fn into_datapack(mut self) -> DatapackRegistration {
        if self.attacks.is_empty() || self.attacks.len() > 256 {
            self.lifetime.invalid = Some(("attacks", "encounter requires 1..=256 attacks"));
        }
        self.lifetime.step = Some("encounter_select");
        let definition = self.clone();
        let scope = self.lifetime.id.clone();
        self.lifetime
            .into_datapack()
            .component(runtime::Function::new(
                &scope,
                "encounter_select",
                move || {
                    let ctx = TaskContext::new(definition.lifetime.id.clone());
                    let mut actions = Actions::default();
                    if let Some(active) = &definition.active {
                        actions.extend([ctx.owner_matches(|owner| active(owner)).guard_actions(
                            true,
                            &[],
                            Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                                prefix: "sand/encounter/inactive".into(),
                                body: ctx.stop(),
                            }))
                            .into_commands(),
                        )]);
                    }
                    actions.extend([ctx.children_finished().guard_actions(
                        true,
                        &[],
                        Cmd::Return(0).into_commands(),
                    )]);
                    for (condition, task) in &definition.attacks {
                        actions.extend([ctx
                            .owner_matches(|owner| condition(owner))
                            .guard_actions(
                                false,
                                &[],
                                Cmd::ReturnRun(Box::new(Cmd::AnonymousFunction {
                                    prefix: "sand/encounter/attack".into(),
                                    body: ctx.start_child(task),
                                }))
                                .into_commands(),
                            )]);
                    }
                    actions.extend([Cmd::Return(0)]);
                    Ok(actions)
                },
            ))
    }
}
