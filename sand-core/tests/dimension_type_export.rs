use sand_core::ComponentFactory;
use sand_core::prelude::*;

fn skylands_type() -> DimensionType {
    DimensionType::overworld_like(
        ResourceLocation::new("dimension_type_export", "skylands").unwrap(),
    )
}

inventory::submit! {
    ComponentFactory { make: || Box::new(skylands_type()) }
}

#[test]
fn dimension_type_exports_to_the_top_level_dimension_type_directory() {
    let records =
        sand_core::try_export_components("dimension_type_export").expect("export should succeed");
    let record = records
        .iter()
        .find(|record| {
            record.namespace == "dimension_type_export"
                && record.dir == "dimension_type"
                && record.path == "skylands"
        })
        .expect("dimension type record should be exported");

    assert_eq!(record.ext, "json");
    assert!(
        !record.dir.starts_with("worldgen/"),
        "dimension types are not nested under worldgen"
    );
    let json: serde_json::Value = serde_json::from_str(&record.content).unwrap();
    assert_eq!(json["effects"], "minecraft:overworld");
}
