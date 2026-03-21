//! Smithing table recipe builders: transform and trim (`minecraft:smithing_*`).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};

// ── SmithingTransformRecipe ───────────────────────────────────────────────────

/// Represents a smithing table recipe that transforms items using a template, base, and addition.
pub struct SmithingTransformRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
    pub result: RecipeResult,
}

impl SmithingTransformRecipe {
    /// Creates a new smithing transform recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            template: Ingredient {
                item: None,
                tag: None,
            },
            base: Ingredient {
                item: None,
                tag: None,
            },
            addition: Ingredient {
                item: None,
                tag: None,
            },
            result: RecipeResult {
                id: String::new(),
                count: 1,
            },
        }
    }

    /// Sets the template ingredient (e.g., netherite upgrade template).
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the base ingredient to be upgraded.
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the addition ingredient (e.g., netherite ingot).
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for SmithingTransformRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String("minecraft:smithing_transform".to_string()),
        );

        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert(
            "template".to_string(),
            serde_json::to_value(&self.template).unwrap(),
        );
        map.insert(
            "base".to_string(),
            serde_json::to_value(&self.base).unwrap(),
        );
        map.insert(
            "addition".to_string(),
            serde_json::to_value(&self.addition).unwrap(),
        );
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

// ── SmithingTrimRecipe ────────────────────────────────────────────────────────

/// Represents a smithing table recipe that applies decorative trim to armor.
pub struct SmithingTrimRecipe {
    pub location: ResourceLocation,
    pub group: Option<String>,
    pub template: Ingredient,
    pub base: Ingredient,
    pub addition: Ingredient,
}

impl SmithingTrimRecipe {
    /// Creates a new smithing trim recipe with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            group: None,
            template: Ingredient {
                item: None,
                tag: None,
            },
            base: Ingredient {
                item: None,
                tag: None,
            },
            addition: Ingredient {
                item: None,
                tag: None,
            },
        }
    }

    /// Sets the trim template ingredient.
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the armor piece to be trimmed.
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the trim material ingredient (e.g., amethyst shard).
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the recipe group for organization.
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }
}

impl DatapackComponent for SmithingTrimRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            Value::String("minecraft:smithing_trim".to_string()),
        );

        if let Some(ref group) = self.group {
            map.insert("group".to_string(), Value::String(group.clone()));
        }

        map.insert(
            "template".to_string(),
            serde_json::to_value(&self.template).unwrap(),
        );
        map.insert(
            "base".to_string(),
            serde_json::to_value(&self.base).unwrap(),
        );
        map.insert(
            "addition".to_string(),
            serde_json::to_value(&self.addition).unwrap(),
        );

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
