mod support;
use support::*;

#[test]
fn authored_biome_colors_and_climate_export_through_canonical_worldgen() {
    let analysis = analyze_one(include_str!("fixtures/custom_biomes.sand"));
    let pack = compile(&analysis);
    let resource = |path: &str| -> serde_json::Value {
        serde_json::from_slice(&pack.resources[path]).unwrap()
    };
    let biome = resource("data/language_probe/worldgen/biome/lantern_grove.json");
    assert_eq!(biome["has_precipitation"], false);
    assert_eq!(biome["temperature"], -0.25);
    assert!((biome["downfall"].as_f64().unwrap() - 0.65).abs() < 0.000_001);
    assert_eq!(
        biome["attributes"],
        serde_json::json!({
            "minecraft:visual/fog_color": "#8595a6",
            "minecraft:visual/water_fog_color": "#183f43",
            "minecraft:visual/sky_color": "#8d98a9",
        })
    );
    assert_eq!(
        biome["effects"],
        serde_json::json!({
            "water_color": "#246a73",
            "grass_color": "#648b72",
            "foliage_color": "#436a55",
        })
    );
    assert_eq!(
        biome["spawners"],
        serde_json::json!({
            "creature": [
                {"type": "minecraft:rabbit", "weight": 12, "minCount": 2, "maxCount": 4},
                {"type": "minecraft:sheep", "weight": 1, "minCount": 1, "maxCount": 1},
            ],
            "monster": [{"type": "minecraft:zombie", "weight": 3, "minCount": 1, "maxCount": 1}],
        }),
    );
    assert_eq!(
        biome["spawn_costs"],
        serde_json::json!({"minecraft:rabbit": {"energy_budget": 1.5, "charge": 0.25}}),
    );
    let dimension = resource("data/language_probe/dimension/grove.json");
    assert_eq!(
        dimension["generator"]["settings"]["biome"],
        "language_probe:lantern_grove"
    );
    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = sand::advanced::compiler::Compiler::decode(&portable, &Default::default())
        .expect("custom biomes decode through the portable compiler");
    let recompiled = sand::advanced::compiler::Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, recompiled.resources);
}

#[test]
fn custom_biomes_validate_climate_and_colors() {
    let invalid = r##"
#![pack(namespace = "world_probe", minecraft = "26.2")]
biome Broken { downfall: 1.25, sky_color: "#ffffff00" }
"##;
    let analysis = analyze_one(invalid);
    assert_eq!(
        spanned(&analysis, "downfall must be between zero and one"),
        "1.25"
    );
    assert_eq!(
        spanned(&analysis, "expected an RGB color"),
        r##""#ffffff00""##
    );

    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
biome Garden {}
"#;
    let analysis = analyze_one(source);
    let mut program = analysis.program.unwrap();
    program.modules[0].biomes[0].water_color = Some(0x0100_0000);
    let diagnostics = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains("water_color") && diagnostic.pointer == "/biomes/0"
    }));
}

#[test]
fn biome_spawn_tables_validate_groups_counts_and_references() {
    let source = include_str!("fixtures/custom_biomes.sand");
    for (original, replacement, expected) in [
        (
            "creature:",
            "critters:",
            "unknown mob spawn category `critters`",
        ),
        ("max_count: 4", "max_count: 1", "expected a value from 2"),
        ("Entity::Rabbit", "Block::Stone", "Entity"),
        ("weight: 12", "weight: 2147483647", "sum of spawn weights"),
    ] {
        let analysis = analyze_one(&source.replacen(original, replacement, 1));
        let messages: Vec<_> = analysis
            .compile()
            .expect_err("invalid biome compiled")
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect();
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "expected {expected:?}, got {messages:?}",
        );
    }
    let mut portable = serde_json::to_value(analyze_one(source).program.unwrap()).unwrap();
    portable["modules"][0]["biomes"][0]["spawners"]["monster"][0]["entity_type"] =
        serde_json::json!("probe:unknown");
    let program = serde_json::from_value(portable).unwrap();
    let diagnostics = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown entity type `probe:unknown`")
            && diagnostic
                .pointer
                .ends_with("/spawners/monster/0/entity_type")
    }));
}

#[test]
fn biome_decoration_keeps_placed_features_in_native_step_order() {
    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
biome Alpine {
    features: {
        vegetal_decoration: ["minecraft:trees_plains"],
        underground_ores: ["minecraft:ore_coal_upper", "minecraft:ore_iron_upper"],
    },
}
dimension Hills { biome: Alpine, noise_settings: "minecraft:overworld" }
"#;
    let analysis = analyze_one(source);
    let pack = compile(&analysis);
    let biome: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/world_probe/worldgen/biome/alpine.json"])
            .unwrap();
    let features = biome["features"].as_array().unwrap();
    assert_eq!(
        features[6],
        serde_json::json!(["minecraft:ore_coal_upper", "minecraft:ore_iron_upper"])
    );
    assert_eq!(features[9], serde_json::json!(["minecraft:trees_plains"]));
    assert_eq!(features[0], serde_json::json!([]));
    let program = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&program, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn biome_decoration_rejects_invalid_steps_and_feature_references() {
    for features in [
        r#"{ middle: ["minecraft:ore_iron_upper"] }"#,
        r#"{ underground_ores: "minecraft:ore_iron_upper" }"#,
        r#"{ underground_ores: ["ore_iron_upper"] }"#,
        r#"{ underground_ores: [Block::Stone] }"#,
        r#"{ underground_ores: [], underground_ores: [] }"#,
    ] {
        let source = format!(
            "#![pack(namespace = \"world_probe\")] biome Alpine {{ features: {features} }}"
        );
        assert!(analyze_one(&source).has_errors(), "{features}");
    }
}
