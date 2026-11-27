mod support;
use support::*;

#[test]
fn state_migrations_emit_ordered_steps_for_each_scope() {
    let source = r#"
#![pack(namespace = "upgrades", minecraft = "26.2")]
#[migrate(bank_v2)]
#[migrate(bank_v3)]
global state Bank { old_balance: number, balance: number }
#[migrate(journal_v2)]
player state Journal { old_name: string, name: string }
#[migrate(creature_v2)]
entity state Creature { old_level: number, level: number }
fn bank_v2() { Bank.balance = Bank.old_balance * 2; }
fn bank_v3() { Bank.balance += 1; }
fn journal_v2() { Journal.name = Journal.old_name; }
fn creature_v2() { Creature.level = Creature.old_level + 10; }
fn check() {
    if has_state(Creature) { add_tag("current"); }
}
fn scan() { for creature in entities(with = Creature) { check(); } }
"#;
    let analysis = analyze_one(source);
    let pack = compile(&analysis);
    let program = analysis.program.as_ref().unwrap();
    let states = &program.modules[0].states;
    assert_eq!(
        states
            .iter()
            .filter(|state| !state.migrations.is_empty())
            .map(|state| state.revision)
            .collect::<Vec<_>>(),
        [3, 2, 2]
    );
    assert_eq!(
        states[0]
            .migrations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["upgrades:bank_v2", "upgrades:bank_v3"]
    );
    let player_init = function(&pack, "upgrades/function/__sand_lifecycle_init");
    assert!(player_init.contains("unless score @s"));
    assert!(player_init.contains("function upgrades:__sand_program/state_migrate/"));
    let migration = function(&pack, "upgrades/function/journal_v2");
    assert!(!migration.contains("__sand_lifecycle_init"));
    let load = function(&pack, "upgrades/function/__sand_lifecycle_load");
    assert!(
        load.find("scoreboard players set #c2").unwrap()
            < load
                .find("function upgrades:__sand_program/state_migrate/")
                .unwrap()
    );
    let tick = function(&pack, "upgrades/function/__sand_lifecycle_tick");
    assert!(
        tick.lines()
            .any(|line| line.contains("as @e[scores=") && line.ends_with("/attach"))
    );
    let encoded = serde_json::to_vec(program).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn migration_callbacks_require_compatible_synchronous_transforms() {
    for (body, expected) in [
        (
            "fn upgrade(input: number) {}",
            "no parameters or return value",
        ),
        (
            "fn upgrade() -> number { return 1; }",
            "no parameters or return value",
        ),
        (
            "fn upgrade() { wait(1, server = true); }",
            "transform values synchronously",
        ),
        (
            "fn helper() {} fn upgrade() { helper(); }",
            "transform values synchronously",
        ),
        (
            "fn upgrade() { add_tag(\"legacy\"); }",
            "callee requires an entity executor",
        ),
    ] {
        let source = format!(
            "#![pack(namespace = \"upgrades\")] #[migrate(upgrade)] global state Bank {{ value: number }} {body}"
        );
        let analysis = analyze_one(&source);
        let Err(diagnostics) = analysis.compile() else {
            panic!("invalid migration compiled");
        };
        assert!(
            diagnostics.iter().any(|d| d.message.contains(expected)),
            "{source}: {diagnostics:?}"
        );
    }
}
