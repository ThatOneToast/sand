//! Stonecutter recipe builder (`minecraft:stonecutting`).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};

/// Represents a stonecutter recipe for cutting stone blocks into other shapes.
pub struct StonecuttingRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
    pub count: u32,
}

impl StonecuttingRecipe {
    /// Creates a new stonecutter recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            ingredient: Ingredient {
                item: None,
                tag: None,
            },
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            count: 1,
        }
    }

    /// Sets the ingredient to be cut by the stonecutter.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredient = ingredient;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the quantity of the result produced.
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for StonecuttingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String("minecraft:stonecutting".to_string()),
        );

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
            "count".to_string(),
            serde_json::to_value(self.count).unwrap(),
        );

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
