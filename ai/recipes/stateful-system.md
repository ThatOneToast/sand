---
id: stateful-system
capabilities:
  - score-state
  - flags
  - cooldowns
  - conditions
  - execute-chains
minecraft:
  minimum: "1.18.0"
  maximum_verified: "26.2.0"
cargo_features: ["systems-cooldowns"]
verification:
  compiles: true
  golden_output: false
  vanilla_reload: false
---

# Stateful system

## Intent

A gated player action: costs a resource, requires an off-cooldown, and is
blocked while another flag is set (e.g. a spell cast). Demonstrates
composing `ScoreVar`, `Flag`, `Cooldown`, and nested `all!`/`any!` conditions
through a typed execute chain.

## Required crates and features

`sand-core` with the `systems-cooldowns` Cargo feature enabled (`Cooldown`
lives behind it). `sand-macros`.

## Code

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static CAST_CD: Cooldown = Cooldown::new("cast", Ticks::seconds(3));

#[component(Load)]
pub fn load_state() {
    MANA.define();
    CASTING.define();
    CAST_CD.define();
    MANA.set(Selector::all_players(), 100);
}

#[component(Tick)]
pub fn tick_state() {
    CAST_CD.tick(Selector::all_players());
}

#[function]
pub fn try_cast() {
    TypedExecute::as_players_at_self()
        .when(all![
            MANA.of("@s").gte(20),
            CASTING.of("@s").is_false(),
            CAST_CD.ready("@s"),
        ])
        .run(Actionbar::show(Selector::self_(), Text::new("Cast ready").aqua()));
}
```

To actually spend mana and start the cooldown once the cast fires, call
`MANA.remove(Selector::self_(), 20)` and `CAST_CD.start(Selector::self_())`
inside the branch that handles a successful cast (e.g. the function this one
calls next).

## Expected generated resources

- `scoreboard objectives add mana dummy`, `... add casting dummy`, and the
  cooldown's backing objective(s), all in `load.mcfunction`.
- `tick.mcfunction` containing the cooldown's per-tick decrement command.
- `try_cast.mcfunction` containing a single `execute as @a at @s if score @s
  mana matches 20.. if score @s casting matches 0 if score @s cast_cd
  matches 0 run title @s actionbar ...` (exact command shape depends on
  `Cooldown`'s internal scoreboard/tag choice — read the generated file
  rather than assuming the string).

## Sand limitations

None beyond the general cooldown/flag/score capabilities, all
`implemented`/`stable`.

## Vanilla limitations

Cooldowns and flags are scoreboard-backed polling constructs, not vanilla
events — `CAST_CD.tick()` must run every tick to decrement, which has a
real (if small) per-player tick cost. This is a vanilla constraint on how
datapacks track time, not a Sand gap.

## Validation steps

1. `cargo build`.
2. `cargo run -p sand -- build`; inspect `dist/.../function/try_cast.mcfunction` for the generated `execute ... if score ...` chain.
3. Not vanilla-reload-verified in this review.

## Common incorrect approaches

- Checking `CASTING.of("@s").is_false()` without ever setting `CASTING` to
  true when a cast starts — the flag needs to be part of the full cast flow,
  not just the gate.
- Using `any![...]` when every condition must hold — `any!` is OR, `all!` is
  AND; mixing them up silently changes the gameplay logic without a compile
  error.
- Forgetting `CAST_CD.tick(...)` in a `#[component(Tick)]` function — without
  it the cooldown never counts down and `.ready(...)` stays false forever.
