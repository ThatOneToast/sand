/// Scoreboard-based cooldown abstraction.
///
/// A cooldown is simply a scoreboard objective whose value counts down from
/// some positive number to zero. While the score is > 0 the ability is on
/// cooldown; when it reaches 0 the ability is ready again.
///
/// # Typical datapack setup
///
/// ```rust,ignore
/// use sand_core::cmd::{Cooldown, Objective, ScoreHolder};
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Cooldown",
    aliases = ["sand::cmd::Cooldown", "sand::prelude::cmd::Cooldown"],
    module = "sand::command",
    summary = "Scoreboard-based cooldown system for ability tracking.",
    context = "Scoreboard-based cooldown system for ability tracking. A cooldown is a countdown timer backed by a scoreboard objective. While the score is > 0, the ability is on cooldown; at 0 it's ready.",
    minecraft = "A cooldown is a countdown timer backed by a scoreboard objective. While the score is > 0, the ability is on cooldown; at 0 it's ready.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Cooldown;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::new",
        aliases = ["sand::cmd::Cooldown::new", "sand::prelude::cmd::Cooldown::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a cooldown instance with duration in ticks.",
        context = "Create a cooldown instance with duration in ticks. The objective must already be defined. Both `Cooldown` and objective are suitable for `static`/`const` declarations (no heap allocation).",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(objective = "`objective` is used when creating a cooldown instance with duration in ticks.", ticks = "`ticks` provides the Minecraft tick duration used to create a cooldown instance with duration in ticks."),
        returns = "A `Cooldown` representing a cooldown instance with duration in ticks.",
        example = "static COOLDOWN_OBJ: Objective = Objective::new(\"spell_cd\");\nstatic SPELL_COOLDOWN: Cooldown = Cooldown::new(&COOLDOWN_OBJ, 60); // 3 seconds",
    )]
    pub const fn new(objective: &'static Objective, ticks: u32) -> Self {
        Self { objective, ticks }
    }

    // ── Scoreboard registration ───────────────────────────────────────────────

    /// `scoreboard objectives add <name> dummy` — register the underlying objective.
    ///
    /// Call this once in your data pack's `load` function or setup phase.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::register",
        aliases = ["sand::cmd::Cooldown::register", "sand::prelude::cmd::Cooldown::register"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives add <name> dummy` — register the underlying objective.",
        context = "`scoreboard objectives add <name> dummy` — register the underlying objective. Call this once in your data pack's `load` function or setup phase.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Call this once in your data pack's `load` function or setup phase."],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to emit the documented `scoreboard objectives add <name> dummy` — register the underlying objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown)  {\n    let register = cooldown_value.register();\n}",
    )]
    pub fn register(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective.name())
    }

    // ── Per-ability-use commands ──────────────────────────────────────────────

    /// Guard clause: return early if the cooldown is active (score > 0).
    ///
    /// Place this at the start of your ability function to prevent use while cooling.
    /// If score is > 0, the function returns 0 immediately. Otherwise execution continues.
    /// Produces: `execute if score <holder> <obj> matches 1.. run return 0`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::guard",
        aliases = ["sand::cmd::Cooldown::guard", "sand::prelude::cmd::Cooldown::guard"],
        module = "sand::command",
        kind = "method",
        summary = "Guard clause: return early if the cooldown is active (score > 0).",
        context = "Guard clause: return early if the cooldown is active (score > 0). Place this at the start of your ability function to prevent use while cooling. If score is > 0, the function returns 0 immediately. Otherwise execution continues. Produces: `execute if score <holder> <obj> matches 1.. run return 0`",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to guard clause: return early if the cooldown is active (score > 0)."),
        returns = "Place this at the start of your ability function to prevent use while cooling. If score is > 0, the function returns 0 immediately. Otherwise execution continues. Produces: `execute if score <holder> <obj> matches 1.. run return 0`",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let guard = cooldown_value.guard(holder);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::start",
        aliases = ["sand::cmd::Cooldown::start", "sand::prelude::cmd::Cooldown::start"],
        module = "sand::command",
        kind = "method",
        summary = "Start the cooldown by setting the score to the configured duration.",
        context = "Start the cooldown by setting the score to the configured duration. Call this after the ability executes to begin the countdown. Produces: `scoreboard players set <holder> <obj> <ticks>`",
        minecraft = "Call this after the ability executes to begin the countdown. Produces: `scoreboard players set <holder> <obj> <ticks>`",
        use_when = ["Call this after the ability executes to begin the countdown. Produces: `scoreboard players set <holder> <obj> <ticks>`"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to start the cooldown by setting the score to the configured duration."),
        returns = "The string value produced to start the cooldown by setting the score to the configured duration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let start = cooldown_value.start(holder);\n}",
    )]
    pub fn start(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, self.ticks as i32)
    }

    /// Reset the cooldown immediately to ready (score = 0).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::reset",
        aliases = ["sand::cmd::Cooldown::reset", "sand::prelude::cmd::Cooldown::reset"],
        module = "sand::command",
        kind = "method",
        summary = "Reset the cooldown immediately to ready (score = 0).",
        context = "Reset the cooldown immediately to ready (score = 0). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to reset the cooldown immediately to ready (score = 0)."),
        returns = "The string value produced to reset the cooldown immediately to ready (score = 0).",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let reset = cooldown_value.reset(holder);\n}",
    )]
    pub fn reset(&self, holder: ScoreHolder) -> String {
        self.objective.set(holder, 0)
    }

    // ── Per-tick command ──────────────────────────────────────────────────────

    /// Decrement the cooldown by 1 tick (only if score > 0).
    ///
    /// Place this in your data pack's tick function to countdown all active cooldowns.
    /// Safe to call repeatedly — only decrements if score is positive.
    /// Produces: `execute if score <holder> <obj> matches 1.. run scoreboard players remove <holder> <obj> 1`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::tick",
        aliases = ["sand::cmd::Cooldown::tick", "sand::prelude::cmd::Cooldown::tick"],
        module = "sand::command",
        kind = "method",
        summary = "Decrement the cooldown by 1 tick (only if score > 0).",
        context = "Decrement the cooldown by 1 tick (only if score > 0). Place this in your data pack's tick function to countdown all active cooldowns. Safe to call repeatedly — only decrements if score is positive. Produces: `execute if score <holder> <obj> matches 1.. run scoreboard players remove <holder> <obj> 1`",
        minecraft = "Place this in your data pack's tick function to countdown all active cooldowns. Safe to call repeatedly — only decrements if score is positive. Produces: `execute if score <holder> <obj> matches 1.. run scoreboard players remove <holder> <obj> 1`",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to decrement the cooldown by 1 tick (only if score > 0)."),
        returns = "The string value produced to decrement the cooldown by 1 tick (only if score > 0).",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let tick = cooldown_value.tick(holder);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::is_active",
        aliases = ["sand::cmd::Cooldown::is_active", "sand::prelude::cmd::Cooldown::is_active"],
        module = "sand::command",
        kind = "method",
        summary = "Return a condition fragment: true while the cooldown is active (score >= 1).",
        context = "Return a condition fragment: true while the cooldown is active (score >= 1). Use with `Execute::if_()` to conditionally execute code when cooldown is active. Produces: `if score <holder> <obj> matches 1..`",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Use with `Execute::if_()` to conditionally execute code when cooldown is active. Produces: `if score <holder> <obj> matches 1..`"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to return a condition fragment: true while the cooldown is active (score >= 1)."),
        returns = "Return a condition fragment: true while the cooldown is active (score >= 1).",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let is_active = cooldown_value.is_active(holder);\n}",
    )]
    pub fn is_active(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 1..", holder, self.objective.name())
    }

    /// Return a condition fragment: true when the cooldown is **ready** (score = 0).
    ///
    /// Use with `Execute::if_()` to conditionally execute code when ability is ready.
    /// Produces: `if score <holder> <obj> matches 0`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::is_ready",
        aliases = ["sand::cmd::Cooldown::is_ready", "sand::prelude::cmd::Cooldown::is_ready"],
        module = "sand::command",
        kind = "method",
        summary = "Return a condition fragment: true when the cooldown is ready (score = 0).",
        context = "Return a condition fragment: true when the cooldown is ready (score = 0). Use with `Execute::if_()` to conditionally execute code when ability is ready. Produces: `if score <holder> <obj> matches 0`",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Use with `Execute::if_()` to conditionally execute code when ability is ready. Produces: `if score <holder> <obj> matches 0`"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to return a condition fragment: true when the cooldown is ready (score = 0)."),
        returns = "Return a condition fragment: true when the cooldown is ready (score = 0).",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown, holder: sand::command::ScoreHolder)  {\n    let is_ready = cooldown_value.is_ready(holder);\n}",
    )]
    pub fn is_ready(&self, holder: ScoreHolder) -> String {
        format!("if score {} {} matches 0", holder, self.objective.name())
    }

    /// Return a reference to the underlying objective.
    ///
    /// Useful if you need direct access to the objective for other operations.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::objective",
        aliases = ["sand::cmd::Cooldown::objective", "sand::prelude::cmd::Cooldown::objective"],
        module = "sand::command",
        kind = "method",
        summary = "Return a reference to the underlying objective. Useful if you need direct access to the objective for other operations.",
        context = "Return a reference to the underlying objective. Useful if you need direct access to the objective for other operations. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "Return a reference to the underlying objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown)  {\n    let objective = cooldown_value.objective();\n}",
    )]
    pub fn objective(&self) -> &Objective {
        self.objective
    }

    /// Return the configured cooldown duration in ticks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Cooldown::ticks",
        aliases = ["sand::cmd::Cooldown::ticks", "sand::prelude::cmd::Cooldown::ticks"],
        module = "sand::command",
        kind = "method",
        summary = "Return the configured cooldown duration in ticks.",
        context = "Return the configured cooldown duration in ticks. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "Return the configured cooldown duration in ticks.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooldown_value: &sand::command::Cooldown)  {\n    let ticks = cooldown_value.ticks();\n}",
    )]
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
