//! An advancement-backed `SandEvent` parent is representable as the sole
//! `after::<Parent>()` occurrence dependency of a chained child (#240 Phase
//! 6) — the parent is bridged directly from its own vanilla advancement
//! reward function rather than polled by `minecraft:tick`. Isolated in its
//! own test binary since `inventory` registrations are process-global.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SameCycleEventDependency, SameCycleEventRequirement,
    TickEventDispatch,
};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_trigger() -> Option<AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<TickEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> EventSetup {
    EventSetup::none()
}

struct AdvancementParent;

fn advancement_parent_dispatch() -> sand_core::events::SandEventDispatch {
    sand_core::events::SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
}
fn advancement_parent_type_id() -> TypeId {
    TypeId::of::<AdvancementParent>()
}
fn advancement_parent_type_name() -> &'static str {
    std::any::type_name::<AdvancementParent>()
}

struct ChildOfAdvancementParent;

fn child_chain() -> Option<ChainEventDispatch> {
    Some(ChainEventDispatch {
        occurrence: vec![SameCycleEventRequirement::After(SameCycleEventDependency {
            event_type_id: advancement_parent_type_id,
            event_type_name: advancement_parent_type_name,
            event_dispatch: advancement_parent_dispatch,
            event_setup: EventSetup::none,
            event_raw_setup: EventSetup::none,
            event_participants: || sand_core::participant::EventParticipantPlan::none(),
            event_revoke: || true,
        })],
        persistent: vec![],
        bounded: vec![],
        when: vec![],
        unless: vec![],
    })
}
fn child_type_id() -> TypeId {
    TypeId::of::<ChildOfAdvancementParent>()
}
fn child_type_name() -> &'static str {
    std::any::type_name::<ChildOfAdvancementParent>()
}
fn on_child_body() -> Vec<String> {
    vec!["say unreachable".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_child_of_advancement_parent",
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

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("advparentpack").expect("export succeeds");
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
fn advancement_backed_sole_after_parent_is_accepted_and_bridged() {
    let records = records();
    let parent_key = key(advancement_parent_type_name());

    let entry_path = format!("__sand_event_advancement_bridge/{parent_key}");
    let entry = function(&records, &entry_path);
    assert_eq!(
        entry,
        format!(
            "advancement revoke @s only advparentpack:{entry_path}\nfunction advparentpack:on_child_of_advancement_parent"
        ),
        "revoke runs first (existing repeatability contract), then the child's dispatch — under the same @s the vanilla reward mechanism already binds to the triggering player"
    );

    let advancement = records
        .iter()
        .find(|record| record["dir"] == "advancement" && record["path"] == entry_path)
        .expect("advancement JSON generated for the bridged parent");
    let content: serde_json::Value =
        serde_json::from_str(advancement["content"].as_str().unwrap()).unwrap();
    assert_eq!(content["criteria"]["event"]["trigger"], "minecraft:tick");
    assert_eq!(
        content["rewards"]["function"],
        format!("advparentpack:{entry_path}")
    );

    // The advancement-backed parent is never a graph node — no `se_{key}_o`
    // occurrence-mark objective, and no minecraft:tick coordinator entry, is
    // generated for it; its detection stays owned entirely by the
    // advancement/reward mechanism above.
    let objective = format!("scoreboard objectives add se_{parent_key}_o dummy");
    assert!(
        !records
            .iter()
            .filter_map(|record| record["content"].as_str())
            .flat_map(str::lines)
            .any(|line| line == objective),
        "advancement-backed parents never get a coordinator occurrence-mark objective"
    );
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("advparentpack").unwrap();
    let second = sand_core::try_export_components_json("advparentpack").unwrap();
    assert_eq!(first, second);
}
