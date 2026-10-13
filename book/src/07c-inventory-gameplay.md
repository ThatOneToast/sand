# Inventory Gameplay

Start with the inventory attached to a player gameplay handle. Slot writes return
owned `Actions`: include them in `mcfunction!`, a runtime guard, or another action
composition so Sand can validate and export them.

```rust,ignore
Target::players().each(|player| {
    let inventory = player.inventory();
    let reward = ItemStack::new(ItemId::minecraft("diamond").unwrap()).count(3);

    mcfunction![
        inventory.offhand().set(reward).unwrap();
        inventory.hotbar(3).unwrap().clear().unwrap();
        when(inventory.mainhand().is_empty().unwrap()).then(
            inventory.mainhand().replace_from(&inventory.offhand()).unwrap()
        );
    ]
})
```

Rust constructs this behavior during compilation. The guard tests the live
main-hand slot when Minecraft executes the function. Discarding the returned
`Actions` does not schedule a write.

## Slots and concrete stacks

The inventory provides `mainhand`, `offhand`, `helmet`, `chestplate`, `leggings`,
and `boots`. `hotbar(index)` validates indexes 0–8; `main_inventory(index)`
validates 0–26 and addresses player inventory slots 9–35. `slot(index)` accepts
the combined range 0–35.

`set` accepts the canonical `ItemStack`, a typed `ItemId`, a generated vanilla
item, or a `CustomItem` builder. Component-bearing stacks retain their count and
components until export. A reusable custom-item definition can supply
`definition.stack(1)` directly. Invalid counts and unsupported locations produce
Sand diagnostics. `clear` empties exactly the selected slot.

## Counts and State

`inventory.count(item_id)` returns a `StatCurve`, the same runtime numeric
expression used by State. Counts can be assigned to a Score or FixedScore,
combined with other numeric sources, or compared into a `Condition`:

```rust,ignore
let inventory = player.inventory();
let diamonds = inventory.count(ItemId::minecraft("diamond").unwrap());
let state = Resources::on(*player);
let actions = mcfunction![
    state.diamonds.set(diamonds.clone());
    when(diamonds.gte(3)).then(inventory.offhand().clear().unwrap());
];
```

Counting does not remove items. It includes player inventory, the player
crafting grid, and the cursor stack. Open-container and ender-chest contents are
excluded. This API currently matches the base item ID, regardless of components;
it does not distinguish a custom item from other stacks of the same base item.
An empty match is zero. A missing or non-player owner fails the numeric read,
so State does not receive a stale value.

## Copying and containers

`destination.replace_from(&source)` copies the source's current stack, including
count and components. It does not remove the source. Copying an empty source
clears the destination. A missing source entity fails without writing. Entity
sources must select one entity; destinations can select several.

The same location model addresses a block container:

```rust,ignore
let chest_slot = ItemLocation::block(BlockPos::here()).slot(0)?;
let actions = chest_slot.replace_from(&inventory.offhand())?;
```

The block must actually be a container with that slot when the action runs.
Sand validates the location's representable slot range; it cannot infer the
runtime block at arbitrary coordinates.

`ender_chest(index)` supports observation through the existing NBT view, but
slot writes reject it because vanilla has no corresponding item-command slot.
There is no atomic move/transfer guarantee. Removal, custom-item count matching,
and inventory-change observation remain separate capability work; use only the
operations documented here as supported gameplay actions.

## Custom-item identity

`CustomItem::item_predicate()` builds the same typed `ItemPredicate` used by
advancement and loot APIs. It returns a `Result`: invalid base IDs and invalid
item definitions produce diagnostics rather than panicking.

```rust,ignore
let wand = CustomItem::new("minecraft:stick").custom_data("my_pack:wand");
let identity = wand.item_predicate()?;
```

Identity matching checks the base item and partially matches its custom data.
Other packs can add unrelated custom-data keys without breaking recognition;
cosmetic components and durability are not identity constraints. Explicit raw
custom data supplied with `typed_custom_data` is retained as an SNBT partial
predicate. Raw component overrides of `custom_data` are rejected by this
conversion because they bypass its identity source.

`on_use_advancement` and `on_use_fn` also return a `Result`, propagating these
identity diagnostics. They use vanilla's using-item trigger (for example,
eating or blocking), which is not a general right-click event.
