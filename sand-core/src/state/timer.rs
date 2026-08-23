//! Tick-based duration and timer utilities.
//!
//! # API hierarchy (see [#146](https://github.com/ThatOneToast/sand/issues/146))
//!
//! 1. **Typed normal API** — `try_*` methods (e.g. [`Timer::try_start`],
//!    [`Timer::try_active`]) take a typed [`sand_commands::ScoreHolder`]
//!    (an entity selector, `@s`, a literal player name, or a `#`-prefixed
//!    fake player) and validate it before generating any command text.
//!    Prefer these in new code.
//! 2. **Validated compatibility adapter** — [`sand_commands::scoreboard::Objective`]
//!    offers the same validated surface directly against
//!    [`sand_commands::ObjectiveName`]/[`sand_commands::ScoreHolder`] for
//!    callers that don't need `Timer`'s countdown ergonomics.
//! 3. **Raw escape hatch** — the plain (infallible) methods (e.g.
//!    [`Timer::start`], [`Timer::active`]) accept any `impl Display`/`&str`
//!    and interpolate it into command text without validation. They remain
//!    available for compatibility and for advanced/modded selector syntax
//!    Sand cannot yet type-check.
//!
//! Every `try_*` method delegates to its raw counterpart once validation
//! succeeds, so the typed and compatibility paths render byte-identically.

pub use sand_components::Ticks;

use crate::state::flag::{validate_holder, validate_single_holder};
use sand_commands::{CommandResult, ScoreHolder};

// ── Timer ─────────────────────────────────────────────────────────────────────

/// A scoreboard-backed countdown timer.
///
/// A `Timer` counts down from a starting value to zero. It does not generate
/// conditions; use [`Cooldown`](super::cooldown::Cooldown) when you need
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
///     BLINK.tick_all_players(),
/// ];
/// ```
pub struct Timer {
    name: &'static str,
    duration: Ticks,
}

impl Timer {
    /// Create a new timer with the given objective name and duration.
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::new` for the canonical contract."]
    pub const fn new(name: &'static str, duration: Ticks) -> Self {
        Self { name, duration }
    }

    /// Return the actual scoreboard objective name.
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::objective_name` for the canonical contract."]
    pub fn objective_name(&self) -> String {
        super::score::objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy`
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::define` for the canonical contract."]
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Set the timer to the configured duration for `selector`.
    ///
    /// Raw compatibility escape hatch: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`Timer::try_start`] in new code — see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::start` for the canonical contract."]
    pub fn start(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            self.duration.get()
        )
    }

    /// Validated counterpart to [`Timer::start`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before generating the
    /// `scoreboard players set` command, instead of interpolating an
    /// unvalidated `Display` value.
    ///
    /// `scoreboard players set` accepts multiple targets, so multi-entity
    /// selectors and the `*` wildcard are permitted here (unlike
    /// [`Timer::try_tick`], which builds a score *condition*).
    ///
    /// ```
    /// use sand_core::state::{Ticks, Timer};
    /// use sand_commands::ScoreHolder;
    ///
    /// static BLINK: Timer = Timer::new("blink_cd", Ticks::new(100));
    ///
    /// assert_eq!(
    ///     BLINK.try_start(ScoreHolder::self_()).unwrap(),
    ///     "scoreboard players set @s blink_cd 100"
    /// );
    /// assert!(BLINK.try_start(ScoreHolder::fake("bad holder")).is_err());
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_start` for the canonical contract."]
    pub fn try_start(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Timer::try_start")?;
        Ok(self.start(holder.to_string()))
    }

    /// Decrement the timer by 1 tick for one score holder (only if > 0).
    ///
    /// Use [`tick_all_players`](Self::tick_all_players) instead of passing a
    /// multi-player selector such as `@a` here.
    ///
    /// Raw compatibility escape hatch — prefer [`Timer::try_tick`].
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::tick` for the canonical contract."]
    pub fn tick(&self, selector: impl std::fmt::Display) -> String {
        let selector = selector.to_string();
        let obj = self.objective_name();
        format!(
            "execute if score {selector} {obj} matches 1.. run scoreboard players remove {selector} {obj} 1"
        )
    }

    /// Validated counterpart to [`Timer::tick`] — takes a typed
    /// [`sand_commands::ScoreHolder`].
    ///
    /// The generated command is guarded by `execute if score <holder> …`,
    /// which requires exactly one score holder, so the `*` wildcard and
    /// multi-entity selectors are rejected via
    /// [`sand_commands::ScoreHolder::validate_single`] — use
    /// [`Timer::tick_all_players`] for the per-player `@a` form.
    ///
    /// ```
    /// use sand_core::state::{Ticks, Timer};
    /// use sand_commands::ScoreHolder;
    /// use sand_commands::selector::Selector;
    ///
    /// static BLINK: Timer = Timer::new("blink_cd", Ticks::new(100));
    ///
    /// assert_eq!(BLINK.try_tick(ScoreHolder::self_()).unwrap(), BLINK.tick("@s"));
    /// assert!(BLINK.try_tick(ScoreHolder::entity(Selector::all_players())).is_err());
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_tick` for the canonical contract."]
    pub fn try_tick(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Timer::try_tick")?;
        Ok(self.tick(holder.to_string()))
    }

    /// Decrement this timer independently for every online player.
    ///
    /// Takes no score holder — the `@a`/`@s` pair is generated by Sand and is
    /// always valid, so there is no `try_*` counterpart.
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::tick_all_players` for the canonical contract."]
    pub fn tick_all_players(&self) -> String {
        let obj = self.objective_name();
        format!(
            "execute as @a if score @s {obj} matches 1.. run scoreboard players remove @s {obj} 1"
        )
    }

    /// Reset the timer to zero for `selector`.
    ///
    /// Raw compatibility escape hatch — prefer [`Timer::try_reset`].
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::reset` for the canonical contract."]
    pub fn reset(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players set {} {} 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Timer::reset`] — see [`Timer::try_start`].
    /// Multi-target holders are permitted (`scoreboard players set` accepts
    /// multiple targets).
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_reset` for the canonical contract."]
    pub fn try_reset(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_holder(&holder, "Timer::try_reset")?;
        Ok(self.reset(holder.to_string()))
    }

    /// Condition: timer has expired (score == 0).
    ///
    /// Use this to check if the timer has counted down to zero.
    ///
    /// Raw compatibility escape hatch — prefer [`Timer::try_expired`].
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::expired` for the canonical contract."]
    pub fn expired(&self, selector: &str) -> crate::condition::Condition {
        crate::condition::Condition::score(
            selector.to_string(),
            self.objective_name(),
            crate::condition::ScoreRange::Eq(0),
        )
    }

    /// Validated counterpart to [`Timer::expired`] — takes a typed
    /// [`sand_commands::ScoreHolder`], which must resolve to exactly one score
    /// holder because the result lowers to `execute if score <holder> …`.
    ///
    /// ```
    /// use sand_core::execute_when::when;
    /// use sand_core::state::{Ticks, Timer};
    /// use sand_commands::ScoreHolder;
    ///
    /// static BLINK: Timer = Timer::new("blink_cd", Ticks::new(100));
    ///
    /// let cond = BLINK.try_expired(ScoreHolder::self_()).unwrap();
    /// assert_eq!(
    ///     when(cond).then_one("say ok"),
    ///     vec!["execute if score @s blink_cd matches 0 run say ok"]
    /// );
    /// assert!(BLINK.try_expired(ScoreHolder::wildcard()).is_err());
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_expired` for the canonical contract."]
    pub fn try_expired(
        &self,
        holder: impl Into<ScoreHolder>,
    ) -> CommandResult<crate::condition::Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Timer::try_expired")?;
        Ok(self.expired(&holder.to_string()))
    }

    /// Condition: timer is still running (score >= 1).
    ///
    /// Raw compatibility escape hatch — prefer [`Timer::try_active`].
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::active` for the canonical contract."]
    pub fn active(&self, selector: &str) -> crate::condition::Condition {
        crate::condition::Condition::score(
            selector.to_string(),
            self.objective_name(),
            crate::condition::ScoreRange::Gte(1),
        )
    }

    /// Validated counterpart to [`Timer::active`] — see
    /// [`Timer::try_expired`]; the holder must resolve to exactly one score
    /// holder.
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_active` for the canonical contract."]
    pub fn try_active(
        &self,
        holder: impl Into<ScoreHolder>,
    ) -> CommandResult<crate::condition::Condition> {
        let holder = holder.into();
        validate_single_holder(&holder, "Timer::try_active")?;
        Ok(self.active(&holder.to_string()))
    }

    /// Guard clause: return early if the timer is still running (score >= 1).
    ///
    /// Raw compatibility escape hatch — prefer [`Timer::try_guard_active`].
    ///
    /// Produces: `execute if score <selector> <obj> matches 1.. run return 0`
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::guard_active` for the canonical contract."]
    pub fn guard_active(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "execute if score {} {} matches 1.. run return 0",
            selector,
            self.objective_name()
        )
    }

    /// Validated counterpart to [`Timer::guard_active`] — takes a typed
    /// [`sand_commands::ScoreHolder`], which must resolve to exactly one score
    /// holder because the guard lowers to `execute if score <holder> …`.
    ///
    /// ```
    /// use sand_core::state::{Ticks, Timer};
    /// use sand_commands::ScoreHolder;
    ///
    /// static BLINK: Timer = Timer::new("blink_cd", Ticks::new(100));
    ///
    /// assert_eq!(
    ///     BLINK.try_guard_active(ScoreHolder::self_()).unwrap(),
    ///     "execute if score @s blink_cd matches 1.. run return 0"
    /// );
    /// assert!(BLINK.try_guard_active(ScoreHolder::wildcard()).is_err());
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::try_guard_active` for the canonical contract."]
    pub fn try_guard_active(&self, holder: impl Into<ScoreHolder>) -> CommandResult<String> {
        let holder = holder.into();
        validate_single_holder(&holder, "Timer::try_guard_active")?;
        Ok(self.guard_active(holder.to_string()))
    }

    /// Return the configured duration.
    #[doc = "**API Contract:** Run `sand api show sand::state::Timer::duration` for the canonical contract."]
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
        let cmd = BLINK.tick("@s");
        assert!(cmd.contains("matches 1.."), "got: {cmd}");
        assert!(cmd.contains("remove @s blink_cd 1"), "got: {cmd}");
    }

    #[test]
    fn timer_tick_all_players_is_per_player_safe() {
        let command = BLINK.tick_all_players();
        assert_eq!(command, BLINK.tick_all_players());
        assert_eq!(
            command,
            "execute as @a if score @s blink_cd matches 1.. run scoreboard players remove @s blink_cd 1"
        );
        assert!(!command.contains("if score @a"));
        assert!(!command.contains("remove @a"));
        assert_eq!(
            BLINK.tick("@s"),
            "execute if score @s blink_cd matches 1.. run scoreboard players remove @s blink_cd 1"
        );
    }

    #[test]
    fn timer_reset() {
        assert_eq!(BLINK.reset("@s"), "scoreboard players set @s blink_cd 0");
    }

    #[test]
    fn timer_expired_condition() {
        use crate::condition::{ConditionKind, ScoreRange};
        let cond = BLINK.expired("@s");
        match cond.kind() {
            ConditionKind::Score {
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
        use crate::condition::{ConditionKind, ScoreRange};
        let cond = BLINK.active("@s");
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
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
