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
                sand_core::__private::event_dispatch_chain(<$event as SandEvent>::dispatch().into())
            }
            fn tick() -> Option<TickEventDispatch> {
                sand_core::__private::event_dispatch_tick(<$event as SandEvent>::dispatch().into())
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

// ── Scenario: bounded `.within(...)` breaks the chain ──────────────────────

struct BoundedParent;
impl SandEvent for BoundedParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264d_bp matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_killer()
    }
}
submit_handler!(
    BoundedParent,
    "on_bounded_parent",
    "say bounded parent fired"
);

struct WithinTrigger;
impl SandEvent for WithinTrigger {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264d_wt matches 1"))
    }
}
submit_handler!(
    WithinTrigger,
    "on_within_trigger",
    "say within trigger fired"
);

struct WithinChild;
impl SandEvent for WithinChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<WithinTrigger>()
            .within::<BoundedParent>(sand_core::events::TickWindow::new(20).unwrap())
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().inherit_entity::<BoundedParent>(EntityParticipantRole::Killer)
    }
}
submit_handler!(WithinChild, "on_within_child", "say within child fired");

// ── Test ────────────────────────────────────────────────────────────────

#[test]
fn bounded_within_is_rejected_with_an_actionable_diagnostic() {
    let err = sand_core::try_export_components_json("diagpack_within").unwrap_err();
    let message = err.to_string();
    assert!(
        message.contains("BoundedParent") && message.contains("WithinChild"),
        "diagnostic must name both the source and the child: {message}"
    );
    assert!(
        message.contains("within")
            || message.contains("bounded")
            || message.contains("tick boundary"),
        "diagnostic must explain the bounded-window reason: {message}"
    );
}
