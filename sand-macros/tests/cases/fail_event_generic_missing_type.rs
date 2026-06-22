use sand_core::prelude::*;
use sand_macros::event;

#[event]
pub fn bad(event: Event) {
    let _ = event;
}

fn main() {}
