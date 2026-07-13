//! Stonecutter recipe builder (`minecraft:stonecutting`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

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
            ingredient: Ingredient::empty(),
            result: RecipeResult::empty(),
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

    fn try_build_json(&self) -> SandResult<Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".into(),
            Value::String("minecraft:stonecutting".into()),
        );
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
        map.insert("count".into(), Value::from(self.count));
        Ok(Value::Object(map))
    }
}

impl DatapackComponent for StonecuttingRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.ingredient.validate_at(&self.location, "ingredient")?;
        self.result.validate_at(&self.location, "result")?;
        if self.count == 0 {
            return Err(error(
                &self.location,
                "count",
                "stonecutting result count must be at least 1",
            ));
        }
        Ok(())
    }
    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!(
                "StonecuttingRecipe::to_json() failed for {}: {e}",
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
