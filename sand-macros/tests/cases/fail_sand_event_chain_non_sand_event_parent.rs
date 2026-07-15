// #240: SandEventDispatch::chain::<Parent>() requires `Parent: SandEvent`.
use sand_core::events::{SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;

pub struct NotASandEvent;

pub struct Child;

impl SandEvent for Child {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::chain::<NotASandEvent>().into()
    }
}

#[event]
pub fn on_child(event: Child) {
    cmd::say("unreachable");
}

fn main() {}
