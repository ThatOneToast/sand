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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SpawnCondition",
    module = "sand::component",
    summary = "One entry of a variant registry's `spawn_conditions` prioritized list.",
    context = "One entry of a variant registry's `spawn_conditions` prioritized list. Serializes as:",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SpawnCondition;",
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnCondition {
    biomes: Value,
    priority: i32,
}

impl SpawnCondition {
    /// A biome-scoped spawn condition for a single biome ID or tag reference
    /// (e.g. `"minecraft:snowy_taiga"` or `"#minecraft:is_snowy"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnCondition::biome",
        module = "sand::component",
        kind = "method",
        summary = "A biome-scoped spawn condition for a single biome ID or tag reference (e.g. `\"minecraft:snowy_taiga\"` or `\"#minecraft:is_snowy\"`).",
        context = "A biome-scoped spawn condition for a single biome ID or tag reference (e.g. `\"minecraft:snowy_taiga\"` or `\"#minecraft:is_snowy\"`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(biome_id = "`biome_id` sets the biome id for a biome-scoped spawn condition for a single biome ID or tag reference (e.g. `\"minecraft:snowy_taiga\"` or `\"#minecraft:is_snowy\"`).", priority = "`priority` sets the priority for a biome-scoped spawn condition for a single biome ID or tag reference (e.g. `\"minecraft:snowy_taiga\"` or `\"#minecraft:is_snowy\"`)."),
        returns = "A `SpawnCondition` configured for a biome-scoped spawn condition for a single biome ID or tag reference (e.g. `\"minecraft:snowy_taiga\"` or `\"#minecraft:is_snowy\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_id: impl Into < String >, priority: i32)  {\n    let spawn_condition = sand::component::SpawnCondition::biome(biome_id, priority);\n}",
    )]
    pub fn biome(biome_id: impl Into<String>, priority: i32) -> Self {
        Self {
            biomes: Value::String(biome_id.into()),
            priority,
        }
    }

    /// A biome-scoped spawn condition matching any of several biome IDs /
    /// tag references.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnCondition::biomes",
        module = "sand::component",
        kind = "method",
        summary = "A biome-scoped spawn condition matching any of several biome IDs / tag references.",
        context = "A biome-scoped spawn condition matching any of several biome IDs / tag references. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(biome_ids = "`biome_ids` sets the biome ids for a biome-scoped spawn condition matching any of several biome IDs / tag references.", priority = "`priority` sets the priority for a biome-scoped spawn condition matching any of several biome IDs / tag references."),
        returns = "A `SpawnCondition` configured for a biome-scoped spawn condition matching any of several biome IDs / tag references.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biome_ids: impl IntoIterator < Item = impl Into < String > >, priority: i32)  {\n    let spawn_condition = sand::component::SpawnCondition::biomes(biome_ids, priority);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnCondition::biomes_raw",
        module = "sand::component",
        kind = "method",
        summary = "Build a biome condition from a raw JSON `biomes` shape.",
        context = "Build a biome condition from a raw JSON `biomes` shape. Explicit escape hatch: the value still passes through the same biome-selector validation as [`SpawnCondition::biome`] / [`SpawnCondition::biomes`] at export time.",
        minecraft = "Explicit escape hatch: the value still passes through the same biome-selector validation as [`SpawnCondition::biome`] / [`SpawnCondition::biomes`] at export time.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(biomes = "Build a biome condition from a raw JSON `biomes` shape.", priority = "`priority` provides the priority used to build a biome condition from a raw JSON `biomes` shape."),
        returns = "A `SpawnCondition` that builds a biome condition from a raw JSON `biomes` shape.",
        example = "use sand::prelude::*;\n\nfn demonstrate(biomes: sand::component::RawJson, priority: i32)  {\n    let spawn_condition = sand::component::SpawnCondition::biomes_raw(biomes, priority);\n}",
    )]
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
