//! Version-keyed aggregate baselines for generated public surfaces.
//!
//! The architectural scope partition is version-independent, but vanilla
//! command and registry generators expose different finite surfaces for each
//! verified Minecraft version. Profiles bind the exact generated-provider
//! version to its corresponding aggregate ratchet and deterministic report.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

const SURFACE_PROFILE_SCHEMA_VERSION: u32 = 1;

/// All verified generated-surface profiles known to a facade build.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceProfileManifest {
    pub schema_version: u32,
    #[serde(rename = "profile")]
    pub profiles: Vec<SurfaceProfile>,
}

/// Exact aggregate ratchet for one resolved generated-provider version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SurfaceProfile {
    pub minecraft_version: String,
    pub static_surface_items: usize,
    pub pending_item_ceiling: usize,
    pub baseline: PathBuf,
}

impl SurfaceProfileManifest {
    /// Load and structurally validate a deterministic profile manifest.
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let manifest: Self = toml::from_str(&source)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Select the one exact profile named by every authoritative provider.
    ///
    /// The keys are provider ids and the values are the resolved versions
    /// embedded in their catalogs. Environment variables are deliberately not
    /// consulted: a stale or mixed provider directory must fail closed.
    pub fn select<'a>(
        &'a self,
        provider_versions: &BTreeMap<String, String>,
    ) -> Result<&'a SurfaceProfile, String> {
        if provider_versions.is_empty() {
            return Err("cannot select a surface profile without generated providers".into());
        }
        let versions = provider_versions.values().collect::<BTreeSet<_>>();
        if versions.len() != 1 {
            let detail = provider_versions
                .iter()
                .map(|(provider, version)| format!("{provider}={version}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "generated API providers disagree on Minecraft version ({detail})"
            ));
        }
        let version = versions.into_iter().next().expect("nonempty version set");
        self.profiles
            .iter()
            .find(|profile| profile.minecraft_version == *version)
            .ok_or_else(|| {
                format!(
                    "generated API providers target unknown Minecraft surface profile `{version}`; add and audit an exact profile before compiling this version"
                )
            })
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != SURFACE_PROFILE_SCHEMA_VERSION {
            return Err(format!(
                "unsupported surface profile schema {}, expected {SURFACE_PROFILE_SCHEMA_VERSION}",
                self.schema_version
            ));
        }
        if self.profiles.is_empty() {
            return Err("surface profile manifest contains no profiles".into());
        }
        let mut versions = BTreeSet::new();
        for profile in &self.profiles {
            if profile.minecraft_version.trim().is_empty() {
                return Err("surface profile has an empty Minecraft version".into());
            }
            if !versions.insert(profile.minecraft_version.as_str()) {
                return Err(format!(
                    "duplicate Minecraft surface profile `{}`",
                    profile.minecraft_version
                ));
            }
            if profile.pending_item_ceiling > profile.static_surface_items {
                return Err(format!(
                    "surface profile `{}` pending item ceiling {} exceeds its static surface {}",
                    profile.minecraft_version,
                    profile.pending_item_ceiling,
                    profile.static_surface_items
                ));
            }
            if profile.baseline.as_os_str().is_empty()
                || profile.baseline.is_absolute()
                || profile
                    .baseline
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err(format!(
                    "surface profile `{}` baseline must be a nonempty relative path without traversal",
                    profile.minecraft_version
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> SurfaceProfileManifest {
        SurfaceProfileManifest {
            schema_version: 1,
            profiles: vec![
                SurfaceProfile {
                    minecraft_version: "1.21.4".into(),
                    static_surface_items: 10,
                    pending_item_ceiling: 10,
                    baseline: "stable.txt".into(),
                },
                SurfaceProfile {
                    minecraft_version: "26.2".into(),
                    static_surface_items: 12,
                    pending_item_ceiling: 12,
                    baseline: "latest.txt".into(),
                },
            ],
        }
    }

    #[test]
    fn selects_exact_provider_version() {
        let providers = BTreeMap::from([
            ("generated_commands".into(), "1.21.4".into()),
            ("generated_registries".into(), "1.21.4".into()),
        ]);
        assert_eq!(
            manifest().select(&providers).unwrap().baseline,
            PathBuf::from("stable.txt")
        );
    }

    #[test]
    fn unknown_provider_version_fails_closed() {
        let providers = BTreeMap::from([("generated_commands".into(), "1.22".into())]);
        assert!(
            manifest()
                .select(&providers)
                .unwrap_err()
                .contains("unknown Minecraft surface profile `1.22`")
        );
    }

    #[test]
    fn mixed_provider_versions_fail_closed() {
        let providers = BTreeMap::from([
            ("generated_commands".into(), "1.21.4".into()),
            ("generated_registries".into(), "26.2".into()),
        ]);
        let error = manifest().select(&providers).unwrap_err();
        assert!(error.contains("generated_commands=1.21.4"));
        assert!(error.contains("generated_registries=26.2"));
    }
}
