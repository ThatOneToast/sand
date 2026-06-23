# Slots And Equipment

`ItemSlot` is the canonical taxonomy; `Slot` is an alias. New code should not use deprecated `InventorySlot` or `SlotPattern`.

```rust
ItemSlot::MainHand; ItemSlot::OffHand; ItemSlot::Head;
ItemSlot::Hotbar(0); ItemSlot::AnyHotbar; ItemSlot::Inventory(0);
ItemSlot::Container(0); ItemSlot::AnyContainer; ItemSlot::Raw("*".into());
```

Armor, weapon, hotbar, inventory, container, horse, and villager wildcard families reflect vanilla slot syntax. `Raw` is for a valid Minecraft slot pattern Sand has not enumerated.
