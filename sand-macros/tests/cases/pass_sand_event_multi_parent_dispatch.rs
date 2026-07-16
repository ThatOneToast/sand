#![allow(refining_impl_trait)]

use sand_core::condition::Condition;
use sand_core::events::{SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::event;
use std::marker::PhantomData;

struct Parent<const N: u8>(PhantomData<()>);
impl<const N: u8> SandEvent for Parent<N> {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct AnyChild;
impl SandEvent for AnyChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(Parent<1>, Parent<2>, Parent<3>)>()
            .when(Condition::entity("@s[tag=ready]"))
    }
}

struct AllChild;
impl SandEvent for AllChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(
            Parent<1>, Parent<2>, Parent<3>, Parent<4>,
            Parent<5>, Parent<6>, Parent<7>, Parent<8>,
        )>()
        .unless(Condition::entity("@s[tag=blocked]"))
    }
}

#[event]
fn on_any(_event: AnyChild) {
    cmd::say("any");
}

#[event]
fn on_all(_event: AllChild) {
    cmd::say("all");
}

fn main() {
    assert!(!on_any().is_empty());
    assert!(!on_all().is_empty());
}
