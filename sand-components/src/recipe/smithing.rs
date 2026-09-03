//! Smithing table recipe builders: transform and trim (`minecraft:smithing_*`).

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::resource_location::ResourceLocation;

use super::types::{Ingredient, RecipeResult};
use sand_version::ComponentFeature;

// ── SmithingTransformRecipe ───────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SmithingTransformRecipe",
    aliases = ["sand::prelude::SmithingTransformRecipe"],
    module = "sand::component",
    summary = "Represents a smithing table recipe that transforms items using a template, base, and addition.",
    context = "Represents a smithing table recipe that transforms items using a template, base, and addition. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SmithingTransformRecipe;",
    fields(location = "`location` provides the location identifier when the variant represents a smithing table recipe that transforms items using a template, base, and addition."),
)]
/// Represents a smithing table recipe that transforms items using a template, base, and addition.
pub struct SmithingTransformRecipe {
    /// `location` provides the location identifier when the variant represents a smithing table recipe that transforms items using a template, base, and addition.
    pub location: ResourceLocation,
    group: Option<String>,
    template: Ingredient,
    base: Ingredient,
    addition: Ingredient,
    result: RecipeResult,
}

impl SmithingTransformRecipe {
    /// Creates a new smithing transform recipe with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::new",
        aliases = ["sand::prelude::SmithingTransformRecipe::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new smithing transform recipe with the given resource location.",
        context = "Creates a new smithing transform recipe with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new smithing transform recipe with the given resource location."),
        returns = "A `SmithingTransformRecipe` representing a new smithing transform recipe with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let smithing_transform_recipe = sand::component::SmithingTransformRecipe::new(location);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::template",
        aliases = ["sand::prelude::SmithingTransformRecipe::template"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the template ingredient (e.g., netherite upgrade template).",
        context = "Sets the template ingredient (e.g., netherite upgrade template). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(template = "`template` provides the template applied when setting the template ingredient (e.g., netherite upgrade template)."),
        returns = "The `SmithingTransformRecipe` value with the documented change applied to set the template ingredient (e.g., netherite upgrade template).",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_transform_recipe_value: sand::component::SmithingTransformRecipe, template: sand::component::Ingredient)  {\n    let updated_smithing_transform_recipe = smithing_transform_recipe_value.template(template);\n}",
    )]
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the base ingredient to be upgraded.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::base",
        aliases = ["sand::prelude::SmithingTransformRecipe::base"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the base ingredient to be upgraded.",
        context = "Sets the base ingredient to be upgraded. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(base = "`base` provides the base applied when setting the base ingredient to be upgraded."),
        returns = "The `SmithingTransformRecipe` value with the documented change applied to set the base ingredient to be upgraded.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_transform_recipe_value: sand::component::SmithingTransformRecipe, base: sand::component::Ingredient)  {\n    let updated_smithing_transform_recipe = smithing_transform_recipe_value.base(base);\n}",
    )]
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the addition ingredient (e.g., netherite ingot).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::addition",
        aliases = ["sand::prelude::SmithingTransformRecipe::addition"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the addition ingredient (e.g., netherite ingot).",
        context = "Sets the addition ingredient (e.g., netherite ingot). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(addition = "`addition` provides the addition applied when setting the addition ingredient (e.g., netherite ingot)."),
        returns = "The `SmithingTransformRecipe` value with the documented change applied to set the addition ingredient (e.g., netherite ingot).",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_transform_recipe_value: sand::component::SmithingTransformRecipe, addition: sand::component::Ingredient)  {\n    let updated_smithing_transform_recipe = smithing_transform_recipe_value.addition(addition);\n}",
    )]
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the result item and quantity produced by this recipe.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::result",
        aliases = ["sand::prelude::SmithingTransformRecipe::result"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the result item and quantity produced by this recipe.",
        context = "Sets the result item and quantity produced by this recipe. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(result = "`result` provides the result applied when setting the result item and quantity produced by this recipe."),
        returns = "The `SmithingTransformRecipe` value with the documented change applied to set the result item and quantity produced by this recipe.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_transform_recipe_value: sand::component::SmithingTransformRecipe, result: sand::component::RecipeResult)  {\n    let updated_smithing_transform_recipe = smithing_transform_recipe_value.result(result);\n}",
    )]
    pub fn result(mut self, result: RecipeResult) -> Self {
        self.result = result;
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTransformRecipe::group",
        aliases = ["sand::prelude::SmithingTransformRecipe::group"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `SmithingTransformRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_transform_recipe_value: sand::component::SmithingTransformRecipe, g: impl Into < String >)  {\n    let updated_smithing_transform_recipe = smithing_transform_recipe_value.group(g);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SmithingTrimRecipe",
    aliases = ["sand::prelude::SmithingTrimRecipe"],
    module = "sand::component",
    summary = "Represents a smithing table recipe that applies decorative trim to armor.",
    context = "Represents a smithing table recipe that applies decorative trim to armor. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SmithingTrimRecipe;",
    fields(location = "`location` provides the location identifier when the variant represents a smithing table recipe that applies decorative trim to armor."),
)]
/// Represents a smithing table recipe that applies decorative trim to armor.
pub struct SmithingTrimRecipe {
    /// `location` provides the location identifier when the variant represents a smithing table recipe that applies decorative trim to armor.
    pub location: ResourceLocation,
    group: Option<String>,
    template: Ingredient,
    base: Ingredient,
    addition: Ingredient,
}

impl SmithingTrimRecipe {
    /// Creates a new smithing trim recipe with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTrimRecipe::new",
        aliases = ["sand::prelude::SmithingTrimRecipe::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new smithing trim recipe with the given resource location.",
        context = "Creates a new smithing trim recipe with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new smithing trim recipe with the given resource location."),
        returns = "A `SmithingTrimRecipe` representing a new smithing trim recipe with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let smithing_trim_recipe = sand::component::SmithingTrimRecipe::new(location);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTrimRecipe::template",
        aliases = ["sand::prelude::SmithingTrimRecipe::template"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the trim template ingredient.",
        context = "Sets the trim template ingredient. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(template = "`template` provides the template applied when setting the trim template ingredient."),
        returns = "The `SmithingTrimRecipe` value with the documented change applied to set the trim template ingredient.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_trim_recipe_value: sand::component::SmithingTrimRecipe, template: sand::component::Ingredient)  {\n    let updated_smithing_trim_recipe = smithing_trim_recipe_value.template(template);\n}",
    )]
    pub fn template(mut self, template: Ingredient) -> Self {
        self.template = template;
        self
    }

    /// Sets the armor piece to be trimmed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTrimRecipe::base",
        aliases = ["sand::prelude::SmithingTrimRecipe::base"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the armor piece to be trimmed.",
        context = "Sets the armor piece to be trimmed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(base = "`base` provides the base applied when setting the armor piece to be trimmed."),
        returns = "The `SmithingTrimRecipe` value with the documented change applied to set the armor piece to be trimmed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_trim_recipe_value: sand::component::SmithingTrimRecipe, base: sand::component::Ingredient)  {\n    let updated_smithing_trim_recipe = smithing_trim_recipe_value.base(base);\n}",
    )]
    pub fn base(mut self, base: Ingredient) -> Self {
        self.base = base;
        self
    }

    /// Sets the trim material ingredient (e.g., amethyst shard).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTrimRecipe::addition",
        aliases = ["sand::prelude::SmithingTrimRecipe::addition"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the trim material ingredient (e.g., amethyst shard).",
        context = "Sets the trim material ingredient (e.g., amethyst shard). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(addition = "`addition` provides the addition applied when setting the trim material ingredient (e.g., amethyst shard)."),
        returns = "The `SmithingTrimRecipe` value with the documented change applied to set the trim material ingredient (e.g., amethyst shard).",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_trim_recipe_value: sand::component::SmithingTrimRecipe, addition: sand::component::Ingredient)  {\n    let updated_smithing_trim_recipe = smithing_trim_recipe_value.addition(addition);\n}",
    )]
    pub fn addition(mut self, addition: Ingredient) -> Self {
        self.addition = addition;
        self
    }

    /// Sets the recipe group for organization.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SmithingTrimRecipe::group",
        aliases = ["sand::prelude::SmithingTrimRecipe::group"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the recipe group for organization.",
        context = "Sets the recipe group for organization. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(g = "`g` provides the recipe group name used to set the recipe group for organization."),
        returns = "The `SmithingTrimRecipe` value with the documented change applied to set the recipe group for organization.",
        example = "use sand::prelude::*;\n\nfn demonstrate(smithing_trim_recipe_value: sand::component::SmithingTrimRecipe, g: impl Into < String >)  {\n    let updated_smithing_trim_recipe = smithing_trim_recipe_value.group(g);\n}",
    )]
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
