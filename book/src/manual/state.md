# State

`ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, and `Ticks` model scoreboard-backed state.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
MANA.define(); MANA.add("@s", 10); DASH.start(Selector::self_());
```

Define objectives in load, tick timers/cooldowns on the appropriate target set, and query them with typed conditions. Objective names are global Minecraft names; choose collision-resistant names.

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
