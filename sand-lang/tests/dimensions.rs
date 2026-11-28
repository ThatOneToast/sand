mod support;
use support::*;

#[test]
fn fixed_biome_noise_dimensions_round_trip_and_share_the_dimension_registry() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
dimension Highlands { noise_settings: "minecraft:overworld", biome: Biome::Desert }
"#,
    );
    let pack = compile(&analysis);
    let dimension: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/world_probe/dimension/highlands.json"])
            .unwrap();
    assert_eq!(
        dimension,
        serde_json::json!({
            "type": "minecraft:overworld",
            "generator": {
                "type": "minecraft:noise",
                "settings": "minecraft:overworld",
                "biome_source": { "type": "minecraft:fixed", "biome": "minecraft:desert" }
            }
        })
    );
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
fn checkerboard_noise_uses_ordered_vanilla_and_authored_biomes() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
biome Dry { temperature: 1.0, downfall: 0.1 }
dimension Tiles {
    noise_settings: "minecraft:overworld",
    biomes: [Biome::Desert, Dry, Biome::Forest],
    biome_scale: 3,
}
"#,
    );
    let pack = compile(&analysis);
    let dimension: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/world_probe/dimension/tiles.json"]).unwrap();
    assert_eq!(
        dimension["generator"]["biome_source"],
        serde_json::json!({
            "type": "minecraft:checkerboard",
            "biomes": ["minecraft:desert", "world_probe:dry", "minecraft:forest"],
            "scale": 3,
        })
    );
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
fn dimensions_require_one_generator_and_keep_flat_structure_settings_separate() {
    for fields in [
        r#"layers: [], biomes: [Biome::Desert]"#,
        r#"noise_settings: "minecraft:overworld", biomes: []"#,
        r#"noise_settings: "minecraft:overworld", biomes: [Biome::Desert], biome: Biome::Plains"#,
        r#"noise_settings: "minecraft:overworld", biomes: [Biome::Desert], biome_scale: 63"#,
        r#"noise_settings: "minecraft:overworld", biome_scale: 0"#,
        "",
        "layers: [], noise_settings: \"minecraft:overworld\"",
        "noise_settings: \"minecraft:overworld\", structures: true",
        "noise_settings: \"overworld\"",
    ] {
        let source = format!("#![pack(namespace = \"world_probe\")] dimension Test {{ {fields} }}");
        assert!(analyze_one(&source).has_errors(), "{fields}");
    }
}

#[test]
fn flat_dimension_export_and_owned_waits_use_the_authored_identity() {
    let analysis = analyze_one(include_str!("fixtures/flat_dimensions.sand"));
    let pack = compile(&analysis);
    let dimension: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/language_probe/dimension/workshop.json"])
            .unwrap();
    assert_eq!(
        dimension,
        serde_json::json!({
            "type": "minecraft:overworld",
            "generator": {
                "type": "minecraft:flat",
                "settings": {
                    "biome": "minecraft:desert",
                    "lakes": false,
                    "features": false,
                    "layers": [
                        {"block": "minecraft:bedrock", "height": 1},
                        {"block": "minecraft:sandstone", "height": 2},
                    ],
                    "structure_overrides": [],
                },
            },
        })
    );
    let run = function(&pack, "language_probe/function/run");
    assert!(run.contains("execute in language_probe:workshop run function "));
    assert!(pack.resources.iter().any(|(path, content)| {
        path.ends_with(".mcfunction")
            && String::from_utf8_lossy(content)
                .contains("execute if dimension language_probe:workshop")
    }));

    let portable = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = sand::advanced::compiler::Compiler::decode(&portable, &Default::default())
        .expect("flat dimensions decode through the portable compiler");
    let recompiled = sand::advanced::compiler::Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, recompiled.resources);
}

#[test]
fn dimensions_use_flat_generator_and_version_validation() {
    for (fields, expected) in [
        ("layers: []", "has no layers"),
        (
            "layers: [{ block: Block::Stone, height: 4065 }]",
            "exceeding the absolute flat-generator height limit",
        ),
    ] {
        let text = format!(
            "#![pack(namespace = \"world_probe\", minecraft = \"26.2\")]\ndimension Test {{ {fields} }}"
        );
        let analysis = analyze_one(&text);
        assert!(!analysis.has_errors(), "{:?}", errors(&analysis));
        let diagnostics = analysis.compile().unwrap_err();
        assert!(
            diagnostics.iter().any(|d| d.message.contains(expected)),
            "{diagnostics:?}"
        );
    }

    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.3")]
dimension Test { layers: [{ block: Block::Stone, height: 1 }], biome: Biome::DappledForest }
"#;
    let analysis = analyze_one(source);
    compile(&analysis);
    let mut program = analysis.program.unwrap();
    program.target.minecraft = "26.2".into();
    let diagnostics = sand::advanced::compiler::Compiler::compile(&program).unwrap_err();
    assert!(diagnostics.iter().any(|d| {
        d.message
            .contains("unknown biome `minecraft:dappled_forest`")
    }));

    let old_target = analyze_one(&source.replace("26.3", "26.2"));
    assert!(
        errors(&old_target)
            .iter()
            .any(|d| d.contains("not available in Minecraft 26.2"))
    );
}

#[test]
fn custom_dimension_rules_use_native_components_and_round_trip() {
    let analysis = analyze_one(
        r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
tag Burnable: Block { values: [Block::Netherrack] }
dimension_type Deep {
    has_skylight: false,
    has_ceiling: true,
    coordinate_scale: 2.0,
    min_y: -32,
    height: 256,
    ambient_light: 0.125,
    infiniburn: Burnable,
    monster_spawn_light_level: { min: 2, max: 5 },
    monster_spawn_block_light_limit: 3,
}
dimension Hollow { type: Deep, layers: [{ block: Block::Bedrock, height: 1 }] }
"#,
    );
    let pack = compile(&analysis);
    let rules: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/world_probe/dimension_type/deep.json"])
            .unwrap();
    assert_eq!(rules["has_skylight"], false);
    assert_eq!(rules["has_ceiling"], true);
    assert_eq!(rules["coordinate_scale"], 2.0);
    assert_eq!(rules["min_y"], -32);
    assert_eq!(rules["height"], 256);
    assert_eq!(rules["logical_height"], 256);
    assert_eq!(rules["ambient_light"], 0.125);
    assert_eq!(rules["infiniburn"], "#world_probe:burnable");
    assert_eq!(
        rules["monster_spawn_light_level"],
        serde_json::json!({
            "type": "minecraft:uniform", "min_inclusive": 2, "max_inclusive": 5,
        })
    );
    let world: serde_json::Value =
        serde_json::from_slice(&pack.resources["data/world_probe/dimension/hollow.json"]).unwrap();
    assert_eq!(world["type"], "world_probe:deep");
    let json = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = sand::advanced::compiler::Compiler::decode(&json, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources
    );
}

#[test]
fn custom_dimension_rules_report_invalid_values_instead_of_panicking() {
    for fields in [
        "min_y: -31",
        "height: 65",
        "height: 64, logical_height: 65",
        "ambient_light: 2.0",
        "infiniburn: Block::Stone",
        "monster_spawn_light_level: { min: 1 }",
        "monster_spawn_light_level: { min: 6, max: 4 }",
    ] {
        let source =
            format!("#![pack(namespace = \"world_probe\")] dimension_type Bad {{ {fields} }}");
        let analysis = analyze_one(&source);
        assert!(
            analysis.has_errors() || analysis.compile().is_err(),
            "{fields}"
        );
    }
}

#[test]
fn vanilla_noise_biomes_use_the_native_source_matching_the_terrain_settings() {
    let source = r#"
#![pack(namespace = "world_probe", minecraft = "26.2")]
dimension Home {
    noise_settings: "minecraft:overworld",
    biome_source: vanilla,
}
dimension Ember {
    type: DimensionType::TheNether,
    noise_settings: "minecraft:nether",
    biome_source: vanilla,
}
dimension Far {
    type: DimensionType::TheEnd,
    noise_settings: "minecraft:end",
    biome_source: vanilla,
}
"#;
    let analysis = analyze_one(source);
    let pack = compile(&analysis);
    for (id, expected) in [
        (
            "home",
            serde_json::json!({"type": "minecraft:multi_noise", "preset": "minecraft:overworld"}),
        ),
        (
            "ember",
            serde_json::json!({"type": "minecraft:multi_noise", "preset": "minecraft:nether"}),
        ),
        ("far", serde_json::json!({"type": "minecraft:the_end"})),
    ] {
        let path = format!("data/world_probe/dimension/{id}.json");
        let dimension: serde_json::Value = serde_json::from_slice(&pack.resources[&path]).unwrap();
        assert_eq!(dimension["generator"]["biome_source"], expected);
    }
    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded =
        sand::advanced::compiler::Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        sand::advanced::compiler::Compiler::compile(&decoded)
            .unwrap()
            .resources,
        pack.resources
    );
}

#[test]
fn a_named_vanilla_biome_source_excludes_overlapping_source_choices() {
    for input in [
        "layers: [{ block: Block::Stone, height: 1 }], biome_source: vanilla",
        "noise_settings: \"minecraft:nether\", biome: Biome::Plains, biome_source: vanilla",
        "noise_settings: \"minecraft:nether\", biomes: [Biome::Plains], biome_source: vanilla",
        "noise_settings: \"minecraft:nether\", biome_source: unknown",
    ] {
        let analysis = analyze_one(&format!(
            "#![pack(namespace = \"world_probe\")] dimension Broken {{ {input} }}"
        ));
        assert!(analysis.has_errors(), "{input}");
    }
}
