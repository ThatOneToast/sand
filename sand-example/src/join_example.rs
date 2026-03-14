//! # Player join example
//!
//! Demonstrates:
//! - Detecting player joins using `AdvancementTrigger::Custom` with `minecraft:tick`
//! - Sending a personalised welcome message
//! - Tracking per-player data with a scoreboard (persistent across sessions)
//! - Reading that data back in a command
//!
//! ## How it works
//!
//! 1. **`#[component(Load)]`** — `join_init` runs once when the datapack loads.
//!    It creates the `join_count` scoreboard objective if it doesn't already
//!    exist. Using `add` (not `set`) is idempotent — safe to call every reload.
//!
//! 2. **`#[component]` advancement** — `detect_join` fires every tick via
//!    `minecraft:tick`. Its reward runs `on_player_join`, then the function
//!    *revokes* the advancement so it re-arms for next login.
//!
//!    Note: `minecraft:player_joined_world` was removed in 1.21.x. Use
//!    `AdvancementTrigger::Tick` (or `Custom` for other version-specific triggers).
//!
//! 3. **`#[function]`** — `on_player_join` contains the actual join logic:
//!    increments the counter, sends the welcome message, and revokes the
//!    advancement so it fires again next time.
//!
//! ## Data persistence
//!
//! `scoreboard players add @s join_count 1` writes to the scoreboard, which
//! Minecraft saves inside the world's `data/scoreboard.dat`. The value persists
//! across server restarts and reloads — no external storage needed for simple
//! integer counters.
//!
//! For richer structured data (e.g. a table of stats per player), use
//! `/data modify storage` instead — see the commented example at the bottom.

use sand_core::mcfunction;
use sand_macros::{component, function};

// ── 1. Initialise scoreboards on load ────────────────────────────────────────

/// Creates the `join_count` scoreboard objective on datapack load.
///
/// Using `scoreboard objectives add` is safe to repeat — Minecraft silently
/// ignores the command if the objective already exists, so reloading the
/// datapack never resets player data.
#[component(Load)]
pub fn join_init() {
    mcfunction! {
        // "dummy" type = a plain integer counter, not tied to any game event.
        "scoreboard objectives add join_count dummy";
    }
}

// ── 2. Detect the join event ──────────────────────────────────────────────────

/// Advancement that fires every tick while the player is in the world.
///
/// The reward runs `hello_world:on_player_join`. That function then revokes
/// this advancement, re-arming it for the player's next session.
///
/// We use `AdvancementTrigger::Tick` because `minecraft:player_joined_world`
/// was removed in 1.21.x. Revoking after the first trigger achieves the same
/// per-session-join behaviour.
#[component]
pub fn detect_join() -> sand_core::Advancement {
    use sand_core::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion};
    Advancement::new("hello_world:detect_join".parse().unwrap())
        .criterion("joined", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("hello_world:on_player_join"))
}

// ── 3. The join handler ───────────────────────────────────────────────────────

/// Runs for the joining player (`@s`) as soon as they enter the world.
///
/// - Increments their persistent `join_count` scoreboard value.
/// - Sends them a personalised welcome message that shows their join count.
/// - Announces their arrival to everyone else on the server.
/// - Revokes the detection advancement so it fires again next login.
#[function]
pub fn on_player_join() {
    mcfunction! {
        // ── Persist the visit counter ──────────────────────────────────────
        // scoreboard stores integers per (player, objective) pair — persists
        // across server restarts inside world/data/scoreboard.dat.
        "scoreboard players add @s join_count 1";

        // ── Welcome the joining player ─────────────────────────────────────
        // `{"score":…}` inlines the scoreboard value directly into the JSON
        // text component, so no intermediate storage is needed.
        r#"tellraw @s [{"text":"Welcome back! ","color":"gold","bold":true},{"text":"This is visit #","color":"yellow"},{"score":{"name":"@s","objective":"join_count"},"color":"aqua"},{"text":".","color":"yellow"}]"#;

        // ── Announce to everyone else ──────────────────────────────────────
        // `@a[tag=!@s]` selects all players *except* the one who just joined.
        // Note: tag=!@s is valid syntax in 1.21.11 selectors.
        r#"tellraw @a[tag=!joined_just_now] [{"selector":"@s","color":"green"},{"text":" joined the server!","color":"gray"}]"#;

        // ── Re-arm the detection advancement ──────────────────────────────
        // Revoking here means the advancement will fire again next login.
        "advancement revoke @s only hello_world:detect_join";
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sand_core::{DatapackComponent, FunctionDescriptor, FunctionTagDescriptor, inventory};

    #[test]
    fn join_init_registered_as_load() {
        // join_init should appear in both FunctionDescriptor and FunctionTagDescriptor
        // (for minecraft:load).
        let paths: Vec<&str> = inventory::iter::<FunctionDescriptor>()
            .map(|d| d.path)
            .collect();
        assert!(
            paths.contains(&"join_init"),
            "join_init not in FunctionDescriptor inventory"
        );

        let tag_entry = inventory::iter::<FunctionTagDescriptor>()
            .find(|d| d.function_path == "join_init")
            .expect("join_init not registered in FunctionTagDescriptor");
        assert_eq!(tag_entry.tag, "minecraft:load");
    }

    #[test]
    fn on_player_join_commands() {
        let cmds = on_player_join();
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players add @s join_count"))
        );
        assert!(cmds.iter().any(|c| c.contains("tellraw @s")));
        assert!(
            cmds.iter()
                .any(|c| c.contains("advancement revoke @s only hello_world:detect_join"))
        );
    }

    #[test]
    fn detect_join_advancement_uses_correct_trigger() {
        let adv = detect_join();
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["joined"]["trigger"].as_str().unwrap(),
            "minecraft:tick",
        );
        assert_eq!(
            json["rewards"]["function"].as_str().unwrap(),
            "hello_world:on_player_join",
        );
    }
}
