# Advancement Triggers

Builders in `sand_core::event::trigger` create `AdvancementTrigger` values, including `UsingItemTrigger`, `ConsumeItemTrigger`, `ItemObtainedTrigger`, `RecipeUnlockedTrigger`, `PlayerInteractedWithEntityTrigger`, and `SummonedEntityTrigger`. `ItemObtainedTrigger` maps to `minecraft:crafted_item`, so it fires only when an item is crafted—not when an item is smelted or otherwise acquired.

```rust
PlayerInteractedWithEntityTrigger::new()
 .item(ItemPredicate::id("minecraft:stick"))
 .entity(EntityPredicate::new().nbt("{Tags:[\"altar\"]}"))
 .build();
```

Use typed predicates first; `RawJson` is an explicit escape hatch for a version field Sand does not model. See [Event Trigger Matrix](../reference/event-trigger-matrix.md).

Resource-bearing trigger variants also provide typed constructors. These keep
block, dimension, potion, effect, recipe, loot-table, and custom trigger IDs on
validated registry/resource-location paths while preserving the same JSON:

```rust
use sand_core::{AdvancementTrigger, BlockId, DimensionId, PotionRegistryId};

let placed = AdvancementTrigger::placed_block(
    Some(BlockId::minecraft("stone")?),
    None,
    None,
    None,
);
let changed_dimension = AdvancementTrigger::changed_dimension(
    Some(DimensionId::minecraft("overworld")?),
    Some(DimensionId::minecraft("the_nether")?),
);
let brewed = AdvancementTrigger::brewed_potion(
    PotionRegistryId::minecraft("swiftness")?,
);
# Ok::<(), sand_core::SandError>(())
```

The public string-field enum variants remain compatibility/raw paths. Prefer
the associated constructors for new code. `custom_trigger(ResourceLocation,
Option<RawJson>)` validates the modded trigger ID while leaving its conditions
intentionally opaque.
