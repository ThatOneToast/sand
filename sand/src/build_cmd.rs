use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Deserialize;

use crate::config::SandConfig;
use crate::pack_format::pack_format_for;

#[derive(Deserialize)]
struct ComponentRecord {
    namespace: String,
    dir: String,
    path: String,
    ext: String,
    content: String,
}

pub fn run(release: bool) -> Result<()> {
    // 1. Read sand.toml
    let config_path = std::env::current_dir()?.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found — run `sand build` from your project root");
    }
    let config: SandConfig =
        toml::from_str(&std::fs::read_to_string(&config_path)?)
            .context("failed to parse sand.toml")?;

    // Resolve mc_version ("latest" → actual version from Mojang manifest)
    let mc_version = resolve_mc_version(&config.pack.mc_version);
    let pack_format = config.pack.pack_format.unwrap_or_else(|| pack_format_for(&mc_version));

    println!(
        "{} {} (Minecraft {}, pack_format {})...",
        "Building".cyan().bold(),
        config.pack.namespace.white().bold(),
        mc_version.yellow(),
        pack_format.to_string().yellow()
    );

    // 2. Compile the export binary
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--bin", "sand_export"]);
    if release {
        cmd.arg("--release");
    }
    // Suppress all compiler warnings during the build — the export binary is a
    // build-time tool, not user-facing code, so warning noise is unhelpful here.
    cmd.env("RUSTFLAGS", "-Awarnings");
    let status = cmd.status().context("failed to invoke `cargo build`")?;
    if !status.success() {
        bail!("`cargo build` failed");
    }

    // 3. Run the export binary
    let profile = if release { "release" } else { "debug" };
    let binary = std::env::current_dir()?
        .join("target")
        .join(profile)
        .join("sand_export");
    let output = std::process::Command::new(&binary)
        .output()
        .with_context(|| format!("failed to run '{}'", binary.display()))?;
    if !output.status.success() {
        bail!(
            "export binary failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // 4. Parse component records
    let records: Vec<ComponentRecord> = serde_json::from_slice(&output.stdout)
        .context("failed to parse component export JSON")?;

    // 5. Write pack.mcmeta
    let dist = PathBuf::from("dist").join(&config.pack.namespace);
    std::fs::create_dir_all(&dist)?;
    write_pack_mcmeta(&dist, &config.pack.namespace, &config.pack.description, pack_format)?;

    // 6. Write each component file
    for record in &records {
        write_component(&dist, record)?;
    }

    println!(
        "{} {} component(s) written to {}",
        "Done!".green().bold(),
        records.len().to_string().white().bold(),
        format!("dist/{}/", config.pack.namespace).white().bold()
    );

    // 7. Zip if --release, otherwise hint how to install manually.
    if release {
        let zip_path = zip_datapack(&dist, &config.pack.namespace)?;
        println!("  {} {}", "zip:".dimmed(), zip_path.display().to_string().white().bold());
        println!(
            "  {} drop {} into your world's datapacks/ folder",
            "install:".dimmed(),
            format!("dist/{}.zip", config.pack.namespace).white().bold()
        );
    } else {
        println!(
            "  {} copy the {} folder into your world's datapacks/ folder, \
             or run `sand build --release` to produce a zip",
            "install:".dimmed(),
            format!("dist/{}/", config.pack.namespace).white().bold()
        );
    }

    Ok(())
}

/// Resolve "latest" to the actual current release from Mojang's manifest.
/// Falls back to the hardcoded default if offline.
fn resolve_mc_version(mc_version: &str) -> String {
    if mc_version == "latest" {
        sand_build::latest_release_version()
    } else {
        mc_version.to_string()
    }
}

fn write_pack_mcmeta(
    dist: &Path,
    namespace: &str,
    description: &str,
    pack_format: u32,
) -> Result<()> {
    let _ = namespace; // available for future use
    let mcmeta = serde_json::json!({
        "pack": {
            "pack_format": pack_format,
            "description": description,
        }
    });
    std::fs::write(
        dist.join("pack.mcmeta"),
        serde_json::to_string_pretty(&mcmeta)?,
    )?;
    Ok(())
}

fn write_component(dist: &Path, record: &ComponentRecord) -> Result<()> {
    // path inside the datapack: data/<namespace>/<dir>/<path>.<ext>
    let file_path = dist
        .join("data")
        .join(&record.namespace)
        .join(&record.dir)
        .join(format!("{}.{}", record.path, record.ext));
    std::fs::create_dir_all(file_path.parent().unwrap())
        .with_context(|| format!("failed to create dir for '{}'", file_path.display()))?;
    std::fs::write(&file_path, &record.content)
        .with_context(|| format!("failed to write '{}'", file_path.display()))?;
    Ok(())
}

fn zip_datapack(dist: &Path, name: &str) -> Result<PathBuf> {
    let zip_path = dist
        .parent()
        .unwrap()
        .join(format!("{name}.zip"));
    let file = std::fs::File::create(&zip_path)
        .with_context(|| format!("failed to create zip '{}'", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(dist).sort_by_file_name() {
        let entry = entry?;
        let abs = entry.path();
        if abs.is_file() {
            // Strip dist itself so pack.mcmeta and data/ sit at the zip root,
            // which is what Minecraft requires.
            let rel = abs.strip_prefix(dist)?;
            zip.start_file(
                rel.to_str().context("non-UTF-8 path")?,
                options,
            )?;
            zip.write_all(&std::fs::read(abs)?)?;
        }
    }
    zip.finish()?;
    Ok(zip_path)
}
