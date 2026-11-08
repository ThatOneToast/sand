//! Entity State: attach, detach, conditions, queries, and context rules.

mod support;

use sand_lang::{Context, DefInfo, FileId, ide};
use support::*;

const PACK: &str = "#![pack(namespace = \"demo\", minecraft = \"26.2\")]\n";

fn pack(body: &str) -> String {
    format!("{PACK}{body}")
}

const MANA: &str = r#"
entity state Mana {
    #[clamp(0..=100)]
    current: number = 10,
}

fn regen() {
    Mana.current += 1;
}

#[tick]
fn tick() {
    for mob in entities(type = Entity::Zombie) {
        if has_state(Mana) {
            regen();
        } else {
            attach_state(Mana);
        }
    }
    for carrier in entities(with = Mana, limit = 1) {
        detach_state(Mana);
    }
}
"#;

#[test]
fn entity_state_uses_the_same_commands_as_rust() {
    let analysis = analyze_one(&pack(MANA));
    let pack = compile(&analysis);
    let all: Vec<String> = pack
        .resources
        .values()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    let joined = all.join("\n");
    let presence = sand_commands_objective("demo:mana.presence");
    let current = sand_commands_objective("demo:mana.current");
    // Attach keeps existing values, defaults the rest, then marks presence.
    assert!(joined.contains(&format!(
        "execute unless score @s {current} matches -2147483648.. run scoreboard players set @s {current} 10"
    )), "{joined}");
    assert!(joined.contains(&format!(
        "execute unless score @s {presence} matches 1.. run scoreboard players set @s {presence} 1"
    )), "{joined}");
    // Detach clears fields, dirty markers, and presence.
    assert!(joined.contains(&format!("scoreboard players reset @s {current}")));
    assert!(joined.contains(&format!("scoreboard players reset @s {presence}")));
    assert!(
        joined.contains(&format!(
            "execute if score @s {presence} matches 1..1 run return run function"
        )),
        "{joined}"
    );
    assert!(
        joined.contains(&format!("@e[scores={{{presence}=1}},limit=1]"))
            || joined.contains(&format!("scores={{{presence}=1}}")),
        "{joined}"
    );
    // Lifecycle declares the objectives through the Rust entity scope.
    let load = function(&pack, "demo/function/__sand_lifecycle_load");
    assert!(
        load.contains(&format!("scoreboard objectives add {presence} dummy")),
        "{load}"
    );
    assert!(
        load.contains(&format!(
            "scoreboard objectives add {} dummy",
            sand_commands_objective("demo:mana.current.dirty")
        )),
        "{load}"
    );

    let def = analysis.defs.iter().find(|d| d.name == "regen").unwrap();
    let DefInfo::Function { context, .. } = &def.info else {
        panic!()
    };
    assert_eq!(*context, Context::Entity);
}

fn sand_commands_objective(logical: &str) -> String {
    sand::__private::resolve_state_objective(logical)
}

#[test]
fn entity_state_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
entity state Mana { current: number }
player state Wallet { coins: number }
fn spend() { Mana.current -= 1; }
#[tick]
fn tick() {
    spend();
    for p in players() { attach_state(Wallet); }
    for e in entities(with = Wallet) {}
    for p in players() { if has_state(Wallet) {} }
}
flow F on Mana.current { 0 -> 1; }
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "`tick` runs from #[tick] on the server, but it calls `spend`, which runs as an entity",
        "`Wallet` is not an entity state",
        "flows run over player or global State, not entity State",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn editor_offers_entity_states_and_predicates_only_where_they_fit() {
    let text = pack(
        "entity state Mana { v: number }\nplayer state Wallet { c: number }\nfn f() {\n    attach_state();\n    if  {}\n    let x = 1;\n    \n}\n",
    );
    let analysis = analyze_one(&text);
    let labels = |offset: usize| -> Vec<String> {
        ide::completions(&analysis, FileId(0), offset)
            .into_iter()
            .map(|c| c.label)
            .collect()
    };
    assert_eq!(
        labels(text.find("attach_state()").unwrap() + 13),
        vec!["Mana"]
    );
    let condition = labels(text.find("if  {").unwrap() + 3);
    assert!(
        condition.contains(&"has_state".to_string()),
        "{condition:?}"
    );
    let value = labels(text.find("let x = 1").unwrap() + 8);
    assert!(
        !value.contains(&"has_state".to_string()),
        "predicates are not values: {value:?}"
    );
    let statement = labels(text.find("1;\n    \n").unwrap() + 7);
    assert!(
        statement.contains(&"attach_state".to_string())
            && !statement.contains(&"has_state".to_string()),
        "{statement:?}"
    );
}
