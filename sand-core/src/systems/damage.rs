//! Damage tracking system (`systems-damage` feature).
//!
//! # Vanilla limitation
//!
//! Pure vanilla datapacks **cannot know the exact damage amount** from a single
//! damage event. Advancement triggers fire *after* damage is applied, but their
//! JSON criteria do not expose the numeric amount. The `DamageAdvancementEvent`
//! pattern in Sand's typed events lets you react *when* damage happens; this
//! module adds the best available approximation of *how much* damage occurred.
//!
//! # How it works
//!
//! Minecraft tracks cumulative damage taken in the scoreboard stat
//! `minecraft.custom:minecraft.damage_taken` (measured in half-hearts × 10,
//! so 10 = 1 heart). By comparing the stat value between ticks, we can detect
//! that a player was hurt and estimate the amount within a tick window.
//!
//! This is not exact: multiple damage sources within the same tick collapse into
//! one delta, and invincibility frames mean some hits are ignored by the game.
//!
//! # Provided types
//!
//! - [`DamageTracker`] — tracks the delta of `damage_taken` per tick per player.
//! - [`recently_damaged`] — condition: player's damage delta > 0 this tick.

use crate::condition::{Condition, ScoreRange};
use crate::state::ScoreVar;

// ── Objective names ────────────────────────────────────────────────────────────

/// Objective name for the raw cumulative `damage_taken` stat.
pub const DAMAGE_STAT_OBJ: &str = "sd_dmg_stat";
/// Objective name for the previous-tick snapshot.
pub const DAMAGE_PREV_OBJ: &str = "sd_dmg_prev";
/// Objective name for the per-tick delta (current − previous).
pub const DAMAGE_DELTA_OBJ: &str = "sd_dmg_delta";

// ── DamageTracker ─────────────────────────────────────────────────────────────

/// A damage tracker that measures per-tick damage taken via cumulative scoreboard stats.
///
/// # Setup
///
/// Call `define()` in your load function and `tick("@a")` every game tick.
/// The tracker maintains three internal objectives:
/// - `sd_dmg_stat` — mirrors `minecraft.custom:minecraft.damage_taken`
/// - `sd_dmg_prev` — previous-tick snapshot
/// - `sd_dmg_delta` — `stat - prev` (damage taken this tick, in half-hearts × 10)
///
/// # Limitations
///
/// - Values are in half-hearts × 10 (e.g. 5 = half a heart, 10 = one heart).
/// - Multiple hits within the same tick are summed.
/// - Invincibility frames cause some hits to register as 0 delta.
/// - Cannot distinguish attacker or damage type (use [`DamageAdvancementEvent`] for that).
///
/// [`DamageAdvancementEvent`]: crate::event::DamageAdvancementEvent
pub struct DamageTracker;

impl DamageTracker {
    /// Define the three required scoreboard objectives.
    ///
    /// Returns three commands suitable for a load function.
    pub fn define() -> Vec<String> {
        vec![
            format!(
                "scoreboard objectives add {DAMAGE_STAT_OBJ} minecraft.custom:minecraft.damage_taken"
            ),
            format!("scoreboard objectives add {DAMAGE_PREV_OBJ} dummy"),
            format!("scoreboard objectives add {DAMAGE_DELTA_OBJ} dummy"),
        ]
    }

    /// Update damage deltas for `selector` (run every tick).
    ///
    /// Returns commands that compute `delta = stat - prev` and then advance `prev`.
    ///
    /// Generated commands:
    /// ```text
    /// scoreboard players operation @a sd_dmg_delta = @a sd_dmg_stat
    /// scoreboard players operation @a sd_dmg_delta -= @a sd_dmg_prev
    /// scoreboard players operation @a sd_dmg_prev = @a sd_dmg_stat
    /// ```
    pub fn tick(selector: impl std::fmt::Display) -> Vec<String> {
        let sel = selector.to_string();
        vec![
            format!(
                "scoreboard players operation {sel} {DAMAGE_DELTA_OBJ} = {sel} {DAMAGE_STAT_OBJ}"
            ),
            format!(
                "scoreboard players operation {sel} {DAMAGE_DELTA_OBJ} -= {sel} {DAMAGE_PREV_OBJ}"
            ),
            format!(
                "scoreboard players operation {sel} {DAMAGE_PREV_OBJ} = {sel} {DAMAGE_STAT_OBJ}"
            ),
        ]
    }

    /// Condition: `selector` was damaged this tick (delta > 0).
    pub fn recently_damaged(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Gte(1),
        }
    }

    /// Condition: `selector` took at least `min_half_hearts_x10` damage this tick.
    ///
    /// The unit is half-hearts × 10 (same as `minecraft.damage_taken`):
    /// - 1 = 0.05 hearts
    /// - 10 = 1 heart
    /// - 20 = 2 hearts (one hit from a zombie on normal)
    pub fn damaged_at_least(selector: &str, min_half_hearts_x10: i32) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Gte(min_half_hearts_x10),
        }
    }

    /// The delta objective name, for use with [`ScoreVar`](crate::state::ScoreVar) conditions.
    pub fn delta_objective() -> String {
        let var: ScoreVar<i32> = ScoreVar::new(DAMAGE_DELTA_OBJ);
        var.objective_name()
    }
}

// ── recently_damaged free function ────────────────────────────────────────────

/// Condition shorthand: player at `selector` took damage this tick.
///
/// Requires the `systems-damage` feature and that [`DamageTracker::tick`] runs
/// every game tick for the relevant selector.
pub fn recently_damaged(selector: &str) -> Condition {
    DamageTracker::recently_damaged(selector)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    #[test]
    fn define_produces_three_objectives() {
        let cmds = DamageTracker::define();
        assert_eq!(cmds.len(), 3);
        assert!(cmds[0].contains(DAMAGE_STAT_OBJ), "stat obj: {}", cmds[0]);
        assert!(cmds[1].contains(DAMAGE_PREV_OBJ), "prev obj: {}", cmds[1]);
        assert!(cmds[2].contains(DAMAGE_DELTA_OBJ), "delta obj: {}", cmds[2]);
        // Stat objective must use the minecraft stat criterion
        assert!(
            cmds[0].contains("minecraft.custom:minecraft.damage_taken"),
            "stat cmd: {}",
            cmds[0]
        );
    }

    #[test]
    fn tick_produces_three_operations() {
        let cmds = DamageTracker::tick("@a");
        assert_eq!(cmds.len(), 3);
        // delta = stat
        assert!(
            cmds[0].contains(&format!("{DAMAGE_DELTA_OBJ} = @a {DAMAGE_STAT_OBJ}")),
            "got: {}",
            cmds[0]
        );
        // delta -= prev
        assert!(
            cmds[1].contains(&format!("{DAMAGE_DELTA_OBJ} -= @a {DAMAGE_PREV_OBJ}")),
            "got: {}",
            cmds[1]
        );
        // prev = stat
        assert!(
            cmds[2].contains(&format!("{DAMAGE_PREV_OBJ} = @a {DAMAGE_STAT_OBJ}")),
            "got: {}",
            cmds[2]
        );
    }

    #[test]
    fn recently_damaged_condition() {
        let cond = DamageTracker::recently_damaged("@s");
        match cond {
            Condition::Score {
                selector,
                objective,
                range: ScoreRange::Gte(1),
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, DAMAGE_DELTA_OBJ);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn damaged_at_least_condition() {
        let cond = DamageTracker::damaged_at_least("@s", 20);
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(20),
                ..
            }
        ));
    }

    #[test]
    fn free_fn_recently_damaged() {
        let cond = recently_damaged("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
    }
}
