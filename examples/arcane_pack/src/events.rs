use sand_core::event::trigger::{ConsumeItemTrigger, UsingItemTrigger};
use sand_core::event::{AdvancementEvent, EventPlayer};
use sand_core::prelude::*;

/// Fires when a player eats a golden apple with mana below max.
pub struct AteGoldenAppleEvent;

impl AdvancementEvent for AteGoldenAppleEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(serde_json::json!({"items": "minecraft:golden_apple"}))
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
        UsingItemTrigger::new().item(serde_json::json!({
            "items": "minecraft:stick",
            "predicates": {
                "minecraft:custom_data": "{arcane_wand:1b}"
            }
        }))
    }

    fn guard() -> Option<Condition> {
        Some(all![
            super::MANA.of("@s").gte(25),
            super::DASH.ready("@s"),
            super::SHIELD.of("@s").is_false(),
        ])
    }
}

impl EventPlayer for UsedDashWandEvent {
    fn player(&self) -> Selector {
        Selector::self_()
    }
}
