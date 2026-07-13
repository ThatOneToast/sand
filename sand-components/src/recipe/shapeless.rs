//! Shapeless crafting recipe builder (`minecraft:crafting_shapeless`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

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
            result: RecipeResult::empty(),
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
