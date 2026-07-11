//! Trigger coverage audit for Sand advancement triggers.
//!
//! This module provides a static, compile-time-verifiable table of every known
//! vanilla advancement trigger, the Minecraft version it was introduced,
//! and its current implementation status in Sand.
//!
//! # Purpose
//!
//! - Gives contributors a single source of truth for trigger parity.
//! - Prevents future regressions: if a trigger is marked `FullyImplemented`,
//!   it must have at least one golden JSON test.
//! - New Minecraft triggers can be added here first (as `Missing`), then
//!   promoted once the typed variant and tests are in place.
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
    /// A typed `AdvancementTrigger` variant exists, conditions are typed,
    /// and at least one golden JSON test verifies correct serialization.
    FullyImplemented,
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

// ── Coverage table ────────────────────────────────────────────────────────────

/// Static coverage table for all known vanilla advancement triggers.
///
/// Triggers are listed alphabetically by trigger ID for easier diffing.
pub const TRIGGER_COVERAGE: &[TriggerCoverage] = &[
    TriggerCoverage {
        trigger_id: "minecraft:allay_drop_item_on_block",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "Fires when an allay drops an item on a note block. AdvancementTrigger::AllayDropItemOnBlock.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:avoid_vibration",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "Fires when a player avoids triggering a sculk sensor. No conditions. AdvancementTrigger::AvoidVibration.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:bee_nest_destroyed",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::BeeNestDestroyed.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:bred_animals",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::BredAnimals. vanilla::AnimalsBreed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:brewed_potion",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::BrewedPotion. vanilla::PotionBrewed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:changed_dimension",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ChangedDimension. vanilla::DimensionChanged event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:channeled_lightning",
        since: "1.13",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ChanneledLightning.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:construct_beacon",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ConstructBeacon.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:consume_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ConsumeItem. vanilla::AnyItemConsumed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:crafted_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::CraftedItem. vanilla::AnyItemCrafted event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:cured_zombie_villager",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::CuredZombieVillager.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:effects_changed",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::EffectsChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:enchanted_item",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::EnchantedItem. vanilla::AnyItemEnchanted event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:enter_block",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::EnterBlock.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:entity_hurt_player",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::EntityHurtPlayer. vanilla::EntityDamagesPlayer event. Does NOT infer attacker.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:entity_killed_player",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::EntityKilledPlayer. vanilla::PlayerKill event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:fall_from_height",
        since: "1.18",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::FallFromHeight.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:filled_bucket",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::FilledBucket.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:fishing_rod_hooked",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::FishingRodHooked.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:hero_of_the_village",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::HeroOfTheVillage.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:impossible",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "Never fires. Used for parent-only advancements. AdvancementTrigger::Impossible.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:inventory_changed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::InventoryChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:item_durability_changed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ItemDurabilityChanged.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:item_used_on_block",
        since: "1.19.4",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "Player right-clicks a block with item. AdvancementTrigger::ItemUsedOnBlock.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:kill_mob_near_sculk_catalyst",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::KillMobNearSculkCatalyst.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:killed_by_crossbow",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::KilledByCrossbow.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:leveled_up",
        since: "1.12",
        removed_in: Some("1.12"),
        api_status: TriggerApiStatus::IntentionallyUnsupported,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: false,
        notes: "minecraft:leveled_up is not in the vanilla trigger registry. Kept only for source compatibility; generation fails with an XP polling migration diagnostic.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:lightning_strike",
        since: "1.17",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::LightningStrike.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:location",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Partial,
        golden_json_tested: true,
        notes: "AdvancementTrigger::Location. Tick-polled player-state events use this.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:nether_travel",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::NetherTravel.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:placed_block",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::PlacedBlock. vanilla::AnyBlockPlaced event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_generates_container_loot",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::PlayerGeneratesContainerLoot.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_hurt_entity",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::PlayerHurtEntity. vanilla::PlayerDamagesEntity event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_interacted_with_entity",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::PlayerInteractedWithEntity. Used by systems-entities interaction builder.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:player_killed_entity",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::PlayerKilledEntity. vanilla::EntityKill event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:recipe_unlocked",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::RecipeUnlocked.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:ride_entity_in_lava",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::RideEntityInLava.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:shot_crossbow",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ShotCrossbow. vanilla::CrossbowShot event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:slept_in_bed",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::SleptInBed.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:slide_down_block",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::SlideDownBlock.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:started_riding",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::StartedRiding. No conditions.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:summoned_entity",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::SummonedEntity. vanilla::EntitySummoned event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:tame_animal",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "AdvancementTrigger::TamedAnimal. vanilla::AnimalTamed event.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:target_hit",
        since: "1.16",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::TargetHit.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:tick",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Supported,
        golden_json_tested: true,
        notes: "Fires every tick. Used for join/first-join detection with revoke. vanilla::OnJoin, FirstJoin.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:thrown_item_picked_up",
        since: "1.15",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::ThrownItemPickedUp.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:used_ender_eye",
        since: "1.12",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::UsedEnderEye.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:used_item",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "Fires when an item finishes being used (right-click release). AdvancementTrigger::UsedItem.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:used_totem",
        since: "1.11",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::UsedTotem.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:using_item",
        since: "1.19",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::Partial,
        golden_json_tested: true,
        notes: "Fires every tick while player is actively using an item. Tick-polled state events use this.",
    },
    TriggerCoverage {
        trigger_id: "minecraft:villager_trade",
        since: "1.14",
        removed_in: None,
        api_status: TriggerApiStatus::FullyImplemented,
        event_wrapper: EventWrapperStatus::None,
        golden_json_tested: true,
        notes: "AdvancementTrigger::VillagerTrade.",
    },
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
    fn fully_implemented_triggers_have_golden_tests_flag() {
        for entry in TRIGGER_COVERAGE {
            if matches!(entry.api_status, TriggerApiStatus::FullyImplemented) {
                assert!(
                    entry.golden_json_tested,
                    "trigger '{}' is FullyImplemented but golden_json_tested is false",
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
            51,
            "trigger coverage table size changed — update this count when adding/removing triggers"
        );
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
