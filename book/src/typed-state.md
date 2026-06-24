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
