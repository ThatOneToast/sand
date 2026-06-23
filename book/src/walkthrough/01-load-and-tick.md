# 1. Load And Tick Components

## What you will build

Add a mana objective and a repeating tick function. This is the project foundation for stateful features.

## Concepts introduced

`ScoreVar`, scoreboard objectives, load initialization, tick work, and per-player targets.

## File changes

Replace `arcane/src/lib.rs` with:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");

#[component(Load)]
pub fn arcane_load() {
    MANA.define();
    MANA.set(Selector::all_players(), 100);
    cmd::tellraw(Selector::all_players(), Text::new("Arcane Powers loaded").gold());
}

#[component(Tick)]
pub fn arcane_tick() { MANA.add(Selector::all_players(), 1); }

#[function("arcane:hello")]
pub fn hello() { cmd::tellraw(Selector::self_(), Text::new("Hello from Arcane Powers").aqua()); }
```

## How it works

Load is for definitions and reset-safe initialization: it emits `scoreboard objectives add arcane_mana dummy`. Tick is recurring work: it runs every game tick, so this demo restores one mana every 1/20 second.

## What Sand generates

Generated load/tick functions are listed in Minecraft's load/tick function tags. `MANA.add(@a, 1)` lowers to `scoreboard players add @a arcane_mana 1`.

## Try it in Minecraft

Build and `/reload`, then run `/scoreboard players get @s arcane_mana`. Wait a second and run it again.

## Common mistakes

- Calling `define()` in tick.
- Treating load as a per-player join event.
- Leaving uncapped regen in production.

## Deeper reading

[Variables](../manual/data-model/variables.md), [State](../manual/state.md), and [Functions](../manual/functions.md).
