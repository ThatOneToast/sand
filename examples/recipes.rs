//! # Recipes
//!
//! Demonstrates creating all recipe types: shaped, shapeless, cooking,
//! stonecutting, and smithing.

use sand_core::{
    CookingRecipe, CookingType, Ingredient, RecipeResult, ShapedRecipe,
    ShapelessRecipe, SmithingTransformRecipe, SmithingTrimRecipe,
    StonecuttingRecipe,
};
use sand_macros::component;

// ── Shaped crafting recipe ───────────────────────────────────────────────────
// A 3x3 grid recipe — the pattern defines the shape.

#[component]
pub fn diamond_hammer() -> ShapedRecipe {
    ShapedRecipe::new("my_pack:diamond_hammer".parse().unwrap())
        .pattern(vec!["DDD", " S ", " S "])
        .key('D', Ingredient::item("minecraft:diamond"))
        .key('S', Ingredient::item("minecraft:stick"))
        .result(RecipeResult::new("minecraft:diamond_pickaxe", 1))
        .group("tools")
        .show_notification(true)
}

// ── Shapeless crafting recipe ────────────────────────────────────────────────
// Order doesn't matter — just requires the right ingredients anywhere in the grid.

#[component]
pub fn packed_mud() -> ShapelessRecipe {
    ShapelessRecipe::new("my_pack:packed_mud".parse().unwrap())
        .ingredient(Ingredient::item("minecraft:mud"))
        .ingredient(Ingredient::item("minecraft:wheat"))
        .result(RecipeResult::new("minecraft:packed_mud", 1))
}

// ── Smelting recipe ──────────────────────────────────────────────────────────
// Works in a furnace. Other cooking types: Blasting, Smoking, CampfireCooking.

#[component]
pub fn smelt_custom_ore() -> CookingRecipe {
    CookingRecipe::new(
        "my_pack:smelt_raw_iron".parse().unwrap(),
        CookingType::Smelting,
    )
    .ingredient(Ingredient::item("minecraft:raw_iron"))
    .result("minecraft:iron_ingot")
    .experience(0.7)
    .cooking_time(200) // ticks (10 seconds)
    .category("misc")
}

// ── Blast furnace recipe ─────────────────────────────────────────────────────

#[component]
pub fn blast_custom_ore() -> CookingRecipe {
    CookingRecipe::new(
        "my_pack:blast_raw_iron".parse().unwrap(),
        CookingType::Blasting,
    )
    .ingredient(Ingredient::item("minecraft:raw_iron"))
    .result("minecraft:iron_ingot")
    .experience(0.7)
    .cooking_time(100) // half the time of regular smelting
}

// ── Stonecutting recipe ──────────────────────────────────────────────────────

#[component]
pub fn cut_stone_bricks() -> StonecuttingRecipe {
    StonecuttingRecipe::new("my_pack:cut_stone_bricks".parse().unwrap())
        .ingredient(Ingredient::item("minecraft:stone"))
        .result("minecraft:stone_bricks")
        .count(1)
}

// ── Smithing transform recipe ────────────────────────────────────────────────
// Used at a smithing table to transform items (e.g. diamond -> netherite).

#[component]
pub fn custom_upgrade() -> SmithingTransformRecipe {
    SmithingTransformRecipe::new("my_pack:custom_upgrade".parse().unwrap())
        .template(Ingredient::item("minecraft:netherite_upgrade_smithing_template"))
        .base(Ingredient::item("minecraft:diamond_sword"))
        .addition(Ingredient::item("minecraft:netherite_ingot"))
        .result("minecraft:netherite_sword")
}

// ── Smithing trim recipe ─────────────────────────────────────────────────────
// Applies decorative trims to armor.

#[component]
pub fn custom_trim() -> SmithingTrimRecipe {
    SmithingTrimRecipe::new("my_pack:custom_trim".parse().unwrap())
        .template(Ingredient::item("minecraft:coast_armor_trim_smithing_template"))
        .base(Ingredient::tag("minecraft:trimmable_armor"))
        .addition(Ingredient::tag("minecraft:trim_materials"))
}

// ── Recipe with tag ingredients ──────────────────────────────────────────────
// Tags match any item in the tag group (e.g. any type of planks).

#[component]
pub fn any_planks_to_sticks() -> ShapedRecipe {
    ShapedRecipe::new("my_pack:planks_to_sticks".parse().unwrap())
        .pattern(vec!["P", "P"])
        .key('P', Ingredient::tag("minecraft:planks"))
        .result(RecipeResult::new("minecraft:stick", 4))
}
