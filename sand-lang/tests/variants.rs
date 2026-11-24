mod support;
use support::*;

#[test]
fn payload_variants_compile_through_the_owned_value_paths() {
    let source = include_str!("fixtures/variants.sand");
    let analysis = analyze_one(source);
    let compiled = compile(&analysis);
    let formatted = sand_lang::format::format(source).unwrap();
    let reformatted = compile(&analyze_one(&formatted));
    assert_eq!(compiled.resources, reformatted.resources);
}

#[test]
fn payload_variants_reject_incorrect_construction_and_matching() {
    for (source, expected) in [
        (
            "fn run() { let value = Choice::Some; }",
            "expects 1 payload values, found 0",
        ),
        (
            "fn run() { let value = Choice::Some(\"text\"); }",
            "expected a value, found text",
        ),
        (
            "fn run() { let value = Choice::Some(value = 2); }",
            "take positional values",
        ),
        (
            "fn run() { match Choice::None { Choice::Some(x, y) => {} Choice::None => {} } }",
            "expects 1 payload values, found 2",
        ),
        (
            "fn run() { match Choice::None { Choice::Some(value) if value > 0 => {} Choice::None => {} } }",
            "non-exhaustive",
        ),
        (
            "fn run() { match Choice::None { Choice::Some(_) => {} Choice::Some(_) => {} Choice::None => {} } }",
            "already covered",
        ),
        (
            "global state Saved { current: Choice = 7 }",
            "expected an enum variant constructor",
        ),
        ("const NUMBER: number = Choice::None;", "owned value"),
        ("enum Cycle { Here(Cycle) }", "layouts must be finite"),
    ] {
        let source = format!(
            "#![pack(namespace = \"variants\", minecraft = \"26.2\")] enum Choice {{ None, Some(number) }} {source}"
        );
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|message| message.contains(expected)),
            "{source}: {diagnostics:?}"
        );
    }
}
