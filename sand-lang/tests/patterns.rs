mod support;
use support::*;

#[test]
fn value_matches_support_owned_values_recursion_and_contextual_types() {
    compile(&analyze_one(
        r#"#![pack(namespace = "patterns", minecraft = "26.2")]
enum Mode { Ready, Done }
struct Point { x: number, title: string }
fn recurse(n: number) -> string {
    return match n { 0 => "done", _ => recurse(n - 1) + "!", };
}
fn run(n: number, mode: Mode) {
    let label = match mode { Mode::Ready => "ready", Mode::Done => recurse(n), };
    let points: Vec<Point> = match n { 0 => [], _ => [Point { x: n, title: label }], };
    let flags = match n { 0 => n == 0, _ => false, };
    let values: [number; 2] = match flags { true => [1, 2], false => [3, 4], };
    let nested = match n { 0 => match mode { Mode::Ready => 1, Mode::Done => 2 }, _ => 3 };
}
"#,
    ));
}

#[test]
fn large_matches_do_not_exceed_operation_depth() {
    let arms = (0..100)
        .map(|n| format!("{n} => {{ S.n = {n}; }},"))
        .collect::<String>();
    let values = (0..100).map(|n| format!("{n} => {n},")).collect::<String>();
    compile(&analyze_one(&format!(
        "#![pack(namespace = \"patterns\")] global state S {{ n: number }} fn run(n: number) {{ match n {{ {arms} _ => {{ S.n = -1; }} }} let result = match n {{ {values} _ => -1 }}; }}"
    )));
}

#[test]
fn value_matches_reject_incomplete_overlapping_and_incompatible_arms() {
    for source in [
        "fn run(n: number) { let a = match n { 0 => 2 }; }",
        "fn run(n: number) { let a = match n { 0 => 1, 0 => 2, _ => 3 }; }",
        "fn run(n: number) { let a = match n { _ => 1, 0 => 2 }; }",
        "fn run(n: number) { let a = match n { 0 => 1, _ => true }; }",
        "fn run() { let a = match true { true => 1 }; }",
    ] {
        let analysis = analyze_one(&format!("#![pack(namespace = \"patterns\")] {source}"));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn destructuring_supports_nested_owned_values_and_shadowing() {
    compile(&analyze_one(
        r#"#![pack(namespace = "patterns", minecraft = "26.2")]
struct Inner { name: string, values: Vec<number> }
struct Pair { inner: Inner, coords: [number; 2], ignored: bool }
fn make() -> Pair { return Pair { inner: Inner { name: "a", values: [1, 2] }, coords: [3, 4], ignored: true }; }
fn run() {
    let Pair { inner: Inner { name, values: mut copied }, coords: [x, mut y], .. } = make();
    copied.push(9); y += x;
    let [x, _]: [number; 2] = [x, y];
    let _ = copied.pop_or(0);
    let []: [number; 0] = [];
    let Inner { name: renamed, .. } = Inner { name, values: copied };
}
"#,
    ));
}

#[test]
fn destructuring_rejects_fallible_incomplete_duplicate_and_mismatched_patterns() {
    for source in [
        "let [a] = [1, 2];",
        "let Pair { x } = Pair { x: 1, y: 2 };",
        "let Pair { x, y: x } = Pair { x: 1, y: 2 };",
        "let Pair { x, x: other, .. } = Pair { x: 1, y: 2 };",
        "let Pair { missing, .. } = Pair { x: 1, y: 2 };",
        "let Pair { x, y } = [1, 2];",
        "let v: Vec<number> = [1, 2]; let [a, b] = v;",
        "let [a, b] = [1, 2]; a = 3;",
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"patterns\")] struct Pair {{ x: number, y: number }} fn run() {{ {source} }}"
        ));
        assert!(analysis.has_errors(), "accepted {source}");
    }
}

#[test]
fn native_patterns_fixture_compiles_and_formats_idempotently() {
    let source = include_str!("fixtures/patterns.sand");
    compile(&analyze_one(source));
    let formatted = sand_lang::format::format(source).unwrap();
    assert_eq!(sand_lang::format::format(&formatted).unwrap(), formatted);
    compile(&analyze_one(&formatted));
}
