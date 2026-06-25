use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use super::records::{ComponentRecord, ContentType, OutputExt, ResourcePackRecord};

pub(crate) fn validate_component_records(
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
                // Covers both the canonical form (dir="tags/function") and the generic
                // form (dir="tags", path starts with "function/").
                let is_function_tag = record.dir.as_str() == "tags/function"
                    || (record.dir.as_str() == "tags"
                        && record.path.as_str().starts_with("function/"));
                if is_function_tag {
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
pub(crate) fn component_output_path(
    dist: &std::path::Path,
    record: &ComponentRecord,
) -> Result<PathBuf> {
    Ok(dist
        .join("data")
        .join(record.namespace.as_str())
        .join(record.dir.as_str())
        .join(format!("{}.{}", record.path.as_str(), record.ext.as_str())))
}

pub(crate) fn validate_resourcepack_records(records: &[ResourcePackRecord]) -> Result<()> {
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
/// field containing a valid resource location (`{"id": "ns:path", "required": false}`).
///
/// Called automatically from [`validate_component_records`] for all
/// `tags/function` and `tags`+`function/` records, and available for
/// standalone validation.
pub(crate) fn validate_function_tag(tag_name: &str, json: &str) -> Result<()> {
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
                if !is_valid_resource_location(target) {
                    bail!(
                        "function tag '{tag_name}' entry {i} '{s}' is not a valid \
                         resource location (expected 'namespace:path' with lowercase \
                         letters, digits, `_`, `-`, `.`)"
                    );
                }
            }
            serde_json::Value::Object(obj) => {
                let id_val = obj.get("id").ok_or_else(|| {
                    anyhow::anyhow!(
                        "function tag '{tag_name}' entry {i} object must have an 'id' field"
                    )
                })?;
                let id = id_val.as_str().ok_or_else(|| {
                    anyhow::anyhow!(
                        "function tag '{tag_name}' entry {i} 'id' must be a string, \
                         got {id_val}"
                    )
                })?;
                if !is_valid_resource_location(id) {
                    bail!(
                        "function tag '{tag_name}' entry {i} 'id' value '{id}' is not \
                         a valid resource location"
                    );
                }
                if obj.get("required").is_some_and(|req| !req.is_boolean()) {
                    bail!(
                        "function tag '{tag_name}' entry {i} 'required' must be \
                         a boolean"
                    );
                }
            }
            other => {
                bail!(
                    "function tag '{tag_name}' entry {i} must be a string or object, \
                     got {other}"
                );
            }
        }
    }

    Ok(())
}

/// Returns `true` if `s` is a valid Minecraft resource location (`namespace:path`).
///
/// Rules:
/// - Must contain exactly one `:`.
/// - Namespace: non-empty, `[a-z0-9_.-]`.
/// - Path: non-empty, `[a-z0-9_./-]`.
fn is_valid_resource_location(s: &str) -> bool {
    let Some((ns, path)) = s.split_once(':') else {
        return false;
    };
    !ns.is_empty()
        && !path.is_empty()
        && ns.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
        })
        && path.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.' | b'/')
        })
}

/// Validates a Minecraft namespace string (lowercase letters, digits, `_`, `-`, `.`).
///
/// Used to validate the `namespace` field from `sand.toml` at build time,
/// before the namespace is used as a filesystem path component.
pub(crate) fn validate_namespace(namespace: &str) -> Result<()> {
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
