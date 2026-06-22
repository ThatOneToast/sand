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

    /// Return the actual scoreboard objective name.
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
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
}
