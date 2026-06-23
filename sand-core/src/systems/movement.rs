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
//! use sand_core::cmd::Selector;
//! use sand_core::state::Ticks;
//! use sand_commands::selector::EntityTargets;
//!
//! // Shockwave push: push all nearby non-player entities away from @s
//! let cmds = PushAway::new()
//!     .source(Selector::self_())
//!     .targets(EntityTargets::nearby(6.0).excluding_players())
//!     .strength(1.5)
//!     .lift(0.25)
//!     .build();
//!
//! // Launch all nearby entities upward
//! let cmds = Launch::targets(EntityTargets::nearby(4.0))
//!     .amount(0.7)
//!     .build();
//!
//! // Speed boost self for 5 seconds
//! let cmd = SpeedBoost::target(Selector::self_())
//!     .amount(0.4)
//!     .duration(Ticks::seconds(5))
//!     .build();
//!
//! // Slow nearby entities for 3 seconds
//! let cmd = Slow::targets(EntityTargets::nearby(5.0))
//!     .amount(0.3)
//!     .duration(Ticks::seconds(3))
//!     .build();
//! ```

use sand_commands::selector::{EntityTargets, Selector};
use sand_components::{EffectId, Ticks};

// ── PushAway ──────────────────────────────────────────────────────────────────

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
    targets: Option<EntityTargets>,
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
    pub fn new() -> Self {
        Self {
            source: None,
            targets: None,
            strength: 1.0,
            lift: 0.0,
        }
    }

    /// Set the source entity (the "center" of the push, typically `@s`).
    pub fn source(mut self, source: Selector) -> Self {
        self.source = Some(source);
        self
    }

    /// Set the entities to push.
    pub fn targets(mut self, targets: EntityTargets) -> Self {
        self.targets = Some(targets);
        self
    }

    /// How far to push each target (in blocks along the away vector).
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }

    /// How far to lift each target upward (in blocks, default `0.0`).
    pub fn lift(mut self, lift: f64) -> Self {
        self.lift = lift;
        self
    }

    /// Build the command string(s).
    ///
    /// Returns one command per source–targets pair. If source or targets are
    /// not set, defaults to `@s` (source) or all entities (targets).
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
    targets: Option<EntityTargets>,
    amount: f64,
}

impl Default for Launch {
    fn default() -> Self {
        Self::new()
    }
}

impl Launch {
    /// Create a new `Launch` builder with default amount `0.5`.
    pub fn new() -> Self {
        Self {
            targets: None,
            amount: 0.5,
        }
    }

    /// Shorthand: create a builder with targets already set.
    pub fn targets(targets: EntityTargets) -> Self {
        Self::new().with_targets(targets)
    }

    /// Set the entities to launch.
    pub fn with_targets(mut self, targets: EntityTargets) -> Self {
        self.targets = Some(targets);
        self
    }

    /// How far to launch upward (in blocks).
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    /// Build the command string(s).
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
    pub fn new() -> Self {
        Self {
            targets: None,
            amplifier: 0,
            duration: Ticks::seconds(30),
        }
    }

    /// Shorthand: create a builder for a single selector target.
    pub fn target(target: Selector) -> Self {
        Self::new().with_target(target)
    }

    /// Shorthand: create a builder for an entity-targets set.
    pub fn target_many(targets: EntityTargets) -> Self {
        let mut s = Self::new();
        s.targets = Some(targets.to_string());
        s
    }

    /// Set the target selector.
    pub fn with_target(mut self, target: Selector) -> Self {
        self.targets = Some(target.to_string());
        self
    }

    /// Set speed amount as a fraction where `1.0` ≈ Speed V (100% extra, amplifier 4).
    ///
    /// Maps to `amplifier = (amount / 0.2).round().max(0) as u8`.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amplifier = ((amount / 0.2).round() as i32).max(0) as u8;
        self
    }

    /// Set the speed amplifier directly (0 = Speed I, 1 = Speed II, …).
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set the effect duration.
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = duration;
        self
    }

    /// Build the command string.
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
    pub fn new() -> Self {
        Self {
            targets: None,
            amplifier: 0,
            duration: Ticks::seconds(30),
        }
    }

    /// Shorthand: create a builder for a single selector target.
    pub fn target(target: Selector) -> Self {
        Self::new().with_target(target)
    }

    /// Shorthand: create a builder for an entity-targets set.
    pub fn targets(targets: EntityTargets) -> Self {
        let mut s = Self::new();
        s.targets = Some(targets.to_string());
        s
    }

    /// Set the target selector.
    pub fn with_target(mut self, target: Selector) -> Self {
        self.targets = Some(target.to_string());
        self
    }

    /// Set slow amount as a fraction where `1.0` ≈ Slowness VI (~90% reduction, amplifier 5).
    ///
    /// Maps to `amplifier = (amount / 0.15).round().max(0) as u8`.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amplifier = ((amount / 0.15).round() as i32).max(0) as u8;
        self
    }

    /// Set the slowness amplifier directly (0 = Slowness I, 1 = Slowness II, …).
    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    /// Set the effect duration.
    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = duration;
        self
    }

    /// Build the command string.
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
    use sand_commands::selector::EntityTargets;

    #[test]
    fn push_away_defaults() {
        let cmds = PushAway::new()
            .source(Selector::self_())
            .targets(EntityTargets::nearby(6.0).excluding_players())
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
            .targets(EntityTargets::nearby(6.0).excluding_players())
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
            .targets(EntityTargets::nearby(4.0))
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
        let cmds = Launch::targets(EntityTargets::nearby(4.0))
            .amount(0.7)
            .build();
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
        let cmd = Slow::targets(EntityTargets::nearby(5.0))
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
