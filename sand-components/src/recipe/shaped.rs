//! Shaped crafting recipe builder (`minecraft:crafting_shaped`).

use std::collections::HashMap;

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
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

impl ShapedRecipe {
    /// Fallible JSON construction used by both `try_content` (export path) and
    /// `to_json` (compatibility). Propagates serialization errors instead of
    /// silently substituting `Value::Null`.
    fn try_build_json(&self) -> SandResult<Value> {
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
            .map(|(ch, ing)| {
                let value = serde_json::to_value(ing).map_err(SandError::from)?;
                Ok::<_, SandError>((ch.to_string(), value))
            })
            .collect::<SandResult<_>>()?;
        map.insert("key".to_string(), Value::Object(key_map));

        map.insert(
            "result".to_string(),
            serde_json::to_value(&self.result).map_err(SandError::from)?,
        );
        map.insert(
            "show_notification".to_string(),
            Value::Bool(self.show_notification),
        );

        Ok(Value::Object(map))
    }
}

impl DatapackComponent for ShapedRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        if self.pattern.is_empty() {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "recipe".to_string(),
                field: "pattern".to_string(),
                message: "shaped recipe pattern must not be empty".to_string(),
            });
        }
        if self.result.id.is_empty() {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "recipe".to_string(),
                field: "result.id".to_string(),
                message: "shaped recipe result item id must not be empty".to_string(),
            });
        }
        if self.result.count == 0 {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "recipe".to_string(),
                field: "result.count".to_string(),
                message: "shaped recipe result count must be at least 1".to_string(),
            });
        }

        let pattern_chars: std::collections::HashSet<char> = self
            .pattern
            .iter()
            .flat_map(|r| r.chars())
            .filter(|c| *c != ' ')
            .collect();

        for ch in &pattern_chars {
            if !self.key.contains_key(ch) {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "recipe".to_string(),
                    field: "key".to_string(),
                    message: format!(
                        "pattern character '{ch}' is not bound to any ingredient \
                         — add .key('{ch}', Ingredient::...)"
                    ),
                });
            }
        }

        for ch in self.key.keys() {
            if !pattern_chars.contains(ch) {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "recipe".to_string(),
                    field: "key".to_string(),
                    message: format!(
                        "key character '{ch}' is not used in the pattern — \
                         remove it or add it to the pattern"
                    ),
                });
            }
        }

        for (ch, ing) in &self.key {
            if ing.is_empty() {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "recipe".to_string(),
                    field: format!("key['{ch}']"),
                    message: "ingredient cannot be empty — use Ingredient::item(...) \
                              or Ingredient::tag(...)"
                        .to_string(),
                });
            }
        }

        Ok(())
    }

    fn to_json(&self) -> Value {
        // Compatibility path: callers accept that an invalid recipe panics
        // rather than silently emitting null. The export path uses
        // try_content() which propagates errors.
        self.try_build_json()
            .unwrap_or_else(|e| panic!("ShapedRecipe::to_json() failed for {}: {e}", self.location))
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        let json = self.try_build_json()?;
        Ok(ComponentContent::Json(json))
    }

    fn component_dir(&self) -> &'static str {
        "recipe"
    }
}
