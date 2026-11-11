//! Entities: queries, entity loops, entity actions, and the entity context.

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

#[test]
fn entity_loops_select_with_typed_filters_and_run_at_each_entity() {
    let analysis = analyze_one(&pack(
        r#"
global state Arena { zombies: number }

/// Clear zombies near each player and count what is left.
#[tick]
fn sweep() {
    for player in players() {
        for zombie in entities(type = Entity::Zombie, distance = ..=8, limit = 3, sort = Sort::Nearest) {
            damage(4.0);
            add_tag("hit");
        }
    }
    Arena.zombies = count_entities(type = Entity::Zombie, tag = "hit");
    for marked in entities(tag = "hit") {
        remove_tag("hit");
        if Arena.zombies > 10 {
            kill();
        }
    }
}
"#,
    ));
    let pack = compile(&analysis);
    let sweep = function(&pack, "demo/function/sweep");
    assert!(
        sweep.contains("execute store result score #sand_demo_arena")
            && sweep.contains("if entity @e[type=minecraft:zombie,tag=hit]"),
        "{sweep}"
    );
    assert!(
        sweep.contains("execute as @e[tag=hit] at @s run function demo:__sand_program/"),
        "{sweep}"
    );
    let all: Vec<String> = pack
        .resources
        .values()
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .collect();
    assert!(
        all.iter().any(|f| f.contains(
            "execute as @e[type=minecraft:zombie,distance=..8,sort=nearest,limit=3] at @s run function"
        )),
        "{all:#?}"
    );
    assert!(
        all.iter()
            .any(|f| f.contains("damage @s 4") && f.contains("tag @s add hit"))
    );
    assert!(all.iter().any(|f| f.contains("tag @s remove hit")));
    assert!(all.iter().any(|f| f.contains("kill @s")));
    assert_eq!(context_of(&analysis, "sweep"), Context::Server);
}

#[test]
fn functions_that_act_on_the_executor_run_as_entities() {
    let analysis = analyze_one(&pack(
        r#"
fn smite() {
    damage(10.0);
    summon(Entity::LightningBolt);
}
fn punish() { smite(); }
#[tick]
fn tick() {
    for target in entities(type = Entity::Zombie) { punish(); }
    for player in players() { punish(); }
}
"#,
    ));
    assert!(!analysis.has_errors(), "{:#?}", errors(&analysis));
    assert_eq!(context_of(&analysis, "smite"), Context::Entity);
    assert_eq!(context_of(&analysis, "punish"), Context::Entity);
    let hover = ide::hover(
        &analysis,
        FileId(0),
        text_offset(&analysis, "fn punish") + 3,
    )
    .unwrap();
    assert!(
        hover.markdown.contains("Runs as an **entity**"),
        "{}",
        hover.markdown
    );
    let program = analysis.program.as_ref().unwrap();
    let smite = program.modules[0]
        .functions
        .iter()
        .find(|f| f.id.to_string() == "demo:smite")
        .unwrap();
    assert_eq!(
        smite.context,
        sand::advanced::compiler::ExecutionContext::Entity
    );
    let pack = compile(&analysis);
    assert!(
        function(&pack, "demo/function/smite").contains("summon minecraft:lightning_bolt ~ ~ ~")
    );
}

fn text_offset(analysis: &sand_lang::Analysis, needle: &str) -> usize {
    analysis.package.files[0].text.find(needle).unwrap()
}

#[test]
fn entity_context_mistakes_are_explained() {
    let analysis = analyze_one(&pack(
        r#"
player state Wallet { coins: number }
fn reward() { Wallet.coins += 1; }
fn zap() { kill(); }
#[tick]
fn tick() {
    for mob in entities(type = Entity::Cow) {
        Wallet.coins += 1;
        reward();
        chat!("moo");
    }
    zap();
    let n = entities(tag = "x");
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "this writes player state `Wallet.coins`, but `entities(..)` loops run as entities that may not be players",
        "`reward` needs a player, but `entities(..)` loops run as entities that may not be players",
        "this uses `chat!`, which shows text to the executing player, but `entities(..)` loops run as entities that may not be players",
        "`tick` runs from #[tick] on the server, but it calls `zap`, which runs as an entity",
        "`entities(..)` is the source of a `for` loop",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn entity_queries_check_their_filters() {
    let analysis = analyze_one(&pack(
        r#"
#[tick]
fn tick() {
    for a in entities(type = Item::Diamond) {}
    for b in entities(tag = "no spaces") {}
    for c in entities(sort = Sort::Closest) {}
    for d in entities(distance = 5) {}
    for e in players(type = Entity::Player) {}
}
"#,
    ));
    let errors = errors(&analysis);
    for expected in [
        "expected an `Entity`, found an `Item`",
        "`no spaces` is not a valid tag; use letters, digits, and `_ . + -`",
        "no sort order named `Closest` in `Sort`",
        "expected a distance range",
        "`players()` takes no arguments; use `entities(type = Entity::Player, ..)` to filter",
    ] {
        assert!(
            errors.iter().any(|e| e == expected),
            "missing {expected:?} in {errors:#?}"
        );
    }
}

#[test]
fn teleports_move_the_executor() {
    let analysis = analyze_one(&pack(
        r#"
#[tick]
fn tick() {
    for player in players() {
        teleport(0.5, 64.0, -2.0);
        teleport_by(0.0, 5.0, 0.0);
        teleport_to(type = Entity::Villager, tag = "guide");
    }
}
fn lift(height: number) {
    teleport_by(0.0, height, 0.0);
}
"#,
    ));
    let errors_ = errors(&analysis);
    assert_eq!(errors_, vec!["coordinates must be constants"]);
    let ok = analyze_one(&pack(
        "fn go() {\n    teleport(0.5, 64.0, -2.0);\n    teleport_by(0.0, 5.0, 0.0);\n    teleport_to(type = Entity::Villager, tag = \"guide\");\n}\n",
    ));
    let pack = compile(&ok);
    let go = function(&pack, "demo/function/go");
    assert!(go.contains("tp @s 0.5 64 -2"), "{go}");
    assert!(go.contains("tp @s ~0 ~5 ~0"), "{go}");
    assert!(
        go.contains("tp @s @e[type=minecraft:villager,tag=guide,sort=nearest,limit=1]"),
        "{go}"
    );
    assert_eq!(context_of(&ok, "go"), Context::Entity);
}
