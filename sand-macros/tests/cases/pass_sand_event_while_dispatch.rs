#![allow(refining_impl_trait)]

use sand_core::condition::Condition;
use sand_core::events::{
    PersistentEventCondition, PersistentSandEvent, SandEvent, SandEventDispatch,
};
use sand_core::prelude::*;
use sand_macros::event;
use std::marker::PhantomData;

struct Parent;
impl SandEvent for Parent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct State<T>(PhantomData<T>);
impl<T> SandEvent for State<T> {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::entity("@s[tag=state]"))
    }
}
impl<T> PersistentSandEvent for State<T> {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(Condition::entity("@s[tag=state]"))
    }
}

struct Active;
struct Child;
impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Parent>()
            .while_::<State<Active>>()
            .when(Condition::entity("@s[tag=ready]"))
            .unless(Condition::entity("@s[tag=blocked]"))
    }
}

#[event]
fn on_child(_event: Child) {
    cmd::say("persistent composition");
}

fn main() {
    assert!(!on_child().is_empty());
}
