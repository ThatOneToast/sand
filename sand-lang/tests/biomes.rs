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

const ORE_FEATURE: &str = r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
tag BaseStone: Block { values: [Block::Stone] }
biome Alpine { features: { underground_ores: [Amber] } }
ore_feature Amber {
    size: 12,
    count: 30,
    min_y: -16,
    max_y: 20,
    targets: [
        { replace: BaseStone, block: Block::GoldOre },
        { replace: Block::Deepslate, block: Block::DeepslateGoldOre },
    ],
    discard_exposed: 0.5,
}
"#;

#[test]
fn declared_ore_builds_both_native_resources_and_a_biome_reference() {
    use sand::advanced::compiler::Compiler;

    let analysis = analyze_one(ORE_FEATURE);
    let pack = compile(&analysis);
    let resource = |path: &str| -> serde_json::Value {
        serde_json::from_slice(&pack.resources[&format!("data/world_probe/worldgen/{path}.json")])
            .unwrap()
    };
    let ore = resource("configured_feature/amber");
    assert_eq!(ore["type"], "minecraft:ore");
    assert_eq!(ore["config"]["size"], 12);
    assert_eq!(ore["config"]["discard_chance_on_air_exposure"], 0.5);
    assert_eq!(
        ore["config"]["targets"],
        serde_json::json!([
            {
                "target": { "predicate_type": "minecraft:tag_match", "tag": "world_probe:base_stone" },
                "state": { "Name": "minecraft:gold_ore" },
            },
            {
                "target": { "predicate_type": "minecraft:block_match", "block": "minecraft:deepslate" },
                "state": { "Name": "minecraft:deepslate_gold_ore" },
            },
        ])
    );
    let placement = resource("placed_feature/amber");
    assert_eq!(placement["feature"], "world_probe:amber");
    assert_eq!(
        placement["placement"],
        serde_json::json!([
            {"type": "minecraft:count", "count": 30},
            {"type": "minecraft:in_square"},
            {"type": "minecraft:height_range", "height": {
                "type": "minecraft:uniform",
                "min_inclusive": {"absolute": -16},
                "max_inclusive": {"absolute": 20},
            }},
            {"type": "minecraft:biome"},
        ])
    );
    assert_eq!(
        resource("biome/alpine")["features"][6],
        serde_json::json!(["world_probe:amber"])
    );

    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn ore_validation_rejects_unsupported_counts_heights_and_target_types() {
    for (original, invalid, expected) in [
        (
            "count: 30",
            "count: 4097",
            "expected a value from 0 to 4096",
        ),
        ("max_y: 20", "max_y: -17", "maximum placement height"),
        (
            "discard_exposed: 0.5",
            "discard_exposed: 1.5",
            "discard probability",
        ),
        (
            "block: Block::GoldOre",
            "block: Item::GoldIngot",
            "expected a `Block`",
        ),
    ] {
        let messages = errors(&analyze_one(&ORE_FEATURE.replacen(original, invalid, 1)));
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "{messages:?}"
        );
    }
    let mut program = analyze_one(ORE_FEATURE).program.unwrap();
    program.modules[0].ore_features[0].count = -1;
    let diagnostics = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|error| error.pointer == "/ore_features/0"
                && error.message.contains("count must be in 0..=4096"))
    );
}
