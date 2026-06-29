# State

`ScoreVar<T>`, `Flag`, `Timer`, `Cooldown`, and `Ticks` model scoreboard-backed state.

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
MANA.define(); MANA.add("@s", 10); DASH.start(Selector::self_());
```

Define objectives in load, tick timers/cooldowns on the appropriate target set, and query them with typed conditions. Objective names are global Minecraft names; choose collision-resistant names.

Use `GameState<S>` for finite enum-backed gameplay phases:

```rust,ignore
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase { Idle = 0, Casting = 1 }

impl TypedGameState for Phase {
    fn to_score(self) -> i32 { self as i32 }
    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Casting),
            _ => None,
        }
    }
}

static PHASE: GameState<Phase> = GameState::with_default("phase", Phase::Idle);

PHASE.of("@s").set(Phase::Casting);
PHASE.of("@s").reset(); // writes Idle's score
```

Explicit discriminants are the storage format. Treat renumbering variants as a
state migration for live datapacks. `GameState::new` has no default, so
`reset()` clears the score entry; `GameState::with_default` resets to the named
variant.

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
