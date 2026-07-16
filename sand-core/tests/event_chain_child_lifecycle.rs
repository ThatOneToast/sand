//! Regression coverage for chained-child lifecycle ordering (#240 follow-up).
//!
//! Before this fix, a chained child's `pre_observation`/`post_observation`
//! ran *inside* its own dispatch function — reached only after the child's
//! condition test succeeded. That meant `pre_observation` could never
//! prepare state used by the child's own condition, and `post_observation`
//! (which must advance synchronized state every observation cycle, per the
//! tick-lifecycle contract) silently skipped whenever the condition failed.
//!
//! This suite proves through the *real* export pipeline that, for a chained
//! child with lifecycle commands, each parent invocation now performs:
//!
//! ```text
//! child pre_observation
//! child condition evaluation
//! child handler/descendant dispatch if matched
//! child post_observation
//! ```
//!
//! with `post_observation` always structurally reached — not only on a
//! successful condition match — via a dedicated
//! `__sand_event_observe/<child>` function, at every chain depth.

use sand_core::condition::Condition;
use sand_core::events::{ChainEventDispatch, EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

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

// ── Parent: tick root, one direct handler ───────────────────────────────────

struct Parent;

fn parent_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s parent_flag matches 1")),
    )
}
fn parent_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add parent_flag dummy".to_string()],
        pre_observation: vec![],
        post_observation: vec![
            "execute as @a run scoreboard players operation @s parent_post = @s parent_flag"
                .to_string(),
        ],
    }
}
fn parent_type_id() -> TypeId {
    TypeId::of::<Parent>()
}
fn parent_type_name() -> &'static str {
    std::any::type_name::<Parent>()
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

fn parent_chain_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::Tick(parent_dispatch().unwrap())
}

// ── SinglePlanChild: chains from Parent with one condition + lifecycle ─────
//
// The condition reads `current`, which `pre_observation` is responsible for
// preparing from `source` — proving pre_observation runs before the
// condition is tested, not after.

struct SinglePlanChild;

fn single_plan_child_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players operation @s current = @s source".to_string()],
        post_observation: vec!["scoreboard players operation @s sync = @s current".to_string()],
    }
}
fn single_plan_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id,
        parent_type_name,
        parent_dispatch: parent_chain_dispatch,
        parent_setup,
        persistent: vec![],
        when: vec![Condition::raw("score @s sync < @s current")],
        unless: vec![],
    })
}
fn single_plan_child_type_id() -> TypeId {
    TypeId::of::<SinglePlanChild>()
}
fn single_plan_child_type_name() -> &'static str {
    std::any::type_name::<SinglePlanChild>()
}
fn on_single_plan_child_body() -> Vec<String> {
    vec!["say single-plan child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_single_plan_child",
        id_override: None,
        make: on_single_plan_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: single_plan_child_chain,
            revoke: revoke_true,
            event_type_id: single_plan_child_type_id,
            event_type_name: single_plan_child_type_name,
            make_setup: single_plan_child_setup,
        },
    }
}

// ── UnconditionalLifecycleChild: chains from Parent, no when/unless, but
//    still owns lifecycle commands ──────────────────────────────────────────

struct UnconditionalLifecycleChild;

fn unconditional_lifecycle_child_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players set @s uncond_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @s uncond_post 1".to_string()],
    }
}
fn unconditional_lifecycle_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id,
        parent_type_name,
        parent_dispatch: parent_chain_dispatch,
        parent_setup,
        persistent: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn unconditional_lifecycle_child_type_id() -> TypeId {
    TypeId::of::<UnconditionalLifecycleChild>()
}
fn unconditional_lifecycle_child_type_name() -> &'static str {
    std::any::type_name::<UnconditionalLifecycleChild>()
}
fn on_unconditional_lifecycle_child_body() -> Vec<String> {
    vec!["say unconditional lifecycle child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_unconditional_lifecycle_child",
        id_override: None,
        make: on_unconditional_lifecycle_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: unconditional_lifecycle_child_chain,
            revoke: revoke_true,
            event_type_id: unconditional_lifecycle_child_type_id,
            event_type_name: unconditional_lifecycle_child_type_name,
            make_setup: unconditional_lifecycle_child_setup,
        },
    }
}

// ── MultiPlanLifecycleChild: chains from Parent with an OR condition and
//    owns lifecycle commands ────────────────────────────────────────────────

struct MultiPlanLifecycleChild;

fn multi_plan_lifecycle_child_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players set @s mp_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @s mp_post 1".to_string()],
    }
}
fn multi_plan_lifecycle_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id,
        parent_type_name,
        parent_dispatch: parent_chain_dispatch,
        parent_setup,
        persistent: vec![],
        when: vec![
            Condition::raw("score @s mp_a matches 1").or(Condition::raw("score @s mp_b matches 1")),
        ],
        unless: vec![],
    })
}
fn multi_plan_lifecycle_child_type_id() -> TypeId {
    TypeId::of::<MultiPlanLifecycleChild>()
}
fn multi_plan_lifecycle_child_type_name() -> &'static str {
    std::any::type_name::<MultiPlanLifecycleChild>()
}
fn on_multi_plan_lifecycle_child_body() -> Vec<String> {
    vec!["say multi-plan lifecycle child fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_multi_plan_lifecycle_child",
        id_override: None,
        make: on_multi_plan_lifecycle_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: multi_plan_lifecycle_child_chain,
            revoke: revoke_true,
            event_type_id: multi_plan_lifecycle_child_type_id,
            event_type_name: multi_plan_lifecycle_child_type_name,
            make_setup: multi_plan_lifecycle_child_setup,
        },
    }
}

// ── UnsatisfiableLifecycleChild: chains from Parent with Condition::any([])
//    (never holds) and owns lifecycle commands ─────────────────────────────

struct UnsatisfiableLifecycleChild;

fn unsatisfiable_lifecycle_child_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players set @s unsat_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @s unsat_post 1".to_string()],
    }
}
fn unsatisfiable_lifecycle_child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id,
        parent_type_name,
        parent_dispatch: parent_chain_dispatch,
        parent_setup,
        persistent: vec![],
        when: vec![Condition::any([])],
        unless: vec![],
    })
}
fn unsatisfiable_lifecycle_child_type_id() -> TypeId {
    TypeId::of::<UnsatisfiableLifecycleChild>()
}
fn unsatisfiable_lifecycle_child_type_name() -> &'static str {
    std::any::type_name::<UnsatisfiableLifecycleChild>()
}
fn on_unsatisfiable_lifecycle_child_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_unsatisfiable_lifecycle_child",
        id_override: None,
        make: on_unsatisfiable_lifecycle_child_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: unsatisfiable_lifecycle_child_chain,
            revoke: revoke_true,
            event_type_id: unsatisfiable_lifecycle_child_type_id,
            event_type_name: unsatisfiable_lifecycle_child_type_name,
            make_setup: unsatisfiable_lifecycle_child_setup,
        },
    }
}

// ── Nested chain: NestedA (root) -> NestedB (lifecycle) -> NestedC (lifecycle) ──

struct NestedA;

fn nested_a_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::raw("score @s nested_a_flag matches 1")),
    )
}
fn nested_a_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add nested_a_flag dummy".to_string()],
        pre_observation: vec![],
        post_observation: vec![],
    }
}
fn nested_a_type_id() -> TypeId {
    TypeId::of::<NestedA>()
}
fn nested_a_type_name() -> &'static str {
    std::any::type_name::<NestedA>()
}
fn on_nested_a_body() -> Vec<String> {
    vec!["say nested a fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_nested_a",
        id_override: None,
        make: on_nested_a_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: nested_a_dispatch,
            make_chain: no_chain,
            revoke: revoke_true,
            event_type_id: nested_a_type_id,
            event_type_name: nested_a_type_name,
            make_setup: nested_a_setup,
        },
    }
}

struct NestedB;

fn nested_b_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players set @s b_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @s b_post 1".to_string()],
    }
}
fn nested_b_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id: nested_a_type_id,
        parent_type_name: nested_a_type_name,
        parent_dispatch: || {
            sand_core::events::SandEventDispatch::Tick(nested_a_dispatch().unwrap())
        },
        parent_setup: nested_a_setup,
        persistent: vec![],
        when: vec![Condition::raw("score @s b_cond matches 1")],
        unless: vec![],
    })
}
fn nested_b_type_id() -> TypeId {
    TypeId::of::<NestedB>()
}
fn nested_b_type_name() -> &'static str {
    std::any::type_name::<NestedB>()
}
fn on_nested_b_body() -> Vec<String> {
    vec!["say nested b fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_nested_b",
        id_override: None,
        make: on_nested_b_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: nested_b_chain,
            revoke: revoke_true,
            event_type_id: nested_b_type_id,
            event_type_name: nested_b_type_name,
            make_setup: nested_b_setup,
        },
    }
}

struct NestedC;

fn nested_c_setup() -> EventSetup {
    EventSetup {
        objectives: vec![],
        pre_observation: vec!["scoreboard players set @s c_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @s c_post 1".to_string()],
    }
}
fn nested_c_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        parent_type_id: nested_b_type_id,
        parent_type_name: nested_b_type_name,
        parent_dispatch: || sand_core::events::SandEventDispatch::Chain(nested_b_chain().unwrap()),
        parent_setup: nested_b_setup,
        persistent: vec![],
        when: vec![Condition::raw("score @s c_cond matches 1")],
        unless: vec![],
    })
}
fn nested_c_type_id() -> TypeId {
    TypeId::of::<NestedC>()
}
fn nested_c_type_name() -> &'static str {
    std::any::type_name::<NestedC>()
}
fn on_nested_c_body() -> Vec<String> {
    vec!["say nested c fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_nested_c",
        id_override: None,
        make: on_nested_c_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: nested_c_chain,
            revoke: revoke_true,
            event_type_id: nested_c_type_id,
            event_type_name: nested_c_type_name,
            make_setup: nested_c_setup,
        },
    }
}

// ── Test helpers ────────────────────────────────────────────────────────────

fn records() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("chainlifecyclepack").expect("export should succeed");
    serde_json::from_str(&json).expect("export output should be valid JSON")
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
fn single_plan_child_pre_observation_runs_before_condition() {
    let records = records();
    let key = expected_key(single_plan_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    let pre_pos = content
        .find("scoreboard players operation @s current = @s source")
        .expect("pre_observation command must be present");
    let cond_pos = content
        .find("score @s sync < @s current")
        .expect("condition clause must be present");
    assert!(
        pre_pos < cond_pos,
        "pre_observation must run before the condition test: {content:?}"
    );
}

#[test]
fn single_plan_child_post_observation_runs_after_condition_attempt() {
    let records = records();
    let key = expected_key(single_plan_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    let cond_pos = content
        .find("score @s sync < @s current")
        .expect("condition clause must be present");
    let post_pos = content
        .find("scoreboard players operation @s sync = @s current")
        .expect("post_observation command must be present");
    assert!(
        cond_pos < post_pos,
        "post_observation must run after the condition test: {content:?}"
    );
}

#[test]
fn single_plan_child_post_observation_is_not_gated_by_the_condition() {
    let records = records();
    let key = expected_key(single_plan_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    // The post_observation line must be a bare, unconditional command in the
    // observe function — not embedded inside the `execute ... run function`
    // line that tests the condition — so it is structurally reached whether
    // or not the condition holds at runtime.
    let post_line = content
        .lines()
        .find(|l| l.contains("scoreboard players operation @s sync = @s current"))
        .expect("post_observation line must be present");
    assert_eq!(
        post_line.trim(),
        "scoreboard players operation @s sync = @s current",
        "post_observation must be a standalone command, not part of an execute clause: {post_line:?}"
    );
}

#[test]
fn unconditional_lifecycle_child_order_is_pre_dispatch_post() {
    let records = records();
    let key = expected_key(unconditional_lifecycle_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    let lines: Vec<&str> = content.lines().collect();
    let pre_idx = lines
        .iter()
        .position(|l| *l == "scoreboard players set @s uncond_pre 1")
        .expect("pre_observation must be present");
    let dispatch_idx = lines
        .iter()
        .position(|l| {
            l.contains("on_unconditional_lifecycle_child") || l.contains("__sand_event_dispatch")
        })
        .expect("dispatch call must be present");
    let post_idx = lines
        .iter()
        .position(|l| *l == "scoreboard players set @s uncond_post 1")
        .expect("post_observation must be present");
    assert!(
        pre_idx < dispatch_idx && dispatch_idx < post_idx,
        "expected pre -> dispatch -> post order, got: {lines:?}"
    );
    // Unconditional child dispatch must not be wrapped in an execute clause.
    assert!(!lines[dispatch_idx].contains("execute"));
}

#[test]
fn multi_plan_lifecycle_child_wraps_guarded_plans_between_pre_and_post() {
    let records = records();
    let key = expected_key(multi_plan_lifecycle_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    let pre_pos = content
        .find("scoreboard players set @s mp_pre 1")
        .expect("pre_observation must be present");
    let guard_reset_pos = content
        .find("scoreboard players set @s se_")
        .expect("guard reset must be present");
    let plan_a_pos = content
        .find("score @s mp_a matches 1")
        .expect("plan a must be present");
    let plan_b_pos = content
        .find("score @s mp_b matches 1")
        .expect("plan b must be present");
    let post_pos = content
        .find("scoreboard players set @s mp_post 1")
        .expect("post_observation must be present");

    assert!(pre_pos < guard_reset_pos);
    assert!(guard_reset_pos < plan_a_pos);
    assert!(plan_a_pos < plan_b_pos || plan_b_pos < plan_a_pos); // both present, order among plans is deterministic elsewhere
    assert!(plan_a_pos.max(plan_b_pos) < post_pos);

    // Each plan line must carry the guard clause, and reference the shared
    // edge function (which itself sets the guard and calls the child's
    // dispatch function) — not the observe function calling dispatch twice.
    for line in content
        .lines()
        .filter(|l| l.contains("score @s mp_a matches 1") || l.contains("score @s mp_b matches 1"))
    {
        assert!(line.contains("unless score @s se_") && line.contains("__sand_event_edge/"));
    }
}

#[test]
fn unsatisfiable_lifecycle_child_still_runs_pre_and_post_observation() {
    let records = records();
    let key = expected_key(unsatisfiable_lifecycle_child_type_name());
    let content = function_content(&records, &format!("__sand_event_observe/{key}"));

    assert!(content.contains("scoreboard players set @s unsat_pre 1"));
    assert!(content.contains("scoreboard players set @s unsat_post 1"));
    assert!(
        !content.contains("on_unsatisfiable_lifecycle_child"),
        "unreachable handler must never be referenced: {content:?}"
    );
    assert!(
        !has_function_with_prefix(
            &records,
            &format!(
                "__sand_event_dispatch/{}",
                expected_key(unsatisfiable_lifecycle_child_type_name())
            )
        ),
        "no dispatch function should be generated for an unreachable child"
    );
}

#[test]
fn nested_chain_lifecycle_ordering_at_every_depth() {
    let records = records();
    let a_key = expected_key(nested_a_type_name());
    let b_key = expected_key(nested_b_type_name());
    let c_key = expected_key(nested_c_type_name());

    // A's own dispatch reaches B's observe function.
    let a_dispatch = function_content(&records, &format!("__sand_event_dispatch/{a_key}"));
    assert!(a_dispatch.contains(&format!("__sand_event_observe/{b_key}")));

    // B's observe function: pre -> test -> dispatch -> post.
    let b_observe = function_content(&records, &format!("__sand_event_observe/{b_key}"));
    let b_pre = b_observe
        .find("scoreboard players set @s b_pre 1")
        .expect("b pre_observation");
    let b_cond = b_observe
        .find("score @s b_cond matches 1")
        .expect("b condition");
    let b_post = b_observe
        .find("scoreboard players set @s b_post 1")
        .expect("b post_observation");
    assert!(b_pre < b_cond && b_cond < b_post);

    // B's dispatch function (handlers + descendants) reaches C's observe
    // function — not B's observe function, which only wraps B's own test.
    let b_dispatch = function_content(&records, &format!("__sand_event_dispatch/{b_key}"));
    assert!(b_dispatch.contains("chainlifecyclepack:on_nested_b"));
    assert!(b_dispatch.contains(&format!("__sand_event_observe/{c_key}")));

    // C's observe function: pre -> test -> dispatch -> post.
    let c_observe = function_content(&records, &format!("__sand_event_observe/{c_key}"));
    let c_pre = c_observe
        .find("scoreboard players set @s c_pre 1")
        .expect("c pre_observation");
    let c_cond = c_observe
        .find("score @s c_cond matches 1")
        .expect("c condition");
    let c_post = c_observe
        .find("scoreboard players set @s c_post 1")
        .expect("c post_observation");
    assert!(c_pre < c_cond && c_cond < c_post);

    // C has a single handler and no descendants, so its dispatch resolves to
    // the direct handler call (the same no-wrapper optimization used
    // elsewhere) — the observe function's condition line calls it directly.
    assert!(c_observe.contains("chainlifecyclepack:on_nested_c"));
}

#[test]
fn parent_post_observation_still_runs_after_child_lifecycle_dispatch() {
    // Regression: the parent's own pre/post_observation ordering (established
    // before chained-child lifecycle support existed) must remain unchanged
    // even when one of its children owns lifecycle commands.
    let records = records();
    let key = expected_key(parent_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    let detect_pos = content
        .find("run function chainlifecyclepack:__sand_event_dispatch")
        .expect("detection line calling dispatch must be present");
    let sync_pos = content
        .find("execute as @a run scoreboard players operation @s parent_post = @s parent_flag")
        .expect("post_observation sync command must be present");
    assert!(
        detect_pos < sync_pos,
        "dispatch must run before parent's post_observation: {content:?}"
    );
}

#[test]
fn repeated_export_produces_identical_output() {
    let first =
        sand_core::try_export_components_json("chainlifecyclepack").expect("export should succeed");
    let second =
        sand_core::try_export_components_json("chainlifecyclepack").expect("export should succeed");
    assert_eq!(first, second);
}
