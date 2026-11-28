mod support;
use support::*;

#[test]
fn dimension_scopes_and_clone_emit_native_commands() {
    let analysis = analyze_one(include_str!("fixtures/world_ops.sand"));
    let pack = compile(&analysis);
    let run = function(&pack, "language_probe/function/run");
    assert!(run.contains("execute in minecraft:the_nether run function "));
    assert!(run.contains("clone 1 80 1 3 80 1 5 80 1 masked normal"));
    assert!(run.contains("clone 1 80 1 3 80 1 10 80 1 filtered minecraft:gold_block move"));
    let output = pack.resources.values().any(|file| {
        String::from_utf8_lossy(file).contains("execute if dimension minecraft:the_end")
    });
    assert!(output);
    assert!(pack.resources.values().any(|file| {
        String::from_utf8_lossy(file).contains("execute if biome 8 80 8 minecraft:plains")
    }));
}

#[test]
fn dimension_calls_preserve_player_context_and_validate_the_target() {
    let src = r#"#![pack(namespace = "world_probe")]
#[player] fn target() { give_item(Item::Bread); }
fn calling() { in_dimension(Dimension::TheEnd, target); }
"#;
    let analysis = analyze_one(src);
    compile(&analysis);
    let program = serde_json::to_value(analysis.program.unwrap()).unwrap();
    let calling = program["modules"][0]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|function| function["id"] == "world_probe:calling")
        .unwrap();
    assert_eq!(calling["context"], "player");
    for (body, expected) in [
        (
            "fn run() { if biome_is(Biome::UnknownBiome, pos(0, 0, 0)) { return; } }",
            "no biome named",
        ),
        (
            "fn run() { in_dimension(Dimension::End, missing); }",
            "no dimension named",
        ),
        (
            "fn run() { in_dimension(Dimension::TheEnd, missing); }",
            "expected a parameterless function name",
        ),
        (
            "fn run() { clone_blocks(pos(0, 0, 0), pos(1, 1, 1), pos(2, 2, 2), masked = true, filter = Block::Stone); }",
            "either a clone filter or a masked clone",
        ),
    ] {
        let source = format!("#![pack(namespace = \"world_probe\")]\n{body}");
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|error| error.contains(expected)),
            "{diagnostics:?}"
        );
    }
}

#[test]
fn biome_identifiers_follow_the_package_target_at_both_boundaries() {
    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.3")]
fn run() { if biome_is(Biome::DappledForest, pos(0, 80, 0)) { return; } }
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let mut program = analysis.program.unwrap();
    program.target.minecraft = "26.2".into();
    let backend = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(backend.iter().any(|error| {
        error
            .message
            .contains("no biome `minecraft:dappled_forest`")
    }));
    let analysis = analyze_one(&source.replace("26.3", "26.2"));
    assert!(
        errors(&analysis)
            .iter()
            .any(|error| error.contains("not available in Minecraft 26.2"))
    );
}

#[test]
fn block_data_uses_checked_paths_and_integer_operands() {
    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
fn run(amount: number) {
    let old = block_data_number(offset(0, 0, 0), "Items[0].count");
    set_block_data(offset(0, 0, 0), "Items[0].count", old + amount);
    remove_block_data(offset(0, 0, 0), "Items[0]");
}
"#;
    let compiled = compile(&analyze_one(source));
    let run = function(&compiled, "world_probe/function/run");
    assert!(run.contains("data get block ~ ~ ~ Items[0].count"));
    assert!(run.contains("store result block ~ ~ ~ Items[0].count int 1"));
    assert!(run.contains("data remove block ~ ~ ~ Items[0]"));
    let invalid = analyze_one(&source.replace("Items[0].count", "Items[0] extra"));
    let diagnostics = invalid.compile().unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|error| error.message.contains("path")),
        "{diagnostics:?}"
    );
}

#[test]
fn block_data_supports_typed_values_and_copies_through_checked_paths() {
    let compiled = compile(&analyze_one(include_str!("fixtures/world_ops.sand")));
    let run = function(&compiled, "language_probe/function/run");
    assert!(run.contains(
        r#"data modify block 14 80 1 Items set value [{Slot:0b,id:"minecraft:bread",count:3}]"#
    ));
    assert!(run.contains("data modify block 15 80 1 Items set from block 14 80 1 Items"));
    assert!(run.contains(r#"data modify block 15 80 1 CustomName set value "Supply cache""#));
}
