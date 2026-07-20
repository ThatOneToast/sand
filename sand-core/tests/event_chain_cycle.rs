//! Integration coverage for same-cycle chained `SandEvent` cycle/scope
//! diagnostics (#240), through the *real* export pipeline. Isolated in its
//! own test binary — like `tick_lifecycle_conflict.rs` — because `inventory`
//! registrations are process-global and a rejected export must not pollute
//! other export tests in this crate.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SameCycleEventDependency, SameCycleEventRequirement,
    SandEventDispatch, TickEventDispatch,
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

fn after(
    event_type_id: fn() -> TypeId,
    event_type_name: fn() -> &'static str,
    event_dispatch: fn() -> SandEventDispatch,
) -> Vec<SameCycleEventRequirement> {
    vec![SameCycleEventRequirement::After(SameCycleEventDependency {
        event_type_id,
        event_type_name,
        event_dispatch,
        event_setup: EventSetup::none,
        event_revoke: || true,
    })]
}

// ── Indirect cycle: A -> B -> C -> A ────────────────────────────────────────

struct CycleA;
struct CycleB;
struct CycleC;

fn a_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::Chain(ChainEventDispatch {
        occurrence: after(
            TypeId::of::<CycleC>,
            std::any::type_name::<CycleC>,
            c_dispatch,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn b_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::Chain(ChainEventDispatch {
        occurrence: after(
            TypeId::of::<CycleA>,
            std::any::type_name::<CycleA>,
            a_dispatch,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn c_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::Chain(ChainEventDispatch {
        occurrence: after(
            TypeId::of::<CycleB>,
            std::any::type_name::<CycleB>,
            b_dispatch,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
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
