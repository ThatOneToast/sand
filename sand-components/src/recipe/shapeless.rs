//! Shapeless crafting recipe builder (`minecraft:crafting_shapeless`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ShapelessRecipe",
    aliases = ["sand::prelude::ShapelessRecipe"],
    module = "sand::component",
    summary = "Represents a shapeless crafting recipe where ingredient order and position don't matter.",
    context = "Represents a shapeless crafting recipe where ingredient order and position don't matter. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ShapelessRecipe;",
    fields(ingredients = "`ingredients` provides the ingredients when the variant represents a shapeless crafting recipe where ingredient order and position don't matter.", location = "`location` provides the location identifier when the variant represents a shapeless crafting recipe where ingredient order and position don't matter."),
)]
/// Represents a shapeless crafting recipe where ingredient order and position don't matter.
pub struct ShapelessRecipe {
    /// `location` provides the location identifier when the variant represents a shapeless crafting recipe where ingredient order and position don't matter.
    pub location: ResourceLocation,
    category: Option<String>,
    group: Option<String>,
    /// `ingredients` provides the ingredients when the variant represents a shapeless crafting recipe where ingredient order and position don't matter.
    pub ingredients: Vec<Ingredient>,
    result: RecipeResult,
}

impl ShapelessRecipe {
    /// Creates a new shapeless recipe with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapelessRecipe::new",
        aliases = ["sand::prelude::ShapelessRecipe::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new shapeless recipe with the given resource location.",
        context = "Creates a new shapeless recipe with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new shapeless recipe with the given resource location."),
        returns = "A `ShapelessRecipe` representing a new shapeless recipe with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let shapeless_recipe = sand::component::ShapelessRecipe::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            ingredients: Vec::new(),
            result: RecipeResult::empty(),
        }
    }

    /// Adds an ingredient to the recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapelessRecipe::ingredient",
        aliases = ["sand::prelude::ShapelessRecipe::ingredient"],
        module = "sand::component",
        kind = "method",
        summary = "Adds an ingredient to the recipe.",
        context = "Adds an ingredient to the recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(ingredient = "`ingredient` provides the ingredient added when building an ingredient to the recipe."),
        returns = "The `ShapelessRecipe` value with the documented change applied to add an ingredient to the recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shapeless_recipe_value: sand::component::ShapelessRecipe, ingredient: sand::component::Ingredient)  {\n    let updated_shapeless_recipe = shapeless_recipe_value.ingredient(ingredient);\n}",
    )]
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapelessRecipe::result",
        aliases = ["sand::prelude::ShapelessRecipe::result"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the result item and quantity produced by this recipe.",
        context = "Sets the result item and quantity produced by this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(result = "`result` provides the result applied when setting the result item and quantity produced by this recipe."),
        returns = "The `ShapelessRecipe` value with the documented change applied to set the result item and quantity produced by this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shapeless_recipe_value: sand::component::ShapelessRecipe, result: sand::component::RecipeResult)  {\n    let updated_shapeless_recipe = shapeless_recipe_value.result(result);\n}",
    )]
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe category for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapelessRecipe::category",
        aliases = ["sand::prelude::ShapelessRecipe::category"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe category for organization.",
        context = "Sets the recipe category for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cat = "`cat` provides the cat applied when setting the recipe category for organization."),
        returns = "The `ShapelessRecipe` value with the documented change applied to set the recipe category for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shapeless_recipe_value: sand::component::ShapelessRecipe, cat: impl Into < String >)  {\n    let updated_shapeless_recipe = shapeless_recipe_value.category(cat);\n}",
    )]
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapelessRecipe::group",
        aliases = ["sand::prelude::ShapelessRecipe::group"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `ShapelessRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shapeless_recipe_value: sand::component::ShapelessRecipe, g: impl Into < String >)  {\n    let updated_shapeless_recipe = shapeless_recipe_value.group(g);\n}",
    )]
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    fn try_build_json(&self) -> SandResult<Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".into(),
            Value::String("minecraft:crafting_shapeless".into()),
        );
        if let Some(category) = &self.category {
            map.insert("category".into(), Value::String(category.clone()));
        }
        if let Some(group) = &self.group {
            map.insert("group".into(), Value::String(group.clone()));
        }
        map.insert(
            "ingredients".into(),
            Value::Array(
                self.ingredients
                    .iter()
                    .map(serde_json::to_value)
                    .collect::<Result<_, _>>()
                    .map_err(SandError::from)?,
            ),
        );
        map.insert(
            "result".into(),
            serde_json::to_value(&self.result).map_err(SandError::from)?,
        );
        Ok(Value::Object(map))
    }
}

impl DatapackComponent for ShapelessRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        if self.ingredients.is_empty() {
            return Err(error(
                &self.location,
                "ingredients",
                "shapeless recipe requires at least one ingredient",
            ));
        }
        if self.ingredients.len() > 9 {
            return Err(error(
                &self.location,
                "ingredients",
                "shapeless recipe supports at most 9 ingredients",
            ));
        }
        for (i, ingredient) in self.ingredients.iter().enumerate() {
            ingredient.validate_at(&self.location, &format!("ingredients[{i}]"))?;
        }
        self.result.validate_at(&self.location, "result")
    }

    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!(
                "ShapelessRecipe::to_json() failed for {}: {e}",
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
