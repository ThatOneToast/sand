use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use colored::Colorize;
use handlebars::Handlebars;
use serde_json::json;

use crate::pack_format::pack_format_for;

// ── Embedded templates ────────────────────────────────────────────────────────

const CARGO_TOML_HBS: &str =
    include_str!("templates/default/Cargo.toml.hbs");
const BUILD_RS_HBS: &str =
    include_str!("templates/default/build.rs.hbs");
const SAND_TOML_HBS: &str =
    include_str!("templates/default/sand.toml.hbs");
const SRC_LIB_RS_HBS: &str =
    include_str!("templates/default/src_lib_rs.hbs");
const SAND_EXPORT_RS_HBS: &str =
    include_str!("templates/default/sand_export_rs.hbs");

// Embedded at compile time by sand/build.rs.
const WORKSPACE_ROOT: &str = env!("SAND_WORKSPACE_ROOT");

// ── Public API ────────────────────────────────────────────────────────────────

/// All parameters needed to scaffold a new datapack project.
pub struct ScaffoldOptions {
    /// Cargo package name (e.g. `my_pack`).
    pub name: String,
    /// MC namespace — same as `name` with hyphens replaced by underscores.
    pub namespace: String,
    /// Short description shown in the datapack menu.
    pub description: String,
    /// Minecraft version string (e.g. `"1.21.4"`).
    pub mc_version: String,
    /// Root directory to create/populate.
    pub dir: PathBuf,
}

/// Validate a project name and return `Err` with a user-friendly message if
/// it doesn't meet Cargo/MC namespace naming rules.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("project name cannot be empty");
    }
    if !name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
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
        bail!("directory '{}' already exists and is not empty", dir.display());
    }

    // Create directory structure.
    std::fs::create_dir_all(dir.join("src/bin"))
        .with_context(|| format!("failed to create project directory '{}'", dir.display()))?;

    let pack_format = pack_format_for(&opts.mc_version);
    let sand_core_path = format!("{}/sand-core", WORKSPACE_ROOT);
    let sand_build_path = format!("{}/sand-build", WORKSPACE_ROOT);
    let sand_macros_path = format!("{}/sand-macros", WORKSPACE_ROOT);

    let ctx = json!({
        "name":            opts.name,
        "name_snake":      opts.namespace,
        "namespace":       opts.namespace,
        "description":     opts.description,
        "mc_version":      opts.mc_version,
        "pack_format":     pack_format,
        "sand_core_path":  sand_core_path,
        "sand_build_path": sand_build_path,
        "sand_macros_path": sand_macros_path,
    });

    let hbs = build_handlebars();

    write_rendered(&hbs, "cargo_toml", CARGO_TOML_HBS, &ctx, &dir.join("Cargo.toml"))?;
    write_rendered(&hbs, "build_rs", BUILD_RS_HBS, &ctx, &dir.join("build.rs"))?;
    write_rendered(&hbs, "sand_toml", SAND_TOML_HBS, &ctx, &dir.join("sand.toml"))?;
    write_rendered(&hbs, "src_lib_rs", SRC_LIB_RS_HBS, &ctx, &dir.join("src/lib.rs"))?;
    write_rendered(&hbs, "sand_export_rs", SAND_EXPORT_RS_HBS, &ctx, &dir.join("src/bin/sand_export.rs"))?;

    run_cargo_build(dir)?;

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_handlebars() -> Handlebars<'static> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    hbs.register_escape_fn(handlebars::no_escape);
    hbs
}

fn write_rendered(
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
        .current_dir(dir)
        .status()
        .context("failed to invoke `cargo build`")?;

    if !status.success() {
        bail!("`cargo build` failed in '{}'", dir.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() {
        assert!(validate_name("my_pack").is_ok());
        assert!(validate_name("my-pack").is_ok());
        assert!(validate_name("pack123").is_ok());
        assert!(validate_name("a").is_ok());
    }

    #[test]
    fn invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("My_pack").is_err()); // uppercase
        assert!(validate_name("1pack").is_err());   // starts with digit
        assert!(validate_name("my pack").is_err()); // space
        assert!(validate_name("my.pack").is_err()); // dot
    }

    #[test]
    fn namespace_conversion() {
        assert_eq!(name_to_namespace("my-pack"), "my_pack");
        assert_eq!(name_to_namespace("my_pack"), "my_pack");
        assert_eq!(name_to_namespace("hello-world-pack"), "hello_world_pack");
    }

    #[test]
    fn scaffold_writes_files() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("test_pack");

        let opts = ScaffoldOptions {
            name: "test_pack".to_string(),
            namespace: "test_pack".to_string(),
            description: "Test pack".to_string(),
            mc_version: "1.21.4".to_string(),
            dir: project_dir.clone(),
        };

        // We can't run cargo build in tests, so test file writing only.
        std::fs::create_dir_all(project_dir.join("src/bin")).unwrap();
        let pack_format = pack_format_for(&opts.mc_version);
        let ctx = serde_json::json!({
            "name":             "test_pack",
            "name_snake":       "test_pack",
            "namespace":        "test_pack",
            "description":      "Test pack",
            "mc_version":       "1.21.4",
            "pack_format":      pack_format,
            "sand_core_path":   "/tmp/sand-core",
            "sand_build_path":  "/tmp/sand-build",
            "sand_macros_path": "/tmp/sand-macros",
        });

        let hbs = build_handlebars();
        write_rendered(&hbs, "sand_toml", SAND_TOML_HBS, &ctx, &project_dir.join("sand.toml"))
            .unwrap();
        write_rendered(&hbs, "src_lib_rs", SRC_LIB_RS_HBS, &ctx, &project_dir.join("src/lib.rs"))
            .unwrap();
        write_rendered(&hbs, "sand_export_rs", SAND_EXPORT_RS_HBS, &ctx, &project_dir.join("src/bin/sand_export.rs"))
            .unwrap();

        let sand_toml = std::fs::read_to_string(project_dir.join("sand.toml")).unwrap();
        assert!(sand_toml.contains("namespace   = \"test_pack\""));
        assert!(sand_toml.contains("mc_version  = \"1.21.4\""));
        // pack_format is now a comment in the template
        assert!(sand_toml.contains("# pack_format"));

        let lib_rs = std::fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
        assert!(lib_rs.contains("#[function]"));
        assert!(lib_rs.contains("Welcome to test_pack!"));
        assert!(lib_rs.contains("#[component]"));
        assert!(lib_rs.contains("__sand_export"));

        let export_rs = std::fs::read_to_string(project_dir.join("src/bin/sand_export.rs")).unwrap();
        assert!(export_rs.contains("__sand_export"));
        assert!(export_rs.contains("test_pack"));
    }
}
