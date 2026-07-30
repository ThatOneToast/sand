//! Boolean flag variable — a scoreboard objective whose value is 0 or 1.
//!
//! # API hierarchy (see [#146](https://github.com/ThatOneToast/sand/issues/146))
//!
//! 1. **Typed normal API** — `try_*` methods (e.g. [`Flag::try_enable`],
//!    [`Flag::try_of`]) take a typed [`sand_commands::ScoreHolder`]
//!    (an entity selector, `@s`, a literal player name, or a `#`-prefixed
//!    fake player) and validate it before generating any command text.
//!    Prefer these in new code.
//! 2. **Validated compatibility adapter** — [`sand_commands::scoreboard::Objective`]
//!    offers the same validated surface directly against
//!    [`sand_commands::ObjectiveName`]/[`sand_commands::ScoreHolder`] for
//!    callers that don't need `Flag`'s boolean/condition ergonomics.
//! 3. **Raw escape hatch** — the plain (infallible) methods (e.g.
//!    [`Flag::enable`], [`Flag::of`]) accept any `impl Display`/`&str` and
//!    interpolate it into command text without validation. They remain
//!    available for compatibility and for advanced/modded selector syntax
//!    Sand cannot yet type-check.
//!
//! Every `try_*` method delegates to its raw counterpart once validation
//! succeeds, so the typed and compatibility paths render byte-identically.

use crate::condition::Condition;
use crate::state::score::objective_name;

use sand_commands::{CommandError, CommandProfile, CommandResult, ScoreHolder, Validate};

// ── Shared typed-holder validation ────────────────────────────────────────────
//
// These helpers are shared by `Flag`, `Timer`, and `Cooldown` so all three
// state primitives produce identical, stable diagnostics for the same class
// of invalid score holder (see #146).

/// Re-tag a [`CommandError`] with the state-primitive operation that rejected
/// it, keeping the original field/message and recording the low-level helper
/// as context. This makes diagnostic codes stable *and* self-identifying,
/// e.g. `command.flag_try_enable.invalid_holder`.
pub(super) fn contextualize(error: CommandError, helper: &'static str) -> CommandError {
    let original = error.helper;
    CommandError::new(helper, error.field, error.message).with_context(original)
}

/// Validate a score holder used in a *mutation* position
/// (`scoreboard players set/remove …`).
///
/// Multi-target holders (`@a`, `*`) are legal vanilla here, so cardinality is
/// **not** constrained — only holder validity.
pub(super) fn validate_holder(holder: &ScoreHolder, helper: &'static str) -> CommandResult<()> {
    Validate::validate(holder, &CommandProfile::unprofiled())
        .map_err(|error| contextualize(error, helper))
}

/// Validate a score holder used in a position where vanilla requires exactly
/// one holder — `execute if/unless score <holder> <obj> …` and the source half
/// of `scoreboard players operation`.
///
/// Rejects the wildcard (`*`) and multi-entity selectors in addition to the
/// ordinary holder-validity rules.
pub(super) fn validate_single_holder(
    holder: &ScoreHolder,
    helper: &'static str,
) -> CommandResult<()> {
    holder
        .validate_single(&CommandProfile::unprofiled())
        .map_err(|error| contextualize(error, helper))
}

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

    /// `scoreboard players set <selector> <obj> 1` — set flag to `true`.
    ///
    /// Raw compatibility escape hatch: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`Flag::try_enable`] in new code — see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    pub fn enable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 1",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Flag::enable`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before generating the
    /// `scoreboard players set` command, instead of interpolating an
    /// unvalidated `Display` value.
    ///
    /// `scoreboard players set` accepts multiple targets, so multi-entity
    /// selectors and the `*` wildcard are permitted here (unlike
    /// [`Flag::try_of`], which builds a score *condition*).
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// assert_eq!(
    ///     CASTING.try_enable(ScoreHolder::self_()).unwrap(),
    ///     "scoreboard players set @s casting 1"
    /// );
    /// assert!(CASTING.try_enable(ScoreHolder::fake("bad holder")).is_err());
    /// ```
    pub fn try_enable(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Flag::try_enable")?;
        Ok(self.enable(holder.to_string()))
    }

    /// `scoreboard players set <selector> <obj> 0` — set flag to `false`.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_disable`].
    pub fn disable(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Flag::disable`] — see [`Flag::try_enable`].
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// assert_eq!(
    ///     CASTING.try_disable(ScoreHolder::player("Notch")).unwrap(),
    ///     "scoreboard players set Notch casting 0"
    /// );
    /// ```
    pub fn try_disable(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Flag::try_disable")?;
        Ok(self.disable(holder.to_string()))
    }

    /// Toggle the flag: set to `1` if currently `0`, else set to `0`.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_toggle`].
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

    /// Validated counterpart to [`Flag::toggle`] — see [`Flag::try_enable`].
    ///
    /// The generated pair uses `execute if score <holder> …`, which requires
    /// exactly one score holder, so the holder is validated with
    /// [`sand_commands::ScoreHolder::validate_single`]: the `*` wildcard and
    /// multi-entity selectors are rejected.
    pub fn try_toggle(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Vec<String>> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_toggle")?;
        Ok(self.toggle(holder.to_string()))
    }

    /// Bind this flag to a selector to produce a condition builder.
    ///
    /// Raw compatibility escape hatch: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`Flag::try_of`] in new code.
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

    /// Validated counterpart to [`Flag::of`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before producing the
    /// bound [`FlagRef`], instead of interpolating an unvalidated selector
    /// string.
    ///
    /// `execute if/unless score <holder> <obj> …` requires exactly one score
    /// holder, so the `*` wildcard and multi-entity selectors are rejected via
    /// [`sand_commands::ScoreHolder::validate_single`].
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// let cond = CASTING.try_of(ScoreHolder::self_()).unwrap().is_true();
    /// assert!(CASTING.try_of(ScoreHolder::wildcard()).is_err());
    /// ```
    pub fn try_of<'a>(&'a self, holder: impl Into<ScoreHolder>) -> CommandResult<FlagRef<'a>> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_of")?;
        Ok(FlagRef {
            objective: self.name,
            selector: holder.to_string(),
        })
    }

    /// Set the flag to an explicit boolean value.
    ///
    /// Equivalent to `enable` when `true` and `disable` when `false`.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_set`].
    pub fn set(&self, selector: impl std::fmt::Display, value: bool) -> String {
        if value {
            self.enable(selector)
        } else {
            self.disable(selector)
        }
    }

    /// Validated counterpart to [`Flag::set`] — see [`Flag::try_enable`].
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// assert_eq!(
    ///     CASTING.try_set(ScoreHolder::self_(), true).unwrap(),
    ///     CASTING.try_enable(ScoreHolder::self_()).unwrap()
    /// );
    /// ```
    pub fn try_set(&self, holder: impl Into<ScoreHolder>, value: bool) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Flag::try_set")?;
        Ok(self.set(holder.to_string(), value))
    }

    /// Alias for [`disable`](Flag::disable) — sets the flag to `false`.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_clear`].
    pub fn clear(&self, selector: impl std::fmt::Display) -> String {
        self.disable(selector)
    }

    /// Validated counterpart to [`Flag::clear`] — see [`Flag::try_enable`].
    pub fn try_clear(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Flag::try_clear")?;
        Ok(self.clear(holder.to_string()))
    }

    /// Initialize the flag to `false` (0) only if the player has no existing score.
    ///
    /// Useful in join handlers to avoid overwriting state set by another system.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_init_false`].
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

    /// Validated counterpart to [`Flag::init_false`] — see [`Flag::try_enable`].
    ///
    /// The generated command is guarded by `execute unless score <holder> …`,
    /// which requires exactly one score holder, so the `*` wildcard and
    /// multi-entity selectors are rejected via
    /// [`sand_commands::ScoreHolder::validate_single`].
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// assert_eq!(
    ///     CASTING.try_init_false(ScoreHolder::self_()).unwrap(),
    ///     CASTING.init_false("@s")
    /// );
    /// assert!(CASTING.try_init_false(ScoreHolder::wildcard()).is_err());
    /// ```
    pub fn try_init_false(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_init_false")?;
        Ok(self.init_false(holder.to_string()))
    }

    /// Initialize the flag to `true` (1) only if the player has no existing score.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_init_true`].
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

    /// Validated counterpart to [`Flag::init_true`] — see
    /// [`Flag::try_init_false`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_init_true(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_init_true")?;
        Ok(self.init_true(holder.to_string()))
    }

    /// Condition shorthand: flag is true. Equivalent to `self.of(selector).is_true()`.
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_when_true`].
    pub fn when_true(&self, selector: &str) -> Condition {
        self.of(selector).is_true()
    }

    /// Validated counterpart to [`Flag::when_true`] — takes a typed
    /// [`sand_commands::ScoreHolder`], which must resolve to exactly one score
    /// holder because the result lowers to `execute if score <holder> …`.
    ///
    /// ```
    /// use sand_core::state::Flag;
    /// use sand_commands::ScoreHolder;
    ///
    /// static CASTING: Flag = Flag::new("casting");
    ///
    /// let cond = CASTING.try_when_true(ScoreHolder::self_()).unwrap();
    /// assert_eq!(
    ///     cond.execute_commands(false, "say ok"),
    ///     vec!["execute if score @s casting matches 1 run say ok"]
    /// );
    /// ```
    pub fn try_when_true(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_when_true")?;
        Ok(self.when_true(&holder.to_string()))
    }

    /// Condition shorthand: flag is false (exact 0). Equivalent to `self.of(selector).is_false()`.
    ///
    /// See [`FlagRef::is_false`] for the difference between this and [`unless_true`](Flag::unless_true).
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_when_false`].
    pub fn when_false(&self, selector: &str) -> Condition {
        self.of(selector).is_false()
    }

    /// Validated counterpart to [`Flag::when_false`] — see
    /// [`Flag::try_when_true`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_when_false(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_when_false")?;
        Ok(self.when_false(&holder.to_string()))
    }

    /// Condition shorthand: flag is not true (missing or 0).
    ///
    /// Equivalent to `self.of(selector).is_not_true()`. Prefer this over `when_false`
    /// when you mean "player does not have this yet".
    ///
    /// Raw compatibility escape hatch — prefer [`Flag::try_unless_true`].
    pub fn unless_true(&self, selector: &str) -> Condition {
        self.of(selector).is_not_true()
    }

    /// Validated counterpart to [`Flag::unless_true`] — see
    /// [`Flag::try_when_true`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_unless_true(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Flag::try_unless_true")?;
        Ok(self.unless_true(&holder.to_string()))
    }
}

// ── FlagRef ───────────────────────────────────────────────────────────────────

/// A [`Flag`] bound to a selector — used to build [`Condition`]s.
///
/// Produced by [`Flag::try_of`] (typed path — the bound holder is a validated
/// [`sand_commands::ScoreHolder`] that is guaranteed to resolve to exactly one
/// score holder) or by [`Flag::of`] (raw compatibility path — the bound
/// selector is an unvalidated string).
#[derive(Debug, Clone)]
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
