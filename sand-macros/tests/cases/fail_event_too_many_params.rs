use sand_core::event::trigger::TickTrigger;
use sand_core::prelude::*;
use sand_macros::event;

pub struct TickEvent;

impl AdvancementEvent for TickEvent {
    type Trigger = TickTrigger;

    fn trigger() -> Self::Trigger {
        TickTrigger::new()
    }
}

#[event]
pub fn bad(event: Event<TickEvent>, other: Event<TickEvent>) {
    let _ = (event, other);
}

fn main() {}
