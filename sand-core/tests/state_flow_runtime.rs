//! Real-server fixture for indexed StateFlow selection and ordered hooks.
use sand::advanced::state::{GameState, StateFlow, TypedGameState};
use sand::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Negative = i32::MIN as isize,
    Idle = 0,
    Active = 10,
    Rest = 20,
    Done = i32::MAX as isize,
}
impl TypedGameState for Phase {
    fn to_score(self) -> i32 {
        self as i32
    }
    fn from_score(value: i32) -> Option<Self> {
        match value {
            i32::MIN => Some(Self::Negative),
            0 => Some(Self::Idle),
            10 => Some(Self::Active),
            20 => Some(Self::Rest),
            i32::MAX => Some(Self::Done),
            _ => None,
        }
    }
}
static PHASE: GameState<Phase> = GameState::with_default_score("fa_phase", 0);

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "flow_audit", scope = player)]
struct Metrics {
    trace: Score,
    pulse: Score,
}

fn fixture() -> serde_json::Value {
    let metrics = Metrics::on(EntityContext::<PlayerKind>::default());
    let tagged = |name| Condition::entity(Target::self_().tag(name));
    StateFlow::players(&PHASE)
        .named("encounter")
        .transition(Phase::Idle, Phase::Active)
        .when(tagged("start").or(tagged("alternate")))
        .priority(100)
        .done()
        .transition(Phase::Idle, Phase::Rest)
        .when(tagged("start"))
        .priority(100)
        .done()
        .transition(Phase::Idle, Phase::Done)
        .when(tagged("start"))
        .priority(50)
        .done()
        .transition(Phase::Active, Phase::Rest)
        .when(tagged("advance"))
        .done()
        .transition(Phase::Rest, Phase::Done)
        .when(tagged("advance"))
        .done()
        .transition(Phase::Negative, Phase::Idle)
        .when(tagged("start"))
        .done()
        .on_exit(Phase::Idle, metrics.trace.add(1))
        .on_enter(Phase::Active, metrics.trace.add(10))
        .on_exit(Phase::Active, metrics.trace.add(2))
        .on_enter(Phase::Rest, metrics.trace.add(20))
        .on_tick(
            Phase::Active,
            when(tagged("mutate_early")).then_one(PHASE.of("@s").set(Phase::Rest)),
        )
        .on_tick(Phase::Active, metrics.trace.add(100))
        .on_tick_every(Phase::Active, Ticks::new(2), metrics.pulse.add(1))
        .on_tick(
            Phase::Active,
            when(tagged("mutate")).then_one(PHASE.of("@s").set(Phase::Rest)),
        )
        .on_tick(Phase::Rest, metrics.trace.add(1000))
        .on_tick(Phase::Done, metrics.trace.add(10000))
        .register();
    let records: serde_json::Value =
        serde_json::from_str(&sand_core::try_export_components_json("flow_audit").unwrap())
            .unwrap();
    let root = records
        .as_array()
        .unwrap()
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"].as_str().is_some_and(|path| {
                    path.starts_with("__sand_transition/flow_") && path.matches('/').count() == 1
                })
        })
        .unwrap()["path"]
        .as_str()
        .unwrap();
    let body = records
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["path"] == root)
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let reset = body
        .lines()
        .find(|line| line.starts_with("execute unless score @s fa_phase matches 10 "))
        .unwrap();
    let counter = reset
        .split("run scoreboard players set @s ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    serde_json::json!({"records": records, "root": root, "counter": counter, "trace": Metrics::trace.objective(), "pulse": Metrics::pulse.objective()})
}

#[test]
fn gameplay_flow_exports_without_transition_lock_and_is_repeatable() {
    let first = fixture();
    assert_eq!(first, fixture());
    let root = first["root"].as_str().unwrap();
    let records = first["records"].as_array().unwrap();
    let body = records
        .iter()
        .find(|record| record["path"] == root)
        .unwrap()["content"]
        .as_str()
        .unwrap();
    let counter = first["counter"].as_str().unwrap();
    let reset = format!(
        "execute unless score @s fa_phase matches 10 unless score @s {counter} matches 0 run scoreboard players set @s {counter} 0"
    );
    let early = body.find("/tick_0").unwrap();
    let reset_position = body.find(&reset).unwrap();
    let later = body.find("/tick_3").unwrap();
    assert!(early < reset_position && reset_position < later);
    assert!(body.contains("/select"));
    assert!(body.find("/select").unwrap() < body.find("/tick_").unwrap());
    assert!(
        !records
            .iter()
            .any(|record| record["content"].as_str().is_some_and(|body| body
                .lines()
                .any(|line| line.starts_with("scoreboard objectives add __sf_")
                    && line.ends_with("r dummy"))))
    );
}

#[test]
#[ignore = "external live-server fixture export"]
fn dump_state_flow_pack() {
    let path = std::env::var("SAND_STATE_FLOW_DUMP_PATH").expect("dump path");
    std::fs::write(path, serde_json::to_vec(&fixture()).unwrap()).unwrap();
}
