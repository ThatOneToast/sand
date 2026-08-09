//! Small deterministic datapack used only by vanilla load/reload validation.

use sand_core::event::vanilla::{OnDeath, OnRespawn, PlayerStartsSneaking, PlayerStopsSneaking};
use sand_core::events::{EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch, TickWindow};
use sand_core::prelude::*;
use sand_core::{FloatRange, IntRange, NumberProvider};
use sand_macros::{State, component, event, function};

fn item_id(path: &str) -> ItemId {
    ItemId::minecraft(path).unwrap()
}

fn block_id(path: &str) -> BlockId {
    BlockId::minecraft(path).unwrap()
}

fn entity_type_id(path: &str) -> EntityTypeId {
    EntityTypeId::minecraft(path).unwrap()
}

fn biome_id(path: &str) -> BiomeId {
    BiomeId::minecraft(path).unwrap()
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "sand_audit", scope = player)]
struct AuditState {
    #[state(default = 7)]
    score: EntityScore<i32>,
    #[state(default = false)]
    flag: EntityFlag,
    #[state(default = 0, auto_tick)]
    timer: EntityTimer,
    #[state(auto_tick)]
    cooldown: EntityCooldown,
}

#[function]
pub fn audit_command() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Sand audit loaded").green(),
    );
}

/// Parser-only coverage for the validated display/media command families.
///
/// Vanilla loads and parses this function during startup and `/reload`; the
/// harness does not invoke it, so it does not claim client-visible or audible
/// confirmation.
#[function]
pub fn audit_command_media() {
    Bossbar::add(
        ResourceLocation::new("sand_audit", "guardian").unwrap(),
        Text::new("Sand Guardian").dark_red().bold(true),
    );
    Bossbar::set_color(
        ResourceLocation::new("sand_audit", "guardian").unwrap(),
        BossbarColor::Red,
    );
    Title::of(Selector::all_players())
        .title(Text::new("Sand audit").gold())
        .subtitle(Text::new("Validated command media").yellow())
        .times(5, 20, 5)
        .build();
    Actionbar::show(
        Selector::all_players(),
        Text::new("Command media parser audit").green(),
    );
    ParticleBuilder::new(Particle::named(
        ResourceLocation::new("minecraft", "flame").unwrap(),
    ))
    .try_points_at(&[[0.0, 1.0, 0.0]])
    .unwrap();
    Sound::play(ResourceLocation::new("minecraft", "block.note_block.pling").unwrap())
        .source(SoundSource::Master)
        .to(Selector::all_players())
        .volume(1.0)
        .pitch(1.0)
        .build();
    cmd::effect_give(Selector::all_players(), EffectId::Speed)
        .seconds(5)
        .particles(false);
}

#[function]
pub fn semantic_placed_reward() {
    cmd::raw("advancement revoke @s only sand_audit:semantic_placed_block");
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_PLACED__"}"#)
}

#[function]
pub fn semantic_item_used_reward() {
    cmd::raw("advancement revoke @s only sand_audit:semantic_item_used_on_block");
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_ITEM_USED__"}"#)
}

/// Real-server evidence for the generated death observation boundary.
#[event]
pub fn semantic_death_lifecycle(_event: OnDeath) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_DEATH__"}"#)
}

/// First subscriber proving every handler receives the same respawn lifecycle.
#[event]
pub fn semantic_respawn_lifecycle_a(_event: OnRespawn) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_RESPAWN_A__"}"#)
}

/// Second subscriber proving fan-out completes before lifecycle reset.
#[event]
pub fn semantic_respawn_lifecycle_b(_event: OnRespawn) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_RESPAWN_B__"}"#)
}

static SEMANTIC_OCCURRENCE: ScoreVar<i32> = ScoreVar::new("sand_sem_occ");
static SEMANTIC_OBSERVED: ScoreVar<i32> = ScoreVar::new("sand_sem_prev");
static SEMANTIC_MULTI_A: ScoreVar<i32> = ScoreVar::new("sand_mp_a");
static SEMANTIC_MULTI_A_OBSERVED: ScoreVar<i32> = ScoreVar::new("sand_mp_ap");
static SEMANTIC_MULTI_B: ScoreVar<i32> = ScoreVar::new("sand_mp_b");
static SEMANTIC_MULTI_B_OBSERVED: ScoreVar<i32> = ScoreVar::new("sand_mp_bp");

/// Client-controlled occurrence used to prove persistent composition against a
/// real server. Increasing `sand_sem_occ` creates one parent occurrence.
pub struct SemanticOccurrence;

impl SandEvent for SemanticOccurrence {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players().when(
            SEMANTIC_OBSERVED
                .of("@s")
                .lt_score(SEMANTIC_OCCURRENCE.of("@s")),
        )
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add sand_sem_occ dummy".into(),
                "scoreboard objectives add sand_sem_prev dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s sand_sem_prev = @s sand_sem_occ"
                    .into(),
            ],
        }
    }
}

pub struct SemanticOccurrenceWhileSneaking;

impl SandEvent for SemanticOccurrenceWhileSneaking {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<SemanticOccurrence>().while_::<PlayerSneakEvent>()
    }
}

#[event]
pub fn semantic_occurrence_while_sneaking(_event: SemanticOccurrenceWhileSneaking) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_WHILE_SNEAKING__"}"#)
}

/// Independently controlled score-delta parent A for real-server
/// `after_any`/`after_all` verification.
pub struct SemanticMultiParentA;

impl SandEvent for SemanticMultiParentA {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players().when(
            SEMANTIC_MULTI_A_OBSERVED
                .of("@s")
                .lt_score(SEMANTIC_MULTI_A.of("@s")),
        )
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add sand_mp_a dummy".into(),
                "scoreboard objectives add sand_mp_ap dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s sand_mp_ap = @s sand_mp_a"
                    .into(),
            ],
        }
    }
}

/// Independently controlled score-delta parent B for real-server
/// `after_any`/`after_all` verification.
pub struct SemanticMultiParentB;

impl SandEvent for SemanticMultiParentB {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::tick().as_players().when(
            SEMANTIC_MULTI_B_OBSERVED
                .of("@s")
                .lt_score(SEMANTIC_MULTI_B.of("@s")),
        )
    }

    fn setup() -> EventSetup {
        EventSetup {
            objectives: vec![
                "scoreboard objectives add sand_mp_b dummy".into(),
                "scoreboard objectives add sand_mp_bp dummy".into(),
            ],
            pre_observation: vec![],
            post_observation: vec![
                "execute as @a run scoreboard players operation @s sand_mp_bp = @s sand_mp_b"
                    .into(),
            ],
        }
    }
}

pub struct SemanticAfterAny;

impl SandEvent for SemanticAfterAny {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_any::<(SemanticMultiParentA, SemanticMultiParentB)>()
    }
}

pub struct SemanticAfterAll;

impl SandEvent for SemanticAfterAll {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::after_all::<(SemanticMultiParentA, SemanticMultiParentB)>()
    }
}

#[event]
pub fn semantic_after_any(_event: SemanticAfterAny) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_AFTER_ANY__"}"#)
}

#[event]
pub fn semantic_after_all(_event: SemanticAfterAll) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_AFTER_ALL__"}"#)
}

/// Phase 5 (#240) bounded correlation: `SemanticOccurrence` is the current
/// trigger, `SemanticMultiParentA` is the bounded prior event. A 5-tick
/// window is small enough for deterministic real-server timing while still
/// distinguishing "recent" from "stale".
pub struct SemanticWithin;

impl SandEvent for SemanticWithin {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::compose()
            .after::<SemanticOccurrence>()
            .within::<SemanticMultiParentA>(TickWindow::new(5).expect("valid window"))
    }
}

#[event]
pub fn semantic_within(_event: SemanticWithin) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_WITHIN__"}"#)
}

/// Phase 6 (#240) advancement-backed graph parent: a provider-only
/// `SandEvent` (no direct `#[event]` handler of its own) whose dispatch is
/// advancement-backed. Sand synthesizes its advancement/entry function
/// purely to bridge `SemanticAdvancementBridgeChild` below — see
/// `EventGraph::advancement_bridges`. Reuses the same marked
/// `item_used_on_block` stimulus (honeycomb on copper block) already
/// exercised by `audit_item_used_on_block_filtered`/the Phase 1/2 semantic
/// client flow, so this fixture's structural correctness (advancement JSON
/// and entry ordering) is covered by existing load/reload validation
/// without introducing a new client interaction.
pub struct SemanticAdvancementParent;

impl SandEvent for SemanticAdvancementParent {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::AdvancementTrigger(AdvancementTrigger::ItemUsedOnBlock {
            item: Some(ItemPredicate::id(item_id("honeycomb")).custom_data_key("sand_audit_item")),
            location: None,
        })
    }
}

/// Bridged child: the sole `after::<SemanticAdvancementParent>()` occurrence
/// dependency, dispatched synchronously from the advancement reward.
pub struct SemanticAdvancementBridgeChild;

impl SandEvent for SemanticAdvancementBridgeChild {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::chain::<SemanticAdvancementParent>()
    }
}

#[event]
pub fn semantic_advancement_bridge_child(_event: SemanticAdvancementBridgeChild) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_ADVANCEMENT_BRIDGE__"}"#)
}

#[function]
pub fn semantic_multi_fire_a() {
    cmd::raw("scoreboard players add @s sand_mp_a 1")
}

#[function]
pub fn semantic_multi_fire_b() {
    cmd::raw("scoreboard players add @s sand_mp_b 1")
}

/// Atomically advances both parents in A-then-B command order. The event
/// coordinator observes both deltas in one later dispatch cycle.
#[function]
pub fn semantic_multi_fire_ab() {
    cmd::raw("scoreboard players add @s sand_mp_a 1");
    cmd::raw("scoreboard players add @s sand_mp_b 1")
}

/// The reverse atomic order proves tuple/stimulus order does not affect
/// same-cycle coalescing.
#[function]
pub fn semantic_multi_fire_ba() {
    cmd::raw("scoreboard players add @s sand_mp_b 1");
    cmd::raw("scoreboard players add @s sand_mp_a 1")
}

#[event]
pub fn starts_sneaking(event: sand_core::event::Event<PlayerStartsSneaking>) {
    let _ = event;
    cmd::raw("say audit started sneaking")
}

#[event]
pub fn stops_sneaking(event: sand_core::event::Event<PlayerStopsSneaking>) {
    let _ = event;
    cmd::raw("say audit stopped sneaking")
}

#[component]
pub fn audit_advancement() -> Advancement {
    Advancement::new("sand_audit:first_tick".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
        .rewards(
            AdvancementRewards::new().function("sand_audit:audit_command".parse().unwrap()),
        )
}

/// Real-vanilla load/reload coverage for the #231/#232 `placed_block` fix:
/// proves the `conditions.location` / `minecraft:location_check` /
/// `minecraft:match_tool` JSON this crate now generates for a block +
/// custom-data-filtered item is accepted by a real server, not merely
/// structurally correct per the golden tests in `sand-components`.
///
/// This only proves the document loads/reloads cleanly — it does not prove
/// the criterion fires only for matching placements in gameplay (that
/// requires a real client-driven positive/negative test; see
/// `docs/vanilla-reload-validation.md`).
#[component]
pub fn audit_placed_block_filtered() -> Advancement {
    Advancement::new("sand_audit:placed_block_filtered".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::placed_block(
                Some(BlockId::minecraft("white_wool").unwrap()),
                Some(ItemPredicate::id(item_id("white_wool")).custom_data_key("elevator")),
                None,
                None,
            )),
        )
        .rewards(
            AdvancementRewards::new().function("sand_audit:audit_command".parse().unwrap()),
        )
}

/// Same coverage as [`audit_placed_block_filtered`] for `item_used_on_block`.
#[component]
pub fn audit_item_used_on_block_filtered() -> Advancement {
    Advancement::new("sand_audit:item_used_on_block_filtered".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::ItemUsedOnBlock {
                item: Some(ItemPredicate::id(item_id("white_wool")).custom_data_key("elevator")),
                location: None,
            }),
        )
        .rewards(
            AdvancementRewards::new().function("sand_audit:audit_command".parse().unwrap()),
        )
}

/// Client-driven semantic fixture. The reward revokes this advancement so a
/// second matching placement proves reset/re-fire behavior in the same run.
#[component]
pub fn semantic_placed_block() -> Advancement {
    Advancement::new("sand_audit:semantic_placed_block".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::placed_block(
                Some(BlockId::minecraft("white_wool").unwrap()),
                Some(ItemPredicate::id(item_id("white_wool")).custom_data_key("elevator")),
                None,
                None,
            )),
        )
        .rewards(
            AdvancementRewards::new()
                .function("sand_audit:semantic_placed_reward".parse().unwrap()),
        )
}

/// Client-driven item-use fixture with the same revoke/re-fire contract.
#[component]
pub fn semantic_item_used_on_block() -> Advancement {
    Advancement::new("sand_audit:semantic_item_used_on_block".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::ItemUsedOnBlock {
                item: Some(
                    ItemPredicate::id(item_id("honeycomb")).custom_data_key("sand_audit_item"),
                ),
                location: Some(
                    LocationPredicate::new()
                        .block(BlockPredicate::new().blocks(vec![block_id("copper_block")])),
                ),
            }),
        )
        .rewards(
            AdvancementRewards::new()
                .function("sand_audit:semantic_item_used_reward".parse().unwrap()),
        )
}

/// Cross-family parse fixture for direct entity, entity-nested location,
/// direct location, nested damage-source entity, and non-placement item
/// predicate consumers. Semantic matching remains a separate client-driven
/// evidence tier.
#[component]
pub fn audit_profiled_trigger_matrix() -> Advancement {
    Advancement::new("sand_audit:profiled_trigger_matrix".parse().unwrap())
        .criterion(
            "entity",
            Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                entity: Some(
                    EntityPredicate::type_(entity_type_id("zombie")).location(
                        LocationPredicate::new()
                            .biome(biome_id("plains"))
                            .y(FloatRange::at_least(0.0)),
                    ),
                ),
                killing_blow: None,
            }),
        )
        .criterion(
            "location",
            Criterion::new(AdvancementTrigger::Location {
                location: Some(
                    LocationPredicate::new()
                        .biome(biome_id("plains"))
                        .y(FloatRange::at_least(0.0)),
                ),
            }),
        )
        .criterion(
            "slept_location",
            Criterion::new(AdvancementTrigger::SleptInBed {
                location: Some(LocationPredicate::new().biome(biome_id("plains"))),
            }),
        )
        .criterion(
            "hero_location",
            Criterion::new(AdvancementTrigger::HeroOfTheVillage {
                location: Some(LocationPredicate::new().biome(biome_id("plains"))),
            }),
        )
        .criterion(
            "damage",
            Criterion::new(AdvancementTrigger::PlayerHurtEntity {
                entity: None,
                damage: Some(
                    DamagePredicate::new().type_(
                        DamageSourcePredicate::new()
                            .direct_entity(EntityPredicate::type_(entity_type_id("arrow"))),
                    ),
                ),
            }),
        )
        .criterion(
            "item",
            Criterion::new(AdvancementTrigger::ConsumeItem {
                item: Some(ItemPredicate::id(item_id("apple")).custom_data_key("sand_audit")),
            }),
        )
        .criterion(
            "ender_eye",
            Criterion::new(AdvancementTrigger::UsedEnderEye { distance: None }),
        )
        .criterion(
            "allay",
            Criterion::new(AdvancementTrigger::AllayDropItemOnBlock {
                item: Some(ItemPredicate::id(item_id("cake"))),
                location: Some(
                    LocationPredicate::new()
                        .block(BlockPredicate::new().blocks(vec![block_id("note_block")])),
                ),
            }),
        )
        .criterion(
            "killed_by_arrow",
            Criterion::new(AdvancementTrigger::KilledByArrow {
                unique_entity_types: Some(IntRange::at_least(2)),
                fired_from_weapon: Some(ItemPredicate::id(item_id("crossbow"))),
                victims: Some(vec![EntityPredicate::type_(entity_type_id("phantom"))]),
            }),
        )
        .criterion(
            "recipe_crafted",
            Criterion::new(AdvancementTrigger::RecipeCrafted {
                recipe_id: "minecraft:decorated_pot".into(),
                ingredients: vec![ItemPredicate::id(item_id("brick"))],
            }),
        )
        .criterion(
            "pickup_by_entity",
            Criterion::new(AdvancementTrigger::ThrownItemPickedUpByEntity {
                item: Some(ItemPredicate::id(item_id("cookie"))),
                entity: Some(EntityPredicate::type_(entity_type_id("allay"))),
            }),
        )
        .criterion(
            "pickup_by_player",
            Criterion::new(AdvancementTrigger::ThrownItemPickedUpByPlayer {
                item: Some(ItemPredicate::id(item_id("cookie"))),
                entity: Some(EntityPredicate::type_(entity_type_id("allay"))),
            }),
        )
}

#[component]
pub fn audit_recipe() -> ShapedRecipe {
    ShapedRecipe::new("sand_audit:diamond".parse().unwrap())
        .pattern(["D"])
        .key('D', Ingredient::item("minecraft:diamond"))
        .result(RecipeResult::new("minecraft:diamond", 1))
}

#[component]
pub fn audit_predicate() -> Predicate {
    Predicate::new(
        "sand_audit:chance".parse().unwrap(),
        PredicateRoot::random_chance(0.5),
    )
}

#[component]
pub fn audit_loot_table() -> LootTable {
    LootTable::chest_loot(
        "sand_audit:chest".parse().unwrap(),
        [("minecraft:diamond", 1, 1, 1)],
    )
}

#[component]
pub fn audit_item_modifier() -> ItemModifier {
    ItemModifier::new("sand_audit:set_count".parse().unwrap()).function(LootFunction::SetCount {
        count: NumberProvider::Constant(1.0),
        add: false,
    })
}

#[cfg(sand_audit_dialogs)]
#[component]
pub fn audit_dialog() -> Dialog {
    Dialog::notice_local("status")
        .title("Sand audit")
        .body(DialogBody::text("Vanilla reload validation"))
}

pub fn export(namespace: &str, version: &str) {
    let resolved = sand_core::version::resolve_export_caps(version).unwrap_or_else(|error| {
        eprintln!("audit export failed: {error}");
        std::process::exit(1);
    });
    let json = sand_core::try_export_components_json_for_version(
        namespace,
        &resolved.caps,
        &resolved.version,
        resolved.is_fallback,
    )
    .unwrap_or_else(|error| {
        eprintln!("audit export failed: {error}");
        std::process::exit(1);
    });
    println!("{json}");
}
