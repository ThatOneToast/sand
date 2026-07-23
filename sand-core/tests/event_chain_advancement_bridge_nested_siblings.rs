//! Exact-output coverage for advancement-bridge parents (#269) combined with
//! sibling and nested (grandchild) same-cycle descendants (#280 item 6
//! re-audit). Proves:
//!
//! - the bridge entry's canonical order is revoke → participant setup →
//!   every dependent (siblings and nested descendants alike) → participant
//!   cleanup;
//! - two independent same-cycle children of the same bridge parent
//!   (siblings) both resolve the identical inherited selector;
//! - a grandchild — chained from a sibling, not from the bridge parent
//!   directly — can still inherit directly from the original bridge parent
//!   (a valid multi-hop ancestor walk, distinct from unsupported transitive
//!   inherit-of-inherit) and runs before cleanup;
//! - cleanup happens after every synchronous descendant, nested or not.
//!
//! Isolated in its own test binary since `inventory` registrations are
//! process-global.

use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, SandEventParticipants,
    TickEventDispatch,
};
use sand_core::participant::{EntityParticipantRole, EventParticipantPlan};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_condition() -> Option<String> {
    None
}
fn revoke_true() -> bool {
    true
}

macro_rules! submit_chain_handler {
    ($event:ty, $path:literal, $body:literal) => {
        const _: () = {
            fn no_trigger() -> Option<AdvancementTrigger> {
                None
            }
            fn chain() -> Option<ChainEventDispatch> {
                match <$event as SandEvent>::dispatch().into() {
                    SandEventDispatch::Chain(chain) => Some(chain),
                    _ => None,
                }
            }
            fn tick() -> Option<TickEventDispatch> {
                None
            }
            fn tracked() -> Option<sand_core::TrackedTransition> {
                None
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
                        make_tracked: tracked,
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

// ── BridgeParent: advancement-backed, declares a correlated killer ─────────

struct BridgeParent;
impl SandEvent for BridgeParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_killer()
    }
}

// ── Two siblings, both chained directly from BridgeParent ──────────────────

struct SiblingA;
impl SandEvent for SiblingA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<BridgeParent>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<BridgeParent>(EntityParticipantRole::Killer)
    }
}
submit_chain_handler!(SiblingA, "on_sibling_a", "say sibling a fired");

struct SiblingB;
impl SandEvent for SiblingB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<BridgeParent>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<BridgeParent>(EntityParticipantRole::Killer)
    }
}
submit_chain_handler!(SiblingB, "on_sibling_b", "say sibling b fired");

// ── Grandchild: chains from SiblingA, inherits directly from BridgeParent
//    (multi-hop ancestor walk — not transitive inherit-of-inherit, since it
//    names BridgeParent, the actual direct capturer, not SiblingA) ─────────

struct Grandchild;
impl SandEvent for Grandchild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<SiblingA>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<BridgeParent>(EntityParticipantRole::Killer)
    }
}
submit_chain_handler!(Grandchild, "on_grandchild", "say grandchild fired");

fn key(type_name: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in type_name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

fn records() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("bridgenestedpack").expect("export should succeed");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == path)
        .and_then(|r| r["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn bridge_entry_order_is_revoke_then_setup_then_every_descendant_then_cleanup() {
    // SiblingA and its own chain child (Grandchild) share one generated
    // dispatch-group function (since SiblingA has no lifecycle wrapper of
    // its own); SiblingB — the other direct child of BridgeParent — is
    // called separately. Both call sites must fall strictly between the
    // bridge's own participant setup and cleanup.
    let records = records();
    let parent_key = key(std::any::type_name::<BridgeParent>());
    let entry = function(
        &records,
        &format!("__sand_event_advancement_bridge/{parent_key}"),
    );

    let revoke_pos = entry.find("advancement revoke").expect("revoke present");
    let mark_pos = entry
        .find("execute on attacker run")
        .expect("BridgeParent's own killer setup must be spliced in");
    let sibling_b_call_pos = entry
        .find("bridgenestedpack:on_sibling_b")
        .expect("sibling b must be dispatched directly from the bridge entry");
    let dispatch_group_call_pos = entry
        .find("bridgenestedpack:__sand_event_dispatch/")
        .expect("sibling a's dispatch group (including its own chain child) must be called");
    let cleanup_pos = entry
        .rfind("tag @e[tag=__sand_observed_")
        .expect("cleanup must be spliced in after every descendant");

    assert!(revoke_pos < mark_pos, "revoke must run before setup");
    assert!(
        mark_pos < sibling_b_call_pos && mark_pos < dispatch_group_call_pos,
        "setup must run before any descendant dispatch: {entry}"
    );
    assert!(
        sibling_b_call_pos < cleanup_pos && dispatch_group_call_pos < cleanup_pos,
        "cleanup must run after every descendant dispatch: {entry}"
    );

    // The dispatch group itself must call SiblingA before Grandchild — the
    // nested descendant runs strictly after its direct ancestor, still
    // inside the same bridge cycle, before cleanup.
    let dispatch_group_path = entry
        .lines()
        .find(|line| line.contains("__sand_event_dispatch/"))
        .and_then(|line| line.strip_prefix("function bridgenestedpack:"))
        .expect("dispatch group call line must be a plain `function <ref>` line");
    let dispatch_group = function(&records, dispatch_group_path);
    let sibling_a_pos = dispatch_group
        .find("on_sibling_a")
        .expect("the dispatch group must call SiblingA");
    let grandchild_pos = dispatch_group
        .find("on_grandchild")
        .expect("the dispatch group must call Grandchild after SiblingA");
    assert!(
        sibling_a_pos < grandchild_pos,
        "SiblingA must run before its own nested child Grandchild: {dispatch_group}"
    );
}

#[test]
fn both_siblings_resolve_the_identical_inherited_selector() {
    let sibling_a_tag = SiblingA.killer().selector().to_string();
    let sibling_b_tag = SiblingB.killer().selector().to_string();
    assert_eq!(
        sibling_a_tag, sibling_b_tag,
        "both siblings of the same bridge parent must resolve to the identical selector"
    );
}

#[test]
fn nested_grandchild_inherits_directly_from_the_bridge_parent() {
    // A multi-hop ancestor walk (Grandchild -> SiblingA -> BridgeParent),
    // not transitive inherit-of-inherit (Grandchild names BridgeParent, the
    // actual direct capturer, not SiblingA).
    let grandchild_tag = Grandchild.killer().selector().to_string();
    let sibling_a_tag = SiblingA.killer().selector().to_string();
    assert_eq!(
        grandchild_tag, sibling_a_tag,
        "the nested grandchild must resolve to the same selector as its sibling ancestor chain"
    );

    let records = records();
    let grandchild_body = function(&records, "on_grandchild");
    assert!(
        grandchild_body.contains("say grandchild fired"),
        "grandchild's own handler command must be present: {grandchild_body}"
    );
}

#[test]
fn repeated_export_is_deterministic() {
    let first = sand_core::try_export_components_json("bridgenestedpack").unwrap();
    let second = sand_core::try_export_components_json("bridgenestedpack").unwrap();
    assert_eq!(first, second);
}
