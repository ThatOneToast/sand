mod support;
use sand_lang::{OverlayFiles, analyze_tests};
use std::path::PathBuf;
use support::*;

const SOURCE: &str = include_str!("fixtures/authored_tests.sand");

#[test]
fn production_omits_test_entries_and_test_builds_emit_wrappers() {
    let production = analyze_one(SOURCE);
    let prod_pack = compile(&production);
    assert_eq!(production.tests.len(), 2);
    assert!(
        !prod_pack
            .resources
            .keys()
            .any(|path| path.contains("owned_packages") || path.contains("sand.lang_test"))
    );
    let root = PathBuf::from("/test/pack.sand");
    let files = OverlayFiles::in_memory([(root.clone(), SOURCE)]);
    let tests = analyze_tests(sand_lang::load(&root, &files));
    let test_pack = compile(&tests);
    for (index, case) in tests.tests.iter().enumerate() {
        let (_, runner) = case.runner.split_once(':').unwrap();
        let file = path_for(runner);
        assert!(test_pack.resources.contains_key(&file));
        assert!(test_pack.resources.values().any(|bytes| {
            String::from_utf8_lossy(bytes).contains(&format!("__SAND_TEST_{index}_PASS__"))
        }));
    }
}
fn path_for(name: &str) -> String {
    format!("data/test_probe/function/{name}.mcfunction")
}

#[test]
fn test_diagnostics_enforce_shape_context_and_runtime_isolation() {
    for (body, message) in [
        (
            "#[test] fn invalid(a: number) -> bool { return true; }",
            "plain function",
        ),
        ("#[test] fn invalid() { }", "plain function"),
        (
            "#[test] #[load] fn invalid() -> bool { return true; }",
            "other function attributes",
        ),
        (
            "#[test] fn invalid() -> bool { kill(); return true; }",
            "without a player or entity",
        ),
        (
            "#[test] fn sample() -> bool { return true; } fn runtime() { sample(); }",
            "only be called by other tests",
        ),
        (
            "fn runtime() { assert!(false); }",
            "only available inside a Sand test",
        ),
    ] {
        let source = format!("#![pack(namespace = \"test_probe\")]\n{body}");
        let diagnostics = errors(&analyze_one(&source));
        assert!(
            diagnostics.iter().any(|d| d.contains(message)),
            "{body}: {diagnostics:?}"
        );
    }
}

#[test]
fn source_map_points_to_real_operations_and_emitted_functions() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "source_probe")]
global state S { value: number }
fn run(n: number) {
    if n > 0 { S.value = 7; }
}
"#,
    );
    let pack = compile(&analysis);
    let map = analysis.source_map().unwrap();
    let module = &serde_json::to_value(&analysis.program.as_ref().unwrap().modules[0]).unwrap();
    let function = &map["functions"][0];
    assert!(
        pack.resources
            .contains_key(function["resource"].as_str().unwrap())
    );
    let mut nested = false;
    for entry in map["entries"].as_array().unwrap() {
        let pointer = entry["pointer"].as_str().unwrap();
        if pointer.contains("/action/then/") {
            nested = true;
            let operation = module.pointer(pointer).unwrap();
            assert_eq!(operation["origin"], entry["source"]);
        }
    }
    assert!(nested);
    assert_eq!(analysis.source_map().unwrap(), map);
}
