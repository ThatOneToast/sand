//! Cooking recipe builders: smelting, blasting, smoking, campfire cooking.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::types::{CookingType, Ingredient, RecipeResult};

/// Represents a cooking recipe (smelting, blasting, smoking, or campfire cooking).
pub struct CookingRecipe {
    pub location: ResourceLocation,
    pub recipe_type: CookingType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
    pub experience: f32,
    pub cooking_time: u32,
}

impl CookingRecipe {
    /// Creates a new cooking recipe with the given location and cooking type.
    pub fn new(location: ResourceLocation, recipe_type: CookingType) -> Self {
        Self {
            location,
            recipe_type,
            category: None,
            group: None,
            ingredient: Ingredient {
                item: None,
                tag: None,
            },
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            experience: 0.0,
            cooking_time: 200,
        }
    }

    /// Sets the ingredient for this cooking recipe.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the amount of experience awarded for completing this recipe.
    pub fn experience(mut self, experience: f32) -> Self {
        self.experience = experience;
        self
    }

    /// Sets the cooking time in ticks required for this recipe.
    pub fn cooking_time(mut self, cooking_time: u32) -> Self {
        self.cooking_time = cooking_time;
        self
    }

    /// Sets the recipe category for organization.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for CookingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String(self.recipe_type.type_str().to_string()),
        );

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert(
            "ingredient".to_string(),
            serde_json::to_value(&self.ingredient).unwrap(),
        );
        map.insert(
            "result".to_string(),
            serde_json::to_value(&self.result).unwrap(),
        );
        map.insert(
            "experience".to_string(),
            serde_json::to_value(self.experience).unwrap(),
        );
        map.insert(
            "cookingtime".to_string(),
            serde_json::to_value(self.cooking_time).unwrap(),
        );

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
