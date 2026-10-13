//! Export coverage for automatic `SandEvent::participants` integration.
//! Capture must execute as the selected subject, while global lifecycle work
//! remains outside the per-player handler tree.

use sand_core::condition::Condition;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::participant::EventParticipantPlan;
use sand_core::{EventDescriptor, EventDispatch};
use std::any::TypeId;

struct OnPlayerHurtViaPlan;

impl SandEvent for OnPlayerHurtViaPlan {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(Condition::raw("score @s p10_plan_trigger matches 1.."))
    }

    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_correlated_attacker()
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec!["scoreboard objectives add p10_plan_trigger dummy".into()],
            pre_observation: Vec::new(),
            post_observation: vec!["scoreboard players set @a p10_plan_trigger 0".into()],
        }
    }
}

fn no_trigger() -> Option<sand_core::AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}
fn revoke_true() -> bool {
    true
}
fn tick() -> Option<sand_core::events::TickEventDispatch> {
    sand_core::__private::event_dispatch_tick(OnPlayerHurtViaPlan::dispatch().into())
}
fn type_id() -> TypeId {
    TypeId::of::<OnPlayerHurtViaPlan>()
}
fn type_name() -> &'static str {
    std::any::type_name::<OnPlayerHurtViaPlan>()
}
fn body() -> Vec<String> {
    vec!["say checked".to_string()]
}
fn setup() -> EventSetup {
    OnPlayerHurtViaPlan::setup()
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_player_hurt_via_plan",
        id_override: None,
        make: || sand::component::IntoCommands::into_commands(body()),
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: OnPlayerHurtViaPlan::participants,
            revoke: revoke_true,
            event_type_id: type_id,
            event_type_name: type_name,
            make_setup: setup,
        },
    }
}

fn records() -> Vec<serde_json::Value> {
    let json = sand_core::try_export_components_json("planpack").expect("export succeeds");
    serde_json::from_str(&json).expect("valid export JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

fn check_key() -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in std::any::type_name::<OnPlayerHurtViaPlan>().bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:08x}")
}

#[test]
fn plan_capture_runs_inside_subject_dispatch_and_global_post_stays_outside() {
    let records = records();
    let key = check_key();
    let check = function(&records, &format!("__sand_event_check/{key}"));
    let dispatch = function(&records, &format!("__sand_event_dispatch/{key}"));
    assert!(
        !check.contains("execute on attacker"),
        "server check cannot capture a subject: {check}"
    );
    let call = format!(
        "execute as @a at @s if score @s p10_plan_trigger matches 1.. run function planpack:__sand_event_dispatch/{key}"
    );
    assert!(
        check.contains(&call),
        "capture wrapper must be called as the matched player: {check}"
    );
    assert!(
        check.find(&call).unwrap()
            < check
                .find("scoreboard players set @a p10_plan_trigger 0")
                .unwrap()
    );
    let capture = dispatch
        .find("execute on attacker run")
        .expect("subject capture");
    let handler = dispatch
        .find("function planpack:on_player_hurt_via_plan")
        .expect("handler");
    let cleanup = dispatch
        .find("tag @e[tag=__sand_observed_")
        .expect("cleanup");
    assert!(capture < handler && handler < cleanup, "{dispatch}");
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("planpack").unwrap();
    let second = sand_core::try_export_components_json("planpack").unwrap();
    assert_eq!(first, second);
}
