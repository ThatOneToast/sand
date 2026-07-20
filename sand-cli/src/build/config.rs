use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

pub(super) fn resolve_mc_version(mc_version: &str) -> String {
    if mc_version == "latest" {
        sand_build::latest_release_version()
    } else {
        mc_version.to_string()
    }
}

pub(super) fn cargo_target_dir() -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct CargoMetadata {
        target_directory: PathBuf,
    }

    let output = std::process::Command::new("cargo")
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
