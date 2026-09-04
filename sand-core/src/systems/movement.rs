//! Typed movement helpers (`systems-movement` feature).
//!
//! Provides high-level command builders for common movement effects:
//! push, launch, speed boost, and slow. All builders emit pure-vanilla commands
//! using local-coordinate teleports and potion effects — no mods required.
//!
//! # Push mechanics
//!
//! `PushAway` uses the facing-entity teleport trick:
//! ```text
//! execute as <targets> at @s facing entity <source> feet
//!     run tp @s ^0 ^<lift> ^-<strength>
//! ```
//! Each target is teleported backward (away from the source) and optionally
//! upward. The source entity must be present in the world at command time.
//!
//! # Launch mechanics
//!
//! `Launch` uses a relative upward teleport:
//! ```text
//! execute as <targets> run tp @s ~ ~<amount> ~
//! ```
//! This is an instant positional shift, not a physics impulse. It works
//! reliably in datapacks without NBT Motion modification.
//!
//! # Speed / slow
//!
//! `SpeedBoost` and `Slow` wrap `effect give` with typed [`EffectId`] values
//! and convert a 0–1 strength fraction to the appropriate vanilla amplifier.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::systems::movement::{PushAway, Launch, SpeedBoost, Slow};
//! use sand_core::cmd::Target;
//! use sand_core::state::Ticks;
//!
//! // Shockwave push: push all nearby non-player entities away from @s
//! let cmds = PushAway::new()
//!     .source(Target::self_())
//!     .targets(Target::nearby(6.0).excluding_players())
//!     .strength(1.5)
//!     .lift(0.25)
//!     .build();
//!
//! // Launch all nearby entities upward
//! let cmds = Launch::targets(Target::nearby(4.0))
//!     .amount(0.7)
//!     .build();
//!
//! // Speed boost self for 5 seconds
//! let cmd = SpeedBoost::target(Target::self_())
//!     .amount(0.4)
//!     .duration(Ticks::seconds(5))
//!     .build();
//!
//! // Slow nearby entities for 3 seconds
//! let cmd = Slow::targets(Target::nearby(5.0))
//!     .amount(0.3)
//!     .duration(Ticks::seconds(3))
//!     .build();
//! ```

use sand_commands::selector::{Selector, TargetArgument};
use sand_components::{EffectId, Ticks};

// ── PushAway ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::movement::PushAway",
    module = "sand::systems",
    summary = "Pushes entities away from a source entity using local-coordinate teleport.",
    context = "Pushes entities away from a source entity using local-coordinate teleport. Uses the `execute facing entity` trick so each target is displaced in the direction away from the source — no nearest-player assumptions are made.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::movement::PushAway;",
    availability = ["Cargo feature: systems-movement"],
)]
/// Pushes entities away from a source entity using local-coordinate teleport.
///
/// Uses the `execute facing entity` trick so each target is displaced in the
/// direction away from the source — no nearest-player assumptions are made.
///
/// # Command emitted
/// ```text
/// execute as <targets> at @s facing entity <source> feet run tp @s ^0 ^<lift> ^-<strength>
/// ```
#[derive(Debug, Clone)]
pub struct PushAway {
    source: Option<Selector>,
    targets: Option<Selector>,
    strength: f64,
    lift: f64,
}

impl Default for PushAway {
    fn default() -> Self {
        Self::new()
    }
}

impl PushAway {
    /// Create a new `PushAway` builder with default strength `1.0` and no lift.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new `PushAway` builder with default strength `1.0` and no lift.",
        context = "Create a new `PushAway` builder with default strength `1.0` and no lift. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "A `PushAway` representing a new `PushAway` builder with default strength `1.0` and no lift.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let push_away = sand::systems::movement::PushAway::new();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn new() -> Self {
        Self {
            source: None,
            targets: None,
            strength: 1.0,
            lift: 0.0,
        }
    }

    /// Set the source entity (the "center" of the push, typically `@s`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::source",
        module = "sand::systems",
        kind = "method",
        summary = "Set the source entity (the \"center\" of the push, typically `@s`).",
        context = "Set the source entity (the \"center\" of the push, typically `@s`). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(source = "`source` provides the Minecraft target selection used to set the source entity (the \"center\" of the push, typically `@s`)."),
        returns = "The `PushAway` value with the documented change applied to set the source entity (the \"center\" of the push, typically `@s`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(push_away_value: sand::systems::movement::PushAway, source: sand::command::Target)  {\n    let updated_push_away = push_away_value.source(source);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn source(mut self, source: impl TargetArgument) -> Self {
        self.source = Some(source.into_target_selector());
        self
    }

    /// Set the entities to push.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::targets",
        module = "sand::systems",
        kind = "method",
        summary = "Set the entities to push.",
        context = "Set the entities to push. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(targets = "`targets` provides the Minecraft target selection used to set the entities to push."),
        returns = "The `PushAway` value with the documented change applied to set the entities to push.",
        example = "use sand::prelude::*;\n\nfn demonstrate(push_away_value: sand::systems::movement::PushAway, targets: sand::command::Target)  {\n    let updated_push_away = push_away_value.targets(targets);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn targets(mut self, targets: impl TargetArgument) -> Self {
        self.targets = Some(targets.into_target_selector());
        self
    }

    /// How far to push each target (in blocks along the away vector).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::strength",
        module = "sand::systems",
        kind = "method",
        summary = "How far to push each target (in blocks along the away vector).",
        context = "How far to push each target (in blocks along the away vector). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(strength = "`strength` is used to represent how far to push each target (in blocks along the away vector)."),
        returns = "The `PushAway` value with the documented change applied to represent how far to push each target (in blocks along the away vector).",
        example = "use sand::prelude::*;\n\nfn demonstrate(push_away_value: sand::systems::movement::PushAway, strength: f64)  {\n    let updated_push_away = push_away_value.strength(strength);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    /// How far to lift each target upward (in blocks, default `0.0`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::lift",
        module = "sand::systems",
        kind = "method",
        summary = "How far to lift each target upward (in blocks, default `0.0`).",
        context = "How far to lift each target upward (in blocks, default `0.0`). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(lift = "`lift` is used to represent how far to lift each target upward (in blocks, default `0.0`)."),
        returns = "The `PushAway` value with the documented change applied to represent how far to lift each target upward (in blocks, default `0.0`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(push_away_value: sand::systems::movement::PushAway, lift: f64)  {\n    let updated_push_away = push_away_value.lift(lift);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn lift(mut self, lift: f64) -> Self {
        self.lift = lift;
        self
    }

    /// Build the command string(s).
    ///
    /// Returns one command per source–targets pair. If source or targets are
    /// not set, defaults to `@s` (source) or all entities (targets).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::PushAway::build",
        module = "sand::systems",
        kind = "method",
        summary = "Build the command string(s). Returns one command per source–targets pair. If source or targets are not set, defaults to `@s` (source) or all entities (targets).",
        context = "Build the command string(s). Returns one command per source–targets pair. If source or targets are not set, defaults to `@s` (source) or all entities (targets). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "Returns one command per source–targets pair. If source or targets are not set, defaults to `@s` (source) or all entities (targets).",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns one command per source–targets pair. If source or targets are not set, defaults to `@s` (source) or all entities (targets).",
        example = "use sand::prelude::*;\n\nfn demonstrate(push_away_value: sand::systems::movement::PushAway)  {\n    let values = push_away_value.build();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn build(self) -> Vec<String> {
        let source = self.source.unwrap_or_else(Selector::self_);
        let targets = self
            .targets
            .map(|t| t.to_string())
            .unwrap_or_else(|| "@e".to_string());

        let forward = fmt_local_coord(-self.strength);
        let up = fmt_local_coord(self.lift);

        vec![format!(
            "execute as {targets} at @s facing entity {source} feet run tp @s ^0 {up} {forward}"
        )]
    }
}

// ── Launch ────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::movement::Launch",
    module = "sand::systems",
    summary = "Launches entities upward by teleporting them a fixed distance along the Y axis.",
    context = "Launches entities upward by teleporting them a fixed distance along the Y axis. This is a positional shift, not a physics impulse — it is reliable across all entity types without NBT access. For a more natural arc, chain multiple smaller launches over successive ticks.",
    minecraft = "This is a positional shift, not a physics impulse — it is reliable across all entity types without NBT access. For a more natural arc, chain multiple smaller launches over successive ticks.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::movement::Launch;",
    availability = ["Cargo feature: systems-movement"],
)]
/// Launches entities upward by teleporting them a fixed distance along the Y axis.
///
/// This is a positional shift, not a physics impulse — it is reliable across all
/// entity types without NBT access. For a more natural arc, chain multiple smaller
/// launches over successive ticks.
///
/// # Command emitted
/// ```text
/// execute as <targets> run tp @s ~ ~<amount> ~
/// ```
#[derive(Debug, Clone)]
pub struct Launch {
    targets: Option<Selector>,
    amount: f64,
}

impl Default for Launch {
    fn default() -> Self {
        Self::new()
    }
}

impl Launch {
    /// Create a new `Launch` builder with default amount `0.5`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Launch::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new `Launch` builder with default amount `0.5`.",
        context = "Create a new `Launch` builder with default amount `0.5`. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "A `Launch` representing a new `Launch` builder with default amount `0.5`.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let launch = sand::systems::movement::Launch::new();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn new() -> Self {
        Self {
            targets: None,
            amount: 0.5,
        }
    }

    /// Shorthand: create a builder with targets already set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Launch::targets",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand: create a builder with targets already set.",
        context = "Shorthand: create a builder with targets already set. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(targets = "`targets` provides the Minecraft target selection used to use shorthand: create a builder with targets already set."),
        returns = "A `Launch` configured for shorthand: create a builder with targets already set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(targets: sand::command::Target)  {\n    let launch = sand::systems::movement::Launch::targets(targets);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn targets(targets: impl TargetArgument) -> Self {
        Self::new().with_targets(targets)
    }

    /// Set the entities to launch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Launch::with_targets",
        module = "sand::systems",
        kind = "method",
        summary = "Set the entities to launch.",
        context = "Set the entities to launch. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(targets = "`targets` provides the Minecraft target selection used to set the entities to launch."),
        returns = "The `Launch` value with the documented change applied to set the entities to launch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(launch_value: sand::systems::movement::Launch, targets: sand::command::Target)  {\n    let updated_launch = launch_value.with_targets(targets);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn with_targets(mut self, targets: impl TargetArgument) -> Self {
        self.targets = Some(targets.into_target_selector());
        self
    }

    /// How far to launch upward (in blocks).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Launch::amount",
        module = "sand::systems",
        kind = "method",
        summary = "How far to launch upward (in blocks).",
        context = "How far to launch upward (in blocks). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(amount = "`amount` provides the requested numeric amount used to represent how far to launch upward (in blocks)."),
        returns = "The `Launch` value with the documented change applied to represent how far to launch upward (in blocks).",
        example = "use sand::prelude::*;\n\nfn demonstrate(launch_value: sand::systems::movement::Launch, amount: f64)  {\n    let updated_launch = launch_value.amount(amount);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    /// Build the command string(s).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Launch::build",
        module = "sand::systems",
        kind = "method",
        summary = "Build the command string(s).",
        context = "Build the command string(s). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The ordered values produced to build the command string(s).",
        example = "use sand::prelude::*;\n\nfn demonstrate(launch_value: sand::systems::movement::Launch)  {\n    let values = launch_value.build();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn build(self) -> Vec<String> {
        let targets = self
            .targets
            .map(|t| t.to_string())
            .unwrap_or_else(|| "@s".to_string());

        let up = fmt_rel_coord(self.amount);
        vec![format!("execute as {targets} run tp @s ~ {up} ~")]
    }
}

// ── SpeedBoost ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::movement::SpeedBoost",
    module = "sand::systems",
    summary = "Applies a speed boost effect to one or more entities.",
    context = "Applies a speed boost effect to one or more entities. The `amount` (0.0–1.0+) is converted to a vanilla amplifier: `amplifier = (amount / 0.2).round() as u8`. Speed I (amplifier 0) ≈ +20% walk speed, Speed II ≈ +40%, etc.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::movement::SpeedBoost;",
    availability = ["Cargo feature: systems-movement"],
)]
/// Applies a speed boost effect to one or more entities.
///
/// The `amount` (0.0–1.0+) is converted to a vanilla amplifier:
/// `amplifier = (amount / 0.2).round() as u8`.
/// Speed I (amplifier 0) ≈ +20% walk speed, Speed II ≈ +40%, etc.
///
/// # Command emitted
/// ```text
/// effect give <targets> minecraft:speed <duration_seconds> <amplifier>
/// ```
#[derive(Debug, Clone)]
pub struct SpeedBoost {
    targets: Option<String>,
    amplifier: u8,
    duration: Ticks,
}

impl Default for SpeedBoost {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedBoost {
    /// Create a new `SpeedBoost` builder with amplifier `0` (Speed I) and 30 s duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new `SpeedBoost` builder with amplifier `0` (Speed I) and 30 s duration.",
        context = "Create a new `SpeedBoost` builder with amplifier `0` (Speed I) and 30 s duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "A `SpeedBoost` representing a new `SpeedBoost` builder with amplifier `0` (Speed I) and 30 s duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let speed_boost = sand::systems::movement::SpeedBoost::new();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn new() -> Self {
        Self {
            targets: None,
            amplifier: 0,
            duration: Ticks::seconds(30),
        }
    }

    /// Shorthand: create a builder for a single selector target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::target",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand: create a builder for a single selector target.",
        context = "Shorthand: create a builder for a single selector target. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(target = "`target` provides the entity, block, or command target used to use shorthand: create a builder for a single selector target."),
        returns = "A `SpeedBoost` configured for shorthand: create a builder for a single selector target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Target)  {\n    let speed_boost = sand::systems::movement::SpeedBoost::target(target);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn target(target: impl TargetArgument) -> Self {
        Self::new().with_target(target)
    }

    /// Shorthand: create a builder for an entity-targets set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::target_many",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand: create a builder for an entity-targets set.",
        context = "Shorthand: create a builder for an entity-targets set. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(targets = "`targets` provides the Minecraft target selection used to use shorthand: create a builder for an entity-targets set."),
        returns = "A `SpeedBoost` configured for shorthand: create a builder for an entity-targets set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(targets: sand::command::Target)  {\n    let speed_boost = sand::systems::movement::SpeedBoost::target_many(targets);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn target_many(targets: impl TargetArgument) -> Self {
        let mut s = Self::new();
        s.targets = Some(targets.to_string());
        s
    }

    /// Set the target selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::with_target",
        module = "sand::systems",
        kind = "method",
        summary = "Set the target selector.",
        context = "Set the target selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(target = "`target` provides the entity, block, or command target used to set the target selector."),
        returns = "The `SpeedBoost` value with the documented change applied to set the target selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(speed_boost_value: sand::systems::movement::SpeedBoost, target: sand::command::Target)  {\n    let updated_speed_boost = speed_boost_value.with_target(target);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn with_target(mut self, target: impl TargetArgument) -> Self {
        self.targets = Some(target.to_string());
        self
    }

    /// Set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4).
    ///
    /// Maps to `amplifier = (amount / 0.2).round().max(0) as u8`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::amount",
        module = "sand::systems",
        kind = "method",
        summary = "Set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4).",
        context = "Set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4). Maps to `amplifier = (amount / 0.2).round().max(0) as u8`.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(amount = "`amount` provides the requested numeric amount used to set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4)."),
        returns = "The `SpeedBoost` value with the documented change applied to set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4).",
        example = "use sand::prelude::*;\n\nfn demonstrate(speed_boost_value: sand::systems::movement::SpeedBoost, amount: f64)  {\n    let updated_speed_boost = speed_boost_value.amount(amount);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn amount(mut self, amount: f64) -> Self {
        self.amplifier = ((amount / 0.2).round() as i32).max(0) as u8;
        self
    }

    /// Set the speed amplifier directly (0 = Speed I, 1 = Speed II, …).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::amplifier",
        module = "sand::systems",
        kind = "method",
        summary = "Set the speed amplifier directly (0 = Speed I, 1 = Speed II, …).",
        context = "Set the speed amplifier directly (0 = Speed I, 1 = Speed II, …). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(amplifier = "`amplifier` provides the amplifier applied when setting the speed amplifier directly (0 = Speed I, 1 = Speed II, …)."),
        returns = "The `SpeedBoost` value with the documented change applied to set the speed amplifier directly (0 = Speed I, 1 = Speed II, …).",
        example = "use sand::prelude::*;\n\nfn demonstrate(speed_boost_value: sand::systems::movement::SpeedBoost, amplifier: u8)  {\n    let updated_speed_boost = speed_boost_value.amplifier(amplifier);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set the effect duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::duration",
        module = "sand::systems",
        kind = "method",
        summary = "Set the effect duration.",
        context = "Set the effect duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(duration = "`duration` provides the Minecraft tick duration used to set the effect duration."),
        returns = "The `SpeedBoost` value with the documented change applied to set the effect duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(speed_boost_value: sand::systems::movement::SpeedBoost, duration: sand::state::Ticks)  {\n    let updated_speed_boost = speed_boost_value.duration(duration);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = duration;
        self
    }

    /// Build the command string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::SpeedBoost::build",
        module = "sand::systems",
        kind = "method",
        summary = "Build the command string.",
        context = "Build the command string. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to build the command string.",
        example = "use sand::prelude::*;\n\nfn demonstrate(speed_boost_value: sand::systems::movement::SpeedBoost)  {\n    let command = speed_boost_value.build();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn build(self) -> String {
        let targets = self.targets.unwrap_or_else(|| "@s".to_string());
        format!(
            "effect give {} {} {} {}",
            targets,
            EffectId::Speed,
            self.duration.as_seconds(),
            self.amplifier
        )
    }
}

// ── Slow ──────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::movement::Slow",
    module = "sand::systems",
    summary = "Applies a slowness effect to one or more entities.",
    context = "Applies a slowness effect to one or more entities. The `amount` (0.0–1.0+) is converted to a vanilla amplifier: `amplifier = (amount / 0.15).round() as u8`. Slowness I (amplifier 0) ≈ −15% walk speed, Slowness II ≈ −30%, etc.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::movement::Slow;",
    availability = ["Cargo feature: systems-movement"],
)]
/// Applies a slowness effect to one or more entities.
///
/// The `amount` (0.0–1.0+) is converted to a vanilla amplifier:
/// `amplifier = (amount / 0.15).round() as u8`.
/// Slowness I (amplifier 0) ≈ −15% walk speed, Slowness II ≈ −30%, etc.
///
/// # Command emitted
/// ```text
/// effect give <targets> minecraft:slowness <duration_seconds> <amplifier>
/// ```
#[derive(Debug, Clone)]
pub struct Slow {
    targets: Option<String>,
    amplifier: u8,
    duration: Ticks,
}

impl Default for Slow {
    fn default() -> Self {
        Self::new()
    }
}

impl Slow {
    /// Create a new `Slow` builder with amplifier `0` (Slowness I) and 30 s duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new `Slow` builder with amplifier `0` (Slowness I) and 30 s duration.",
        context = "Create a new `Slow` builder with amplifier `0` (Slowness I) and 30 s duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "A `Slow` representing a new `Slow` builder with amplifier `0` (Slowness I) and 30 s duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let slow = sand::systems::movement::Slow::new();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn new() -> Self {
        Self {
            targets: None,
            amplifier: 0,
            duration: Ticks::seconds(30),
        }
    }

    /// Shorthand: create a builder for a single selector target.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::target",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand: create a builder for a single selector target.",
        context = "Shorthand: create a builder for a single selector target. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(target = "`target` provides the entity, block, or command target used to use shorthand: create a builder for a single selector target."),
        returns = "A `Slow` configured for shorthand: create a builder for a single selector target.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Target)  {\n    let slow = sand::systems::movement::Slow::target(target);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn target(target: impl TargetArgument) -> Self {
        Self::new().with_target(target)
    }

    /// Shorthand: create a builder for an entity-targets set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::targets",
        module = "sand::systems",
        kind = "method",
        summary = "Shorthand: create a builder for an entity-targets set.",
        context = "Shorthand: create a builder for an entity-targets set. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(targets = "`targets` provides the Minecraft target selection used to use shorthand: create a builder for an entity-targets set."),
        returns = "A `Slow` configured for shorthand: create a builder for an entity-targets set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(targets: sand::command::Target)  {\n    let slow = sand::systems::movement::Slow::targets(targets);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn targets(targets: impl TargetArgument) -> Self {
        let mut s = Self::new();
        s.targets = Some(targets.to_string());
        s
    }

    /// Set the target selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::with_target",
        module = "sand::systems",
        kind = "method",
        summary = "Set the target selector.",
        context = "Set the target selector. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(target = "`target` provides the entity, block, or command target used to set the target selector."),
        returns = "The `Slow` value with the documented change applied to set the target selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(slow_value: sand::systems::movement::Slow, target: sand::command::Target)  {\n    let updated_slow = slow_value.with_target(target);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn with_target(mut self, target: impl TargetArgument) -> Self {
        self.targets = Some(target.to_string());
        self
    }

    /// Set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5).
    ///
    /// Maps to `amplifier = (amount / 0.15).round().max(0) as u8`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::amount",
        module = "sand::systems",
        kind = "method",
        summary = "Set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5).",
        context = "Set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5). Maps to `amplifier = (amount / 0.15).round().max(0) as u8`.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(amount = "`amount` provides the requested numeric amount used to set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5)."),
        returns = "The `Slow` value with the documented change applied to set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5).",
        example = "use sand::prelude::*;\n\nfn demonstrate(slow_value: sand::systems::movement::Slow, amount: f64)  {\n    let updated_slow = slow_value.amount(amount);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn amount(mut self, amount: f64) -> Self {
        self.amplifier = ((amount / 0.15).round() as i32).max(0) as u8;
        self
    }

    /// Set the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::amplifier",
        module = "sand::systems",
        kind = "method",
        summary = "Set the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …).",
        context = "Set the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(amplifier = "`amplifier` provides the amplifier applied when setting the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …)."),
        returns = "The `Slow` value with the documented change applied to set the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …).",
        example = "use sand::prelude::*;\n\nfn demonstrate(slow_value: sand::systems::movement::Slow, amplifier: u8)  {\n    let updated_slow = slow_value.amplifier(amplifier);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set the effect duration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::duration",
        module = "sand::systems",
        kind = "method",
        summary = "Set the effect duration.",
        context = "Set the effect duration. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(duration = "`duration` provides the Minecraft tick duration used to set the effect duration."),
        returns = "The `Slow` value with the documented change applied to set the effect duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(slow_value: sand::systems::movement::Slow, duration: sand::state::Ticks)  {\n    let updated_slow = slow_value.duration(duration);\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = duration;
        self
    }

    /// Build the command string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::movement::Slow::build",
        module = "sand::systems",
        kind = "method",
        summary = "Build the command string.",
        context = "Build the command string. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The rendered Minecraft command text produced to build the command string.",
        example = "use sand::prelude::*;\n\nfn demonstrate(slow_value: sand::systems::movement::Slow)  {\n    let command = slow_value.build();\n}",
        availability = ["Cargo feature: systems-movement"],
    )]
    pub fn build(self) -> String {
        let targets = self.targets.unwrap_or_else(|| "@s".to_string());
        format!(
            "effect give {} {} {} {}",
            targets,
            EffectId::Slowness,
            self.duration.as_seconds(),
            self.amplifier
        )
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_local_coord(v: f64) -> String {
    if v == 0.0 {
        "^".to_string()
    } else if v == v.trunc() {
        format!("^{}", v as i64)
    } else {
        format!("^{v}")
    }
}

fn fmt_rel_coord(v: f64) -> String {
    if v == 0.0 {
        "~".to_string()
    } else if v == v.trunc() {
        format!("~{}", v as i64)
    } else {
        format!("~{v}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn push_away_defaults() {
        let cmds = PushAway::new()
            .source(Selector::self_())
            .targets(Target::nearby(6.0).excluding_players())
            .build();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].starts_with("execute as "), "cmd: {}", cmds[0]);
        assert!(
            cmds[0].contains("facing entity @s feet"),
            "cmd: {}",
            cmds[0]
        );
        assert!(cmds[0].contains("run tp @s ^0 ^ ^-1"), "cmd: {}", cmds[0]);
    }

    #[test]
    fn push_away_with_strength_and_lift() {
        let cmds = PushAway::new()
            .source(Selector::self_())
            .targets(Target::nearby(6.0).excluding_players())
            .strength(1.5)
            .lift(0.25)
            .build();
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("run tp @s ^0 ^0.25 ^-1.5"),
            "cmd: {}",
            cmds[0]
        );
    }

    #[test]
    fn push_away_integer_lift() {
        let cmds = PushAway::new()
            .source(Selector::self_())
            .targets(Target::nearby(4.0))
            .strength(2.0)
            .lift(1.0)
            .build();
        assert!(cmds[0].contains("run tp @s ^0 ^1 ^-2"), "cmd: {}", cmds[0]);
    }

    #[test]
    fn launch_defaults() {
        let cmds = Launch::new().build();
        assert_eq!(cmds, vec!["execute as @s run tp @s ~ ~0.5 ~"]);
    }

    #[test]
    fn launch_with_targets_and_amount() {
        let cmds = Launch::targets(Target::nearby(4.0)).amount(0.7).build();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("~ ~0.7 ~"), "cmd: {}", cmds[0]);
    }

    #[test]
    fn launch_integer_amount() {
        let cmds = Launch::new().amount(2.0).build();
        assert_eq!(cmds, vec!["execute as @s run tp @s ~ ~2 ~"]);
    }

    #[test]
    fn speed_boost_default() {
        let cmd = SpeedBoost::target(Selector::self_())
            .duration(Ticks::seconds(5))
            .build();
        assert_eq!(cmd, "effect give @s minecraft:speed 5 0");
    }

    #[test]
    fn speed_boost_amount_maps_to_amplifier() {
        let cmd = SpeedBoost::target(Selector::self_())
            .amount(0.4)
            .duration(Ticks::seconds(10))
            .build();
        assert_eq!(cmd, "effect give @s minecraft:speed 10 2");
    }

    #[test]
    fn speed_boost_explicit_amplifier() {
        let cmd = SpeedBoost::target(Selector::self_())
            .amplifier(3)
            .duration(Ticks::seconds(20))
            .build();
        assert_eq!(cmd, "effect give @s minecraft:speed 20 3");
    }

    #[test]
    fn slow_default() {
        let cmd = Slow::target(Selector::self_())
            .duration(Ticks::seconds(5))
            .build();
        assert_eq!(cmd, "effect give @s minecraft:slowness 5 0");
    }

    #[test]
    fn slow_amount_maps_to_amplifier() {
        let cmd = Slow::target(Selector::self_())
            .amount(0.3)
            .duration(Ticks::seconds(10))
            .build();
        assert_eq!(cmd, "effect give @s minecraft:slowness 10 2");
    }

    #[test]
    fn slow_targets_many() {
        let cmd = Slow::targets(Target::nearby(5.0))
            .amount(0.4)
            .duration(Ticks::seconds(3))
            .build();
        assert!(cmd.starts_with("effect give "), "cmd: {cmd}");
        assert!(cmd.contains("minecraft:slowness"), "cmd: {cmd}");
        assert!(cmd.ends_with(" 3 3"), "cmd: {cmd}");
    }

    #[test]
    fn fmt_local_coord_zero() {
        assert_eq!(fmt_local_coord(0.0), "^");
    }

    #[test]
    fn fmt_local_coord_int() {
        assert_eq!(fmt_local_coord(2.0), "^2");
        assert_eq!(fmt_local_coord(-1.0), "^-1");
    }

    #[test]
    fn fmt_local_coord_float() {
        assert_eq!(fmt_local_coord(0.5), "^0.5");
        assert_eq!(fmt_local_coord(-1.5), "^-1.5");
    }

    #[test]
    fn fmt_rel_coord_zero() {
        assert_eq!(fmt_rel_coord(0.0), "~");
    }

    #[test]
    fn fmt_rel_coord_int() {
        assert_eq!(fmt_rel_coord(1.0), "~1");
    }

    #[test]
    fn fmt_rel_coord_float() {
        assert_eq!(fmt_rel_coord(0.7), "~0.7");
    }
}
