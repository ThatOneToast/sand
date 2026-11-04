use super::Lifetime;
use crate::{
    entity::{EntityKind, PlayerKind},
    ir::Actions,
    registration::{DatapackRegistration, IntoDatapack},
};

#[derive(Clone, Copy)]
pub(super) struct StatusPolicy {
    pub refresh: bool,
    pub limit: u32,
}

/// A reusable timed status built on the same owned lifetime as abilities.
/// Reapplying refreshes duration without repeating start/stop callbacks by
/// default. `stacking` instead admits independently expiring instances up to
/// a per-owner limit. Timers count loaded ticks; refreshes reach unloaded
/// instances when they next load. Removal invalidates unloaded instances too.
/// Each stack can own ordinary State through the configured lifetime.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::StatusEffect", module = "sand::systems",
summary = "A reusable timed status built on the same owned lifetime as abilities.",
context = "A reusable timed status built on the same owned lifetime as abilities. Reapplying refreshes duration without repeating start/stop callbacks by default. `stacking` instead admits independently expiring instances up to a per-owner limit. Timers count loaded ticks; refreshes reach unloaded instances when they next load. Removal invalidates unloaded instances too. Each stack can own ordinary State through the configured lifetime.",
minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
use_when = ["Composing reusable owner-bound gameplay"],
avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
example = "use sand::systems::StatusEffect;",
)]
pub struct StatusEffect<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
}
impl<K: EntityKind> StatusEffect<K> {
    /// Define a refreshing status from its duration, tick work and cleanup.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::StatusEffect::new", module = "sand::systems",
    kind = "method",
    summary = "Define a refreshing status from its duration, tick work and cleanup.",
    context = "Adds per-owner status admission to the lifetime. By default reapplication refreshes duration without repeating start callbacks or consuming another capacity slot.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(lifetime = "Owned lifetime supplying identity, admission capacity, duration and cleanup."),
    returns = "The StatusEffect definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{StatusEffect, Lifetime};\n\nfn demonstrate(lifetime: Lifetime) {\n    let _result = StatusEffect::new(lifetime);\n}",
    )]
    pub fn new(mut lifetime: Lifetime<K>) -> Self {
        lifetime.status = Some(StatusPolicy {
            refresh: true,
            limit: 1,
        });
        Self { lifetime }
    }
    /// Admit separate stacks rather than refreshing; limit must be positive.
    /// Each stack has independent attachment, callbacks, expiry and cleanup.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::StatusEffect::stacking", module = "sand::systems",
    kind = "method",
    summary = "Admit separate stacks rather than refreshing; limit must be positive.",
    context = "Changes reapplication from refresh to independent stacks up to a positive per-owner limit. Each admitted stack has its own duration, callbacks and attached State.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(limit = "Positive maximum independently active stacks per owner."),
    returns = "The StatusEffect definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{StatusEffect, Lifetime};\n\nfn demonstrate(value: StatusEffect) {\n    let _result = value.stacking(3);\n}",
    )]
    pub fn stacking(mut self, limit: u32) -> Self {
        self.lifetime.status = Some(StatusPolicy {
            refresh: false,
            limit,
        });
        self
    }
    /// Apply or refresh for the current owner. Rejected stacks run no callbacks.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::StatusEffect::apply", module = "sand::systems",
    kind = "method",
    summary = "Apply or refresh for the current owner.",
    context = "Applies the registered status to the current owner. Refresh mode extends existing work; stacking mode admits another instance only below both owner and global limits.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{StatusEffect, Lifetime};\n\nfn demonstrate(value: StatusEffect) {\n    let _result = value.apply();\n}",
    )]
    pub fn apply(&self) -> Actions {
        self.lifetime.start()
    }
    /// Remove all stacks belonging to this owner, including unloaded stacks
    /// when they next become observable by the runtime.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::StatusEffect::remove", module = "sand::systems",
    kind = "method",
    summary = "Remove all stacks belonging to this owner, including unloaded stacks when they next become observable by the runtime.",
    context = "Removes all stacks of this status for the current owner through lifetime cancellation. Unloaded instances clean up when they return.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{StatusEffect, Lifetime};\n\nfn demonstrate(value: StatusEffect) {\n    let _result = value.remove();\n}",
    )]
    pub fn remove(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> IntoDatapack for StatusEffect<K> {
    fn into_datapack(self) -> DatapackRegistration {
        self.lifetime.into_datapack()
    }
}
