//! # Basic functions
//!
//! Demonstrates the core Sand macros for defining datapack functions.

use sand_core::mcfunction;
use sand_macros::{component, function, run_fn};

// ── Simple function ──────────────────────────────────────────────────────────
// #[function] turns a Rust function into a .mcfunction file.
// The function name becomes the resource path: data/<namespace>/function/greet.mcfunction

#[function]
pub fn greet() {
    mcfunction! {
        r#"tellraw @a {"text":"Hello from Sand!","color":"gold","bold":true}"#;
        "playsound minecraft:entity.experience_orb.pickup master @a ~ ~ ~ 1 1";
    }
}

// ── Tick function ────────────────────────────────────────────────────────────
// #[component(Tick)] registers in minecraft:tick — runs every game tick (20/sec).

#[component(Tick)]
pub fn game_tick() {
    mcfunction! {
        "scoreboard players add @a tick_count 1";
    }
}

// ── Load function ────────────────────────────────────────────────────────────
// #[component(Load)] registers in minecraft:load — runs once when the datapack loads.

#[component(Load)]
pub fn on_load() {
    mcfunction! {
        "scoreboard objectives add tick_count dummy";
        r#"tellraw @a {"text":"Datapack loaded!","color":"green"}"#;
    }
}

// ── Inline functions with run_fn! ────────────────────────────────────────────
// run_fn! defines a function AND returns the command to call it — useful for
// execute chains where you want to define the target inline.

#[function]
pub fn main_loop() {
    mcfunction! {
        // Call an existing function by resource location
        run_fn!("my_pack:greet");

        // Define + call inline — the body becomes a separate .mcfunction file
        run_fn!("my_pack:helpers/reset_scores" {
            "scoreboard players set @a tick_count 0";
        });
    }
}

// ── Custom function tags ─────────────────────────────────────────────────────
// #[component(Tag = "ns:name")] registers in any function tag — useful for
// inter-datapack APIs.

#[component(Tag = "my_lib:api/on_player_death")]
pub fn handle_death() {
    mcfunction! {
        r#"tellraw @a [{"selector":"@s"},{"text":" has fallen!","color":"red"}]"#;
    }
}
