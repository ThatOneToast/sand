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

Structure templates are binary `.nbt` assets and are copied into the datapack
rather than generated as Rust data:

```rust
#[component]
pub fn starter_room() -> StructureTemplate {
    StructureTemplate::new(
        ResourceLocation::new("example", "rooms/starter").unwrap(),
        "src/structures/starter.nbt",
    )
}
```

This writes `src/structures/starter.nbt` to
`data/example/structure/rooms/starter.nbt`. Sand rejects unsafe paths and
requires structure template assets to use `.nbt`.

Custom items use typed item components instead of handwritten component SNBT:

```rust
let wand = CustomItem::new(ItemId::minecraft("stick").unwrap())
    .id("example:dash_wand")
    .component(ItemComponent::custom_name(Text::new("Dash Wand").aqua()))
    .component(ItemComponent::lore(vec![Text::new("Right click to dash").gray()]))
    .component(ItemComponent::custom_model_data(1001))
    .component(ItemComponent::rarity(Rarity::Rare))
    .component(ItemComponent::max_stack_size(1));
```

For modded or future components, use an explicit `RawComponent` escape hatch:

```rust
let relic = CustomItem::new("minecraft:amethyst_shard")
    .with_raw_component(RawComponent::new("mymod:charge", "{value:3}"));
```
