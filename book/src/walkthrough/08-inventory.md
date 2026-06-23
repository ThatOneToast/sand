# 8. Inventory Checks

Enable `systems-inventory` and check an exact item/slot before running a command.

```rust
InventorySystem::for_entity(Selector::self_()).has(WAND).in_mainhand()
    .run(cmd::call(cast_spell));
```

Use `.clear_item(...).amount(1)` to consume a crystal and `.replace` for a cooldown item. See [Inventory](../manual/inventory.md) and [Slots](../manual/slots.md).
