//! # Trailforge
//!
//! The canonical Sand example datapack: upgradeable equipment and traversal.
//! Every chapter of the Sand book pulls its snippets from this project, so
//! each system here is deliberately small and readable.
//!
//! The pack adds a craftable **Grapple Core** and a pair of **Trail Striders**
//! (upgraded boots), a stamina resource that fuels a grapple dash, and a
//! small upgrade menu — exercising state, functions, events, items, recipes,
//! dialogs, conditions, and VFX through the `sand` façade crate alone.
//!
//! ## Build
//!
//! ```sh
//! cd examples/book_project
//! sand build          # full datapack under dist/
//! cargo build         # library + sand_export binary only
//! ```

use sand::event::AdvancementEvent;
use sand::event::trigger::InventoryChangedTrigger;
use sand::event::vanilla::{FirstJoin, OnDeath};
use sand::events::{PlayerSprintEvent, SandEvent, SandEventDispatch};
use sand::prelude::*;

// ── State ─────────────────────────────────────────────────────────────────────
//
// Scoreboard scores, flags, and timers plus one storage-backed variable.

/// Stamina fuels the grapple dash. Regenerates over time, capped at 100.
static STAMINA: ScoreVar<i32> = ScoreVar::new("trail_stamina");

/// Grapple dash cooldown (4 seconds).
static GRAPPLE: Cooldown = Cooldown::new("trail_grapple", Ticks::seconds(4));

/// Set once the player has crafted and claimed the Trail Striders upgrade.
static HAS_STRIDERS: Flag = Flag::new("trail_striders");

/// Set while the player is exhausted (stamina ran out).
static EXHAUSTED: Flag = Flag::new("trail_tired");

/// Stamina regen pulse: every 2 seconds each player regains some stamina.
static REGEN: Timer = Timer::new("trail_regen", Ticks::seconds(2));

/// Persistent pack tuning value kept in command storage.
static GRAPPLE_RANGE: StorageVar<i32> = StorageVar::new("trail:data", "config.grapple_range");

// ── Load / Tick ───────────────────────────────────────────────────────────────

/// Runs once on `/reload` and world load: define objectives, seed storage.
#[component(Load)]
pub fn load() {
    STAMINA.define();
    GRAPPLE.define();
    HAS_STRIDERS.define();
    EXHAUSTED.define();
    REGEN.define();
    DamageTracker::define();
    GRAPPLE_RANGE.set_int(8);
    cmd::tellraw(
        Selector::all_players(),
        Text::new("[Trailforge] loaded.").gold(),
    );
}

/// Runs every tick: advance timers, regenerate stamina, drive the actionbar.
#[component(Tick)]
pub fn tick() {
    GRAPPLE.tick_all_players();
    REGEN.tick_all_players();
    DamageTracker::tick_players();

    // Stamina regen pulse: when the regen timer expires, restore 10 stamina
    // to every player below the cap, then restart the timer.
    TypedExecute::as_players()
        .when(all![REGEN.expired("@s"), STAMINA.of("@s").lt(100)])
        .run(STAMINA.add(Selector::self_(), 10));
    TypedExecute::as_players()
        .when(REGEN.expired("@s"))
        .run(REGEN.start(Selector::self_()));

    // Exhaustion clears once stamina recovers past half.
    TypedExecute::as_players()
        .when(all![EXHAUSTED.of("@s").is_true(), STAMINA.of("@s").gte(50)])
        .run(cmd::function(
            ResourceLocation::new("trail", "recover").unwrap(),
        ));

    // Actionbar: grapple readiness for upgraded players.
    TypedExecute::as_players()
        .when(all![
            HAS_STRIDERS.of("@s").is_true(),
            GRAPPLE.ready("@s"),
            STAMINA.of("@s").gte(30),
            EXHAUSTED.of("@s").is_false(),
        ])
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Grapple ready").aqua().bold(true),
        ));

    // Actionbar: warn players who were hurt within the last 3 seconds
    // (systems-damage feature).
    TypedExecute::as_players()
        .when(DamageTracker::hurt_within("@s", Ticks::seconds(3)))
        .run(Actionbar::show(
            Selector::self_(),
            Text::new("Catch your breath...").red(),
        ));
}

// ── Items ─────────────────────────────────────────────────────────────────────

/// The craftable upgrade material. `#[item]` generates a `GrappleCore` struct
/// with a `PREDICATE` for `execute if items` checks.
#[item]
pub fn grapple_core() -> CustomItem {
    CustomItem::new("minecraft:heart_of_the_sea")
        .custom_data("grapple_core")
        .component(ItemComponent::custom_name(
            Text::new("Grapple Core").aqua().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Pulses with stored momentum").gray(),
            Text::new("Craft into Trail Striders").dark_gray(),
        ]))
        .component(ItemComponent::rarity(Rarity::Rare))
        .component(ItemComponent::max_stack_size(1))
}

/// The upgraded boots granted by `trail:claim_striders`.
pub fn trail_striders() -> CustomItem {
    CustomItem::new(ItemId::minecraft("leather_boots").unwrap())
        .component(ItemComponent::custom_name(
            Text::new("Trail Striders").gold().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Light as a rumor").gray(),
        ]))
        .component(ItemComponent::custom_data_marker("trail_striders"))
        .component(ItemComponent::rarity(Rarity::Epic))
        .component(ItemComponent::attribute_modifier(
            AttributeModifier::new(AttributeId::MovementSpeed)
                .amount(0.02)
                .operation(AttributeOperation::AddValue)
                .slot(EquipmentSlotGroup::Feet),
        ))
}

// ── Recipe ────────────────────────────────────────────────────────────────────

/// Shaped recipe for the Grapple Core: string frame around an ender pearl.
#[component]
pub fn grapple_core_recipe() -> ShapedRecipe {
    ShapedRecipe::new("trail:grapple_core".parse().unwrap())
        .pattern(["SSS", "SES", "SSS"])
        .key('S', Ingredient::item("minecraft:string"))
        .key('E', Ingredient::item("minecraft:ender_pearl"))
        .result(RecipeResult::new("minecraft:heart_of_the_sea", 1))
        .category("equipment")
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Grapple dash entry point: gate on upgrade, stamina, cooldown, exhaustion.
#[function("trail:grapple")]
pub fn grapple() {
    TypedExecute::as_players_at_self()
        .when(all![
            HAS_STRIDERS.of("@s").is_true(),
            GRAPPLE.ready("@s"),
            STAMINA.of("@s").gte(30),
            EXHAUSTED.of("@s").is_false(),
        ])
        .run(cmd::function(
            ResourceLocation::new("trail", "grapple/execute").unwrap(),
        ));
}

/// Applies the grapple dash: pay stamina, start the cooldown, launch, sparkle.
#[function("trail:grapple/execute")]
pub fn grapple_execute() {
    STAMINA.remove(Selector::self_(), 30);
    GRAPPLE.start(Selector::self_());
    cmd::effect_give(Selector::self_(), EffectId::Speed)
        .duration(Ticks::seconds(3))
        .amplifier(2)
        .particles(false);
    cmd::effect_give(Selector::self_(), EffectId::SlowFalling)
        .duration(Ticks::seconds(4))
        .particles(false);
    grapple_vfx().play_at(Selector::self_());
    cmd::tellraw(Selector::self_(), Text::new("Whoosh!").aqua());
}

/// Clears exhaustion once stamina has recovered (called from `tick`).
#[function("trail:recover")]
pub fn recover() {
    EXHAUSTED.disable(Selector::self_());
    cmd::tellraw(
        Selector::self_(),
        Text::new("You feel steady again.").green(),
    );
}

/// Claim the Trail Striders upgrade while holding a Grapple Core.
///
/// Demonstrates the `if_()` grouped-branch API: the `if` arm rejects players
/// who already own the upgrade; the `else` arm consumes nothing (teaching
/// simplicity), grants the boots, and sets the flag.
#[function("trail:claim_striders")]
pub fn claim_striders() {
    if_(HAS_STRIDERS.of("@s").is_true())
        .then_all(mcfunction![
            cmd::tellraw(
                Selector::self_(),
                Text::new("You already wear Trail Striders.").red(),
            );
            cmd::return_fail();
        ])
        .else_all(mcfunction![
            cmd::raw(format!("give @s {}", trail_striders()));
            HAS_STRIDERS.enable("@s");
            cmd::tellraw(
                Selector::self_(),
                Text::new("Trail Striders bound to your feet!").gold(),
            );
            cmd::return_cmd(1);
        ]);
}

/// Opens the trailhead menu dialog for the current player.
#[function("trail:menu")]
pub fn open_menu() {
    cmd::show_dialog(Selector::self_(), DialogRef::local("trailhead"));
}

// ── Vfx ───────────────────────────────────────────────────────────────────────

/// Reusable dash effect: a particle burst plus a launch sound.
fn grapple_vfx() -> Vfx {
    Vfx::new("grapple_dash")
        .particle(
            VfxParticle::named("minecraft:cloud")
                .count(24)
                .spread(0.4, 0.2, 0.4),
        )
        .sound(
            VfxSound::new("minecraft:entity.ender_pearl.throw")
                .source(SoundSource::Player)
                .volume(0.8)
                .pitch(1.4),
        )
}

// ── Dialog ────────────────────────────────────────────────────────────────────

/// The trailhead menu: one button per pack action.
#[component]
pub fn trailhead_dialog() -> Dialog {
    Dialog::multi_action_local("trailhead")
        .title(Text::new("Trailforge").gold())
        .body(DialogBody::text(Text::new("Choose your path.").aqua()))
        .button(
            DialogButton::new(Text::new("Grapple dash").aqua())
                .action(DialogAction::run_function(grapple)),
        )
        .button(
            DialogButton::new(Text::new("Claim Trail Striders").gold())
                .action(DialogAction::run_function(claim_striders)),
        )
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Advancement-backed custom event: fires when a Grapple Core enters the
/// player's inventory (guarded so it stays quiet after the upgrade).
pub struct ObtainedGrappleCoreEvent;

impl AdvancementEvent for ObtainedGrappleCoreEvent {
    type Trigger = InventoryChangedTrigger;

    fn trigger() -> Self::Trigger {
        InventoryChangedTrigger::new()
            .item(ItemPredicate::id("minecraft:heart_of_the_sea").custom_data_key("grapple_core"))
    }

    fn guard() -> Option<Condition> {
        Some(HAS_STRIDERS.of("@s").is_false())
    }
}

/// Tick-backed custom event: fires when a player runs out of stamina.
/// The handler sets the `EXHAUSTED` flag, which also stops the event from
/// re-firing until `trail:recover` clears it.
pub struct StaminaExhaustedEvent;

impl SandEvent for StaminaExhaustedEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick()
            .as_players()
            .when(all![STAMINA.of("@s").lte(0), EXHAUSTED.of("@s").is_false(),])
    }
}

/// Chained event: composes off the built-in sprint detection and only fires
/// while the sprinting player is exhausted.
pub struct SprintingWhileExhaustedEvent;

impl SandEvent for SprintingWhileExhaustedEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<PlayerSprintEvent>().when(EXHAUSTED.of("@s").is_true())
    }
}

/// First-ever join: seed stamina and greet the player.
#[event]
pub fn on_first_join(event: FirstJoin) {
    STAMINA.set(event.player(), 100);
    Title::of(event.player())
        .title(Text::new("Trailforge").gold().bold(true))
        .subtitle(Text::new("Craft a Grapple Core to begin").aqua())
        .build();
}

/// Death resets the traversal state so respawned players start steady.
#[event]
pub fn on_death(event: OnDeath) {
    EXHAUSTED.disable(Selector::self_());
    GRAPPLE.stop(Selector::self_());
    STAMINA.set(event.player(), 100);
}

/// A Grapple Core arrives: point the player at the upgrade.
#[event]
pub fn on_obtained_grapple_core(event: Event<ObtainedGrappleCoreEvent>) {
    cmd::tellraw(
        event.player(),
        Text::new("The core hums. Run /function trail:claim_striders").aqua(),
    );
    cmd::call(open_menu);
}

/// Stamina hit zero: mark the player exhausted.
#[event]
pub fn on_stamina_exhausted(_event: StaminaExhaustedEvent) {
    EXHAUSTED.enable("@s");
    cmd::tellraw(
        Selector::self_(),
        Text::new("You are exhausted!").red().bold(true),
    );
}

/// Sprinting while exhausted is punished with brief slowness.
#[event]
pub fn on_sprint_while_exhausted(_event: SprintingWhileExhaustedEvent) {
    cmd::effect_give(Selector::self_(), EffectId::Slowness)
        .duration(Ticks::seconds(2))
        .particles(false);
}

// ── Export hook (required by `sand build`) ────────────────────────────────────

/// Invoked by the generated `sand_export` binary.
///
/// Calling into the library from the binary forces the linker to keep this
/// object file, which is required for `inventory` registrations to run.
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    let resolved = match sand::version::resolve_export_caps(mc_version) {
        Ok(resolved) => resolved,
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    };
    match sand::advanced::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    ) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_defines_state_and_storage() {
        let cmds = load();
        let stamina_define = STAMINA.define();
        assert!(cmds.contains(&stamina_define), "defines stamina: {cmds:?}");
        assert!(
            cmds.iter().any(|c| c.contains("storage trail:data")),
            "seeds storage: {cmds:?}"
        );
    }

    #[test]
    fn tick_regenerates_and_warns() {
        let cmds = tick();
        assert!(
            cmds.iter().any(|c| c.contains("Grapple ready")),
            "readiness actionbar: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("Catch your breath")),
            "damage warning: {cmds:?}"
        );
    }

    #[test]
    fn grapple_execute_pays_stamina_and_plays_vfx() {
        let cmds = grapple_execute();
        assert!(
            cmds.iter().any(|c| c.contains("scoreboard players remove")),
            "pays stamina: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("particle")),
            "vfx particle: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("playsound")),
            "vfx sound: {cmds:?}"
        );
    }

    #[test]
    fn grapple_core_predicate_matches_custom_data() {
        assert!(GrappleCore::PREDICATE.contains("grapple_core"));
        assert_eq!(GrappleCore::BASE, "minecraft:heart_of_the_sea");
    }
}
