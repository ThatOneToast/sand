//! Smithing table recipe builders: transform and trim (`minecraft:smithing_*`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

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
            template: Ingredient::empty(),
            base: Ingredient::empty(),
            addition: Ingredient::empty(),
            result: RecipeResult::empty(),
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

    fn try_build_json(&self) -> SandResult<Value> {
        build_smithing_json(
            "minecraft:smithing_transform",
            self.group.as_ref(),
            [
                ("template", &self.template),
                ("base", &self.base),
                ("addition", &self.addition),
            ],
            Some(&self.result),
        )
    }
}

impl DatapackComponent for SmithingTransformRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.template.validate_at(&self.location, "template")?;
        self.base.validate_at(&self.location, "base")?;
        self.addition.validate_at(&self.location, "addition")?;
        self.result.validate_at(&self.location, "result")
    }
    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!(
                "SmithingTransformRecipe::to_json() failed for {}: {e}",
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
            template: Ingredient::empty(),
            base: Ingredient::empty(),
            addition: Ingredient::empty(),
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

    fn try_build_json(&self) -> SandResult<Value> {
        build_smithing_json(
            "minecraft:smithing_trim",
            self.group.as_ref(),
            [
                ("template", &self.template),
                ("base", &self.base),
                ("addition", &self.addition),
            ],
            None,
        )
    }
}

impl DatapackComponent for SmithingTrimRecipe {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.template.validate_at(&self.location, "template")?;
        self.base.validate_at(&self.location, "base")?;
        self.addition.validate_at(&self.location, "addition")
    }
    fn to_json(&self) -> Value {
        self.try_build_json().unwrap_or_else(|e| {
            panic!(
                "SmithingTrimRecipe::to_json() failed for {}: {e}",
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
}

fn build_smithing_json<'a>(
    kind: &str,
    group: Option<&String>,
    ingredients: impl IntoIterator<Item = (&'a str, &'a Ingredient)>,
    result: Option<&RecipeResult>,
) -> SandResult<Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".into(), Value::String(kind.into()));
    if let Some(group) = group {
        map.insert("group".into(), Value::String(group.clone()));
    }
    for (name, ingredient) in ingredients {
        map.insert(
            name.into(),
            serde_json::to_value(ingredient).map_err(SandError::from)?,
        );
    }
    if let Some(result) = result {
        map.insert(
            "result".into(),
            serde_json::to_value(result).map_err(SandError::from)?,
        );
    }
    Ok(Value::Object(map))
}
