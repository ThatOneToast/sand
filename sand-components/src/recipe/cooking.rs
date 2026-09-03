//! Cooking recipe builders: smelting, blasting, smoking, campfire cooking.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{CookingType, Ingredient, RecipeResult};
use sand_version::ComponentFeature;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::CookingRecipe",
    module = "sand::component",
    summary = "Represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).",
    context = "Represents a cooking recipe (smelting, blasting, smoking, or campfire cooking). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::CookingRecipe;",
    fields(location = "`location` provides the location identifier when the variant represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).", recipe_type = "`recipe_type` provides the recipe type when the variant represents a cooking recipe (smelting, blasting, smoking, or campfire cooking)."),
)]
/// Represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).
pub struct CookingRecipe {
    /// `location` provides the location identifier when the variant represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).
    pub location: ResourceLocation,
    /// `recipe_type` provides the recipe type when the variant represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).
    pub recipe_type: CookingType,
    category: Option<String>,
    group: Option<String>,
    ingredient: Ingredient,
    result: RecipeResult,
    experience: f32,
    cooking_time: u32,
}

impl CookingRecipe {
    /// Creates a new cooking recipe with the given location and cooking type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::new",
        module = "sand::component",
        kind = "method",
        summary = "Creates a new cooking recipe with the given location and cooking type.",
        context = "Creates a new cooking recipe with the given location and cooking type. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new cooking recipe with the given location and cooking type.", recipe_type = "`recipe_type` is used when creating a new cooking recipe with the given location and cooking type."),
        returns = "A `CookingRecipe` representing a new cooking recipe with the given location and cooking type.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, recipe_type: sand::component::CookingType)  {\n    let cooking_recipe = sand::component::CookingRecipe::new(location, recipe_type);\n}",
    )]
    pub fn new(location: ResourceLocation, recipe_type: CookingType) -> Self {
        Self {
            location,
            recipe_type,
            category: None,
            group: None,
            ingredient: Ingredient::empty(),
            result: RecipeResult::empty(),
            experience: 0.0,
            cooking_time: 200,
        }
    }

    /// Sets the ingredient for this cooking recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::ingredient",
        module = "sand::component",
        kind = "method",
        summary = "Sets the ingredient for this cooking recipe.",
        context = "Sets the ingredient for this cooking recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(ingredient = "`ingredient` provides the ingredient applied when setting the ingredient for this cooking recipe."),
        returns = "The `CookingRecipe` value with the documented change applied to set the ingredient for this cooking recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, ingredient: sand::component::Ingredient)  {\n    let updated_cooking_recipe = cooking_recipe_value.ingredient(ingredient);\n}",
    )]
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::result",
        module = "sand::component",
        kind = "method",
        summary = "Sets the result item and quantity produced by this recipe.",
        context = "Sets the result item and quantity produced by this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(result = "`result` provides the result applied when setting the result item and quantity produced by this recipe."),
        returns = "The `CookingRecipe` value with the documented change applied to set the result item and quantity produced by this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, result: sand::component::RecipeResult)  {\n    let updated_cooking_recipe = cooking_recipe_value.result(result);\n}",
    )]
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the amount of experience awarded for completing this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::experience",
        module = "sand::component",
        kind = "method",
        summary = "Sets the amount of experience awarded for completing this recipe.",
        context = "Sets the amount of experience awarded for completing this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(experience = "`experience` is used when completing this recipe."),
        returns = "The `CookingRecipe` value with the documented change applied to set the amount of experience awarded for completing this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, experience: f32)  {\n    let updated_cooking_recipe = cooking_recipe_value.experience(experience);\n}",
    )]
    pub fn experience(mut self, experience: f32) -> Self {
        self.experience = experience;
        self
    }

    /// Sets the cooking time in ticks required for this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::cooking_time",
        module = "sand::component",
        kind = "method",
        summary = "Sets the cooking time in ticks required for this recipe.",
        context = "Sets the cooking time in ticks required for this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cooking_time = "`cooking_time` provides the cooking time applied when setting the cooking time in ticks required for this recipe."),
        returns = "The `CookingRecipe` value with the documented change applied to set the cooking time in ticks required for this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, cooking_time: u32)  {\n    let updated_cooking_recipe = cooking_recipe_value.cooking_time(cooking_time);\n}",
    )]
    pub fn cooking_time(mut self, cooking_time: u32) -> Self {
        self.cooking_time = cooking_time;
        self
    }

    /// Sets the recipe category for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::category",
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe category for organization.",
        context = "Sets the recipe category for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cat = "`cat` provides the cat applied when setting the recipe category for organization."),
        returns = "The `CookingRecipe` value with the documented change applied to set the recipe category for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, cat: impl Into < String >)  {\n    let updated_cooking_recipe = cooking_recipe_value.category(cat);\n}",
    )]
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::CookingRecipe::group",
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `CookingRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cooking_recipe_value: sand::component::CookingRecipe, g: impl Into < String >)  {\n    let updated_cooking_recipe = cooking_recipe_value.group(g);\n}",
    )]
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    fn try_build_json(&self) -> SandResult<Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".into(),
            Value::String(self.recipe_type.type_str().into()),
        );
        if let Some(category) = &self.category {
            map.insert("category".into(), Value::String(category.clone()));
        }
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
        map.insert(
            "experience".into(),
            serde_json::to_value(self.experience).map_err(SandError::from)?,
        );
        map.insert("cookingtime".into(), Value::from(self.cooking_time));
        Ok(Value::Object(map))
    }
}

impl DatapackComponent for CookingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.ingredient.validate_at(&self.location, "ingredient")?;
        self.result.validate_at(&self.location, "result")?;
        if !self.experience.is_finite() {
            return Err(error(
                &self.location,
                "experience",
                "cooking experience must be finite",
            ));
        }
        if self.cooking_time == 0 {
            return Err(error(
                &self.location,
                "cookingtime",
                "cooking time must be at least 1 tick",
            ));
        }
        Ok(())
    }
    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!("CookingRecipe::to_json() failed for {}: {e}", self.location)
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
