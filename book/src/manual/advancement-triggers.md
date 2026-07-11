# Advancement Triggers

Builders in `sand_core::event::trigger` create `AdvancementTrigger` values, including `UsingItemTrigger`, `ConsumeItemTrigger`, `ItemObtainedTrigger`, `RecipeUnlockedTrigger`, `PlayerInteractedWithEntityTrigger`, and `SummonedEntityTrigger`. `ItemObtainedTrigger` maps to `minecraft:crafted_item`, so it fires only when an item is crafted—not when an item is smelted or otherwise acquired.

```rust
PlayerInteractedWithEntityTrigger::new()
 .item(ItemPredicate::id("minecraft:stick"))
 .entity(EntityPredicate::new().nbt("{Tags:[\"altar\"]}"))
 .build();
```

Use typed predicates first; `RawJson` is an explicit escape hatch for a version field Sand does not model. See [Event Trigger Matrix](../reference/event-trigger-matrix.md).
