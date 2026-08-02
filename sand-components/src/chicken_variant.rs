//! Builder for `data/<namespace>/chicken_variant/` JSON files (Minecraft
//! 1.21.5+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `asset_id` must be non-empty and a valid plain resource location
//!   (e.g. `"minecraft:entity/chicken/cold_chicken"`).
//! - each [`crate::animal_variant::SpawnCondition`] in `spawn_conditions`
//!   must reference one of the supported biome-selector JSON shapes (a
//!   single non-empty biome ID/tag string, or a non-empty array of such
//!   strings). Empty arrays, empty strings, and unsupported JSON shapes are
//!   rejected.
//!
//! Only the `minecraft:biome` spawn-condition type is modeled on the normal
//! path — see [`crate::animal_variant`]. Use [`ChickenVariant::raw_field`]
//! for other vanilla or modded fields (e.g. a future condition type).

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::animal_variant::SpawnCondition;
use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::resource_location::ResourceLocation;
use crate::validation;

const TYPED_FIELDS: &[&str] = &["asset_id", "spawn_conditions"];

/// A chicken variant definition (`data/<namespace>/chicken_variant/<id>.json`).
///
/// Chicken variants select the texture used when a chicken spawns, based on
/// an ordered, prioritized list of biome spawn conditions.
pub struct ChickenVariant {
    location: ResourceLocation,
    asset_id: String,
    spawn_conditions: Vec<SpawnCondition>,
    raw_fields: BTreeMap<String, RawJson>,
}

impl ChickenVariant {
    /// Create a new chicken variant with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            spawn_conditions: Vec::new(),
            raw_fields: BTreeMap::new(),
        }
    }

    /// Set the texture asset ID (e.g. `"minecraft:entity/chicken/cold_chicken"`).
    pub fn asset_id(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    /// Add one prioritized biome spawn condition.
    pub fn spawn_condition(mut self, condition: SpawnCondition) -> Self {
        self.spawn_conditions.push(condition);
        self
    }

    /// Replace the full ordered list of biome spawn conditions.
    pub fn spawn_conditions(
        mut self,
        conditions: impl IntoIterator<Item = SpawnCondition>,
    ) -> Self {
        self.spawn_conditions = conditions.into_iter().collect();
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

impl DatapackComponent for ChickenVariant {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "chicken_variant";
        validation::require_non_empty(&self.location, kind, "asset_id", &self.asset_id)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "asset_id",
            &self.asset_id,
        )?;
        for condition in &self.spawn_conditions {
            condition.validate(&self.location, kind, "spawn_conditions")?;
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

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("asset_id".into(), Value::String(self.asset_id.clone()));
        if !self.spawn_conditions.is_empty() {
            map.insert(
                "spawn_conditions".into(),
                Value::Array(
                    self.spawn_conditions
                        .iter()
                        .map(SpawnCondition::to_json)
                        .collect(),
                ),
            );
        }
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "chicken_variant"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::AnimalVariants]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "cold").unwrap()
    }

    fn valid() -> ChickenVariant {
        ChickenVariant::new(rl())
            .asset_id("minecraft:entity/chicken/cold_chicken")
            .spawn_condition(SpawnCondition::biome("minecraft:snowy_taiga", 1))
    }

    #[test]
    fn valid_chicken_variant_passes_validation() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let cv = ChickenVariant::new(rl());
        let err = cv.validate().unwrap_err();
        assert!(err.to_string().contains("asset_id"), "{err}");
    }

    #[test]
    fn malformed_asset_id_is_rejected() {
        let cv = valid().asset_id("Not Valid!");
        assert!(cv.validate().is_err());
    }

    #[test]
    fn malformed_spawn_condition_is_rejected() {
        let cv = ChickenVariant::new(rl())
            .asset_id("minecraft:entity/chicken/cold_chicken")
            .spawn_condition(SpawnCondition::biome("", 1));
        let err = cv.validate().unwrap_err();
        assert!(err.to_string().contains("spawn_conditions"), "{err}");
    }

    #[test]
    fn raw_field_cannot_override_typed_field() {
        let cv = valid().raw_field("asset_id", RawJson::new(serde_json::json!("bad")));
        assert!(cv.validate().is_err());
    }

    #[test]
    fn raw_field_extends_json() {
        let cv = valid().raw_field("custom_extra", RawJson::new(serde_json::json!(true)));
        assert!(cv.validate().is_ok());
        assert_eq!(cv.to_json()["custom_extra"], true);
    }

    #[test]
    fn no_spawn_conditions_is_valid() {
        let cv = ChickenVariant::new(rl()).asset_id("minecraft:entity/chicken/temperate_chicken");
        assert!(cv.validate().is_ok());
        assert!(cv.to_json().get("spawn_conditions").is_none());
    }

    #[test]
    fn invalid_chicken_variant_fails_export() {
        let cv = ChickenVariant::new(rl());
        assert!(cv.try_content().is_err());
    }

    #[test]
    fn component_dir_and_feature_gate_are_correct() {
        let cv = valid();
        assert_eq!(cv.component_dir(), "chicken_variant");
        assert_eq!(
            cv.required_features(),
            &[sand_version::ComponentFeature::AnimalVariants]
        );
    }

    #[test]
    fn valid_chicken_variant_json_is_stable() {
        let cv = valid();
        let json = cv.to_json();
        assert_eq!(json["asset_id"], "minecraft:entity/chicken/cold_chicken");
        assert_eq!(
            json["spawn_conditions"],
            serde_json::json!([{
                "priority": 1,
                "condition": {
                    "type": "minecraft:biome",
                    "biomes": "minecraft:snowy_taiga",
                }
            }])
        );
        let a = serde_json::to_string_pretty(&cv.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&cv.to_json()).unwrap();
        assert_eq!(a, b);
    }
}
