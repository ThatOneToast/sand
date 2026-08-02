//! Shared worldgen value types used by more than one worldgen builder.
//!
//! These types model small vanilla worldgen shapes that appear in several
//! registries (structures, processor lists, features). They are deliberately
//! kept here — rather than duplicated per module — so that a block state or a
//! height provider serializes identically everywhere Sand emits one.
//!
//! Every type keeps an explicitly named raw escape hatch where vanilla has
//! more variants than Sand models today.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::BlockId;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// Inclusive world-height bounds accepted by vanilla vertical anchors.
const MIN_ANCHOR: i32 = -2032;
const MAX_ANCHOR: i32 = 2031;

/// A vanilla vertical anchor (`{"absolute": 0}`, `{"above_bottom": 8}`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerticalAnchor {
    /// An absolute Y coordinate.
    Absolute(i32),
    /// An offset above the dimension's minimum build height.
    AboveBottom(i32),
    /// An offset below the dimension's maximum build height.
    BelowTop(i32),
}

impl VerticalAnchor {
    /// Serialize to the vanilla single-key anchor object.
    pub fn to_json(&self) -> Value {
        let (key, value) = match self {
            Self::Absolute(value) => ("absolute", *value),
            Self::AboveBottom(value) => ("above_bottom", *value),
            Self::BelowTop(value) => ("below_top", *value),
        };
        let mut map = Map::new();
        map.insert(key.into(), value.into());
        Value::Object(map)
    }

    fn value(&self) -> i32 {
        match self {
            Self::Absolute(value) | Self::AboveBottom(value) | Self::BelowTop(value) => *value,
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        let value = self.value();
        if !(MIN_ANCHOR..=MAX_ANCHOR).contains(&value) {
            return Err(validation::error(
                location,
                kind,
                field,
                &format!(
                    "vertical anchor must be in {MIN_ANCHOR}..={MAX_ANCHOR}; received {value}"
                ),
            ));
        }
        Ok(())
    }
}

/// A vanilla height provider.
///
/// [`HeightProvider::Raw`] is the explicit escape hatch for provider types
/// Sand does not model yet (for example `weighted_list`).
#[derive(Debug, Clone, PartialEq)]
pub enum HeightProvider {
    /// A constant anchor, emitted using the vanilla inline shorthand.
    Constant(VerticalAnchor),
    /// A uniformly sampled inclusive anchor range.
    Uniform {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
    },
    /// A trapezoidal distribution between two anchors.
    Trapezoid {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        plateau: i32,
    },
    /// An explicitly raw height provider object.
    Raw(RawJson),
}

impl HeightProvider {
    /// A constant absolute-Y height provider.
    pub fn absolute(y: i32) -> Self {
        Self::Constant(VerticalAnchor::Absolute(y))
    }

    /// Serialize to the vanilla height provider JSON.
    pub fn to_json(&self) -> Value {
        match self {
            Self::Constant(anchor) => anchor.to_json(),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => serde_json::json!({
                "type": "minecraft:uniform",
                "min_inclusive": min_inclusive.to_json(),
                "max_inclusive": max_inclusive.to_json(),
            }),
            Self::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => serde_json::json!({
                "type": "minecraft:trapezoid",
                "min_inclusive": min_inclusive.to_json(),
                "max_inclusive": max_inclusive.to_json(),
                "plateau": plateau,
            }),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        match self {
            Self::Constant(anchor) => anchor.validate(location, kind, field)?,
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => {
                min_inclusive.validate(location, kind, field)?;
                max_inclusive.validate(location, kind, field)?;
                require_ordered_anchors(location, kind, field, min_inclusive, max_inclusive)?;
            }
            Self::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => {
                min_inclusive.validate(location, kind, field)?;
                max_inclusive.validate(location, kind, field)?;
                require_ordered_anchors(location, kind, field, min_inclusive, max_inclusive)?;
                if *plateau < 0 {
                    return Err(validation::error(
                        location,
                        kind,
                        field,
                        &format!("plateau must be non-negative; received {plateau}"),
                    ));
                }
            }
            Self::Raw(raw) => {
                validation::require_json_object(location, kind, field, raw.as_value())?;
            }
        }
        Ok(())
    }
}

/// Compares two anchors only when they use the same anchor kind; mixed kinds
/// depend on the dimension's height and cannot be ordered at export time.
fn require_ordered_anchors(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    min: &VerticalAnchor,
    max: &VerticalAnchor,
) -> SandResult<()> {
    let comparable = matches!(
        (min, max),
        (VerticalAnchor::Absolute(_), VerticalAnchor::Absolute(_))
            | (
                VerticalAnchor::AboveBottom(_),
                VerticalAnchor::AboveBottom(_)
            )
            | (VerticalAnchor::BelowTop(_), VerticalAnchor::BelowTop(_))
    );
    if comparable && min.value() > max.value() {
        return Err(validation::error(
            location,
            kind,
            field,
            &format!(
                "min_inclusive must not exceed max_inclusive; received {}..={}",
                min.value(),
                max.value()
            ),
        ));
    }
    Ok(())
}

/// A vanilla chunk heightmap selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heightmap {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

impl Heightmap {
    /// The vanilla uppercase enum name written into datapack JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }
}

/// A worldgen block state object (`{"Name": …, "Properties": {…}}`).
///
/// This is the datapack JSON form used by structure processors and feature
/// configs. It is deliberately distinct from the command-side
/// `sand_commands::BlockState`, which renders `minecraft:stone[facing=north]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenBlockState {
    name: BlockId,
    properties: BTreeMap<String, String>,
}

impl WorldgenBlockState {
    /// A block state with no explicit properties.
    pub fn new(name: BlockId) -> Self {
        Self {
            name,
            properties: BTreeMap::new(),
        }
    }

    /// Set a block-state property (deterministically ordered on export).
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// The block identifier this state refers to.
    pub fn name(&self) -> &BlockId {
        &self.name
    }

    /// Serialize to the vanilla block state object.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("Name".into(), Value::String(self.name.to_string()));
        if !self.properties.is_empty() {
            let properties: Map<String, Value> = self
                .properties
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect();
            map.insert("Properties".into(), Value::Object(properties));
        }
        Value::Object(map)
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            kind,
            &format!("{field}.Name"),
            &self.name.to_string(),
        )?;
        for (key, value) in &self.properties {
            let property_field = format!("{field}.Properties.{key}");
            validation::require_non_empty(location, kind, &property_field, key)?;
            validation::reject_whitespace_only(location, kind, &property_field, key)?;
            validation::reject_control_chars(location, kind, &property_field, key)?;
            validation::require_non_empty(location, kind, &property_field, value)?;
            validation::reject_whitespace_only(location, kind, &property_field, value)?;
            validation::reject_control_chars(location, kind, &property_field, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "providers").unwrap()
    }

    #[test]
    fn constant_height_provider_uses_inline_anchor_shorthand() {
        let provider = HeightProvider::absolute(0);
        provider
            .validate(&location(), "kind", "start_height")
            .unwrap();
        assert_eq!(provider.to_json(), serde_json::json!({ "absolute": 0 }));
    }

    #[test]
    fn uniform_height_provider_serializes_typed_anchors() {
        let provider = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::AboveBottom(0),
            max_inclusive: VerticalAnchor::BelowTop(8),
        };
        provider
            .validate(&location(), "kind", "start_height")
            .unwrap();
        let json = provider.to_json();
        assert_eq!(json["type"], "minecraft:uniform");
        assert_eq!(json["min_inclusive"]["above_bottom"], 0);
        assert_eq!(json["max_inclusive"]["below_top"], 8);
    }

    #[test]
    fn inverted_and_out_of_range_anchors_are_rejected() {
        assert!(
            HeightProvider::Uniform {
                min_inclusive: VerticalAnchor::Absolute(64),
                max_inclusive: VerticalAnchor::Absolute(0),
            }
            .validate(&location(), "kind", "start_height")
            .is_err()
        );
        assert!(
            HeightProvider::absolute(9000)
                .validate(&location(), "kind", "start_height")
                .is_err()
        );
    }

    #[test]
    fn raw_height_provider_must_be_a_json_object() {
        assert!(
            HeightProvider::Raw(RawJson::new(serde_json::json!([1, 2])))
                .validate(&location(), "kind", "start_height")
                .is_err()
        );
        assert!(
            HeightProvider::Raw(RawJson::new(serde_json::json!({ "type": "mymod:custom" })))
                .validate(&location(), "kind", "start_height")
                .is_ok()
        );
    }

    #[test]
    fn block_state_serializes_name_and_sorted_properties() {
        let state = WorldgenBlockState::new(BlockId::minecraft("oak_log").unwrap())
            .property("axis", "y")
            .property("waterlogged", "false");
        state.validate(&location(), "kind", "block").unwrap();
        let json = state.to_json();
        assert_eq!(json["Name"], "minecraft:oak_log");
        assert_eq!(json["Properties"]["axis"], "y");
        assert_eq!(
            serde_json::to_string(&json).unwrap(),
            r#"{"Name":"minecraft:oak_log","Properties":{"axis":"y","waterlogged":"false"}}"#
        );
    }

    #[test]
    fn block_state_rejects_empty_property_values() {
        let state =
            WorldgenBlockState::new(BlockId::minecraft("stone").unwrap()).property("axis", "");
        assert!(state.validate(&location(), "kind", "block").is_err());
    }

    #[test]
    fn heightmap_names_use_vanilla_uppercase_form() {
        assert_eq!(Heightmap::WorldSurfaceWg.as_str(), "WORLD_SURFACE_WG");
        assert_eq!(
            Heightmap::MotionBlockingNoLeaves.as_str(),
            "MOTION_BLOCKING_NO_LEAVES"
        );
    }
}
