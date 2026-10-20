use std::fs;
use std::process::Command;

#[test]
fn exported_builder_signatures_use_the_explicit_raw_json_boundary() {
    let output_dir = tempfile::tempdir().expect("create catalog output directory");
    let output_path = output_dir.path().join("api.json");
    let output = Command::new(env!("CARGO_BIN_EXE_sand"))
        .args([
            "api",
            "export",
            "--output",
            output_path.to_str().expect("UTF-8 temporary path"),
        ])
        .output()
        .expect("run API catalog export");
    assert!(
        output.status.success(),
        "catalog export failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json = fs::read_to_string(output_path).expect("read exported API catalog");
    let catalog: serde_json::Value = serde_json::from_str(&json).expect("parse API catalog JSON");
    let entries = catalog["entries"]
        .as_array()
        .expect("catalog entries are an array");

    for path in [
        "sand::component::BiomeEffects::particle",
        "sand::component::BiomeEffects::mood_sound",
        "sand::component::BiomeEffects::additions_sound",
        "sand::component::BiomeEffects::music",
        "sand::component::Biome::raw_carvers",
        "sand::component::Biome::raw_features",
        "sand::component::Biome::raw_spawners",
        "sand::component::Biome::raw_spawn_costs",
        "sand::component::ChatDecoration::style_raw",
        "sand::component::PlacementModifier::Raw",
        "sand::build::Generator::Raw",
        "sand::build::BiomeSource::Raw",
        "sand::component::TradeItem::components_raw",
        "sand::component::VillagerTrade::modify_given_item_raw",
        "sand::component::VillagerTrade::offered_when_raw",
        "sand::component::SpawnCondition::biomes_raw",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["canonical_path"] == path)
            .unwrap_or_else(|| panic!("raw boundary API is absent from the catalog: {path}"));
        let signature = entry["signature"]
            .as_str()
            .expect("catalog signature is a string");
        assert!(
            signature.contains("RawJson"),
            "{path} must expose RawJson in its installed signature: {signature}"
        );
        assert!(
            !signature.contains("serde_json::Value"),
            "{path} leaks serde_json::Value through the installed signature: {signature}"
        );
    }

    for path in [
        "sand::component::PlacedFeature::placement_modifier",
        "sand::component::PlacedFeature::placement",
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["canonical_path"] == path)
            .unwrap_or_else(|| panic!("typed placement method is absent from the catalog: {path}"));
        let signature = entry["signature"]
            .as_str()
            .expect("catalog signature is a string");
        assert!(
            signature.contains("PlacementModifier"),
            "{path}: {signature}"
        );
        assert!(
            !signature.contains("RawJson"),
            "{path} must expose typed modifiers: {signature}"
        );
        assert!(
            !signature.contains("Value"),
            "{path} leaks JSON: {signature}"
        );
    }

    for (path, model) in [
        ("sand::component::Biome::spawners", "SpawnEntry"),
        ("sand::component::Dimension::new", "Generator"),
        (
            "sand::component::Dimension::new_raw_dimension_type",
            "Generator",
        ),
        ("sand::component::Dimension::generator", "Generator"),
        (
            "sand::component::Dimension::noise_generator",
            "NoiseGenerator",
        ),
        (
            "sand::component::Dimension::flat_generator",
            "FlatGenerator",
        ),
        ("sand::component::Biome::spawn_costs", "SpawnCost"),
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry["canonical_path"] == path)
            .unwrap_or_else(|| panic!("typed biome method absent: {path}"));
        let signature = entry["signature"].as_str().unwrap();
        assert!(signature.contains(model), "{path}: {signature}");
        assert!(
            !signature.contains("RawJson") && !signature.contains("Value"),
            "{path}: {signature}"
        );
    }

    let raw_json_new = entries
        .iter()
        .find(|entry| entry["canonical_path"] == "sand::component::RawJson::new")
        .expect("RawJson constructor is the intentional serde JSON boundary");
    assert!(
        raw_json_new["signature"]
            .as_str()
            .expect("RawJson::new signature is a string")
            .contains("Value")
    );
}
