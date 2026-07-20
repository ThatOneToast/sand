//! Export coverage for persistent `while_<E>()` composition (#240).

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, PersistentSandEvent, PlayerSneakEvent, SandEvent,
    SandEventDispatch, TickEventDispatch,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct ParentOccurrence;

impl SandEvent for ParentOccurrence {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s parent_now > @s parent_before"))
    }
}

struct WhileSneaking;

impl SandEvent for WhileSneaking {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ParentOccurrence>()
            .while_::<PlayerSneakEvent>()
            .while_::<PlayerSneakEvent>()
            .when(Condition::entity("@s[tag=ready]"))
            .unless(Condition::entity("@s[tag=blocked]"))
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
    match WhileSneaking::dispatch().into() {
        SandEventDispatch::Chain(chain) => Some(chain),
        _ => None,
    }
}
fn child_type_id() -> TypeId {
    TypeId::of::<WhileSneaking>()
}
fn child_type_name() -> &'static str {
    std::any::type_name::<WhileSneaking>()
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
}
fn revoke_true() -> bool {
    true
}
fn child_body() -> Vec<String> {
    vec!["say parent occurrence while currently sneaking".into()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_parent_while_sneaking",
        id_override: None,
        make: child_body,
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

fn expected_key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

#[test]
fn persistent_state_is_a_distinct_live_child_condition() {
    let json = sand_core::try_export_components_json("whilepack").expect("export succeeds");
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).expect("valid export JSON");
    let parent_key = expected_key(std::any::type_name::<ParentOccurrence>());

    let parent_dispatch = records
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"] == format!("__sand_event_dispatch/{parent_key}")
        })
        .expect("parent fan-out exists");
    let parent_dispatch = parent_dispatch["content"].as_str().expect("text function");
    assert!(parent_dispatch.contains("run function whilepack:on_parent_while_sneaking"));
    assert!(parent_dispatch.contains(
        "if predicate whilepack:__sand/player_sneaking if entity @s[tag=ready] unless entity @s[tag=blocked]"
    ), "unexpected parent dispatch: {parent_dispatch}");
    assert_eq!(
        parent_dispatch
            .matches("predicate whilepack:__sand/player_sneaking")
            .count(),
        1,
        "duplicate while requirements are rendered once"
    );
    assert!(
        !parent_dispatch.contains("execute as @a"),
        "the persistent check inherits the successful parent's @s"
    );

    let parent_check = records
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"] == format!("__sand_event_check/{parent_key}")
        })
        .expect("parent detector exists");
    assert!(
        parent_check["content"]
            .as_str()
            .expect("text function")
            .contains("execute as @a at @s")
    );

    assert!(records.iter().any(|record| {
        record["dir"] == "predicate" && record["path"] == "__sand/player_sneaking"
    }));
    let sneaking_key = expected_key(std::any::type_name::<PlayerSneakEvent>());
    assert!(
        !records.iter().any(|record| {
            record["dir"] == "function"
                && (record["path"] == format!("__sand_event_check/{sneaking_key}")
                    || record["path"] == format!("__sand_event_dispatch/{sneaking_key}"))
        }),
        "persistent state does not create or invoke its occurrence detector"
    );
}

#[test]
fn persistent_generation_is_deterministic() {
    let first = sand_core::try_export_components_json("whilepack").expect("first export succeeds");
    let second =
        sand_core::try_export_components_json("whilepack").expect("second export succeeds");
    assert_eq!(first, second);
}

#[test]
fn built_in_persistent_state_is_player_scoped() {
    assert_eq!(
        PlayerSneakEvent::persistent_condition().scope(),
        sand_core::events::TickScope::Players
    );
}
