//! Placed feature builder for `data/<namespace>/worldgen/placed_feature/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/placed_feature";

/// A placed feature definition (`data/<namespace>/worldgen/placed_feature/<id>.json`).
///
/// Placed features reference a configured feature and a list of placement
/// modifiers that determine where and how often they generate in the world.
pub struct PlacedFeature {
    location: ResourceLocation,
    /// The ID of the configured feature to place (e.g. `"minecraft:oak"`).
    feature: String,
    /// Placement modifier entries as raw JSON objects.
    placement: Vec<Value>,
}

impl PlacedFeature {
    /// Creates a new placed feature with the given resource location and feature ID.
    pub fn new(location: ResourceLocation, feature: impl Into<String>) -> Self {
        Self {
            location,
            feature: feature.into(),
            placement: Vec::new(),
        }
    }

    /// Adds a placement modifier as a raw JSON object.
    ///
    /// # Example
    /// ```rust,ignore
    /// use serde_json::json;
    /// feature.placement_modifier(json!({ "type": "minecraft:count", "count": 5 }));
    /// ```
    pub fn placement_modifier(mut self, modifier: Value) -> Self {
        self.placement.push(modifier);
        self
    }

    /// Sets all placement modifiers at once from an iterator of raw JSON values.
    pub fn placement(mut self, modifiers: impl IntoIterator<Item = Value>) -> Self {
        self.placement = modifiers.into_iter().collect();
        self
    }
}

impl DatapackComponent for PlacedFeature {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "feature": self.feature,
            "placement": self.placement,
        })
    }

    fn validate(&self) -> SandResult<()> {
        validation::validate_resource_location_str(&self.location, KIND, "feature", &self.feature)?;
        for (index, modifier) in self.placement.iter().enumerate() {
            let field = format!("placement[{index}]");
            validation::require_json_object(&self.location, KIND, &field, modifier)?;
            let modifier_type = modifier.get("type").and_then(Value::as_str);
            match modifier_type {
                Some(ty) if !ty.trim().is_empty() => {
                    validation::validate_resource_location_str(
                        &self.location,
                        KIND,
                        &format!("{field}.type"),
                        ty,
                    )?;
                }
                _ => {
                    return Err(validation::error(
                        &self.location,
                        KIND,
                        format!("{field}.type").as_str(),
                        "placement modifier must be a JSON object with a non-empty string `type` field",
                    ));
                }
            }
        }
        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/placed_feature"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "sparse_oak").unwrap()
    }

    #[test]
    fn valid_placed_feature_exports_unchanged() {
        let feature = PlacedFeature::new(location(), "minecraft:oak")
            .placement_modifier(serde_json::json!({ "type": "minecraft:count", "count": 5 }));
        assert!(feature.validate().is_ok());
        assert_eq!(feature.to_json()["feature"], "minecraft:oak");
    }

    #[test]
    fn empty_feature_id_rejected() {
        let feature = PlacedFeature::new(location(), "");
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("feature"), "{err}");
    }

    #[test]
    fn malformed_feature_id_rejected() {
        let feature = PlacedFeature::new(location(), "Not A Valid Id");
        assert!(feature.validate().is_err());
    }

    #[test]
    fn non_object_placement_modifier_rejected() {
        let feature =
            PlacedFeature::new(location(), "minecraft:oak").placement([serde_json::json!(5)]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn empty_placement_modifier_object_rejected() {
        let feature =
            PlacedFeature::new(location(), "minecraft:oak").placement([serde_json::json!({})]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn placement_modifier_missing_type_rejected() {
        let feature = PlacedFeature::new(location(), "minecraft:oak")
            .placement([serde_json::json!({"count": 5})]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0].type"), "{err}");
    }

    #[test]
    fn raw_placement_modifier_escape_hatch_still_works() {
        let feature = PlacedFeature::new(location(), "minecraft:oak")
            .placement([serde_json::json!({"type": "modded:custom_modifier", "value": 1})]);
        assert!(feature.validate().is_ok());
    }
}
