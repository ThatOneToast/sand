//! Shapeless crafting recipe builder (`minecraft:crafting_shapeless`).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};

/// Represents a shapeless crafting recipe where ingredient order and position don't matter.
pub struct ShapelessRecipe {
    pub location: ResourceLocation,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub result: RecipeResult,
}

impl ShapelessRecipe {
    /// Creates a new shapeless recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            ingredients: Vec::new(),
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
        }
    }

    /// Adds an ingredient to the recipe.
    pub fn ingredient(mut self, ingredient: Ingredient) -> Self {
        self.ingredients.push(ingredient);
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
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

impl DatapackComponent for ShapelessRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String("minecraft:crafting_shapeless".to_string()),
        );

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        let ingredients: Vec<Value> = self
            .ingredients
            .iter()
            .map(|i| serde_json::to_value(i).unwrap())
            .collect();
        map.insert("ingredients".to_string(), Value::Array(ingredients));
        map.insert(
            "result".to_string(),
            serde_json::to_value(&self.result).unwrap(),
        );

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
