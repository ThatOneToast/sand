use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use handlebars::Handlebars;
use serde_json::json;

use crate::pack_format::pack_format_for;
use sand_resourcepack::resource_pack_format_for;

// ── Embedded templates ────────────────────────────────────────────────────────

const CARGO_TOML_HBS: &str = include_str!("templates/default/Cargo.toml.hbs");
const BUILD_RS_HBS: &str = include_str!("templates/default/build.rs.hbs");
const SAND_TOML_HBS: &str = include_str!("templates/default/sand.toml.hbs");
const SRC_LIB_RS_HBS: &str = include_str!("templates/default/src_lib_rs.hbs");
const SAND_EXPORT_RS_HBS: &str = include_str!("templates/default/sand_export_rs.hbs");
const SAND_RESOURCE_EXPORT_RS_HBS: &str =
    include_str!("templates/default/sand_resource_export_rs.hbs");

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
    /// Minecraft version string (e.g. `"1.21.4"`).
    pub mc_version: String,
    /// Root directory to create/populate.
    pub dir: PathBuf,
    /// Whether to scaffold with resource pack support enabled from the start.
    ///
    /// When `true`:
    /// - `Cargo.toml` gets `sand-resourcepack` dep, `sand-macros` resourcepack
    ///   feature, and a `[[bin]] sand_resource_export` target.
    /// - `sand.toml` gets a `[resourcepack]` section.
    /// - `src/lib.rs` gets the active `__sand_resource_export` hook and macro
    ///   import stubs.
    /// - `src/bin/sand_resource_export.rs` is created.
    pub resourcepack: bool,
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

    // Create directory structure.
    std::fs::create_dir_all(dir.join("src/bin"))
        .with_context(|| format!("failed to create project directory '{}'", dir.display()))?;

    let pack_format = pack_format_for(&opts.mc_version);
    let resource_pack_format = resource_pack_format_for(&opts.mc_version);
    let sand_path = format!("{}/sand", WORKSPACE_ROOT);
    let sand_build_path = format!("{}/sand-build", WORKSPACE_ROOT);

    let ctx = json!({
        "name":                   opts.name,
        "name_snake":             opts.namespace,
        "namespace":              opts.namespace,
        "description":            opts.description,
        "mc_version":             opts.mc_version,
        "pack_format":            pack_format,
        "resource_pack_format":   resource_pack_format,
        "sand_path":              sand_path,
        "sand_build_path":        sand_build_path,
        "resourcepack":           opts.resourcepack,
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

    if opts.resourcepack {
        write_rendered(
            &hbs,
            "sand_resource_export_rs",
            SAND_RESOURCE_EXPORT_RS_HBS,
            &ctx,
            &dir.join("src/bin/sand_resource_export.rs"),
        )?;
        // Create the assets directory placeholder.
        std::fs::create_dir_all(dir.join("src/assets"))
            .with_context(|| format!("failed to create src/assets in '{}'", dir.display()))?;
    }

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
        assert!(validate_name("1pack").is_err()); // starts with digit
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

        std::fs::create_dir_all(project_dir.join("src/bin")).unwrap();
        let pack_format = pack_format_for("1.21.4");
        let resource_pack_format = resource_pack_format_for("1.21.4");
        let ctx = serde_json::json!({
            "name":                   "test_pack",
            "name_snake":             "test_pack",
            "namespace":              "test_pack",
            "description":            "Test pack",
            "mc_version":             "1.21.4",
            "pack_format":            pack_format,
            "resource_pack_format":   resource_pack_format,
            "sand_path":              "/tmp/sand",
            "sand_build_path":        "/tmp/sand-build",
            "resourcepack":           false,
            "use_path_deps":          false,
            "sand_version":           "0.1.0",
            "sand_git_url":           "https://github.com/ThatOneToast/sand",
        });

        let hbs = build_handlebars();
        write_rendered(
            &hbs,
            "sand_toml",
            SAND_TOML_HBS,
            &ctx,
            &project_dir.join("sand.toml"),
        )
        .unwrap();
        write_rendered(
            &hbs,
            "src_lib_rs",
            SRC_LIB_RS_HBS,
            &ctx,
            &project_dir.join("src/lib.rs"),
        )
        .unwrap();
        write_rendered(
            &hbs,
            "sand_export_rs",
            SAND_EXPORT_RS_HBS,
            &ctx,
            &project_dir.join("src/bin/sand_export.rs"),
        )
        .unwrap();

        // sand.toml.hbs is short and fully deterministic for this fixed
        // context, so pin the whole rendered file rather than piecemeal
        // substring checks.
        let sand_toml = std::fs::read_to_string(project_dir.join("sand.toml")).unwrap();
        assert_eq!(
            sand_toml,
            "[pack]\n\
             namespace   = \"test_pack\"\n\
             description = \"Test pack\"\n\
             mc_version  = \"1.21.4\"\n\
             # pack_format is derived automatically from mc_version; uncomment to override:\n\
             # pack_format = 61\n\
             \n"
        );

        // src_lib_rs.hbs is a long, doc-comment-heavy template; pinning the
        // whole file would make this brittle to unrelated doc wording
        // changes, so we assert on the markers that matter for this test.
        let lib_rs = std::fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
        assert!(lib_rs.contains("#[function]"));
        assert!(lib_rs.contains("Welcome to test_pack!"));
        // Join detection uses Sand's native OnJoinEvent, not a hand-written
        // advancement/tick #[component].
        assert!(lib_rs.contains("#[event]"));
        assert!(lib_rs.contains("OnJoinEvent"));
        assert!(lib_rs.contains("Event<OnJoinEvent>"));
        assert!(!lib_rs.contains("#[component]"));
        assert!(!lib_rs.contains("AdvancementTrigger::Tick"));
        assert!(lib_rs.contains("__sand_export"));
        // Attribute-first: scaffold uses typed commands, not raw mcfunction!
        assert!(lib_rs.contains("use sand::prelude::*"));
        assert!(lib_rs.contains("cmd::tellraw("));
        assert!(lib_rs.contains("cmd::call(hello_world)"));
        assert!(lib_rs.contains("Text::new("));
        assert!(lib_rs.contains("Selector::self_()"));
        // No raw mcfunction usage in generated beginner code
        assert!(!lib_rs.contains("mcfunction!"));
        assert!(!lib_rs.contains("tellraw @"));
        assert!(!lib_rs.contains("playsound minecraft:"));
        // Commented-out snippet mentions the name, but the active definition
        // must NOT be present when resourcepack: false.
        assert!(!lib_rs.contains("\npub fn __sand_resource_export"));

        // sand_export_rs.hbs is short and fully deterministic for this fixed
        // context, so pin the whole rendered file.
        let export_rs =
            std::fs::read_to_string(project_dir.join("src/bin/sand_export.rs")).unwrap();
        assert_eq!(
            export_rs,
            "//! Generated by `sand` — do not edit by hand.\n\
             //!\n\
             //! Invoked by `sand build` to collect all registered datapack components and\n\
             //! print them as JSON. `#[function]` and `#[component]` items are discovered\n\
             //! automatically via inventory — no manual registration needed.\n\
             \n\
             fn main() {\n\
             \x20   let mc_version = std::env::var(\"SAND_EXPORT_MC_VERSION\").unwrap_or_else(|_| {\n\
             \x20       eprintln!(\n\
             \x20           \"sand export failed: SAND_EXPORT_MC_VERSION is missing; invoke this binary through `sand build`\"\n\
             \x20       );\n\
             \x20       std::process::exit(1);\n\
             \x20   });\n\
             \x20   test_pack::__sand_export(\"test_pack\", &mc_version);\n\
             }\n"
        );
    }

    #[test]
    fn scaffold_with_resourcepack_flag() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("rp_pack");

        std::fs::create_dir_all(project_dir.join("src/bin")).unwrap();
        let pack_format = pack_format_for("1.21.4");
        let resource_pack_format = resource_pack_format_for("1.21.4");
        let ctx = serde_json::json!({
            "name":                   "rp_pack",
            "name_snake":             "rp_pack",
            "namespace":              "rp_pack",
            "description":            "Test pack",
            "mc_version":             "1.21.4",
            "pack_format":            pack_format,
            "resource_pack_format":   resource_pack_format,
            "sand_path":              "/tmp/sand",
            "sand_build_path":        "/tmp/sand-build",
            "resourcepack":           true,
            "use_path_deps":          false,
            "sand_version":           "0.1.0",
            "sand_git_url":           "https://github.com/ThatOneToast/sand",
        });

        let hbs = build_handlebars();
        write_rendered(
            &hbs,
            "sand_toml",
            SAND_TOML_HBS,
            &ctx,
            &project_dir.join("sand.toml"),
        )
        .unwrap();
        write_rendered(
            &hbs,
            "src_lib_rs",
            SRC_LIB_RS_HBS,
            &ctx,
            &project_dir.join("src/lib.rs"),
        )
        .unwrap();
        write_rendered(
            &hbs,
            "cargo_toml",
            CARGO_TOML_HBS,
            &ctx,
            &project_dir.join("Cargo.toml"),
        )
        .unwrap();
        write_rendered(
            &hbs,
            "sand_resource_export_rs",
            SAND_RESOURCE_EXPORT_RS_HBS,
            &ctx,
            &project_dir.join("src/bin/sand_resource_export.rs"),
        )
        .unwrap();

        // sand.toml.hbs is short and fully deterministic for this fixed
        // context, so pin the whole rendered file.
        let sand_toml = std::fs::read_to_string(project_dir.join("sand.toml")).unwrap();
        assert_eq!(
            sand_toml,
            "[pack]\n\
             namespace   = \"rp_pack\"\n\
             description = \"Test pack\"\n\
             mc_version  = \"1.21.4\"\n\
             # pack_format is derived automatically from mc_version; uncomment to override:\n\
             # pack_format = 61\n\
             \n\
             [resourcepack]\n\
             description = \"Test pack\"\n\
             # namespace defaults to [pack].namespace; uncomment to override:\n\
             # namespace = \"rp_pack\"\n\
             # resource_pack_format is derived automatically; uncomment to override:\n\
             # resource_pack_format = 46\n"
        );

        let lib_rs = std::fs::read_to_string(project_dir.join("src/lib.rs")).unwrap();
        assert!(lib_rs.contains("pub fn __sand_resource_export"));

        // Cargo.toml.hbs is short and fully deterministic for this fixed
        // context (resourcepack: true, use_path_deps: false), so pin the
        // whole rendered file instead of checking fragments piecemeal.
        let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert_eq!(
            cargo_toml,
            "[package]\n\
             name = \"rp_pack\"\n\
             version = \"0.1.0\"\n\
             edition = \"2024\"\n\
             \n\
             [lib]\n\
             name = \"rp_pack\"\n\
             path = \"src/lib.rs\"\n\
             \n\
             [[bin]]\n\
             name = \"sand_export\"\n\
             path = \"src/bin/sand_export.rs\"\n\
             \n\
             [[bin]]\n\
             name = \"sand_resource_export\"\n\
             path = \"src/bin/sand_resource_export.rs\"\n\
             \n\
             [dependencies]\n\
             sand = { git = \"https://github.com/ThatOneToast/sand\", branch = \"main\", features = [\"resourcepack\"] }\n\
             \n\
             [build-dependencies]\n\
             sand-build = { git = \"https://github.com/ThatOneToast/sand\", branch = \"main\" }\n"
        );

        let resource_export =
            std::fs::read_to_string(project_dir.join("src/bin/sand_resource_export.rs")).unwrap();
        assert!(resource_export.contains("__sand_resource_export"));
    }

    #[test]
    fn default_scaffold_emits_git_deps() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("ver_pack");
        std::fs::create_dir_all(project_dir.join("src/bin")).unwrap();

        let ctx = serde_json::json!({
            "name":                   "ver_pack",
            "name_snake":             "ver_pack",
            "namespace":              "ver_pack",
            "description":            "Test",
            "mc_version":             "1.21.4",
            "pack_format":            61,
            "resource_pack_format":   46,
            "sand_path":              "/should/not/appear",
            "sand_build_path":        "/should/not/appear",
            "resourcepack":           false,
            "use_path_deps":          false,
            "sand_version":           "0.1.0",
            "sand_git_url":           "https://github.com/ThatOneToast/sand",
        });

        let hbs = build_handlebars();
        write_rendered(
            &hbs,
            "cargo_toml",
            CARGO_TOML_HBS,
            &ctx,
            &project_dir.join("Cargo.toml"),
        )
        .unwrap();

        let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(
            cargo_toml.contains("sand = { git"),
            "single `sand` authoring dep must be present"
        );
        assert!(
            cargo_toml.contains("sand-build"),
            "sand-build build-dependency must be present"
        );
        assert!(
            !cargo_toml.contains("sand-core") && !cargo_toml.contains("sand-macros"),
            "generated projects must not depend on internal Sand crates"
        );
        assert!(
            cargo_toml.contains("git = \"https://github.com/ThatOneToast/sand\""),
            "default scaffold must depend on the Sand GitHub repo — Sand is not on crates.io"
        );
        assert!(
            cargo_toml.contains("branch = \"main\""),
            "default scaffold must track the main branch, not a tag/rev"
        );
        assert!(
            !cargo_toml.contains("sand = \""),
            "default scaffold must not emit a bare crates.io version string for sand"
        );
        assert!(
            !cargo_toml.contains("sand = { path"),
            "default scaffold must not emit a path dep for sand"
        );
        assert!(
            !cargo_toml.contains("sand-build = { path"),
            "default scaffold must not emit path dep for sand-build"
        );
        assert!(
            !cargo_toml.contains("/should/not/appear"),
            "workspace paths must not leak"
        );
        assert!(
            !cargo_toml.contains("sand-resourcepack"),
            "no RP dep when resourcepack: false"
        );
    }

    #[test]
    fn path_deps_scaffold_emits_workspace_paths() {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("path_pack");
        std::fs::create_dir_all(project_dir.join("src/bin")).unwrap();

        let ctx = serde_json::json!({
            "name":                   "path_pack",
            "name_snake":             "path_pack",
            "namespace":              "path_pack",
            "description":            "Test",
            "mc_version":             "1.21.4",
            "pack_format":            61,
            "resource_pack_format":   46,
            "sand_path":              "/workspace/sand",
            "sand_build_path":        "/workspace/sand-build",
            "resourcepack":           false,
            "use_path_deps":          true,
            "sand_version":           "0.1.0",
            "sand_git_url":           "https://github.com/ThatOneToast/sand",
        });

        let hbs = build_handlebars();
        write_rendered(
            &hbs,
            "cargo_toml",
            CARGO_TOML_HBS,
            &ctx,
            &project_dir.join("Cargo.toml"),
        )
        .unwrap();

        let cargo_toml = std::fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
        assert!(
            cargo_toml.contains("path ="),
            "--path-deps scaffold must emit path deps"
        );
        assert!(
            cargo_toml.contains("sand = { path = \"/workspace/sand\""),
            "workspace sand path must appear"
        );
        assert!(
            cargo_toml.contains("/workspace/sand-build"),
            "workspace build path must appear"
        );
    }
}
