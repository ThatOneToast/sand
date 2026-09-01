//! Golden-file snapshot tests for `sand_core::build`-generated JSON/text
//! output (issue #317 §3.4: "Snapshot tests for generated JSON output
//! (flat/void/noise/dimension/border resources)").
//!
//! Each fixture under `tests/fixtures/build/` is a byte-for-byte expected
//! rendering of one generator/resource kind. Run with `UPDATE_SNAPSHOTS=1`
//! to regenerate a fixture after a deliberate output-format change:
//!
//! ```sh
//! UPDATE_SNAPSHOTS=1 cargo test -p sand-core --test build_snapshots
//! ```
//!
//! This mirrors the project's existing golden-test convention (full ordered
//! output comparison, not substring matching — see
//! `docs/architecture/adr-001-crate-boundaries.md`'s "Test target" section)
//! without adding a new `insta`-style dependency.

use std::path::{Path, PathBuf};

use sand_components::resource_location::ResourceLocation;
use sand_core::build::{
    Dimension, DimensionSlot, DimensionType, Dimensions, FlatGenerator, FlatLayer, Generator,
    NoiseGenerator, SandBuild, Spawn, VanillaNoiseSettings, World, WorldBorder, lower_world,
};

fn rl(namespace: &str, path: &str) -> ResourceLocation {
    ResourceLocation::new(namespace, path).unwrap()
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/build")
}

/// Renders `actual` as pretty JSON and compares it against the named
/// fixture file, byte-for-byte.
fn assert_json_snapshot(name: &str, actual: &serde_json::Value) {
    let rendered = serde_json::to_string_pretty(actual).unwrap() + "\n";
    assert_text_snapshot(name, &rendered);
}

/// Compares `actual` text against the named fixture file, byte-for-byte.
fn assert_text_snapshot(name: &str, actual: &str) {
    let path = fixtures_dir().join(name);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read snapshot fixture '{}': {error}\n\
             (run `UPDATE_SNAPSHOTS=1 cargo test -p sand-core --test build_snapshots` \
             to create it)",
            path.display()
        )
    });
    assert_eq!(
        actual,
        expected,
        "snapshot '{}' differs from the fixture at {}\n\
         (if this is a deliberate output-format change, rerun with \
         UPDATE_SNAPSHOTS=1 to regenerate it)",
        name,
        path.display()
    );
}

#[test]
fn flat_dimension_snapshot() {
    let dim = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
        Generator::Flat(
            FlatGenerator::new(vec![
                FlatLayer::new(rl("minecraft", "bedrock"), 1),
                FlatLayer::new(rl("minecraft", "dirt"), 2),
                FlatLayer::new(rl("minecraft", "grass_block"), 1),
            ])
            .biome(rl("minecraft", "plains")),
        ),
    );
    let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(dim)));
    let resources = lower_world("snapshot_pack", &build);
    let dim_resource = resources.iter().find(|r| r.dir == "dimension").unwrap();
    let json: serde_json::Value = serde_json::from_str(&dim_resource.content).unwrap();
    assert_json_snapshot("flat_dimension.json", &json);
}

#[test]
fn void_dimension_snapshot() {
    let dim = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
        .generator(Generator::Void);
    let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(dim)));
    let resources = lower_world("snapshot_pack", &build);
    let dim_resource = resources.iter().find(|r| r.dir == "dimension").unwrap();
    let json: serde_json::Value = serde_json::from_str(&dim_resource.content).unwrap();
    assert_json_snapshot("void_dimension.json", &json);
}

#[test]
fn noise_dimension_snapshot() {
    let dim = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
        Generator::Noise(NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)),
    );
    let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(dim)));
    let resources = lower_world("snapshot_pack", &build);
    let dim_resource = resources.iter().find(|r| r.dir == "dimension").unwrap();
    let json: serde_json::Value = serde_json::from_str(&dim_resource.content).unwrap();
    assert_json_snapshot("noise_dimension.json", &json);
}

#[test]
fn noise_single_biome_dimension_snapshot() {
    let dim = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
        Generator::Noise(
            NoiseGenerator::vanilla(VanillaNoiseSettings::Overworld)
                .single_biome(rl("minecraft", "desert")),
        ),
    );
    let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(dim)));
    let resources = lower_world("snapshot_pack", &build);
    let dim_resource = resources.iter().find(|r| r.dir == "dimension").unwrap();
    let json: serde_json::Value = serde_json::from_str(&dim_resource.content).unwrap();
    assert_json_snapshot("noise_single_biome_dimension.json", &json);
}

#[test]
fn custom_dimension_slot_and_type_snapshot() {
    let dim = Dimension::new(
        DimensionSlot::Custom(rl("snapshot_pack", "sky_realm")),
        DimensionType::Custom(rl("snapshot_pack", "sky_realm_type")),
    )
    .generator(Generator::Void);
    let build = SandBuild::new().world(World::new().dimensions(Dimensions::new().with(dim)));
    let resources = lower_world("snapshot_pack", &build);
    let dim_resource = resources.iter().find(|r| r.dir == "dimension").unwrap();
    assert_eq!(dim_resource.namespace, "snapshot_pack");
    assert_eq!(dim_resource.path, "sky_realm");
    let json: serde_json::Value = serde_json::from_str(&dim_resource.content).unwrap();
    assert_json_snapshot("custom_dimension.json", &json);
}

#[test]
fn world_border_and_spawn_init_function_snapshot() {
    let build = SandBuild::new().world(
        World::new()
            .spawn(Spawn::at(0, 65, 0).platform(rl("minecraft", "stone"), 4))
            .border(
                WorldBorder::diameter(6000.0)
                    .center(100.0, -50.0)
                    .damage_per_block(1.0)
                    .warning_distance(10)
                    .warning_time(30),
            )
            .gamerule("keepInventory", "true")
            .gamerule("doMobSpawning", "false"),
    );
    let resources = lower_world("snapshot_pack", &build);
    let init_fn = resources
        .iter()
        .find(|r| r.dir == "function" && r.path == "__sand_world_init")
        .unwrap();
    assert_text_snapshot("world_init.mcfunction", &init_fn.content);

    let load_tag = resources
        .iter()
        .find(|r| r.namespace == "minecraft" && r.dir == "tags/function")
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&load_tag.content).unwrap();
    assert_json_snapshot("load_tag.json", &json);
}
