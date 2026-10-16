//! Real exporter fixture for namespace-owned joins and reconnects.
use sand_core::{EventDescriptor, EventDispatch};
use serde_json::Value;
use std::collections::BTreeSet;

sand_core::inventory::submit! {
    EventDescriptor {
        owner: module_path!(),
        path: "join_b", id_override: None,
        make: || sand::component::IntoCommands::into_commands(vec![
            "execute if entity @s[name=SandAlpha] run scoreboard players add #__sand_local:SandAlpha join_b 1".to_string(),
            "execute if entity @s[name=SandBeta] run scoreboard players add #__sand_local:SandBeta join_b 1".to_string(),
        ]),
        dispatch: EventDispatch::JoinTick,
    }
}
sand_core::inventory::submit! {
    EventDescriptor {
        owner: module_path!(),
        path: "join_a", id_override: None,
        make: || sand::component::IntoCommands::into_commands(vec![
            "execute if entity @s[name=SandAlpha] run scoreboard players add #__sand_local:SandAlpha join_a 1".to_string(),
            "execute if entity @s[name=SandBeta] run scoreboard players add #__sand_local:SandBeta join_a 1".to_string(),
            // The generated check must be safe to reenter from a callback.
            "function __sand_local:__sand_join_check".to_string(),
        ]),
        dispatch: EventDispatch::JoinTick,
    }
}

fn export(namespace: &str) -> String {
    sand_core::try_export_components_json(namespace).expect("join export succeeds")
}
fn records(namespace: &str) -> Vec<Value> {
    serde_json::from_str(&export(namespace)).unwrap()
}
fn function<'a>(records: &'a [Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|r| r["dir"] == "function" && r["path"] == path)
        .unwrap_or_else(|| panic!("missing function {path}"))["content"]
        .as_str()
        .unwrap()
}
fn objectives(records: &[Value]) -> BTreeSet<&str> {
    function(records, "__sand_lifecycle_load")
        .lines()
        .filter_map(|line| line.strip_prefix("scoreboard objectives add "))
        .map(|line| line.split_whitespace().next().unwrap())
        .collect()
}

#[test]
fn join_uses_private_leave_statistic_and_commits_before_reentrant_handlers() {
    let records = records("joina");
    let load = function(&records, "__sand_lifecycle_load");
    assert!(
        load.contains("minecraft.custom:minecraft.leave_game"),
        "{load}"
    );
    let names = objectives(&records);
    assert_eq!(names.len(), 2);
    assert!(
        names
            .iter()
            .all(|name| sand::command::ObjectiveName::try_dynamic(*name).is_ok())
    );
    let reset = load
        .lines()
        .find_map(|l| l.strip_prefix("scoreboard players reset * "))
        .unwrap();
    assert!(names.contains(reset));
    let check = function(&records, "__sand_join_check");
    assert!(
        check
            .lines()
            .next()
            .unwrap()
            .contains("run return run function joina:__sand_join_dispatch")
    );
    assert!(check.contains("unless score @s"));
    let dispatch = function(&records, "__sand_join_dispatch");
    let lines = dispatch.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], format!("scoreboard players set @s {reset} 0"));
    assert!(lines[1].contains(&format!("run scoreboard players operation @s {reset} =")));
    assert_eq!(
        &lines[2..],
        [
            "execute at @s run function joina:join_a",
            "execute at @s run function joina:join_b"
        ]
    );
    assert_eq!(
        function(&records, "__sand_lifecycle_tick"),
        "execute as @a run function joina:__sand_join_check"
    );
    assert!(!load.contains(" __sand_join "));
    assert_eq!(
        function(&records, "join_a").lines().next().unwrap(),
        "execute if entity @s[name=SandAlpha] run scoreboard players add #joina:SandAlpha join_a 1"
    );
}

#[test]
fn independent_namespaces_and_concurrent_exports_keep_distinct_state() {
    let a = records("joina");
    let b = records("joinb");
    assert!(objectives(&a).is_disjoint(&objectives(&b)));
    let expected = export("joina");
    let workers = (0..4)
        .map(|i| {
            std::thread::spawn(move || {
                let namespace = if i % 2 == 0 { "joina" } else { "joinb" };
                (namespace, export(namespace))
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        let (namespace, actual) = worker.join().unwrap();
        assert_eq!(actual, export(namespace));
    }
    assert_eq!(expected, export("joina"));
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_join_packs() {
    let output = std::env::var("SAND_JOIN_DUMP_PATH").expect("dump path");
    let packs = serde_json::json!({"joina": records("joina"), "joinb": records("joinb")});
    std::fs::write(output, serde_json::to_vec(&packs).unwrap()).unwrap();
}

#[test]
fn long_join_namespace_is_preserved_in_each_exact_objective() {
    let namespace = "join_namespace_".repeat(12);
    let records = records(&namespace);
    let names = objectives(&records);
    assert_eq!(names.len(), 2);
    for name in names {
        assert!(name.contains(&namespace));
        assert!(sand::command::ObjectiveName::try_dynamic(name).is_ok());
    }
}
