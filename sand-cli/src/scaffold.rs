use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use handlebars::Handlebars;
use serde_json::json;

use crate::pack_format::pack_format_for;

// ── Embedded templates ────────────────────────────────────────────────────────

const CARGO_TOML_HBS: &str = include_str!("templates/default/Cargo.toml.hbs");
const BUILD_RS_HBS: &str = include_str!("templates/default/build.rs.hbs");
const SAND_TOML_HBS: &str = include_str!("templates/default/sand.toml.hbs");
const SRC_LIB_RS_HBS: &str = include_str!("templates/default/src_lib_rs.hbs");
const SAND_EXPORT_RS_HBS: &str = include_str!("templates/default/sand_export_rs.hbs");

// Embedded at compile time by sand/build.rs.
pub(crate) const WORKSPACE_ROOT: &str = env!("SAND_WORKSPACE_ROOT");

/// The Sand workspace version — surfaced in scaffolded projects for
/// diagnostics; no longer used to build dependency version strings, since
/// Sand is not published to crates.io (see `SAND_GIT_URL`).
const SAND_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Sand GitHub repository URL. Scaffolded projects that don't use
/// `--path-deps` depend on this repo's `main` branch via a git dependency,
/// since Sand has no crates.io release yet.
const SAND_GIT_URL: &str = env!("CARGO_PKG_REPOSITORY");

// ── Public API ────────────────────────────────────────────────────────────────

/// All parameters needed to scaffold a new datapack project.
pub struct ScaffoldOptions {
    /// Cargo package name (e.g. `my_pack`).
    pub name: String,
    /// MC namespace — same as `name` with hyphens replaced by underscores.
    pub namespace: String,
    /// Short description shown in the datapack menu.
    pub description: String,
    /// Minecraft version string (e.g. `"26.2"`).
    pub mc_version: String,
    /// Root directory to create/populate.
    pub dir: PathBuf,
    /// When `true`, scaffolded `Cargo.toml` uses local `path = "..."` deps
    /// pointing into the Sand workspace (useful for Sand contributors).
    ///
    /// When `false` (the default), git dependencies are emitted instead:
    /// `sand-core = { git = "https://github.com/ThatOneToast/sand", branch = "main" }`.
    /// Sand has no crates.io release yet, so this is the only dependency form
    /// that resolves without a local Sand workspace checkout.
    pub use_path_deps: bool,
}

/// Validate a project name and return `Err` with a user-friendly message if
/// it doesn't meet Cargo/MC namespace naming rules.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("project name cannot be empty");
    }
    if !name
        .chars()
        .next()
        .map(|c| c.is_ascii_lowercase())
        .unwrap_or(false)
    {
        bail!("project name must start with a lowercase letter, got '{name}'");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        bail!(
            "project name '{name}' contains invalid characters — \
             use only lowercase letters, digits, underscores, or hyphens"
        );
    }
    Ok(())
}

/// Derive the MC namespace from a project name by replacing hyphens with
/// underscores (Cargo allows hyphens; MC namespaces don't).
pub fn name_to_namespace(name: &str) -> String {
    name.replace('-', "_")
}

/// Create a new project at `opts.dir`, render all templates, and run
/// `cargo build` to pre-warm the cache.
pub fn scaffold(opts: &ScaffoldOptions) -> Result<()> {
    let dir = &opts.dir;

    if dir.exists() && dir.read_dir()?.next().is_some() {
        bail!(
            "directory '{}' already exists and is not empty",
            dir.display()
        );
    }

    write_scaffold_files(opts)?;
    run_cargo_build(dir)?;

    Ok(())
}

/// Write all project files for `opts` without running `cargo build`.
///
/// Used by the CLI's `sand new` flow (via [`scaffold`]) and by integration
/// tests that need to inspect the generated file layout without the expense
/// of a full Cargo compilation.
pub fn write_scaffold_files(opts: &ScaffoldOptions) -> Result<()> {
    let dir = &opts.dir;
    let pack_format = pack_format_for(&opts.mc_version)?;

    // Create directory structure.
    std::fs::create_dir_all(dir.join("src/bin"))
        .with_context(|| format!("failed to create project directory '{}'", dir.display()))?;

    let sand_path = format!("{}/sand", WORKSPACE_ROOT);
    let sand_build_path = format!("{}/sand-build", WORKSPACE_ROOT);

    let ctx = json!({
        "name":                   opts.name,
        "name_snake":             opts.namespace,
        "namespace":              opts.namespace,
        "description":            opts.description,
        "mc_version":             opts.mc_version,
        "pack_format":            pack_format,
        "sand_path":              sand_path,
        "sand_build_path":        sand_build_path,
        "use_path_deps":          opts.use_path_deps,
        "sand_version":           SAND_VERSION,
        "sand_git_url":           SAND_GIT_URL,
    });

    let hbs = build_handlebars();

    write_rendered(
        &hbs,
        "cargo_toml",
        CARGO_TOML_HBS,
        &ctx,
        &dir.join("Cargo.toml"),
    )?;
    write_rendered(&hbs, "build_rs", BUILD_RS_HBS, &ctx, &dir.join("build.rs"))?;
    write_rendered(
        &hbs,
        "sand_toml",
        SAND_TOML_HBS,
        &ctx,
        &dir.join("sand.toml"),
    )?;
    write_rendered(
        &hbs,
        "src_lib_rs",
        SRC_LIB_RS_HBS,
        &ctx,
        &dir.join("src/lib.rs"),
    )?;
    write_rendered(
        &hbs,
        "sand_export_rs",
        SAND_EXPORT_RS_HBS,
        &ctx,
        &dir.join("src/bin/sand_export.rs"),
    )?;

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn build_handlebars() -> Handlebars<'static> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    hbs.register_escape_fn(handlebars::no_escape);
    hbs
}

pub(crate) fn write_rendered(
    hbs: &Handlebars,
    name: &str,
    template: &str,
    ctx: &serde_json::Value,
    dest: &Path,
) -> Result<()> {
    let rendered = hbs
        .render_template(template, ctx)
        .with_context(|| format!("failed to render template '{name}'"))?;
    std::fs::write(dest, rendered)
        .with_context(|| format!("failed to write '{}'", dest.display()))?;
    Ok(())
}

fn run_cargo_build(dir: &Path) -> Result<()> {
    println!(
        "  {} {} (this may take a while on the first run...)",
        "Running".dimmed(),
        "`cargo build`".white()
    );

    let status = std::process::Command::new("cargo")
        .arg("build")
        // Intentionally no RUSTFLAGS override: `sand build` compiles exporters
        // (see `build::export::ExportBuildPlan::compile`) with the same plain
        // dev profile and no exporter-specific compiler flags, so this
        // pre-warm shares Cargo's fingerprint/artifact cache with the first
        // real `sand build` instead of being thrown away by a flag mismatch.
        .current_dir(dir)
        .status()
        .context("failed to invoke `cargo build`")?;

    if !status.success() {
        bail!("`cargo build` failed in '{}'", dir.display());
    }
    Ok(())
}
