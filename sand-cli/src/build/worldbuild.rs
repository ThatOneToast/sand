//! Compilation and execution of a project's optional `sand.build.rs` typed
//! world/server configuration script (issue #317).
//!
//! Modeled on `export.rs`'s handling of `sand_export`/`sand_resource_export`:
//! `sand-cli` compiles a project-provided binary and parses one JSON value
//! from its stdout. `sand.build.rs` is wired in as an ordinary Cargo
//! `[[bin]]` target named `sand_build_world` (see `sand add worldbuild`),
//! rather than being discovered as a bare loose file — this reuses Cargo's
//! existing dependency resolution/compilation instead of Sand inventing a
//! second build system.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

/// The `[[bin]]` target name a project's `sand.build.rs` is wired to.
pub(super) const WORLDBUILD_BIN_NAME: &str = "sand_build_world";

/// One generated datapack file, as printed by the `sand_build_world`
/// binary. Mirrors `sand_core::build::WorldResource`'s JSON shape, which
/// mirrors `ComponentRecord`'s wire format.
#[derive(Debug, Deserialize)]
pub(super) struct WorldResourceRecord {
    pub namespace: String,
    pub dir: String,
    pub path: String,
    pub ext: String,
    /// Always `"text"` today (`sand_core::build::WorldResource` never
    /// produces anything else); kept for wire-format parity with
    /// `ComponentRecord` and forward compatibility.
    #[serde(default)]
    #[allow(dead_code)]
    pub content_type: String,
    pub content: String,
}

/// The `server_config` object a `sand_build_world` binary may print
/// alongside its resources. 🖥️ Server (host) only — consumed by `sand run`,
/// never written into `dist/<namespace>/...`.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ServerConfigRecord {
    pub view_distance: u8,
    pub simulation_distance: u8,
    #[serde(default)]
    pub difficulty: DifficultyRecord,
    pub online_mode: bool,
    pub world_reset_policy: bool,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum DifficultyRecord {
    Peaceful,
    Easy,
    #[default]
    Normal,
    Hard,
}

impl DifficultyRecord {
    pub fn as_str(self) -> &'static str {
        match self {
            DifficultyRecord::Peaceful => "peaceful",
            DifficultyRecord::Easy => "easy",
            DifficultyRecord::Normal => "normal",
            DifficultyRecord::Hard => "hard",
        }
    }
}

/// Everything one `sand_build_world` invocation prints, parsed. The `seed`
/// and `level_type` fields are 🖥️ Server (host) only despite coming from
/// `World::seed`/`World::preset` — both only take effect at world creation,
/// which `sand run`'s local bootstrap applies via `server.properties`.
#[derive(Debug, Deserialize)]
pub(super) struct WorldBuildOutput {
    pub(super) resources: Vec<WorldResourceRecord>,
    pub(super) server_config: Option<ServerConfigRecord>,
    /// `None` for an unset or `Seed::Random` world.
    #[serde(default)]
    pub(super) seed: Option<i64>,
    /// `None` only when the build configured no `World` at all; otherwise
    /// always `Some` (defaults to `"minecraft:normal"` for an unset
    /// preset, matching vanilla's own default).
    #[serde(default)]
    pub(super) level_type: Option<String>,
}

/// Whether this project has a `sand.build.rs` wired in as the
/// `sand_build_world` binary target.
pub fn project_has_worldbuild(project_root: &Path) -> bool {
    let cargo_toml = project_root.join("Cargo.toml");
    let Ok(src) = std::fs::read_to_string(&cargo_toml) else {
        return false;
    };
    src.contains(WORLDBUILD_BIN_NAME)
}

/// Compiles the `sand_build_world` binary.
pub(super) fn compile(project_root: &Path, mc_version: &str) -> Result<()> {
    let _ = project_root;
    let status = std::process::Command::new("cargo")
        .args(["build", "--bin", WORLDBUILD_BIN_NAME])
        .env("SAND_MC_VERSION", mc_version)
        .status()
        .context("failed to invoke `cargo build --bin sand_build_world`")?;
    if !status.success() {
        bail!("`cargo build --bin sand_build_world` failed");
    }
    Ok(())
}

/// Runs the compiled `sand_build_world` binary with the given profile and
/// Minecraft version, and parses its JSON output.
pub(super) fn run(binary: &Path, profile: &str, mc_version: &str) -> Result<WorldBuildOutput> {
    let output = std::process::Command::new(binary)
        .env("SAND_BUILD_PROFILE", profile)
        .env("SAND_EXPORT_MC_VERSION", mc_version)
        .output()
        .with_context(|| format!("failed to run '{}'", binary.display()))?;
    if !output.status.success() {
        bail!(
            "sand.build.rs failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout).context("failed to parse sand_build_world JSON output")
}

/// Resolves the compiled binary's path under Cargo's target directory.
pub(super) fn binary_path(cargo_target_dir: &Path) -> PathBuf {
    cargo_target_dir.join("debug").join(WORLDBUILD_BIN_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_worldbuild_bin_target_in_cargo_toml() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[[bin]]\nname = \"sand_build_world\"\npath = \"sand.build.rs\"\n",
        )
        .unwrap();
        assert!(project_has_worldbuild(temp.path()));
    }

    #[test]
    fn absent_when_no_cargo_toml_mentions_it() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        assert!(!project_has_worldbuild(temp.path()));
    }

    #[test]
    fn parses_world_build_output_with_server_config() {
        let json = serde_json::json!({
            "resources": [{
                "namespace": "minecraft",
                "dir": "dimension",
                "path": "overworld",
                "ext": "json",
                "content_type": "text",
                "content": "{}",
            }],
            "server_config": {
                "view_distance": 12,
                "simulation_distance": 8,
                "difficulty": "hard",
                "online_mode": false,
                "world_reset_policy": true,
            },
            "seed": 1337,
            "level_type": "minecraft:flat",
        });
        let parsed: WorldBuildOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.resources.len(), 1);
        let server = parsed.server_config.unwrap();
        assert_eq!(server.view_distance, 12);
        assert!(!server.online_mode);
        assert!(server.world_reset_policy);
        assert_eq!(server.difficulty.as_str(), "hard");
        assert_eq!(parsed.seed, Some(1337));
        assert_eq!(parsed.level_type.as_deref(), Some("minecraft:flat"));
    }

    #[test]
    fn parses_world_build_output_without_server_config_or_seed() {
        let json = serde_json::json!({
            "resources": [], "server_config": null, "seed": null, "level_type": null,
        });
        let parsed: WorldBuildOutput = serde_json::from_value(json).unwrap();
        assert!(parsed.resources.is_empty());
        assert!(parsed.server_config.is_none());
        assert!(parsed.seed.is_none());
        assert!(parsed.level_type.is_none());
    }

    #[test]
    fn seed_and_level_type_default_to_none_when_absent_entirely() {
        // Older sand_build_world binaries (before these fields existed)
        // won't emit them at all; #[serde(default)] keeps sand-cli forward
        // compatible with them rather than failing to parse.
        let json = serde_json::json!({ "resources": [], "server_config": null });
        let parsed: WorldBuildOutput = serde_json::from_value(json).unwrap();
        assert!(parsed.seed.is_none());
        assert!(parsed.level_type.is_none());
    }
}
