use std::path::Path;

use anyhow::{Context, Result, bail};

use super::records::{
    ComponentContentType, ComponentRecord, ContentType, OutputExt, PackOverlay,
    PackSupportedFormats, ResourcePackRecord,
};

/// Builds the `pack.mcmeta` JSON body shared by datapacks and resource packs:
/// `pack_format`/`description`, an optional `pack.supported_formats`, and an
/// optional `overlays.entries` array. Kept identical to the legacy
/// `pack_format`-only shape when `supported_formats` is `None` and `overlays`
/// is empty, so existing single-version packs are byte-for-byte unchanged.
fn build_mcmeta(
    description: &str,
    pack_format: u32,
    supported_formats: Option<PackSupportedFormats>,
    overlays: &[PackOverlay],
) -> serde_json::Value {
    let mut pack = serde_json::json!({
        "pack_format": pack_format,
        "description": description,
    });
    if let Some(formats) = supported_formats {
        pack["supported_formats"] = formats.to_json();
    }

    let mut mcmeta = serde_json::json!({ "pack": pack });
    if !overlays.is_empty() {
        let entries: Vec<serde_json::Value> = overlays.iter().map(PackOverlay::to_json).collect();
        mcmeta["overlays"] = serde_json::json!({ "entries": entries });
    }
    mcmeta
}

pub fn write_pack_mcmeta(
    dist: &Path,
    namespace: &str,
    description: &str,
    pack_format: u32,
    supported_formats: Option<PackSupportedFormats>,
    overlays: &[PackOverlay],
) -> Result<()> {
    let _ = namespace; // available for future use
    let mut pack_body = build_mcmeta(description, pack_format, supported_formats, overlays);
    // Vanilla requires explicit supported-format bounds for modern packs.
    // Retain pack_format for older tooling while pinning this generated pack
    // to the exact verified profile used by Sand, unless the project already
    // configured its own `supported_formats`.
    if pack_format > 81 && supported_formats.is_none() {
        pack_body["pack"]["min_format"] = pack_format.into();
        pack_body["pack"]["max_format"] = pack_format.into();
    }
    std::fs::write(
        dist.join("pack.mcmeta"),
        serde_json::to_string_pretty(&pack_body)?,
    )?;
    Ok(())
}

pub fn write_component(dist: &Path, project_root: &Path, record: &ComponentRecord) -> Result<()> {
    // path inside the datapack: data/<namespace>/<dir>/<path>.<ext>
    let file_path = dist
        .join("data")
        .join(record.namespace.as_str())
        .join(record.dir.as_str())
        .join(format!("{}.{}", record.path.as_str(), record.ext.as_str()));
    std::fs::create_dir_all(file_path.parent().unwrap())
        .with_context(|| format!("failed to create dir for '{}'", file_path.display()))?;
    match record.content_type {
        ComponentContentType::Text => {
            // Minecraft accepts LF on every supported platform. Normalizing here makes
            // generated functions deterministic and follows the validation contract.
            let content = if record.ext == OutputExt::Mcfunction {
                record.content.replace("\r\n", "\n").replace('\r', "\n")
            } else {
                record.content.clone()
            };
            std::fs::write(&file_path, content)
                .with_context(|| format!("failed to write '{}'", file_path.display()))?;
        }
        ComponentContentType::Copy => {
            let src = project_root.join(&record.content);
            if !src.exists() {
                bail!(
                    "datapack structure asset not found: '{}'\n\
                     Make sure the file exists relative to your project root.",
                    src.display()
                );
            }
            let mut input = std::io::BufReader::new(
                std::fs::File::open(&src)
                    .with_context(|| format!("failed to open '{}'", src.display()))?,
            );
            let mut output = std::io::BufWriter::new(
                std::fs::File::create(&file_path)
                    .with_context(|| format!("failed to create '{}'", file_path.display()))?,
            );
            std::io::copy(&mut input, &mut output).with_context(|| {
                format!(
                    "failed to copy '{}' → '{}'",
                    src.display(),
                    file_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn write_resourcepack_mcmeta(
    dist: &Path,
    description: &str,
    pack_format: u32,
    supported_formats: Option<PackSupportedFormats>,
    overlays: &[PackOverlay],
) -> Result<()> {
    let mcmeta = build_mcmeta(description, pack_format, supported_formats, overlays);
    std::fs::write(
        dist.join("pack.mcmeta"),
        serde_json::to_string_pretty(&mcmeta)?,
    )?;
    Ok(())
}

pub fn write_rp_record(
    dist: &Path,
    project_root: &Path,
    record: &ResourcePackRecord,
) -> Result<()> {
    // The `path` field is already a full pack-relative path, e.g.
    // "assets/my_pack/font/hud.json". Strip any leading separator just in
    // case, then join to the dist directory.
    let rel = record.path.as_str().trim_start_matches('/');
    let dest = dist.join(rel);
    std::fs::create_dir_all(dest.parent().unwrap())
        .with_context(|| format!("failed to create dir for '{}'", dest.display()))?;

    match record.content_type {
        ContentType::Json => {
            std::fs::write(&dest, &record.content)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }
        ContentType::Copy => {
            let src = project_root.join(&record.content);
            if !src.exists() {
                bail!(
                    "resource pack asset not found: '{}'\n\
                     Make sure the file exists relative to your project root.",
                    src.display()
                );
            }
            let mut input = std::io::BufReader::new(
                std::fs::File::open(&src)
                    .with_context(|| format!("failed to open '{}'", src.display()))?,
            );
            let mut output = std::io::BufWriter::new(
                std::fs::File::create(&dest)
                    .with_context(|| format!("failed to create '{}'", dest.display()))?,
            );
            std::io::copy(&mut input, &mut output).with_context(|| {
                format!("failed to copy '{}' → '{}'", src.display(), dest.display())
            })?;
        }
        ContentType::Bytes => {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&record.content)
                .with_context(|| format!("failed to base64-decode '{}'", record.path.as_str()))?;
            std::fs::write(&dest, &bytes)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }
    }

    Ok(())
}
