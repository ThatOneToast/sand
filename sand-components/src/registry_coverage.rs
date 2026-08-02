//! Datapack component coverage audit for Sand.
//!
//! This module provides a static compile-time table of every known vanilla Java
//! Edition data-driven registry, along with Sand's implementation status. It
//! also tracks datapack assets that are not data-driven registries, such as
//! copy-backed binary structure templates.
//!
//! # Purpose
//!
//! - Single source of truth for registry, tag, and non-registry datapack asset
//!   parity.
//! - Makes gaps explicit: missing registries are listed as `Missing` rather
//!   than silently absent from the codebase.
//! - Checked-in fixtures compare this table against Mojang's generated
//!   `datapack.json` report to detect newly-added or renamed registries.
//! - Tag-only coverage lives in [`TAG_COVERAGE`] and never masquerades as a
//!   vanilla registry identifier.
//! - Non-registry files live in [`DATAPACK_ASSET_COVERAGE`] so they remain
//!   visible without pretending they are entries in Mojang's registry report.
//!
//! # Usage
//!
//! ```
//! use sand_components::registry_coverage::{
//!     DATAPACK_ASSET_COVERAGE, REGISTRY_COVERAGE, RegistryApiStatus,
//! };
//!
//! let missing: Vec<_> = REGISTRY_COVERAGE
//!     .iter()
//!     .filter(|r| matches!(r.api_status, RegistryApiStatus::Missing))
//!     .collect();
//! println!("{} missing registries", missing.len());
//! println!("{} non-registry assets", DATAPACK_ASSET_COVERAGE.len());
//! ```

// ── Status enums ──────────────────────────────────────────────────────────────

/// How well Sand's typed API covers the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryApiStatus {
    /// A typed Sand module exists and generates correct JSON paths.
    ///
    /// This is a stronger claim than "a module exists": it requires
    /// **all** of the following to hold for the normal (non-escape-hatch)
    /// public API —
    ///
    /// - every field a typed builder exposes on the normal path is either
    ///   a typed/validated value or goes through export-time validation
    ///   (`DatapackComponent::validate`) that rejects malformed input
    ///   before JSON is written (no silently-serialized empty strings,
    ///   unchecked numeric ranges, or unvalidated resource IDs);
    /// - correct datapack path generation for the registry's directory;
    /// - at least one focused serialization/golden JSON test.
    ///
    /// See [`KNOWN_PARTIAL_REGISTRIES`] and
    /// `registry_coverage_status_matches_known_gaps` for the automated
    /// guard that keeps this claim from silently going stale. A row may
    /// only be promoted to `FullyImplemented` by also updating (or
    /// removing) its entry in `KNOWN_PARTIAL_REGISTRIES`, and its `notes`
    /// field should explain *why* the claim holds (not just restate the
    /// module name) — see [`RegistryCoverage::notes`].
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
    /// The registry containing values referenced by this tag family.
    pub value_registry: &'static str,
    /// The tag folder path relative to `data/<namespace>/`.
    pub datapack_dir: &'static str,
    /// The Sand module that provides this tag API, if any.
    pub sand_module: Option<&'static str>,
    /// Implementation status.
    pub api_status: RegistryApiStatus,
    /// Notes about typed coverage or escape hatches.
    pub notes: &'static str,
}

/// How a non-registry datapack asset is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatapackAssetKind {
    /// Sand validates and copies an existing binary file without parsing its
    /// format or generating its contents.
    CopyBackedBinary,
}

/// Coverage for a datapack component that is not a data-driven registry or tag.
///
/// Mojang's generated `datapack.json` report cannot discover these files, so
/// they are deliberately separate from [`REGISTRY_COVERAGE`].
#[derive(Debug)]
pub struct DatapackAssetCoverage {
    /// Stable Sand-facing name for the asset family.
    pub asset_name: &'static str,
    /// The datapack folder path relative to `data/<namespace>/`.
    pub datapack_dir: &'static str,
    /// File extension without a leading dot.
    pub file_extension: &'static str,
    /// The Sand module that provides this asset API.
    pub sand_module: &'static str,
    /// How Sand produces the asset.
    pub kind: DatapackAssetKind,
    /// Implementation status.
    pub api_status: RegistryApiStatus,
    /// Notes about validation boundaries and unsupported behavior.
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
        notes: "Advancement, AdvancementTrigger, AdvancementDisplay. 50+ trigger variants. Normal display, icon, \
                parent, reward, and trigger reference paths are typed; unsupported/custom shapes require explicitly \
                named raw escape hatches. See trigger_coverage.",
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
        notes: "LootTable, LootPool, LootEntry exist. #185 typed the normal item/tag/loot-table-reference \
                entry constructors (ItemId, TagId<ItemId>, LootTableId), set_name/set_lore (TextComponent-backed \
                LootText), enchant_with_levels/enchant_randomly (EnchantmentSelector), and random_sequence \
                (ResourceLocation), each with explicitly named raw escape hatches still checked by export \
                validation. LootCondition::{EntityProperties, MatchTool, TimeCheck} and \
                NumberProvider::Score::target remain raw serde_json::Value predicate/range/target payloads \
                pending shared predicate/range validation (#137). \
                #17 (original registry-coverage generation issue) is closed and does not track this specific gap.",
    },
    RegistryCoverage {
        registry_key: "minecraft:predicate",
        datapack_dir: "predicate",
        tag_dir: None,
        sand_module: Some("sand_components::predicate"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "#204 added a dedicated PredicateRoot typed condition tree (AllOf/AnyOf/Inverted, \
                EntityProperties with a typed EntityPredicateTarget, LocationCheck, WeatherCheck, \
                TimeCheck, RandomChance, Reference via the typed PredicateId, and an explicit Raw \
                escape hatch), replacing the previous LootCondition-only wrapper. Predicate is no \
                longer coupled to loot-table condition internals as the only normal authoring path; \
                Predicate::from_loot_condition remains for LootCondition-based compatibility and \
                falls back to the Raw escape hatch for loot-only condition shapes. Export-time \
                validation covers empty all_of/any_of terms, chance/range bounds, and nested \
                predicate validation.",
    },
    RegistryCoverage {
        registry_key: "minecraft:recipe",
        datapack_dir: "recipe",
        tag_dir: None,
        sand_module: Some("sand_components::recipe"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "All standard recipe types implemented: shaped, shapeless, smelting, blasting, smoking, campfire, \
                smithing_transform, smithing_trim, stonecutting. #178 (route recipe ingredient/result IDs through \
                typed item/tag IDs) is closed and confirmed done: Ingredient::item_id/item_tag and \
                RecipeResult::item take IntoRecipeItemId/TagId<ItemId> on the normal path; raw_item/raw_tag/raw \
                remain as explicitly named compatibility escape hatches. Not downgraded per #193 (a prior review \
                pass incorrectly cited #178 as still open; verified closed 2026-07-12 with the typed API already \
                present on this branch's base).",
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
        notes: "Enchantment uses typed TextComponent description, ItemOrTag/EnchantmentOrTag \
                (ItemId/EnchantmentId/TagId<T>) for supported_items/primary_items/exclusive_set, \
                and the reused EquipmentSlotGroup enum for slots on the normal path. Effects use a \
                typed EnchantmentEffectComponentId key plus a small value-effect slice covering \
                minecraft:damage, minecraft:knockback, and minecraft:armor_effectiveness \
                (EnchantmentValueOperation + LevelBasedValue); raw_effect_component and raw_effects \
                remain explicit escape hatches for other/custom effect components. Full vanilla \
                effect-component coverage is intentionally partial — see module docs. Typed \
                migration: #202.",
    },
    RegistryCoverage {
        registry_key: "minecraft:enchantment_provider",
        datapack_dir: "enchantment_provider",
        tag_dir: None,
        sand_module: Some("sand_components::enchantment_provider"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21"),
        notes: "EnchantmentProvider covers single, by_cost, and by_cost_with_difficulty with typed \
                enchantment IDs/tags and constant/uniform integer providers. Other integer-provider \
                and modded provider shapes use the explicit whole-provider RawJson escape hatch. \
                Added with data-driven enchantments in 1.21. Follow-up: #188.",
    },
    RegistryCoverage {
        registry_key: "minecraft:jukebox_song",
        datapack_dir: "jukebox_song",
        tag_dir: None,
        sand_module: Some("sand_components::jukebox_song"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21"),
        notes: "JukeboxSong. Introduced in 1.21. sound_event/song_length/comparator_output are \
                typed/validated on the normal path, but description is `Option<serde_json::Value>` \
                behind the unnamed `description()` setter with no typed TextComponent path and no \
                export-time validation at all. Overstated FullyImplemented found during #193's \
                audit; tracked by #321.",
    },
    RegistryCoverage {
        registry_key: "minecraft:instrument",
        datapack_dir: "instrument",
        tag_dir: None,
        sand_module: Some("sand_components::instrument"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Instrument (goat horn, etc.). sound_event/use_duration/range are typed/validated on \
                the normal path, but description is `Option<serde_json::Value>` behind the unnamed \
                `description()` setter with no typed TextComponent path and no export-time \
                validation at all. Overstated FullyImplemented found during #193's audit; tracked \
                by #321.",
    },
    RegistryCoverage {
        registry_key: "minecraft:painting_variant",
        datapack_dir: "painting_variant",
        tag_dir: None,
        sand_module: Some("sand_components::painting_variant"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "PaintingVariant. validate() rejects empty/malformed asset_id and width/height outside 1..=16 \
                before export. Golden JSON test: painting_variant::tests::valid_painting_variant_json_is_stable. \
                See #141.",
    },
    RegistryCoverage {
        registry_key: "minecraft:banner_pattern",
        datapack_dir: "banner_pattern",
        tag_dir: None,
        sand_module: Some("sand_components::banner_pattern"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: None,
        notes: "BannerPattern. validate() rejects empty/malformed asset_id and empty/control-char translation_key \
                before export. Golden JSON test: banner_pattern::tests::valid_banner_pattern_json_is_stable. \
                See #141.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trim_material",
        datapack_dir: "trim_material",
        tag_dir: None,
        sand_module: Some("sand_components::trim"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.19.4"),
        notes: "TrimMaterial uses typed ItemId, TextComponent, ResourceLocation armor-material keys, and \
                validated TrimAssetName values on the normal path; raw descriptions/override objects are \
                explicit. item_model_index remains the legacy pre-1.21.4 field and current \
                override_armor_assets is not yet modeled. Golden JSON test: \
                trim::tests::valid_trim_material_json_is_stable. #198 (typed ID migration) is closed; \
                the remaining override_armor_assets gap is tracked by #322.",
    },
    RegistryCoverage {
        registry_key: "minecraft:trim_pattern",
        datapack_dir: "trim_pattern",
        tag_dir: None,
        sand_module: Some("sand_components::trim"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19.4"),
        notes: "TrimPattern uses typed ResourceLocation, ItemId, and TextComponent inputs on the normal \
                path, with an explicitly named raw text escape hatch. Golden JSON test: \
                trim::tests::valid_trim_pattern_json_is_stable. Typed migration: #198.",
    },
    RegistryCoverage {
        registry_key: "minecraft:wolf_variant",
        datapack_dir: "wolf_variant",
        tag_dir: None,
        sand_module: Some("sand_components::wolf_variant"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.20.5"),
        notes: "WolfVariant. Introduced in 1.20.5. validate() rejects empty/malformed texture paths and \
                unsupported biomes JSON shapes (empty string/array, non-string entries, non-string/array \
                top-level shapes) before export. Golden JSON test: wolf_variant::tests::valid_wolf_variant_json_is_stable. \
                See #141.",
    },
    RegistryCoverage {
        registry_key: "minecraft:chat_type",
        datapack_dir: "chat_type",
        tag_dir: None,
        sand_module: Some("sand_components::chat_type"),
        api_status: RegistryApiStatus::FullyImplemented,
        version_gate: Some("1.19"),
        notes: "ChatType, ChatDecoration. Introduced in 1.19. Normal-path authoring is typed: \
                ChatDecorationParameter (Sender/Target/Content, with an explicit Custom(_) escape \
                hatch that still fails validation unless it matches a known vanilla value) and \
                ChatStyle (color/color_hex/bold/italic/underlined/strikethrough/obfuscated/insertion) \
                cover normal authoring; validate() rejects unknown/duplicate parameters, empty or \
                control-character translation keys, invalid hex colors, and non-object raw styles. \
                The raw serde_json::Value path remains available only via the explicitly named \
                ChatDecoration::style_raw escape hatch. Golden JSON test: \
                chat_type::tests::golden_chat_type_json and \
                canonical_26_2_chat_type_with_typed_parameters_and_style. Restored/kept \
                FullyImplemented per #199 and #193 (this row was left for this stream to update by \
                #288/#193 to avoid a merge conflict; see #199).",
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
        sand_module: Some("sand_components::chicken_variant"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21.5"),
        notes: "ChickenVariant covers asset_id and the minecraft:biome spawn_conditions shape \
                (shared SpawnCondition model, also used by cow_variant/pig_variant) on the normal \
                path; other vanilla/modded fields and spawn-condition types use the explicit \
                raw_field escape hatch. Introduced in 1.21.5. Golden JSON test: \
                chicken_variant::tests::valid_chicken_variant_json_is_stable. #201.",
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
        sand_module: Some("sand_components::cow_variant"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21.5"),
        notes: "CowVariant covers asset_id and the minecraft:biome spawn_conditions shape (shared \
                SpawnCondition model). The vanilla model selector and other fields use the explicit \
                raw_field escape hatch pending confirmation of its exact accepted values. Introduced \
                in 1.21.5. Golden JSON test: cow_variant::tests::valid_cow_variant_json_is_stable. \
                #201.",
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
        sand_module: Some("sand_components::pig_variant"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: Some("1.21.5"),
        notes: "PigVariant covers asset_id and the minecraft:biome spawn_conditions shape (shared \
                SpawnCondition model). Other vanilla/modded fields and spawn-condition types use the \
                explicit raw_field escape hatch. Introduced in 1.21.5. Golden JSON test: \
                pig_variant::tests::valid_pig_variant_json_is_stable. #201.",
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
        sand_module: Some("sand_components::worldgen::configured_feature"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed ConfiguredFeatureId references and a small common builder slice (no_op, simple_block, fill_layer, ore); other vanilla feature shapes use the explicit ConfiguredFeature::raw escape hatch.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/structure",
        datapack_dir: "worldgen/structure",
        tag_dir: Some("tags/worldgen/structure"),
        sand_module: Some("sand_components::worldgen::structure"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed builder covers minecraft:jigsaw structures (biomes, step, terrain adaptation, spawn overrides, jigsaw config) with a raw_field escape hatch. Other structure types (e.g. minecraft:mineshaft, minecraft:end_city) and their type-specific fields go through Structure::new plus raw_field. #187",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/structure_set",
        datapack_dir: "worldgen/structure_set",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::structure_set"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed builder covers weighted structure entries, random_spread and concentric_rings placement (with spacing/separation/salt/distance/count validation), and exclusion zones, with a raw_field escape hatch. #187",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/processor_list",
        datapack_dir: "worldgen/processor_list",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::processor_list"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed builder covers block_ignore, protected_blocks, gravity, jigsaw_replacement, and rule processors (typed output_state, raw predicates), with Processor::Raw as the escape hatch for other processor types. #187",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/template_pool",
        datapack_dir: "worldgen/template_pool",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::template_pool"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed builder covers single, legacy_single, empty, feature, and list pool elements plus named/inline processor references, with PoolElement::Raw as the escape hatch for other element types. #187",
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
        sand_module: Some("sand_components::worldgen::density_function"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed DensityFunctionExpr covers constant/reference/noise sampling, unary/binary combinators, clamp, and y_clamped_gradient with DensityFunctionId references; other vanilla variants (spline, weird_scaled_sampler, etc.) use the explicit DensityFunctionExpr::raw/DensityFunction::new_raw escape hatch.",
    },
    RegistryCoverage {
        registry_key: "minecraft:worldgen/noise",
        datapack_dir: "worldgen/noise",
        tag_dir: None,
        sand_module: Some("sand_components::worldgen::noise"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed Noise builder covers the stable firstOctave/amplitudes shape and NoiseId references, validated for finite octave ranges; version-specific additions use an explicit raw_field escape hatch.",
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
        sand_module: Some("sand_components::worldgen::dimension_type"),
        api_status: RegistryApiStatus::PartiallyImplemented,
        version_gate: None,
        notes: "Typed builder covers the stable vanilla-like field set and DimensionTypeId references; version-specific additions use an explicit raw_field escape hatch.",
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

/// Coverage for public datapack assets omitted from Mojang's registry report.
///
/// These rows complement [`REGISTRY_COVERAGE`] and [`TAG_COVERAGE`]. They must
/// not be fed into registry-drift comparisons because their directories are
/// ordinary datapack file families rather than registry identifiers.
pub const DATAPACK_ASSET_COVERAGE: &[DatapackAssetCoverage] = &[DatapackAssetCoverage {
    asset_name: "structure_template",
    datapack_dir: "structure",
    file_extension: "nbt",
    sand_module: "sand_components::structure_template",
    kind: DatapackAssetKind::CopyBackedBinary,
    api_status: RegistryApiStatus::FullyImplemented,
    notes: "StructureTemplate validates a project-relative .nbt source and copies it to \
            data/<namespace>/structure/<path>.nbt. Sand does not parse, generate, or \
            semantically validate the binary NBT payload.",
}];

/// Exclusive upper version gates for registries removed or renamed by Mojang.
///
/// The table is currently empty because both checked profiles are additive.
/// Keeping removals separate preserves the existing `version_gate` (introduced
/// in) API while allowing a registry to remain valid for older fixtures.
pub const REGISTRY_REMOVED_IN: &[(&str, &str)] = &[];

// ── Coverage-status consistency guard (#193) ───────────────────────────────────

/// Registries known to have normal-path typedness gaps, keyed to the issue(s)
/// tracking their remaining work.
///
/// This is a small, explicit allowlist. `docs/typedness-audit.md` (a
/// separate prose inventory of the same migrations, maintained alongside
/// each typed-migration PR) is **not** the source of truth this fixture
/// checks itself against — an earlier version of this comment claimed that
/// file had been removed in #262; it was not (verified against the current
/// repository while auditing #193). Keep both updated when a registry's
/// typedness changes, but treat this table plus [`REGISTRY_COVERAGE`]'s own
/// `notes` as authoritative for the automated guards below. Each entry here
/// asserts two things about the matching [`REGISTRY_COVERAGE`] row:
///
/// 1. its `api_status` is **not** [`RegistryApiStatus::FullyImplemented`];
/// 2. its `notes` mention every listed issue reference.
///
/// When a registry genuinely becomes fully typed, remove its entry here in
/// the same change that promotes its `REGISTRY_COVERAGE` row — that keeps a
/// promotion from silently slipping in without a corresponding audit.
pub const KNOWN_PARTIAL_REGISTRIES: &[(&str, &[&str])] = &[
    ("minecraft:loot_table", &["#137"]),
    ("minecraft:jukebox_song", &["#321"]),
    ("minecraft:instrument", &["#321"]),
    ("minecraft:trim_material", &["#322"]),
];

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
    fn datapack_asset_rows_are_valid_and_unique() {
        assert!(
            !DATAPACK_ASSET_COVERAGE.is_empty(),
            "non-registry datapack assets must remain visible to coverage audits"
        );

        let mut names = BTreeSet::new();
        for asset in DATAPACK_ASSET_COVERAGE {
            assert!(
                names.insert(asset.asset_name),
                "duplicate datapack asset coverage row: {}",
                asset.asset_name
            );
            assert!(
                valid_dir(asset.datapack_dir),
                "invalid datapack asset directory: {}",
                asset.datapack_dir
            );
            assert!(
                !asset.file_extension.is_empty()
                    && !asset.file_extension.starts_with('.')
                    && asset
                        .file_extension
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()),
                "invalid datapack asset extension: {}",
                asset.file_extension
            );
            assert!(!asset.sand_module.is_empty());
            assert!(!asset.notes.is_empty());
        }
    }

    #[test]
    fn structure_templates_are_tracked_as_copy_backed_binary_assets() {
        let asset = DATAPACK_ASSET_COVERAGE
            .iter()
            .find(|asset| asset.asset_name == "structure_template")
            .expect("StructureTemplate must remain represented in the coverage audit");

        assert_eq!(asset.datapack_dir, "structure");
        assert_eq!(asset.file_extension, "nbt");
        assert_eq!(asset.sand_module, "sand_components::structure_template");
        assert_eq!(asset.kind, DatapackAssetKind::CopyBackedBinary);
        assert_eq!(asset.api_status, RegistryApiStatus::FullyImplemented);
        assert!(
            asset.notes.contains("does not parse") && asset.notes.contains("semantically validate"),
            "coverage must not overstate Sand's binary NBT support"
        );
        assert!(
            !REGISTRY_COVERAGE
                .iter()
                .any(|registry| registry.datapack_dir == asset.datapack_dir),
            "structure templates are assets, not data-driven registry rows"
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

    /// Regression guard for #193: `RegistryCoverage` must not claim
    /// `FullyImplemented` for a registry that [`KNOWN_PARTIAL_REGISTRIES`]
    /// still identifies as having normal-path typedness gaps, and the
    /// row's notes must actually reference the owning issue(s) rather than
    /// a vague "partial" statement.
    ///
    /// If this test fails because a registry was promoted to
    /// `FullyImplemented`, either the promotion is real (remove the entry
    /// from `KNOWN_PARTIAL_REGISTRIES` in the same change) or the row was
    /// promoted without actually closing the gap (fix the row instead).
    #[test]
    fn registry_coverage_status_matches_known_gaps() {
        for (registry_key, issues) in KNOWN_PARTIAL_REGISTRIES {
            let entry = REGISTRY_COVERAGE
                .iter()
                .find(|e| &e.registry_key == registry_key)
                .unwrap_or_else(|| {
                    panic!("KNOWN_PARTIAL_REGISTRIES references unknown registry '{registry_key}'")
                });
            assert_ne!(
                entry.api_status,
                RegistryApiStatus::FullyImplemented,
                "registry '{registry_key}' is listed in KNOWN_PARTIAL_REGISTRIES but its \
                 REGISTRY_COVERAGE row claims FullyImplemented; either the typedness gap is \
                 closed (remove it from KNOWN_PARTIAL_REGISTRIES) or the row is overstated \
                 (downgrade api_status)"
            );
            for issue in *issues {
                assert!(
                    entry.notes.contains(issue),
                    "registry '{registry_key}' notes must reference tracking issue {issue}: {:?}",
                    entry.notes
                );
            }
        }
    }

    /// Every row claiming `FullyImplemented` must carry non-trivial notes
    /// that justify the claim (not merely restate the module name), per the
    /// stricter definition documented on [`RegistryApiStatus::FullyImplemented`].
    #[test]
    fn fully_implemented_rows_have_justification_notes() {
        for entry in REGISTRY_COVERAGE {
            if entry.api_status == RegistryApiStatus::FullyImplemented {
                assert!(
                    entry.notes.len() >= 20,
                    "registry '{}' claims FullyImplemented but notes are too short to justify \
                     it: {:?}",
                    entry.registry_key,
                    entry.notes
                );
            }
        }
    }

    // ── Structural evidence guard (#193) ────────────────────────────────────

    fn collect_rs_files(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if path.is_file() {
            out.push(path.to_path_buf());
            return;
        }
        if path.join("mod.rs").is_file() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file = entry.path();
                    if file.extension().is_some_and(|ext| ext == "rs") {
                        out.push(file);
                    }
                }
            }
            return;
        }
        let single_file = path.with_extension("rs");
        if single_file.is_file() {
            out.push(single_file);
        }
    }

    /// Finds public setters that take a bare `Value` parameter (not
    /// `RawJson`) whose name does not identify itself as a raw escape hatch.
    ///
    /// This is a source-text heuristic, not a parser: it is deliberately
    /// simple so its false-positive/false-negative behavior stays legible in
    /// review, and it is verified against every currently `FullyImplemented`
    /// module's real source in [`fully_implemented_modules_have_no_unnamed_raw_value_setters`].
    fn unnamed_raw_value_setters(source: &str) -> Vec<String> {
        let mut findings = Vec::new();
        for chunk in source.split("pub fn ").skip(1) {
            let Some(signature_end) = chunk.find('{') else {
                continue;
            };
            let signature = &chunk[..signature_end];
            let Some(paren) = signature.find('(') else {
                continue;
            };
            let name = signature[..paren].trim();
            if name.contains("raw") {
                continue;
            }
            if signature.contains(": Value") && !signature.contains("RawJson") {
                findings.push(name.to_string());
            }
        }
        findings
    }

    /// Structural regression guard for #193's core complaint: a
    /// `FullyImplemented` claim was previously backed only by free-text
    /// `notes`, which nothing checked against the row's actual source and
    /// which a future edit could invalidate without anyone re-listing the
    /// registry in [`KNOWN_PARTIAL_REGISTRIES`]. #321 found exactly this
    /// shape for `minecraft:jukebox_song`/`minecraft:instrument`: both
    /// claimed `FullyImplemented` while `description` was a bare
    /// `serde_json::Value` behind an *unnamed* setter (no typed path, no
    /// export-time validation, and never manually added to the allowlist).
    ///
    /// This test scans the actual current source of every `FullyImplemented`
    /// row's `sand_module` for that exact anti-pattern so the same class of
    /// regression fails the build even when nobody remembers to update
    /// `KNOWN_PARTIAL_REGISTRIES` by hand. It is intentionally a heuristic
    /// (see [`unnamed_raw_value_setters`]) rather than a full typedness
    /// checker: it cannot prove a row *is* fully implemented, only catch this
    /// specific shape once it appears in source.
    ///
    /// `sand_core::function` is skipped: it generates `.mcfunction` text via
    /// a macro rather than a JSON builder, so the setter-shape heuristic does
    /// not apply.
    #[test]
    fn fully_implemented_modules_have_no_unnamed_raw_value_setters() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let modules = REGISTRY_COVERAGE
            .iter()
            .filter(|e| e.api_status == RegistryApiStatus::FullyImplemented)
            .map(|e| (e.registry_key, e.sand_module))
            .chain(
                TAG_COVERAGE
                    .iter()
                    .filter(|e| e.api_status == RegistryApiStatus::FullyImplemented)
                    .map(|e| (e.value_registry, e.sand_module)),
            );

        for (registry_key, sand_module) in modules {
            let Some(module) = sand_module else { continue };
            if module == "sand_core::function" {
                continue;
            }
            let Some(relative) = module.strip_prefix("sand_components::") else {
                continue;
            };
            let base = manifest_dir.join("src").join(relative.replace("::", "/"));
            let mut files = Vec::new();
            collect_rs_files(&base, &mut files);
            assert!(
                !files.is_empty(),
                "could not locate any source file for '{registry_key}' module '{module}' \
                 (looked under {}); update the path-derivation logic in this test if the \
                 module was moved",
                base.display()
            );
            for file in files {
                let source = std::fs::read_to_string(&file).unwrap_or_else(|error| {
                    panic!(
                        "failed to read {} while auditing '{registry_key}': {error}",
                        file.display()
                    )
                });
                let findings = unnamed_raw_value_setters(&source);
                assert!(
                    findings.is_empty(),
                    "registry '{registry_key}' claims FullyImplemented, but {} exposes \
                     unnamed setter(s) {findings:?} taking a bare `Value` parameter with no \
                     typed alternative and no compile-time-visible export validation; either \
                     rename to an explicit `raw_*` escape hatch, add a typed alternative, or \
                     downgrade this row to PartiallyImplemented",
                    file.display()
                );
            }
        }
    }
}
