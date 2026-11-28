mod support;

use sand::advanced::compiler::Compiler;
use sand_lang::DiskFiles;
use std::path::Path;
use support::*;

fn fixture() -> sand_lang::Analysis {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    sand_lang::analyze_path(&fixtures.join("structure_templates.sand"), &DiskFiles)
}

#[test]
fn structure_assets_export_losslessly_and_place_with_native_positions() {
    let analysis = fixture();
    let pack = compile(&analysis);
    let asset = include_bytes!("fixtures/structures/pedestal.nbt");
    assert_eq!(
        pack.resources["data/language_probe/structure/pedestal.nbt"],
        asset
    );
    let run = function(&pack, "language_probe/function/run");
    assert!(run.contains("place template language_probe:pedestal 4 80 1"));
    assert!(run.contains("place template language_probe:pedestal ~ ~ ~4"));

    let portable = serde_json::to_value(analysis.program.unwrap()).unwrap();
    assert!(
        portable["structure_assets"]["structures/pedestal.nbt"]
            .as_str()
            .unwrap()
            .starts_with("H4sI")
    );
    let decoded =
        Compiler::decode(&serde_json::to_vec(&portable).unwrap(), &Default::default()).unwrap();
    let recompiled = Compiler::compile(&decoded).unwrap();
    assert_eq!(pack.resources, recompiled.resources);
}

#[test]
fn structure_assets_require_local_nbt_files_and_explicit_binary_content() {
    for source in [
        "../private.nbt",
        "C:/private.nbt",
        "/tmp/private.nbt",
        "private.txt",
    ] {
        let text = format!(
            "#![pack(namespace = \"world_probe\")]\nstructure_template Broken {{ source: {source:?} }}"
        );
        let analysis = analyze_one(&text);
        assert!(
            errors(&analysis)
                .iter()
                .any(|error| error.contains("unsafe structure template source path")),
            "{:?}",
            errors(&analysis)
        );
    }

    let mut program = fixture().program.unwrap();
    program.structure_assets.clear();
    let missing = Compiler::compile(&program).unwrap_err();
    assert!(missing.iter().any(|error| {
        error.pointer == "/structure_templates/0/source"
            && error.message.contains("missing structure asset")
    }));

    program.structure_assets.insert(
        "structures/pedestal.nbt".into(),
        b"uncompressed text".to_vec(),
    );
    let invalid = Compiler::compile(&program).unwrap_err();
    assert!(invalid.iter().any(|error| {
        error.pointer == "/structure_templates/0/source"
            && error.message.contains("must be a gzip NBT file")
    }));
}

#[cfg(unix)]
#[test]
fn local_asset_symlinks_cannot_escape_the_package() {
    use std::{fs, os::unix::fs::symlink, time::SystemTime};

    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("sand-structure-{}-{stamp}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let package = root.join("package");
    fs::create_dir(&package).unwrap();
    fs::write(
        root.join("outside.nbt"),
        include_bytes!("fixtures/structures/pedestal.nbt"),
    )
    .unwrap();
    symlink("../outside.nbt", package.join("saved.nbt")).unwrap();
    let source = package.join("pack.sand");
    fs::write(
        &source,
        r#"#![pack(namespace = "structure_probe")]
structure_template Probe { source: "saved.nbt" }"#,
    )
    .unwrap();
    let analysis = sand_lang::analyze_path(&source, &DiskFiles);
    fs::remove_dir_all(&root).unwrap();

    assert!(
        errors(&analysis)
            .iter()
            .any(|error| error.contains("asset resolves outside the package")),
        "{:?}",
        errors(&analysis)
    );
}

#[test]
fn native_structure_sets_export_weighted_spread_and_rings() {
    let analysis = analyze_one(
        r##"
#![pack(namespace = "world_probe", minecraft = "26.2")]
structure_set Towns {
    structures: [
        { structure: "minecraft:village_plains", weight: 3 },
        { structure: "minecraft:village_desert" },
    ],
    random_spread: { spacing: 20, separation: 6, salt: 421, frequency: 0.25, triangular: true },
    exclusion: { set: "minecraft:pillager_outposts", chunks: 3 },
}
structure_set Rings {
    structures: [{ structure: "minecraft:stronghold" }],
    concentric_rings: { distance: 32, spread: 3, count: 16, preferred_biomes: ["#minecraft:stronghold_biased_to"] },
}
"##,
    );
    let pack = compile(&analysis);
    let towns: serde_json::Value = serde_json::from_slice(
        &pack.resources["data/world_probe/worldgen/structure_set/towns.json"],
    )
    .unwrap();
    assert_eq!(
        towns,
        serde_json::json!({
            "structures": [{"structure": "minecraft:village_plains", "weight": 3}, {"structure": "minecraft:village_desert", "weight": 1}],
            "placement": {
                "type": "minecraft:random_spread", "spacing": 20, "separation": 6, "salt": 421,
                "spread_type": "triangular", "frequency": 0.25,
                "exclusion_zone": {"other_set": "minecraft:pillager_outposts", "chunk_count": 3},
            }
        })
    );
    let rings: serde_json::Value = serde_json::from_slice(
        &pack.resources["data/world_probe/worldgen/structure_set/rings.json"],
    )
    .unwrap();
    assert_eq!(
        rings["placement"],
        serde_json::json!({
            "type": "minecraft:concentric_rings", "distance": 32, "spread": 3, "count": 16,
            "salt": 0, "preferred_biomes": "#minecraft:stronghold_biased_to",
        })
    );
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
fn structure_sets_diagnose_missing_rules_and_invalid_placement() {
    for fields in [
        "",
        r#"random_spread: { spacing: 16, separation: 16, salt: 1 }"#,
        r#"random_spread: { spacing: 16, separation: 2, salt: -1 }"#,
        r#"random_spread: { spacing: 16, salt: 1 }"#,
        r##"concentric_rings: { distance: 32, spread: 3, count: 16, preferred_biomes: ["#minecraft:stronghold_biased_to", Biome::Plains] }"##,
    ] {
        let source = format!(
            r#"#![pack(namespace = "world_probe")]
structure_set Invalid {{ structures: [{{ structure: "minecraft:village_plains" }}], {fields} }}"#
        );
        let analysis = analyze_one(&source);
        assert!(
            analysis.has_errors() || analysis.compile().is_err(),
            "{fields}"
        );
    }
}
