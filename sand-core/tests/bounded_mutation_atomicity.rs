//! A numeric mutation and its clamps stay indivisible under guarded composition.
use sand::prelude::*;
use serde_json::Value;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "bounded_mutation", scope = player)]
struct Vitals {
    #[state(default = 95, min = 0, max = 100)]
    energy: Score,
}

// Preserve the formerly unsafe caller shape as a regression. A write is one
// logical action even when callers compose returned actions through iteration.
#[function("iterated_add")]
fn iterated_add() {
    let state = Vitals::on(EntityContext::<PlayerKind>::default());
    state
        .energy
        .add(10)
        .into_iter()
        .flat_map(|action| {
            TypedExecute::as_players()
                .when(state.energy.matches(..100).unwrap())
                .run(action)
        })
        .collect::<Actions>();
}

#[function("iterated_subtract")]
fn iterated_subtract() {
    let state = Vitals::on(EntityContext::<PlayerKind>::default());
    state
        .energy
        .subtract(10)
        .into_iter()
        .flat_map(|action| {
            TypedExecute::as_players()
                .when(state.energy.matches(1..).unwrap())
                .run(action)
        })
        .collect::<Actions>();
}

#[function("whole_write")]
fn whole_write() {
    let state = Vitals::on(EntityContext::<PlayerKind>::default());
    TypedExecute::as_players()
        .when(state.energy.matches(..100).unwrap())
        .run(state.energy.add(10));
}

fn export() -> Vec<Value> {
    serde_json::from_str(&sand_core::try_export_components_json("bounded_mutation").unwrap())
        .unwrap()
}
fn function<'a>(records: &'a [Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .unwrap_or_else(|| panic!("missing function {path}"))["content"]
        .as_str()
        .unwrap()
}

#[test]
fn each_guard_runs_once_and_the_reachable_body_keeps_both_clamps() {
    let records = export();
    let objective = Vitals::energy.objective();
    for (path, guard, mutation) in [
        ("iterated_add", "..99", "add"),
        ("iterated_subtract", "1..", "remove"),
        ("whole_write", "..99", "add"),
    ] {
        let entry = function(&records, path);
        let lines = entry.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 1, "one guard evaluation for {path}: {entry}");
        let prefix = format!(
            "execute as @a if score @s {objective} matches {guard} run function bounded_mutation:"
        );
        let helper = entry
            .strip_prefix(&prefix)
            .unwrap_or_else(|| panic!("expected one guarded call: {entry}"));
        let body = function(&records, helper);
        assert_eq!(body.lines().count(), 3, "{body}");
        assert!(
            body.starts_with(&format!(
                "scoreboard players {mutation} @s {objective} 10\n"
            )),
            "{body}"
        );
        assert!(body.contains(&format!("execute if score @s {objective} matches ..-1 run scoreboard players set @s {objective} 0")), "{body}");
        assert!(body.contains(&format!("execute if score @s {objective} matches 101.. run scoreboard players set @s {objective} 100")), "{body}");
        assert!(
            !body.contains(&format!("matches {guard} ")),
            "the original guard must not be rechecked inside the write: {body}"
        );
    }
}

#[test]
fn mutation_helpers_are_deterministic_across_concurrent_exports() {
    let expected = sand_core::try_export_components_json("atomic_a").unwrap();
    let workers = (0..4)
        .map(|index| {
            std::thread::spawn(move || {
                let namespace = if index % 2 == 0 {
                    "atomic_a"
                } else {
                    "atomic_b"
                };
                let json = sand_core::try_export_components_json(namespace).unwrap();
                assert!(!json.contains("__sand_local:"));
                (namespace, json)
            })
        })
        .collect::<Vec<_>>();
    for worker in workers {
        let (namespace, actual) = worker.join().unwrap();
        assert_eq!(
            actual,
            sand_core::try_export_components_json(namespace).unwrap()
        );
    }
    assert_eq!(
        expected,
        sand_core::try_export_components_json("atomic_a").unwrap()
    );
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_bounded_mutation_pack() {
    let output = std::env::var("SAND_BOUNDED_MUTATION_DUMP_PATH").expect("dump path");
    let records = export();
    let fixture = serde_json::json!({"records": records, "objective": Vitals::energy.objective()});
    std::fs::write(output, serde_json::to_vec(&fixture).unwrap()).unwrap();
}
