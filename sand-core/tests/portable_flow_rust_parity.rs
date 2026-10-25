//! Compare every generated flow function from Rust authoring and portable data.
use sand::advanced::compiler::{Compiler, Program};
use sand_core::state::{GameState, StateFlow, Ticks, TypedGameState};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Active,
    Rest,
}
impl TypedGameState for Phase {
    fn to_score(self) -> i32 {
        self as i32
    }
    fn from_score(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Idle),
            1 => Some(Self::Active),
            2 => Some(Self::Rest),
            _ => None,
        }
    }
}

fn normalized(functions: BTreeMap<String, String>) -> BTreeMap<String, String> {
    let root = functions
        .keys()
        .find(|path| path.matches('/').count() == 1)
        .unwrap();
    let key = root.strip_prefix("__sand_transition/flow_").unwrap();
    functions
        .iter()
        .map(|(path, body)| (path.replace(key, "FLOW"), body.replace(key, "FLOW")))
        .collect()
}

#[test]
fn rust_and_portable_frontends_emit_identical_player_flow_commands() {
    let mut program: Program = Compiler::decode(
        include_bytes!("../../tests/fixtures/program/flows.sand.json"),
        &BTreeMap::new(),
    )
    .unwrap();
    program.modules[0].flows.truncate(1);
    let pack = Compiler::compile(&program).unwrap();
    let functions: BTreeMap<_, _> = pack
        .resources
        .iter()
        .filter_map(|(path, bytes)| {
            path.strip_prefix("data/demo/function/")
                .and_then(|p| p.strip_suffix(".mcfunction"))
                .map(|p| (p.to_owned(), String::from_utf8(bytes.clone()).unwrap()))
        })
        .collect();
    let phase_objective = functions
        .iter()
        .find(|(p, _)| p.starts_with("__sand_transition/flow_") && p.matches('/').count() == 1)
        .unwrap()
        .1
        .split_whitespace()
        .nth(4)
        .unwrap();
    let energy_objective = functions["start"]
        .lines()
        .find(|line| line.starts_with("scoreboard players remove"))
        .unwrap()
        .split_whitespace()
        .nth(4)
        .unwrap();
    let phase = GameState::<Phase>::with_default_score(
        Box::leak(phase_objective.to_owned().into_boxed_str()),
        0,
    );
    let energy = sand_core::state::ScoreVar::<i32>::new(Box::leak(
        energy_objective.to_owned().into_boxed_str(),
    ));
    let call = |name: &str| {
        sand_core::cmd::function(
            format!("demo:{name}")
                .parse::<sand_components::FunctionId>()
                .unwrap(),
        )
    };
    StateFlow::players(&phase)
        .transition(Phase::Idle, Phase::Active)
        .when(energy.of("@s").gte(30))
        .priority(100)
        .done()
        .transition(Phase::Idle, Phase::Rest)
        .done()
        .transition(Phase::Active, Phase::Rest)
        .when(energy.of("@s").lte(10))
        .done()
        .on_enter(Phase::Active, call("start"))
        .on_tick(Phase::Active, call("active"))
        .on_tick_every(Phase::Active, Ticks::new(2), call("pulse"))
        .on_enter(Phase::Rest, call("rest"))
        .register();
    let export = sand_core::try_export_components_json("demo").unwrap();
    let records: Vec<serde_json::Value> = serde_json::from_str(&export).unwrap();
    let rust = records
        .iter()
        .filter(|r| {
            r["dir"] == "function"
                && r["path"]
                    .as_str()
                    .unwrap()
                    .starts_with("__sand_transition/flow_")
        })
        .map(|r| {
            (
                r["path"].as_str().unwrap().to_owned(),
                r["content"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let portable = functions
        .into_iter()
        .filter(|(path, _)| path.starts_with("__sand_transition/flow_"))
        .collect();
    // The Rust identity includes the enum type and selector; the portable one
    // uses an explicit namespaced ID. Only that generated identity is normalized.
    let rust = normalized(rust);
    let portable = normalized(portable);
    let root = &portable["__sand_transition/flow_FLOW"];
    assert!(root.contains(
        "unless score @s __sf_FLOWt1 matches 0 run scoreboard players set @s __sf_FLOWt1 0"
    ));
    assert_eq!(rust, portable);
}

#[test]
fn portable_global_interval_uses_the_same_holder_for_guard_and_reset() {
    let program: Program = Compiler::decode(
        include_bytes!("../../tests/fixtures/program/flows.sand.json"),
        &BTreeMap::new(),
    )
    .unwrap();
    let pack = Compiler::compile(&program).unwrap();
    let mut found = false;
    for (path, bytes) in &pack.resources {
        if !path.ends_with(".mcfunction") {
            continue;
        }
        let body = std::str::from_utf8(bytes).unwrap();
        for line in body
            .lines()
            .filter(|line| line.starts_with("execute unless score #") && line.contains("__sf_"))
        {
            let words: Vec<_> = line.split_whitespace().collect();
            assert_eq!(words[3], words[9]);
            assert_eq!(words[3], words[17]);
            assert_eq!(words[10], words[18]);
            assert_eq!(
                &words[7..13],
                &["unless", "score", words[3], words[18], "matches", "0"]
            );
            assert_eq!(
                &words[13..],
                &[
                    "run",
                    "scoreboard",
                    "players",
                    "set",
                    words[3],
                    words[18],
                    "0"
                ]
            );
            found = true;
        }
    }
    assert!(
        found,
        "global fixture must contain an inactive interval reset"
    );
}

#[test]
#[ignore = "external old/new portable fixture export"]
fn dump_portable_flow_pack() {
    let program: Program = Compiler::decode(
        include_bytes!("../../tests/fixtures/program/flows.sand.json"),
        &BTreeMap::new(),
    )
    .unwrap();
    let pack = Compiler::compile(&program).unwrap();
    let path = std::env::var("SAND_PORTABLE_FLOW_DUMP_PATH").expect("dump path");
    std::fs::write(path, serde_json::to_vec(&pack.resources).unwrap()).unwrap();
}
