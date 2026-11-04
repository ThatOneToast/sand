use super::Lifetime;
use crate::{
    IntoCommands,
    condition::Condition,
    entity::{EntityCooldown, EntityKind, EntityScore, EntityStateField, PlayerKind},
    ir::Actions,
    registration::{DatapackRegistration, IntoDatapack},
    state::Ticks,
};

/// An activation policy around an owned lifetime. Costs and cooldowns use the
/// owner's canonical State fields, so UI and gameplay read the same values.
/// Admission checks all conditions before creating an instance; successful
/// admission commits cost and cooldown before any user callback can reenter.
/// Attach the referenced State to owners through the normal State lifecycle.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::Ability", module = "sand::systems",
summary = "An activation policy around an owned lifetime.",
context = "An activation policy around an owned lifetime. Costs and cooldowns use the owner's canonical State fields, so UI and gameplay read the same values. Admission checks all conditions before creating an instance; successful admission commits cost and cooldown before any user callback can reenter. Attach the referenced State to owners through the normal State lifecycle.",
minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
use_when = ["Composing reusable owner-bound gameplay"],
avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
example = "use sand::systems::Ability;",
)]
pub struct Ability<K: EntityKind = PlayerKind> {
    lifetime: Lifetime<K>,
    projectile: Option<super::Projectile<K>>,
    cost: Option<(EntityScore<i32>, u32)>,
    cooldown: Option<(EntityCooldown, Ticks)>,
    conditions: Vec<Condition>,
}
impl<K: EntityKind> Ability<K> {
    /// Wrap a configured lifetime, retaining its identity and callbacks.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::new", module = "sand::systems",
    kind = "method",
    summary = "Wrap a configured lifetime, retaining its identity and callbacks.",
    context = "Wraps one registered lifetime with activation guards and committed owner State mutations. Use Ability::projectile when the admitted instance itself should be a projectile.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(lifetime = "Owned lifetime supplying identity, admission capacity, duration and cleanup."),
    returns = "The Ability definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(lifetime: Lifetime) {\n    let _result = Ability::new(lifetime);\n}",
    )]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            projectile: None,
            cost: None,
            cooldown: None,
            conditions: vec![],
        }
    }
    /// Use a projectile as the admitted ability instance. Cost, cooldown,
    /// capacity and projectile creation share one admission boundary.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Ability::projectile", module = "sand::systems", kind = "method",
        summary = "Builds an ability around one managed projectile definition.",
        context = "Shares the projectile lifetime admission and identity, so capacity rejection cannot spend the owner's cost or cooldown.",
        minecraft = "Commits owner State after marker admission and before callbacks or motion; the projectile's canonical lifetime owns cleanup.",
        use_when = ["Defining a projectile spell with cost and cooldown"], avoid_when = ["Launching an unrelated child projectile after already spending resources"],
        params(projectile = "The projectile definition with motion and hit behavior."), returns = "The Ability definition with this configuration; register the completed definition before invoking its runtime actions.", example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime, Projectile};\n\nfn demonstrate(projectile: Projectile) {\n    let _result = Ability::projectile(projectile);\n}",
    )]
    pub fn projectile(projectile: super::Projectile<K>) -> Self {
        let mut ability = Self::new(projectile.lifetime.clone());
        ability.projectile = Some(projectile);
        ability
    }
    /// Charge a nonnegative amount from the owner's integer State field. A new
    /// call replaces the earlier cost; values above i32::MAX fail export.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::cost", module = "sand::systems",
    kind = "method",
    summary = "Charge a nonnegative amount from the owner's integer State field.",
    context = "Uses an existing owner score as the resource balance. Admission checks that subtraction fits the field minimum, then commits the cost before callbacks; rejection spends nothing.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(field = "Canonical owner score used as the resource balance.", amount = "Nonnegative resource points spent only after successful admission."),
    returns = "The Ability definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability, mana: sand::entity::EntityScore<i32>) {\n    let _result = value.cost(mana, 20);\n}",
    )]
    pub fn cost(mut self, field: EntityScore<i32>, amount: u32) -> Self {
        self.cost = Some((field, amount));
        self
    }
    /// Require a ready canonical cooldown and start it on successful admission.
    /// Normal State lifecycle ticking remains responsible for its countdown.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::cooldown", module = "sand::systems",
    kind = "method",
    summary = "Require a ready canonical cooldown and start it on successful admission.",
    context = "Requires an existing owner cooldown to be ready and starts it only after admission. Mark the State field auto_tick or provide its normal countdown system.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(field = "Canonical owner cooldown; its State lifecycle must advance the timer.", duration = "Positive duration in loaded Minecraft ticks."),
    returns = "The Ability definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability, cooldown: sand::entity::EntityCooldown) {\n    let _result = value.cooldown(cooldown, Ticks::seconds(3));\n}",
    )]
    pub fn cooldown(mut self, field: EntityCooldown, duration: Ticks) -> Self {
        self.cooldown = Some((field, duration));
        self
    }
    /// Add an owner-context activation condition. Every condition must pass.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::when", module = "sand::systems",
    kind = "method",
    summary = "Add an owner-context activation condition.",
    context = "Adds an owner-context condition to admission. All configured conditions must pass alongside resource, cooldown, State and capacity checks.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(condition = "Typed runtime condition evaluated in the execution context described above."),
    returns = "The Ability definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability, condition: Condition) {\n    let _result = value.when(condition);\n}",
    )]
    pub fn when(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }
    /// Attempt activation; rejected attempts neither spend cost nor start cooldown.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::activate", module = "sand::systems",
    kind = "method",
    summary = "Attempt activation; rejected attempts neither spend cost nor start cooldown.",
    context = "Runs admission for the current owner. A successful attempt creates the owned instance and commits cost and cooldown before callbacks; rejection leaves those fields unchanged.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability) {\n    let _result = value.activate();\n}",
    )]
    pub fn activate(&self) -> Actions {
        self.lifetime.start()
    }
    /// Show failed cost, cooldown, custom and State requirements plus active
    /// capacity to the current player, without activating or changing State.
    /// Non-player execution is a no-op; call from a developer command or UI.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::Ability::diagnose", module = "sand::systems", kind = "method",
        summary = "Explains activation requirements to the current player without spending resources.",
        context = "Reports failed cost, cooldown, custom and membership checks and current global lifetime capacity. The registered ability supplies the same predicates used for admission.",
        minecraft = "Executes read-only conditions and tellraw messages for the current player.",
        use_when = ["Debugging why a player cannot activate an ability"], avoid_when = ["Running automatic chat output every tick"],
        returns = "Retained actions that execute this operation in Minecraft when called from the documented context.", example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability) {\n    let _result = value.diagnose();\n}",
    )]
    pub fn diagnose(&self) -> Actions {
        super::identity::call(&self.lifetime.id, "diagnose")
    }
    /// Cancel the owner's instances, preserving the committed cost and cooldown.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Ability::cancel", module = "sand::systems",
    kind = "method",
    summary = "Cancel the owner's instances, preserving the committed cost and cooldown.",
    context = "Stops this owner active ability instances and invalidates unloaded ones. Previously spent resources and the cooldown remain committed.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Ability, Lifetime};\n\nfn demonstrate(value: Ability) {\n    let _result = value.cancel();\n}",
    )]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> IntoDatapack for Ability<K> {
    fn into_datapack(mut self) -> DatapackRegistration {
        self.lifetime
            .diagnostics
            .extend(
                self.conditions
                    .iter()
                    .enumerate()
                    .map(|(index, condition)| {
                        (
                            condition.clone(),
                            format!("Custom activation condition {} is false", index + 1),
                        )
                    }),
            );
        if let Some((field, amount)) = self.cost {
            if let Ok(amount) = i32::try_from(amount) {
                let minimum = field.descriptor().bounds.map_or(0, |(min, _)| min.max(0));
                let Some(threshold) = amount.checked_add(minimum) else {
                    self.lifetime.invalid =
                        Some(("cost", "cost cannot fit above the resource field minimum"));
                    return self.lifetime.into_datapack();
                };
                let condition = field
                    .matches(threshold..)
                    .expect("nonnegative bounded range")
                    .condition();
                self.lifetime.diagnostics.push((
                    condition.clone(),
                    format!("Insufficient resource: requires {amount}"),
                ));
                self.conditions.push(condition);
                self.lifetime.commit.extend([field.bind().add(-amount)]);
            } else {
                // Retain invalid values until normal export validation.
                self.lifetime.invalid = Some(("cost", "cost must not exceed 2147483647"));
            }
        }
        if let Some((field, duration)) = self.cooldown {
            if duration.get() == 0 || duration.get() > i32::MAX as u32 {
                self.lifetime.invalid = Some(("cooldown", "duration must be in 1..=2147483647"));
            }
            self.lifetime
                .diagnostics
                .push((field.bind().ready(), "Cooldown is not ready".into()));
            self.conditions.push(field.bind().ready());
            self.lifetime
                .commit
                .extend([field.bind().start(duration).into_commands()]);
        }
        self.lifetime.guard = Some(Condition::all(self.conditions));
        if let Some(mut projectile) = self.projectile {
            projectile.lifetime = self.lifetime;
            projectile.into_datapack()
        } else {
            self.lifetime.into_datapack()
        }
    }
}
impl<K: EntityKind> AsRef<Lifetime<K>> for Ability<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}
