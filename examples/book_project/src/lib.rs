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

// ANCHOR: imports
use sand::command::Anchor;
use sand::event::AdvancementEvent;
use sand::event::trigger::InventoryChangedTrigger;
use sand::event::vanilla::{FirstJoin, OnDeath};
use sand::events::{PlayerSprintEvent, SandEvent, SandEventDispatch};
use sand::prelude::*;
use sand::systems::raycast::BlockRaycast;
// ANCHOR_END: imports

// ── State ─────────────────────────────────────────────────────────────────────
//
// One scoped gameplay-data schema plus one command-storage reference.

// ANCHOR: state
#[derive(State)]
#[state(namespace = "trail", scope = player)]
#[allow(dead_code)]
struct Traversal {
    /// Stamina fuels the grapple dash. Regenerates over time, capped at 100.
    #[state(default = 100, min = 0, max = 100)]
    stamina: Score,
    /// Grapple dash cooldown.
    #[state(auto_tick)]
    grapple: Cooldown,
    /// Set once the player has claimed the Trail Striders upgrade.
    has_striders: Flag,
    /// Set while the player is exhausted.
    exhausted: Flag,
    /// Stamina regeneration pulse.
    #[state(auto_tick)]
    regen: Timer,
}

// Stateless custom dispatch markers have no event participant value. Their
// as_players dispatcher establishes the current player used by this binding.
fn traversal() -> TraversalBound {
    Traversal::on(EntityContext::<PlayerKind>::default())
}

fn grapple_range() -> NbtRef<i32> {
    Nbt::storage(ResourceLocation::new("trail", "data").unwrap()).typed_path("config.grapple_range")
}
// ANCHOR_END: state

// ── Load / Tick ───────────────────────────────────────────────────────────────

// ANCHOR: load
/// Runs once on `/reload` and world load. State owns its own schema lifecycle.
#[datapack_component(Load)]
pub fn load() {
    grapple_range().set(8);
    cmd::tellraw(Target::players(), Text::new("[Trailforge] loaded.").gold());
}
// ANCHOR_END: load

// ANCHOR: tick
/// Runs every tick: advance timers, regenerate stamina, drive the actionbar.
#[datapack_component(Tick)]
pub fn tick() {
    Target::players().each(|player| {
        let player = *player;
        let state = Traversal::on(player);
        mcfunction![
            when(state.regen.elapsed()).then_all(mcfunction![
                when(state.stamina.matches(..100).unwrap()).then_one(state.stamina.add(10));
                state.regen.start(Ticks::seconds(2));
            ]);
            when(all![
                state.exhausted.is_enabled(),
                state.stamina.matches(50..).unwrap(),
            ]).then_one(cmd::function(recover));
            when(ability_ready(player, 30)).then_one(Actionbar::show(
                Target::from(player), Text::new("Grapple ready").aqua().bold(true),
            ));
        ]
    });
}
// ANCHOR_END: tick

// ── Items ─────────────────────────────────────────────────────────────────────

// ANCHOR: item_grapple_core
/// The craftable upgrade material. `#[custom_item]` generates a `GrappleCore` struct
/// with a `PREDICATE` for `execute if items` checks.
#[custom_item]
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
        .component(ItemComponent::rarity(ItemRarity::Rare))
        .component(ItemComponent::max_stack_size(1))
}
// ANCHOR_END: item_grapple_core

// ANCHOR: item_trail_striders
/// The upgraded boots granted by `trail:claim_striders`.
pub fn trail_striders() -> CustomItem {
    CustomItem::new(ItemId::minecraft("leather_boots").unwrap())
        .component(ItemComponent::custom_name(
            Text::new("Trail Striders").gold().bold(true),
        ))
        .component(ItemComponent::lore(vec![
            Text::new("Light as a rumor").gray(),
        ]))
        .custom_data("trail_striders")
        .component(ItemComponent::rarity(ItemRarity::Epic))
        .component(ItemComponent::attribute_modifier(
            AttributeModifier::new(AttributeType::MovementSpeed)
                .amount(0.02)
                .operation(AttributeOperation::AddValue)
                .slot(EquipmentSlotGroup::Feet),
        ))
}
// ANCHOR_END: item_trail_striders

// ── Recipe ────────────────────────────────────────────────────────────────────

// ANCHOR: recipe
/// Shaped recipe for the Grapple Core: string frame around an ender pearl.
#[datapack_component]
pub fn grapple_core_recipe() -> ShapedRecipe {
    ShapedRecipe::new("trail:grapple_core".parse().unwrap())
        .pattern(["SSS", "SES", "SSS"])
        .key('S', Ingredient::item("minecraft:string"))
        .key('E', Ingredient::item("minecraft:ender_pearl"))
        .result(RecipeResult::custom_item(&grapple_core()).unwrap())
        .category("equipment")
}
// ANCHOR_END: recipe

// ── Functions ─────────────────────────────────────────────────────────────────

// ANCHOR: entity_capabilities
/// A façade-only example that discovers entity operations from the context.
pub fn capability_facade_example() -> Actions {
    let active = EntityTag::new("trail_active").unwrap();

    Target::players().each(|player| {
        let boots = player.equipment().boots();
        let first_hotbar_slot = player.inventory().hotbar(0).unwrap();
        let wearing_boots = boots.matches(vanilla::Item::LeatherBoots).unwrap();
        let speed = player
            .living()
            .give_effect(EffectId::Speed)
            .duration(Ticks::seconds(1))
            .particles(false);

        let mut commands = mcfunction![
            player.identity().add_tag(&active).unwrap();
            player
                .transform()
                .face_position(Vec3::absolute(0.0, 80.0, 0.0))
                .unwrap();
            first_hotbar_slot.nbt().get().to_string();
        ];
        commands.extend(
            TypedExecute::as_self_at_self()
                .when(wearing_boots)
                .run(speed),
        );
        commands
    })
}
// ANCHOR_END: entity_capabilities

// ANCHOR: fn_grapple
/// Grapple entry: validate the player once, then look for a gold-block anchor.
#[function("trail:grapple", context = player)]
pub fn grapple(player: EntityContext<PlayerKind>) {
    attempt_anchor(player, 30, grapple_ray());
}
// ANCHOR_END: fn_grapple

// ANCHOR: fn_grapple_execute
/// Accepted gold anchor: pay stamina, start cooldown, apply traversal effects.
/// Internal callback; the entry and hit functions already validated eligibility.
#[function("trail:grapple/execute", context = player)]
pub fn grapple_execute(player: EntityContext<PlayerKind>) {
    let state = Traversal::on(player);
    state.stamina.subtract(30);
    state.grapple.start(Ticks::seconds(4));
    player
        .living()
        .give_effect(EffectId::Speed)
        .duration(Ticks::seconds(3))
        .amplifier(2)
        .particles(false);
    player
        .living()
        .give_effect(EffectId::SlowFalling)
        .duration(Ticks::seconds(4))
        .particles(false);
    grapple_vfx().play_at(Target::self_());
    cmd::tellraw(Target::self_(), Text::new("Whoosh!").aqua());
}
// ANCHOR_END: fn_grapple_execute

// ANCHOR: anchor_workflow
/// Shared eligibility for the actionbar and both anchor consumers.
fn ability_ready(player: EntityContext<PlayerKind>, cost: i32) -> Condition {
    let state = Traversal::on(player);
    all![
        state.has_striders.is_enabled(),
        player
            .equipment()
            .boots()
            .matches(trail_striders().item_predicate().unwrap())
            .unwrap(),
        state.grapple.ready(),
        state.stamina.matches(cost..).unwrap(),
        state.exhausted.is_disabled(),
    ]
}

/// Ordinary Rust composition: one eligibility decision before any dependent writes.
/// A sufficient balance permits the cast; only an accepted anchor spends the
/// full cost and starts cooldown. Misses and invalid hits spend nothing. This
/// accepts an interaction, not a transaction proving vanilla effect success.
/// Restoration uses the same bounded State: +10 saturates at 100 (95 restores
/// only five); a full pool skips the write while its regen timer still restarts.
fn attempt_anchor(player: EntityContext<PlayerKind>, cost: i32, ray: BlockRaycast) -> Actions {
    TypedExecute::as_self_at_self()
        .when(ability_ready(player, cost))
        .run(ray.cast())
}

fn anchor_ray(name: &str, distance: f64, hit: impl FunctionRef) -> BlockRaycast {
    BlockRaycast::new(ResourceLocation::local(name).unwrap())
        .origin(Anchor::Eyes)
        .first_collision()
        .pass_through(BlockPredicate::new().blocks(vec![BlockId::minecraft("air").unwrap()]))
        .max_distance(distance)
        .step(0.25)
        .on_hit(hit)
}

#[datapack_component]
fn grapple_ray() -> BlockRaycast {
    anchor_ray("grapple", 8.0, grapple_hit)
}

/// At the first collision, retain the caster and validate the sampled block once.
#[function("grapple/hit", context = player)]
fn grapple_hit() {
    when(
        WorldContext::current()
            .block(BlockPos::here())
            .matches(vanilla::Block::GoldBlock),
    )
    .then_one(cmd::function(grapple_execute));
}

/// A cheaper, shorter iron-anchor ability reuses the same resource and cooldown.
#[function("trail:scout", context = player)]
pub fn scout(player: EntityContext<PlayerKind>) {
    attempt_anchor(player, 10, scout_ray());
}

#[datapack_component]
fn scout_ray() -> BlockRaycast {
    anchor_ray("scout", 4.0, scout_hit)
}

#[function("scout/hit", context = player)]
fn scout_hit() {
    when(
        WorldContext::current()
            .block(BlockPos::here())
            .matches(vanilla::Block::IronBlock),
    )
    .then_one(cmd::function(scout_execute));
}

/// Internal accepted-hit callback, not an independently guarded entrypoint.
#[function("trail:scout/execute", context = player)]
fn scout_execute(player: EntityContext<PlayerKind>) {
    let state = Traversal::on(player);
    state.stamina.subtract(10);
    state.grapple.start(Ticks::seconds(1));
    player
        .living()
        .give_effect(EffectId::Speed)
        .duration(Ticks::seconds(1))
        .particles(false);
}
// ANCHOR_END: anchor_workflow

// ANCHOR: fn_recover
/// Clears exhaustion once stamina has recovered (called from `tick`).
#[function("trail:recover", context = player)]
pub fn recover(player: EntityContext<PlayerKind>) {
    Traversal::on(player).exhausted.disable();
    cmd::tellraw(Target::self_(), Text::new("You feel steady again.").green());
}
// ANCHOR_END: fn_recover

// ANCHOR: fn_claim_striders
/// Consume one Grapple Core from this player's inventory to claim Trail Striders.
///
/// Already-upgraded players and players without a matching core change nothing.
/// The successful branch consumes one core and gives the boots to the current
/// player; it does not replace occupied armor slots or equip the boots.
#[function("trail:claim_striders", context = player)]
pub fn claim_striders(player: EntityContext<PlayerKind>) {
    let state = Traversal::on(player);
    let inventory = player.inventory();
    let core = grapple_core().item_predicate().unwrap();
    if_(state.has_striders.is_enabled())
        .then_all(mcfunction![
            cmd::tellraw(
                Target::self_(),
                Text::new("You already unlocked Trail Striders.").red(),
            );
            cmd::return_fail();
        ])
        .else_all(mcfunction![
            if_(inventory.contains(&core))
                .then_all(mcfunction![
                    inventory.remove(&core, 1);
                    cmd::give(Target::self_(), trail_striders());
                    state.has_striders.enable();
                    cmd::tellraw(
                        Target::self_(),
                        Text::new("Trail Striders unlocked. Equip your new boots!").gold(),
                    );
                    cmd::return_cmd(1);
                ])
                .else_all(mcfunction![
                    cmd::tellraw(
                        Target::self_(),
                        Text::new("Craft a Grapple Core before claiming Trail Striders.").red(),
                    );
                    cmd::return_fail();
                ]);
        ]);
}
// ANCHOR_END: fn_claim_striders

// ANCHOR: fn_open_menu
/// Opens the trailhead menu dialog for the current player.
#[function("trail:menu", context = player)]
pub fn open_menu() {
    cmd::show_dialog(Target::self_(), DialogId::local("trailhead"));
}
// ANCHOR_END: fn_open_menu

// ── Vfx ───────────────────────────────────────────────────────────────────────

// ANCHOR: vfx
/// Reusable dash effect: a particle burst plus a launch sound.
fn grapple_vfx() -> Vfx {
    Vfx::new("grapple_dash")
        .particle(
            VfxParticle::named(ParticleId::minecraft("cloud").unwrap())
                .count(24)
                .spread(0.4, 0.2, 0.4),
        )
        .sound(
            VfxSound::new(SoundEventId::minecraft("entity.ender_pearl.throw").unwrap())
                .source(SoundSource::Player)
                .volume(0.8)
                .pitch(1.4),
        )
}
// ANCHOR_END: vfx

// ── Dialog ────────────────────────────────────────────────────────────────────

// ANCHOR: dialog
/// The trailhead menu: one button per pack action.
#[datapack_component]
pub fn trailhead_dialog() -> Dialog {
    Dialog::multi_action_local("trailhead")
        .title(Text::new("Trailforge").gold())
        .body(DialogBody::text(Text::new("Choose your path.").aqua()))
        .button(
            DialogButton::new(Text::new("Grapple dash").aqua())
                .action(DialogAction::callback(grapple)),
        )
        .button(
            DialogButton::new(Text::new("Claim Trail Striders").gold())
                .action(DialogAction::callback(claim_striders)),
        )
}
// ANCHOR_END: dialog

// ── Events ────────────────────────────────────────────────────────────────────

// ANCHOR: event_obtained_grapple_core
/// Advancement-backed custom event: fires when a Grapple Core enters the
/// player's inventory (guarded so it stays quiet after the upgrade).
pub struct ObtainedGrappleCoreEvent;

impl AdvancementEvent for ObtainedGrappleCoreEvent {
    type Trigger = InventoryChangedTrigger;

    fn trigger() -> Self::Trigger {
        InventoryChangedTrigger::new().item(
            grapple_core()
                .item_predicate()
                .expect("valid Grapple Core identity"),
        )
    }

    fn guard() -> Option<Condition> {
        Some(traversal().has_striders.is_disabled())
    }
}
// ANCHOR_END: event_obtained_grapple_core

// ANCHOR: event_stamina_exhausted
/// Tick-backed custom event: fires when a player runs out of stamina.
/// The handler sets the `EXHAUSTED` flag, which also stops the event from
/// re-firing until `trail:recover` clears it.
pub struct StaminaExhaustedEvent;

impl SandEvent for StaminaExhaustedEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players().when(all![
            traversal().stamina.matches(..=0).unwrap(),
            traversal().exhausted.is_disabled(),
        ])
    }
}
// ANCHOR_END: event_stamina_exhausted

// ANCHOR: event_sprint_while_exhausted
/// Chained event: composes off the built-in sprint detection and only fires
/// while the sprinting player is exhausted.
pub struct SprintingWhileExhaustedEvent;

impl SandEvent for SprintingWhileExhaustedEvent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<PlayerSprintEvent>().when(traversal().exhausted.is_enabled())
    }
}
// ANCHOR_END: event_sprint_while_exhausted

// ANCHOR: event_on_first_join
/// First-ever join: seed stamina and greet the player.
#[on_event]
pub fn on_first_join(event: FirstJoin) {
    Traversal::on(event.player()).stamina.set(100);
    Title::of(Target::from(event.player()))
        .title(Text::new("Trailforge").gold().bold(true))
        .subtitle(Text::new("Craft a Grapple Core to begin").aqua())
        .build();
}
// ANCHOR_END: event_on_first_join

// ANCHOR: event_on_death
/// Death resets the traversal state so respawned players start steady.
#[on_event]
pub fn on_death(event: OnDeath) {
    let state = Traversal::on(event.player());
    state.exhausted.disable();
    state.grapple.start(Ticks::new(0));
    state.stamina.set(100);
}
// ANCHOR_END: event_on_death

// ANCHOR: event_on_obtained_grapple_core
/// A Grapple Core arrives: point the player at the upgrade.
#[on_event]
pub fn on_obtained_grapple_core(event: Event<ObtainedGrappleCoreEvent>) {
    cmd::tellraw(
        Target::from(event.player()),
        Text::new("The core hums. Run /function trail:claim_striders").aqua(),
    );
    cmd::function(open_menu);
}
// ANCHOR_END: event_on_obtained_grapple_core

// ANCHOR: event_on_stamina_exhausted
/// Stamina hit zero: mark the player exhausted.
#[on_event]
pub fn on_stamina_exhausted(_event: StaminaExhaustedEvent) {
    traversal().exhausted.enable();
    cmd::tellraw(
        Target::self_(),
        Text::new("You are exhausted!").red().bold(true),
    );
}
// ANCHOR_END: event_on_stamina_exhausted

// ANCHOR: event_on_sprint_while_exhausted
/// Sprinting while exhausted is punished with brief slowness.
#[on_event]
pub fn on_sprint_while_exhausted(_event: SprintingWhileExhaustedEvent) {
    EntityContext::<PlayerKind>::default()
        .living()
        .give_effect(EffectId::Slowness)
        .duration(Ticks::seconds(2))
        .particles(false);
}
// ANCHOR_END: event_on_sprint_while_exhausted

// ── Export hook (required by `sand build`) ────────────────────────────────────

/// Invoked by the generated `sand_export` binary.
///
/// Calling into the library from the binary forces the linker to keep this
/// object file, which is required for `inventory` registrations to run.
// ANCHOR: export_hook
#[doc(hidden)]
pub fn __sand_export(namespace: &str, mc_version: &str) {
    match sand::advanced::try_export_pack_json(namespace, mc_version) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            eprintln!("sand export failed: {e}");
            std::process::exit(1);
        }
    }
}
// ANCHOR_END: export_hook

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn emitted(actions: Actions) -> Vec<String> {
        use sand::registration::{ComponentContent, DatapackComponent};
        let function =
            sand::component::McFunction::new("test:body".parse().unwrap()).commands(actions);
        let ComponentContent::Text(content) = function.try_content().unwrap() else {
            panic!("a function must export text");
        };
        content.lines().map(str::to_owned).collect()
    }

    fn exported_functions() -> std::collections::BTreeMap<String, String> {
        let export: serde_json::Value =
            serde_json::from_str(&sand::advanced::try_export_pack_json("trail", "26.2").unwrap())
                .unwrap();
        let records: Vec<serde_json::Value> =
            serde_json::from_value(export["components"].clone()).unwrap();
        records
            .into_iter()
            .filter(|record| record["dir"] == "function")
            .map(|record| {
                (
                    record["path"].as_str().unwrap().to_owned(),
                    record["content"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    fn reachable_body(
        functions: &std::collections::BTreeMap<String, String>,
        entry: &str,
    ) -> String {
        let mut pending = vec![entry.to_owned()];
        let mut seen = std::collections::BTreeSet::new();
        let mut result = String::new();
        while let Some(path) = pending.pop() {
            if !seen.insert(path.clone()) {
                continue;
            }
            let body = &functions[&path];
            result.push_str(body);
            result.push('\n');
            for line in body.lines() {
                let words = line.split_whitespace().collect::<Vec<_>>();
                for pair in words.windows(2) {
                    if pair[0] == "function"
                        && let Some(target) = pair[1].strip_prefix("trail:")
                    {
                        pending.push(target.to_owned());
                    }
                }
            }
        }
        result
    }

    #[test]
    fn menu_buttons_use_survival_callbacks() {
        use sand::registration::{ComponentContent, DatapackComponent};
        let ComponentContent::Json(dialog) = trailhead_dialog().try_content().unwrap() else {
            panic!("dialog must emit JSON");
        };
        let actions = dialog["actions"].as_array().unwrap();
        assert_eq!(actions.len(), 2);
        for button in actions {
            assert_eq!(button["action"]["type"], "minecraft:run_command");
            assert!(
                button["action"]["command"]
                    .as_str()
                    .unwrap()
                    .starts_with("/trigger ")
            );
        }
        let functions = exported_functions();
        let dispatcher = &functions["__sand_dialog_tick"];
        assert!(dispatcher.contains("run function trail:grapple"));
        assert!(dispatcher.contains("run function trail:claim_striders"));
    }

    #[test]
    fn load_seeds_storage() {
        let cmds = emitted(load());
        assert!(
            cmds.iter().any(|c| c.contains("storage trail:data")),
            "seeds storage: {cmds:?}"
        );
    }

    #[test]
    fn tick_drives_stamina_recovery_and_readiness() {
        let functions = exported_functions();
        let body = reachable_body(&functions, "tick");
        assert!(body.contains("Grapple ready"));
        assert!(
            !body.contains("__sand_system/raycast/"),
            "idle maintenance must not cast: {body}"
        );
        assert!(body.contains("matches 50.."));
        assert!(body.contains("run function trail:recover"));
        assert!(body.contains("armor.feet"));
        assert!(body.contains("trail_striders"));
        let elapsed = format!("if score @s {} matches 0", Traversal::regen.objective());
        assert_eq!(
            body.matches(&elapsed).count(),
            1,
            "one timer decision: {body}"
        );
        let below_cap = body
            .lines()
            .filter(|line| line.contains("matches ..99"))
            .collect::<Vec<_>>();
        assert_eq!(below_cap.len(), 1, "one resource decision: {body}");
        let helper = below_cap[0].split_once(" run function trail:").unwrap().1;
        let restore = &functions[helper];
        assert!(restore.contains(&format!(
            "scoreboard players add @s {} 10",
            Traversal::stamina.objective()
        )));
        assert!(restore.contains("matches 101.."));
        assert!(!restore.contains("matches ..99"));
        assert!(!restore.contains(&elapsed));
        assert!(body.contains(&format!(
            "scoreboard players set @s {} 40",
            Traversal::regen.objective()
        )));
    }

    #[test]
    fn grapple_execute_pays_stamina_and_plays_vfx() {
        let cmds = emitted(grapple_execute());
        assert!(
            cmds.iter()
                .any(|line| line == "effect give @s minecraft:speed 3 2 true")
        );
        assert!(
            cmds.iter()
                .any(|line| line == "effect give @s minecraft:slow_falling 4 0 true")
        );
        let functions = exported_functions();
        let body = reachable_body(&functions, "grapple/execute");
        assert!(
            body.lines().any(|line| line
                == format!(
                    "scoreboard players remove @s {} 30",
                    Traversal::stamina.objective()
                )),
            "pays stamina through a reachable helper: {body}"
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

    #[test]
    fn crafted_core_retains_the_identity_required_by_the_upgrade() {
        use sand::registration::{ComponentContent, DatapackComponent};
        let ComponentContent::Json(recipe) = grapple_core_recipe().try_content().unwrap() else {
            panic!("recipe must emit JSON");
        };
        assert_eq!(recipe["result"]["id"], "minecraft:heart_of_the_sea");
        assert_eq!(recipe["result"]["count"], 1);
        assert_eq!(
            recipe["result"]["components"]["minecraft:custom_data"]["grapple_core"],
            true
        );
        assert!(
            recipe["result"]["components"]
                .get("minecraft:custom_name")
                .is_some()
        );
        assert_eq!(
            recipe["result"]["components"]["minecraft:max_stack_size"],
            1
        );
    }

    #[test]
    fn upgrade_exports_a_single_subject_cost_and_typed_reward() {
        let functions = exported_functions();
        let cost = functions
            .values()
            .flat_map(|body| body.lines())
            .filter(|line| {
                line.contains("clear @s ") && line.contains("grapple_core") && line.ends_with(" 1")
            })
            .collect::<Vec<_>>();
        assert_eq!(cost.len(), 1, "one bounded custom-item cost: {cost:?}");
        assert!(cost[0].ends_with(" 1"), "{cost:?}");
        assert!(functions.values().any(|body| {
            body.lines().any(|line| {
                line.starts_with("give @s minecraft:leather_boots[")
                    && line.contains("trail_striders")
            })
        }));
        assert!(
            functions
                .values()
                .any(|body| body.contains("Craft a Grapple Core before claiming"))
        );
        assert!(
            functions
                .values()
                .any(|body| body.contains("You already unlocked Trail Striders"))
        );
    }

    #[test]
    fn menu_ability_never_fans_out_to_other_players() {
        let functions = exported_functions();
        let body = reachable_body(&functions, "grapple");
        assert!(
            !body.contains("@a"),
            "one menu click must only affect its player: {body}"
        );
        assert!(body.contains("run function trail:grapple/execute"));
        assert!(body.contains("armor.feet"));
        assert!(body.contains("trail_striders"));
        assert!(body.contains("matches 30.."));
    }
    #[test]
    fn anchor_paths_gate_costs_and_keep_work_bounded() {
        let functions = exported_functions();
        for (name, block, cost, duration, samples) in [
            ("grapple", "gold_block", 30, 80, 32),
            ("scout", "iron_block", 10, 20, 16),
        ] {
            let entry = &functions[name];
            assert_eq!(
                entry.matches(&format!("matches {cost}..")).count(),
                1,
                "{entry}"
            );
            assert!(!entry.contains("scoreboard players remove"));
            assert!(!entry.contains("effect give"));
            for (objective, range) in [
                (Traversal::has_striders.objective(), "1"),
                (Traversal::grapple.objective(), "0"),
                (Traversal::exhausted.objective(), "0"),
            ] {
                assert_eq!(
                    entry
                        .matches(&format!("if score @s {objective} matches {range}"))
                        .count(),
                    1,
                    "{entry}"
                );
            }
            let gate = reachable_body(&functions, name);
            assert!(
                gate.contains("if items entity @s armor.feet minecraft:leather_boots"),
                "{gate}"
            );
            let hit = reachable_body(&functions, &format!("{name}/hit"));
            assert_eq!(
                hit.matches(&format!("if block ~ ~ ~ minecraft:{block}"))
                    .count(),
                1,
                "{hit}"
            );
            assert!(hit.contains(&format!("run function trail:{name}/execute")));
            assert!(!functions[&format!("{name}/hit")].contains("scoreboard players remove"));
            let apply = reachable_body(&functions, &format!("{name}/execute"));
            assert!(apply.contains(&format!(
                "scoreboard players remove @s {} {cost}",
                Traversal::stamina.objective()
            )));
            assert!(apply.contains(&format!(
                "scoreboard players set @s {} {duration}",
                Traversal::grapple.objective()
            )));
            let ray = reachable_body(&functions, &format!("__sand_system/raycast/{name}/start"));
            assert!(ray.contains("anchored eyes"));
            assert!(!ray.contains("@a"));
            assert_eq!(
                if name == "grapple" {
                    grapple_ray()
                } else {
                    scout_ray()
                }
                .sample_count()
                .unwrap(),
                samples
            );
            let sample = &functions[&format!("__sand_system/raycast/{name}/sample")];
            assert!(sample.contains(&format!("execute unless loaded ~ ~ ~ run return run function trail:__sand_system/raycast/{name}/miss")));
            assert!(sample.contains(&format!("execute unless predicate trail:__sand_system/raycast/{name}/criterion run return run function trail:__sand_system/raycast/{name}/hit")));
            assert!(!sample.contains("scoreboard players set"));
            let miss = &functions[&format!("__sand_system/raycast/{name}/miss")];
            assert!(!miss.contains(&Traversal::stamina.objective().to_string()));
            assert!(!miss.contains(&Traversal::grapple.objective().to_string()));
            assert!(!miss.contains("/execute"));
        }
    }

    #[test]
    fn boots_identity_and_repeated_exports_are_stable() {
        let predicate = serde_json::to_value(trail_striders().item_predicate().unwrap()).unwrap();
        assert_eq!(
            predicate["items"],
            serde_json::json!(["minecraft:leather_boots"])
        );
        assert!(predicate.to_string().contains("trail_striders"));
        let before = sand::advanced::try_export_pack_json("trail", "26.2").unwrap();
        let _other = sand::advanced::try_export_pack_json("alternate", "26.2").unwrap();
        assert_eq!(
            before,
            sand::advanced::try_export_pack_json("trail", "26.2").unwrap()
        );
    }
}
