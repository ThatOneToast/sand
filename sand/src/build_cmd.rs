use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::Deserialize;

use crate::config::SandConfig;
use crate::pack_format::pack_format_for;
use sand_resourcepack::resource_pack_format_for;

// ── Datapack record (from sand_export) ────────────────────────────────────────

#[derive(Deserialize)]
struct ComponentRecord {
    namespace: String,
    dir: String,
    path: String,
    ext: String,
    content: String,
}

// ── Resource pack record (from sand_resource_export) ─────────────────────────

#[derive(Deserialize)]
struct ResourcePackRecord {
    /// Full path from the pack root, e.g. `"assets/ns/font/hud.json"`.
    path: String,
    /// `"json"` — write `content` as UTF-8 text.
    /// `"copy"` — copy the file at `content` (project-root-relative path).
    content_type: String,
    /// JSON string or project-root-relative source path.
    content: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(release: bool, resourcepack: bool) -> Result<()> {
    // 1. Read sand.toml
    let config_path = std::env::current_dir()?.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found — run `sand build` from your project root");
    }
    let config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
        .context("failed to parse sand.toml")?;

    // Resolve mc_version ("latest" → actual version from Mojang manifest)
    let mc_version = resolve_mc_version(&config.pack.mc_version);
    let pack_format = config
        .pack
        .pack_format
        .unwrap_or_else(|| pack_format_for(&mc_version));

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
    let records: Vec<ComponentRecord> = serde_json::from_slice(&output.stdout).map_err(|e| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let hint = if stdout.contains("export_resourcepack_json") {
            "\n\nHint: it looks like __sand_export is calling \
             export_resourcepack_json. Resource pack output must go in \
             __sand_resource_export (src/bin/sand_resource_export.rs), \
             not in the datapack export. Remove the \
             sand_resourcepack::export_resourcepack_json call from \
             __sand_export in src/lib.rs."
        } else if stdout.trim_start().starts_with('[') && stdout.matches('[').count() > 1 {
            "\n\nHint: the export binary printed more than one JSON value. \
             __sand_export must print exactly one JSON array \
             (from sand_core::export_components_json). Resource pack \
             output belongs in __sand_resource_export instead."
        } else {
            ""
        };
        anyhow::anyhow!("failed to parse component export JSON: {}{}", e, hint)
    })?;

    // 5. Write pack.mcmeta
    let dist = PathBuf::from("dist").join(&config.pack.namespace);
    std::fs::create_dir_all(&dist)?;
    write_pack_mcmeta(
        &dist,
        &config.pack.namespace,
        &config.pack.description,
        pack_format,
    )?;

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
        let zip_path = zip_dir(&dist, &config.pack.namespace)?;
        println!(
            "  {} {}",
            "zip:".dimmed(),
            zip_path.display().to_string().white().bold()
        );
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

    // 8. Resource pack build (optional, --resourcepack flag)
    if resourcepack {
        build_resourcepack(&config, &mc_version, release)?;
    }

    Ok(())
}

// ── Resource pack build ───────────────────────────────────────────────────────

fn build_resourcepack(config: &SandConfig, mc_version: &str, release: bool) -> Result<()> {
    // Resolve resource pack config, falling back to pack defaults.
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
    let binary = std::env::current_dir()?
        .join("target")
        .join(profile)
        .join("sand_resource_export");
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

fn write_resourcepack_mcmeta(dist: &Path, description: &str, pack_format: u32) -> Result<()> {
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

fn write_rp_record(dist: &Path, project_root: &Path, record: &ResourcePackRecord) -> Result<()> {
    // The `path` field is already a full pack-relative path, e.g.
    // "assets/my_pack/font/hud.json". Strip any leading separator just in
    // case, then join to the dist directory.
    let rel = record.path.trim_start_matches('/');
    let dest = dist.join(rel);
    std::fs::create_dir_all(dest.parent().unwrap())
        .with_context(|| format!("failed to create dir for '{}'", dest.display()))?;

    match record.content_type.as_str() {
        "json" => {
            std::fs::write(&dest, &record.content)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }
        "copy" => {
            let src = project_root.join(&record.content);
            if !src.exists() {
                bail!(
                    "resource pack asset not found: '{}'\n\
                     Make sure the file exists relative to your project root.",
                    src.display()
                );
            }
            std::fs::copy(&src, &dest).with_context(|| {
                format!("failed to copy '{}' → '{}'", src.display(), dest.display())
            })?;
        }
        "bytes" => {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&record.content)
                .with_context(|| format!("failed to base64-decode '{}'", record.path))?;
            std::fs::write(&dest, &bytes)
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }
        other => {
            bail!(
                "unknown resource pack content_type '{}' for '{}'",
                other,
                record.path
            );
        }
    }

    Ok(())
}

// ── Shared helpers ────────────────────────────────────────────────────────────

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

fn zip_dir(dist: &Path, name: &str) -> Result<PathBuf> {
    let zip_path = dist.parent().unwrap().join(format!("{name}.zip"));
    let file = std::fs::File::create(&zip_path)
        .with_context(|| format!("failed to create zip '{}'", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(dist).sort_by_file_name() {
        let entry = entry?;
        let abs = entry.path();
        if abs.is_file() {
            // Strip dist itself so pack.mcmeta and assets/ sit at the zip root,
            // which is what Minecraft requires.
            let rel = abs.strip_prefix(dist)?;
            zip.start_file(rel.to_str().context("non-UTF-8 path")?, options)?;
            zip.write_all(&std::fs::read(abs)?)?;
        }
    }
    zip.finish()?;
    Ok(zip_path)
}
