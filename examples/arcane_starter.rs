//! # Arcane Starter — a complete dogfood datapack
//!
//! A real gameplay datapack demonstrating every major Sand API in a single
//! coherent system. Load this into a Minecraft world to get:
//!
//! - A mana system with scoreboard tracking
//! - A dash ability with cooldown
//! - Actionbar status display
//! - A named cast function
//! - Typed execute conditions
//! - Storage-backed persistent settings
//! - One explicit interop escape hatch (isolated and documented)
//!
//! ## How it works
//!
//! 1. **Load**: defines scoreboards, initializes storage, broadcasts a welcome.
//! 2. **Tick**: decrements cooldowns, shows actionbar status.
//! 3. **cast_dash**: the player-facing ability — costs mana, triggers cooldown.
//! 4. **welcome_dialog**: a typed dialog component (1.21.6+ / 26.x).
//!
//! ## Build
//!
//! ```sh
//! cargo run -p sand -- new arcane_starter
//! # paste this file into src/lib.rs
//! cargo run -p sand -- build
//! ```

use sand_core::prelude::*;
use sand_macros::{component, function};

// -- State ------------------------------------------------------------------

/// Player mana (scoreboard integer).
static MANA: ScoreVar<i32> = ScoreVar::new("mana");

/// Dash cooldown (scoreboard-based timer, 3 seconds).
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

/// Persistent player settings (NBT storage).
static PLAYER_DATA: StorageVar<i32> = StorageVar::new("arcane:data", "player.settings");

// -- Load ------------------------------------------------------------------

/// Initialize scoreboards and storage on datapack load.
#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
    PLAYER_DATA.set_int(100);
    cmd::tellraw(
        Selector::all_players(),
        Text::new("[Arcane] Datapack loaded.").gold().bold(true),
    );
}

// -- Tick ------------------------------------------------------------------

/// Per-tick logic: decrement cooldowns, show actionbar status.
#[component(Tick)]
pub fn tick() {
    DASH.tick(Selector::all_players());

    // Show "Dash ready" when the player has enough mana and dash is off cooldown.
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));
}

// -- Functions -------------------------------------------------------------

/// Cast the dash ability — costs 25 mana, starts 3-second cooldown.
#[function("arcane:cast_dash")]
pub fn cast_dash() {
    TypedExecute::as_players_at_self()
        .when(all![MANA.of("@s").gte(25), DASH.ready("@s")])
        .run(cmd::function(
            ResourceLocation::new("arcane", "cast_dash/execute").unwrap(),
        ));
}

/// Internal: actually apply the dash effect (called by cast_dash via function ref).
#[function("arcane:cast_dash/execute")]
pub fn cast_dash_execute() {
    MANA.remove(Selector::self_(), 25);
    DASH.start(Selector::self_());
    cmd::tellraw(
        Selector::self_(),
        Text::new("Dash cast!").gold(),
    );
}

/// Show the current mana in chat (debug/info command).
#[function("arcane:show_mana")]
pub fn show_mana() {
    TypedExecute::as_players()
        .when(MANA.of("@s").gte(0))
        .run(cmd::tellraw(
            Selector::self_(),
            Text::new("Your mana is available.").green(),
        ));
}

// -- Dialog (1.21.6+ / 26.x) ----------------------------------------------

/// A welcome dialog presented to players.
#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::notice_local("welcome")
        .title(Text::new("Welcome to Arcane Starter").gold())
        .body(DialogBody::text(Text::new("Choose an action below.").aqua()))
        .button(
            DialogButton::new(Text::new("Cast Dash").aqua())
                .action(DialogAction::run_function(cast_dash)),
        )
}

/// Opens the local welcome dialog for the current player.
#[function("arcane:open_welcome_menu")]
pub fn open_welcome_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
}

// -- Interop escape hatch --------------------------------------------------

/// One explicit raw command for cross-datapack interop.
/// This calls another datapack's documented public API.
#[function("arcane:interop_example")]
pub fn interop_example() {
    // Escape hatch: this is another datapack's documented public contract.
    cmd::raw("function other_pack:api/do_special_thing");
}

// -- Export hook (required by sand build) ----------------------------------

/// Invoked by the generated `sand_export` binary.
#[doc(hidden)]
pub fn __sand_export(namespace: &str) {
    match sand_core::try_export_components_json(namespace) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    }
}

// -- Tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_defines_scoreboards_and_storage() {
        let cmds = load();
        assert!(cmds.iter().any(|c| c.contains("scoreboard objectives add mana")));
        assert!(cmds.iter().any(|c| c.contains("scoreboard objectives add dash")));
        assert!(cmds.iter().any(|c| c.contains("data modify storage arcane:data")));
        assert!(cmds.iter().any(|c| c.contains("Welcome")));
    }

    #[test]
    fn tick_decrements_cooldown_and_shows_actionbar() {
        let cmds = tick();
        // Cooldown decrement
        assert!(cmds.iter().any(|c| c.contains("scoreboard players remove") && c.contains("dash")));
        // Actionbar for dash-ready players
        assert!(cmds.iter().any(|c| c.contains("actionbar") && c.contains("Dash ready")));
    }

    #[test]
    fn cast_dash_checks_mana_and_cooldown() {
        let cmds = cast_dash();
        // Should chain mana check + cooldown check + function call
        assert!(cmds.iter().any(|c| c.contains("score @s mana matches 25..")));
        assert!(cmds.iter().any(|c| c.contains("function arcane:cast_dash/execute")));
    }

    #[test]
    fn cast_dash_execute_applies_effects() {
        let cmds = cast_dash_execute();
        assert!(cmds.iter().any(|c| c.contains("scoreboard players remove") && c.contains("mana")));
        assert!(cmds.iter().any(|c| c.contains("scoreboard players set") && c.contains("dash")));
        assert!(cmds.iter().any(|c| c.contains("Dash cast")));
    }

    #[test]
    fn interop_uses_raw_escape_hatch() {
        let cmds = interop_example();
        assert_eq!(cmds, vec!["function other_pack:api/do_special_thing"]);
    }

    #[test]
    fn welcome_dialog_json() {
        let json = welcome_dialog().to_json();
        assert_eq!(json["title"]["text"], serde_json::Value::String("Welcome to Arcane Starter".to_string()));
        assert!(json["buttons"].is_array());
    }
}
