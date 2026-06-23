# Entities And Interactables

Enable `systems-entities`. `Interactable` summons Minecraft's invisible `interaction` entity and builds a filtered player-interaction advancement.

```rust
let altar = Interactable::new(ResourceLocation::new("arcane", "altar/use").unwrap())
 .size(InteractSize { width: 2.0, height: 2.0 }).tag("arcane_altar").response(altar_response);
altar.summon_at(Vec3::absolute(0.0, 64.0, 0.0));
let advancement = altar.advancement();
```

Use `summon_here`, `advancement_with`, and tags to isolate instances. `advancement()` panics during generation if no response exists. Cleanup tagged entities explicitly. Interaction is right-click, not proximity.
