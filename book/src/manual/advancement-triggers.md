# Advancement Triggers

Builders in `sand_core::event::trigger` create `AdvancementTrigger` values, including `UsingItemTrigger`, `ConsumeItemTrigger`, `ItemObtainedTrigger`, `RecipeUnlockedTrigger`, `PlayerInteractedWithEntityTrigger`, and `SummonedEntityTrigger`.

```rust
PlayerInteractedWithEntityTrigger::new()
 .item(ItemPredicate::id("minecraft:stick"))
 .entity(EntityPredicate::new().nbt("{Tags:[\"altar\"]}"))
 .build();
```

Use typed predicates first; `RawJson` is an explicit escape hatch for a version field Sand does not model. See [Event Trigger Matrix](../reference/event-trigger-matrix.md).
