// #240 Phase 5: `.within::<E>(window)` requires `E: SandEvent`.
use sand_core::events::{SandEvent, SandEventDispatch, TickWindow};
use sand_core::prelude::*;
use sand_macros::event;

pub struct NotASandEvent;

pub struct Current;
impl SandEvent for Current {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

pub struct Child;

impl SandEvent for Child {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::compose()
            .after::<Current>()
            .within::<NotASandEvent>(TickWindow::new(20).unwrap())
            .into()
    }
}

#[event]
pub fn on_child(event: Child) {
    cmd::say("unreachable");
}

fn main() {}
