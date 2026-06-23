# A Tiny Datapack

Create a project with `cargo run -p sand -- new arcane`, open `src/lib.rs`, and add:

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[component(Load)] pub fn load() { cmd::say("Arcane loaded"); }
#[function] pub fn hello() { cmd::tellraw(Selector::self_(), Text::new("Hello").gold()); }
```

Build, copy the generated datapack into `world/datapacks`, run `/reload`, then invoke `/function arcane:hello`. Sand writes functions under `data/<namespace>/function`; inspect that output whenever Minecraft reports a reload error. Next: [load and tick](01-load-and-tick.md).
