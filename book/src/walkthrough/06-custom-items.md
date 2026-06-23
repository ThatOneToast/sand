# 6. Custom Items

Give custom items durable identities through `custom_data`, not names.

```rust
static WAND: CustomItemId = CustomItemId::new("minecraft:stick", "arcane_wand");
let wand = CustomItem::new("minecraft:stick").custom_data("arcane_wand");
```

The full item builder owns supported data components. [Custom Items](../manual/custom-items.md) explains identities, durability, and raw components.
