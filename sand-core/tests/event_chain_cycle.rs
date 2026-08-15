//! Integration coverage for same-cycle chained `SandEvent` cycle/scope
//! diagnostics (#240), through the *real* export pipeline. Isolated in its
//! own test binary — like `tick_lifecycle_conflict.rs` — because `inventory`
//! registrations are process-global and a rejected export must not pollute
//! other export tests in this crate.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, TickEventDispatch,
};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_trigger() -> Option<AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<TickEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
}

// ── Indirect cycle: A -> B -> C -> A ────────────────────────────────────────

struct CycleA;
struct CycleB;
struct CycleC;

fn a_dispatch() -> sand_core::events::SandEventDispatch {
    SandEventDispatch::chain::<CycleC>().into()
}
fn b_dispatch() -> sand_core::events::SandEventDispatch {
    SandEventDispatch::chain::<CycleA>().into()
}
fn c_dispatch() -> sand_core::events::SandEventDispatch {
    SandEventDispatch::chain::<CycleB>().into()
}

impl SandEvent for CycleA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        a_dispatch()
    }
}
impl SandEvent for CycleB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        b_dispatch()
    }
}
impl SandEvent for CycleC {
    fn dispatch() -> impl Into<SandEventDispatch> {
        c_dispatch()
    }
}

fn a_chain() -> Option<ChainEventDispatch> {
    match a_dispatch() {
        sand_core::events::SandEventDispatch::Chain(c) => Some(c),
        _ => None,
    }
}
fn a_type_id() -> TypeId {
    TypeId::of::<CycleA>()
}
fn a_type_name() -> &'static str {
    std::any::type_name::<CycleA>()
}
fn on_cycle_a_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_cycle_a",
        id_override: None,
        make: on_cycle_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: a_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: a_type_id,
            event_type_name: a_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn indirect_dependency_cycle_is_rejected_with_readable_path() {
    let error = sand_core::try_export_components_json("cyclepack")
        .expect_err("export must fail on a chain dependency cycle");
    let message = error.to_string();
    assert!(
        message.contains("dependency cycle"),
        "error must name the cycle: {message}"
    );
    assert!(
        message.contains("CycleA") && message.contains("CycleB") && message.contains("CycleC"),
        "error must name every event in the cycle: {message}"
    );
}
