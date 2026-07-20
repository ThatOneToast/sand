//! An advancement-backed graph parent that declares `post_observation`
//! setup is rejected (#240 Phase 6) — see
//! `event_chain_advancement_parent_setup_objectives_rejected.rs` for the
//! full rationale. Isolated in its own test binary since `inventory`
//! registrations are process-global.

use sand_core::AdvancementTrigger;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct AdvancementParent;
impl SandEvent for AdvancementParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![],
            pre_observation: vec![],
            post_observation: vec!["scoreboard players set @s p6_post 1".into()],
        }
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
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
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
fn advancement_parent_with_post_observation_is_rejected() {
    let error = sand_core::try_export_components_json("advsetuppostrejected").expect_err(
        "export must fail: an advancement bridge parent with non-empty setup is unsupported (#240 Phase 6)",
    );
    let message = error.to_string();
    assert!(
        message.contains("Child") && message.contains("AdvancementParent"),
        "error must name both the child and the advancement parent: {message}"
    );
    assert!(
        message.contains("post_observation"),
        "error must identify the non-empty setup category: {message}"
    );
    assert!(
        message.contains("do not execute parent lifecycle setup"),
        "error must explain why the setup is unsupported: {message}"
    );
}
