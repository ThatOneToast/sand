# 6. Recipes

Trailforge makes the Grapple Core craftable with a standard shaped recipe:

{{#include ../../examples/book_project/src/lib.rs:recipe}}

`ShapedRecipe` mirrors vanilla's shaped-crafting JSON schema directly: a
3×3 `pattern` of single-character keys, a `key(...)` mapping from character
to `Ingredient`, a `result`, and an optional `category` used by the recipe
book UI. This is the same shape you'd hand-write in
`data/trail/recipe/grapple_core.json` — Sand's value here is catching
mistakes Rust can check (an unmapped pattern character, a malformed item
ID) at `cargo build` time instead of at datapack load, where Minecraft's
own error reporting for malformed recipe JSON is much harder to trace back
to a cause.

## Why the recipe result is `minecraft:heart_of_the_sea`, not `GrappleCore`

`RecipeResult::new("minecraft:heart_of_the_sea", 1)` — the recipe crafts
the vanilla base item, not a component-decorated custom item. This is a
real vanilla limitation, not a Sand gap: crafting-recipe JSON in Minecraft
can only reference a base item ID plus (on modern versions) fixed result
components, and can't invoke arbitrary custom-item construction logic the
way a `give` command backed by `CustomItem::new(...)` can. Trailforge works
around this the standard way: `trail:claim_striders` (chapter 8) is a
*function*, gated on already holding a plain Grapple Core, that manually
`give`s the fully-decorated item. If you need "crafting always produces the
exact custom item," route the recipe's raw result through a loot-table-style
advancement/function combo rather than expecting the recipe itself to carry
full item component data — see
[Vanilla Limitations](reference/vanilla-limitations.md).

## `#[component]` vs `#[item]` vs `#[function]`

Notice `grapple_core_recipe` uses the bare `#[component]` attribute, the
same one `load` and `tick` use with an explicit `(Load)`/`(Tick)` kind.
`#[component]` is Sand's general "export this as a datapack component"
macro; recipes, advancements, loot tables, and predicates all register this
way, distinguished by their return type (`ShapedRecipe` here) rather than by
a different macro. Items use the more specific `#[item]` (chapter 5, when
you need the generated predicate) or no macro at all (a plain builder
function called where needed, like `trail_striders`). Functions use
`#[function]`. Keeping these three macros distinct — rather than one
do-everything macro — is what lets each one specialize its generated code:
`#[item]` generates a predicate struct; `#[function]` registers a callable
`ResourceLocation` target usable from `cmd::call` and `cmd::function`.
