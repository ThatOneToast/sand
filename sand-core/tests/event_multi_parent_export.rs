//! Export coverage for deterministic same-cycle multi-parent composition.

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch,
    TickEventDispatch,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct ParentA;
struct ParentB;
struct ProviderOnlyParent;

impl SandEvent for ParentA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s phase4_a matches 1.."))
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add phase4_a dummy".into(),
                "scoreboard objectives add phase4_sync dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s phase4_sync = @s phase4_a"
                    .into(),
            ],
        }
    }
}
impl SandEvent for ParentB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s phase4_b matches 1.."))
    }
}
impl SandEvent for ProviderOnlyParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s phase4_c matches 1.."))
    }
}

struct AnyChild;
struct AllChild;
struct MixedChild;
struct NestedChild;
struct ImmediateIntermediate;
struct DownstreamOfImmediate;

impl SandEvent for AnyChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(ParentB, ParentA)>()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![],
            pre_observation: vec![],
            post_observation: vec!["say phase4_any_post".into()],
        }
    }
}
impl SandEvent for AllChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(ParentB, ParentA)>()
            .while_::<PlayerSneakEvent>()
            .when(Condition::entity("@s[tag=ready]"))
            .unless(Condition::entity("@s[tag=blocked]"))
    }
}
impl SandEvent for MixedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ParentA>().after_any::<(ParentB, ProviderOnlyParent)>()
    }
}
impl SandEvent for NestedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(AnyChild, ParentA)>()
    }
}
impl SandEvent for ImmediateIntermediate {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ParentA>()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![],
            pre_observation: vec![],
            post_observation: vec!["say phase4_immediate_post".into()],
        }
    }
}
impl SandEvent for DownstreamOfImmediate {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(ImmediateIntermediate, ParentB)>()
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
fn revoke_true() -> bool {
    true
}

macro_rules! submit_handler {
    ($event:ty, $path:literal, $body:literal) => {
        const _: () = {
            fn chain() -> Option<ChainEventDispatch> {
                match <$event as SandEvent>::dispatch().into() {
                    SandEventDispatch::Chain(chain) => Some(chain),
                    _ => None,
                }
            }
            fn type_id() -> TypeId {
                TypeId::of::<$event>()
            }
            fn type_name() -> &'static str {
                std::any::type_name::<$event>()
            }
            fn body() -> Vec<String> {
                vec![$body.to_string()]
            }
            fn setup() -> EventSetup {
                <$event as SandEvent>::setup()
            }
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: body,
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: no_tick,
                        make_chain: chain,
                        revoke: revoke_true,
                        event_type_id: type_id,
                        event_type_name: type_name,
                        make_setup: setup,
                    },
                }
            }
        };
    };
}

submit_handler!(AnyChild, "on_any_child", "say any");
submit_handler!(AllChild, "on_all_child", "say all");
submit_handler!(MixedChild, "on_mixed_child", "say mixed");
submit_handler!(NestedChild, "on_nested_child", "say nested");
submit_handler!(
    ImmediateIntermediate,
    "on_immediate_intermediate",
    "say immediate"
);
submit_handler!(
    DownstreamOfImmediate,
    "on_downstream_of_immediate",
    "say downstream"
);

fn parent_a_tick() -> Option<TickEventDispatch> {
    match ParentA::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
}
fn parent_a_type_id() -> TypeId {
    TypeId::of::<ParentA>()
}
fn parent_a_type_name() -> &'static str {
    std::any::type_name::<ParentA>()
}
fn parent_a_body() -> Vec<String> {
    vec!["say parent a".into()]
}
fn parent_a_setup() -> EventSetup {
    ParentA::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_parent_a",
        id_override: None,
        make: parent_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: parent_a_tick,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: parent_a_type_id,
            event_type_name: parent_a_type_name,
            make_setup: parent_a_setup,
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

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("multipack").expect("export succeeds");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn occurrence_marks_are_subject_scoped_and_detectors_are_reused() {
    let records = records();
    let cycle = function(&records, "__sand_event_cycle");
    assert!(cycle.contains("execute as @a run scoreboard players set @s se_"));
    assert!(!cycle.contains("scoreboard players set @a se_"));

    for parent in [
        std::any::type_name::<ParentA>(),
        std::any::type_name::<ParentB>(),
        std::any::type_name::<ProviderOnlyParent>(),
    ] {
        let parent_key = key(parent);
        assert_eq!(
            records
                .iter()
                .filter(|record| {
                    record["dir"] == "function"
                        && record["path"] == format!("__sand_event_check/{parent_key}")
                })
                .count(),
            1,
            "one detector for {parent}"
        );
        let dispatch = function(&records, &format!("__sand_event_dispatch/{parent_key}"));
        assert!(
            dispatch
                .lines()
                .next()
                .unwrap()
                .contains("scoreboard players set @s")
        );
    }

    let sneaking_key = key(std::any::type_name::<PlayerSneakEvent>());
    assert!(!records.iter().any(|record| {
        record["dir"] == "function"
            && record["path"] == format!("__sand_event_check/{sneaking_key}")
    }));
}

#[test]
fn any_all_and_mixed_groups_lower_deterministically() {
    let records = records();
    let cycle = function(&records, "__sand_event_cycle");
    let any_key = key(std::any::type_name::<AnyChild>());
    let all_key = key(std::any::type_name::<AllChild>());
    let nested_key = key(std::any::type_name::<NestedChild>());
    let immediate_key = key(std::any::type_name::<ImmediateIntermediate>());
    let downstream_key = key(std::any::type_name::<DownstreamOfImmediate>());
    let a_key = key(std::any::type_name::<ParentA>());
    let b_key = key(std::any::type_name::<ParentB>());

    assert_eq!(
        cycle
            .lines()
            .filter(|line| { line.contains(&format!("unless score @s se_{any_key}_m matches 1")) })
            .count(),
        2,
        "{cycle}"
    );
    assert!(cycle.contains(&format!("if score @s se_{a_key}_o matches 1")));
    assert!(cycle.contains(&format!("if score @s se_{b_key}_o matches 1")));
    let all_line = cycle
        .lines()
        .find(|line| line.contains(&format!("__sand_event_multi_eval/{all_key}")))
        .expect("all-parent resolver");
    assert!(all_line.contains(&format!("if score @s se_{a_key}_o matches 1")));
    assert!(all_line.contains(&format!("if score @s se_{b_key}_o matches 1")));
    let any_position = cycle
        .find(&format!("__sand_event_multi_gate/{any_key}"))
        .expect("any group resolver");
    let nested_position = cycle
        .find(&format!("__sand_event_multi_eval/{nested_key}"))
        .expect("nested resolver");
    assert!(any_position < nested_position, "topological resolver order");
    let parent_post_position = cycle
        .find("scoreboard players operation @s phase4_sync = @s phase4_a")
        .expect("deferred parent post-observation");
    assert!(
        nested_position < parent_post_position,
        "all staged descendants must observe event-time state before parent post-observation"
    );
    let any_post_position = cycle
        .find("function multipack:__sand_event_multi_post/")
        .expect("deferred staged-parent post-observation");
    assert!(
        nested_position < any_post_position && any_post_position < parent_post_position,
        "staged parent post-observation must follow staged descendants and precede root post"
    );
    let downstream_position = cycle
        .find(&format!("__sand_event_multi_eval/{downstream_key}"))
        .expect("downstream staged resolver");
    let immediate_post_position = cycle
        .find(&format!("__sand_event_multi_post/{immediate_key}"))
        .expect("deferred immediate-intermediate post-observation");
    assert!(downstream_position < immediate_post_position);
    assert!(cycle.contains(&format!(
        "execute as @a run scoreboard players set @s se_{immediate_key}_c 0"
    )));
    let immediate_observe = function(&records, &format!("__sand_event_observe/{immediate_key}"));
    assert!(
        immediate_observe.contains(&format!("scoreboard players set @s se_{immediate_key}_c 1"))
    );
    assert!(!immediate_observe.contains("phase4_immediate_post"));

    let all_eval = function(&records, &format!("__sand_event_multi_eval/{all_key}"));
    assert!(all_eval.contains("predicate multipack:__sand/player_sneaking"));
    assert!(all_eval.contains("if entity @s[tag=ready]"));
    assert!(all_eval.contains("unless entity @s[tag=blocked]"));

    let any_gate = function(&records, &format!("__sand_event_multi_gate/{any_key}"));
    assert_eq!(
        any_gate,
        format!(
            "scoreboard players set @s se_{any_key}_m 1\nfunction multipack:__sand_event_multi_eval/{any_key}"
        )
    );

    for event in [
        std::any::type_name::<ParentA>(),
        std::any::type_name::<ParentB>(),
        std::any::type_name::<AnyChild>(),
    ] {
        let event_key = key(event);
        let objective = format!("scoreboard objectives add se_{event_key}_o dummy");
        assert_eq!(
            records
                .iter()
                .filter_map(|record| record["content"].as_str())
                .flat_map(str::lines)
                .filter(|line| *line == objective)
                .count(),
            1,
            "occurrence setup emitted once for {event}"
        );
    }
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("multipack").unwrap();
    let second = sand_core::try_export_components_json("multipack").unwrap();
    assert_eq!(first, second);
}
