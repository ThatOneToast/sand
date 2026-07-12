use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result, SandError};
use crate::resource_location::ResourceLocation;

use crate::loot_table::LootFunction;

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
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            functions: Vec::new(),
        }
    }

    /// Add a loot function to this item modifier.
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
