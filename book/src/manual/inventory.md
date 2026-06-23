# Inventory

Enable `systems-inventory`. `InventorySystem::for_entity` produces vanilla inventory commands and item conditions.

```rust
let inv = InventorySystem::for_entity(Selector::self_());
inv.has(SHIELD).in_offhand().run(cmd::say("shield"));
inv.replace(ItemSlot::MainHand, "minecraft:stick");
inv.replace_count(ItemSlot::MainHand, "minecraft:stick", 1);
inv.clear_slot(ItemSlot::OffHand);
inv.clear_item("minecraft:amethyst_shard").amount(1);
inv.give("minecraft:diamond");
```

Checks support `.in_slot`, mainhand/offhand, armor, hotbar, inventory, any slot, `.not_in_slot`, and `.not_anywhere`. Item arguments are vanilla item predicates; use `CustomItemId` for persistent custom identity. No transaction is implied across multiple commands or players.
