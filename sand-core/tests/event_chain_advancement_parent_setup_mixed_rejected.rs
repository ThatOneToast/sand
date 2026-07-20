//! An advancement-backed graph parent that declares more than one non-empty
//! setup category is rejected deterministically (#240 Phase 6): the
//! diagnostic always names the first non-empty category in lifecycle order
//! (`objectives`, then `pre_observation`, then `post_observation`) via
//! `EventSetup::first_non_empty_category`, independent of registration
//! order. Isolated in its own test binary since `inventory` registrations
//! are process-global.

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
        // Both pre_observation and post_observation are non-empty
        // (objectives left empty), proving the diagnostic still picks
        // exactly one deterministic category — the first in lifecycle
        // order — rather than concatenating or non-deterministically
        // choosing among several non-empty categories.
        EventSetup {
            objectives: vec![],
            pre_observation: vec!["scoreboard players set @s p6_pre 1".into()],
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
fn advancement_parent_with_mixed_setup_picks_the_first_lifecycle_category_deterministically() {
    // Two independent export calls exercise the same discovery path twice
    // (fresh BTreeMap-backed graph state each time); both must select the
    // identical diagnostic, proving the choice is a pure function of the
    // parent's own EventSetup value rather than any iteration/registration
    // order artifact.
    let first = sand_core::try_export_components_json("advsetupmixedrejected")
        .expect_err("export must fail: mixed non-empty setup is unsupported (#240 Phase 6)")
        .to_string();
    let second = sand_core::try_export_components_json("advsetupmixedrejected")
        .expect_err("export must fail: mixed non-empty setup is unsupported (#240 Phase 6)")
        .to_string();
    assert_eq!(
        first, second,
        "the selected diagnostic must be deterministic across repeated export calls"
    );

    assert!(
        first.contains("Child") && first.contains("AdvancementParent"),
        "error must name both the child and the advancement parent: {first}"
    );
    assert!(
        first.contains("pre_observation"),
        "pre_observation (checked before post_observation) must be the named category: {first}"
    );
    assert!(
        !first.contains("`post_observation`"),
        "only one category should be named even though two are non-empty: {first}"
    );
}
