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
//! - Enables future automation: scripts can compare this table against the
//!   vanilla generated report to detect newly-added registries.
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
    // ── Tags ─────────────────────────────────────────────────────────────────
    RegistryCoverage {
        registry_key: "minecraft:block (tags)",
        datapack_dir: "tags/block",
        tag_dir: None,
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Tag<Block> exists. No built-in block ID enum yet; use string IDs.",
    },
    RegistryCoverage {
        registry_key: "minecraft:item (tags)",
        datapack_dir: "tags/item",
        tag_dir: None,
        sand_module: Some("sand_components::tag"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Tag<Item> exists. No built-in item ID enum yet; use string IDs.",
    },
    RegistryCoverage {
        registry_key: "minecraft:entity_type (tags)",
        datapack_dir: "tags/entity_type",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::Missing,
        version_gate: None,
        notes: "Not implemented. Use RawComponent or Tag<String>.",
    },
    RegistryCoverage {
        registry_key: "minecraft:fluid (tags)",
        datapack_dir: "tags/fluid",
        tag_dir: None,
        sand_module: None,
        api_status: RegistryApiStatus::IntentionallyUnsupported,
        version_gate: None,
        notes: "Fluid tags are rarely needed in datapacks. Use RawComponent if required.",
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
        tag_dir: None,
        sand_module: Some("sand_components::dialog"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21.6"),
        notes: "Dialog stub exists. cmd::show_dialog() works. Full dialog JSON serialization incomplete. See #16.",
    },
];

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_table_is_non_empty() {
        assert!(!REGISTRY_COVERAGE.is_empty());
    }

    #[test]
    fn all_registry_keys_are_namespaced() {
        for entry in REGISTRY_COVERAGE {
            assert!(
                entry.registry_key.contains(':'),
                "registry_key must be namespaced: '{}'",
                entry.registry_key
            );
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
    fn coverage_table_is_stable() {
        assert_eq!(
            REGISTRY_COVERAGE.len(),
            35,
            "registry coverage table size changed — update this count when adding/removing entries"
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
