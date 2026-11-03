use sand::prelude::*;

#[function]
fn direct_storage_handle() {
    let value = KeyedData::<i32>::new("standalone:state", "components.manual.value");
    value.set(7);
    value.get();
}

#[test]
fn standalone_keyed_handles_receive_canonical_upgrade_lifecycle() {
    let first = sand_core::try_export_components_json("standalone_pack").unwrap();
    assert_eq!(
        first,
        sand_core::try_export_components_json("standalone_pack").unwrap()
    );
    let records: serde_json::Value = serde_json::from_str(&first).unwrap();
    let records = records.as_array().unwrap();
    let body = |path: &str| {
        records.iter().find(|r| r["path"] == path).unwrap()["content"]
            .as_str()
            .unwrap()
    };
    let objective = ObjectiveName::logical("standalone:state::keyed_migration");
    assert_eq!(
        body("__sand_lifecycle_load")
            .matches(&format!("scoreboard objectives add {objective} dummy"))
            .count(),
        1
    );
    assert!(body("__sand_lifecycle_tick").contains(&format!(
        "function standalone_pack:sand/state_data/migrate/{objective}/step"
    )));
    let all = records
        .iter()
        .filter_map(|r| r["content"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("unless data storage standalone:state owners run data modify storage standalone:state __sand_keyed set value {version:2}"));
    assert!(all.contains("if data storage standalone:state owners run data modify storage standalone:state __sand_keyed set value {version:1,cursor:0}"));
    assert!(!all.contains("__sand_local:"));
    assert!(!all.contains("owners[{uuid:"));
}
