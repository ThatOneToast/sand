use sand::component::{Biome, BiomeEffects};
use sand::prelude::*;
use sand_core::{ComponentFactory, IntoDatapack};
use std::collections::BTreeMap;

inventory::submit! {
    ComponentFactory {
        owner: module_path!(),
        make: || Biome::new(
            ResourceLocation::new("biome_spawn_export", "forest").unwrap(),
            BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF),
        ).spawners(BTreeMap::from([
            (MobCategory::Creature, vec![SpawnEntry::new(EntityTypeId::minecraft("pig").unwrap(), 10, 2, 4)]),
            (MobCategory::Monster, vec![SpawnEntry::new(EntityTypeId::minecraft("zombie").unwrap(), 90, 1, 3)]),
        ])).spawn_costs([
            (EntityTypeId::minecraft("zombie").unwrap(), SpawnCost::new(0.7, 0.15)),
        ]).into_datapack(),
    }
}

#[test]
fn spawn_tables_survive_the_component_export_pipeline_deterministically() {
    let export = || sand_core::try_export_components_json("biome_spawn_export").unwrap();
    let first = export();
    assert_eq!(first, export());
    let records = sand_core::try_export_components("biome_spawn_export").unwrap();
    let biome = records
        .iter()
        .find(|r| r.dir == "worldgen/biome" && r.path == "forest")
        .unwrap();
    assert_eq!(biome.namespace, "biome_spawn_export");
    let json: serde_json::Value = serde_json::from_str(&biome.content).unwrap();
    assert_eq!(
        json["spawners"],
        serde_json::json!({
            "creature": [{"type": "minecraft:pig", "weight": 10, "minCount": 2, "maxCount": 4}],
            "monster": [{"type": "minecraft:zombie", "weight": 90, "minCount": 1, "maxCount": 3}],
        })
    );
    assert_eq!(
        json["spawn_costs"],
        serde_json::json!({
            "minecraft:zombie": {"energy_budget": 0.7, "charge": 0.15},
        })
    );
}
