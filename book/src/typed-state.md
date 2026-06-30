# Typed State

Use typed state wrappers instead of handwritten scoreboard commands.

```rust
use sand_core::prelude::*;
use sand_macros::component;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load_state() {
    MANA.define();
    CASTING.define();
    DASH.define();
    MANA.set(Selector::all_players(), 100);
    CASTING.disable(Selector::all_players());
}

#[component(Tick)]
pub fn tick_state() {
    DASH.tick(Selector::all_players());
}
```

State wrappers also produce typed conditions for execute chains.

## Enum-backed gameplay states

Use `GameState<S>` when a scoreboard value represents a finite gameplay phase
instead of a free-form number. Each variant maps to a fixed `i32` scoreboard
discriminant; `TypedGameState` carries the round-trip conversion both ways.

```rust,ignore
use sand_core::prelude::*;
use sand_macros::component;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BossPhase {
    Idle = 0,
    Fighting = 1,
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

static PHASE: GameState<BossPhase> =
    GameState::with_default_score("boss_phase", 0);

#[component(Load)]
pub fn phase_load() {
    let _ = PHASE.register();
    define_registered_state();
    PHASE.of("@a").reset();
}
```

`with_default_score` makes `reset()` write the declared default score. A state
created with `GameState::new` has no default, so `reset()` clears the scoreboard
entry. Use explicit discriminants for persistent state; reordering or renumbering
variants changes the meaning of existing scoreboard values. Object discriminants
(`= 0`, `= 1`, …) are the on-disk format.

## Transitions

A transition is "the current state is `X`, write `Y`". Read the current state
with `PHASE.of("@s").is(X)`, write the next state with
`PHASE.of("@s").set(Y)`. The recommended pattern pairs a `when(...)` guard
with a `then_one(...)` body so the transition runs only when the source
state matches.

```rust,ignore
use sand_core::prelude::*;
use sand_macros::function;

#[function("example:phase/enter_fighting")]
pub fn enter_fighting() {
    PHASE.of("@s").set(BossPhase::Fighting);
}

#[function("example:phase/enter_fighting_from_idle")]
pub fn enter_fighting_from_idle() {
    when(PHASE.of("@s").is(BossPhase::Idle))
        .then_one(PHASE.of("@s").set(BossPhase::Fighting));
}
```

The guarded form lowers to a single `execute if score @s boss_phase matches 0
run scoreboard players set @s boss_phase 1` line. The unguarded form is just
`scoreboard players set @s boss_phase 1` and is what you want when the call
site already knows the source state.

## Enter and exit hooks

Enter and exit hooks are "fired only on the tick the state actually changed"
bodies. Build them by combining `is_not(target)` with the transition write:

```rust,ignore
use sand_core::prelude::*;
use sand_macros::function;

#[function("example:phase/on_enter_fighting")]
pub fn on_enter_fighting() {
    when(PHASE.of("@s").is_not(BossPhase::Fighting))
        .then_one(cmd::tellraw(
            Selector::self_(),
            Text::new("Boss has engaged!").red(),
        ));
    PHASE.of("@s").set(BossPhase::Fighting);
}
```

The `when(PHASE.is_not(Fighting))` clause lowers to
`execute unless score @s boss_phase matches 1 run …`, so the body fires only
when the player was *not* in Fighting the previous tick. The trailing
`set(Fighting)` then commits the transition so subsequent ticks see the new
state and the hook no longer fires.

Exit hooks guard on the state being left so the exit body and the transition
only run while the player is still in that state:

```rust,ignore
#[function("example:phase/on_exit_enraged")]
pub fn on_exit_enraged() {
    when(PHASE.of("@s").is(BossPhase::Enraged))
        .then_one(cmd::tellraw(
            Selector::self_(),
            Text::new("Boss calms down.").green(),
        ));
    when(PHASE.of("@s").is(BossPhase::Enraged))
        .then_one(PHASE.of("@s").set(BossPhase::Fighting));
}
```

The cost is one extra guarded execute per state per tick — only add it to
states that have meaningful enter/exit logic. Do not add it to every state;
see the tick-cost guidance below.

## Per-state tick

Run different logic in each state by gating the body with
`TypedExecute::as_players().when(PHASE.is(V)).run(body)`. `#[component(Tick)]`
starts in server context, so player-scoped `@s` checks need an explicit player
executor. The body emits one
`execute as @a if score @s boss_phase matches <n> run <body>` per state per
tick:

```rust,ignore
use sand_core::prelude::*;
use sand_macros::component;

#[component(Tick)]
pub fn boss_tick() {
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Idle))
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("[Idle] regenerating").gray(),
        ));
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
```

If several states share the same body, promote the work to an unconditional
step in the tick function and gate only the parts that genuinely differ
between states. Per-state work scales `O(N)` per player per tick, where `N`
is the number of distinct branches.

## Lifecycle notes

`.register()` enrolls a typed state variable in Sand's global lifecycle
registry. `define_registered_state()` then drains the registry and emits a
sorted list of `scoreboard objectives add …` commands. This is the
lifecycle foundation introduced for #47.

```rust,ignore
#[component(Load)]
pub fn phase_load() {
    let _ = PHASE.register();
    let _ = MANA.register();
    let _ = DASH.register();
    define_registered_state();
}
```

Notes:

- The registry is deduplicated among `.register()` calls. Calling
  `.register()` twice for the same variable is a no-op.
- `.define()` returns a single command string and is independent of the
  registry. Do not mix a manual `.define()` command with the
  `define_registered_state()` drain in the same load function — the
  deduplication only applies to `.register()` calls.
- Output is sorted by objective name for determinism. The registry is
  drained once per `define_registered_state()` call; subsequent calls
  return an empty `Vec` until new state is registered.

## Transition backend

The transition backend is a thin layer over Minecraft's scoreboard: each
`GameState<S>` owns a single scoreboard objective (named with the configured
prefix, or its stable FNV-1a hash if longer than 16 characters), and each
variant of `S` is a discriminant in that objective. The lowerings are:

| Sand call                  | Generated command                                                |
|----------------------------|------------------------------------------------------------------|
| `PHASE.define()`           | `scoreboard objectives add boss_phase dummy`                     |
| `PHASE.register()`         | (registry side effect; no command emitted directly)             |
| `define_registered_state()`| sorted `scoreboard objectives add …` lines for every registered  |
| `PHASE.of(sel).set(V)`     | `scoreboard players set <sel> boss_phase <V.to_score()>`         |
| `PHASE.of(sel).reset()`    | writes the configured default, or `scoreboard players reset`     |
| `PHASE.of(sel).clear()`    | `scoreboard players reset <sel> boss_phase`                      |
| `PHASE.of(sel).is(V)`      | `execute if score <sel> boss_phase matches <V.to_score()>`       |
| `PHASE.of(sel).is_not(V)`  | `execute unless score <sel> boss_phase matches <V.to_score()>`   |

Hooks and transitions compose by chaining these primitives. The
`is(target)` + `set(target)` pair is the "edge detector" used by
`on_enter_*`/`on_exit_*` helpers; `is_not(target)` + `set(target)` is the
"fire on the boundary tick" pair.

## Tick-cost guidance

1. **Constant work** (`#[component(Tick)]` body that does not depend on
   state): one `scoreboard players operation / remove` per state variable,
   regardless of how many variants you have. Use this for cooldowns,
   timers, and regen ticks.

2. **Per-state work** (one `when(PHASE.is(...)).then_one(body)` per
   registered state): one `execute if score @s boss_phase matches <n> run
   <body>` line per state per tick. This is what `boss_tick` uses above.
   Cost is `O(N)` per player per tick, where `N` is the number of distinct
   phase branches.

3. **Per-state with hooks**: add one extra `unless` clause per state to
   detect a transition. Cost is `O(2N)` per player per tick. Only do this
   for states that have meaningful enter/exit logic.

4. **Avoid `is(...)` in tick when a `when/then_one` body would do**.
   Building a `Condition` and using it inside `when(...)` keeps the
   execute-chain output a single `execute if` line. Calling
   `condition.execute_commands(false, body)` works but emits a
   `Vec<String>` you have to splice back into the tick list manually.

5. **Prefer `GameState::with_default_score` over `GameState::new`** when
   the state has a known reset value. `reset()` then writes the default
   score, which is cheaper and more predictable than relying on
   Minecraft's missing-score behavior.

## Runnable example

The [`examples/gameplay_state.rs`](https://github.com/ThatOneToast/sand/blob/main/examples/gameplay_state.rs)
file is a runnable, compile-tested example pack that wires every pattern in
this page into a single small `BossPhase` machine: lifecycle registration,
transitions, guarded transitions, enter/exit hooks, per-state ticks, and the
`with_default_score` reset. The example is also mirrored under
`sand-example/src/gameplay_state_example.rs` so the workspace test suite
asserts the exact command output.

```sh
cargo run -p sand -- new boss_phases
# paste examples/gameplay_state.rs into src/lib.rs
cargo run -p sand -- build
```

## Score comparisons and integer math

Score entries can be compared and combined without raw command strings:

```rust,ignore
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static COST: ScoreVar<i32> = ScoreVar::new("mana_cost");

when(MANA.of("@s").gte_score(COST.of("@s"))).then_all([
    MANA.of("@s").sub_score(COST.of("@s")),
    cmd::call(cast_spell),
]);
```

Use `ScoreConst::new("tick_rate", 20).ref_()` when an operation needs a
constant right-hand score. Sand creates its fake-player entry in
`__sand_score_init` on load. Scoreboards are integer-only; division and modulo
by zero retain vanilla's runtime behavior.

## Score math helpers

Use the higher-level score math helpers for common gameplay formulas:

```rust,ignore
static HEALTH: ScoreVar<i32> = ScoreVar::new("health");
static MAX_HEALTH: ScoreVar<i32> = ScoreVar::new("max_health");
static HEALTH_PERCENT: ScoreVar<i32> = ScoreVar::new("health_pct");
static DAMAGE: ScoreVar<i32> = ScoreVar::new("damage");
static SCALED_DAMAGE: ScoreVar<i32> = ScoreVar::new("scaled_damage");
static MIN_MANA: ScoreVar<i32> = ScoreVar::new("min_mana");
static MAX_MANA: ScoreVar<i32> = ScoreVar::new("max_mana");

HEALTH_PERCENT
    .of("@s")
    .set_ratio(HEALTH.of("@s"), MAX_HEALTH.of("@s"), 100);

SCALED_DAMAGE.of("@s").set_percent(DAMAGE.of("@s"), 150);
MANA.of("@s").saturating_sub(10, 0, 100);
MANA.of("@s").clamp_score(MIN_MANA.of("@s"), MAX_MANA.of("@s"));
```

`set_percent`, `scale_percent`, and `set_ratio` register deterministic fake-player
constants and emit plain scoreboard operations. `safe_divide` emits an explicit
zero-divisor branch with a fallback value. All helpers use vanilla integer
scoreboard math, so percentages and ratios truncate toward zero.
