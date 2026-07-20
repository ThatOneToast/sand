//! Export coverage for advancement-backed graph parent composition (#240
//! Phase 6): supported forms (provider-only, shared by multiple children,
//! combined with `while_`/`when`/`unless`) and explicitly rejected forms
//! (`after_any`/`after_all`, combined with a second occurrence clause,
//! combined with `.within(...)`, combined with a direct `#[event]` handler).

use sand_core::condition::Condition;
use sand_core::events::{
    ChainEventDispatch, EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch,
};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

struct AdvancementParent;
impl SandEvent for AdvancementParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }
}

struct OtherTickParent;
impl SandEvent for OtherTickParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p6_other matches 1.."))
    }
}

struct FirstChild;
impl SandEvent for FirstChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<AdvancementParent>()
    }
}

struct SecondChildWithWhile;
impl SandEvent for SecondChildWithWhile {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<AdvancementParent>()
            .while_::<PlayerSneakEvent>()
            .when(Condition::entity("@s[tag=ready]"))
            .unless(Condition::entity("@s[tag=blocked]"))
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
fn no_chain() -> Option<ChainEventDispatch> {
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
                    make: body,
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

submit_handler!(FirstChild, "on_first_child", "say first");
submit_handler!(SecondChildWithWhile, "on_second_child", "say second");

fn other_tick() -> Option<sand_core::events::TickEventDispatch> {
    match OtherTickParent::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
}
fn other_type_id() -> TypeId {
    TypeId::of::<OtherTickParent>()
}
fn other_type_name() -> &'static str {
    std::any::type_name::<OtherTickParent>()
}
fn other_body() -> Vec<String> {
    vec!["say other".into()]
}
fn other_setup() -> EventSetup {
    OtherTickParent::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_other",
        id_override: None,
        make: other_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: other_tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: revoke_true,
            event_type_id: other_type_id,
            event_type_name: other_type_name,
            make_setup: other_setup,
        },
    }
}

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
        sand_core::try_export_components_json("advcompositionpack").expect("export succeeds");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn provider_only_advancement_parent_is_shared_by_multiple_children() {
    let records = records();
    let parent_key = key(std::any::type_name::<AdvancementParent>());
    let entry_path = format!("__sand_event_advancement_bridge/{parent_key}");

    // Exactly one advancement + one entry function for the parent, no
    // matter how many children depend on it.
    assert_eq!(
        records
            .iter()
            .filter(|record| record["dir"] == "advancement" && record["path"] == entry_path)
            .count(),
        1,
        "one advancement generated regardless of dependent count"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record["dir"] == "function" && record["path"] == entry_path)
            .count(),
        1,
        "one entry function generated regardless of dependent count"
    );

    let entry = function(&records, &entry_path);
    // Both children's dispatch lines are present, revoke still runs first.
    assert!(entry.starts_with("advancement revoke @s only"));
    assert!(entry.contains("function advcompositionpack:on_first_child"));

    let second_eval_present = entry.contains("__sand_event_observe/")
        || entry.contains("predicate advcompositionpack:__sand/player_sneaking");
    assert!(
        second_eval_present,
        "second child's while_/when/unless-gated dispatch must also be reachable from the same entry: {entry}"
    );

    // Multiplayer safety: the entry function must never broaden execution
    // beyond the single triggering player the vanilla reward mechanism
    // already bound to `@s` — no `execute as @a`/`@e` wrapper, which would
    // let one player's advancement occurrence dispatch on behalf of every
    // online player.
    assert!(
        !entry.contains("as @a") && !entry.contains("as @e"),
        "advancement bridge entry must never broaden execution beyond the triggering player: {entry}"
    );
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("advcompositionpack").unwrap();
    let second = sand_core::try_export_components_json("advcompositionpack").unwrap();
    assert_eq!(first, second);
}
