# Mana System

Scoreboards are the default per-player mana store. Define once, initialize on join/lifecycle, regenerate in tick, and spend behind a condition.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("arcane_mana");
#[component(Load)] pub fn load() { MANA.define(); }
#[component(Tick)] pub fn regen() { MANA.add(Selector::all_players(), 1); }
#[function] pub fn spend() { when(MANA.of("@s").gte(20)).then(MANA.remove("@s", 20)); }
```

Attach a `PlayerSchema` if you want default score/flag/cooldown setup. A derived storage schema is useful for global configuration, but storage is not automatically per-player; do not put `players.@s` into a static path.
