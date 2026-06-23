# 5. State, Flags, Timers, And Cooldowns

Use score-backed state for per-player numbers, booleans, and timers.

```rust
static CASTING: Flag = Flag::new("arcane_casting");
static CAST_CD: Cooldown = Cooldown::new("arcane_cast", Ticks::seconds(3));
#[component(Tick)] pub fn tick_state() { CAST_CD.tick(Selector::all_players()); }
```

Guard casts with `CAST_CD.ready("@s")`, then `start`. [State](../manual/state.md) covers score operations and lifecycle details.
