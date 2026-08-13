use sand_core::prelude::*;
use sand_macros::on_event;

#[on_event]
pub fn bad(event: Event) {
    let _ = event;
}

fn main() {}
