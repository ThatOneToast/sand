# Migration Notes

Use `ItemSlot`/`Slot` in new code; `InventorySlot` and `SlotPattern` are deprecated compatibility APIs. Use `DamageAmount::hearts`, `points`, or `fixed`; removed score/event amount variants were not valid command generation paths. Replace raw normal effects, teleport, and summon commands with their typed builders where possible.
