use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::records::{
    ComponentContentType, ComponentRecord, ContentType, OutputExt, ResourcePackRecord,
};

pub fn validate_component_records(
    dist: &std::path::Path,
    records: &[ComponentRecord],
) -> Result<()> {
    validate_component_records_impl(dist, records, None)
}

pub fn validate_component_records_for_project(
    dist: &std::path::Path,
    project_root: &std::path::Path,
    records: &[ComponentRecord],
) -> Result<()> {
    validate_component_records_impl(dist, records, Some(project_root))
}

fn validate_component_records_impl(
    dist: &std::path::Path,
    records: &[ComponentRecord],
    project_root: Option<&std::path::Path>,
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
        if record.dir.as_str() == "structure" && record.ext != OutputExt::Nbt {
            bail!(
                "structure template {}:{} must use .nbt output",
                record.namespace,
                record.path
            );
        }
        match record.ext {
            OutputExt::Json => {
                if record.content_type != ComponentContentType::Text {
                    bail!(
                        "generated JSON component {}:{}/{} must use text content",
                        record.namespace,
                        record.dir,
                        record.path
                    );
                }
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
                if record.content_type != ComponentContentType::Text {
                    bail!(
                        "generated function {}:{}/{} must use text content",
                        record.namespace,
                        record.dir,
                        record.path
                    );
                }
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
            OutputExt::Nbt => {
                if record.dir.as_str() != "structure" {
                    bail!(
                        "binary NBT component {}:{}/{} must use the structure directory",
                        record.namespace,
                        record.dir,
                        record.path
                    );
                }
                if record.content_type != ComponentContentType::Copy {
                    bail!(
                        "structure template {}:{} must copy a source .nbt file",
                        record.namespace,
                        record.path
                    );
                }
                validate_structure_source_path(&record.content)?;
                if let Some(project_root) = project_root {
                    validate_structure_source_file(project_root, record)?;
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
pub fn component_output_path(dist: &std::path::Path, record: &ComponentRecord) -> Result<PathBuf> {
    Ok(dist
        .join("data")
        .join(record.namespace.as_str())
        .join(record.dir.as_str())
        .join(format!("{}.{}", record.path.as_str(), record.ext.as_str())))
}

fn validate_structure_source_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.contains('\0')
        || !path.ends_with(".nbt")
        || Path::new(path).is_absolute()
        || Path::new(path).components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!(
            "unsafe structure template source path '{path}'; expected a project-root-relative .nbt file"
        );
    }
    Ok(())
}

fn validate_structure_source_file(project_root: &Path, record: &ComponentRecord) -> Result<()> {
    validate_copy_source_within_project(
        project_root,
        &record.content,
        "datapack structure asset",
        "a project-root-relative .nbt file",
    )
}

/// Resolves a project-relative copy source and proves it stays inside the
/// project root.
///
/// Shared by the datapack structure-template path and the resource-pack copy
/// path so the two export boundaries cannot drift apart again.
///
/// The lexical checks performed by the callers (`..`, absolute paths, prefixes,
/// null bytes) are not sufficient on their own: `std::fs::metadata` follows
/// symlinks, so a project-relative source such as `structures/leak.nbt ->
/// ../../secrets/leak.nbt` passes every lexical rule while resolving outside
/// the project. Canonicalizing both sides and requiring containment closes
/// that hole, including for symlinked intermediate directories.
///
/// **Symlink policy:** a symlink is accepted only when its fully resolved
/// target is still inside the canonical project root. A project may therefore
/// keep assets behind an internal symlink, but a symlink pointing out of the
/// project tree is rejected.
///
/// Containment is checked component-wise via [`Path::starts_with`], so a
/// sibling directory like `/tmp/project-other` is not treated as being inside
/// `/tmp/project`. Both sides are canonical, so a project root reached through
/// a symlink compares correctly against sources under it.
///
/// # Limits
///
/// - **Hard links are invisible here.** Canonicalization resolves symlinks, not
///   hard links, so a hard link created inside the project to an outside file
///   resolves to an in-project path and is accepted. Nothing at this layer can
///   distinguish that from an ordinary file.
/// - **Not race-free.** The source is canonicalized and opened here, then
///   re-joined and copied later by [`super::write`]. A source swapped between
///   the two steps is out of scope, exactly as it is for the resource-pack
///   path. Closing that window would require copying through the handle opened
///   during validation.
///
/// `label` names the artifact family for diagnostics ("datapack structure
/// asset", "resource-pack asset") and `expected` completes the remediation
/// hint after "Make sure the source path points to ".
fn validate_copy_source_within_project(
    project_root: &Path,
    source: &str,
    label: &str,
    expected: &str,
) -> Result<()> {
    let src = project_root.join(source);
    let canonical_project_root = project_root.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize project root '{}'",
            project_root.display()
        )
    })?;
    // Probe metadata first so a missing or unreadable source keeps its specific
    // diagnostic instead of surfacing a bare canonicalization failure.
    let metadata = std::fs::metadata(&src).with_context(|| {
        // `metadata` follows symlinks, so a link whose target is missing or
        // unreadable looks identical to a missing file — say which it is, or
        // the author sees a "not found" error for a path they can see on disk.
        let is_symlink = std::fs::symlink_metadata(&src).is_ok_and(|m| m.file_type().is_symlink());
        if is_symlink {
            format!(
                "{label} is a symlink whose target is missing or unreadable: '{}'\n\
                 Make sure the link resolves to {expected} inside your project root.",
                src.display()
            )
        } else {
            format!(
                "{label} not found or unreadable before writing output: '{}'\n\
                 Make sure the file exists relative to your project root.",
                src.display()
            )
        }
    })?;
    if !metadata.is_file() {
        bail!(
            "{label} is not a file: '{}'\n\
             Make sure the source path points to {expected}.",
            src.display()
        );
    }

    let canonical_src = src.canonicalize().with_context(|| {
        format!(
            "{label} not found or unreadable before writing output: '{}'",
            src.display()
        )
    })?;
    if !canonical_src.starts_with(&canonical_project_root) {
        bail!(
            "{label} escapes the project root: '{}'\n\
             Make sure the source path points to {expected}.",
            src.display()
        );
    }

    std::fs::File::open(&src).with_context(|| {
        format!(
            "{label} not readable before writing output: '{}'",
            src.display()
        )
    })?;

    Ok(())
}

pub fn validate_resourcepack_records(records: &[ResourcePackRecord]) -> Result<()> {
    validate_resourcepack_records_impl(records, None)
}

pub fn validate_resourcepack_records_for_project(
    project_root: &std::path::Path,
    records: &[ResourcePackRecord],
) -> Result<()> {
    validate_resourcepack_records_impl(records, Some(project_root))
}

fn validate_resourcepack_records_impl(
    records: &[ResourcePackRecord],
    project_root: Option<&std::path::Path>,
) -> Result<()> {
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
        match record.content_type {
            ContentType::Json => {
                serde_json::from_str::<serde_json::Value>(&record.content).map_err(|e| {
                    anyhow::anyhow!("invalid resource-pack JSON '{}': {e}", record.path)
                })?;
            }
            ContentType::Copy => {
                validate_resourcepack_copy_source_path(record.path.as_str(), &record.content)?;
                if let Some(project_root) = project_root {
                    validate_resourcepack_copy_source_file(project_root, record)?;
                }
            }
            ContentType::Bytes => {
                use base64::Engine as _;

                base64::engine::general_purpose::STANDARD
                    .decode(&record.content)
                    .with_context(|| {
                        format!(
                            "invalid base64 bytes for resource-pack asset '{}'",
                            record.path
                        )
                    })?;
            }
        }
    }
    Ok(())
}

fn validate_resourcepack_copy_source_path(asset_path: &str, source_path: &str) -> Result<()> {
    if source_path.is_empty()
        || source_path.contains('\0')
        || Path::new(source_path).is_absolute()
        || Path::new(source_path).components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!(
            "unsafe resource-pack copy source path '{source_path}' for asset '{asset_path}'; \
             expected a project-root-relative file"
        );
    }
    Ok(())
}

fn validate_resourcepack_copy_source_file(
    project_root: &Path,
    record: &ResourcePackRecord,
) -> Result<()> {
    validate_copy_source_within_project(
        project_root,
        &record.content,
        "resource-pack asset",
        "a project-root-relative file",
    )
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
pub fn validate_function_tag(tag_name: &str, json: &str) -> Result<()> {
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
                let target = id.trim_start_matches('#');
                if !is_valid_resource_location(target) {
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
pub fn validate_namespace(namespace: &str) -> Result<()> {
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_string_tag_ref() {
        // String form: "#other_pack:startup"
        assert!(validate_function_tag("load", r##"{"values":["#other_pack:startup"]}"##).is_ok());
    }

    #[test]
    fn accepts_object_tag_ref() {
        // Object form with tag ref id
        assert!(
            validate_function_tag(
                "load",
                r##"{"values":[{"id":"#other_pack:startup","required":false}]}"##
            )
            .is_ok()
        );
    }

    #[test]
    fn accepts_object_function_ref() {
        // Object form with regular function id (no #)
        assert!(
            validate_function_tag(
                "load",
                r#"{"values":[{"id":"other_pack:startup","required":false}]}"#
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_object_ref_with_invalid_id() {
        // Invalid resource location in object form
        let err = validate_function_tag(
            "load",
            r#"{"values":[{"id":"BadNamespace:load","required":false}]}"#,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("BadNamespace:load"),
            "error must mention the bad id: {err}"
        );
    }
}
