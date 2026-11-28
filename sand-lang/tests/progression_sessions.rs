mod support;

use support::*;

const QUEST: &str = include_str!("../../examples/lang_quest_session/pack.sand");

#[test]
fn sessions_share_one_progression_and_emit_all_controls() {
    let source = format!("{QUEST}\nsession SecondRun {{ progression: Trail, ticks: 20 }}");
    let analysis = analyze_one(&source);
    let pack = compile(&analysis);
    let base = "scout_trial/function/__sand_system/lifetime";
    assert!(function(&pack, &format!("{base}/trail/progression_advance")).contains("function"));
    assert!(
        function(&pack, &format!("{base}/trail_run/session_advance")).contains("session_after")
    );
    assert!(
        function(&pack, &format!("{base}/trail_run/after_tick"))
            .lines()
            .any(|line| line.starts_with("execute if score @s ") && line.contains("matches ..0 run function scout_trial:__sand_system/lifetime/trail_run/session_step"))
    );
    assert!(function(&pack, "scout_trial/function/replay").contains("trail_run/session_restart"));
    assert!(function(&pack, "scout_trial/function/check").contains("function"));
    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&portable, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn quests_check_the_progress_score_branch_direction_and_session_cadence() {
    for (from, to, expected) in [
        ("stage: number", "stage: number = 1", "default zero"),
        (
            "player state Journal",
            "global state Journal",
            "player State number",
        ),
        (
            "from: 0, to: 2",
            "from: 2, to: 1",
            "expected a value from 3",
        ),
        ("every: 20", "every: 0", "expected a value from 1"),
        (
            "fn issue_rations()",
            "fn issue_rations(x: number)",
            "parameterless function",
        ),
    ] {
        let analysis = analyze_one(&QUEST.replace(from, to));
        let diagnostics = analysis
            .compile()
            .expect_err("invalid progression compiled");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "{diagnostics:?}"
        );
    }
}
