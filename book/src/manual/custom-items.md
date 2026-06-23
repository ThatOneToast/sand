# Custom Items

`CustomItem` creates an item with data components. Use a stable `custom_data` marker as identity; names and lore are presentation, not identity.

```rust
let shield = CustomItem::new("minecraft:shield").custom_data("arcane_shockwave");
static SHIELD: CustomItemId = CustomItemId::new("minecraft:shield", "arcane_shockwave");
```

Build supported custom names, lore, max damage/current damage, stack size, attributes, and components on `CustomItem`. The marker becomes `custom_data={arcane_shockwave:1b}` in checks. Use lowercase stable keys; changing one invalidates world items. Use raw components only for a missing version-specific component.
