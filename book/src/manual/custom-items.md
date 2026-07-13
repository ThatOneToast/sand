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

## Validation

`CustomItem`'s `Display`/`Into<String>` impls remain fully infallible — they
never validate, so malformed numeric or string state (a non-finite attribute
amount, a zero-level enchantment, an empty raw resource-id string) formats
as-is and can silently emit item-component SNBT Minecraft rejects at
command-dispatch time. This is a deliberate, documented raw escape hatch, not
the recommended path.

Prefer `CustomItem::validate()` or `CustomItem::try_to_string()` at
command-generation boundaries:

```rust
let item = CustomItem::new("minecraft:diamond_sword").max_stack_size(0);

// Fails with a Sand diagnostic naming the item base and the invalid field —
// never reaches cmd::give / a generated .mcfunction line.
let result = item.try_to_string();
assert!(result.is_err());
```

`validate()`/`try_to_string()` check numeric invariants (`max_stack_size` in
`1..=99`, non-negative `max_damage`/`damage`/`repair_cost`, `damage <=
max_damage`, non-zero enchantment levels, finite attribute amounts/consume
seconds/tool speeds/use-cooldown, a 24-bit `PotionContents::custom_color`)
and string invariants (non-empty, quote/backslash-free enchantment ids,
attribute ids, sounds, models, tool-rule block strings, and a non-empty
`custom_data` marker key). `stack_components()` (used for recipe results)
calls `validate()` internally, so an invalid `CustomItem` used as a recipe
result also fails before JSON is written.
