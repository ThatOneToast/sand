//! Integration coverage for the structured `SandEventDispatch::Tick` lifecycle
//! (#239): proves through the *real* export pipeline that
//!
//! - multiple `#[event]` handlers on the same `SandEvent` type share one
//!   generated detector/dispatch function instead of duplicating detection
//!   (setup dedup, keyed by `event_type_id`);
//! - lifecycle setup objectives are emitted exactly once into `minecraft:load`;
//! - detection always runs before `post_observation` commands in the
//!   generated tick function (required so a jump-delta-style sync doesn't
//!   erase the value being compared before it's observed).

use sand_core::events::{EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

/// Stand-in for a user's `SandEvent` marker type — never constructed, only
/// used for its `TypeId`.
struct PlayerJumpEvent;

fn jump_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(sand_core::condition::Condition::raw(
                "score @s sync_jumps < @s jumps",
            )),
    )
}

fn jump_setup() -> EventSetup {
    EventSetup {
        objectives: vec![
            "scoreboard objectives add jumps minecraft.custom:minecraft.jump".to_string(),
            "scoreboard objectives add sync_jumps dummy".to_string(),
        ],
        pre_observation: vec![],
        post_observation: vec!["scoreboard players operation @a sync_jumps = @a jumps".to_string()],
    }
}

fn jump_event_type_id() -> TypeId {
    TypeId::of::<PlayerJumpEvent>()
}

fn handler_a_body() -> Vec<String> {
    vec!["say jumped (handler a)".to_string()]
}

fn handler_b_body() -> Vec<String> {
    vec!["say jumped (handler b)".to_string()]
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn revoke_true() -> bool {
    true
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_jump_a",
        id_override: None,
        make: handler_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: jump_dispatch,
            revoke: revoke_true,
            event_type_id: jump_event_type_id,
            make_setup: jump_setup,
        },
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_jump_b",
        id_override: None,
        make: handler_b_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: jump_dispatch,
            revoke: revoke_true,
            event_type_id: jump_event_type_id,
            make_setup: jump_setup,
        },
    }
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("jumppack").expect("export should succeed");
    serde_json::from_str(&json).expect("export output should be valid JSON")
}

fn tag_values(records: &[serde_json::Value], tag_rl: &str) -> Vec<String> {
    let tag_path = tag_rl.split_once(':').map(|(_, p)| p).unwrap_or(tag_rl);
    for r in records {
        if r["dir"].as_str() == Some("tags/function")
            && r["path"].as_str() == Some(tag_path)
            && let Some(arr) = r["content"]
                .as_str()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v["values"].as_array().cloned())
        {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
        }
    }
    Vec::new()
}

fn function_records(records: &[serde_json::Value]) -> Vec<&serde_json::Value> {
    records
        .iter()
        .filter(|r| r["dir"].as_str() == Some("function"))
        .collect()
}

#[test]
fn two_handlers_on_same_event_share_one_detector() {
    let records = records();
    let tick_tag = tag_values(&records, "minecraft:tick");

    let generated_checks: Vec<&String> = tick_tag
        .iter()
        .filter(|f| f.contains("__sand_event_check"))
        .collect();
    assert_eq!(
        generated_checks.len(),
        1,
        "two handlers on the same SandEvent must share exactly one detector, got: {tick_tag:?}"
    );
}

#[test]
fn setup_objectives_emitted_exactly_once() {
    let records = records();
    let load_tag = tag_values(&records, "minecraft:load");

    let generated_setups: Vec<&String> = load_tag
        .iter()
        .filter(|f| f.contains("__sand_event_setup"))
        .collect();
    assert_eq!(
        generated_setups.len(),
        1,
        "setup objectives must be deduplicated across handlers, got: {load_tag:?}"
    );

    let fns = function_records(&records);
    let setup_fn = fns
        .iter()
        .find(|r| {
            r["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("__sand_event_setup")
        })
        .expect("setup function record must exist");
    let content = setup_fn["content"].as_str().unwrap_or_default();
    assert!(content.contains("scoreboard objectives add jumps"));
    assert!(content.contains("scoreboard objectives add sync_jumps"));
}

#[test]
fn detection_runs_before_synchronization() {
    let records = records();
    let fns = function_records(&records);
    let check_fn = fns
        .iter()
        .find(|r| {
            r["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("__sand_event_check")
        })
        .expect("detector function record must exist");
    let content = check_fn["content"].as_str().unwrap_or_default();

    let detect_pos = content
        .find("if score @s sync_jumps < @s jumps")
        .expect("detection clause must be present");
    let sync_pos = content
        .find("scoreboard players operation @a sync_jumps = @a jumps")
        .expect("post_observation sync command must be present");
    assert!(
        detect_pos < sync_pos,
        "detection must run before the synchronizing post_observation command: {content:?}"
    );
}

#[test]
fn both_handler_bodies_are_reachable_from_the_shared_dispatch() {
    let records = records();
    let fns = function_records(&records);
    let dispatch_fn = fns
        .iter()
        .find(|r| {
            r["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("__sand_event_dispatch")
        })
        .expect("fan-out dispatch function record must exist for two handlers");
    let content = dispatch_fn["content"].as_str().unwrap_or_default();
    assert!(content.contains("jumppack:on_jump_a"));
    assert!(content.contains("jumppack:on_jump_b"));
}
