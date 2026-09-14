use sand_core::{ComponentFactory, DatapackRegistration, IntoDatapack, LootPool, LootTable};

fn invalid_loot_table() -> DatapackRegistration {
    LootTable::new("audit:invalid_loot".parse().unwrap())
        .pool(LootPool::new())
        .into_datapack()
}

sand_core::inventory::submit! {
    ComponentFactory { owner: "invalid_loot_table", make: invalid_loot_table }
}

#[test]
fn invalid_loot_table_fails_the_normal_export_before_record_creation() {
    let error = sand_core::try_export_components_json("audit").unwrap_err();
    let message = error.to_string();
    assert!(message.contains("audit:invalid_loot"));
    assert!(message.contains("loot_table"));
    assert!(message.contains("pools[0].entries"));
}
