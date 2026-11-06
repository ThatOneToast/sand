//! Vanilla Expedition: multiplayer exploration, crafting and combat progression.
use sand::events::{ChangeDimensionEvent, EntityKillEvent, FirstJoinEvent};
use sand::prelude::*;
use sand::registry::ItemId;

mod resources;
pub use resources::*;

fn id(path: &str) -> sand::ResourceLocation {
    sand::ResourceLocation::new("vanilla_expedition", path).unwrap()
}
fn item(path: &str) -> ItemId {
    ItemId::minecraft(path).unwrap()
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "vanilla_expedition", scope = player)]
pub struct Expedition {
    #[state(default = 0)]
    kills: EntityScore<i32>,
    #[state(default = 0)]
    journeys: EntityScore<i32>,
}

pub fn field_compass() -> CustomItem {
    CustomItem::new(item("compass"))
        .id("vanilla_expedition:field_compass")
        .custom_name(Text::new("Expedition Compass").gold())
        .lore_line(Text::new(
            "Craft supplies, cross dimensions, and earn your charter.",
        ))
        .enchantment_glint_override(true)
}

#[function]
pub fn journal() {
    cmd::tellraw(Target::self_(), Text::new("Vanilla Expedition | Craft a compass from iron + redstone. Cross into the Nether. Defeat threats to earn experience. Your expedition persists across reconnects.").gold());
}

#[on_event]
pub fn begin(event: Event<FirstJoinEvent>) {
    let _ = event;
    cmd::give(Target::self_(), field_compass());
    cmd::team_join("ve_explorers").members(Target::self_().to_string());
    cmd::function(journal);
}

#[on_event]
pub fn combat(event: Event<EntityKillEvent>) {
    event.player().state(Expedition::kills).add(1);
    sand::execute_when::when(
        event
            .player()
            .state(Expedition::kills)
            .matches(5..=5)
            .unwrap(),
    )
    .then(cmd::function(veteran_reward));
    cmd::title_actionbar(
        Target::self_(),
        Text::new("Expedition: encounter recorded").gold(),
    );
}

#[on_event]
pub fn travel(event: Event<ChangeDimensionEvent>) {
    event.player().state(Expedition::journeys).add(1);
    cmd::function(journal);
}

#[datapack_component]
pub fn compass_recipe() -> ShapelessRecipe {
    ShapelessRecipe::new(id("field_compass"))
        .ingredient(Ingredient::item_id(item("iron_ingot")))
        .ingredient(Ingredient::item_id(item("redstone")))
        .result(field_compass().recipe_result(1).unwrap())
}

#[function]
pub fn nether_reward() {
    cmd::give(Target::self_(), item("golden_apple"));
    cmd::loot_give_loot(Target::self_(), id("expedition_cache"));
    sand::vfx::Vfx::new("charter")
        .particle(
            sand::vfx::VfxParticle::happy_villager()
                .count(24)
                .spread_uniform(0.5),
        )
        .play_at(Target::self_());
    cmd::playsound_master(sand::ResourceLocation::minecraft("entity.player.levelup").unwrap())
        .targets(Target::self_());
    cmd::tellraw(
        Target::self_(),
        Text::new("Beyond the frontier! Your expedition charter is complete.").gold(),
    );
}

#[datapack_component]
pub fn frontier() -> Advancement {
    Advancement::new(id("frontier"))
        .display(
            AdvancementDisplay::new(
                AdvancementIcon::new(item("compass")),
                Text::new("Beyond the Frontier"),
                Text::new("Enter the Nether on your expedition"),
            )
            .background("minecraft:textures/block/stone.png".parse().unwrap()),
        )
        .criterion(
            "cross",
            Criterion::new(AdvancementTrigger::changed_dimension(
                None,
                Some(sand::registry::DimensionId::minecraft("the_nether").unwrap()),
            )),
        )
        .rewards(
            AdvancementRewards::new()
                .experience(100)
                .function(nether_reward),
        )
}

#[datapack_component]
pub fn expedition_cache() -> LootTable {
    LootTable::new(id("expedition_cache"))
        .loot_type(LootTableType::Chest)
        .pool(
            LootPool::new()
                .rolls(2)
                .entry(LootEntry::item(item("bread")))
                .entry(LootEntry::item(item("iron_ingot"))),
        )
}

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

#[function]
pub fn veteran_reward() {
    cmd::give(
        Target::self_(),
        CustomItem::new(item("iron_sword"))
            .custom_name(Text::new("Pathfinder Blade").gold())
            .typed_enchantment(sand::registry::EnchantmentId::custom(id("pathfinder")), 1),
    );
    veteran_scene().start();
}

#[system(tick, every = 100)]
pub fn expedition_hud(query: Expedition) {
    query.each(|_explorer| {
        sand::execute_when::when(sand::condition::Condition::predicate(
            sand::predicate::PredicateId::custom(id("in_frontier")),
        ))
        .then(cmd::title_actionbar(
            Target::self_(),
            Text::new("Frontier expedition | 5 encounters: Pathfinder Blade").gold(),
        ))
    });
}

static PACK_STATUS: sand::data::StorageVar<String> =
    sand::data::StorageVar::new("vanilla_expedition:world", "status");

#[function]
pub fn setup() {
    PACK_STATUS.set_string("Expedition season active");
    cmd::team_add("ve_explorers");
    cmd::bossbar_add(
        id("season"),
        Text::new("Vanilla Expedition • Explore together").gold(),
    );
    cmd::bossbar_set_max(id("season"), 1);
    cmd::bossbar_set_value(id("season"), 1);
    cmd::function(season_pulse);
}

#[function]
pub fn season_pulse() {
    cmd::bossbar_set_players(id("season")).targets(Target::players());
    cmd::schedule_function_replace(id("season_pulse"), "10s");
}

#[datapack_component]
pub fn setup_tag() -> sand::registration::DatapackRegistration {
    sand::registration::DatapackRegistration::new().lifecycle(
        sand::registration::LifecycleContribution::load(cmd::function(setup)),
    )
}

/// Place a campfire only into air at the executor's feet; intended for operator setup.
#[function]
pub fn establish_camp() {
    let block = sand::world::WorldContext::current().block(BlockPos::here());
    sand::execute_when::when(block.matches(sand::vanilla::Block::Air))
        .then(block.replace(sand::vanilla::Block::Campfire));
}

/// Personal cosmetic equipment inscription, available from the expedition journal.
#[function]
pub fn inscribe_equipment() {
    sand::command::Inventory::of(Target::self_()).modify(
        sand::command::ItemSlot::MainHand,
        id("chartered").to_string(),
    );
}

#[datapack_component]
pub fn veteran_scene() -> sand::systems::Scene<sand::entity::PlayerKind> {
    sand::systems::Scene::<sand::entity::PlayerKind>::new(sand::systems::Lifetime::new(
        id("veteran_scene"),
        sand::state::Ticks::seconds(5),
    ))
    .text(
        "charter",
        Text::new("Veteran Explorer").gold(),
        sand::systems::Transform::default().translated([0.0, 2.5, 0.0]),
    )
}
