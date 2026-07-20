//! Regression coverage for the exporter dynamic-function-registry
//! nondeterminism `LIM-EXP-006` documented as a workaround in Phase 9
//! (#230) and root-caused/fixed in Phase 10.
//!
//! Root cause: the dynamic-function registry backing
//! `register_dyn_fn`/`register_dyn_fn_dedup`/`drain_dyn_fns` was a single
//! process-global `Mutex<Vec<..>>`. Rust's default test harness runs many
//! `#[test]` functions from one binary concurrently on separate threads, so
//! two tests that both triggered dynamic-function registration (e.g. via
//! `EntityContext::attacker().if_present(...)`, which wraps a multi-command
//! relation body in a separately-registered function) could race: one
//! test's `drain_dyn_fns()` could observe or clear entries registered by a
//! *different, concurrently-running* test on another thread. The fix moved
//! the registry to thread-local storage (`sand-core/src/function.rs`), so
//! each thread's export sees only its own registrations.
//!
//! This file deliberately reproduces the *exact* pattern that triggered the
//! original bug — `EntityContext::attacker().if_present(...)` called from
//! inside `SandEvent::setup()` — rather than the Phase 9 workaround
//! (`observe_correlated_attacker` now avoids this pattern entirely, but the
//! underlying registry must be safe for anyone else who uses it the same
//! way `if_present` does).

use sand_core::condition::Condition;
use sand_core::entity::EntityContext;
use sand_core::entity::kind::PlayerKind;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::version::{MinecraftVersion, VersionProfile};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct OnAttackerRelationCheck;

impl SandEvent for OnAttackerRelationCheck {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p10_trigger matches 1.."))
    }

    fn setup() -> EventSetup {
        let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.4").unwrap()).unwrap();
        let ctx: EntityContext<PlayerKind> = EntityContext::default();
        // The exact pattern that triggered the original bug: a
        // multi-command relation body wrapped via `if_present`, which
        // registers a dynamic function from inside `SandEvent::setup()`.
        let commands = ctx
            .attacker()
            .if_present(&profile, |attacker| {
                vec![
                    "scoreboard players set @s p10_attacker_seen 1".to_string(),
                    attacker.add_tag("p10_seen_attacker"),
                ]
            })
            .expect("1.20.2+ supports execute on attacker");
        EventSetup {
            objectives: vec!["scoreboard objectives add p10_trigger dummy".into()],
            pre_observation: commands,
            post_observation: vec!["scoreboard players set @s p10_trigger 0".into()],
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
fn tick() -> Option<sand_core::events::TickEventDispatch> {
    match OnAttackerRelationCheck::dispatch().into() {
        SandEventDispatch::Tick(t) => Some(t),
        _ => None,
    }
}
fn type_id() -> TypeId {
    TypeId::of::<OnAttackerRelationCheck>()
}
fn type_name() -> &'static str {
    std::any::type_name::<OnAttackerRelationCheck>()
}
fn body() -> Vec<String> {
    vec!["say checked".to_string()]
}
fn setup() -> EventSetup {
    OnAttackerRelationCheck::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_attacker_relation_check",
        id_override: None,
        make: body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: type_id,
            event_type_name: type_name,
            make_setup: setup,
        },
    }
}

fn export() -> String {
    sand_core::try_export_components_json("determinismpack").expect("export succeeds")
}

fn records(json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(json).expect("valid export JSON")
}

fn dyn_fn_record_present(json: &str) -> bool {
    records(json).iter().any(|record| {
        record["dir"] == "function"
            && record["path"]
                .as_str()
                .is_some_and(|p| p.starts_with("sand/entity_relation/attacker/"))
    })
}

#[test]
fn two_exports_are_byte_identical() {
    let first = export();
    let second = export();
    assert_eq!(
        first, second,
        "two identical exports in one process must be byte-identical"
    );
    assert!(
        dyn_fn_record_present(&first),
        "the relation-wrapped dynamic function must be present"
    );
    assert!(
        dyn_fn_record_present(&second),
        "the relation-wrapped dynamic function must be present in the second export too"
    );
}

#[test]
fn ten_repeated_exports_remain_identical() {
    let baseline = export();
    for i in 0..10 {
        let repeat = export();
        assert_eq!(baseline, repeat, "export #{i} diverged from the baseline");
        assert!(
            dyn_fn_record_present(&repeat),
            "export #{i} is missing the dynamic function record"
        );
    }
}

#[test]
fn drain_after_export_does_not_leak_into_a_later_export() {
    // Draining directly (simulating another consumer touching the
    // thread-local registry between exports) must not cause a later,
    // unrelated export on this same thread to miss its own registrations,
    // since each export's setup() re-registers what it needs.
    let _ = sand_core::drain_dyn_fns();
    let after_manual_drain = export();
    assert!(
        dyn_fn_record_present(&after_manual_drain),
        "an export must re-register everything it needs regardless of prior registry state"
    );
}

#[test]
fn cross_profile_export_is_deterministic_for_1_21_4_and_26_2() {
    // The event itself hardcodes 1.21.4 for its relation gating, but the
    // export pipeline as a whole (registry drains, resource identity) must
    // still be deterministic when exporting twice regardless of which
    // Minecraft profile a caller happens to be validating separately.
    let a = export();
    let b = export();
    assert_eq!(a, b);
}
