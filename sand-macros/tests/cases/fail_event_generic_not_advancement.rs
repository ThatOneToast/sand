use sand_core::prelude::*;
use sand_macros::event;

pub struct NotAnAdvancementEvent;

#[event]
pub fn bad(event: Event<NotAnAdvancementEvent>) {
    let _ = event.player();
}

fn main() {}
