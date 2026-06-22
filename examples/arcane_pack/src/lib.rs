//! # Arcane Pack
//!
//! A complete dogfood datapack built with [Sand](https://crates.io/crates/sand),
//! demonstrating the full attribute-first typed API in a single coherent system.
//!
//! Features:
//! - Mana system with scoreboard tracking
//! - Dash ability with cooldown
//! - Fireball spell with conditions
//! - Shield spell with flag
//! - Actionbar status display
//! - Welcome dialog component
//! - Storage-backed player settings
//!
//! ## Build
//!
//! ```sh
//! cargo run -p arcane_pack
//! # or from the workspace root
//! cargo run -p arcane_pack
//! ```

use sand_core::prelude::*;
use sand_macros::{component, function};

// -- State ------------------------------------------------------------------

/// Player mana (scoreboard integer).
static MANA: ScoreVar<i32> = ScoreVar::new("mana");

/// Dash cooldown (scoreboard-based timer, 3 seconds).
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

/// Fireball cooldown (scoreboard-based timer, 5 seconds).
static FIREBALL: Cooldown = Cooldown::new("fireball", Ticks::seconds(5));

/// Shield active flag.
static SHIELD: Flag = Flag::new("shield");

/// Persistent player settings (NBT storage).
static PLAYER_DATA: StorageVar<i32> = StorageVar::new("arcane:data", "player.settings");

// -- Load ------------------------------------------------------------------

/// Initialize scoreboards and storage on datapack load.
#[component(Load)]
pub fn load() {
    MANA.define();
    DASH.define();
    FIREBALL.define();
    SHIELD.define();
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
    FIREBALL.tick(Selector::all_players());

    // Show "Dash ready" when the player has enough mana and dash is off cooldown.
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            SHIELD.of("@s").is_false(),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Dash ready").aqua().bold(true),
        ));

    // Show "Fireball ready" when the player has enough mana and fireball is off cooldown.
    TypedExecute::as_players()
        .when(all![
            MANA.of("@s").gte(30),
            FIREBALL.ready("@s"),
            SHIELD.of("@s").is_false(),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Fireball ready").gold(),
        ));

    // Show "Shield active" when shield is active.
    TypedExecute::as_players()
        .when(SHIELD.of("@s").is_true())
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Shield active").green().bold(true),
        ));
}

// -- Functions -------------------------------------------------------------

/// Cast the dash ability — costs 25 mana, starts 3-second cooldown.
#[function("arcane:cast_dash")]
pub fn cast_dash() {
    TypedExecute::as_players_at_self()
        .when(all![
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            SHIELD.of("@s").is_false()
        ])
        .run(cmd::function(
            ResourceLocation::new("arcane", "cast_dash/execute").unwrap(),
        ));
}

/// Internal: actually apply the dash effect (called by cast_dash via function ref).
#[function("arcane:cast_dash/execute")]
pub fn cast_dash_execute() {
    MANA.remove(Selector::self_(), 25);
    DASH.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Dash cast!").gold());
}

/// Cast the fireball ability — costs 30 mana, starts 5-second cooldown.
#[function("arcane:cast_fireball")]
pub fn cast_fireball() {
    TypedExecute::as_players_at_self()
        .when(all![
            MANA.of("@s").gte(30),
            FIREBALL.ready("@s"),
            SHIELD.of("@s").is_false(),
        ])
        .run(cmd::function(
            ResourceLocation::new("arcane", "cast_fireball/execute").unwrap(),
        ));
}

/// Internal: actually apply the fireball effect (called by cast_fireball via function ref).
#[function("arcane:cast_fireball/execute")]
pub fn cast_fireball_execute() {
    MANA.remove(Selector::self_(), 30);
    FIREBALL.start(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Fireball cast!").red());
}

/// Toggle shield — costs 10 mana, sets shield flag.
#[function("arcane:toggle_shield")]
pub fn toggle_shield() {
    TypedExecute::as_players_at_self()
        .when(all![MANA.of("@s").gte(10), SHIELD.of("@s").is_false()])
        .run(cmd::function(
            ResourceLocation::new("arcane", "toggle_shield/on").unwrap(),
        ));

    TypedExecute::as_players_at_self()
        .when(SHIELD.of("@s").is_true())
        .run(cmd::function(
            ResourceLocation::new("arcane", "toggle_shield/off").unwrap(),
        ));
}

/// Internal: turn shield on (called by toggle_shield via function ref).
#[function("arcane:toggle_shield/on")]
pub fn toggle_shield_on() {
    MANA.remove(Selector::self_(), 10);
    SHIELD.enable(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Shield activated!").green());
}

/// Internal: turn shield off (called by toggle_shield via function ref).
#[function("arcane:toggle_shield/off")]
pub fn toggle_shield_off() {
    SHIELD.disable(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Shield deactivated.").red());
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

// -- Dialog (1.21.5+ / 26.x) ----------------------------------------------

/// A welcome dialog presented to players.
pub fn welcome_dialog() -> Dialog {
    Dialog::notice("arcane:welcome")
        .title("Welcome to Arcane Pack")
        .body(DialogBody::text("Choose an action below."))
        .button(
            DialogButton::new("Cast Dash").action(DialogAction::run_command(
                cmd::function(ResourceLocation::new("arcane", "cast_dash").unwrap()).to_string(),
            )),
        )
        .button(
            DialogButton::new("Cast Fireball").action(DialogAction::run_command(
                cmd::function(ResourceLocation::new("arcane", "cast_fireball").unwrap())
                    .to_string(),
            )),
        )
        .button(
            DialogButton::new("Toggle Shield").action(DialogAction::run_command(
                cmd::function(ResourceLocation::new("arcane", "toggle_shield").unwrap())
                    .to_string(),
            )),
        )
}

// -- Export hook (required by sand build) ----------------------------------

/// Invoked by the generated `sand_export` binary.
#[doc(hidden)]
pub fn __sand_export(namespace: &str) {
    println!("{}", sand_core::export_components_json(namespace));
}

// -- Tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_defines_scoreboards_and_storage() {
        let cmds = load();
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard objectives add mana"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard objectives add dash"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard objectives add fireball"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard objectives add shield"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("data modify storage arcane:data"))
        );
        assert!(cmds.iter().any(|c| c.contains("Datapack loaded")));
    }

    #[test]
    fn tick_decrements_cooldowns_and_shows_actionbar() {
        let cmds = tick();
        // Cooldown decrements
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players remove") && c.contains("dash"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players remove") && c.contains("fireball"))
        );
        // Actionbar for dash-ready players
        assert!(
            cmds.iter()
                .any(|c| c.contains("actionbar") && c.contains("Dash ready"))
        );
        // Actionbar for fireball-ready players
        assert!(
            cmds.iter()
                .any(|c| c.contains("actionbar") && c.contains("Fireball ready"))
        );
        // Actionbar for shield active
        assert!(
            cmds.iter()
                .any(|c| c.contains("actionbar") && c.contains("Shield active"))
        );
    }

    #[test]
    fn cast_dash_checks_mana_and_cooldown() {
        let cmds = cast_dash();
        // Should chain mana check + cooldown check + function call
        assert!(
            cmds.iter()
                .any(|c| c.contains("score @s mana matches 25.."))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("function arcane:cast_dash/execute"))
        );
    }

    #[test]
    fn cast_dash_execute_applies_effects() {
        let cmds = cast_dash_execute();
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players remove") && c.contains("mana"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players set") && c.contains("dash"))
        );
        assert!(cmds.iter().any(|c| c.contains("Dash cast")));
    }

    #[test]
    fn cast_fireball_checks_mana_and_cooldown() {
        let cmds = cast_fireball();
        assert!(
            cmds.iter()
                .any(|c| c.contains("score @s mana matches 30.."))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("function arcane:cast_fireball/execute"))
        );
    }

    #[test]
    fn cast_fireball_execute_applies_effects() {
        let cmds = cast_fireball_execute();
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players remove") && c.contains("mana"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players set") && c.contains("fireball"))
        );
        assert!(cmds.iter().any(|c| c.contains("Fireball cast")));
    }

    #[test]
    fn toggle_shield_turns_on() {
        let cmds = toggle_shield_on();
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players remove") && c.contains("mana"))
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("scoreboard players set") && c.contains("shield"))
        );
        assert!(cmds.iter().any(|c| c.contains("Shield activated")));
    }

    #[test]
    fn toggle_shield_turns_off() {
        let cmds = toggle_shield_off();
        assert!(cmds.iter().any(|c| c.contains("scoreboard players set")
            && c.contains("shield")
            && c.contains("0")));
        assert!(cmds.iter().any(|c| c.contains("Shield deactivated")));
    }

    #[test]
    fn welcome_dialog_json() {
        let json = welcome_dialog().to_json();
        assert_eq!(
            json["title"]["text"],
            serde_json::Value::String("Welcome to Arcane Pack".to_string())
        );
        assert!(json["buttons"].is_array());
        assert_eq!(json["buttons"].as_array().unwrap().len(), 3);
    }
}
