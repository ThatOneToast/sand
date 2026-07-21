//! Table-driven subject-capability audit of Sand's currently supported
//! event families (#230 Phase 8; narrowed by #274).
//!
//! This does not implement any participant recovery — it records, with a
//! test per family, exactly what `EventContextCapabilities::for_event`
//! honestly derives about the **subject** participant for each *built-in*
//! `SandEvent` marker type today. Every family below is currently
//! `AdvancementTrigger`- or `Tick(Players)`-backed, so every one of them
//! gets an exact player subject. Real entity/item participant declarations
//! (attacker, weapon, etc.) live on `EventParticipantPlan` instead — see
//! `sand-core/tests/participant_plan_export.rs` and
//! `sand-core/tests/event_chain_participant_inheritance_diag_*.rs` for that
//! coverage.

use sand_core::events::*;
use sand_core::participant::{EventContextCapabilities, ParticipantReliability, SubjectScope};

fn assert_exact_player_subject<E: SandEvent + 'static>(family: &str) {
    let caps = EventContextCapabilities::for_event::<E>();
    assert_eq!(
        caps.subject.scope,
        SubjectScope::Player,
        "{family} should have a player-scoped subject"
    );
    assert_eq!(
        caps.subject.reliability,
        ParticipantReliability::Exact,
        "{family} should have an exact subject"
    );
}

#[test]
fn player_join_and_leave_family() {
    // `OnJoinEvent`/`FirstJoinEvent` are `AdvancementEvent`-backed (handled
    // through `Event<T>`, not `SandEvent`) — outside this audit's scope,
    // which covers the `SandEvent`-backed families `EventContextCapabilities`
    // resolves against. `PlayerSleepEvent` is the nearest `SandEvent`
    // lifecycle-style family available.
    assert_exact_player_subject::<PlayerSleepEvent>("PlayerSleepEvent");
}

#[test]
fn player_state_tick_family() {
    assert_exact_player_subject::<PlayerSneakEvent>("PlayerSneakEvent");
    assert_exact_player_subject::<PlayerSprintEvent>("PlayerSprintEvent");
    assert_exact_player_subject::<PlayerSwimmingEvent>("PlayerSwimmingEvent");
    assert_exact_player_subject::<PlayerFlyingEvent>("PlayerFlyingEvent");
}

#[test]
fn kill_advancement_trigger_family() {
    // These carry a `killed_entity`/`killer` field on the *trigger* JSON
    // (see sand-core/src/event/trigger.rs), but the SandEvent marker types
    // in events/mod.rs still construct their triggers with entity/killing
    // blow left `None` — no typed entity participant is exposed by the
    // marker type today. Capturing that predicate data as a real
    // `EntityParticipant` is exactly the Phase 9 work this audit exists to
    // make visible when it lands.
    assert_exact_player_subject::<EntityKillEvent>("EntityKillEvent");
    assert_exact_player_subject::<PlayerKillEvent>("PlayerKillEvent");
}

#[test]
fn damage_family() {
    assert_exact_player_subject::<PlayerDamageEntityEvent>("PlayerDamageEntityEvent");
    assert_exact_player_subject::<EntityDamagePlayerEvent>("EntityDamagePlayerEvent");
}

#[test]
fn item_used_family() {
    assert_exact_player_subject::<ItemConsumeEvent>("ItemConsumeEvent");
    assert_exact_player_subject::<ItemCraftEvent>("ItemCraftEvent");
    assert_exact_player_subject::<ItemPickedUpEvent>("ItemPickedUpEvent");
}

#[test]
fn placed_block_family() {
    assert_exact_player_subject::<BlockPlaceEvent>("BlockPlaceEvent");
}

#[test]
fn interaction_family() {
    assert_exact_player_subject::<InteractWithEntityEvent>("InteractWithEntityEvent");
}

#[test]
fn projectile_family() {
    // No dedicated "projectile hit" SandEvent exists yet; ShotCrossbowEvent
    // and ChanneledLightningEvent are the closest built-in markers that
    // touch projectile-like vanilla criteria.
    assert_exact_player_subject::<ShotCrossbowEvent>("ShotCrossbowEvent");
    assert_exact_player_subject::<ChanneledLightningEvent>("ChanneledLightningEvent");
}

#[test]
fn ride_vehicle_family() {
    assert_exact_player_subject::<StartRidingEvent>("StartRidingEvent");
}

#[test]
fn advancement_backed_custom_event_gets_exact_subject() {
    struct CustomAdvancementEvent;
    impl SandEvent for CustomAdvancementEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::TickCondition("score @s custom_flag matches 1".into())
        }
    }
    assert_exact_player_subject::<CustomAdvancementEvent>("CustomAdvancementEvent");
}

#[test]
fn tick_backed_custom_event_gets_exact_subject() {
    struct CustomTickEvent;
    impl SandEvent for CustomTickEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    assert_exact_player_subject::<CustomTickEvent>("CustomTickEvent");
}
