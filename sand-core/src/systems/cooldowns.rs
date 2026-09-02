//! Auto-tick system for registered cooldowns (`systems-cooldowns` feature).
//!
//! Register cooldowns here so the export pipeline can generate a single
//! tick-tag entry that decrements all of them, rather than requiring each
//! `#[datapack_component(Tick)]` function to manually list them.
//!
//! # Usage
//!
//! ```rust,ignore
//! use sand_core::state::{Cooldown, Ticks};
//! use sand_core::systems::cooldowns::register_cooldown;
//!
//! static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
//! static BLINK: Cooldown = Cooldown::new("blink", Ticks::new(100));
//!
//! // In your pack initializer (called at startup, before export):
//! register_cooldown(&DASH);
//! register_cooldown(&BLINK);
//! ```
//!
//! The export pipeline picks up registrations and emits tick commands for
//! all players (`@a`) for every registered cooldown.

use crate::state::Cooldown;
use std::sync::Mutex;

static REGISTERED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Register a cooldown for automatic ticking by the export pipeline.
///
/// Each registered cooldown will have `cooldown.tick_all_players()` included in the
/// generated tick function. Duplicate objective names are deduplicated.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::cooldowns::register_cooldown",
    module = "sand::systems",
    summary = "Register a cooldown for automatic ticking by the export pipeline.",
    context = "Register a cooldown for automatic ticking by the export pipeline. Each registered cooldown will have `cooldown.tick_all_players()` included in the generated tick function. Duplicate objective names are deduplicated.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    params(cd = "`cd` supplies the cd value used to register a cooldown for automatic ticking by the export pipeline."),
    example = "use sand::prelude::*;\n\nfn demonstrate(cd: & sand::state::Cooldown)  {\n    let register_cooldown = sand::systems::cooldowns::register_cooldown(cd);\n}",
    availability = ["Cargo feature: systems-cooldowns"],
)]
pub fn register_cooldown(cd: &Cooldown) {
    let cmd = cd.tick_all_players();
    let mut guard = REGISTERED.lock().expect("cooldown registry poisoned");
    if !guard.contains(&cmd) {
        guard.push(cmd);
    }
}

/// Drain all registered cooldown-tick commands.
///
/// Called by the export pipeline. Not intended for end-user use.
pub fn drain_cooldown_tick_commands() -> Vec<String> {
    REGISTERED
        .lock()
        .expect("cooldown registry poisoned")
        .drain(..)
        .collect()
}

/// Snapshot (without draining) all registered cooldown-tick commands.
pub fn cooldown_tick_commands() -> Vec<String> {
    REGISTERED
        .lock()
        .expect("cooldown registry poisoned")
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Ticks;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn make_cd(name: &'static str) -> Cooldown {
        Cooldown::new(name, Ticks::new(60))
    }

    #[test]
    fn register_and_snapshot() {
        let _test_guard = TEST_LOCK.lock().expect("cooldown test lock poisoned");
        // Use a fresh state by draining first
        drain_cooldown_tick_commands();

        let cd = make_cd("test_dash");
        register_cooldown(&cd);
        let cmds = cooldown_tick_commands();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("test_dash"), "got: {}", cmds[0]);
    }

    #[test]
    fn no_duplicates() {
        let _test_guard = TEST_LOCK.lock().expect("cooldown test lock poisoned");
        drain_cooldown_tick_commands();

        let cd = make_cd("test_blink");
        register_cooldown(&cd);
        register_cooldown(&cd);
        let cmds = cooldown_tick_commands();
        assert_eq!(cmds.len(), 1, "duplicates must be deduplicated");
    }
}
