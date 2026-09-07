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

/// Reserve the resource-pack capability name until Sand ships its replacement.
///
/// This command intentionally performs no filesystem writes.
pub fn run_resourcepack() -> Result<()> {
    bail!(
        "Sand resource-pack support is temporarily unavailable and planned for a future implementation"
    )
}

/// Load and parse `sand.toml` from the current directory.
fn load_config() -> Result<SandConfig> {
    let path = "sand.toml";
    if !std::path::Path::new(path).exists() {
        bail!("sand.toml not found — run this command from your project root");
    }
    toml::from_str(&std::fs::read_to_string(path)?).context("failed to parse sand.toml")
}
