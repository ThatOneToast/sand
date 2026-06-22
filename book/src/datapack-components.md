# Datapack Components

Plain `#[component]` functions return typed datapack JSON components:

```rust
#[component]
pub fn welcome_advancement() -> Advancement {
    Advancement::new("example:welcome".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}
```

Use typed builders for advancements, recipes, loot tables, predicates, tags,
dialogs, and custom item data.
