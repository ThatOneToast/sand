//! Spell system example — demonstrates the full batch-2 API surface.
//!
//! Run with:
//! ```sh
//! cargo run --example spell_system -p sand-core
//! ```
//!
//! This example implements a minimal mana-based spell system:
//! - `MANA` — an i32 scoreboard variable tracking current mana (0–100)
//! - `CASTING` — a boolean flag set while a spell is active
//! - `ACTIVE_SPELL` — an NBT storage var holding the spell name string
//! - Spells gated behind `all!`/`any!` conditions
//! - A welcome dialog (1.21.5+ only) shown on first join

use sand_core::prelude::*;
use sand_core::{all, any, mcfunction};

// ── State variables ───────────────────────────────────────────────────────────

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static CASTING: Flag = Flag::new("casting");
static ACTIVE_SPELL: StorageVar<String> = StorageVar::new("spells:data", "active_spell");
static DASH_CD: Cooldown = Cooldown::new("dash_cd", Ticks::new(40));

// ── Load ──────────────────────────────────────────────────────────────────────

fn load_commands() -> Vec<String> {
    mcfunction![
        MANA.define();
        CASTING.define();
        DASH_CD.define();
        MANA.set("@a", 100);
    ]
}

// ── Tick ──────────────────────────────────────────────────────────────────────

fn tick_commands() -> Vec<String> {
    let mut cmds = mcfunction![
        DASH_CD.tick("@a");
        MANA.add("@a", 1);
    ];

    // Cap mana at 100 via condition
    let over_max = MANA.of("@a").gte(101);
    let cap = TypedExecute::as_players()
        .when(over_max)
        .run(MANA.set("@s", 100));
    cmds.extend(cap);
    cmds
}

// ── Fireball spell ────────────────────────────────────────────────────────────

fn fireball_commands() -> Vec<String> {
    let has_mana = MANA.of("@s").gte(30);
    let not_casting = CASTING.of("@s").is_false();

    let cast_guard = all![has_mana, not_casting];

    let mut cmds = TypedExecute::as_self_at_self()
        .when(cast_guard)
        .run("summon minecraft:fireball ~ ~1.5 ~ {direction:[0.0,0.0,1.0],ExplosionPower:2}");

    cmds.push(MANA.remove("@s", 30));
    cmds.push(CASTING.enable("@s"));
    cmds.push(ACTIVE_SPELL.set_string("fireball"));
    cmds
}

// ── Heal spell ────────────────────────────────────────────────────────────────

fn heal_commands() -> Vec<String> {
    let has_mana = MANA.of("@s").gte(20);

    TypedExecute::as_self_at_self()
        .when(has_mana)
        .run("effect give @s minecraft:regeneration 5 2")
}

// ── Welcome dialog (version-gated) ────────────────────────────────────────────

fn maybe_welcome_dialog(profile: &VersionProfile) -> Option<Dialog> {
    if !profile.supports_dialogs() {
        return None;
    }

    Some(
        Dialog::notice("spells:welcome")
            .title("Welcome to the Spell System!")
            .body(DialogBody::text(
                "You have 100 mana. Run /function spells:fireball to cast!",
            ))
            .button(
                DialogButton::new("Got it!")
                    .action(DialogAction::run_command("/function spells:greet")),
            )
            .pause(true),
    )
}

// ── Active spell default via NBT existence check ───────────────────────────────

fn active_spell_init() -> Vec<String> {
    // If no active_spell stored yet, default it to "none"
    let has_active = ACTIVE_SPELL.exists();
    TypedExecute::as_self_at_self()
        .unless(has_active)
        .run(ACTIVE_SPELL.set_string("none"))
}

// ── Complex condition combinator showcase ─────────────────────────────────────

fn complex_act_commands() -> Vec<String> {
    let high_mana = MANA.of("@s").gte(50);
    let not_casting = CASTING.of("@s").is_false();
    let dash_ready = DASH_CD.ready("@s");

    // "(has high mana OR dash is ready) AND not casting"
    let can_act = all![any![high_mana, dash_ready], not_casting];

    TypedExecute::as_players()
        .when(can_act)
        .run("say I can act!")
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Spell System — Sand batch-2 example ===\n");

    println!("--- load ---");
    for cmd in load_commands() {
        println!("  {cmd}");
    }

    println!("\n--- tick ---");
    for cmd in tick_commands() {
        println!("  {cmd}");
    }

    println!("\n--- fireball ---");
    for cmd in fireball_commands() {
        println!("  {cmd}");
    }

    println!("\n--- heal ---");
    for cmd in heal_commands() {
        println!("  {cmd}");
    }

    println!("\n--- active_spell init (NBT existence) ---");
    for cmd in active_spell_init() {
        println!("  {cmd}");
    }

    println!("\n--- complex act condition (any/all nesting) ---");
    for cmd in complex_act_commands() {
        println!("  {cmd}");
    }

    println!("\n--- welcome dialog (1.21.5+) ---");
    let v = MinecraftVersion::parse("1.21.5").unwrap();
    let profile = VersionProfile::resolve(&v).unwrap();
    match maybe_welcome_dialog(&profile) {
        Some(d) => {
            println!("  path: {}", d.resource_path());
            println!("  json: {}", d.to_json());
        }
        None => println!("  (not supported on this version)"),
    }

    println!("\n--- welcome dialog (1.21.4 — no dialogs) ---");
    let v_old = MinecraftVersion::parse("1.21.4").unwrap();
    let profile_old = VersionProfile::resolve(&v_old).unwrap();
    match maybe_welcome_dialog(&profile_old) {
        Some(_) => println!("  (unexpected)"),
        None => println!("  (correctly gated — dialogs not supported on 1.21.4)"),
    }
}
