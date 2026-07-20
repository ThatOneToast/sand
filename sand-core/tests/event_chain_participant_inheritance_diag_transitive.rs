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

// ── Scenario: transitive inherit-of-inherit is rejected ────────────────────

struct TransitiveRoot;
impl SandEvent for TransitiveRoot {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264d_tr matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }
}
submit_handler!(TransitiveRoot, "on_transitive_root", "say root fired");

struct TransitiveMid;
impl SandEvent for TransitiveMid {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<TransitiveRoot>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new()
            .inherit_entity::<TransitiveRoot>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(TransitiveMid, "on_transitive_mid", "say mid fired");

struct TransitiveLeaf;
impl SandEvent for TransitiveLeaf {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<TransitiveMid>()
    }
    fn participants() -> EventParticipantPlan {
        // Wrong: names the intermediate re-borrower, not the actual capturing
        // ancestor — must be rejected, not silently resolved through Mid.
        EventParticipantPlan::new().inherit_entity::<TransitiveMid>(EntityParticipantRole::Attacker)
    }
}
submit_handler!(TransitiveLeaf, "on_transitive_leaf", "say leaf fired");

// ── Test ────────────────────────────────────────────────────────────────

#[test]
fn transitive_inheritance_is_rejected_with_an_actionable_diagnostic() {
    let err = sand_core::try_export_components_json("diagpack_transitive").unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("TransitiveMid") && message.contains("TransitiveLeaf"),
        "diagnostic must name both the source and the child: {message}"
    );
    assert!(
        message.contains("transitive"),
        "diagnostic must explain the transitive-inheritance reason: {message}"
    );
}
