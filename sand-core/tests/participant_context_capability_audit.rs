//! Table-driven capability audit of Sand's currently supported event
//! families (#230 Phase 8).
//!
//! This does not implement any participant recovery — it records, with a
//! test per family, exactly what `EventContextCapabilities::for_event`
//! honestly derives for each *built-in* `SandEvent` marker type today.
//! Every family below is currently `AdvancementTrigger`- or
//! `Tick(Players)`-backed, so every one of them gets an exact player
//! subject and zero entity/item/location capabilities — there is no
//! attacker/victim/interacted-entity/projectile-owner recovery backend
//! anywhere in Sand yet (that is #230's Phase 9). This test exists so a
//! future Phase 9 change that starts populating `entities`/`items`/
//! `locations` for one of these types is a visible, deliberate diff here,
//! not a silent behavior change.

use sand_core::events::*;
use sand_core::participant::{EventContextCapabilities, ParticipantReliability, SubjectScope};

fn assert_exact_player_subject_only<E: SandEvent + 'static>(family: &str) {
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
    assert!(
        caps.entities.is_empty(),
        "{family} should declare no entity participants yet"
    );
    assert!(
        caps.items.is_empty(),
        "{family} should declare no item participants yet"
    );
    assert!(
        caps.locations.is_empty(),
        "{family} should declare no location participants yet"
    );
}

#[test]
fn player_join_and_leave_family() {
    // `OnJoinEvent`/`FirstJoinEvent` are `AdvancementEvent`-backed (handled
    // through `Event<T>`, not `SandEvent`) — outside this audit's scope,
    // which covers the `SandEvent`-backed families `EventContextCapabilities`
    // resolves against. `PlayerSleepEvent` is the nearest `SandEvent`
    // lifecycle-style family available.
    assert_exact_player_subject_only::<PlayerSleepEvent>("PlayerSleepEvent");
}

#[test]
fn player_state_tick_family() {
    assert_exact_player_subject_only::<PlayerSneakEvent>("PlayerSneakEvent");
    assert_exact_player_subject_only::<PlayerSprintEvent>("PlayerSprintEvent");
    assert_exact_player_subject_only::<PlayerSwimmingEvent>("PlayerSwimmingEvent");
    assert_exact_player_subject_only::<PlayerFlyingEvent>("PlayerFlyingEvent");
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
    assert_exact_player_subject_only::<EntityKillEvent>("EntityKillEvent");
    assert_exact_player_subject_only::<PlayerKillEvent>("PlayerKillEvent");
}

#[test]
fn damage_family() {
    assert_exact_player_subject_only::<PlayerDamageEntityEvent>("PlayerDamageEntityEvent");
    assert_exact_player_subject_only::<EntityDamagePlayerEvent>("EntityDamagePlayerEvent");
}

#[test]
fn item_used_family() {
    assert_exact_player_subject_only::<ItemConsumeEvent>("ItemConsumeEvent");
    assert_exact_player_subject_only::<ItemCraftEvent>("ItemCraftEvent");
    assert_exact_player_subject_only::<ItemPickedUpEvent>("ItemPickedUpEvent");
}

#[test]
fn placed_block_family() {
    assert_exact_player_subject_only::<BlockPlaceEvent>("BlockPlaceEvent");
}

#[test]
fn interaction_family() {
    assert_exact_player_subject_only::<InteractWithEntityEvent>("InteractWithEntityEvent");
}

#[test]
fn projectile_family() {
    // No dedicated "projectile hit" SandEvent exists yet; ShotCrossbowEvent
    // and ChanneledLightningEvent are the closest built-in markers that
    // touch projectile-like vanilla criteria.
    assert_exact_player_subject_only::<ShotCrossbowEvent>("ShotCrossbowEvent");
    assert_exact_player_subject_only::<ChanneledLightningEvent>("ChanneledLightningEvent");
}

#[test]
fn ride_vehicle_family() {
    assert_exact_player_subject_only::<StartRidingEvent>("StartRidingEvent");
}

#[test]
fn advancement_backed_custom_event_gets_exact_subject() {
    struct CustomAdvancementEvent;
    impl SandEvent for CustomAdvancementEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::TickCondition("score @s custom_flag matches 1".into())
        }
    }
    assert_exact_player_subject_only::<CustomAdvancementEvent>("CustomAdvancementEvent");
}

#[test]
fn tick_backed_custom_event_gets_exact_subject() {
    struct CustomTickEvent;
    impl SandEvent for CustomTickEvent {
        fn dispatch() -> impl Into<SandEventDispatch> {
            SandEventDispatch::tick().as_players()
        }
    }
    assert_exact_player_subject_only::<CustomTickEvent>("CustomTickEvent");
}
