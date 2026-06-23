# State

`ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, and `Ticks` model scoreboard-backed state.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
MANA.define(); MANA.add("@s", 10); DASH.start(Selector::self_());
```

Define objectives in load, tick timers/cooldowns on the appropriate target set, and query them with typed conditions. Objective names are global Minecraft names; choose collision-resistant names.
