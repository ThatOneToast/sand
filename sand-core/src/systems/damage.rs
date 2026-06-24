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
//! `minecraft.custom:minecraft.damage_taken` (units: 1 stat = 0.1 hearts,
//! so 10 = 1 heart). By comparing the stat value between ticks we detect
//! that a player was hurt and approximate the amount.
//!
//! # Accuracy limitations
//!
//! - Multiple hits within the same tick are summed into one delta.
//! - Invincibility frames cause some hits to register as 0 delta.
//! - Damage cause/type/attacker cannot be tracked here — use
//!   `DamageAdvancementEvent` for source-aware events.
//!
//! # Units
//!
//! Sand user-facing APIs use **hearts** (1 heart = 2 HP). Internal scoreboard
//! values use the Minecraft stat unit (1 stat = 0.1 hearts). Use
//! [`DamageThreshold::hearts`] and [`DamageThreshold::raw_stat`] to convert.
//!
//! # Setup
//!
//! ```rust,ignore
//! #[component(Load)]
//! fn load() {
//!     DamageTracker::define();
//! }
//!
//! #[component(Tick)]
//! fn tick() {
//!     DamageTracker::tick_players();
//! }
//! ```

use crate::condition::{Condition, ScoreRange};
use crate::state::{ScoreVar, Ticks};

// ── Objective names ────────────────────────────────────────────────────────────

/// Objective: cumulative `damage_taken` vanilla stat (mirrors Minecraft's value).
pub const DAMAGE_STAT_OBJ: &str = "sd_dmg_stat";
/// Objective: previous-tick stat snapshot.
pub const DAMAGE_PREV_OBJ: &str = "sd_dmg_prev";
/// Objective: per-tick damage delta (`stat - prev`); 0 when not hurt this tick.
pub const DAMAGE_DELTA_OBJ: &str = "sd_dmg_delta";
/// Objective: last non-zero damage delta (persists until next damage event).
pub const DAMAGE_LAST_OBJ: &str = "sd_dmg_last";
/// Objective: ticks since last damage; `0` on the tick damage is taken.
pub const DAMAGE_HURT_AGE_OBJ: &str = "sd_dmg_hurt";

// ── DamageThreshold ───────────────────────────────────────────────────────────

/// A damage amount threshold for querying [`DamageTracker`] conditions.
///
/// # Units
///
/// Prefer [`DamageThreshold::hearts`] for user-facing values:
/// - `1.0` heart = one full heart = 10 internal stat units
/// - `0.5` hearts = half a heart = 5 stat units
///
/// Use [`DamageThreshold::raw_stat`] only when you need to match the raw
/// Minecraft scoreboard stat value directly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageThreshold {
    /// Number of hearts (1.0 = 1 heart = 10 stat units).
    Hearts(f32),
    /// Raw Minecraft stat units (same scale as `minecraft.damage_taken`).
    RawStat(i32),
}

impl DamageThreshold {
    /// Threshold in hearts (1.0 = one heart, 0.5 = half a heart).
    pub fn hearts(h: f32) -> Self {
        Self::Hearts(h)
    }

    /// Raw scoreboard stat units — advanced use only.
    pub fn raw_stat(v: i32) -> Self {
        Self::RawStat(v)
    }

    /// Convert to the raw Minecraft scoreboard stat integer used internally.
    pub fn to_raw_stat(self) -> i32 {
        match self {
            Self::Hearts(h) => (h * 10.0).round() as i32,
            Self::RawStat(v) => v,
        }
    }
}

// ── DamageTracker ─────────────────────────────────────────────────────────────

/// Tracks per-tick damage state for players via cumulative scoreboard stats.
///
/// Maintains five objectives:
/// - `sd_dmg_stat` — mirrors `minecraft.custom:minecraft.damage_taken`
/// - `sd_dmg_prev` — previous-tick snapshot
/// - `sd_dmg_delta` — damage this tick (0 when not hurt)
/// - `sd_dmg_last` — last non-zero delta (persists between hurt events)
/// - `sd_dmg_hurt` — ticks since last damage; `0` on the hurt tick
pub struct DamageTracker;

impl DamageTracker {
    /// Define all five required scoreboard objectives.
    ///
    /// Call once in a `#[component(Load)]` function.
    pub fn define() -> Vec<String> {
        vec![
            format!(
                "scoreboard objectives add {DAMAGE_STAT_OBJ} \
                 minecraft.custom:minecraft.damage_taken"
            ),
            format!("scoreboard objectives add {DAMAGE_PREV_OBJ} dummy"),
            format!("scoreboard objectives add {DAMAGE_DELTA_OBJ} dummy"),
            format!("scoreboard objectives add {DAMAGE_LAST_OBJ} dummy"),
            format!("scoreboard objectives add {DAMAGE_HURT_AGE_OBJ} dummy"),
        ]
    }

    /// Update all damage tracking state for `selector` (call every tick).
    ///
    /// Algorithm (in order):
    /// 1. `delta = stat`
    /// 2. `delta -= prev`
    /// 3. If `delta > 0`: `last = delta`
    /// 4. If `delta > 0`: `hurt_age = 0`
    /// 5. Unless `delta > 0`: `hurt_age += 1`
    /// 6. `prev = stat`
    pub fn tick(selector: impl std::fmt::Display) -> Vec<String> {
        let sel = selector.to_string();
        vec![
            // 1+2: delta = stat - prev
            format!(
                "scoreboard players operation {sel} {DAMAGE_DELTA_OBJ} = {sel} {DAMAGE_STAT_OBJ}"
            ),
            format!(
                "scoreboard players operation {sel} {DAMAGE_DELTA_OBJ} -= {sel} {DAMAGE_PREV_OBJ}"
            ),
            // 3: if delta > 0: last = delta
            format!(
                "execute as {sel}[scores={{{DAMAGE_DELTA_OBJ}=1..}}] \
                 run scoreboard players operation @s {DAMAGE_LAST_OBJ} = @s {DAMAGE_DELTA_OBJ}"
            ),
            // 4: if delta > 0: hurt_age = 0
            format!(
                "execute as {sel}[scores={{{DAMAGE_DELTA_OBJ}=1..}}] \
                 run scoreboard players set @s {DAMAGE_HURT_AGE_OBJ} 0"
            ),
            // 5: unless delta > 0: hurt_age += 1
            format!(
                "execute as {sel}[scores={{{DAMAGE_DELTA_OBJ}=..0}}] \
                 run scoreboard players add @s {DAMAGE_HURT_AGE_OBJ} 1"
            ),
            // 6: prev = stat
            format!(
                "scoreboard players operation {sel} {DAMAGE_PREV_OBJ} = {sel} {DAMAGE_STAT_OBJ}"
            ),
        ]
    }

    /// Shorthand: `tick("@a")` — tick all online players.
    pub fn tick_players() -> Vec<String> {
        Self::tick("@a")
    }

    // ── Conditions ────────────────────────────────────────────────────────────

    /// Condition: `selector` was damaged this tick (delta > 0).
    pub fn damaged_this_tick(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Gte(1),
        }
    }

    /// Condition: `selector` was NOT damaged this tick (delta == 0).
    pub fn not_damaged_this_tick(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Eq(0),
        }
    }

    /// Condition: `selector` took at least `threshold` damage this tick.
    pub fn current_damage_at_least(selector: &str, threshold: DamageThreshold) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Gte(threshold.to_raw_stat()),
        }
    }

    /// Condition: the last recorded damage for `selector` was at least `threshold`.
    ///
    /// Uses `sd_dmg_last`, which persists between damage events.
    pub fn last_damage_at_least(selector: &str, threshold: DamageThreshold) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_LAST_OBJ.to_string(),
            range: ScoreRange::Gte(threshold.to_raw_stat()),
        }
    }

    /// Condition: `selector` was last hurt within `ticks` ticks ago.
    pub fn hurt_within(selector: &str, ticks: Ticks) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_HURT_AGE_OBJ.to_string(),
            range: ScoreRange::Lte(ticks.get() as i32),
        }
    }

    // ── Raw score accessors (advanced use) ────────────────────────────────────

    /// The raw current-tick delta objective name.
    pub fn current_damage_raw(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_DELTA_OBJ.to_string(),
            range: ScoreRange::Gte(1),
        }
    }

    /// The raw last-damage objective name.
    pub fn last_damage_raw(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_LAST_OBJ.to_string(),
            range: ScoreRange::Gte(1),
        }
    }

    /// The ticks-since-hurt objective name (for use with ScoreVar).
    pub fn ticks_since_hurt(selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_HURT_AGE_OBJ.to_string(),
            range: ScoreRange::Gte(0),
        }
    }

    // ── Additional helpers (no cause inference) ───────────────────────────────

    /// Condition: `selector` was hurt this tick (same as `damaged_this_tick`).
    ///
    /// Convenient alias for common event-gating patterns:
    /// ```rust,ignore
    /// if DamageTracker::was_hurt("@s") { ... }
    /// ```
    ///
    /// Does **not** tell you the cause, attacker, damage type, or weapon.
    /// Use advancement predicate events for cause-specific logic.
    pub fn was_hurt(selector: &str) -> Condition {
        Self::damaged_this_tick(selector)
    }

    /// Condition: `selector` has not been hurt for at least `ticks` ticks.
    ///
    /// This is the complement of [`hurt_within`](DamageTracker::hurt_within):
    /// - `hurt_within(n)` → age ≤ n → hurt recently
    /// - `not_hurt_for(n)` → age > n → safe for at least n ticks
    ///
    /// Useful for ability cooldown windows that reset on damage.
    pub fn not_hurt_for(selector: &str, ticks: Ticks) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: DAMAGE_HURT_AGE_OBJ.to_string(),
            range: ScoreRange::Gte(ticks.get() as i32 + 1),
        }
    }

    /// Reset the last-recorded damage delta for `selector` to 0.
    ///
    /// Useful after consuming a damage event so stale deltas don't re-fire
    /// condition checks on the next tick.
    ///
    /// Returns a single scoreboard `set ... 0` command.
    pub fn clear_recent_damage(selector: impl std::fmt::Display) -> String {
        format!("scoreboard players set {} {DAMAGE_LAST_OBJ} 0", selector)
    }

    // ── Deprecated compatibility shims ────────────────────────────────────────

    /// Condition: `selector` was damaged this tick (delta > 0).
    ///
    /// Deprecated: use [`damaged_this_tick`](DamageTracker::damaged_this_tick).
    #[deprecated(note = "use DamageTracker::damaged_this_tick")]
    pub fn recently_damaged(selector: &str) -> Condition {
        Self::damaged_this_tick(selector)
    }

    /// Condition: `selector` took at least `min_raw` damage this tick.
    ///
    /// Deprecated: use [`current_damage_at_least`](DamageTracker::current_damage_at_least)
    /// with [`DamageThreshold`].
    #[deprecated(note = "use DamageTracker::current_damage_at_least(DamageThreshold::raw_stat(v))")]
    pub fn damaged_at_least(selector: &str, min_raw: i32) -> Condition {
        Self::current_damage_at_least(selector, DamageThreshold::raw_stat(min_raw))
    }

    /// The delta objective name.
    ///
    /// Deprecated: use objective constants directly.
    #[deprecated(note = "use DAMAGE_DELTA_OBJ constant")]
    pub fn delta_objective() -> String {
        let var: ScoreVar<i32> = ScoreVar::new(DAMAGE_DELTA_OBJ);
        var.objective_name()
    }
}

// ── Free function shims ───────────────────────────────────────────────────────

/// Condition shorthand: player at `selector` took damage this tick.
///
/// Requires `DamageTracker::tick()` to run every game tick.
pub fn recently_damaged(selector: &str) -> Condition {
    DamageTracker::damaged_this_tick(selector)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    #[test]
    fn define_produces_five_objectives() {
        let cmds = DamageTracker::define();
        assert_eq!(cmds.len(), 5);
        assert!(cmds[0].contains(DAMAGE_STAT_OBJ), "stat obj: {}", cmds[0]);
        assert!(cmds[1].contains(DAMAGE_PREV_OBJ), "prev obj: {}", cmds[1]);
        assert!(cmds[2].contains(DAMAGE_DELTA_OBJ), "delta obj: {}", cmds[2]);
        assert!(cmds[3].contains(DAMAGE_LAST_OBJ), "last obj: {}", cmds[3]);
        assert!(
            cmds[4].contains(DAMAGE_HURT_AGE_OBJ),
            "age obj: {}",
            cmds[4]
        );
        assert!(
            cmds[0].contains("minecraft.custom:minecraft.damage_taken"),
            "stat criterion: {}",
            cmds[0]
        );
    }

    #[test]
    fn tick_produces_six_commands_in_correct_order() {
        let cmds = DamageTracker::tick("@a");
        assert_eq!(cmds.len(), 6, "expected 6 tick commands: {cmds:?}");

        // 1: delta = stat
        assert!(
            cmds[0].contains(&format!("{DAMAGE_DELTA_OBJ} = @a {DAMAGE_STAT_OBJ}")),
            "step 1 delta=stat: {}",
            cmds[0]
        );
        // 2: delta -= prev
        assert!(
            cmds[1].contains(&format!("{DAMAGE_DELTA_OBJ} -= @a {DAMAGE_PREV_OBJ}")),
            "step 2 delta-=prev: {}",
            cmds[1]
        );
        // 3: if delta > 0: last = delta
        assert!(
            cmds[2].contains(&format!("{DAMAGE_DELTA_OBJ}=1.."))
                && cmds[2].contains(&format!("{DAMAGE_LAST_OBJ} = @s {DAMAGE_DELTA_OBJ}")),
            "step 3 last=delta: {}",
            cmds[2]
        );
        // 4: if delta > 0: hurt_age = 0
        assert!(
            cmds[3].contains(&format!("{DAMAGE_DELTA_OBJ}=1.."))
                && cmds[3].contains(DAMAGE_HURT_AGE_OBJ)
                && cmds[3].contains("set @s")
                && cmds[3].contains(" 0"),
            "step 4 hurt_age=0: {}",
            cmds[3]
        );
        // 5: unless delta > 0: hurt_age += 1
        assert!(
            cmds[4].contains(&format!("{DAMAGE_DELTA_OBJ}=..0"))
                && cmds[4].contains(DAMAGE_HURT_AGE_OBJ)
                && cmds[4].contains("add @s"),
            "step 5 hurt_age+=1: {}",
            cmds[4]
        );
        // 6: prev = stat (MUST be last)
        assert!(
            cmds[5].contains(&format!("{DAMAGE_PREV_OBJ} = @a {DAMAGE_STAT_OBJ}")),
            "step 6 prev=stat: {}",
            cmds[5]
        );
    }

    #[test]
    fn tick_players_is_tick_at_a() {
        assert_eq!(DamageTracker::tick_players(), DamageTracker::tick("@a"));
    }

    // ── DamageThreshold unit conversion ──────────────────────────────────────

    #[test]
    fn threshold_hearts_one_heart() {
        assert_eq!(DamageThreshold::hearts(1.0).to_raw_stat(), 10);
    }

    #[test]
    fn threshold_hearts_half_heart() {
        assert_eq!(DamageThreshold::hearts(0.5).to_raw_stat(), 5);
    }

    #[test]
    fn threshold_hearts_two_hearts() {
        assert_eq!(DamageThreshold::hearts(2.0).to_raw_stat(), 20);
    }

    #[test]
    fn threshold_raw_stat_passthrough() {
        assert_eq!(DamageThreshold::raw_stat(42).to_raw_stat(), 42);
    }

    // ── Conditions ────────────────────────────────────────────────────────────

    #[test]
    fn damaged_this_tick_condition() {
        let cond = DamageTracker::damaged_this_tick("@s");
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
    fn not_damaged_this_tick_condition() {
        let cond = DamageTracker::not_damaged_this_tick("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn current_damage_at_least_hearts() {
        let cond = DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0));
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(10),
                ..
            }
        ));
    }

    #[test]
    fn last_damage_at_least_half_heart() {
        let cond = DamageTracker::last_damage_at_least("@s", DamageThreshold::hearts(0.5));
        match cond {
            Condition::Score {
                objective,
                range: ScoreRange::Gte(5),
                ..
            } => assert_eq!(objective, DAMAGE_LAST_OBJ),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn hurt_within_ticks() {
        let cond = DamageTracker::hurt_within("@s", Ticks::new(20));
        match cond {
            Condition::Score {
                objective,
                range: ScoreRange::Lte(20),
                ..
            } => assert_eq!(objective, DAMAGE_HURT_AGE_OBJ),
            other => panic!("unexpected: {other:?}"),
        }
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

    // ── Deprecated API still compiles and works ───────────────────────────────

    #[test]
    #[allow(deprecated)]
    fn deprecated_recently_damaged_still_works() {
        let cond = DamageTracker::recently_damaged("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_damaged_at_least_still_works() {
        let cond = DamageTracker::damaged_at_least("@s", 20);
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(20),
                ..
            }
        ));
    }

    // ── New helpers: was_hurt, not_hurt_for, clear_recent_damage ─────────────

    #[test]
    fn was_hurt_is_alias_for_damaged_this_tick() {
        let a = DamageTracker::was_hurt("@s");
        let b = DamageTracker::damaged_this_tick("@s");
        // Both must be Gte(1) on the delta objective
        assert!(matches!(
            a,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
        assert!(matches!(
            b,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
        // Same objective
        if let (Condition::Score { objective: oa, .. }, Condition::Score { objective: ob, .. }) =
            (a, b)
        {
            assert_eq!(oa, ob);
        }
    }

    #[test]
    fn not_hurt_for_uses_age_gte_n_plus_one() {
        let cond = DamageTracker::not_hurt_for("@s", Ticks::new(20));
        match cond {
            Condition::Score {
                ref objective,
                range: ScoreRange::Gte(21),
                ..
            } => assert_eq!(objective, DAMAGE_HURT_AGE_OBJ),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn not_hurt_for_zero_ticks() {
        // not_hurt_for(0) → age >= 1, i.e. "not hurt this tick"
        let cond = DamageTracker::not_hurt_for("@s", Ticks::new(0));
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
    }

    #[test]
    fn was_hurt_and_not_hurt_for_are_complementary() {
        // was_hurt → delta >= 1
        // not_hurt_for(0) → age >= 1 (not hurt this tick)
        // They use different objectives so they are not direct complements,
        // but both should produce Gte conditions.
        let hurt = DamageTracker::was_hurt("@s");
        let safe = DamageTracker::not_hurt_for("@s", Ticks::new(10));
        assert!(matches!(
            hurt,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
        assert!(matches!(
            safe,
            Condition::Score {
                range: ScoreRange::Gte(11),
                ..
            }
        ));
    }

    #[test]
    fn clear_recent_damage_golden_command() {
        let cmd = DamageTracker::clear_recent_damage("@s");
        assert_eq!(
            cmd,
            format!("scoreboard players set @s {DAMAGE_LAST_OBJ} 0")
        );
    }

    #[test]
    fn clear_recent_damage_all_players() {
        let cmd = DamageTracker::clear_recent_damage("@a");
        assert!(cmd.contains(DAMAGE_LAST_OBJ));
        assert!(cmd.contains("set @a"));
        assert!(cmd.ends_with(" 0"));
    }

    #[test]
    fn clear_recent_damage_does_not_infer_cause() {
        // The command must only touch the 'last delta' scoreboard — not any
        // cause-specific score or storage key.
        let cmd = DamageTracker::clear_recent_damage("@s");
        assert!(!cmd.contains("attacker"), "must not mention attacker");
        assert!(!cmd.contains("source"), "must not mention damage source");
        assert!(!cmd.contains("weapon"), "must not mention weapon");
        assert!(!cmd.contains("type"), "must not mention damage type");
    }
}
