# Interactable Altar

An altar is a tagged interaction hitbox plus a response reward function.

```rust
#[function]
pub fn altar_response() {
    InventorySystem::for_entity(Selector::self_()).has("minecraft:amethyst_shard")
      .in_any_slot().run(InventorySystem::for_entity(Selector::self_())
      .clear_item("minecraft:amethyst_shard").amount(1));
    InventorySystem::for_entity(Selector::self_()).give("minecraft:diamond");
}
let altar = Interactable::new(ResourceLocation::new("arcane", "altar/use").unwrap())
 .tag("arcane_altar").response(altar_response);
altar.summon_here(); let component = altar.advancement();
```

Add typed particles/sound in the response. Remove `@e[tag=arcane_altar]` when consumed, or retain the tag for repeatable interaction and administrative cleanup.
