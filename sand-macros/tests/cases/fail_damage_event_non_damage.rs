use sand_core::prelude::*;
use sand_macros::on_event;

pub struct AteEvent;

impl AdvancementEvent for AteEvent {
    type Trigger = sand_core::event::trigger::ConsumeItemTrigger;

    fn trigger() -> Self::Trigger {
        sand_core::event::trigger::ConsumeItemTrigger::new()
    }
}

#[on_event]
pub fn bad(event: DamageEvent<AteEvent>) {
    event.reflect_damage().run();
}

fn main() {}
