//! Constant assignment export, lifecycle, ownership, and static cost regressions.
use sand::prelude::*;
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "constant_fixture", scope = entity)]
struct Numbers {
    #[state(default = 2, min = 0, max = 10)]
    integer: Score,
    #[state(default = 1.25, min = -2, max = 5, scale = 100)]
    fraction: FixedScore,
}

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "constant_fixture", scope = global)]
struct World {
    #[state(default = 2, min = 0, max = 10)]
    integer: Score,
}

fn cached() -> Actions {
    static BODY: OnceLock<Actions> = OnceLock::new();
    BODY.get_or_init(|| Numbers::fraction.bind_to("*", true).set(9.0))
        .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        owner: concat!(module_path!(), "::wildcard"),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "wildcard",
        make: cached,
    }
}

#[function]
fn single() {
    Numbers::integer.bind().set(4);
}
#[function]
fn random() {
    Numbers::fraction.bind_to("@r", true).set(-9.0);
}
#[function]
fn players() {
    Numbers::fraction.bind_to("@a", true).set(0.875);
}
#[function]
fn fake() {
    Numbers::integer.bind_to("#fixture", true).set(99);
}
#[function]
fn global() {
    World::global().integer.set(-1);
}

#[test]
fn cached_constants_export_with_owned_lifecycle_and_runtime_report() {
    let _ = cached();
    let first = sand::advanced::try_export_pack_json("constant_a", "26.2").unwrap();
    for namespace in ["constant_a", "constant_b", "constant_a"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        if namespace == "constant_a" {
            assert_eq!(first, json);
        }
        let pack: serde_json::Value = serde_json::from_str(&json).unwrap();
        let records = pack["components"].as_array().unwrap();
        for path in ["single", "random", "players", "fake", "global", "wildcard"] {
            let report = &pack["runtime_report"]["functions"][format!("{namespace}:{path}")];
            assert_eq!(
                report["owners"],
                serde_json::json!([format!("{}::{path}", module_path!())])
            );
        }
        let body = |path: &str| {
            records
                .iter()
                .find(|r| r["dir"] == "function" && r["path"] == path)
                .unwrap()["content"]
                .as_str()
                .unwrap()
        };
        let component_dirty = ObjectiveName::logical("constant_fixture:numbers.reconcile_dirty");
        let expected = |holder: &str, objective: String, dirty: String, value: i32| {
            format!(
                "scoreboard players set {holder} {objective} {value}\nscoreboard players set {holder} {dirty} 1\nscoreboard players set {holder} {component_dirty} 1"
            )
        };
        for (path, holder, value) in [("single", "@s", 4), ("fake", "#fixture", 10)] {
            assert_eq!(
                body(path),
                expected(
                    holder,
                    Numbers::integer.objective(),
                    Numbers::integer.dirty_objective(),
                    value
                )
            );
        }
        assert_eq!(
            body("wildcard"),
            expected(
                "*",
                Numbers::fraction.objective(),
                Numbers::fraction.dirty_objective(),
                500
            )
        );
        for (path, selector, value) in [("random", "@r", -200), ("players", "@a", 88)] {
            let entry = body(path);
            let prefix = format!("execute as {selector} run function {namespace}:");
            let helper = entry.strip_prefix(&prefix).unwrap();
            assert_eq!(
                body(helper),
                expected(
                    "@s",
                    Numbers::fraction.objective(),
                    Numbers::fraction.dirty_objective(),
                    value
                )
            );
            let report = &pack["runtime_report"]["functions"][format!("{namespace}:{helper}")];
            assert_eq!(
                report["owners"],
                serde_json::json!([format!("{}::{path}", module_path!())])
            );
            assert_eq!(
                report["evaluations"],
                serde_json::json!({"minimum": 3, "maximum": 3, "unknown": []})
            );
            // Reporting remains conservative about selector cardinality.
            assert!(pack["runtime_report"]["functions"][format!("{namespace}:{path}")]["evaluations"]["maximum"].is_null());
        }
        for (path, count) in [("single", 3), ("fake", 3), ("wildcard", 3), ("global", 1)] {
            assert_eq!(
                pack["runtime_report"]["functions"][format!("{namespace}:{path}")]["evaluations"],
                serde_json::json!({"minimum": count, "maximum": count, "unknown": []})
            );
        }
        assert!(body("global").starts_with("scoreboard players set #sand_constant_fixture_world_"));
        assert!(body("global").ends_with(&format!(" {} 0", World::integer.objective())));
        assert!(
            !records.iter().any(|r| r["path"] == "__sand_score_init"),
            "constant-only writes need no numeric objective initializer"
        );
        let setup = body("__sand_lifecycle_load");
        for objective in [
            Numbers::integer.objective(),
            Numbers::integer.dirty_objective(),
            Numbers::fraction.objective(),
            Numbers::fraction.dirty_objective(),
            component_dirty.to_string(),
            World::integer.objective(),
        ] {
            assert!(
                setup.contains(&format!("scoreboard objectives add {objective} dummy")),
                "missing lifecycle objective {objective}"
            );
        }
        let load = records
            .iter()
            .find(|r| {
                r["namespace"] == "minecraft" && r["dir"] == "tags/function" && r["path"] == "load"
            })
            .unwrap();
        let load: serde_json::Value =
            serde_json::from_str(load["content"].as_str().unwrap()).unwrap();
        assert_eq!(
            load["values"][0],
            format!("{namespace}:__sand_lifecycle_load")
        );
        for record in records.iter().filter(|r| r["dir"] == "function") {
            assert!(
                !record["content"].as_str().unwrap().contains("#value"),
                "constant-only writes must not provision or use bound scratch"
            );
            assert!(
                !record["path"]
                    .as_str()
                    .unwrap()
                    .starts_with("sand/numeric/")
            );
        }
    }
}

/// Reusable before/after probe: copy this fixture to the baseline checkout and
/// run only this ignored test, leaving candidate regression expectations unused.
#[test]
#[ignore = "explicit static-cost evidence capture; does not run Minecraft"]
fn report_constant_assignment_costs() {
    let pack: serde_json::Value =
        serde_json::from_str(&sand::advanced::try_export_pack_json("constant_a", "26.2").unwrap())
            .unwrap();
    println!("CONSTANT_REPORT {}", pack["runtime_report"]);
    let commands: usize = pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| {
            r["content"]
                .as_str()
                .unwrap()
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .count()
        })
        .sum();
    println!("CONSTANT_STATIC_COMMAND_LINES {commands}");
}
