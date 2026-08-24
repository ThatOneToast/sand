use sand::{EntityStateEnum, armor_event, datapack_component, function, on_event, schedule};
use sand::events::OnJoinEvent;

#[function]
pub fn generated_function() {}

#[datapack_component(Tick)]
pub fn generated_component() {}

#[on_event]
pub fn generated_event(_event: OnJoinEvent) {}

#[armor_event(Equip, slot = Head)]
pub fn generated_armor_event() {}

#[schedule(ticks = 4, every = 2)]
pub fn generated_schedule() {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, EntityStateEnum)]
pub enum GeneratedPhase {
    Idle,
    Active = 4,
}

#[doc(hidden)]
pub mod __private {
    include!(concat!(env!("OUT_DIR"), "/api_enforcement.rs"));
}
