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
use sand_commands::{Objective, ScoreHolder};

// ── Cooldown ──────────────────────────────────────────────────────────────────

/// Scoreboard-based cooldown system for ability tracking.
///
/// A cooldown is a countdown timer backed by a scoreboard objective.
/// While the score is > 0, the ability is on cooldown; at 0 it's ready.
pub struct Cooldown {
    objective: &'static Objective,
    /// Duration in ticks that the cooldown lasts (e.g., 60 = 3 seconds at 20 tps).
    ticks: u32,
}

impl Cooldown {
    /// Create a cooldown instance with duration in ticks.
    ///
    /// The objective must already be defined. Both `Cooldown` and objective are
    /// suitable for `static`/`const` declarations (no heap allocation).
    ///
    /// # Example
    /// ```rust,ignore
    /// static COOLDOWN_OBJ: Objective = Objective::new("spell_cd");
    /// static SPELL_COOLDOWN: Cooldown = Cooldown::new(&COOLDOWN_OBJ, 60); // 3 seconds
    /// ```
    pub const fn new(objective: &'static Objective, ticks: u32) -> Self {
        Self { objective, ticks }
    }

    // ── Scoreboard registration ───────────────────────────────────────────────

    /// `scoreboard objectives add <name> dummy` — register the underlying objective.
    ///
    /// Call this once in your data pack's `load` function or setup phase.
    pub fn register(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective.name())
    }

    // ── Per-ability-use commands ──────────────────────────────────────────────

    /// Guard clause: return early if the cooldown is active (score > 0).
    ///
    /// Place this at the start of your ability function to prevent use while cooling.
    /// If score is > 0, the function returns 0 immediately. Otherwise execution continues.
    /// Produces: `execute if score <holder> <obj> matches 1.. run return 0`
    pub fn guard(&self, holder: ScoreHolder) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
            holder,
            self.objective.name(),
        )
    }

    /// Start the cooldown by setting the score to the configured duration.
    ///
    /// Call this after the ability executes to begin the countdown.
    /// Produces: `scoreboard players set <holder> <obj> <ticks>`
    pub fn start(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, self.ticks as i32)
    }

    /// Reset the cooldown immediately to ready (score = 0).
    pub fn reset(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, 0)
    }

    // ── Per-tick command ──────────────────────────────────────────────────────

    /// Decrement the cooldown by 1 tick (only if score > 0).
    ///
    /// Place this in your data pack's tick function to countdown all active cooldowns.
    /// Safe to call repeatedly — only decrements if score is positive.
    /// Produces: `execute if score <holder> <obj> matches 1.. run scoreboard players remove <holder> <obj> 1`
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

    /// Return a condition fragment: true while the cooldown is **active** (score >= 1).
    ///
    /// Use with `Execute::if_()` to conditionally execute code when cooldown is active.
    /// Produces: `if score <holder> <obj> matches 1..`
    pub fn is_active(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 1..", holder, self.objective.name())
    }

    /// Return a condition fragment: true when the cooldown is **ready** (score = 0).
    ///
    /// Use with `Execute::if_()` to conditionally execute code when ability is ready.
    /// Produces: `if score <holder> <obj> matches 0`
    pub fn is_ready(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 0", holder, self.objective.name())
    }

    /// Return a reference to the underlying objective.
    ///
    /// Useful if you need direct access to the objective for other operations.
    pub fn objective(&self) -> &Objective {
        self.objective
    }

    /// Return the configured cooldown duration in ticks.
    pub fn ticks(&self) -> u32 {
        self.ticks
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::{Objective, ScoreHolder};

    static OBJ: Objective = Objective::new("fireball_cd");
    static CD: Cooldown = Cooldown::new(&OBJ, 60);

    #[test]
    fn register() {
        assert_eq!(CD.register(), "scoreboard objectives add fireball_cd dummy");
    }

    #[test]
    fn guard() {
        let cmd = CD.guard(ScoreHolder::self_());
        assert_eq!(
            cmd,
            "execute if score @s fireball_cd matches 1.. run return 0"
        );
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
        assert_eq!(
            CD.is_active(ScoreHolder::self_()),
            "if score @s fireball_cd matches 1.."
        );
        assert_eq!(
            CD.is_ready(ScoreHolder::self_()),
            "if score @s fireball_cd matches 0"
        );
    }
}
