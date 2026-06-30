# Agent Guide — Typed Gameplay State (Sand)

This is the concise agent-facing companion to
[`docs/typed-state.md`](../typed-state.md) and
[`book/src/typed-state.md`](../../book/src/typed-state.md). It is a quick
reference, not a tutorial. If you are writing new code that touches
`GameState<S>`, follow the patterns below; the linked docs explain the
"why" behind them.

## The mental model

- A `GameState<S>` is a single scoreboard objective plus a Rust enum that
  names the legal values.
- Each variant maps to a fixed `i32` discriminant; that discriminant is the
  on-disk format. Renumbering a variant is a state migration.
- `PHASE.of(sel)` is a typed accessor for that one objective on a given
  selector. Every method (`set`, `reset`, `is`, `is_not`, `clear`) returns
  a vanilla command string or a `Condition` you can drop into `when`.

## Lifecycle: always use the registry

For load-time objective definitions, prefer `.register()` +
`define_registered_state()` over a hand-written `.define()` call:

```rust
#[component(Load)]
pub fn load() {
    let _ = PHASE.register();
    let _ = MANA.register();
    let _ = DASH.register();
    define_registered_state();
}
```

- `.register()` returns `()`, so discard it with `let _ = ...` inside an
  attribute body.
- `define_registered_state()` returns a sorted, deduplicated `Vec<String>`.
  Calling it drains the registry; subsequent calls return an empty `Vec`
  until new state is registered.
- Do not mix a manual `.define()` command with the registry drain in the
  same load function. Deduplication only applies among `.register()` calls.

## Transitions: read with `is`, write with `set`

```rust
// Unconditional set — call site already knows the source state.
PHASE.of("@s").set(BossPhase::Fighting);

// Guarded transition — refuses to write unless the current state is Idle.
when(PHASE.of("@s").is(BossPhase::Idle))
    .then_one(PHASE.of("@s").set(BossPhase::Fighting));
```

Output:

- Unconditional: `scoreboard players set @s boss_phase 1`
- Guarded: `execute if score @s boss_phase matches 0 run scoreboard players set @s boss_phase 1`

`PHASE.of(...)` takes `&str`. Use the string literal `"@a"` rather than
`Selector::all_players()` (which is a struct, not a string).

## Enter and exit hooks: `is_not` for enter, `is` for exit

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

- `is_not(target)` lowers to `unless score … matches <n>`, so the body
  fires only when the player was *not* in the target state the previous
  tick. The trailing `set(target)` commits the transition.
- Exit hooks guard the current state instead:
  `when(PHASE.of("@s").is(BossPhase::Enraged)).then_one(body); PHASE.of("@s").set(BossPhase::Fighting);`
- Use this only for states that have meaningful enter/exit logic. It
  doubles the per-state tick cost for that state.

## Per-state tick: one `when` branch per state

```rust
#[component(Tick)]
pub fn boss_tick() {
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Idle))
        .run(/* body */);
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Fighting))
        .run(/* body */);
    TypedExecute::as_players()
        .when(PHASE.of("@s").is(BossPhase::Enraged))
        .run(/* body */);
}
```

- In a tick component, each branch lowers to one
  `execute as @a if score @s boss_phase matches <n> run <body>` per tick.
  Cost is `O(N)` per player per tick for `N`
  branches.
- If several states share a body, hoist the body out of the branches and
  gate only the parts that differ. Do not add `is_not` clauses to every
  state; only states with hooks need them.
- Prefer `then_one` over `then_all` when the body is a single
  `Display`-able value. `then_one` emits the body inline; `then_all`
  registers a branch function and emits a `function ns:branch_X` call.

## Reset and clear

- `PHASE.of(sel).reset()` writes the configured default score (if the
  state was declared with `with_default_score`) or clears the scoreboard
  entry (if declared with `GameState::new`).
- `PHASE.of(sel).clear()` always clears the scoreboard entry, regardless
  of the configured default.

Prefer `with_default_score` over `new` when the state has a known reset
value — it is cheaper and more predictable than relying on Minecraft's
missing-score behavior.

## Backend cheatsheet

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

## Don'ts

- Do not use `cmd::raw` to write a `scoreboard players set` command. Use
  `PHASE.of(sel).set(V)` instead.
- Do not call `.define()` for a state that is also `.register()`-ed. Pick
  one path per objective.
- Do not add `is_not(...)` clauses to every state in tick. Only states
  with hooks need them; the rest stay `O(N)`.
- Do not assume `PHASE.of(Selector::all_players())` works. `PHASE.of`
  takes `&str`; pass `"@a"` (or any other selector string) directly.
- Do not return a `()` (unit) value from `#[function]` or
  `#[component]` bodies. Every expression in those bodies must produce a
  value implementing `IntoCommands` (e.g. `String`, `Vec<String>`, or a
  `Command`). `PHASE.register()` returns `()`, so wrap it in
  `let _ = ...` and discard.
- Do not rewrite the scoreboard math by hand. Use the typed helpers
  (`set_percent`, `scale_percent`, `set_ratio`, `safe_divide`,
  `clamp_score`, `saturating_add`, `saturating_sub`).

## Cross-references

- Full reference: [`docs/typed-state.md`](../typed-state.md)
- Book guide: [`book/src/typed-state.md`](../../book/src/typed-state.md)
- Manual page: [`book/src/manual/state.md`](../../book/src/manual/state.md)
- Runnable, compile-tested example:
  [`examples/gameplay_state.rs`](https://github.com/ThatOneToast/sand/blob/main/examples/gameplay_state.rs)
- Source: `sand-core/src/state/typed_state.rs` (`GameState`,
  `TypedGameState`, `GameStateRef`) and `sand-core/src/state/registry.rs`
  (lifecycle registry, `define_registered_state`).
