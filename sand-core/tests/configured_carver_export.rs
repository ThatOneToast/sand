use sand_core::ComponentFactory;
use sand_core::prelude::*;
use sand_core::sand_components::worldgen::providers::{HeightProvider, VerticalAnchor};

fn shallow_cave() -> ConfiguredCarver {
    ConfiguredCarver::cave(
        ResourceLocation::new("configured_carver_export", "shallow_cave").unwrap(),
        CaveCarverConfig::new(
            0.15,
            HeightProvider::absolute(0),
            CarverFloatRange::new(0.1, 0.9),
            VerticalAnchor::Absolute(-54),
        ),
    )
}

fn shallow_caves_biome() -> sand_core::Biome {
    sand_core::Biome::new(
        ResourceLocation::new("configured_carver_export", "shallow_caves").unwrap(),
        sand_core::BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF),
    )
    .carver_step(CarvingStep::Air, shallow_cave().id())
}

inventory::submit! {
    ComponentFactory { make: || Box::new(shallow_cave()) }
}

inventory::submit! {
    ComponentFactory { make: || Box::new(shallow_caves_biome()) }
}

#[test]
fn configured_carver_exports_under_worldgen_configured_carver_directory() {
    let records = sand_core::try_export_components("configured_carver_export")
        .expect("export should succeed");

    let carver_record = records
        .iter()
        .find(|record| {
            record.namespace == "configured_carver_export"
                && record.dir == "worldgen/configured_carver"
                && record.path == "shallow_cave"
        })
        .expect("configured carver record should be exported");
    assert_eq!(carver_record.ext, "json");
    let carver_json: serde_json::Value = serde_json::from_str(&carver_record.content).unwrap();
    assert_eq!(carver_json["type"], "minecraft:cave");
    assert_eq!(carver_json["config"]["probability"], 0.15);

    let biome_record = records
        .iter()
        .find(|record| {
            record.namespace == "configured_carver_export"
                && record.dir == "worldgen/biome"
                && record.path == "shallow_caves"
        })
        .expect("biome record should be exported");
    let biome_json: serde_json::Value = serde_json::from_str(&biome_record.content).unwrap();
    assert_eq!(
        biome_json["carvers"]["air"][0],
        "configured_carver_export:shallow_cave"
    );
}
