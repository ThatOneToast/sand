use sand_example::gameplay_state_example::{
    cache_selected_item, enraged_tick, start_enrage, stop_fighting,
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

#[test]
fn example_connects_data_inventory_schema_and_state_flow_deterministically() {
    // Force the linked functions into this integration binary and verify the
    // typed authoring boundary before inspecting the real exporter.
    assert!(!start_enrage().is_empty());
    assert!(!stop_fighting().is_empty());
    assert!(!enraged_tick().is_empty());
    assert_eq!(
        cache_selected_item(),
        vec!["data modify storage boss_phases:cache last_item set from entity @s SelectedItem"]
    );

    let first = sand_core::try_export_components_json("hello_world").unwrap();
    let second = sand_core::try_export_components_json("hello_world").unwrap();
    assert_eq!(first, second);
    let records = records(&first);

    let root_record = records
        .iter()
        .find(|record| {
            record["dir"] == "function"
                && record["path"].as_str().is_some_and(|path| {
                    path.starts_with("__sand_transition/flow_")
                        && !path["__sand_transition/".len()..].contains('/')
                })
        })
        .expect("generated state-flow root");
    let root = root_record["content"].as_str().unwrap();
    assert!(root.contains("if score @s boss_health_pct matches ..0"));
    assert!(root.contains("if score @s boss_health_pct matches ..50"));
    assert!(root.find("matches ..0").unwrap() < root.find("matches ..50").unwrap());

    let flow_prefix = root_record["path"].as_str().unwrap();
    let transition = function(&records, &format!("{flow_prefix}/transition_1"));
    let exit = transition
        .find("function hello_world:stop_fighting")
        .unwrap();
    let write = transition
        .find("scoreboard players set @s boss_phase 2")
        .unwrap();
    let enter = transition
        .find("function hello_world:start_enrage")
        .unwrap();
    assert!(exit < write && write < enter);

    assert_eq!(
        function(&records, "cache_selected_item"),
        "data modify storage boss_phases:cache last_item set from entity @s SelectedItem"
    );
    assert!(
        function(&records, "__sand_lifecycle_load")
            .contains("scoreboard objectives add boss_phase dummy")
    );
    assert!(
        function(&records, "__sand_lifecycle_tick")
            .contains("execute as @a at @s run function hello_world:__sand_transition/flow_")
    );
}
