//! Integration coverage for the structured `SandEventDispatch::Tick` lifecycle
//! (#239): proves through the *real* export pipeline that
//!
//! - an unconditional (`.every_tick()`, no `when`/`unless`) `SandEvent` is
//!   wired into `minecraft:tick` rather than silently dropped;
//! - a single typed condition renders one detection line;
//! - an OR/multi-plan condition renders one detection line per plan, guarded
//!   so at most one dispatch happens per player per tick;
//! - multiple `#[event]` handlers on the same `SandEvent` type share one
//!   generated detector/dispatch function instead of duplicating detection
//!   (setup dedup, keyed by `event_type_id` for in-process grouping);
//! - the generated detector/setup resource key is a deterministic function of
//!   the canonical event type name, not of the handler-path list — so it does
//!   not depend on how many handlers subscribe or in what order they were
//!   registered;
//! - fan-out dispatch bodies list handler paths in sorted order, independent
//!   of registration order;
//! - lifecycle setup objectives are emitted exactly once into `minecraft:load`;
//! - detection always runs before `post_observation` commands in the
//!   generated tick function (required so a jump-delta-style sync doesn't
//!   erase the value being compared before it's observed).

use sand_core::condition::Condition;
use sand_core::events::{EventSetup, TickEventDispatch};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

/// Reimplementation of the exporter's private FNV-1a hex hash, so tests can
/// independently compute the expected deterministic resource key from a
/// canonical type name and compare against the real generated path — proving
/// the key is a pure function of the type name, not of the handler set.
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
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
}

// ── PlayerJumpEvent: two handlers sharing one detector, delta-sync ordering ──

struct PlayerJumpEvent;

fn jump_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default().as_players().when(
            Condition::raw("block ~ ~-1 ~ minecraft:iron_block")
                .and(Condition::raw("score @s sync_jumps < @s jumps")),
        ),
    )
}

fn jump_setup() -> EventSetup {
    EventSetup {
        objectives: vec![
            "scoreboard objectives add jumps minecraft.custom:minecraft.jump".to_string(),
            "scoreboard objectives add sync_jumps dummy".to_string(),
        ],
        pre_observation: vec![],
        post_observation: vec![
            "execute as @a run scoreboard players operation @s sync_jumps = @s jumps".to_string(),
        ],
    }
}

fn jump_event_type_id() -> TypeId {
    TypeId::of::<PlayerJumpEvent>()
}
fn jump_event_type_name() -> &'static str {
    std::any::type_name::<PlayerJumpEvent>()
}

// Handler paths deliberately registered in reverse-sorted order (`zzz_` before
// `aaa_`) so the fan-out-sorted-order test proves the exporter sorts rather
// than preserving registration/inventory order.
fn zzz_jump_handler_body() -> Vec<String> {
    vec!["say jumped (z)".to_string()]
}

fn aaa_jump_handler_body() -> Vec<String> {
    vec!["say jumped (a)".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "zzz_jump_handler",
        id_override: None,
        make: zzz_jump_handler_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: jump_dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: jump_event_type_id,
            event_type_name: jump_event_type_name,
            make_setup: jump_setup,
        },
    }
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "aaa_jump_handler",
        id_override: None,
        make: aaa_jump_handler_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: jump_dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: jump_event_type_id,
            event_type_name: jump_event_type_name,
            make_setup: jump_setup,
        },
    }
}

// ── EveryTickEvent: unconditional dispatch, single handler ──────────────────

struct EveryTickEvent;

fn every_tick_dispatch() -> Option<TickEventDispatch> {
    Some(TickEventDispatch::default().as_players().every_tick())
}

fn every_tick_handler_body() -> Vec<String> {
    vec!["say every tick".to_string()]
}

fn every_tick_event_type_id() -> TypeId {
    TypeId::of::<EveryTickEvent>()
}
fn every_tick_event_type_name() -> &'static str {
    std::any::type_name::<EveryTickEvent>()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "every_tick_handler",
        id_override: None,
        make: every_tick_handler_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: every_tick_dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: every_tick_event_type_id,
            event_type_name: every_tick_event_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── OrConditionEvent: OR/multi-plan dispatch, single handler ────────────────

struct OrConditionEvent;

fn or_condition_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default().as_players().when(
            Condition::raw("score @s a matches 1").or(Condition::raw("score @s b matches 1")),
        ),
    )
}

fn or_condition_handler_body() -> Vec<String> {
    vec!["say or matched".to_string()]
}

fn or_condition_event_type_id() -> TypeId {
    TypeId::of::<OrConditionEvent>()
}
fn or_condition_event_type_name() -> &'static str {
    std::any::type_name::<OrConditionEvent>()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "or_condition_handler",
        id_override: None,
        make: or_condition_handler_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: or_condition_dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: or_condition_event_type_id,
            event_type_name: or_condition_event_type_name,
            make_setup: empty_setup,
        },
    }
}

// ── NeverFiresEvent: Condition::any([]) — declared but unsatisfiable ────────
//
// Detection can never happen, but pre/post_observation must still run every
// tick (per the EventSetup contract), and no dangling/dead dispatch function
// should be emitted for an unreachable detector.

struct NeverFiresEvent;

fn never_fires_dispatch() -> Option<TickEventDispatch> {
    Some(
        TickEventDispatch::default()
            .as_players()
            .when(Condition::any([])),
    )
}

fn never_fires_setup() -> EventSetup {
    EventSetup {
        objectives: vec!["scoreboard objectives add nf_seen dummy".to_string()],
        pre_observation: vec!["scoreboard players set @a nf_pre 1".to_string()],
        post_observation: vec!["scoreboard players set @a nf_post 1".to_string()],
    }
}

fn never_fires_handler_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

fn never_fires_event_type_id() -> TypeId {
    TypeId::of::<NeverFiresEvent>()
}
fn never_fires_event_type_name() -> &'static str {
    std::any::type_name::<NeverFiresEvent>()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "never_fires_handler",
        id_override: None,
        make: never_fires_handler_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: never_fires_dispatch,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: never_fires_event_type_id,
            event_type_name: never_fires_event_type_name,
            make_setup: never_fires_setup,
        },
    }
}

// ── Test helpers ──────────────────────────────────────────────────────────

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

// ── Tests: PlayerJumpEvent (two handlers, single condition, detect-before-sync) ──

#[test]
fn two_handlers_on_same_event_share_one_detector() {
    let records = records();
    let tick_tag = tag_values(&records, "minecraft:tick");

    let generated_checks: Vec<&String> = tick_tag
        .iter()
        .filter(|f| {
            f.contains("__sand_event_check") && f.contains(&expected_key(jump_event_type_name()))
        })
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
    let key = expected_key(jump_event_type_name());
    let load_tag = tag_values(&records, "minecraft:load");

    let generated_setups: Vec<&String> = load_tag
        .iter()
        .filter(|f| f.contains("__sand_event_setup") && f.contains(&key))
        .collect();
    assert_eq!(
        generated_setups.len(),
        1,
        "setup objectives must be deduplicated across handlers, got: {load_tag:?}"
    );

    let content = function_content(&records, &format!("__sand_event_setup/{key}"));
    assert!(content.contains("scoreboard objectives add jumps"));
    assert!(content.contains("scoreboard objectives add sync_jumps"));
}

#[test]
fn detection_runs_before_synchronization() {
    let records = records();
    let key = expected_key(jump_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    let detect_pos = content
        .find("if score @s sync_jumps < @s jumps")
        .expect("detection clause must be present");
    let sync_pos = content
        .find("execute as @a run scoreboard players operation @s sync_jumps = @s jumps")
        .expect("post_observation sync command must be present");
    assert!(
        detect_pos < sync_pos,
        "detection must run before the synchronizing post_observation command: {content:?}"
    );
}

#[test]
fn conditional_dispatch_sets_player_position_before_position_sensitive_clauses() {
    let records = records();
    let key = expected_key(jump_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    assert!(
        content.contains(
            "execute as @a at @s if block ~ ~-1 ~ minecraft:iron_block if score @s sync_jumps < @s jumps run function"
        ),
        "position-sensitive clauses must be evaluated after `at @s`: {content:?}"
    );
    assert!(
        !content.contains("execute as @a if block ~ ~-1 ~ minecraft:iron_block at @s"),
        "position-sensitive clauses must not precede `at @s`: {content:?}"
    );
}

#[test]
fn both_handler_bodies_are_reachable_from_the_shared_dispatch() {
    let records = records();
    let key = expected_key(jump_event_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(content.contains("jumppack:aaa_jump_handler"));
    assert!(content.contains("jumppack:zzz_jump_handler"));
}

#[test]
fn fan_out_dispatch_lists_handlers_in_sorted_order_not_registration_order() {
    // Registered zzz_ first, aaa_ second (see the submit! calls above) — the
    // generated dispatch body must list them alphabetically sorted
    // regardless, proving output does not depend on inventory/link order.
    let records = records();
    let key = expected_key(jump_event_type_name());
    let content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    let a_pos = content
        .find("jumppack:aaa_jump_handler")
        .expect("aaa handler call present");
    let z_pos = content
        .find("jumppack:zzz_jump_handler")
        .expect("zzz handler call present");
    assert!(
        a_pos < z_pos,
        "fan-out body must list handlers in sorted order regardless of registration order: {content:?}"
    );
}

#[test]
fn detector_and_setup_keys_are_deterministic_functions_of_canonical_type_name() {
    // The two-handler PlayerJumpEvent group's generated paths must exactly
    // match a key computed purely from its canonical type name — proving the
    // key does not depend on the handler-path list (join, count, or order).
    let records = records();
    let key = expected_key(jump_event_type_name());

    let fns = function_records(&records);
    assert!(
        fns.iter()
            .any(|r| r["path"].as_str() == Some(&format!("__sand_event_check/{key}"))),
        "expected deterministic check path __sand_event_check/{key}"
    );
    assert!(
        fns.iter()
            .any(|r| r["path"].as_str() == Some(&format!("__sand_event_setup/{key}"))),
        "expected deterministic setup path __sand_event_setup/{key}"
    );
}

// ── Tests: EveryTickEvent (unconditional dispatch) ──────────────────────────

#[test]
fn unconditional_every_tick_event_is_wired_into_minecraft_tick() {
    let records = records();
    let key = expected_key(every_tick_event_type_name());
    let tick_tag = tag_values(&records, "minecraft:tick");
    assert!(
        tick_tag
            .iter()
            .any(|f| f == &format!("jumppack:__sand_event_check/{key}")),
        "unconditional SandEvent must be registered in minecraft:tick, got: {tick_tag:?}"
    );
}

#[test]
fn unconditional_dispatch_has_no_if_or_unless_clause() {
    let records = records();
    let key = expected_key(every_tick_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));
    assert!(
        content.contains("execute as @a at @s run function jumppack:every_tick_handler"),
        "expected unconditional execute line, got: {content:?}"
    );
    assert!(
        !content.contains(" if ") && !content.contains(" unless "),
        "unconditional dispatch must not contain if/unless clauses: {content:?}"
    );
}

// ── Tests: OrConditionEvent (multi-plan, coalesced dispatch) ────────────────

#[test]
fn or_condition_emits_one_execute_line_per_plan() {
    let records = records();
    let key = expected_key(or_condition_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    let a_count = content.matches("if score @s a matches 1").count();
    let b_count = content.matches("if score @s b matches 1").count();
    assert_eq!(a_count, 1, "expected exactly one plan for `a`: {content:?}");
    assert_eq!(b_count, 1, "expected exactly one plan for `b`: {content:?}");
}

#[test]
fn or_condition_uses_a_reset_and_fired_guard_to_avoid_duplicate_dispatch() {
    let records = records();
    let key = expected_key(or_condition_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));

    // Guard reset must run before both detection lines.
    let reset_pos = content
        .find("scoreboard players set @a")
        .expect("guard reset command must be present");
    let first_plan_pos = content
        .find("if score @s a matches 1")
        .expect("plan for `a` must be present");
    assert!(
        reset_pos < first_plan_pos,
        "guard must reset before detection: {content:?}"
    );

    // Each plan's execute line must be guarded by `unless score @s <guard> matches 1`.
    for line in content.lines().filter(|l| l.contains("if score @s")) {
        assert!(
            line.contains("unless score @s") && line.contains("matches 1"),
            "each multi-plan execute line must carry the fired-guard clause: {line:?}"
        );
    }

    // The dispatch/fan-out function must mark the guard as fired.
    let dispatch_content = function_content(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(
        dispatch_content
            .lines()
            .next()
            .unwrap_or_default()
            .contains("scoreboard players set @s"),
        "dispatch function must set the fired-guard as its first command: {dispatch_content:?}"
    );
    assert!(dispatch_content.contains("jumppack:or_condition_handler"));
}

// ── Tests: NeverFiresEvent (Condition::any([])) ──────────────────────────────

#[test]
fn unsatisfiable_condition_still_runs_pre_and_post_observation() {
    let records = records();
    let key = expected_key(never_fires_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));
    assert!(content.contains("scoreboard players set @a nf_pre 1"));
    assert!(content.contains("scoreboard players set @a nf_post 1"));
}

#[test]
fn unsatisfiable_condition_emits_no_detection_line_or_dead_dispatch_function() {
    let records = records();
    let key = expected_key(never_fires_event_type_name());
    let content = function_content(&records, &format!("__sand_event_check/{key}"));
    assert!(
        !content.contains("never_fires_handler"),
        "unreachable handler must never be referenced from the check function: {content:?}"
    );
    let fns = function_records(&records);
    assert!(
        !fns.iter().any(|r| r["path"]
            .as_str()
            .unwrap_or_default()
            .starts_with(&format!("__sand_event_dispatch/{key}"))),
        "no dispatch/fan-out function should be generated for an unreachable detector"
    );
}

#[test]
fn unsatisfiable_condition_setup_objective_still_created() {
    let records = records();
    let key = expected_key(never_fires_event_type_name());
    let content = function_content(&records, &format!("__sand_event_setup/{key}"));
    assert!(content.contains("scoreboard objectives add nf_seen dummy"));
}

// ── Determinism ──────────────────────────────────────────────────────────────

#[test]
fn repeated_export_produces_identical_output() {
    let first = sand_core::try_export_components_json("jumppack").expect("export should succeed");
    let second = sand_core::try_export_components_json("jumppack").expect("export should succeed");
    assert_eq!(
        first, second,
        "repeated exports of the same registered events must be byte-identical"
    );
}
