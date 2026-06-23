# Inventory Quest Check

Check an item in a specific slot, consume it, then give a reward.

```rust
InventorySystem::for_entity(Selector::self_()).has("minecraft:emerald")
 .in_hotbar().run(InventorySystem::for_entity(Selector::self_())
 .clear_item("minecraft:emerald").amount(5));
InventorySystem::for_entity(Selector::self_()).give("minecraft:diamond");
```

For a custom quest token use `CustomItemId` instead of matching a display name. The command sequence is vanilla commands, not an atomic transaction; design repeat prevention with state/cooldowns.
