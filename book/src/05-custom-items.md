# 5. Custom Items

Trailforge has two custom items: the **Grapple Core**, a craftable upgrade
material, and the **Trail Striders**, upgraded boots granted once a player
claims the upgrade. They demonstrate two different reasons to reach for
`CustomItem`.

## The Grapple Core: `#[item]` and its generated predicate

{{#include ../../examples/book_project/src/lib.rs:item_grapple_core}}

`CustomItem::new("minecraft:heart_of_the_sea")` starts from a vanilla base
item — Trailforge doesn't need new item textures or models to ship this
tutorial, only new *data*: a custom name, lore, rarity, a `max_stack_size`
of 1 (it's a key item, not a stack of ten), and a `custom_data` marker
(`"grapple_core"`) baked into the item's NBT so Sand — and vanilla
predicates — can distinguish "a Grapple Core" from "a plain Heart of the
Sea."

The `#[item]` attribute macro does one more thing beyond registering the
item for export: it generates a `GrappleCore` struct with an associated
`PREDICATE` constant and a `BASE` constant, exercised directly in
Trailforge's own test suite:

```rust,ignore
#[test]
fn grapple_core_predicate_matches_custom_data() {
    assert!(GrappleCore::PREDICATE.contains("grapple_core"));
    assert_eq!(GrappleCore::BASE, "minecraft:heart_of_the_sea");
}
```

That generated predicate is what makes `execute if items entity @s
container.16 <predicate>`-style inventory checks possible without
hand-writing the JSON predicate yourself — see chapter 9, where
`ObtainedGrappleCoreEvent`'s `InventoryChangedTrigger` matches items by
`ItemPredicate::id(...).custom_data_key("grapple_core")` using the exact
same custom-data key.

## Trail Striders: a plain function, not `#[item]`

{{#include ../../examples/book_project/src/lib.rs:item_trail_striders}}

`trail_striders()` is a normal Rust function returning `CustomItem` — no
`#[item]` attribute. Trailforge doesn't need a generated predicate for the
boots (nothing checks "is this player holding Trail Striders" via an
inventory predicate elsewhere in the pack), so there's no reason to pay for
one. `#[item]` is opt-in exactly where you need the predicate/struct
machinery it generates; a plain builder function is enough whenever an item
is only ever *given*, never *matched against*.

The boots carry an `AttributeModifier` (movement speed, `+0.02`,
`AddValue`, scoped to the `Feet` equipment slot) — see chapter 11 for how
equipment attributes work and why the operation and slot matter.

## Where items get used, not just defined

Neither function above is called from `load` or `tick`. Items in Sand are
*data* — they get referenced by ID or by value from commands that grant
them. `claim_striders` (chapter 8) is what actually puts a Trail Striders
item stack into a player's inventory:

```rust,ignore
cmd::raw(format!("give @s {}", trail_striders()));
```

`CustomItem` implements `Display` to render itself as the `give`-command
item argument (`minecraft:leather_boots[...components...]`), so calling
`trail_striders()` again here reconstructs the same item definition rather
than requiring Trailforge to keep an item string in sync by hand in two
places.
