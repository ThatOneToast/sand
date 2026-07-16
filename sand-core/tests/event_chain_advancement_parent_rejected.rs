//! An advancement-backed `SandEvent` parent is not yet supported by
//! same-cycle chained dispatch (#240, deferred) — the export pipeline must
//! reject it with a clear diagnostic rather than silently generating an
//! approximation. Isolated in its own test binary since `inventory`
//! registrations are process-global.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SameCycleEventDependency, SameCycleEventRequirement,
    TickEventDispatch,
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

struct AdvancementParent;

fn advancement_parent_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
}
fn advancement_parent_type_id() -> TypeId {
    TypeId::of::<AdvancementParent>()
}
fn advancement_parent_type_name() -> &'static str {
    std::any::type_name::<AdvancementParent>()
}

struct ChildOfAdvancementParent;

fn child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: vec![SameCycleEventRequirement::After(SameCycleEventDependency {
            event_type_id: advancement_parent_type_id,
            event_type_name: advancement_parent_type_name,
            event_dispatch: advancement_parent_dispatch,
            event_setup: EventSetup::none,
        })],
        persistent: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn child_type_id() -> TypeId {
    TypeId::of::<ChildOfAdvancementParent>()
}
fn child_type_name() -> &'static str {
    std::any::type_name::<ChildOfAdvancementParent>()
}
fn on_child_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_child_of_advancement_parent",
        id_override: None,
        make: on_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: child_chain,
            revoke: revoke_true,
            event_type_id: child_type_id,
            event_type_name: child_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn chaining_from_an_advancement_backed_parent_is_rejected() {
    let error = sand_core::try_export_components_json("advparentpack")
        .expect_err("export must fail: advancement-backed parents are deferred (#240)");
    let message = error.to_string();
    assert!(
        message.contains("player execution context"),
        "error must explain the player-context limitation: {message}"
    );
    assert!(
        message.contains("ChildOfAdvancementParent") && message.contains("AdvancementParent"),
        "error must name both the child and the unsupported parent: {message}"
    );
}
