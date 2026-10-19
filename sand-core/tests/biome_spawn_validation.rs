//! Aggregate spawn weights must fail before an invalid biome reaches Minecraft.
use sand::component::{Biome, BiomeEffects, MobCategory, SpawnEntry};
use sand::registry::EntityTypeId;
use sand_core::{ComponentFactory, IntoDatapack};
use std::collections::BTreeMap;

inventory::submit! {
    ComponentFactory {
        owner: module_path!(),
        make: || Biome::new(
            "invalid_spawns:forest".parse().unwrap(),
            BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF),
        ).spawners(BTreeMap::from([(
            MobCategory::Monster, vec![
                SpawnEntry::new(EntityTypeId::minecraft("zombie").unwrap(), i32::MAX as u32, 1, 1),
                SpawnEntry::new(EntityTypeId::minecraft("skeleton").unwrap(), 1, 1, 1),
            ],
        )])).into_datapack(),
    }
}

#[test]
fn overflowing_spawn_weight_total_fails_component_export() {
    let error = sand_core::try_export_components_json("invalid_spawns")
        .expect_err("Minecraft rejects this category's total weight")
        .to_string();
    for expected in [
        "invalid_spawns:forest",
        "spawners.monster",
        "sum of spawn weights",
    ] {
        assert!(error.contains(expected), "{error}");
    }
}
