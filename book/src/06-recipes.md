# 6. Recipes

Trailforge makes the Grapple Core craftable with a standard shaped recipe:

```rust,ignore
{{#include ../../examples/book_project/src/lib.rs:recipe}}
```

`ShapedRecipe` mirrors vanilla's shaped-crafting JSON schema directly: a
3×3 `pattern` of single-character keys, a `key(...)` mapping from character
to `Ingredient`, a `result`, and an optional `category` used by the recipe
book UI. This is the same shape you'd hand-write in
`data/trail/recipe/grapple_core.json` — Sand's value here is catching
mistakes Rust can check (an unmapped pattern character, a malformed item
ID) at `cargo build` time instead of at datapack load, where Minecraft's
own error reporting for malformed recipe JSON is much harder to trace back
to a cause.

## Preserve the custom item's identity

`RecipeResult::custom_item(&grapple_core())` carries the base item, custom-data
marker, name, lore, rarity, and stack limit into the recipe result. The same
`grapple_core()` definition supplies the upgrade's inventory predicate and the
acquisition event's identity. A plain Heart of the Sea does not qualify.

This uses the supported component-bearing recipe result; it needs no reward
function workaround. Recipe **ingredients** still match vanilla items or tags,
not custom-data identity. Use an inventory predicate and an explicit gameplay
function when a cost must match a custom item.

The upgrade function consumes one matching core from the current player's
inventory and grants Trail Striders. It does not consume another player's core
or allow an already-unlocked player to claim another pair.

## `#[datapack_component]` vs `#[custom_item]` vs `#[function]`

Notice `grapple_core_recipe` uses the bare `#[datapack_component]` attribute, the
same one `load` and `tick` use with an explicit `(Load)`/`(Tick)` kind.
`#[datapack_component]` is Sand's general "export this as a datapack component"
macro; recipes, advancements, loot tables, and predicates all register this
way, distinguished by their return type (`ShapedRecipe` here) rather than by
a different macro. Items use the more specific `#[custom_item]` (chapter 5, when
you need the generated predicate) or no macro at all (a plain builder
function called where needed, like `trail_striders`). Functions use
`#[function]`. Keeping these three macros distinct — rather than one
do-everything macro — is what lets each one specialize its generated code:
`#[custom_item]` generates a predicate struct; `#[function]` registers a callable
`FunctionId` target usable from `cmd::function` and callback APIs.
