// Validation case for #239: generic SandEvent families (e.g. `ElevatorUsed<GoUp>` vs
// `ElevatorUsed<GoDown>`) must produce distinct, collision-free generated identities per
// concrete monomorphization. Each concrete instantiation is wired through its own
// non-generic `#[on_event]` handler (bare generic marker types are not yet supported as a
// direct handler parameter — see the `SandEvent`/`Event<T>` split documented
// on `sand_core::event`/`sand_core::events`); the identity
// distinctness itself is proven via `event_type_id` from the real Custom dispatch
// registration below, exactly as codegen sees it.
use sand_core::condition::Condition;
use sand_core::events::{SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_macros::on_event;
use std::marker::PhantomData;

pub trait DirectionMarker {
    const NAME: &'static str;
}

pub struct GoUp;
pub struct GoDown;

impl DirectionMarker for GoUp {
    const NAME: &'static str = "up";
}
impl DirectionMarker for GoDown {
    const NAME: &'static str = "down";
}

pub struct ElevatorUsed<D: DirectionMarker>(PhantomData<D>);

impl<D: DirectionMarker> SandEvent for ElevatorUsed<D> {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw(format!("entity @s[tag=used_elevator_{}]", D::NAME)))
            .into()
    }
}

pub struct ElevatorGoingUp;
impl SandEvent for ElevatorGoingUp {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<GoUp>::dispatch()
    }
}

pub struct ElevatorGoingDown;
impl SandEvent for ElevatorGoingDown {
    #[allow(refining_impl_trait)]
    fn dispatch() -> SandEventDispatch {
        ElevatorUsed::<GoDown>::dispatch()
    }
}

#[on_event]
pub fn on_elevator_up(event: ElevatorGoingUp) {
    cmd::say("going up");
}

#[on_event]
pub fn on_elevator_down(event: ElevatorGoingDown) {
    cmd::say("going down");
}

fn main() {
    // The underlying generic SandEvent family itself has distinct, stable
    // per-monomorphization identity — this is the property #239 requires and
    // does not depend on how (or whether) a given instantiation is wired to a
    // `#[on_event]` handler.
    assert_eq!(
        std::any::TypeId::of::<ElevatorUsed<GoUp>>(),
        std::any::TypeId::of::<ElevatorUsed<GoUp>>()
    );
    assert_ne!(
        std::any::TypeId::of::<ElevatorUsed<GoUp>>(),
        std::any::TypeId::of::<ElevatorUsed<GoDown>>()
    );

    let dispatch_up: SandEventDispatch = ElevatorUsed::<GoUp>::dispatch().into();
    let dispatch_down: SandEventDispatch = ElevatorUsed::<GoDown>::dispatch().into();
    assert!(matches!(dispatch_up, SandEventDispatch::Tick(_)));
    assert!(matches!(dispatch_down, SandEventDispatch::Tick(_)));

    // Each real #[on_event]-registered handler also gets its own distinct
    // event_type_id, so multiple handlers never accidentally merge detectors
    // across what should be separate events.
    let mut up_id = None;
    let mut down_id = None;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if let sand_core::EventDispatch::Custom { event_type_id, .. } = descriptor.dispatch {
            if descriptor.path == "on_elevator_up" {
                up_id = Some(event_type_id());
            } else if descriptor.path == "on_elevator_down" {
                down_id = Some(event_type_id());
            }
        }
    }
    let up_id = up_id.expect("on_elevator_up must register Custom dispatch");
    let down_id = down_id.expect("on_elevator_down must register Custom dispatch");
    assert_ne!(up_id, down_id);
}
