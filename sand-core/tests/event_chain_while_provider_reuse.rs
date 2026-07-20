//! A persistent provider with a direct handler keeps one occurrence detector;
//! `while_` still queries its live condition without invoking that detector.

use sand_core::events::{
    ChainEventDispatch, EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch,
    TickEventDispatch,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct Parent;
impl SandEvent for Parent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}

struct Child;
impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Parent>().while_::<PlayerSneakEvent>()
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
fn no_chain() -> Option<ChainEventDispatch> {
    None
}
fn sneaking_condition() -> Option<String> {
    let SandEventDispatch::TickCondition(condition) = PlayerSneakEvent::dispatch() else {
        unreachable!("PlayerSneakEvent remains a condition event")
    };
    Some(condition)
}
fn child_chain() -> Option<ChainEventDispatch> {
    let SandEventDispatch::Chain(chain) = Child::dispatch().into() else {
        unreachable!("Child remains chained")
    };
    Some(chain)
}
fn setup() -> EventSetup {
    EventSetup::none()
}
fn revoke() -> bool {
    true
}
fn sneaking_id() -> TypeId {
    TypeId::of::<PlayerSneakEvent>()
}
fn sneaking_name() -> &'static str {
    std::any::type_name::<PlayerSneakEvent>()
}
fn child_id() -> TypeId {
    TypeId::of::<Child>()
}
fn child_name() -> &'static str {
    std::any::type_name::<Child>()
}
fn sneaking_body() -> Vec<String> {
    vec!["say direct sneaking handler".into()]
}
fn child_body() -> Vec<String> {
    vec!["say parent while sneaking".into()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_sneaking_direct",
        id_override: None,
        make: sneaking_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: sneaking_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: || None,
            revoke,
            event_type_id: sneaking_id,
            event_type_name: sneaking_name,
            make_setup: setup,
        },
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_child",
        id_override: None,
        make: child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: child_chain,
            make_tracked: || None,
            revoke,
            event_type_id: child_id,
            event_type_name: child_name,
            make_setup: setup,
        },
    }
}

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

#[test]
fn provider_detector_and_predicate_are_reused_without_cross_dispatch() {
    let json = sand_core::try_export_components_json("reusepack").expect("export succeeds");
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).expect("valid JSON");
    let provider_key = key(std::any::type_name::<PlayerSneakEvent>());
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["dir"] == "function"
                    && record["path"] == format!("__sand_event_check/{provider_key}")
            })
            .count(),
        1
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| {
                record["dir"] == "predicate" && record["path"] == "__sand/player_sneaking"
            })
            .count(),
        1
    );

    let parent_key = key(std::any::type_name::<Parent>());
    let parent_dispatch = records
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"] == format!("__sand_event_dispatch/{parent_key}")
        })
        .and_then(|record| record["content"].as_str())
        .expect("parent dispatch function");
    assert!(parent_dispatch.contains("if predicate reusepack:__sand/player_sneaking"));
    assert!(!parent_dispatch.contains(&format!("__sand_event_dispatch/{provider_key}")));
}
