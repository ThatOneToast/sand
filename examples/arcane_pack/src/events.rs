use sand_core::ItemPredicate;
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

    /// Declares that this event depends on the MANA scoreboard variable.
    ///
    /// Calling `Event::<AteGoldenAppleEvent>::state_init()` in load will
    /// include `MANA.define()` automatically.
    fn state_defines() -> Vec<String> {
        vec![super::MANA.define()]
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
        UsingItemTrigger::new()
            .item(ItemPredicate::id("minecraft:stick").custom_data_key("arcane_wand"))
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
