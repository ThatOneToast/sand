use super::{Lifetime, TaskContext, identity};
use crate::{
    IntoCommands, McFunction,
    entity::{AnyEntity, EntityContext, EntityKind, PlayerKind},
    ir::{Actions, Cmd},
    registration::{DatapackRegistration, IntoDatapack},
};
use sand_commands::{
    CommandError, CommandProfile, CommandResult, Selector, Target, TargetArgument, Validate,
};
use sand_components::{
    BlockId, BlockPredicate, ComponentContent, DatapackComponent, LocationPredicate, Predicate,
    PredicateId, PredicateRoot, ResourceLocation,
};
use std::sync::Arc;

/// Bounded sampled projectile motion over an owned lifetime. Collision tests
/// occur at intervals no greater than min(radius, 0.25) blocks, up to sixteen
/// blocks per tick and 256 samples. These are point
/// samples, not swept hitboxes: thin blocks or entities between samples can be
/// missed. No chunks are loaded. Unloaded samples end the projectile.
///
/// Entity collision checks at most sixteen nearest candidates per sample,
/// excluding the owner. A hit runs once in instance context, then stops the
/// lifetime. `TaskContext::target` resolves the selected hit entity, and `owner`
/// resolves the original caster even when the hit callback changes execution.
#[derive(Clone)]
#[sand_macros::api(
registry = sand_api_contract,
path = "sand::systems::Projectile", module = "sand::systems",
summary = "Bounded sampled projectile motion over an owned lifetime.",
context = "Bounded sampled projectile motion over an owned lifetime. Collision tests occur at intervals no greater than min(radius, 0.25) blocks, up to sixteen blocks per tick and 256 samples. These are point samples, not swept hitboxes: thin blocks or entities between samples can be missed. No chunks are loaded. Unloaded samples end the projectile.  Entity collision checks at most sixteen nearest candidates per sample, excluding the owner. A hit runs once in instance context, then stops the lifetime. `TaskContext::target` resolves the selected hit entity, and `owner` resolves the original caster even when the hit callback changes execution.",
minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
use_when = ["Composing reusable owner-bound gameplay"],
avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
example = "use sand::systems::Projectile;",
)]
pub struct Projectile<K: EntityKind = PlayerKind> {
    pub(super) lifetime: Lifetime<K>,
    speed: f64,
    radius: f64,
    targets: Selector,
    passable: BlockPredicate,
    hit: Arc<dyn Fn(TaskContext<K>) -> Actions>,
}
impl<K: EntityKind> Projectile<K> {
    /// Wrap an owned lifetime; defaults to one block per tick, radius 0.5 and
    /// air-only passage. Its tick callback is replaced by projectile motion.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::new", module = "sand::systems",
    kind = "method",
    summary = "Wrap an owned lifetime; defaults to one block per tick, radius 0.5 and air-only passage.",
    context = "Uses the supplied lifetime for admission, duration and cleanup. Projectile motion replaces its tick callback with one-block-per-tick sampling and a half-block entity radius.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(lifetime = "Owned lifetime supplying identity, admission capacity, duration and cleanup."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(lifetime: Lifetime) {\n    let _result = Projectile::new(lifetime);\n}",
    )]
    pub fn new(lifetime: Lifetime<K>) -> Self {
        Self {
            lifetime,
            speed: 1.0,
            radius: 0.5,
            targets: Target::entities().into(),
            passable: BlockPredicate::new().blocks(vec![
                BlockId::minecraft("air").unwrap(),
                BlockId::minecraft("cave_air").unwrap(),
                BlockId::minecraft("void_air").unwrap(),
            ]),
            hit: Arc::new(|_| Actions::default()),
        }
    }
    /// Set speed in blocks per tick, finite and in (0, 16].
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::speed", module = "sand::systems",
    kind = "method",
    summary = "Set speed in blocks per tick, finite and in (0, 16].",
    context = "Sets forward travel per loaded tick. Finite values in (0, 16] are subdivided into samples at most min(radius, 0.25) blocks apart; export rejects more than 256 samples per tick.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(speed = "Finite forward distance per loaded tick, greater than zero and at most 16 blocks."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.speed(1.5);\n}",
    )]
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }
    /// Set entity sampling radius, finite and in (0, 8].
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::radius", module = "sand::systems",
    kind = "method",
    summary = "Set entity sampling radius, finite and in (0, 8].",
    context = "Sets the entity search radius at each motion sample. Finite values in (0, 8] are accepted; this is a sampled proximity test rather than swept hitbox collision.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(radius = "Finite entity search radius, greater than zero and at most 8 blocks."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.radius(0.75);\n}",
    )]
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }
    /// Select potential hit entities using Sand's canonical target filters.
    /// Sampling replaces supplied distance, ordering and limit with the radius,
    /// nearest ordering and a sixteen-candidate cap; other filters are preserved.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::targets", module = "sand::systems",
    kind = "method",
    summary = "Select potential hit entities using Sand's canonical target filters.",
    context = "Filters candidate hit entities through a canonical target selector. Motion replaces supplied distance, ordering and limit with the sampling radius, nearest ordering and a sixteen-candidate limit, preserves other filters and excludes the owner identity.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(targets = "Canonical target filters for candidate entity hits."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.targets(Target::players());\n}",
    )]
    pub fn targets(mut self, targets: impl TargetArgument) -> Self {
        self.targets = targets.into_target_selector();
        self
    }
    /// Set the typed block predicate through which samples may travel.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::pass_through", module = "sand::systems",
    kind = "method",
    summary = "Set the typed block predicate through which samples may travel.",
    context = "Replaces the air-only passability predicate. A sample in a nonmatching block triggers a block hit with no entity target and then stops the projectile.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(passable = "The passable used to configure or execute this operation."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile, passable: sand::predicate::BlockPredicate) {\n    let _result = value.pass_through(passable);\n}",
    )]
    pub fn pass_through(mut self, passable: BlockPredicate) -> Self {
        self.passable = passable;
        self
    }
    /// Handle either a block or entity hit. Target traversal is empty for block
    /// hits. The lifetime stops after this callback, including if it returns.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::on_hit", module = "sand::systems",
    kind = "method",
    summary = "Handle either a block or entity hit.",
    context = "Builds actions in instance context for the first entity or block collision. Use target for entity-specific work or damage_target for owner-attributed native damage.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    params(hit = "The hit used to configure or execute this operation."),
    returns = "The Projectile definition with this configuration; register the completed definition before invoking its runtime actions.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.on_hit(|ctx| ctx.damage_target(6.0, sand::command::DamageKind::Magic));\n}",
    )]
    pub fn on_hit<R: IntoCommands>(mut self, hit: impl Fn(TaskContext<K>) -> R + 'static) -> Self {
        self.hit = Arc::new(move |ctx| hit(ctx).into_commands());
        self
    }
    /// Launch in the current owner's facing direction.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::launch", module = "sand::systems",
    kind = "method",
    summary = "Launch in the current owner's facing direction.",
    context = "Starts a projectile for the current executor using lifetime admission. For costs and cooldowns, register it through Ability::projectile and activate that ability instead.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.launch();\n}",
    )]
    pub fn launch(&self) -> Actions {
        self.lifetime.start()
    }
    /// Cancel the owner's projectiles through their shared lifetime.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::Projectile::cancel", module = "sand::systems",
    kind = "method",
    summary = "Cancel the owner's projectiles through their shared lifetime.",
    context = "Cancels all projectiles of this definition belonging to the current owner. Cleanup is immediate for loaded instances and deferred until load for others.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{Projectile, Lifetime};\n\nfn demonstrate(value: Projectile) {\n    let _result = value.cancel();\n}",
    )]
    pub fn cancel(&self) -> Actions {
        self.lifetime.cancel()
    }
}
impl<K: EntityKind> TaskContext<K> {
    /// Apply native damage to an entity hit, attributed to the original owner.
    /// Missing targets or owners do nothing. Vanilla armor, immunity and damage
    /// rules remain authoritative; this does not synthesize an exact hit event.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::systems::TaskContext::damage_target", module = "sand::systems", kind = "method",
        summary = "Applies owner-attributed native damage to the captured hit target.",
        context = "Both hit and owner identities are captured before execution. Block hits have no target, and absent owners prevent unattributed fallback damage.",
        minecraft = "Resolves unique owner and target scores in a macro frame and emits the validated native damage command with by attribution.",
        use_when = ["Damaging an entity hit by an owned projectile"], avoid_when = ["Updating a custom health State field instead of vanilla health"],
        params(amount = "Native hit-point damage or a typed DamageAmount.", kind = "The native damage kind."),
        returns = "Retained actions that execute this operation in Minecraft when called from the documented context.", example = "use sand::prelude::*;\nuse sand::systems::{TaskContext, Lifetime};\n\nfn demonstrate(value: TaskContext) {\n    let _result = value.damage_target(6.0, sand::command::DamageKind::Magic);\n}",
    )]
    pub fn damage_target(
        &self,
        amount: impl Into<sand_commands::DamageAmount>,
        kind: sand_commands::DamageKind,
    ) -> Actions {
        Cmd::OwnedContext(Box::new(identity::OwnedContext {
            scope: self.scope.clone(),
            kind: K::LABEL,
            field: "target",
            body: Actions::default(),
            child: None,
            damage: Some((amount.into(), kind)),
        }))
        .into_commands()
    }

    /// Run actions as the hit entity. Block hits and unavailable targets produce
    /// no actions; identities are captured before nested callbacks execute.
    #[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::TaskContext::target", module = "sand::systems",
    params(body = "Actions authored in hit entity context."),
    kind = "method",
    summary = "Run actions as the hit entity.",
    context = "Call inside a projectile hit callback to resolve the captured entity hit. Block hits have no target and therefore skip the body.",
    minecraft = "Exports owner-bound marker instances with per-instance timers, stable identities, capacity admission and idempotent cleanup.",
    use_when = ["Composing reusable owner-bound gameplay"],
    avoid_when = ["Unloaded work must run or chunks must be force-loaded"],
    returns = "Retained actions that execute this operation in Minecraft when called from the documented context.",
    example = "use sand::prelude::*;\nuse sand::systems::{TaskContext, Lifetime};\n\nfn demonstrate(value: TaskContext) {\n    let _result = value.target(|_| cmd::say(\"Entity hit\"));\n}",
    )]
    pub fn target<R: IntoCommands>(
        &self,
        body: impl FnOnce(EntityContext<AnyEntity>) -> R,
    ) -> Actions {
        Cmd::OwnedContext(Box::new(identity::OwnedContext {
            scope: self.scope.clone(),
            kind: "entity",
            field: "target",
            body: body(EntityContext::default()).into_commands(),
            child: None,
            damage: None,
        }))
        .into_commands()
    }
}
impl<K: EntityKind> IntoDatapack for Projectile<K> {
    fn into_datapack(self) -> DatapackRegistration {
        let scope = self.lifetime.id.clone();
        let step_scope = scope.clone();
        let lifetime = self
            .lifetime
            .clone()
            .every(crate::state::Ticks::new(1))
            .on_tick(move |_| identity::call(&step_scope, "projectile_move"));
        let mut registration = lifetime.into_datapack().component(Predicate::new(
            PredicateId::custom(identity::resource(&scope, "projectile_passable")),
            PredicateRoot::location(LocationPredicate::new().block(self.passable.clone())),
        ));
        for part in [
            "projectile_move",
            "projectile_sample",
            "projectile_search",
            "projectile_candidate",
            "projectile_hit",
            "projectile_callback",
        ] {
            registration = registration.component(ProjectileFunction {
                location: identity::resource(&scope, part),
                definition: self.clone(),
                part,
            });
        }
        registration
    }
}
struct ProjectileFunction<K: EntityKind> {
    location: ResourceLocation,
    definition: Projectile<K>,
    part: &'static str,
}
impl<K: EntityKind> ProjectileFunction<K> {
    fn build(&self) -> CommandResult<McFunction> {
        let d = &self.definition;
        for (field, value, max) in [("speed", d.speed, 16.0), ("radius", d.radius, 8.0)] {
            if !value.is_finite() || value <= 0.0 || value > max {
                return Err(CommandError::new(
                    "projectile",
                    field,
                    format!("must be finite and in (0, {max}]"),
                ));
            }
        }
        let sample_count = (d.speed / d.radius.min(0.25)).ceil();
        if sample_count > 256.0 {
            return Err(CommandError::new(
                "projectile",
                "sampling",
                "radius and speed require more than 256 samples per tick; reduce speed or increase radius",
            ));
        }
        let id = crate::function::resolve_export_location(&d.lifetime.id)?;
        let call = |part| identity::resource(&id, part).to_string();
        let objective = identity::objective(&id)?.to_string();
        let tag = identity::task_tag(&id)?;
        let storage = format!("{}:__sand_runtime", id.namespace());
        let marker = format!(
            "@e[type=minecraft:marker,tag={tag},scores={{{objective}=$(instance)}},limit=1]"
        );
        if self.part == "projectile_callback" {
            return Ok(
                McFunction::new(self.location.clone()).commands((d.hit)(TaskContext::new(id)))
            );
        }
        let targets = d.targets.clone().with_sampling_bounds(d.radius, 16);
        targets.validate(&CommandProfile::unprofiled())?;
        let lines: Vec<String> = match self.part {
            "projectile_move" => {
                let count = sample_count as usize;
                let step = d.speed / count as f64;
                let mut lines = vec![];
                for _ in 0..count {
                    lines.push(format!(
                        "execute unless entity @s[type=minecraft:marker,tag={tag}] run return 0"
                    ));
                    lines.push(format!("execute at @s rotated as @s run tp @s ^ ^ ^{step}"));
                    lines.push(format!(
                        "execute at @s rotated as @s run function {}",
                        call("projectile_sample")
                    ));
                }
                lines
            }
            "projectile_sample" => vec![
                "execute unless loaded ~ ~ ~ run return run function ".to_string() + &call("stop"),
                format!(
                    "execute unless predicate {} run return run function {}",
                    call("projectile_passable"),
                    call("projectile_hit")
                ),
                format!("function {} with entity @s data", call("projectile_search")),
            ],
            "projectile_search" => vec![format!(
                "$execute as {targets} unless entity @s[type=minecraft:marker] unless entity @s[nbt={{Health:0.0f}}] unless score @s {objective} matches $(owner) run function {} {{instance:$(instance)}}",
                call("projectile_candidate")
            )],
            "projectile_candidate" => vec![
                format!("$execute unless entity {marker} run return 0"),
                format!(
                    "execute unless score @s {objective} matches 1.. if score #next {objective} matches 2147483647 run return 0"
                ),
                format!(
                    "execute unless score @s {objective} matches 1.. run scoreboard players add #next {objective} 1"
                ),
                format!(
                    "execute unless score @s {objective} matches 1.. run scoreboard players operation @s {objective} = #next {objective}"
                ),
                format!(
                    "execute store result storage {storage} hit.target int 1 run scoreboard players get @s {objective}"
                ),
                format!(
                    "$data modify entity {marker} data.target set from storage {storage} hit.target"
                ),
                format!("data remove storage {storage} hit"),
                format!(
                    "$execute as {marker} at @s run function {}",
                    call("projectile_hit")
                ),
            ],
            "projectile_hit" => vec![
                format!("function {}", call("projectile_callback")),
                format!("function {}", call("stop")),
            ],
            _ => unreachable!(),
        };
        Ok(McFunction::new(self.location.clone()).commands(lines))
    }
}
impl<K: EntityKind> DatapackComponent for ProjectileFunction<K> {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }
    fn component_dir(&self) -> &'static str {
        "function"
    }
    fn file_extension(&self) -> &'static str {
        "mcfunction"
    }
    fn to_json(&self) -> serde_json::Value {
        match self.try_content().expect("projectile must validate") {
            ComponentContent::Text(s) => s.into(),
            ComponentContent::Json(v) => v,
        }
    }
    fn try_content(&self) -> sand_components::error::Result<ComponentContent> {
        self.build()
            .map_err(|error| sand_components::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "projectile".into(),
                field: error.field.clone(),
                message: error.to_string(),
            })?
            .try_content()
    }
}

impl<K: EntityKind> AsRef<Lifetime<K>> for Projectile<K> {
    fn as_ref(&self) -> &Lifetime<K> {
        &self.lifetime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_sampling_parameters_fail_export() {
        for (speed, radius, field) in [
            (0.0, 0.5, "speed"),
            (f64::NAN, 0.5, "speed"),
            (16.1, 0.5, "speed"),
            (1.0, 0.0, "radius"),
            (1.0, 8.1, "radius"),
            (16.0, 0.001, "sampling"),
        ] {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let definition = Projectile::<PlayerKind>::new(Lifetime::new(
                "test:bolt".parse().unwrap(),
                crate::state::Ticks::new(1),
            ))
            .speed(speed)
            .radius(radius);
            let function = ProjectileFunction {
                location: identity::resource(&definition.lifetime.id, "projectile_move"),
                definition,
                part: "projectile_move",
            };
            assert_eq!(
                function
                    .build()
                    .err()
                    .expect("invalid definition must fail")
                    .field,
                field
            );
        }
    }
}
