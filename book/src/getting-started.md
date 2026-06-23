# Getting Started

Create a project with the workspace CLI, then build it from the generated project:

```sh
cargo run -p sand -- new my_pack
cd my_pack
cargo run -p sand -- build
```

The generated crate owns your Rust authoring code. Sand collects annotated functions during compilation and writes a vanilla datapack into the configured build output. Copy or link that output into your world's `datapacks/` directory, start the world, and run `/reload`.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

static TICKS: ScoreVar<i32> = ScoreVar::new("example_ticks");

#[component(Load)]
pub fn load() {
    TICKS.define();
    cmd::tellraw(Selector::all_players(), Text::new("Example loaded").green());
}

#[component(Tick)]
pub fn tick() {
    TICKS.add(Selector::all_players(), 1);
}

#[function]
pub fn greet() {
    cmd::tellraw(Selector::self_(), Text::new("Hello from Sand").gold());
}
```

After `/reload`, invoke the generated function using its namespace and path, for example `/function my_pack:greet`. See [Functions And Components](functions-and-components.md) for path rules.

<div class="sand-note"><strong>Reload failures.</strong> Minecraft reports JSON and function parse errors in the game log. Rust type errors happen during <code>cargo build</code>; a missing function, stale datapack copy, invalid resource location, or a version-incompatible component usually appears after <code>/reload</code>. Delete old output before diagnosing a stale installation.</div>
