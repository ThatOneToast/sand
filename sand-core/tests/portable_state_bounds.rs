//! Bounded State semantics must agree across Rust collection and owned programs.
use sand::prelude::*;
use std::collections::BTreeMap;

#[derive(State)]
#[state(namespace = "demo", name = "counter", scope = player, version = 1)]
pub struct Counter {
    #[state(default = 150, min = 0, max = 100)]
    pub value: Score,
}

#[function("demo:increment", context = player)]
pub fn increment() {
    let state = Counter::on(EntityContext::<PlayerKind>::default());
    state.value.set(200);
    state.value.add(-300);
}

fn expand(functions: &BTreeMap<String, String>, id: &str, depth: usize) -> Vec<String> {
    assert!(depth < 16, "unexpected recursive helper");
    functions[id]
        .lines()
        .flat_map(|line| {
            if let Some(callee) = line.strip_prefix("function ") {
                expand(functions, callee, depth + 1)
            } else {
                vec![line.to_owned()]
            }
        })
        .collect()
}

#[test]
fn rust_and_portable_writes_have_identical_reachable_commands_and_lifecycle() {
    use sand::advanced::compiler::*;
    let document = serde_json::json!({
        "format": "sand.program", "format_version": 1,
        "target": {"minecraft": "26.2"},
        "pack": {"namespace": "demo", "description": "Bounded State"},
        "requires": ["state_bounds"],
        "modules": [{"id": "demo:module", "states": [{
            "id": "demo:counter", "scope": "player", "revision": 1,
            "fields": [{"name": "value", "default": 150, "bounds": [0,100]}]
        }], "functions": [{"id": "demo:increment", "context": "player", "body": [
            {"action": {"op": "score_set", "score": {"state": "demo:counter", "field": "value"}, "value": 200}},
            {"action": {"op": "score_add", "score": {"state": "demo:counter", "field": "value"}, "value": -300}}
        ]}]}]
    });
    let program =
        Compiler::decode(&serde_json::to_vec(&document).unwrap(), &BTreeMap::new()).unwrap();
    let compiled = Compiler::compile(&program).unwrap().resources;
    let portable: BTreeMap<_, _> = compiled
        .clone()
        .into_iter()
        .filter_map(|(path, bytes)| {
            path.strip_prefix("data/demo/function/")
                .and_then(|path| path.strip_suffix(".mcfunction"))
                .map(|path| (format!("demo:{path}"), String::from_utf8(bytes).unwrap()))
        })
        .collect();
    // Repeat collection around portable compilation to catch ambient registry leaks.
    for _ in 0..2 {
        let json = sand::advanced::try_export_pack_json("demo", "26.2").unwrap();
        let pack: serde_json::Value = serde_json::from_str(&json).unwrap();
        let rust: BTreeMap<_, _> = pack["components"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| record["dir"] == "function")
            .map(|record| {
                (
                    format!(
                        "{}:{}",
                        record["namespace"].as_str().unwrap(),
                        record["path"].as_str().unwrap()
                    ),
                    record["content"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        for entry in [
            "demo:__sand_lifecycle_load",
            "demo:__sand_lifecycle_init",
            "demo:increment",
        ] {
            assert_eq!(
                expand(&portable, entry, 0),
                expand(&rust, entry, 0),
                "{entry}"
            );
        }
        assert_eq!(Compiler::compile(&program).unwrap().resources, compiled);
    }
}
