//! Enum-backed typed gameplay states backed by scoreboard objectives.
//!
//! Provides [`GameState<S>`] — a scoreboard-backed variable where valid values
//! are the variants of a user-defined enum implementing [`TypedGameState`].
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::state::{GameState, TypedGameState};
//!
//! #[derive(Clone, Copy, PartialEq, Eq)]
//! pub enum BossPhase { Idle = 0, Fighting = 1, Enraged = 2 }
//!
//! impl TypedGameState for BossPhase {
//!     fn to_score(self) -> i32 { self as i32 }
//!     fn from_score(n: i32) -> Option<Self> {
//!         match n {
//!             0 => Some(Self::Idle),
//!             1 => Some(Self::Fighting),
//!             2 => Some(Self::Enraged),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! static PHASE: GameState<BossPhase> = GameState::with_default_score("boss_phase", 0);
//!
//! // In your load function:
//! let load_cmd = PHASE.define(); // "scoreboard objectives add boss_phase dummy"
//!
//! // Set state:
//! let set_cmd = PHASE.of("@s").set(BossPhase::Enraged);
//! // "scoreboard players set @s boss_phase 2"
//!
//! // Check state:
//! let cond = PHASE.of("@s").is(BossPhase::Fighting);
//! // Condition: if score @s boss_phase matches 1
//!
//! // Reset to the configured default:
//! let reset_cmd = PHASE.of("@s").reset();
//! // "scoreboard players set @s boss_phase 0"
//!
//! // Negative check:
//! let not_cond = PHASE.of("@s").is_not(BossPhase::Idle);
//! // Condition: unless score @s boss_phase matches 0
//! ```

use std::marker::PhantomData;

use crate::condition::{Condition, ScoreRange};

use super::score::objective_name;

// ── TypedGameState ─────────────────────────────────────────────────────────────

/// Implement this trait on an enum to use it as a typed gameplay state.
///
/// # Example
/// ```rust,ignore
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// pub enum BossPhase { Idle = 0, Fighting = 1, Enraged = 2 }
///
/// impl TypedGameState for BossPhase {
///     fn to_score(self) -> i32 { self as i32 }
///     fn from_score(n: i32) -> Option<Self> {
///         match n { 0 => Some(Self::Idle), 1 => Some(Self::Fighting), 2 => Some(Self::Enraged), _ => None }
///     }
/// }
/// ```
pub trait TypedGameState: Copy + Eq + 'static {
    /// Map this variant to its scoreboard integer representation.
    fn to_score(self) -> i32;

    /// Attempt to construct this variant from a scoreboard integer.
    ///
    /// Returns `None` if `n` does not correspond to any valid variant.
    fn from_score(n: i32) -> Option<Self>;
}

// ── GameState ─────────────────────────────────────────────────────────────────

/// A scoreboard-backed typed gameplay state variable.
///
/// Declare once as a `static` and use throughout your datapack:
///
/// ```rust,ignore
/// static PHASE: GameState<BossPhase> = GameState::new("boss_phase");
/// ```
///
/// # Thread safety
///
/// `GameState<S>` stores only a `&'static str` and a `PhantomData<fn() -> S>`.
/// The function-pointer phantom keeps auto-trait derivation sound — `fn() -> S`
/// is always `Send + Sync` regardless of `S`, which is correct because the
/// struct does not actually store or move a value of type `S`.
pub struct GameState<S: TypedGameState> {
    name: &'static str,
    default_score: Option<i32>,
    _marker: PhantomData<fn() -> S>,
}

impl<S: TypedGameState> GameState<S> {
    /// Declare a new typed state variable with the given objective name.
    ///
    /// Names longer than 16 characters are automatically hashed to a stable
    /// 16-character objective name (see [`GameState::objective_name`]).
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            default_score: None,
            _marker: PhantomData,
        }
    }

    /// Declare a new typed state variable with a reset/default score.
    ///
    /// [`GameStateRef::reset`] restores this score instead of clearing the
    /// scoreboard entry. This keeps user-facing code free of magic scoreboard
    /// integers at call sites while making the stored default explicit at the
    /// declaration.
    ///
    /// ```rust,ignore
    /// static PHASE: GameState<BossPhase> =
    ///     GameState::with_default_score("boss_phase", 0);
    /// ```
    pub const fn with_default_score(name: &'static str, default_score: i32) -> Self {
        Self {
            name,
            default_score: Some(default_score),
            _marker: PhantomData,
        }
    }

    /// Build an automatic export lifecycle descriptor, carrying this typed
    /// state's configured default when present.
    pub const fn lifecycle(&self) -> crate::state::StateLifecycle {
        let lifecycle = crate::state::StateLifecycle::score(self.name);
        match self.default_score {
            Some(default) => lifecycle.default(default),
            None => lifecycle,
        }
    }

    /// Return the actual scoreboard objective name used in commands.
    ///
    /// This is either `name` directly (≤16 chars) or a stable FNV-1a hash (>16 chars).
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// Return the configured default score, if this state has one.
    pub fn default_score(&self) -> Option<i32> {
        self.default_score
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `load` function.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// Enroll this typed state in Sand's global lifecycle registry.
    ///
    /// The objective will be included in the next call to
    /// [`define_registered_state`](crate::state::define_registered_state).
    /// Calling `.register()` multiple times for the same variable is a no-op.
    pub fn register(&self) {
        crate::state::register_load_objective(self.objective_name(), "dummy");
    }

    /// Bind this state to a selector to produce a typed accessor.
    ///
    /// ```rust,ignore
    /// let ref_ = PHASE.of("@s");
    /// let cmd = ref_.set(BossPhase::Enraged);
    /// ```
    pub fn of<'a>(&'a self, selector: &str) -> GameStateRef<'a, S> {
        GameStateRef {
            state: self,
            selector: selector.to_string(),
        }
    }
}

// ── GameStateRef ──────────────────────────────────────────────────────────────

/// A [`GameState`] bound to a selector — provides typed get/set/check helpers.
///
/// Produced by [`GameState::of`].
pub struct GameStateRef<'a, S: TypedGameState> {
    state: &'a GameState<S>,
    selector: String,
}

impl<'a, S: TypedGameState> GameStateRef<'a, S> {
    /// `scoreboard players set <sel> <obj> <variant.to_score()>`
    pub fn set(&self, variant: S) -> String {
        self.set_score(variant.to_score())
    }

    fn set_score(&self, score: i32) -> String {
        format!(
            "scoreboard players set {} {} {}",
            self.selector,
            self.state.objective_name(),
            score
        )
    }

    /// Reset this selector's state.
    ///
    /// For states declared with [`GameState::with_default_score`], this writes
    /// the configured default score. For states declared with [`GameState::new`],
    /// this clears the scoreboard entry with `scoreboard players reset`.
    pub fn reset(&self) -> String {
        match self.state.default_score() {
            Some(default_score) => self.set_score(default_score),
            None => self.clear(),
        }
    }

    /// Clear this selector's scoreboard entry regardless of any default state.
    pub fn clear(&self) -> String {
        format!(
            "scoreboard players reset {} {}",
            self.selector,
            self.state.objective_name()
        )
    }

    /// Condition: state equals the given variant.
    ///
    /// Renders as `if score <sel> <obj> matches <variant.to_score()>`.
    pub fn is(&self, variant: S) -> Condition {
        Condition::Score {
            selector: self.selector.clone(),
            objective: self.state.objective_name(),
            range: ScoreRange::Eq(variant.to_score()),
        }
    }

    /// Condition: state does NOT equal the given variant.
    ///
    /// Renders as `unless score <sel> <obj> matches <variant.to_score()>`.
    pub fn is_not(&self, variant: S) -> Condition {
        !self.is(variant)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // ── BossPhase example ─────────────────────────────────────────────────────

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum BossPhase {
        Idle = 0,
        Fighting = 1,
        Enraged = 2,
    }

    impl TypedGameState for BossPhase {
        fn to_score(self) -> i32 {
            self as i32
        }
        fn from_score(n: i32) -> Option<Self> {
            match n {
                0 => Some(Self::Idle),
                1 => Some(Self::Fighting),
                2 => Some(Self::Enraged),
                _ => None,
            }
        }
    }

    static PHASE: GameState<BossPhase> = GameState::new("boss_phase");

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct LocalOnlyState(*const Cell<i32>);

    impl TypedGameState for LocalOnlyState {
        fn to_score(self) -> i32 {
            0
        }

        fn from_score(_n: i32) -> Option<Self> {
            None
        }
    }

    static LOCAL_ONLY: GameState<LocalOnlyState> = GameState::new("local_only");

    #[test]
    fn define_cmd() {
        assert_eq!(PHASE.define(), "scoreboard objectives add boss_phase dummy");
    }

    #[test]
    fn game_state_new_preserves_auto_traits_for_local_only_state() {
        assert_eq!(
            LOCAL_ONLY.define(),
            "scoreboard objectives add local_only dummy"
        );
    }

    #[test]
    fn set_enraged() {
        let cmd = PHASE.of("@s").set(BossPhase::Enraged);
        assert_eq!(cmd, "scoreboard players set @s boss_phase 2");
    }

    #[test]
    fn reset_without_default_clears_score() {
        let cmd = PHASE.of("@s").reset();
        assert_eq!(cmd, "scoreboard players reset @s boss_phase");
    }

    #[test]
    fn clear_always_removes_score() {
        let cmd = PHASE.of("@s").clear();
        assert_eq!(cmd, "scoreboard players reset @s boss_phase");
    }

    #[test]
    fn default_score_is_none_for_new_state() {
        assert_eq!(PHASE.default_score(), None);
    }

    #[test]
    fn is_fighting_condition() {
        let cond = PHASE.of("@s").is(BossPhase::Fighting);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute if score @s boss_phase matches 1 run say ok"
        );
    }

    #[test]
    fn is_not_idle_condition_uses_unless() {
        let cond = PHASE.of("@s").is_not(BossPhase::Idle);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("unless"),
            "is_not should render as 'unless', got: {}",
            cmds[0]
        );
        assert!(cmds[0].contains("matches 0"), "got: {}", cmds[0]);
    }

    #[test]
    fn round_trip_from_score() {
        let variant = BossPhase::Enraged;
        let back = BossPhase::from_score(variant.to_score());
        assert_eq!(back, Some(BossPhase::Enraged));
    }

    #[test]
    fn from_score_invalid_returns_none() {
        assert_eq!(BossPhase::from_score(99), None);
        assert_eq!(BossPhase::from_score(-1), None);
    }

    #[test]
    fn long_name_hashed_to_16_chars() {
        // Name > 16 chars should be hashed deterministically to ≤ 16 chars.
        static LONG_STATE: GameState<BossPhase> =
            GameState::new("this_is_a_very_long_state_name_exceeding_limit");
        let name = LONG_STATE.objective_name();
        assert!(
            name.len() <= 16,
            "expected ≤16 chars, got {} chars: {name}",
            name.len()
        );
        // Deterministic — calling again produces the same value.
        assert_eq!(LONG_STATE.objective_name(), name);
        // Hashed names start with 's' (same convention as ScoreVar).
        assert!(
            name.starts_with('s'),
            "hashed name should start with 's', got: {name}"
        );
    }

    // ── MenuState example (second enum showing reusability) ───────────────────

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MenuState {
        MainMenu = 0,
        Playing = 1,
        Paused = 2,
    }

    impl TypedGameState for MenuState {
        fn to_score(self) -> i32 {
            self as i32
        }
        fn from_score(n: i32) -> Option<Self> {
            match n {
                0 => Some(Self::MainMenu),
                1 => Some(Self::Playing),
                2 => Some(Self::Paused),
                _ => None,
            }
        }
    }

    static MENU: GameState<MenuState> = GameState::new("menu_state");
    static MENU_WITH_DEFAULT: GameState<MenuState> =
        GameState::with_default_score("menu_default", 0);

    #[test]
    fn menu_state_define() {
        assert_eq!(MENU.define(), "scoreboard objectives add menu_state dummy");
    }

    #[test]
    fn menu_state_set_playing() {
        let cmd = MENU.of("@s").set(MenuState::Playing);
        assert_eq!(cmd, "scoreboard players set @s menu_state 1");
    }

    #[test]
    fn with_default_score_records_default_score() {
        assert_eq!(MENU_WITH_DEFAULT.default_score(), Some(0));
    }

    #[test]
    fn reset_with_default_sets_default_variant() {
        let cmd = MENU_WITH_DEFAULT.of("@s").reset();
        assert_eq!(cmd, "scoreboard players set @s menu_default 0");
    }

    #[test]
    fn clear_with_default_still_removes_score() {
        let cmd = MENU_WITH_DEFAULT.of("@s").clear();
        assert_eq!(cmd, "scoreboard players reset @s menu_default");
    }

    #[test]
    fn menu_state_is_paused() {
        let cond = MENU.of("@s").is(MenuState::Paused);
        let cmds = cond.execute_commands(false, "say paused");
        assert_eq!(
            cmds[0],
            "execute if score @s menu_state matches 2 run say paused"
        );
    }

    #[test]
    fn menu_state_round_trip() {
        for variant in [MenuState::MainMenu, MenuState::Playing, MenuState::Paused] {
            assert_eq!(MenuState::from_score(variant.to_score()), Some(variant));
        }
    }
}
