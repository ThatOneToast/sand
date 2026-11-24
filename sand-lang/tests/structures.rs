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
