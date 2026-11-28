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

#[test]
fn jigsaw_structures_connect_authored_templates_pools_biomes_and_sets() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let analysis = sand_lang::analyze_path(&fixtures.join("jigsaw_structures.sand"), &DiskFiles);
    let pack = compile(&analysis);
    let worldgen = |path: &str| -> serde_json::Value {
        serde_json::from_slice(&pack.resources[&format!("data/world_probe/worldgen/{path}.json")])
            .unwrap()
    };
    let pool = worldgen("template_pool/pieces");
    assert_eq!(pool["fallback"], "minecraft:empty");
    assert_eq!(
        pool["elements"],
        serde_json::json!([{
            "weight": 3,
            "element": {
                "element_type": "minecraft:single_pool_element",
                "location": "world_probe:pedestal",
                "processors": "minecraft:empty",
                "projection": "rigid",
            },
        }])
    );
    let structure = worldgen("structure/waystation");
    assert_eq!(structure["start_pool"], "world_probe:pieces");
    assert_eq!(structure["type"], "minecraft:jigsaw");
    assert_eq!(
        structure["biomes"],
        serde_json::json!(["world_probe:alpine"])
    );
    assert_eq!(structure["size"], 1);
    assert_eq!(structure["spawn_overrides"], serde_json::json!({}));
    assert_eq!(structure["terrain_adaptation"], "beard_thin");
    assert_eq!(structure["project_start_to_heightmap"], "WORLD_SURFACE_WG");
    assert_eq!(
        worldgen("structure_set/waystations")["structures"][0]["structure"],
        "world_probe:waystation",
    );
    assert_eq!(
        pack.resources["data/world_probe/structure/pedestal.nbt"],
        include_bytes!("fixtures/structures/pedestal.nbt")
    );

    let encoded = serde_json::to_vec(&analysis.program.unwrap()).unwrap();
    let decoded = Compiler::decode(&encoded, &Default::default()).unwrap();
    assert_eq!(
        pack.resources,
        Compiler::compile(&decoded).unwrap().resources
    );
}

#[test]
fn jigsaw_inputs_reject_mismatched_references_and_limits() {
    let source = include_str!("fixtures/jigsaw_structures.sand");
    for (old, invalid, expected) in [
        ("weight: 3", "weight: 151", "expected a value from 1 to 150"),
        (
            "start_pool: Pieces",
            "start_pool: Pedestal",
            "expected an authored template pool name",
        ),
        ("size: 1", "size: 0", "expected a value from 1 to 20"),
        ("beard_thin", "unknown", "unknown terrain adaptation"),
        (
            "structure: Waystation",
            "structure: Pieces",
            "expected an authored jigsaw structure name",
        ),
    ] {
        let messages = errors(&analyze_one(&source.replacen(old, invalid, 1)));
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "{messages:?}"
        );
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let analysis = sand_lang::analyze_path(&fixtures.join("jigsaw_structures.sand"), &DiskFiles);
    let mut program = analysis.program.unwrap();
    program.modules[0].template_pools[0].pieces[0].weight = 151;
    let diagnostics = Compiler::compile(&program).unwrap_err();
    assert!(diagnostics.iter().any(|error| {
        error.pointer == "/template_pools/0"
            && error
                .message
                .contains("pool entry weight must be at most 150")
    }));
}
