# 11. Equipment And Attributes

Trail Striders (chapter 5) grant a real gameplay effect purely through item
data — no function, event, or tick logic is involved:

{{#include ../../examples/book_project/src/lib.rs:item_trail_striders}}

## `AttributeModifier`

```rust,ignore
.component(ItemComponent::attribute_modifier(
    AttributeModifier::new(AttributeId::MovementSpeed)
        .amount(0.02)
        .operation(AttributeOperation::AddValue)
        .slot(EquipmentSlotGroup::Feet),
))
```

Attribute modifiers are Minecraft's native mechanism for equipment-driven
stat changes — the same system vanilla enchantments and armor trims use.
Three fields matter:

- **`AttributeId::MovementSpeed`** — which stat this modifier changes.
  Sand's `AttributeId` is a typed enum over vanilla's attribute registry,
  so `AttributeId::MovementspeEd` (a typo) is a compile error, not a
  silently-ignored malformed attribute ID string in the exported item JSON.
- **`.operation(AttributeOperation::AddValue)`** — vanilla attribute
  modifiers support three operations (`AddValue`, `AddMultipliedBase`,
  `AddMultipliedTotal`), each composing differently when multiple modifiers
  stack on the same attribute. `AddValue` adds `0.02` directly to the base
  movement speed (`0.1` by default), a flat, easy-to-reason-about bonus —
  the right choice when you want a modifier that doesn't compound
  unpredictably with other percentage-based speed boosts.
- **`.slot(EquipmentSlotGroup::Feet)`** — attribute modifiers on items only
  apply while the item is equipped in a matching slot. Scoping to `Feet`
  means the speed bonus applies only while Trail Striders are actually worn
  as boots, not merely held or stored in the inventory — the same
  slot-gating vanilla boots enchantments rely on.

## Why this needs no Sand-side logic at all

This is the chapter's central point: equipment attributes are pure
datapack-JSON item data. Once `trail_striders()` is `give`n to a player
(chapter 8's `trail:claim_striders`), vanilla's own equipment/attribute
system applies and removes the modifier automatically as the item is worn
or unworn — there's no tick check, no event, nothing for Trailforge's
`tick` function to maintain. This is worth internalizing as a general
principle: whenever an effect can be expressed as *data on an item*
(attribute modifiers, enchantments, custom-model-data-driven behavior),
prefer that over reproducing the same effect procedurally in a tick
function. Data-driven effects are more robust (they survive `/reload`,
work correctly for every player wearing the item, and don't need
Trailforge's own tick loop to enforce them) and cheaper (no per-tick
command execution cost).

## Where procedural logic *is* still needed

Contrast this with the Grapple Core's movement effect (chapter 8's
`trail:grapple/execute`), which applies `Speed`/`SlowFalling` *status
effects* via commands rather than an item attribute. That's a deliberate,
different choice: the dash is a discrete, cooldown-gated *action* (not a
passive, always-on-while-worn stat), so it has to be triggered
procedurally — there's no vanilla item-attribute equivalent for "grant a
timed burst on demand."
