// AdvancementEvent markers are stateless trigger definitions — Sand never
// constructs an instance of T, so ordinary fields declared on T are not
// runtime event data and are not reachable through the `Event<T>` handler
// context. This proves the boundary: `event.previous_jumps` does not exist,
// because `event: Event<T>` only ever exposes `Event<T>`'s own methods.
use sand_core::prelude::*;
use sand_macros::event;

pub struct JumpDelta {
    pub previous_jumps: i32,
}

impl AdvancementEvent for JumpDelta {
    type Trigger = AdvancementTrigger;
    fn trigger() -> Self::Trigger {
        AdvancementTrigger::Tick
    }
}

#[event]
pub fn on_jump_delta(event: Event<JumpDelta>) {
    // `JumpDelta::previous_jumps` is a Rust-level field on the marker type,
    // never a runtime value — Sand never instantiates `JumpDelta`, so it is
    // not accessible through `event`.
    let _ = event.previous_jumps;
}

fn main() {}
