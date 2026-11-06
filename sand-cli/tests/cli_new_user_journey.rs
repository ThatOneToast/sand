//! Focused regression coverage for project scaffolding and the `sand add`
//! capability boundary.

use std::path::{Path, PathBuf};

use sand_cli::scaffold::{ScaffoldOptions, name_to_namespace, validate_name, write_scaffold_files};

fn test_opts(name: &str, dir: PathBuf) -> ScaffoldOptions {
    ScaffoldOptions {
        name: name.to_owned(),
        namespace: name_to_namespace(name),
        description: "Test datapack".to_owned(),
        mc_version: "26.2".to_owned(),
        dir,
        use_path_deps: true,
    }
}

fn scaffold_in_tempdir(name: &str) -> (tempfile::TempDir, PathBuf) {
    let temporary = tempfile::tempdir().expect("create temp directory");
    let project = temporary.path().join(name);
    write_scaffold_files(&test_opts(name, project.clone())).expect("write scaffold");
    (temporary, project)
}

fn sand_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sand"))
}

fn write_minimal_project(root: &Path) {
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"capability_test\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("sand.toml"),
        "[pack]\nnamespace = \"capability_test\"\ndescription = \"test\"\nmc_version = \"26.2\"\n",
    )
    .unwrap();
}

fn directory_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries = walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let path = entry.path().strip_prefix(root).unwrap().to_owned();
            let contents = std::fs::read(entry.path()).unwrap();
            (path, contents)
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

#[test]
fn project_name_validation_is_table_driven() {
    for name in ["my_pack", "hello-world", "pack123", "a"] {
        assert!(validate_name(name).is_ok(), "expected {name:?} to be valid");
    }
    for name in ["", "MyPack", "1pack", "my pack", "my.pack"] {
        assert!(
            validate_name(name).is_err(),
            "expected {name:?} to be invalid"
        );
    }
    assert_eq!(name_to_namespace("hello-world"), "hello_world");
}

#[test]
fn scaffold_is_a_parseable_current_datapack_project() {
    let (_temporary, project) = scaffold_in_tempdir("my-pack");
    for relative in [
        "Cargo.toml",
        "build.rs",
        "sand.toml",
        "src/lib.rs",
        "src/bin/sand_export.rs",
    ] {
        let path = project.join(relative);
        assert!(
            path.is_file() && path.metadata().unwrap().len() > 0,
            "missing {relative}"
        );
    }

    let manifest = std::fs::read_to_string(project.join("Cargo.toml")).unwrap();
    let config = std::fs::read_to_string(project.join("sand.toml")).unwrap();
    let library = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();
    let exporter = std::fs::read_to_string(project.join("src/bin/sand_export.rs")).unwrap();
    let build_script = std::fs::read_to_string(project.join("build.rs")).unwrap();

    toml::from_str::<toml::Value>(&manifest).expect("generated Cargo.toml parses");
    toml::from_str::<toml::Value>(&config).expect("generated sand.toml parses");
    assert!(config.contains("namespace   = \"my_pack\"") && config.contains("26.2"));
    assert!(manifest.contains("sand = { path") && manifest.contains("sand-build"));
    assert!(library.contains("#[function]") && library.contains("#[on_event]"));
    assert!(library.contains("cmd::tellraw(") && !library.contains("mcfunction!"));
    assert!(
        !library.contains("StorageVar"),
        "the starter must not suggest global storage is per-player State"
    );
    assert!(exporter.contains("__sand_export") && exporter.contains("SAND_EXPORT_MC_VERSION"));
    assert!(build_script.contains("sand_build::generate(") && build_script.contains("26.2"));
}

#[test]
fn new_rejects_pre_26_without_creating_a_project() {
    let temporary = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(sand_bin())
        .args(["new", "old_pack", "--mc-version", "25.9", "--path-deps"])
        .current_dir(temporary.path())
        .env("NO_COLOR", "1")
        .output()
        .expect("run sand new with a pre-26 target");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Sand targets Minecraft Java 26.x and newer")
    );
    assert!(!temporary.path().join("old_pack").exists());
}

#[test]
fn add_resourcepack_is_opt_in_idempotent_and_preserves_authored_assets() {
    let temporary = tempfile::tempdir().unwrap();
    write_minimal_project(temporary.path());
    let run = || {
        std::process::Command::new(sand_bin())
            .args(["add", "resourcepack"])
            .current_dir(temporary.path())
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        temporary
            .path()
            .join("resourcepack/assets/capability_test")
            .is_dir()
    );
    let text = std::fs::read_to_string(temporary.path().join("sand.toml")).unwrap();
    let config: sand_cli::config::SandConfig = toml::from_str(&text).unwrap();
    assert_eq!(
        config.resource_pack.unwrap().source.as_str(),
        "resourcepack"
    );
    let asset = temporary
        .path()
        .join("resourcepack/assets/capability_test/sounds.json");
    std::fs::write(asset, b"{}").unwrap();
    let before = directory_snapshot(temporary.path());
    assert!(run().status.success());
    assert_eq!(directory_snapshot(temporary.path()), before);
}

#[test]
fn add_resourcepack_rejects_unverified_targets_before_creating_files() {
    let temporary = tempfile::tempdir().unwrap();
    write_minimal_project(temporary.path());
    let path = temporary.path().join("sand.toml");
    std::fs::write(
        &path,
        std::fs::read_to_string(&path)
            .unwrap()
            .replace("26.2", "27.0"),
    )
    .unwrap();
    let before = directory_snapshot(temporary.path());
    let output = std::process::Command::new(sand_bin())
        .args(["add", "resourcepack"])
        .current_dir(temporary.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("resource pack format is not verified")
    );
    assert_eq!(directory_snapshot(temporary.path()), before);
}

#[test]
fn add_worldbuild_still_scaffolds_the_typed_build_script() {
    let temporary = tempfile::tempdir().unwrap();
    write_minimal_project(temporary.path());

    let output = std::process::Command::new(sand_bin())
        .args(["add", "worldbuild"])
        .current_dir(temporary.path())
        .env("NO_COLOR", "1")
        .output()
        .expect("run sand add worldbuild");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temporary.path().join("sand.build.rs").is_file());
    assert!(
        std::fs::read_to_string(temporary.path().join("Cargo.toml"))
            .unwrap()
            .contains("name = \"sand_build_world\"")
    );
}

/// The single compile-heavy fixture protects the complete new → build →
/// generated-datapack boundary. Enable it in the consolidated CI test pass.
#[cfg(feature = "integration-tests")]
#[test]
fn new_and_build_produce_a_valid_26_2_datapack() {
    let temporary = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(sand_bin())
        .args(["new", "my_pack", "--mc-version", "26.2", "--path-deps"])
        .current_dir(temporary.path())
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project = temporary.path().join("my_pack");
    let output = std::process::Command::new(sand_bin())
        .arg("build")
        .current_dir(&project)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let root = project.join("dist/my_pack");
    let metadata: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("pack.mcmeta")).unwrap()).unwrap();
    assert_eq!(metadata["pack"]["pack_format"], 107);
    assert!(
        root.join("data/my_pack/function/hello_world.mcfunction")
            .is_file()
    );
    assert!(
        root.join("data/my_pack/function/__sand_join_check.mcfunction")
            .is_file()
    );
}

#[cfg(unix)]
#[test]
fn add_resourcepack_rejects_source_links_without_rewriting_configuration() {
    use std::os::unix::fs::symlink;
    let temporary = tempfile::tempdir().unwrap();
    write_minimal_project(temporary.path());
    let external = tempfile::tempdir().unwrap();
    symlink(external.path(), temporary.path().join("resourcepack")).unwrap();
    let before = std::fs::read(temporary.path().join("sand.toml")).unwrap();
    let output = std::process::Command::new(sand_bin())
        .args(["add", "resourcepack"])
        .current_dir(temporary.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        std::fs::read(temporary.path().join("sand.toml")).unwrap(),
        before
    );
    assert!(external.path().read_dir().unwrap().next().is_none());
}
