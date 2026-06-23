# 10. Interactable Entities

Enable `systems-entities`; interaction entities turn right-clicks into typed advancement rewards.

```rust
let altar = Interactable::new(ResourceLocation::new("arcane", "altar/use").unwrap())
 .tag("arcane_altar").response(altar_response);
altar.summon_here();
let altar_advancement = altar.advancement();
```

See [Entities And Interactables](../manual/entities.md) for dimensions, tags, and cleanup.
