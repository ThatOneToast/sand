//! # Gameplay State — enum phases, transitions, hooks, and per-state ticks
//!
//! A runnable example that puts the typed gameplay-state API surface to work.
//! Demonstrates, in a single small `BossPhase` machine:
//!
//! - [`GameState<S>`] and the [`TypedGameState`] trait
//! - explicit scoreboard discriminants and `with_default_score`
//! - lifecycle-managed load setup via `.register()` + `define_registered_state()`
//! - transitions (current state → next state) using typed conditions
//! - "enter" hooks built from `is_not(...).then_one(...)`
//! - "exit" hooks built from guarding the state being left before writing the
//!   next state
//! - per-state tick logic gated by `TypedExecute::as_players().when(...).run(...)`
//! - tick-cost guidance (constant vs. `when`-guarded execute chains)
//!
//! ## How it works
//!
//! 1. **Load** registers the `boss_phase` objective through the lifecycle
//!    registry and emits the deterministic `scoreboard objectives add` line.
//! 2. **Tick** runs once per game tick. `#[component(Tick)]` starts in server
//!    context, so `boss_tick` first establishes `execute as @a`. Each
//!    `TypedExecute::as_players().when(PHASE.is(V)).run(body)` then lowers to a
//!    single `execute as @a if score @s boss_phase matches <n> run <body>`
//!    command, so the total per-tick cost is one execute per registered state
//!    plus the body of the matching branch.
//! 3. **Transitions** are ordinary `#[function]` entries that read the
//!    current state with `PHASE.of("@s").is(...)` and write the new state
//!    with `PHASE.of("@s").set(...)`. The `phase/enter_*` and `phase/exit_*`
//!    helpers show the recommended way to express the "fired only on the tick
//!    the state actually changed" hook pattern.
//!
//! ## Build
//!
//! ```sh
//! cargo run -p sand -- new boss_phases
//! # paste this file into src/lib.rs
//! cargo run -p sand -- build
//! ```
//!
//! Then in Minecraft, run the transition functions to walk the boss through
//! the phase machine:
//!
//! ```text
//! /function boss_phases:phase/enter_fighting
//! /function boss_phases:phase/enter_enraged
//! /function boss_phases:phase/exit_enraged
//! /function boss_phases:phase/reset_phase
//! /function boss_phases:phase/on_enter_fighting
//! /function boss_phases:phase/on_exit_enraged
//! ```
//!
//! The tick function will print a phase-specific actionbar message each
//! tick while the corresponding phase is active.
//!
//! ## Tick-cost guidance
//!
//! 1. **Constant work** (`#[component(Tick)]` body that does not depend on
//!    state): one `scoreboard players operation / remove` per state variable,
//!    regardless of how many variants you have. Use this for cooldowns,
//!    timers, and regen ticks.
//! 2. **Per-state work** (one
//!    `TypedExecute::as_players().when(PHASE.is(...)).run(body)` per
//!    registered state): one `execute as @a if score @s boss_phase matches <n>
//!    run <body>` line per state per tick. This is what the `boss_tick`
//!    body uses. Cost is `O(N)` per player per tick, where `N` is the
//!    number of *distinct* phase branches.
//! 3. **Per-state *with hooks***: add one extra guarded execute per state to
//!    detect a transition. Cost is `O(2N)` per player per tick. Only do this
//!    for the states that have meaningful enter/exit logic.
//! 4. **Avoid `is(...)` in tick when a `when/then_one` body would do**.
//!    Building a `Condition` and using it inside `when(...)` keeps the
//!    execute-chain output a single `execute if` line.

use sand_core::prelude::*;
use sand_macros::{component, function};

// -- State -----------------------------------------------------------------

/// The boss phase machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossPhase {
    /// No encounter active; the boss is regenerating.
    Idle = 0,
    /// Encounter started; the boss is engaging the player.
    Fighting = 1,
    /// Health dropped below the enrage threshold; abilities mutate.
    Enraged = 2,
}

impl TypedGameState for BossPhase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Fighting),
            2 => Some(Self::Enraged),
            _ => None,
        }
    }
}

/// Single source of truth for the phase objective. Stored in the lifecycle
/// registry; manually building a `scoreboard objectives add boss_phase dummy`
/// line is unnecessary and would duplicate the drained command.
static PHASE: GameState<BossPhase> = GameState::with_default_score("boss_phase", 0);

// -- Lifecycle ------------------------------------------------------------

/// Register the phase objective through the lifecycle registry, then emit
/// the deterministic list of `scoreboard objectives add` commands.
///
/// Calling `.register()` instead of `.define()` keeps the load function
/// in lockstep with any other typed state you enroll later; the
/// `define_registered_state()` helper dedupes and sorts by objective name.
#[component(Load)]
pub fn boss_load() {
    // The `let _ = …` discards the `()` return of `.register()` so the
    // surrounding `#[component(Load)]` macro only sees `IntoCommands`
    // values. `clippy::let_unit_value` is intentionally allowed here —
    // removing the binding would make the expression evaluate to `()`,
    // which is not `IntoCommands`.
    #[allow(clippy::let_unit_value)]
    let _ = PHASE.register();
    define_registered_state();
    PHASE.of("@a").reset();
}

// -- Per-state tick -------------------------------------------------------

/// Per-tick phase work. `#[component(Tick)]` runs in server context, so
/// player-scoped `@s` checks must be wrapped in `execute as @a` first. Each
/// `TypedExecute::as_players().when(...).run(body)` lowers to a single
/// `execute as @a if score @s boss_phase matches <n> run <body>` line, so
/// the total per-tick cost is *one execute per registered state* plus the
/// body of the matching branch.
///
/// If you find yourself adding the same body to several phases, prefer
/// promoting the work to an unconditional step in the tick function and
/// gate only the parts that genuinely differ between states.
#[component(Tick)]
pub fn boss_tick() {
    TypedExecute::as_players().when(PHASE.of("@s").is(BossPhase::Idle)).run(
        Actionbar::show(Selector::self_(), Text::new("[Idle] regenerating").gray()),
    );
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Fighting))
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("[Fighting] engage").red(),
        ));
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Enraged))
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("[Enraged] berserk").dark_red().bold(true),
        ));
}

// -- Transitions ----------------------------------------------------------

/// `Idle → Fighting`. Calling this from any non-Idle state overwrites the
/// previous state with Fighting. If you need a "transition only from X"
/// guard, chain a `when(...)` condition (see `enter_fighting_from_idle`).
#[function("boss_phases:phase/enter_fighting")]
pub fn enter_fighting() {
    PHASE.of("@s").set(BossPhase::Fighting);
}

/// Same as `enter_fighting`, but the `when(...)` guard rejects the call
/// when the player is already Fighting or Enraged. Use this form when the
/// state machine should refuse illegal transitions.
#[function("boss_phases:phase/enter_fighting_from_idle")]
pub fn enter_fighting_from_idle() {
    when(PHASE.of("@s").is(BossPhase::Idle)).then_one(PHASE.of("@s").set(BossPhase::Fighting));
}

/// `→ Enraged`. Reachable from any state because the body is unconditional.
#[function("boss_phases:phase/enter_enraged")]
pub fn enter_enraged() {
    PHASE.of("@s").set(BossPhase::Enraged);
}

/// `Enraged → Fighting` — used by the enrage timeout to de-escalate
/// without resetting the encounter.
#[function("boss_phases:phase/exit_enraged")]
pub fn exit_enraged() {
    PHASE.of("@s").set(BossPhase::Fighting);
}

/// Reset to the declared default (Idle). The state was created with
/// `with_default_score(..., 0)`, so `reset()` writes `0` rather than
/// removing the scoreboard entry.
#[function("boss_phases:phase/reset_phase")]
pub fn reset_phase() {
    PHASE.of("@s").reset();
}

// -- Enter / exit hooks --------------------------------------------------

/// Enter hook for the Fighting phase. Lowered by
/// `when(PHASE.is_not(Fighting)).then_one(body)` so the body fires only
/// when the player is not already in Fighting — a state was set last tick
/// but isn't Fighting this tick.
///
/// This is the recommended "enter hook" pattern. It costs exactly one
/// `execute unless score @s boss_phase matches 1 run …` per state per
/// tick and does not require any extra storage. The example then writes
/// the new state so subsequent ticks see Fighting and the hook no longer
/// fires.
#[function("boss_phases:phase/on_enter_fighting")]
pub fn on_enter_fighting() {
    when(PHASE.of("@s").is_not(BossPhase::Fighting)).then_one(cmd::tellraw(
        Selector::self_(),
        Text::new("Boss has engaged!").red(),
    ));
    PHASE.of("@s").set(BossPhase::Fighting);
}

/// Exit hook for the Enraged phase. Guard on the state being left before
/// running the exit body and writing the next state.
#[function("boss_phases:phase/on_exit_enraged")]
pub fn on_exit_enraged() {
    when(PHASE.of("@s").is(BossPhase::Enraged)).then_one(cmd::tellraw(
        Selector::self_(),
        Text::new("Boss calms down.").green(),
    ));
    when(PHASE.of("@s").is(BossPhase::Enraged)).then_one(PHASE.of("@s").set(BossPhase::Fighting));
}

// -- Tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::dyn_fn_test_lock;
    use sand_core::drain_dyn_fns;

    #[test]
    fn typed_phase_round_trip() {
        for variant in [BossPhase::Idle, BossPhase::Fighting, BossPhase::Enraged] {
            assert_eq!(BossPhase::from_score(variant.to_score()), Some(variant));
        }
        assert_eq!(BossPhase::from_score(99), None);
        assert_eq!(BossPhase::from_score(-1), None);
    }

    #[test]
    fn transition_writes_next_state() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        let cmds = enter_fighting();
        assert!(
            cmds.iter()
                .any(|c| c == "scoreboard players set @s boss_phase 1"),
            "enter_fighting should set the boss_phase score to 1, got {cmds:?}"
        );
        let cmds = enter_enraged();
        assert!(
            cmds.iter()
                .any(|c| c == "scoreboard players set @s boss_phase 2"),
            "enter_enraged should set the boss_phase score to 2, got {cmds:?}"
        );
        let cmds = exit_enraged();
        assert!(
            cmds.iter()
                .any(|c| c == "scoreboard players set @s boss_phase 1"),
            "exit_enraged should set the boss_phase score to 1, got {cmds:?}"
        );
    }

    #[test]
    fn guarded_transition_emits_guarded_execute() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        // `when(PHASE.is(Idle)).then_one(PHASE.set(Fighting))` should
        // produce a single `execute if score @s boss_phase matches 0 run
        // scoreboard players set @s boss_phase 1` line.
        let cmds = enter_fighting_from_idle();
        assert_eq!(cmds.len(), 1, "expected one guarded command, got {cmds:?}");
        assert_eq!(
            cmds[0],
            "execute if score @s boss_phase matches 0 run scoreboard players set @s boss_phase 1"
        );
    }

    #[test]
    fn reset_writes_default_score_not_remove() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        // The phase was declared with `with_default_score(_, 0)`, so
        // reset() must write the configured default — not call `scoreboard
        // players reset`.
        let cmds = reset_phase();
        assert!(
            cmds.iter()
                .any(|c| c == "scoreboard players set @s boss_phase 0"),
            "reset_phase should write the default Idle score, got {cmds:?}"
        );
    }

    #[test]
    fn enter_hook_uses_unless() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        // `is_not(Fighting)` lowers to `unless score … matches 1`.
        let cmds = on_enter_fighting();
        let body = cmds
            .iter()
            .find(|c| c.contains("Boss has engaged"))
            .expect("expected tellraw body in enter hook");
        assert!(
            body.contains("unless score @s boss_phase matches 1"),
            "expected enter hook to use `unless score … matches 1`, got: {body}"
        );
    }

    #[test]
    fn exit_hook_guards_the_state_being_left() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        let cmds = on_exit_enraged();
        let body = cmds
            .iter()
            .find(|c| c.contains("Boss calms"))
            .expect("expected tellraw body in exit hook");
        assert!(
            body.contains("if score @s boss_phase matches 2"),
            "expected exit hook to guard on Enraged, got: {body}"
        );
        assert!(
            cmds.iter()
                .any(|c| c == "execute if score @s boss_phase matches 2 run scoreboard players set @s boss_phase 1"),
            "expected exit hook to guard the transition to Fighting, got {cmds:?}"
        );
    }

    #[test]
    fn per_state_tick_emits_three_guarded_chains() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();
        // `TypedExecute::as_players().when(PHASE.is(X)).run(body)` lowers to
        // one `execute as @a if score @s boss_phase matches <n> run <body>`
        // line.
        // The tick body has three branches, so we expect exactly three
        // such lines.
        let cmds = boss_tick();
        let guarded = cmds
            .iter()
            .filter(|c| c.starts_with("execute as @a if score @s boss_phase matches"))
            .count();
        assert_eq!(
            guarded, 3,
            "expected 3 per-state execute chains, got {cmds:?}"
        );
    }
}
