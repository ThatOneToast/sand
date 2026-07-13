# Datapack Components

Plain `#[component]` functions return typed datapack JSON components:

```rust
#[component]
pub fn welcome_advancement() -> Advancement {
    Advancement::new("example:welcome".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
}
```

Use typed builders for advancements, recipes, loot tables, predicates, tags,
dialogs, and custom item data.

Recipe item and tag identifiers use registry-specific types on the preferred
path. Generated vanilla items work directly, while custom tags retain their
item-registry marker:

```rust
use sand_core::{Ingredient, ItemId, RecipeResult, TagId, generated::Item};

let diamond = Ingredient::item_id(Item::Diamond);
let planks = Ingredient::item_tag(
    TagId::<ItemId>::minecraft("planks").unwrap(),
);
let result = RecipeResult::item(Item::DiamondSword, 1);
```

Use `raw_item`, `raw_tag`, or `RecipeResult::raw` only as explicit compatibility
escape hatches for identifiers Sand cannot model yet.

Structure templates are binary `.nbt` assets and are copied into the datapack
rather than generated as Rust data:

```rust
#[component]
pub fn starter_room() -> StructureTemplate {
    StructureTemplate::new(
        ResourceLocation::new("example", "rooms/starter").unwrap(),
        "src/structures/starter.nbt",
    )
}
```

This writes `src/structures/starter.nbt` to
`data/example/structure/rooms/starter.nbt`. Sand rejects unsafe paths and
requires structure template assets to use `.nbt`.

Custom items use typed item components instead of handwritten component SNBT:

```rust
let wand = CustomItem::new(ItemId::minecraft("stick").unwrap())
    .id("example:dash_wand")
    .component(ItemComponent::custom_name(Text::new("Dash Wand").aqua()))
    .component(ItemComponent::lore(vec![Text::new("Right click to dash").gray()]))
    .component(ItemComponent::custom_model_data(1001))
    .component(ItemComponent::rarity(Rarity::Rare))
    .component(ItemComponent::max_stack_size(1));
```

For modded or future components, use an explicit `RawComponent` escape hatch:

```rust
let relic = CustomItem::new("minecraft:amethyst_shard")
    .with_raw_component(RawComponent::new("mymod:charge", "{value:3}"));
```

## Crafting a component-bearing `CustomItem`

A `CustomItem` can be used directly as a recipe result — the base item and its
data components (identity markers, display text, glint override, etc.) are
carried through into the recipe JSON's `result.components` object, so the
crafted item keeps its custom identity rather than being reduced to a plain
vanilla item:

```rust
let elevator = CustomItem::new("minecraft:white_wool")
    .custom_data("elevator_block_item")
    .component(ItemComponent::EnchantmentGlintOverride(true))
    .item_name(
        TextComponent::literal("Elevator Block")
            .bold(true)
            .color(ChatColor::Aqua),
    );

#[component]
pub fn elevator_block_recipe() -> ShapedRecipe {
    ShapedRecipe::new("example:elevator_block".parse().unwrap())
        .pattern(["WWW", "WGW", "WWW"])
        .key('W', Ingredient::item("minecraft:white_wool"))
        .key('G', Ingredient::item("minecraft:glowstone"))
        // Defaults the result count to 1.
        .result(RecipeResult::custom_item(&elevator).unwrap())
}
```

Use `RecipeResult::from_custom_item(&elevator, count)` (or
`elevator.recipe_result(count)`, or `RecipeResult::try_from(&elevator)` /
`TryFrom<CustomItem>`) to select a count other than 1. All of these are
fallible — they return `Err` rather than silently dropping a component that
cannot be safely represented, which is the only failure mode: a raw component
(`with_raw_component`) whose value is genuine SNBT (not also valid JSON), or
`custom_data` set via `typed_custom_data(CustomData::raw(...))`. Both cases
have no general SNBT→JSON conversion, so Sand rejects them explicitly instead
of guessing.

Component-free results (`RecipeResult::item`, `::raw`, `::new`) serialize
exactly as before — `{"id": ..., "count": ...}` with no `components` key —
so existing recipes are unaffected.

**Supported Minecraft versions:** component-bearing recipe results require
the 1.20.5+ item data component system. Exporting one for an older target (or
an unverified/fallback version profile) fails with a clear `VersionGating`
error naming the missing `item_components` capability and the requested
version — it is never silently stripped down to a component-free result.
`mc_version = "latest"` always resolves to a version that supports it.

**Component-aware ingredients are not supported.** Vanilla's crafting recipe
`Ingredient` schema (shaped, shapeless, cooking, stonecutting, smithing) only
ever matches by item ID or item tag — in every Minecraft version Sand
targets, there is no way to require a specific data component (like
`elevator_block_item`) in the ingredient slot. `Ingredient::custom_item(...)`
exists but always returns a descriptive error rather than silently matching
on the base item alone, which would let any plain `minecraft:white_wool`
satisfy the recipe. If you need to verify a crafted item's identity, check
`minecraft:custom_data` in a follow-up function or predicate instead.
