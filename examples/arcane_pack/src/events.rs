use sand_core::{ItemPredicate, RawJson};
use sand_core::event::trigger::{ConsumeItemTrigger, UsingItemTrigger};
use sand_core::event::{AdvancementEvent, DamageAdvancementEvent};
use sand_core::prelude::*;

/// Fires when a player eats a golden apple with mana below max.
pub struct AteGoldenAppleEvent;

impl AdvancementEvent for AteGoldenAppleEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        // Typed predicate — no raw serde_json
        ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
    }

    fn guard() -> Option<Condition> {
        Some(super::MANA.of("@s").lt(100))
    }
}

/// Fires when an enhanced-cells player is damaged.
pub struct EnhancedCellsDamagedEvent;

impl AdvancementEvent for EnhancedCellsDamagedEvent {
    type Trigger = sand_core::AdvancementTrigger;

    fn trigger() -> Self::Trigger {
        sand_core::AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        }
    }

    fn guard() -> Option<Condition> {
        Some(super::HAS_ENHANCED_CELLS.of("@s").is_true())
    }
}

impl DamageAdvancementEvent for EnhancedCellsDamagedEvent {}

/// Fires when a player uses a dash wand (stick with custom data) while eligible.
pub struct UsedDashWandEvent;

impl AdvancementEvent for UsedDashWandEvent {
    type Trigger = UsingItemTrigger;

    fn trigger() -> Self::Trigger {
        // Custom data matching uses typed item + raw SNBT predicate escape hatch
        UsingItemTrigger::new().item(
            ItemPredicate::id("minecraft:stick")
                .raw_predicates(RawJson::new(serde_json::json!({
                    "minecraft:custom_data": "{arcane_wand:1b}"
                }))),
        )
    }

    fn guard() -> Option<Condition> {
        // Fluent chaining: mana AND dash ready AND shield not active
        Some(
            super::MANA
                .of("@s")
                .gte(25)
                .and(super::DASH.ready("@s"))
                .and_not(super::SHIELD.of("@s").is_true()),
        )
    }
}
