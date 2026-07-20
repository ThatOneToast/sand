//! Export-time rejection coverage for `inherit_entity`/`inherit_item`
//! declarations whose edge shape cannot honestly carry a same-cycle
//! borrowed participant (#264) — proven through the *real* export
//! pipeline, not just the isolated unit tests in
//! `sand-core/src/compiler/export/participant_transport.rs`.

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, TickEventDispatch,
};
use sand_core::participant::{EntityParticipantRole, EventParticipantPlan};
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

// ── Scenario 1: multi-parent (after_any) fan-in breaks the chain ───────────

struct MultiParentA;
impl SandEvent for MultiParentA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264d_a matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }
}
submit_handler!(MultiParentA, "on_multi_parent_a", "say a fired");

struct MultiParentB;
impl SandEvent for MultiParentB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264d_b matches 1"))
    }
}
submit_handler!(MultiParentB, "on_multi_parent_b", "say b fired");

struct AfterAnyChild;
impl SandEvent for AfterAnyChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after_any::<(MultiParentA, MultiParentB)>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<MultiParentA>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(AfterAnyChild, "on_after_any_child", "say after_any child fired");

// ── Test ────────────────────────────────────────────────────────────────

#[test]
fn after_any_fan_in_is_rejected_with_an_actionable_diagnostic() {
    let err = sand_core::try_export_components_json("diagpack_after_any").unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("MultiParentA") && message.contains("AfterAnyChild"),
        "diagnostic must name both the source and the child: {message}"
    );
    assert!(
        message.contains("after_any") || message.contains("multi-parent") || message.contains("fan-in"),
        "diagnostic must explain the multi-parent reason: {message}"
    );
}
