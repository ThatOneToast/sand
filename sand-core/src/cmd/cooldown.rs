/// Scoreboard-based cooldown abstraction.
///
/// A cooldown is simply a scoreboard objective whose value counts down from
/// some positive number to zero. While the score is > 0 the ability is on
/// cooldown; when it reaches 0 the ability is ready again.
///
/// # Typical datapack setup
///
/// ```rust,ignore
/// use sand_core::cmd::{Cooldown, Objective, ScoreHolder, Selector};
///
/// // One objective per cooldown (registered in your load function):
/// static FIREBALL_CD: Cooldown = Cooldown::new(&FIREBALL_COOLDOWN_OBJ, 60); // 3 s @ 20 tps
///
/// // In your ability-use function:
/// let cmds = mcfunction![
///     // Guard: bail if still cooling down
///     FIREBALL_CD.guard(ScoreHolder::self_());
///     // ... do the ability ...
///     // Start the cooldown
///     FIREBALL_CD.start(ScoreHolder::self_());
/// ];
///
/// // In your tick function (runs every tick for every player):
/// let cmds = mcfunction![
///     FIREBALL_CD.tick(ScoreHolder::self_());
/// ];
/// ```
///
/// # Wiring
///
/// The objective needs to be registered once (e.g. in a `load` function):
/// ```rust,ignore
/// "scoreboard objectives add fireball_cd dummy"
/// ```
/// The `Cooldown::register()` helper generates that command for you.

use super::objective::Objective;
use super::types::ScoreHolder;

// ── Cooldown ──────────────────────────────────────────────────────────────────

pub struct Cooldown {
    objective: &'static Objective,
    /// Duration in ticks that the cooldown lasts.
    ticks: u32,
}

impl Cooldown {
    /// Create a cooldown backed by `objective` lasting `ticks` ticks.
    ///
    /// Both `objective` and the `Cooldown` itself are suitable for `static`
    /// declarations (no heap allocation).
    pub const fn new(objective: &'static Objective, ticks: u32) -> Self {
        Self { objective, ticks }
    }

    // ── Scoreboard registration ───────────────────────────────────────────────

    /// `scoreboard objectives add <name> dummy` — register the objective.
    ///
    /// Call once in your `load` / `setup` function.
    pub fn register(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective.name())
    }

    // ── Per-ability-use commands ──────────────────────────────────────────────

    /// Guard the cooldown: if the score is > 0 the function **returns**
    /// immediately (ability blocked).
    ///
    /// Generates:
    /// ```text
    /// execute if score <holder> <obj> matches 1.. run return 0
    /// ```
    pub fn guard(&self, holder: ScoreHolder) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
            holder,
            self.objective.name(),
        )
    }

    /// Start the cooldown for `holder`, setting the score to `ticks`.
    ///
    /// Generates:
    /// ```text
    /// scoreboard players set <holder> <obj> <ticks>
    /// ```
    pub fn start(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, self.ticks as i32)
    }

    /// Reset the cooldown immediately (set score to 0).
    pub fn reset(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, 0)
    }

    // ── Per-tick command ──────────────────────────────────────────────────────

    /// Decrement the cooldown by 1 tick (only if it is currently > 0).
    ///
    /// Generates:
    /// ```text
    /// execute if score <holder> <obj> matches 1.. run scoreboard players remove <holder> <obj> 1
    /// ```
    pub fn tick(&self, holder: ScoreHolder) -> String {
        format!(
            "execute if score {} {} matches 1.. run scoreboard players remove {} {} 1",
            holder,
            self.objective.name(),
            holder,
            self.objective.name(),
        )
    }

    // ── Condition helpers ─────────────────────────────────────────────────────

    /// Returns a condition fragment `"if score <holder> <obj> matches 1.."` for
    /// use inside an `execute` chain — true while the cooldown is **active**.
    pub fn is_active(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 1..", holder, self.objective.name())
    }

    /// Returns a condition fragment `"if score <holder> <obj> matches 0"` —
    /// true when the cooldown is **ready**.
    pub fn is_ready(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 0", holder, self.objective.name())
    }

    /// Returns the current objective.
    pub fn objective(&self) -> &Objective {
        self.objective
    }

    /// Returns the cooldown duration in ticks.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::objective::Objective;
    use crate::cmd::selector::Selector;
    use crate::cmd::types::ScoreHolder;

    static OBJ: Objective = Objective::new("fireball_cd");
    static CD: Cooldown = Cooldown::new(&OBJ, 60);

    #[test]
    fn register() {
        assert_eq!(CD.register(), "scoreboard objectives add fireball_cd dummy");
    }

    #[test]
    fn guard() {
        let cmd = CD.guard(ScoreHolder::self_());
        assert_eq!(cmd, "execute if score @s fireball_cd matches 1.. run return 0");
    }

    #[test]
    fn start() {
        let cmd = CD.start(ScoreHolder::self_());
        assert_eq!(cmd, "scoreboard players set @s fireball_cd 60");
    }

    #[test]
    fn tick() {
        let cmd = CD.tick(ScoreHolder::self_());
        assert_eq!(
            cmd,
            "execute if score @s fireball_cd matches 1.. run scoreboard players remove @s fireball_cd 1"
        );
    }

    #[test]
    fn reset() {
        let cmd = CD.reset(ScoreHolder::self_());
        assert_eq!(cmd, "scoreboard players set @s fireball_cd 0");
    }

    #[test]
    fn is_active_ready() {
        assert_eq!(CD.is_active(ScoreHolder::self_()), "if score @s fireball_cd matches 1..");
        assert_eq!(CD.is_ready(ScoreHolder::self_()), "if score @s fireball_cd matches 0");
    }
}
