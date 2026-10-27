use sand_commands::Target;
use sand_core::condition::Condition;
use sand_core::state::{GameState, StateFlow, Ticks, TypedGameState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BossPhase {
    Idle = 0,
    Fighting = 1,
    Enraged = 2,
    Defeated = 3,
}

impl TypedGameState for BossPhase {
    fn to_score(self) -> i32 {
        self as i32
    }

    fn from_score(score: i32) -> Option<Self> {
        match score {
            0 => Some(Self::Idle),
            1 => Some(Self::Fighting),
            2 => Some(Self::Enraged),
            3 => Some(Self::Defeated),
            _ => None,
        }
    }
}

static PHASE: GameState<BossPhase> = GameState::with_default_score("boss_phase", 0);

static THRESHOLD: sand_core::state::ScoreConst =
    sand_core::state::ScoreConst::new("flow_threshold", 43);
static FLOW_GUARD: sand_core::state::ScoreConst =
    sand_core::state::ScoreConst::new("flow_guard", 47);
static FLOW_HOOK: sand_core::state::ScoreConst = sand_core::state::ScoreConst::new("flow_hook", 53);
static SOURCE: sand_core::state::ScoreVar = sand_core::state::ScoreVar::new("flow_value");
static CACHED: std::sync::OnceLock<sand::command::Actions> = std::sync::OnceLock::new();

fn cached_transition() -> sand::command::Actions {
    CACHED
        .get_or_init(|| {
            PHASE
                .of("@s")
                .transition()
                .from(BossPhase::Idle)
                .when(SOURCE.of("@s").gte_score(THRESHOLD.ref_()))
                .to(BossPhase::Fighting)
        })
        .clone()
}
sand_core::inventory::submit! {
    sand_core::FunctionDescriptor { owner: module_path!(),
        context: sand::advanced::compiler::ExecutionContext::Server,
        path: "cached_transition",
        make: cached_transition,
    }
}

fn register_boss_flow() {
    StateFlow::players(&PHASE)
        .named("boss")
        .transition(BossPhase::Fighting, BossPhase::Enraged)
        .when(
            Condition::entity(Target::self_().tag("low_health"))
                .and(SOURCE.of("@s").gte_score(FLOW_GUARD.ref_())),
        )
        .priority(100)
        .done()
        .transition(BossPhase::Fighting, BossPhase::Defeated)
        .when(Condition::entity(Target::self_().tag("dead")))
        .priority(50)
        .done()
        .on_exit(BossPhase::Fighting, "function flowpack:stop_fighting")
        .on_enter(BossPhase::Enraged, "function flowpack:start_enrage")
        .on_tick(BossPhase::Enraged, "function flowpack:enraged_tick")
        .on_tick(
            BossPhase::Defeated,
            SOURCE.of("@s").assign(FLOW_HOOK.ref_()),
        )
        .on_tick_every(
            BossPhase::Enraged,
            Ticks::new(5),
            "function flowpack:enraged_pulse",
        )
        .register();
}

fn export() -> String {
    register_boss_flow();
    sand_core::try_export_components_json("flowpack").expect("state flow should export")
}

fn functions(records: &[serde_json::Value]) -> Vec<(&str, &str)> {
    records
        .iter()
        .filter(|record| record["namespace"] == "flowpack" && record["dir"] == "function")
        .map(|record| {
            (
                record["path"].as_str().unwrap(),
                record["content"].as_str().unwrap(),
            )
        })
        .collect()
}

#[test]
fn unified_flow_exports_one_ordered_lifecycle() {
    let _cached = cached_transition();
    let first = export();
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let setup = records
        .iter()
        .find(|record| record["path"] == "__sand_score_init")
        .unwrap()["content"]
        .as_str()
        .unwrap();
    for value in [43, 47, 53] {
        assert!(setup.contains(&format!(" sand_consts {value}")), "{setup}");
    }
    let functions = functions(&records);
    let (_, root) = functions
        .iter()
        .find(|(path, _)| {
            path.starts_with("__sand_transition/flow_")
                && !path["__sand_transition/".len()..].contains('/')
        })
        .expect("flow root");

    let default = root
        .find("run scoreboard players set @s boss_phase 0")
        .unwrap();
    let dispatch = root.find("/select").unwrap();
    let tick = root.find("/tick_0").unwrap();
    assert!(default < dispatch && dispatch < tick);
    let (_, selection) = functions
        .iter()
        .find(|(path, _)| path.ends_with("/select"))
        .unwrap();
    let high = selection.find("tag=low_health").unwrap();
    let low = selection.find("tag=dead").unwrap();
    assert!(high < low);
    assert!(
        selection
            .lines()
            .filter(|line| line.contains("/transition_"))
            .all(|line| line.contains("run return run function"))
    );
    assert!(
        !root
            .lines()
            .any(|line| line.starts_with("scoreboard players set @s __sf_"))
    );

    let (_, transition) = functions
        .iter()
        .find(|(path, _)| path.ends_with("/transition_0"))
        .expect("highest-priority transition helper");
    let exit = transition.find("function flowpack:stop_fighting").unwrap();
    let write = transition
        .find("scoreboard players set @s boss_phase 2")
        .unwrap();
    let enter = transition.find("function flowpack:start_enrage").unwrap();
    assert!(exit < write && write < enter);
    assert!(!transition.contains("scoreboard players set @s __sf_"));
    assert_eq!(transition.matches("stop_fighting").count(), 1);
    assert_eq!(transition.matches("start_enrage").count(), 1);

    assert!(functions.iter().any(|(_, content)| {
        content.contains("function flowpack:enraged_tick")
            && !content.contains("function flowpack:enraged_pulse")
    }));
    let lines: Vec<_> = root.lines().collect();
    let reset_index = lines
        .iter()
        .position(|line| line.starts_with("execute unless score @s boss_phase matches 2 "))
        .unwrap();
    let counter = lines[reset_index]
        .split("run scoreboard players set @s ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap();
    assert_eq!(
        lines[reset_index],
        format!(
            "execute unless score @s boss_phase matches 2 unless score @s {counter} matches 0 run scoreboard players set @s {counter} 0"
        )
    );
    assert_eq!(
        lines[reset_index + 1],
        format!(
            "execute if score @s boss_phase matches 2 run scoreboard players add @s {counter} 1"
        )
    );
    let (pulse_path, pulse_body) = functions
        .iter()
        .find(|(_, body)| body.contains("function flowpack:enraged_pulse"))
        .unwrap();
    assert_eq!(
        lines[reset_index + 2],
        format!(
            "execute if score @s boss_phase matches 2 if score @s {counter} matches 5.. run function flowpack:{pulse_path}"
        )
    );
    assert_eq!(
        *pulse_body,
        format!("function flowpack:enraged_pulse\nscoreboard players set @s {counter} 0")
    );
    // The lifecycle caller consumes neither the reset's result nor its success.
    let (root_path, _) = functions.iter().find(|(_, body)| *body == *root).unwrap();
    let callers: Vec<_> = functions
        .iter()
        .flat_map(|(_, body)| body.lines())
        .filter(|line| line.ends_with(&format!("function flowpack:{root_path}")))
        .collect();
    assert_eq!(
        callers,
        [format!(
            "execute as @a at @s run function flowpack:{root_path}"
        )]
    );

    register_boss_flow();
    register_boss_flow();
    let repeated =
        sand_core::try_export_components_json("flowpack").expect("repeated export should succeed");
    assert_eq!(
        first, repeated,
        "repeated real exports must be byte-identical and duplicate registrations must deduplicate"
    );
}
