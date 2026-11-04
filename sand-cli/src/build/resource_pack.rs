//! Optional companion client assets. This packages authored asset bytes and
//! validates their paths/JSON; it does not implement Minecraft's client codecs.
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use super::output_manifest::{ChangeSummary, reject_symlink_ancestors, validate_resource_path};
use crate::config::{ResourcePackConfig, SandConfig};

/// One exact native resource format, including its independent minor revision.
/// Accepts `88` or `[97, 1]` in `[resource_pack].pack_format`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ResourcePackFormat {
    Major(u32),
    Version([u32; 2]),
}
impl ResourcePackFormat {
    fn parts(self) -> (u32, u32) {
        match self {
            Self::Major(major) => (major, 0),
            Self::Version([major, minor]) => (major, minor),
        }
    }
    fn validate(self) -> Result<Self> {
        let (major, minor) = self.parts();
        ensure!(
            major >= 1 && major <= i32::MAX as u32 && minor <= i32::MAX as u32,
            "resource pack format major must be in 1..=i32::MAX and minor in 0..=i32::MAX"
        );
        Ok(self)
    }
    pub(super) fn json(self) -> serde_json::Value {
        let (major, minor) = self.parts();
        if minor == 0 {
            major.into()
        } else {
            serde_json::json!([major, minor])
        }
    }
}

/// The exact formats extracted from `version.json` inside Mojang server jars.
/// This table is independent of Sand's datapack format profile and has no future
/// version fallback. The bundled fixture retains artifact SHA-1 provenance.
pub fn format_for(
    version: &str,
    configured: Option<ResourcePackFormat>,
) -> Result<ResourcePackFormat> {
    if let Some(format) = configured {
        return format.validate();
    }
    let version = if version == "latest" {
        sand_core::version::LATEST_KNOWN
    } else {
        version
    };
    let records: serde_json::Value =
        serde_json::from_str(include_str!("resource_pack_versions.json"))?;
    let entry = records.as_array().expect("bundled resource format records").iter().find(|record| record["version_json"]["id"] == version)
        .with_context(|| format!("resource pack format is not verified for Minecraft '{version}'; set [resource_pack].pack_format explicitly to the target's resource format"))?;
    let pack = &entry["version_json"]["pack_version"];
    ResourcePackFormat::Version([
        pack["resource_major"].as_u64().unwrap() as u32,
        pack["resource_minor"].as_u64().unwrap() as u32,
    ])
    .validate()
}

/// A complete companion output set captured before any build output is changed.
pub struct PreparedResourcePack {
    pub destination: PathBuf,
    pub resources: BTreeMap<String, Vec<u8>>,
    pub format: ResourcePackFormat,
    pub asset_count: usize,
}

/// Optional machine-readable result; absence preserves the datapack-only JSON
/// response shape. This is a packaging result, not a client-installation claim.
#[derive(Serialize)]
pub struct ResourcePackBuildResult {
    pub output: String,
    pub format: serde_json::Value,
    pub asset_count: usize,
    pub changes: ChangeSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
}

/// Prepare a configured companion pack without modifying source or outputs.
/// Binary assets retain exact bytes. JSON assets must parse, but model/sound/
/// font/texture schemas and reference existence are the client's responsibility.
pub fn prepare(
    project_root: &Path,
    config: &SandConfig,
    version: &str,
) -> Result<Option<PreparedResourcePack>> {
    let Some(resources) = &config.resource_pack else {
        return Ok(None);
    };
    let format = format_for(version, resources.pack_format)?;
    let destination = project_root
        .join("dist")
        .join(format!("{}-resources", config.pack.namespace));
    let source = source_path(project_root, resources)?;
    ensure!(
        !destination.starts_with(&source) && !source.starts_with(project_root.join("dist")),
        "resource pack source must be outside dist and cannot contain its output"
    );
    let mut outputs = collect(&source)?;
    let description = resources
        .description
        .clone()
        .unwrap_or_else(|| format!("{} resources", config.pack.description));
    let (major, _) = format.parts();
    let metadata = serde_json::json!({"pack":{"description":description,"pack_format":major,"min_format":format.json(),"max_format":format.json()}});
    let asset_count = outputs.len();
    outputs.insert("pack.mcmeta".into(), serde_json::to_vec_pretty(&metadata)?);
    Ok(Some(PreparedResourcePack {
        destination,
        resources: outputs,
        format,
        asset_count,
    }))
}

/// Resolve the explicitly configured project-local source without following
/// links. A source rooted at the project itself could ingest generated output.
pub fn source_path(project_root: &Path, config: &ResourcePackConfig) -> Result<PathBuf> {
    ensure!(
        !config.source.as_str().contains('\\'),
        "resource pack source must use forward-slash path separators"
    );
    let source = Path::new(config.source.as_str());
    ensure!(
        source
            .components()
            .all(|part| matches!(part, Component::Normal(_))),
        "resource pack source must be a nonempty project-relative directory without '.' or '..'"
    );
    let source = project_root.join(source);
    reject_symlink_ancestors(&source)?;
    Ok(source)
}

/// Read the allowed source layout in stable order. A top-level README.md is
/// author guidance and is deliberately not packaged. Metadata belongs to the
/// typed project configuration so source files cannot silently override it.
fn collect(source: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    ensure!(
        source.is_dir(),
        "resource pack source directory does not exist: {}",
        source.display()
    );
    reject_symlink_ancestors(source)?;
    let mut resources = BTreeMap::new();
    for entry in walkdir::WalkDir::new(source)
        .follow_links(false)
        .sort_by_file_name()
    {
        let entry = entry.context("read resource pack source")?;
        ensure!(
            entry.file_type().is_dir() || entry.file_type().is_file(),
            "resource pack source contains symlink or special file: {}",
            entry.path().display()
        );
        let relative = entry.path().strip_prefix(source)?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let path = relative
            .to_str()
            .context("resource pack source path is not UTF-8")?
            .replace(std::path::MAIN_SEPARATOR, "/");
        validate_resource_path(&path)?;
        if entry.file_type().is_dir() {
            ensure!(
                path == "assets" || path.starts_with("assets/"),
                "resource pack directories must be under assets/: {path}"
            );
            continue;
        }
        if path == "README.md" {
            continue;
        }
        if path == "pack.mcmeta" {
            bail!(
                "resource pack metadata is generated from [resource_pack]; remove source pack.mcmeta or move its settings to sand.toml"
            );
        }
        if path == "pack.png" {
            let bytes = std::fs::read(entry.path())?;
            ensure!(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                "resource pack icon is not a PNG: {path}"
            );
            resources.insert(path, bytes);
            continue;
        }
        let remainder = path.strip_prefix("assets/").with_context(|| {
            format!("unsupported resource pack root file: {path}; use assets/ or pack.png")
        })?;
        let (namespace, asset) = remainder
            .split_once('/')
            .with_context(|| format!("resource asset requires a namespace and path: {path}"))?;
        sand_components::ResourceLocation::new(namespace, asset)
            .with_context(|| format!("invalid resource asset path: {path}"))?;
        let bytes =
            std::fs::read(entry.path()).with_context(|| format!("read resource asset {path}"))?;
        if path.ends_with(".json") || path.ends_with(".mcmeta") {
            serde_json::from_slice::<serde_json::Value>(&bytes)
                .with_context(|| format!("invalid JSON resource asset: {path}"))?;
        }
        resources.insert(path, bytes);
    }
    Ok(resources)
}
