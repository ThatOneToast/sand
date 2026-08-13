//! Common player-state transition events (#49) through the generic
//! tracked-transition backend — movement/posture, fire, gamemode, health,
//! and status effects all share the same generated-provider architecture
//! that backs `PlayerStartSneakingEvent`/`PlayerStopSneakingEvent`.

use sand_core::cmd;
use sand_core::events::{
    EffectStarted, EffectStopped, PlayerEnteredCreativeEvent, PlayerExitedCreativeEvent,
    PlayerHealthChangedEvent, PlayerLowHealthEvent, PlayerRecoveredHealthEvent,
    PlayerStartSprintingEvent, PlayerStopSprintingEvent, Speed,
};
use sand_macros::on_event;

#[on_event]
pub fn on_start_sprinting(event: sand_core::event::Event<PlayerStartSprintingEvent>) {
    let _ = event;
    cmd::raw("say sprint started")
}

#[on_event]
pub fn on_start_sprinting_second_handler(
    event: sand_core::event::Event<PlayerStartSprintingEvent>,
) {
    let _ = event;
    cmd::raw("say sprint started (second handler)")
}

#[on_event]
pub fn on_stop_sprinting(event: sand_core::event::Event<PlayerStopSprintingEvent>) {
    let _ = event;
    cmd::raw("say sprint stopped")
}

#[on_event]
pub fn on_enter_creative(event: sand_core::event::Event<PlayerEnteredCreativeEvent>) {
    let _ = event;
    cmd::raw("say entered creative")
}

#[on_event]
pub fn on_exit_creative(event: sand_core::event::Event<PlayerExitedCreativeEvent>) {
    let _ = event;
    cmd::raw("say exited creative")
}

#[on_event]
pub fn on_health_changed(event: sand_core::event::Event<PlayerHealthChangedEvent>) {
    let _ = event;
    cmd::raw("say health changed")
}

#[on_event]
pub fn on_low_health(event: sand_core::event::Event<PlayerLowHealthEvent<6>>) {
    let _ = event;
    cmd::raw("say low health")
}

#[on_event]
pub fn on_recovered_health(event: sand_core::event::Event<PlayerRecoveredHealthEvent<6>>) {
    let _ = event;
    cmd::raw("say recovered health")
}

#[on_event]
pub fn on_speed_start(event: sand_core::event::Event<EffectStarted<Speed>>) {
    let _ = event;
    cmd::raw("say speed started")
}

#[on_event]
pub fn on_speed_stop(event: sand_core::event::Event<EffectStopped<Speed>>) {
    let _ = event;
    cmd::raw("say speed stopped")
}
