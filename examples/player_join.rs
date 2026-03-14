//! # Player join detection
//!
//! A complete working example that demonstrates:
//!
//! - Detecting player joins using an advancement with `minecraft:tick` trigger
//! - Sending personalized welcome messages with JSON text components
//! - Tracking per-player data with scoreboards (persistent across sessions)
//! - Using `#[component(Load)]`, `#[component]`, and `#[function]` together
//!
//! ## How it works
//!
//! 1. **`on_load`** runs once when the datapack loads, creating the scoreboard.
//! 2. **`detect_join`** is an advancement that fires every tick. Its reward
//!    calls `on_player_join`, which then revokes the advancement so it re-arms.
//! 3. **`on_player_join`** increments the visit counter, sends a welcome
//!    message, announces the join, and revokes the detection advancement.
//!
//! ## Data persistence
//!
//! Scoreboards persist in `world/data/scoreboard.dat` across server restarts.
//! No external storage needed for simple integer counters.

use sand_core::mcfunction;
use sand_macros::{component, function};

// ── 1. Initialize scoreboards on load ────────────────────────────────────────

/// Creates the `join_count` scoreboard objective.
/// `scoreboard objectives add` is idempotent — safe to call every reload.
#[component(Load)]
pub fn on_load() {
    mcfunction! {
        "scoreboard objectives add join_count dummy";
    }
}

// ── 2. Detect the join event ─────────────────────────────────────────────────

/// Advancement that fires every tick. The reward runs `on_player_join`,
/// which revokes this advancement to re-arm it for next login.
#[component]
pub fn detect_join() -> sand_core::Advancement {
    use sand_core::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion};
    Advancement::new("my_pack:detect_join".parse().unwrap())
        .criterion("joined", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function("my_pack:on_player_join"))
}

// ── 3. Handle the join ───────────────────────────────────────────────────────

/// Runs for the joining player (`@s`):
/// - Increments their persistent join counter
/// - Sends a personalized welcome message showing their visit number
/// - Announces arrival to other players
/// - Revokes the detection advancement so it fires again next login
#[function]
pub fn on_player_join() {
    mcfunction! {
        // Increment visit counter (persists in scoreboard.dat)
        "scoreboard players add @s join_count 1";

        // Welcome the joining player with their visit number
        r#"tellraw @s [{"text":"Welcome back! ","color":"gold","bold":true},{"text":"This is visit #","color":"yellow"},{"score":{"name":"@s","objective":"join_count"},"color":"aqua"},{"text":".","color":"yellow"}]"#;

        // Announce to everyone else
        r#"tellraw @a[tag=!joined_just_now] [{"selector":"@s","color":"green"},{"text":" joined the server!","color":"gray"}]"#;

        // Re-arm the detection advancement
        "advancement revoke @s only my_pack:detect_join";
    }
}
