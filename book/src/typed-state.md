# Typed State

Use typed state wrappers instead of handwritten scoreboard commands.

```rust
use sand_core::prelude::*;
use sand_macros::component;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load_state() {
    MANA.define();
    CASTING.define();
    DASH.define();
    MANA.set(Selector::all_players(), 100);
    CASTING.disable(Selector::all_players());
}

#[component(Tick)]
pub fn tick_state() {
    DASH.tick(Selector::all_players());
}
```

State wrappers also produce typed conditions for execute chains.

## Score comparisons and integer math

Score entries can be compared and combined without raw command strings:

```rust,ignore
static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static COST: ScoreVar<i32> = ScoreVar::new("mana_cost");

when(MANA.of("@s").gte_score(COST.of("@s"))).then_all([
    MANA.of("@s").sub_score(COST.of("@s")),
    cmd::call(cast_spell),
]);
```

Use `ScoreConst::new("tick_rate", 20).ref_()` when an operation needs a
constant right-hand score. Sand creates its fake-player entry in
`__sand_score_init` on load. Scoreboards are integer-only; division and modulo
by zero retain vanilla's runtime behavior.

## Score math helpers

Use the higher-level score math helpers for common gameplay formulas:

```rust,ignore
static HEALTH: ScoreVar<i32> = ScoreVar::new("health");
static MAX_HEALTH: ScoreVar<i32> = ScoreVar::new("max_health");
static HEALTH_PERCENT: ScoreVar<i32> = ScoreVar::new("health_pct");
static DAMAGE: ScoreVar<i32> = ScoreVar::new("damage");
static SCALED_DAMAGE: ScoreVar<i32> = ScoreVar::new("scaled_damage");
static MIN_MANA: ScoreVar<i32> = ScoreVar::new("min_mana");
static MAX_MANA: ScoreVar<i32> = ScoreVar::new("max_mana");

HEALTH_PERCENT
    .of("@s")
    .set_ratio(HEALTH.of("@s"), MAX_HEALTH.of("@s"), 100);

SCALED_DAMAGE.of("@s").set_percent(DAMAGE.of("@s"), 150);
MANA.of("@s").saturating_sub(10, 0, 100);
MANA.of("@s").clamp_score(MIN_MANA.of("@s"), MAX_MANA.of("@s"));
```

`set_percent`, `scale_percent`, and `set_ratio` register deterministic fake-player
constants and emit plain scoreboard operations. `safe_divide` emits an explicit
zero-divisor branch with a fallback value. All helpers use vanilla integer
scoreboard math, so percentages and ratios truncate toward zero.
