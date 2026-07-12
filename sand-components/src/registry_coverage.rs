//! Registry coverage audit for Sand data-driven component modules.
//!
//! This module provides a static compile-time table of every known vanilla Java
//! Edition data-driven registry, along with Sand's implementation status.
//!
//! # Purpose
//!
//! - Single source of truth for datapack component parity.
//! - Makes gaps explicit: missing registries are listed as `Missing` rather
//!   than silently absent from the codebase.
//! - Checked-in fixtures compare this table against Mojang's generated
//!   `datapack.json` report to detect newly-added or renamed registries.
//! - Tag-only coverage lives in [`TAG_COVERAGE`] and never masquerades as a
//!   vanilla registry identifier.
//!
//! # Usage
//!
//! ```
//! use sand_components::registry_coverage::{REGISTRY_COVERAGE, RegistryApiStatus};
//!
//! let missing: Vec<_> = REGISTRY_COVERAGE
//!     .iter()
//!     .filter(|r| matches!(r.api_status, RegistryApiStatus::Missing))
//!     .collect();
//! println!("{} missing registries", missing.len());
//! ```

// ── Status enums ──────────────────────────────────────────────────────────────

/// How well Sand's typed API covers the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryApiStatus {
    /// A typed Sand module exists and generates correct JSON paths.
    /// At least one serialization/golden test exists.
    FullyImplemented,
    /// A module exists but coverage is incomplete — some fields are raw
    /// `serde_json::Value`, required builders are missing, or tests are absent.
    PartiallyImplemented,
    /// No typed module. Use `RawComponent` as the named escape hatch.
    Missing,
    /// Reachable only via `RawComponent` or inline raw JSON.
    RawOnly,
    /// Only present in a newer Minecraft version not yet verified by Sand.
    VersionGated,
    /// Intentionally not modelled (too obscure, server-only, or out of scope).
    IntentionallyUnsupported,
}

/// One row of the registry coverage table.
#[derive(Debug)]
pub struct RegistryCoverage {
    /// The vanilla registry key (e.g. `"minecraft:recipe"`).
    pub registry_key: &'static str,
    /// The datapack folder path relative to `data/<namespace>/`.
    pub datapack_dir: &'static str,
    /// The tag path, if this registry is taggable (relative to `data/<namespace>/`).
    pub tag_dir: Option<&'static str>,
    /// The sand-components module that covers this registry, if any.
    pub sand_module: Option<&'static str>,
    /// Implementation status.
    pub api_status: RegistryApiStatus,
    /// Minecraft version gate for this registry (`None` = present in all Sand-supported versions).
    pub version_gate: Option<&'static str>,
    /// Notes about gaps, escape hatches, or follow-up issues.
    pub notes: &'static str,
}

/// Coverage for a datapack tag directory and the registry its values belong to.
///
/// Tags are deliberately separate from [`RegistryCoverage`]: their value
/// registry is a valid resource location, but a tag directory is not itself a
/// vanilla data-driven registry.
#[derive(Debug)]
pub struct TagCoverage {
    pub value_registry: &'static str,
    pub datapack_dir: &'static str,
    pub sand_module: Option<&'static str>,
    pub api_status: RegistryApiStatus,
    pub notes: &'static str,
}

// ── Coverage table ────────────────────────────────────────────────────────────

/// Static coverage table for all known vanilla Java Edition data-driven registries.
pub const REGISTRY_COVERAGE: &[RegistryCoverage] = &[
    // ── Core datapack types ───────────────────────────────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:advancement",
        datapack_dir: "advancement",
        tag_dir: None,
        sand_module: Some("sand_components::advancement"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "Advancement, AdvancementTrigger, AdvancementDisplay. 50+ trigger variants. See trigger_coverage.",
    },
    RegistryCoverage {
        registry_key: "minecraft:function",
        datapack_dir: "function",
        tag_dir: Some("tags/function"),
        sand_module: Some("sand_core::function"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "#[function] macro generates .mcfunction files. load/tick/custom tags supported.",
    },
    RegistryCoverage {
        registry_key: "minecraft:loot_table",
        datapack_dir: "loot_table",
        tag_dir: None,
        sand_module: Some("sand_components::loot_table"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "LootTable, LootPool, LootEntry exist but coverage is partial. Complex pool conditions and entry types are missing. Follow-up: #17.",
    },
    RegistryCoverage {
        registry_key: "minecraft:predicate",
        datapack_dir: "predicate",
        tag_dir: None,
        sand_module: Some("sand_components::predicate"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Predicate type exists. EntityPredicate, ItemPredicate, DamagePredicate typed. Location, weather, time predicates are partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:recipe",
        datapack_dir: "recipe",
        tag_dir: None,
        sand_module: Some("sand_components::recipe"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "All standard recipe types implemented: shaped, shapeless, smelting, blasting, smoking, campfire, smithing_transform, smithing_trim, stonecutting.",
    },
    RegistryCoverage {
        registry_key: "minecraft:item_modifier",
        datapack_dir: "item_modifier",
        tag_dir: None,
        sand_module: Some("sand_components::item_modifier"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "ItemModifier exists. SetCount, SetComponents, EnchantRandomly present. Full modifier set incomplete.",
    },
    // ── Entity / world types ──────────────────────────────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:damage_type",
        datapack_dir: "damage_type",
        tag_dir: Some("tags/damage_type"),
        sand_module: Some("sand_components::damage_type"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19.4"),
        notes: "DamageType, DamageScaling, DamageEffects, DeathMessageType. Introduced in 1.19.4.",
    },
    RegistryCoverage {
        registry_key: "minecraft:enchantment",
        datapack_dir: "enchantment",
        tag_dir: Some("tags/enchantment"),
        sand_module: Some("sand_components::enchantment"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21"),
        notes: "Enchantment struct exists. Data-driven enchantments fully introduced in 1.21. Full effect type coverage is partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:enchantment_provider",
        datapack_dir: "enchantment_provider",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: Some("1.21"),
        notes: "Not implemented. Use RawComponent. Added with data-driven enchantments in 1.21.",
    },
    RegistryCoverage {
        registry_key: "minecraft:jukebox_song",
        datapack_dir: "jukebox_song",
        tag_dir: None,
        sand_module: Some("sand_components::jukebox_song"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.21"),
        notes: "JukeboxSong. Introduced in 1.21.",
    },
    RegistryCoverage {
        registry_key: "minecraft:instrument",
        datapack_dir: "instrument",
        tag_dir: None,
        sand_module: Some("sand_components::instrument"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "Instrument (goat horn, etc.).",
    },
    RegistryCoverage {
        registry_key: "minecraft:painting_variant",
        datapack_dir: "painting_variant",
        tag_dir: None,
        sand_module: Some("sand_components::painting_variant"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "PaintingVariant.",
    },
    RegistryCoverage {
        registry_key: "minecraft:banner_pattern",
        datapack_dir: "banner_pattern",
        tag_dir: None,
        sand_module: Some("sand_components::banner_pattern"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "BannerPattern.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trim_material",
        datapack_dir: "trim_material",
        tag_dir: None,
        sand_module: Some("sand_components::trim"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19.4"),
        notes: "TrimMaterial in sand_components::trim module.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trim_pattern",
        datapack_dir: "trim_pattern",
        tag_dir: None,
        sand_module: Some("sand_components::trim"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19.4"),
        notes: "TrimPattern in sand_components::trim module.",
    },
    RegistryCoverage {
        registry_key: "minecraft:wolf_variant",
        datapack_dir: "wolf_variant",
        tag_dir: None,
        sand_module: Some("sand_components::wolf_variant"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.20.5"),
        notes: "WolfVariant. Introduced in 1.20.5.",
    },
    RegistryCoverage {
        registry_key: "minecraft:chat_type",
        datapack_dir: "chat_type",
        tag_dir: None,
        sand_module: Some("sand_components::chat_type"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19"),
        notes: "ChatType, ChatDecoration. Introduced in 1.19.",
    },
    // ── Vanilla value registries with datapack elements ─────────────────────
    RegistryCoverage {
        registry_key: "minecraft:cat_variant",
        datapack_dir: "cat_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:frog_variant",
        datapack_dir: "frog_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trial_spawner",
        datapack_dir: "trial_spawner",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: None,
        notes: "No typed component builder. Use RawComponent.",
    },
    // ── Latest verified (26.2) data-driven registries ────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:cat_sound_variant",
        datapack_dir: "cat_sound_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:chicken_sound_variant",
        datapack_dir: "chicken_sound_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:chicken_variant",
        datapack_dir: "chicken_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:cow_sound_variant",
        datapack_dir: "cow_sound_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:cow_variant",
        datapack_dir: "cow_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:pig_sound_variant",
        datapack_dir: "pig_sound_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:pig_variant",
        datapack_dir: "pig_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:sulfur_cube_archetype",
        datapack_dir: "sulfur_cube_archetype",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::IntentionallyUnsupported,
        version_gate: Some("26.2"),
        notes: "Explicit raw-only compatibility row; no typed API planned in #176.",
    },
    RegistryCoverage {
        registry_key: "minecraft:test_environment",
        datapack_dir: "test_environment",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::IntentionallyUnsupported,
        version_gate: Some("1.21.5"),
        notes: "Vanilla test framework data. Use RawComponent if required.",
    },
    RegistryCoverage {
        registry_key: "minecraft:test_instance",
        datapack_dir: "test_instance",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::IntentionallyUnsupported,
        version_gate: Some("1.21.5"),
        notes: "Vanilla test framework data. Use RawComponent if required.",
    },
    RegistryCoverage {
        registry_key: "minecraft:timeline",
        datapack_dir: "timeline",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trade_set",
        datapack_dir: "trade_set",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:villager_trade",
        datapack_dir: "villager_trade",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:wolf_sound_variant",
        datapack_dir: "wolf_sound_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    RegistryCoverage {
        registry_key: "minecraft:world_clock",
        datapack_dir: "world_clock",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("26.1"),
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:zombie_nautilus_variant",
        datapack_dir: "zombie_nautilus_variant",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: Some("1.21.5"),
        notes: "No typed component builder. Use RawComponent. Follow-up: #201.",
    },
    // ── Worldgen ─────────────────────────────────────────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:worldgen/biome",
        datapack_dir: "worldgen/biome",
        tag_dir: Some("tags/worldgen/biome"),
        sand_module: Some("sand_components::worldgen::biome"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Biome struct exists with basic fields. Spawn costs, effects, mob spawning rules are partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/placed_feature",
        datapack_dir: "worldgen/placed_feature",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::placed_feature"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "PlacedFeature struct exists. Full placement modifier coverage is partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/configured_feature",
        datapack_dir: "worldgen/configured_feature",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/structure",
        datapack_dir: "worldgen/structure",
        tag_dir: Some("tags/worldgen/structure"),
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/structure_set",
        datapack_dir: "worldgen/structure_set",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/processor_list",
        datapack_dir: "worldgen/processor_list",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/template_pool",
        datapack_dir: "worldgen/template_pool",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/noise_settings",
        datapack_dir: "worldgen/noise_settings",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::noise_settings"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "NoiseSettings struct exists. Full surface rule coverage is partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/flat_level_generator_preset",
        datapack_dir: "worldgen/flat_level_generator_preset",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: None,
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        datapack_dir: "worldgen/multi_noise_biome_source_parameter_list",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: None,
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/world_preset",
        datapack_dir: "worldgen/world_preset",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::RawOnly,
        version_gate: None,
        notes: "No typed component builder. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/density_function",
        datapack_dir: "worldgen/density_function",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/noise",
        datapack_dir: "worldgen/noise",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/configured_carver",
        datapack_dir: "worldgen/configured_carver",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    RegistryCoverage {
        registry_key: "minecraft:dimension",
        datapack_dir: "dimension",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::dimension"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Dimension struct exists. MonsterSettings, DragonFight coverage is partial.",
    },
    RegistryCoverage {
        registry_key: "minecraft:dimension_type",
        datapack_dir: "dimension_type",
        tag_dir: Some("tags/dimension_type"),
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent.",
    },
    // ── 1.21.6+ dialog (version-gated) ───────────────────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:dialog",
        datapack_dir: "dialog",
        tag_dir: Some("tags/dialog"),
        sand_module: Some("sand_components::dialog"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21.6"),
        notes: "Dialog builder and well-known pause_screen_additions/quick_actions tag helpers exist. Broader validation remains partial.",
    },
];

/// Tag-only coverage, kept out of vanilla registry-ID drift comparisons.
pub const TAG_COVERAGE: &[TagCoverage] = &[
    TagCoverage {
        value_registry: "minecraft:block",
        datapack_dir: "tags/block",
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::FullyImplemented,
        notes: "TypedTag<BlockId> enforces registry-safe values and paths; raw Tag remains available.",
    },
    TagCoverage {
        value_registry: "minecraft:item",
        datapack_dir: "tags/item",
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::FullyImplemented,
        notes: "TypedTag<ItemId> enforces registry-safe values and paths; raw Tag remains available.",
    },
    TagCoverage {
        value_registry: "minecraft:entity_type",
        datapack_dir: "tags/entity_type",
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::FullyImplemented,
        notes: "TypedTag<EntityTypeId> enforces registry-safe values and paths; raw Tag remains available.",
    },
    TagCoverage {
        value_registry: "minecraft:fluid",
        datapack_dir: "tags/fluid",
        sand_module: None,
        api_status: RegistryApiStatus::IntentionallyUnsupported,
        notes: "Use RawComponent if required.",
    },
    TagCoverage {
        value_registry: "minecraft:function",
        datapack_dir: "tags/function",
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::FullyImplemented,
        notes: "TypedTag<FunctionId> supports required and optional function and tag references.",
    },
];

/// Exclusive upper version gates for registries removed or renamed by Mojang.
///
/// The table is currently empty because both checked profiles are additive.
/// Keeping removals separate preserves the existing `version_gate` (introduced
/// in) API while allowing a registry to remain valid for older fixtures.
pub const REGISTRY_REMOVED_IN: &[(&str, &str)] = &[];

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Debug, Deserialize)]
    struct Fixture {
        minecraft_version: String,
        provenance: String,
        registries: Vec<FixtureRegistry>,
    }

    #[derive(Debug, Deserialize)]
    struct FixtureRegistry {
        registry_id: String,
        datapack_dir: String,
    }

    fn parse_fixture(json: &str) -> Fixture {
        serde_json::from_str(json).expect("checked-in registry fixture must parse")
    }

    fn version_parts(version: &str) -> Option<Vec<u32>> {
        version.split('.').map(|part| part.parse().ok()).collect()
    }

    fn active_for(entry: &RegistryCoverage, version: &str, removals: &[(&str, &str)]) -> bool {
        let introduced = entry.version_gate.is_none_or(|gate| {
            version_parts(version).expect("fixture version must be numeric")
                >= version_parts(gate).expect("coverage version gate must be numeric")
        });
        let not_removed = removals
            .iter()
            .find(|(id, _)| *id == entry.registry_key)
            .is_none_or(|(_, removed_in)| {
                version_parts(version).expect("fixture version must be numeric")
                    < version_parts(removed_in).expect("removal gate must be numeric")
            });
        introduced && not_removed
    }

    fn valid_resource_location(id: &str) -> bool {
        let Some((namespace, path)) = id.split_once(':') else {
            return false;
        };
        !namespace.is_empty()
            && !path.is_empty()
            && namespace
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b))
            && path
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_./-".contains(&b))
    }

    fn valid_dir(path: &str) -> bool {
        !path.is_empty()
            && !path.starts_with('/')
            && !path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            && path
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_./-".contains(&b))
    }

    fn drift_with_removals(
        fixture: &Fixture,
        coverage: &[RegistryCoverage],
        removals: &[(&str, &str)],
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let mut removal_ids = BTreeSet::new();
        for (id, removed_in) in removals {
            if !removal_ids.insert(*id) {
                diagnostics.push(format!("duplicate removal gate: {id}"));
            }
            if version_parts(removed_in).is_none() {
                diagnostics.push(format!("invalid removal version for {id}: {removed_in}"));
            }
            if !coverage.iter().any(|entry| entry.registry_key == *id) {
                diagnostics.push(format!("removal gate has no RegistryCoverage entry: {id}"));
            }
        }
        let mut fixture_ids: BTreeMap<&str, &str> = BTreeMap::new();
        let mut previous = None;
        for registry in &fixture.registries {
            if previous.is_some_and(|id: &str| id >= registry.registry_id.as_str()) {
                diagnostics.push(format!(
                    "fixture registries are not strictly ordered at {}",
                    registry.registry_id
                ));
            }
            previous = Some(&registry.registry_id);
            if fixture_ids
                .insert(&registry.registry_id, &registry.datapack_dir)
                .is_some()
            {
                diagnostics.push(format!(
                    "duplicate fixture registry: {}",
                    registry.registry_id
                ));
            }
        }

        let mut coverage_ids = BTreeSet::new();
        for entry in coverage {
            if !valid_resource_location(entry.registry_key) {
                diagnostics.push(format!(
                    "invalid registry resource location: {}",
                    entry.registry_key
                ));
                continue;
            }
            if !valid_dir(entry.datapack_dir) {
                diagnostics.push(format!(
                    "invalid datapack directory for {}: {}",
                    entry.registry_key, entry.datapack_dir
                ));
            }
            if entry
                .version_gate
                .is_some_and(|gate| version_parts(gate).is_none())
            {
                diagnostics.push(format!(
                    "invalid version gate for {}: {:?}",
                    entry.registry_key, entry.version_gate
                ));
                continue;
            }
            if !active_for(entry, &fixture.minecraft_version, removals) {
                continue;
            }
            if !coverage_ids.insert(entry.registry_key) {
                diagnostics.push(format!(
                    "duplicate RegistryCoverage entry: {}",
                    entry.registry_key
                ));
            }
            match fixture_ids.get(entry.registry_key) {
                None => diagnostics.push(format!(
                    "stale RegistryCoverage entry for {} in Minecraft {}",
                    entry.registry_key, fixture.minecraft_version
                )),
                Some(dir) if *dir != entry.datapack_dir => diagnostics.push(format!(
                    "datapack directory mismatch for {}: vanilla={}, Sand={}",
                    entry.registry_key, dir, entry.datapack_dir
                )),
                Some(_) => {}
            }
        }

        for registry in fixture_ids.keys() {
            if !coverage_ids.contains(*registry) {
                diagnostics.push(format!(
                    "missing RegistryCoverage entry: {} -> data/<namespace>/{}",
                    registry, fixture_ids[registry]
                ));
            }
        }
        diagnostics.sort();
        diagnostics
    }

    fn drift(fixture: &Fixture, coverage: &[RegistryCoverage]) -> Vec<String> {
        drift_with_removals(fixture, coverage, REGISTRY_REMOVED_IN)
    }

    fn test_row(
        key: &'static str,
        dir: &'static str,
        gate: Option<&'static str>,
    ) -> RegistryCoverage {
        RegistryCoverage {
            registry_key: key,
            datapack_dir: dir,
            tag_dir: None,
            sand_module: None,
            api_status: RegistryApiStatus::RawOnly,
            version_gate: gate,
            notes: "test",
        }
    }

    #[test]
    fn coverage_table_is_non_empty() {
        assert!(!REGISTRY_COVERAGE.is_empty());
    }

    #[test]
    fn all_registry_keys_are_namespaced() {
        for entry in REGISTRY_COVERAGE {
            assert!(
                valid_resource_location(entry.registry_key),
                "registry_key must be a resource location: '{}'",
                entry.registry_key
            );
            assert!(valid_dir(entry.datapack_dir));
        }
    }

    #[test]
    fn fully_implemented_registries_have_sand_module() {
        for entry in REGISTRY_COVERAGE {
            if matches!(entry.api_status, RegistryApiStatus::FullyImplemented) {
                assert!(
                    entry.sand_module.is_some(),
                    "registry '{}' is FullyImplemented but has no sand_module",
                    entry.registry_key
                );
            }
        }
    }

    #[test]
    fn no_duplicate_registry_keys() {
        let mut seen = std::collections::HashSet::new();
        for entry in REGISTRY_COVERAGE {
            assert!(
                seen.insert(entry.registry_key),
                "duplicate registry_key: '{}'",
                entry.registry_key
            );
        }
    }

    #[test]
    fn checked_in_fixtures_match_coverage() {
        for fixture in [
            parse_fixture(include_str!("../fixtures/registry-coverage/1.21.4.json")),
            parse_fixture(include_str!("../fixtures/registry-coverage/26.2.json")),
        ] {
            assert!(fixture.provenance.contains("datapack.json"));
            assert_eq!(drift(&fixture, REGISTRY_COVERAGE), Vec::<String>::new());
        }
    }

    #[test]
    fn latest_fixture_tracks_latest_known() {
        let fixture = parse_fixture(include_str!("../fixtures/registry-coverage/26.2.json"));
        assert_eq!(fixture.minecraft_version, sand_version::LATEST_KNOWN);
    }

    #[test]
    fn synthetic_drift_diagnostics_are_actionable() {
        let fixture = parse_fixture(
            r#"{"minecraft_version":"1.0","provenance":"test","registries":[{"registry_id":"minecraft:new","datapack_dir":"new"}]}"#,
        );
        assert_eq!(
            drift(&fixture, &[]),
            ["missing RegistryCoverage entry: minecraft:new -> data/<namespace>/new"]
        );

        let diagnostics = drift(&fixture, &[test_row("minecraft:old", "old", None)]);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.contains("stale RegistryCoverage entry"))
        );
    }

    #[test]
    fn detects_directory_duplicates_invalid_ids_and_version_gates() {
        let fixture = parse_fixture(
            r#"{"minecraft_version":"26.2","provenance":"test","registries":[{"registry_id":"minecraft:ok","datapack_dir":"ok"}]}"#,
        );
        assert!(
            drift(&fixture, &[test_row("minecraft:ok", "wrong", None)])
                .iter()
                .any(|d| d.contains("datapack directory mismatch"))
        );
        assert!(
            drift(
                &fixture,
                &[
                    test_row("minecraft:ok", "ok", None),
                    test_row("minecraft:ok", "ok", None)
                ]
            )
            .iter()
            .any(|d| d.contains("duplicate RegistryCoverage"))
        );
        assert!(
            drift(
                &fixture,
                &[test_row("minecraft:block (tags)", "tags/block", None)]
            )
            .iter()
            .any(|d| d.contains("invalid registry resource location"))
        );
        assert!(
            drift(&fixture, &[test_row("minecraft:ok", "ok", Some("future"))])
                .iter()
                .any(|d| d.contains("invalid version gate"))
        );
    }

    #[test]
    fn version_gates_and_explicit_non_typed_statuses_are_valid() {
        let fixture =
            parse_fixture(r#"{"minecraft_version":"1.0","provenance":"test","registries":[]}"#);
        let gated = test_row("minecraft:future", "future", Some("2.0"));
        assert!(drift(&fixture, &[gated]).is_empty());

        let old_fixture = parse_fixture(
            r#"{"minecraft_version":"1.0","provenance":"test","registries":[{"registry_id":"minecraft:old","datapack_dir":"old"}]}"#,
        );
        let new_fixture =
            parse_fixture(r#"{"minecraft_version":"2.0","provenance":"test","registries":[]}"#);
        let old = test_row("minecraft:old", "old", None);
        assert!(drift_with_removals(&old_fixture, &[old], &[("minecraft:old", "2.0")]).is_empty());
        let old = test_row("minecraft:old", "old", None);
        assert!(drift_with_removals(&new_fixture, &[old], &[("minecraft:old", "2.0")]).is_empty());

        for status in [
            RegistryApiStatus::RawOnly,
            RegistryApiStatus::PartiallyImplemented,
            RegistryApiStatus::IntentionallyUnsupported,
        ] {
            let mut row = test_row("minecraft:ok", "ok", None);
            row.api_status = status;
            let present = parse_fixture(
                r#"{"minecraft_version":"1.0","provenance":"test","registries":[{"registry_id":"minecraft:ok","datapack_dir":"ok"}]}"#,
            );
            assert!(drift(&present, &[row]).is_empty());
        }
    }

    #[test]
    fn tag_rows_are_separate_and_pseudo_ids_cannot_masquerade_as_registries() {
        assert!(
            TAG_COVERAGE
                .iter()
                .all(|tag| valid_resource_location(tag.value_registry))
        );
        assert!(TAG_COVERAGE.iter().all(|tag| valid_dir(tag.datapack_dir)));
        assert!(
            !REGISTRY_COVERAGE
                .iter()
                .any(|entry| entry.registry_key.contains("(tags)"))
        );
    }

    #[test]
    fn missing_registries_have_notes() {
        for entry in REGISTRY_COVERAGE {
            if matches!(entry.api_status, RegistryApiStatus::Missing) {
                assert!(
                    !entry.notes.is_empty(),
                    "missing registry '{}' must have notes explaining the gap or escape hatch",
                    entry.registry_key
                );
            }
        }
    }
}
