mod config;
mod explain;
mod export;
pub mod output_manifest;
pub mod package;
pub mod records;
pub mod timing;
pub mod validate;
pub mod validate_output;
pub mod worldbuild;
pub mod write;

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use colored::Colorize;
use serde::Serialize;

use crate::config::SandConfig;
use crate::output::OutputFormat;
use crate::pack_format::pack_format_for;

use config::{cargo_target_dir, resolve_mc_version};
use explain::{RebuildExplanation, observe_exporter_rebuild};
use export::{ExportBuildPlan, Exporter, run_exporter};
use output_manifest::{ChangeSummary, OutputManifest};
use package::zip_dir;
use records::ComponentRecord;
use timing::{Phase, Timings};
use validate::validate_component_records_for_project;
use write::{component_output, pack_mcmeta_output};

/// Filename of the JSON manifest `sand build` writes alongside `dist/` (not
/// inside `dist/<namespace>/`) when a project's `sand.build.rs` configures a
/// `ServerConfig`. 🖥️ Server (host) only — never part of the datapack;
/// `sand run` reads this file to apply local dev-server settings.
pub const SERVER_CONFIG_FILE_NAME: &str = ".sand-server-config.json";
pub const BUILD_DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct BuildJsonOutput<'a> {
    schema_version: u32,
    success: bool,
    namespace: &'a str,
    minecraft_version: &'a str,
    profile: &'a str,
    component_count: usize,
    datapack_output: String,
    datapack_changes: ChangeSummary,
    diagnostics: Vec<BuildJsonDiagnostic>,
}

#[derive(Serialize)]
pub struct BuildJsonDiagnostic {
    severity: String,
    code: String,
    category: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minecraft_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_action: Option<String>,
    source: String,
}

pub fn error_json(error: &anyhow::Error, profile: &str) -> Result<String> {
    let message = format!("{error:#}");
    let mut diagnostics = cargo_diagnostics(&message, profile);
    let (code, category, source) = if message.contains("cargo build") {
        ("SAND_BUILD_CARGO", "cargo", "cargo_rustc")
    } else if message.contains("export") {
        ("SAND_BUILD_EXPORTER", "exporter", "exporter")
    } else if message.contains("Minecraft") || message.contains("version") {
        (
            "SAND_BUILD_PROFILE",
            "minecraft_profile",
            "minecraft_profile",
        )
    } else if message.contains("sand.toml") {
        ("SAND_BUILD_CONFIG", "configuration", "sand_validation")
    } else {
        ("SAND_BUILD_FAILED", "build", "sand_validation")
    };
    if diagnostics.is_empty() {
        diagnostics.push(BuildJsonDiagnostic {
            severity: "error".into(),
            code: code.into(),
            category: category.into(),
            message,
            file: None,
            line: None,
            column: None,
            api_path: None,
            minecraft_version: None,
            profile: Some(profile.to_owned()),
            suggested_action: None,
            source: source.into(),
        });
    }
    serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": BUILD_DIAGNOSTIC_SCHEMA_VERSION,
        "success": false,
        "diagnostics": diagnostics,
    }))
    .context("failed to serialize build diagnostic")
}

fn cargo_diagnostics(message: &str, profile: &str) -> Vec<BuildJsonDiagnostic> {
    let mut diagnostics = Vec::new();
    for line in message.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-message") {
            continue;
        }
        let Some(diagnostic) = value.get("message") else {
            continue;
        };
        let level = diagnostic
            .get("level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("error");
        if !matches!(level, "error" | "warning") {
            continue;
        }
        let primary = diagnostic
            .get("spans")
            .and_then(serde_json::Value::as_array)
            .and_then(|spans| {
                spans.iter().find(|span| {
                    span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true)
                })
            });
        let suggested_action = diagnostic
            .get("children")
            .and_then(serde_json::Value::as_array)
            .and_then(|children| {
                children.iter().find(|child| {
                    child.get("level").and_then(serde_json::Value::as_str) == Some("help")
                })
            })
            .and_then(|child| child.get("message"))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        diagnostics.push(BuildJsonDiagnostic {
            severity: level.into(),
            code: diagnostic
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("SAND_BUILD_RUSTC")
                .to_owned(),
            category: "cargo".into(),
            message: diagnostic
                .get("message")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Cargo compilation failed")
                .to_owned(),
            file: primary
                .and_then(|span| span.get("file_name"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned),
            line: primary
                .and_then(|span| span.get("line_start"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|line| usize::try_from(line).ok()),
            column: primary
                .and_then(|span| span.get("column_start"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|column| usize::try_from(column).ok()),
            api_path: None,
            minecraft_version: None,
            profile: Some(profile.to_owned()),
            suggested_action,
            source: "cargo_rustc".into(),
        });
    }
    diagnostics
}

pub fn run(release: bool) -> Result<()> {
    run_with_timings(release, false)
}

/// Same as [`run`], but prints a `Sand build timings` phase breakdown
/// (`sand build --timings`) when `print_timings` is set. Phase collection
/// itself always runs — only the printing is conditional — see
/// `timing.rs`.
pub fn run_with_timings(release: bool, print_timings: bool) -> Result<()> {
    run_with_options(BuildOptions {
        release,
        print_timings,
        explain_rebuild: false,
        profile: "dev".to_string(),
        output_format: OutputFormat::Human,
    })
}

/// Flags controlling one `sand build` invocation's diagnostics. `release`
/// controls packaging semantics; `print_timings` and
/// `explain_rebuild` are purely additive reporting (issue #347 Phases 0/8)
/// and never change what gets built or written. `profile` selects the
/// `BuildProfile` a project's optional `sand.build.rs` receives (issue
/// #317); defaults to `"dev"`.
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub release: bool,
    pub print_timings: bool,
    pub explain_rebuild: bool,
    pub profile: String,
    pub output_format: OutputFormat,
}

pub fn run_with_options(options: BuildOptions) -> Result<()> {
    let BuildOptions {
        release,
        print_timings,
        explain_rebuild,
        profile,
        output_format,
    } = options;
    let machine = output_format.is_json();
    let mut timings = Timings::new();

    // 1. Read sand.toml
    let (config, project_root) = timings.record(Phase::Configuration, || {
        let project_root = std::env::current_dir()?;
        let config_path = project_root.join("sand.toml");
        if !config_path.exists() {
            bail!("sand.toml not found — run `sand build` from your project root");
        }
        let config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
            .context("failed to parse sand.toml")?;
        Ok((config, project_root))
    })?;

    // Resolve mc_version ("latest" → bundled latest-known verified version)
    let configured_version =
        std::env::var("SAND_MC_VERSION").unwrap_or_else(|_| config.pack.mc_version.clone());
    let mc_version = resolve_mc_version(&configured_version);

    // Resolve pack format: explicit override in sand.toml wins; otherwise derive
    // from the version profile.  If the version is not in the known table the
    // profile is a conservative fallback and we warn the user.
    let (pack_format, format_is_fallback) = {
        use sand_core::version::{MinecraftVersion, VersionProfile};
        if let Some(explicit) = config.pack.pack_format {
            (explicit, false)
        } else if let Ok(v) = MinecraftVersion::parse(&mc_version) {
            let p = VersionProfile::resolve(&v).unwrap_or_else(|_| {
                VersionProfile::resolve(
                    &MinecraftVersion::parse(sand_core::version::LATEST_KNOWN).unwrap(),
                )
                .unwrap()
            });
            let meta = p.datapack_metadata();
            (meta.pack_format(), meta.is_fallback())
        } else {
            (pack_format_for(&mc_version)?, false)
        }
    };

    if format_is_fallback && !machine {
        eprintln!(
            "{} Minecraft version '{}' is not in Sand's known version table. \
             Using pack_format {} as a conservative fallback. \
             Add `pack_format = {}` to [pack] in sand.toml to silence this warning.",
            "warning:".yellow().bold(),
            mc_version,
            pack_format,
            pack_format
        );
    }

    if !machine {
        println!(
            "{} {} (Minecraft {}, pack_format {})...",
            "Building".cyan().bold(),
            config.pack.namespace.as_str().white().bold(),
            mc_version.yellow(),
            pack_format.to_string().yellow()
        );
    }

    // 2. Compile the datapack exporter.
    let target_dir = cargo_target_dir()?;
    let plan = ExportBuildPlan::new();
    let binaries = plan.binaries(&target_dir);
    let (_, exporter_outcome) = timings.record(Phase::ExporterCompile, || {
        observe_exporter_rebuild(&binaries.datapack, || plan.compile(&mc_version, machine))
    })?;

    // 3. Run the datapack export binary — pass the target mc_version via env
    //    var so the subprocess can gate components against the resolved
    //    VersionProfile.
    let stdout = timings.record(Phase::ExporterExecution, || {
        run_exporter(
            Exporter::Datapack,
            &binaries.datapack,
            &[("SAND_EXPORT_MC_VERSION", &mc_version)],
        )
    })?;

    // 4. Parse component records
    let records: Vec<ComponentRecord> = timings.record(Phase::RecordParsing, || {
        serde_json::from_slice(&stdout).map_err(|e| {
            let stdout = String::from_utf8_lossy(&stdout);
            let hint = if stdout.trim_start().starts_with('[') && stdout.matches('[').count() > 1 {
                "\n\nHint: the export binary printed more than one JSON value. \
                 __sand_export must print exactly one JSON array \
                 (from sand_core::export_components_json)."
            } else {
                ""
            };
            anyhow::anyhow!("failed to parse component export JSON: {}{}", e, hint)
        })
    })?;

    // 4b. Compile and run the optional typed world build before any output is
    // written. This keeps ordinary component output and world-build output in
    // one transaction: a broken sand.build.rs cannot leave a half-updated
    // datapack behind.
    let worldbuild_output = timings.record(Phase::WorldBuild, || {
        prepare_worldbuild(&project_root, &mc_version, &profile, machine)
    })?;

    // 5. Validate every record before creating the output directory.  A build
    // must fail before it produces a partially valid datapack.
    let dist = PathBuf::from("dist").join(config.pack.namespace.as_str());
    timings.record(Phase::Validation, || {
        validate_component_records_for_project(&dist, &project_root, &records)
    })?;

    // Prepare the complete output set before touching dist. World-build
    // resources participate in the same ownership manifest as component
    // resources, so switching profiles prunes files that disappeared.
    let outputs = prepare_datapack_outputs(
        &project_root,
        &config,
        pack_format,
        &records,
        worldbuild_output.as_ref(),
    )?;

    // 6-7. Write pack.mcmeta and every generated file through the output
    //    manifest (issue #347 Phase 7): unchanged content is left untouched
    //    (mtime included), changed content is rewritten atomically, and
    //    anything the previous build wrote that this build no longer
    //    produces is removed. See output_manifest.rs.
    let change_summary = timings.record(Phase::DatapackWriting, || {
        std::fs::create_dir_all(&dist)?;
        let mut manifest = OutputManifest::load(&dist);
        for (rel_path, bytes) in &outputs {
            manifest.write_if_changed(rel_path, bytes)?;
        }
        manifest.finish()
    })?;

    write_server_config(&dist, worldbuild_output.as_ref())?;

    if !machine {
        println!(
            "{} {} component(s) written to {} ({} written, {} unchanged, {} removed)",
            "Done!".green().bold(),
            records.len().to_string().white().bold(),
            format!("dist/{}/", config.pack.namespace.as_str())
                .white()
                .bold(),
            change_summary.written,
            change_summary.unchanged,
            change_summary.removed
        );
    }

    // 8. Zip if --release, otherwise hint how to install manually.
    timings.record(Phase::Packaging, || {
        if release {
            let zip_path = zip_dir(&dist, config.pack.namespace.as_str())?;
            if !machine {
                println!(
                    "  {} {}",
                    "zip:".dimmed(),
                    zip_path.display().to_string().white().bold()
                );
                println!(
                    "  {} drop {} into your world's datapacks/ folder",
                    "install:".dimmed(),
                    format!("dist/{}.zip", config.pack.namespace.as_str())
                        .white()
                        .bold()
                );
            }
        } else {
            if !machine {
                println!(
                    "  {} copy the {} folder into your world's datapacks/ folder, \
                 or run `sand build --release` to produce a zip",
                    "install:".dimmed(),
                    format!("dist/{}/", config.pack.namespace.as_str())
                        .white()
                        .bold()
                );
            }
        }
        Ok(())
    })?;

    if explain_rebuild && !machine {
        RebuildExplanation {
            exporter: exporter_outcome,
            datapack: change_summary,
        }
        .print();
    }

    if print_timings && !machine {
        timings.print();
    }

    if machine {
        let diagnostics = if format_is_fallback {
            vec![BuildJsonDiagnostic {
                severity: "warning".into(),
                code: "SAND_BUILD_PACK_FORMAT_FALLBACK".into(),
                category: "minecraft_profile".into(),
                message: format!(
                    "Minecraft version '{mc_version}' is not in Sand's known version table; using pack_format {pack_format}."
                ),
                file: Some(project_root.join("sand.toml").display().to_string()),
                line: None,
                column: None,
                api_path: None,
                minecraft_version: Some(mc_version.clone()),
                profile: Some(profile.clone()),
                suggested_action: Some(format!(
                    "Set pack_format = {pack_format} explicitly or use a supported Minecraft version."
                )),
                source: "minecraft_profile".into(),
            }]
        } else {
            Vec::new()
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&BuildJsonOutput {
                schema_version: BUILD_DIAGNOSTIC_SCHEMA_VERSION,
                success: true,
                namespace: config.pack.namespace.as_str(),
                minecraft_version: &mc_version,
                profile: &profile,
                component_count: records.len(),
                datapack_output: dist.display().to_string(),
                datapack_changes: change_summary,
                diagnostics,
            })?
        );
    }
    Ok(())
}

/// Compiles and runs a project's optional `sand.build.rs` (issue #317)
/// without writing output. The returned records are combined with ordinary
/// component output before the manifest transaction begins.
fn prepare_worldbuild(
    project_root: &Path,
    mc_version: &str,
    profile: &str,
    quiet: bool,
) -> Result<Option<worldbuild::WorldBuildOutput>> {
    if !worldbuild::project_has_worldbuild(project_root) {
        return Ok(None);
    }

    if !quiet {
        println!(
            "{} sand.build.rs (profile: {})...",
            "Building".cyan().bold(),
            profile.yellow()
        );
    }

    worldbuild::compile(project_root, mc_version, quiet)?;
    let target_dir = cargo_target_dir()?;
    let binary = worldbuild::binary_path(&target_dir);
    let output = worldbuild::run(&binary, profile, mc_version)?;
    Ok(Some(output))
}

fn prepare_datapack_outputs(
    project_root: &Path,
    config: &SandConfig,
    pack_format: u32,
    records: &[ComponentRecord],
    worldbuild: Option<&worldbuild::WorldBuildOutput>,
) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut outputs = BTreeMap::new();
    let (mcmeta_path, mcmeta_bytes) = pack_mcmeta_output(
        &config.pack.description,
        pack_format,
        config.pack.supported_formats,
        &config.pack.overlays,
    )?;
    outputs.insert(mcmeta_path, mcmeta_bytes);

    for record in records {
        let (rel_path, bytes) = component_output(project_root, record)?;
        if outputs.insert(rel_path.clone(), bytes).is_some() {
            bail!("multiple component records produce '{rel_path}'");
        }
    }

    if let Some(worldbuild) = worldbuild {
        for resource in &worldbuild.resources {
            let (rel_path, bytes) = world_resource_output(resource)?;
            if let Some(existing) = outputs.get_mut(&rel_path) {
                if resource.dir != "tags/function" {
                    bail!(
                        "sand.build.rs resource '{rel_path}' collides with ordinary component output"
                    );
                }
                let existing_text = std::str::from_utf8(existing)
                    .context("existing function-tag output is not UTF-8")?;
                *existing = merge_function_tag_json(existing_text, &resource.content)?.into_bytes();
            } else {
                outputs.insert(rel_path, bytes);
            }
        }
    }
    Ok(outputs)
}

fn world_resource_output(resource: &worldbuild::WorldResourceRecord) -> Result<(String, Vec<u8>)> {
    for (label, value) in [
        ("namespace", resource.namespace.as_str()),
        ("directory", resource.dir.as_str()),
        ("path", resource.path.as_str()),
        ("extension", resource.ext.as_str()),
    ] {
        let path = Path::new(value);
        if value.is_empty()
            || path.is_absolute()
            || path.components().any(|part| {
                matches!(
                    part,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            bail!("sand.build.rs resource has unsafe {label} '{value}'");
        }
    }
    Ok((
        format!(
            "data/{}/{}/{}.{}",
            resource.namespace, resource.dir, resource.path, resource.ext
        ),
        resource.content.as_bytes().to_vec(),
    ))
}

fn write_server_config(dist: &Path, output: Option<&worldbuild::WorldBuildOutput>) -> Result<()> {
    let server_config_path = dist
        .parent()
        .expect("dist/<namespace> always has a parent")
        .join(SERVER_CONFIG_FILE_NAME);
    let Some(output) = output else {
        if server_config_path.exists() {
            std::fs::remove_file(&server_config_path).with_context(|| {
                format!("failed to remove stale '{}'", server_config_path.display())
            })?;
        }
        return Ok(());
    };
    if output.server_config.is_some() || output.seed.is_some() || output.level_type.is_some() {
        let mut json = output
            .server_config
            .map(|server| {
                serde_json::json!({
                    "view_distance": server.view_distance,
                    "simulation_distance": server.simulation_distance,
                    "difficulty": server.difficulty.as_str(),
                    "online_mode": server.online_mode,
                    "world_reset_policy_always_reset": server.world_reset_policy,
                })
            })
            .unwrap_or_else(|| serde_json::json!({}));
        json["seed"] = serde_json::json!(output.seed);
        json["level_type"] = serde_json::json!(output.level_type);
        std::fs::write(&server_config_path, serde_json::to_string_pretty(&json)?)
            .with_context(|| format!("failed to write '{}'", server_config_path.display()))?;
    } else if server_config_path.exists() {
        std::fs::remove_file(&server_config_path).with_context(|| {
            format!("failed to remove stale '{}'", server_config_path.display())
        })?;
    }

    Ok(())
}

/// Merges a `sand_build_world`-produced function tag JSON's `"values"`
/// array into an existing tag file's `"values"` array (deduplicated,
/// preserving the existing file's entries first).
fn merge_function_tag_json(existing: &str, addition: &str) -> Result<String> {
    let mut existing_json: serde_json::Value =
        serde_json::from_str(existing).context("existing tag file is not valid JSON")?;
    let addition_json: serde_json::Value =
        serde_json::from_str(addition).context("world-build tag output is not valid JSON")?;

    let mut values: Vec<serde_json::Value> = existing_json
        .get("values")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if let Some(new_values) = addition_json.get("values").and_then(|v| v.as_array()) {
        for value in new_values {
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
    }
    existing_json["values"] = serde_json::Value::Array(values);
    Ok(serde_json::to_string_pretty(&existing_json)?)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::output_manifest::OutputManifest;
    use super::package::zip_dir;
    use super::records::{ComponentContentType, ComponentRecord, OutputExt};
    use super::validate::{
        component_output_path, validate_component_records, validate_component_records_for_project,
        validate_function_tag,
    };
    use super::worldbuild::{WorldBuildOutput, WorldResourceRecord};
    use super::write::{write_component, write_pack_mcmeta};
    use super::{error_json, prepare_datapack_outputs, world_resource_output, write_server_config};
    use sand_components::registry_coverage::{REGISTRY_COVERAGE, TAG_COVERAGE};

    /// Construct a valid ComponentRecord from parts via JSON deserialization.
    ///
    /// Uses "audit" as the namespace. Panics if the inputs are invalid (which
    /// makes test failures obvious at the point of construction).
    fn record(dir: &str, path: &str, ext: &str, content: &str) -> ComponentRecord {
        serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": dir,
            "path": path,
            "ext": ext,
            "content": content,
        }))
        .unwrap_or_else(|e| panic!("invalid test record ({dir}/{path}.{ext}): {e}"))
    }

    #[test]
    fn machine_build_errors_preserve_rustc_codes_and_primary_spans() {
        let cargo = serde_json::json!({
            "reason": "compiler-message",
            "message": {
                "level": "error",
                "message": "cannot find function `missing`",
                "code": { "code": "E0425" },
                "spans": [{
                    "file_name": "src/lib.rs",
                    "line_start": 7,
                    "column_start": 5,
                    "is_primary": true
                }],
                "children": [{ "level": "help", "message": "import the function" }]
            }
        });
        let error = anyhow::anyhow!("`cargo build` failed:\n{cargo}");
        let rendered: serde_json::Value =
            serde_json::from_str(&error_json(&error, "dev").unwrap()).unwrap();
        let diagnostic = &rendered["diagnostics"][0];
        assert_eq!(diagnostic["code"], "E0425");
        assert_eq!(diagnostic["file"], "src/lib.rs");
        assert_eq!(diagnostic["line"], 7);
        assert_eq!(diagnostic["column"], 5);
        assert_eq!(diagnostic["source"], "cargo_rustc");
        assert_eq!(diagnostic["suggested_action"], "import the function");
    }

    // ── sand.toml namespace validation at config parse time ───────────────────

    fn parse_config(namespace: &str) -> Result<crate::config::SandConfig, toml::de::Error> {
        let toml = format!(
            "[pack]\nnamespace = {namespace:?}\ndescription = \"test\"\nmc_version = \"26.1\"\n"
        );
        toml::from_str(&toml)
    }

    fn world_resource(dir: &str, path: &str, content: &str) -> WorldResourceRecord {
        WorldResourceRecord {
            namespace: "audit".to_string(),
            dir: dir.to_string(),
            path: path.to_string(),
            ext: "json".to_string(),
            content_type: "text".to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn worldbuild_outputs_share_manifest_ownership_and_prune_across_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let config = parse_config("audit").unwrap();
        let dev = WorldBuildOutput {
            resources: vec![world_resource("dimension", "dev_only", "{}")],
            server_config: None,
            seed: None,
            level_type: None,
        };
        let dev_outputs =
            prepare_datapack_outputs(dir.path(), &config, 61, &[], Some(&dev)).unwrap();
        let pack_root = dir.path().join("dist/audit");
        std::fs::create_dir_all(&pack_root).unwrap();
        let mut first = OutputManifest::load(&pack_root);
        for (path, bytes) in dev_outputs {
            first.write_if_changed(&path, &bytes).unwrap();
        }
        first.finish().unwrap();
        let stale = pack_root.join("data/audit/dimension/dev_only.json");
        assert!(stale.exists());

        let release_outputs = prepare_datapack_outputs(dir.path(), &config, 61, &[], None).unwrap();
        let mut second = OutputManifest::load(&pack_root);
        for (path, bytes) in release_outputs {
            second.write_if_changed(&path, &bytes).unwrap();
        }
        let summary = second.finish().unwrap();
        assert!(!stale.exists());
        assert_eq!(summary.removed, 1);
    }

    #[test]
    fn worldbuild_rejects_traversal_and_non_tag_collisions() {
        let unsafe_resource = world_resource("dimension", "../escape", "{}");
        assert!(world_resource_output(&unsafe_resource).is_err());

        let dir = tempfile::tempdir().unwrap();
        let config = parse_config("audit").unwrap();
        let ordinary = record("dimension", "same", "json", "{}");
        let world = WorldBuildOutput {
            resources: vec![world_resource("dimension", "same", "{}")],
            server_config: None,
            seed: None,
            level_type: None,
        };
        let error = prepare_datapack_outputs(dir.path(), &config, 61, &[ordinary], Some(&world))
            .unwrap_err();
        assert!(error.to_string().contains("collides"));
    }

    #[test]
    fn removing_worldbuild_removes_stale_server_config() {
        let dir = tempfile::tempdir().unwrap();
        let dist = dir.path().join("dist/audit");
        std::fs::create_dir_all(dist.parent().unwrap()).unwrap();
        let config_path = dir.path().join("dist/.sand-server-config.json");
        std::fs::write(&config_path, "{}").unwrap();
        write_server_config(&dist, None).unwrap();
        assert!(!config_path.exists());
    }

    #[test]
    fn valid_config_namespace_parses() {
        for ns in ["my_pack", "test-pack", "ns.v2", "a", "abc123"] {
            assert!(
                parse_config(ns).is_ok(),
                "namespace '{ns}' should be valid in sand.toml"
            );
        }
    }

    #[test]
    fn invalid_config_namespace_rejected_at_parse() {
        for ns in [
            "",
            "MyPack",
            "has space",
            "upper/slash",
            "UPPER",
            "../escape",
        ] {
            assert!(
                parse_config(ns).is_err(),
                "namespace '{ns}' should be rejected when parsing sand.toml"
            );
        }
    }

    // ── Record validation ─────────────────────────────────────────────────────

    #[test]
    fn validates_component_records_before_writing() {
        let dist = Path::new("dist/audit");
        assert!(
            validate_component_records(
                dist,
                &[record(
                    "recipe",
                    "valid",
                    "json",
                    "{\"type\":\"minecraft:crafting_shaped\"}"
                )]
            )
            .is_ok()
        );
        assert!(
            validate_component_records(dist, &[record("recipe", "invalid", "json", "{")]).is_err()
        );
        assert!(
            validate_component_records(
                dist,
                &[record("function", "null", "mcfunction", "say hi\0")]
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_duplicate_component_outputs() {
        let dist = Path::new("dist/audit");
        assert!(
            validate_component_records(
                dist,
                &[
                    record("recipe", "same", "json", "{}"),
                    record("recipe", "same", "json", "{}"),
                ]
            )
            .is_err()
        );
    }

    // ── Newtype boundary validation ───────────────────────────────────────────

    #[test]
    fn path_traversal_rejected_at_deserialization() {
        let bad: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "recipe",
            "path": "../escape",
            "ext": "json",
            "content": "{}",
        }));
        assert!(
            bad.is_err(),
            "path traversal must be rejected at deserialization"
        );

        let abs: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "recipe",
            "path": "/etc/passwd",
            "ext": "json",
            "content": "{}",
        }));
        assert!(
            abs.is_err(),
            "absolute path must be rejected at deserialization"
        );
    }

    #[test]
    fn invalid_namespace_rejected_at_deserialization() {
        for bad_ns in ["", "My_Pack", "has space", "upper/slash", "UPPER"] {
            let result: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
                "namespace": bad_ns,
                "dir": "function",
                "path": "load",
                "ext": "mcfunction",
                "content": "",
            }));
            assert!(
                result.is_err(),
                "namespace '{bad_ns}' must be rejected at deserialization"
            );
        }
    }

    #[test]
    fn unsupported_component_dir_rejected_at_deserialization() {
        for bad_dir in ["assets", "data", "META-INF", "../data", "unknown_dir"] {
            let result: Result<ComponentRecord, _> = serde_json::from_value(serde_json::json!({
                "namespace": "audit",
                "dir": bad_dir,
                "path": "test",
                "ext": "json",
                "content": "{}",
            }));
            assert!(
                result.is_err(),
                "dir '{bad_dir}' must be rejected at deserialization"
            );
        }
    }

    #[test]
    fn registry_coverage_component_dirs_are_supported() {
        for entry in REGISTRY_COVERAGE {
            let datapack_record = record(entry.datapack_dir, "sample", "json", "{}");
            assert_eq!(datapack_record.dir.as_str(), entry.datapack_dir);

            if let Some(tag_dir) = entry.tag_dir {
                let tag_record = record(tag_dir, "sample", "json", "{}");
                assert_eq!(tag_record.dir.as_str(), tag_dir);
            }
        }
        for entry in TAG_COVERAGE {
            let tag_record = record(entry.datapack_dir, "sample", "json", "{}");
            assert_eq!(tag_record.dir.as_str(), entry.datapack_dir);
        }
    }

    #[test]
    fn missing_registry_raw_json_component_passes_build_validation() {
        let dist = Path::new("dist/audit");
        let record = record(
            "enchantment_provider",
            "bonus_enchants",
            "json",
            r#"{"type":"minecraft:single_enchantment","enchantment":"minecraft:sharpness"}"#,
        );

        assert!(validate_component_records(dist, &[record]).is_ok());
    }

    // ── Pack metadata and zip ─────────────────────────────────────────────────

    #[test]
    fn pack_metadata_and_release_zip_stay_with_the_datapack_root() {
        let temp = tempfile::tempdir().unwrap();
        let datapack = temp.path().join("audit");
        std::fs::create_dir_all(datapack.join("data/audit/function")).unwrap();
        write_pack_mcmeta(&datapack, "audit", "data", 71, None, &[]).unwrap();
        std::fs::write(
            datapack.join("data/audit/function/load.mcfunction"),
            "say loaded",
        )
        .unwrap();

        let data_meta: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(datapack.join("pack.mcmeta")).unwrap())
                .unwrap();
        assert_eq!(data_meta["pack"]["pack_format"], 71);

        let zip_path = zip_dir(&datapack, "audit").unwrap();
        let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_path).unwrap()).unwrap();
        assert!(zip.by_name("pack.mcmeta").is_ok());
        assert!(zip.by_name("data/audit/function/load.mcfunction").is_ok());
    }

    #[test]
    fn modern_datapack_metadata_includes_required_format_bounds() {
        let temp = tempfile::tempdir().unwrap();
        write_pack_mcmeta(temp.path(), "audit", "modern", 107, None, &[]).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        assert_eq!(metadata["pack"]["pack_format"], 107);
        assert_eq!(metadata["pack"]["min_format"], 107);
        assert_eq!(metadata["pack"]["max_format"], 107);
    }

    // ── supported_formats / overlays (#149) ────────────────────────────────────

    #[test]
    fn minimal_pack_mcmeta_unchanged_without_compatibility_fields() {
        let temp = tempfile::tempdir().unwrap();
        write_pack_mcmeta(temp.path(), "audit", "minimal", 71, None, &[]).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            metadata,
            serde_json::json!({
                "pack": {
                    "pack_format": 71,
                    "description": "minimal",
                }
            }),
            "no supported_formats/overlays configured must keep the legacy minimal shape"
        );
    }

    #[test]
    fn single_supported_format_serializes_as_bare_integer() {
        use super::records::PackSupportedFormats;

        let temp = tempfile::tempdir().unwrap();
        write_pack_mcmeta(
            temp.path(),
            "audit",
            "single",
            71,
            Some(PackSupportedFormats::Single(71)),
            &[],
        )
        .unwrap();
        let metadata: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        assert_eq!(metadata["pack"]["supported_formats"], 71);
        assert!(metadata.get("overlays").is_none());
    }

    #[test]
    fn range_supported_format_serializes_as_min_max_object() {
        use super::records::PackSupportedFormats;

        let temp = tempfile::tempdir().unwrap();
        write_pack_mcmeta(
            temp.path(),
            "audit",
            "range",
            71,
            Some(PackSupportedFormats::Range { min: 71, max: 72 }),
            &[],
        )
        .unwrap();
        let metadata: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        assert_eq!(metadata["pack"]["supported_formats"]["min_inclusive"], 71);
        assert_eq!(metadata["pack"]["supported_formats"]["max_inclusive"], 72);
    }

    #[test]
    fn overlays_are_written_as_overlays_entries() {
        use super::records::{PackOverlay, PackSupportedFormats};

        let overlay: PackOverlay =
            toml::from_str("directory = \"overlays/26_2\"\nformats = { min = 72, max = 72 }\n")
                .unwrap();

        let temp = tempfile::tempdir().unwrap();
        write_pack_mcmeta(
            temp.path(),
            "audit",
            "with-overlay",
            71,
            Some(PackSupportedFormats::Range { min: 71, max: 72 }),
            &[overlay],
        )
        .unwrap();
        let metadata: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(temp.path().join("pack.mcmeta")).unwrap(),
        )
        .unwrap();
        let entries = metadata["overlays"]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["directory"], "overlays/26_2");
        assert_eq!(entries[0]["formats"]["min_inclusive"], 72);
        assert_eq!(entries[0]["formats"]["max_inclusive"], 72);
    }

    // ── sand.toml supported_formats / overlays parsing (#149) ──────────────────

    #[test]
    fn config_parses_single_supported_format() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\nsupported_formats = 71\n";
        let config: crate::config::SandConfig = toml::from_str(toml).unwrap();
        assert_eq!(
            config.pack.supported_formats,
            Some(super::records::PackSupportedFormats::Single(71))
        );
    }

    #[test]
    fn config_parses_supported_format_range() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\nsupported_formats = { min = 71, max = 72 }\n";
        let config: crate::config::SandConfig = toml::from_str(toml).unwrap();
        assert_eq!(
            config.pack.supported_formats,
            Some(super::records::PackSupportedFormats::Range { min: 71, max: 72 })
        );
    }

    #[test]
    fn config_rejects_inverted_supported_format_range() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\nsupported_formats = { min = 72, max = 71 }\n";
        let err = toml::from_str::<crate::config::SandConfig>(toml).unwrap_err();
        assert!(
            err.to_string().contains("min") && err.to_string().contains("max"),
            "error should name min/max: {err}"
        );
    }

    #[test]
    fn config_rejects_zero_supported_format() {
        for toml in [
            "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
             mc_version = \"26.1\"\nsupported_formats = 0\n",
            "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
             mc_version = \"26.1\"\nsupported_formats = { min = 0, max = 71 }\n",
        ] {
            assert!(
                toml::from_str::<crate::config::SandConfig>(toml).is_err(),
                "format number 0 must be rejected: {toml}"
            );
        }
    }

    #[test]
    fn config_parses_pack_overlays() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\nsupported_formats = { min = 71, max = 72 }\n\
                     [[pack.overlays]]\ndirectory = \"overlays/26_2\"\n\
                     formats = { min = 72, max = 72 }\n";
        let config: crate::config::SandConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.pack.overlays.len(), 1);
        assert_eq!(config.pack.overlays[0].directory.as_str(), "overlays/26_2");
        assert_eq!(
            config.pack.overlays[0].formats,
            super::records::PackSupportedFormats::Range { min: 72, max: 72 }
        );
    }

    #[test]
    fn config_rejects_absolute_overlay_directory() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\n\
                     [[pack.overlays]]\ndirectory = \"/etc/overlays\"\n\
                     formats = { min = 72, max = 72 }\n";
        assert!(toml::from_str::<crate::config::SandConfig>(toml).is_err());
    }

    #[test]
    fn config_rejects_path_traversing_overlay_directory() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\n\
                     mc_version = \"26.1\"\n\
                     [[pack.overlays]]\ndirectory = \"../escape\"\n\
                     formats = { min = 72, max = 72 }\n";
        assert!(toml::from_str::<crate::config::SandConfig>(toml).is_err());
    }

    #[test]
    fn config_without_compatibility_fields_defaults_to_backward_compatible_shape() {
        let toml = "[pack]\nnamespace = \"audit\"\ndescription = \"test\"\nmc_version = \"26.1\"\n";
        let config: crate::config::SandConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.pack.supported_formats, None);
        assert!(config.pack.overlays.is_empty());
    }

    // ── Component output path computation ─────────────────────────────────────

    #[test]
    fn locks_modern_singular_datapack_component_paths() {
        let dist = Path::new("dist/audit");
        let cases = [
            (
                "function",
                "load",
                "mcfunction",
                "data/audit/function/load.mcfunction",
            ),
            (
                "tags/function",
                "load",
                "json",
                "data/audit/tags/function/load.json",
            ),
            (
                "advancement",
                "test",
                "json",
                "data/audit/advancement/test.json",
            ),
            ("recipe", "test", "json", "data/audit/recipe/test.json"),
            (
                "predicate",
                "test",
                "json",
                "data/audit/predicate/test.json",
            ),
            (
                "loot_table",
                "test",
                "json",
                "data/audit/loot_table/test.json",
            ),
            (
                "item_modifier",
                "test",
                "json",
                "data/audit/item_modifier/test.json",
            ),
            (
                "damage_type",
                "test",
                "json",
                "data/audit/damage_type/test.json",
            ),
            (
                "enchantment",
                "test",
                "json",
                "data/audit/enchantment/test.json",
            ),
            (
                "banner_pattern",
                "test",
                "json",
                "data/audit/banner_pattern/test.json",
            ),
            (
                "painting_variant",
                "test",
                "json",
                "data/audit/painting_variant/test.json",
            ),
            (
                "trim_material",
                "test",
                "json",
                "data/audit/trim_material/test.json",
            ),
            (
                "trim_pattern",
                "test",
                "json",
                "data/audit/trim_pattern/test.json",
            ),
            (
                "chat_type",
                "test",
                "json",
                "data/audit/chat_type/test.json",
            ),
            (
                "wolf_variant",
                "test",
                "json",
                "data/audit/wolf_variant/test.json",
            ),
            (
                "jukebox_song",
                "test",
                "json",
                "data/audit/jukebox_song/test.json",
            ),
            (
                "worldgen/biome",
                "test",
                "json",
                "data/audit/worldgen/biome/test.json",
            ),
            (
                "worldgen/noise_settings",
                "test",
                "json",
                "data/audit/worldgen/noise_settings/test.json",
            ),
            (
                "worldgen/placed_feature",
                "test",
                "json",
                "data/audit/worldgen/placed_feature/test.json",
            ),
        ];
        for (dir, path, ext, expected) in cases {
            let output = component_output_path(dist, &record(dir, path, ext, "{}")).unwrap();
            let actual = output
                .strip_prefix(dist)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            assert_eq!(actual, expected, "wrong directory for {dir}");
        }

        let minecraft_tag: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags/function",
            "path": "tick",
            "ext": "json",
            "content": "{}",
        }))
        .unwrap();
        assert_eq!(
            component_output_path(dist, &minecraft_tag)
                .unwrap()
                .strip_prefix(dist)
                .unwrap(),
            PathBuf::from("data/minecraft/tags/function/tick.json")
        );
    }

    // ── OutputExt / ContentType deserialization ───────────────────────────────

    #[test]
    fn typed_output_ext_deserializes_from_json() {
        let json = r#"{"namespace":"ns","dir":"function","path":"load","ext":"mcfunction","content":"say hi"}"#;
        let rec: ComponentRecord = serde_json::from_str(json).unwrap();
        assert_eq!(rec.ext, OutputExt::Mcfunction);
        assert_eq!(rec.content_type, ComponentContentType::Text);

        let json2 =
            r#"{"namespace":"ns","dir":"recipe","path":"test","ext":"json","content":"{}"}"#;
        let rec2: ComponentRecord = serde_json::from_str(json2).unwrap();
        assert_eq!(rec2.ext, OutputExt::Json);

        let json3 = r#"{"namespace":"ns","dir":"structure","path":"rooms/start","ext":"nbt","content_type":"copy","content":"structures/start.nbt"}"#;
        let rec3: ComponentRecord = serde_json::from_str(json3).unwrap();
        assert_eq!(rec3.ext, OutputExt::Nbt);
        assert_eq!(rec3.content_type, ComponentContentType::Copy);
    }

    #[test]
    fn unknown_ext_rejected_at_deserialize() {
        let json = r#"{"namespace":"ns","dir":"function","path":"load","ext":"lua","content":""}"#;
        assert!(serde_json::from_str::<ComponentRecord>(json).is_err());
    }

    #[test]
    fn validates_structure_template_copy_records() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist/audit");
        let src = project_root.join("src/structures/start.nbt");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, [0x0a, 0x00, 0x00]).unwrap();

        let good: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/start.nbt",
        }))
        .unwrap();
        assert!(validate_component_records_for_project(&dist, &project_root, &[good]).is_ok());

        let unsafe_source: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "../start.nbt",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[unsafe_source]).is_err()
        );

        let missing_source: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/missing",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/missing.nbt",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[missing_source])
                .is_err()
        );
        assert!(
            !dist.exists(),
            "copy-backed structure preflight must not create output"
        );

        let wrong_ext: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "json",
            "content": "{}",
        }))
        .unwrap();
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[wrong_ext]).is_err(),
            "structure outputs must use .nbt"
        );

        let text_nbt: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content": "not binary content",
        }))
        .unwrap();
        assert!(validate_component_records_for_project(&dist, &project_root, &[text_nbt]).is_err());
    }

    /// The generated bytes depend only on the records, not on how many times or
    /// in what order the write phase runs.
    #[test]
    fn generated_output_is_byte_stable_across_repeated_writes() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        std::fs::create_dir_all(&project_root).unwrap();

        let records: Vec<ComponentRecord> = serde_json::from_value(serde_json::json!([
            {
                "namespace": "audit",
                "dir": "function",
                "path": "load",
                "ext": "mcfunction",
                "content": "say loaded\r\nsay again",
            },
            {
                "namespace": "minecraft",
                "dir": "tags/function",
                "path": "load",
                "ext": "json",
                "content": "{\"values\":[\"audit:load\"]}",
            },
        ]))
        .unwrap();

        let render = |dist: &Path| {
            std::fs::create_dir_all(dist).unwrap();
            write_pack_mcmeta(dist, "audit", "byte-stable", 71, None, &[]).unwrap();
            for record in &records {
                write_component(dist, &project_root, record).unwrap();
            }
            let mut files: Vec<(String, Vec<u8>)> = Vec::new();
            let mut stack = vec![dist.to_path_buf()];
            while let Some(dir) = stack.pop() {
                for entry in std::fs::read_dir(&dir).unwrap() {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        stack.push(path);
                    } else {
                        let rel = path
                            .strip_prefix(dist)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/");
                        files.push((rel, std::fs::read(&path).unwrap()));
                    }
                }
            }
            files.sort();
            files
        };

        let first = render(&temp.path().join("out-a"));
        let second = render(&temp.path().join("out-b"));
        assert_eq!(
            first, second,
            "generated datapack output must be byte-stable"
        );
        assert!(
            first.iter().any(
                |(name, bytes)| name == "data/audit/function/load.mcfunction"
                    && bytes == b"say loaded\nsay again"
            ),
            "function output must keep its normalized LF content: {first:?}"
        );
    }

    // ── Structure copy-source containment (#158) ──────────────────────────────

    /// Build a structure copy record whose source is `content`.
    fn structure_record(path: &str, content: &str) -> ComponentRecord {
        serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": path,
            "ext": "nbt",
            "content_type": "copy",
            "content": content,
        }))
        .unwrap_or_else(|e| panic!("invalid structure test record ({path}): {e}"))
    }

    #[test]
    fn structure_source_lexical_escapes_stay_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path();
        let dist = temp.path().join("dist/audit");

        for bad_source in [
            "",
            "../escape.nbt",
            "src/../../escape.nbt",
            "/tmp/escape.nbt",
            "src\0bad.nbt",
            "src/structures/start.txt",
        ] {
            let record = structure_record("rooms/start", bad_source);
            let err =
                validate_component_records_for_project(&dist, project_root, &[record]).unwrap_err();
            assert!(
                err.to_string()
                    .contains("unsafe structure template source path"),
                "source '{bad_source}' should be rejected lexically: {err}"
            );
        }
        assert!(!dist.exists(), "validation must not create output");
    }

    #[test]
    fn structure_source_missing_and_directory_sources_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path();
        let dist = temp.path().join("dist/audit");

        let missing = structure_record("rooms/missing", "src/structures/missing.nbt");
        let err =
            validate_component_records_for_project(&dist, project_root, &[missing]).unwrap_err();
        let rendered = format!("{err:#}");
        assert!(
            rendered.contains("datapack structure asset not found or unreadable")
                && rendered.contains("src/structures/missing.nbt"),
            "missing source needs an actionable diagnostic naming the path: {rendered}"
        );

        // A *directory* whose name ends in .nbt passes every lexical rule.
        std::fs::create_dir_all(project_root.join("src/structures/dir.nbt")).unwrap();
        let directory = structure_record("rooms/dir", "src/structures/dir.nbt");
        let err =
            validate_component_records_for_project(&dist, project_root, &[directory]).unwrap_err();
        assert!(
            err.to_string()
                .contains("datapack structure asset is not a file"),
            "directory source should be rejected before writing: {err}"
        );

        assert!(!dist.exists(), "validation must not create output");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_structure_source_symlink_escape_before_writing() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let outside_root = temp.path().join("outside");
        let dist = temp.path().join("dist/audit");
        std::fs::create_dir_all(project_root.join("src/structures")).unwrap();
        std::fs::create_dir_all(&outside_root).unwrap();
        std::fs::write(outside_root.join("leak.nbt"), [0x0a, 0x00, 0x00]).unwrap();

        // Skip only the symlink assertion where symlink creation is unavailable.
        let link_path = project_root.join("src/structures/leak.nbt");
        if std::os::unix::fs::symlink(outside_root.join("leak.nbt"), &link_path).is_err() {
            eprintln!("skipping: symlink creation not permitted in this environment");
            return;
        }

        let record = structure_record("rooms/leak", "src/structures/leak.nbt");
        let err =
            validate_component_records_for_project(&dist, &project_root, &[record]).unwrap_err();
        assert!(
            err.to_string()
                .contains("datapack structure asset escapes the project root"),
            "symlinked structure source escapes should be rejected: {err}"
        );
        assert!(
            err.to_string().contains("leak.nbt"),
            "diagnostic must identify the unsafe source: {err}"
        );
        assert!(
            !dist.exists(),
            "structure preflight must fail before output is created"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_structure_source_through_symlinked_directory() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let outside_root = temp.path().join("outside");
        let dist = temp.path().join("dist/audit");
        std::fs::create_dir_all(project_root.join("src")).unwrap();
        std::fs::create_dir_all(&outside_root).unwrap();
        std::fs::write(outside_root.join("start.nbt"), [0x0a, 0x00, 0x00]).unwrap();

        // src/structures -> ../outside : the *directory* escapes, not the file.
        let link_path = project_root.join("src/structures");
        if std::os::unix::fs::symlink(&outside_root, &link_path).is_err() {
            eprintln!("skipping: symlink creation not permitted in this environment");
            return;
        }

        let record = structure_record("rooms/start", "src/structures/start.nbt");
        let err =
            validate_component_records_for_project(&dist, &project_root, &[record]).unwrap_err();
        assert!(
            err.to_string()
                .contains("datapack structure asset escapes the project root"),
            "intermediate directory symlink escapes should be rejected: {err}"
        );
        assert!(!dist.exists(), "validation must not create output");
    }

    #[cfg(unix)]
    #[test]
    fn accepts_structure_source_symlink_that_stays_inside_the_project() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist/audit");
        std::fs::create_dir_all(project_root.join("assets")).unwrap();
        std::fs::create_dir_all(project_root.join("src/structures")).unwrap();
        std::fs::write(project_root.join("assets/start.nbt"), [0x0a, 0x00, 0x00]).unwrap();

        let link_path = project_root.join("src/structures/start.nbt");
        if std::os::unix::fs::symlink(project_root.join("assets/start.nbt"), &link_path).is_err() {
            eprintln!("skipping: symlink creation not permitted in this environment");
            return;
        }

        let record = structure_record("rooms/start", "src/structures/start.nbt");
        assert!(
            validate_component_records_for_project(&dist, &project_root, &[record]).is_ok(),
            "a symlink resolving inside the project root is allowed"
        );
    }

    /// A sibling directory sharing a name prefix must not count as "inside":
    /// `/tmp/x/project-other` is not under `/tmp/x/project`.
    #[cfg(unix)]
    #[test]
    fn structure_source_containment_is_component_wise() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let sibling = temp.path().join("project-other");
        let dist = temp.path().join("dist/audit");
        std::fs::create_dir_all(project_root.join("src")).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();
        std::fs::write(sibling.join("start.nbt"), [0x0a, 0x00, 0x00]).unwrap();

        // The only way to reach the prefix-sharing sibling without a lexical
        // `..` is through a symlink, so this exercises the real containment
        // check at the string-prefix boundary.
        let link_path = project_root.join("src/structures");
        if std::os::unix::fs::symlink(&sibling, &link_path).is_err() {
            eprintln!("skipping: symlink creation not permitted in this environment");
            return;
        }

        let record = structure_record("rooms/start", "src/structures/start.nbt");
        let err =
            validate_component_records_for_project(&dist, &project_root, &[record]).unwrap_err();
        assert!(
            err.to_string()
                .contains("datapack structure asset escapes the project root"),
            "'project-other' must not be treated as inside 'project': {err}"
        );
    }

    /// A symlink whose target is gone is reported as a broken link, not as a
    /// missing file the author can plainly see in their tree.
    #[cfg(unix)]
    #[test]
    fn dangling_structure_source_symlink_is_named_as_a_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist/audit");
        std::fs::create_dir_all(project_root.join("src/structures")).unwrap();

        let link_path = project_root.join("src/structures/start.nbt");
        if std::os::unix::fs::symlink(temp.path().join("gone.nbt"), &link_path).is_err() {
            eprintln!("skipping: symlink creation not permitted in this environment");
            return;
        }

        let record = structure_record("rooms/start", "src/structures/start.nbt");
        let err =
            validate_component_records_for_project(&dist, &project_root, &[record]).unwrap_err();
        let rendered = format!("{err:#}");
        assert!(
            rendered.contains("is a symlink whose target is missing or unreadable"),
            "broken links need their own diagnostic: {rendered}"
        );
        assert!(!dist.exists(), "validation must not create output");
    }

    #[test]
    fn dialog_tag_records_use_generic_tags_dir_and_dialog_path() {
        let record: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags",
            "path": "dialog/quick_actions",
            "ext": "json",
            "content_type": "text",
            "content": r#"{"replace":false,"values":["example:welcome"]}"#,
        }))
        .unwrap();

        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("dist/example");
        assert!(validate_component_records_for_project(&dist, temp.path(), &[record]).is_ok());
    }

    #[test]
    fn writes_and_zips_structure_template_assets() {
        let temp = tempfile::tempdir().unwrap();
        let project_root = temp.path().join("project");
        let dist = temp.path().join("dist").join("audit");
        let src = project_root.join("src/structures/start.nbt");
        std::fs::create_dir_all(src.parent().unwrap()).unwrap();
        std::fs::write(&src, [0x0a, 0x00, 0x00]).unwrap();

        let record: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "audit",
            "dir": "structure",
            "path": "rooms/start",
            "ext": "nbt",
            "content_type": "copy",
            "content": "src/structures/start.nbt",
        }))
        .unwrap();

        validate_component_records_for_project(&dist, &project_root, std::slice::from_ref(&record))
            .unwrap();
        write_component(&dist, &project_root, &record).unwrap();

        let output = dist.join("data/audit/structure/rooms/start.nbt");
        assert_eq!(std::fs::read(&output).unwrap(), [0x0a, 0x00, 0x00]);

        let zip_path = zip_dir(&dist, "audit").unwrap();
        let zip_file = std::fs::File::open(zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        let mut file = archive
            .by_name("data/audit/structure/rooms/start.nbt")
            .unwrap();
        let mut bytes = Vec::new();
        use std::io::Read as _;
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, [0x0a, 0x00, 0x00]);
    }

    // ── Function tag validation ───────────────────────────────────────────────

    #[test]
    fn function_tag_accepts_valid_load_tick_tags() {
        // Typical load tag
        let load = r#"{"values":["my_pack:load"]}"#;
        assert!(validate_function_tag("load", load).is_ok());

        // Typical tick tag with multiple entries
        let tick = r#"{"values":["my_pack:tick","other_pack:tick"]}"#;
        assert!(validate_function_tag("tick", tick).is_ok());

        // Empty values array is valid (no functions registered)
        let empty = r#"{"values":[]}"#;
        assert!(validate_function_tag("load", empty).is_ok());

        // Tag reference (#-prefixed) with valid resource location
        let tag_ref = "{\"values\":[\"#minecraft:some_tag\"]}";
        assert!(validate_function_tag("load", tag_ref).is_ok());

        // Object form with valid resource location and required=false
        let optional = r#"{"values":[{"id":"my_pack:optional","required":false}]}"#;
        assert!(validate_function_tag("load", optional).is_ok());

        // Object form with id only (required is optional)
        let id_only = r#"{"values":[{"id":"my_pack:fn"}]}"#;
        assert!(validate_function_tag("load", id_only).is_ok());

        // Paths with subdirectories are valid
        let subdir = r#"{"values":["my_pack:subfolder/load"]}"#;
        assert!(validate_function_tag("load", subdir).is_ok());
    }

    #[test]
    fn function_tag_rejects_invalid_structures() {
        // Not an object
        assert!(validate_function_tag("load", r#"[]"#).is_err());

        // Missing values key
        assert!(validate_function_tag("load", r#"{}"#).is_err());

        // values is not an array
        assert!(validate_function_tag("load", r#"{"values":"my_pack:load"}"#).is_err());

        // String entry missing ':' entirely
        assert!(validate_function_tag("load", r#"{"values":["no_colon_here"]}"#).is_err());

        // Uppercase namespace is rejected
        assert!(validate_function_tag("load", r#"{"values":["Bad:load"]}"#).is_err());

        // Empty namespace
        assert!(validate_function_tag("load", r#"{"values":[":load"]}"#).is_err());

        // Empty path
        assert!(validate_function_tag("load", r#"{"values":["minecraft:"]}"#).is_err());

        // Object entry missing 'id'
        assert!(validate_function_tag("load", r#"{"values":[{"required":false}]}"#).is_err());

        // Object id is not a string
        assert!(validate_function_tag("load", r#"{"values":[{"id":42}]}"#).is_err());

        // Object id is not a valid resource location
        assert!(validate_function_tag("load", r#"{"values":[{"id":"not_a_location"}]}"#).is_err());

        // Object id with uppercase namespace
        assert!(validate_function_tag("load", r#"{"values":[{"id":"Bad:load"}]}"#).is_err());

        // required is not a boolean
        assert!(
            validate_function_tag(
                "load",
                r#"{"values":[{"id":"my_pack:fn","required":"yes"}]}"#
            )
            .is_err()
        );

        // Invalid JSON
        assert!(validate_function_tag("load", r#"{"values": ["#).is_err());
    }

    #[test]
    fn function_tag_validation_applies_to_generic_tags_dir() {
        // A record with dir="tags" and path="function/load" should also be
        // validated as a function tag by validate_component_records.
        let dist = std::path::Path::new("dist/audit");

        // Valid function tag via the generic dir="tags" form
        let good: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags",
            "path": "function/load",
            "ext": "json",
            "content": r#"{"values":["my_pack:load"]}"#,
        }))
        .unwrap();
        assert!(validate_component_records(dist, &[good]).is_ok());

        // Malformed function tag via the generic dir="tags" form should fail
        let bad: ComponentRecord = serde_json::from_value(serde_json::json!({
            "namespace": "minecraft",
            "dir": "tags",
            "path": "function/load",
            "ext": "json",
            "content": r#"{"values":["BadNamespace:load"]}"#,
        }))
        .unwrap();
        assert!(
            validate_component_records(dist, &[bad]).is_err(),
            "invalid resource location in tags dir+function/ path must be caught"
        );
    }

    // ── Golden fixture ────────────────────────────────────────────────────────

    /// End-to-end fixture: given a minimal set of records (functions + tags),
    /// the build pipeline writes the expected files with the expected content.
    #[test]
    fn golden_fixture_minimal_pack() {
        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("golden");

        let tick_tag_json = r#"{"values":["golden:tick"]}"#;
        let load_tag_json = r#"{"values":["golden:load"]}"#;

        let records: Vec<ComponentRecord> = serde_json::from_value(serde_json::json!([
            {
                "namespace": "golden",
                "dir": "function",
                "path": "load",
                "ext": "mcfunction",
                "content": "say loaded",
            },
            {
                "namespace": "golden",
                "dir": "function",
                "path": "tick",
                "ext": "mcfunction",
                "content": "say tick",
            },
            {
                "namespace": "minecraft",
                "dir": "tags/function",
                "path": "load",
                "ext": "json",
                "content": load_tag_json,
            },
            {
                "namespace": "minecraft",
                "dir": "tags/function",
                "path": "tick",
                "ext": "json",
                "content": tick_tag_json,
            },
        ]))
        .unwrap();

        // Validate before writing
        validate_component_records(&dist, &records).unwrap();

        // Validate load/tick tag structure explicitly
        validate_function_tag("load", load_tag_json).unwrap();
        validate_function_tag("tick", tick_tag_json).unwrap();

        // Write the pack
        std::fs::create_dir_all(&dist).unwrap();
        write_pack_mcmeta(&dist, "golden", "Golden fixture pack", 71, None, &[]).unwrap();
        for r in &records {
            write_component(&dist, temp.path(), r).unwrap();
        }

        // Verify pack.mcmeta
        let mcmeta: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dist.join("pack.mcmeta")).unwrap())
                .unwrap();
        assert_eq!(mcmeta["pack"]["pack_format"], 71);
        assert_eq!(mcmeta["pack"]["description"], "Golden fixture pack");

        // Verify functions
        assert_eq!(
            std::fs::read_to_string(dist.join("data/golden/function/load.mcfunction")).unwrap(),
            "say loaded"
        );
        assert_eq!(
            std::fs::read_to_string(dist.join("data/golden/function/tick.mcfunction")).unwrap(),
            "say tick"
        );

        // Verify function tags
        let load_tag: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dist.join("data/minecraft/tags/function/load.json")).unwrap(),
        )
        .unwrap();
        assert!(
            load_tag["values"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "golden:load"),
            "load tag must reference golden:load"
        );

        let tick_tag: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dist.join("data/minecraft/tags/function/tick.json")).unwrap(),
        )
        .unwrap();
        assert!(
            tick_tag["values"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "golden:tick"),
            "tick tag must reference golden:tick"
        );
    }

    /// CLI-workflow fixture: a `sand.toml` configuring `supported_formats`
    /// and `overlays` under `[pack]` parses successfully and drives the same
    /// `write_pack_mcmeta` call `sand build` uses (step 6 of [`run`]),
    /// producing the canonical `pack.supported_formats` / `overlays.entries`
    /// shape in the generated `pack.mcmeta`.
    #[test]
    fn golden_fixture_multi_version_pack_toml_drives_expected_mcmeta() {
        let toml = "[pack]\n\
                     namespace = \"golden\"\n\
                     description = \"Golden multi-version pack\"\n\
                     mc_version = \"26.1\"\n\
                     pack_format = 71\n\
                     supported_formats = { min = 71, max = 72 }\n\
                     \n\
                     [[pack.overlays]]\n\
                     directory = \"overlays/26_2\"\n\
                     formats = { min = 72, max = 72 }\n";
        let config: crate::config::SandConfig = toml::from_str(toml).unwrap();

        let temp = tempfile::tempdir().unwrap();
        let dist = temp.path().join("golden");
        std::fs::create_dir_all(&dist).unwrap();

        write_pack_mcmeta(
            &dist,
            config.pack.namespace.as_str(),
            &config.pack.description,
            config.pack.pack_format.unwrap(),
            config.pack.supported_formats,
            &config.pack.overlays,
        )
        .unwrap();

        let mcmeta: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dist.join("pack.mcmeta")).unwrap())
                .unwrap();
        assert_eq!(mcmeta["pack"]["pack_format"], 71);
        assert_eq!(mcmeta["pack"]["supported_formats"]["min_inclusive"], 71);
        assert_eq!(mcmeta["pack"]["supported_formats"]["max_inclusive"], 72);
        let entries = mcmeta["overlays"]["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["directory"], "overlays/26_2");
        assert_eq!(entries[0]["formats"]["min_inclusive"], 72);
        assert_eq!(entries[0]["formats"]["max_inclusive"], 72);

        // Output-validation hook still accepts the generated pack.
        std::fs::create_dir_all(dist.join("data/golden/function")).unwrap();
        std::fs::write(
            dist.join("data/golden/function/load.mcfunction"),
            "say loaded",
        )
        .unwrap();
        assert!(super::validate_output::validate_output_dir(&dist).is_ok());
    }
}
