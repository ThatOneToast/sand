//! Automatic participant-plan integration for advancement-backed combat
//! events (#230) — proves `AdvancementEvent::participants()` is applied by
//! the export pipeline without any manual `EventSetup`/`with_participants`
//! wiring from the handler author.

use sand_core::cmd;
use sand_core::events::{EntityDamagePlayerEvent, PlayerDamageEntityEvent};
use sand_core::participant::ParticipantAvailability;
use sand_macros::event;

#[event]
pub fn on_hurt_by_entity(event: sand_core::event::Event<EntityDamagePlayerEvent>) {
    let attacker: ParticipantAvailability<sand_core::participant::EntityParticipant> =
        event.attacker();
    cmd::raw(format!(
        "# attacker available = {}",
        attacker.is_available()
    ))
}

#[event]
pub fn on_hurt_entity(event: sand_core::event::Event<PlayerDamageEntityEvent>) {
    let weapon: ParticipantAvailability<sand_core::item::ItemSnapshot> = event.weapon();
    cmd::raw(format!("# weapon available = {}", weapon.is_available()))
}
