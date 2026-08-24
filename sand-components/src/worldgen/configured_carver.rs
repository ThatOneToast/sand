//! Builder for `data/<namespace>/worldgen/configured_carver/<id>.json`.
//!
//! A configured carver pairs a carver type with the config that carver type
//! expects. [`crate::worldgen::Biome::carver_step`] then references a
//! configured carver by typed ID, grouped by carving step (`air`/`liquid`).
//!
//! This module models the common vanilla `minecraft:cave` and
//! `minecraft:nether_cave` carver shapes, which share the same config
//! (`CaveCarverConfig`). Vanilla's carver schemas are version-sensitive, so
//! anything outside that slice — including modded carver types — uses the
//! explicitly named [`ConfiguredCarver::raw`] escape hatch rather than
//! anonymous JSON on the normal path. `CaveCarverConfig::config_field` is a
//! narrower escape hatch for extra config keys (for example
//! `horizontal_radius_multiplier`) on an otherwise typed cave-shaped config.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::ConfiguredCarverId;
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::providers::{HeightProvider, VerticalAnchor};

const KIND: &str = "worldgen/configured_carver";

/// Serialize an `f32` through its own shortest decimal representation.
///
/// `serde_json::json!`/`to_value` on an `f32` widen it to `f64` by bit
/// pattern, which surfaces the `f32`'s binary rounding error in the JSON
/// text (e.g. `0.15f32` becomes `0.15000000596046448`). Round-tripping
/// through `f32`'s `Display` (which is shortest-round-trip) avoids that.
fn f32_to_json(value: f32) -> Value {
    Value::from(value.to_string().parse::<f64>().unwrap())
}

/// Config keys `CaveCarverConfig` models directly; `config_field` refuses to
/// override these.
const TYPED_CONFIG_FIELDS: &[&str] = &["probability", "y", "yScale", "lava_level"];

/// A `minecraft:uniform` float provider (`{"type": "minecraft:uniform",
/// "min_inclusive": …, "max_inclusive": …}`), used for the `yScale` field of
/// cave-shaped carver configs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarverFloatRange {
    min_inclusive: f32,
    max_inclusive: f32,
}

impl CarverFloatRange {
    /// Create a uniformly sampled inclusive float range.
    #[doc = "**API Contract:** Run `sand api show sand::component::CarverFloatRange::new` for the canonical contract."]
    pub fn new(min_inclusive: f32, max_inclusive: f32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
        }
    }

    fn to_json(self) -> Value {
        serde_json::json!({
            "type": "minecraft:uniform",
            "min_inclusive": f32_to_json(self.min_inclusive),
            "max_inclusive": f32_to_json(self.max_inclusive),
        })
    }

    fn validate(self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        validation::require_finite_f32(
            location,
            KIND,
            &format!("{field}.min_inclusive"),
            self.min_inclusive,
        )?;
        validation::require_finite_f32(
            location,
            KIND,
            &format!("{field}.max_inclusive"),
            self.max_inclusive,
        )?;
        if self.min_inclusive > self.max_inclusive {
            return Err(validation::error(
                location,
                KIND,
                field,
                &format!(
                    "min_inclusive must not exceed max_inclusive; received {}..={}",
                    self.min_inclusive, self.max_inclusive
                ),
            ));
        }
        Ok(())
    }
}

/// Config shared by the `minecraft:cave` and `minecraft:nether_cave` carver
/// types.
#[derive(Debug, Clone, PartialEq)]
pub struct CaveCarverConfig {
    probability: f32,
    y: HeightProvider,
    y_scale: CarverFloatRange,
    lava_level: VerticalAnchor,
    extra_fields: BTreeMap<String, RawJson>,
}

impl CaveCarverConfig {
    /// Create a cave-shaped carver config.
    ///
    /// `probability` (`0..=1`) is the per-chunk chance this carver runs.
    #[doc = "**API Contract:** Run `sand api show sand::component::CaveCarverConfig::new` for the canonical contract."]
    pub fn new(
        probability: f32,
        y: HeightProvider,
        y_scale: CarverFloatRange,
        lava_level: VerticalAnchor,
    ) -> Self {
        Self {
            probability,
            y,
            y_scale,
            lava_level,
            extra_fields: BTreeMap::new(),
        }
    }

    /// Add a modded or version-specific config key not represented by the
    /// typed fields (for example `horizontal_radius_multiplier`).
    ///
    /// Typed field names (`probability`, `y`, `yScale`, `lava_level`) cannot
    /// be overridden through this escape hatch.
    #[doc = "**API Contract:** Run `sand api show sand::component::CaveCarverConfig::config_field` for the canonical contract."]
    pub fn config_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.extra_fields.insert(key.into(), value);
        self
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("probability".into(), f32_to_json(self.probability));
        map.insert("y".into(), self.y.to_json());
        map.insert("yScale".into(), self.y_scale.to_json());
        map.insert("lava_level".into(), self.lava_level.to_json());
        for (key, value) in &self.extra_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        validation::require_finite_f32(location, KIND, "config.probability", self.probability)?;
        if !(0.0..=1.0).contains(&self.probability) {
            return Err(validation::error(
                location,
                KIND,
                "config.probability",
                &format!(
                    "probability must be in 0..=1; received {}",
                    self.probability
                ),
            ));
        }
        self.y.validate(location, KIND, "config.y")?;
        self.y_scale.validate(location, "config.yScale")?;
        self.lava_level
            .validate(location, KIND, "config.lava_level")?;
        for key in self.extra_fields.keys() {
            validation::require_non_empty(location, KIND, "config_field", key)?;
            validation::reject_whitespace_only(location, KIND, "config_field", key)?;
            validation::reject_control_chars(location, KIND, "config_field", key)?;
            if TYPED_CONFIG_FIELDS.contains(&key.as_str()) {
                return Err(validation::error(
                    location,
                    KIND,
                    "config_field",
                    &format!("config field `{key}` would override a typed field"),
                ));
            }
        }
        Ok(())
    }
}

/// The typed carver type and its config.
#[derive(Debug, Clone, PartialEq)]
enum Carver {
    Cave(CaveCarverConfig),
    NetherCave(CaveCarverConfig),
    Raw {
        carver_type: ResourceLocation,
        config: RawJson,
    },
}

impl Carver {
    fn carver_type(&self) -> String {
        match self {
            Self::Cave(_) => "minecraft:cave".to_string(),
            Self::NetherCave(_) => "minecraft:nether_cave".to_string(),
            Self::Raw { carver_type, .. } => carver_type.to_string(),
        }
    }

    fn config_json(&self) -> Value {
        match self {
            Self::Cave(config) | Self::NetherCave(config) => config.to_json(),
            Self::Raw { config, .. } => config.as_value().clone(),
        }
    }
}

/// A configured carver definition
/// (`data/<namespace>/worldgen/configured_carver/<id>.json`).
///
/// ```
/// use sand_components::{DatapackComponent, ResourceLocation};
/// use sand_components::worldgen::ConfiguredCarver;
/// use sand_components::worldgen::configured_carver::{CarverFloatRange, CaveCarverConfig};
/// use sand_components::worldgen::providers::{HeightProvider, VerticalAnchor};
///
/// let carver = ConfiguredCarver::cave(
///     ResourceLocation::new("example", "shallow_cave").unwrap(),
///     CaveCarverConfig::new(
///         0.15,
///         HeightProvider::absolute(0),
///         CarverFloatRange::new(0.1, 0.9),
///         VerticalAnchor::Absolute(-54),
///     ),
/// );
/// assert_eq!(carver.component_dir(), "worldgen/configured_carver");
/// assert_eq!(carver.to_json()["type"], "minecraft:cave");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ConfiguredCarver {
    location: ResourceLocation,
    carver: Carver,
}

impl ConfiguredCarver {
    /// A `minecraft:cave` carver.
    #[doc = "**API Contract:** Run `sand api show sand::component::ConfiguredCarver::cave` for the canonical contract."]
    pub fn cave(location: ResourceLocation, config: CaveCarverConfig) -> Self {
        Self {
            location,
            carver: Carver::Cave(config),
        }
    }

    /// A `minecraft:nether_cave` carver.
    #[doc = "**API Contract:** Run `sand api show sand::component::ConfiguredCarver::nether_cave` for the canonical contract."]
    pub fn nether_cave(location: ResourceLocation, config: CaveCarverConfig) -> Self {
        Self {
            location,
            carver: Carver::NetherCave(config),
        }
    }

    /// Author a configured carver from an explicitly raw carver type and
    /// config object.
    ///
    /// Prefer [`ConfiguredCarver::cave`] / [`ConfiguredCarver::nether_cave`].
    /// This escape hatch exists for modded carver types and for vanilla
    /// configs outside the typed slice. The config must still be a JSON
    /// object.
    #[doc = "**API Contract:** Run `sand api show sand::component::ConfiguredCarver::raw` for the canonical contract."]
    pub fn raw(location: ResourceLocation, carver_type: ResourceLocation, config: RawJson) -> Self {
        Self {
            location,
            carver: Carver::Raw {
                carver_type,
                config,
            },
        }
    }

    /// The typed ID other worldgen components use to reference this carver.
    ///
    /// ```
    /// use sand_components::ResourceLocation;
    /// use sand_components::worldgen::configured_carver::{CarverFloatRange, CaveCarverConfig};
    /// use sand_components::worldgen::providers::{HeightProvider, VerticalAnchor};
    /// use sand_components::worldgen::ConfiguredCarver;
    ///
    /// let carver = ConfiguredCarver::cave(
    ///     ResourceLocation::new("example", "shallow_cave").unwrap(),
    ///     CaveCarverConfig::new(
    ///         0.15,
    ///         HeightProvider::absolute(0),
    ///         CarverFloatRange::new(0.1, 0.9),
    ///         VerticalAnchor::Absolute(-54),
    ///     ),
    /// );
    /// let id = carver.id();
    /// assert_eq!(id.as_resource_location().to_string(), "example:shallow_cave");
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::component::ConfiguredCarver::id` for the canonical contract."]
    pub fn id(&self) -> ConfiguredCarverId {
        ConfiguredCarverId::custom(self.location.clone())
    }
}

impl DatapackComponent for ConfiguredCarver {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "type": self.carver.carver_type(),
            "config": self.carver.config_json(),
        })
    }

    fn validate(&self) -> SandResult<()> {
        match &self.carver {
            Carver::Cave(config) | Carver::NetherCave(config) => config.validate(&self.location),
            Carver::Raw {
                carver_type,
                config,
            } => {
                validation::validate_resource_location_str(
                    &self.location,
                    KIND,
                    "type",
                    &carver_type.to_string(),
                )?;
                validation::require_json_object(&self.location, KIND, "config", config.as_value())
            }
        }
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/configured_carver"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("my_pack", "shallow_cave").unwrap()
    }

    fn config() -> CaveCarverConfig {
        CaveCarverConfig::new(
            0.15,
            HeightProvider::absolute(0),
            CarverFloatRange::new(0.1, 0.9),
            VerticalAnchor::Absolute(-54),
        )
    }

    #[test]
    fn cave_carver_shape_serializes_and_validates() {
        let carver = ConfiguredCarver::cave(location(), config());
        carver.validate().unwrap();
        assert_eq!(
            carver.to_json(),
            serde_json::json!({
                "type": "minecraft:cave",
                "config": {
                    "probability": 0.15,
                    "y": { "absolute": 0 },
                    "yScale": {
                        "type": "minecraft:uniform",
                        "min_inclusive": 0.1,
                        "max_inclusive": 0.9,
                    },
                    "lava_level": { "absolute": -54 },
                }
            })
        );
        assert_eq!(carver.component_dir(), "worldgen/configured_carver");
    }

    #[test]
    fn nether_cave_carver_shape_serializes() {
        let carver = ConfiguredCarver::nether_cave(location(), config());
        carver.validate().unwrap();
        assert_eq!(carver.to_json()["type"], "minecraft:nether_cave");
    }

    #[test]
    fn out_of_range_probability_is_rejected() {
        for probability in [f32::NAN, -0.1, 1.5] {
            let mut c = config();
            c.probability = probability;
            let carver = ConfiguredCarver::cave(location(), c);
            let err = carver.validate().unwrap_err().to_string();
            assert!(err.contains("config.probability"), "{err}");
        }
    }

    #[test]
    fn inverted_y_scale_range_is_rejected() {
        let mut c = config();
        c.y_scale = CarverFloatRange::new(0.9, 0.1);
        let carver = ConfiguredCarver::cave(location(), c);
        let err = carver.validate().unwrap_err().to_string();
        assert!(err.contains("config.yScale"), "{err}");
    }

    #[test]
    fn out_of_range_height_provider_is_rejected() {
        let mut c = config();
        c.y = HeightProvider::absolute(9000);
        let carver = ConfiguredCarver::cave(location(), c);
        assert!(carver.validate().is_err());
    }

    #[test]
    fn config_field_escape_hatch_extends_typed_config() {
        let carver = ConfiguredCarver::cave(
            location(),
            config().config_field(
                "horizontal_radius_multiplier",
                RawJson::new(serde_json::json!({
                    "type": "minecraft:uniform",
                    "min_inclusive": 0.7,
                    "max_inclusive": 1.4,
                })),
            ),
        );
        carver.validate().unwrap();
        assert_eq!(
            carver.to_json()["config"]["horizontal_radius_multiplier"]["min_inclusive"],
            0.7
        );
    }

    #[test]
    fn config_field_cannot_override_typed_field() {
        let carver = ConfiguredCarver::cave(
            location(),
            config().config_field("probability", RawJson::new(serde_json::json!(0.5))),
        );
        let err = carver.validate().unwrap_err().to_string();
        assert!(err.contains("config_field"), "{err}");
    }

    #[test]
    fn malformed_config_field_key_is_rejected() {
        let carver = ConfiguredCarver::cave(
            location(),
            config().config_field("", RawJson::new(serde_json::json!(1))),
        );
        assert!(carver.validate().is_err());
    }

    #[test]
    fn raw_escape_hatch_preserves_custom_carver_json() {
        let carver = ConfiguredCarver::raw(
            location(),
            ResourceLocation::new("modded", "arcane_cave").unwrap(),
            RawJson::new(serde_json::json!({"potency": 3})),
        );
        carver.validate().unwrap();
        assert_eq!(
            carver.to_json(),
            serde_json::json!({
                "type": "modded:arcane_cave",
                "config": {"potency": 3},
            })
        );
    }

    #[test]
    fn raw_config_with_invalid_top_level_shape_is_rejected() {
        for value in [
            serde_json::json!(5),
            serde_json::json!([{"type": "minecraft:cave"}]),
            serde_json::json!("cave"),
            serde_json::json!(null),
        ] {
            let carver = ConfiguredCarver::raw(
                location(),
                ResourceLocation::new("modded", "arcane_cave").unwrap(),
                RawJson::new(value),
            );
            let err = carver.validate().unwrap_err().to_string();
            assert!(err.contains("config"), "{err}");
        }
    }

    #[test]
    fn malformed_resource_ids_are_rejected_at_construction() {
        assert!("minecraft:bad path".parse::<ResourceLocation>().is_err());
        assert!("minecraft:Bad".parse::<ConfiguredCarverId>().is_err());
    }

    #[test]
    fn typed_id_accessor_matches_the_component_location() {
        let carver = ConfiguredCarver::cave(location(), config());
        assert_eq!(carver.id().to_string(), "my_pack:shallow_cave");
    }
}
