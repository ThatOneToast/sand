use sand_core::ItemPredicate;
use sand_core::event::trigger::{ConsumeItemTrigger, UsingItemTrigger};
use sand_core::event::{AdvancementEvent, EventPlayer};
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

impl EventPlayer for AteGoldenAppleEvent {
    fn player(&self) -> Selector {
        Selector::self_()
    }
}

/// Fires when a player uses a dash wand (stick with custom data) while eligible.
pub struct UsedDashWandEvent;

impl AdvancementEvent for UsedDashWandEvent {
    type Trigger = UsingItemTrigger;

    fn trigger() -> Self::Trigger {
        // Custom data matching requires raw JSON escape hatch — arcane_wand:1b predicate
        UsingItemTrigger::new().item(serde_json::json!({
            "items": "minecraft:stick",
            "predicates": {
                "minecraft:custom_data": "{arcane_wand:1b}"
            }
        }))
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

impl EventPlayer for UsedDashWandEvent {
    fn player(&self) -> Selector {
        Selector::self_()
    }
}
