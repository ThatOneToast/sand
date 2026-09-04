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

fn register_boss_flow() {
    StateFlow::players(&PHASE)
        .named("boss")
        .transition(BossPhase::Fighting, BossPhase::Enraged)
        .when(Condition::entity(Target::self_().tag("low_health")))
        .priority(100)
        .done()
        .transition(BossPhase::Fighting, BossPhase::Defeated)
        .when(Condition::entity(Target::self_().tag("dead")))
        .priority(50)
        .done()
        .on_exit(BossPhase::Fighting, "function flowpack:stop_fighting")
        .on_enter(BossPhase::Enraged, "function flowpack:start_enrage")
        .on_tick(BossPhase::Enraged, "function flowpack:enraged_tick")
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
    let first = export();
    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
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
    let high = root.find("tag=low_health").unwrap();
    let low = root.find("tag=dead").unwrap();
    let tick = root.find("/tick_0").unwrap();
    assert!(default < high && high < low && low < tick);
    assert!(
        root.lines()
            .filter(|line| line.contains("/transition_"))
            .all(|line| line.contains("if score @s __sf_") && line.contains("matches 0"))
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
    let lock = transition.rfind("scoreboard players set @s __sf_").unwrap();
    assert!(exit < write && write < enter && enter < lock);
    assert_eq!(transition.matches("stop_fighting").count(), 1);
    assert_eq!(transition.matches("start_enrage").count(), 1);

    assert!(functions.iter().any(|(_, content)| {
        content.contains("function flowpack:enraged_tick")
            && !content.contains("function flowpack:enraged_pulse")
    }));
    assert!(root.contains("matches 5.."));

    register_boss_flow();
    register_boss_flow();
    let repeated =
        sand_core::try_export_components_json("flowpack").expect("repeated export should succeed");
    assert_eq!(
        first, repeated,
        "repeated real exports must be byte-identical and duplicate registrations must deduplicate"
    );
}
