//! Automatic participant-plan integration for advancement-backed combat
//! events (#230) — proves `AdvancementEvent::participants()` is applied by
//! the export pipeline without any manual `EventSetup`/`with_participants`
//! wiring from the handler author, and that the resulting accessors are
//! infallible typed values, not a `ParticipantAvailability` wrapper (#273).

use sand_core::cmd;
use sand_core::events::{EntityDamagePlayerEvent, PlayerDamageEntityEvent};
use sand_macros::event;

#[event]
pub fn on_hurt_by_entity(event: sand_core::event::Event<EntityDamagePlayerEvent>) {
    let attacker: sand_core::participant::EntityParticipant = event.attacker();
    cmd::raw(format!("# attacker = {}", attacker.selector()))
}

#[event]
pub fn on_hurt_entity(event: sand_core::event::Event<PlayerDamageEntityEvent>) {
    let weapon: sand_core::item::ItemSnapshot = event.weapon();
    cmd::raw(format!("# weapon storage = {}", weapon.storage()))
}
