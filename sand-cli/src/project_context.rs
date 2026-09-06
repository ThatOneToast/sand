//! Local, deterministic identity of the Sand project containing the current directory.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use sand_api_contract::{ApiCatalog, CoverageStatus};
use serde::{Deserialize, Serialize};

use crate::config::SandConfig;

pub const CONTEXT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectContext {
    pub schema_version: u32,
    pub is_sand_project: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectIdentity>,
    pub cli: CliIdentity,
    pub api_catalog: CatalogIdentity,
    pub compatibility: Compatibility,
    pub warnings: Vec<ContextWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectIdentity {
    pub root: PathBuf,
    pub namespace: String,
    pub minecraft_version: String,
    pub sand_dependency: SandDependency,
    pub sand_features: Vec<String>,
    pub active_profile: String,
    pub resource_pack_enabled: bool,
    pub build_script_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SandDependency {
    pub version: String,
    pub source_kind: DependencySourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_git_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencySourceKind {
    Git,
    Path,
    Registry,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CliIdentity {
    pub sand_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_revision: Option<String>,
    pub workspace_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogIdentity {
    pub schema_version: u32,
    pub minecraft_version: String,
    pub surface_profile: String,
    pub cargo_features: Vec<String>,
    pub placeholder_codegen: bool,
    pub coverage_status: CoverageStatus,
    pub static_surface_items: usize,
    pub compiled_surface_items: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Compatibility {
    pub status: CompatibilityStatus,
    pub compatible: Option<bool>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Compatible,
    Incompatible,
    Unverified,
    NotAProject,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ContextWarning {
    pub code: String,
    pub message: String,
}

impl ProjectContext {
    pub fn discover(catalog: &ApiCatalog, active_profile: &str) -> Result<Self> {
        Self::discover_from(catalog, active_profile, &std::env::current_dir()?)
    }

    pub fn discover_from(catalog: &ApiCatalog, active_profile: &str, cwd: &Path) -> Result<Self> {
        let cli = cli_identity(catalog);
        let api_catalog = catalog_identity(catalog);
        let Some(root) = find_project_root(cwd) else {
            return Ok(Self {
                schema_version: CONTEXT_SCHEMA_VERSION,
                is_sand_project: false,
                project: None,
                cli,
                api_catalog,
                compatibility: Compatibility {
                    status: CompatibilityStatus::NotAProject,
                    compatible: None,
                    reasons: vec!["no sand.toml was found in this directory or its ancestors".into()],
                },
                warnings: vec![ContextWarning {
                    code: "SAND_CONTEXT_NOT_PROJECT".into(),
                    message: "No Sand project was found; the API catalog describes the installed CLI only."
                        .into(),
                }],
            });
        };

        let config_path = root.join("sand.toml");
        let config: SandConfig = toml::from_str(
            &std::fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read '{}'", config_path.display()))?,
        )
        .with_context(|| format!("failed to parse '{}'", config_path.display()))?;
        let (dependency, features) = match cargo_metadata(&root)
            .and_then(|metadata| metadata.sand_dependency(&root))
        {
            Ok(identity) => identity,
            Err(metadata_error) => manifest_sand_dependency(&root).with_context(|| {
                format!(
                    "locked, offline Cargo dependency resolution failed first: {metadata_error:#}"
                )
            })?,
        };
        let minecraft_version = if config.pack.mc_version == "latest" {
            sand_build::latest_release_version()
        } else {
            config.pack.mc_version.clone()
        };
        let project = ProjectIdentity {
            root: root.canonicalize().unwrap_or(root),
            namespace: config.pack.namespace.as_str().to_owned(),
            minecraft_version,
            sand_dependency: dependency,
            sand_features: features,
            active_profile: active_profile.to_owned(),
            resource_pack_enabled: config.resourcepack.is_some(),
            build_script_present: project_build_script_present(&config_path),
        };
        let compatibility = compare(&project, &cli, &api_catalog);
        let mut warnings = Vec::new();
        if compatibility.status != CompatibilityStatus::Compatible {
            warnings.push(ContextWarning {
                code: "SAND_API_CATALOG_INCOMPATIBLE".into(),
                message: format!(
                    "The installed CLI API catalog cannot be proven compatible with this project: {}",
                    compatibility.reasons.join("; ")
                ),
            });
        }
        if api_catalog.coverage_status != CoverageStatus::Complete {
            warnings.push(ContextWarning {
                code: "SAND_API_COVERAGE_PARTIAL".into(),
                message: "The installed API contract catalog has partial coverage.".into(),
            });
        }
        if api_catalog.placeholder_codegen {
            warnings.push(ContextWarning {
                code: "SAND_API_PLACEHOLDER_PROFILE".into(),
                message: "The installed API catalog uses placeholder generated data.".into(),
            });
        }

        Ok(Self {
            schema_version: CONTEXT_SCHEMA_VERSION,
            is_sand_project: true,
            project: Some(project),
            cli,
            api_catalog,
            compatibility,
            warnings,
        })
    }

    pub fn catalog_is_compatible(&self) -> bool {
        self.compatibility.compatible != Some(false)
    }
}

fn cli_identity(catalog: &ApiCatalog) -> CliIdentity {
    let revision = env!("SAND_CLI_GIT_REVISION");
    CliIdentity {
        sand_version: catalog.sand_version.clone(),
        git_revision: (revision != "unknown").then(|| revision.to_owned()),
        workspace_root: PathBuf::from(env!("SAND_WORKSPACE_ROOT")),
    }
}

fn catalog_identity(catalog: &ApiCatalog) -> CatalogIdentity {
    CatalogIdentity {
        schema_version: catalog.schema_version,
        minecraft_version: catalog.configuration.minecraft_version.clone(),
        surface_profile: catalog.configuration.surface_profile.clone(),
        cargo_features: catalog.configuration.cargo_features.clone(),
        placeholder_codegen: catalog.configuration.placeholder_codegen,
        coverage_status: catalog.coverage.status,
        static_surface_items: catalog.coverage.static_surface_items,
        compiled_surface_items: catalog.configuration.compiled_surface_items,
    }
}

fn find_project_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|directory| directory.join("sand.toml").is_file())
        .map(Path::to_path_buf)
}

fn project_build_script_present(config_path: &Path) -> bool {
    config_path
        .parent()
        .is_some_and(|root| root.join("sand.build.rs").is_file())
}

fn compare(
    project: &ProjectIdentity,
    cli: &CliIdentity,
    catalog: &CatalogIdentity,
) -> Compatibility {
    let mut reasons = Vec::new();
    if project.minecraft_version != catalog.minecraft_version {
        reasons.push(format!(
            "project targets Minecraft {}, catalog targets {}",
            project.minecraft_version, catalog.minecraft_version
        ));
    }
    if project.sand_dependency.version != cli.sand_version {
        reasons.push(format!(
            "project resolves Sand {}, CLI embeds {}",
            project.sand_dependency.version, cli.sand_version
        ));
    }
    if project.sand_features != catalog.cargo_features {
        reasons.push(format!(
            "project Sand features [{}] differ from catalog features [{}]",
            project.sand_features.join(", "),
            catalog.cargo_features.join(", ")
        ));
    }
    match project.sand_dependency.source_kind {
        DependencySourceKind::Git => match (
            project.sand_dependency.resolved_git_revision.as_deref(),
            cli.git_revision.as_deref(),
        ) {
            (Some(project_revision), Some(cli_revision)) if project_revision == cli_revision => {}
            (Some(project_revision), Some(cli_revision)) => reasons.push(format!(
                "project Sand revision {project_revision} differs from CLI revision {cli_revision}"
            )),
            _ => reasons.push("git revision identity is unavailable".into()),
        },
        DependencySourceKind::Path => {
            let installed_sand = cli.workspace_root.join("sand");
            if project
                .sand_dependency
                .local_path
                .as_deref()
                .and_then(|path| path.canonicalize().ok())
                != installed_sand.canonicalize().ok()
            {
                reasons.push("project uses a different local Sand checkout than the CLI".into());
            }
        }
        DependencySourceKind::Registry => {}
        DependencySourceKind::Unknown => reasons.push("Sand dependency source is unknown".into()),
    }
    if catalog.placeholder_codegen {
        reasons.push("catalog uses placeholder generated data".into());
    }
    if catalog.coverage_status != CoverageStatus::Complete {
        reasons.push("catalog contract coverage is partial".into());
    }

    if reasons.is_empty() {
        Compatibility {
            status: CompatibilityStatus::Compatible,
            compatible: Some(true),
            reasons,
        }
    } else {
        Compatibility {
            status: CompatibilityStatus::Incompatible,
            compatible: Some(false),
            reasons,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    resolve: Option<CargoResolve>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
}

#[derive(Debug, Deserialize)]
struct CargoNode {
    id: String,
    #[serde(default)]
    deps: Vec<CargoNodeDep>,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CargoNodeDep {
    pkg: String,
}

impl CargoMetadata {
    fn sand_dependency(&self, project_root: &Path) -> Result<(SandDependency, Vec<String>)> {
        let resolve = self
            .resolve
            .as_ref()
            .context("Cargo metadata did not include dependency resolution")?;
        let project_manifest = project_root.join("Cargo.toml");
        let project_package = self
            .packages
            .iter()
            .find(|package| same_path(&package.manifest_path, &project_manifest))
            .with_context(|| {
                format!(
                    "Cargo metadata did not contain the project package '{}'",
                    project_manifest.display()
                )
            })?;
        let root_node = resolve
            .nodes
            .iter()
            .find(|node| node.id == project_package.id)
            .context("Cargo metadata did not contain the project dependency node")?;
        let sand_id = root_node
            .deps
            .iter()
            .find(|dependency| {
                self.packages
                    .iter()
                    .any(|package| package.id == dependency.pkg && package.name == "sand")
            })
            .map(|dependency| dependency.pkg.as_str())
            .context("Cargo.toml does not directly depend on the author-facing `sand` crate")?;
        let package = self
            .packages
            .iter()
            .find(|package| package.id == sand_id)
            .context("Cargo metadata omitted the resolved `sand` package")?;
        let mut features = resolve
            .nodes
            .iter()
            .find(|node| node.id == sand_id)
            .map(|node| node.features.clone())
            .unwrap_or_default();
        features.retain(|feature| feature != "default");
        features.sort();
        features.dedup();
        let (source_kind, resolved_git_revision, local_path) = match package.source.as_deref() {
            Some(source) if source.starts_with("git+") => (
                DependencySourceKind::Git,
                source
                    .rsplit_once('#')
                    .map(|(_, revision)| revision.to_owned()),
                None,
            ),
            Some(source) if source.starts_with("registry+") => {
                (DependencySourceKind::Registry, None, None)
            }
            None => (
                DependencySourceKind::Path,
                None,
                package.manifest_path.parent().map(Path::to_path_buf),
            ),
            _ => (DependencySourceKind::Unknown, None, None),
        };
        Ok((
            SandDependency {
                version: package.version.clone(),
                source_kind,
                source: package.source.clone(),
                resolved_git_revision,
                local_path,
            },
            features,
        ))
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    left.canonicalize().ok() == right.canonicalize().ok()
}

fn cargo_metadata(project_root: &Path) -> Result<CargoMetadata> {
    let manifest = project_root.join("Cargo.toml");
    if !manifest.is_file() {
        bail!("Sand project has no Cargo.toml at '{}'", manifest.display());
    }
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version=1",
            "--locked",
            "--offline",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .context("failed to invoke `cargo metadata`")?;
    if !output.status.success() {
        bail!(
            "`cargo metadata --locked --offline` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    serde_json::from_slice(&output.stdout).context("failed to parse Cargo metadata")
}

fn manifest_sand_dependency(project_root: &Path) -> Result<(SandDependency, Vec<String>)> {
    let manifest_path = project_root.join("Cargo.toml");
    let manifest: toml::Value = toml::from_str(
        &std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read '{}'", manifest_path.display()))?,
    )
    .with_context(|| format!("failed to parse '{}'", manifest_path.display()))?;
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .context("Cargo.toml does not have a [dependencies] table")?;
    let dependency = dependencies
        .iter()
        .find(|(name, dependency)| {
            name.as_str() == "sand"
                || dependency
                    .as_table()
                    .and_then(|table| table.get("package"))
                    .and_then(toml::Value::as_str)
                    == Some("sand")
        })
        .map(|(_, dependency)| dependency)
        .context("Cargo.toml does not directly depend on the author-facing `sand` crate")?;
    let table = dependency.as_table();
    let mut features = table
        .and_then(|table| table.get("features"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    features.sort();
    features.dedup();
    if let Some(path) = table
        .and_then(|table| table.get("path"))
        .and_then(toml::Value::as_str)
    {
        let local_path = project_root.join(path);
        let sand_manifest = local_path.join("Cargo.toml");
        let sand_toml: toml::Value = toml::from_str(
            &std::fs::read_to_string(&sand_manifest)
                .with_context(|| format!("failed to read '{}'", sand_manifest.display()))?,
        )?;
        let version = sand_toml
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str)
            .unwrap_or(env!("CARGO_PKG_VERSION"))
            .to_owned();
        return Ok((
            SandDependency {
                version,
                source_kind: DependencySourceKind::Path,
                source: None,
                resolved_git_revision: None,
                local_path: Some(local_path),
            },
            features,
        ));
    }
    if let Some(git) = table
        .and_then(|table| table.get("git"))
        .and_then(toml::Value::as_str)
    {
        let locked = locked_sand_package(project_root, Some(git));
        let requested_revision = locked
            .as_ref()
            .and_then(|(_, source)| source.as_deref())
            .and_then(|source| {
                source
                    .rsplit_once('#')
                    .map(|(_, revision)| revision.to_owned())
            })
            .or_else(|| {
                table
                    .and_then(|table| table.get("rev"))
                    .and_then(toml::Value::as_str)
                    .map(ToOwned::to_owned)
            });
        return Ok((
            SandDependency {
                version: locked
                    .as_ref()
                    .map(|(version, _)| version.clone())
                    .or_else(|| {
                        table
                            .and_then(|table| table.get("version"))
                            .and_then(toml::Value::as_str)
                            .map(ToOwned::to_owned)
                    })
                    .unwrap_or_else(|| "unresolved".into()),
                source_kind: DependencySourceKind::Git,
                source: locked
                    .and_then(|(_, source)| source)
                    .or_else(|| Some(format!("git+{git}"))),
                resolved_git_revision: requested_revision,
                local_path: None,
            },
            features,
        ));
    }
    let locked = locked_sand_package(project_root, None);
    let version = locked
        .as_ref()
        .map(|(version, _)| version.as_str())
        .or_else(|| dependency.as_str())
        .or_else(|| {
            table
                .and_then(|table| table.get("version"))
                .and_then(toml::Value::as_str)
        })
        .unwrap_or("unresolved")
        .to_owned();
    Ok((
        SandDependency {
            version,
            source_kind: DependencySourceKind::Registry,
            source: locked.and_then(|(_, source)| source),
            resolved_git_revision: None,
            local_path: None,
        },
        features,
    ))
}

fn locked_sand_package(
    project_root: &Path,
    expected_git: Option<&str>,
) -> Option<(String, Option<String>)> {
    let lock: toml::Value =
        toml::from_str(&std::fs::read_to_string(project_root.join("Cargo.lock")).ok()?).ok()?;
    lock.get("package")?
        .as_array()?
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("sand"))
        .find_map(|package| {
            let source = package
                .get("source")
                .and_then(toml::Value::as_str)
                .map(ToOwned::to_owned);
            if expected_git.is_some_and(|git| {
                !source
                    .as_deref()
                    .is_some_and(|source| source.starts_with(&format!("git+{git}")))
            }) {
                return None;
            }
            Some((package.get("version")?.as_str()?.to_owned(), source))
        })
}

pub fn render_human(context: &ProjectContext) -> String {
    let mut output = String::new();
    if let Some(project) = &context.project {
        output.push_str(&format!(
            "Sand project {}\n  root: {}\n  Minecraft: {}\n  Sand: {} ({:?})\n  profile: {}\n  resource pack: {}\n  sand.build.rs: {}\n",
            project.namespace,
            project.root.display(),
            project.minecraft_version,
            project.sand_dependency.version,
            project.sand_dependency.source_kind,
            project.active_profile,
            project.resource_pack_enabled,
            project.build_script_present,
        ));
    } else {
        output.push_str("Not inside a Sand project.\n");
    }
    output.push_str(&format!(
        "Installed CLI catalog\n  Sand: {}\n  Minecraft: {}\n  profile: {}\n  coverage: {:?}\n  compatibility: {:?}\n",
        context.cli.sand_version,
        context.api_catalog.minecraft_version,
        context.api_catalog.surface_profile,
        context.api_catalog.coverage_status,
        context.compatibility.status,
    ));
    for warning in &context.warnings {
        output.push_str(&format!(
            "warning [{}]: {}\n",
            warning.code, warning.message
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use sand_api_contract::{ApiConfiguration, ApiCoverage};
    use tempfile::TempDir;

    fn catalog() -> ApiCatalog {
        ApiCatalog::from_entries_with_coverage(
            env!("CARGO_PKG_VERSION"),
            ApiConfiguration {
                surface_profile: "26.2".into(),
                minecraft_version: "26.2".into(),
                cargo_features: Vec::new(),
                placeholder_codegen: false,
                compiled_surface_items: 0,
            },
            Vec::new(),
            ApiCoverage {
                status: CoverageStatus::Complete,
                static_surface_items: 0,
                pending_item_ceiling: 0,
                pending_scope_ceiling: 0,
                pending_scopes: Vec::new(),
            },
        )
        .unwrap()
    }

    #[test]
    fn outside_project_is_explicit_and_deterministic() {
        let temp = TempDir::new().unwrap();
        let first = ProjectContext::discover_from(&catalog(), "dev", temp.path()).unwrap();
        let second = ProjectContext::discover_from(&catalog(), "dev", temp.path()).unwrap();
        assert_eq!(first, second);
        assert!(!first.is_sand_project);
        assert_eq!(first.compatibility.status, CompatibilityStatus::NotAProject);
    }

    fn project(dependency: SandDependency) -> ProjectIdentity {
        ProjectIdentity {
            root: PathBuf::from(env!("SAND_WORKSPACE_ROOT")),
            namespace: "fixture".into(),
            minecraft_version: "26.2".into(),
            sand_dependency: dependency,
            sand_features: Vec::new(),
            active_profile: "dev".into(),
            resource_pack_enabled: false,
            build_script_present: false,
        }
    }

    fn cli() -> CliIdentity {
        CliIdentity {
            sand_version: env!("CARGO_PKG_VERSION").into(),
            git_revision: Some("abc123".into()),
            workspace_root: PathBuf::from(env!("SAND_WORKSPACE_ROOT")),
        }
    }

    fn catalog_identity() -> CatalogIdentity {
        CatalogIdentity {
            schema_version: 3,
            minecraft_version: "26.2".into(),
            surface_profile: "26.2".into(),
            cargo_features: Vec::new(),
            placeholder_codegen: false,
            coverage_status: CoverageStatus::Complete,
            static_surface_items: 1,
            compiled_surface_items: 1,
        }
    }

    #[test]
    fn path_pinned_project_matches_only_the_cli_checkout() {
        let matching = SandDependency {
            version: env!("CARGO_PKG_VERSION").into(),
            source_kind: DependencySourceKind::Path,
            source: None,
            resolved_git_revision: None,
            local_path: Some(PathBuf::from(env!("SAND_WORKSPACE_ROOT")).join("sand")),
        };
        assert_eq!(
            compare(&project(matching), &cli(), &catalog_identity()).status,
            CompatibilityStatus::Compatible
        );
        let different = SandDependency {
            local_path: Some(PathBuf::from("/different/sand")),
            ..project(SandDependency {
                version: env!("CARGO_PKG_VERSION").into(),
                source_kind: DependencySourceKind::Path,
                source: None,
                resolved_git_revision: None,
                local_path: None,
            })
            .sand_dependency
        };
        assert_eq!(
            compare(&project(different), &cli(), &catalog_identity()).status,
            CompatibilityStatus::Incompatible
        );
    }

    #[test]
    fn git_pinned_revision_must_match_the_cli_revision() {
        let dependency = |revision: &str| SandDependency {
            version: env!("CARGO_PKG_VERSION").into(),
            source_kind: DependencySourceKind::Git,
            source: Some("git+https://github.com/ThatOneToast/sand".into()),
            resolved_git_revision: Some(revision.into()),
            local_path: None,
        };
        assert_eq!(
            compare(&project(dependency("abc123")), &cli(), &catalog_identity()).status,
            CompatibilityStatus::Compatible
        );
        assert_eq!(
            compare(&project(dependency("def456")), &cli(), &catalog_identity()).status,
            CompatibilityStatus::Incompatible
        );
    }

    #[test]
    fn minecraft_profile_and_partial_coverage_fail_closed() {
        let dependency = SandDependency {
            version: env!("CARGO_PKG_VERSION").into(),
            source_kind: DependencySourceKind::Git,
            source: None,
            resolved_git_revision: Some("abc123".into()),
            local_path: None,
        };
        let mut identity = catalog_identity();
        identity.minecraft_version = "1.21.4".into();
        assert_eq!(
            compare(&project(dependency.clone()), &cli(), &identity).status,
            CompatibilityStatus::Incompatible
        );
        identity.minecraft_version = "26.2".into();
        identity.coverage_status = CoverageStatus::Partial;
        assert_eq!(
            compare(&project(dependency), &cli(), &identity).status,
            CompatibilityStatus::Incompatible
        );
    }

    #[test]
    fn manifest_reader_resolves_path_features_without_a_lockfile() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            format!(
                "[package]\nname='fixture'\nversion='0.1.0'\n[dependencies]\nsand_framework={{package='sand',path={:?},features=['systems-damage']}}\n",
                PathBuf::from(env!("SAND_WORKSPACE_ROOT")).join("sand")
            ),
        )
        .unwrap();
        let (dependency, features) = manifest_sand_dependency(temp.path()).unwrap();
        assert_eq!(dependency.source_kind, DependencySourceKind::Path);
        assert_eq!(dependency.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(features, ["systems-damage"]);
    }

    #[test]
    fn cargo_resolution_handles_renamed_dependencies_and_expanded_features() {
        let temp = TempDir::new().unwrap();
        let project_manifest = temp.path().join("Cargo.toml");
        std::fs::write(
            &project_manifest,
            "[package]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        let sand_manifest = PathBuf::from(env!("SAND_WORKSPACE_ROOT")).join("sand/Cargo.toml");
        let metadata = CargoMetadata {
            packages: vec![
                CargoPackage {
                    id: "fixture-id".into(),
                    name: "fixture".into(),
                    version: "0.1.0".into(),
                    source: None,
                    manifest_path: project_manifest,
                },
                CargoPackage {
                    id: "sand-id".into(),
                    name: "sand".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                    source: None,
                    manifest_path: sand_manifest,
                },
            ],
            resolve: Some(CargoResolve {
                nodes: vec![
                    CargoNode {
                        id: "fixture-id".into(),
                        deps: vec![CargoNodeDep {
                            pkg: "sand-id".into(),
                        }],
                        features: Vec::new(),
                    },
                    CargoNode {
                        id: "sand-id".into(),
                        deps: Vec::new(),
                        features: vec!["systems-all".into(), "systems-damage".into()],
                    },
                ],
            }),
        };
        let (dependency, features) = metadata.sand_dependency(temp.path()).unwrap();
        assert_eq!(dependency.source_kind, DependencySourceKind::Path);
        assert_eq!(features, ["systems-all", "systems-damage"]);
    }

    #[test]
    fn manifest_reader_prefers_the_cargo_lock_git_revision() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n[dependencies]\nsand={git='https://github.com/ThatOneToast/sand',rev='requested'}\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = 'sand'\nversion = '0.1.0'\nsource = 'git+https://github.com/ThatOneToast/sand?rev=requested#resolved123'\n",
        )
        .unwrap();
        let (dependency, _) = manifest_sand_dependency(temp.path()).unwrap();
        assert_eq!(dependency.source_kind, DependencySourceKind::Git);
        assert_eq!(
            dependency.resolved_git_revision.as_deref(),
            Some("resolved123")
        );
    }
}
