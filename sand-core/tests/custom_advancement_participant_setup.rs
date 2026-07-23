//! Export coverage for `CustomDispatchBackend::Advancement`'s participant
//! wiring (#280 item 4). A `SandEvent` whose `dispatch()` resolves directly
//! to an advancement trigger (not via the typed `AdvancementEvent` path, and
//! not as a same-cycle bridge parent) previously never called
//! `make_participants()` at all — a declared plan was silently and
//! completely ignored, the same class of bug #264/#270 fixed for
//! `Chain`/`Tracked` dispatch. Isolated in its own test binary since
//! `inventory` registrations are process-global.

use sand_core::events::{SandEvent, SandEventDispatch, SandEventParticipants};
use sand_core::participant::EventParticipantPlan;
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

fn no_condition() -> Option<String> {
    None
}
fn no_tick() -> Option<sand_core::events::TickEventDispatch> {
    None
}
fn no_chain() -> Option<sand_core::events::ChainEventDispatch> {
    None
}
fn no_tracked() -> Option<sand_core::TrackedTransition> {
    None
}
fn revoke_true() -> bool {
    true
}
fn empty_setup() -> sand_core::events::EventSetup {
    sand_core::events::EventSetup::none()
}

/// A `SandEvent` dispatched directly as an advancement trigger, with a
/// directly-declared weapon snapshot — structurally identical to the typed
/// `AdvancementEvent` path, just reached through `SandEvent::dispatch()`
/// returning `SandEventDispatch::AdvancementTrigger` instead.
struct CustomAdvancementWithWeapon;
impl SandEvent for CustomAdvancementWithWeapon {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::Tick)
    }
    fn participants() -> EventParticipantPlan {
        EventParticipantPlan::new().observe_weapon()
    }
}

fn trigger() -> Option<AdvancementTrigger> {
    match <CustomAdvancementWithWeapon as SandEvent>::dispatch().into() {
        SandEventDispatch::AdvancementTrigger(t) => Some(t),
        _ => None,
    }
}
fn type_id() -> TypeId {
    TypeId::of::<CustomAdvancementWithWeapon>()
}
fn type_name() -> &'static str {
    std::any::type_name::<CustomAdvancementWithWeapon>()
}
fn participants() -> EventParticipantPlan {
    <CustomAdvancementWithWeapon as SandEvent>::participants()
}
fn body() -> Vec<String> {
    vec!["say custom advancement handler fired".to_string()]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "on_custom_advancement_with_weapon",
        id_override: None,
        make: body,
        dispatch: EventDispatch::Custom {
            make_trigger: trigger,
            make_condition: no_condition,
            make_tick: no_tick,
            make_chain: no_chain,
            make_tracked: no_tracked,
            make_participants: participants,
            revoke: revoke_true,
            event_type_id: type_id,
            event_type_name: type_name,
            make_setup: empty_setup,
        },
    }
}

fn records() -> Vec<serde_json::Value> {
    let json =
        sand_core::try_export_components_json("customadvpack").expect("export should succeed");
    serde_json::from_str(&json).expect("export output should be valid JSON")
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

#[test]
fn declared_weapon_plan_wraps_the_body_setup_before_dispatch() {
    let records = records();
    let body = function(&records, "on_custom_advancement_with_weapon/body");

    let capture_pos = body
        .find("SelectedItem")
        .expect("weapon snapshot capture must be spliced into the body");
    let handler_pos = body
        .find("say custom advancement handler fired")
        .expect("the handler's own command must still be present");

    assert!(
        capture_pos < handler_pos,
        "participant setup must run before the handler's own commands: {body}"
    );
}

#[test]
fn accessor_resolves_to_the_declared_weapon_snapshot() {
    // #273: the infallible accessor (via the blanket `SandEventParticipants`
    // impl) must resolve, proving the plan really was applied end-to-end,
    // not merely present as dead metadata in the generated body.
    let snapshot = CustomAdvancementWithWeapon.weapon();
    assert!(
        !snapshot.storage().is_empty(),
        "expected a real snapshot storage path"
    );
}

#[test]
fn entry_revokes_before_calling_the_body() {
    let records = records();
    let entry = function(&records, "on_custom_advancement_with_weapon");
    assert_eq!(
        entry,
        "advancement revoke @s only customadvpack:on_custom_advancement_with_weapon\nfunction customadvpack:on_custom_advancement_with_weapon/body"
    );
}

#[test]
fn repeated_export_is_deterministic() {
    let first =
        sand_core::try_export_components_json("customadvpack").expect("export should succeed");
    let second =
        sand_core::try_export_components_json("customadvpack").expect("export should succeed");
    assert_eq!(first, second);
}
