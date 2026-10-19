//! Placed feature builder for `data/<namespace>/worldgen/placed_feature/<id>.json`.

use serde_json::Value;

use super::{HeightProvider, Heightmap};
use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::ConfiguredFeatureId;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/placed_feature";

// Codec bounds verified against Minecraft 26.1 and 26.2 server classes:
// CountPlacement (IntProviders.codec(0, 4096)), RarityFilter (POSITIVE_INT),
// and SurfaceWaterDepthFilter (unbounded Codec.INT).
/// One ordered placement operation for a configured feature.
///
/// Modifiers form a pipeline: count repeats each input position, in-square
/// scatters it horizontally, and height modifiers choose its elevation. Order
/// is significant and is preserved on export. Numeric bounds are validated by
/// `PlacedFeature` before export. Unsupported providers/modifiers remain explicit
/// through `Raw`; raw payloads receive structural, not full codec validation.
#[sand_macros::api(
    registry = sand_api_contract, path = "sand::component::PlacementModifier",
    aliases = ["sand::prelude::PlacementModifier"], module = "sand::component",
    summary = "An ordered, typed worldgen placement operation.",
    context = "Compose common vanilla feature placement without anonymous JSON, reusing HeightProvider and Heightmap.",
    minecraft = "Serializes an ordered placement entry; PlacedFeature validates numeric bounds and nested providers before export.",
    use_when = ["Placing configured features with common vanilla modifiers"],
    avoid_when = ["Describing a configured feature's block-generation algorithm"],
    variants(Count = "Repeat each position zero through 4096 times.", RarityFilter = "Retain each position with probability one in the positive chance.", InSquare = "Scatter positions within the chunk horizontally.", Biome = "Keep positions where the biome allows this placed feature.", HeightRange = "Choose elevation with the shared height provider.", Heightmap = "Choose elevation from a chunk heightmap.", SurfaceWaterDepth = "Keep positions whose surface water depth does not exceed the maximum.", Raw = "Explicit unsupported placement JSON, structurally validated."),
    variant_fields(Count = ["Number of repetitions, in 0..=4096."], RarityFilter = ["Positive reciprocal probability."], HeightRange = ["The canonical height provider."], Heightmap = ["The canonical heightmap selector."], SurfaceWaterDepth = ["Maximum water depth in blocks."], Raw = ["An object with a valid resource-location type field."]),
    example = "use sand::component::PlacementModifier; let scatter = PlacementModifier::InSquare;",
)]
#[derive(Debug, Clone)]
pub enum PlacementModifier {
    /// Repeat each input position `0..=4096` times (a constant count provider).
    Count(i32),
    /// Keep each position with probability `1 / chance`; chance must be positive.
    RarityFilter(i32),
    /// Randomize horizontal coordinates within the chunk.
    InSquare,
    /// Keep only positions whose biome includes this placed feature.
    Biome,
    /// Sample elevation using the shared worldgen height-provider model.
    HeightRange(HeightProvider),
    /// Place at the selected chunk heightmap elevation.
    Heightmap(Heightmap),
    /// Filter by maximum surface water depth, in blocks.
    SurfaceWaterDepth(i32),
    /// Explicit unsupported or custom modifier JSON.
    Raw(RawJson),
}

impl PlacementModifier {
    fn to_json(&self) -> Value {
        match self {
            Self::Count(count) => serde_json::json!({"type": "minecraft:count", "count": count}),
            Self::RarityFilter(chance) => {
                serde_json::json!({"type": "minecraft:rarity_filter", "chance": chance})
            }
            Self::InSquare => serde_json::json!({"type": "minecraft:in_square"}),
            Self::Biome => serde_json::json!({"type": "minecraft:biome"}),
            Self::HeightRange(height) => {
                serde_json::json!({"type": "minecraft:height_range", "height": height.to_json()})
            }
            Self::Heightmap(heightmap) => {
                serde_json::json!({"type": "minecraft:heightmap", "heightmap": heightmap.as_str()})
            }
            Self::SurfaceWaterDepth(max) => {
                serde_json::json!({"type": "minecraft:surface_water_depth_filter", "max_water_depth": max})
            }
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        match self {
            Self::Count(count) if !(0..=4096).contains(count) => Err(validation::error(
                location,
                KIND,
                &format!("{field}.count"),
                "count must be in 0..=4096",
            )),
            Self::RarityFilter(chance) if *chance < 1 => Err(validation::error(
                location,
                KIND,
                &format!("{field}.chance"),
                "chance must be positive",
            )),
            Self::HeightRange(height) => {
                height.validate(location, KIND, &format!("{field}.height"))
            }
            Self::Raw(raw) => {
                let value = raw.as_value();
                validation::require_json_object(location, KIND, field, value)?;
                let ty = value.get("type").and_then(Value::as_str).ok_or_else(|| {
                    validation::error(
                        location,
                        KIND,
                        &format!("{field}.type"),
                        "placement modifier requires a string type",
                    )
                })?;
                validation::validate_resource_location_str(
                    location,
                    KIND,
                    &format!("{field}.type"),
                    ty,
                )
            }
            _ => Ok(()),
        }
    }
}

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
    /// Ordered placement pipeline.
    placement: Vec<PlacementModifier>,
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

    /// Append one typed modifier to the ordered placement pipeline.
    /// Use `PlacementModifier::Raw` for unsupported modifier/provider shapes.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::PlacedFeature::placement_modifier",
        aliases = ["sand::prelude::PlacedFeature::placement_modifier"], module = "sand::component", kind = "method",
        summary = "Append a typed placement modifier.", context = "Modifiers execute in insertion order; use PlacementModifier::Raw for unsupported schemas.",
        minecraft = "Adds one entry to the placement array, validated before export.",
        use_when = ["Building a feature placement pipeline"], avoid_when = ["Replacing the whole pipeline; use placement"],
        params(modifier = "The next placement operation."), returns = "The placed feature with the modifier appended.",
        example = "let placed = placed.placement_modifier(sand::component::PlacementModifier::InSquare);",
    )]
    pub fn placement_modifier(mut self, modifier: PlacementModifier) -> Self {
        self.placement.push(modifier);
        self
    }

    /// Replace the complete ordered placement pipeline.
    /// An empty pipeline places once at the supplied origin; it is valid.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::PlacedFeature::placement",
        aliases = ["sand::prelude::PlacedFeature::placement"], module = "sand::component", kind = "method",
        summary = "Replace the ordered placement pipeline.", context = "Keeps modifier order and validates typed numeric values and raw shapes at export.",
        minecraft = "Writes the placement array; an empty array leaves the input position unchanged.",
        use_when = ["Defining the complete placement pipeline"], avoid_when = ["Appending a single modifier; use placement_modifier"],
        params(modifiers = "Ordered placement operations."), returns = "The placed feature with the replacement pipeline.",
        example = "let placed = placed.placement([sand::component::PlacementModifier::InSquare, sand::component::PlacementModifier::Biome]);",
    )]
    pub fn placement(mut self, modifiers: impl IntoIterator<Item = PlacementModifier>) -> Self {
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
            "feature": self.feature.as_string(),
            "placement": self.placement.iter().map(PlacementModifier::to_json).collect::<Vec<_>>(),
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
            modifier.validate(&self.location, &field)?;
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

    fn raw(value: Value) -> PlacementModifier {
        PlacementModifier::Raw(RawJson::new(value))
    }

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "sparse_oak").unwrap()
    }

    fn oak() -> ConfiguredFeatureId {
        ConfiguredFeatureId::minecraft("oak").unwrap()
    }

    #[test]
    fn valid_placed_feature_exports_unchanged() {
        let feature = PlacedFeature::new(location(), oak()).placement_modifier(raw(
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
        let feature = PlacedFeature::new(location(), oak()).placement([raw(serde_json::json!(5))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn empty_placement_modifier_object_rejected() {
        let feature = PlacedFeature::new(location(), oak()).placement([raw(serde_json::json!({}))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0]"), "{err}");
    }

    #[test]
    fn placement_modifier_missing_type_rejected() {
        let feature =
            PlacedFeature::new(location(), oak()).placement([raw(serde_json::json!({"count": 5}))]);
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("placement[0].type"), "{err}");
    }

    #[test]
    fn raw_placement_modifier_escape_hatch_still_works() {
        let feature = PlacedFeature::new(location(), oak()).placement([raw(
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
    #[test]
    fn typed_pipeline_preserves_order_and_vanilla_shapes() {
        let feature = PlacedFeature::new(location(), oak()).placement([
            PlacementModifier::Count(5),
            PlacementModifier::InSquare,
            PlacementModifier::RarityFilter(3),
            PlacementModifier::HeightRange(HeightProvider::absolute(64)),
            PlacementModifier::Heightmap(Heightmap::OceanFloorWg),
            PlacementModifier::SurfaceWaterDepth(2),
            PlacementModifier::Biome,
        ]);
        feature.validate().unwrap();
        assert_eq!(
            feature.to_json(),
            serde_json::json!({
                "feature": "minecraft:oak",
                "placement": [
                    {"type": "minecraft:count", "count": 5},
                    {"type": "minecraft:in_square"},
                    {"type": "minecraft:rarity_filter", "chance": 3},
                    {"type": "minecraft:height_range", "height": {"absolute": 64}},
                    {"type": "minecraft:heightmap", "heightmap": "OCEAN_FLOOR_WG"},
                    {"type": "minecraft:surface_water_depth_filter", "max_water_depth": 2},
                    {"type": "minecraft:biome"}
                ]
            })
        );
    }

    #[test]
    fn typed_numeric_bounds_report_the_placement_index() {
        for modifier in [
            PlacementModifier::Count(-1),
            PlacementModifier::Count(4097),
            PlacementModifier::RarityFilter(0),
            PlacementModifier::RarityFilter(-1),
        ] {
            let feature = PlacedFeature::new(location(), oak())
                .placement([PlacementModifier::InSquare, modifier]);
            let error = feature.validate().unwrap_err().to_string();
            assert!(error.contains("placement[1]"), "{error}");
            assert!(error.contains("my_pack:sparse_oak"), "{error}");
        }
        for modifier in [
            PlacementModifier::Count(0),
            PlacementModifier::Count(4096),
            PlacementModifier::RarityFilter(1),
            PlacementModifier::RarityFilter(i32::MAX),
        ] {
            PlacedFeature::new(location(), oak())
                .placement_modifier(modifier)
                .validate()
                .unwrap();
        }
    }

    #[test]
    fn nested_height_provider_validation_and_pipeline_replacement() {
        use super::super::VerticalAnchor;
        let invalid = PlacementModifier::HeightRange(HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::Absolute(64),
            max_inclusive: VerticalAnchor::Absolute(0),
        });
        let feature = PlacedFeature::new(location(), oak()).placement_modifier(invalid);
        let error = feature.validate().unwrap_err().to_string();
        assert!(error.contains("placement[0].height"), "{error}");
        let replaced = feature.placement([]);
        replaced.validate().unwrap();
        assert_eq!(replaced.to_json()["placement"], serde_json::json!([]));
    }
}
