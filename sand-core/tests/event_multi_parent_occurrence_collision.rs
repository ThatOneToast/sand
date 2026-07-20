//! Regression coverage for generated multi-parent occurrence objective
//! collisions across distinct event definitions.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, TickEventDispatch,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct ParentA;
struct ParentB;
struct MultiChild;

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

impl SandEvent for ParentA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

impl SandEvent for ParentB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }

    fn setup() -> EventSetup {
        let other_parent_key = key(std::any::type_name::<ParentA>());
        EventSetup {
            objectives: vec![format!(
                "scoreboard objectives add se_{other_parent_key}_o dummy"
            )],
            pre_observation: vec![],
            post_observation: vec![],
        }
    }
}

impl SandEvent for MultiChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(ParentA, ParentB)>()
    }
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}

fn no_condition() -> Option<String> {
    None
}

fn no_tick() -> Option<TickEventDispatch> {
    None
}

fn child_chain() -> Option<ChainEventDispatch> {
    match MultiChild::dispatch().into() {
        SandEventDispatch::Chain(chain) => Some(chain),
        _ => None,
    }
}

fn revoke_true() -> bool {
    true
}

fn child_type_id() -> TypeId {
    TypeId::of::<MultiChild>()
}

fn child_type_name() -> &'static str {
    std::any::type_name::<MultiChild>()
}

fn child_body() -> Vec<String> {
    vec!["say child".into()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_multi_child",
        id_override: None,
        make: child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: child_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: child_type_id,
            event_type_name: child_type_name,
            make_setup: EventSetup::none,
        },
    }
}

#[test]
fn another_events_setup_cannot_claim_a_generated_occurrence_objective() {
    let parent_key = key(std::any::type_name::<ParentA>());
    let objective = format!("se_{parent_key}_o");
    let error = sand_core::try_export_components_json("collisionpack")
        .expect_err("cross-node occurrence objective collision must fail export")
        .to_string();

    assert!(error.contains("generated occurrence-state identity collision"));
    assert!(error.contains(std::any::type_name::<ParentA>()));
    assert!(error.contains(std::any::type_name::<ParentB>()));
    assert!(error.contains(&objective));
}
