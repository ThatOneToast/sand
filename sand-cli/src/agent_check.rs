//! Sand-specific validation for automated authoring loops.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sand_api_contract::{ApiCatalog, CoverageStatus};
use serde::Serialize;
use walkdir::WalkDir;

use crate::api_cmd::typed_alternative_paths;
use crate::output::OutputFormat;
use crate::project_context::{CompatibilityStatus, ProjectContext};

pub const DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<String>,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Serialize)]
struct CheckOutput<'a> {
    schema_version: u32,
    success: bool,
    project_context: &'a ProjectContext,
    diagnostic_count: usize,
    diagnostics: &'a [Diagnostic],
}

pub fn run(catalog: &ApiCatalog, profile: &str, format: OutputFormat) -> Result<()> {
    let context = match ProjectContext::discover(catalog, profile) {
        Ok(context) => context,
        Err(error) if format.is_json() => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schema_version": DIAGNOSTIC_SCHEMA_VERSION,
                    "success": false,
                    "diagnostics": [{
                        "severity": "error",
                        "code": "SAND_CHECK_CONTEXT_FAILED",
                        "category": "configuration",
                        "message": format!("{error:#}"),
                        "source": "sand_validation",
                    }]
                }))?
            );
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    let diagnostics = validate(catalog, &context)?;
    let success = !diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    if format.is_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&CheckOutput {
                schema_version: DIAGNOSTIC_SCHEMA_VERSION,
                success,
                project_context: &context,
                diagnostic_count: diagnostics.len(),
                diagnostics: &diagnostics,
            })?
        );
    } else if diagnostics.is_empty() {
        println!("Sand agent validation passed.");
    } else {
        for diagnostic in &diagnostics {
            println!(
                "{:?} [{}]: {}",
                diagnostic.severity, diagnostic.code, diagnostic.message
            );
        }
    }
    if success {
        Ok(())
    } else {
        bail!("Sand agent validation failed")
    }
}

pub fn validate(catalog: &ApiCatalog, context: &ProjectContext) -> Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let Some(project) = context.project.as_ref() else {
        diagnostics.push(diagnostic(
            Severity::Error,
            "SAND_CHECK_NOT_PROJECT",
            "configuration",
            "No sand.toml was found in this directory or its ancestors.",
            None,
            None,
            Some("Run this command from a Sand project.".into()),
        ));
        return Ok(diagnostics);
    };
    if context.compatibility.status != CompatibilityStatus::Compatible {
        diagnostics.push(diagnostic(
            Severity::Error,
            "SAND_CHECK_API_MISMATCH",
            "api_compatibility",
            &context.compatibility.reasons.join("; "),
            None,
            Some(project),
            Some(
                "Use a Sand CLI built from the project's resolved Sand revision and profile."
                    .into(),
            ),
        ));
    }
    if catalog.coverage.status != CoverageStatus::Complete {
        diagnostics.push(diagnostic(
            Severity::Error,
            "SAND_CHECK_PARTIAL_CONTRACTS",
            "api_coverage",
            "The installed API catalog has partial contract coverage.",
            None,
            Some(project),
            Some("Use a CLI with complete API contract coverage.".into()),
        ));
    }
    if output_is_stale(&project.root, &project.namespace)? {
        diagnostics.push(diagnostic(
            Severity::Error,
            "SAND_CHECK_STALE_OUTPUT",
            "build_output",
            "Generated output is missing or older than project inputs.",
            Some(project.root.join("dist").join(&project.namespace)),
            Some(project),
            Some("Run `sand build` and repeat validation.".into()),
        ));
    }
    if context.compatibility.status == CompatibilityStatus::Compatible {
        diagnostics.extend(raw_escape_hatch_diagnostics(
            catalog,
            &project.root,
            project,
        )?);
    }
    diagnostics.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.code.cmp(&right.code))
    });
    Ok(diagnostics)
}

fn diagnostic(
    severity: Severity,
    code: &str,
    category: &str,
    message: &str,
    file: Option<PathBuf>,
    project: Option<&crate::project_context::ProjectIdentity>,
    suggested_action: Option<String>,
) -> Diagnostic {
    Diagnostic {
        severity,
        code: code.into(),
        category: category.into(),
        message: message.into(),
        file,
        line: None,
        column: None,
        api_path: None,
        minecraft_version: project.map(|project| project.minecraft_version.clone()),
        profile: project.map(|project| project.active_profile.clone()),
        suggested_action,
        source: "sand_validation".into(),
    }
}

fn output_is_stale(root: &Path, namespace: &str) -> Result<bool> {
    let manifest = root
        .join("dist")
        .join(namespace)
        .join(crate::build::output_manifest::MANIFEST_FILE_NAME);
    let Ok(output_time) = std::fs::metadata(&manifest).and_then(|metadata| metadata.modified())
    else {
        return Ok(true);
    };
    for entry in WalkDir::new(root.join("src"))
        .into_iter()
        .filter_entry(|entry| entry.file_name() != "target" && entry.file_name() != "dist")
    {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry
                .metadata()?
                .modified()
                .is_ok_and(|modified| modified > output_time)
        {
            return Ok(true);
        }
    }
    for input in ["Cargo.toml", "Cargo.lock", "sand.toml", "sand.build.rs"] {
        let path = root.join(input);
        if path.is_file()
            && std::fs::metadata(path)?
                .modified()
                .is_ok_and(|modified| modified > output_time)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn raw_escape_hatch_diagnostics(
    catalog: &ApiCatalog,
    root: &Path,
    project: &crate::project_context::ProjectIdentity,
) -> Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let src = root.join("src");
    if !src.is_dir() {
        return Ok(diagnostics);
    }
    for entry in WalkDir::new(src) {
        let entry = entry?;
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(entry.path())
            .with_context(|| format!("failed to read '{}'", entry.path().display()))?;
        for (index, line) in source.lines().enumerate() {
            if !(line.contains("cmd::raw(") || line.contains("command::raw(")) {
                continue;
            }
            let query = first_string_literal(line).unwrap_or("raw minecraft command");
            let alternatives = typed_alternative_paths(catalog, query, 3);
            if alternatives.is_empty() {
                continue;
            }
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "SAND_CHECK_AVOIDABLE_RAW_COMMAND".into(),
                category: "typed_api".into(),
                message: format!(
                    "A raw command may have a typed Sand alternative: {}",
                    alternatives.join(", ")
                ),
                file: Some(entry.path().to_path_buf()),
                line: Some(index + 1),
                column: line.find("raw(").map(|column| column + 1),
                api_path: alternatives.first().cloned(),
                minecraft_version: Some(project.minecraft_version.clone()),
                profile: Some(project.active_profile.clone()),
                suggested_action: Some(format!(
                    "Inspect {} before using the raw escape hatch.",
                    alternatives[0]
                )),
                source: "sand_validation".into(),
            });
        }
    }
    Ok(diagnostics)
}

fn first_string_literal(line: &str) -> Option<&str> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}
