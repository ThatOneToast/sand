//! Shaped crafting recipe builder (`minecraft:crafting_shaped`).

use std::collections::HashMap;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};

/// Represents a shaped crafting recipe where items must be placed in specific grid positions.
pub struct ShapedRecipe {
    pub location: ResourceLocation,
    pub category: Option<String>,
    pub group: Option<String>,
    pub pattern: Vec<String>,
    pub key: HashMap<char, Ingredient>,
    pub result: RecipeResult,
    pub show_notification: bool,
}

impl ShapedRecipe {
    /// Creates a new shaped recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            pattern: Vec::new(),
            key: HashMap::new(),
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
            show_notification: true,
        }
    }

    /// Sets the crafting pattern rows (e.g., 3x3 grid layout).
    pub fn pattern(mut self, rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pattern = rows.into_iter().map(|r| r.into()).collect();
        self
    }

    /// Maps a character to an ingredient in the recipe pattern.
    pub fn key(mut self, ch: char, ingredient: Ingredient) -> Self {
        self.key.insert(ch, ingredient);
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

    /// Sets whether a notification is shown when the recipe is unlocked.
    pub fn show_notification(mut self, v: bool) -> Self {
        self.show_notification = v;
        self
    }
}

impl DatapackComponent for ShapedRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String("minecraft:crafting_shaped".to_string()),
        );

        if let Some(ref category) = self.category {
            map.insert("category".to_string(), Value::String(category.clone()));
        }
        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert(
            "pattern".to_string(),
            Value::Array(
                self.pattern
                    .iter()
                    .map(|r| Value::String(r.clone()))
                    .collect(),
            ),
        );

        let key_map: serde_json::Map<String, Value> = self
            .key
            .iter()
            .map(|(ch, ing)| (ch.to_string(), serde_json::to_value(ing).unwrap()))
            .collect();
        map.insert("key".to_string(), Value::Object(key_map));

        map.insert(
            "result".to_string(),
            serde_json::to_value(&self.result).unwrap(),
        );
        map.insert(
            "show_notification".to_string(),
            Value::Bool(self.show_notification),
        );

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
