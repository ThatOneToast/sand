//! Cooking recipe builders: smelting, blasting, smoking, campfire cooking.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
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
            ingredient: Ingredient::empty(),
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
}

fn error(location: &ResourceLocation, field: &str, message: &str) -> SandError {
    SandError::ComponentValidation {
        location: location.clone(),
        kind: "recipe".into(),
        field: field.into(),
        message: message.into(),
    }
}
