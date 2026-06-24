use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use colored::Colorize;
use sand_resourcepack::resource_pack_format_for;

use crate::config::SandConfig;

use super::package::zip_dir;
use super::records::ResourcePackRecord;
use super::validate::validate_resourcepack_records;
use super::write::{write_resourcepack_mcmeta, write_rp_record};

pub(super) fn build_resourcepack(
    config: &SandConfig,
    mc_version: &str,
    release: bool,
    cargo_target_dir: &std::path::Path,
) -> Result<()> {
    let rp_cfg = config.resourcepack.as_ref();
    let rp_namespace = rp_cfg
        .and_then(|c| c.namespace.as_deref())
        .unwrap_or(&config.pack.namespace);
    let rp_description = rp_cfg
        .and_then(|c| c.description.as_deref())
        .unwrap_or(&config.pack.description);
    let rp_format = rp_cfg
        .and_then(|c| c.resource_pack_format)
        .unwrap_or_else(|| resource_pack_format_for(mc_version));

    println!(
        "{} {} (resource_pack_format {})...",
        "Building resourcepack".cyan().bold(),
        rp_namespace.white().bold(),
        rp_format.to_string().yellow()
    );

    // Check that the resource export binary source exists before attempting
    // compilation so we can emit a helpful error message.
    let export_src = std::env::current_dir()?.join("src/bin/sand_resource_export.rs");
    if !export_src.exists() {
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
            ns = rp_namespace
        );
    }

    // Compile the resource export binary.
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--bin", "sand_resource_export"]);
    if release {
        cmd.arg("--release");
    }
    cmd.env("RUSTFLAGS", "-Awarnings");
    let status = cmd.status().context("failed to invoke `cargo build`")?;
    if !status.success() {
        bail!("`cargo build --bin sand_resource_export` failed");
    }

    // Run the resource export binary.
    let profile = if release { "release" } else { "debug" };
    let binary = cargo_target_dir.join(profile).join("sand_resource_export");
    let output = std::process::Command::new(&binary)
        .output()
        .with_context(|| format!("failed to run '{}'", binary.display()))?;
    if !output.status.success() {
        bail!(
            "resource export binary failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Parse resource pack records.
    let records: Vec<ResourcePackRecord> = serde_json::from_slice(&output.stdout)
        .context("failed to parse resource pack export JSON")?;

    validate_resourcepack_records(&records)?;

    // Write pack.mcmeta for the resource pack.
    let rp_dist_name = format!("{}-resources", config.pack.namespace);
    let rp_dist = PathBuf::from("dist").join(&rp_dist_name);
    std::fs::create_dir_all(&rp_dist)?;
    write_resourcepack_mcmeta(&rp_dist, rp_description, rp_format)?;

    // Write each resource pack record.
    let project_root = std::env::current_dir()?;
    let mut written = 0usize;
    for record in &records {
        write_rp_record(&rp_dist, &project_root, record)?;
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
