mod support;

use sand_lang::{FileId, ide};
use support::*;

#[test]
fn match_bindings_fixture_compiles() {
    compile(&analyze_one(include_str!("fixtures/match_bindings.sand")));
}

#[test]
fn arm_bindings_are_visible_in_guards_and_values_only() {
    let source = r#"#![pack(namespace = "bindings")]
fn value(input: [number; 2]) -> number {
    return match input { let [left, right] if left > 0 => left + right, _ => 0 };
}
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let guard = source.find("left >").unwrap();
    let value = source.find("left +").unwrap();
    let binding = source.find("left,").unwrap();
    let target = ide::definition(&analysis, FileId(0), guard).unwrap();
    assert_eq!(target.1.start, binding);
    assert_eq!(
        ide::definition(&analysis, FileId(0), value)
            .unwrap()
            .1
            .start,
        binding
    );
}

#[test]
fn binding_patterns_validate_shape_scope_and_exhaustiveness() {
    for (source, message) in [
        (
            "fn f(n: number) { match n { let bound if bound > 0 => {} } }",
            "non-exhaustive",
        ),
        (
            "fn f(n: number) -> number { return match n { let whole => whole, _ => 0 }; }",
            "after an unguarded binding",
        ),
        (
            "fn f(n: [number; 2]) -> number { return match n { let [x, x] => x }; }",
            "duplicate binding",
        ),
        (
            "fn f(n: [number; 2]) -> number { return match n { let [x] => x }; }",
            "expected 2 array patterns",
        ),
        (
            "fn f(n: number) -> number { return match n { let x if x > 0 => x, _ => x }; }",
            "cannot find `x` in this scope",
        ),
    ] {
        let analysis = analyze_one(&format!(r#"#![pack(namespace = "bindings")] {source}"#));
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains(message)),
            "{:#?}",
            errors(&analysis)
        );
    }
}

#[test]
fn generic_arms_infer_the_bound_type_per_specialization() {
    let analysis = analyze_one(
        r#"#![pack(namespace = "bindings")]
struct Packet<T> { value: T }
fn unpack<T>(input: Packet<T>) -> T {
    let value = match input { let Packet { value } => value };
    return value;
}
fn use_bindings() {
    let label: string = unpack(Packet { value: "sample" });
    let count: number = unpack(Packet { value: 1 });
    broadcast!("{label}: {count}");
}
"#,
    );
    compile(&analysis);
}
