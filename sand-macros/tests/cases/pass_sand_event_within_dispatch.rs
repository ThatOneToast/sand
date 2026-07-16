#![allow(refining_impl_trait)]

use sand_core::events::{PlayerSneakEvent, SandEvent, SandEventDispatch, TickWindow};
use sand_core::prelude::*;
use sand_macros::event;

struct CurrentEvent;
impl SandEvent for CurrentEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct PriorEvent;
impl SandEvent for PriorEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct OtherEvent;
impl SandEvent for OtherEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct BoundedChild;
impl SandEvent for BoundedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<PriorEvent>(TickWindow::new(20).expect("nonzero, in range"))
    }
}

struct BoundedWithWhileChild;
impl SandEvent for BoundedWithWhileChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<CurrentEvent>()
            .within::<PriorEvent>(TickWindow::new(5).expect("nonzero, in range"))
            .while_::<PlayerSneakEvent>()
    }
}

struct BoundedAfterAnyChild;
impl SandEvent for BoundedAfterAnyChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(CurrentEvent, OtherEvent)>()
            .within::<PriorEvent>(TickWindow::new(10).expect("nonzero, in range"))
    }
}

#[event]
fn on_bounded(_event: BoundedChild) {
    cmd::say("bounded");
}

#[event]
fn on_bounded_with_while(_event: BoundedWithWhileChild) {
    cmd::say("bounded_with_while");
}

#[event]
fn on_bounded_after_any(_event: BoundedAfterAnyChild) {
    cmd::say("bounded_after_any");
}

fn main() {
    assert!(!on_bounded().is_empty());
    assert!(!on_bounded_with_while().is_empty());
    assert!(!on_bounded_after_any().is_empty());
}
