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
