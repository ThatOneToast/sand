mod support;

use sand::advanced::compiler::Compiler;
use serde_json::{Value, json};
use support::*;

const SOURCE: &str = include_str!("fixtures/terrain_features.sand");

#[test]
fn configured_features_and_placements_share_ids_across_authored_resources() {
    for (version, registry) in [("26.2", "configured_feature"), ("26.3", "feature")] {
        let analysis = analyze_one(&SOURCE.replace("26.2", version));
        let pack = compile(&analysis);
        let resource = |path: &str| -> Value {
            serde_json::from_slice(&pack.resources[&format!("data/terrains/worldgen/{path}.json")])
                .unwrap()
        };
        let feature = resource(&format!("{registry}/markers"));
        assert_eq!(feature["type"], "minecraft:simple_block");
        let config = if version == "26.2" {
            &feature["config"]
        } else {
            &feature
        };
        let (provider, block) = if version == "26.2" {
            (
                "minecraft:weighted_state_provider",
                json!({"Name": "minecraft:oak_log", "Properties": {"axis": "x"}}),
            )
        } else {
            (
                "minecraft:weighted",
                json!({"id": "minecraft:oak_log", "properties": {"axis": "x"}}),
            )
        };
        assert_eq!(config["to_place"]["type"], provider);
        assert_eq!(
            config["to_place"]["entries"][0],
            json!({"data": block, "weight": 2})
        );
        assert_eq!(
            resource(&format!("{registry}/glass"))["type"],
            "minecraft:fill_layer"
        );
        assert_eq!(
            resource(&format!("{registry}/quiet"))["type"],
            "minecraft:no_op"
        );
        assert_eq!(
            resource(&format!("{registry}/ore"))["type"],
            "minecraft:ore"
        );
        assert_eq!(
            resource("placed_feature/surface"),
            json!({
                "feature": "terrains:markers",
                "placement": [
                    {"type": "minecraft:count", "count": 30},
                    {"type": "minecraft:in_square"},
                    {"type": "minecraft:heightmap", "heightmap": "WORLD_SURFACE_WG"},
                    {"type": "minecraft:biome"}
                ]
            })
        );
        assert_eq!(
            resource("placed_feature/buried")["placement"][1],
            json!({
                "type": "minecraft:height_range",
                "height": {
                    "type": "minecraft:trapezoid", "min_inclusive": {"above_bottom": 4},
                    "max_inclusive": {"absolute": 48}, "plateau": 8
                }
            })
        );
        assert_eq!(
            resource("placed_feature/floating")["feature"],
            "terrains:markers"
        );
        assert_eq!(
            resource("biome/foothills")["features"][9],
            json!(["terrains:surface", "terrains:floating"])
        );
        let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
        let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
        assert_eq!(
            pack.resources,
            Compiler::compile(&decoded).unwrap().resources
        );
    }
}

#[test]
fn terrain_features_report_invalid_algorithms_and_placements() {
    for (original, replacement, expected) in [
        (
            "no_op: true",
            "no_op: true, simple_block: { block: Block::Stone }",
            "choose exactly one configured feature kind",
        ),
        (
            "weight: 2",
            "weight: 2147483647",
            "sum of block-state weights",
        ),
        (
            "feature: Markers",
            "feature: Highland",
            "expected an authored configured feature name",
        ),
        ("count: 30", "count: 4097", "expected a value from 0"),
        (
            "heightmap: world_surface_wg",
            "heightmap: ocean_floor_typo",
            "expected a heightmap",
        ),
        (
            "block: Block::Glass, height: 10",
            "height: 10",
            "needs `block`",
        ),
        (
            "max: { absolute: 48 }",
            "max: { above_bottom: 2 }",
            "min_inclusive must not exceed",
        ),
    ] {
        let analysis = analyze_one(&SOURCE.replacen(original, replacement, 1));
        let messages = analysis
            .compile()
            .unwrap_err()
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>();
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "{messages:?}"
        );
    }
}
