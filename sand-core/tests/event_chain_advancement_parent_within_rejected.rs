//! `.within::<AdvancementParent>(...)` is rejected (#240 Phase 6): bounded
//! correlation requires the tick coordinator to maintain a per-tick age
//! counter, which an advancement reward's synchronous, coordinator-external
//! execution cannot safely participate in. Isolated in its own test binary
//! since `inventory` registrations are process-global.

use sand_core::AdvancementTrigger;
use sand_core::events::{SandEvent, SandEventDispatch, TickWindow};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct AdvancementParent;
impl SandEvent for AdvancementParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }
}

struct TickParent;
impl SandEvent for TickParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct Child;
impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<TickParent>()
            .within::<AdvancementParent>(TickWindow::new(5).unwrap())
    }
}

fn no_trigger() -> Option<AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<sand_core::events::TickEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> sand_core::events::EventSetup {
    sand_core::events::EventSetup::none()
}

fn child_chain() -> Option<sand_core::events::ChainEventDispatch> {
    match Child::dispatch().into() {
        SandEventDispatch::Chain(chain) => Some(chain),
        _ => None,
    }
}
fn child_type_id() -> TypeId {
    TypeId::of::<Child>()
}
fn child_type_name() -> &'static str {
    std::any::type_name::<Child>()
}
fn on_child_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_child",
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
fn within_on_advancement_parent_is_rejected() {
    let error = sand_core::try_export_components_json("advwithinrejected").expect_err(
        "export must fail: within() on an advancement-backed parent is unsupported (#240 Phase 6)",
    );
    let message = error.to_string();
    assert!(
        message.contains("within"),
        "error must name the unsupported operator: {message}"
    );
    assert!(
        message.contains("Child") && message.contains("AdvancementParent"),
        "error must name both the child and the advancement parent: {message}"
    );
    assert!(
        message.contains("coordinator"),
        "error must explain the coordinator-maintenance mismatch: {message}"
    );
}
