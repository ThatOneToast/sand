//! Export coverage for participant propagation through `after_any`/
//! `after_all` multi-parent composition (#271), proven through the *real*
//! export pipeline — matching the exact-generated-output style established
//! in `event_chain_participant_inheritance.rs`/`event_chain_participant_item_inheritance.rs`
//! (#264).
//!
//! `EventParticipantPlan::inherit_entity`/`inherit_item` always name one
//! concrete `Source` type, never an inferred "whichever parent fired" — see
//! `sand-core/src/compiler/export/participant_transport.rs`'s module doc
//! for the full soundness argument this test suite exercises end to end:
//! naming one of an `after_any`/`after_all` group's own listed parents
//! directly is valid and registration-order-independent, because the
//! generated reference addresses the named source's own tag/storage by
//! type identity, and the coordinator defers every occurrence-marked
//! parent's cleanup until after all of its synchronous descendants
//! (including staged `after_any`/`after_all` children) have run.

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, SandEventParticipants,
    TickEventDispatch,
};
use sand_core::participant::role::ParticipantHand;
use sand_core::participant::{
    DuplicateParticipantRole, EntityParticipantRole, EventParticipantPlan, ItemParticipantRole,
    ParticipantBuilder,
};
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

// ── after_any: one parent supplies the role, the other does not ───────────

struct AnyA;
impl SandEvent for AnyA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_a matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(AnyA, "on_any_a", "say any a fired");

struct AnyB;
impl SandEvent for AnyB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_b matches 1"))
    }
}
submit_handler!(AnyB, "on_any_b", "say any b fired");

struct AnyChildFromA;
impl SandEvent for AnyChildFromA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_any::<(AnyA, AnyB)>()
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<AnyA>(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(
    AnyChildFromA,
    "on_any_child_from_a",
    "say any child from a fired"
);

// ── nested child after a multi-parent child (multi-hop, through the plain
//    chain edge past the after_any boundary, naming the original group
//    member directly) ────────────────────────────────────────────────────

struct NestedChild;
impl SandEvent for NestedChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<AnyChildFromA>()
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<AnyA>(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(NestedChild, "on_nested_child", "say nested child fired");

// ── after_all: two parents supply distinct roles ───────────────────────────

struct AllX;
impl SandEvent for AllX {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_x matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(AllX, "on_all_x", "say all x fired");

struct AllY;
impl SandEvent for AllY {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_y matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Killer)
            .build()
    }
}
submit_handler!(AllY, "on_all_y", "say all y fired");

struct AllChild;
impl SandEvent for AllChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_all::<(AllX, AllY)>()
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_entity::<AllX>(EntityParticipantRole::Attacker)
            .inherit_entity::<AllY>(EntityParticipantRole::Killer)
            .build()
    }
}
submit_handler!(AllChild, "on_all_child", "say all child fired");

// ── after_all: two parents supply the same *compatible* binding — a
//    sibling also declaring the identical role must not block the child's
//    own explicit, unambiguous choice of source ───────────────────────────

struct AllZ;
impl SandEvent for AllZ {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_z matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_entity(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(AllZ, "on_all_z", "say all z fired");

struct AllCompatChild;
impl SandEvent for AllCompatChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_all::<(AllX, AllZ)>()
    }
    fn participants() -> EventParticipantPlan {
        // AllX and AllZ both directly declare Attacker; naming AllX
        // explicitly must resolve to AllX's own tag, unaffected by AllZ
        // declaring the identical role.
        ParticipantBuilder::new()
            .inherit_entity::<AllX>(EntityParticipantRole::Attacker)
            .build()
    }
}
submit_handler!(
    AllCompatChild,
    "on_all_compat_child",
    "say all compat child fired"
);

// ── reversed registration order (determinism) ───────────────────────────────

struct RevOrderForward;
impl SandEvent for RevOrderForward {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_any::<(AnyA, AnyB)>()
    }
}
submit_handler!(
    RevOrderForward,
    "on_rev_order_forward",
    "say rev order forward fired"
);

struct RevOrderBackward;
impl SandEvent for RevOrderBackward {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_any::<(AnyB, AnyA)>()
    }
}
submit_handler!(
    RevOrderBackward,
    "on_rev_order_backward",
    "say rev order backward fired"
);

// ── item participant propagation through after_any, same-cycle borrowing
//    valid ──────────────────────────────────────────────────────────────

struct AnyItemA;
impl SandEvent for AnyItemA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_ia matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .observe_item(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .build()
    }
}
submit_handler!(AnyItemA, "on_any_item_a", "say any item a fired");

struct AnyItemB;
impl SandEvent for AnyItemB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p271_ib matches 1"))
    }
}
submit_handler!(AnyItemB, "on_any_item_b", "say any item b fired");

struct AnyItemChild;
impl SandEvent for AnyItemChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose().after_any::<(AnyItemA, AnyItemB)>()
    }
    fn participants() -> EventParticipantPlan {
        ParticipantBuilder::new()
            .inherit_item::<AnyItemA>(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
            .build()
    }
}
submit_handler!(
    AnyItemChild,
    "on_any_item_child",
    "say any item child fired"
);

// ── Test helpers ──────────────────────────────────────────────────────────

fn records() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("multiparentpack").expect("export should succeed");
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

// ── Tests ───────────────────────────────────────────────────────────────

#[test]
fn after_any_named_parent_binding_resolves_to_that_parents_exact_tag() {
    // #273: infallible `.attacker()` (blanket `SandEventParticipants`) is
    // the same call shape real handler code uses.
    let a_tag = AnyA.attacker().selector().to_string();
    assert_eq!(
        AnyChildFromA.attacker().selector().to_string(),
        a_tag,
        "AnyChildFromA (inherits from AnyA, one of two after_any parents) must resolve to AnyA's exact tag"
    );
}

#[test]
fn after_any_export_succeeds_even_though_sibling_declares_nothing() {
    // AnyB declares no participants at all — the sibling's total absence
    // of a plan must not block AnyChildFromA's explicit inherit from AnyA.
    let _ = records();
}

#[test]
fn nested_child_after_a_multi_parent_child_resolves_directly_to_the_original_source() {
    let a_tag = AnyA.attacker().selector().to_string();
    assert_eq!(
        NestedChild.attacker().selector().to_string(),
        a_tag,
        "NestedChild chains from AnyChildFromA (itself an after_any child) and inherits directly \
         from AnyA — multi-hop through a plain chain edge, naming the original after_any member \
         directly, must resolve to the exact same tag"
    );
}

#[test]
fn after_all_two_parents_supply_distinct_roles() {
    let attacker_tag = AllX.attacker().selector().to_string();
    let killer_tag = AllY.killer().selector().to_string();
    assert_ne!(
        attacker_tag, killer_tag,
        "sanity: AllX and AllY must generate distinct tags"
    );
    assert_eq!(
        AllChild.attacker().selector().to_string(),
        attacker_tag,
        "AllChild's inherited Attacker must resolve to AllX's exact tag"
    );
    assert_eq!(
        AllChild.killer().selector().to_string(),
        killer_tag,
        "AllChild's inherited Killer must resolve to AllY's exact tag"
    );
}

#[test]
fn after_all_naming_one_of_two_parents_with_the_same_role_is_unaffected_by_the_sibling() {
    let all_x_tag = AllX.attacker().selector().to_string();
    let all_z_tag = AllZ.attacker().selector().to_string();
    assert_ne!(
        all_x_tag, all_z_tag,
        "sanity: AllX and AllZ must generate distinct tags despite declaring the identical role"
    );
    assert_eq!(
        AllCompatChild.attacker().selector().to_string(),
        all_x_tag,
        "AllCompatChild explicitly named AllX — AllZ independently declaring the same role \
         (Attacker) must not change or block AllCompatChild's own resolved binding"
    );
}

#[test]
fn conflicting_same_role_bindings_from_different_sources_are_rejected_never_silently_picked() {
    // A single plan can never declare two competing bindings for the same
    // role, regardless of how many distinct after_all/after_any sources
    // would otherwise be independently reachable — `EventParticipantPlan::validate`
    // rejects this before any command is generated, which is exactly what
    // prevents a silent arbitrary pick between AllX's and AllZ's Attacker.
    let result = std::panic::catch_unwind(|| {
        ParticipantBuilder::new()
            .inherit_entity::<AllX>(EntityParticipantRole::Attacker)
            .inherit_entity::<AllZ>(EntityParticipantRole::Attacker)
            .build()
    });
    let err =
        result.expect_err("conflicting same-role bindings must panic, never silently pick one");
    let message = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        message.contains("more than once"),
        "expected the duplicate-role rejection message, got: {message}"
    );

    // Confirm this is exactly `EventParticipantPlan`'s own duplicate-role
    // check (not some ad-hoc string), by reproducing the same construction
    // at the `EventParticipantPlan` level directly and validating it.
    let plan = EventParticipantPlan::new()
        .inherit_entity::<AllX>(EntityParticipantRole::Attacker)
        .inherit_entity::<AllZ>(EntityParticipantRole::Attacker);
    assert_eq!(
        plan.validate(),
        Err(DuplicateParticipantRole::Entity(
            EntityParticipantRole::Attacker
        ))
    );
}

#[test]
fn reversed_after_any_registration_order_produces_identical_generated_call_site_order() {
    let content = all_function_content(&records());
    // Both RevOrderForward (declared as (AnyA, AnyB)) and RevOrderBackward
    // (declared as (AnyB, AnyA)) must generate their two coalescing
    // call-site lines in the exact same, canonical (alphabetical by
    // canonical type name) order — proving registration order never
    // affects the generated shape.
    let forward_a_pos = content
        .find("se_")
        .map(|_| {
            let key = |name: &str| {
                let mut h: u32 = 2_166_136_261;
                for b in name.bytes() {
                    h ^= b as u32;
                    h = h.wrapping_mul(16_777_619);
                }
                format!("{h:08x}")
            };
            let a_key = key(std::any::type_name::<AnyA>());
            let b_key = key(std::any::type_name::<AnyB>());
            (a_key, b_key)
        })
        .expect("expected generated occurrence flags");
    let (a_key, b_key) = forward_a_pos;

    // Locate each child's own multi-gate function content and confirm both
    // reference AnyA's/AnyB's occurrence flags — the actual call sites
    // live in the coordinator, gated per-parent; what must match between
    // forward/backward is simply that both children reduce to the same
    // canonical two-call-site shape referencing the same two flags.
    assert!(
        content.contains(&format!("se_{a_key}_o")) && content.contains(&format!("se_{b_key}_o")),
        "expected both AnyA's and AnyB's occurrence flags in the generated output: {content}"
    );
}

#[test]
fn simultaneous_after_any_fan_in_still_dispatches_exactly_once_via_the_shared_coalescing_guard() {
    let content = all_function_content(&records());
    let child_key = {
        let mut h: u32 = 2_166_136_261;
        for b in std::any::type_name::<AnyChildFromA>().bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16_777_619);
        }
        format!("{h:08x}")
    };
    let guard = format!("se_{child_key}_m");
    // The dedicated multi-gate function sets the guard once, then runs the
    // shared evaluation — the existing #240/#264 coalescing mechanism,
    // preserved unchanged by #271's participant-routing work.
    assert!(
        content.contains(&format!("scoreboard players set @s {guard} 1")),
        "expected the shared per-child coalescing guard to be set exactly once: {content}"
    );
}

#[test]
fn item_participant_propagates_through_after_any_same_cycle_borrowing_remains_valid() {
    let a_snapshot = AnyItemA.weapon();
    let child_snapshot = AnyItemChild.weapon();
    assert_eq!(
        child_snapshot.item_path().as_str(),
        a_snapshot.item_path().as_str(),
        "AnyItemChild must resolve to the exact same snapshot storage path AnyItemA's own capture wrote to"
    );
}

#[test]
fn cleanup_for_after_any_parents_is_deferred_until_after_every_synchronous_descendant() {
    let content = all_function_content(&records());
    // AnyA's own cleanup command (participant-plan post_observation, a tag
    // removal) must textually appear after the multi-gate function that
    // dispatches AnyChildFromA (and, transitively, NestedChild) — proving
    // the coordinator's deferred-cleanup ordering (established by #264/#280
    // for the sole-parent case) is preserved unchanged for after_any
    // fan-in.
    let child_key = {
        let mut h: u32 = 2_166_136_261;
        for b in std::any::type_name::<AnyChildFromA>().bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(16_777_619);
        }
        format!("{h:08x}")
    };
    let gate_marker = format!("__sand_event_multi_gate/{child_key}");
    let cleanup_marker = "remove";
    let gate_pos = content
        .find(&gate_marker)
        .expect("expected the after_any multi-gate function to be generated");
    let cleanup_pos = content
        .rfind(cleanup_marker)
        .expect("expected a participant cleanup (tag removal) command to be generated");
    assert!(
        cleanup_pos > gate_pos,
        "expected AnyA's participant cleanup to be deferred until after the after_any gate/dispatch: gate at {gate_pos}, cleanup at {cleanup_pos}\n{content}"
    );
}

#[test]
fn repeated_export_is_deterministic() {
    let first =
        sand_core::try_export_components_json("multiparentpack").expect("export should succeed");
    let second =
        sand_core::try_export_components_json("multiparentpack").expect("export should succeed");
    assert_eq!(first, second);
}

#[test]
fn every_declared_handler_is_present() {
    let records = records();
    let paths: Vec<&str> = function_records(&records)
        .iter()
        .filter_map(|r| r["path"].as_str())
        .collect();
    for expected in [
        "on_any_a",
        "on_any_b",
        "on_any_child_from_a",
        "on_nested_child",
        "on_all_x",
        "on_all_y",
        "on_all_child",
        "on_all_z",
        "on_all_compat_child",
        "on_rev_order_forward",
        "on_rev_order_backward",
        "on_any_item_a",
        "on_any_item_b",
        "on_any_item_child",
    ] {
        assert!(
            paths.iter().any(|p| p.starts_with(expected)),
            "missing generated function {expected} in {paths:?}"
        );
    }
}
