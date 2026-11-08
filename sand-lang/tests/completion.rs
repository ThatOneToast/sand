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
            "flow",
            "const",
            "item",
            "recipe shaped",
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
        &["number", "float", "bool", "Phase", "tools"],
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
        &["tick", "load", "on", "tag", "player"],
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
        ],
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
        &["number", "float", "bool", "Phase", "tools"],
    );
    exactly(
        "player state S { v: number = | }",
        &["MAX", "Phase", "tools", "crate", "self", "super"],
    );
    exactly("player state S {\n    |\n}", &["#[clamp]"]);
    exactly(
        "fn f(n: |) {}",
        &["number", "float", "bool", "Phase", "tools"],
    );
    exactly(
        "fn f() -> |",
        &["number", "float", "bool", "Phase", "tools"],
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
            "get_attribute",
            "count_entities",
        ],
        &["act", "let", "give_item", "entities"],
    );
    exactly("fn f() {\n    let |\n}", &[]);
    exactly("fn f() {\n    for p |\n}", &["in"]);
    exactly(
        "fn f() {\n    for p in |\n}",
        &["players()", "entities(..)"],
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
            "result",
            "count",
        ],
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
