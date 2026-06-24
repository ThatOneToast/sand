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
pub use types::{CookingType, Ingredient, RecipeResult};

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
}
