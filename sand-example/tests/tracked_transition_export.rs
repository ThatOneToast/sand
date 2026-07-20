use sand_example::tracked_sneaking_example::{
    on_start_sneaking, on_start_sneaking_audit, on_stop_sneaking,
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

fn tag_values(records: &[serde_json::Value], path: &str) -> Vec<String> {
    let content = records
        .iter()
        .find(|record| record["namespace"] == "minecraft" && record["path"] == path)
        .and_then(|record| record["content"].as_str())
        .unwrap_or_else(|| panic!("missing tag {path}"));
    serde_json::from_str::<serde_json::Value>(content).unwrap()["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn proof_events_share_one_lifecycle_managed_tracker_and_export_stably() {
    // Force the example module into this integration binary and verify normal
    // typed handler functions remain callable.
    assert!(!on_start_sneaking().is_empty());
    assert!(!on_start_sneaking_audit().is_empty());
    assert!(!on_stop_sneaking().is_empty());

    let first = sand_core::try_export_components_json("transitionpack").unwrap();
    let second = sand_core::try_export_components_json("transitionpack").unwrap();
    assert_eq!(
        first, second,
        "repeated transition export must be byte-identical"
    );
    let records = records(&first);

    // The `sand-example` crate registers other tracked-transition events too
    // (e.g. sprinting, gamemode) via `#[event]` handlers elsewhere in the
    // crate — this test only asserts on the sneaking-specific tracker, not
    // on the total count of trackers in the whole exported pack.
    let trackers: Vec<_> = records
        .iter()
        .filter(|record| {
            record["dir"] == "function"
                && record["path"]
                    .as_str()
                    .is_some_and(|path| path.starts_with("__sand_transition/"))
        })
        .collect();
    let sneaking_trackers: Vec<_> = trackers
        .iter()
        .filter(|record| {
            record["content"]
                .as_str()
                .is_some_and(|content| content.contains("player_sneaking"))
        })
        .collect();
    assert_eq!(
        sneaking_trackers.len(),
        1,
        "all sneaking handlers share one tracker"
    );
    let tracker_path = sneaking_trackers[0]["path"].as_str().unwrap();
    let tracker = sneaking_trackers[0]["content"].as_str().unwrap();

    for handler in [
        "on_start_sneaking",
        "on_start_sneaking_audit",
        "on_stop_sneaking",
    ] {
        assert_eq!(
            tracker
                .lines()
                .filter(|line| line.ends_with(&format!("function transitionpack:{handler}")))
                .count(),
            1
        );
    }
    let first_update = tracker.find("scoreboard players operation").unwrap();
    assert!(tracker.find("on_stop_sneaking").unwrap() < first_update);
    assert!(tracker.contains("if score @s __st_"));
    assert!(tracker.contains("predicate transitionpack:__sand/player_sneaking"));
    assert!(!tracker.contains("scoreboard players set @a"));

    let load = function(&records, "__sand_lifecycle_load");
    let sneaking_key = tracker_path
        .strip_prefix("__sand_transition/")
        .expect("tracker path has the expected prefix");
    assert_eq!(
        load.lines()
            .filter(|line| line.contains(&format!("__st_{sneaking_key}")))
            .count(),
        3,
        "sneaking's own tracker declares exactly 3 private objectives (previous/current/seen)"
    );
    let tick = function(&records, "__sand_lifecycle_tick");
    assert!(tick.contains(&format!(
        "execute as @a run function transitionpack:{tracker_path}"
    )));
    assert!(tag_values(&records, "load").contains(&"transitionpack:__sand_lifecycle_load".into()));
    assert!(tag_values(&records, "tick").contains(&"transitionpack:__sand_lifecycle_tick".into()));

    let predicate = records
        .iter()
        .find(|record| record["dir"] == "predicate" && record["path"] == "__sand/player_sneaking");
    assert!(
        predicate.is_some(),
        "proof event must emit the typed sneaking predicate"
    );

    // Existing advancement-backed output remains present and unchanged in shape.
    assert!(
        records
            .iter()
            .any(|record| { record["dir"] == "advancement" && record["path"] == "player_join" })
    );
}
