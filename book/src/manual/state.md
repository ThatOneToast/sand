# State

`ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, and `Ticks` model scoreboard-backed state.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
MANA.define(); MANA.add("@s", 10); DASH.start(Selector::self_());
```

Define objectives in load, tick timers/cooldowns on the appropriate target set, and query them with typed conditions. Objective names are global Minecraft names; choose collision-resistant names.

## Enum-backed gameplay phases

Use `GameState<S>` for finite enum-backed gameplay phases:

```rust,ignore
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase { Idle = 0, Casting = 1 }

impl TypedGameState for Phase {
    fn to_score(self) -> i32 { self as i32 }
    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Casting),
            _ => None,
        }
    }
}

static PHASE: GameState<Phase> = GameState::with_default_score("phase", 0);

PHASE.of("@s").set(Phase::Casting);
PHASE.of("@s").reset(); // writes Idle's score
```

Explicit discriminants are the storage format. Treat renumbering variants as a
state migration for live datapacks. `GameState::new` has no default, so
`reset()` clears the score entry; `GameState::with_default_score` resets to the
declared default score.

## Lifecycle registration

Call `.register()` on a typed state variable to enroll it in Sand's global
lifecycle registry, then call `define_registered_state()` once in your load
function to drain the registry. The drain emits a sorted list of
`scoreboard objectives add …` commands and is deduplicated among
`.register()` calls.

```rust,ignore
use sand_core::prelude::*;
use sand_macros::component;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static ALIVE: Flag = Flag::new("alive");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load_state() {
    let _ = MANA.register();
    let _ = ALIVE.register();
    let _ = DASH.register();
    define_registered_state();
    MANA.set(Selector::all_players(), 100);
    ALIVE.disable(Selector::all_players());
}
```

`.define()` returns a command string and is independent of the registry.
Mixing a manual `.define()` command and the registry drain for the same
objective in the same load function will duplicate output. Choose one
approach per objective.

## Transitions, hooks, and per-state tick

A *transition* is `when(PHASE.is(X)).then_one(PHASE.set(Y))`. The guarded
form refuses to write the new state unless the current state matches `X`,
so illegal transitions become no-ops.

An *enter hook* is `when(PHASE.is_not(target)).then_one(body)` followed by
`PHASE.set(target)`. The `unless` clause fires the body only on the tick the
state actually changes, and the trailing `set` commits the transition so
subsequent ticks see the new state and the hook no longer fires. *Exit
hooks* instead guard on the state being left, then guard the transition write
with that same `is(previous)` condition.

A *per-state tick* is one `TypedExecute::as_players().when(PHASE.is(V))
.run(body)` per state. `#[component(Tick)]` starts in server context, so this
establishes the player executor before using `@s`. The per-tick cost is one
`execute as @a if score @s phase matches <n> run <body>` per player per
branch. Add a second guarded execute only for states that need hooks; the rest
stay `O(N)`.

See [Typed State](../typed-state.md) for the full discussion, the
[transition backend table](../typed-state.md#transition-backend), the
[tick-cost guidance](../typed-state.md#tick-cost-guidance), and the runnable
[`gameplay_state.rs`](https://github.com/ThatOneToast/sand/blob/main/examples/gameplay_state.rs)
example.

## Score helpers

Score helpers cover common integer formulas without raw scoreboard strings:

```rust
static HEALTH: ScoreVar<i32> = ScoreVar::new("health");
static MAX_HEALTH: ScoreVar<i32> = ScoreVar::new("max_health");
static HEALTH_PERCENT: ScoreVar<i32> = ScoreVar::new("health_pct");

HEALTH_PERCENT.of("@s").set_ratio(HEALTH.of("@s"), MAX_HEALTH.of("@s"), 100);
MANA.of("@s").scale_percent(150);
MANA.of("@s").saturating_add(5, 0, 100);
```

Scoreboard math is integer-only. Ratios and percentages truncate toward zero; use `safe_divide` when the divisor may be zero.
