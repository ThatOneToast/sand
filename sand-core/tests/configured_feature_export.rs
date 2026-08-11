use sand_core::ComponentFactory;
use sand_core::prelude::*;
use sand_core::sand_components::worldgen::providers::{BlockState, BlockStateProvider};

fn ashen_shrub() -> ConfiguredFeature {
    ConfiguredFeature::simple_block(
        ResourceLocation::new("configured_feature_export", "ashen_shrub").unwrap(),
        BlockStateProvider::simple(BlockState::new(BlockId::minecraft("fern").unwrap())),
    )
}

fn ashen_shrub_placement() -> PlacedFeature {
    PlacedFeature::new(
        ResourceLocation::new("configured_feature_export", "ashen_shrub").unwrap(),
        ashen_shrub().id(),
    )
    .placement_modifier(serde_json::json!({"type": "minecraft:count", "count": 3}))
}

inventory::submit! {
    ComponentFactory { make: || Box::new(ashen_shrub()) }
}

inventory::submit! {
    ComponentFactory { make: || Box::new(ashen_shrub_placement()) }
}

#[test]
fn configured_feature_exports_under_worldgen_configured_feature_directory() {
    let records = sand_core::try_export_components("configured_feature_export")
        .expect("export should succeed");

    let feature_record = records
        .iter()
        .find(|record| {
            record.namespace == "configured_feature_export"
                && record.dir == "worldgen/configured_feature"
                && record.path == "ashen_shrub"
        })
        .expect("configured feature record should be exported");
    assert_eq!(feature_record.ext, "json");
    let feature_json: serde_json::Value = serde_json::from_str(&feature_record.content).unwrap();
    assert_eq!(feature_json["type"], "minecraft:simple_block");
    assert_eq!(
        feature_json["config"]["to_place"]["state"]["Name"],
        "minecraft:fern"
    );

    let placed_record = records
        .iter()
        .find(|record| {
            record.namespace == "configured_feature_export"
                && record.dir == "worldgen/placed_feature"
                && record.path == "ashen_shrub"
        })
        .expect("placed feature record should be exported");
    let placed_json: serde_json::Value = serde_json::from_str(&placed_record.content).unwrap();
    assert_eq!(
        placed_json["feature"],
        "configured_feature_export:ashen_shrub"
    );
}
