//! High-level scoreboard-based ability cooldown.
//!
//! This is the typed state API counterpart to [`crate::cmd::Cooldown`].
//! The two types have the same conceptual purpose but different constructors:
//! - `state::Cooldown` takes `(&'static str, Ticks)` and hides the `Objective` plumbing.
//! - `cmd::Cooldown` takes `(&'static Objective, u32)` for lower-level control.
//!
//! # API hierarchy (see [#146](https://github.com/ThatOneToast/sand/issues/146))
//!
//! 1. **Typed normal API** — `try_*` methods (e.g. [`Cooldown::try_start`],
//!    [`Cooldown::try_ready`]) take a typed [`sand_commands::ScoreHolder`]
//!    (an entity selector, `@s`, a literal player name, or a `#`-prefixed
//!    fake player) and validate it before generating any command text.
//!    Prefer these in new code.
//! 2. **Validated compatibility adapter** — [`sand_commands::scoreboard::Objective`]
//!    offers the same validated surface directly against
//!    [`sand_commands::ObjectiveName`]/[`sand_commands::ScoreHolder`] for
//!    callers that don't need `Cooldown`'s ready/active ergonomics.
//! 3. **Raw escape hatch** — the plain (infallible) methods (e.g.
//!    [`Cooldown::start`], [`Cooldown::ready`]) accept any `impl Display`/`&str`
//!    and interpolate it into command text without validation. They remain
//!    available for compatibility and for advanced/modded selector syntax
//!    Sand cannot yet type-check.
//!
//! Every `try_*` method delegates to its raw counterpart once validation
//! succeeds, so the typed and compatibility paths render byte-identically.

use crate::condition::Condition;
use crate::state::flag::{validate_holder, validate_single_holder};
use crate::state::score::objective_name;
use crate::state::timer::Ticks;

use sand_commands::{CommandResult, ScoreHolder};

// ── Cooldown ──────────────────────────────────────────────────────────────────

/// A scoreboard-backed ability cooldown timer with typed condition support.
///
/// ```rust,ignore
/// use sand_core::state::{Cooldown, Ticks};
///
/// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
///
/// let load_cmds = vec![DASH.define()];
/// let tick_cmds = vec![DASH.tick_all_players()];
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
    ///
    /// Raw compatibility escape hatch: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`Cooldown::try_start`] in new code — see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    pub fn start(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            self.duration.get()
        )
    }

    /// Validated counterpart to [`Cooldown::start`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before generating the
    /// `scoreboard players set` command, instead of interpolating an
    /// unvalidated `Display` value.
    ///
    /// `scoreboard players set` accepts multiple targets, so multi-entity
    /// selectors and the `*` wildcard are permitted here (unlike
    /// [`Cooldown::try_guard`], which builds a score *condition*).
    ///
    /// ```
    /// use sand_core::state::{Cooldown, Ticks};
    /// use sand_commands::ScoreHolder;
    ///
    /// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    ///
    /// assert_eq!(
    ///     DASH.try_start(ScoreHolder::self_()).unwrap(),
    ///     "scoreboard players set @s dash 60"
    /// );
    /// assert!(DASH.try_start(ScoreHolder::fake("bad holder")).is_err());
    /// ```
    pub fn try_start(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Cooldown::try_start")?;
        Ok(self.start(holder.to_string()))
    }

    /// Reset the cooldown to 0 for `selector` (immediately ready).
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_stop`].
    pub fn stop(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Cooldown::stop`] — see
    /// [`Cooldown::try_start`]. Multi-target holders are permitted.
    pub fn try_stop(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Cooldown::try_stop")?;
        Ok(self.stop(holder.to_string()))
    }

    /// Decrement the cooldown by 1 tick for one score holder (only if score > 0).
    ///
    /// Use [`tick_all_players`](Self::tick_all_players) instead of passing a
    /// multi-player selector such as `@a` here.
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_tick`].
    pub fn tick(&self, selector: impl std::fmt::Display) -> String {
        let selector = selector.to_string();
        let obj = self.objective_name();
        format!(
            "execute if score {selector} {obj} matches 1.. run scoreboard players remove {selector} {obj} 1"
        )
    }

    /// Validated counterpart to [`Cooldown::tick`] — takes a typed
    /// [`sand_commands::ScoreHolder`].
    ///
    /// The generated command is guarded by `execute if score <holder> …`,
    /// which requires exactly one score holder, so the `*` wildcard and
    /// multi-entity selectors are rejected via
    /// [`sand_commands::ScoreHolder::validate_single`] — use
    /// [`Cooldown::tick_all_players`] for the per-player `@a` form.
    ///
    /// ```
    /// use sand_core::state::{Cooldown, Ticks};
    /// use sand_commands::ScoreHolder;
    /// use sand_commands::selector::Selector;
    ///
    /// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    ///
    /// assert_eq!(DASH.try_tick(ScoreHolder::self_()).unwrap(), DASH.tick("@s"));
    /// assert!(DASH.try_tick(ScoreHolder::entity(Selector::all_players())).is_err());
    /// ```
    pub fn try_tick(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_tick")?;
        Ok(self.tick(holder.to_string()))
    }

    /// Guard clause: return early if the cooldown is still active (score > 0).
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_guard`].
    ///
    /// Produces: `execute if score <selector> <obj> matches 1.. run return 0`
    pub fn guard(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Cooldown::guard`] — takes a typed
    /// [`sand_commands::ScoreHolder`], which must resolve to exactly one score
    /// holder because the guard lowers to `execute if score <holder> …`.
    ///
    /// ```
    /// use sand_core::state::{Cooldown, Ticks};
    /// use sand_commands::ScoreHolder;
    ///
    /// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    ///
    /// assert_eq!(
    ///     DASH.try_guard(ScoreHolder::self_()).unwrap(),
    ///     "execute if score @s dash matches 1.. run return 0"
    /// );
    /// assert!(DASH.try_guard(ScoreHolder::wildcard()).is_err());
    /// ```
    pub fn try_guard(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_guard")?;
        Ok(self.guard(holder.to_string()))
    }

    /// Condition: cooldown is ready (`if score <sel> <obj> matches 0`).
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_ready`].
    pub fn ready(&self, selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            self.objective_name(),
            crate::condition::ScoreRange::Eq(0),
        )
    }

    /// Validated counterpart to [`Cooldown::ready`] — takes a typed
    /// [`sand_commands::ScoreHolder`], which must resolve to exactly one score
    /// holder because the result lowers to `execute if score <holder> …`.
    ///
    /// ```
    /// use sand_core::execute_when::when;
    /// use sand_core::state::{Cooldown, Ticks};
    /// use sand_commands::ScoreHolder;
    ///
    /// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    ///
    /// let cond = DASH.try_ready(ScoreHolder::self_()).unwrap();
    /// assert_eq!(
    ///     when(cond).then_one("say ok"),
    ///     vec!["execute if score @s dash matches 0 run say ok"]
    /// );
    /// assert!(DASH.try_ready(ScoreHolder::wildcard()).is_err());
    /// ```
    pub fn try_ready(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_ready")?;
        Ok(self.ready(&holder.to_string()))
    }

    /// Condition: cooldown is active (`if score <sel> <obj> matches 1..`).
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_active`].
    pub fn active(&self, selector: &str) -> Condition {
        Condition::score(
            selector.to_string(),
            self.objective_name(),
            crate::condition::ScoreRange::Gte(1),
        )
    }

    /// Validated counterpart to [`Cooldown::active`] — see
    /// [`Cooldown::try_ready`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_active(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_active")?;
        Ok(self.active(&holder.to_string()))
    }

    /// Alias for [`ready`](Cooldown::ready) — more intuitive name when thinking about
    /// whether the timer has "expired" (counted down to zero).
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_expired`].
    pub fn expired(&self, selector: &str) -> Condition {
        self.ready(selector)
    }

    /// Validated counterpart to [`Cooldown::expired`] — see
    /// [`Cooldown::try_ready`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_expired(&self, holder: impl Into<ScoreHolder>) -> CommandResult<Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_expired")?;
        Ok(self.expired(&holder.to_string()))
    }

    /// Alias for [`start`](Cooldown::start) — emphasizes the selector context.
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_start_for`].
    pub fn start_for(&self, selector: impl std::fmt::Display) -> String {
        self.start(selector)
    }

    /// Validated counterpart to [`Cooldown::start_for`] — see
    /// [`Cooldown::try_start`]. Multi-target holders are permitted.
    pub fn try_start_for(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Cooldown::try_start_for")?;
        Ok(self.start_for(holder.to_string()))
    }

    /// Alias for [`stop`](Cooldown::stop) — resets the cooldown to zero immediately.
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_reset_for`].
    pub fn reset_for(&self, selector: impl std::fmt::Display) -> String {
        self.stop(selector)
    }

    /// Validated counterpart to [`Cooldown::reset_for`] — see
    /// [`Cooldown::try_start`]. Multi-target holders are permitted.
    pub fn try_reset_for(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Cooldown::try_reset_for")?;
        Ok(self.reset_for(holder.to_string()))
    }

    /// Tick the cooldown for all players (`@a`).
    ///
    /// Convenience for placing in a `#[component(Tick)]` function. Takes no
    /// score holder — the `@a`/`@s` pair is generated by Sand and is always
    /// valid, so there is no `try_*` counterpart.
    pub fn tick_all_players(&self) -> String {
        let obj = self.objective_name();
        format!(
            "execute as @a if score @s {obj} matches 1.. run scoreboard players remove @s {obj} 1"
        )
    }

    /// Alias for [`guard`](Cooldown::guard) — guards if the cooldown is NOT ready.
    ///
    /// Returns early (`return 0`) if the cooldown is still active.
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_guard_active`].
    pub fn guard_active(&self, selector: impl std::fmt::Display) -> String {
        self.guard(selector)
    }

    /// Validated counterpart to [`Cooldown::guard_active`] — see
    /// [`Cooldown::try_guard`]; the holder must resolve to exactly one score
    /// holder.
    pub fn try_guard_active(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_guard_active")?;
        Ok(self.guard_active(holder.to_string()))
    }

    /// Guard clause: return early if the cooldown IS ready (score == 0).
    ///
    /// Useful when you only want to run logic while the cooldown is active.
    ///
    /// Raw compatibility escape hatch — prefer [`Cooldown::try_guard_ready`].
    ///
    /// Produces: `execute if score <selector> <obj> matches 0 run return 0`
    pub fn guard_ready(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 0 run return 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Cooldown::guard_ready`] — see
    /// [`Cooldown::try_guard`]; the holder must resolve to exactly one score
    /// holder.
    ///
    /// ```
    /// use sand_core::state::{Cooldown, Ticks};
    /// use sand_commands::ScoreHolder;
    ///
    /// static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
    ///
    /// assert_eq!(
    ///     DASH.try_guard_ready(ScoreHolder::self_()).unwrap(),
    ///     "execute if score @s dash matches 0 run return 0"
    /// );
    /// ```
    pub fn try_guard_ready(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Cooldown::try_guard_ready")?;
        Ok(self.guard_ready(holder.to_string()))
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
    use crate::condition::{ConditionKind, ScoreRange};

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
        let cmd = DASH.tick("@s");
        assert!(cmd.contains("matches 1.."), "got: {cmd}");
        assert!(cmd.contains("remove @s dash 1"), "got: {cmd}");
    }

    #[test]
    fn tick_all_players_is_per_player_safe() {
        let command = DASH.tick_all_players();
        assert_eq!(command, DASH.tick_all_players());
        assert_eq!(
            command,
            "execute as @a if score @s dash matches 1.. run scoreboard players remove @s dash 1"
        );
        assert!(!command.contains("if score @a"));
        assert!(!command.contains("remove @a"));
    }

    #[test]
    fn guard_cmd() {
        let cmd = DASH.guard("@s");
        assert_eq!(cmd, "execute if score @s dash matches 1.. run return 0");
    }

    #[test]
    fn ready_condition() {
        let cond = DASH.ready("@s");
        match cond.kind() {
            ConditionKind::Score {
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
        match cond.kind() {
            ConditionKind::Score {
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
            a.kind(),
            ConditionKind::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
        assert!(matches!(
            b.kind(),
            ConditionKind::Score {
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
        assert_eq!(
            DASH.tick_all_players(),
            "execute as @a if score @s dash matches 1.. run scoreboard players remove @s dash 1"
        );
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
