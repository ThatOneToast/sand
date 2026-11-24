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
