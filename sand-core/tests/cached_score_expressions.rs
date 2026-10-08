//! Cached score expressions retain load-time requirements through helper lowering.
use sand::prelude::*;
use sand_core::state::{ScoreConst, ScoreVar};
use std::sync::OnceLock;

static SOURCE: ScoreVar = ScoreVar::new("cached_value");
static INCREMENT: ScoreConst = ScoreConst::new("cached_increment", 7);
static THRESHOLD: ScoreConst = ScoreConst::new("cached_threshold", 12);
static BODY: OnceLock<Actions> = OnceLock::new();

fn body() -> Actions {
    BODY.get_or_init(|| {
        let direct = when(
            SOURCE
                .of("@s")
                .expr()
                .plus(INCREMENT.ref_())
                .gte_score(THRESHOLD.ref_()),
        )
        .then_one("say computed direct");
        let nested = unless(SOURCE.of("@s").expr().plus(INCREMENT.ref_()).eq(0))
            .then_all(["say computed nested"]);
        mcfunction![
            direct;
            when(SOURCE.of("@s").gte(0)).then_all(nested);
        ]
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor {
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "cached_scores",
        make: body,
    }
}

#[test]
fn prebuilt_score_expressions_replay_setup_on_every_export() {
    let _cached = body();
    let first = sand_core::try_export_components_json("cached_scores").unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            sand_core::try_export_components_json("cached_scores").unwrap()
        );
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let setup = records
        .iter()
        .find(|record| record["path"] == "__sand_score_init")
        .expect("cached expressions require a score initializer")["content"]
        .as_str()
        .unwrap();
    assert_eq!(
        setup
            .lines()
            .filter(|line| *line == "scoreboard objectives add __sand_tmp dummy")
            .count(),
        1
    );
    assert_eq!(
        setup
            .lines()
            .filter(|line| *line == "scoreboard objectives add sand_consts dummy")
            .count(),
        1
    );
    for value in [7, 12] {
        assert_eq!(
            setup
                .lines()
                .filter(|line| line.starts_with("scoreboard players set #sand_")
                    && line.ends_with(&format!(" sand_consts {value}")))
                .count(),
            1
        );
    }
    let functions = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| record["content"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        functions.contains("scoreboard players operation @s __sand_tmp += #sand_cached_increment_")
    );
    assert!(functions.contains("say computed direct"));
    assert!(functions.contains("say computed nested"));
}
