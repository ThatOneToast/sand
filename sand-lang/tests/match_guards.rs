mod support;
use support::*;

#[test]
fn guards_can_fall_through_on_the_same_pattern_and_use_runtime_functions() {
    let analysis = analyze_one(include_str!("fixtures/match_guards.sand"));
    let pack = compile(&analysis);
    assert!(pack.resources.values().any(|body| {
        String::from_utf8_lossy(body).contains("function language_probe:test_guard")
    }));
}

#[test]
fn guards_do_not_make_a_pattern_exhaustive() {
    for body in [
        "fn choose(v: bool) -> number { return match v { true if v => 1, false => 2 }; }",
        "fn choose(v: number) { match v { _ if v == 1 => {} } }",
    ] {
        let source = format!("#![pack(namespace = \"patterns\")]\n{body}");
        let issues = errors(&analyze_one(&source));
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("non-exhaustive match")),
            "{issues:?}"
        );
    }
}
