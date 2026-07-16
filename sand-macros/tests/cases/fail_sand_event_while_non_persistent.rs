#![allow(refining_impl_trait)]

use sand_core::events::{SandEvent, SandEventDispatch};

struct Parent;
impl SandEvent for Parent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct OccurrenceOnly;
impl SandEvent for OccurrenceOnly {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

fn main() {
    let _ = SandEventDispatch::chain::<Parent>().while_::<OccurrenceOnly>();
}
