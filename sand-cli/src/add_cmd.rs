//! `sand add` — add features to an existing Sand project.

use anyhow::{Context, Result, bail};
use colored::Colorize;

use crate::config::SandConfig;
use crate::scaffold::{build_handlebars, write_rendered};
const SAND_BUILD_RS_HBS: &str = include_str!("templates/default/sand_build_rs.hbs");

// ── `sand add worldbuild` ─────────────────────────────────────────────────────

/// Add a typed `sand.build.rs` world/server-configuration script to an
/// existing Sand project (issue #317).
///
/// Idempotent — if `sand.build.rs` already exists, or the `sand_build_world`
/// bin target is already wired in Cargo.toml, this prints a notice and
/// leaves those files untouched (Cargo.toml still gets the `[[bin]]` entry
/// if it's missing but `sand.build.rs` already exists by hand).
pub fn run_worldbuild() -> Result<()> {
    let config = load_config()?;
    let namespace = config.pack.namespace.as_str().to_string();

    println!(
        "{} sand.build.rs to {}...",
        "Adding".cyan().bold(),
        namespace.white().bold()
    );

    let build_rs_path = std::path::PathBuf::from("sand.build.rs");
    if build_rs_path.exists() {
        println!("  {} sand.build.rs already exists", "skipped".dimmed());
    } else {
        let hbs = build_handlebars();
        let ctx = serde_json::json!({ "namespace": namespace });
        write_rendered(
            &hbs,
            "sand_build_rs",
            SAND_BUILD_RS_HBS,
            &ctx,
            &build_rs_path,
        )?;
        println!("  {} sand.build.rs", "created".green());
    }

    let cargo_toml_src =
        std::fs::read_to_string("Cargo.toml").context("failed to read Cargo.toml")?;
    if cargo_toml_src.contains("sand_build_world") {
        println!(
            "  {} Cargo.toml already has a sand_build_world bin target",
            "skipped".dimmed()
        );
    } else {
        let mut new_content = cargo_toml_src.trim_end().to_string();
        new_content
            .push_str("\n\n[[bin]]\nname = \"sand_build_world\"\npath = \"sand.build.rs\"\n");
        std::fs::write("Cargo.toml", new_content).context("failed to write Cargo.toml")?;
        println!("  {} Cargo.toml", "updated".green());
    }

    println!();
    println!("{}", "Done! Next steps:".green().bold());
    println!(
        "  1. Edit {} — the starter script already branches on dev vs. release",
        "sand.build.rs".white().bold()
    );
    println!(
        "  2. Run {} to build the dev profile, or {} for release",
        "`sand build`".white().bold(),
        "`sand build --release`".white().bold()
    );
    println!(
        "  3. See the {} mdBook chapter for the full typed API",
        "\"Build scripts\"".white().bold()
    );

    Ok(())
}

// ── `sand migrate` ────────────────────────────────────────────────────────────

/// Migrate legacy `sand.toml` world/server fields to `sand.build.rs`
/// (issue #317).
///
/// `sand.toml`'s `[pack]` section carries no world- or server-shaped fields to migrate away from —
/// Sand never had built-in `sand.toml` world/server configuration before
/// this issue. This command is therefore forward-looking scaffolding: it
/// prints that finding explicitly (rather than silently claiming a
/// migration happened) and then runs the same scaffolding as
/// `sand add worldbuild` so a project can start using the typed API.
pub fn run_migrate() -> Result<()> {
    let _ = load_config()?; // fail fast with a clear "not a Sand project" error

    println!("{}", "sand migrate".cyan().bold());
    println!();
    println!(
        "  {} sand.toml has no world/server fields to migrate. Sand's \
         configuration surface is limited to [pack] (namespace, description, \
         mc_version, pack_format, supported_formats, overlays), which does not cover world generation, \
         dimensions, gamerules, or server bootstrap settings, so there is \
         nothing to move out of sand.toml.",
        "Note:".yellow().bold()
    );
    println!();
    println!(
        "  Scaffolding a starter sand.build.rs so you can start using the \
         typed World/ServerConfig API described in the \"Migrating from \
         Sand.toml\" mdBook chapter:"
    );
    println!();

    run_worldbuild()
}

// ── `sand add resourcepack` ───────────────────────────────────────────────────

/// Opt in to companion client assets without changing existing source files.
/// Creates a project-local asset directory and appends `[resource_pack]` only
/// when absent. Repeated calls preserve the configuration and authored assets.
pub fn run_resourcepack() -> Result<()> {
    let project = std::env::current_dir()?;
    scaffold_resourcepack_at(&project)?;
    println!(
        "{} companion resource pack configured; add client assets under the configured source/assets/ directory",
        "Done!".green().bold()
    );
    println!(
        "  Run `sand build` for separate datapack and resource-pack folders, or `sand build --release` for both zip files."
    );
    Ok(())
}

fn scaffold_resourcepack_at(project: &std::path::Path) -> Result<()> {
    use crate::build::{output_manifest::reject_symlink_ancestors, resource_pack};
    use std::io::Write;
    let path = project.join("sand.toml");
    reject_symlink_ancestors(&path)?;
    let original = std::fs::read_to_string(&path)
        .context("sand.toml not found — run this command from your project root")?;
    let mut config: SandConfig = toml::from_str(&original).context("failed to parse sand.toml")?;
    let append = config.resource_pack.is_none();
    let configured = if append {
        format!(
            "{}\n\n[resource_pack]\nsource = \"resourcepack\"\n",
            original.trim_end()
        )
    } else {
        original.clone()
    };
    if append {
        config = toml::from_str(&configured)?;
    }
    let resources = config
        .resource_pack
        .as_ref()
        .expect("resource configuration");
    resource_pack::format_for(&config.pack.mc_version, resources.pack_format)?;
    let source = resource_pack::source_path(project, resources)?;
    anyhow::ensure!(
        !source.starts_with(project.join("dist")),
        "resource pack source must be outside dist"
    );
    if source.exists() {
        resource_pack::prepare(project, &config, &config.pack.mc_version)?;
    }
    let asset_root = source.join("assets").join(config.pack.namespace.as_str());
    anyhow::ensure!(
        !matches!(config.pack.namespace.as_str(), "." | ".."),
        "resource pack namespace must name a directory"
    );
    reject_symlink_ancestors(&asset_root)?;
    let mut created = Vec::new();
    let result = (|| {
        let mut current = project.to_path_buf();
        for part in asset_root.strip_prefix(project)?.components() {
            current.push(part);
            if !current.exists() {
                std::fs::create_dir(&current)?;
                created.push(current.clone());
            } else {
                anyhow::ensure!(
                    current.is_dir(),
                    "asset source parent is not a directory: {}",
                    current.display()
                );
            }
        }
        let readme = source.join("README.md");
        if !readme.exists() {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&readme)?;
            created.push(readme);
            file.write_all(b"# Companion resource pack\n\nPlace client assets under assets/<namespace>/ using Minecraft resource paths.\nOptional pack.png belongs here. Sand generates pack.mcmeta from [resource_pack]\nin sand.toml; do not add a source pack.mcmeta. This README is not packaged.\n\nRun sand build for separate dist/<namespace>-resources output, or\nsand build --release for a companion zip. Sand packages asset bytes and checks\nJSON syntax; verify models, textures, sounds and fonts in a Minecraft client.\nClient installation and server resource-pack delivery are not performed.\n")?;
        }
        if append {
            let mut temporary = tempfile::NamedTempFile::new_in(project)?;
            temporary.write_all(configured.as_bytes())?;
            temporary
                .as_file()
                .set_permissions(std::fs::metadata(&path)?.permissions())?;
            temporary
                .persist(&path)
                .context("install resource pack configuration")?;
        }
        Ok(())
    })();
    if result.is_err() {
        for path in created.into_iter().rev() {
            if path.is_dir() {
                let _ = std::fs::remove_dir(path);
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    result
}

/// Load and parse `sand.toml` from the current directory.
fn load_config() -> Result<SandConfig> {
    let path = "sand.toml";
    if !std::path::Path::new(path).exists() {
        bail!("sand.toml not found — run this command from your project root");
    }
    toml::from_str(&std::fs::read_to_string(path)?).context("failed to parse sand.toml")
}
