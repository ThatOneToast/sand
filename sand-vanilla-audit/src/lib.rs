//! Small deterministic datapack used only by vanilla load/reload validation.

use sand_core::event::vanilla::{PlayerStartsSneaking, PlayerStopsSneaking};
use sand_core::events::{EventSetup, PlayerSneakEvent, SandEvent, SandEventDispatch};
use sand_core::prelude::*;
use sand_core::sand_state;
use sand_core::{FloatRange, IntRange, NumberProvider};
use sand_macros::{component, event, function};

sand_state! {
    static AUDIT_SCORE: ScoreVar<i32> = ScoreVar::new("sand_audit_score") =>
        AUDIT_SCORE.lifecycle().default(7);
    static AUDIT_FLAG: Flag = Flag::new("sand_audit_flag") =>
        AUDIT_FLAG.lifecycle().default(0);
    static AUDIT_TIMER: Timer = Timer::new("sand_audit_timer", Ticks::seconds(1)) =>
        AUDIT_TIMER.lifecycle().default(0).auto_tick();
    static AUDIT_COOLDOWN: Cooldown = Cooldown::new("sand_audit_cd", Ticks::seconds(1)) =>
        AUDIT_COOLDOWN.lifecycle().default(0).auto_tick();
}

#[function]
pub fn audit_command() {
    cmd::tellraw(
        Selector::all_players(),
        Text::new("Sand audit loaded").green(),
    );
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

static SEMANTIC_OCCURRENCE: ScoreVar<i32> = ScoreVar::new("sand_sem_occ");
static SEMANTIC_OBSERVED: ScoreVar<i32> = ScoreVar::new("sand_sem_prev");

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
        .rewards(AdvancementRewards::new().function("sand_audit:audit_command"))
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
                Some(ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")),
                None,
                None,
            )),
        )
        .rewards(AdvancementRewards::new().function("sand_audit:audit_command"))
}

/// Same coverage as [`audit_placed_block_filtered`] for `item_used_on_block`.
#[component]
pub fn audit_item_used_on_block_filtered() -> Advancement {
    Advancement::new("sand_audit:item_used_on_block_filtered".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::ItemUsedOnBlock {
                item: Some(ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")),
                location: None,
            }),
        )
        .rewards(AdvancementRewards::new().function("sand_audit:audit_command"))
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
                Some(ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")),
                None,
                None,
            )),
        )
        .rewards(AdvancementRewards::new().function("sand_audit:semantic_placed_reward"))
}

/// Client-driven item-use fixture with the same revoke/re-fire contract.
#[component]
pub fn semantic_item_used_on_block() -> Advancement {
    Advancement::new("sand_audit:semantic_item_used_on_block".parse().unwrap())
        .criterion(
            "event",
            Criterion::new(AdvancementTrigger::ItemUsedOnBlock {
                item: Some(
                    ItemPredicate::id("minecraft:honeycomb").custom_data_key("sand_audit_item"),
                ),
                location: Some(
                    LocationPredicate::new()
                        .block(BlockPredicate::new().blocks(vec!["minecraft:copper_block".into()])),
                ),
            }),
        )
        .rewards(AdvancementRewards::new().function("sand_audit:semantic_item_used_reward"))
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
                    EntityPredicate::type_("minecraft:zombie").location(
                        LocationPredicate::new()
                            .biome("minecraft:plains")
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
                        .biome("minecraft:plains")
                        .y(FloatRange::at_least(0.0)),
                ),
            }),
        )
        .criterion(
            "slept_location",
            Criterion::new(AdvancementTrigger::SleptInBed {
                location: Some(LocationPredicate::new().biome("minecraft:plains")),
            }),
        )
        .criterion(
            "hero_location",
            Criterion::new(AdvancementTrigger::HeroOfTheVillage {
                location: Some(LocationPredicate::new().biome("minecraft:plains")),
            }),
        )
        .criterion(
            "damage",
            Criterion::new(AdvancementTrigger::PlayerHurtEntity {
                entity: None,
                damage: Some(
                    DamagePredicate::new().type_(
                        DamageSourcePredicate::new()
                            .direct_entity(EntityPredicate::type_("minecraft:arrow")),
                    ),
                ),
            }),
        )
        .criterion(
            "item",
            Criterion::new(AdvancementTrigger::ConsumeItem {
                item: Some(ItemPredicate::id("minecraft:apple").custom_data_key("sand_audit")),
            }),
        )
        .criterion(
            "ender_eye",
            Criterion::new(AdvancementTrigger::UsedEnderEye { distance: None }),
        )
        .criterion(
            "allay",
            Criterion::new(AdvancementTrigger::AllayDropItemOnBlock {
                item: Some(ItemPredicate::id("minecraft:cake")),
                location: Some(
                    LocationPredicate::new()
                        .block(BlockPredicate::new().blocks(vec!["minecraft:note_block".into()])),
                ),
            }),
        )
        .criterion(
            "killed_by_arrow",
            Criterion::new(AdvancementTrigger::KilledByArrow {
                unique_entity_types: Some(IntRange::at_least(2)),
                fired_from_weapon: Some(ItemPredicate::id("minecraft:crossbow")),
                victims: Some(vec![EntityPredicate::type_("minecraft:phantom")]),
            }),
        )
        .criterion(
            "recipe_crafted",
            Criterion::new(AdvancementTrigger::RecipeCrafted {
                recipe_id: "minecraft:decorated_pot".into(),
                ingredients: vec![ItemPredicate::id("minecraft:brick")],
            }),
        )
        .criterion(
            "pickup_by_entity",
            Criterion::new(AdvancementTrigger::ThrownItemPickedUpByEntity {
                item: Some(ItemPredicate::id("minecraft:cookie")),
                entity: Some(EntityPredicate::type_("minecraft:allay")),
            }),
        )
        .criterion(
            "pickup_by_player",
            Criterion::new(AdvancementTrigger::ThrownItemPickedUpByPlayer {
                item: Some(ItemPredicate::id("minecraft:cookie")),
                entity: Some(EntityPredicate::type_("minecraft:allay")),
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
        LootCondition::RandomChance { chance: 0.5 },
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
