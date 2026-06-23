# Components

`#[component]` exports typed datapack JSON such as an advancement, recipe, predicate, dialog, loot table, tag, or item modifier. `#[component(Load)]` and `#[component(Tick)]` instead export functions referenced from Minecraft function tags.

```rust
#[component]
pub fn welcome() -> Dialog { Dialog::notice("arcane:welcome").title("Welcome") }
```

Use a component when Minecraft expects JSON under `data/<namespace>/...`; use `#[function]` when Minecraft expects commands. Component type determines destination and JSON shape. See [Datapack output](../reference/cheat-sheet.md).
