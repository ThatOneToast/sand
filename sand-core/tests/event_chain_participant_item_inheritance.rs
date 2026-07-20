//! Export coverage for `EventParticipantPlan::inherit_item` — the item-
//! snapshot counterpart to `inherit_entity` (#264). Proves a same-cycle
//! chain child resolves to the *same* generated snapshot storage its
//! parent's own weapon capture used, not a fresh (empty) one of its own.

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, SandEvent, SandEventDispatch, TickEventDispatch,
};
use sand_core::participant::role::ParticipantHand;
use sand_core::participant::{EventParticipantPlan, ItemParticipantRole};
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

struct Root;
impl SandEvent for Root {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p264i_root matches 1"))
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_weapon()
    }
}
submit_handler!(Root, "on_root", "say root fired");

struct Child;
impl SandEvent for Child {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<Root>()
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new()
            .inherit_item::<Root>(ItemParticipantRole::Weapon, ParticipantHand::MainHand)
    }
}
submit_handler!(Child, "on_child", "say child fired");

#[test]
fn child_resolves_to_roots_exact_snapshot_storage() {
    let root_snapshot = Root::participants()
        .resolve_item(std::any::type_name::<Root>(), ItemParticipantRole::Weapon)
        .available()
        .expect("Root directly captures the weapon");
    let child_snapshot = Child::participants()
        .resolve_item(std::any::type_name::<Child>(), ItemParticipantRole::Weapon)
        .available()
        .expect("Child inherits the weapon from Root");

    assert_eq!(
        child_snapshot.item_path().as_str(),
        root_snapshot.item_path().as_str(),
        "Child must resolve to the exact same snapshot storage path Root's own capture wrote to"
    );
}

#[test]
fn export_generates_exactly_one_capture_no_child_capture_duplication() {
    let json =
        sand_core::try_export_components_json("itemheritpack").expect("export should succeed");
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).expect("valid JSON");
    let capture_count = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .filter_map(|r| r["content"].as_str())
        .filter(|content| content.contains("SelectedItem"))
        .count();
    assert_eq!(
        capture_count, 1,
        "only Root's own setup should capture the weapon snapshot — inherit_item must not generate a second capture"
    );
}
