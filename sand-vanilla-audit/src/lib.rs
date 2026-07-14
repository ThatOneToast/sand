//! Small deterministic datapack used only by vanilla load/reload validation.

use sand_core::NumberProvider;
use sand_core::event::vanilla::{PlayerStartsSneaking, PlayerStopsSneaking};
use sand_core::prelude::*;
use sand_core::sand_state;
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
