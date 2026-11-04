//! Companion assets remain separate, deterministic and owned across rebuilds.
use sand_cli::{
    build::{
        package::zip_dir,
        publication::{publish_pack, publish_packs},
        resource_pack::{self, ResourcePackFormat},
    },
    config::SandConfig,
};
use std::{collections::BTreeMap, path::Path};

fn root() -> tempfile::TempDir {
    tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
}
fn config(extra: &str) -> SandConfig {
    toml::from_str(&format!(
        "[pack]\nnamespace=\"demo\"\ndescription=\"Demo\"\nmc_version=\"26.3\"\n{extra}"
    ))
    .unwrap()
}
fn source(root: &Path) {
    std::fs::create_dir_all(root.join("resourcepack/assets/demo/models/item")).unwrap();
    std::fs::write(
        root.join("resourcepack/assets/demo/models/item/icon.json"),
        b"{\"parent\":\"minecraft:item/generated\"}\n",
    )
    .unwrap();
    std::fs::write(root.join("resourcepack/README.md"), b"Author guidance").unwrap();
}
#[test]
fn resource_formats_follow_verified_mojang_metadata_and_preserve_minor_versions() {
    for (version, expected) in [
        ("26.1", [84, 0]),
        ("26.1.2", [84, 0]),
        ("26.2", [88, 0]),
        ("26.3", [97, 1]),
        ("latest", [97, 1]),
    ] {
        assert_eq!(
            resource_pack::format_for(version, None).unwrap(),
            ResourcePackFormat::Version(expected)
        );
    }
    assert!(resource_pack::format_for("27.0", None).is_err());
    assert_eq!(
        resource_pack::format_for("27.0", Some(ResourcePackFormat::Version([105, 2]))).unwrap(),
        ResourcePackFormat::Version([105, 2])
    );
    assert!(resource_pack::format_for("26.3", Some(ResourcePackFormat::Major(0))).is_err());
    assert!(resource_pack::format_for("26.3", Some(ResourcePackFormat::Major(u32::MAX))).is_err());
    assert!(
        resource_pack::format_for("26.3", Some(ResourcePackFormat::Version([97, u32::MAX])))
            .is_err()
    );
}
#[test]
fn companion_output_preserves_asset_bytes_and_uses_its_own_exact_metadata() {
    let temp = root();
    source(temp.path());
    let config = config("[resource_pack]\nsource=\"resourcepack\"\n");
    let pack = resource_pack::prepare(temp.path(), &config, "26.3")
        .unwrap()
        .unwrap();
    assert_eq!(pack.destination, temp.path().join("dist/demo-resources"));
    assert_eq!(pack.asset_count, 1);
    assert_eq!(
        pack.resources["assets/demo/models/item/icon.json"],
        b"{\"parent\":\"minecraft:item/generated\"}\n"
    );
    assert!(!pack.resources.contains_key("README.md"));
    let metadata: serde_json::Value =
        serde_json::from_slice(&pack.resources["pack.mcmeta"]).unwrap();
    assert_eq!(metadata["pack"]["pack_format"], 97);
    assert_eq!(metadata["pack"]["min_format"], serde_json::json!([97, 1]));
    assert_eq!(metadata["pack"]["max_format"], serde_json::json!([97, 1]));
    assert_eq!(metadata["pack"]["description"], "Demo resources");
    let legacy = config_without_resources();
    assert!(
        resource_pack::prepare(temp.path(), &legacy, "26.3")
            .unwrap()
            .is_none()
    );
}
fn config_without_resources() -> SandConfig {
    config("")
}
#[test]
fn builds_are_deterministic_and_prune_only_owned_companion_assets() {
    let temp = root();
    source(temp.path());
    let config = config("[resource_pack]\nsource=\"resourcepack\"\n");
    let first = resource_pack::prepare(temp.path(), &config, "26.2")
        .unwrap()
        .unwrap();
    let datapack = temp.path().join("dist/demo");
    let data = BTreeMap::from([(
        "data/demo/function/load.mcfunction".into(),
        b"say demo\n".to_vec(),
    )]);
    publish_packs(&[(&datapack, &data), (&first.destination, &first.resources)]).unwrap();
    std::fs::write(first.destination.join("notes.txt"), b"Mine").unwrap();
    let archive = zip_dir(&first.destination, "demo-resources").unwrap();
    let bytes = std::fs::read(&archive).unwrap();
    let second = resource_pack::prepare(temp.path(), &config, "26.2")
        .unwrap()
        .unwrap();
    let published = publish_pack(&second.destination, &second.resources).unwrap();
    assert_eq!(published.summary.written, 0);
    assert_eq!(published.summary.unchanged, 2);
    assert_eq!(
        std::fs::read(zip_dir(&second.destination, "demo-resources").unwrap()).unwrap(),
        bytes
    );
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive).unwrap()).unwrap();
    assert!(zip.by_name("pack.mcmeta").is_ok());
    assert!(zip.by_name("assets/demo/models/item/icon.json").is_ok());
    assert!(zip.by_name(".sand-build-manifest.json").is_err());
    assert!(zip.by_name("data/demo/function/load.mcfunction").is_err());
    std::fs::remove_file(
        temp.path()
            .join("resourcepack/assets/demo/models/item/icon.json"),
    )
    .unwrap();
    let third = resource_pack::prepare(temp.path(), &config, "26.2")
        .unwrap()
        .unwrap();
    assert_eq!(
        publish_pack(&third.destination, &third.resources)
            .unwrap()
            .summary
            .removed,
        1
    );
    assert_eq!(
        std::fs::read(third.destination.join("notes.txt")).unwrap(),
        b"Mine"
    );
    assert_eq!(
        std::fs::read(datapack.join("data/demo/function/load.mcfunction")).unwrap(),
        b"say demo\n"
    );
}
#[test]
fn invalid_sources_fail_before_output_changes() {
    for (path, bytes) in [
        ("assets/demo/models/bad.json", b"{".as_slice()),
        ("pack.mcmeta", b"{}"),
        ("unexpected.txt", b"data"),
        ("assets/demo/Uppercase.json", b"{}"),
        ("pack.png", b"not a png"),
    ] {
        let temp = root();
        source(temp.path());
        let file = temp.path().join("resourcepack").join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
        let config = config("[resource_pack]\nsource=\"resourcepack\"\n");
        assert!(
            resource_pack::prepare(temp.path(), &config, "26.2").is_err(),
            "{path}"
        );
        assert!(!temp.path().join("dist").exists());
    }
    for value in ["../outside", "/tmp/outside", ".", "dist", "dist/resources"] {
        let text = format!(
            "[pack]\nnamespace=\"demo\"\ndescription=\"Demo\"\nmc_version=\"26.2\"\n[resource_pack]\nsource={value:?}\n"
        );
        let temp = root();
        if let Ok(config) = toml::from_str::<SandConfig>(&text) {
            assert!(
                resource_pack::prepare(temp.path(), &config, "26.2").is_err(),
                "{value}"
            );
        }
    }
}
#[cfg(unix)]
#[test]
fn source_and_release_zip_links_are_rejected_without_touching_the_target() {
    use std::os::unix::fs::symlink;
    let temp = root();
    source(temp.path());
    let config = config("[resource_pack]\nsource=\"resourcepack\"\n");
    let external = temp.path().join("external");
    std::fs::write(&external, b"private").unwrap();
    let link = temp.path().join("resourcepack/assets/demo/linked.bin");
    symlink(&external, &link).unwrap();
    assert!(resource_pack::prepare(temp.path(), &config, "26.2").is_err());
    std::fs::remove_file(link).unwrap();
    let pack = resource_pack::prepare(temp.path(), &config, "26.2")
        .unwrap()
        .unwrap();
    publish_pack(&pack.destination, &pack.resources).unwrap();
    symlink(&external, temp.path().join("dist/demo-resources.zip")).unwrap();
    assert!(zip_dir(&pack.destination, "demo-resources").is_err());
    assert_eq!(std::fs::read(&external).unwrap(), b"private");
    std::fs::remove_file(temp.path().join("dist/demo-resources.zip")).unwrap();
    let archive = zip_dir(&pack.destination, "demo-resources").unwrap();
    let before = std::fs::read(&archive).unwrap();
    symlink(&external, pack.destination.join("linked.bin")).unwrap();
    assert!(zip_dir(&pack.destination, "demo-resources").is_err());
    assert_eq!(std::fs::read(&archive).unwrap(), before);
    std::fs::remove_dir_all(temp.path().join("resourcepack")).unwrap();
    symlink(&pack.destination, temp.path().join("resourcepack")).unwrap();
    assert!(resource_pack::prepare(temp.path(), &config, "26.2").is_err());
}

#[test]
fn malformed_assets_produce_a_structured_build_error_before_cargo_or_publication() {
    let temp = root();
    source(temp.path());
    std::fs::write(temp.path().join("sand.toml"),"[pack]\nnamespace=\"demo\"\ndescription=\"Demo\"\nmc_version=\"26.3\"\n[resource_pack]\nsource=\"resourcepack\"\n").unwrap();
    std::fs::write(
        temp.path()
            .join("resourcepack/assets/demo/models/item/icon.json"),
        b"{",
    )
    .unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_sand"))
        .args(["build", "--format", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["success"], false);
    assert_eq!(
        response["diagnostics"][0]["code"],
        "SAND_BUILD_RESOURCE_PACK"
    );
    assert!(!temp.path().join("dist").exists());
}
