//! Export coverage proving `ItemSnapshot::capture`'s commands, embedded into
//! a tick-backed `SandEvent`'s `EventSetup::pre_observation`, land before
//! the condition test and before any cleanup — the documented "capture
//! before handler, before cleanup, before mutation" ordering contract
//! (#229 Phase 7). This is a manual integration example (Phase 7 does not
//! auto-wire capture into `#[event]`/the tick coordinator — see
//! `docs/items.md`), proving the pattern a `SandEvent` author would use
//! actually produces the documented order once exported.

use sand_core::condition::Condition;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::item::{ItemLocation, ItemSnapshot, SnapshotReliability, SnapshotSchema};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct OnHeldItemCheck;

impl SandEvent for OnHeldItemCheck {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p7_trigger matches 1.."))
    }

    fn setup() -> EventSetup {
        let (_, capture) = ItemSnapshot::capture(
            &ItemLocation::PlayerMainHand,
            SnapshotSchema::new(
                "itemsnappack:snapshots",
                std::any::type_name::<OnHeldItemCheck>(),
            ),
            SnapshotReliability::Exact,
        )
        .expect("valid location");
        EventSetup {
            objectives: vec!["scoreboard objectives add p7_trigger dummy".into()],
            pre_observation: capture,
            post_observation: vec!["scoreboard players set @s p7_trigger 0".into()],
        }
    }
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

fn on_held_item_check_tick() -> Option<sand_core::events::TickEventDispatch> {
    match OnHeldItemCheck::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
}
fn type_id() -> TypeId {
    TypeId::of::<OnHeldItemCheck>()
}
fn type_name() -> &'static str {
    std::any::type_name::<OnHeldItemCheck>()
}
fn body() -> Vec<String> {
    vec!["say checked".to_string()]
}
fn setup() -> EventSetup {
    OnHeldItemCheck::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_held_item_check",
        id_override: None,
        make: body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: on_held_item_check_tick,
            make_chain: no_chain,
            make_tracked: || None,
            revoke: revoke_true,
            event_type_id: type_id,
            event_type_name: type_name,
            make_setup: setup,
        },
    }
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("itemsnappack").expect("export succeeds");
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
fn capture_commands_run_before_condition_test_and_cleanup_after() {
    let records = records();
    // Root tick events are keyed by the same FNV-1a scheme as event graph
    // resource keys; recompute it the same way sand-core's own tests do to
    // find the generated check function.
    let key = {
        let mut hash: u32 = 2_166_136_261;
        for byte in std::any::type_name::<OnHeldItemCheck>().bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16_777_619);
        }
        format!("{hash:08x}")
    };
    let check = function(&records, &format!("__sand_event_check/{key}"));

    let capture_position = check
        .find("data modify storage itemsnappack:snapshots")
        .expect("capture commands are present");
    let detection_position = check
        .find("execute as @a at @s")
        .expect("detection line is present");
    let cleanup_position = check
        .find("scoreboard players set @s p7_trigger 0")
        .expect("post_observation cleanup is present");

    assert!(
        capture_position < detection_position,
        "capture must run before the condition test: {check}"
    );
    assert!(
        detection_position < cleanup_position,
        "condition test must run before post_observation cleanup: {check}"
    );

    // The presence-gated copy/mark commands are the very first four lines —
    // nothing Sand-generated runs before them.
    let first_line = check.lines().next().unwrap();
    assert!(
        first_line.starts_with("data modify storage itemsnappack:snapshots"),
        "capture must be the first command in the check function: {first_line}"
    );
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("itemsnappack").unwrap();
    let second = sand_core::try_export_components_json("itemsnappack").unwrap();
    assert_eq!(first, second);
}
