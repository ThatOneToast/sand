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

use sand_core::EventPlayer;
use sand_core::event::vanilla::{FirstJoin, OnDeath, OnJoin, OnRespawn};
use sand_core::prelude::*;
use sand_macros::{component, event, function};

mod events;
use crate::events::{AteGoldenAppleEvent, UsedDashWandEvent};

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
    GOLDEN_APPLE_HANDLE.define();
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
            DialogButton::new("Cast Dash").action(DialogAction::run_function(
                ResourceLocation::new("arcane", "cast_dash").unwrap(),
            )),
        )
        .button(
            DialogButton::new("Cast Fireball").action(DialogAction::run_function(
                ResourceLocation::new("arcane", "cast_fireball").unwrap(),
            )),
        )
        .button(
            DialogButton::new("Toggle Shield").action(DialogAction::run_function(
                ResourceLocation::new("arcane", "toggle_shield").unwrap(),
            )),
        )
}

// -- EventHandle: lifecycle control for advancement events -----------------

/// Enables/disables the golden apple event per player (typed — no string needed).
static GOLDEN_APPLE_HANDLE: EventHandle<events::AteGoldenAppleEvent> = EventHandle::new();

// -- Events ----------------------------------------------------------------
//
// Demonstrates every dispatch mode: join tick, death/respawn tick, and
// custom advancement events with guard(), function pointer calls,
// EventHandle, and typed trigger builders.

/// Fires every time a player joins the world.
#[event]
pub fn on_join(event: OnJoin) {
    cmd::tellraw(
        event.player(),
        Text::new("Welcome to the Arcane Pack!").gold(),
    );
}

/// Fires once per player — initializes mana and shows a welcome title.
///
/// The dispatch = "advancement" attribute is ignored for FirstJoinEvent
/// (the macro hardcodes a Tick advancement with no revoke).
#[event]
pub fn on_first_join(event: FirstJoin) {
    MANA.set(event.player(), 100);
    Title::of(event.player())
        .title(Text::new("Arcane Pack").gold().bold(true))
        .subtitle(Text::new("Your journey begins").green())
        .build();
    cmd::tellraw(
        event.player(),
        Text::new("You have been granted 100 mana!").aqua(),
    );
}

/// Fires when a player dies — disables the golden apple handle,
/// resets shield flag, and shows a death title.
#[event]
pub fn on_death(event: OnDeath) {
    GOLDEN_APPLE_HANDLE.disable("@s");
    SHIELD.disable(Selector::self_());
    Title::of(event.player())
        .title(Text::new("You died!").red())
        .subtitle(Text::new("Shield deactivated, cooldowns cleared").gray())
        .build();
}

/// Fires when a player respawns — re-enables the golden apple handle,
/// restores 50 mana, and stops all cooldowns.
#[event]
pub fn on_respawn(event: OnRespawn) {
    GOLDEN_APPLE_HANDLE.enable("@s");
    MANA.set(Selector::self_(), 50);
    DASH.stop(Selector::self_());
    FIREBALL.stop(Selector::self_());
    cmd::tellraw(
        event.player(),
        Text::new("You have been granted 50 mana on respawn.").aqua(),
    );
}

/// Fired when a golden apple is consumed with mana below 100 (see guard).
/// Uses a custom AdvancementEvent with guard() and function pointer call.
#[event(dispatch = "advancement")]
pub fn on_ate_golden_apple(event: AteGoldenAppleEvent) {
    MANA.add(event.player(), 10);
    Actionbar::show(event.player(), Text::new("+10 mana (golden apple)").green());
    cmd::call(golden_apple_reward as fn() -> Vec<String>);
}

/// Sound reward for golden apple — called via function pointer.
#[function]
pub fn golden_apple_reward() {
    cmd::say("Delicious!");
}

/// Fired when a player uses a dash wand (stick with custom data) while
/// eligible (mana >= 25, dash cooldown ready, shield inactive).
/// Uses a custom AdvancementEvent with guard() and function pointer call.
#[event(dispatch = "advancement")]
pub fn on_used_dash_wand(event: UsedDashWandEvent) {
    MANA.remove(event.player(), 25);
    DASH.start(event.player());
    Actionbar::show(event.player(), Text::new("Dash wand activated!").gold());
    cmd::call(dash_wand_effect as fn() -> Vec<String>);
}

/// Speed boost feedback — called via function pointer.
#[function]
pub fn dash_wand_effect() {
    cmd::say("Whoosh!");
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

    #[test]
    fn golden_advancements_generated() {
        let json_str = sand_core::export_components_json("arcane");
        let records: Vec<serde_json::Value> =
            serde_json::from_str(&json_str).expect("valid JSON from export");

        // ─────────────────────────────────────────────────────────────────────
        // AteGoldenAppleEvent — advancement-backed with guard and fn-ptr call
        // ─────────────────────────────────────────────────────────────────────

        // Advancement JSON: trigger + reward pointing at entry function
        let apple_adv = records
            .iter()
            .find(|r| r["path"] == "on_ate_golden_apple" && r["dir"] == "advancement")
            .expect("ate_golden_apple advancement record");
        let apple_json: serde_json::Value =
            serde_json::from_str(apple_adv["content"].as_str().unwrap())
                .expect("valid advancement JSON");
        assert_eq!(
            apple_json["criteria"]["event"]["trigger"], "minecraft:consume_item",
            "golden apple trigger"
        );
        // Reward must call the entry function (same path as before).
        assert_eq!(
            apple_json["rewards"]["function"], "arcane:on_ate_golden_apple",
            "advancement reward must call entry function"
        );

        // Entry function: revoke → guard → call body (Part 4+5)
        let apple_entry = records
            .iter()
            .find(|r| r["path"] == "on_ate_golden_apple" && r["dir"] == "function")
            .expect("ate_golden_apple entry function");
        let entry_content = apple_entry["content"].as_str().unwrap();

        // Revoke must be first (re-arms even when guard rejects)
        let revoke_pos = entry_content.find("advancement revoke");
        let guard_pos = entry_content.find("execute unless");
        let body_call_pos = entry_content.find("function arcane:on_ate_golden_apple/body");
        assert!(revoke_pos.is_some(), "entry must revoke: {entry_content}");
        assert!(
            guard_pos.is_some(),
            "entry must check guard: {entry_content}"
        );
        assert!(
            body_call_pos.is_some(),
            "entry must call body: {entry_content}"
        );
        assert!(
            revoke_pos.unwrap() < guard_pos.unwrap(),
            "revoke must precede guard so event re-arms on guard failure:\n{entry_content}"
        );
        assert!(
            guard_pos.unwrap() < body_call_pos.unwrap(),
            "guard must precede body call:\n{entry_content}"
        );
        // Guard must be correct Minecraft syntax (no 'unless if')
        assert!(
            entry_content.contains("execute unless score @s mana matches ..99 run return 0"),
            "guard must use correct execute unless syntax, got:\n{entry_content}"
        );

        // Body function: pure user commands, no plumbing (Part 4)
        let apple_body = records
            .iter()
            .find(|r| r["path"] == "on_ate_golden_apple/body" && r["dir"] == "function")
            .expect("ate_golden_apple body function");
        let body_content = apple_body["content"].as_str().unwrap();
        assert!(
            body_content.contains("mana"),
            "body updates mana: {body_content}"
        );
        // Bare function pointer resolved to real namespace (Parts 2+3)
        assert!(
            body_content.contains("function arcane:golden_apple_reward"),
            "cmd::call() must resolve to 'function arcane:golden_apple_reward':\n{body_content}"
        );
        // No sentinel leaks into exported content
        assert!(
            !body_content.contains("__sand_local"),
            "sentinel must be resolved, found in:\n{body_content}"
        );
        assert!(
            !entry_content.contains("__sand_local"),
            "sentinel must be resolved, found in entry:\n{entry_content}"
        );

        // ─────────────────────────────────────────────────────────────────────
        // UsedDashWandEvent — also advancement-backed with compound guard
        // ─────────────────────────────────────────────────────────────────────

        let wand_adv = records
            .iter()
            .find(|r| r["path"] == "on_used_dash_wand" && r["dir"] == "advancement")
            .expect("used_dash_wand advancement record");
        let wand_json: serde_json::Value =
            serde_json::from_str(wand_adv["content"].as_str().unwrap())
                .expect("valid advancement JSON");
        assert_eq!(
            wand_json["criteria"]["event"]["trigger"], "minecraft:using_item",
            "dash wand trigger"
        );

        let wand_body = records
            .iter()
            .find(|r| r["path"] == "on_used_dash_wand/body" && r["dir"] == "function")
            .expect("used_dash_wand body function");
        assert!(
            wand_body["content"].as_str().unwrap().contains("mana"),
            "wand body removes mana"
        );

        // ─────────────────────────────────────────────────────────────────────
        // FirstJoin — Tick advancement, OncePerPlayer (no revoke)
        // ─────────────────────────────────────────────────────────────────────

        let join_adv = records
            .iter()
            .find(|r| r["path"] == "on_first_join" && r["dir"] == "advancement")
            .expect("first_join advancement record");
        let join_json: serde_json::Value =
            serde_json::from_str(join_adv["content"].as_str().unwrap())
                .expect("valid advancement JSON");
        assert_eq!(
            join_json["criteria"]["event"]["trigger"], "minecraft:tick",
            "first join trigger"
        );
        assert!(join_json.get("rewards").is_some(), "first join has rewards");

        // ─────────────────────────────────────────────────────────────────────
        // Helper function — registered via #[function] with no explicit path
        // ─────────────────────────────────────────────────────────────────────

        let reward_fn = records
            .iter()
            .find(|r| r["path"] == "golden_apple_reward" && r["dir"] == "function")
            .expect("golden_apple_reward function");
        assert!(reward_fn["content"].as_str().unwrap().contains("Delicious"));
    }
}
