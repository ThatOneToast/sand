# Custom Items

`CustomItem` creates an item with data components. Use a stable `custom_data` marker as identity; names and lore are presentation, not identity.

```rust
let shield = CustomItem::new("minecraft:shield").custom_data("arcane_shockwave");
static SHIELD: CustomItemId = CustomItemId::new("minecraft:shield", "arcane_shockwave");
```

Build supported custom names, lore, max damage/current damage, stack size, attributes, and components on `CustomItem`. The marker becomes `custom_data={arcane_shockwave:1b}` in checks. Use lowercase stable keys; changing one invalidates world items. Use raw components only for a missing version-specific component.

## Typed enchantments

```rust
let crossbow = CustomItem::new("minecraft:crossbow")
    .typed_enchantment(EnchantmentId::minecraft("quick_charge").unwrap(), 3);
let book = CustomItem::new("minecraft:enchanted_book")
    .typed_stored_enchantment(EnchantmentId::minecraft("sharpness").unwrap(), 5);
```

These emit current item components such as `enchantments={"minecraft:quick_charge":3}` and `stored_enchantments={"minecraft:sharpness":5}`. They are not legacy pre-1.20.5 item NBT. Parsing and gameplay validity are separate: Minecraft may accept an enchantment/item combination without giving it useful gameplay behavior.

<div class="sand-warning"><strong>Version-sensitive escape hatch.</strong> Sand emits Minecraft item components, not legacy item NBT. If a component is missing or version-sensitive, use <code>RawComponent</code> explicitly and file an issue:</div>

```rust
let crossbow = CustomItem::new("minecraft:crossbow").with_raw_component(
    RawComponent::new("enchantments", r#"{"minecraft:quick_charge":10}"#),
);
```
