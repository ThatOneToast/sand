//! Trigger coverage audit for Sand advancement triggers.
//!
//! This module provides a static, compile-time-verifiable table of every known
//! vanilla advancement trigger, the Minecraft version it was introduced,
//! and its current implementation status in Sand.
//!
//! # Purpose
//!
//! - Gives contributors a single source of truth for trigger parity.
//! - Keeps typed API presence separate from serialization, vanilla parsing,
//!   and real gameplay evidence.
//! - New Minecraft triggers can be added here first (as `Missing`), then
//!   promoted once the typed variant and tests are in place.
//!
//! # Evidence tiers (#232)
//!
//! `golden_json_tested` alone does not prove a trigger's condition schema is
//! correct for every supported Minecraft profile — see #231, where
//! `minecraft:placed_block`/`minecraft:item_used_on_block` had passing golden
//! tests for years while silently firing unconditionally in real gameplay.
//! [`TriggerCoverage`] tracks four independent, increasingly strong levels
//! of evidence:
//!
//! 1. `api_status` — whether Sand exposes a typed construction API.
//! 2. `golden_json_tested` / `schema_golden_tested_profiles` — fixed-input/fixed-output serialization tests
//!    exist. They prove Sand's *current* output is stable, not that it matches
//!    vanilla's schema.
//! 3. `vanilla_load_tested_profiles` — the JSON was loaded and `reload`-tested
//!    against a real vanilla server (`sand-vanilla-audit` +
//!    `scripts/validate-vanilla-reload.sh`); proves it parses, not that the
//!    criterion fires correctly.
//! 4. `semantic_runtime_tested_profiles` — a real, client-driven gameplay
//!    test proved the criterion fires only for matching events and not for
//!    non-matching ones, for the listed profiles. This is the only tier that
//!    proves in-game correctness, and it is empty for every trigger in this
//!    table today — no automation in this repository can currently issue a
//!    real client-originated action (block placement, item use, etc.); see
//!    `docs/vanilla-reload-validation.md`.
//!
//! Do not treat `Typed` API status, `golden_json_tested`, or a
//! non-empty `vanilla_load_tested_profiles` as proof of semantic runtime
//! correctness — only `semantic_runtime_tested_profiles` is.
//!
//! # How to use
//!
//! ```
//! use sand_components::advancement::trigger_coverage::{TRIGGER_COVERAGE, TriggerApiStatus};
//!
//! for entry in TRIGGER_COVERAGE {
//!     if matches!(entry.api_status, TriggerApiStatus::Missing) {
//!         // entry.trigger_id is not yet implemented
//!     }
//! }
//! ```

// ── Status enums ──────────────────────────────────────────────────────────────

/// How well Sand's typed API covers the trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerApiStatus {
    /// A typed `AdvancementTrigger` variant exists. This says nothing about
    /// schema, vanilla-load, or gameplay evidence; those are tracked
    /// independently on [`TriggerCoverage`].
    Typed,
    /// A typed variant exists but some condition fields are raw `Value`
    /// or not yet modelled, OR no golden JSON test exists yet.
    PartiallyImplemented,
    /// No typed variant exists. Use `AdvancementTrigger::Custom` as the
    /// explicit escape hatch.
    Missing,
    /// Present in Sand but only reachable through `AdvancementTrigger::Custom`.
    RawOnly,
    /// Added in a Minecraft version newer than Sand's known table.
    VersionGated,
    /// Intentionally not modelled (too obscure, server-only, or removed).
    IntentionallyUnsupported,
}

/// Whether Sand can generate a working event wrapper for this trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventWrapperStatus {
    /// An event struct exists and can correctly handle reset/revoke/re-trigger.
    Supported,
    /// Partial support — reward wiring exists but reset/revoke is manual.
    Partial,
    /// No event wrapper; raw advancement JSON only.
    None,
}

/// One row of the advancement trigger coverage table.
#[derive(Debug)]
pub struct TriggerCoverage {
    /// The vanilla trigger ID (e.g. `"minecraft:tick"`).
    pub trigger_id: &'static str,
    /// Minecraft version that introduced this trigger (e.g. `"1.12"`).
    pub since: &'static str,
    /// Minecraft version this trigger was removed, if applicable.
    pub removed_in: Option<&'static str>,
    /// Sand API status for this trigger.
    pub api_status: TriggerApiStatus,
    /// Sand event wrapper status.
    pub event_wrapper: EventWrapperStatus,
    /// Whether at least one golden JSON test exists.
    pub golden_json_tested: bool,
    /// Profiles whose schema-specific condition shape has a focused golden
    /// test. This is serialization evidence, not vanilla-load or gameplay
    /// evidence.
    pub schema_golden_tested_profiles: &'static [&'static str],
    /// Minecraft profiles (version strings, e.g. `"26.2"`) whose generated
    /// JSON for this trigger has been loaded and `reload`-tested against a
    /// real vanilla server via the vanilla audit harness
    /// (`sand-vanilla-audit`, `scripts/validate-vanilla-reload.sh`).
    /// Proves the document parses and the server accepts it — NOT that the
    /// criterion fires with the intended semantics in gameplay.
    pub vanilla_load_tested_profiles: &'static [&'static str],
    /// Minecraft profiles whose criterion has been proven, via a real
    /// gameplay-driven positive/negative test, to fire only for matching
    /// events and not for non-matching ones. Empty unless real client-driven
    /// semantic verification has actually been performed — see
    /// `docs/vanilla-reload-validation.md` for why this differs from
    /// `vanilla_load_tested_profiles`.
    pub semantic_runtime_tested_profiles: &'static [&'static str],
    /// Notes about limitations, version differences, or escape hatches.
    pub notes: &'static str,
}

/// Runtime availability metadata for a vanilla advancement trigger.
///
/// The coverage table records API parity; this metadata is the separate source
/// of truth used to prevent serializing known-invalid trigger IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerMetadata {
    pub id: &'static str,
    pub supported: bool,
    pub diagnostic: Option<&'static str>,
}

/// Return availability metadata for a trigger ID in Sand's supported modern
/// Java profiles. Unknown IDs are intentionally left to `Custom` users.
pub fn trigger_metadata(id: &str) -> TriggerMetadata {
    trigger_metadata_for(id, None)
}

/// Resolve trigger-ID availability for a concrete target profile.
pub fn trigger_metadata_for(id: &str, caps: Option<&sand_version::VersionCaps>) -> TriggerMetadata {
    match id {
        // Verified against the vanilla generated trigger registry for current
        // 1.21.x and 26.x targets: this ID is not present in either registry.
        "minecraft:leveled_up" => TriggerMetadata {
            id: "minecraft:leveled_up",
            supported: false,
            diagnostic: Some(
                "use tick polling: `execute store result score @s <objective> run experience query @s levels`, then compare the stored score",
            ),
        },
        "minecraft:crafted_item" => TriggerMetadata {
            id: "minecraft:crafted_item",
            supported: false,
            diagnostic: Some(
                "this legacy/non-vanilla trigger ID is not registered by Sand's verified vanilla profiles; use the corresponding current typed trigger when available or AdvancementTrigger::Custom with user-verified modded JSON",
            ),
        },
        "minecraft:emptied_bucket" => TriggerMetadata {
            id: "minecraft:emptied_bucket",
            supported: false,
            diagnostic: Some(
                "this legacy/non-vanilla trigger ID is not registered by Sand's verified vanilla profiles; use AdvancementTrigger::Custom only with user-verified modded JSON",
            ),
        },
        "minecraft:thrown_item_picked_up" => TriggerMetadata {
            id: "minecraft:thrown_item_picked_up",
            supported: false,
            diagnostic: Some(
                "this ambiguous legacy trigger was split into minecraft:thrown_item_picked_up_by_entity and minecraft:thrown_item_picked_up_by_player; use AdvancementTrigger::ThrownItemPickedUpByEntity or AdvancementTrigger::ThrownItemPickedUpByPlayer",
            ),
        },
        "minecraft:used_item" => TriggerMetadata {
            id: "minecraft:used_item",
            supported: false,
            diagnostic: Some(
                "this trigger ID is not registered by Sand's verified vanilla profiles; use consume_item for completed consumption, using_item for active-use ticks, or AdvancementTrigger::Custom with user-verified modded JSON",
            ),
        },
        "minecraft:killed_by_crossbow"
            if caps.is_none_or(|caps| caps.is_fallback() || caps.is_at_least(1, 20, 5)) =>
        {
            TriggerMetadata {
                id: "minecraft:killed_by_crossbow",
                supported: false,
                diagnostic: Some(
                    "this trigger was replaced by `minecraft:killed_by_arrow` on current profiles; use AdvancementTrigger::KilledByArrow",
                ),
            }
        }
        _ => TriggerMetadata {
            id: "",
            supported: true,
            diagnostic: None,
        },
    }
}

/// Find the [`TriggerCoverage`] entry for a given trigger ID, if present.
///
/// Returns `None` for unknown/custom trigger IDs — those are allowed through
/// the version gate (custom/modded escape hatch).
pub fn find_coverage(trigger_id: &str) -> Option<&'static TriggerCoverage> {
    TRIGGER_COVERAGE
        .iter()
        .find(|entry| entry.trigger_id == trigger_id)
}

const fn missing_trigger(
    trigger_id: &'static str,
    since: &'static str,
    notes: &'static str,
) -> TriggerCoverage {
    TriggerCoverage {
        trigger_id,
        since,
        removed_in: None,
        api_status: TriggerApiStatus::Missing,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: false,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes,
    }
}

const fn typed_profiled_trigger(
    trigger_id: &'static str,
    since: &'static str,
    event_wrapper: EventWrapperStatus,
    notes: &'static str,
) -> TriggerCoverage {
    TriggerCoverage {
        trigger_id,
        since,
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes,
    }
}

// ── Coverage table ────────────────────────────────────────────────────────────

/// Static coverage table for all known vanilla advancement triggers.
///
/// Triggers are listed alphabetically by trigger ID for easier diffing.
pub const TRIGGER_COVERAGE: &[TriggerCoverage] = &[
    TriggerCoverage {
        trigger_id: "minecraft:allay_drop_item_on_block",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &["1.21.4"],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "Fires when an allay drops an item on a note block. AdvancementTrigger::AllayDropItemOnBlock.",
    },
    missing_trigger(
        "minecraft:any_block_use",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:avoid_vibration",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Fires when a player avoids triggering a sculk sensor. No conditions. AdvancementTrigger::AvoidVibration.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:bee_nest_destroyed",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::BeeNestDestroyed.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:bred_animals",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::BredAnimals. vanilla::AnimalsBreed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:brewed_potion",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::BrewedPotion. vanilla::PotionBrewed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:changed_dimension",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ChangedDimension. vanilla::DimensionChanged event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:channeled_lightning",
        since: "1.13",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ChanneledLightning.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:construct_beacon",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ConstructBeacon.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:consume_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ConsumeItem. vanilla::AnyItemConsumed event.",
    },
    missing_trigger(
        "minecraft:crafter_recipe_crafted",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:crafted_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Legacy source-compatibility variant only. Current vanilla profiles use recipe_crafted/crafter_recipe_crafted; target-aware export rejects this ID instead of emitting an advancement that cannot load.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:cured_zombie_villager",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::CuredZombieVillager.",
    },
    missing_trigger(
        "minecraft:default_block_use",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:effects_changed",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::EffectsChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:enchanted_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::EnchantedItem. vanilla::AnyItemEnchanted event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:emptied_bucket",
        since: "unknown",
        removed_in: None,
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Legacy source-compatibility variant only. This trigger ID is absent from verified current vanilla registries; target-aware export rejects it.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:enter_block",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::EnterBlock.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:entity_hurt_player",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::EntityHurtPlayer. vanilla::EntityDamagesPlayer event. Does NOT infer attacker.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:entity_killed_player",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::EntityKilledPlayer. vanilla::PlayerKill event.",
    },
    missing_trigger(
        "minecraft:fall_after_explosion",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:fall_from_height",
        since: "1.18",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::FallFromHeight.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:filled_bucket",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::FilledBucket.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:fishing_rod_hooked",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::FishingRodHooked.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:hero_of_the_village",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::HeroOfTheVillage.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:impossible",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Never fires. Used for parent-only advancements. AdvancementTrigger::Impossible.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:inventory_changed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::InventoryChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:item_durability_changed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ItemDurabilityChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:item_used_on_block",
        since: "1.19.4",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &["26.2"],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &["1.21.4"],
        notes: "Player right-clicks a block with item. AdvancementTrigger::ItemUsedOnBlock. \
            Filtering renders through AdvancementSchemaFamily-aware conditions.location \
            (#231/#232); real-vanilla load/reload is verified on 1.21.4 and 26.2. \
            A protocol client verifies matching and non-matching gameplay, final-stack \
            behavior, and revoke/re-fire on 1.21.4.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:kill_mob_near_sculk_catalyst",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::KillMobNearSculkCatalyst.",
    },
    typed_profiled_trigger(
        "minecraft:killed_by_arrow",
        "1.21.4",
        EventWrapperStatus::None,
        "AdvancementTrigger::KilledByArrow. Current replacement for killed_by_crossbow in the verified registries.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:killed_by_crossbow",
        since: "1.14",
        removed_in: Some("1.20.5"),
        api_status: TriggerApiStatus::VersionGated,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Legacy source-compatibility variant only on verified current profiles. Vanilla uses killed_by_arrow; target-aware export rejects the stale ID with a migration diagnostic.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:leveled_up",
        since: "1.12",
        removed_in: Some("1.12"),
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: false,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "minecraft:leveled_up is not in the vanilla trigger registry. Kept only for source compatibility; generation fails with an XP polling migration diagnostic.",
    },
    missing_trigger(
        "minecraft:levitation",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:lightning_strike",
        since: "1.17",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::LightningStrike.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:location",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Partial,
        golden_json_tested: true,
        schema_golden_tested_profiles: &["1.21.4", "26.2"],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::Location. Tick-polled player-state events use this.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:nether_travel",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::NetherTravel.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:placed_block",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &["1.21.4", "26.2"],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &["1.21.4"],
        notes: "AdvancementTrigger::PlacedBlock. vanilla::AnyBlockPlaced event. Filtering renders \
            through AdvancementSchemaFamily-aware conditions.location (#231/#232); \
            real-vanilla load/reload is verified on 1.21.4 and 26.2. A protocol client \
            verifies matching and non-matching gameplay, final-stack behavior, and \
            revoke/re-fire on 1.21.4.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_generates_container_loot",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::PlayerGeneratesContainerLoot.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_hurt_entity",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::PlayerHurtEntity. vanilla::PlayerDamagesEntity event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_interacted_with_entity",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::PlayerInteractedWithEntity. Used by systems-entities interaction builder.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_killed_entity",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &["1.21.4", "26.2"],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::PlayerKilledEntity. vanilla::EntityKill event.",
    },
    missing_trigger(
        "minecraft:player_sheared_equipment",
        "26.2",
        "Present in the verified 26.2 registry but not 1.21.4; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    typed_profiled_trigger(
        "minecraft:recipe_crafted",
        "1.21.4",
        EventWrapperStatus::None,
        "AdvancementTrigger::RecipeCrafted. Vanilla requires recipe_id and optionally exposes ingredients; the generic ItemCraftEvent cannot represent that contract.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:recipe_unlocked",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::RecipeUnlocked.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:ride_entity_in_lava",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::RideEntityInLava.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:shot_crossbow",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::ShotCrossbow. vanilla::CrossbowShot event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:slept_in_bed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::SleptInBed.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:slide_down_block",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::SlideDownBlock.",
    },
    missing_trigger(
        "minecraft:spear_mobs",
        "26.2",
        "Present in the verified 26.2 registry but not 1.21.4; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:started_riding",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::StartedRiding. No conditions.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:summoned_entity",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::SummonedEntity. vanilla::EntitySummoned event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:tame_animal",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::TamedAnimal. vanilla::AnimalTamed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:target_hit",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::TargetHit.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:tick",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Fires every tick. Used for join/first-join detection with revoke. vanilla::OnJoin, FirstJoin.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:thrown_item_picked_up",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Legacy source-compatibility variant only. Current vanilla splits this into thrown_item_picked_up_by_entity and thrown_item_picked_up_by_player; target-aware export rejects the ambiguous stale ID.",
    },
    typed_profiled_trigger(
        "minecraft:thrown_item_picked_up_by_entity",
        "1.21.4",
        EventWrapperStatus::None,
        "AdvancementTrigger::ThrownItemPickedUpByEntity.",
    ),
    typed_profiled_trigger(
        "minecraft:thrown_item_picked_up_by_player",
        "1.21.4",
        EventWrapperStatus::Supported,
        "AdvancementTrigger::ThrownItemPickedUpByPlayer. ItemPickedUpEvent.",
    ),
    TriggerCoverage {
        trigger_id: "minecraft:used_ender_eye",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &["1.21.4", "26.2"],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::UsedEnderEye.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:used_item",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Legacy source-compatibility variant only. This trigger ID is absent from verified current vanilla registries; target-aware export rejects it.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:used_totem",
        since: "1.11",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::UsedTotem.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:using_item",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::Partial,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "Fires every tick while player is actively using an item. Tick-polled state events use this.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:villager_trade",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::Typed,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        schema_golden_tested_profiles: &[],
        vanilla_load_tested_profiles: &[],
        semantic_runtime_tested_profiles: &[],
        notes: "AdvancementTrigger::VillagerTrade.",
    },
    missing_trigger(
        "minecraft:voluntary_exile",
        "1.21.4 or earlier",
        "Present in the verified 1.21.4 and 26.2 registries; no typed AdvancementTrigger variant yet. Use Custom with profile-verified conditions.",
    ),
];

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_table_is_non_empty() {
        assert!(
            !TRIGGER_COVERAGE.is_empty(),
            "trigger coverage table must not be empty"
        );
    }

    #[test]
    fn all_trigger_ids_are_namespaced() {
        for entry in TRIGGER_COVERAGE {
            assert!(
                entry.trigger_id.contains(':'),
                "trigger_id must be namespaced: '{}'",
                entry.trigger_id
            );
        }
    }

    #[test]
    fn placed_block_and_item_used_on_block_have_real_vanilla_load_evidence() {
        // #231/#232: these are the two triggers whose filtering was fixed by
        // AdvancementSchemaFamily-aware rendering. Lock in that the coverage
        // table records real vanilla server evidence for it (not merely a
        // golden JSON assertion) — see sand-vanilla-audit's
        // `audit_placed_block_filtered`/`audit_item_used_on_block_filtered`.
        for trigger_id in ["minecraft:placed_block", "minecraft:item_used_on_block"] {
            let entry = find_coverage(trigger_id).unwrap();
            assert!(
                entry.vanilla_load_tested_profiles.contains(&"26.2"),
                "{trigger_id} should record 26.2 vanilla-load evidence"
            );
            assert_eq!(entry.semantic_runtime_tested_profiles, &["1.21.4"]);
        }
    }

    #[test]
    fn no_duplicate_trigger_ids() {
        let mut seen = std::collections::HashSet::new();
        for entry in TRIGGER_COVERAGE {
            assert!(
                seen.insert(entry.trigger_id),
                "duplicate trigger_id in coverage table: '{}'",
                entry.trigger_id
            );
        }
    }

    #[test]
    fn coverage_matches_verified_vanilla_trigger_registry_union() {
        let stable: Vec<&str> =
            serde_json::from_str(include_str!("../../fixtures/trigger-coverage/1.21.4.json"))
                .unwrap();
        let latest: Vec<&str> =
            serde_json::from_str(include_str!("../../fixtures/trigger-coverage/26.2.json"))
                .unwrap();
        let registry_union = stable
            .iter()
            .chain(&latest)
            .copied()
            .collect::<std::collections::HashSet<_>>();

        for trigger_id in &registry_union {
            assert!(
                find_coverage(trigger_id).is_some(),
                "verified vanilla trigger `{trigger_id}` is missing from TRIGGER_COVERAGE"
            );
        }
        for entry in TRIGGER_COVERAGE {
            match entry.api_status {
                TriggerApiStatus::Typed => {
                    assert!(stable.contains(&entry.trigger_id));
                    assert!(latest.contains(&entry.trigger_id));
                }
                TriggerApiStatus::IntentionallyUnsupported => assert!(
                    !registry_union.contains(entry.trigger_id),
                    "unsupported source-compatibility trigger unexpectedly exists in a verified registry: {}",
                    entry.trigger_id
                ),
                _ => {}
            }
        }
    }

    #[test]
    fn typed_triggers_have_golden_tests_flag() {
        for entry in TRIGGER_COVERAGE {
            if matches!(entry.api_status, TriggerApiStatus::Typed) {
                assert!(
                    entry.golden_json_tested,
                    "trigger '{}' is typed but golden_json_tested is false",
                    entry.trigger_id
                );
            }
        }
    }

    #[test]
    fn coverage_table_is_stable() {
        // Snapshot the count so adding a trigger without updating the table causes a failure.
        assert_eq!(
            TRIGGER_COVERAGE.len(),
            64,
            "trigger coverage table size changed — update this count when adding/removing triggers"
        );
    }

    #[test]
    fn coverage_status_counts_are_explicit() {
        let count = |status| {
            TRIGGER_COVERAGE
                .iter()
                .filter(|entry| entry.api_status == status)
                .count()
        };
        assert_eq!(count(TriggerApiStatus::Typed), 50);
        assert_eq!(count(TriggerApiStatus::Missing), 8);
        assert_eq!(count(TriggerApiStatus::IntentionallyUnsupported), 5);
        assert_eq!(count(TriggerApiStatus::VersionGated), 1);
    }

    #[test]
    fn known_invalid_leveled_up_trigger_has_a_migration_diagnostic() {
        let metadata = trigger_metadata("minecraft:leveled_up");
        assert!(!metadata.supported);
        assert!(metadata.diagnostic.unwrap().contains("experience query"));
        let coverage = TRIGGER_COVERAGE
            .iter()
            .find(|entry| entry.trigger_id == "minecraft:leveled_up")
            .unwrap();
        assert!(matches!(
            coverage.api_status,
            TriggerApiStatus::IntentionallyUnsupported
        ));
    }
}
