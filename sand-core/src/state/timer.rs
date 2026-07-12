//! Tick-based duration and timer utilities.

pub use sand_components::Ticks;

// ── Timer ─────────────────────────────────────────────────────────────────────

/// A scoreboard-backed countdown timer.
///
/// A `Timer` counts down from a starting value to zero. It does not generate
/// conditions; use [`CooldownVar`](super::cooldown::CooldownVar) when you need
/// ready/active conditions.
///
/// ```rust,ignore
/// use sand_core::state::{Timer, Ticks};
///
/// static BLINK: Timer = Timer::new("blink_cd", Ticks::seconds(5));
///
/// let cmds = vec![
///     BLINK.define(),
///     BLINK.start("@s"),
///     BLINK.tick("@a"),
/// ];
/// ```
pub struct Timer {
    name: &'static str,
    duration: Ticks,
}

impl Timer {
    /// Create a new timer with the given objective name and duration.
    pub const fn new(name: &'static str, duration: Ticks) -> Self {
        Self { name, duration }
    }

    /// Build an automatic export lifecycle descriptor for this timer.
    /// Call `.auto_tick()` on the result to opt into per-player ticking.
    pub const fn lifecycle(&self) -> crate::state::StateLifecycle {
        crate::state::StateLifecycle::score(self.name)
    }

    /// Return the actual scoreboard objective name.
    pub fn objective_name(&self) -> String {
        super::score::objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy`
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Enroll this timer in Sand's global lifecycle registry.
    ///
    /// The objective will be included in the next call to
    /// [`define_registered_state`](crate::state::define_registered_state).
    /// Calling `.register()` multiple times for the same timer is a no-op.
    pub fn register(&self) {
        crate::state::register_load_objective(self.objective_name(), "dummy");
    }

    /// Set the timer to the configured duration for `selector`.
    pub fn start(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            self.duration.get()
        )
    }

    /// Decrement the timer by 1 tick for `selector` (only if > 0).
    pub fn tick(&self, selector: impl std::fmt::Display) -> String {
        let selector = selector.to_string();
        let obj = self.objective_name();
        format!(
            "execute if score {selector} {obj} matches 1.. run scoreboard players remove {selector} {obj} 1"
        )
    }

    /// Reset the timer to zero for `selector`.
    pub fn reset(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Condition: timer has expired (score == 0).
    ///
    /// Use this to check if the timer has counted down to zero.
    pub fn expired(&self, selector: &str) -> crate::condition::Condition {
        crate::condition::Condition::Score {
            selector: selector.to_string(),
            objective: self.objective_name(),
            range: crate::condition::ScoreRange::Eq(0),
        }
    }

    /// Condition: timer is still running (score >= 1).
    pub fn active(&self, selector: &str) -> crate::condition::Condition {
        crate::condition::Condition::Score {
            selector: selector.to_string(),
            objective: self.objective_name(),
            range: crate::condition::ScoreRange::Gte(1),
        }
    }

    /// Guard clause: return early if the timer is still running (score >= 1).
    ///
    /// Produces: `execute if score <selector> <obj> matches 1.. run return 0`
    pub fn guard_active(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
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

    #[test]
    fn ticks_new() {
        assert_eq!(Ticks::new(60).get(), 60);
    }

    #[test]
    fn ticks_seconds() {
        assert_eq!(Ticks::seconds(3).get(), 60);
    }

    #[test]
    fn ticks_minutes() {
        assert_eq!(Ticks::minutes(1).get(), 1200);
    }

    #[test]
    fn ticks_as_seconds() {
        assert_eq!(Ticks::new(60).as_seconds(), 3);
        assert_eq!(Ticks::new(25).as_seconds(), 1); // floor division
    }

    static BLINK: Timer = Timer::new("blink_cd", Ticks::new(100));

    #[test]
    fn timer_define() {
        assert_eq!(BLINK.define(), "scoreboard objectives add blink_cd dummy");
    }

    #[test]
    fn timer_start() {
        assert_eq!(BLINK.start("@s"), "scoreboard players set @s blink_cd 100");
    }

    #[test]
    fn timer_tick() {
        let cmd = BLINK.tick("@a");
        assert!(cmd.contains("matches 1.."), "got: {cmd}");
        assert!(cmd.contains("remove @a blink_cd 1"), "got: {cmd}");
    }

    #[test]
    fn timer_reset() {
        assert_eq!(BLINK.reset("@s"), "scoreboard players set @s blink_cd 0");
    }

    #[test]
    fn timer_expired_condition() {
        use crate::condition::{Condition, ScoreRange};
        let cond = BLINK.expired("@s");
        match cond {
            Condition::Score {
                selector,
                objective,
                range: ScoreRange::Eq(0),
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, "blink_cd");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn timer_active_condition() {
        use crate::condition::{Condition, ScoreRange};
        let cond = BLINK.active("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gte(1),
                ..
            }
        ));
    }

    #[test]
    fn timer_guard_active() {
        let cmd = BLINK.guard_active("@s");
        assert_eq!(cmd, "execute if score @s blink_cd matches 1.. run return 0");
    }
}
