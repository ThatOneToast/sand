//! Scoreboard-only player initialization keeps repair semantics without idle calls.
use sand::prelude::*;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "init_test", scope = player)]
struct PlayerStats {
    #[state(default = 100)]
    stamina: Score,
    #[state(auto_tick)]
    cooldown: Cooldown,
    flag: Flag,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "init_test", scope = entity)]
struct UnusedStats {
    first: Score,
    second: Score,
}

#[function("entry", context = player)]
fn entry() {
    let stats = PlayerStats::on(EntityContext::<PlayerKind>::default());
    stats.stamina.add(1);
}

fn functions(namespace: &str) -> std::collections::BTreeMap<String, String> {
    let records: Vec<serde_json::Value> =
        serde_json::from_str(&sand_core::try_export_components_json(namespace).unwrap()).unwrap();
    records
        .into_iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| {
            (
                record["path"].as_str().unwrap().to_owned(),
                record["content"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn healthy_scores_skip_only_the_schema_initializer_and_keep_repair_body() {
    let functions = functions("initpack");
    let init = &functions["__sand_lifecycle_init"];
    assert_eq!(init.lines().count(), 1);
    let (guard, call) = init.split_once(" run function initpack:").unwrap();
    let body = &functions[call];
    let defaults: Vec<_> = body
        .lines()
        .filter(|line| line.contains("matches -2147483648.."))
        .collect();
    assert_eq!(defaults.len(), 3);
    for line in &defaults {
        let objective = line
            .split("unless score @s ")
            .nth(2)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        assert!(
            guard.contains(&format!("{objective}=-2147483648..")),
            "{guard}"
        );
        assert!(line.contains("unless score @s"));
    }
    let publish = body.lines().last().unwrap();
    let tail = publish
        .split("run scoreboard players set @s ")
        .nth(1)
        .unwrap();
    let (presence, version) = tail.split_once(' ').unwrap();
    assert!(guard.starts_with("execute unless entity @s[scores={"));
    assert!(guard.contains(&format!("{presence}={version}")));
    assert!(guard.ends_with("}]"));
    assert!(
        body.lines().all(|line| line.contains("unless score @s")),
        "suppression retained for every slow-path command"
    );
    assert!(!body.contains("return"));
    assert!(!init.contains("return"));
    assert!(functions["entry"].starts_with("function initpack:__sand_lifecycle_init\n"));
    let tick = &functions["__sand_lifecycle_tick"];
    assert!(tick.starts_with("execute as @a run function initpack:__sand_lifecycle_init\n"));
    assert!(tick.contains("matches 1.. run scoreboard players remove @s"));
}

#[test]
fn initializer_helpers_are_deterministic_namespace_local_and_unused_state_is_absent() {
    let first = functions("first");
    let second = functions("second");
    assert_eq!(first, functions("first"));
    assert!(first["__sand_lifecycle_init"].contains("run function first:sand/state_init/"));
    assert!(second["__sand_lifecycle_init"].contains("run function second:sand/state_init/"));
    let all = first.values().cloned().collect::<Vec<_>>().join("\n");
    for body in [
        &first["__sand_lifecycle_init"],
        &first["__sand_lifecycle_tick"],
    ] {
        assert!(!body.contains(&UnusedStats::first.objective()));
        assert!(!body.contains(&UnusedStats::second.objective()));
    }
    assert!(!all.contains("second:"));
}

#[test]
fn report_retains_initializer_owner_and_conditional_evaluation_range() {
    let pack: serde_json::Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("initpack", "26.2").unwrap())
            .unwrap();
    let functions = &pack["runtime_report"]["functions"];
    let init = &functions["initpack:__sand_lifecycle_init"];
    assert_eq!(init["evaluations"]["minimum"], 1);
    assert_eq!(init["evaluations"]["maximum"], 5);
    assert_eq!(init["evaluations"]["unknown"], serde_json::json!([]));
    let helpers: Vec<_> = functions
        .as_object()
        .unwrap()
        .iter()
        .filter(|(id, _)| id.starts_with("initpack:sand/state_init/"))
        .collect();
    assert_eq!(helpers.len(), 1);
    assert_eq!(helpers[0].1["owners"], init["owners"]);
    assert_eq!(helpers[0].1["evaluations"]["minimum"], 4);
    assert_eq!(helpers[0].1["evaluations"]["maximum"], 4);
}
