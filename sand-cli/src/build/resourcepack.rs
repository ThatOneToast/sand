use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::config::SandConfig;

use super::export::{Exporter, run_exporter};
use super::package::zip_dir;
use super::records::ResourcePackRecord;
use super::validate::validate_resourcepack_records_for_project;
use super::write::{write_resourcepack_mcmeta, write_rp_record};

/// The namespace the resource pack is exported under.
///
/// Falls back to the datapack namespace when `[resourcepack]` does not set one.
fn resourcepack_namespace(config: &SandConfig) -> &str {
    config
        .resourcepack
        .as_ref()
        .and_then(|c| c.namespace.as_ref().map(|n| n.as_str()))
        .unwrap_or(config.pack.namespace.as_str())
}

/// Fails with scaffolding instructions when the project has no resource
/// exporter binary source.
///
/// Run before the exporters are compiled so `cargo build --bin
/// sand_resource_export` never fails with an opaque "no bin target" error, and
/// so a `--resourcepack` build that cannot succeed stops before writing any
/// datapack output.
pub(super) fn ensure_resource_export_source(
    config: &SandConfig,
    project_root: &Path,
) -> Result<()> {
    let export_src = project_root.join("src/bin/sand_resource_export.rs");
    if export_src.exists() {
        return Ok(());
    }
    bail!(
        "src/bin/sand_resource_export.rs not found.\n\n\
         To enable resource pack builds, add the following to your project:\n\n\
         1. Create src/bin/sand_resource_export.rs:\n\n\
         {}fn main() {{ {ns}::__sand_resource_export(\"{ns}\"); }}\n\n\
         2. Add to Cargo.toml:\n\n\
         {}[[bin]]\n\
         {}name = \"sand_resource_export\"\n\
         {}path = \"src/bin/sand_resource_export.rs\"\n\n\
         3. Add to src/lib.rs:\n\n\
         {}#[doc(hidden)]\n\
         {}pub fn __sand_resource_export(namespace: &str) {{\n\
         {}    println!(\"{{}}\", sand_resourcepack::export_resourcepack_json(namespace));\n\
         {}}}\n",
        "    ",
        "    ",
        "    ",
        "    ",
        "    ",
        "    ",
        "    ",
        "    ",
        ns = resourcepack_namespace(config)
    );
}

/// Runs the already-compiled resource exporter and writes the resource pack.
///
/// `binary` is compiled alongside the datapack exporter by
/// [`super::export::ExportBuildPlan`]; this function never invokes Cargo. The
/// records it parses, validates, and writes stay entirely within the resource
/// pack's own output root, and copy sources are resolved against the same
/// `project_root` the datapack half used.
pub(super) fn build_resourcepack(
    config: &SandConfig,
    project_root: &Path,
    mc_version: &str,
    release: bool,
    binary: &Path,
) -> Result<()> {
    use sand_core::version::{MinecraftVersion, VersionProfile};

    let rp_cfg = config.resourcepack.as_ref();
    let rp_namespace = resourcepack_namespace(config);
    let rp_description = rp_cfg
        .and_then(|c| c.description.as_deref())
        .unwrap_or(&config.pack.description);

    let (rp_format, rp_format_is_fallback) =
        if let Some(explicit) = rp_cfg.and_then(|c| c.resource_pack_format) {
            (explicit, false)
        } else if let Ok(v) = MinecraftVersion::parse(mc_version) {
            let p = VersionProfile::resolve(&v).unwrap_or_else(|_| {
                VersionProfile::resolve(
                    &MinecraftVersion::parse(sand_core::version::LATEST_KNOWN).unwrap(),
                )
                .unwrap()
            });
            let meta = p.resourcepack_metadata();
            (meta.pack_format, meta.is_fallback)
        } else {
            (
                sand_resourcepack::resource_pack_format_for(mc_version),
                false,
            )
        };

    if rp_format_is_fallback {
        eprintln!(
            "{} Minecraft version '{}' is not in Sand's known version table. \
             Using resource_pack_format {} as a conservative fallback. \
             Add `resource_pack_format = {}` to [resourcepack] in sand.toml to silence this warning.",
            "warning:".yellow().bold(),
            mc_version,
            rp_format,
            rp_format
        );
    }

    println!(
        "{} {} (resource_pack_format {})...",
        "Building resourcepack".cyan().bold(),
        rp_namespace.white().bold(),
        rp_format.to_string().yellow()
    );

    // Run the resource export binary (compiled alongside the datapack exporter).
    let stdout = run_exporter(Exporter::ResourcePack, binary, &[])?;

    // Parse resource pack records — a separate stream from the datapack's
    // ComponentRecords, validated against its own rules.
    let records: Vec<ResourcePackRecord> =
        serde_json::from_slice(&stdout).context("failed to parse resource pack export JSON")?;

    validate_resourcepack_records_for_project(project_root, &records)?;

    // Write pack.mcmeta for the resource pack.
    let rp_dist_name = format!("{}-resources", config.pack.namespace.as_str());
    let rp_dist = PathBuf::from("dist").join(&rp_dist_name);
    std::fs::create_dir_all(&rp_dist)?;
    write_resourcepack_mcmeta(&rp_dist, rp_description, rp_format)?;

    // Write each resource pack record.
    let mut written = 0usize;
    for record in &records {
        write_rp_record(&rp_dist, project_root, record)?;
        written += 1;
    }

    println!(
        "{} {} asset(s) written to {}",
        "Done!".green().bold(),
        written.to_string().white().bold(),
        format!("dist/{}/", rp_dist_name).white().bold()
    );

    if release {
        let zip_path = zip_dir(&rp_dist, &rp_dist_name)?;
        println!(
            "  {} {}",
            "zip:".dimmed(),
            zip_path.display().to_string().white().bold()
        );
        println!(
            "  {} drop {} into your world's resourcepacks/ folder",
            "install:".dimmed(),
            format!("dist/{}.zip", rp_dist_name).white().bold()
        );
    } else {
        println!(
            "  {} copy the {} folder into your world's resourcepacks/ folder",
            "install:".dimmed(),
            format!("dist/{}/", rp_dist_name).white().bold()
        );
    }

    Ok(())
}
