//! Canonical event identity collision diagnostics for graph export.

use sand_core::events::{ChainEventDispatch, EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct First;
struct Second;

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn tick() -> Option<TickEventDispatch> {
    Some(TickEventDispatch::default().as_players())
}
fn no_chain() -> Option<ChainEventDispatch> {
    None
}
fn setup() -> EventSetup {
    EventSetup::none()
}
fn revoke() -> bool {
    true
}
fn first_id() -> TypeId {
    TypeId::of::<First>()
}
fn second_id() -> TypeId {
    TypeId::of::<Second>()
}
fn colliding_name() -> &'static str {
    "collision::SameCanonicalEvent"
}
fn first_body() -> Vec<String> {
    vec!["say first".into()]
}
fn second_body() -> Vec<String> {
    vec!["say second".into()]
}

macro_rules! submit_collision {
    ($path:literal, $make:ident, $type_id:ident) => {
        sand_core::inventory::submit! {
            EventDescriptor {
                path: $path,
                id_override: None,
                make: $make,
                dispatch: EventDispatch::Custom {
                    make_trigger: no_trigger,
                    make_condition: no_condition,
                    make_tick: tick,
                    make_chain: no_chain,
                    revoke,
                    event_type_id: $type_id,
                    event_type_name: colliding_name,
                    make_setup: setup,
                },
            }
        }
    };
}

submit_collision!("first", first_body, first_id);
submit_collision!("second", second_body, second_id);

#[test]
fn canonical_name_collision_is_rejected_before_nodes_can_overwrite() {
    let error = sand_core::try_export_components_json("collisionpack")
        .expect_err("distinct TypeIds with one canonical name must fail");
    let message = error.to_string();
    assert!(message.contains("canonical event identity collision"));
    assert!(message.contains("collision::SameCanonicalEvent"));
}
