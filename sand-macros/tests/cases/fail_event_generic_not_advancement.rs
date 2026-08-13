use sand_core::prelude::*;
use sand_macros::on_event;

pub struct NotAnAdvancementEvent;

#[on_event]
pub fn bad(event: Event<NotAnAdvancementEvent>) {
    let _ = event.player();
}

fn main() {}
