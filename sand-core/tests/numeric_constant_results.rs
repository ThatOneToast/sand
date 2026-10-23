//! Public State/Execute authoring preserves result-sensitive function boundaries.
use sand::prelude::*;
use sand_core::state::{ScoreConst, ScoreVar};

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "result_author", scope = global)]
struct World {
    #[state(default = 0, min = 0, max = 10)]
    bounded: Score,
    unbounded: Score,
}

static GATE: ScoreVar = ScoreVar::new("author_gate");
static ONE: ScoreConst = ScoreConst::new("author_one", 1);

#[function]
fn bounded_result() {
    Execute::new()
        .store_result_score(ScoreHolder::fake("#observed"), "observed")
        .run_actions(World::global().bounded.set(7));
}
#[function]
fn bounded_success() {
    Execute::new()
        .store_success_score(ScoreHolder::fake("#observed"), "observed")
        .run_actions(World::global().bounded.set(7));
}
#[function]
fn unbounded_result() {
    Execute::new()
        .store_result_score(ScoreHolder::fake("#observed"), "observed")
        .run_actions(World::global().unbounded.set(7));
}
#[function]
fn operand_wrapper() {
    Execute::new()
        .store_result_score(ScoreHolder::fake("#observed"), "observed")
        .when(GATE.of("#gate").gte_score(ONE.ref_()))
        .run(World::global().bounded.set(7));
}
#[function]
fn condition_wrapper() {
    Execute::new()
        .store_result_score(ScoreHolder::fake("#observed"), "observed")
        .when(StatCurve::from(World::global().unbounded).gte(1))
        .run(World::global().bounded.set(7));
}

#[test]
fn public_execute_state_export_retains_helpers_and_resource_setup() {
    let first = sand::advanced::try_export_pack_json("author_a", "26.2").unwrap();
    for namespace in ["author_a", "author_b", "author_a"] {
        let json = sand::advanced::try_export_pack_json(namespace, "26.2").unwrap();
        if namespace == "author_a" {
            assert_eq!(json, first);
        }
        let export: serde_json::Value = serde_json::from_str(&json).unwrap();
        let records = export["components"].as_array().unwrap();
        let body = |path: &str| {
            records
                .iter()
                .find(|r| r["dir"] == "function" && r["path"] == path)
                .unwrap()["content"]
                .as_str()
                .unwrap()
        };
        for path in [
            "bounded_result",
            "bounded_success",
            "operand_wrapper",
            "condition_wrapper",
        ] {
            let entry = body(path);
            let helper = entry
                .split(&format!("run function {namespace}:"))
                .last()
                .unwrap();
            assert!(
                helper.starts_with("sand/action_sequence/"),
                "{path}: {entry}"
            );
            let helper_body = body(helper);
            assert_eq!(helper_body.lines().count(), 1);
            assert!(helper_body.starts_with("scoreboard players set #sand_result_author_world_"));
            assert!(helper_body.ends_with(&format!(" {} 7", World::bounded.objective())));
            assert!(
                export["runtime_report"]["functions"][format!("{namespace}:{helper}")]["owners"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(format!("{}::{path}", module_path!())))
            );
        }
        assert!(body("unbounded_result").contains("run scoreboard players set"));
        assert!(!body("unbounded_result").contains("run function"));
        assert!(body("__sand_score_init").contains("sand_consts"));
    }
}

#[function]
fn unbounded_success() {
    Execute::new()
        .store_success_score(ScoreHolder::fake("#observed"), "observed")
        .run_actions(World::global().unbounded.set(7));
}

#[datapack_component]
fn return_result() -> sand::component::McFunction {
    sand::component::McFunction::new("__sand_local:return_result".parse().unwrap()).command(
        sand_core::ir::Cmd::ReturnRun(Box::new(sand_core::ir::Cmd::RegisteredFunction(
            bounded_result.function_id(),
        ))),
    )
}

#[datapack_component]
fn return_success() -> sand::component::McFunction {
    sand::component::McFunction::new("__sand_local:return_success".parse().unwrap()).command(
        sand_core::ir::Cmd::ReturnRun(Box::new(sand_core::ir::Cmd::RegisteredFunction(
            bounded_success.function_id(),
        ))),
    )
}

#[test]
#[ignore = "explicit public authoring fixture export; set SAND_NUMERIC_AUTHOR_FIXTURE to an absolute JSON path"]
fn export_public_numeric_result_fixture() {
    let destination = std::env::var("SAND_NUMERIC_AUTHOR_FIXTURE").unwrap();
    assert!(std::path::Path::new(&destination).is_absolute());
    let mut pack: serde_json::Value = serde_json::from_str(
        &sand::advanced::try_export_pack_json("author_probe", "26.2").unwrap(),
    )
    .unwrap();
    let entry = pack["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["path"] == "unbounded_result")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let holder = entry
        .split("run scoreboard players set ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    pack["holder"] = serde_json::json!(holder);
    pack["bounded"] = serde_json::json!(World::bounded.objective());
    pack["unbounded"] = serde_json::json!(World::unbounded.objective());
    pack["gate"] = serde_json::json!(GATE.objective_name());
    std::fs::write(destination, serde_json::to_vec_pretty(&pack).unwrap()).unwrap();
}
