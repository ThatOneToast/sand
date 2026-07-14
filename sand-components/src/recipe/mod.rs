//! Recipe builders for `data/<namespace>/recipe/` JSON files (Minecraft 1.21+).
//!
//! Each recipe type lives in its own submodule:
//!
//! | Module          | Type(s)                                         |
//! |----------------|-------------------------------------------------|
//! | `shaped`       | [`ShapedRecipe`]                                |
//! | `shapeless`    | [`ShapelessRecipe`]                             |
//! | `cooking`      | [`CookingRecipe`]                               |
//! | `stonecutting` | [`StonecuttingRecipe`]                          |
//! | `smithing`     | [`SmithingTransformRecipe`], [`SmithingTrimRecipe`] |
//! | `types`        | [`Ingredient`], [`RecipeResult`], [`CookingType`] (shared) |

pub mod cooking;
pub mod shaped;
pub mod shapeless;
pub mod smithing;
pub mod stonecutting;
pub mod types;

pub use cooking::CookingRecipe;
pub use shaped::ShapedRecipe;
pub use shapeless::ShapelessRecipe;
pub use smithing::{SmithingTransformRecipe, SmithingTrimRecipe};
pub use stonecutting::StonecuttingRecipe;
pub use types::{
    CookingType, Ingredient, IntoRecipeItemId, RecipeResult, TryIntoIngredient, TryIntoRecipeResult,
};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{DatapackComponent, ResourceLocation};

    use super::{
        CookingRecipe, CookingType, Ingredient, RecipeResult, ShapedRecipe, ShapelessRecipe,
        SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
    };

    fn id(path: &str) -> ResourceLocation {
        ResourceLocation::new("audit", path).unwrap()
    }

    fn result() -> RecipeResult {
        RecipeResult::new("audit:result", 2)
    }

    #[test]
    fn supported_recipe_builders_emit_latest_schema() {
        let shaped = ShapedRecipe::new(id("shaped"))
            .pattern(["PNP", "PPP", "IPI"])
            .key('P', Ingredient::item("minecraft:oak_planks"))
            .key('N', Ingredient::item("minecraft:netherite_ingot"))
            .key('I', Ingredient::item("minecraft:iron_ingot"))
            .result(result())
            .to_json();
        assert_eq!(shaped["key"]["P"], json!("minecraft:oak_planks"));
        assert_eq!(shaped["result"], json!({"id": "audit:result", "count": 2}));

        let shapeless = ShapelessRecipe::new(id("shapeless"))
            .ingredient(Ingredient::tag("minecraft:planks"))
            .result(result())
            .to_json();
        assert_eq!(shapeless["ingredients"], json!(["#minecraft:planks"]));

        for recipe_type in [
            CookingType::Smelting,
            CookingType::Blasting,
            CookingType::Smoking,
            CookingType::CampfireCooking,
        ] {
            let cooking = CookingRecipe::new(id("cooking"), recipe_type)
                .ingredient(Ingredient::item("minecraft:potato"))
                .result(result())
                .to_json();
            assert_eq!(cooking["ingredient"], json!("minecraft:potato"));
            assert_eq!(cooking["result"], json!({"id": "audit:result", "count": 2}));
        }

        let stonecutting = StonecuttingRecipe::new(id("stonecutting"))
            .ingredient(Ingredient::item("minecraft:stone"))
            .result(result())
            .to_json();
        assert_eq!(stonecutting["ingredient"], json!("minecraft:stone"));
        assert_eq!(
            stonecutting["result"],
            json!({"id": "audit:result", "count": 2})
        );

        let transform = SmithingTransformRecipe::new(id("transform"))
            .template(Ingredient::item(
                "minecraft:netherite_upgrade_smithing_template",
            ))
            .base(Ingredient::item("minecraft:diamond_sword"))
            .addition(Ingredient::item("minecraft:netherite_ingot"))
            .result(result())
            .to_json();
        assert_eq!(
            transform["template"],
            json!("minecraft:netherite_upgrade_smithing_template")
        );
        assert_eq!(
            transform["result"],
            json!({"id": "audit:result", "count": 2})
        );

        let trim = SmithingTrimRecipe::new(id("trim"))
            .template(Ingredient::item(
                "minecraft:sentry_armor_trim_smithing_template",
            ))
            .base(Ingredient::item("minecraft:diamond_chestplate"))
            .addition(Ingredient::item("minecraft:amethyst_shard"))
            .to_json();
        assert_eq!(trim["addition"], json!("minecraft:amethyst_shard"));
    }

    // ── ShapedRecipe fallible validation contract (#145) ────────────────────────

    #[test]
    fn valid_shaped_recipe_try_content_preserves_output() {
        let recipe = ShapedRecipe::new(id("valid"))
            .pattern(["PNP", "PPP", "IPI"])
            .key('P', Ingredient::item("minecraft:oak_planks"))
            .key('N', Ingredient::item("minecraft:netherite_ingot"))
            .key('I', Ingredient::item("minecraft:iron_ingot"))
            .result(RecipeResult::new("audit:result", 2));

        // try_content() must succeed and produce the same output as to_json().
        let content = recipe.try_content().expect("valid recipe should pass");
        let json = match content {
            crate::component::ComponentContent::Json(v) => v,
            _ => panic!("expected JSON content"),
        };
        assert_eq!(json, recipe.to_json());

        // Validate() must also succeed.
        recipe.validate().expect("valid recipe should validate");
    }

    #[test]
    fn empty_pattern_fails_validation() {
        let recipe = ShapedRecipe::new(id("no_pattern"))
            .key('X', Ingredient::item("minecraft:stone"))
            .result(RecipeResult::new("audit:result", 1));
        // pattern defaults to empty Vec — no need to call .pattern().

        let err = recipe.validate().expect_err("empty pattern should fail");
        assert!(err.to_string().contains("pattern"), "err: {err}");
        assert!(err.to_string().contains("audit:no_pattern"), "err: {err}");
        assert!(err.to_string().contains("recipe"), "err: {err}");
    }

    #[test]
    fn empty_result_id_fails_validation() {
        let recipe = ShapedRecipe::new(id("no_result"))
            .pattern(["X"])
            .key('X', Ingredient::item("minecraft:stone"))
            .result(RecipeResult::new("", 1));

        let err = recipe.validate().expect_err("empty result id should fail");
        assert!(err.to_string().contains("result.id"), "err: {err}");
    }

    #[test]
    fn zero_result_count_fails_validation() {
        let recipe = ShapedRecipe::new(id("zero_count"))
            .pattern(["X"])
            .key('X', Ingredient::item("minecraft:stone"))
            .result(RecipeResult::new("audit:result", 0));

        let err = recipe.validate().expect_err("zero count should fail");
        assert!(err.to_string().contains("result.count"), "err: {err}");
    }

    #[test]
    fn missing_key_for_pattern_char_fails_validation() {
        let recipe = ShapedRecipe::new(id("missing_key"))
            .pattern(["XY"])
            .key('X', Ingredient::item("minecraft:stone"))
            .result(RecipeResult::new("audit:result", 1));

        let err = recipe.validate().expect_err("missing key should fail");
        assert!(err.to_string().contains("'Y'"), "err: {err}");
        assert!(err.to_string().contains("key"), "err: {err}");
    }

    #[test]
    fn unused_key_fails_validation() {
        let recipe = ShapedRecipe::new(id("unused_key"))
            .pattern(["X"])
            .key('X', Ingredient::item("minecraft:stone"))
            .key('Y', Ingredient::item("minecraft:dirt"))
            .result(RecipeResult::new("audit:result", 1));

        let err = recipe.validate().expect_err("unused key should fail");
        assert!(err.to_string().contains("'Y'"), "err: {err}");
    }

    #[test]
    fn empty_ingredient_fails_validation_not_panic() {
        // Ingredient::empty() is crate-private so simulate via alternatives
        // with an empty inner ingredient.
        let empty_inner = Ingredient::alternatives([]);
        let recipe = ShapedRecipe::new(id("empty_ing"))
            .pattern(["X"])
            .key('X', empty_inner)
            .result(RecipeResult::new("audit:result", 1));

        let err = recipe.validate().expect_err("empty ingredient should fail");
        assert!(err.to_string().contains("ingredient"), "err: {err}");
        assert!(err.to_string().contains("empty"), "err: {err}");
    }

    #[test]
    fn try_content_on_invalid_recipe_returns_error_not_panic() {
        let recipe = ShapedRecipe::new(id("invalid")).result(RecipeResult::new("audit:result", 1));

        let result = recipe.try_content();
        assert!(result.is_err(), "try_content must return Err, not panic");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("audit:invalid"), "err: {err}");
        assert!(err.to_string().contains("recipe"), "err: {err}");
    }

    #[test]
    fn golden_valid_shaped_recipe_output_unchanged() {
        let recipe = ShapedRecipe::new(id("golden"))
            .pattern(["AAA", "BCB", "AAA"])
            .key('A', Ingredient::item("minecraft:iron_ingot"))
            .key('B', Ingredient::tag("minecraft:logs"))
            .key('C', Ingredient::item("minecraft:diamond"))
            .category("building")
            .group("iron_frame")
            .show_notification(false)
            .result(RecipeResult::new("audit:iron_frame", 1));

        let json = recipe.to_json();
        assert_eq!(json["type"], json!("minecraft:crafting_shaped"));
        assert_eq!(json["category"], json!("building"));
        assert_eq!(json["group"], json!("iron_frame"));
        assert_eq!(json["pattern"], json!(["AAA", "BCB", "AAA"]));
        assert_eq!(json["key"]["A"], json!("minecraft:iron_ingot"));
        assert_eq!(json["key"]["B"], json!("#minecraft:logs"));
        assert_eq!(json["key"]["C"], json!("minecraft:diamond"));
        assert_eq!(
            json["result"],
            json!({"id": "audit:iron_frame", "count": 1})
        );
        assert_eq!(json["show_notification"], json!(false));

        // try_content must produce byte-identical JSON.
        let content = recipe.try_content().unwrap();
        let v = match content {
            crate::component::ComponentContent::Json(v) => v,
            _ => panic!("expected JSON"),
        };
        assert_eq!(v, json, "try_content output must match to_json output");
    }

    #[test]
    fn shaped_grid_invariants_are_contextual() {
        let uneven = ShapedRecipe::new(id("uneven"))
            .pattern(["XX", "X"])
            .key('X', Ingredient::item("minecraft:stone"))
            .result(result());
        assert!(
            uneven
                .validate()
                .unwrap_err()
                .to_string()
                .contains("pattern[1]")
        );

        let oversized = ShapedRecipe::new(id("oversized"))
            .pattern(["XXXX"])
            .key('X', Ingredient::item("minecraft:stone"))
            .result(result());
        assert!(
            oversized
                .validate()
                .unwrap_err()
                .to_string()
                .contains("pattern[0]")
        );

        let space = ShapedRecipe::new(id("space"))
            .pattern(["X"])
            .key('X', Ingredient::item("minecraft:stone"))
            .key(' ', Ingredient::item("minecraft:air"))
            .result(result());
        assert!(
            space
                .validate()
                .unwrap_err()
                .to_string()
                .contains("key[' ']")
        );
    }

    #[test]
    fn shapeless_validates_count_nested_ingredients_and_result() {
        let empty = ShapelessRecipe::new(id("empty_shapeless")).result(result());
        assert!(
            empty
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("ingredients")
        );

        let nested = ShapelessRecipe::new(id("nested_shapeless"))
            .ingredient(Ingredient::alternatives([Ingredient::alternatives([])]))
            .result(result());
        assert!(
            nested
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("ingredients[0].alternatives[0]")
        );

        let too_many = (0..10).fold(
            ShapelessRecipe::new(id("too_many")).result(result()),
            |recipe, _| recipe.ingredient(Ingredient::item("minecraft:stone")),
        );
        assert!(
            too_many
                .validate()
                .unwrap_err()
                .to_string()
                .contains("at most 9")
        );
    }

    #[test]
    fn cooking_validates_required_fields_finite_experience_and_time() {
        let missing = CookingRecipe::new(id("missing_cooking"), CookingType::Smelting);
        assert!(
            missing
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("ingredient")
        );
        for experience in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let recipe = CookingRecipe::new(id("xp"), CookingType::Smelting)
                .ingredient(Ingredient::item("minecraft:potato"))
                .result(result())
                .experience(experience);
            assert!(
                recipe
                    .try_content()
                    .unwrap_err()
                    .to_string()
                    .contains("experience")
            );
        }
        let zero = CookingRecipe::new(id("time"), CookingType::Smelting)
            .ingredient(Ingredient::item("minecraft:potato"))
            .result(result())
            .cooking_time(0);
        assert!(
            zero.validate()
                .unwrap_err()
                .to_string()
                .contains("cookingtime")
        );
    }

    #[test]
    fn stonecutting_and_smithing_validate_each_required_field() {
        let stone = StonecuttingRecipe::new(id("stone"))
            .ingredient(Ingredient::item("minecraft:stone"))
            .result(result())
            .count(0);
        assert!(
            stone
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("count")
        );

        let transform = SmithingTransformRecipe::new(id("transform_missing"));
        assert!(
            transform
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("template")
        );
        let trim = SmithingTrimRecipe::new(id("trim_missing"))
            .template(Ingredient::item("minecraft:template"));
        assert!(trim.try_content().unwrap_err().to_string().contains("base"));
    }
}
