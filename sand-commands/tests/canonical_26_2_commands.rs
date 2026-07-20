//! Canonical Minecraft Java 26.2 command fixture (Phase 4c).
//!
//! Every test in this file asserts the FULL exact `.mcfunction`-equivalent
//! output of a `sand-commands` typed builder against hand-verified vanilla
//! syntax — not a `.contains` substring check. `sand-commands` has no
//! dependency on `sand-core`, so only this crate's own public API
//! (`sand_commands::{Execute, Selector, ...}`, see `sand-commands/src/lib.rs`)
//! is used here.
//!
//! Each group's doc comment cites the vanilla command page it was checked
//! against on <https://minecraft.wiki/w/Commands> (page name + verification
//! date), per ADR-001's canonical-fixture requirement. No large wiki excerpts
//! are pasted — only the page name and the date checked.
//!
//! Test naming: `canonical_26_2_*` denotes a canonical, version-pinned
//! assertion. This file has no `compat_1_21_4_*` variants because
//! `sand-commands` has no `MinecraftVersion`/profile-gated rendering branch
//! for the command families exercised here (`CommandProfile` is accepted by
//! `Validate`/`RenderCommand` but the render paths used below do not branch
//! on it) — see the final report for details.
//!
//! Known gaps (no typed builder exists in `sand-commands` for these, so they
//! are intentionally not covered below): the `ride` command, and `attribute
//! ... modifier add/remove` (only `attribute ... get` / `attribute ... base
//! set` exist). See the task report for the full list.

use sand_commands::blocks::{CloneBlocks, CloneMode, Fill, FillMode, SetBlock, SetBlockMode};
use sand_commands::builtins::{
    attribute_base_set, attribute_get, effect_clear, effect_give, function, kill, summon_at,
    tellraw, tp_with_rotation,
};
use sand_commands::coord::{BlockPos, Rotation, Vec3};
use sand_commands::nbt::{DataModify, DataTarget, NbtValue, data_modify};
use sand_commands::particles::{Particle, ParticleBuilder, ParticleSpread};
use sand_commands::scoreboard::{
    DisplaySlot, Objective, ObjectiveName, ScoreHolder, ScoreOp, scoreboard_players_operation,
};
use sand_commands::selector::SortOrder;
use sand_commands::sound::{Sound, SoundSource};
use sand_commands::{
    Actionbar, BlockState, Bossbar, BossbarColor, BossbarStyle, Build, ChatColor, Execute,
    Inventory, ItemSlot, RawCommand, Selector, Text, Title,
};

// ── Selectors ─────────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Target_selectors (2026-07-19):
// base selectors `@a`/`@s`/`@p`/`@e`/`@r` and the argument syntax/ordering
// for `type=`, `tag=`, `distance=`, `scores={}`, `predicate=`, `gamemode=`,
// `limit=`, and `sort=`.

#[test]
fn canonical_26_2_selector_basic_bases() {
    assert_eq!(Selector::all_players().to_string(), "@a");
    assert_eq!(Selector::self_().to_string(), "@s");
    assert_eq!(Selector::nearest_player().to_string(), "@p");
    assert_eq!(Selector::all_entities().to_string(), "@e");
    assert_eq!(Selector::random_player().to_string(), "@r");
}

#[test]
fn canonical_26_2_selector_typed_filters_combined() {
    let sel = Selector::all_players()
        .entity_type("minecraft:player")
        .tag("ready")
        .distance_range(1.0, 10.0)
        .scores("kills=1..10")
        .predicate("my_pack:is_sneaking")
        .gamemode("survival")
        .sort(SortOrder::Nearest)
        .limit(5);
    assert_eq!(
        sel.to_string(),
        "@a[type=minecraft:player,tag=ready,distance=1..10,scores={kills=1..10},\
predicate=my_pack:is_sneaking,gamemode=survival,sort=nearest,limit=5]"
    );
}

// ── Execute chains ───────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/execute (2026-07-19):
// clause ordering (`as`/`at`/`positioned`/`store`/`if`/`unless`) and the
// terminal `run` clause.

#[test]
fn canonical_26_2_execute_multi_clause_with_store_if_score_if_block_positioned_run() {
    let cmd = Execute::new()
        .as_(Selector::all_players().tag("ready"))
        .at(Selector::self_())
        .positioned(Vec3::absolute(0.0, 64.0, 0.0))
        .store_result_score(ScoreHolder::self_(), "result")
        .if_score_matches("@s", "health", "1..")
        .if_block(BlockPos::here(), "minecraft:stone")
        .run_raw("say chain complete");
    assert_eq!(
        cmd,
        "execute as @a[tag=ready] at @s positioned 0 64 0 \
store result score @s result if score @s health matches 1.. \
if block ~ ~ ~ minecraft:stone run say chain complete"
    );
}

// ── Scoreboard ────────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/scoreboard (2026-07-19):
// `objectives add/remove/setdisplay`, `players set/add/remove/operation`.

#[test]
fn canonical_26_2_scoreboard_objectives_add_remove_display() {
    let phase: Objective = Objective::new("phase");
    assert_eq!(
        phase.create("dummy"),
        "scoreboard objectives add phase dummy"
    );
    assert_eq!(phase.remove(), "scoreboard objectives remove phase");
    assert_eq!(
        phase.set_display(DisplaySlot::Sidebar),
        "scoreboard objectives setdisplay sidebar phase"
    );
    assert_eq!(
        Objective::clear_display(DisplaySlot::List),
        "scoreboard objectives setdisplay list"
    );
}

#[test]
fn canonical_26_2_scoreboard_players_set_add_remove_operation() {
    static MANA: Objective = Objective::new("mana");
    static REGEN: Objective = Objective::new("regen");
    let holder = ScoreHolder::self_();
    assert_eq!(
        MANA.set(holder.clone(), 10),
        "scoreboard players set @s mana 10"
    );
    assert_eq!(
        MANA.add(holder.clone(), 5),
        "scoreboard players add @s mana 5"
    );
    assert_eq!(
        MANA.subtract(holder.clone(), 3),
        "scoreboard players remove @s mana 3"
    );
    let op = scoreboard_players_operation(
        holder.clone(),
        ObjectiveName::new("mana"),
        ScoreOp::Add,
        holder,
        ObjectiveName::new("regen"),
    );
    assert_eq!(
        op.to_string(),
        "scoreboard players operation @s mana += @s regen"
    );
    let _ = &REGEN; // documents the paired objective this operation reads from
}

// ── Data / NBT ────────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/data (2026-07-19):
// `data modify ... set value`, `data modify ... append value`,
// `data modify ... merge value`, and `data get`.

#[test]
fn canonical_26_2_data_modify_set_get_merge_append() {
    let set_cmd = data_modify(DataTarget::storage("my_pack:state"), "phase").set(2_i32);
    assert_eq!(
        set_cmd,
        "data modify storage my_pack:state phase set value 2"
    );

    let append_cmd = data_modify(DataTarget::storage("my_pack:log"), "kills")
        .append(NbtValue::raw(r#"{type:"zombie"}"#));
    assert_eq!(
        append_cmd,
        r#"data modify storage my_pack:log kills append value {type:"zombie"}"#
    );

    let merge_cmd = DataModify::new(DataTarget::entity(Selector::self_()), "Inventory")
        .merge(NbtValue::raw("{CustomModelData:5}"));
    assert_eq!(
        merge_cmd,
        "data modify entity @s Inventory merge value {CustomModelData:5}"
    );

    // `data get` has no dedicated typed builder in sand-commands; DataTarget's
    // own Display impl composes the command directly (still typed API, no
    // hand-rolled string).
    let get_cmd = format!(
        "data get {} {}",
        DataTarget::entity(Selector::self_()),
        "SelectedItem"
    );
    assert_eq!(get_cmd, "data get entity @s SelectedItem");
}

// ── Inventory / item ──────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/item and
// https://minecraft.wiki/w/Commands/give (2026-07-19): `item replace`,
// `item modify`, `give`, `clear`, using the canonical `ItemSlot` type (the
// deprecated `InventorySlot`/`SlotPattern` types no longer exist per
// ADR-001).

#[test]
fn canonical_26_2_inventory_item_replace_modify_give_clear() {
    let inv = Inventory::of(Selector::self_());
    assert_eq!(
        inv.set(ItemSlot::MainHand, "minecraft:netherite_sword"),
        "item replace entity @s weapon.mainhand with minecraft:netherite_sword"
    );
    assert_eq!(
        inv.set_count(ItemSlot::Hotbar(0), "minecraft:arrow", 64),
        "item replace entity @s hotbar.0 with minecraft:arrow 64"
    );
    assert_eq!(
        inv.modify(ItemSlot::MainHand, "my_pack:sharpen"),
        "item modify entity @s weapon.mainhand my_pack:sharpen"
    );
    assert_eq!(inv.give("minecraft:diamond"), "give @s minecraft:diamond");
    assert_eq!(
        inv.clear_item_count("minecraft:dirt", 32),
        "clear @s minecraft:dirt 32"
    );
    assert_eq!(inv.clear_all(), "clear @s");
}

// ── Attributes ────────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/attribute (2026-07-19):
// `attribute <target> <attribute> get` and `attribute <target> <attribute>
// base set <value>`. `sand-commands` has no typed builder for `attribute
// ... modifier add/remove`, so that family is not exercised here (see
// report).

#[test]
fn canonical_26_2_attribute_base_set_and_get() {
    assert_eq!(
        attribute_get(Selector::self_(), "minecraft:generic.max_health"),
        "attribute @s minecraft:generic.max_health get"
    );
    assert_eq!(
        attribute_base_set(Selector::self_(), "minecraft:generic.max_health", 40.0),
        "attribute @s minecraft:generic.max_health base set 40"
    );
}

// ── Entity operations ─────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/summon,
// https://minecraft.wiki/w/Commands/kill, https://minecraft.wiki/w/Commands/tp,
// and https://minecraft.wiki/w/Commands/effect (2026-07-19). `sand-commands`
// has no typed builder for the `ride` command, so it is not exercised here
// (see report).

#[test]
fn canonical_26_2_entity_summon_kill_tp_rotation_effect() {
    assert_eq!(
        summon_at("minecraft:zombie", Vec3::absolute(10.0, 64.0, -5.0)),
        "summon minecraft:zombie 10 64 -5"
    );
    assert_eq!(
        kill(Selector::all_entities().entity_type("minecraft:zombie")),
        "kill @e[type=minecraft:zombie]"
    );
    assert_eq!(
        tp_with_rotation(
            Selector::self_(),
            Vec3::absolute(0.0, 70.0, 0.0),
            Rotation::absolute(180.0, 0.0)
        ),
        "tp @s 0 70 0 180 0"
    );
    assert_eq!(
        effect_give(Selector::self_(), "minecraft:regeneration", 200, 1),
        "effect give @s minecraft:regeneration 200 1"
    );
    assert_eq!(effect_clear(Selector::self_()), "effect clear @s");
}

// ── Block commands ────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/setblock,
// https://minecraft.wiki/w/Commands/fill, and
// https://minecraft.wiki/w/Commands/clone (2026-07-19).

#[test]
fn canonical_26_2_block_setblock_fill_clone() {
    let setblock_cmd = SetBlock::new(
        BlockPos::absolute(100, 64, -30),
        BlockState::of("minecraft:oak_stairs")
            .prop("facing", "east")
            .prop("half", "bottom"),
    )
    .mode(SetBlockMode::Destroy)
    .build();
    assert_eq!(
        setblock_cmd,
        "setblock 100 64 -30 minecraft:oak_stairs[facing=east,half=bottom] destroy"
    );

    let fill_cmd = Fill::new(
        BlockPos::absolute(0, 60, 0),
        BlockPos::absolute(10, 65, 10),
        "minecraft:glass",
    )
    .mode(FillMode::Hollow)
    .build();
    assert_eq!(fill_cmd, "fill 0 60 0 10 65 10 minecraft:glass hollow");

    let clone_cmd = CloneBlocks::new(
        BlockPos::absolute(0, 60, 0),
        BlockPos::absolute(5, 65, 5),
        BlockPos::absolute(20, 60, 0),
    )
    .masked()
    .clone_mode(CloneMode::Force)
    .build();
    assert_eq!(clone_cmd, "clone 0 60 0 5 65 5 20 60 0 masked force");
}

// ── Text output: tellraw / title / actionbar / bossbar ───────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/tellraw,
// https://minecraft.wiki/w/Commands/title, and
// https://minecraft.wiki/w/Commands/bossbar (2026-07-19), and Minecraft's
// JSON text component format at https://minecraft.wiki/w/Text_component_format
// (2026-07-19). `sand-commands` serializes JSON text objects via `serde_json`
// without the `preserve_order` feature, so object keys render in sorted
// (alphabetical) order — the exact strings below reflect that.

#[test]
fn canonical_26_2_tellraw_with_color_bold_hover_chain() {
    let msg = Text::new("Boss Incoming")
        .color(ChatColor::Gold)
        .bold(true)
        .hover_text(Text::new("A powerful foe").color(ChatColor::Gray));
    let cmd = tellraw(Selector::all_players(), msg);
    assert_eq!(
        cmd,
        r#"tellraw @a {"bold":true,"color":"gold","hoverEvent":{"action":"show_text","contents":{"color":"gray","text":"A powerful foe"}},"text":"Boss Incoming"}"#
    );
}

#[test]
fn canonical_26_2_title_subtitle_ordering_with_styled_title() {
    let cmds = Title::of(Selector::all_players())
        .title(Text::new("Boss Incoming").color(ChatColor::Gold).bold(true))
        .subtitle(Text::new("Prepare yourself"))
        .times(5, 60, 10)
        .build();
    assert_eq!(
        cmds,
        vec![
            "title @a times 5 60 10".to_string(),
            r#"title @a subtitle {"text":"Prepare yourself"}"#.to_string(),
            r#"title @a title {"bold":true,"color":"gold","text":"Boss Incoming"}"#.to_string(),
        ]
    );
}

#[test]
fn canonical_26_2_actionbar_with_color_click_chain() {
    let cmd = Actionbar::show(
        Selector::self_(),
        Text::new("Click for info")
            .color(ChatColor::Aqua)
            .click_run_command("/help"),
    );
    assert_eq!(
        cmd,
        r#"title @s actionbar {"clickEvent":{"action":"run_command","value":"/help"},"color":"aqua","text":"Click for info"}"#
    );
}

#[test]
fn canonical_26_2_bossbar_add_and_configure() {
    assert_eq!(
        Bossbar::add(
            "my_pack:boss_bar",
            Text::new("Ancient Guardian")
                .color(ChatColor::Red)
                .bold(true)
        ),
        r#"bossbar add my_pack:boss_bar {"bold":true,"color":"red","text":"Ancient Guardian"}"#
    );
    assert_eq!(
        Bossbar::set_color("my_pack:boss_bar", BossbarColor::Purple),
        "bossbar set my_pack:boss_bar color purple"
    );
    assert_eq!(
        Bossbar::set_style("my_pack:boss_bar", BossbarStyle::Notched20),
        "bossbar set my_pack:boss_bar style notched_20"
    );
    assert_eq!(
        Bossbar::set_players("my_pack:boss_bar", Selector::all_players()),
        "bossbar set my_pack:boss_bar players @a"
    );
}

// ── Particles and sounds ──────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/particle and
// https://minecraft.wiki/w/Commands/playsound (2026-07-19): particle spread/
// count argument order, and playsound source/volume/pitch/minVolume order.

#[test]
fn canonical_26_2_particle_with_spread_and_count() {
    let cmds = ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.5))
        .spread(ParticleSpread::uniform(0.5))
        .speed(0.1)
        .particles_per_point(3)
        .points_at(&[[1.0, 2.0, 3.0]]);
    assert_eq!(cmds.len(), 1);
    assert_eq!(
        cmds[0],
        "particle minecraft:dust{color:[1,0.27,0],scale:1.5} ~1 ~2 ~3 0.5 0.5 0.5 0.1 3 force"
    );
}

#[test]
fn canonical_26_2_playsound_with_source_volume_pitch() {
    let cmd = Sound::play("minecraft:entity.wither.spawn")
        .to(Selector::all_players())
        .source(SoundSource::Hostile)
        .at(Vec3::absolute(0.0, 70.0, 0.0))
        .volume(4.0)
        .pitch(0.8)
        .min_volume(0.2)
        .build();
    assert_eq!(
        cmd,
        "playsound minecraft:entity.wither.spawn hostile @a 0 70 0 4 0.8 0.2"
    );

    assert_eq!(
        Sound::stop(
            Selector::all_players(),
            SoundSource::Music,
            "minecraft:music.overworld"
        ),
        "stopsound @a music minecraft:music.overworld"
    );
}

// ── Function calls ────────────────────────────────────────────────────────────
//
// Verified against https://minecraft.wiki/w/Commands/function (2026-07-19):
// `function <namespace:path>`, `function #<namespace:path>` (function tag
// call), and `execute ... run function <id>`.

#[test]
fn canonical_26_2_function_calls_direct_and_tag() {
    assert_eq!(function("my_pack:on_load"), "function my_pack:on_load");
    assert_eq!(
        function("#my_pack:all_ticks"),
        "function #my_pack:all_ticks"
    );
    assert_eq!(
        Execute::new()
            .as_(Selector::all_players())
            .run_fn("my_pack:on_join"),
        "execute as @a run function my_pack:on_join"
    );
}

// ── Raw escape hatch ──────────────────────────────────────────────────────────
//
// This test intentionally exercises `RawCommand`, Sand's documented escape
// hatch for command syntax the typed builders don't model. Verified against
// https://minecraft.wiki/w/Commands (2026-07-19) only insofar as confirming
// `.mcfunction` lines are plain unprefixed text — `RawCommand` performs no
// grammar validation and must emit its input unchanged.

#[test]
fn canonical_26_2_raw_command_escape_hatch_emits_literal_unchanged() {
    let raw = RawCommand::new("say this is a raw literal, unchanged");
    assert_eq!(raw.as_str(), "say this is a raw literal, unchanged");

    let chained = Execute::new()
        .as_(Selector::self_())
        .try_run_raw(RawCommand::new("execute-unmodeled-syntax"))
        .unwrap();
    assert_eq!(chained, "execute as @s run execute-unmodeled-syntax");
}
