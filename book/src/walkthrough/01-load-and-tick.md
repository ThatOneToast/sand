# 1. Load And Tick Components

Load components set up objectives; tick components perform repeated work.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
#[component(Load)] pub fn load() { MANA.define(); }
#[component(Tick)] pub fn tick() { MANA.add(Selector::all_players(), 1); }
```

The exporter adds generated functions to `minecraft:load` and `minecraft:tick`. Do not put `define()` in tick. See [Functions](../manual/functions.md) and [State](../manual/state.md).
