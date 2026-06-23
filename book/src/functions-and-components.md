# Functions And Components

`#[function]` exports a callable `.mcfunction`; `#[component]` exports typed JSON components; and `#[component(Load)]` and `#[component(Tick)]` contribute functions to Minecraft's load and tick tags.

```rust
use sand_core::prelude::*;
use sand_macros::{component, function};

#[function("demo:spells/cast")]
pub fn cast() { cmd::say("cast"); }

#[component(Load)]
pub fn load() { cmd::say("loaded"); }

#[component(Tick)]
pub fn tick() { cmd::say("tick"); }
```

Without an explicit path, Sand derives the function path from the Rust item. Prefer explicit paths for public entry points. Call a registered function with `cmd::call(cast)` or `cmd::function(ResourceLocation::new("demo", "spells/cast").unwrap())`; the former keeps a Rust function reference rather than a reward-function string.

Conditions that use branch helpers such as `when(...).then_all([...])` register generated helper functions. Keep branch bodies small and let Sand export them; do not hand-create matching paths. Small functions with one responsibility are easier to test, reuse, and inspect in generated output.

<div class="sand-note"><strong>Components are data.</strong> A plain <code>#[component]</code> function returns a typed datapack component such as an advancement, recipe, predicate, dialog, or loot table. It is not a function-tag component.</div>
