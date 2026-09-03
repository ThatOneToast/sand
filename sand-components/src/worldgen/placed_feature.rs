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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::PlacedFeature",
    aliases = ["sand::prelude::PlacedFeature"],
    module = "sand::component",
    summary = "A placed feature definition (`data/<namespace>/worldgen/placed_feature/<id>.json`).",
    context = "A placed feature definition (`data/<namespace>/worldgen/placed_feature/<id>.json`). Placed features reference a configured feature and a list of placement modifiers that determine where and how often they generate in the world. Author the referenced feature with [`ConfiguredFeature`](sand::component::ConfiguredFeature) and pass [`ConfiguredFeature::id`](sand::component::ConfiguredFeature::id) here.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::PlacedFeature;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::new",
        aliases = ["sand::prelude::PlacedFeature::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new placed feature referencing a typed configured feature.",
        context = "Creates a new placed feature referencing a typed configured feature. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new placed feature referencing a typed configured feature.", feature = "`feature` provides the typed Minecraft resource identifier used to create a new placed feature referencing a typed configured feature."),
        returns = "A `PlacedFeature` representing a new placed feature referencing a typed configured feature.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, feature: sand::registry::ConfiguredFeatureId)  {\n    let placed_feature = sand::component::PlacedFeature::new(location, feature);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::new_raw_feature",
        aliases = ["sand::prelude::PlacedFeature::new_raw_feature"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a placed feature with an explicitly raw configured-feature reference.",
        context = "Creates a placed feature with an explicitly raw configured-feature reference. Prefer [`PlacedFeature::new`] with a [`ConfiguredFeatureId`]. This escape hatch exists for modded or version-specific reference syntax.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`PlacedFeature::new`] with a [`ConfiguredFeatureId`]. This escape hatch exists for modded or version-specific reference syntax."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a placed feature with an explicitly raw configured-feature reference.", feature = "`feature` is used when creating a placed feature with an explicitly raw configured-feature reference."),
        returns = "A `PlacedFeature` representing a placed feature with an explicitly raw configured-feature reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, feature: impl Into < String >)  {\n    let placed_feature = sand::component::PlacedFeature::new_raw_feature(location, feature);\n}",
    )]
    pub fn new_raw_feature(location: ResourceLocation, feature: impl Into<String>) -> Self {
        Self {
            location,
            feature: ConfiguredFeatureReference::Raw(feature.into()),
            placement: Vec::new(),
        }
    }

    /// Updates the referenced configured feature.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::feature",
        aliases = ["sand::prelude::PlacedFeature::feature"],
        module = "sand::component",
        kind = "method",
        summary = "Updates the referenced configured feature.",
        context = "Updates the referenced configured feature. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(feature = "`feature` provides the typed Minecraft resource identifier used to update the referenced configured feature."),
        returns = "The `PlacedFeature` value with the documented change applied to update the referenced configured feature.",
        example = "use sand::prelude::*;\n\nfn demonstrate(placed_feature_value: sand::component::PlacedFeature, feature: sand::registry::ConfiguredFeatureId)  {\n    let updated_placed_feature = placed_feature_value.feature(feature);\n}",
    )]
    pub fn feature(mut self, feature: ConfiguredFeatureId) -> Self {
        self.feature = ConfiguredFeatureReference::Typed(feature);
        self
    }

    /// Updates the referenced configured feature through the explicit raw
    /// compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::raw_feature",
        aliases = ["sand::prelude::PlacedFeature::raw_feature"],
        module = "sand::component",
        kind = "method",
        summary = "Updates the referenced configured feature through the explicit raw compatibility path.",
        context = "Updates the referenced configured feature through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(feature = "`feature` is used to update the referenced configured feature through the explicit raw compatibility path."),
        returns = "The `PlacedFeature` value with the documented change applied to update the referenced configured feature through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(placed_feature_value: sand::component::PlacedFeature, feature: impl Into < String >)  {\n    let updated_placed_feature = placed_feature_value.raw_feature(feature);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::placement_modifier",
        aliases = ["sand::prelude::PlacedFeature::placement_modifier"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a placement modifier through the explicit raw JSON escape hatch.",
        context = "Adds a placement modifier through the explicit raw JSON escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(modifier = "`modifier` provides the modifier added when building a placement modifier through the explicit raw JSON escape hatch."),
        returns = "The `PlacedFeature` value with the documented change applied to add a placement modifier through the explicit raw JSON escape hatch.",
        example = "use sand::component::RawJson;\nuse serde_json::json;\nfeature.placement_modifier(RawJson::new(json!({ \"type\": \"minecraft:count\", \"count\": 5 })));",
    )]
    pub fn placement_modifier(mut self, modifier: RawJson) -> Self {
        self.placement.push(modifier.into_value());
        self
    }

    /// Sets all placement modifiers from explicit raw JSON escape-hatch values.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PlacedFeature::placement",
        aliases = ["sand::prelude::PlacedFeature::placement"],
        module = "sand::component",
        kind = "method",
        summary = "Sets all placement modifiers from explicit raw JSON escape-hatch values.",
        context = "Sets all placement modifiers from explicit raw JSON escape-hatch values. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(modifiers = "`modifiers` provides the modifiers applied when setting all placement modifiers from explicit raw JSON escape-hatch values."),
        returns = "The `PlacedFeature` value with the documented change applied to set all placement modifiers from explicit raw JSON escape-hatch values.",
        example = "use sand::prelude::*;\n\nfn demonstrate(placed_feature_value: sand::component::PlacedFeature, modifiers: impl IntoIterator < Item = sand::component::RawJson >)  {\n    let updated_placed_feature = placed_feature_value.placement(modifiers);\n}",
    )]
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
