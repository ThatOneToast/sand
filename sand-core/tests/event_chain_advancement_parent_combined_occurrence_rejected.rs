//! An advancement-backed parent combined with a second occurrence clause
//! (`.after::<A>().after::<AdvancementB>()`) is rejected (#240 Phase 6): an
//! advancement-backed parent must be the child's sole occurrence
//! dependency. Isolated in its own test binary since `inventory`
//! registrations are process-global.

use sand_core::AdvancementTrigger;
use sand_core::events::{SandEvent, SandEventDispatch};
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
            .after::<AdvancementParent>()
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
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: child_type_id,
            event_type_name: child_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn advancement_parent_combined_with_another_occurrence_clause_is_rejected() {
    let error = sand_core::try_export_components_json("advcombinedocc").expect_err(
        "export must fail: advancement parent must be the sole occurrence clause (#240 Phase 6)",
    );
    let message = error.to_string();
    assert!(
        message.contains("sole occurrence clause"),
        "error must explain the sole-clause requirement: {message}"
    );
    assert!(
        message.contains("Child") && message.contains("AdvancementParent"),
        "error must name both the child and the advancement parent: {message}"
    );
}
