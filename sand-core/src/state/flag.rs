//! Boolean flag variable — a scoreboard objective whose value is 0 or 1.

use crate::condition::Condition;
use crate::state::score::objective_name;

// ── Flag ──────────────────────────────────────────────────────────────────────

/// A boolean scoreboard flag (score = 1 means `true`, score = 0 means `false`).
///
/// Declare once as a `static` and use throughout your datapack:
///
/// ```rust,ignore
/// use sand_core::state::Flag;
///
/// static CASTING: Flag = Flag::new("casting");
///
/// let cmds = vec![
///     CASTING.define(),
///     CASTING.enable("@s"),
/// ];
/// ```
pub struct Flag {
    name: &'static str,
}

impl Flag {
    /// Create a new flag with the given objective name.
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// Return the actual scoreboard objective name used in commands.
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Enroll this flag in Sand's global lifecycle registry.
    ///
    /// The objective will be included in the next call to
    /// [`define_registered_state`](crate::state::define_registered_state).
    /// Calling `.register()` multiple times for the same flag is a no-op.
    pub fn register(&self) {
        crate::state::register_load_objective(self.objective_name(), "dummy");
    }

    /// `scoreboard players set <selector> <obj> 1` — set flag to `true`.
    pub fn enable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 1",
            selector,
            self.objective_name()
        )
    }

    /// `scoreboard players set <selector> <obj> 0` — set flag to `false`.
    pub fn disable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Toggle the flag: set to `1` if currently `0`, else set to `0`.
    ///
    /// Returns two commands that together implement the toggle via a temp score.
    /// Generated commands:
    /// ```text
    /// execute if score <selector> <obj> matches 0 run scoreboard players set <selector> <obj> 1
    /// execute if score <selector> <obj> matches 1.. run scoreboard players set <selector> <obj> 0
    /// ```
    pub fn toggle(&self, selector: impl std::fmt::Display) -> Vec<String> {
        let selector = selector.to_string();
        let obj = self.objective_name();
        vec![
            format!(
                "execute if score {selector} {obj} matches 0 run scoreboard players set {selector} {obj} 1"
            ),
            format!(
                "execute if score {selector} {obj} matches 1.. run scoreboard players set {selector} {obj} 0"
            ),
        ]
    }

    /// Bind this flag to a selector to produce a condition builder.
    ///
    /// ```rust,ignore
    /// let cond = CASTING.of("@s").is_true();
    /// ```
    pub fn of<'a>(&'a self, selector: &str) -> FlagRef<'a> {
        FlagRef {
            objective: self.name,
            selector: selector.to_string(),
        }
    }

    /// Set the flag to an explicit boolean value.
    ///
    /// Equivalent to `enable` when `true` and `disable` when `false`.
    pub fn set(&self, selector: impl std::fmt::Display, value: bool) -> String {
        if value {
            self.enable(selector)
        } else {
            self.disable(selector)
        }
    }

    /// Alias for [`disable`](Flag::disable) — sets the flag to `false`.
    pub fn clear(&self, selector: impl std::fmt::Display) -> String {
        self.disable(selector)
    }

    /// Initialize the flag to `false` (0) only if the player has no existing score.
    ///
    /// Useful in join handlers to avoid overwriting state set by another system.
    ///
    /// Generated command:
    /// ```text
    /// execute unless score <selector> <obj> matches -2147483648.. run scoreboard players set <selector> <obj> 0
    /// ```
    pub fn init_false(&self, selector: impl std::fmt::Display) -> String {
        let obj = self.objective_name();
        format!(
            "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} 0"
        )
    }

    /// Initialize the flag to `true` (1) only if the player has no existing score.
    ///
    /// Generated command:
    /// ```text
    /// execute unless score <selector> <obj> matches -2147483648.. run scoreboard players set <selector> <obj> 1
    /// ```
    pub fn init_true(&self, selector: impl std::fmt::Display) -> String {
        let obj = self.objective_name();
        format!(
            "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} 1"
        )
    }

    /// Condition shorthand: flag is true. Equivalent to `self.of(selector).is_true()`.
    pub fn when_true(&self, selector: &str) -> Condition {
        self.of(selector).is_true()
    }

    /// Condition shorthand: flag is false (exact 0). Equivalent to `self.of(selector).is_false()`.
    ///
    /// See [`FlagRef::is_false`] for the difference between this and [`unless_true`](Flag::unless_true).
    pub fn when_false(&self, selector: &str) -> Condition {
        self.of(selector).is_false()
    }

    /// Condition shorthand: flag is not true (missing or 0).
    ///
    /// Equivalent to `self.of(selector).is_not_true()`. Prefer this over `when_false`
    /// when you mean "player does not have this yet".
    pub fn unless_true(&self, selector: &str) -> Condition {
        self.of(selector).is_not_true()
    }
}

// ── FlagRef ───────────────────────────────────────────────────────────────────

/// A [`Flag`] bound to a selector — used to build [`Condition`]s.
///
/// Produced by [`Flag::of`].
pub struct FlagRef<'a> {
    objective: &'a str,
    selector: String,
}

impl<'a> FlagRef<'a> {
    fn obj(&self) -> String {
        objective_name(self.objective)
    }

    /// `if score <sel> <obj> matches 1` — flag is `true`.
    pub fn is_true(self) -> Condition {
        let objective = self.obj();
        Condition::Flag {
            selector: self.selector,
            objective,
            value: true,
        }
    }

    /// `if score <sel> <obj> matches 0` — flag is `false`.
    pub fn is_false(self) -> Condition {
        let objective = self.obj();
        Condition::Flag {
            selector: self.selector,
            objective,
            value: false,
        }
    }

    /// Alias for [`is_true`](FlagRef::is_true).
    pub fn is_set(self) -> Condition {
        self.is_true()
    }

    /// Checks `score … matches 0` exactly — the flag score exists **and** equals 0.
    ///
    /// This is **not** equivalent to "the flag was never set". A player whose flag score
    /// has never been touched has *no* score entry, so `is_unset()` returns `false` for
    /// them. Use [`is_not_true`](FlagRef::is_not_true) for "player does not have this yet".
    pub fn is_unset(self) -> Condition {
        self.is_false()
    }

    /// `unless score <sel> <obj> matches 1` — flag is not `true` (missing or non-1).
    ///
    /// Lowers to `Condition::Not(is_true())`, which generates `unless score … matches 1`.
    /// This matches both score = 0 **and** missing scores, unlike `is_false()` which
    /// requires the score to exist and equal exactly 0.
    ///
    /// ```rust,ignore
    /// // Prefer this over is_false() for "player doesn't have this yet" checks:
    /// when(HAS_CELLS.of("@s").is_not_true()).then_all([...]);
    /// unless(HAS_CELLS.of("@s").is_true()).then_all([...]);  // equivalent
    /// ```
    pub fn is_not_true(self) -> Condition {
        Condition::Not(Box::new(self.is_true()))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

    static CASTING: Flag = Flag::new("casting");

    #[test]
    fn define_cmd() {
        assert_eq!(CASTING.define(), "scoreboard objectives add casting dummy");
    }

    #[test]
    fn enable_cmd() {
        assert_eq!(CASTING.enable("@s"), "scoreboard players set @s casting 1");
    }

    #[test]
    fn disable_cmd() {
        assert_eq!(CASTING.disable("@s"), "scoreboard players set @s casting 0");
    }

    #[test]
    fn toggle_cmds() {
        let cmds = CASTING.toggle("@s");
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("matches 0"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("set @s casting 1"), "got: {}", cmds[0]);
        assert!(cmds[1].contains("matches 1.."), "got: {}", cmds[1]);
        assert!(cmds[1].contains("set @s casting 0"), "got: {}", cmds[1]);
    }

    #[test]
    fn condition_is_true() {
        let cond = CASTING.of("@s").is_true();
        match cond {
            Condition::Flag {
                selector,
                objective,
                value: true,
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, "casting");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_is_false() {
        let cond = CASTING.of("@s").is_false();
        match cond {
            Condition::Flag { value: false, .. } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn is_set_is_unset_aliases() {
        let a = CASTING.of("@s").is_set();
        let b = CASTING.of("@s").is_true();
        assert!(matches!(a, Condition::Flag { value: true, .. }));
        assert!(matches!(b, Condition::Flag { value: true, .. }));

        let c = CASTING.of("@s").is_unset();
        assert!(matches!(c, Condition::Flag { value: false, .. }));
    }

    #[test]
    fn is_not_true_generates_unless() {
        let cond = CASTING.of("@s").is_not_true();
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"],
            "is_not_true() must use unless, not if"
        );
    }

    #[test]
    fn is_false_is_exact_zero() {
        let cond = CASTING.of("@s").is_false();
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds,
            vec!["execute if score @s casting matches 0 run say ok"],
            "is_false() requires exactly 0"
        );
    }

    #[test]
    fn set_true_is_enable() {
        assert_eq!(CASTING.set("@s", true), CASTING.enable("@s"));
    }

    #[test]
    fn set_false_is_disable() {
        assert_eq!(CASTING.set("@s", false), CASTING.disable("@s"));
    }

    #[test]
    fn clear_is_disable() {
        assert_eq!(CASTING.clear("@s"), CASTING.disable("@s"));
    }

    #[test]
    fn init_false_uses_unless() {
        let cmd = CASTING.init_false("@s");
        assert!(
            cmd.contains("unless score @s casting matches -2147483648.."),
            "got: {cmd}"
        );
        assert!(cmd.contains("set @s casting 0"), "got: {cmd}");
    }

    #[test]
    fn init_true_uses_unless() {
        let cmd = CASTING.init_true("@s");
        assert!(
            cmd.contains("unless score @s casting matches -2147483648.."),
            "got: {cmd}"
        );
        assert!(cmd.contains("set @s casting 1"), "got: {cmd}");
    }

    #[test]
    fn when_true_shorthand() {
        let a = CASTING.when_true("@s");
        let b = CASTING.of("@s").is_true();
        assert!(matches!(a, Condition::Flag { value: true, .. }));
        assert!(matches!(b, Condition::Flag { value: true, .. }));
    }

    #[test]
    fn when_false_shorthand() {
        let cond = CASTING.when_false("@s");
        assert!(matches!(cond, Condition::Flag { value: false, .. }));
    }

    #[test]
    fn unless_true_shorthand() {
        let cond = CASTING.unless_true("@s");
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn is_not_true_is_distinct_from_is_false() {
        let not_true = CASTING.of("@s").is_not_true();
        assert!(
            matches!(not_true, Condition::Not(_)),
            "is_not_true should wrap in Not"
        );
        let is_false = CASTING.of("@s").is_false();
        assert!(
            matches!(is_false, Condition::Flag { value: false, .. }),
            "is_false should be Flag(false)"
        );
    }
}
