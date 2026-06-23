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

use sand_core::event::vanilla::{FirstJoin, OnDeath, OnJoin, OnRespawn};
use sand_core::prelude::*;
use sand_macros::{component, event, function};

mod events;
use crate::events::{AteGoldenAppleEvent, EnhancedCellsDamagedEvent, UsedDashWandEvent};

// -- State ------------------------------------------------------------------

/// Player mana (scoreboard integer).
static MANA: ScoreVar<i32> = ScoreVar::new("mana");

/// Dash cooldown (scoreboard-based timer, 3 seconds).
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

/// Fireball cooldown (scoreboard-based timer, 5 seconds).
static FIREBALL: Cooldown = Cooldown::new("fireball", Ticks::seconds(5));

/// Shield active flag.
static SHIELD: Flag = Flag::new("shield");

/// Enhanced cells — grants +20 max HP (40 total).
static HAS_ENHANCED_CELLS: Flag = Flag::new("has_enhanced_cells");

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
    HAS_ENHANCED_CELLS.define();
    GOLDEN_APPLE_HANDLE.define();
    PLAYER_DATA.set_int(100);

    // EventBuilder-declared state: merchant rank scoreboard.
    MERCHANT_RANK.define();

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
    cmd::effect_give(Selector::self_(), EffectId::Speed)
        .duration(Ticks::seconds(2))
        .amplifier(1)
        .particles(false);
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
    cmd::effect_give(Selector::self_(), EffectId::Resistance).seconds(15);
    cmd::tellraw(Selector::self_(), Text::new("Shield activated!").green());
}

/// Internal: turn shield off (called by toggle_shield via function ref).
#[function("arcane:toggle_shield/off")]
pub fn toggle_shield_off() {
    SHIELD.disable(Selector::self_());
    cmd::effect_clear_effect(Selector::self_(), EffectId::Resistance);
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

// -- Dialog (1.21.6+ / 26.x) ----------------------------------------------

/// A welcome dialog presented to players.
#[component]
pub fn welcome_dialog() -> Dialog {
    Dialog::multi_action_local("welcome")
        .title(Text::new("Welcome to Arcane Pack").gold())
        .body(DialogBody::text(
            Text::new("Choose an action below.").aqua(),
        ))
        .button(
            DialogButton::new(Text::new("Cast Dash").aqua())
                .action(DialogAction::run_function(cast_dash)),
        )
        .button(
            DialogButton::new(Text::new("Cast Fireball").red())
                .action(DialogAction::run_function(cast_fireball)),
        )
        .button(
            DialogButton::new(Text::new("Toggle Shield").green())
                .action(DialogAction::run_function(toggle_shield)),
        )
}

/// Dash wand item used by the custom advancement event.
pub fn dash_wand_item() -> CustomItem {
    CustomItem::new(ItemId::minecraft("stick").unwrap())
        .id("arcane:dash_wand")
        .component(ItemComponent::custom_name(
            Text::new("Dash Wand").aqua().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Right click to dash").gray(),
            Text::new("Consumes 25 mana").dark_gray(),
        ]))
        .component(ItemComponent::custom_data_marker("arcane_wand"))
        .component(ItemComponent::custom_model_data(1001))
        .component(ItemComponent::rarity(Rarity::Rare))
        .component(ItemComponent::attribute_modifier(
            AttributeModifier::new(AttributeId::AttackSpeed)
                .amount(0.2)
                .operation(AttributeOperation::AddValue)
                .slot(EquipmentSlotGroup::Mainhand),
        ))
        .component(ItemComponent::potion_contents(
            PotionContents::new().custom_effect(
                StatusEffectInstance::new(EffectId::Speed).duration(Ticks::seconds(2)),
            ),
        ))
        .component(ItemComponent::max_stack_size(1))
}

/// Intentional raw escape hatch example for a future/modded component.
pub fn resonance_relic_item() -> CustomItem {
    CustomItem::new(ItemId::minecraft("amethyst_shard").unwrap())
        .component(ItemComponent::custom_name(
            Text::new("Resonance Relic").light_purple(),
        ))
        .component(ItemComponent::custom_data_marker("resonance_relic"))
        .with_raw_component(RawComponent::new(
            "arcane:resonance",
            "{frequency:7,stable:true}",
        ))
}

/// Opens the local welcome dialog for the current player.
#[function("arcane:open_welcome_menu")]
pub fn open_welcome_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("welcome"));
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
/// FirstJoinEvent uses a tick advancement with no revoke.
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
    cmd::effect_clear(Selector::self_());
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
#[event]
pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
    MANA.add(event.player(), 10);
    Actionbar::show(event.player(), Text::new("+10 mana (golden apple)").green());
    cmd::call(golden_apple_reward);
}

/// Sound reward for golden apple — called via function pointer.
#[function]
pub fn golden_apple_reward() {
    cmd::say("Delicious!");
}

/// Fired when a player uses a dash wand (stick with custom data) while
/// eligible (mana >= 25, dash cooldown ready, shield inactive).
/// Uses a custom AdvancementEvent with guard() and function pointer call.
#[event]
pub fn on_used_dash_wand(event: Event<UsedDashWandEvent>) {
    MANA.remove(event.player(), 25);
    DASH.start(event.player());
    Actionbar::show(event.player(), Text::new("Dash wand activated!").gold());
    cmd::call(dash_wand_effect);
}

/// Speed boost feedback — called via function pointer.
#[function]
pub fn dash_wand_effect() {
    cmd::effect_give(Selector::self_(), EffectId::JumpBoost)
        .seconds(3)
        .particles(false);
    cmd::effect_give(
        Selector::self_(),
        EffectId::custom("arcane:dash_resonance").unwrap(),
    )
    .seconds(1)
    .particles(false);
    cmd::say("Whoosh!");
}

// -- Enhanced cells (conditional branch dogfood) ---------------------------

/// Grant enhanced cells — gives the player +20 max health (40 total hearts).
///
/// Demonstrates the `if_()` grouped-branch API:
/// - The `if` branch detects the flag is already set and returns early.
/// - The `else` branch sets the attribute, notifies the player, sets the flag,
///   and returns. The flag is mutated inside the else branch; the return still
///   fires because all commands run in order inside one branch function.
#[function("arcane:grant_enhanced_cells")]
pub fn grant_enhanced_cells() {
    cmd::say("Enhanced cells was called");
    if_(HAS_ENHANCED_CELLS.of("@s").is_true())
        .then_all(mcfunction![
            cmd::tellraw(
                Selector::self_(),
                Text::new("You already have enhanced cells").red(),
            );
            cmd::return_fail();
        ])
        .else_all(mcfunction![
            cmd::attribute_base_set(
                Selector::self_(),
                AttributeType::MaxHealth.as_str(),
                40.0,
            );
            cmd::tellraw(
                Selector::self_(),
                Text::new("Granted enhanced cells!").green(),
            );
            HAS_ENHANCED_CELLS.enable("@s");
            cmd::return_cmd(0);
        ]);
}

/// Reflect fixed damage to nearby non-player entities when an enhanced-cells
/// player is damaged.
#[event]
pub fn on_damaged_damage_nearby(event: DamageEvent<EnhancedCellsDamagedEvent>) {
    event
        .reflect_damage()
        .to(EntityTargets::nearby(5.0)
            .excluding_players()
            .excluding_self())
        .amount(DamageAmount::fixed(4.0))
        .damage_type(DamageKind::Generic)
        .run();
}

// -- EventBuilder demo: villager trade event --------------------------------
//
// This event is defined entirely via EventBuilder — no AdvancementEvent impl
// needed. The advancement is generated in a #[component] function, and the
// reward function is wired up with a matching #[function] path.
//
// State variables are declared on the builder so the load function can call
// `villager_trade_config().state_defines()` to obtain the define commands
// without duplicating the list.

/// Merchant rank: how many trades the player has completed.
static MERCHANT_RANK: ScoreVar<i32> = ScoreVar::new("merchant_rank");

/// Returns the EventConfig for the villager trade event.
///
/// The config is a value, so it can be inspected in tests or extended by
/// calling code without touching the advancement generation pipeline.
pub fn villager_trade_config() -> EventConfig {
    EventBuilder::new()
        .trigger(sand_core::AdvancementTrigger::VillagerTrade {
            item: Some(ItemPredicate::new()),
            villager: None,
        })
        .reset(EventReset::AfterFire)
        .visibility(EventVisibility::Hidden)
        .guard(MERCHANT_RANK.of("@s").gte(0))
        .score(&MERCHANT_RANK)
        .score(&MANA)
        .build()
}

/// Generates the advancement component for the villager trade event.
///
/// Demonstrates `EventConfig::advancement()` — the advancement ID and
/// reward function path must match the `#[function]` below.
#[component]
pub fn villager_trade_advancement() -> sand_core::Advancement {
    villager_trade_config().advancement("arcane:villager_trade", "arcane:on_villager_trade")
}

/// Handler for the villager trade event.
///
/// The `advancement revoke` and guard check are handled manually here
/// using `EventConfig::reward_prologue()` — exactly as the export pipeline
/// would emit them for a trait-based `#[event]` handler.
#[function("arcane:on_villager_trade")]
pub fn on_villager_trade() {
    // Revoke the advancement so it re-arms on the next trade (AfterFire reset).
    cmd::raw("advancement revoke @s only arcane:villager_trade");
    // Guard: skip if merchant_rank < 0 (sanity check).
    cmd::raw(format!(
        "execute unless score @s {} matches 0.. run return 0",
        MERCHANT_RANK.objective_name()
    ));
    // Handler body:
    MERCHANT_RANK.add(Selector::self_(), 1);
    MANA.add(Selector::self_(), 5);
    Actionbar::show(
        Selector::self_(),
        Text::new("+5 mana (villager trade)").aqua(),
    );
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
        // has_enhanced_cells is >16 chars so gets hashed; check via define()
        let enhanced_cells_define = HAS_ENHANCED_CELLS.define();
        assert!(
            cmds.iter().any(|c| c == &enhanced_cells_define),
            "expected {enhanced_cells_define} in load cmds: {cmds:?}"
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("data modify storage arcane:data"))
        );
        assert!(cmds.iter().any(|c| c.contains("Datapack loaded")));
    }

    // ── Enhanced cells dogfood — conditional branch semantics ─────────────────
    //
    // These three sub-checks are in one test function because they share the
    // same `drain_dyn_fns()` registry (global mutable state), and parallel
    // execution of separate tests would cause them to drain each other's entries.

    #[test]
    fn grant_enhanced_cells_branches() {
        // Drain any leftover entries from other tests
        sand_core::drain_dyn_fns();

        let cmds = grant_enhanced_cells();
        let fns = sand_core::drain_dyn_fns();

        // ── Parent function structure ─────────────────────────────────────────
        assert_eq!(cmds[0], "say Enhanced cells was called", "first cmd");
        assert_eq!(
            cmds.len(),
            3,
            "parent: say + if-branch + unless-branch: {cmds:?}"
        );

        let flag_obj = HAS_ENHANCED_CELLS.objective_name();
        assert!(
            cmds[1].contains(&format!("execute if score @s {flag_obj} matches 1")),
            "then arm should be 'if': {}",
            cmds[1]
        );
        assert!(
            cmds[2].contains(&format!("execute unless score @s {flag_obj} matches 1")),
            "else arm should be 'unless': {}",
            cmds[2]
        );
        assert!(
            cmds[1].contains("function __sand_local:sand/branches/"),
            "then arm calls branch fn"
        );
        assert!(
            cmds[2].contains("function __sand_local:sand/branches/"),
            "else arm calls branch fn"
        );

        // Exactly two branch functions registered for this one call
        assert_eq!(fns.len(), 2, "exactly 2 branch fns: {fns:?}");

        // ── Already-have branch: message + return fail ────────────────────────
        let already = fns
            .iter()
            .find(|(_, b)| b.iter().any(|c| c.contains("already have enhanced cells")))
            .expect("already-have branch not found");
        assert!(
            already.1.iter().any(|c| c == "return fail"),
            "already branch must return fail: {:?}",
            already.1
        );

        // ── Grant branch: attribute → message → flag set → return 0 ──────────
        let grant = fns
            .iter()
            .find(|(_, b)| b.iter().any(|c| c.contains("Granted enhanced cells")))
            .expect("grant branch not found");
        let gb = &grant.1;

        assert!(
            gb.iter().any(|c| c.contains("minecraft:max_health")),
            "sets max_health: {gb:?}"
        );
        assert!(
            gb.iter().any(|c| c.contains("Granted enhanced cells")),
            "message: {gb:?}"
        );
        assert!(
            gb.iter().any(|c| c.contains("scoreboard players set")
                && c.contains(&flag_obj)
                && c.ends_with(" 1")),
            "enables flag: {gb:?}"
        );
        assert!(gb.iter().any(|c| c == "return 0"), "return 0: {gb:?}");

        // Order: attribute → message → flag set → return
        let attr_i = gb.iter().position(|c| c.contains("max_health")).unwrap();
        let msg_i = gb
            .iter()
            .position(|c| c.contains("Granted enhanced cells"))
            .unwrap();
        let flag_i = gb
            .iter()
            .position(|c| c.contains(&flag_obj) && c.contains("set"))
            .unwrap();
        let ret_i = gb.iter().position(|c| c == "return 0").unwrap();
        assert!(attr_i < msg_i, "attribute before message");
        assert!(msg_i < flag_i, "message before flag set");
        assert!(flag_i < ret_i, "flag set before return");
    }

    #[test]
    fn enhanced_cells_reflect_damage_uses_typed_lowering() {
        let cmds = on_damaged_damage_nearby();
        assert_eq!(
            cmds,
            vec![
                "execute at @s as @e[distance=0.1..5,type=!minecraft:player] run damage @s 4 minecraft:generic"
            ]
        );
        assert!(!cmds.iter().any(|cmd| cmd.contains("damage @e[")));
        assert!(cmds.iter().any(|cmd| cmd.contains("execute at @s as @e[")));
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
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:multi_action");
        assert_eq!(
            json["title"]["text"],
            serde_json::Value::String("Welcome to Arcane Pack".to_string())
        );
        assert!(json["actions"].is_array());
        assert_eq!(json["actions"].as_array().unwrap().len(), 3);
        assert_eq!(
            json["actions"][0]["action"]["command"],
            serde_json::Value::String("/function arcane:cast_dash".to_string())
        );
    }

    #[test]
    fn open_welcome_menu_shows_dialog() {
        assert_eq!(
            open_welcome_menu(),
            vec!["dialog show @s __sand_local:welcome".to_string()]
        );
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

        // ─────────────────────────────────────────────────────────────────────
        // Dialog component — local component path and typed function actions
        // ─────────────────────────────────────────────────────────────────────

        let dialog = records
            .iter()
            .find(|r| r["path"] == "welcome" && r["dir"] == "dialog")
            .expect("welcome dialog component");
        assert_eq!(dialog["namespace"], "arcane");
        let dialog_json: serde_json::Value =
            serde_json::from_str(dialog["content"].as_str().unwrap()).expect("dialog JSON");
        assert_eq!(dialog_json["title"]["text"], "Welcome to Arcane Pack");
        assert_eq!(dialog_json["title"]["color"], "gold");
        assert_eq!(
            dialog_json["actions"][0]["action"]["command"],
            "/function arcane:cast_dash"
        );
        assert!(
            !dialog["content"].as_str().unwrap().contains("__sand_local"),
            "dialog export should resolve local refs"
        );

        let open_menu = records
            .iter()
            .find(|r| r["path"] == "open_welcome_menu" && r["dir"] == "function")
            .expect("open_welcome_menu function");
        assert_eq!(
            open_menu["content"], "dialog show @s arcane:welcome",
            "show_dialog should resolve local dialog refs during export"
        );
    }

    // ── EventBuilder / EventConfig phase-6 tests ──────────────────────────────

    #[test]
    fn villager_trade_config_has_correct_trigger() {
        use sand_core::DatapackComponent;
        let config = villager_trade_config();
        let adv = config.advancement("arcane:villager_trade", "arcane:on_villager_trade");
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["event"]["trigger"].as_str().unwrap(),
            "minecraft:villager_trade",
        );
        assert_eq!(
            json["rewards"]["function"].as_str().unwrap(),
            "arcane:on_villager_trade",
        );
    }

    #[test]
    fn villager_trade_reward_prologue_revokes_and_guards() {
        let config = villager_trade_config();
        let prologue = config.reward_prologue("arcane:villager_trade");

        // First command must revoke (AfterFire reset).
        assert_eq!(
            prologue[0],
            "advancement revoke @s only arcane:villager_trade"
        );

        // Second command must guard on merchant_rank.
        assert!(
            prologue[1].contains("unless"),
            "must use unless: {}",
            prologue[1]
        );
        assert!(
            prologue[1].contains("return 0"),
            "must return 0: {}",
            prologue[1]
        );
        assert!(
            prologue[1].contains("merchant_rank"),
            "must reference merchant_rank: {}",
            prologue[1]
        );
    }

    #[test]
    fn villager_trade_config_state_defines_merchant_rank_and_mana() {
        let config = villager_trade_config();
        let defs = config.state_defines();
        assert!(
            defs.iter().any(|d| d.contains("merchant_rank")),
            "should define merchant_rank: {defs:?}"
        );
        assert!(
            defs.iter().any(|d| d.contains("mana")),
            "should define mana: {defs:?}"
        );
    }

    #[test]
    fn ate_golden_apple_state_defines_returns_mana() {
        use crate::events::AteGoldenAppleEvent;
        use sand_core::event::AdvancementEvent;
        let defs = AteGoldenAppleEvent::state_defines();
        assert_eq!(defs.len(), 1);
        assert!(
            defs[0].contains("mana"),
            "expected mana define: {}",
            defs[0]
        );
    }

    #[test]
    fn event_state_init_accessor() {
        use crate::events::AteGoldenAppleEvent;
        use sand_core::Event;
        let defs = Event::<AteGoldenAppleEvent>::state_init();
        assert!(!defs.is_empty());
        assert!(defs.iter().any(|d| d.contains("mana")));
    }

    #[test]
    fn load_initializes_merchant_rank() {
        let cmds = load();
        assert!(
            cmds.iter().any(|c| c.contains("merchant_rank")),
            "load should define merchant_rank: {cmds:?}"
        );
    }

    #[test]
    fn on_villager_trade_prologue_and_body() {
        let cmds = on_villager_trade();
        // Prologue: revoke + guard.
        assert_eq!(cmds[0], "advancement revoke @s only arcane:villager_trade");
        assert!(
            cmds[1].contains("unless") && cmds[1].contains("return 0"),
            "{}",
            cmds[1]
        );
        // Body: rank up, mana bonus, actionbar.
        assert!(cmds.iter().any(|c| c.contains("scoreboard players add")
            && c.contains("merchant_rank")
            && c.contains("1")));
        assert!(cmds.iter().any(|c| c.contains("scoreboard players add")
            && c.contains("mana")
            && c.contains("5")));
    }
}
