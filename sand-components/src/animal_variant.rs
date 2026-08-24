//! Shared spawn-condition model for the biome-scoped animal variant
//! registries introduced alongside `chicken_variant`, `cow_variant`, and
//! `pig_variant` (Minecraft 1.21.5+).
//!
//! Each of these registries selects a variant for a freshly spawned animal
//! using an ordered, prioritized list of spawn conditions. Sand only models
//! the `minecraft:biome` condition type today (the common, documented case);
//! other condition types are out of scope for this narrow first pass — see
//! the per-registry module docs for their raw-field escape hatch.

use serde_json::Value;

use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// One entry of a variant registry's `spawn_conditions` prioritized list.
///
/// Serializes as:
/// ```json
/// {
///   "priority": 1,
///   "condition": {
///     "type": "minecraft:biome",
///     "biomes": "minecraft:snowy_taiga"
///   }
/// }
/// ```
#[doc = "**API Contract:** Run `sand api show sand::component::SpawnCondition` for the canonical contract."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnCondition {
    biomes: Value,
    priority: i32,
}

impl SpawnCondition {
    /// A biome-scoped spawn condition for a single biome ID or tag reference
    /// (e.g. `"minecraft:snowy_taiga"` or `"#minecraft:is_snowy"`).
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnCondition::biome` for the canonical contract."]
    pub fn biome(biome_id: impl Into<String>, priority: i32) -> Self {
        Self {
            biomes: Value::String(biome_id.into()),
            priority,
        }
    }

    /// A biome-scoped spawn condition matching any of several biome IDs /
    /// tag references.
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnCondition::biomes` for the canonical contract."]
    pub fn biomes(biome_ids: impl IntoIterator<Item = impl Into<String>>, priority: i32) -> Self {
        Self {
            biomes: Value::Array(
                biome_ids
                    .into_iter()
                    .map(|s| Value::String(s.into()))
                    .collect(),
            ),
            priority,
        }
    }

    /// Build a biome condition from a raw JSON `biomes` shape.
    ///
    /// Explicit escape hatch: the value still passes through the same
    /// biome-selector validation as [`SpawnCondition::biome`] /
    /// [`SpawnCondition::biomes`] at export time.
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnCondition::biomes_raw` for the canonical contract."]
    pub fn biomes_raw(biomes: RawJson, priority: i32) -> Self {
        Self {
            biomes: biomes.into_value(),
            priority,
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        validation::validate_biome_selector(location, kind, field, &self.biomes)
    }

    pub(crate) fn to_json(&self) -> Value {
        serde_json::json!({
            "priority": self.priority,
            "condition": {
                "type": "minecraft:biome",
                "biomes": self.biomes,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "cond").unwrap()
    }

    #[test]
    fn single_biome_condition_json_is_stable() {
        let cond = SpawnCondition::biome("minecraft:snowy_taiga", 1);
        assert_eq!(
            cond.to_json(),
            serde_json::json!({
                "priority": 1,
                "condition": {
                    "type": "minecraft:biome",
                    "biomes": "minecraft:snowy_taiga",
                }
            })
        );
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_ok());
    }

    #[test]
    fn multi_biome_condition_json_is_stable() {
        let cond = SpawnCondition::biomes(["minecraft:plains", "minecraft:desert"], 2);
        assert_eq!(
            cond.to_json()["condition"]["biomes"],
            serde_json::json!(["minecraft:plains", "minecraft:desert"])
        );
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_ok());
    }

    #[test]
    fn empty_biome_string_is_rejected() {
        let cond = SpawnCondition::biome("", 1);
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_err());
    }

    #[test]
    fn malformed_biome_string_is_rejected() {
        let cond = SpawnCondition::biome("Not Valid!", 1);
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_err());
    }

    #[test]
    fn empty_biomes_array_is_rejected() {
        let cond = SpawnCondition::biomes(Vec::<String>::new(), 1);
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_err());
    }

    #[test]
    fn tag_biome_is_accepted() {
        let cond = SpawnCondition::biome("#minecraft:is_forest", 1);
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_ok());
    }

    #[test]
    fn raw_non_string_array_entry_is_rejected() {
        let cond =
            SpawnCondition::biomes_raw(RawJson::new(serde_json::json!(["minecraft:plains", 5])), 1);
        assert!(cond.validate(&rl(), "test", "spawn_conditions").is_err());
    }
}
