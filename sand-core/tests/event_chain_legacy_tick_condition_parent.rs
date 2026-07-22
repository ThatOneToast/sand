//! Regression coverage for a legacy `SandEventDispatch::TickCondition` parent
//! participating in same-cycle chained dispatch (#240 follow-up).
//!
//! Before this fix, a `SandEvent` whose `dispatch()` returned
//! `SandEventDispatch::TickCondition(..)` was routed through the unrelated
//! legacy `tick_poll_events` aggregation for its own direct handlers, while
//! chain discovery (`parent_dispatch().normalize()`) independently turned the
//! same parent into a graph `Tick` root — producing **two** detectors for one
//! concrete parent type. This suite proves through the *real* export pipeline
//! that a legacy `TickCondition` parent now resolves to exactly one graph
//! node/detector, shared by its direct handlers and its chain children, and
//! that ordinary legacy tick events with no children remain unaffected.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SameCycleEventDependency, SameCycleEventRequirement,
    SandEventDispatch,
};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

fn after(
    event_type_id: fn() -> TypeId,
    event_type_name: fn() -> &'static str,
    event_dispatch: fn() -> SandEventDispatch,
    event_setup: fn() -> EventSetup,
) -> Vec<SameCycleEventRequirement> {
    vec![SameCycleEventRequirement::After(SameCycleEventDependency {
        event_type_id,
        event_type_name,
        event_dispatch,
        event_setup,
        event_raw_setup: event_setup,
        event_participants: || sand_core::participant::EventParticipantPlan::none(),
        event_revoke: || true,
    })]
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_tick() -> Option<sand_core::events::TickEventDispatch> {
    None
}
fn no_chain() -> Option<ChainEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
}

// ── LegacyParent: SandEventDispatch::TickCondition, two direct handlers,
//    one chain child ───────────────────────────────────────────────────────

struct LegacyParent;

fn legacy_parent_condition() -> Option<String> {
    Some("score @s legacy matches 1".to_string())
}
fn legacy_parent_type_id() -> TypeId {
    TypeId::of::<LegacyParent>()
}
fn legacy_parent_type_name() -> &'static str {
    std::any::type_name::<LegacyParent>()
}
fn legacy_parent_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add legacy dummy".to_string()],
        pre_observation: vec![],
        post_observation: vec![],
    }
}

fn on_legacy_a_body() -> Vec<String> {
    vec!["say legacy a".to_string()]
}
fn on_legacy_b_body() -> Vec<String> {
    vec!["say legacy b".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_legacy_a",
        id_override: None,
        make: on_legacy_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: legacy_parent_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: legacy_parent_type_id,
            event_type_name: legacy_parent_type_name,
            make_setup: legacy_parent_setup,
        },
    }
}
sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_legacy_b",
        id_override: None,
        make: on_legacy_b_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: legacy_parent_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: legacy_parent_type_id,
            event_type_name: legacy_parent_type_name,
            make_setup: legacy_parent_setup,
        },
    }
}

struct LegacyChild;

fn legacy_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            legacy_parent_type_id,
            legacy_parent_type_name,
            || {
                sand_core::events::SandEventDispatch::TickCondition(
                    legacy_parent_condition().unwrap(),
                )
            },
            legacy_parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![sand_core::condition::Condition::raw(
            "score @s legchild matches 1",
        )],
        unless: vec![],
    })
}
fn legacy_child_type_id() -> TypeId {
    TypeId::of::<LegacyChild>()
}
fn legacy_child_type_name() -> &'static str {
    std::any::type_name::<LegacyChild>()
}
fn on_legacy_child_body() -> Vec<String> {
    vec!["say legacy child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_legacy_child",
        id_override: None,
        make: on_legacy_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition_none,
            make_tick: no_tick,
            make_chain: legacy_child_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: legacy_child_type_id,
            event_type_name: legacy_child_type_name,
            make_setup: empty_setup,
        },
    }
}
fn no_condition_none() -> Option<String> {
    None
}

// ── LegacyOrphanParent: legacy TickCondition, no direct handler, referenced
//    only by a chain child ──────────────────────────────────────────────────

struct LegacyOrphanParent;

fn legacy_orphan_parent_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::TickCondition(
        "score @s legacy_orphan matches 1".to_string(),
    )
}
fn legacy_orphan_parent_type_id() -> TypeId {
    TypeId::of::<LegacyOrphanParent>()
}
fn legacy_orphan_parent_type_name() -> &'static str {
    std::any::type_name::<LegacyOrphanParent>()
}

struct LegacyOrphanChild;

fn legacy_orphan_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            legacy_orphan_parent_type_id,
            legacy_orphan_parent_type_name,
            legacy_orphan_parent_dispatch,
            EventSetup::none,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn legacy_orphan_child_type_id() -> TypeId {
    TypeId::of::<LegacyOrphanChild>()
}
fn legacy_orphan_child_type_name() -> &'static str {
    std::any::type_name::<LegacyOrphanChild>()
}
fn on_legacy_orphan_child_body() -> Vec<String> {
    vec!["say legacy orphan child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_legacy_orphan_child",
        id_override: None,
        make: on_legacy_orphan_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition_none,
            make_tick: no_tick,
            make_chain: legacy_orphan_child_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: legacy_orphan_child_type_id,
            event_type_name: legacy_orphan_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── LegacyNoChildEvent: ordinary legacy tick event with no children,
//    including a Sand-owned entity-predicate condition string, proving
//    predicate JSON emission still works after moving off the old
//    `tick_poll_events` aggregation path ───────────────────────────────────

struct LegacyNoChildEvent;

fn legacy_no_child_condition() -> Option<String> {
    Some("predicate __sand_local:__sand/player_sneaking".to_string())
}
fn legacy_no_child_type_id() -> TypeId {
    TypeId::of::<LegacyNoChildEvent>()
}
fn legacy_no_child_type_name() -> &'static str {
    std::any::type_name::<LegacyNoChildEvent>()
}
fn on_legacy_no_child_body() -> Vec<String> {
    vec!["say sneaking".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_legacy_no_child",
        id_override: None,
        make: on_legacy_no_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: legacy_no_child_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: legacy_no_child_type_id,
            event_type_name: legacy_no_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── Test helpers ────────────────────────────────────────────────────────────

fn expected_key(canonical_type_name: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in canonical_type_name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("legacychainpack").expect("export should succeed");
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

fn function_content<'a>(records: &'a [serde_json::Value], path_prefix: &str) -> &'a str {
    function_records(records)
        .into_iter()
        .find(|r| {
            r["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with(path_prefix)
        })
        .unwrap_or_else(|| panic!("no function record with path prefix `{path_prefix}`"))
        .get("content")
        .and_then(|c| c.as_str())
        .unwrap_or_default()
}

// ── Tests ────────────────────────────────────────────────────────────────

#[test]
fn legacy_condition_occurs_in_exactly_one_generated_detector() {
    let records = records();
    let condition_needle = "score @s legacy matches 1";

    let occurrences: usize = function_records(&records)
        .iter()
        .filter(|r| {
            r["content"]
                .as_str()
                .unwrap_or_default()
                .contains(condition_needle)
        })
        .count();
    assert_eq!(
        occurrences, 1,
        "the legacy parent condition must appear in exactly one generated function"
    );
}

#[test]
fn legacy_parent_detector_calls_one_shared_dispatch() {
    let records = records();
    let key = expected_key(legacy_parent_type_name());
    let check_content = function_content(&records, &format!("__sand_event_check/{key}"));

    let dispatch_calls = check_content
        .matches(&format!("__sand_event_dispatch/{key}"))
        .count();
    assert_eq!(
        dispatch_calls, 1,
        "the detector must call exactly one shared dispatch function: {check_content:?}"
    );
}

#[test]
fn legacy_parent_dispatch_reaches_direct_handlers() {
    let records = records();
    let key = expected_key(legacy_parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(
        content.contains("legacychainpack:on_legacy_a"),
        "{content:?}"
    );
    assert!(
        content.contains("legacychainpack:on_legacy_b"),
        "{content:?}"
    );
}

#[test]
fn legacy_parent_dispatch_reaches_the_child() {
    let records = records();
    let key = expected_key(legacy_parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(
        content.contains("legacychainpack:on_legacy_child")
            || content.contains("score @s legchild matches 1"),
        "parent dispatch must reach the chained child: {content:?}"
    );
}

#[test]
fn legacy_child_inherits_at_s_without_execute_as() {
    let records = records();
    let key = expected_key(legacy_parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    let line = content
        .lines()
        .find(|l| l.contains("legchild"))
        .unwrap_or_else(|| panic!("expected child condition line, got: {content:?}"));
    assert!(
        !line.contains("as @a"),
        "child edge must not re-issue `execute as @a`: {line:?}"
    );
}

#[test]
fn legacy_parent_detector_appears_exactly_once_in_minecraft_tick() {
    let records = records();
    let key = expected_key(legacy_parent_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    let matches: Vec<&String> = tick_tag
        .iter()
        .filter(|f| f.contains("__sand_event_check") && f.contains(&key))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one legacy parent detector in minecraft:tick, got {tick_tag:?}"
    );
}

#[test]
fn legacy_orphan_parent_with_no_direct_handler_still_gets_detector() {
    let records = records();
    let key = expected_key(legacy_orphan_parent_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    assert!(
        tick_tag
            .iter()
            .any(|f| f.contains("__sand_event_check") && f.contains(&key)),
        "legacy parent referenced only by a chain child must still get a detector: {tick_tag:?}"
    );
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(content.contains("legacychainpack:on_legacy_orphan_child"));
}

#[test]
fn legacy_no_child_event_remains_functional_with_predicate_emission() {
    let records = records();
    let key = expected_key(legacy_no_child_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    assert!(
        tick_tag
            .iter()
            .any(|f| f.contains("__sand_event_check") && f.contains(&key)),
        "no-child legacy event must still be wired into minecraft:tick: {tick_tag:?}"
    );

    let content = function_content(&records, &format!("__sand_event_check/{key}"));
    assert!(
        content.contains("execute as @a at @s if predicate legacychainpack:__sand/player_sneaking"),
        "expected correct at-@s-before-clause ordering for the legacy condition: {content:?}"
    );

    // The Sand-owned entity-predicate JSON must still be emitted even though
    // this SandEvent now goes through the graph rather than the old
    // `tick_poll_events` aggregation.
    let predicate_record = records
        .iter()
        .find(|r| {
            r["dir"].as_str() == Some("predicate")
                && r["path"].as_str() == Some("__sand/player_sneaking")
        })
        .expect("expected generated player_sneaking predicate JSON");
    let predicate_content: serde_json::Value =
        serde_json::from_str(predicate_record["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        predicate_content["predicate"]["flags"]["is_sneaking"],
        serde_json::json!(true)
    );
}

#[test]
fn repeated_export_produces_identical_output() {
    let first =
        sand_core::try_export_components_json("legacychainpack").expect("export should succeed");
    let second =
        sand_core::try_export_components_json("legacychainpack").expect("export should succeed");
    assert_eq!(first, second);
}
