# Introduction

Sand is a Rust-first framework for Minecraft Java datapacks. You write normal
Rust functions with typed builders, and Sand exports `.mcfunction` files,
datapack JSON, function tags, and optional resource-pack assets.

The main authoring experience is:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
}

#[component(Tick)]
pub fn tick() {
    DASH.tick(Selector::all_players());
    TypedExecute::as_players()
        .when(all![MANA.of("@s").gte(25), DASH.ready("@s")])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua(),
        ));
}

#[function]
pub fn greet() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Hello from Sand").gold(),
    );
}
```

This style keeps rust-analyzer completion, formatting, imports, refactors, and
compiler diagnostics attached to ordinary Rust source.
