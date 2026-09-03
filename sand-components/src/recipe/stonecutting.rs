//! Stonecutter recipe builder (`minecraft:stonecutting`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::StonecuttingRecipe",
    aliases = ["sand::prelude::StonecuttingRecipe"],
    module = "sand::component",
    summary = "Represents a stonecutter recipe for cutting stone blocks into other shapes.",
    context = "Represents a stonecutter recipe for cutting stone blocks into other shapes. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::StonecuttingRecipe;",
    fields(location = "`location` is used when cutting stone blocks into other shapes."),
)]
/// Represents a stonecutter recipe for cutting stone blocks into other shapes.
pub struct StonecuttingRecipe {
    /// `location` is used when cutting stone blocks into other shapes.
    pub location: ResourceLocation,
    group: Option<String>,
    ingredient: Ingredient,
    result: RecipeResult,
    count: u32,
}

impl StonecuttingRecipe {
    /// Creates a new stonecutter recipe with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StonecuttingRecipe::new",
        aliases = ["sand::prelude::StonecuttingRecipe::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new stonecutter recipe with the given resource location.",
        context = "Creates a new stonecutter recipe with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new stonecutter recipe with the given resource location."),
        returns = "A `StonecuttingRecipe` representing a new stonecutter recipe with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let stonecutting_recipe = sand::component::StonecuttingRecipe::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            ingredient: Ingredient::empty(),
            result: RecipeResult::empty(),
            count: 1,
        }
    }

    /// Sets the ingredient to be cut by the stonecutter.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StonecuttingRecipe::ingredient",
        aliases = ["sand::prelude::StonecuttingRecipe::ingredient"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the ingredient to be cut by the stonecutter.",
        context = "Sets the ingredient to be cut by the stonecutter. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(ingredient = "`ingredient` provides the ingredient applied when setting the ingredient to be cut by the stonecutter."),
        returns = "The `StonecuttingRecipe` value with the documented change applied to set the ingredient to be cut by the stonecutter.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stonecutting_recipe_value: sand::component::StonecuttingRecipe, ingredient: sand::component::Ingredient)  {\n    let updated_stonecutting_recipe = stonecutting_recipe_value.ingredient(ingredient);\n}",
    )]
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StonecuttingRecipe::result",
        aliases = ["sand::prelude::StonecuttingRecipe::result"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the result item and quantity produced by this recipe.",
        context = "Sets the result item and quantity produced by this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(result = "`result` provides the result applied when setting the result item and quantity produced by this recipe."),
        returns = "The `StonecuttingRecipe` value with the documented change applied to set the result item and quantity produced by this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stonecutting_recipe_value: sand::component::StonecuttingRecipe, result: sand::component::RecipeResult)  {\n    let updated_stonecutting_recipe = stonecutting_recipe_value.result(result);\n}",
    )]
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the quantity of the result produced.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StonecuttingRecipe::count",
        aliases = ["sand::prelude::StonecuttingRecipe::count"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the quantity of the result produced.",
        context = "Sets the quantity of the result produced. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(count = "`count` provides the requested numeric amount used to set the quantity of the result produced."),
        returns = "The `StonecuttingRecipe` value with the documented change applied to set the quantity of the result produced.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stonecutting_recipe_value: sand::component::StonecuttingRecipe, count: u32)  {\n    let updated_stonecutting_recipe = stonecutting_recipe_value.count(count);\n}",
    )]
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StonecuttingRecipe::group",
        aliases = ["sand::prelude::StonecuttingRecipe::group"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `StonecuttingRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stonecutting_recipe_value: sand::component::StonecuttingRecipe, g: impl Into < String >)  {\n    let updated_stonecutting_recipe = stonecutting_recipe_value.group(g);\n}",
    )]
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    fn try_build_json(&self) -> SandResult<Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".into(),
            Value::String("minecraft:stonecutting".into()),
        );
        if let Some(group) = &self.group {
            map.insert("group".into(), Value::String(group.clone()));
        }
        map.insert(
            "ingredient".into(),
            serde_json::to_value(&self.ingredient).map_err(SandError::from)?,
        );
        map.insert(
            "result".into(),
            serde_json::to_value(&self.result).map_err(SandError::from)?,
        );
        map.insert("count".into(), Value::from(self.count));
        Ok(Value::Object(map))
    }
}

impl DatapackComponent for StonecuttingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.ingredient.validate_at(&self.location, "ingredient")?;
        self.result.validate_at(&self.location, "result")?;
        if self.count == 0 {
            return Err(error(
                &self.location,
                "count",
                "stonecutting result count must be at least 1",
            ));
        }
        Ok(())
    }
    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!(
                "StonecuttingRecipe::to_json() failed for {}: {e}",
                self.location
            )
        })
    }
    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(ComponentContent::Json(self.try_build_json()?))
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }

    fn required_features(&self) -> &'static [ComponentFeature] {
        if self.result.has_components() {
            &[ComponentFeature::ItemComponents]
        } else {
            &[]
        }
    }
}

fn error(location: &ResourceLocation, field: &str, message: &str) -> SandError {
    SandError::ComponentValidation {
        location: location.clone(),
        kind: "recipe".into(),
        field: field.into(),
        message: message.into(),
    }
}
