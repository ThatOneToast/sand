//! Shaped crafting recipe builder (`minecraft:crafting_shaped`).

use std::collections::HashMap;

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ShapedRecipe",
    aliases = ["sand::prelude::ShapedRecipe"],
    module = "sand::component",
    summary = "Represents a shaped crafting recipe where items must be placed in specific grid positions.",
    context = "Represents a shaped crafting recipe where items must be placed in specific grid positions. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ShapedRecipe;",
    fields(location = "`location` provides the location identifier when the variant represents a shaped crafting recipe where items must be placed in specific grid positions."),
)]
/// Represents a shaped crafting recipe where items must be placed in specific grid positions.
pub struct ShapedRecipe {
    /// `location` provides the location identifier when the variant represents a shaped crafting recipe where items must be placed in specific grid positions.
    pub location: ResourceLocation,
    category: Option<String>,
    group: Option<String>,
    pattern: Vec<String>,
    key: HashMap<char, Ingredient>,
    result: RecipeResult,
    show_notification: bool,
}

impl ShapedRecipe {
    /// Creates a new shaped recipe with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::new",
        aliases = ["sand::prelude::ShapedRecipe::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new shaped recipe with the given resource location.",
        context = "Creates a new shaped recipe with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new shaped recipe with the given resource location."),
        returns = "A `ShapedRecipe` representing a new shaped recipe with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let shaped_recipe = sand::component::ShapedRecipe::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            category: None,
            group: None,
            pattern: Vec::new(),
            key: HashMap::new(),
            result: RecipeResult::empty(),
            show_notification: true,
        }
    }

    /// Sets the crafting pattern rows (e.g., 3x3 grid layout).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::pattern",
        aliases = ["sand::prelude::ShapedRecipe::pattern"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the crafting pattern rows (e.g., 3x3 grid layout).",
        context = "Sets the crafting pattern rows (e.g., 3x3 grid layout). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(rows = "`rows` provides the rows applied when setting the crafting pattern rows (e.g., 3x3 grid layout)."),
        returns = "The `ShapedRecipe` value with the documented change applied to set the crafting pattern rows (e.g., 3x3 grid layout).",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, rows: impl IntoIterator < Item = impl Into < String > >)  {\n    let updated_shaped_recipe = shaped_recipe_value.pattern(rows);\n}",
    )]
    pub fn pattern(mut self, rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pattern = rows.into_iter().map(|r| r.into()).collect();
        self
    }

    /// Maps a character to an ingredient in the recipe pattern.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::key",
        aliases = ["sand::prelude::ShapedRecipe::key"],
        module = "sand::component",
        kind = "method",
        summary = "Maps a character to an ingredient in the recipe pattern.",
        context = "Maps a character to an ingredient in the recipe pattern. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(ch = "`ch` is used to map a character to an ingredient in the recipe pattern.", ingredient = "`ingredient` is used to map a character to an ingredient in the recipe pattern."),
        returns = "The `ShapedRecipe` value with the documented change applied to map a character to an ingredient in the recipe pattern.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, ch: char, ingredient: sand::component::Ingredient)  {\n    let updated_shaped_recipe = shaped_recipe_value.key(ch, ingredient);\n}",
    )]
    pub fn key(mut self, ch: char, ingredient: Ingredient) -> Self {
        self.key.insert(ch, ingredient);
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::result",
        aliases = ["sand::prelude::ShapedRecipe::result"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the result item and quantity produced by this recipe.",
        context = "Sets the result item and quantity produced by this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(result = "`result` provides the result applied when setting the result item and quantity produced by this recipe."),
        returns = "The `ShapedRecipe` value with the documented change applied to set the result item and quantity produced by this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, result: sand::component::RecipeResult)  {\n    let updated_shaped_recipe = shaped_recipe_value.result(result);\n}",
    )]
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe category for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::category",
        aliases = ["sand::prelude::ShapedRecipe::category"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe category for organization.",
        context = "Sets the recipe category for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cat = "`cat` provides the cat applied when setting the recipe category for organization."),
        returns = "The `ShapedRecipe` value with the documented change applied to set the recipe category for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, cat: impl Into < String >)  {\n    let updated_shaped_recipe = shaped_recipe_value.category(cat);\n}",
    )]
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::group",
        aliases = ["sand::prelude::ShapedRecipe::group"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `ShapedRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, g: impl Into < String >)  {\n    let updated_shaped_recipe = shaped_recipe_value.group(g);\n}",
    )]
    pub fn group(mut self, g: impl Into<String>) -> Self {
        self.group = Some(g.into());
        self
    }

    /// Sets whether a notification is shown when the recipe is unlocked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ShapedRecipe::show_notification",
        aliases = ["sand::prelude::ShapedRecipe::show_notification"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether a notification is shown when the recipe is unlocked.",
        context = "Sets whether a notification is shown when the recipe is unlocked. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether a notification is shown when the recipe is unlocked."),
        returns = "The `ShapedRecipe` value with the documented change applied to set whether a notification is shown when the recipe is unlocked.",
        example = "use sand::prelude::*;\n\nfn demonstrate(shaped_recipe_value: sand::component::ShapedRecipe, v: bool)  {\n    let updated_shaped_recipe = shaped_recipe_value.show_notification(v);\n}",
    )]
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
        if self.pattern.len() > 3 {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "recipe".to_string(),
                field: "pattern".to_string(),
                message: "shaped recipe pattern must have at most 3 rows".to_string(),
            });
        }
        let width = self.pattern[0].chars().count();
        if width == 0 || width > 3 {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "recipe".to_string(),
                field: "pattern[0]".to_string(),
                message: "shaped recipe rows must contain 1 to 3 columns".to_string(),
            });
        }
        for (index, row) in self.pattern.iter().enumerate().skip(1) {
            if row.chars().count() != width {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "recipe".to_string(),
                    field: format!("pattern[{index}]"),
                    message: "shaped recipe rows must have equal widths".to_string(),
                });
            }
        }
        self.result.validate_at(&self.location, "result")?;

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
            if *ch == ' ' {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "recipe".to_string(),
                    field: "key[' ']".to_string(),
                    message: "space is reserved for empty pattern slots and cannot be bound"
                        .to_string(),
                });
            }
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
            ing.validate_at(&self.location, &format!("key['{ch}']"))?;
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

    fn required_features(&self) -> &'static [ComponentFeature] {
        if self.result.has_components() {
            &[ComponentFeature::ItemComponents]
        } else {
            &[]
        }
    }
}
