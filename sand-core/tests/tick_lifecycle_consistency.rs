//! A malformed manual descriptor must not silently inherit another handler's
//! lifecycle definition when both claim the same SandEvent type.

use sand_core::events::{EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct SharedEvent;

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}

fn no_condition() -> Option<String> {
    None
}
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}

fn dispatch() -> Option<TickEventDispatch> {
    Some(TickEventDispatch::default().as_players().every_tick())
}

fn shared_type_id() -> TypeId {
    TypeId::of::<SharedEvent>()
}

fn shared_type_name() -> &'static str {
    "SharedEvent"
}

fn first_setup() -> EventSetup {
    EventSetup::none()
}

fn conflicting_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add conflicting dummy".to_string()],
        ..EventSetup::none()
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "first_shared_event_handler",
        id_override: None,
        make: || Vec::new(),
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: || true,
            event_type_id: shared_type_id,
            event_type_name: shared_type_name,
            make_setup: first_setup,
        },
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "conflicting_shared_event_handler",
        id_override: None,
        make: || Vec::new(),
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: || true,
            event_type_id: shared_type_id,
            event_type_name: shared_type_name,
            make_setup: conflicting_setup,
        },
    }
}

#[test]
fn conflicting_grouped_lifecycle_definitions_fail_export() {
    let error = sand_core::try_export_components("consistency")
        .expect_err("conflicting lifecycle definitions must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("conflicting SandEvent definitions"),
        "got: {message}"
    );
    assert!(message.contains("SharedEvent"), "got: {message}");
    assert!(
        message.contains("first_shared_event_handler")
            && message.contains("conflicting_shared_event_handler"),
        "got: {message}"
    );
    assert!(
        message.contains("setup()"),
        "error should say setup() differed, not dispatch(): {message}"
    );
}
