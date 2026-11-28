mod support;
use support::*;

use sand::advanced::compiler::Compiler;

#[test]
fn breakpoint_records_a_source_site_and_suspends_through_the_wait_queue() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "debug_probe", minecraft = "26.2")]
global state DebugLog { value: number }
fn wait(_value: number) {}
pub fn run(input: number) {
    let doubled = input * 2;
    breakpoint("before_write", server = true);
    DebugLog.value = doubled;
}
"#,
    );
    let pack = compile(&analysis);
    let &(file, span) = &analysis.breakpoints["before_write"];
    assert!(analysis.package.files[file.0].text[span.start..span.end].starts_with("breakpoint("));
    let program = analysis.program.as_ref().unwrap();
    assert!(matches!(
        &program.modules[0].load[0].action,
        sand::advanced::compiler::Action::ScoreSet { score, value: 0 }
            if score.field == "before_write"
    ));
    let arm = Compiler::breakpoint_control(program, "before_write", true).unwrap();
    let release = Compiler::breakpoint_control(program, "before_write", false).unwrap();
    assert_eq!(arm.strip_suffix(" 1"), release.strip_suffix(" 0"));
    assert!(Compiler::breakpoint_control(program, "other", true).is_err());

    let map = analysis.source_map().unwrap();
    assert!(
        map["commands"]
            .as_object()
            .unwrap()
            .keys()
            .any(|name| name.ends_with("/continuation.mcfunction"))
    );
    assert!(
        pack.resources
            .keys()
            .any(|name| name.contains("server_wait_drain"))
    );
}

#[test]
fn breakpoint_requires_a_unique_literal_label_and_a_supported_context() {
    for (body, expected) in [
        (
            "fn run() { breakpoint(\"Bad\"); }",
            "lowercase breakpoint label",
        ),
        (
            "fn run() { breakpoint(\"same\"); breakpoint(\"same\"); }",
            "duplicate breakpoint label",
        ),
        (
            "#[load] fn run() { breakpoint(\"server\"); }",
            "on the server",
        ),
    ] {
        let source = format!("#![pack(namespace = \"debug_probe\")]\n{body}");
        let messages = errors(&analyze_one(&source));
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "{messages:?}"
        );
    }
}
