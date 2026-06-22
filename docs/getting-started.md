# Getting Started

Use the typed prelude for normal datapack code:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};
```

Define state, functions, and components with typed builders:

```rust
static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[component(Load)]
pub fn load() {
    mcfunction! {
        MANA.define();
        MANA.set(Selector::all_players(), 100);
    }
}

#[function]
pub fn greet() {
    mcfunction! {
        cmd::tellraw(Selector::all_players(), Text::new("Hello from Sand").gold());
    }
}
```

The CLI is currently used from the workspace while publishing is unfinished:

```sh
cargo run -p sand -- new my_pack
cargo run -p sand -- build
```
