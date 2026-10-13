//! Isolated registration fixture for the synchronous advancement boundary.
//! One test changes the consumer between exports, never during an export.
use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, PersistentEventCondition, PersistentSandEvent, SandEvent,
    SandEventDispatch, TickWindow,
};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;
use std::sync::atomic::{AtomicU8, Ordering};

static MODE: AtomicU8 = AtomicU8::new(0);
struct AdvancementRoot;
impl SandEvent for AdvancementRoot {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }
}
struct BridgeChild;
impl SandEvent for BridgeChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<AdvancementRoot>()
    }
}
struct BridgeGrandchild;
impl SandEvent for BridgeGrandchild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<BridgeChild>()
    }
}
impl PersistentSandEvent for BridgeGrandchild {
    fn persistent_condition() -> PersistentEventCondition {
        PersistentEventCondition::players(Condition::entity(
            sand_core::Target::self_().tag("ready"),
        ))
    }
}
struct TickRoot;
impl SandEvent for TickRoot {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}
struct OtherTickRoot;
impl SandEvent for OtherTickRoot {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
}
struct Consumer;
impl SandEvent for Consumer {
    fn dispatch() -> impl Into<SandEventDispatch> {
        match MODE.load(Ordering::SeqCst) {
            0 => SandEventDispatch::after_any::<(BridgeChild, TickRoot)>(),
            1 => SandEventDispatch::after_any::<(TickRoot, BridgeChild)>(),
            2 => SandEventDispatch::after_all::<(BridgeChild, TickRoot)>(),
            3 => SandEventDispatch::chain::<TickRoot>()
                .within::<BridgeChild>(TickWindow::new(3).unwrap()),
            4 => SandEventDispatch::after_any::<(BridgeGrandchild, TickRoot)>(),
            5 => SandEventDispatch::chain::<BridgeGrandchild>(),
            6 => SandEventDispatch::after_any::<(TickRoot, OtherTickRoot)>()
                .while_::<BridgeGrandchild>(),
            7 => SandEventDispatch::chain::<BridgeChild>()
                .within::<TickRoot>(TickWindow::new(3).unwrap()),
            8 => SandEventDispatch::chain::<BridgeChild>().after::<TickRoot>(),
            _ => unreachable!(),
        }
    }
}

fn no_trigger() -> Option<AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<sand_core::events::TickEventDispatch> {
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
            sand_core::inventory::submit! {
                EventDescriptor {
                    path: $path,
                    id_override: None,
                    make: || sand::component::IntoCommands::into_commands(body()),
                    dispatch: EventDispatch::Custom {
                        make_trigger: no_trigger,
                        make_condition: no_condition,
                        make_tick: no_tick,
                        make_chain: chain,
                        make_tracked: || None,
                        make_participants: || sand_core::participant::EventParticipantPlan::none(),
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

submit_handler!(Consumer, "on_consumer", "say consumer");

#[test]
fn staging_rejects_advancement_ancestry_but_preserves_synchronous_and_query_uses() {
    let mut first_any = None;
    for mode in [0, 1, 2, 3, 4, 7, 8] {
        MODE.store(mode, Ordering::SeqCst);
        let error = match sand_core::try_export_components_json("boundarypack") {
            Err(error) => error.to_string(),
            Ok(_) => {
                panic!("mode {mode}: accepted an advancement descendant as a tick-staged source")
            }
        };
        assert!(error.contains("Consumer"), "{error}");
        assert!(error.contains("AdvancementRoot"), "{error}");
        let source = if mode == 4 {
            "BridgeGrandchild"
        } else {
            "BridgeChild"
        };
        assert!(error.contains(source), "{error}");
        assert!(error.contains("outside the tick coordinator"), "{error}");
        assert!(error.contains("unbounded single-parent"), "{error}");
        assert_eq!(
            error,
            sand_core::try_export_components_json("boundarypack")
                .unwrap_err()
                .to_string()
        );
        if mode == 0 {
            first_any = Some(error);
        } else if mode == 1 {
            assert_eq!(
                first_any.as_ref().unwrap(),
                &error,
                "tuple order cannot change validation"
            );
        }
    }
    for mode in [5, 6] {
        MODE.store(mode, Ordering::SeqCst);
        let first =
            sand_core::try_export_components_json("boundarypack").expect("supported composition");
        assert_eq!(
            first,
            sand_core::try_export_components_json("boundarypack").unwrap()
        );
        assert!(first.contains("on_consumer"));
        if mode == 5 {
            assert!(first.contains("__sand_event_advancement_bridge"));
        } else {
            assert!(first.contains("@s[tag=ready]"));
            assert!(
                !first.contains("__sand_event_advancement_bridge"),
                "while_ does not dispatch its provider"
            );
        }
    }
}
