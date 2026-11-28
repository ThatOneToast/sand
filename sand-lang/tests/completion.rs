//! Completion offers exactly what is valid at each position, including while
//! the surrounding code does not parse yet. `|` marks the cursor.

mod support;

use sand_lang::{FileId, ide};

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

const CONTEXT: &str = r#"
const MAX: number = 10;
enum Phase { Idle, Busy }
player state Hero { phase: Phase, energy: number }
fn act() {}
fn price(n: number) -> number { return n; }
mod tools { pub fn hammer() {} }
"#;

/// Completion labels at the `|` in `source`, which follows a shared context.
fn labels(source: &str) -> Vec<String> {
    let text = format!("{PACK}{CONTEXT}{source}");
    let cursor = text.find('|').expect("cursor marker");
    let text = text.replacen('|', "", 1);
    let analysis = support::analyze_one(&text);
    ide::completions(&analysis, FileId(0), cursor)
        .into_iter()
        .map(|c| c.label)
        .collect()
}

#[track_caller]
fn exactly(source: &str, expected: &[&str]) {
    let mut got = labels(source);
    got.sort();
    let mut want: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    want.sort();
    assert_eq!(got, want, "at {source:?}");
}

#[track_caller]
fn includes_excludes(source: &str, include: &[&str], exclude: &[&str]) {
    let got = labels(source);
    for label in include {
        assert!(
            got.iter().any(|l| l == label),
            "{label:?} missing at {source:?}: {got:?}"
        );
    }
    for label in exclude {
        assert!(
            !got.iter().any(|l| l == label),
            "{label:?} must not appear at {source:?}: {got:?}"
        );
    }
}

#[test]
fn item_level() {
    exactly(
        "pub |",
        &[
            "fn",
            "fn ->",
            "player state",
            "global state",
            "entity state",
            "enum",
            "struct",
            "trait",
            "flow",
            "const",
            "item",
            "recipe shaped",
            "bossbar",
            "damage_type",
            "predicate",
            "advancement",
            "dialog",
            "dimension",
            "dimension_type",
            "biome",
            "structure_template",
            "structure_set",
            "ore_feature",
            "configured_feature",
            "placed_feature",
            "jigsaw_structure",
            "template_pool",
            "inventory_transfer",
            "checkpoint",
            "encounter",
            "roster",
            "status_effect",
            "vfx",
            "archetype",
            "hud",
            "menu",
            "projectile",
            "progression",
            "scene",
            "job",
            "session",
            "ability",
            "combat",
            "event",
            "loot",
            "recipe shapeless",
            "recipe smelting",
            "tag",
            "mod",
            "use",
            "extern fn",
        ],
    );
    includes_excludes(
        "|",
        &["pub", "tick fn", "event fn", "fn"],
        &["let", "give_item", "Item"],
    );
    exactly("player |", &["state"]);
    exactly("pub global |", &["state"]);
    exactly("flow F |", &["on"]);
    exactly("flow F on |", &["Hero"]);
    exactly("tag Ores: |", &["Item", "Block", "Entity"]);
    exactly(
        "const LIMIT: |",
        &[
            "number",
            "float",
            "bool",
            "EntityRef",
            "string",
            "Vec",
            "Phase",
            "tools",
        ],
    );
    exactly(
        "const LIMIT: number = |",
        &["MAX", "Phase", "tools", "crate", "self", "super"],
    );
    // New names: nothing to suggest.
    exactly("fn |", &[]);
    exactly("enum |", &[]);
    exactly("item |", &[]);
    exactly("mod |", &[]);
}

#[test]
fn attributes_fit_what_they_are_attached_to() {
    exactly(
        "#[|\nfn later() {}",
        &["tick", "load", "on", "tag", "player", "test"],
    );
    exactly("#[|\nextern \"x:y\" fn ext();", &["player"]);
    exactly("#[|\nenum E { A }", &[]);
    exactly("player state S {\n    #[|\n    v: number,\n}", &["clamp"]);
    exactly("#![|", &["pack"]);
    exactly(
        "#[on(|)]\nfn later() {}",
        &[
            "join",
            "first_join",
            "death",
            "kill",
            "kill_player",
            "advancement",
            "killed_by",
            "use_item",
            "craft_item",
            "pick_up_item",
            "mine_block",
            "jump",
            "drop_item",
            "sleep",
            "take_damage",
            "deal_damage",
            "start_sneaking",
            "stop_sneaking",
            "start_sprinting",
            "stop_sprinting",
            "start_swimming",
            "stop_swimming",
            "start_flying",
            "stop_flying",
            "start_burning",
            "stop_burning",
            "health_changed",
            "health_lost",
            "health_gained",
        ],
    );
    includes_excludes(
        "advancement Scout { criteria: { found: { inventory_item: Item::Paper } } }\n#[on(advancement = |)]\nfn later() {}",
        &["Scout"],
        &["Item::Paper", "later"],
    );
    includes_excludes(
        "predicate Clear { chance: 1 }\n#[on(start_flying, when = |)]\nfn later() {}",
        &["Clear"],
        &["start_flying", "later"],
    );
    includes_excludes(
        "#[on(use_item = |)]\nfn later() {}",
        &["Item::Diamond"],
        &["Entity::Zombie"],
    );
}

#[test]
fn types_and_state_bodies() {
    exactly(
        "player state S { v: | }",
        &[
            "number",
            "float",
            "bool",
            "EntityRef",
            "string",
            "Vec",
            "timer",
            "Phase",
            "tools",
        ],
    );
    exactly(
        "player state S { v: number = | }",
        &["MAX", "Phase", "tools", "crate", "self", "super"],
    );
    exactly("player state S {\n    |\n}", &["#[clamp]"]);
    exactly(
        "fn f(n: |) {}",
        &[
            "number",
            "float",
            "bool",
            "EntityRef",
            "string",
            "Vec",
            "Phase",
            "tools",
        ],
    );
    exactly(
        "fn f() -> |",
        &[
            "number",
            "float",
            "bool",
            "EntityRef",
            "string",
            "Vec",
            "Phase",
            "tools",
        ],
    );
    exactly("fn f(|) {}", &[]);
    exactly(
        "enum E { A = | }",
        &["MAX", "Phase", "tools", "crate", "self", "super"],
    );
    exactly("enum E { | }", &[]);
}

#[test]
fn statements_and_expressions() {
    let statement = labels("fn f() {\n    |\n}");
    for label in [
        "let",
        "if",
        "while",
        "for player in players()",
        "return",
        "give_item",
        "chat!",
        "act",
        "Hero",
        "tools",
    ] {
        assert!(
            statement.iter().any(|l| l == label),
            "{label} missing: {statement:?}"
        );
    }
    for label in ["continue", "random", "Item", "MAX", "fn", "pub", "entities"] {
        assert!(
            !statement.iter().any(|l| l == label),
            "{label} must not appear: {statement:?}"
        );
    }
    // Values only after `=`; functions without a return value are not values.
    includes_excludes(
        "fn f() {\n    let x = |\n}",
        &[
            "price",
            "MAX",
            "Hero",
            "Item",
            "random",
            "Player",
            "count_entities",
        ],
        &["act", "let", "give_item", "entities"],
    );
    exactly("fn f() {\n    let |\n}", &[]);
    exactly("fn f() {\n    for p |\n}", &["in"]);
    exactly(
        "fn f() {\n    for p in |\n}",
        &[
            "players()",
            "entities(..)",
            "resolve_entity(..)",
            "resolve_player(..)",
        ],
    );
    exactly("fn f() {\n    if true {} else |\n}", &["if"]);
    includes_excludes(
        "fn f() -> number {\n    return |\n}",
        &["price", "MAX"],
        &["let"],
    );
    exactly("fn f() {\n    return |\n}", &[]);
    // Loops: `continue`, and no player-only builtins as an entity.
    includes_excludes(
        "fn f() {\n    for e in entities() {\n        |\n    }\n}",
        &["continue", "kill", "add_tag"],
        &["chat!", "give_item", "play_sound"],
    );
    includes_excludes(
        "fn f() {\n    for p in players() {\n        |\n    }\n}",
        &["continue", "chat!"],
        &[],
    );
    // Named arguments not yet given, and typed values for the argument.
    includes_excludes(
        "fn f() {\n    add_effect(Effect::Speed, seconds = 5, |)\n}",
        &["level =", "hide_particles ="],
        &["seconds ="],
    );
    includes_excludes(
        "fn f() {\n    give_item(|)\n}",
        &["Item::Diamond"],
        &["Effect::Speed"],
    );
}

#[test]
fn flows() {
    exactly(
        "flow F on Hero.phase {\n    |\n}",
        &["enter", "exit", "tick", "Idle", "Busy"],
    );
    exactly(
        "flow F on Hero.phase {\n    Idle -> |\n}",
        &["Idle", "Busy"],
    );
    exactly("flow F on Hero.phase {\n    enter |\n}", &["Idle", "Busy"]);
    exactly("flow F on Hero.phase {\n    enter Busy => |\n}", &["act"]);
    includes_excludes(
        "flow F on Hero.phase {\n    tick Busy {\n        |\n    }\n}",
        &["let", "act"],
        &["enter"],
    );
}

#[test]
fn data_declarations_offer_only_remaining_valid_fields() {
    exactly("vfx Celebration {\n    |\n}", &["cues"]);
    exactly(
        "configured_feature Marker {\n    |\n}",
        &[
            "simple_block",
            "weighted_blocks",
            "fill_layer",
            "ore",
            "no_op",
        ],
    );
    exactly(
        "placed_feature Surface {\n    |\n}",
        &["feature", "placement"],
    );
    exactly(
        "item Gem {\n    base: Item::Emerald,\n    |\n}",
        &[
            "name",
            "lore",
            "rarity",
            "glint",
            "unbreakable",
            "max_stack",
            "enchantments",
            "custom_model_data",
            "book",
        ],
    );
    exactly(
        "recipe R {\n    shaped: [\"A\"],\n    |\n}",
        &["key", "result", "count"],
    );
    exactly(
        "recipe R {\n    smelting: Item::Sand,\n    |\n}",
        &["result", "count", "experience", "time"],
    );
    exactly(
        "recipe R {\n    |\n}",
        &[
            "shaped",
            "shapeless",
            "smelting",
            "blasting",
            "smoking",
            "campfire",
            "stonecutting",
            "smithing",
            "smithing_trim",
            "result",
            "count",
        ],
    );
    exactly(
        "recipe R {\n    smithing_trim: Item::SentryArmorTrimSmithingTemplate,\n    |\n}",
        &["trim_pattern", "base", "addition"],
    );
    includes_excludes(
        "item Notice { name: | }",
        &["Text::translate"],
        &["Item::Diamond"],
    );
    includes_excludes(
        "fn f() { chat!(|); }",
        &["Text::translate", "Text::hover", "Text::suggest"],
        &[],
    );
    includes_excludes(
        "fn f() { title!(|); }",
        &["Text::translate"],
        &["Text::hover", "Text::suggest"],
    );
    includes_excludes(
        "advancement Find { criteria: { collect: { inventory_item: Item::Paper } } }\nfn f() { if advancement_done(|) {} }",
        &["Find"],
        &["MAX"],
    );
    exactly(
        "advancement Find {\n    |\n}",
        &["criteria", "display", "require_any", "experience"],
    );
    exactly(
        "dimension Workshop {\n    |\n}",
        &[
            "layers",
            "noise_settings",
            "biome",
            "biome_source",
            "biomes",
            "type",
            "structures",
        ],
    );
    exactly(
        "dimension Workshop { noise_settings: \"minecraft:overworld\", biomes: [Biome::Plains],\n    |\n}",
        &["biome_scale", "type"],
    );
    includes_excludes(
        "dimension_type Deep {}\ndimension Workshop { type: | }",
        &["Deep", "DimensionType::Overworld"],
        &["Dimension::Overworld"],
    );
    includes_excludes(
        "player state Live { score: number }\nplayer state Saved { valid: bool, score: number }\ncheckpoint Snapshot { valid: Saved.valid, fields: [{ live: Live.score, saved: Saved.score }] }\n#[player] fn f() { restore_checkpoint(|); }",
        &["Snapshot"],
        &["Live", "Saved"],
    );
    includes_excludes(
        "dimension Workshop { layers: [{ block: Block::Bedrock, height: 1 }] }\nfn f() { in_dimension(|, setup); }",
        &["Workshop", "Dimension::Overworld"],
        &["Biome::Plains"],
    );
    exactly(
        "biome Garden {\n    |\n}",
        &[
            "precipitation",
            "temperature",
            "downfall",
            "fog_color",
            "water_color",
            "water_fog_color",
            "sky_color",
            "grass_color",
            "foliage_color",
            "spawners",
            "spawn_costs",
            "features",
        ],
    );
    includes_excludes(
        "biome Garden {}\ndimension Greenhouse { biome: | }",
        &["Garden", "Biome::Plains"],
        &["Greenhouse"],
    );
    includes_excludes(
        "biome Garden {}\nfn check() { if biome_is(|, pos(0, 80, 0)) {} }",
        &["Garden", "Biome::Plains"],
        &["Dimension::Overworld"],
    );
    exactly(
        "inventory_transfer Rations {\n    |\n}",
        &["from", "to", "amount", "success", "rejected"],
    );
    includes_excludes(
        "inventory_transfer Rations { from: 0, to: 1 }\nfn move() { transfer_inventory(|); }",
        &["Rations"],
        &["Block::Stone"],
    );
    exactly(
        "hud Status {\n    |\n}",
        &["ticks", "every", "actionbar", "bossbar"],
    );
    includes_excludes(
        "hud Status { ticks: 100, actionbar: \"Ready\" }\nfn show() { start_hud(|); }",
        &["Status"],
        &["Block::Stone"],
    );
    exactly(
        "combat Training {\n    |\n}",
        &[
            "health",
            "max",
            "shield",
            "resistance",
            "immunity",
            "critical",
            "hit",
            "defeat",
            "healed",
        ],
    );
    exactly(
        "ability CastSpark {\n    |\n}",
        &["ticks", "projectile", "cost", "cooldown", "when", "start"],
    );
    includes_excludes(
        "projectile Spark {}\nability Cast { projectile: | }",
        &["Spark"],
        &["Cast"],
    );
    exactly(
        "progression Trail {\n    |\n}",
        &["progress", "objectives", "branches"],
    );
    exactly(
        "session TrailRun {\n    |\n}",
        &["progression", "ticks", "every"],
    );
    includes_excludes(
        "progression Trail {}\n session Run { progression: | }",
        &["Trail"],
        &["Run"],
    );
    exactly(
        "projectile Spark {\n    |\n}",
        &["ticks", "speed", "radius", "target", "damage", "hit"],
    );
    includes_excludes(
        "projectile Spark { ticks: 20 }\nfn fire() { launch_projectile(|); }",
        &["Spark"],
        &["Entity::Zombie"],
    );
    exactly(
        "menu Supplies {\n    |\n}",
        &[
            "title",
            "ticks",
            "page_size",
            "choices",
            "views",
            "start",
            "while_open",
        ],
    );
    includes_excludes(
        "menu Supplies { title: \"Camp\", ticks: 100, choices: [{ label: \"Leave\", action: leave }] }\nfn f() { open_menu(|); }",
        &["Supplies"],
        &["Block::Stone"],
    );
    exactly("structure_template Marker {\n    |\n}", &["source"]);
    includes_excludes(
        "structure_template Marker { source: \"structures/marker.nbt\" }\nfn place() { place_template(|, pos(0, 80, 0)); }",
        &["Marker"],
        &["Block::Stone"],
    );
    exactly("dialog Notice {\n    |\n}", &["title", "body", "button"]);
    includes_excludes(
        "dialog Notice { title: \"Hello\" }\nfn f() { show_dialog(|); }",
        &["Notice"],
        &["MAX"],
    );
    exactly("tag T: Block {\n    |\n}", &["values", "replace"]);
    includes_excludes(
        "tag T: Block { values: [|] }",
        &["Block::Stone"],
        &["Item::Diamond"],
    );
    includes_excludes(
        "recipe R { shaped: [\"A\"], key: { A: | } }",
        &["Item::Stick"],
        &["Block::Stone"],
    );
    exactly(
        "item Gem { rarity: | }",
        &[
            "Rarity::Common",
            "Rarity::Uncommon",
            "Rarity::Rare",
            "Rarity::Epic",
        ],
    );
}

#[test]
fn comments_and_strings_offer_nothing_except_interpolation() {
    exactly("// a comment |", &[]);
    exactly("fn f() {\n    chat!(\"plain |\");\n}", &[]);
    includes_excludes(
        "fn f() {\n    chat!(\"{|\");\n}",
        &["player", "gold", "Hero.energy"],
        &["let"],
    );
}

#[test]
fn job_steps_complete_to_functions_with_an_index_parameter() {
    includes_excludes(
        "fn walk(i: number) {}
fn no_index() {}
fn wrong_index(f: float) {}
job Route { ticks: 20, count: 2, step: | }",
        &["walk"],
        &["no_index", "wrong_index"],
    );
}
