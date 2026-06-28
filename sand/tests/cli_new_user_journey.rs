//! Regression tests for the documented new-user journey: `sand new` → `sand build`.
//!
//! # Tiers
//!
//! **Fast (always run)** — call `write_scaffold_files` directly to inspect
//! generated file content without invoking `cargo build`.
//!
//! **End-to-end (feature-gated)** — invoke the compiled `sand` binary, run
//! `sand new` and `sand build` in a temporary directory, and assert that the
//! expected datapack output is present.  Enable with:
//!
//! ```sh
//! cargo test -p sand --features integration-tests --test cli_new_user_journey
//! ```
//!
//! The end-to-end tests require a Rust toolchain and network access (for the
//! Mojang version manifest fetched by the scaffolded project's `build.rs`).

#[cfg(feature = "integration-tests")]
use std::path::Path;
use std::path::PathBuf;

use sand::scaffold::{ScaffoldOptions, name_to_namespace, validate_name, write_scaffold_files};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build `ScaffoldOptions` pointing at `dir` with sensible test defaults.
/// Uses path deps so the scaffolded project builds against local workspace
/// crates (required for the end-to-end tests).
fn test_opts(name: &str, dir: PathBuf) -> ScaffoldOptions {
    ScaffoldOptions {
        name: name.to_owned(),
        namespace: name_to_namespace(name),
        description: "Test datapack".to_owned(),
        mc_version: "1.21.4".to_owned(),
        dir,
        resourcepack: false,
        use_path_deps: true,
    }
}

/// Scaffold files into a fresh subdirectory of a tempdir and return both the
/// tempdir (kept alive) and the project path.
fn scaffold_in_tempdir(name: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let project_dir = tmp.path().join(name);
    let opts = test_opts(name, project_dir.clone());
    write_scaffold_files(&opts).expect("write_scaffold_files failed");
    (tmp, project_dir)
}

// ── Name validation (quick sanity re-check at integration boundary) ───────────

#[test]
fn valid_project_names_pass_validation() {
    for name in ["my_pack", "hello-world", "pack123", "a"] {
        assert!(validate_name(name).is_ok(), "expected '{name}' to be valid");
    }
}

#[test]
fn invalid_project_names_are_rejected() {
    for name in ["", "MyPack", "1pack", "my pack", "my.pack"] {
        assert!(
            validate_name(name).is_err(),
            "expected '{name}' to be rejected"
        );
    }
}

#[test]
fn hyphen_in_name_becomes_underscore_namespace() {
    assert_eq!(name_to_namespace("my-pack"), "my_pack");
    assert_eq!(name_to_namespace("hello-world-pack"), "hello_world_pack");
    assert_eq!(name_to_namespace("my_pack"), "my_pack");
}

// ── Scaffold file structure ───────────────────────────────────────────────────

#[test]
fn sand_new_creates_all_required_files() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let required = [
        "Cargo.toml",
        "build.rs",
        "sand.toml",
        "src/lib.rs",
        "src/bin/sand_export.rs",
    ];
    for rel in required {
        let path = project.join(rel);
        assert!(
            path.exists(),
            "expected '{rel}' to exist after `sand new`, but it was missing"
        );
        assert!(
            path.metadata().map(|m| m.len()).unwrap_or(0) > 0,
            "'{rel}' must not be empty"
        );
    }
}

#[test]
fn sand_new_does_not_create_resourcepack_files_by_default() {
    let (_tmp, project) = scaffold_in_tempdir("no_rp_pack");

    assert!(
        !project.join("src/bin/sand_resource_export.rs").exists(),
        "sand_resource_export.rs must not exist when resourcepack=false"
    );
    assert!(
        !project.join("src/assets").exists(),
        "src/assets must not exist when resourcepack=false"
    );
}

#[test]
fn sand_new_with_resourcepack_creates_extra_files() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("rp_pack");
    let opts = ScaffoldOptions {
        name: "rp_pack".to_owned(),
        namespace: "rp_pack".to_owned(),
        description: "RP test".to_owned(),
        mc_version: "1.21.4".to_owned(),
        dir: project_dir.clone(),
        resourcepack: true,
        use_path_deps: true,
    };
    write_scaffold_files(&opts).unwrap();

    assert!(
        project_dir.join("src/bin/sand_resource_export.rs").exists(),
        "sand_resource_export.rs must be created when resourcepack=true"
    );
    assert!(
        project_dir.join("src/assets").is_dir(),
        "src/assets must be created when resourcepack=true"
    );
}

// ── sand.toml content ─────────────────────────────────────────────────────────

#[test]
fn generated_sand_toml_has_correct_namespace_and_version() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("sand.toml")).unwrap();
    assert!(
        content.contains("namespace   = \"my_pack\""),
        "sand.toml must contain the correct namespace"
    );
    assert!(
        content.contains("mc_version  = \"1.21.4\""),
        "sand.toml must contain the mc_version"
    );
    assert!(
        !content.contains("[resourcepack]"),
        "sand.toml must not contain [resourcepack] section by default"
    );
}

#[test]
fn generated_sand_toml_with_resourcepack_has_rp_section() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("rp_pack");
    let opts = ScaffoldOptions {
        name: "rp_pack".to_owned(),
        namespace: "rp_pack".to_owned(),
        description: "RP test".to_owned(),
        mc_version: "1.21.4".to_owned(),
        dir: project_dir.clone(),
        resourcepack: true,
        use_path_deps: true,
    };
    write_scaffold_files(&opts).unwrap();

    let content = std::fs::read_to_string(project_dir.join("sand.toml")).unwrap();
    assert!(
        content.contains("[resourcepack]"),
        "sand.toml must contain [resourcepack] section when resourcepack=true"
    );
}

#[test]
fn generated_sand_toml_is_valid_toml_parseable() {
    let (_tmp, project) = scaffold_in_tempdir("toml_check");

    let content = std::fs::read_to_string(project.join("sand.toml")).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "sand.toml must be valid TOML, got: {:?}",
        parsed.err()
    );

    let table = parsed.unwrap();
    let ns = table["pack"]["namespace"].as_str().unwrap();
    assert_eq!(ns, "toml_check");
    let ver = table["pack"]["mc_version"].as_str().unwrap();
    assert_eq!(ver, "1.21.4");
}

// ── Cargo.toml content ────────────────────────────────────────────────────────

#[test]
fn generated_cargo_toml_has_required_binary_targets() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(
        content.contains("[[bin]]"),
        "Cargo.toml must declare at least one [[bin]] target"
    );
    assert!(
        content.contains("sand_export"),
        "Cargo.toml must declare the sand_export binary"
    );
    assert!(
        content.contains("src/bin/sand_export.rs"),
        "sand_export binary must point to src/bin/sand_export.rs"
    );
}

#[test]
fn generated_cargo_toml_uses_path_deps_when_requested() {
    let (_tmp, project) = scaffold_in_tempdir("path_dep_pack");

    let content = std::fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(
        content.contains("path ="),
        "Cargo.toml must use path deps when use_path_deps=true"
    );
    assert!(
        content.contains("sand-core"),
        "Cargo.toml must depend on sand-core"
    );
    assert!(
        content.contains("sand-build"),
        "Cargo.toml must depend on sand-build (build-dep)"
    );
}

#[test]
fn generated_cargo_toml_is_valid_toml_parseable() {
    let (_tmp, project) = scaffold_in_tempdir("toml_parse_pack");

    let content = std::fs::read_to_string(project.join("Cargo.toml")).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(
        parsed.is_ok(),
        "Cargo.toml must be valid TOML, got: {:?}",
        parsed.err()
    );

    let pkg_name = parsed.unwrap()["package"]["name"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(pkg_name, "toml_parse_pack");
}

// ── src/lib.rs content ────────────────────────────────────────────────────────

#[test]
fn generated_lib_rs_has_required_sand_exports() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();

    assert!(
        content.contains("pub fn __sand_export"),
        "src/lib.rs must define __sand_export"
    );
    assert!(
        content.contains("sand_core::export_components_json"),
        "src/lib.rs must call export_components_json"
    );
    assert!(
        content.contains("use sand_core::prelude::*"),
        "src/lib.rs must import sand_core prelude"
    );
}

#[test]
fn generated_lib_rs_has_starter_function_and_component() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();

    assert!(
        content.contains("#[function]"),
        "src/lib.rs must have at least one #[function]"
    );
    assert!(
        content.contains("#[component]"),
        "src/lib.rs must have at least one #[component]"
    );
    assert!(
        content.contains("hello_world"),
        "starter function hello_world must be present"
    );
    assert!(
        content.contains("Welcome to my_pack!"),
        "starter message must reference the pack name"
    );
}

#[test]
fn generated_lib_rs_uses_typed_commands_not_raw_mcfunction() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();

    assert!(
        !content.contains("mcfunction!"),
        "starter code must not use raw mcfunction! macro"
    );
    assert!(
        !content.contains("tellraw @"),
        "starter code must not use raw tellraw command strings"
    );
    assert!(
        content.contains("cmd::tellraw("),
        "starter code must use typed cmd::tellraw"
    );
    assert!(
        content.contains("Text::new("),
        "starter code must use typed Text::new"
    );
}

#[test]
fn generated_lib_rs_without_resourcepack_has_no_rp_export() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("src/lib.rs")).unwrap();

    assert!(
        !content.contains("\npub fn __sand_resource_export"),
        "src/lib.rs must not define __sand_resource_export when resourcepack=false"
    );
}

// ── src/bin/sand_export.rs content ───────────────────────────────────────────

#[test]
fn generated_sand_export_binary_calls_the_crate_export_fn() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("src/bin/sand_export.rs")).unwrap();

    assert!(
        content.contains("__sand_export"),
        "sand_export.rs must call __sand_export"
    );
    assert!(
        content.contains("my_pack"),
        "sand_export.rs must reference the crate name"
    );
}

// ── build.rs content ─────────────────────────────────────────────────────────

#[test]
fn generated_build_rs_calls_sand_build_generate() {
    let (_tmp, project) = scaffold_in_tempdir("my_pack");

    let content = std::fs::read_to_string(project.join("build.rs")).unwrap();

    assert!(
        content.contains("sand_build::generate("),
        "build.rs must call sand_build::generate"
    );
    assert!(
        content.contains("1.21.4"),
        "build.rs must embed the mc_version"
    );
}

// ── Namespace derived from hyphenated name ────────────────────────────────────

#[test]
fn hyphenated_pack_name_produces_underscore_namespace_in_files() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("my-pack");
    let opts = ScaffoldOptions {
        name: "my-pack".to_owned(),
        namespace: name_to_namespace("my-pack"),
        description: "Hyphen test".to_owned(),
        mc_version: "1.21.4".to_owned(),
        dir: project_dir.clone(),
        resourcepack: false,
        use_path_deps: true,
    };
    write_scaffold_files(&opts).unwrap();

    let sand_toml = std::fs::read_to_string(project_dir.join("sand.toml")).unwrap();
    assert!(
        sand_toml.contains("namespace   = \"my_pack\""),
        "namespace must use underscores even when pack name uses hyphens"
    );

    let lib_rs = std::fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
    assert!(
        lib_rs.contains("my_pack"),
        "lib.rs must reference the underscore namespace, not the hyphenated name"
    );
}

// ── End-to-end CLI journey ────────────────────────────────────────────────────
//
// These tests invoke the compiled `sand` binary and verify the full user
// journey from `sand new` to `sand build`.  They require:
//   - A working Rust toolchain on PATH
//   - Network access (for the Mojang version manifest)
//
// Run with:
//   cargo test -p sand --features integration-tests --test cli_new_user_journey

#[cfg(feature = "integration-tests")]
mod end_to_end {
    use super::*;

    /// Path to the compiled `sand` binary (set by Cargo when building tests).
    fn sand_bin() -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_sand"))
    }

    /// Run a command, return its output, and panic with a clear message on failure.
    fn run(cmd: &mut std::process::Command) -> std::process::Output {
        let output = cmd
            .output()
            .unwrap_or_else(|e| panic!("failed to spawn {:?}: {e}", cmd.get_program()));
        if !output.status.success() {
            panic!(
                "command {:?} failed (exit {:?})\n--- stdout ---\n{}\n--- stderr ---\n{}",
                cmd.get_program(),
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
        output
    }

    /// Assert that `path` exists and contains `needle` as a substring.
    fn assert_file_contains(path: &Path, needle: &str) {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        assert!(
            content.contains(needle),
            "expected '{}' to contain {:?}, but got:\n{}",
            path.display(),
            needle,
            content
        );
    }

    #[test]
    fn sand_new_and_build_produce_valid_datapack() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");

        // ── 1. sand new my_pack ───────────────────────────────────────────────
        let mut new_cmd = std::process::Command::new(sand_bin());
        new_cmd
            .args(["new", "my_pack", "--mc-version", "1.21.4", "--path-deps"])
            .current_dir(tmp.path())
            .env("RUST_LOG", "") // silence any log spam
            .env("NO_COLOR", "1"); // suppress ANSI in output
        run(&mut new_cmd);

        let project = tmp.path().join("my_pack");
        assert!(
            project.is_dir(),
            "sand new must create the project directory"
        );

        // Verify the expected file layout before building.
        for rel in [
            "Cargo.toml",
            "build.rs",
            "sand.toml",
            "src/lib.rs",
            "src/bin/sand_export.rs",
        ] {
            assert!(
                project.join(rel).exists(),
                "expected '{rel}' after `sand new`"
            );
        }

        // ── 2. sand build ─────────────────────────────────────────────────────
        let mut build_cmd = std::process::Command::new(sand_bin());
        build_cmd
            .arg("build")
            .current_dir(&project)
            .env("NO_COLOR", "1");
        run(&mut build_cmd);

        // ── 3. Verify datapack output ─────────────────────────────────────────
        let dist = project.join("dist/my_pack");
        assert!(dist.is_dir(), "dist/my_pack/ must exist after `sand build`");

        // pack.mcmeta must be valid JSON with the right pack_format.
        let mcmeta_path = dist.join("pack.mcmeta");
        assert!(mcmeta_path.exists(), "dist/my_pack/pack.mcmeta must exist");
        let mcmeta_raw = std::fs::read_to_string(&mcmeta_path).unwrap();
        let mcmeta: serde_json::Value =
            serde_json::from_str(&mcmeta_raw).expect("pack.mcmeta must be valid JSON");
        assert!(
            mcmeta["pack"]["pack_format"].is_number(),
            "pack.mcmeta must contain a numeric pack_format"
        );

        // Namespace folder and function output.
        let data_ns = dist.join("data/my_pack");
        assert!(
            data_ns.is_dir(),
            "data/my_pack/ namespace folder must exist"
        );

        let hello_fn = data_ns.join("function/hello_world.mcfunction");
        assert!(
            hello_fn.exists(),
            "data/my_pack/function/hello_world.mcfunction must be written by `sand build`"
        );
        assert_file_contains(&hello_fn, "tellraw");

        // Advancement output.
        let advancement = data_ns.join("advancement/player_join.json");
        assert!(
            advancement.exists(),
            "data/my_pack/advancement/player_join.json must be written by `sand build`"
        );
        let adv_value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&advancement).unwrap())
                .expect("player_join.json must be valid JSON");
        // Advancement JSON must have a criteria section.
        assert!(
            adv_value.get("criteria").is_some(),
            "player_join.json must have a 'criteria' field"
        );
    }
}
