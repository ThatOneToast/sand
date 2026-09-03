use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result, SandError};
use crate::resource_location::ResourceLocation;

use crate::loot_table::LootFunction;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ItemModifier",
    aliases = ["sand::prelude::ItemModifier"],
    module = "sand::component",
    summary = "An item modifier that applies loot functions to transform items in Minecraft.",
    context = "An item modifier that applies loot functions to transform items in Minecraft. Normal fallible export rejects modifiers without functions. Construction is intentionally incremental, so this invariant is checked by [`validate`](Self::validate) rather than by [`new`](Self::new). Direct legacy [`to_json`](Self::to_json) calls retain their historical empty-array behavior.",
    minecraft = "Normal fallible export rejects modifiers without functions. Construction is intentionally incremental, so this invariant is checked by [`validate`](Self::validate) rather than by [`new`](Self::new). Direct legacy [`to_json`](Self::to_json) calls retain their historical empty-array behavior.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ItemModifier;",
    fields(functions = "List of loot functions to apply to items.", location = "The resource location for this item modifier."),
)]
/// An item modifier that applies loot functions to transform items in Minecraft.
///
/// Normal fallible export rejects modifiers without functions. Construction is
/// intentionally incremental, so this invariant is checked by [`validate`](Self::validate)
/// rather than by [`new`](Self::new). Direct legacy [`to_json`](Self::to_json)
/// calls retain their historical empty-array behavior.
pub struct ItemModifier {
    /// The resource location for this item modifier.
    pub location: ResourceLocation,
    /// List of loot functions to apply to items.
    pub functions: Vec<LootFunction>,
}

impl ItemModifier {
    /// Create a new item modifier with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ItemModifier::new",
        aliases = ["sand::prelude::ItemModifier::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a new item modifier with the given resource location.",
        context = "Create a new item modifier with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new item modifier with the given resource location."),
        returns = "An `ItemModifier` representing a new item modifier with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let item_modifier = sand::component::ItemModifier::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            functions: Vec::new(),
        }
    }

    /// Add a loot function to this item modifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ItemModifier::function",
        aliases = ["sand::prelude::ItemModifier::function"],
        module = "sand::component",
        kind = "method",
        summary = "Add a loot function to this item modifier.",
        context = "Add a loot function to this item modifier. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(f = "`f` provides the f added when building a loot function to this item modifier."),
        returns = "The `ItemModifier` value with the documented change applied to add a loot function to this item modifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(item_modifier_value: sand::component::ItemModifier, f: sand::component::LootFunction)  {\n    let updated_item_modifier = item_modifier_value.function(f);\n}",
    )]
    pub fn function(mut self, f: LootFunction) -> Self {
        self.functions.push(f);
        self
    }
}

impl DatapackComponent for ItemModifier {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        self.try_to_json()
            .unwrap_or_else(|error| panic!("item modifier serialization failed: {error}"))
    }

    fn validate(&self) -> Result<()> {
        if self.functions.is_empty() {
            return Err(SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "item_modifier".to_string(),
                field: "functions".to_string(),
                message: "item modifier must contain at least one loot function".to_string(),
            });
        }

        for (index, function) in self.functions.iter().enumerate() {
            if let Err(failure) = function.validate_at(&format!("functions[{index}]")) {
                return Err(SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "item_modifier".to_string(),
                    field: failure.path,
                    message: failure.message,
                });
            }
        }
        Ok(())
    }

    fn try_content(&self) -> Result<ComponentContent> {
        self.validate()?;
        self.try_to_json()
            .map(ComponentContent::Json)
            .map_err(|error| SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "item_modifier".to_string(),
                field: "<serialization>".to_string(),
                message: error.to_string(),
            })
    }

    fn component_dir(&self) -> &'static str {
        "item_modifier"
    }
}

impl ItemModifier {
    fn try_to_json(&self) -> std::result::Result<Value, serde_json::Error> {
        match self.functions.as_slice() {
            [] => Ok(Value::Array(vec![])),
            [single] => serde_json::to_value(single),
            many => serde_json::to_value(many),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::loot_table::NumberProvider;
    use crate::raw::RawJson;

    fn modifier(path: &str) -> ItemModifier {
        ItemModifier::new(format!("test:{path}").parse().unwrap())
    }

    #[test]
    fn empty_modifier_is_rejected_by_fallible_export() {
        let modifier = modifier("empty");
        let error = modifier.try_content().unwrap_err().to_string();
        assert!(error.contains("test:empty"));
        assert!(error.contains("item_modifier"));
        assert!(error.contains("functions"));

        // Preserve the legacy direct-serialization compatibility contract.
        assert_eq!(modifier.to_json(), json!([]));
    }

    #[test]
    fn nested_function_validation_retains_owner_and_path() {
        let modifier = modifier("invalid_count").function(LootFunction::SetCount {
            count: NumberProvider::Constant(f64::NAN),
            add: false,
        });
        let error = modifier.try_content().unwrap_err().to_string();
        assert!(error.contains("test:invalid_count"));
        assert!(error.contains("functions[0].count"));
        assert!(error.contains("finite"));
    }

    #[test]
    fn single_and_multiple_function_shapes_are_unchanged() {
        let single = modifier("single").function(LootFunction::SetCount {
            count: NumberProvider::Constant(2.0),
            add: false,
        });
        let expected_single = json!({
            "function": "minecraft:set_count",
            "count": 2.0,
            "add": false
        });
        assert_eq!(single.to_json(), expected_single);
        assert_eq!(
            single.try_content().unwrap(),
            ComponentContent::Json(expected_single)
        );

        let multiple = modifier("multiple")
            .function(LootFunction::ExplosionDecay)
            .function(LootFunction::FurnaceSmelt);
        let expected_multiple = json!([
            {"function": "minecraft:explosion_decay"},
            {"function": "minecraft:furnace_smelt"}
        ]);
        assert_eq!(multiple.to_json(), expected_multiple);
        assert_eq!(
            multiple.try_content().unwrap(),
            ComponentContent::Json(expected_multiple)
        );
    }

    #[test]
    fn valid_custom_function_remains_an_escape_hatch() {
        let modifier = modifier("custom").function(LootFunction::Custom {
            function: "modded:transform".to_string(),
            data: RawJson::new(json!({"strength": 2})),
        });
        assert!(modifier.try_content().is_ok());
    }
}
