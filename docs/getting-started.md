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
    MANA.define();
    MANA.set(Selector::all_players(), 100);
}

#[function]
pub fn greet() {
    cmd::tellraw(Selector::all_players(), Text::new("Hello from Sand").gold());
}
```

The CLI is currently used from the workspace while publishing is unfinished:

```sh
cargo run -p sand -- new my_pack
cargo run -p sand -- build
```

Use `mcfunction!` later when you need advanced command grouping or explicit
interop fragments. Normal Sand datapack functions should stay as ordinary Rust
functions with typed command expressions.
