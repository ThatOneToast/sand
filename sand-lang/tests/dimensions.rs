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
fn dimensions_require_one_generator_and_keep_flat_structure_settings_separate() {
    for fields in [
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
