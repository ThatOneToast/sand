//! Builder for `data/<namespace>/worldgen/configured_feature/<id>.json`.
//!
//! A configured feature pairs a feature type with the config that feature type
//! expects. [`PlacedFeature`](crate::worldgen::PlacedFeature) then references a
//! configured feature by ID and decides where it generates.
//!
//! This module models a small typed slice of the stable vanilla feature
//! shapes ([`ConfiguredFeature::no_op`], [`ConfiguredFeature::simple_block`],
//! [`ConfiguredFeature::fill_layer`], [`ConfiguredFeature::ore`]). Vanilla's
//! feature schemas are broad and version-sensitive, so anything outside that
//! slice — including modded feature types — uses the explicitly named
//! [`ConfiguredFeature::raw`] escape hatch rather than anonymous JSON on the
//! normal path.

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{BlockId, ConfiguredFeatureId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::providers::{BlockState, BlockStateProvider};

const KIND: &str = "worldgen/configured_feature";

/// Vanilla's maximum world height, used as the `fill_layer` height bound.
const MAX_FILL_LAYER_HEIGHT: u32 = 4064;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::RuleTest",
    aliases = ["sand::prelude::RuleTest"],
    module = "sand::component",
    summary = "A block-matching rule test used by ore-like feature configs.",
    context = "A block-matching rule test used by ore-like feature configs. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::RuleTest;",
    variants(AlwaysTrue = "`minecraft:always_true` — replaces any block.", BlockMatch = "`minecraft:block_match` — replaces exactly one block.", TagMatch = "`minecraft:tag_match` — replaces any block in a block tag."),
    variant_fields(BlockMatch(block = "`block` provides the block identifier when `minecraft:block_match` — replaces exactly one block."), TagMatch(tag = "`tag` provides the tag identifier when `minecraft:tag_match` — replaces any block in a block tag.")),
)]
/// A block-matching rule test used by ore-like feature configs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTest {
    /// `minecraft:always_true` — replaces any block.
    AlwaysTrue,
    /// `minecraft:block_match` — replaces exactly one block.
    BlockMatch {
        #[doc = "`block` provides the block identifier when `minecraft:block_match` — replaces exactly one block."]
        block: BlockId,
    },
    /// `minecraft:tag_match` — replaces any block in a block tag.
    TagMatch {
        #[doc = "`tag` provides the tag identifier when `minecraft:tag_match` — replaces any block in a block tag."]
        tag: TagId<BlockId>,
    },
}

impl RuleTest {
    fn to_json(&self) -> Value {
        match self {
            Self::AlwaysTrue => serde_json::json!({ "predicate_type": "minecraft:always_true" }),
            Self::BlockMatch { block } => serde_json::json!({
                "predicate_type": "minecraft:block_match",
                "block": block.to_string(),
            }),
            Self::TagMatch { tag } => serde_json::json!({
                "predicate_type": "minecraft:tag_match",
                "tag": tag.to_string(),
            }),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::OreTarget",
    aliases = ["sand::prelude::OreTarget"],
    module = "sand::component",
    summary = "One replaceable-target entry of an ore feature config.",
    context = "One replaceable-target entry of an ore feature config. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::OreTarget;",
)]
/// One replaceable-target entry of an ore feature config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OreTarget {
    target: RuleTest,
    state: BlockState,
}

impl OreTarget {
    /// Place `state` wherever `target` matches.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::OreTarget::new",
        aliases = ["sand::prelude::OreTarget::new"],
        module = "sand::component",
        kind = "method",
        summary = "Place `state` wherever `target` matches.",
        context = "Place `state` wherever `target` matches. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(target = "Place `state` wherever `target` matches.", state = "Place `state` wherever `target` matches."),
        returns = "A newly constructed `OreTarget` configured to place `state` wherever `target` matches.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::component::RuleTest, state: sand::component::BlockState)  {\n    let ore_target = sand::component::OreTarget::new(target, state);\n}",
    )]
    pub fn new(target: RuleTest, state: BlockState) -> Self {
        Self { target, state }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "target": self.target.to_json(),
            "state": self.state.to_json(),
        })
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::OreConfig",
    aliases = ["sand::prelude::OreConfig"],
    module = "sand::component",
    summary = "Config for the `minecraft:ore` feature type.",
    context = "Config for the `minecraft:ore` feature type. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::OreConfig;",
)]
/// Config for the `minecraft:ore` feature type.
#[derive(Debug, Clone, PartialEq)]
pub struct OreConfig {
    size: u32,
    targets: Vec<OreTarget>,
    discard_chance_on_air_exposure: f32,
}

impl OreConfig {
    /// Create an ore config with the given vein size and replaceable targets.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::OreConfig::new",
        aliases = ["sand::prelude::OreConfig::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create an ore config with the given vein size and replaceable targets.",
        context = "Create an ore config with the given vein size and replaceable targets. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(size = "`size` supplies the size value used to create an ore config with the given vein size and replaceable targets.", targets = "`targets` provides the Minecraft target selection used to create an ore config with the given vein size and replaceable targets."),
        returns = "A newly constructed `OreConfig` configured to create an ore config with the given vein size and replaceable targets.",
        example = "use sand::prelude::*;\n\nfn demonstrate(size: u32, targets: impl IntoIterator < Item = sand::component::OreTarget >)  {\n    let ore_config = sand::component::OreConfig::new(size, targets);\n}",
    )]
    pub fn new(size: u32, targets: impl IntoIterator<Item = OreTarget>) -> Self {
        Self {
            size,
            targets: targets.into_iter().collect(),
            discard_chance_on_air_exposure: 0.0,
        }
    }

    /// Probability (`0..=1`) that a vein block exposed to air is discarded.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::OreConfig::discard_chance_on_air_exposure",
        aliases = ["sand::prelude::OreConfig::discard_chance_on_air_exposure"],
        module = "sand::component",
        kind = "method",
        summary = "Probability (`0..=1`) that a vein block exposed to air is discarded.",
        context = "Probability (`0..=1`) that a vein block exposed to air is discarded. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(chance = "`chance` supplies the chance value used to probability (`0..=1`) that a vein block exposed to air is discarded."),
        returns = "The `OreConfig` value with the documented change applied to probability (`0..=1`) that a vein block exposed to air is discarded.",
        example = "use sand::prelude::*;\n\nfn demonstrate(ore_config_value: sand::component::OreConfig, chance: f32)  {\n    let updated_ore_config = ore_config_value.discard_chance_on_air_exposure(chance);\n}",
    )]
    pub fn discard_chance_on_air_exposure(mut self, chance: f32) -> Self {
        self.discard_chance_on_air_exposure = chance;
        self
    }

    fn to_json(&self) -> Value {
        let targets: Vec<Value> = self.targets.iter().map(OreTarget::to_json).collect();
        serde_json::json!({
            "size": self.size,
            "discard_chance_on_air_exposure": self.discard_chance_on_air_exposure,
            "targets": targets,
        })
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        validation::require_u32_in_range(location, KIND, "config.size", self.size, 0, 64)?;
        validation::require_non_empty_collection(
            location,
            KIND,
            "config.targets",
            self.targets.len(),
        )?;
        validation::require_finite_f32(
            location,
            KIND,
            "config.discard_chance_on_air_exposure",
            self.discard_chance_on_air_exposure,
        )?;
        if !(0.0..=1.0).contains(&self.discard_chance_on_air_exposure) {
            return Err(validation::error(
                location,
                KIND,
                "config.discard_chance_on_air_exposure",
                &format!(
                    "discard_chance_on_air_exposure must be in 0..=1; received {}",
                    self.discard_chance_on_air_exposure
                ),
            ));
        }
        for (index, target) in self.targets.iter().enumerate() {
            target
                .state
                .validate(location, KIND, &format!("config.targets[{index}].state"))?;
        }
        Ok(())
    }
}

/// The typed feature type and its config.
#[derive(Debug, Clone, PartialEq)]
enum Feature {
    NoOp,
    SimpleBlock {
        to_place: BlockStateProvider,
    },
    FillLayer {
        state: BlockState,
        height: u32,
    },
    Ore(OreConfig),
    Raw {
        feature_type: ResourceLocation,
        config: RawJson,
    },
}

impl Feature {
    fn feature_type(&self) -> String {
        match self {
            Self::NoOp => "minecraft:no_op".to_string(),
            Self::SimpleBlock { .. } => "minecraft:simple_block".to_string(),
            Self::FillLayer { .. } => "minecraft:fill_layer".to_string(),
            Self::Ore(_) => "minecraft:ore".to_string(),
            Self::Raw { feature_type, .. } => feature_type.to_string(),
        }
    }

    fn config_json(&self) -> Value {
        match self {
            Self::NoOp => serde_json::json!({}),
            Self::SimpleBlock { to_place } => serde_json::json!({
                "to_place": to_place.to_json(),
            }),
            Self::FillLayer { state, height } => serde_json::json!({
                "state": state.to_json(),
                "height": height,
            }),
            Self::Ore(config) => config.to_json(),
            Self::Raw { config, .. } => config.as_value().clone(),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ConfiguredFeature",
    aliases = ["sand::prelude::ConfiguredFeature"],
    module = "sand::component",
    summary = "A configured feature definition (`data/<namespace>/worldgen/configured_feature/<id>.json`).",
    context = "A configured feature definition (`data/<namespace>/worldgen/configured_feature/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ConfiguredFeature;",
)]
/// A configured feature definition
/// (`data/<namespace>/worldgen/configured_feature/<id>.json`).
///
/// ```
/// use sand_components::{BlockId, DatapackComponent, ResourceLocation};
/// use sand_components::worldgen::ConfiguredFeature;
/// use sand_components::worldgen::providers::{BlockState, BlockStateProvider};
///
/// let feature = ConfiguredFeature::simple_block(
///     ResourceLocation::new("example", "lone_fern").unwrap(),
///     BlockStateProvider::simple(BlockState::new(BlockId::minecraft("fern").unwrap())),
/// );
/// assert_eq!(feature.component_dir(), "worldgen/configured_feature");
/// assert_eq!(feature.to_json()["type"], "minecraft:simple_block");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ConfiguredFeature {
    location: ResourceLocation,
    feature: Feature,
}

impl ConfiguredFeature {
    /// A `minecraft:no_op` feature that generates nothing.
    ///
    /// Useful as a placeholder target while a pack's placement rules are being
    /// developed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::no_op",
        aliases = ["sand::prelude::ConfiguredFeature::no_op"],
        module = "sand::component",
        kind = "method",
        summary = "A `minecraft:no_op` feature that generates nothing.",
        context = "A `minecraft:no_op` feature that generates nothing. Useful as a placeholder target while a pack's placement rules are being developed.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use a `minecraft:no_op` feature that generates nothing."),
        returns = "A newly constructed `ConfiguredFeature` configured to use a `minecraft:no_op` feature that generates nothing.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let configured_feature = sand::component::ConfiguredFeature::no_op(location);\n}",
    )]
    pub fn no_op(location: ResourceLocation) -> Self {
        Self {
            location,
            feature: Feature::NoOp,
        }
    }

    /// A `minecraft:simple_block` feature that places a single block state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::simple_block",
        aliases = ["sand::prelude::ConfiguredFeature::simple_block"],
        module = "sand::component",
        kind = "method",
        summary = "A `minecraft:simple_block` feature that places a single block state.",
        context = "A `minecraft:simple_block` feature that places a single block state. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use a `minecraft:simple_block` feature that places a single block state.", to_place = "`to_place` supplies the to place value used to use a `minecraft:simple_block` feature that places a single block state."),
        returns = "A newly constructed `ConfiguredFeature` configured to use a `minecraft:simple_block` feature that places a single block state.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, to_place: sand::component::BlockStateProvider)  {\n    let configured_feature = sand::component::ConfiguredFeature::simple_block(location, to_place);\n}",
    )]
    pub fn simple_block(location: ResourceLocation, to_place: BlockStateProvider) -> Self {
        Self {
            location,
            feature: Feature::SimpleBlock { to_place },
        }
    }

    /// A `minecraft:fill_layer` feature that fills one world layer with a state.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::fill_layer",
        aliases = ["sand::prelude::ConfiguredFeature::fill_layer"],
        module = "sand::component",
        kind = "method",
        summary = "A `minecraft:fill_layer` feature that fills one world layer with a state.",
        context = "A `minecraft:fill_layer` feature that fills one world layer with a state. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use a `minecraft:fill_layer` feature that fills one world layer with a state.", state = "`state` supplies the state value used to use a `minecraft:fill_layer` feature that fills one world layer with a state.", height = "`height` supplies the height value used to use a `minecraft:fill_layer` feature that fills one world layer with a state."),
        returns = "A newly constructed `ConfiguredFeature` configured to use a `minecraft:fill_layer` feature that fills one world layer with a state.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, state: sand::component::BlockState, height: u32)  {\n    let configured_feature = sand::component::ConfiguredFeature::fill_layer(location, state, height);\n}",
    )]
    pub fn fill_layer(location: ResourceLocation, state: BlockState, height: u32) -> Self {
        Self {
            location,
            feature: Feature::FillLayer { state, height },
        }
    }

    /// A `minecraft:ore` feature that replaces matching blocks with ore veins.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::ore",
        aliases = ["sand::prelude::ConfiguredFeature::ore"],
        module = "sand::component",
        kind = "method",
        summary = "A `minecraft:ore` feature that replaces matching blocks with ore veins.",
        context = "A `minecraft:ore` feature that replaces matching blocks with ore veins. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use a `minecraft:ore` feature that replaces matching blocks with ore veins.", config = "`config` supplies the config value used to use a `minecraft:ore` feature that replaces matching blocks with ore veins."),
        returns = "A newly constructed `ConfiguredFeature` configured to use a `minecraft:ore` feature that replaces matching blocks with ore veins.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, config: sand::component::OreConfig)  {\n    let configured_feature = sand::component::ConfiguredFeature::ore(location, config);\n}",
    )]
    pub fn ore(location: ResourceLocation, config: OreConfig) -> Self {
        Self {
            location,
            feature: Feature::Ore(config),
        }
    }

    /// Author a configured feature from an explicitly raw feature type and
    /// config object.
    ///
    /// Prefer the typed constructors. This escape hatch exists for modded
    /// feature types and for vanilla configs outside the typed slice (trees,
    /// selectors, decorated shapes, and other version-sensitive schemas).
    /// The config must still be a JSON object.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::raw",
        aliases = ["sand::prelude::ConfiguredFeature::raw"],
        module = "sand::component",
        kind = "method",
        summary = "Author a configured feature from an explicitly raw feature type and config object.",
        context = "Author a configured feature from an explicitly raw feature type and config object. Prefer the typed constructors. This escape hatch exists for modded feature types and for vanilla configs outside the typed slice (trees, selectors, decorated shapes, and other version-sensitive schemas). The config must still be a JSON object.",
        minecraft = "Prefer the typed constructors. This escape hatch exists for modded feature types and for vanilla configs outside the typed slice (trees, selectors, decorated shapes, and other version-sensitive schemas). The config must still be a JSON object.",
        use_when = ["Prefer the typed constructors. This escape hatch exists for modded feature types and for vanilla configs outside the typed slice (trees, selectors, decorated shapes, and other version-sensitive schemas). The config must still be a JSON object."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use author a configured feature from an explicitly raw feature type and config object.", feature_type = "`feature_type` provides the typed Minecraft resource identifier used to use author a configured feature from an explicitly raw feature type and config object.", config = "`config` supplies the config value used to use author a configured feature from an explicitly raw feature type and config object."),
        returns = "A newly constructed `ConfiguredFeature` configured to use author a configured feature from an explicitly raw feature type and config object.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, feature_type: sand::ResourceLocation, config: sand::component::RawJson)  {\n    let configured_feature = sand::component::ConfiguredFeature::raw(location, feature_type, config);\n}",
    )]
    pub fn raw(
        location: ResourceLocation,
        feature_type: ResourceLocation,
        config: RawJson,
    ) -> Self {
        Self {
            location,
            feature: Feature::Raw {
                feature_type,
                config,
            },
        }
    }

    /// The typed ID other worldgen components use to reference this feature.
    ///
    /// ```
    /// use sand_components::ResourceLocation;
    /// use sand_components::worldgen::{ConfiguredFeature, PlacedFeature};
    ///
    /// let feature =
    ///     ConfiguredFeature::no_op(ResourceLocation::new("example", "nothing").unwrap());
    /// let placed = PlacedFeature::new(
    ///     ResourceLocation::new("example", "nowhere").unwrap(),
    ///     feature.id(),
    /// );
    /// # let _ = placed;
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ConfiguredFeature::id",
        aliases = ["sand::prelude::ConfiguredFeature::id"],
        module = "sand::component",
        kind = "method",
        summary = "The typed ID other worldgen components use to reference this feature.",
        context = "The typed ID other worldgen components use to reference this feature. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `ConfiguredFeatureId` value produced to use the typed ID other worldgen components use to reference this feature.",
        example = "use sand::ResourceLocation;\nuse {sand::component::ConfiguredFeature, sand::component::PlacedFeature};\nlet feature =\nConfiguredFeature::no_op(ResourceLocation::new(\"example\", \"nothing\").unwrap());\nlet placed = PlacedFeature::new(\nResourceLocation::new(\"example\", \"nowhere\").unwrap(),\nfeature.id(),\n);",
    )]
    pub fn id(&self) -> ConfiguredFeatureId {
        ConfiguredFeatureId::custom(self.location.clone())
    }
}

impl DatapackComponent for ConfiguredFeature {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "type": self.feature.feature_type(),
            "config": self.feature.config_json(),
        })
    }

    fn validate(&self) -> SandResult<()> {
        match &self.feature {
            Feature::NoOp => Ok(()),
            Feature::SimpleBlock { to_place } => {
                to_place.validate(&self.location, KIND, "config.to_place")
            }
            Feature::FillLayer { state, height } => {
                state.validate(&self.location, KIND, "config.state")?;
                validation::require_u32_in_range(
                    &self.location,
                    KIND,
                    "config.height",
                    *height,
                    0,
                    MAX_FILL_LAYER_HEIGHT,
                )
            }
            Feature::Ore(config) => config.validate(&self.location),
            Feature::Raw {
                feature_type,
                config,
            } => {
                validation::validate_resource_location_str(
                    &self.location,
                    KIND,
                    "type",
                    &feature_type.to_string(),
                )?;
                validation::require_json_object(&self.location, KIND, "config", config.as_value())
            }
        }
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/configured_feature"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "ashen_ore").unwrap()
    }

    #[test]
    fn minimal_no_op_feature_is_valid_and_stable() {
        let feature = ConfiguredFeature::no_op(location());
        feature.validate().unwrap();
        assert_eq!(
            feature.to_json(),
            serde_json::json!({"type": "minecraft:no_op", "config": {}})
        );
        assert_eq!(feature.component_dir(), "worldgen/configured_feature");
    }

    #[test]
    fn simple_block_and_fill_layer_shapes_serialize() {
        let simple = ConfiguredFeature::simple_block(
            location(),
            BlockStateProvider::simple(BlockState::new(BlockId::minecraft("fern").unwrap())),
        );
        simple.validate().unwrap();
        assert_eq!(
            simple.to_json(),
            serde_json::json!({
                "type": "minecraft:simple_block",
                "config": {
                    "to_place": {
                        "type": "minecraft:simple_state_provider",
                        "state": {"Name": "minecraft:fern"},
                    }
                }
            })
        );

        let fill = ConfiguredFeature::fill_layer(
            location(),
            BlockState::new(BlockId::minecraft("bedrock").unwrap()),
            0,
        );
        fill.validate().unwrap();
        assert_eq!(fill.to_json()["config"]["height"], 0);
    }

    #[test]
    fn ore_feature_serializes_typed_targets() {
        let feature = ConfiguredFeature::ore(
            location(),
            OreConfig::new(
                9,
                [
                    OreTarget::new(
                        RuleTest::TagMatch {
                            tag: TagId::minecraft("stone_ore_replaceables").unwrap(),
                        },
                        BlockState::new(BlockId::minecraft("iron_ore").unwrap()),
                    ),
                    OreTarget::new(
                        RuleTest::BlockMatch {
                            block: BlockId::minecraft("deepslate").unwrap(),
                        },
                        BlockState::new(BlockId::minecraft("deepslate_iron_ore").unwrap()),
                    ),
                ],
            )
            .discard_chance_on_air_exposure(0.5),
        );
        feature.validate().unwrap();
        let json = feature.to_json();
        assert_eq!(json["type"], "minecraft:ore");
        assert_eq!(json["config"]["size"], 9);
        assert_eq!(json["config"]["discard_chance_on_air_exposure"], 0.5);
        assert_eq!(
            json["config"]["targets"][0]["target"],
            serde_json::json!({
                "predicate_type": "minecraft:tag_match",
                "tag": "minecraft:stone_ore_replaceables",
            })
        );
        assert_eq!(
            json["config"]["targets"][1]["state"]["Name"],
            "minecraft:deepslate_iron_ore"
        );
    }

    #[test]
    fn always_true_rule_test_serializes() {
        let feature = ConfiguredFeature::ore(
            location(),
            OreConfig::new(
                1,
                [OreTarget::new(
                    RuleTest::AlwaysTrue,
                    BlockState::new(BlockId::minecraft("gold_ore").unwrap()),
                )],
            ),
        );
        feature.validate().unwrap();
        assert_eq!(
            feature.to_json()["config"]["targets"][0]["target"]["predicate_type"],
            "minecraft:always_true"
        );
    }

    #[test]
    fn invalid_ore_configs_are_rejected() {
        let empty_targets = ConfiguredFeature::ore(location(), OreConfig::new(4, []));
        let err = empty_targets.validate().unwrap_err().to_string();
        assert!(err.contains("config.targets"), "{err}");

        let oversized = ConfiguredFeature::ore(
            location(),
            OreConfig::new(
                65,
                [OreTarget::new(
                    RuleTest::AlwaysTrue,
                    BlockState::new(BlockId::minecraft("gold_ore").unwrap()),
                )],
            ),
        );
        assert!(oversized.validate().is_err());

        for chance in [f32::NAN, -0.1, 1.5] {
            let feature = ConfiguredFeature::ore(
                location(),
                OreConfig::new(
                    4,
                    [OreTarget::new(
                        RuleTest::AlwaysTrue,
                        BlockState::new(BlockId::minecraft("gold_ore").unwrap()),
                    )],
                )
                .discard_chance_on_air_exposure(chance),
            );
            assert!(feature.validate().is_err());
        }
    }

    #[test]
    fn out_of_range_fill_layer_height_is_rejected() {
        let feature = ConfiguredFeature::fill_layer(
            location(),
            BlockState::new(BlockId::minecraft("bedrock").unwrap()),
            MAX_FILL_LAYER_HEIGHT + 1,
        );
        let err = feature.validate().unwrap_err().to_string();
        assert!(err.contains("config.height"), "{err}");
    }

    #[test]
    fn malformed_block_state_properties_are_rejected() {
        let feature = ConfiguredFeature::simple_block(
            location(),
            BlockStateProvider::simple(
                BlockState::new(BlockId::minecraft("oak_log").unwrap()).property("", "y"),
            ),
        );
        assert!(feature.validate().is_err());
    }

    #[test]
    fn malformed_resource_ids_are_rejected_at_construction() {
        assert!("minecraft:bad path".parse::<ResourceLocation>().is_err());
        assert!("minecraft:Bad".parse::<ConfiguredFeatureId>().is_err());
    }

    #[test]
    fn raw_escape_hatch_preserves_custom_feature_json() {
        let feature = ConfiguredFeature::raw(
            location(),
            ResourceLocation::new("modded", "arcane_growth").unwrap(),
            RawJson::new(serde_json::json!({"potency": 3})),
        );
        feature.validate().unwrap();
        assert_eq!(
            feature.to_json(),
            serde_json::json!({
                "type": "modded:arcane_growth",
                "config": {"potency": 3},
            })
        );
    }

    #[test]
    fn raw_config_with_invalid_top_level_shape_is_rejected() {
        for config in [
            serde_json::json!(5),
            serde_json::json!([{"type": "minecraft:no_op"}]),
            serde_json::json!("no_op"),
            serde_json::json!(null),
        ] {
            let feature = ConfiguredFeature::raw(
                location(),
                ResourceLocation::new("modded", "arcane_growth").unwrap(),
                RawJson::new(config),
            );
            let err = feature.validate().unwrap_err().to_string();
            assert!(err.contains("config"), "{err}");
        }
    }

    #[test]
    fn typed_id_accessor_matches_the_component_location() {
        let feature = ConfiguredFeature::no_op(location());
        assert_eq!(feature.id().to_string(), "my_pack:ashen_ore");
    }
}
