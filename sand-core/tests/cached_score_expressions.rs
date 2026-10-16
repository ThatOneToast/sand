//! Cached score expressions retain load-time requirements through helper lowering.
use sand::command::ExecuteExt;
use sand::prelude::*;
use sand_core::state::{ScoreConst, ScoreVar};
use std::sync::OnceLock;

static SOURCE: ScoreVar = ScoreVar::new("cached_value");
static INCREMENT: ScoreConst = ScoreConst::new("cached_increment", 7);
static THRESHOLD: ScoreConst = ScoreConst::new("cached_threshold", 12);
static EXEC_THRESHOLD: ScoreConst = ScoreConst::new("cached_execute", 23);
static DIRECT_CONSTANT: ScoreConst = ScoreConst::new("cached_direct", 31);
static CONSUMER_CONSTANT: ScoreConst = ScoreConst::new("cached_consumer", 37);
static TICK_THRESHOLD: ScoreConst = ScoreConst::new("cached_tick", 29);
static TICK: OnceLock<sand_core::events::TickEventDispatch> = OnceLock::new();
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
            SOURCE.of("@s").mul_score(DIRECT_CONSTANT.ref_());
            SOURCE.of("@s").set_percent(DIRECT_CONSTANT.ref_(), 75);
            SOURCE.of("@s").scale_percent(50);
            SOURCE.of("@s").set_ratio(DIRECT_CONSTANT.ref_(), DIRECT_CONSTANT.ref_(), 1000);
            SOURCE.of("@s").safe_divide(DIRECT_CONSTANT.ref_(), 0);
            SOURCE.of("@s").clamp_score(INCREMENT.ref_(), DIRECT_CONSTANT.ref_());
            direct;
            when(SOURCE.of("@s").gte(0)).then(SOURCE.of("@s").assign(CONSUMER_CONSTANT.ref_()));
            when(SOURCE.of("@s").gte(0)).then_one(SOURCE.of("@s").mul_score(CONSUMER_CONSTANT.ref_()));
            unless(SOURCE.of("@s").lt(0)).then(SOURCE.of("@s").assign(CONSUMER_CONSTANT.ref_()));
            unless(SOURCE.of("@s").lt(0)).then_one(SOURCE.of("@s").mul_score(CONSUMER_CONSTANT.ref_()));
            when(SOURCE.of("@s").gte(0))
                .and_then(SOURCE.of("@s").assign(CONSUMER_CONSTANT.ref_()))
                .then(SOURCE.of("@s").scale_percent(50));
            unless(SOURCE.of("@s").lt(0))
                .and_then(SOURCE.of("@s").assign(CONSUMER_CONSTANT.ref_()))
                .then(SOURCE.of("@s").scale_percent(50));
            sand::command::TypedExecute::as_players()
                .when(SOURCE.of("@s").gte(0))
                .run(SOURCE.of("@s").assign(CONSUMER_CONSTANT.ref_()));
            sand::command::TypedExecute::as_players()
                .unless(SOURCE.of("@s").lt(0))
                .run(SOURCE.of("@s").set_percent(CONSUMER_CONSTANT.ref_(), 75));
            sand::command::TypedExecute::as_players()
                .when(SOURCE.of("@s").gte_score(EXEC_THRESHOLD.ref_()))
                .run("say cached typed execute");
            when(SOURCE.of("@s").gte(0)).then_all(nested);
        ]
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "cached_scores",
        make: body,
    }
}

fn tick_dispatch() -> Option<sand_core::events::TickEventDispatch> {
    Some(
        TICK.get_or_init(|| {
            sand_core::events::TickEventDispatch::default()
                .as_players()
                .when(SOURCE.of("@s").gte_score(TICK_THRESHOLD.ref_()))
        })
        .clone(),
    )
}

struct CachedTick;

sand_core::inventory::submit! {
    sand_core::EventDescriptor {
        path: "cached_tick",
        id_override: None,
        make: Actions::default,
        dispatch: sand_core::EventDispatch::Custom {
            make_trigger: || None,
            make_condition: || None,
            make_tick: tick_dispatch,
            make_chain: || None,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: || true,
            event_type_id: std::any::TypeId::of::<CachedTick>,
            event_type_name: std::any::type_name::<CachedTick>,
            make_setup: sand_core::events::EventSetup::none,
        },
    }
}

#[test]
fn prebuilt_score_expressions_replay_setup_on_every_export() {
    let _cached = body();
    let _cached_tick = tick_dispatch();
    let first = sand_core::try_export_components_json("cached_scores").unwrap();
    for _ in 0..2 {
        assert_eq!(
            first,
            sand_core::try_export_components_json("cached_scores").unwrap()
        );
    }
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| sand_core::try_export_components_json("cached_scores").unwrap())
            })
            .collect();
        for worker in workers {
            assert_eq!(first, worker.join().unwrap());
        }
    });
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
    for value in [7, 12, 23, 29, 31, 37, 75, 50, 100, 1000] {
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
    let grouped = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .filter_map(|record| record["content"].as_str())
        .find(|body| {
            body.lines().count() == 3
                && body.starts_with(
                    "scoreboard players operation @s cached_value = #sand_cached_consumer_",
                )
                && body.contains(" *= ")
                && body.contains(" /= ")
        })
        .expect("typed execute retains a multi-action percentage operation in one helper");
    assert!(
        !grouped.contains("execute "),
        "the condition belongs to the call, not each action"
    );
    assert!(functions.contains("say computed direct"));
    assert!(functions.contains("say computed nested"));
    assert!(functions.contains("say cached typed execute"));
    assert!(functions.contains("if score @s cached_value >= #sand_cached_tick_"));
}
