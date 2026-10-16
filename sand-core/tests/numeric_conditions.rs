//! Runtime numeric Conditions retain their helpers across every export boundary.
use sand::command::ExecuteExt;
use sand::prelude::*;
use sand_core::state::{GameState, StateFlow, TypedGameState};
use std::sync::OnceLock;

#[allow(dead_code)]
#[derive(State)]
#[state(namespace = "numeric_conditions", scope = player)]
struct Numbers {
    amount: Score,
    #[state(scale = 100)]
    threshold: FixedScore,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Waiting,
    Ready,
}
impl TypedGameState for Phase {
    fn to_score(self) -> i32 {
        if self == Self::Waiting { 0 } else { 1 }
    }
    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Waiting),
            1 => Some(Self::Ready),
            _ => None,
        }
    }
}
static PHASE: GameState<Phase> = GameState::with_default_score("numeric_phase", 0);
static FLOW_GUARD: OnceLock<Condition> = OnceLock::new();
static FLOW_HOOK: OnceLock<Actions> = OnceLock::new();

fn register_flow() {
    let player = sand::entity::EntityContext::<sand::entity::PlayerKind>::default();
    let health = StatCurve::from(player.living().health());
    let guard = FLOW_GUARD.get_or_init(|| health.gte(7)).clone();
    let hook = FLOW_HOOK
        .get_or_init(|| when(health.lt(4)).then_all(["say retained flow hook"]))
        .clone();
    StateFlow::players(&PHASE)
        .transition(Phase::Waiting, Phase::Ready)
        .when(guard)
        .done()
        .on_enter(Phase::Ready, hook)
        .register();
}

fn export() -> String {
    register_flow();
    sand_core::try_export_components_json("numeric_conditions").unwrap()
}

static BODY: OnceLock<Actions> = OnceLock::new();
static TICK: OnceLock<sand_core::events::TickEventDispatch> = OnceLock::new();

fn body() -> Actions {
    BODY.get_or_init(|| {
        Target::players().each(|player| {
            let state = Numbers::on(*player);
            let health = StatCurve::from(player.living().health());
            let enough = StatCurve::from(state.amount).gte(state.threshold);
            mcfunction![
                when(enough.and(health.lt(10))).then_all(state.amount.set(3));
                sand::command::TypedExecute::as_players()
                    .when(health.gte(5).and_not(health.gt(20)))
                    .run("say numeric typed guard");
                if_(health.eq(0)).then_all(["say zero"]).else_all(["say nonzero"]);
                PHASE.of("@s").transition().when(health.lt(2)).to(Phase::Ready);
            ]
        })
    })
    .clone()
}

sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "numeric_guards",
        make: body,
    }
}

struct NumericTick;
fn tick() -> Option<sand_core::events::TickEventDispatch> {
    Some(
        TICK.get_or_init(|| {
            let player = sand::entity::EntityContext::<sand::entity::PlayerKind>::default();
            sand_core::events::TickEventDispatch::default()
                .as_players()
                .when(StatCurve::from(player.living().health()).lte(2))
        })
        .clone(),
    )
}

sand_core::inventory::submit! {
    sand_core::EventDescriptor {
        path: "numeric_tick",
        id_override: None,
        make: Actions::default,
        dispatch: sand_core::EventDispatch::Custom {
            make_trigger: || None,
            make_condition: || None,
            make_tick: tick,
            make_chain: || None,
            make_tracked: || None,
            make_participants: || sand_core::participant::EventParticipantPlan::none(),
            revoke: || true,
            event_type_id: std::any::TypeId::of::<NumericTick>,
            event_type_name: std::any::type_name::<NumericTick>,
            make_setup: sand_core::events::EventSetup::none,
        },
    }
}

#[test]
fn cached_numeric_guards_own_helpers_and_initializers() {
    let _ = body();
    let _ = tick();
    let first = export();
    for _ in 0..2 {
        assert_eq!(first, export());
    }
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let functions: std::collections::BTreeMap<_, _> = records
        .iter()
        .filter(|record| record["dir"] == "function")
        .map(|record| {
            (
                record["path"].as_str().unwrap(),
                record["content"].as_str().unwrap(),
            )
        })
        .collect();
    let mut comparisons = 0;
    for body in functions.values() {
        assert!(!body.contains("__sand_local:"));
        for line in body.lines() {
            let Some((_, target)) = line.split_once("function numeric_conditions:") else {
                continue;
            };
            let path = target.split_whitespace().next().unwrap();
            assert!(
                functions.contains_key(path),
                "missing {path} referenced by {line}"
            );
            comparisons +=
                usize::from(line.contains("if function") || line.contains("unless function"));
        }
    }
    assert!(comparisons >= 5);
    let setup = functions["__sand_score_init"];
    assert!(setup.contains("scoreboard objectives add"));
    assert!(
        functions
            .values()
            .any(|body| body.contains("append from entity @s Health"))
    );
    assert!(
        functions
            .values()
            .any(|body| body.contains("say numeric typed guard"))
    );
    assert!(functions.values().any(|body| body.ends_with("return 0")));
}

struct RebuiltCustomTick;
fn rebuilt_custom_tick() -> Option<sand_core::events::TickEventDispatch> {
    // Each subscriber constructs a fresh host callback for the same runtime ID.
    let curve = StatCurve::custom(
        "numeric_conditions:custom_value"
            .parse::<sand::resource_ref::FunctionId>()
            .unwrap(),
        |_, _| Ok(sand::entity::FixedValue::from_units(1)),
    );
    Some(
        sand_core::events::TickEventDispatch::default()
            .as_players()
            .when(curve.gt(0)),
    )
}
macro_rules! custom_tick_handler {
    ($path:literal) => {
        sand_core::inventory::submit! {
            sand_core::EventDescriptor {
                path: $path,
                id_override: None,
                make: Actions::default,
                dispatch: sand_core::EventDispatch::Custom {
                    make_trigger: || None,
                    make_condition: || None,
                    make_tick: rebuilt_custom_tick,
                    make_chain: || None,
                    make_tracked: || None,
                    make_participants: || sand_core::participant::EventParticipantPlan::none(),
                    revoke: || true,
                    event_type_id: std::any::TypeId::of::<RebuiltCustomTick>,
                    event_type_name: std::any::type_name::<RebuiltCustomTick>,
                    make_setup: sand_core::events::EventSetup::none,
                },
            }
        }
    };
}
custom_tick_handler!("custom_first_handler");
custom_tick_handler!("custom_second_handler");
sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "custom_value",
        make: || mcfunction![sand::command::return_cmd(1);],
    }
}
