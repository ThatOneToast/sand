//! The Sand language is a frontend over the portable compiler: equivalent
//! packs written in Sand source and as portable JSON (the Python producer's
//! fixtures, which the Rust frontend also matches) emit identical resources.

mod support;

use sand::advanced::compiler::{Compiler, Program};
use support::*;

fn compile_json(program: serde_json::Value) -> sand::advanced::compiler::CompiledPack {
    let program: Program = serde_json::from_value(program).unwrap();
    Compiler::compile(&program).unwrap()
}

#[test]
fn counter_matches_the_portable_json_frontend() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "demo", minecraft = "26.2", description = "Player counter")]

player state Counter {
    value: number = 0,
}

#[tag("minecraft:tick")]
fn tick() {
    for player in players() {
        increment();
    }
}

fn increment() {
    Counter.value += 1;
}
"#,
    );
    let reference = serde_json::json!({
        "format": "sand.program", "format_version": 1,
        "target": {"minecraft": "26.2"},
        "pack": {"namespace": "demo", "description": "Player counter"},
        "modules": [{"id": "demo:counter_module", "states": [{
            "id": "demo:counter", "scope": "player", "revision": 1,
            "fields": [{"name": "value", "default": 0}]}],
            "functions": [
                {"id": "demo:tick", "context": "server", "body": [{"action": {
                    "op": "players", "body": [{"action": {"op": "call",
                        "function": {"kind": "internal", "id": "demo:increment"}}}]}}]},
                {"id": "demo:increment", "context": "player", "body": [{"action": {
                    "op": "score_add", "score": {"state": "demo:counter", "field": "value"}, "value": 1}}]}],
            "tags": [{"tag": "minecraft:tick", "function": {"kind": "internal", "id": "demo:tick"}}]}]
    });
    assert_eq!(
        compile(&analysis).resources,
        compile_json(reference).resources
    );
}

#[test]
fn player_and_global_flows_match_the_portable_json_frontend() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "demo", minecraft = "26.2", description = "Player encounters and shared world flow")]

player state Player {
    phase: number,
    #[clamp(0..=100)]
    energy: number = 100,
    events: number,
}

global state World {
    phase: number,
    ticks: number,
}

fn start() { Player.energy -= 30; Player.events += 10; }
fn active() { Player.events += 1; }
fn pulse() { Player.energy -= 5; }
fn rest() { Player.events += 100; }
fn world_tick() { World.ticks += 1; }

flow Encounter on Player.phase {
    0 -> 1 if Player.energy >= 30 priority 100;
    0 -> 2;
    1 -> 2 if Player.energy <= 10;
    enter 1 => start;
    tick 1 => active;
    tick 1 every 2 => pulse;
    enter 2 => rest;
}

flow Round on World.phase {
    0 -> 1;
    tick 1 every 2 => world_tick;
}
"#,
    );
    let field = |state: &str, name: &str| serde_json::json!({"state": format!("demo:{state}"), "field": name});
    let write = |state: &str, name: &str, amount: i32| serde_json::json!({"action": {"op": "score_add", "score": field(state, name), "value": amount}});
    let hook = |phase: &str, state: i32, name: &str, cadence: u32| {
        serde_json::json!({"phase": phase, "state": state,
            "function": {"kind": "internal", "id": format!("demo:{name}")}, "cadence": cadence})
    };
    let function = |name: &str, context: &str, body: Vec<serde_json::Value>| serde_json::json!({"id": format!("demo:{name}"), "context": context, "body": body});
    let reference = serde_json::json!({
        "format": "sand.program", "format_version": 1,
        "target": {"minecraft": "26.2"},
        "pack": {"namespace": "demo", "description": "Player encounters and shared world flow"},
        "requires": ["state_flows", "global_state", "state_defaults", "state_bounds"],
        "modules": [{"id": "demo:encounters", "states": [
            {"id": "demo:player", "scope": "player", "revision": 1, "fields": [
                {"name": "phase", "default": 0}, {"name": "energy", "default": 100, "bounds": [0, 100]},
                {"name": "events", "default": 0}]},
            {"id": "demo:world", "scope": "global", "revision": 1, "fields": [
                {"name": "phase", "default": 0}, {"name": "ticks", "default": 0}]}
        ], "functions": [
            function("start", "player", vec![write("player", "energy", -30), write("player", "events", 10)]),
            function("active", "player", vec![write("player", "events", 1)]),
            function("pulse", "player", vec![write("player", "energy", -5)]),
            function("rest", "player", vec![write("player", "events", 100)]),
            function("world_tick", "server", vec![write("world", "ticks", 1)]),
        ], "flows": [
            {"id": "demo:encounter", "state": field("player", "phase"), "transitions": [
                {"from": 0, "to": 1, "when": {"score": field("player", "energy"), "min": 30}, "priority": 100},
                {"from": 0, "to": 2, "priority": 0},
                {"from": 1, "to": 2, "when": {"score": field("player", "energy"), "max": 10}}
            ], "hooks": [hook("enter", 1, "start", 1), hook("tick", 1, "active", 1),
                         hook("tick", 1, "pulse", 2), hook("enter", 2, "rest", 1)]},
            {"id": "demo:round", "state": field("world", "phase"), "transitions": [{"from": 0, "to": 1}],
             "hooks": [hook("tick", 1, "world_tick", 2)]}
        ]}]
    });
    assert_eq!(
        compile(&analysis).resources,
        compile_json(reference).resources
    );
}
