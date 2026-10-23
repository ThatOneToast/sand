use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

pub(super) fn resolve_mc_version(mc_version: &str) -> String {
    if mc_version == "latest" {
        sand_build::latest_release_version()
    } else {
        mc_version.to_string()
    }
}

pub(crate) fn cargo_target_dir() -> Result<PathBuf> {
    cargo_target_dir_at(&std::env::current_dir()?)
}

pub(crate) fn cargo_target_dir_at(root: &Path) -> Result<PathBuf> {
    target_dir_from_metadata(std::process::Command::new("cargo").current_dir(root))
}

fn target_dir_from_metadata(command: &mut std::process::Command) -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct CargoMetadata {
        target_directory: PathBuf,
    }

    let output = command
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .context("failed to invoke `cargo metadata`")?;
    if !output.status.success() {
        bail!(
            "`cargo metadata` failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(serde_json::from_slice::<CargoMetadata>(&output.stdout)
        .context("failed to parse `cargo metadata` output")?
        .target_directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_resolves_configured_and_environment_target_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::create_dir(root.join(".cargo")).unwrap();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname='watch-target-fixture'\nversion='0.1.0'\n[workspace]\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".cargo/config.toml"),
            "[build]\ntarget-dir='build'\n",
        )
        .unwrap();
        let mut command = std::process::Command::new("cargo");
        command
            .current_dir(&root)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_BUILD_TARGET_DIR");
        assert_eq!(
            target_dir_from_metadata(&mut command).unwrap(),
            root.join("build")
        );
        let mut command = std::process::Command::new("cargo");
        command
            .current_dir(&root)
            .env("CARGO_TARGET_DIR", root.join("custom-output"));
        assert_eq!(
            target_dir_from_metadata(&mut command).unwrap(),
            root.join("custom-output")
        );
    }
}
