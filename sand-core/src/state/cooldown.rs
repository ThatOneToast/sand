//! High-level scoreboard-based ability cooldown.
//!
//! This is the typed state API counterpart to [`sand_core::cmd::Cooldown`].
//! The two types have the same conceptual purpose but different constructors:
//! - `state::Cooldown` takes `(&'static str, Ticks)` and hides the `Objective` plumbing.
//! - `cmd::Cooldown` takes `(&'static Objective, u32)` for lower-level control.

use crate::condition::Condition;
use crate::state::score::objective_name;
use crate::state::timer::Ticks;

// ── Cooldown ──────────────────────────────────────────────────────────────────

/// A scoreboard-backed ability cooldown timer with typed condition support.
///
/// ```rust,ignore
/// use sand_core::state::{Cooldown, Ticks};
///
/// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
///
/// let load_cmds = vec![DASH.define()];
/// let tick_cmds = vec![DASH.tick("@a")];
///
/// // Condition: cooldown is ready (score == 0)
/// let cond = DASH.ready("@s");
/// ```
pub struct Cooldown {
    name: &'static str,
    duration: Ticks,
}

impl Cooldown {
    /// Create a cooldown with the given objective name and duration.
    pub const fn new(name: &'static str, duration: Ticks) -> Self {
        Self { name, duration }
    }

    /// Build an automatic export lifecycle descriptor for this cooldown.
    /// Call `.auto_tick()` on the result to opt into per-player ticking.
    pub const fn lifecycle(&self) -> crate::state::StateLifecycle {
        crate::state::StateLifecycle::score(self.name)
    }

    /// Return the actual scoreboard objective name.
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Enroll this cooldown in Sand's global lifecycle registry.
    ///
    /// The objective will be included in the next call to
    /// [`define_registered_state`](crate::state::define_registered_state).
    /// Calling `.register()` multiple times for the same cooldown is a no-op.
    pub fn register(&self) {
        crate::state::register_load_objective(self.objective_name(), "dummy");
    }

    /// Set the cooldown score to the configured duration for `selector`.
    pub fn start(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            self.duration.get()
        )
    }

    /// Reset the cooldown to 0 for `selector` (immediately ready).
    pub fn stop(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Decrement the cooldown by 1 tick for `selector` (only if score > 0).
    ///
    /// Place this in your tick function.
    pub fn tick(&self, selector: impl std::fmt::Display) -> String {
        let selector = selector.to_string();
        let obj = self.objective_name();
        format!(
            "execute if score {selector} {obj} matches 1.. run scoreboard players remove {selector} {obj} 1"
        )
    }

    /// Guard clause: return early if the cooldown is still active (score > 0).
    ///
    /// Produces: `execute if score <selector> <obj> matches 1.. run return 0`
    pub fn guard(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
            selector,
            self.objective_name()
        )
    }

    /// Condition: cooldown is ready (`if score <sel> <obj> matches 0`).
    pub fn ready(&self, selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: self.objective_name(),
            range: crate::condition::ScoreRange::Eq(0),
        }
    }

    /// Condition: cooldown is active (`if score <sel> <obj> matches 1..`).
    pub fn active(&self, selector: &str) -> Condition {
        Condition::Score {
            selector: selector.to_string(),
            objective: self.objective_name(),
            range: crate::condition::ScoreRange::Gte(1),
        }
    }

    /// Alias for [`ready`](Cooldown::ready) — more intuitive name when thinking about
    /// whether the timer has "expired" (counted down to zero).
    pub fn expired(&self, selector: &str) -> Condition {
        self.ready(selector)
    }

    /// Alias for [`start`](Cooldown::start) — emphasizes the selector context.
    pub fn start_for(&self, selector: impl std::fmt::Display) -> String {
        self.start(selector)
    }

    /// Alias for [`stop`](Cooldown::stop) — resets the cooldown to zero immediately.
    pub fn reset_for(&self, selector: impl std::fmt::Display) -> String {
        self.stop(selector)
    }

    /// Tick the cooldown for all players (`@a`).
    ///
    /// Convenience for placing in a `#[component(Tick)]` function.
    pub fn tick_all_players(&self) -> String {
        self.tick("@a")
    }

    /// Alias for [`guard`](Cooldown::guard) — guards if the cooldown is NOT ready.
    ///
    /// Returns early (`return 0`) if the cooldown is still active.
    pub fn guard_active(&self, selector: impl std::fmt::Display) -> String {
        self.guard(selector)
    }

    /// Guard clause: return early if the cooldown IS ready (score == 0).
    ///
    /// Useful when you only want to run logic while the cooldown is active.
    ///
    /// Produces: `execute if score <selector> <obj> matches 0 run return 0`
    pub fn guard_ready(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 0 run return 0",
            selector,
            self.objective_name()
        )
    }

    /// Return the configured duration.
    pub fn duration(&self) -> Ticks {
        self.duration
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{Condition, ScoreRange};

    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));

    #[test]
    fn define_cmd() {
        assert_eq!(DASH.define(), "scoreboard objectives add dash dummy");
    }

    #[test]
    fn start_cmd() {
        assert_eq!(DASH.start("@s"), "scoreboard players set @s dash 60");
    }

    #[test]
    fn stop_cmd() {
        assert_eq!(DASH.stop("@s"), "scoreboard players set @s dash 0");
    }

    #[test]
    fn tick_cmd() {
        let cmd = DASH.tick("@a");
        assert!(cmd.contains("matches 1.."), "got: {cmd}");
        assert!(cmd.contains("remove @a dash 1"), "got: {cmd}");
    }

    #[test]
    fn guard_cmd() {
        let cmd = DASH.guard("@s");
        assert_eq!(cmd, "execute if score @s dash matches 1.. run return 0");
    }

    #[test]
    fn ready_condition() {
        let cond = DASH.ready("@s");
        match cond {
            Condition::Score {
                selector,
                objective,
                range: ScoreRange::Eq(0),
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, "dash");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn active_condition() {
        let cond = DASH.active("@s");
        match cond {
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn expired_is_alias_for_ready() {
        let a = DASH.expired("@s");
        let b = DASH.ready("@s");
        // Both should be Eq(0)
        assert!(matches!(
            a,
            Condition::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
        assert!(matches!(
            b,
            Condition::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn start_for_is_start() {
        assert_eq!(DASH.start_for("@s"), DASH.start("@s"));
    }

    #[test]
    fn reset_for_is_stop() {
        assert_eq!(DASH.reset_for("@s"), DASH.stop("@s"));
    }

    #[test]
    fn tick_all_players() {
        assert_eq!(DASH.tick_all_players(), DASH.tick("@a"));
    }

    #[test]
    fn guard_ready_cmd() {
        let cmd = DASH.guard_ready("@s");
        assert_eq!(cmd, "execute if score @s dash matches 0 run return 0");
    }

    #[test]
    fn guard_active_is_guard() {
        assert_eq!(DASH.guard_active("@s"), DASH.guard("@s"));
    }
}
