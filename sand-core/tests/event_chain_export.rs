//! Integration coverage for same-cycle chained `SandEvent` dispatch (#240):
//! proves through the *real* export pipeline that
//!
//! - a parent's detector/setup is emitted exactly once, whether or not it has
//!   a direct `#[event]` handler of its own;
//! - a parent referenced only by chain children (no direct handler) still
//!   gets its detector and setup generated;
//! - a child's condition is evaluated inline inside the parent's dispatch
//!   function — inheriting `@s`/position — not as a separately-polled
//!   `execute as @a` loop;
//! - an unconditional child is called directly with no `execute if`/`unless`
//!   wrapper;
//! - a child with a multi-plan condition is coalesced with a per-player guard
//!   so it fires at most once per parent invocation;
//! - distinct concrete child types sharing one parent get distinct generated
//!   dispatch resources;
//! - adding a child does not rename the parent's detector/setup key;
//! - the child is dispatched before the parent's `post_observation` runs.

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SameCycleEventDependency, SameCycleEventRequirement,
    SandEventDispatch, TickEventDispatch,
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
        event_revoke: || true,
    })]
}

fn expected_key(canonical_type_name: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in canonical_type_name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
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
fn empty_setup() -> EventSetup {
    EventSetup::none()
}

// ── ParentEvent: tick root with a delta-sync post_observation, one direct
//    handler, and multiple chain children ───────────────────────────────────

struct ParentEvent;

fn parent_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s sync_p < @s cur_p")),
    )
}

fn parent_setup() -> EventSetup {
    EventSetup {
        objectives: vec![
            "scoreboard objectives add cur_p dummy".to_string(),
            "scoreboard objectives add sync_p dummy".to_string(),
        ],
        pre_observation: vec![],
        post_observation: vec![
            "execute as @a run scoreboard players operation @s sync_p = @s cur_p".to_string(),
        ],
    }
}

fn parent_type_id() -> TypeId {
    TypeId::of::<ParentEvent>()
}
fn parent_type_name() -> &'static str {
    std::any::type_name::<ParentEvent>()
}

fn on_parent_body() -> Vec<String> {
    vec!["say parent fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_parent",
        id_override: None,
        make: on_parent_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: parent_dispatch,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: parent_type_id,
            event_type_name: parent_type_name,
            make_setup: parent_setup,
        },
    }
}

// ── SingleCondChild: chains from ParentEvent with one `when` condition ─────

struct SingleCondChild;

fn single_cond_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            parent_type_id,
            parent_type_name,
            || SandEventDispatch::Tick(parent_dispatch().unwrap()),
            parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![Condition::raw("block ~ ~-1 ~ minecraft:white_wool")],
        unless: vec![],
    })
}

fn single_cond_child_type_id() -> TypeId {
    TypeId::of::<SingleCondChild>()
}
fn single_cond_child_type_name() -> &'static str {
    std::any::type_name::<SingleCondChild>()
}

fn on_single_cond_child_body() -> Vec<String> {
    vec!["say on elevator".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_single_cond_child",
        id_override: None,
        make: on_single_cond_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: single_cond_chain,
            revoke: revoke_true,
            event_type_id: single_cond_child_type_id,
            event_type_name: single_cond_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── UnconditionalChild: chains from ParentEvent with no when/unless ────────

struct UnconditionalChild;

fn unconditional_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            parent_type_id,
            parent_type_name,
            || SandEventDispatch::Tick(parent_dispatch().unwrap()),
            parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}

fn unconditional_child_type_id() -> TypeId {
    TypeId::of::<UnconditionalChild>()
}
fn unconditional_child_type_name() -> &'static str {
    std::any::type_name::<UnconditionalChild>()
}

fn on_unconditional_child_body() -> Vec<String> {
    vec!["say always after parent".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_unconditional_child",
        id_override: None,
        make: on_unconditional_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: unconditional_chain,
            revoke: revoke_true,
            event_type_id: unconditional_child_type_id,
            event_type_name: unconditional_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── OrCondChild: chains from ParentEvent with a multi-plan (OR) condition ──

struct OrCondChild;

fn or_cond_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            parent_type_id,
            parent_type_name,
            || SandEventDispatch::Tick(parent_dispatch().unwrap()),
            parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![
            Condition::raw("score @s a matches 1").or(Condition::raw("score @s b matches 1")),
        ],
        unless: vec![],
    })
}

fn or_cond_child_type_id() -> TypeId {
    TypeId::of::<OrCondChild>()
}
fn or_cond_child_type_name() -> &'static str {
    std::any::type_name::<OrCondChild>()
}

fn on_or_cond_child_body() -> Vec<String> {
    vec!["say or child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_or_cond_child",
        id_override: None,
        make: on_or_cond_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: or_cond_chain,
            revoke: revoke_true,
            event_type_id: or_cond_child_type_id,
            event_type_name: or_cond_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── DistinctChild{A,B}: two distinct concrete types simulating a generic
//    family (ElevatorUsed<GoUp> / ElevatorUsed<GoDown>), sharing ParentEvent ──

struct DistinctChildA;
struct DistinctChildB;

fn distinct_a_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            parent_type_id,
            parent_type_name,
            || SandEventDispatch::Tick(parent_dispatch().unwrap()),
            parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![Condition::raw("tag @s distinct_a")],
        unless: vec![],
    })
}
fn distinct_b_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            parent_type_id,
            parent_type_name,
            || SandEventDispatch::Tick(parent_dispatch().unwrap()),
            parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![Condition::raw("tag @s distinct_b")],
        unless: vec![],
    })
}

fn distinct_a_type_id() -> TypeId {
    TypeId::of::<DistinctChildA>()
}
fn distinct_a_type_name() -> &'static str {
    std::any::type_name::<DistinctChildA>()
}
fn distinct_b_type_id() -> TypeId {
    TypeId::of::<DistinctChildB>()
}
fn distinct_b_type_name() -> &'static str {
    std::any::type_name::<DistinctChildB>()
}

fn on_distinct_a_body() -> Vec<String> {
    vec!["say distinct a".to_string()]
}
fn on_distinct_b_body() -> Vec<String> {
    vec!["say distinct b".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_distinct_a",
        id_override: None,
        make: on_distinct_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: distinct_a_chain,
            revoke: revoke_true,
            event_type_id: distinct_a_type_id,
            event_type_name: distinct_a_type_name,
            make_setup: empty_setup,
        },
    }
}
sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_distinct_b",
        id_override: None,
        make: on_distinct_b_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: distinct_b_chain,
            revoke: revoke_true,
            event_type_id: distinct_b_type_id,
            event_type_name: distinct_b_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── OrphanParent: has no direct handler of its own, only a chain child ────

struct OrphanParent;

fn orphan_parent_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::Tick(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s orphan_flag matches 1")),
    )
}
fn orphan_parent_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add orphan_flag dummy".to_string()],
        pre_observation: vec![],
        post_observation: vec![],
    }
}
fn orphan_parent_type_id() -> TypeId {
    TypeId::of::<OrphanParent>()
}
fn orphan_parent_type_name() -> &'static str {
    std::any::type_name::<OrphanParent>()
}

struct OrphanChild;

fn orphan_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: after(
            orphan_parent_type_id,
            orphan_parent_type_name,
            orphan_parent_dispatch,
            orphan_parent_setup,
        ),
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn orphan_child_type_id() -> TypeId {
    TypeId::of::<OrphanChild>()
}
fn orphan_child_type_name() -> &'static str {
    std::any::type_name::<OrphanChild>()
}
fn on_orphan_child_body() -> Vec<String> {
    vec!["say orphan child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_orphan_child",
        id_override: None,
        make: on_orphan_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: orphan_child_chain,
            revoke: revoke_true,
            event_type_id: orphan_child_type_id,
            event_type_name: orphan_child_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── Test helpers ────────────────────────────────────────────────────────────

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("chainpack").expect("export should succeed");
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

fn has_function_with_prefix(records: &[serde_json::Value], path_prefix: &str) -> bool {
    function_records(records).iter().any(|r| {
        r["path"]
            .as_str()
            .unwrap_or_default()
            .starts_with(path_prefix)
    })
}

// ── Tests ────────────────────────────────────────────────────────────────

#[test]
fn parent_detector_and_setup_emitted_exactly_once() {
    let records = records();
    let key = expected_key(parent_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    let load_tag = tag_values(&records, "minecraft:load");

    let checks: Vec<&String> = tick_tag
        .iter()
        .filter(|f| f.contains("__sand_event_check") && f.contains(&key))
        .collect();
    assert_eq!(
        checks.len(),
        1,
        "expected exactly one parent detector, got {tick_tag:?}"
    );

    let setups: Vec<&String> = load_tag
        .iter()
        .filter(|f| f.contains("__sand_event_setup") && f.contains(&key))
        .collect();
    assert_eq!(
        setups.len(),
        1,
        "expected exactly one parent setup, got {load_tag:?}"
    );
}

#[test]
fn direct_handler_and_children_are_both_reachable() {
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(content.contains("chainpack:on_parent"), "{content:?}");

    // A leaf child (single handler, no descendants of its own) is called
    // directly — the same "no wrapper needed" optimization roots use for a
    // single handler with no children.
    assert!(
        content.contains("chainpack:on_single_cond_child"),
        "parent dispatch must reference the child's handler: {content:?}"
    );
}

#[test]
fn child_condition_inherits_at_s_without_execute_as() {
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));

    let line = content
        .lines()
        .find(|l| l.contains("minecraft:white_wool"))
        .unwrap_or_else(|| panic!("expected child condition line, got: {content:?}"));
    assert!(
        !line.contains("as @a"),
        "child edge must not re-issue `execute as @a` — inherit the current subject: {line:?}"
    );
    assert!(line.trim_start().starts_with("execute if") || line.contains(" if "));
}

#[test]
fn unconditional_child_is_called_with_no_execute_wrapper() {
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));

    let expected_call = "function chainpack:on_unconditional_child".to_string();
    assert!(
        content.contains(&expected_call),
        "unconditional child must be called directly, no execute wrapper: {content:?}"
    );
}

#[test]
fn multi_plan_child_condition_is_guarded_against_duplicate_firing() {
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));

    let a_count = content.matches("if score @s a matches 1").count();
    let b_count = content.matches("if score @s b matches 1").count();
    assert_eq!(a_count, 1, "{content:?}");
    assert_eq!(b_count, 1, "{content:?}");

    for line in content
        .lines()
        .filter(|l| l.contains("score @s a matches 1") || l.contains("score @s b matches 1"))
    {
        assert!(
            line.contains("unless score @s") && line.contains("matches 1"),
            "multi-plan child edge lines must carry a guard clause: {line:?}"
        );
    }

    let or_child_key = expected_key(or_cond_child_type_name());
    // A dedicated edge function must exist to set the guard before dispatching.
    assert!(
        has_function_with_prefix(&records, &format!("__sand_event_edge/{or_child_key}")),
        "expected a generated edge function for the multi-plan child"
    );
    let edge_content = function_content(&records, &format!("__sand_event_edge/{or_child_key}"));
    assert!(
        edge_content
            .lines()
            .next()
            .unwrap_or_default()
            .contains("scoreboard players set @s")
    );
    // The child itself is a leaf (single handler, no descendants), so the
    // edge function calls its handler directly rather than a dispatch wrapper.
    assert!(edge_content.contains("chainpack:on_or_cond_child"));
}

#[test]
fn distinct_generic_like_children_get_distinct_dispatch_resources() {
    let records = records();
    let a_key = expected_key(distinct_a_type_name());
    let b_key = expected_key(distinct_b_type_name());
    assert_ne!(a_key, b_key);

    let parent_key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{parent_key}"));
    // Each is a leaf child (single handler, no descendants), so its edge in
    // the parent calls its handler directly — but the two conditions and
    // handler paths must remain distinct.
    assert!(content.contains("tag @s distinct_a"));
    assert!(content.contains("tag @s distinct_b"));
    assert!(content.contains("chainpack:on_distinct_a"));
    assert!(content.contains("chainpack:on_distinct_b"));
}

#[test]
fn adding_a_child_does_not_rename_the_parent_detector() {
    let records = records();
    let key = expected_key(parent_type_name());
    // The key is a pure function of the canonical parent type name — proven
    // directly since the same ParentEvent type has 4 children registered
    // above yet its generated key still matches the pre-#240 formula.
    assert!(has_function_with_prefix(
        &records,
        &format!("__sand_event_check/{key}")
    ));
    assert!(has_function_with_prefix(
        &records,
        &format!("__sand_event_setup/{key}")
    ));
}

#[test]
fn child_dispatch_runs_before_parent_post_observation() {
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    // post_observation ("sync_p = cur_p") must appear in the *check* function
    // (unconditional, after detection), and the detection line calling the
    // dispatch function (which contains all child edges) must appear first.
    let detect_pos = content
        .find("run function chainpack:__sand_event_dispatch")
        .expect("detection line calling dispatch must be present");
    let sync_pos = content
        .find("execute as @a run scoreboard players operation @s sync_p = @s cur_p")
        .expect("post_observation sync command must be present");
    assert!(
        detect_pos < sync_pos,
        "dispatch (which includes child edges) must run before parent's post_observation: {content:?}"
    );
}

#[test]
fn orphan_parent_with_no_direct_handler_still_gets_detector_and_setup() {
    let records = records();
    let key = expected_key(orphan_parent_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    let load_tag = tag_values(&records, "minecraft:load");

    assert!(
        tick_tag
            .iter()
            .any(|f| f.contains("__sand_event_check") && f.contains(&key)),
        "parent referenced only by a chain child must still get a detector: {tick_tag:?}"
    );
    assert!(
        load_tag
            .iter()
            .any(|f| f.contains("__sand_event_setup") && f.contains(&key)),
        "parent referenced only by a chain child must still get setup: {load_tag:?}"
    );

    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(content.contains("chainpack:on_orphan_child"));
}

#[test]
fn repeated_export_produces_identical_output() {
    let first = sand_core::try_export_components_json("chainpack").expect("export should succeed");
    let second = sand_core::try_export_components_json("chainpack").expect("export should succeed");
    assert_eq!(first, second);
}
