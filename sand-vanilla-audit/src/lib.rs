//! Small deterministic datapack used only by vanilla load/reload validation.

use sand::vanilla;
use sand_core::advanced::state::ScoreVar;
use sand_core::event::vanilla::{OnDeath, OnRespawn, PlayerStartsSneaking, PlayerStopsSneaking};
use sand_core::events::{EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch, TickWindow};
use sand_core::prelude::*;
use sand_core::{FloatRange, IntRange, NumberProvider};
use sand_macros::{State, datapack_component, function, on_event};

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
    cmd::tellraw(Target::players(), Text::new("Sand audit loaded").green());
}

/// Parser-only coverage for the validated display/media command families.
///
/// Vanilla loads and parses this function during startup and `/reload`; the
/// harness does not invoke it, so it does not claim client-visible or audible
/// confirmation.
#[function]
pub fn audit_command_media() {
    Bossbar::add(
        BossbarId::parse("sand_audit:guardian").unwrap(),
        Text::new("Sand Guardian").dark_red().bold(true),
    );
    Bossbar::set_color(
        BossbarId::parse("sand_audit:guardian").unwrap(),
        BossbarColor::Red,
    );
    Title::of(Target::players())
        .title(Text::new("Sand audit").gold())
        .subtitle(Text::new("Validated command media").yellow())
        .times(5, 20, 5)
        .build();
    Actionbar::show(
        Target::players(),
        Text::new("Command media parser audit").green(),
    );
    ParticleBuilder::new(Particle::named(ParticleId::minecraft("flame").unwrap()))
        .try_points_at(&[[0.0, 1.0, 0.0]])
        .unwrap();
    Sound::play(SoundEventId::minecraft("block.note_block.pling").unwrap())
        .source(SoundSource::Master)
        .to(Target::players())
        .volume(1.0)
        .pitch(1.0)
        .build();
    cmd::effect_give(Target::players(), EffectId::Speed)
        .seconds(5)
        .particles(false);
}

/// Parser coverage for owned inventory actions on player and container slots.
/// The load/reload harness parses this function without invoking it.
#[function]
pub fn audit_inventory_actions() {
    Target::players().each(|player| {
        let inventory = player.inventory();
        let reward = ItemStack::new(item_id("diamond"))
            .count(3)
            .component(ItemComponent::custom_data_marker("audit_reward"));
        mcfunction![
            inventory.offhand().set(reward).unwrap();
            inventory.hotbar(3).unwrap().clear().unwrap();
            when(inventory.mainhand().is_empty().unwrap()).then(
                inventory.mainhand().replace_from(&inventory.offhand()).unwrap()
            );
            ItemLocation::block(BlockPos::here()).slot(0).unwrap()
                .replace_from(&inventory.offhand()).unwrap();
        ]
    });
}

/// Parser coverage for typed predicates and their compiler-owned item tags.
/// The reload harness parses these helpers; it does not simulate player inventory.
#[function]
pub fn audit_inventory_predicates() {
    Target::players().each(|player| {
        let predicate = ItemPredicate::id(item_id("diamond"))
            .item(item_id("emerald"))
            .count_min(2)
            .custom_data_key("audit_reward");
        let inventory = player.inventory();
        mcfunction![
            player.inventory().remove(item_id("diamond"), 3);
            when(inventory.offhand().matches(&predicate).unwrap())
                .then_all(["say typed slot predicate"]);
            when(inventory.contains(&predicate))
                .then_all(["say typed inventory predicate"]);
            when(inventory.count(predicate).gte(3))
                .then_all(["say typed inventory count"]);
        ]
    });
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
#[on_event]
pub fn semantic_death_lifecycle(_event: OnDeath) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_DEATH__"}"#)
}

/// First subscriber proving every handler receives the same respawn lifecycle.
#[on_event]
pub fn semantic_respawn_lifecycle_a(_event: OnRespawn) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_RESPAWN_A__"}"#)
}

/// Second subscriber proving fan-out completes before lifecycle reset.
#[on_event]
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

#[on_event]
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

#[on_event]
pub fn semantic_after_any(_event: SemanticAfterAny) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_AFTER_ANY__"}"#)
}

#[on_event]
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

#[on_event]
pub fn semantic_within(_event: SemanticWithin) {
    cmd::raw(r#"tellraw @s {"text":"__SAND_SEMANTIC_WITHIN__"}"#)
}

/// Phase 6 (#240) advancement-backed graph parent: a provider-only
/// `SandEvent` (no direct `#[on_event]` handler of its own) whose dispatch is
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

#[on_event]
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

#[on_event]
pub fn starts_sneaking(event: sand_core::event::Event<PlayerStartsSneaking>) {
    let _ = event;
    cmd::raw("say audit started sneaking")
}

#[on_event]
pub fn stops_sneaking(event: sand_core::event::Event<PlayerStopsSneaking>) {
    let _ = event;
    cmd::raw("say audit stopped sneaking")
}

// The world audit drives these shared samplers as controlled tagged entities.
// The separate player-observation audit verifies native health/effect changes
// through normal tick dispatch with two connected players.
#[on_event]
pub fn audit_health_changed(_: sand_core::events::PlayerHealthChangedEvent) {
    cmd::raw("scoreboard players add @s sand_obs_change 1")
}

#[on_event]
pub fn audit_health_lost(_: sand_core::events::PlayerHealthLostEvent) {
    cmd::raw("scoreboard players add @s sand_obs_lost 1")
}

#[on_event]
pub fn audit_health_gained(_: sand_core::events::PlayerHealthGainedEvent) {
    cmd::raw("scoreboard players add @s sand_obs_gain 1")
}

#[on_event]
pub fn audit_health_low(_: sand_core::event::Event<sand_core::events::PlayerLowHealthEvent<6>>) {
    cmd::raw("scoreboard players add @s sand_obs_low 1")
}

#[on_event]
pub fn audit_health_recovered(
    _: sand_core::event::Event<sand_core::events::PlayerRecoveredHealthEvent<6>>,
) {
    cmd::raw("scoreboard players add @s sand_obs_back 1")
}

pub struct AuditNativeHealthChanged;
impl SandEvent for AuditNativeHealthChanged {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health()))
            .changed()
    }
}
#[on_event]
pub fn audit_native_health_changed(_: AuditNativeHealthChanged) {
    cmd::raw("scoreboard players add @s sand_obs_native 1")
}

pub struct AuditNativeHealthLost;
impl SandEvent for AuditNativeHealthLost {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| StatCurve::from(player.living().health()))
            .decreased()
    }
}
#[on_event]
pub fn audit_native_health_lost(_: AuditNativeHealthLost) {
    cmd::raw("scoreboard players add @s sand_obs_nlost 1")
}

pub struct AuditSpeedStarted;
impl SandEvent for AuditSpeedStarted {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| player.living().has_effect(EffectId::Speed))
            .became_true()
    }
}
#[on_event]
pub fn audit_speed_started(_: AuditSpeedStarted) {
    cmd::raw("scoreboard players add @s sand_obs_speed 1")
}

pub struct AuditSpeedStopped;
impl SandEvent for AuditSpeedStopped {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|player| player.living().has_effect(EffectId::Speed))
            .became_false()
    }
}
#[on_event]
pub fn audit_speed_stopped(_: AuditSpeedStopped) {
    cmd::raw("scoreboard players add @s sand_obs_nspeed 1")
}

pub struct AuditStandingOnGold;
impl SandEvent for AuditStandingOnGold {
    fn dispatch() -> impl Into<SandEventDispatch> {
        SandEventDispatch::observe_players(|_player| {
            WorldContext::current()
                .block(BlockPos::below(1))
                .matches(vanilla::Block::GoldBlock)
        })
        .became_true()
    }
}
#[on_event]
pub fn audit_standing_on_gold(_: AuditStandingOnGold) {
    cmd::raw("scoreboard players add @s sand_obs_gold 1")
}

#[datapack_component]
pub fn audit_advancement() -> Advancement {
    Advancement::new("sand_audit:first_tick".parse().unwrap())
        .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
        .rewards(AdvancementRewards::new().function(audit_command))
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
#[datapack_component]
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
        .rewards(AdvancementRewards::new().function(audit_command))
}

/// Same coverage as [`audit_placed_block_filtered`] for `item_used_on_block`.
#[datapack_component]
pub fn audit_item_used_on_block_filtered() -> Advancement {
    Advancement::new("sand_audit:item_used_on_block_filtered".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::ItemUsedOnBlock {
                item: Some(ItemPredicate::id(item_id("white_wool")).custom_data_key("elevator")),
                location: None,
            }),
        )
        .rewards(AdvancementRewards::new().function(audit_command))
}

/// Client-driven semantic fixture. The reward revokes this advancement so a
/// second matching placement proves reset/re-fire behavior in the same run.
#[datapack_component]
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
        .rewards(AdvancementRewards::new().function(semantic_placed_reward))
}

/// Client-driven item-use fixture with the same revoke/re-fire contract.
#[datapack_component]
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
        .rewards(AdvancementRewards::new().function(semantic_item_used_reward))
}

/// Cross-family parse fixture for direct entity, entity-nested location,
/// direct location, nested damage-source entity, and non-placement item
/// predicate consumers. Semantic matching remains a separate client-driven
/// evidence tier.
#[datapack_component]
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

/// Vanilla codec coverage for typed, required instrument descriptions.
#[datapack_component]
pub fn audit_instrument() -> sand_core::Instrument {
    sand_core::Instrument::new("sand_audit:horn".parse().unwrap())
        .sound_event("minecraft:item.goat_horn.sound.0")
        .description(TextComponent::translate("instrument.sand_audit.horn"))
}

/// Vanilla codec coverage for song duration, description and comparator zero.
#[datapack_component]
pub fn audit_jukebox_song() -> sand_core::JukeboxSong {
    sand_core::JukeboxSong::new("sand_audit:song".parse().unwrap())
        .sound_event("minecraft:music_disc.13")
        .song_length(178.0)
        .comparator_output(0)
        .description(TextComponent::literal("Sand audit song"))
}

/// Typed interaction resources used by the vanilla text decode/encode audit.
#[datapack_component]
pub fn audit_typed_text_interactions() -> sand::registration::DatapackRegistration {
    let mut registration = sand::registration::DatapackRegistration::new();
    let uuid = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000").unwrap();
    for (name, description) in [
        (
            "empty_suggest",
            Text::new("Clear suggestion").click_suggest_command(""),
        ),
        (
            "empty_run",
            Text::new("Empty command").click_run_command(""),
        ),
        (
            "shorthand",
            Text::new("Default identifiers")
                .font("default")
                .hover_item("stone"),
        ),
        (
            "run_command",
            Text::new("Run").click_run_command("/say hello"),
        ),
        (
            "suggest_command",
            Text::new("Suggest").click_suggest_command("/tell @s "),
        ),
        (
            "open_url",
            Text::new("Visit").click_open_url("https://example.com"),
        ),
        ("copy", Text::new("Copy").click_copy("copy")),
        ("page", Text::new("Page").click_change_page(2)),
        (
            "text",
            Text::new("Hover").hover_text(Text::new("hint").bold(true)),
        ),
        (
            "item",
            Text::new("Item").hover_item_with_count("minecraft:stone", 3),
        ),
        (
            "entity",
            Text::new("Entity").hover_entity(entity_type_id("zombie"), uuid, Text::new("Undead")),
        ),
    ] {
        registration = registration.component(
            sand_core::Instrument::new(format!("sand_audit:typed_text/{name}").parse().unwrap())
                .sound_event("minecraft:item.goat_horn.sound.0")
                .description(description),
        );
    }
    registration
}

/// Parser coverage for the explicit raw-text escape hatch and current interaction codecs.
/// JSON is intentional here: this fixture checks raw inputs independently of typed rendering.
#[datapack_component]
pub fn audit_raw_text_interactions() -> sand::registration::DatapackRegistration {
    use serde_json::json;
    let mut registration = sand::registration::DatapackRegistration::new();
    for (name, field, event) in [
        (
            "run_command",
            "click_event",
            json!({"action": "run_command", "command": "/say hello"}),
        ),
        (
            "suggest_command",
            "click_event",
            json!({"action": "suggest_command", "command": "/tell @s "}),
        ),
        (
            "open_url",
            "click_event",
            json!({"action": "open_url", "url": "https://example.com"}),
        ),
        (
            "copy",
            "click_event",
            json!({"action": "copy_to_clipboard", "value": "copy"}),
        ),
        (
            "page",
            "click_event",
            json!({"action": "change_page", "page": 1}),
        ),
        (
            "text",
            "hover_event",
            json!({"action": "show_text", "value": {"text": "hint", "bold": true}}),
        ),
        (
            "item",
            "hover_event",
            json!({"action": "show_item", "id": "minecraft:stone", "count": 99}),
        ),
        (
            "entity",
            "hover_event",
            json!({"action": "show_entity", "id": "minecraft:zombie", "uuid": "123e4567-e89b-12d3-a456-426614174000", "name": {"text": "zombie"}}),
        ),
    ] {
        let mut description = json!({"text": "Raw interaction audit", "insertion": "audit"});
        description[field] = event;
        registration = registration.component(
            sand_core::Instrument::new(format!("sand_audit:raw_text/{name}").parse().unwrap())
                .sound_event("minecraft:item.goat_horn.sound.0")
                .raw_description(sand::component::RawJson::new(description)),
        );
    }
    for (name, description) in [
        (
            "shadow_signed",
            json!({"text": "Shadow", "shadow_color": -1}),
        ),
        (
            "shadow_unsigned",
            json!({"text": "Shadow", "shadow_color": 4294967295_u64}),
        ),
        (
            "shadow_rgba",
            json!({"text": "Shadow", "shadow_color": [1, 0.5, 0, 1]}),
        ),
        (
            "shadow_wide",
            json!({"text": "Shadow", "shadow_color": [2, -1, 0, 1]}),
        ),
        ("nbt_named", json!({"nbt": "CustomName", "entity": "Steve"})),
        ("nbt_match", json!({"nbt":"foo[{a:1}].bar", "entity":"@s"})),
        ("nbt_quoted", json!({"nbt":"'foo..bar'", "entity":"@s"})),
        ("nbt_empty", json!({"nbt":"", "entity":"@s"})),
        ("nbt_all_elements", json!({"nbt":"foo[]", "entity":"@s"})),
        (
            "hover_component_patch",
            json!({"text":"Patch", "hover_event":{"action":"show_item", "id":"stone", "components":{"custom_name":{"text":"Patched"}, "!minecraft:lore":{}}}}),
        ),
        (
            "nbt_uuid",
            json!({"nbt": "CustomName", "entity": "123e4567-e89b-12d3-a456-426614174000"}),
        ),
        (
            "selector_separator",
            json!({"selector": "Steve", "separator": {"text": ", "}}),
        ),
        (
            "object_atlas",
            json!({"object": "atlas", "sprite": "block/stone", "fallback": {"text": "Stone"}}),
        ),
        (
            "object_player",
            json!({"object": "player", "player": "Steve", "hat": false}),
        ),
        (
            "custom_click",
            json!({"text":"Custom","click_event":{"action":"custom","id":"sand_audit:click","payload":{"x":1}}}),
        ),
        (
            "inline_dialog",
            json!({"text":"Dialog","click_event":{"action":"show_dialog","dialog":{"type":"minecraft:notice","title":{"text":"Hello"}}}}),
        ),
        (
            "default_hover_item",
            json!({"text":"Item","hover_event":{"action":"show_item","id":"stone"}}),
        ),
        (
            "default_hover_entity",
            json!({"text":"Entity","hover_event":{"action":"show_entity","id":"zombie","uuid":[0,0,0,1]}}),
        ),
        (
            "escaped_url",
            json!({"text":"Link","click_event":{"action":"open_url","url":"HTTPS://example.com/café?q=%20#ok"}}),
        ),
        (
            "dialog_confirmation",
            json!({"text":"Dialog", "click_event":{"action":"show_dialog", "dialog":json!({"type":"confirmation", "title":"Confirm", "yes":{"label":"Yes"}, "no":{"label":"No"}})}}),
        ),
        (
            "dialog_multi_action",
            json!({"text":"Dialog", "click_event":{"action":"show_dialog", "dialog":json!({"type":"multi_action", "title":"Menu", "actions":[{"label":"Go", "tooltip":"Hint", "width":1024, "action":{"type":"run_command", "command":"/say hi"}}], "columns":1})}}),
        ),
        (
            "dialog_server_links",
            json!({"text":"Dialog", "click_event":{"action":"show_dialog", "dialog":json!({"type":"server_links", "title":"Links", "button_width":1, "exit_action":{"label":"Done"}})}}),
        ),
        (
            "dialog_dialog_list",
            json!({"text":"Dialog", "click_event":{"action":"show_dialog", "dialog":json!({"type":"dialog_list", "title":"Dialogs", "dialogs":[{"type":"notice", "title":"Nested"}]})}}),
        ),
        (
            "dialog_variables",
            json!({"text":"x", "click_event":{"action":"show_dialog", "dialog":{"type":"notice", "title":"Variables", "inputs":[{"key":"λ١_中文", "type":"boolean", "label":"Flag"}, {"key":"", "type":"text", "label":"Empty key"}], "action":{"label":"Go", "action":{"type":"dynamic/run_command", "template":"say $(λ١_中文) $()"}}}}}),
        ),
        (
            "dialog_controls",
            json!({"text":"Dialog", "click_event":{"action":"show_dialog", "dialog":json!({"type":"notice", "title":"Info", "pause":false, "after_action":"none", "external_title":"Open", "body":[
                {"type":"plain_message", "contents":"Message", "width":1024},
                {"type":"item", "item":{"id":"stone", "count":2}, "description":{"contents":"Stone", "width":100}, "show_tooltip":false, "width":256, "height":1}
            ], "inputs":[
                {"key":"flag", "type":"boolean", "label":"Flag", "initial":true},
                {"key":"text", "type":"text", "label":"Text", "initial":"Hi", "multiline":{"height":512, "max_lines":2}},
                {"key":"choice", "type":"single_option", "label":"Choice", "options":["first", {"id":"second", "display":"Second", "initial":true}]},
                {"key":"range", "type":"number_range", "label":"Range", "start":10, "end":0, "initial":5, "step":0.5}
            ]})}}),
        ),
        (
            "primitive_arguments",
            json!({"translate":"key", "with":[7, 1.5, true, false]}),
        ),
        ("storage_shorthand", json!({"nbt":"foo", "storage":"data"})),
        ("font_shorthand", json!({"text":"Font", "font":"default"})),
        (
            "object_profile",
            json!({"player": {"name": "Steve", "id": [306070887, -392490285, -1537850778, 337068032]}, "fallback": {"text": "Player"}}),
        ),
        ("nbt_entity", json!({"nbt": "CustomName", "entity": "@s"})),
        (
            "nbt_storage",
            json!({"nbt": "message", "storage": "sand_audit:text"}),
        ),
        (
            "nbt_block",
            json!({"nbt": "CustomName", "block": "~ ~0.5 -2"}),
        ),
        (
            "nbt_local",
            json!({"nbt": "CustomName", "block": "^ ^1.5 ^"}),
        ),
        (
            "nbt_lenient",
            json!({"nbt": "CustomName", "entity": "@s", "interpret": "yes"}),
        ),
    ] {
        registration = registration.component(
            sand_core::Instrument::new(format!("sand_audit:raw_text/{name}").parse().unwrap())
                .sound_event("minecraft:item.goat_horn.sound.0")
                .raw_description(sand_core::RawJson::new(description)),
        );
    }

    registration
}

#[datapack_component]
pub fn audit_recipe() -> ShapedRecipe {
    ShapedRecipe::new("sand_audit:diamond".parse().unwrap())
        .pattern(["D"])
        .key('D', Ingredient::item("minecraft:diamond"))
        .result(RecipeResult::new("minecraft:diamond", 1))
}

#[datapack_component]
pub fn audit_predicate() -> Predicate {
    Predicate::new(
        "sand_audit:chance".parse().unwrap(),
        PredicateRoot::random_chance(0.5),
    )
}

#[datapack_component]
pub fn audit_loot_table() -> LootTable {
    LootTable::chest_loot(
        "sand_audit:chest".parse().unwrap(),
        [("minecraft:diamond", 1, 1, 1)],
    )
}

#[datapack_component]
pub fn audit_item_modifier() -> ItemModifier {
    ItemModifier::new("sand_audit:set_count".parse().unwrap()).function(LootFunction::SetCount {
        count: NumberProvider::Constant(1.0),
        add: false,
    })
}

#[cfg(sand_audit_dialogs)]
#[datapack_component]
pub fn audit_dialog() -> Dialog {
    // A `notice` dialog requires at least one button (Minecraft's own
    // dialog schema, enforced by DatapackComponent validation) -- this
    // fixture was missing one, which made every 26.x vanilla-reload
    // validation fail at datapack-export time before a server was even
    // started. Found while investigating issue #355.
    Dialog::notice_local("status")
        .title("Sand audit")
        .body(DialogBody::text("Vanilla reload validation"))
        .button(DialogButton::new("OK"))
}

/// A small, safe `sand::build`-generated dimension override for the audit
/// pack: a flat Overworld (bedrock/dirt/grass) with a spawn platform.
/// Real-server-load coverage for issue #355 -- until this, every real
/// Minecraft server load/reload validation (`.github/workflows/
/// vanilla-reload.yml`) only exercised ordinary component output, never
/// world-build-generated dimension/function/tag resources. `sand-vanilla-audit`
/// doesn't go through `sand-cli`'s `sand.build.rs` discovery (it has its own
/// bespoke `export()` below), so this calls `sand_core::build` directly and
/// merges the lowered `WorldResource`s into the versioned pack export. The
/// component records share the world-resource wire format; the runtime report
/// is reanalyzed after the merge so direct consumers see the added load work.
fn audit_world_build() -> Vec<sand_core::build::WorldResource> {
    use sand_core::build::{
        Dimension, DimensionSlot, DimensionType, Dimensions, FlatGenerator, FlatLayer, Generator,
        SandBuild, Spawn, World,
    };

    let overworld = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
        Generator::Flat(FlatGenerator::new(vec![
            FlatLayer::new(ResourceLocation::new("minecraft", "bedrock").unwrap(), 1),
            FlatLayer::new(ResourceLocation::new("minecraft", "dirt").unwrap(), 2),
            FlatLayer::new(
                ResourceLocation::new("minecraft", "grass_block").unwrap(),
                1,
            ),
        ])),
    );
    let build = SandBuild::new().world(
        World::new()
            .spawn(Spawn::at(0, 5, 0))
            .gamerule(sand_core::cmd::gamerule_keep_inventory().value(true))
            .gamerule(sand_core::cmd::gamerule_random_tick_speed().value(0))
            .dimensions(Dimensions::new().with(overworld)),
    );
    if let Err(diagnostics) = build.validate() {
        eprintln!("audit world-build validation failed:");
        for d in &diagnostics {
            eprintln!("  - {d}");
        }
        std::process::exit(1);
    }
    sand_core::build::lower_world("sand_audit", &build)
}

pub fn export(namespace: &str, version: &str) {
    let json =
        sand_core::advanced::try_export_pack_json(namespace, version).unwrap_or_else(|error| {
            eprintln!("audit export failed: {error}");
            std::process::exit(1);
        });

    let json = merge_world_export(&json).unwrap_or_else(|error| {
        eprintln!("audit export failed: could not merge pack JSON: {error}");
        std::process::exit(1);
    });
    println!("{json}");
}

/// Add world resources and recompute costs while preserving compiler provenance.
fn merge_world_export(json: &str) -> Result<String, sand_core::serde_json::Error> {
    let mut export: sand_core::serde_json::Value = sand_core::serde_json::from_str(json)?;
    let records = export["components"]
        .as_array_mut()
        .expect("the compiler pack export contains a component array");
    merge_world_resources(records, audit_world_build());
    let resources = records
        .iter()
        .filter(|record| {
            record["content_type"] == "text"
                || (record["content_type"] == "copy"
                    && (record["dir"] == "function" || record["dir"] == "tags/function"))
        })
        .map(|record| {
            let text = |key: &str| record[key].as_str().expect("compiler record field is text");
            let bytes = if text("content_type") == "copy" {
                std::fs::read(text("content")).map_err(sand_core::serde_json::Error::io)?
            } else {
                text("content").as_bytes().to_vec()
            };
            Ok((
                format!(
                    "data/{}/{}/{}.{}",
                    text("namespace"),
                    text("dir"),
                    text("path"),
                    text("ext")
                ),
                bytes,
            ))
        })
        .collect::<Result<_, sand_core::serde_json::Error>>()?;
    let report: sand_core::advanced::compiler::RuntimeReport =
        sand_core::serde_json::from_value(export["runtime_report"].clone())?;
    export["runtime_report"] = sand_core::serde_json::to_value(report.reanalyze(&resources))?;
    sand_core::serde_json::to_string(&export)
}

fn merge_world_resources(
    records: &mut Vec<sand_core::serde_json::Value>,
    resources: Vec<sand_core::build::WorldResource>,
) {
    for resource in resources {
        if resource.namespace == "minecraft"
            && resource.dir == "tags/function"
            && resource.path == "load"
        {
            let incoming: sand_core::serde_json::Value =
                sand_core::serde_json::from_str(&resource.content)
                    .expect("generated load tag is valid JSON");
            let incoming_values = incoming["values"]
                .as_array()
                .expect("generated load tag has values");
            if let Some(existing) = records.iter_mut().find(|record| {
                record["namespace"] == "minecraft"
                    && record["dir"] == "tags/function"
                    && record["path"] == "load"
                    && record["ext"] == "json"
            }) {
                let mut content: sand_core::serde_json::Value = sand_core::serde_json::from_str(
                    existing["content"].as_str().expect("tag content is text"),
                )
                .expect("ordinary load tag is valid JSON");
                let values = content["values"]
                    .as_array_mut()
                    .expect("ordinary load tag has values");
                for value in incoming_values {
                    if !values.contains(value) {
                        values.push(value.clone());
                    }
                }
                existing["content"] = sand_core::serde_json::Value::String(
                    sand_core::serde_json::to_string_pretty(&content)
                        .expect("merged load tag is serializable"),
                );
                continue;
            }
        }
        records.push(
            sand_core::serde_json::to_value(resource)
                .expect("WorldResource is always serializable"),
        );
    }
}

#[cfg(test)]
mod world_build_audit_tests {
    use super::*;

    /// The custom audit exporter must preserve the versioned build protocol.
    #[test]
    fn world_merge_preserves_the_export_envelope_and_runtime_report() {
        let input: sand_core::serde_json::Value = sand_core::serde_json::from_str(
            &sand_core::advanced::try_export_pack_json("sand_audit", "26.2").unwrap(),
        )
        .unwrap();
        let merged = merge_world_export(&input.to_string()).unwrap();
        let output: sand_core::serde_json::Value =
            sand_core::serde_json::from_str(&merged).unwrap();
        assert_eq!(output["schema_version"], input["schema_version"]);
        assert_eq!(
            output["runtime_report"]["provenance"],
            input["runtime_report"]["provenance"]
        );
        let world_init = "sand_audit:__sand_world_init";
        assert!(
            input["runtime_report"]["functions"]
                .get(world_init)
                .is_none()
        );
        let cost = &output["runtime_report"]["functions"][world_init]["evaluations"];
        assert!(cost["minimum"].as_u64().unwrap() > 0);
        assert_eq!(cost["minimum"], cost["maximum"]);
        let load = output["runtime_report"]["load"].as_array().unwrap();
        let entry = load
            .iter()
            .find(|entry| entry["reference"] == world_init)
            .unwrap();
        assert_eq!(&entry["evaluations"], cost);
        assert!(!output["components"].as_array().unwrap().is_empty());
    }

    /// Copy-backed runtime resources must be resolved before replacing the report.
    #[test]
    fn world_merge_reports_copied_functions_and_rejects_missing_sources() {
        let path = std::env::temp_dir().join(format!(
            "sand-audit-report-{}.mcfunction",
            std::process::id()
        ));
        std::fs::write(&path, "say copied\nsay twice\n").unwrap();
        let mut input: sand_core::serde_json::Value = sand_core::serde_json::from_str(
            &sand_core::advanced::try_export_pack_json("sand_audit", "26.2").unwrap(),
        )
        .unwrap();
        input["components"]
            .as_array_mut()
            .unwrap()
            .push(sand_core::serde_json::json!({
                "namespace": "sand_audit", "dir": "function", "path": "copied",
                "ext": "mcfunction", "content_type": "copy", "content": path.to_str().unwrap()
            }));
        let output: sand_core::serde_json::Value =
            sand_core::serde_json::from_str(&merge_world_export(&input.to_string()).unwrap())
                .unwrap();
        let cost = &output["runtime_report"]["functions"]["sand_audit:copied"]["evaluations"];
        assert_eq!(cost["minimum"], 2);
        assert_eq!(cost["maximum"], 2);
        std::fs::remove_file(path).unwrap();
        assert!(merge_world_export(&input.to_string()).is_err());
    }

    #[test]
    fn audit_includes_dimension_init_function_and_merged_load_tag() {
        let mut records = vec![sand_core::serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags/function",
            "path": "load",
            "ext": "json",
            "content_type": "text",
            "content": "{\"values\":[\"sand_audit:ordinary_load\"]}"
        })];
        merge_world_resources(&mut records, audit_world_build());

        assert!(records.iter().any(|record| {
            record["namespace"] == "minecraft"
                && record["dir"] == "dimension"
                && record["path"] == "overworld"
        }));
        assert!(records.iter().any(|record| {
            record["namespace"] == "sand_audit"
                && record["dir"] == "function"
                && record["path"] == "__sand_world_init"
                && record["content"]
                    .as_str()
                    .is_some_and(|content| content.contains("setworldspawn 0 5 0 0 0"))
        }));
        let load = records
            .iter()
            .find(|record| {
                record["namespace"] == "minecraft"
                    && record["dir"] == "tags/function"
                    && record["path"] == "load"
            })
            .unwrap();
        let content: sand_core::serde_json::Value =
            sand_core::serde_json::from_str(load["content"].as_str().unwrap()).unwrap();
        assert!(content["values"].as_array().unwrap().contains(
            &sand_core::serde_json::Value::String("sand_audit:ordinary_load".into())
        ));
        assert!(content["values"].as_array().unwrap().contains(
            &sand_core::serde_json::Value::String("sand_audit:__sand_world_init".into())
        ));
    }
}

/// The server harness validates generated attribute commands; no live player is implied.
#[function]
pub fn audit_native_attribute_values() {
    Target::players().each(|player| {
        when(
            player
                .living()
                .attribute(AttributeType::MovementSpeed)
                .gt(player.living().attribute_base(AttributeType::MovementSpeed)),
        )
        .then_all(["say native attribute modifier boost"])
    });
}

/// Parse the owned entity-properties resource and effect condition helper.
#[function]
pub fn audit_living_effect_presence() {
    Target::players().each(|player| {
        when(player.living().has_effect(EffectId::Speed)).then_all(["say speed effect present"])
    });
}

/// Exercise block, container, and data actions in a disposable runtime audit world.
#[function]
pub fn audit_runtime_world_blocks() {
    let world = WorldContext::current();
    world.in_dimension(
        "minecraft:the_nether"
            .parse::<sand::registry::DimensionId>()
            .unwrap(),
        |world| {
            let chest = world.block(BlockPos::absolute(4, 80, 4));
            let slot = chest.inventory().slot(0).unwrap();
            let name = chest
                .data()
                .typed_path::<String>(NbtPath::root("CustomName"));
            mcfunction![
                chest.replace(vanilla::Block::Chest);
                when(chest.matches(vanilla::Block::Chest)).then_all(mcfunction![
                    slot.set(ItemStack::new(vanilla::Item::Diamond.into()).count(3)).unwrap();
                    name.set("Sand world audit");
                ]);
                world.block(BlockPos::absolute(5, 80, 4)).replace(vanilla::Block::GoldBlock);
                world.block(BlockPos::absolute(6, 80, 4)).destroy();
                world.block(BlockPos::here()).replace(vanilla::Block::EmeraldBlock);
            ]
        },
    );
    when(
        !world
            .block(BlockPos::absolute(1_000_000, 80, 1_000_000))
            .matches(vanilla::Block::Air),
    )
    .then_all(["say __SAND_WORLD_UNLOADED_FALSE__"]);
    world.block(BlockPos::here()).replace(vanilla::Block::Stone);
}

/// Resolve fractional relative and local block offsets through the world API.
#[function]
pub fn audit_world_block_offsets() {
    let world = WorldContext::current();
    world
        .block(BlockPos::new(Coord::rel_n(0.5), Coord::rel(), Coord::rel()))
        .replace(vanilla::Block::Stone);
    world
        .block(BlockPos::new(
            Coord::local(),
            Coord::local(),
            Coord::local_n(1.25),
        ))
        .replace(vanilla::Block::GoldBlock);
    let relative_chest = world.block(BlockPos::new(Coord::rel_n(1.5), Coord::rel(), Coord::rel()));
    let relative_name = relative_chest
        .data()
        .typed_path::<String>(NbtPath::root("CustomName"));
    relative_chest.replace(vanilla::Block::Chest);
    relative_name.set("Relative offset");
    let local_chest = world.block(BlockPos::new(
        Coord::local(),
        Coord::local(),
        Coord::local_n(2.25),
    ));
    let local_name = local_chest
        .data()
        .typed_path::<String>(NbtPath::root("CustomName"));
    local_chest.replace(vanilla::Block::Chest);
    local_name.set("Local offset");
}

/// Keep function-macro arguments available across dimension-scoped actions.
#[function]
pub fn audit_world_macro() {
    let args = FunctionMacroArgs::new(["block"]).unwrap();
    WorldContext::current().in_dimension(
        "minecraft:the_nether"
            .parse::<sand::registry::DimensionId>()
            .unwrap(),
        |_| {
            format!(
                "{}\n# dimension macro audit\n\nsetblock ~1 ~ ~ minecraft:lapis_block",
                args.line("setblock ~ ~ ~ $(block)").unwrap()
            )
        },
    );
}

/// Select only current-dimension entities and bind each entity's position.
#[function]
pub fn audit_world_targets() {
    WorldContext::current().in_dimension(
        "minecraft:the_nether"
            .parse::<sand::registry::DimensionId>()
            .unwrap(),
        |world| {
            world
                .entities()
                .tag("sand_world_probe")
                .within_blocks(32.0)
                .each(|entity| {
                    let visited = sand::entity::EntityTag::new("sand_world_visited").unwrap();
                    mcfunction![
                        entity.add_tag(&visited);
                        world.block(BlockPos::here()).replace(vanilla::Block::GoldBlock);
                    ]
                })
        },
    );
}
