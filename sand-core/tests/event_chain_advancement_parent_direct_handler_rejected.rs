//! An advancement-backed graph parent that also has a direct `#[event]`
//! handler is rejected (#240 Phase 6): combining a direct handler with graph
//! composition on the same advancement-backed event is not yet supported —
//! it would otherwise require either duplicating the advancement grant or
//! splicing into the pre-existing per-handler advancement lowering path,
//! both out of scope for this phase. Isolated in its own test binary since
//! `inventory` registrations are process-global.

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

struct Child;
impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<AdvancementParent>()
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
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
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

// A direct #[event]-style handler on `AdvancementParent` itself, registered
// exactly as the macro would (see `EventDispatch::Custom` with
// `make_chain: no_chain`, resolving to advancement-backed dispatch).
fn advancement_parent_trigger() -> Option<AdvancementTrigger> {
    Some(AdvancementTrigger::Tick)
}
fn advancement_parent_type_id() -> TypeId {
    TypeId::of::<AdvancementParent>()
}
fn advancement_parent_type_name() -> &'static str {
    std::any::type_name::<AdvancementParent>()
}
fn on_advancement_parent_body() -> Vec<String> {
    vec!["say direct handler".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_advancement_parent_direct",
        id_override: None,
        make: on_advancement_parent_body,
        dispatch: EventDispatch::Custom {
            make_trigger: advancement_parent_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: advancement_parent_type_id,
            event_type_name: advancement_parent_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn advancement_parent_with_both_a_direct_handler_and_a_graph_child_is_rejected() {
    let error = sand_core::try_export_components_json("advdirecthandler").expect_err(
        "export must fail: a direct handler combined with graph composition is unsupported (#240 Phase 6)",
    );
    let message = error.to_string();
    assert!(
        message.contains("AdvancementParent"),
        "error must name the advancement-backed parent: {message}"
    );
    assert!(
        message.contains("direct #[event] handler"),
        "error must explain the direct-handler conflict: {message}"
    );
}
