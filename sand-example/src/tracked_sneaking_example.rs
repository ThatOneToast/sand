//! Start/stop sneaking through Sand's generic tracked-transition backend.
//!
//! The vanilla signal is the generated entity predicate
//! `flags.is_sneaking`. It is sampled once per online player per tick. The
//! first sample establishes a baseline without firing; later false→true and
//! true→false edges call the handlers below.

use sand_core::cmd;
use sand_core::event::vanilla::{PlayerStartsSneaking, PlayerStopsSneaking};
use sand_macros::event;

#[event]
pub fn on_start_sneaking(event: sand_core::event::Event<PlayerStartsSneaking>) {
    let _ = event;
    cmd::raw("tellraw @s {\"text\":\"Sneaking started\"}")
}

#[event]
pub fn on_stop_sneaking(event: sand_core::event::Event<PlayerStopsSneaking>) {
    let _ = event;
    cmd::raw("tellraw @s {\"text\":\"Sneaking stopped\"}")
}

#[event]
pub fn on_start_sneaking_audit(event: sand_core::event::Event<PlayerStartsSneaking>) {
    let _ = event;
    cmd::raw("say second start handler")
}
