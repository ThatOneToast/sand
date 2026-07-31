//! Builder for `data/<namespace>/dimension_type/<id>.json`.
//!
//! [`DimensionType::new`] provides a valid, familiar starting
//! point. All stable fields have typed setters, while [`DimensionType::raw_field`]
//! is an explicit escape hatch for modded or version-specific additions.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{BlockId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;

const TYPED_FIELDS: &[&str] = &[
    "fixed_time",
    "has_skylight",
    "has_ceiling",
    "ultrawarm",
    "natural",
    "coordinate_scale",
    "bed_works",
    "respawn_anchor_works",
    "min_y",
    "height",
    "logical_height",
    "infiniburn",
    "effects",
    "ambient_light",
    "piglin_safe",
    "has_raids",
    "monster_spawn_light_level",
    "monster_spawn_block_light_limit",
];

/// The sky-light range in which monsters may spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterSpawnLightLevel {
    /// A single light level.
    Constant(u8),
    /// A uniformly sampled inclusive light-level range.
    Uniform {
        min_inclusive: u8,
        max_inclusive: u8,
    },
}

impl MonsterSpawnLightLevel {
    fn to_json(&self) -> Value {
        match self {
            Self::Constant(level) => serde_json::json!(level),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => serde_json::json!({
                "type": "minecraft:uniform",
                "value": {
                    "min_inclusive": min_inclusive,
                    "max_inclusive": max_inclusive,
                }
            }),
        }
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        let (min, max) = match self {
            Self::Constant(level) => (*level, *level),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => (*min_inclusive, *max_inclusive),
        };
        if max > 15 {
            return Err(validation::error(
                location,
                "dimension_type",
                "monster_spawn_light_level",
                &format!("light levels must be in 0..=15; received {min}..={max}"),
            ));
        }
        if min > max {
            return Err(validation::error(
                location,
                "dimension_type",
                "monster_spawn_light_level",
                &format!("min_inclusive must not exceed max_inclusive; received {min}..={max}"),
            ));
        }
        Ok(())
    }
}

/// A Minecraft dimension type definition.
///
/// The constructor uses overworld-like defaults, producing a complete valid
/// shape without requiring raw JSON:
///
/// ```
/// use sand_components::{DatapackComponent, DimensionType, ResourceLocation};
///
/// let ty = DimensionType::new(
///     ResourceLocation::new("example", "bright_overworld").unwrap(),
/// );
/// assert_eq!(ty.component_dir(), "dimension_type");
/// assert_eq!(ty.to_json()["effects"], "minecraft:overworld");
/// ```
pub struct DimensionType {
    location: ResourceLocation,
    fixed_time: Option<i64>,
    has_skylight: bool,
    has_ceiling: bool,
    ultrawarm: bool,
    natural: bool,
    coordinate_scale: f64,
    bed_works: bool,
    respawn_anchor_works: bool,
    min_y: i32,
    height: u32,
    logical_height: u32,
    infiniburn: TagId<BlockId>,
    effects: ResourceLocation,
    ambient_light: f32,
    piglin_safe: bool,
    has_raids: bool,
    monster_spawn_light_level: MonsterSpawnLightLevel,
    monster_spawn_block_light_limit: u8,
    raw_fields: BTreeMap<String, RawJson>,
}

impl DimensionType {
    /// Create a complete dimension type with vanilla overworld-like defaults.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            fixed_time: None,
            has_skylight: true,
            has_ceiling: false,
            ultrawarm: false,
            natural: true,
            coordinate_scale: 1.0,
            bed_works: true,
            respawn_anchor_works: false,
            min_y: -64,
            height: 384,
            logical_height: 384,
            infiniburn: TagId::minecraft("infiniburn_overworld")
                .expect("built-in infiniburn tag is valid"),
            effects: ResourceLocation::minecraft("overworld")
                .expect("built-in dimension effects ID is valid"),
            ambient_light: 0.0,
            piglin_safe: false,
            has_raids: true,
            monster_spawn_light_level: MonsterSpawnLightLevel::Uniform {
                min_inclusive: 0,
                max_inclusive: 7,
            },
            monster_spawn_block_light_limit: 0,
            raw_fields: BTreeMap::new(),
        }
    }

    /// Create a complete dimension type with vanilla overworld-like defaults.
    ///
    /// This named alias is convenient when an example should emphasize which
    /// vanilla behavior the defaults model.
    pub fn overworld_like(location: ResourceLocation) -> Self {
        Self::new(location)
    }

    pub fn fixed_time(mut self, time: i64) -> Self {
        self.fixed_time = Some(time);
        self
    }

    pub fn without_fixed_time(mut self) -> Self {
        self.fixed_time = None;
        self
    }

    pub fn has_skylight(mut self, value: bool) -> Self {
        self.has_skylight = value;
        self
    }

    pub fn has_ceiling(mut self, value: bool) -> Self {
        self.has_ceiling = value;
        self
    }

    pub fn ultrawarm(mut self, value: bool) -> Self {
        self.ultrawarm = value;
        self
    }

    pub fn natural(mut self, value: bool) -> Self {
        self.natural = value;
        self
    }

    pub fn coordinate_scale(mut self, value: f64) -> Self {
        self.coordinate_scale = value;
        self
    }

    pub fn bed_works(mut self, value: bool) -> Self {
        self.bed_works = value;
        self
    }

    pub fn respawn_anchor_works(mut self, value: bool) -> Self {
        self.respawn_anchor_works = value;
        self
    }

    pub fn min_y(mut self, value: i32) -> Self {
        self.min_y = value;
        self
    }

    pub fn height(mut self, value: u32) -> Self {
        self.height = value;
        self
    }

    pub fn logical_height(mut self, value: u32) -> Self {
        self.logical_height = value;
        self
    }

    pub fn infiniburn(mut self, value: TagId<BlockId>) -> Self {
        self.infiniburn = value;
        self
    }

    pub fn effects(mut self, value: ResourceLocation) -> Self {
        self.effects = value;
        self
    }

    pub fn ambient_light(mut self, value: f32) -> Self {
        self.ambient_light = value;
        self
    }

    pub fn piglin_safe(mut self, value: bool) -> Self {
        self.piglin_safe = value;
        self
    }

    pub fn has_raids(mut self, value: bool) -> Self {
        self.has_raids = value;
        self
    }

    pub fn monster_spawn_light_level(mut self, value: MonsterSpawnLightLevel) -> Self {
        self.monster_spawn_light_level = value;
        self
    }

    pub fn monster_spawn_block_light_limit(mut self, value: u8) -> Self {
        self.monster_spawn_block_light_limit = value;
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    pub fn raw_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.raw_fields.insert(key.into(), value);
        self
    }
}

impl DatapackComponent for DimensionType {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "dimension_type";
        if !self.coordinate_scale.is_finite()
            || !(0.000_01..=30_000_000.0).contains(&self.coordinate_scale)
        {
            return Err(validation::error(
                &self.location,
                kind,
                "coordinate_scale",
                &format!(
                    "coordinate_scale must be finite and in 0.00001..=30000000; received {}",
                    self.coordinate_scale
                ),
            ));
        }
        validation::require_finite_f32(&self.location, kind, "ambient_light", self.ambient_light)?;
        if !(0.0..=1.0).contains(&self.ambient_light) {
            return Err(validation::error(
                &self.location,
                kind,
                "ambient_light",
                &format!(
                    "ambient_light must be in 0..=1; received {}",
                    self.ambient_light
                ),
            ));
        }
        if !(-2032..=2031).contains(&self.min_y) || self.min_y % 16 != 0 {
            return Err(validation::error(
                &self.location,
                kind,
                "min_y",
                &format!(
                    "min_y must be in -2032..=2031 and a multiple of 16; received {}",
                    self.min_y
                ),
            ));
        }
        if !(16..=4064).contains(&self.height) || !self.height.is_multiple_of(16) {
            return Err(validation::error(
                &self.location,
                kind,
                "height",
                &format!(
                    "height must be in 16..=4064 and a multiple of 16; received {}",
                    self.height
                ),
            ));
        }
        if i64::from(self.min_y) + i64::from(self.height) > 2032 {
            return Err(validation::error(
                &self.location,
                kind,
                "height",
                &format!(
                    "min_y + height must not exceed 2032; received {} + {}",
                    self.min_y, self.height
                ),
            ));
        }
        if self.logical_height == 0 || self.logical_height > self.height {
            return Err(validation::error(
                &self.location,
                kind,
                "logical_height",
                &format!(
                    "logical_height must be in 1..=height ({}); received {}",
                    self.height, self.logical_height
                ),
            ));
        }
        self.monster_spawn_light_level.validate(&self.location)?;
        if self.monster_spawn_block_light_limit > 15 {
            return Err(validation::error(
                &self.location,
                kind,
                "monster_spawn_block_light_limit",
                &format!(
                    "monster_spawn_block_light_limit must be in 0..=15; received {}",
                    self.monster_spawn_block_light_limit
                ),
            ));
        }
        for key in self.raw_fields.keys() {
            validation::require_non_empty(&self.location, kind, "raw_field", key)?;
            validation::reject_whitespace_only(&self.location, kind, "raw_field", key)?;
            validation::reject_control_chars(&self.location, kind, "raw_field", key)?;
            if TYPED_FIELDS.contains(&key.as_str()) {
                return Err(validation::error(
                    &self.location,
                    kind,
                    "raw_field",
                    &format!("raw field `{key}` would override a typed field"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Some(fixed_time) = self.fixed_time {
            map.insert("fixed_time".into(), serde_json::json!(fixed_time));
        }
        map.insert("has_skylight".into(), self.has_skylight.into());
        map.insert("has_ceiling".into(), self.has_ceiling.into());
        map.insert("ultrawarm".into(), self.ultrawarm.into());
        map.insert("natural".into(), self.natural.into());
        map.insert(
            "coordinate_scale".into(),
            serde_json::json!(self.coordinate_scale),
        );
        map.insert("bed_works".into(), self.bed_works.into());
        map.insert(
            "respawn_anchor_works".into(),
            self.respawn_anchor_works.into(),
        );
        map.insert("min_y".into(), self.min_y.into());
        map.insert("height".into(), self.height.into());
        map.insert("logical_height".into(), self.logical_height.into());
        map.insert(
            "infiniburn".into(),
            Value::String(self.infiniburn.to_tag_string()),
        );
        map.insert("effects".into(), Value::String(self.effects.to_string()));
        map.insert(
            "ambient_light".into(),
            serde_json::json!(self.ambient_light),
        );
        map.insert("piglin_safe".into(), self.piglin_safe.into());
        map.insert("has_raids".into(), self.has_raids.into());
        map.insert(
            "monster_spawn_light_level".into(),
            self.monster_spawn_light_level.to_json(),
        );
        map.insert(
            "monster_spawn_block_light_limit".into(),
            self.monster_spawn_block_light_limit.into(),
        );
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "dimension_type"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "skylands").unwrap()
    }

    #[test]
    fn minimal_overworld_like_shape_is_valid() {
        let ty = DimensionType::overworld_like(location());
        ty.validate().unwrap();
        let json = ty.to_json();
        assert_eq!(json["min_y"], -64);
        assert_eq!(json["height"], 384);
        assert_eq!(json["infiniburn"], "#minecraft:infiniburn_overworld");
        assert_eq!(json["effects"], "minecraft:overworld");
        assert!(json.get("fixed_time").is_none());
        assert_eq!(ty.component_dir(), "dimension_type");
    }

    #[test]
    fn full_nether_like_shape_and_raw_extension_serialize() {
        let ty = DimensionType::overworld_like(location())
            .fixed_time(18_000)
            .has_skylight(false)
            .has_ceiling(true)
            .ultrawarm(true)
            .natural(false)
            .coordinate_scale(8.0)
            .bed_works(false)
            .respawn_anchor_works(true)
            .min_y(0)
            .height(256)
            .logical_height(128)
            .infiniburn(TagId::minecraft("infiniburn_nether").unwrap())
            .effects(ResourceLocation::minecraft("the_nether").unwrap())
            .ambient_light(0.1)
            .piglin_safe(true)
            .has_raids(false)
            .monster_spawn_light_level(MonsterSpawnLightLevel::Constant(7))
            .monster_spawn_block_light_limit(15)
            .raw_field("example:weather", RawJson::new(serde_json::json!("ash")));
        ty.validate().unwrap();
        let json = ty.to_json();
        assert_eq!(json["fixed_time"], 18_000);
        assert_eq!(json["monster_spawn_light_level"], 7);
        assert_eq!(json["example:weather"], "ash");
    }

    #[test]
    fn non_finite_numeric_values_are_rejected() {
        assert!(
            DimensionType::overworld_like(location())
                .coordinate_scale(f64::NAN)
                .validate()
                .is_err()
        );
        assert!(
            DimensionType::overworld_like(location())
                .ambient_light(f32::INFINITY)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn malformed_resource_and_tag_ids_are_rejected_at_construction() {
        assert!("minecraft:bad path".parse::<ResourceLocation>().is_err());
        assert!("minecraft:bad tag".parse::<TagId<BlockId>>().is_err());
    }

    #[test]
    fn invalid_height_relationships_are_rejected() {
        for ty in [
            DimensionType::overworld_like(location()).min_y(-63),
            DimensionType::overworld_like(location()).height(15),
            DimensionType::overworld_like(location())
                .min_y(0)
                .height(2048 + 16),
            DimensionType::overworld_like(location()).logical_height(385),
        ] {
            assert!(ty.validate().is_err());
        }
    }

    #[test]
    fn invalid_light_levels_and_typed_raw_overrides_are_rejected() {
        assert!(
            DimensionType::overworld_like(location())
                .monster_spawn_light_level(MonsterSpawnLightLevel::Uniform {
                    min_inclusive: 8,
                    max_inclusive: 7,
                })
                .validate()
                .is_err()
        );
        assert!(
            DimensionType::overworld_like(location())
                .raw_field("height", RawJson::new(serde_json::json!(16)))
                .validate()
                .is_err()
        );
    }
}
