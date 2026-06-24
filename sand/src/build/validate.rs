use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, bail};

use super::records::{ComponentRecord, ContentType, OutputExt, ResourcePackRecord};

pub(super) fn validate_component_records(dist: &Path, records: &[ComponentRecord]) -> Result<()> {
    let mut paths = HashSet::new();
    for record in records {
        let output_path = component_output_path(dist, record)?;
        if !paths.insert(output_path.clone()) {
            bail!(
                "duplicate generated component output path '{}': {}:{}/{}",
                output_path.display(),
                record.namespace,
                record.dir,
                record.path
            );
        }
        match record.ext {
            OutputExt::Json => {
                serde_json::from_str::<serde_json::Value>(&record.content).map_err(|e| {
                    anyhow::anyhow!(
                        "invalid generated JSON for component {}:{}/{} at '{}': {e}",
                        record.namespace,
                        record.dir,
                        record.path,
                        output_path.display()
                    )
                })?;
            }
            OutputExt::Mcfunction => {
                if record.content.contains('\0') {
                    bail!(
                        "invalid generated function {}:{}/{} at '{}': embedded null byte",
                        record.namespace,
                        record.dir,
                        record.path,
                        output_path.display()
                    );
                }
            }
        }
    }
    Ok(())
}

pub(super) fn component_output_path(dist: &Path, record: &ComponentRecord) -> Result<PathBuf> {
    validate_namespace(&record.namespace)?;
    validate_relative("component directory", &record.dir)?;
    validate_relative("component path", &record.path)?;
    if !supported_component_dir(&record.dir) {
        bail!(
            "unsupported component directory '{}' for {}:{}",
            record.dir,
            record.namespace,
            record.path
        );
    }
    Ok(dist
        .join("data")
        .join(&record.namespace)
        .join(&record.dir)
        .join(format!("{}.{}", record.path, record.ext.as_str())))
}

pub(super) fn validate_resourcepack_records(records: &[ResourcePackRecord]) -> Result<()> {
    let mut paths = HashSet::new();
    for record in records {
        validate_relative("resource-pack asset path", &record.path)?;
        if !record.path.starts_with("assets/") {
            bail!(
                "resource-pack record '{}' must be under assets/ (data/ belongs to the datapack)",
                record.path
            );
        }
        if !paths.insert(record.path.as_str()) {
            bail!("duplicate resource-pack output path '{}'", record.path);
        }
        if record.content_type == ContentType::Json {
            serde_json::from_str::<serde_json::Value>(&record.content).map_err(|e| {
                anyhow::anyhow!("invalid resource-pack JSON '{}': {e}", record.path)
            })?;
        }
    }
    Ok(())
}

pub(super) fn validate_relative(kind: &str, value: &str) -> Result<()> {
    if value.is_empty() || Path::new(value).is_absolute() || value.contains('\0') {
        bail!("invalid {kind} '{value}'");
    }
    if Path::new(value).components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("unsafe {kind} '{value}'");
    }
    Ok(())
}

pub(super) fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        bail!("invalid namespace '{namespace}'");
    }
    Ok(())
}

pub(super) fn supported_component_dir(dir: &str) -> bool {
    matches!(
        dir,
        "advancement"
            | "banner_pattern"
            | "chat_type"
            | "damage_type"
            | "dialog"
            | "dimension"
            | "enchantment"
            | "function"
            | "instrument"
            | "item_modifier"
            | "jukebox_song"
            | "loot_table"
            | "painting_variant"
            | "predicate"
            | "recipe"
            | "tags"
            | "tags/function"
            | "trim_material"
            | "trim_pattern"
            | "wolf_variant"
            | "worldgen/biome"
            | "worldgen/noise_settings"
            | "worldgen/placed_feature"
    )
}
