//! Language features that close gaps with the Rust entry point: custom events,
//! timers, world actions and tests, boss bars, loot tables, and smithing.

mod support;

use sand_lang::{Context, DefInfo, FileId, ide};
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

fn context_of(analysis: &sand_lang::Analysis, name: &str) -> Context {
    let def = analysis.defs.iter().find(|d| d.name == name).unwrap();
    let DefInfo::Function { context, .. } = &def.info else {
        panic!("{name} is not a function")
    };
    *context
}

fn every_function(pack: &sand::advanced::compiler::CompiledPack) -> String {
    pack.resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

fn labels(analysis: &sand_lang::Analysis, offset: usize) -> Vec<String> {
    ide::completions(analysis, FileId(0), offset)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

#[test]
fn custom_events_call_every_handler_with_the_payload() {
    let analysis = analyze_one(&pack(
        r#"
player state Wallet { coins: number }

event Payday(amount: number);

#[on(Payday)]
fn deposit(amount: number) { Wallet.coins += amount; }

#[on(Payday)]
fn thank(amount: number) { chat!("paid"); }

fn wages() { emit(Payday, 2 + 3); }
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    // A handler needs a player, so the event and whoever emits it do too.
    assert_eq!(context_of(&analysis, "Payday"), Context::Player);
    assert_eq!(context_of(&analysis, "wages"), Context::Player);
    let pack = compile(&analysis);
    let dispatch = function(&pack, "demo/function/sand_event/payday");
    let calls: Vec<_> = dispatch
        .lines()
        .filter(|line| line.starts_with("function ") && !line.contains("__sand_"))
        .collect();
    assert_eq!(
        calls,
        ["function demo:deposit", "function demo:thank"],
        "{dispatch}"
    );
    assert_eq!(
        dispatch
            .lines()
            .filter(|line| line.contains("scoreboard players operation"))
            .count(),
        2,
        "{dispatch}"
    );
    let wages = function(&pack, "demo/function/wages");
    assert!(wages.contains("function demo:sand_event/payday"), "{wages}");
    assert_eq!(
        wages.matches("scoreboard players set").count(),
        1,
        "the payload is evaluated once: {wages}"
    );
}

#[test]
fn custom_event_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
player state Wallet { coins: number }
event Payday(amount: number);
event Quiet;

#[on(Payday)]
fn wrong() {}

#[on(Quiet)]
fn pay() { Wallet.coins += 1; }

fn a() { emit(Payday); }
fn b() { emit(Missing); }
fn c() { emit(); }
fn d() { Payday(1); }
#[tick]
fn e() { emit(Quiet); }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`wrong` handles `Payday`, so it must take (amount: number)",
        "`Payday` takes 1 argument, but 0 were given",
        "`emit` needs an event first",
        "expected a function, found event `Payday`",
        "`e` runs from #[tick] without a player, but it calls `Quiet`, which needs a player",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert!(
        errors.iter().any(|e| e.contains("Missing")),
        "unknown event: {errors:#?}"
    );
}

#[test]
fn emitting_an_event_nobody_handles_warns() {
    let analysis = analyze_one(&pack("event Quiet;\nfn a() { emit(Quiet); }\n"));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|d| d.message == "`Quiet` has no handlers, so this does nothing"),
        "{:#?}",
        analysis.diagnostics
    );
}

#[test]
fn events_have_editor_support() {
    let text = pack(
        "/// Paid weekly.\nevent Payday;\n#[on()]\nfn pay() {}\nfn a() {\n    emit();\n    emit(Payday);\n}\n",
    );
    let analysis = analyze_one(&text);
    let on = text.find("#[on(").unwrap() + 5;
    let names = labels(&analysis, on);
    assert!(names.iter().any(|l| l == "Payday"), "{names:?}");
    assert!(names.iter().any(|l| l == "death"), "{names:?}");
    assert_eq!(
        labels(&analysis, text.find("emit()").unwrap() + 5),
        ["Payday"]
    );
    let hover = ide::hover(&analysis, FileId(0), text.find("emit(Payday").unwrap() + 6).unwrap();
    assert!(
        hover.markdown.contains("event Payday()"),
        "{}",
        hover.markdown
    );
    assert!(
        hover.markdown.contains("Paid weekly."),
        "{}",
        hover.markdown
    );
    let keyword = ide::hover(&analysis, FileId(0), text.find("event Payday").unwrap() + 1).unwrap();
    assert!(
        keyword.markdown.contains("custom event"),
        "{}",
        keyword.markdown
    );
    assert!(labels(&analysis, PACK.len()).iter().any(|l| l == "event"));
}

#[test]
fn timers_count_down_through_the_state_lifecycle() {
    let analysis = analyze_one(&pack(
        r#"
player state Ability { dash: timer }
global state World { storm: timer }

fn use_dash() {
    if timer_running(Ability.dash) {
        actionbar!("{Ability.dash} ticks left");
        return;
    }
    start_timer(Ability.dash, seconds = 3);
}
fn cancel() { stop_timer(Ability.dash); }
fn begin_storm() { start_timer(World.storm, ticks = 40); }
fn storm_over() -> bool { return World.storm == 0; }
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert_eq!(context_of(&analysis, "use_dash"), Context::Player);
    assert_eq!(context_of(&analysis, "begin_storm"), Context::Server);
    let pack = compile(&analysis);
    let tick = String::from_utf8_lossy(
        &pack.resources["data/demo/function/__sand_lifecycle_tick.mcfunction"],
    )
    .into_owned();
    assert_eq!(
        tick.matches("scoreboard players remove").count(),
        2,
        "player and global timers tick: {tick}"
    );
    assert!(every_function(&pack).contains(" 60"));
}

#[test]
fn timer_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
player state Ability {
    level: number,
    #[clamp(0..=5)]
    dash: timer,
}
fn a() { start_timer(Ability.level, seconds = 1); }
fn b(t: timer) {}
fn c() { start_timer(Ability.dash); }
fn d() { start_timer(5, ticks = 1); }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "only `number` and `float` fields can be clamped",
        "`level` is not a `timer` field",
        "`timer` is only for `state` fields",
        "`start_timer` needs exactly one of `ticks` or `seconds`",
        "expected a `timer` field",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn timer_completion_is_scoped() {
    let text = pack(
        "player state A { dash: timer, gold: number }\nfn a() {\n    start_timer();\n}\nplayer state B {\n    x: \n}\nfn b(n: ) {}\n",
    );
    let analysis = analyze_one(&text);
    assert_eq!(
        labels(&analysis, text.find("start_timer()").unwrap() + 12),
        ["A.dash"]
    );
    assert!(
        labels(&analysis, text.find("x: \n").unwrap() + 3)
            .iter()
            .any(|l| l == "timer")
    );
    assert!(
        !labels(&analysis, text.find("n: )").unwrap() + 3)
            .iter()
            .any(|l| l == "timer")
    );
}

#[test]
fn world_actions_and_tests_lower_to_commands() {
    let analysis = analyze_one(&pack(
        r#"
tag Soft: Block { values: [Block::Sand] }
item Wand { base: Item::Stick }

fn build() {
    set_block(Block::Stone, offset(0, -1, 0));
    set_block(Block::Glass, pos(10, 64, -3));
    fill_blocks(Block::Air, offset(-1, 0, -1), offset(1, 2, 1));
    add_experience(points = 10);
    add_experience(levels = -2);
    set_gamemode(GameMode::Adventure);
    drop_loot(Prize);
}
loot Prize { items: [Item::Diamond] }

fn world() {
    set_time(6000);
    set_weather(Weather::Rain, seconds = 30);
    set_weather(Weather::Clear);
}

fn probes() {
    if block_is(Block::Water, offset(0, -1, 0)) { kill(); }
    if block_is(Soft, offset(0, -1, 0)) { kill(); }
    if holding(Item::Diamond) { kill(); }
    if holding(Wand, offhand = true) { kill(); }
    if is_sneaking() { kill(); }
    if is_on_fire() && any_entities(type = Entity::Zombie, distance = ..=8) { kill(); }
}
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert_eq!(context_of(&analysis, "build"), Context::Player);
    assert_eq!(context_of(&analysis, "world"), Context::Server);
    let pack = compile(&analysis);
    let build = function(&pack, "demo/function/build");
    for expected in [
        "setblock ~ ~-1 ~ minecraft:stone",
        "setblock 10 64 -3 minecraft:glass",
        "fill ~-1 ~ ~-1 ~1 ~2 ~1 minecraft:air",
        "xp add @s 10 points",
        "xp add @s -2 levels",
        "gamemode adventure @s",
        "loot spawn ~ ~ ~ loot demo:prize",
    ] {
        assert!(build.contains(expected), "missing {expected:?} in {build}");
    }
    let world = function(&pack, "demo/function/world");
    for expected in ["time set 6000", "weather rain 30s", "weather clear"] {
        assert!(world.contains(expected), "missing {expected:?} in {world}");
    }
    let all = every_function(&pack);
    for expected in [
        "if block ~ ~-1 ~ minecraft:water",
        "if block ~ ~-1 ~ #demo:soft",
        "if items entity @s weapon.mainhand minecraft:diamond",
        "if items entity @s weapon.offhand minecraft:stick[",
        "if predicate demo:__sand_program/flag/sneaking",
        "if entity @e[type=minecraft:zombie,distance=..8]",
    ] {
        assert!(all.contains(expected), "missing {expected:?} in {all}");
    }
    let predicate: serde_json::Value = serde_json::from_slice(
        &pack.resources["data/demo/predicate/__sand_program/flag/sneaking.json"],
    )
    .unwrap();
    assert_eq!(predicate["condition"], "minecraft:entity_properties");
    assert_eq!(
        predicate["predicate"]["minecraft:flags"]["is_sneaking"], true,
        "{predicate}"
    );
}

#[test]
fn event_handlers_run_at_the_player_and_player_loops_keep_the_position() {
    let analysis = analyze_one(&pack(
        r#"
fn near() { set_block(Block::Stone, offset(0, -1, 0)); }
#[on(jump)]
fn hop() { near(); }
#[tick]
fn tick() {
    for player in players() {
        set_block(Block::Glass, offset(0, -1, 0));
    }
}
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let pack = compile(&analysis);
    let lifecycle = String::from_utf8_lossy(
        &pack.resources["data/demo/function/__sand_lifecycle_tick.mcfunction"],
    )
    .into_owned();
    assert!(
        lifecycle.contains("] at @s run function demo:__sand_program/event/"),
        "{lifecycle}"
    );
    // Like Rust's `Execute::new().as_(Target::players())`, iteration changes
    // only the executor.
    let tick = function(&pack, "demo/function/tick");
    assert!(tick.contains("execute as @a run function"), "{tick}");
    assert!(!tick.contains("at @s"), "{tick}");
}

#[test]
fn world_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
tag Things: Item { values: [Item::Stick] }
#[tick]
fn a() { set_block(Block::Stone, offset(0, 0, 0)); }
fn b() { set_block(Block::Stone, 1); }
fn c() { add_experience(); }
fn d() { set_gamemode(Weather::Rain); }
fn e() { if block_is(Things, pos(0, 0, 0)) { return; } }
fn f(x: number) { set_block(Block::Stone, pos(x, 0, 0)); }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`a` runs from #[tick] on the server, but it uses `offset`, which is relative to the executing entity",
        "expected a block position",
        "`add_experience` needs exactly one of `points` or `levels`",
        "expected a `GameMode`, found a `Weather`",
        "`Things` is not a `Block` tag",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert_eq!(
        errors.iter().filter(|e| e.contains("constant")).count(),
        1,
        "positions are constants: {errors:#?}"
    );
}

#[test]
fn boss_bars_are_created_at_load_and_track_viewers() {
    let analysis = analyze_one(&pack(
        r#"
global state Fight { boss_hp: number = 80 }

/// The raid boss.
bossbar Raid {
    title: "{red}Raid Boss",
    color: BossBarColor::Purple,
    style: BossBarStyle::Notched10,
    max: 80,
}

fn begin() {
    show_bossbar_all(Raid);
    set_bossbar(Raid, Fight.boss_hp);
    set_bossbar(Raid, Fight.boss_hp - 5);
    set_bossbar(Raid, 80);
    set_bossbar_max(Raid, 100);
}
fn peek() { show_bossbar(Raid); }
fn leave() { hide_bossbar(Raid); }
fn finish() { hide_bossbar_all(Raid); }
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert_eq!(context_of(&analysis, "begin"), Context::Server);
    assert_eq!(context_of(&analysis, "peek"), Context::Player);
    let pack = compile(&analysis);
    let load = String::from_utf8_lossy(
        &pack.resources["data/demo/function/__sand_lifecycle_load.mcfunction"],
    )
    .into_owned();
    for expected in [
        "bossbar add demo:raid ",
        "bossbar set demo:raid color purple",
        "bossbar set demo:raid style notched_10",
        "bossbar set demo:raid max 80",
    ] {
        assert!(load.contains(expected), "missing {expected:?} in {load}");
    }
    let begin = function(&pack, "demo/function/begin");
    assert!(begin.contains("tag @a add sand."), "{begin}");
    assert!(
        begin.contains("execute store result bossbar demo:raid value run scoreboard players get"),
        "{begin}"
    );
    assert!(begin.contains("bossbar set demo:raid value 80"), "{begin}");
    assert!(begin.contains("bossbar set demo:raid max 100"), "{begin}");
    let peek = function(&pack, "demo/function/peek");
    assert!(peek.contains("tag @s add sand."), "{peek}");
    assert!(
        peek.contains("bossbar set demo:raid players @a[tag=sand."),
        "{peek}"
    );
}

#[test]
fn loot_tables_and_smithing_lower_to_typed_resources() {
    let analysis = analyze_one(&pack(
        r#"
loot Chest {
    rolls: 2,
    items: [Item::Diamond, { item: Item::Emerald, weight: 4, count: 3 }],
}
recipe Upgrade {
    smithing: Item::NetheriteUpgradeSmithingTemplate,
    base: Item::DiamondAxe,
    addition: Item::NetheriteIngot,
    result: Item::NetheriteAxe,
}
fn reward() { give_loot(Chest); }
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert_eq!(context_of(&analysis, "reward"), Context::Player);
    let pack = compile(&analysis);
    let table: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/demo/loot_table/chest.json"]).unwrap();
    assert_eq!(table["pools"][0]["rolls"], 2.0);
    let entries = &table["pools"][0]["entries"];
    assert_eq!(entries[0]["name"], "minecraft:diamond");
    assert_eq!(entries[1]["weight"], 4);
    assert_eq!(
        entries[1]["functions"][0]["function"],
        "minecraft:set_count"
    );
    let recipe: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/demo/recipe/upgrade.json"]).unwrap();
    assert_eq!(recipe["type"], "minecraft:smithing_transform");
    assert_eq!(recipe["base"], "minecraft:diamond_axe");
    assert!(function(&pack, "demo/function/reward").contains("loot give @s loot demo:chest"));
}

#[test]
fn data_mistakes_and_completion_for_new_declarations() {
    let analysis = analyze_one(&pack(
        r#"
item Gem { base: Item::Emerald }
bossbar A { color: BossBarColor::Red }
bossbar B { title: "{Score.v}" }
loot C { items: [] }
loot D { items: [Gem] }
loot E { items: [{ weight: 1 }] }
recipe F { smithing: Item::Stick, result: Item::Stick }
recipe G { shapeless: [Item::Stick], base: Item::Stick, result: Item::Stick }
fn h() { show_bossbar(C); give_loot(A); }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`bossbar` needs `title`",
        "a loot table needs at least one item",
        "`Gem` is a custom item; the base must be a vanilla item",
        "a loot item needs `item`",
        "a smithing recipe needs `base` and `addition`",
        "`base` does not apply to a `shapeless` recipe",
        "expected a boss bar, found loot table `C`",
        "expected a loot table, found boss bar `A`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    let text = pack(
        "bossbar Bar { title: \"Bar\" }\nbossbar Draft {\n    \n    color: ,\n}\nloot Prize { items: [Item::Stick] }\nfn a() {\n    show_bossbar();\n    give_loot();\n    set_block(Block::Stone, );\n}\n",
    );
    let analysis = analyze_one(&text);
    let fields = labels(&analysis, text.find("Draft {\n    \n").unwrap() + 12);
    assert!(fields.iter().any(|l| l == "title"), "{fields:?}");
    assert!(
        labels(&analysis, text.find("color: ,").unwrap() + 7)
            .iter()
            .any(|l| l == "BossBarColor::Red")
    );
    assert_eq!(
        labels(&analysis, text.find("show_bossbar()").unwrap() + 13),
        ["Bar"]
    );
    assert_eq!(
        labels(&analysis, text.find("give_loot()").unwrap() + 10),
        ["Prize"]
    );
    assert_eq!(
        labels(&analysis, text.find("Block::Stone, )").unwrap() + 14),
        ["offset", "pos"]
    );
}

#[test]
fn public_events_are_handled_across_modules() {
    let analysis = analyze(&[
        (
            "pack.sand",
            "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\npub mod raid;\nglobal state S { n: number }\n#[on(raid::Struck)]\nfn count(damage: number) { S.n += damage; }\nfn a() { raid::hit(); }\n",
        ),
        (
            "raid.sand",
            "pub event Struck(damage: number);\npub fn hit() { emit(Struck, 4); }\n",
        ),
    ]);
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    let pack = compile(&analysis);
    let dispatch = function(&pack, "demo/function/sand_event/raid/struck");
    assert!(dispatch.contains("function demo:count"), "{dispatch}");
}

#[test]
fn predicates_use_the_schema_of_the_target_version() {
    let predicate = |minecraft: &str| {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"demo\", minecraft = \"{minecraft}\")]\n\
             pub fn a() {{ if is_sneaking() {{ chat!(\"down\"); }} }}\n"
        ));
        assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
        let pack = compile(&analysis);
        let (_, bytes) = pack
            .resources
            .iter()
            .find(|(path, _)| path.contains("/predicate/"))
            .expect("is_sneaking generates a predicate");
        serde_json::from_slice::<serde_json::Value>(bytes).unwrap()
    };
    let before = predicate("26.2");
    assert_eq!(before["condition"], "minecraft:entity_properties");
    assert!(before.get("type").is_none());
    let after = predicate("26.3");
    assert_eq!(after["type"], "minecraft:entity_properties");
    assert!(after.get("condition").is_none());
}
