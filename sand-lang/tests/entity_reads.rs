mod support;

use support::*;

#[test]
fn inventory_counts_use_the_native_query_and_match_custom_identity() {
    let source = r#"
#![pack(namespace = "inspect", minecraft = "26.2")]
item Ration { base: Item::Bread, name: "Ration" }
player state Report { total: number, authored: number }
fn inspect() {
    Report.total = inventory_count(Item::Bread);
    Report.authored = inventory_count(Ration);
}
"#;
    let pack = compile(&analyze_one(source));
    let source = function(&pack, "inspect/function/inspect");
    assert!(source.contains("run clear @s minecraft:bread 0"));
    assert!(source.lines().any(|line| {
        line.contains("run clear @s minecraft:bread[")
            && line.contains("minecraft:custom_data")
            && line.contains("inspect:ration")
            && line.ends_with(" 0")
    }));
}

#[test]
fn entity_paths_use_canonical_read_commands() {
    let source = r#"
#![pack(namespace = "inspect", minecraft = "26.2")]
entity state Report { current: number }
fn inspect() {
    attach_state(Report);
    Report.current = entity_data_number("Pos[0]");
}
"#;
    let pack = compile(&analyze_one(source));
    let source = function(&pack, "inspect/function/inspect");
    assert!(source.contains("run data get entity @s Pos[0]"));

    let invalid = r#"
#![pack(namespace = "inspect", minecraft = "26.2")]
fn inspect() { let value = entity_data_number("Pos["); broadcast!("{value}"); }
"#;
    let diagnostics = analyze_one(invalid).compile().unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|error| { error.message.contains("path") && error.message.contains("Pos[") }),
        "{diagnostics:?}"
    );
}

#[test]
fn inventory_and_entity_queries_reject_a_server_context() {
    for expr in [
        "inventory_count(Item::Paper)",
        "entity_data_number(\"Health\")",
    ] {
        let source = format!(
            "#![pack(namespace = \"inspect\")]\n#[load] fn inspect() {{ let value = {expr}; broadcast!(\"{{value}}\"); }}"
        );
        assert!(analyze_one(&source).has_errors(), "{expr}");
    }
}
