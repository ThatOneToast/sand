use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use super::records::{ComponentRecord, ContentType, OutputExt, ResourcePackRecord};

pub(super) fn validate_component_records(
    dist: &std::path::Path,
    records: &[ComponentRecord],
) -> Result<()> {
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
                // Function tags get structural validation in addition to JSON parsing.
                if record.dir.as_str() == "tags/function" {
                    validate_function_tag(record.path.as_str(), &record.content)?;
                }
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

/// Returns the absolute output path for a component record under `dist`.
///
/// Namespace, directory, and path traversal safety are guaranteed by the
/// newtypes on [`ComponentRecord`] — this function only assembles the path.
pub(super) fn component_output_path(
    dist: &std::path::Path,
    record: &ComponentRecord,
) -> Result<PathBuf> {
    Ok(dist
        .join("data")
        .join(record.namespace.as_str())
        .join(record.dir.as_str())
        .join(format!("{}.{}", record.path.as_str(), record.ext.as_str())))
}

pub(super) fn validate_resourcepack_records(records: &[ResourcePackRecord]) -> Result<()> {
    let mut paths = HashSet::new();
    for record in records {
        // RelativePackPath guarantees no traversal — check asset root prefix.
        if !record.path.as_str().starts_with("assets/") {
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

/// Validates a Minecraft function tag JSON string.
///
/// A valid function tag is a JSON object with a `"values"` array. Each entry
/// must be either a resource-location string (`"namespace:path"`, optionally
/// prefixed with `#` to reference another tag) or an object with an `"id"`
/// field (`{"id": "...", "required": false}`).
///
/// Called automatically from [`validate_component_records`] for all
/// `tags/function` records, and available for standalone validation.
pub(super) fn validate_function_tag(tag_name: &str, json: &str) -> Result<()> {
    let v: serde_json::Value = serde_json::from_str(json)
        .with_context(|| format!("invalid JSON in function tag '{tag_name}'"))?;

    let obj = v.as_object().ok_or_else(|| {
        anyhow::anyhow!("function tag '{tag_name}' must be a JSON object, got {v}")
    })?;

    let values = obj.get("values").ok_or_else(|| {
        anyhow::anyhow!("function tag '{tag_name}' missing required 'values' array")
    })?;

    let arr = values.as_array().ok_or_else(|| {
        anyhow::anyhow!("function tag '{tag_name}'.values must be an array, got {values}")
    })?;

    for (i, entry) in arr.iter().enumerate() {
        match entry {
            serde_json::Value::String(s) => {
                let target = s.trim_start_matches('#');
                if !target.contains(':') {
                    bail!(
                        "function tag '{tag_name}' entry {i} '{s}' is not a valid \
                         resource location (missing ':')"
                    );
                }
            }
            serde_json::Value::Object(obj) => {
                if !obj.contains_key("id") {
                    bail!("function tag '{tag_name}' entry {i} object must have an 'id' field");
                }
            }
            other => {
                bail!(
                    "function tag '{tag_name}' entry {i} must be a string or object, got {other}"
                );
            }
        }
    }

    Ok(())
}

/// Validates a Minecraft namespace string (lowercase letters, digits, `_`, `-`, `.`).
///
/// Used to validate the `namespace` field from `sand.toml` at build time,
/// before the namespace is used as a filesystem path component.
pub(super) fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        bail!(
            "invalid namespace '{namespace}' in sand.toml: expected lowercase letters, digits, `_`, `-`, or `.`"
        );
    }
    Ok(())
}
