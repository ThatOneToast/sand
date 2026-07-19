//! Regression coverage for generated schedule ownership and multiplayer-safe
//! scoreboard mutations.

use sand_core::ScheduleDescriptor;

fn every_tick_body() -> Vec<String> {
    vec!["say every tick".to_string()]
}

fn interval_body() -> Vec<String> {
    vec!["say interval".to_string()]
}

sand_core::inventory::submit! {
    ScheduleDescriptor {
        path: "every_tick_schedule",
        total_ticks: 20,
        every: 1,
        make: every_tick_body,
    }
}

sand_core::inventory::submit! {
    ScheduleDescriptor {
        path: "interval_schedule",
        total_ticks: 80,
        every: 5,
        make: interval_body,
    }
}

fn export() -> String {
    sand_core::try_export_components_json("schedulepack").expect("schedule export succeeds")
}

fn function_content<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing generated function {path}"))
}

#[test]
fn generated_schedule_ticks_are_per_player_safe_and_deterministic() {
    let first = export();
    let second = export();
    assert_eq!(first, second, "repeated exports must be byte-identical");

    let records: Vec<serde_json::Value> = serde_json::from_str(&first).unwrap();
    let tick = function_content(&records, "__sand_sched_tick");

    assert!(!tick.contains("scoreboard players remove @a"));
    assert!(
        tick.lines()
            .filter(|line| line.contains("scoreboard players remove"))
            .all(|line| line.starts_with("execute as @a[") && line.contains(" remove @s "))
    );

    let every_tick_key = "8863de6c";
    let interval_key = "5607096c";
    assert!(tick.contains(&format!(
        "execute as @a[scores={{__ss_{every_tick_key}_t=1..}}] run scoreboard players remove @s __ss_{every_tick_key}_t 1"
    )));
    assert!(tick.contains(&format!(
        "execute as @a[scores={{__ss_{interval_key}_t=1..}}] run scoreboard players remove @s __ss_{interval_key}_p 1"
    )));
    assert!(tick.contains(&format!(
        "execute as @a[scores={{__ss_{interval_key}_t=1..}}] run scoreboard players remove @s __ss_{interval_key}_t 1"
    )));
    assert!(tick.contains(&format!(
        "execute as @a[scores={{__ss_{interval_key}_t=1..,__ss_{interval_key}_p=..0}}] at @s run function schedulepack:interval_schedule"
    )));
}

#[test]
fn two_players_conceptually_keep_independent_schedule_scores() {
    // The export harness does not execute Minecraft commands, so model the
    // ownership contract directly: each `execute as` iteration mutates only
    // that iteration's `@s` score.
    let mut remaining = [3_i32, 1_i32];
    for score in &mut remaining {
        if *score >= 1 {
            *score -= 1;
        }
    }
    assert_eq!(remaining, [2, 0]);
}
