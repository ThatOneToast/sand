use sand_core::event::trigger::ConsumeItemTrigger;
use sand_core::prelude::*;
use sand_macros::{event, function};

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

pub struct AteGoldenAppleEvent;

impl AdvancementEvent for AteGoldenAppleEvent {
    type Trigger = ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
    }

    fn guard() -> Option<Condition> {
        Some(MANA.of("@s").lt(100))
    }
}

#[function]
pub fn golden_apple_reward() {
    cmd::say("reward");
}

#[event]
pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
    MANA.add(event.player(), 10);
    cmd::call(golden_apple_reward);
}

fn main() {
    let commands = on_ate_golden_apple();
    assert!(commands.iter().any(|cmd| cmd.contains("scoreboard players add")));
    assert!(commands
        .iter()
        .any(|cmd| cmd.contains("function __sand_local:golden_apple_reward")));

    let mut found_advancement_dispatch = false;
    let mut found_event_path = false;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if descriptor.path == "on_ate_golden_apple" {
            match descriptor.dispatch {
                sand_core::EventDispatch::Advancement { guard, .. } => {
                    let guard = guard.expect("typed advancement guard should be registered");
                    let condition = guard().expect("guard should return a condition");
                    let commands = condition.execute_commands(true, "return 0");
                    assert!(commands
                        .iter()
                        .any(|cmd| cmd.contains("score @s mana matches ..99")));
                    found_advancement_dispatch = true;
                }
                _ => panic!("Event<T> handler must use advancement dispatch"),
            }
        }
    }
    for entry in inventory::iter::<sand_core::EventPathEntry>() {
        if entry.type_id == std::any::TypeId::of::<AteGoldenAppleEvent>()
            && entry.path == "on_ate_golden_apple"
        {
            found_event_path = true;
        }
    }

    assert!(found_advancement_dispatch);
    assert!(found_event_path);
}
