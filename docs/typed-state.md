# Typed State

Use typed state wrappers instead of hand-writing scoreboard plumbing.

```rust
use sand_core::prelude::*;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static TIMER: Timer = Timer::new("blink", Ticks::seconds(5));
```

State APIs return command builders or command strings that can be used directly
inside `#[function]`, `#[component(Load)]`, and `#[component(Tick)]` bodies.

```rust
#[component(Tick)]
pub fn tick_state() {
    MANA.set(Selector::all_players(), 100);
    MANA.add(Selector::all_players(), 1);
    CASTING.disable(Selector::all_players());
    DASH.tick(Selector::all_players());
    TIMER.tick(Selector::all_players());
}
```

## Enum-backed gameplay states

Enum-backed gameplay states use explicit scoreboard discriminants while
keeping callers on named Rust variants. The trait carries the round-trip
conversion both ways, and the lifecycle registration is one call away from
`define_registered_state()`.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

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
pub fn load_phase() {
    let _ = PHASE.register();
    define_registered_state();
    PHASE.of("@a").reset();
}

#[function("example:phase/enrage_boss")]
pub fn enrage_boss() {
    PHASE.of("@s").set(BossPhase::Enraged);
}

#[function("example:phase/reset_phase")]
pub fn reset_phase() {
    PHASE.of("@s").reset();
}
```

Use explicit discriminants for persistent player/world state. Reordering or
renumbering variants changes the meaning of stored scoreboard values.
`with_default_score` stores the default as the enum's scoreboard value so
`reset()` can restore it without storing the enum type itself. `clear()` removes
the score entry when you need Minecraft's missing-score behavior.

## Transitions

A transition is "the current state is `X`, write `Y`". Read the current
state with `PHASE.of("@s").is(X)`, write the next state with
`PHASE.of("@s").set(Y)`. The recommended pattern pairs a `when(...)` guard
with a `then_one(...)` body so the transition runs only when the source
state matches:

```rust
#[function("example:phase/enter_fighting_from_idle")]
pub fn enter_fighting_from_idle() {
    when(PHASE.of("@s").is(BossPhase::Idle))
        .then_one(PHASE.of("@s").set(BossPhase::Fighting));
}
```

The guarded form lowers to a single
`execute if score @s boss_phase matches 0 run scoreboard players set @s boss_phase 1`
line. The unguarded form is just `scoreboard players set @s boss_phase 1` and
is what you want when the call site already knows the source state.

## Enter and exit hooks

Enter and exit hooks are "fired only on the tick the state actually changed"
bodies. Build them by combining `is_not(target)` with the transition write:

```rust
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

The `unless` clause fires the body only when the player was *not* in
Fighting the previous tick. The trailing `set(Fighting)` then commits the
transition so subsequent ticks see the new state and the hook no longer
fires.

Exit hooks guard on the state being left so the exit body and the transition
only run while the player is still in that state:

```rust
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
states that have meaningful enter/exit logic.

## Per-state tick

Run different logic in each state by gating the body with
`TypedExecute::as_players().when(PHASE.is(V)).run(body)`. `#[component(Tick)]`
starts in server context, so player-scoped `@s` checks need an explicit player
executor. The body emits one
`execute as @a if score @s boss_phase matches <n> run <body>` per state per
tick:

```rust
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

## Lifecycle notes (issue #47)

`.register()` enrolls a typed state variable in Sand's global lifecycle
registry. `define_registered_state()` then drains the registry and emits a
sorted list of `scoreboard objectives add …` commands. This is the
lifecycle foundation introduced for #47.

```rust
#[component(Load)]
pub fn load_state() {
    let _ = MANA.register();
    let _ = CASTING.register();
    let _ = DASH.register();
    define_registered_state();
}
```

Notes:

- The registry deduplicates among `.register()` calls. Calling
  `.register()` twice for the same variable is a no-op.
- `.define()` returns a single command string and is independent of the
  registry. Do not mix a manual `.define()` command with the
  `define_registered_state()` drain in the same load function — the
  deduplication only applies to `.register()` calls.
- Output is sorted by objective name for determinism. The registry is
  drained once per `define_registered_state()` call; subsequent calls
  return an empty `Vec` until new state is registered.

## Transition backend (issue #48)

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

## Runnable example pack

The [`examples/gameplay_state.rs`](https://github.com/ThatOneToast/sand/blob/main/examples/gameplay_state.rs)
file is a runnable, compile-tested example pack that wires every pattern in
this page into a single small `BossPhase` machine: lifecycle registration,
transitions, guarded transitions, enter/exit hooks, per-state ticks, and
the `with_default_score` reset. The example is mirrored under
`sand-example/src/gameplay_state_example.rs` so the workspace test suite
asserts the exact command output.

```sh
cargo run -p sand -- new boss_phases
# paste examples/gameplay_state.rs into src/lib.rs
cargo run -p sand -- build
```

## Structured state

For structured state, prefer `StorageSchema<T>` and typed fields:

```rust
#[derive(Debug)]
struct PlayerMagic;

static MAGIC: StorageSchema<PlayerMagic> =
    StorageSchema::new("example:data", "players.self.magic");
static MANA_DATA: StorageField<PlayerMagic, i32> = MAGIC.field("mana");
static SCHOOL: StorageField<PlayerMagic, String> = MAGIC.field("school");

#[component(Load)]
pub fn load_storage() {
    MANA_DATA.set(100);
    SCHOOL.set("pyromancy");
}
```

`StorageVar<T>` remains available for simple legacy variables. Use `RawSnbt`
only as an explicit escape hatch when typed `SnbtValue`/`SnbtCompound` builders
do not cover the shape.

`#[derive(SandStorage)]` is the preferred schema declaration for new code. `PlayerSchema` tracks score/flag/cooldown initialization and storage descriptors but does not create per-player dynamic NBT: Minecraft storage is global. See [storage reference](storage-nbt.md) and the [player-data guide](../book/src/manual/player-data.md).
