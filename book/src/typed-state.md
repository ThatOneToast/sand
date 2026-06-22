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
