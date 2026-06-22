# Components

Datapack components are ordinary Rust values registered with `#[component]`.

```rust
use sand_core::prelude::*;
use sand_macros::component;

#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice("example:welcome")
        .title("Welcome")
        .body(DialogBody::text("Typed components compile to datapack JSON."))
}
```

Component families include advancements, recipes, loot tables, predicates, item
modifiers, tags, dialogs, damage types, enchantments, chat types, trims, banner
patterns, wolf variants, worldgen, and item component builders.
