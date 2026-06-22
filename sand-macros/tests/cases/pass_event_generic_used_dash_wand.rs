use sand_core::event::trigger::UsingItemTrigger;
use sand_core::prelude::*;
use sand_macros::event;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));
static SHIELD: Flag = Flag::new("shield");

pub struct UsedDashWandEvent;

impl AdvancementEvent for UsedDashWandEvent {
    type Trigger = UsingItemTrigger;

    fn trigger() -> Self::Trigger {
        UsingItemTrigger::new().item(ItemPredicate::id("minecraft:stick"))
    }

    fn guard() -> Option<Condition> {
        Some(
            MANA.of("@s")
                .gte(25)
                .and(DASH.ready("@s"))
                .and_not(SHIELD.of("@s").is_true()),
        )
    }
}

#[event]
pub fn on_used_dash_wand(event: Event<UsedDashWandEvent>) {
    MANA.remove(event.player(), 25);
    DASH.start(event.player());
}

fn main() {
    let mut found = false;

    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if descriptor.path == "on_used_dash_wand" {
            match descriptor.dispatch {
                sand_core::EventDispatch::Advancement { guard, .. } => {
                    let guard = guard.expect("Event<T> guard must be typed Condition guard");
                    let commands = guard()
                        .expect("dash wand guard should be present")
                        .execute_commands(true, "return 0");
                    assert!(commands
                        .iter()
                        .any(|cmd| cmd.contains("score @s mana matches 25..")));
                    assert!(commands
                        .iter()
                        .any(|cmd| cmd.contains("score @s dash matches 0")));
                    assert!(commands
                        .iter()
                        .any(|cmd| cmd.contains("score @s shield matches 1")));
                    found = true;
                }
                _ => panic!("Event<UsedDashWandEvent> must not use legacy string conditions"),
            }
        }
    }

    assert!(found);
}
