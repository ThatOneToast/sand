//! Export coverage for tracked-transition `SandEvent` participant wiring
//! (#270). Before this fix, neither `make_setup()` nor `make_participants()`
//! was ever called for `CustomDispatchBackend::Tracked` — a tracked event's
//! own declared participant plan was silently and completely ignored.
//! Isolated in its own test binary since `inventory` registrations are
//! process-global.

use sand_core::events::{SandEvent, SandEventDispatch, SandEventParticipants};
use sand_core::participant::{EntityParticipantRole, EventParticipantPlan};
use sand_core::{EventDescriptor, EventDispatch, TrackedSource, TrackedTransition, TransitionKind};
use std::any::TypeId;

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<sand_core::events::TickEventDispatch> {
    None
}
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> sand_core::events::EventSetup {
    sand_core::events::EventSetup::none()
}

/// A tracked-transition `SandEvent` that directly declares a correlated
/// attacker — structurally identical to how a `TickLifecycle` event would.
struct TrackedWithAttacker;
impl SandEvent for TrackedWithAttacker {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::Tracked(TrackedTransition::new(
            "p270_tracker",
            TrackedSource::Score {
                description: "p270 score source",
                objective: "p270_score",
                criterion: "dummy",
            },
            TransitionKind::ScoreIncreased,
        ))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }
}

fn tracked_dispatch() -> Option<TrackedTransition> {
    match <TrackedWithAttacker as SandEvent>::dispatch().into() {
        SandEventDispatch::Tracked(t) => Some(t),
        _ => None,
    }
}
fn tracked_type_id() -> TypeId {
    TypeId::of::<TrackedWithAttacker>()
}
fn tracked_type_name() -> &'static str {
    std::any::type_name::<TrackedWithAttacker>()
}
fn tracked_participants() -> EventParticipantPlan {
    <TrackedWithAttacker as SandEvent>::participants()
}
fn tracked_body() -> Vec<String> {
    vec!["say tracked handler fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_tracked_with_attacker",
        id_override: None,
        make: tracked_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: tracked_dispatch,
            make_participants: tracked_participants,
            revoke: revoke_true,
            event_type_id: tracked_type_id,
            event_type_name: tracked_type_name,
            make_setup: empty_setup,
        },
    }
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("trackedp270").expect("export should succeed");
    serde_json::from_str(&json).expect("export output should be valid JSON")
}

fn handler_body(records: &[serde_json::Value]) -> &str {
    records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "on_tracked_with_attacker")
        .and_then(|r| r["content"].as_str())
        .expect("on_tracked_with_attacker function must exist")
}

#[test]
fn declared_participant_plan_produces_setup_and_cleanup_around_the_handler_body() {
    let records = records();
    let body = handler_body(&records);

    let mark = body
        .find("execute on attacker run")
        .expect("declared attacker observation must be spliced into the handler body");
    let dispatch = body
        .find("say tracked handler fired")
        .expect("the handler's own commands must still be present");
    let cleanup = body
        .rfind("tag @e[tag=__sand_observed_")
        .expect("cleanup must be spliced in after the handler's own commands");

    assert!(
        mark < dispatch,
        "participant setup must run before the handler's own commands: {body}"
    );
    assert!(
        dispatch < cleanup,
        "cleanup must run after the handler's own commands: {body}"
    );
}

#[test]
fn accessor_resolves_to_the_declared_attacker() {
    // #273: the infallible accessor (via the blanket `SandEventParticipants`
    // impl) must resolve, proving the plan really was applied, not just
    // present as dead metadata.
    let selector = TrackedWithAttacker.attacker().selector().to_string();
    assert!(
        selector.contains("__sand_observed_"),
        "expected a correlated-observation tag selector: {selector}"
    );
    let _ = EntityParticipantRole::Attacker; // keep import honest if body ever removed
}

/// A tracked-transition `SandEvent` with no declared participants — proves
/// the default empty plan remains a true no-op for this backend, matching
/// every other dispatch backend's empty-plan behavior.
struct TrackedWithoutParticipants;
impl SandEvent for TrackedWithoutParticipants {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::Tracked(TrackedTransition::new(
            "p270_tracker_bare",
            TrackedSource::Score {
                description: "p270 bare score source",
                objective: "p270_bare_score",
                criterion: "dummy",
            },
            TransitionKind::ScoreIncreased,
        ))
    }
}

fn bare_tracked_dispatch() -> Option<TrackedTransition> {
    match <TrackedWithoutParticipants as SandEvent>::dispatch().into() {
        SandEventDispatch::Tracked(t) => Some(t),
        _ => None,
    }
}
fn bare_tracked_type_id() -> TypeId {
    TypeId::of::<TrackedWithoutParticipants>()
}
fn bare_tracked_type_name() -> &'static str {
    std::any::type_name::<TrackedWithoutParticipants>()
}
fn bare_tracked_participants() -> EventParticipantPlan {
    <TrackedWithoutParticipants as SandEvent>::participants()
}
fn bare_tracked_body() -> Vec<String> {
    vec!["say bare tracked handler fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_tracked_without_participants",
        id_override: None,
        make: bare_tracked_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: bare_tracked_dispatch,
            make_participants: bare_tracked_participants,
            revoke: revoke_true,
            event_type_id: bare_tracked_type_id,
            event_type_name: bare_tracked_type_name,
            make_setup: empty_setup,
        },
    }
}

#[test]
fn empty_participant_plan_is_a_true_no_op() {
    let records = records();
    let body = records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == "on_tracked_without_participants")
        .and_then(|r| r["content"].as_str())
        .expect("on_tracked_without_participants function must exist");
    assert_eq!(
        body, "say bare tracked handler fired",
        "an event with no declared participants must generate exactly its own commands, unwrapped"
    );
}

#[test]
fn repeated_export_is_deterministic() {
    let first =
        sand_core::try_export_components_json("trackedp270").expect("export should succeed");
    let second =
        sand_core::try_export_components_json("trackedp270").expect("export should succeed");
    assert_eq!(first, second);
}
