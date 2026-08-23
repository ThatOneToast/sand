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
//! #[datapack_component(Load)]
//! fn load() {
//!     DamageTracker::define();
//! }
//!
//! #[datapack_component(Tick)]
//! fn tick() {
//!     DamageTracker::tick_players();
//! }
//! ```

use crate::cmd::SingleEntity;
use crate::condition::{Condition, ScoreRange};
use crate::state::Ticks;

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
/// - threshold queries require a positive finite value that rounds to at least
///   1 internal stat unit
///
/// Use [`DamageThreshold::raw_stat`] only when you need to match the raw
/// Minecraft scoreboard stat value directly. Threshold queries require a
/// positive raw stat value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageThreshold {
    /// Number of hearts (1.0 = 1 heart = 10 stat units).
    Hearts(f32),
    /// Raw Minecraft stat units (same scale as `minecraft.damage_taken`).
    RawStat(i32),
}

impl DamageThreshold {
    /// Threshold in hearts (1.0 = one heart, 0.5 = half a heart).
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageThreshold::hearts` for the canonical contract."]
    pub fn hearts(h: f32) -> Self {
        Self::Hearts(h)
    }

    /// Fallible threshold in hearts.
    ///
    /// Values must be finite, greater than `0.0`, and round to at least
    /// 1 raw Minecraft damage stat unit. One raw stat unit is 0.1 heart, so
    /// values below 0.05 heart are not meaningful for `*_at_least` queries.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageThreshold::try_hearts` for the canonical contract."]
    pub fn try_hearts(h: f32) -> Result<Self, String> {
        Self::validate_hearts(h).map(|_| Self::Hearts(h))
    }

    /// Raw scoreboard stat units — advanced use only.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageThreshold::raw_stat` for the canonical contract."]
    pub fn raw_stat(v: i32) -> Self {
        Self::RawStat(v)
    }

    /// Fallible raw scoreboard stat threshold.
    ///
    /// Values must be greater than zero for `*_at_least` queries.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageThreshold::try_raw_stat` for the canonical contract."]
    pub fn try_raw_stat(v: i32) -> Result<Self, String> {
        Self::validate_raw_stat(v).map(|_| Self::RawStat(v))
    }

    /// Convert to the raw Minecraft scoreboard stat integer used internally.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageThreshold::to_raw_stat` for the canonical contract."]
    pub fn to_raw_stat(self) -> i32 {
        match self {
            Self::Hearts(h) => (h * 10.0).round() as i32,
            Self::RawStat(v) => v,
        }
    }

    fn to_query_raw_stat(self, helper: &str) -> i32 {
        match self {
            Self::Hearts(h) => Self::validate_hearts(h)
                .unwrap_or_else(|message| panic!("DamageTracker::{helper}: {message}")),
            Self::RawStat(v) => Self::validate_raw_stat(v)
                .unwrap_or_else(|message| panic!("DamageTracker::{helper}: {message}")),
        }
    }

    fn validate_hearts(h: f32) -> Result<i32, String> {
        if !h.is_finite() {
            return Err(format!(
                "invalid DamageThreshold::hearts({h:?}); threshold must be finite"
            ));
        }
        if h <= 0.0 {
            return Err(format!(
                "invalid DamageThreshold::hearts({h:?}); threshold must be greater than 0.0 hearts"
            ));
        }

        let raw = (f64::from(h) * 10.0).round();
        if !raw.is_finite() || raw > f64::from(i32::MAX) {
            return Err(format!(
                "invalid DamageThreshold::hearts({h:?}); value rounds to {raw:?} raw damage stat units, which exceeds the Minecraft scoreboard range"
            ));
        }

        let raw = raw as i32;
        if raw <= 0 {
            return Err(format!(
                "invalid DamageThreshold::hearts({h:?}); value rounds to {raw} raw damage stat units, but threshold queries require at least 1"
            ));
        }

        Ok(raw)
    }

    fn validate_raw_stat(v: i32) -> Result<i32, String> {
        if v <= 0 {
            return Err(format!(
                "invalid DamageThreshold::raw_stat({v}); threshold queries require a positive raw damage stat value"
            ));
        }

        Ok(v)
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
    /// Call once in a `#[datapack_component(Load)]` function.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::define` for the canonical contract."]
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

    /// Update damage tracking for one entity (call every tick).
    ///
    /// Algorithm (in order):
    /// 1. `delta = stat`
    /// 2. `delta -= prev`
    /// 3. If `delta > 0`: `last = delta`
    /// 4. If `delta > 0`: `hurt_age = 0`
    /// 5. Unless `delta > 0`: `hurt_age += 1`
    /// 6. `prev = stat`
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::tick` for the canonical contract."]
    pub fn tick(target: SingleEntity) -> Vec<String> {
        Self::tick_selector(target.to_string())
    }

    /// Explicit unchecked compatibility path for selector syntax Sand cannot
    /// model. Passing a multi-entity selector produces invalid scoreboard
    /// operation sources; prefer [`tick`](Self::tick) or
    /// [`tick_players`](Self::tick_players).
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::tick_raw` for the canonical contract."]
    pub fn tick_raw(selector: impl std::fmt::Display) -> Vec<String> {
        Self::tick_selector(selector.to_string())
    }

    fn tick_selector(sel: String) -> Vec<String> {
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

    /// Tick every online player independently.
    ///
    /// Scoreboard operation sources must resolve to one holder, so this lowers
    /// through `execute as @a` and uses `@s` on both sides of each operation.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::tick_players` for the canonical contract."]
    pub fn tick_players() -> Vec<String> {
        Self::tick(SingleEntity::self_())
            .into_iter()
            .map(|command| {
                command.strip_prefix("execute as @s").map_or_else(
                    || format!("execute as @a run {command}"),
                    |rest| format!("execute as @a{rest}"),
                )
            })
            .collect()
    }

    // ── Conditions ────────────────────────────────────────────────────────────

    /// Condition: `selector` was damaged this tick (delta > 0).
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::damaged_this_tick` for the canonical contract."]
    pub fn damaged_this_tick(selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_DELTA_OBJ.to_string(),
            ScoreRange::Gte(1),
        )
    }

    /// Condition: `selector` was NOT damaged this tick (delta == 0).
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::not_damaged_this_tick` for the canonical contract."]
    pub fn not_damaged_this_tick(selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_DELTA_OBJ.to_string(),
            ScoreRange::Eq(0),
        )
    }

    /// Condition: `selector` took at least `threshold` damage this tick.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::current_damage_at_least` for the canonical contract."]
    pub fn current_damage_at_least(selector: &str, threshold: DamageThreshold) -> Condition {
        let min_raw = threshold.to_query_raw_stat("current_damage_at_least");
        Condition::score(
            selector.to_string(),
            DAMAGE_DELTA_OBJ.to_string(),
            ScoreRange::Gte(min_raw),
        )
    }

    /// Condition: the last recorded damage for `selector` was at least `threshold`.
    ///
    /// Uses `sd_dmg_last`, which persists between damage events.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::last_damage_at_least` for the canonical contract."]
    pub fn last_damage_at_least(selector: &str, threshold: DamageThreshold) -> Condition {
        let min_raw = threshold.to_query_raw_stat("last_damage_at_least");
        Condition::score(
            selector.to_string(),
            DAMAGE_LAST_OBJ.to_string(),
            ScoreRange::Gte(min_raw),
        )
    }

    /// Condition: `selector` was last hurt within `ticks` ticks ago.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::hurt_within` for the canonical contract."]
    pub fn hurt_within(selector: &str, ticks: Ticks) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_HURT_AGE_OBJ.to_string(),
            ScoreRange::Lte(ticks.get() as i32),
        )
    }

    // ── Raw score accessors (advanced use) ────────────────────────────────────

    /// The raw current-tick delta objective name.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::current_damage_raw` for the canonical contract."]
    pub fn current_damage_raw(selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_DELTA_OBJ.to_string(),
            ScoreRange::Gte(1),
        )
    }

    /// The raw last-damage objective name.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::last_damage_raw` for the canonical contract."]
    pub fn last_damage_raw(selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_LAST_OBJ.to_string(),
            ScoreRange::Gte(1),
        )
    }

    /// The ticks-since-hurt objective name (for use with ScoreVar).
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::ticks_since_hurt` for the canonical contract."]
    pub fn ticks_since_hurt(selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_HURT_AGE_OBJ.to_string(),
            ScoreRange::Gte(0),
        )
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
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::was_hurt` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::not_hurt_for` for the canonical contract."]
    pub fn not_hurt_for(selector: &str, ticks: Ticks) -> Condition {
        Condition::score(
            selector.to_string(),
            DAMAGE_HURT_AGE_OBJ.to_string(),
            ScoreRange::Gte(ticks.get() as i32 + 1),
        )
    }

    /// Reset the last-recorded damage delta for `selector` to 0.
    ///
    /// Useful after consuming a damage event so stale deltas don't re-fire
    /// condition checks on the next tick.
    ///
    /// Returns a single scoreboard `set ... 0` command.
    #[doc = "**API Contract:** Run `sand api show sand::systems::damage::DamageTracker::clear_recent_damage` for the canonical contract."]
    pub fn clear_recent_damage(selector: impl std::fmt::Display) -> String {
        format!("scoreboard players set {} {DAMAGE_LAST_OBJ} 0", selector)
    }
}

// ── Free function shims ───────────────────────────────────────────────────────

/// Condition shorthand: player at `selector` took damage this tick.
///
/// Requires `DamageTracker::tick()` to run every game tick.
#[doc = "**API Contract:** Run `sand api show sand::systems::damage::recently_damaged` for the canonical contract."]
pub fn recently_damaged(selector: &str) -> Condition {
    DamageTracker::damaged_this_tick(selector)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    fn condition_command(condition: Condition) -> String {
        condition
            .execute_commands(false, "say matched")
            .into_iter()
            .next()
            .expect("damage conditions lower to one command")
    }

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
        let cmds = DamageTracker::tick(SingleEntity::self_());
        assert_eq!(cmds.len(), 6, "expected 6 tick commands: {cmds:?}");

        // 1: delta = stat
        assert!(
            cmds[0].contains(&format!("{DAMAGE_DELTA_OBJ} = @s {DAMAGE_STAT_OBJ}")),
            "step 1 delta=stat: {}",
            cmds[0]
        );
        // 2: delta -= prev
        assert!(
            cmds[1].contains(&format!("{DAMAGE_DELTA_OBJ} -= @s {DAMAGE_PREV_OBJ}")),
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
            cmds[5].contains(&format!("{DAMAGE_PREV_OBJ} = @s {DAMAGE_STAT_OBJ}")),
            "step 6 prev=stat: {}",
            cmds[5]
        );
    }

    #[test]
    fn tick_players_uses_single_holder_operations() {
        let commands = DamageTracker::tick_players();
        assert_eq!(commands, DamageTracker::tick_players());
        assert!(
            commands
                .iter()
                .all(|command| command.starts_with("execute as @a"))
        );
        assert!(
            commands
                .iter()
                .all(|command| !command.contains("operation @a"))
        );
        assert!(commands.iter().all(|command| !command.contains(" = @a")));
        assert!(commands.iter().all(|command| !command.contains("-= @a")));
        for index in [0, 1, 2, 5] {
            assert!(commands[index].contains("operation @s"));
            assert!(commands[index].contains(" @s sd_dmg_"));
        }
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

    #[test]
    fn threshold_try_hearts_accepts_meaningful_values() {
        assert_eq!(DamageThreshold::try_hearts(1.0).unwrap().to_raw_stat(), 10);
        assert_eq!(DamageThreshold::try_hearts(0.5).unwrap().to_raw_stat(), 5);
        assert_eq!(DamageThreshold::try_hearts(0.05).unwrap().to_raw_stat(), 1);
    }

    #[test]
    fn threshold_try_hearts_rejects_invalid_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 0.0, 0.01] {
            let err = DamageThreshold::try_hearts(value).unwrap_err();
            assert!(
                err.contains("DamageThreshold::hearts"),
                "error should name hearts constructor: {err}"
            );
        }
    }

    #[test]
    fn threshold_try_hearts_rejects_unrepresentable_values() {
        for value in [214_748_364.8, 300_000_000.0, f32::MAX] {
            let err = DamageThreshold::try_hearts(value).unwrap_err();
            assert!(
                err.contains("exceeds the Minecraft scoreboard range"),
                "error should mention scoreboard range: {err}"
            );
        }
    }

    #[test]
    fn threshold_try_raw_stat_rejects_zero_and_negative_values() {
        for value in [0, -1] {
            let err = DamageThreshold::try_raw_stat(value).unwrap_err();
            assert!(
                err.contains("DamageThreshold::raw_stat"),
                "error should name raw stat constructor: {err}"
            );
        }
    }

    #[test]
    fn threshold_try_raw_stat_accepts_positive_values() {
        assert_eq!(DamageThreshold::try_raw_stat(42).unwrap().to_raw_stat(), 42);
    }

    // ── Conditions ────────────────────────────────────────────────────────────

    #[test]
    fn damaged_this_tick_condition() {
        assert_eq!(
            condition_command(DamageTracker::damaged_this_tick("@s")),
            format!("execute if score @s {DAMAGE_DELTA_OBJ} matches 1.. run say matched")
        );
    }

    #[test]
    fn not_damaged_this_tick_condition() {
        assert_eq!(
            condition_command(DamageTracker::not_damaged_this_tick("@s")),
            format!("execute if score @s {DAMAGE_DELTA_OBJ} matches 0 run say matched")
        );
    }

    #[test]
    fn current_damage_at_least_hearts() {
        assert_eq!(
            condition_command(DamageTracker::current_damage_at_least(
                "@s",
                DamageThreshold::hearts(1.0),
            )),
            format!("execute if score @s {DAMAGE_DELTA_OBJ} matches 10.. run say matched")
        );
    }

    #[test]
    fn last_damage_at_least_half_heart() {
        assert_eq!(
            condition_command(DamageTracker::last_damage_at_least(
                "@s",
                DamageThreshold::hearts(0.5),
            )),
            format!("execute if score @s {DAMAGE_LAST_OBJ} matches 5.. run say matched")
        );
    }

    #[test]
    #[should_panic(expected = "DamageTracker::current_damage_at_least")]
    fn current_damage_at_least_rejects_nan_hearts() {
        let _ = DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(f32::NAN));
    }

    #[test]
    #[should_panic(expected = "DamageTracker::current_damage_at_least")]
    fn current_damage_at_least_rejects_infinite_hearts() {
        let _ =
            DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(f32::INFINITY));
    }

    #[test]
    #[should_panic(expected = "DamageThreshold::hearts")]
    fn current_damage_at_least_rejects_negative_hearts() {
        let _ = DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(-1.0));
    }

    #[test]
    #[should_panic(expected = "rounds to 0 raw damage stat units")]
    fn current_damage_at_least_rejects_hearts_that_round_to_zero() {
        let _ = DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(0.01));
    }

    #[test]
    #[should_panic(expected = "exceeds the Minecraft scoreboard range")]
    fn current_damage_at_least_rejects_hearts_above_scoreboard_range() {
        let _ =
            DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(300_000_000.0));
    }

    #[test]
    #[should_panic(expected = "exceeds the Minecraft scoreboard range")]
    fn current_damage_at_least_rejects_boundary_hearts_above_scoreboard_range() {
        let _ =
            DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(214_748_364.8));
    }

    #[test]
    #[should_panic(expected = "DamageTracker::last_damage_at_least")]
    fn last_damage_at_least_rejects_zero_raw_stat() {
        let _ = DamageTracker::last_damage_at_least("@s", DamageThreshold::raw_stat(0));
    }

    #[test]
    #[should_panic(expected = "DamageThreshold::raw_stat")]
    fn last_damage_at_least_rejects_negative_raw_stat() {
        let _ = DamageTracker::last_damage_at_least("@s", DamageThreshold::raw_stat(-1));
    }

    #[test]
    fn hurt_within_ticks() {
        assert_eq!(
            condition_command(DamageTracker::hurt_within("@s", Ticks::new(20))),
            format!("execute if score @s {DAMAGE_HURT_AGE_OBJ} matches ..20 run say matched")
        );
    }

    #[test]
    fn free_fn_recently_damaged() {
        assert_eq!(
            condition_command(recently_damaged("@s")),
            format!("execute if score @s {DAMAGE_DELTA_OBJ} matches 1.. run say matched")
        );
    }

    // ── New helpers: was_hurt, not_hurt_for, clear_recent_damage ─────────────

    #[test]
    fn was_hurt_is_alias_for_damaged_this_tick() {
        assert_eq!(
            condition_command(DamageTracker::was_hurt("@s")),
            condition_command(DamageTracker::damaged_this_tick("@s"))
        );
    }

    #[test]
    fn not_hurt_for_uses_age_gte_n_plus_one() {
        assert_eq!(
            condition_command(DamageTracker::not_hurt_for("@s", Ticks::new(20))),
            format!("execute if score @s {DAMAGE_HURT_AGE_OBJ} matches 21.. run say matched")
        );
    }

    #[test]
    fn not_hurt_for_zero_ticks() {
        // not_hurt_for(0) → age >= 1, i.e. "not hurt this tick"
        assert_eq!(
            condition_command(DamageTracker::not_hurt_for("@s", Ticks::new(0))),
            format!("execute if score @s {DAMAGE_HURT_AGE_OBJ} matches 1.. run say matched")
        );
    }

    #[test]
    fn was_hurt_and_not_hurt_for_are_complementary() {
        // was_hurt → delta >= 1
        // not_hurt_for(0) → age >= 1 (not hurt this tick)
        // They use different objectives so they are not direct complements,
        // but both should produce Gte conditions.
        assert_eq!(
            condition_command(DamageTracker::was_hurt("@s")),
            format!("execute if score @s {DAMAGE_DELTA_OBJ} matches 1.. run say matched")
        );
        assert_eq!(
            condition_command(DamageTracker::not_hurt_for("@s", Ticks::new(10))),
            format!("execute if score @s {DAMAGE_HURT_AGE_OBJ} matches 11.. run say matched")
        );
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
