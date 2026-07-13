# Advancement Triggers

Builders in `sand_core::event::trigger` create `AdvancementTrigger` values, including `UsingItemTrigger`, `ConsumeItemTrigger`, `ItemObtainedTrigger`, `RecipeUnlockedTrigger`, `PlayerInteractedWithEntityTrigger`, and `SummonedEntityTrigger`. `ItemObtainedTrigger` maps to `minecraft:crafted_item`, so it fires only when an item is crafted—not when an item is smelted or otherwise acquired.

```rust
PlayerInteractedWithEntityTrigger::new()
 .item(ItemPredicate::id("minecraft:stick"))
 .entity(EntityPredicate::new().nbt("{Tags:[\"altar\"]}"))
 .build();
```

Use typed predicates first; `RawJson` is an explicit escape hatch for a version field Sand does not model. See [Event Trigger Matrix](../reference/event-trigger-matrix.md).

Resource-bearing trigger variants also provide typed constructors. These keep
block, dimension, potion, effect, recipe, loot-table, and custom trigger IDs on
validated registry/resource-location paths while preserving the same JSON:

```rust
use sand_core::{AdvancementTrigger, BlockId, DimensionId, PotionRegistryId};

let placed = AdvancementTrigger::placed_block(
    Some(BlockId::minecraft("stone")?),
    None,
    None,
    None,
);
let changed_dimension = AdvancementTrigger::changed_dimension(
    Some(DimensionId::minecraft("overworld")?),
    Some(DimensionId::minecraft("the_nether")?),
);
let brewed = AdvancementTrigger::brewed_potion(
    PotionRegistryId::minecraft("swiftness")?,
);
# Ok::<(), sand_core::SandError>(())
```

The public string-field enum variants remain compatibility/raw paths. Prefer
the associated constructors for new code. `custom_trigger(ResourceLocation,
Option<RawJson>)` validates the modded trigger ID while leaving its conditions
intentionally opaque.

## Version-aware rendering: `placed_block` and `item_used_on_block`

`AdvancementTrigger::PlacedBlock` and `AdvancementTrigger::ItemUsedOnBlock` are
the two variants whose *item* filter has a version-dependent JSON schema. On
every currently supported Minecraft target (the 1.20.5+ item-component era —
`ComponentFeature::ItemComponents`), Sand lowers the `block`/`item`/`state`
fields into the modern loot-condition array Minecraft actually evaluates:

```rust
use sand_core::{AdvancementTrigger, BlockId, ItemPredicate};

let trigger = AdvancementTrigger::placed_block(
    Some(BlockId::minecraft("white_wool")?),
    Some(ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")),
    None,
    None,
);
# Ok::<(), sand_core::SandError>(())
```

renders as:

```json
{
  "trigger": "minecraft:placed_block",
  "conditions": {
    "location": [
      {
        "condition": "minecraft:location_check",
        "predicate": { "block": { "blocks": ["minecraft:white_wool"] } }
      },
      {
        "condition": "minecraft:match_tool",
        "predicate": {
          "items": ["minecraft:white_wool"],
          "predicates": { "minecraft:custom_data": "{elevator:1b}" }
        }
      }
    ]
  }
}
```

Earlier Sand versions emitted `conditions.block` / `conditions.item` directly,
which vanilla silently ignores for this trigger — the generated advancement
fired for **every** block placement regardless of the configured filter. See
[#233](https://github.com/ThatOneToast/sand/issues/233) and
[#232](https://github.com/ThatOneToast/sand/issues/232).

This rendering is version-profile-aware, not a hardcoded special case: the
export pipeline resolves the target `VersionCaps` and calls
`AdvancementTrigger::render_for(caps)`, which selects the trigger's schema
family per profile. Targets that predate the item-component system keep the
legacy flat shape unchanged, since their vanilla schema never had
`location_check`/`match_tool` wrapping for these two triggers. Passing `None`
(the unprofiled compatibility export path, and the plain `Serialize` impl used
by tests) defaults to the modern schema.

`render_for` never silently drops a filter: supplying both the trigger-level
`block`/`state` shorthand *and* a `location` predicate that already sets its
own `block` fails with an actionable `SandError` instead of picking one side.

### `components` vs `predicates`, exact vs partial matching

`ItemPredicate::custom_data_key(...)` lowers to a **partial** NBT match under
`predicates.minecraft:custom_data` (an SNBT string), not an exact `components`
equality check. Partial matching is required here: the item may carry
additional `custom_data` keys added by other packs/features and still match —
exact `components` equality would reject any item whose `custom_data` differs
by even one unrelated key. `ItemPredicate::raw_components(...)` remains
available as an explicit escape hatch for genuine exact-match component
predicates; `ItemPredicate::raw_predicates(...)` merges additional raw
partial-match sub-predicates into the same `predicates` bag.

`items` always serializes as a JSON array (`["minecraft:white_wool"]`), even
for a single item — the previous single-element-collapses-to-a-bare-string
behavior was itself part of the #233 regression, since vanilla's array-typed
fields do not accept a bare string in the modern schema.

### Why placement events must evaluate the event-time item

`minecraft:match_tool` checks the *item stack used for the interaction that
fired the trigger*, evaluated at trigger time — not a snapshot taken earlier
or a re-query of the player's current held item after the fact. This is why
the item filter lives inside the same `conditions.location` condition array as
the block filter: both are evaluated together against the triggering event.

## Generated `requirements`

`Advancement::try_to_json` always emits a top-level `requirements` array.
Minecraft treats a missing/empty `requirements` array as "no criteria
required," which makes the advancement fire unconditionally regardless of how
restrictive the criteria conditions are — this was the other half of #233's
regression (the generated event advancement never set `requirements` at all).

When `Advancement::requirements(...)` is not called explicitly, Sand derives a
single AND-group covering every defined criterion, sorted by name:

```rust
// one criterion named "event" → requirements: [["event"]]
// criteria "a" and "b"        → requirements: [["a", "b"]]
```

This is not a hardcoded `"event"` special case — it reads the actual criterion
names from the advancement. Call `.requirements(...)` explicitly for OR
semantics across criteria groups; `Advancement::validate()` rejects explicit
requirement groups that are empty or reference an unknown criterion name.
