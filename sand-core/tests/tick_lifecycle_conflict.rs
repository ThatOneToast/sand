//! Integration coverage: two `#[event]`-style handlers that claim to
//! subscribe to the same concrete `SandEvent` type must not silently group
//! together when their `dispatch()`/`setup()` results actually differ.
//!
//! This mirrors `transition_conflict.rs`'s pattern of submitting raw,
//! deliberately-conflicting `EventDescriptor`s and asserting the whole export
//! fails with a clear diagnostic — this must live in its own test binary
//! (separate from `tick_lifecycle_export.rs`) because the conflict here is
//! permanent process-global inventory state that would otherwise fail every
//! other export test sharing the same binary.

use sand_core::condition::Condition;
use sand_core::events::{EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct ConflictingEvent;

fn conflicting_event_type_id() -> TypeId {
    TypeId::of::<ConflictingEvent>()
}
fn conflicting_event_type_name() -> &'static str {
    std::any::type_name::<ConflictingEvent>()
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}

fn dispatch_a() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s a matches 1")),
    )
}

fn dispatch_b() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s b matches 1")),
    )
}

fn empty_setup() -> EventSetup {
    EventSetup::none()
}

fn handler_a_body() -> Vec<String> {
    vec!["say a".to_string()]
}
fn handler_b_body() -> Vec<String> {
    vec!["say b".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "conflicting_handler_a",
        id_override: None,
        make: handler_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: dispatch_a,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: conflicting_event_type_id,
            event_type_name: conflicting_event_type_name,
            make_setup: empty_setup,
        },
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "conflicting_handler_b",
        id_override: None,
        make: handler_b_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: dispatch_b,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: conflicting_event_type_id,
            event_type_name: conflicting_event_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn conflicting_dispatch_definitions_for_the_same_event_type_fail_export() {
    let error =
        sand_core::try_export_components_json("conflictpack").expect_err("export must fail");
    let message = error.to_string();
    assert!(
        message.contains("conflicting_handler_a") && message.contains("conflicting_handler_b"),
        "error must name both conflicting handler paths: {message}"
    );
    assert!(
        message.contains(conflicting_event_type_name()) || message.contains("ConflictingEvent"),
        "error must name the canonical event type identity: {message}"
    );
    assert!(
        message.contains("dispatch()"),
        "error must say which part of the definition differed: {message}"
    );
}
