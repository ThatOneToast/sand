//! World queries reuse canonical typed targets and execution scoping.
use sand::prelude::*;

#[sand_macros::function]
fn world_targets() {
    WorldContext::current().in_dimension(
        "minecraft:the_nether".parse::<sand::registry::DimensionId>().unwrap(),
        |world| {
            mcfunction![
                world.players().gamemode(GameMode::Survival).each(|_| {
                    world.block(BlockPos::here()).replace(vanilla::Block::GoldBlock)
                });
                world.entities().entity_type(vanilla::EntityType::ArmorStand).tag("probe").within_blocks(32.0).each(|_| {
                    world.block(BlockPos::here()).replace(vanilla::Block::Stone)
                });
            ]
        },
    );
}

#[test]
fn world_queries_retain_dimension_player_capabilities_and_entity_positions() {
    let json = sand_core::try_export_components_json("world_targets").unwrap();
    assert_eq!(
        json,
        sand_core::try_export_components_json("world_targets").unwrap()
    );
    let records: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let functions: std::collections::BTreeMap<_, _> = records
        .iter()
        .filter(|r| r["dir"] == "function")
        .map(|r| (r["path"].as_str().unwrap(), r["content"].as_str().unwrap()))
        .collect();
    let scoped = functions["world_targets"]
        .strip_prefix("execute in minecraft:the_nether run function world_targets:")
        .unwrap();
    let lines: Vec<_> = functions[scoped].lines().collect();
    assert_eq!(lines.len(), 2);
    let player = lines[0]
        .strip_prefix(
            "execute as @a[gamemode=survival,distance=0..] at @s run function world_targets:",
        )
        .unwrap();
    assert_eq!(functions[player], "setblock ~ ~ ~ minecraft:gold_block");
    let entity = lines[1].strip_prefix("execute as @e[type=minecraft:armor_stand,tag=probe,distance=..32] at @s run function world_targets:").unwrap();
    assert_eq!(functions[entity], "setblock ~ ~ ~ minecraft:stone");
    let second = sand_core::try_export_components_json("other_world").unwrap();
    assert!(!second.contains("world_targets:"));
    assert!(!second.contains("__sand_local:"));
}
