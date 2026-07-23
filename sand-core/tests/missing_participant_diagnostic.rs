//! Export coverage for the missing-participant structured diagnostic (#280
//! item 2): an infallible accessor (`event.killer()`/`.weapon()`/…) called
//! on an event whose plan does not declare that role must fail `sand
//! build`'s export with a clean `SAND-EVENT-PARTICIPANT` diagnostic — never
//! an unhandled panic/backtrace, never partial datapack output. Covers both
//! handler forms: `Event<E: AdvancementEvent>` and a bare `SandEvent`
//! marker. Isolated in its own test binary since `inventory` registrations
//! are process-global and a rejected export must not pollute other tests.

use sand_core::event::{AdvancementEvent, Event, EventId, EventReset, EventVisibility};
use sand_core::events::{SandEvent, SandEventDispatch, SandEventParticipants};
use sand_core::{AdvancementTrigger, EventDescriptor, EventDispatch};
use std::any::TypeId;

// ── Handler form 1: Event<E: AdvancementEvent>, no participants declared ────

struct SomeAdvancementEvent;
impl AdvancementEvent for SomeAdvancementEvent {
    type Trigger = AdvancementTrigger;
    fn trigger() -> Self::Trigger {
        AdvancementTrigger::Tick
    }
    fn id() -> EventId {
        EventId::Auto
    }
    fn reset() -> EventReset {
        EventReset::AfterFire
    }
    fn visibility() -> EventVisibility {
        EventVisibility::Hidden
    }
    // No `participants()` override — defaults to `EventParticipantPlan::none()`.
}

fn adv_trigger() -> AdvancementTrigger {
    <SomeAdvancementEvent as AdvancementEvent>::trigger()
}
fn adv_revoke() -> bool {
    true
}
fn adv_participants() -> sand_core::participant::EventParticipantPlan {
    <SomeAdvancementEvent as AdvancementEvent>::participants()
}
fn invalid_advancement_body() -> Vec<String> {
    // Mirrors: `#[event] fn invalid_advancement(event: Event<SomeAdvancementEvent>) { event.killer(); }`
    let event = Event::<SomeAdvancementEvent>::context();
    let killer = event.killer();
    vec![format!("# unreachable: {}", killer.selector())]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "invalid_advancement",
        id_override: None,
        make: invalid_advancement_body,
        dispatch: EventDispatch::Advancement {
            make_trigger: adv_trigger,
            revoke: adv_revoke,
            guard: None,
            make_participants: adv_participants,
            event_type_name: (|| std::any::type_name::<SomeAdvancementEvent>()) as fn() -> &'static str,
        },
    }
}

// ── Handler form 2: bare SandEvent marker, no participants declared ────────

struct SomeSandEvent;
impl SandEvent for SomeSandEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players()
    }
    // No `participants()` override.
}

fn no_trigger() -> Option<AdvancementTrigger> {
    None
}
fn no_condition() -> Option<String> {
    None
}
fn sand_tick() -> Option<sand_core::events::TickEventDispatch> {
    match <SomeSandEvent as SandEvent>::dispatch().into() {
        SandEventDispatch::Tick(tick) => Some(tick),
        _ => None,
    }
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
fn sand_type_id() -> TypeId {
    TypeId::of::<SomeSandEvent>()
}
fn sand_type_name() -> &'static str {
    std::any::type_name::<SomeSandEvent>()
}
fn sand_participants() -> sand_core::participant::EventParticipantPlan {
    <SomeSandEvent as SandEvent>::participants()
}
fn sand_setup() -> sand_core::events::EventSetup {
    <SomeSandEvent as SandEvent>::setup()
}
fn invalid_sand_body() -> Vec<String> {
    // Mirrors: `#[event] fn invalid_sand(event: SomeSandEvent) { event.killer(); }`
    let event = SomeSandEvent;
    let killer = event.killer();
    vec![format!("# unreachable: {}", killer.selector())]
}

sand_core::inventory::submit! {
    EventDescriptor {
        path: "invalid_sand",
        id_override: None,
        make: invalid_sand_body,
        dispatch: EventDispatch::Custom {
            make_trigger: no_trigger,
            make_condition: no_condition,
            make_tick: sand_tick,
            make_chain: no_chain,
            make_tracked: no_tracked,
            make_participants: sand_participants,
            revoke: revoke_true,
            event_type_id: sand_type_id,
            event_type_name: sand_type_name,
            make_setup: sand_setup,
        },
    }
}

#[test]
fn typed_advancement_handler_missing_participant_fails_with_structured_diagnostic() {
    // Only this one handler's descriptor is exercised by giving it its own
    // namespace — `every_declared_handler_is_present`-style cross-pollution
    // from `invalid_sand` in the same export is fine, since both descriptors
    // are process-global regardless, and both are expected to fail export
    // here (the export must fail on the *first* one it walks into, since
    // both are invalid).
    let err = sand_core::try_export_components_json("missingparticipantpack")
        .expect_err("export must fail, not silently succeed or write partial output");
    let message = err.to_string();

    assert!(
        message.contains("SAND-EVENT-PARTICIPANT"),
        "diagnostic must use the structured SAND-EVENT-PARTICIPANT code: {message}"
    );
    assert!(
        message.contains("Required role: EntityParticipantRole::Killer"),
        "diagnostic must name the required role: {message}"
    );
    // One of the two invalid handlers — whichever inventory iteration
    // reaches first — must be named as the failing handler.
    assert!(
        message.contains("Handler: invalid_advancement")
            || message.contains("Handler: invalid_sand"),
        "diagnostic must name the failing handler: {message}"
    );
    assert!(
        message.contains("ParticipantBuilder::new()"),
        "diagnostic must suggest a ParticipantBuilder declaration: {message}"
    );
    assert!(
        !message.contains("panicked at"),
        "diagnostic must not leak a raw Rust panic message: {message}"
    );
    assert!(
        !message.to_lowercase().contains("backtrace"),
        "diagnostic must not leak a raw Rust backtrace: {message}"
    );
}

#[test]
fn export_failure_does_not_leave_a_poisoned_panic_hook_for_later_exports() {
    // A second, independent export attempt (of a different, non-conflicting
    // namespace) after a caught MissingParticipantPanic must behave
    // normally — proving the panic-hook swap in `invoke_event_handler_body`
    // fully restores the previous hook even on the failing path, and the
    // guarding mutex is not left poisoned/locked.
    let _ = sand_core::try_export_components_json("missingparticipantpack");
    let second = sand_core::try_export_components_json("missingparticipantpack2");
    assert!(
        second.is_err(),
        "the same invalid handlers are still registered, so this must still fail cleanly, not hang or panic"
    );
}
