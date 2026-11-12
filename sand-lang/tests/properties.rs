//! Entity properties: `Player.health`, `Player.max_health = 40.0;`, and loop
//! handles such as `zombie.step_height`.

mod support;

use sand_lang::{FileId, ide};
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

/// The portable actions of one function, as JSON.
fn actions(analysis: &sand_lang::Analysis, name: &str) -> Vec<serde_json::Value> {
    let id = format!("demo:{name}");
    let function = analysis.program.as_ref().expect("lowered").modules[0]
        .functions
        .iter()
        .find(|f| f.id.to_string() == id)
        .unwrap();
    serde_json::to_value(&function.body)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|op| op["action"].clone())
        .collect()
}

#[test]
fn player_data_and_attributes_read_as_typed_values() {
    let analysis = analyze_one(&pack(
        r#"
player state Stats { hp: float, hunger: number, grounded: bool, step: float }
fn sample() {
    Stats.hp = Player.health;
    Stats.hunger = Player.food;
    Stats.grounded = Player.on_ground;
    Stats.step = Player.step_height;
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let ops = actions(&analysis, "sample");
    assert_eq!(ops[0]["op"], "data_get");
    assert_eq!(ops[0]["value"], "health");
    assert_eq!(ops[0]["scale"], 1000.0);
    assert_eq!(ops[1]["value"], "food");
    assert_eq!(ops[1]["scale"], 1.0);
    assert_eq!(ops[2]["value"], "on_ground");
    assert_eq!(ops[3]["op"], "attribute_get");
    assert_eq!(ops[3]["attribute"], "minecraft:step_height");
    // Reading player data makes the function need a player.
    let pack = compile(&analysis);
    let sample = function(&pack, "demo/function/sample");
    assert!(
        sample.contains("run data get entity @s Health 1000"),
        "{sample}"
    );
    assert!(sample.contains("run data get entity @s foodLevel 1"), "{sample}");
}

#[test]
fn attributes_assign_constants_expressions_and_compound_updates() {
    let analysis = analyze_one(&pack(
        r#"
fn grow(bonus: float) {
    Player.max_health = 40.0;
    Player.scale = bonus * 2.0;
    Player.movement_speed *= 1.5;
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let ops = actions(&analysis, "grow");
    let sets: Vec<_> = ops.iter().filter(|o| o["op"] == "attribute_set").collect();
    assert_eq!(sets.len(), 3, "{ops:#?}");
    assert_eq!(sets[0]["value"], 40.0);
    assert_eq!(sets[1]["value"]["scale"], 1000.0);
    // The compound update reads the current base value first.
    assert!(ops.iter().any(|o| o["op"] == "attribute_get"
        && o["attribute"] == "minecraft:movement_speed"));
}

#[test]
fn loop_bindings_are_handles_for_the_entity_they_run_as() {
    let analysis = analyze_one(&pack(
        r#"
global state Totals { grounded: number }
fn shrink_zombies() {
    for zombie in entities(type = Entity::Zombie) {
        zombie.scale = 0.5;
        if zombie.on_ground {
            Totals.grounded += 1;
        }
    }
}
"#,
    ));
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));
    let pack = compile(&analysis);
    let helpers: String = pack
        .resources
        .iter()
        .filter(|(path, _)| path.ends_with(".mcfunction"))
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        helpers.contains("attribute @s minecraft:scale base set 0.5"),
        "{helpers}"
    );
    assert!(helpers.contains("data get entity @s OnGround 1"), "{helpers}");
}

#[test]
fn property_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
fn mistakes() {
    Player.health = 20.0;
    let _a = Player.max_helth;
    for zombie in entities(type = Entity::Zombie) {
        let _b = zombie.food;
        let _c = Player.health;
    }
    for player in players() {
        for zombie in entities(type = Entity::Zombie) {
            let _d = player.health;
        }
    }
    let _e: number = Player.health;
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`Player.health` is read-only",
        "`Player` has no property `max_helth`",
        "only players have `food`, but `entities(..)` may match other entities",
        "`player` is not the executing entity here",
        "mismatched types: expected `number`, found `float`",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
    assert!(
        errors.iter().any(|e| e.contains("may not be players")),
        "`Player` inside an entity loop is rejected: {errors:#?}"
    );
    let help = analysis
        .diagnostics
        .iter()
        .find(|d| d.message == "`Player` has no property `max_helth`")
        .and_then(|d| d.help.clone())
        .unwrap_or_default();
    assert!(help.contains("`max_health`"), "{help}");
}

#[test]
fn properties_complete_after_a_handle_and_explain_themselves() {
    let text = pack(
        r#"
fn f() {
    let _h = Player.health;
    for zombie in entities(type = Entity::Zombie) {
        zombie.scale = 2.0;
    }
}
"#,
    );
    let analysis = analyze_one(&text);
    assert!(errors(&analysis).is_empty(), "{:#?}", errors(&analysis));

    let hover = ide::hover(&analysis, FileId(0), text.find("health").unwrap()).unwrap();
    assert!(hover.markdown.contains("Player.health: float"), "{}", hover.markdown);
    assert!(hover.markdown.contains("read-only"), "{}", hover.markdown);
    let hover = ide::hover(&analysis, FileId(0), text.find("scale").unwrap()).unwrap();
    assert!(hover.markdown.contains("zombie.scale: float"), "{}", hover.markdown);
    assert!(hover.markdown.contains("read and write"), "{}", hover.markdown);

    // After `Player.`: data and attributes, including player-only data.
    let cursor = text.find("Player.").unwrap() + "Player.".len();
    let labels: Vec<String> = ide::completions(&analysis, FileId(0), cursor)
        .into_iter()
        .map(|c| c.label)
        .collect();
    for expected in ["health", "food", "max_health", "step_height"] {
        assert!(labels.iter().any(|l| l == expected), "{expected}: {labels:?}");
    }
    // After `zombie.`: no player-only data.
    let cursor = text.find("zombie.").unwrap() + "zombie.".len();
    let labels: Vec<String> = ide::completions(&analysis, FileId(0), cursor)
        .into_iter()
        .map(|c| c.label)
        .collect();
    assert!(labels.iter().any(|l| l == "scale"), "{labels:?}");
    assert!(!labels.iter().any(|l| l == "food"), "{labels:?}");
}
