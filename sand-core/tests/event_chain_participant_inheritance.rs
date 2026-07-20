//! Export coverage for same-cycle participant propagation across chain
//! graph edges (#264): [`EventParticipantPlan::inherit_entity`]/
//! [`inherit_item`](sand_core::participant::EventParticipantPlan::inherit_item)
//! resolve to the *same* generated tag/storage reference the source event's
//! own accessor resolves to — not merely compatible capability metadata —
//! and reject, with an actionable export diagnostic, every edge shape that
//! cannot honestly carry that reference (multi-parent fan-in, a bounded
//! `.within(...)` window, transitive inherit-of-inherit).

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, TickEventDispatch,
};
use sand_core::participant::{EventParticipantPlan, EntityParticipantRole};
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
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
            fn tick() -> Option<TickEventDispatch> {
                match <$event as SandEvent>::dispatch().into() {
                    SandEventDispatch::Tick(tick) => Some(tick),
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
            fn participants() -> EventParticipantPlan {
                <$event as SandEvent>::participants()
            }
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: body,
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: tick,
                        make_chain: chain,
                        make_tracked: || None,
                        make_participants: participants,
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

// ── Root: tick root, captures the attacker directly ─────────────────────────

struct Root;
impl SandEvent for Root {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264_root matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }
}
submit_handler!(Root, "on_root", "say root fired");

// ── ChildA / ChildB: two siblings, both inherit Root's attacker ─────────────

struct ChildA;
impl SandEvent for ChildA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Root>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<Root>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(ChildA, "on_child_a", "say child a fired");

struct ChildB;
impl SandEvent for ChildB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Root>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<Root>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(ChildB, "on_child_b", "say child b fired");

// ── Grandchild: chains from ChildA, inherits directly from Root (multi-hop) ─

struct Grandchild;
impl SandEvent for Grandchild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<ChildA>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<Root>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(Grandchild, "on_grandchild", "say grandchild fired");

// ── Test helpers ──────────────────────────────────────────────────────────

fn expected_key(canonical_type_name: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in canonical_type_name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("inheritpack").expect("export should succeed");
    serde_json::from_str(&json).expect("export output should be valid JSON")
}

fn function_records(records: &[serde_json::Value]) -> Vec<&serde_json::Value> {
    records
        .iter()
        .filter(|r| r["dir"].as_str() == Some("function"))
        .collect()
}

fn all_function_content(records: &[serde_json::Value]) -> String {
    function_records(records)
        .into_iter()
        .filter_map(|r| r["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn source_tag(source_event_label: &str) -> String {
    // Mirrors ObservationSchema::tag()'s derivation
    // (sand-core/src/participant/observation.rs) — this test asserts on
    // the exact generated selector, so it must reproduce the same key
    // scheme, not just hope the two independently agree.
    format!("__sand_observed_{}", expected_key(source_event_label))
}

#[test]
fn root_captures_the_attacker_directly() {
    let content = all_function_content(&records());
    assert!(
        content.contains("execute on attacker"),
        "root must run the direct correlated-attacker observation: {content}"
    );
}

#[test]
fn siblings_both_resolve_to_roots_exact_tag() {
    // Neither child generates its own attacker capture — inherited entries
    // contribute zero setup commands (see `EventParticipantPlan::inherit_entity`).
    // The proof that inheritance actually works is that when each sibling's
    // *handler body itself* would reference the resolved selector (via
    // `Event::attacker()`/`EventParticipantPlan::resolve` in real user code),
    // it reconstructs the identical tag Root's own setup created — verified
    // here directly against the plan, the same mechanism `Event::attacker()`
    // calls through.
    let root_selector = Root::participants()
        .resolve(std::any::type_name::<Root>(), EntityParticipantRole::Attacker);
    let child_a_selector = ChildA::participants()
        .resolve(std::any::type_name::<ChildA>(), EntityParticipantRole::Attacker);
    let child_b_selector = ChildB::participants()
        .resolve(std::any::type_name::<ChildB>(), EntityParticipantRole::Attacker);
    let grandchild_selector = Grandchild::participants().resolve(
        std::any::type_name::<Grandchild>(),
        EntityParticipantRole::Attacker,
    );

    let root_tag = root_selector.available().unwrap().selector().to_string();
    assert_eq!(
        child_a_selector.available().unwrap().selector().to_string(),
        root_tag,
        "ChildA must resolve to the exact same selector Root's own accessor resolves to"
    );
    assert_eq!(
        child_b_selector.available().unwrap().selector().to_string(),
        root_tag,
        "ChildB (sibling) must resolve to the same selector as ChildA"
    );
    assert_eq!(
        grandchild_selector
            .available()
            .unwrap()
            .selector()
            .to_string(),
        root_tag,
        "Grandchild (multi-hop, inherits directly from Root) must resolve to the same selector"
    );
    assert!(
        root_tag.contains(&source_tag(std::any::type_name::<Root>())),
        "sanity: the shared selector must actually be Root's own generated tag: {root_tag}"
    );
}

#[test]
fn inheriting_children_generate_no_extra_participant_setup_or_cleanup() {
    let records = records();
    let child_a_key = expected_key(std::any::type_name::<ChildA>());
    let child_b_key = expected_key(std::any::type_name::<ChildB>());

    // Neither child needs a dedicated __sand_event_observe wrapper for
    // participant lifecycle purposes — inherited entries produce empty
    // pre/post_observation, so if the child has no *other* lifecycle
    // commands of its own, it takes the no-wrapper direct-call fast path
    // exactly like a plain lifecycle-free chain child.
    let has_observe_a = function_records(&records)
        .iter()
        .any(|r| r["path"].as_str().unwrap_or_default() == format!("__sand_event_observe/{child_a_key}"));
    let has_observe_b = function_records(&records)
        .iter()
        .any(|r| r["path"].as_str().unwrap_or_default() == format!("__sand_event_observe/{child_b_key}"));
    assert!(
        !has_observe_a,
        "ChildA must not need a lifecycle-wrapping observe function — inherit_entity contributes zero commands"
    );
    assert!(
        !has_observe_b,
        "ChildB must not need a lifecycle-wrapping observe function — inherit_entity contributes zero commands"
    );
}

#[test]
fn repeated_export_is_deterministic() {
    let first = sand_core::try_export_components_json("inheritpack").expect("export should succeed");
    let second = sand_core::try_export_components_json("inheritpack").expect("export should succeed");
    assert_eq!(first, second);
}

#[test]
fn every_declared_handler_is_present() {
    let records = records();
    let paths: Vec<&str> = function_records(&records)
        .iter()
        .filter_map(|r| r["path"].as_str())
        .collect();
    for expected in ["on_root", "on_child_a", "on_child_b", "on_grandchild"] {
        assert!(
            paths.iter().any(|p| p.starts_with(expected)),
            "missing generated function {expected} in {paths:?}"
        );
    }
}
