# Item Stacks And Matchers

`ItemStack`, `ItemMatcher`, and `CustomItemDefinition` are a shared foundation
for item identity: one definition that produces consistent give-able stacks,
detection matchers, and adapters into the existing predicate/recipe APIs,
instead of repeating a base item ID and `custom_data` marker across every
surface that needs it.

This is phase 1 of issue #229. `ItemLocation` (typed entity/block accessors)
and `ItemSnapshot` (immutable event-time item data) are **not** part of this
phase — they land in a follow-up once this foundation is stable.

## Stack vs matcher

- **`ItemStack`** represents a concrete item that exists — something to give,
  craft, or place in a container. It wraps `CustomItem`'s existing component
  model with a typed `ItemId` and a validated count.
- **`ItemMatcher`** represents a *condition* used to detect an item. It never
  represents something that exists, and it is never accepted where a stack is
  required (or vice versa).

```rust
let stack = ItemStack::new(ItemId::minecraft("bow")?)
    .count(1)
    .component(ItemComponent::custom_data_marker("special_bow"));

let matcher = ItemMatcher::item(ItemId::minecraft("bow")?)
    .custom_data_partial("special_bow");
```

## `CustomItemDefinition`

A `CustomItemDefinition` ties one base item ID and `custom_data` marker to
every representation that identity needs to appear in:

```rust
let special_bow = CustomItemDefinition::new(ItemId::minecraft("bow")?)
    .marker("special_bow")
    .component(ItemComponent::item_name(TextComponent::literal("Special Bow")));

inventory.give(special_bow.stack(1));
let matcher = special_bow.matcher();
let result = special_bow.try_recipe_result(1)?;
```

`special_bow.stack(1)` and `special_bow.matcher()` always share the same base
item ID and marker — there's no second place to keep them in sync.

Note `CustomItemDefinition` is not const-constructible (it owns `String`/`Vec`
state, the same as `CustomItem`). Build one from a plain function, or wrap it
in `std::sync::LazyLock` for a `static`.

## Exact vs partial matching

`ItemMatcher` keeps exact and partial semantics as separate, explicitly named
methods rather than overloading one method whose meaning would otherwise
silently vary by consumer:

| Method | Semantics |
|---|---|
| `custom_data_exact(data)` | The item's `minecraft:custom_data` must equal `data` exactly — any extra key fails the match. |
| `custom_data_partial(key)` | The item's `minecraft:custom_data` must contain `key`, ignoring any other keys. This is the correct way to detect "is this one of my custom items." |
| `raw_components_exact(json)` | Escape hatch — exact-match arbitrary component JSON. |
| `raw_predicates_partial(json)` | Escape hatch — partial-match arbitrary sub-predicate JSON. |

`CustomItemDefinition::matcher()` always uses **partial** custom-data
matching, since that's the correct semantics for identity detection — other
packs may add unrelated `custom_data` keys to the same item.

## Consumer- and version-aware conversion

Converting an `ItemMatcher` into a concrete `predicates::ItemPredicate`
depends on *where* it's being used and *which* Minecraft version is
targeted — the same item filter renders differently on a modern
(1.20.5+/26.x) profile than on a legacy one, and some consumers (recipe
ingredients) can't represent component matching at all.

```rust
let predicate = matcher.try_render_for(
    ItemMatcherConsumer::Predicate,
    Some(&caps),
)?;
```

`ItemMatcherConsumer` names the exact consuming surface (`Advancement`,
`Predicate`, `RecipeIngredient`, `InventoryCondition`, `LootCondition`) so
diagnostics can say precisely what was rejected and why, rather than emitting
a JSON shape the target doesn't recognize:

```text
component `sand:item_matcher` (advancement trigger `minecraft:placed_block`):
advancement trigger `minecraft:placed_block` requested item-component matching
(custom data, enchantments, damage, or raw component/predicate constraints),
but the target Minecraft profile is a pre-item-component profile (predates
1.20.5). ... [field: components]
```

This is the same seam `AdvancementTrigger::render_for` uses internally (see
[Advancement Triggers](advancement-triggers.md)) — `AdvancementItemConsumer`
from that page converts into `ItemMatcherConsumer::Advancement` for free, and
`try_into_advancement_predicate` is a convenience wrapper:

```rust
let predicate = matcher.try_into_advancement_predicate(
    AdvancementItemConsumer::PlacedBlockTool,
    Some(&caps),
)?;
```

## Recipe integration

`TryIntoIngredient`/`TryIntoRecipeResult` integrate `ItemMatcher`/`ItemStack`
with the existing `Ingredient`/`RecipeResult` types without changing their
public API:

```rust
let ingredient = ItemMatcher::item(ItemId::minecraft("oak_planks")?)
    .try_into_ingredient()?;

let result = ItemStack::new(ItemId::minecraft("white_wool")?)
    .component(ItemComponent::custom_data_marker("elevator_block_item"))
    .try_into_recipe_result(1)?;
```

A matcher that constrains any data component (custom data, enchantments,
damage, raw component/predicate) always fails `try_into_ingredient` — vanilla
recipe ingredients only match by item ID or tag in every Minecraft version
Sand targets, so converting one would silently accept any item of the same
base type. This mirrors `Ingredient::custom_item`'s existing behavior.

## Compatibility

Existing `CustomItem`, `ItemPredicate`, `Ingredient`, and `RecipeResult` APIs
are unchanged and remain fully supported — `ItemStack` wraps `CustomItem`
internally rather than replacing it, and `CustomItemDefinition::as_custom_item()`
converts back to a `CustomItem` for code that still expects one (e.g.
`sand_core::custom_item_ext`).
