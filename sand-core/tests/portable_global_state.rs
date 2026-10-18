//! Shared global/player ownership across Rust and portable authoring.
use sand::prelude::*;
use sand::registration::{DatapackRegistration, LifecycleContribution};
use std::collections::BTreeMap;

#[derive(State)]
#[state(namespace = "global_test", name = "round", scope = global, version = 1)]
pub struct Round {
    #[state(default = 10, min = 0, max = 100)]
    pub value: Score,
}

#[derive(State)]
#[state(namespace = "global_test", name = "player", scope = player, version = 1)]
pub struct Player {
    #[state(default = 2)]
    pub value: Score,
}

#[function("global_test:advance")]
pub fn advance() {
    Round::global().value.add(-1);
}

#[function("global_test:award", context = player)]
pub fn award() {
    Player::on(EntityContext::<PlayerKind>::default())
        .value
        .add(1);
    Round::global().value.add(1);
}

#[datapack_component]
fn load() -> DatapackRegistration {
    DatapackRegistration::new().lifecycle(LifecycleContribution::load(cmd::function(advance)))
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
fn shared_global_and_player_resources_agree_with_rust_collection() {
    use sand::advanced::compiler::*;
    let document = serde_json::json!({
        "format": "sand.program", "format_version": 1, "target": {"minecraft": "26.2"},
        "pack": {"namespace": "global_test", "description": "Mixed State ownership"},
        "requires": ["global_state", "state_defaults", "state_bounds"],
        "modules": [{"id": "global_test:module", "states": [
            {"id": "global_test:round", "scope": "global", "revision": 1,
             "fields": [{"name": "value", "default": 10, "bounds": [0,100]}]},
            {"id": "global_test:player", "scope": "player", "revision": 1,
             "fields": [{"name": "value", "default": 2}]}
        ], "load": [{"action": {"op": "call", "function": {"kind": "internal", "id": "global_test:advance"}}}], "functions": [
            {"id": "global_test:advance", "context": "server", "body": [
                {"action": {"op": "score_add", "score": {"state": "global_test:round", "field": "value"}, "value": -1}}
            ]},
            {"id": "global_test:award", "context": "player", "body": [
                {"action": {"op": "score_add", "score": {"state": "global_test:player", "field": "value"}, "value": 1}},
                {"action": {"op": "score_add", "score": {"state": "global_test:round", "field": "value"}, "value": 1}}
            ]}
        ]}]
    });
    let program =
        Compiler::decode(&serde_json::to_vec(&document).unwrap(), &BTreeMap::new()).unwrap();
    let compiled = Compiler::compile(&program).unwrap().resources;
    let portable: BTreeMap<_, _> = compiled
        .iter()
        .filter_map(|(path, bytes)| {
            path.strip_prefix("data/global_test/function/")
                .and_then(|path| path.strip_suffix(".mcfunction"))
                .map(|path| {
                    (
                        format!("global_test:{path}"),
                        String::from_utf8(bytes.clone()).unwrap(),
                    )
                })
        })
        .collect();
    for _ in 0..2 {
        let json = sand::advanced::try_export_pack_json("global_test", "26.2").unwrap();
        let pack: serde_json::Value = serde_json::from_str(&json).unwrap();
        let records = pack["components"].as_array().unwrap();
        let rust: BTreeMap<_, _> = records
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
            "__sand_lifecycle_load",
            "__sand_lifecycle_init",
            "__sand_lifecycle_tick",
            "advance",
            "award",
        ] {
            let id = format!("global_test:{entry}");
            assert_eq!(expand(&portable, &id, 0), expand(&rust, &id, 0), "{entry}");
        }
        for record in records.iter().filter(|record| record["dir"] != "function") {
            let path = format!(
                "data/{}/{}/{}.{}",
                record["namespace"].as_str().unwrap(),
                record["dir"].as_str().unwrap(),
                record["path"].as_str().unwrap(),
                record["ext"].as_str().unwrap()
            );
            assert_eq!(
                compiled[&path],
                record["content"].as_str().unwrap().as_bytes(),
                "{path}"
            );
        }
        assert_eq!(Compiler::compile(&program).unwrap().resources, compiled);
    }
}
