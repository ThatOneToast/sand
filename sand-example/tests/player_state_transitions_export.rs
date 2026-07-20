//! Export coverage for the common player-state transition events (#49):
//! movement/posture, gamemode, health, and status-effect transitions all
//! sharing the generic tracked-transition provider backend.

use sand_example::player_state_transitions_example::{
    on_enter_creative, on_exit_creative, on_health_changed, on_low_health, on_recovered_health,
    on_speed_start, on_speed_stop, on_start_sprinting, on_start_sprinting_second_handler,
    on_stop_sprinting,
};

fn records(json: &str) -> Vec<serde_json::Value> {
    serde_json::from_str(json).unwrap()
}

fn function<'a>(records: &'a [serde_json::Value], path: &str) -> &'a str {
    records
        .iter()
        .find(|record| record["dir"] == "function" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing function {path}"))
}

fn tracker_functions(records: &[serde_json::Value]) -> Vec<(&str, &str)> {
    records
        .iter()
        .filter_map(|record| {
            if record["dir"] != "function" {
                return None;
            }
            let path = record["path"].as_str()?;
            if !path.starts_with("__sand_transition/") {
                return None;
            }
            Some((path, record["content"].as_str().unwrap_or_default()))
        })
        .collect()
}

fn load_commands(records: &[serde_json::Value]) -> &str {
    function(records, "__sand_lifecycle_load")
}

#[test]
fn player_state_transitions_export_deterministically() {
    assert!(!on_start_sprinting().is_empty());
    assert!(!on_start_sprinting_second_handler().is_empty());
    assert!(!on_stop_sprinting().is_empty());
    assert!(!on_enter_creative().is_empty());
    assert!(!on_exit_creative().is_empty());
    assert!(!on_health_changed().is_empty());
    assert!(!on_low_health().is_empty());
    assert!(!on_recovered_health().is_empty());
    assert!(!on_speed_start().is_empty());
    assert!(!on_speed_stop().is_empty());

    let first = sand_core::try_export_components_json("playerstatepack").unwrap();
    let second = sand_core::try_export_components_json("playerstatepack").unwrap();
    assert_eq!(
        first, second,
        "repeated player-state transition export must be byte-identical"
    );
    let records = records(&first);

    // Sprinting: two start handlers + one stop handler share exactly one
    // generated tracker (one provider per state, not per handler).
    let trackers = tracker_functions(&records);
    let sprint_tracker = trackers
        .iter()
        .find(|(_, content)| content.contains("player_sprinting"))
        .unwrap_or_else(|| panic!("no tracker references player_sprinting"));
    let sprint_content = sprint_tracker.1;
    assert_eq!(
        sprint_content
            .lines()
            .filter(|line| line.contains("on_start_sprinting"))
            .count(),
        2,
        "both sprint-start handlers must be called from the shared tracker"
    );
    assert_eq!(
        sprint_content
            .lines()
            .filter(|line| line.ends_with("function playerstatepack:on_stop_sprinting"))
            .count(),
        1
    );
    assert!(sprint_content.contains("predicate playerstatepack:__sand/player_sprinting"));

    // Gamemode: enter/exit creative share one tracker keyed on
    // `entity @s[gamemode=creative]`.
    let gamemode_tracker = trackers
        .iter()
        .find(|(_, content)| content.contains("gamemode=creative"))
        .unwrap_or_else(|| panic!("no tracker references gamemode=creative"));
    assert!(gamemode_tracker.1.contains("on_enter_creative"));
    assert!(gamemode_tracker.1.contains("on_exit_creative"));

    // Health: change/low/recovered handlers share the auto-declared
    // `sand_health` objective and the `health` criterion is only declared
    // once, deterministically, in the load function.
    let load = load_commands(&records);
    assert_eq!(
        load.lines()
            .filter(|line| *line == "scoreboard objectives add sand_health health")
            .count(),
        1,
        "the health criterion objective must be declared exactly once"
    );

    let health_tracker = trackers
        .iter()
        .find(|(_, content)| content.contains("on_health_changed"))
        .unwrap_or_else(|| panic!("no tracker references on_health_changed"));
    assert!(health_tracker.1.contains("sand_health"));

    let low_health_tracker = trackers
        .iter()
        .find(|(_, content)| content.contains("on_low_health"))
        .unwrap_or_else(|| panic!("no tracker references on_low_health"));
    assert!(
        low_health_tracker
            .1
            .contains("score @s sand_health matches ..6"),
        "low-health threshold must render the const-generic bound into the comparison: {}",
        low_health_tracker.1
    );
    assert!(low_health_tracker.1.contains("on_recovered_health"));

    // Status effects: start/stop share one tracker keyed on the generated
    // per-effect predicate, and the predicate JSON is emitted with the
    // correct vanilla effect ID.
    let speed_tracker = trackers
        .iter()
        .find(|(_, content)| content.contains("on_speed_start"))
        .unwrap_or_else(|| panic!("no tracker references on_speed_start"));
    assert!(speed_tracker.1.contains("on_speed_stop"));
    assert!(
        speed_tracker
            .1
            .contains("predicate playerstatepack:__sand/effect_speed")
    );

    let effect_predicate = records
        .iter()
        .find(|record| record["dir"] == "predicate" && record["path"] == "__sand/effect_speed")
        .unwrap_or_else(|| panic!("missing effect_speed predicate"));
    assert_eq!(
        effect_predicate["content"]
            .as_str()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap(),
        serde_json::json!({
            "condition": "minecraft:entity_properties",
            "entity": "this",
            "predicate": { "effects": { "minecraft:speed": {} } },
        })
    );
}
