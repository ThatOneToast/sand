//! Placed feature builder for `data/<namespace>/worldgen/placed_feature/<id>.json`.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::ConfiguredFeatureId;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/placed_feature";

/// How the referenced configured feature was supplied.
#[derive(Debug, Clone)]
enum ConfiguredFeatureReference {
    Typed(ConfiguredFeatureId),
    Raw(String),
}

impl ConfiguredFeatureReference {
    fn as_string(&self) -> String {
        match self {
            Self::Typed(id) => id.to_string(),
            Self::Raw(id) => id.clone(),
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature` for the canonical contract."]
/// A placed feature definition (`data/<namespace>/worldgen/placed_feature/<id>.json`).
///
/// Placed features reference a configured feature and a list of placement
/// modifiers that determine where and how often they generate in the world.
/// Author the referenced feature with
/// [`ConfiguredFeature`](crate::worldgen::ConfiguredFeature) and pass
/// [`ConfiguredFeature::id`](crate::worldgen::ConfiguredFeature::id) here.
pub struct PlacedFeature {
    location: ResourceLocation,
    /// The configured feature to place.
    feature: ConfiguredFeatureReference,
    /// Placement modifier entries as raw JSON objects.
    placement: Vec<Value>,
}

impl PlacedFeature {
    /// Creates a new placed feature referencing a typed configured feature.
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::new` for the canonical contract."]
    pub fn new(location: ResourceLocation, feature: ConfiguredFeatureId) -> Self {
        Self {
            location,
            feature: ConfiguredFeatureReference::Typed(feature),
            placement: Vec::new(),
        }
    }

    /// Creates a placed feature with an explicitly raw configured-feature
    /// reference.
    ///
    /// Prefer [`PlacedFeature::new`] with a [`ConfiguredFeatureId`]. This
    /// escape hatch exists for modded or version-specific reference syntax.
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::new_raw_feature` for the canonical contract."]
    pub fn new_raw_feature(location: ResourceLocation, feature: impl Into<String>) -> Self {
        Self {
            location,
            feature: ConfiguredFeatureReference::Raw(feature.into()),
            placement: Vec::new(),
        }
    }

    /// Updates the referenced configured feature.
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::feature` for the canonical contract."]
    pub fn feature(mut self, feature: ConfiguredFeatureId) -> Self {
        self.feature = ConfiguredFeatureReference::Typed(feature);
        self
    }

    /// Updates the referenced configured feature through the explicit raw
    /// compatibility path.
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::raw_feature` for the canonical contract."]
    pub fn raw_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature = ConfiguredFeatureReference::Raw(feature.into());
        self
    }

    /// Adds a placement modifier through the explicit raw JSON escape hatch.
    ///
    /// # Example
    /// ```rust,ignore
    /// use sand_components::RawJson;
    /// use serde_json::json;
    /// feature.placement_modifier(RawJson::new(json!({ "type": "minecraft:count", "count": 5 })));
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::placement_modifier` for the canonical contract."]
    pub fn placement_modifier(mut self, modifier: RawJson) -> Self {
        self.placement.push(modifier.into_value());
        self
    }

    /// Sets all placement modifiers from explicit raw JSON escape-hatch values.
    #[doc = "**API Contract:** Run `sand api show sand::component::PlacedFeature::placement` for the canonical contract."]
    pub fn placement(mut self, modifiers: impl IntoIterator<Item = RawJson>) -> Self {
        self.placement = modifiers.into_iter().map(RawJson::into_value).collect();
        self
    }
}

impl DatapackComponent for PlacedFeature {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "feature": self.feature.as_string(),
            "placement": self.placement,
        })
    }

    fn validate(&self) -> SandResult<()> {
        validation::validate_resource_location_str(
            &self.location,
            KIND,
            "feature",
            &self.feature.as_string(),
        )?;
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

    use crate::worldgen::ConfiguredFeature;

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "sparse_oak").unwrap()
    }

    fn oak() -> ConfiguredFeatureId {
        ConfiguredFeatureId::minecraft("oak").unwrap()
    }

    #[test]
    fn valid_placed_feature_exports_unchanged() {
        let feature = PlacedFeature::new(location(), oak()).placement_modifier(RawJson::new(
            serde_json::json!({ "type": "minecraft:count", "count": 5 }),
        ));
        assert!(feature.validate().is_ok());
        assert_eq!(feature.to_json()["feature"], "minecraft:oak");
    }

    #[test]
    fn typed_configured_feature_component_reference_round_trips() {
        let configured = ConfiguredFeature::no_op(
            ResourceLocation::new("my_pack", "ashen_shrub_feature").unwrap(),
        );
        let placed = PlacedFeature::new(location(), configured.id());
        assert!(placed.validate().is_ok());
        assert_eq!(
            placed.to_json(),
            serde_json::json!({
                "feature": "my_pack:ashen_shrub_feature",
                "placement": [],
            })
        );
    }

    #[test]
    fn empty_feature_id_rejected() {
        let feature = PlacedFeature::new_raw_feature(location(), "");
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("feature"), "{err}");
    }

    #[test]
    fn malformed_feature_id_rejected() {
        let feature = PlacedFeature::new_raw_feature(location(), "Not A Valid Id");
        assert!(feature.validate().is_err());
        assert!("Not A Valid Id".parse::<ConfiguredFeatureId>().is_err());
    }

    #[test]
    fn non_object_placement_modifier_rejected() {
        let feature =
            PlacedFeature::new(location(), oak()).placement([RawJson::new(serde_json::json!(5))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn empty_placement_modifier_object_rejected() {
        let feature =
            PlacedFeature::new(location(), oak()).placement([RawJson::new(serde_json::json!({}))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn placement_modifier_missing_type_rejected() {
        let feature = PlacedFeature::new(location(), oak())
            .placement([RawJson::new(serde_json::json!({"count": 5}))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0].type"), "{err}");
    }

    #[test]
    fn raw_placement_modifier_escape_hatch_still_works() {
        let feature = PlacedFeature::new(location(), oak()).placement([RawJson::new(
            serde_json::json!({"type": "modded:custom_modifier", "value": 1}),
        )]);
        assert!(feature.validate().is_ok());
    }

    #[test]
    fn raw_feature_reference_escape_hatch_is_explicit_and_preserved() {
        let feature = PlacedFeature::new_raw_feature(location(), "placeholder")
            .raw_feature("modded:custom_feature");
        assert!(feature.validate().is_ok());
        assert_eq!(feature.to_json()["feature"], "modded:custom_feature");

        let retyped = PlacedFeature::new_raw_feature(location(), "modded:custom_feature")
            .feature(ConfiguredFeatureId::minecraft("oak").unwrap());
        assert_eq!(retyped.to_json()["feature"], "minecraft:oak");
    }
}
