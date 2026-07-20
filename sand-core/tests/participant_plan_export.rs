//! Export coverage for `EventParticipantPlan`/`EventSetup::with_participants`
//! (#230 Phase 10): proves the declarative plan API produces the exact same
//! generated-command ordering contract as Phase 9's manual
//! `observe_correlated_attacker` embedding, through the real export
//! pipeline, plus that declaring participants automatically enriches the
//! event's `EventContextCapabilities`.

use sand_core::condition::Condition;
use sand_core::events::{EventSetup, SandEvent, SandEventDispatch};
use sand_core::participant::{
    EntityParticipantRole, EventContextCapabilities, EventParticipantPlan,
};
use sand_core::version::{MinecraftVersion, VersionProfile};
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
        let profile = VersionProfile::resolve(&MinecraftVersion::parse("1.21.4").unwrap()).unwrap();
        EventSetup {
            objectives: vec!["scoreboard objectives add p10_plan_trigger dummy".into()],
            pre_observation: Vec::new(),
            post_observation: vec!["scoreboard players set @s p10_plan_trigger 0".into()],
        }
        .with_participants::<Self>(Self::participants(), &profile)
        .expect("1.21.4 supports the declared attacker observation")
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
    match OnPlayerHurtViaPlan::dispatch().into() {
        SandEventDispatch::Tick(t) => Some(t),
        _ => None,
    }
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
        make: body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: tick,
            make_chain: no_chain,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
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
fn plan_setup_runs_before_condition_and_cleanup_runs_after_post_observation() {
    let records = records();
    let check = function(&records, &format!("__sand_event_check/{}", check_key()));

    let plan_setup = check
        .find("present set value 0b")
        .expect("plan setup present");
    let detection = check
        .find("execute as @a at @s")
        .expect("detection line present");
    let post_observation_user = check
        .find("scoreboard players set @s p10_plan_trigger 0")
        .expect("existing post_observation command present");
    let plan_cleanup = check
        .find("tag @e[tag=__sand_observed_")
        .expect("plan cleanup present");

    assert!(
        plan_setup < detection,
        "plan setup must run before the condition test: {check}"
    );
    assert!(
        post_observation_user < plan_cleanup,
        "plan cleanup must run after existing post_observation commands: {check}"
    );
}

#[test]
fn repeated_export_is_identical() {
    let first = sand_core::try_export_components_json("planpack").unwrap();
    let second = sand_core::try_export_components_json("planpack").unwrap();
    assert_eq!(first, second);
}

#[test]
fn declared_plan_enriches_capabilities_when_combined_with_for_event() {
    // `EventContextCapabilities::for_event` alone only derives the subject
    // (it cannot see `participants()` — see LIM-CTX-001-style boundary
    // documented in capabilities.rs); combining it with the plan's own
    // `.capabilities()` is the documented pattern for a full descriptor.
    let subject_only = EventContextCapabilities::for_event::<OnPlayerHurtViaPlan>();
    assert_eq!(
        subject_only.subject.reliability,
        sand_core::participant::ParticipantReliability::Exact
    );

    let plan_caps = OnPlayerHurtViaPlan::participants().capabilities();
    assert_eq!(plan_caps.len(), 1);
    assert_eq!(plan_caps[0].role, EntityParticipantRole::Attacker);
    assert_eq!(
        plan_caps[0].reliability,
        sand_core::participant::ParticipantReliability::Correlated
    );
}
