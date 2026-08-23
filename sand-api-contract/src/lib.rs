#![forbid(unsafe_code)]

//! The versioned, serializable model behind Sand's public API contracts.
//!
//! Author-facing items register [`ApiRegistration`] values at compile time.
//! Facades combine those registrations with any generated providers, then use
//! [`ApiCatalog::from_entries_with_coverage`] to build the complete, validated,
//! deterministically ordered catalog consumed by the CLI and external tooling.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Current machine-readable catalog schema.
pub const SCHEMA_VERSION: u32 = 3;

/// The Rust-level shape of a supported API item.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKind {
    Module,
    Struct,
    Enum,
    Variant,
    Trait,
    Function,
    Method,
    TraitMethod,
    TypeAlias,
    Constant,
    AssociatedConst,
    AssociatedType,
    Field,
    Macro,
}

/// Shared parser and validator for the `#[api(...)]` authoring syntax.
///
/// This is feature-gated because installed catalogs do not otherwise need a
/// Rust parser. Procedural macros and build-time surface enforcement enable
/// the `syntax` feature so they cannot drift into accepting different
/// contract dialects.
#[cfg(feature = "syntax")]
pub mod syntax;

/// Parameter documentation in declaration order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiParameter {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rust_type: Option<String>,
    /// Source-authored behavioral meaning when the defining Rustdoc explains
    /// this argument. An absent string is distinct from generated filler: the
    /// structural name and Rust type remain authoritative.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// One complete, owned public API contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiEntry {
    pub canonical_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub canonical_module: String,
    pub kind: ApiKind,
    pub signature: String,
    pub summary: String,
    /// Additional source-authored domain context when the defining Rustdoc
    /// provides it. Omitted rather than synthesized from the module name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub context: String,
    /// Observable Minecraft behavior documented by the source or generator.
    /// Pure Rust value APIs may legitimately omit this section.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub minecraft: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub use_when: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avoid_when: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ApiParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    /// Smallest source-authored behavioral example, when the defining item
    /// provides one. The catalog omits this field rather than fabricating an
    /// invocation with undefined placeholder variables.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub example: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub availability: Vec<String>,
}

/// Complete metadata export for one installed Sand version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiCatalog {
    pub schema_version: u32,
    pub sand_version: String,
    pub configuration: ApiConfiguration,
    pub coverage: ApiCoverage,
    pub entries: Vec<ApiEntry>,
}

/// Exact build/profile identity of an installed API surface.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiConfiguration {
    /// Ratchet profile selected from `api-surface-profiles.toml`.
    pub surface_profile: String,
    /// Minecraft version declared by the generated providers.
    pub minecraft_version: String,
    /// Cargo features enabled for this compiled Sand facade.
    pub cargo_features: Vec<String>,
    /// Whether generated APIs came from the explicit placeholder fallback.
    pub placeholder_codegen: bool,
    /// Number of identities reachable in this exact compiled configuration.
    pub compiled_surface_items: usize,
}

/// Whether the installed catalog covers the complete supported surface.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Partial,
    Complete,
}

/// Machine-visible state of the module/generator migration ratchet.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiCoverage {
    pub status: CoverageStatus,
    pub static_surface_items: usize,
    pub pending_item_ceiling: usize,
    pub pending_scope_ceiling: usize,
    pub pending_scopes: Vec<String>,
}

impl ApiCoverage {
    /// Conservative coverage for callers that only possess linked
    /// registrations and no build-generated surface report.
    pub fn unverified() -> Self {
        Self {
            status: CoverageStatus::Partial,
            static_surface_items: 0,
            pending_item_ceiling: 0,
            pending_scope_ceiling: 1,
            pending_scopes: vec!["coverage report unavailable".to_owned()],
        }
    }
}

/// Borrowed parameter data emitted by `#[api]`.
#[derive(Clone, Copy, Debug)]
pub struct StaticApiParameter {
    pub name: &'static str,
    pub description: &'static str,
}

/// Link-time registration emitted by `#[api]`.
///
/// Arrays retain source declaration order. Catalog construction sorts only
/// identities and set-like values (aliases and availability).
#[derive(Clone, Copy, Debug)]
pub struct ApiRegistration {
    pub canonical_path: &'static str,
    pub aliases: &'static [&'static str],
    pub canonical_module: &'static str,
    pub kind: ApiKind,
    pub signature: &'static str,
    pub summary: &'static str,
    pub context: &'static str,
    pub minecraft: &'static str,
    pub use_when: &'static [&'static str],
    pub avoid_when: &'static [&'static str],
    pub parameters: &'static [StaticApiParameter],
    pub returns: Option<&'static str>,
    pub example: &'static str,
    pub availability: &'static [&'static str],
}

inventory::collect!(ApiRegistration);
pub use inventory;

/// Validation failure while assembling the installed catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogError {
    DuplicateCanonicalPath(String),
    DuplicateLookupPath { path: String, owners: Vec<String> },
    InvalidCanonicalPath(String),
    InvalidAlias { owner: String, alias: String },
    InvalidCoverage(String),
    InvalidConfiguration(String),
    InvalidEntry { path: String, message: String },
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCanonicalPath(path) => {
                write!(f, "duplicate canonical API path `{path}`")
            }
            Self::DuplicateLookupPath { path, owners } => write!(
                f,
                "API lookup path `{path}` belongs to multiple contracts: {}",
                owners.join(", ")
            ),
            Self::InvalidCanonicalPath(path) => write!(f, "invalid canonical API path `{path}`"),
            Self::InvalidAlias { owner, alias } => {
                write!(f, "invalid alias `{alias}` on API `{owner}`")
            }
            Self::InvalidCoverage(message) => write!(f, "invalid API coverage: {message}"),
            Self::InvalidConfiguration(message) => {
                write!(f, "invalid API configuration: {message}")
            }
            Self::InvalidEntry { path, message } => {
                write!(f, "invalid API contract `{path}`: {message}")
            }
        }
    }
}

impl std::error::Error for CatalogError {}

impl ApiCatalog {
    /// Applies quality gates that require a fully resolved installed surface.
    ///
    /// Raw linked registrations can still contain migration-time structural
    /// placeholders; installed metadata must replace those from reachability
    /// before calling this validator.
    pub fn validate_quality(&self) -> Result<(), CatalogError> {
        validate_resolved_quality(&self.entries)
    }

    /// Builds a catalog from owned entries plus an explicit scope report.
    ///
    /// Generated API providers use this after combining their build-selected
    /// entries with the registrations linked into the executable. The same
    /// ordering, identity, alias, and coverage validation therefore applies
    /// to handwritten and generated contracts.
    pub fn from_entries_with_coverage(
        sand_version: impl Into<String>,
        mut configuration: ApiConfiguration,
        mut entries: Vec<ApiEntry>,
        mut coverage: ApiCoverage,
    ) -> Result<Self, CatalogError> {
        inherit_associated_aliases(&mut entries);
        for entry in &mut entries {
            entry.aliases.sort();
            entry.aliases.dedup();
            entry.availability.sort();
            entry.availability.dedup();
        }
        entries.sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));
        validate_entries(&entries)?;
        configuration.cargo_features.sort();
        configuration.cargo_features.dedup();
        validate_configuration(&configuration, entries.len())?;
        coverage.pending_scopes.sort();
        coverage.pending_scopes.dedup();
        validate_coverage(&coverage)?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            sand_version: sand_version.into(),
            configuration,
            coverage,
            entries,
        })
    }

    /// Builds a catalog from explicit registrations, primarily for generated
    /// catalogs and focused tests.
    pub fn from_registrations<'a>(
        sand_version: impl Into<String>,
        configuration: ApiConfiguration,
        registrations: impl IntoIterator<Item = &'a ApiRegistration>,
    ) -> Result<Self, CatalogError> {
        Self::from_registrations_with_coverage(
            sand_version,
            configuration,
            registrations,
            ApiCoverage::unverified(),
        )
    }

    /// Builds a catalog from registrations plus an explicit scope report.
    pub fn from_registrations_with_coverage<'a>(
        sand_version: impl Into<String>,
        configuration: ApiConfiguration,
        registrations: impl IntoIterator<Item = &'a ApiRegistration>,
        coverage: ApiCoverage,
    ) -> Result<Self, CatalogError> {
        let entries = registrations
            .into_iter()
            .map(ApiEntry::from)
            .collect::<Vec<_>>();
        Self::from_entries_with_coverage(sand_version, configuration, entries, coverage)
    }

    /// Resolves either a canonical path or a declared alias.
    pub fn find(&self, path: &str) -> Option<&ApiEntry> {
        self.entries.iter().find(|entry| {
            entry.canonical_path == path || entry.aliases.iter().any(|alias| alias == path)
        })
    }

    /// Deterministic, explainable text search. Results sort by descending
    /// score, then canonical path. Exact path, path segment, alias, summary,
    /// and descriptive-field matches receive progressively lower weights.
    pub fn search(&self, query: &str) -> Vec<&ApiEntry> {
        let query = query.trim().to_lowercase();
        let words = query
            .split_whitespace()
            .map(normalize_search_word)
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>();
        if words.is_empty() {
            return Vec::new();
        }
        let mut results = self
            .entries
            .iter()
            .filter_map(|entry| search_score(entry, &query, &words).map(|score| (score, entry)))
            .collect::<Vec<_>>();
        results.sort_by(|(left_score, left), (right_score, right)| {
            right_score
                .cmp(left_score)
                .then_with(|| left.canonical_path.cmp(&right.canonical_path))
        });
        results.into_iter().map(|(_, entry)| entry).collect()
    }

    /// Returns direct children of a canonical module, grouped by kind. Nested
    /// descendants are intentionally excluded.
    pub fn module(&self, module: &str) -> BTreeMap<ApiKind, Vec<&ApiEntry>> {
        let mut groups = BTreeMap::<ApiKind, Vec<&ApiEntry>>::new();
        for entry in &self.entries {
            if entry.canonical_module == module {
                groups.entry(entry.kind).or_default().push(entry);
            }
        }
        groups
    }

    /// Stable pretty JSON with a trailing newline for command-line output.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self).map(|json| format!("{json}\n"))
    }
}

impl ApiEntry {
    /// Validate the schema and ownership invariants of one resolved contract.
    /// Catalog and generated-provider assembly share this path so structured
    /// semantic metadata cannot drift between producers.
    pub fn validate(&self) -> Result<(), CatalogError> {
        validate_entry(self)
    }
}

/// Mirrors Rust's associated-item reachability for catalog entries whose
/// contract intentionally names only the canonical path.  A method or
/// associated item of an aliased public type is reachable through each type
/// alias as well; deriving those paths here keeps link-time registrations,
/// build-time enforcement, and installed metadata in agreement without
/// repeating fragile alias lists on every method contract.
fn inherit_associated_aliases(entries: &mut [ApiEntry]) {
    let parent_aliases = entries
        .iter()
        .filter(|entry| matches!(entry.kind, ApiKind::Struct | ApiKind::Enum | ApiKind::Trait))
        .map(|entry| (entry.canonical_path.clone(), entry.aliases.clone()))
        .collect::<BTreeMap<_, _>>();
    for entry in entries {
        if !entry.aliases.is_empty() {
            continue;
        }
        let Some((parent, member)) = entry.canonical_path.rsplit_once("::") else {
            continue;
        };
        let Some(aliases) = parent_aliases.get(parent) else {
            continue;
        };
        entry.aliases = aliases
            .iter()
            .map(|alias| format!("{alias}::{member}"))
            .collect();
    }
}

fn validate_coverage(coverage: &ApiCoverage) -> Result<(), CatalogError> {
    if coverage.pending_scopes.len() > coverage.pending_scope_ceiling {
        return Err(CatalogError::InvalidCoverage(format!(
            "{} pending scopes exceed ceiling {}",
            coverage.pending_scopes.len(),
            coverage.pending_scope_ceiling
        )));
    }
    if coverage.status == CoverageStatus::Complete
        && (coverage.pending_item_ceiling != 0
            || coverage.pending_scope_ceiling != 0
            || !coverage.pending_scopes.is_empty())
    {
        return Err(CatalogError::InvalidCoverage(
            "complete status requires zero pending items and scopes".to_owned(),
        ));
    }
    Ok(())
}

fn validate_configuration(
    configuration: &ApiConfiguration,
    entry_count: usize,
) -> Result<(), CatalogError> {
    if configuration.surface_profile.trim().is_empty()
        || configuration.minecraft_version.trim().is_empty()
    {
        return Err(CatalogError::InvalidConfiguration(
            "surface_profile and minecraft_version must be nonempty".to_owned(),
        ));
    }
    if configuration
        .cargo_features
        .iter()
        .any(|feature| feature.trim().is_empty())
    {
        return Err(CatalogError::InvalidConfiguration(
            "Cargo feature names must be nonempty".to_owned(),
        ));
    }
    if configuration.compiled_surface_items != entry_count {
        return Err(CatalogError::InvalidConfiguration(format!(
            "compiled surface count {} does not match {} catalog entries",
            configuration.compiled_surface_items, entry_count
        )));
    }
    if configuration.placeholder_codegen != (configuration.surface_profile == "placeholder-codegen")
    {
        return Err(CatalogError::InvalidConfiguration(
            "placeholder_codegen must agree with the placeholder-codegen surface profile"
                .to_owned(),
        ));
    }
    Ok(())
}

impl From<&ApiRegistration> for ApiEntry {
    fn from(value: &ApiRegistration) -> Self {
        let mut aliases = value
            .aliases
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        aliases.sort();
        aliases.dedup();
        let mut availability = value
            .availability
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        availability.sort();
        availability.dedup();
        Self {
            canonical_path: value.canonical_path.to_owned(),
            aliases,
            canonical_module: value.canonical_module.to_owned(),
            kind: value.kind,
            signature: value.signature.to_owned(),
            summary: value.summary.to_owned(),
            context: value.context.to_owned(),
            minecraft: value.minecraft.to_owned(),
            use_when: value.use_when.iter().map(ToString::to_string).collect(),
            avoid_when: value.avoid_when.iter().map(ToString::to_string).collect(),
            parameters: value
                .parameters
                .iter()
                .map(|parameter| ApiParameter {
                    name: parameter.name.to_owned(),
                    rust_type: None,
                    description: parameter.description.to_owned(),
                })
                .collect(),
            returns: value.returns.map(ToOwned::to_owned),
            return_type: None,
            example: value.example.to_owned(),
            availability,
        }
    }
}

fn validate_entries(entries: &[ApiEntry]) -> Result<(), CatalogError> {
    let mut lookup = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut previous = None;
    for entry in entries {
        entry.validate()?;
        if previous == Some(entry.canonical_path.as_str()) {
            return Err(CatalogError::DuplicateCanonicalPath(
                entry.canonical_path.clone(),
            ));
        }
        previous = Some(&entry.canonical_path);
        lookup
            .entry(&entry.canonical_path)
            .or_default()
            .insert(&entry.canonical_path);
        for alias in &entry.aliases {
            if !valid_path(alias) || alias == &entry.canonical_path {
                return Err(CatalogError::InvalidAlias {
                    owner: entry.canonical_path.clone(),
                    alias: alias.clone(),
                });
            }
            lookup
                .entry(alias)
                .or_default()
                .insert(&entry.canonical_path);
        }
    }
    if let Some((path, owners)) = lookup.into_iter().find(|(_, owners)| owners.len() > 1) {
        return Err(CatalogError::DuplicateLookupPath {
            path: path.to_owned(),
            owners: owners.into_iter().map(ToOwned::to_owned).collect(),
        });
    }
    Ok(())
}

fn validate_entry(entry: &ApiEntry) -> Result<(), CatalogError> {
    for (name, value) in [
        ("signature", entry.signature.as_str()),
        ("summary", entry.summary.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: format!("required field `{name}` is empty"),
            });
        }
    }
    if entry
        .use_when
        .iter()
        .chain(&entry.avoid_when)
        .any(|value| value.trim().is_empty())
    {
        return Err(CatalogError::InvalidEntry {
            path: entry.canonical_path.clone(),
            message: "use_when and avoid_when cannot contain empty guidance".into(),
        });
    }
    if !valid_path(&entry.canonical_path) {
        return Err(CatalogError::InvalidCanonicalPath(
            entry.canonical_path.clone(),
        ));
    }
    if !valid_module_path(&entry.canonical_module)
        || (entry.canonical_path != entry.canonical_module
            && !entry
                .canonical_path
                .starts_with(&format!("{}::", entry.canonical_module)))
    {
        return Err(CatalogError::InvalidEntry {
            path: entry.canonical_path.clone(),
            message: format!(
                "canonical_module `{}` is not a valid owner of this path",
                entry.canonical_module
            ),
        });
    }
    let mut parameter_names = BTreeSet::new();
    for parameter in &entry.parameters {
        if parameter.name.trim().is_empty()
            || parameter
                .rust_type
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "parameters require nonempty names and Rust types".into(),
            });
        }
        if !parameter_names.insert(parameter.name.as_str()) {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: format!("duplicate parameter `{}`", parameter.name),
            });
        }
    }
    if entry
        .returns
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(CatalogError::InvalidEntry {
            path: entry.canonical_path.clone(),
            message: "returns documentation cannot be empty".into(),
        });
    }
    if entry
        .return_type
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(CatalogError::InvalidEntry {
            path: entry.canonical_path.clone(),
            message: "return_type cannot be empty".into(),
        });
    }
    if entry
        .availability
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(CatalogError::InvalidEntry {
            path: entry.canonical_path.clone(),
            message: "availability values cannot be empty".into(),
        });
    }
    Ok(())
}

fn validate_resolved_quality(entries: &[ApiEntry]) -> Result<(), CatalogError> {
    for entry in entries {
        let compact_signature = entry.signature.replace(char::is_whitespace, "");
        if entry.signature.contains("#[doc")
            || entry.signature.contains("# [doc")
            || entry.signature.contains("```")
            || [
                "sand_core::",
                "sand_commands::",
                "sand_components::",
                "sand_resourcepack::",
                "sand_version::",
                "crate::",
            ]
            .iter()
            .any(|path| compact_signature.contains(path))
        {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: format!(
                    "signature contains Rustdoc attributes or implementation-only crate paths: {}",
                    entry.signature
                ),
            });
        }
        if entry.summary.starts_with("Configures or performs ")
            || entry.summary.starts_with("Builds or resolves ")
            || entry
                .summary
                .contains("on this typed datapack component definition")
            || !has_specific_semantics(&entry.summary)
        {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "summary is generic family filler rather than API-specific semantics"
                    .into(),
            });
        }
        if entry.parameters.iter().any(|parameter| {
            parameter
                .description
                .starts_with("Rust parameter with type `")
        }) {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "parameter documentation only repeats its Rust type".into(),
            });
        }
        if entry
            .returns
            .as_deref()
            .is_some_and(|returns| returns.starts_with("A value with Rust type `"))
        {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "return documentation only repeats its Rust type".into(),
            });
        }
        if [
            "author-facing entity API",
            "author-facing component API",
            "author-facing typed state API",
            "handwritten typed Minecraft command API",
        ]
        .iter()
        .any(|placeholder| entry.signature.contains(placeholder))
        {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "signature is a family placeholder rather than source-derived Rust shape"
                    .into(),
            });
        }
        if entry.summary.ends_with("(e") {
            return Err(CatalogError::InvalidEntry {
                path: entry.canonical_path.clone(),
                message: "summary appears truncated at an abbreviation".into(),
            });
        }
        if matches!(
            entry.kind,
            ApiKind::Function | ApiKind::Method | ApiKind::TraitMethod
        ) {
            if entry.context == entry.summary
                || entry.minecraft.starts_with(
                    "Minecraft and generated-output behavior follows the defining item's documented semantics:",
                )
                || entry.use_when.iter().any(|guidance| {
                    guidance.starts_with(
                        "When the defining item's documented behavior is required:",
                    )
                })
                || entry.avoid_when.iter().any(|guidance| {
                    guidance
                        == "When the defining item's documented preconditions or scope do not apply."
                })
            {
                return Err(CatalogError::InvalidEntry {
                    path: entry.canonical_path.clone(),
                    message: "callable guidance is unresolved family-level prose".into(),
                });
            }
            if !entry.example.trim().is_empty()
                && entry.example.trim().starts_with("use sand::")
                && entry.example.trim().lines().count() == 1
            {
                return Err(CatalogError::InvalidEntry {
                    path: entry.canonical_path.clone(),
                    message: "nontrivial callable requires a behavioral example".into(),
                });
            }
            if [
                "This declaration belongs to Sand's typed entity model.",
                "This declaration provides the typed scoreboard or lifecycle primitives",
                "This opt-in system composes Sand's typed primitives",
                "This semantic component model describes a datapack resource",
            ]
            .iter()
            .any(|template| entry.context.starts_with(template))
            {
                return Err(CatalogError::InvalidEntry {
                    path: entry.canonical_path.clone(),
                    message: "callable context is unresolved family-level prose".into(),
                });
            }
        }
    }
    Ok(())
}

/// Returns whether prose carries at least one API-specific concept rather
/// than consisting entirely of contract boilerplate.
pub fn has_specific_semantics(summary: &str) -> bool {
    const GENERIC: &[&str] = &[
        "a",
        "an",
        "and",
        "api",
        "author",
        "builder",
        "component",
        "configures",
        "constructs",
        "creates",
        "datapack",
        "definition",
        "minecraft",
        "new",
        "of",
        "operation",
        "or",
        "performs",
        "provides",
        "public",
        "represents",
        "resource",
        "resolves",
        "returns",
        "json",
        "pack",
        "the",
        "this",
        "to",
        "typed",
        "use",
        "value",
    ];
    let words = summary
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let specific = words
        .iter()
        .filter(|word| !GENERIC.contains(&word.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    let generic_action = words.first().is_some_and(|word| {
        [
            "configures",
            "constructs",
            "creates",
            "performs",
            "provides",
            "represents",
            "resolves",
            "returns",
        ]
        .contains(&word.as_str())
    });
    !specific.is_empty() && (!generic_action || specific.len() >= 2)
}

/// Extracts ordinary Rustdoc prose paragraphs while excluding fenced code,
/// headings, and the generated API-contract lookup footer.
pub fn rustdoc_prose_paragraphs(documentation: &str) -> Vec<String> {
    let mut in_code = false;
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in documentation.lines() {
        let line = line.trim();
        if line.eq_ignore_ascii_case("# API Contract") {
            break;
        }
        if line.contains("API Contract:") && line.contains("sand api show ") {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        if line.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code || markdown_heading(line) {
            continue;
        }
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        current.push(line.replace("**", ""));
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    paragraphs
}

/// Returns whether the first author-facing prose paragraph is substantive.
/// Fenced examples and generated headings cannot satisfy this semantic gate.
pub fn rustdoc_has_specific_semantics(documentation: &str) -> bool {
    let first = rustdoc_prose_paragraphs(documentation)
        .into_iter()
        .next()
        .unwrap_or_default();
    !first.starts_with("use ") && has_specific_semantics(&first)
}

fn markdown_heading(line: &str) -> bool {
    let hashes = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    (1..=6).contains(&hashes)
        && line[hashes..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace)
}

fn valid_path(path: &str) -> bool {
    let segments = path.split("::").collect::<Vec<_>>();
    matches!(segments.first(), Some(&"sand"))
        && segments.len() > 1
        && segments.iter().enumerate().all(|(position, segment)| {
            !segment.is_empty()
                && (position + 1 == segments.len() && segment.chars().all(|ch| ch.is_ascii_digit())
                    || segment.chars().enumerate().all(|(index, ch)| {
                        ch == '_'
                            || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit())
                    }))
        })
}

fn valid_module_path(path: &str) -> bool {
    path == "sand" || valid_path(path)
}

fn normalize_search_word(word: &str) -> String {
    let word = word.trim_matches(|character: char| !character.is_ascii_alphanumeric());
    if let Some(stem) = word.strip_suffix("ies")
        && !stem.is_empty()
    {
        return format!("{stem}y");
    }
    if let Some(stem) = word.strip_suffix('s')
        && stem.len() > 2
    {
        return stem.to_owned();
    }
    word.to_owned()
}

fn search_score(entry: &ApiEntry, query: &str, words: &[String]) -> Option<u32> {
    let path = entry.canonical_path.to_lowercase();
    if path == query {
        return Some(10_000);
    }
    let aliases = entry
        .aliases
        .iter()
        .map(|alias| alias.to_lowercase())
        .collect::<Vec<_>>();
    if aliases.iter().any(|alias| alias == query) {
        return Some(9_000);
    }
    let summary = entry.summary.to_lowercase();
    let descriptive = entry
        .parameters
        .iter()
        .map(|parameter| parameter.description.as_str())
        .chain(entry.use_when.iter().map(String::as_str))
        .chain(entry.avoid_when.iter().map(String::as_str))
        .chain(std::iter::once(entry.minecraft.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    let mut matched = 0_u32;
    let mut score = 0_u32;
    for word in words {
        let word_score = if path.rsplit("::").next() == Some(word.as_str()) {
            500
        } else if path.split("::").any(|segment| segment == word) {
            350
        } else if path.contains(word) {
            300
        } else if aliases.iter().any(|alias| alias.contains(word)) {
            250
        } else if summary.contains(word) {
            120
        } else if descriptive.contains(word) {
            20
        } else {
            continue;
        };
        matched += 1;
        score += word_score;
    }
    (matched > 0).then_some(matched * 1_000 + score)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration(compiled_surface_items: usize) -> ApiConfiguration {
        ApiConfiguration {
            surface_profile: "test".into(),
            minecraft_version: "test".into(),
            cargo_features: Vec::new(),
            placeholder_codegen: false,
            compiled_surface_items,
        }
    }

    static PARAMETERS: &[StaticApiParameter] = &[StaticApiParameter {
        name: "slot",
        description: "The equipment slot to inspect.",
    }];

    static REGISTRATION: ApiRegistration = ApiRegistration {
        canonical_path: "sand::predicate::EquipmentPredicate::slot",
        aliases: &["sand::prelude::EquipmentPredicate::slot"],
        canonical_module: "sand::predicate",
        kind: ApiKind::Method,
        signature: "pub fn slot(&mut self, slot: ItemSlot)",
        summary: "Adds an equipment-slot condition.",
        context: "Equipment predicates describe item conditions on entities.",
        minecraft: "Adds an equipment entry to an entity predicate resource.",
        use_when: &["Matching equipment"],
        avoid_when: &["Mutating equipment"],
        parameters: PARAMETERS,
        returns: None,
        example: "predicate.slot(ItemSlot::Head);",
        availability: &["feature = predicates"],
    };

    static ALIASED_TYPE: ApiRegistration = ApiRegistration {
        canonical_path: "sand::topic::Thing",
        aliases: &["sand::prelude::Thing"],
        canonical_module: "sand::topic",
        kind: ApiKind::Struct,
        signature: "pub struct Thing",
        summary: "Names a typed fixture value.",
        context: "The fixture exposes an aliased public type.",
        minecraft: "Represents a checked Minecraft fixture value.",
        use_when: &["Testing associated aliases"],
        avoid_when: &["Authoring a production value"],
        parameters: &[],
        returns: None,
        example: "let value = Thing;",
        availability: &[],
    };

    static ASSOCIATED_ITEM: ApiRegistration = ApiRegistration {
        canonical_path: "sand::topic::Thing::build",
        aliases: &[],
        canonical_module: "sand::topic",
        kind: ApiKind::Method,
        signature: "pub fn build() -> Thing",
        summary: "Builds a typed fixture value.",
        context: "The fixture intentionally omits repeated aliases.",
        minecraft: "Creates the checked Minecraft fixture value.",
        use_when: &["Testing associated aliases"],
        avoid_when: &["Authoring a production value"],
        parameters: &[],
        returns: Some("A fixture value."),
        example: "Thing::build()",
        availability: &[],
    };

    #[test]
    fn export_is_deterministic() {
        let first =
            ApiCatalog::from_registrations("0.1.0", configuration(1), [&REGISTRATION]).unwrap();
        let second =
            ApiCatalog::from_registrations("0.1.0", configuration(1), [&REGISTRATION]).unwrap();
        assert_eq!(
            first.to_json_pretty().unwrap(),
            second.to_json_pretty().unwrap()
        );
    }

    #[test]
    fn owned_entries_use_the_same_deterministic_validation() {
        let mut entry = ApiEntry::from(&REGISTRATION);
        entry.aliases = vec![
            "sand::prelude::EquipmentPredicate::slot".into(),
            "sand::prelude::EquipmentPredicate::slot".into(),
        ];
        entry.availability = vec!["z".into(), "a".into(), "a".into()];
        let catalog = ApiCatalog::from_entries_with_coverage(
            "0.1.0",
            configuration(1),
            vec![entry],
            ApiCoverage::unverified(),
        )
        .unwrap();
        assert_eq!(
            catalog.entries[0].aliases,
            ["sand::prelude::EquipmentPredicate::slot"]
        );
        assert_eq!(catalog.entries[0].availability, ["a", "z"]);
    }

    #[test]
    fn entry_validation_rejects_invalid_ownership_and_empty_structured_fields() {
        let mut entry = ApiEntry::from(&REGISTRATION);
        entry.canonical_module = "sand::wrong".into();
        assert!(
            entry
                .validate()
                .unwrap_err()
                .to_string()
                .contains("is not a valid owner")
        );

        let mut entry = ApiEntry::from(&REGISTRATION);
        entry.parameters[0].description.clear();
        entry.context.clear();
        entry.minecraft.clear();
        entry.use_when.clear();
        entry.avoid_when.clear();
        entry.validate().unwrap();
        entry.parameters[0].name.clear();
        assert!(
            entry
                .validate()
                .unwrap_err()
                .to_string()
                .contains("parameters require nonempty")
        );

        let mut entry = ApiEntry::from(&REGISTRATION);
        entry.availability = vec![" ".into()];
        assert!(
            entry
                .validate()
                .unwrap_err()
                .to_string()
                .contains("availability values cannot be empty")
        );
    }

    #[test]
    fn resolved_quality_rejects_family_placeholders_and_import_only_callables() {
        let mut placeholder = ApiEntry::from(&REGISTRATION);
        placeholder.signature = "author-facing entity API".into();
        let catalog = ApiCatalog::from_entries_with_coverage(
            "0.1.0",
            configuration(1),
            vec![placeholder],
            ApiCoverage::unverified(),
        )
        .unwrap();
        assert!(
            catalog
                .validate_quality()
                .unwrap_err()
                .to_string()
                .contains("placeholder")
        );

        let mut import_only = ApiEntry::from(&REGISTRATION);
        import_only.example = "use sand::prelude::*;".into();
        let catalog = ApiCatalog::from_entries_with_coverage(
            "0.1.0",
            configuration(1),
            vec![import_only],
            ApiCoverage::unverified(),
        )
        .unwrap();
        assert!(
            catalog
                .validate_quality()
                .unwrap_err()
                .to_string()
                .contains("behavioral example")
        );

        for (summary, parameter, returns, expected) in [
            (
                "Builds or resolves value.",
                "The selected equipment slot.",
                "A predicate builder.",
                "generic family filler",
            ),
            (
                "Selects an equipment slot.",
                "Rust parameter with type `EquipmentSlot`.",
                "A predicate builder.",
                "only repeats its Rust type",
            ),
            (
                "Creates this typed datapack component definition.",
                "The selected equipment slot.",
                "A predicate builder.",
                "generic family filler",
            ),
            (
                "Creates widget.",
                "The selected equipment slot.",
                "A predicate builder.",
                "generic family filler",
            ),
            (
                "Configures to json for this typed resource-pack definition.",
                "The selected equipment slot.",
                "A predicate builder.",
                "generic family filler",
            ),
            (
                "Selects an equipment slot.",
                "The selected equipment slot.",
                "A value with Rust type `Predicate`.",
                "only repeats its Rust type",
            ),
        ] {
            let mut filler = ApiEntry::from(&REGISTRATION);
            filler.summary = summary.into();
            filler.parameters[0].description = parameter.into();
            filler.returns = Some(returns.into());
            let catalog = ApiCatalog::from_entries_with_coverage(
                "0.1.0",
                configuration(1),
                vec![filler],
                ApiCoverage::unverified(),
            )
            .unwrap();
            assert!(
                catalog
                    .validate_quality()
                    .unwrap_err()
                    .to_string()
                    .contains(expected)
            );
        }
    }

    #[test]
    fn aliases_resolve_and_search_is_stable() {
        let catalog =
            ApiCatalog::from_registrations("0.1.0", configuration(1), [&REGISTRATION]).unwrap();
        assert_eq!(
            catalog
                .find("sand::prelude::EquipmentPredicate::slot")
                .unwrap()
                .canonical_path,
            REGISTRATION.canonical_path
        );
        assert_eq!(
            catalog.search("equipment")[0].canonical_path,
            REGISTRATION.canonical_path
        );
        assert_eq!(
            catalog.search("equipment inspect")[0].canonical_path,
            REGISTRATION.canonical_path
        );
        assert_eq!(
            catalog.search("equipment missing")[0].canonical_path,
            REGISTRATION.canonical_path
        );
        assert!(catalog.search("!!!").is_empty());
        assert_eq!(
            catalog.search("!!! equipment ???")[0].canonical_path,
            REGISTRATION.canonical_path
        );
    }

    #[test]
    fn associated_items_inherit_aliased_type_lookup_paths() {
        let catalog = ApiCatalog::from_registrations(
            "0.1.0",
            configuration(2),
            [&ALIASED_TYPE, &ASSOCIATED_ITEM],
        )
        .unwrap();
        let method = catalog.find("sand::prelude::Thing::build").unwrap();
        assert_eq!(method.canonical_path, "sand::topic::Thing::build");
        assert_eq!(method.aliases, ["sand::prelude::Thing::build"]);
    }

    #[test]
    fn coverage_is_sorted_and_complete_status_is_validated() {
        let coverage = ApiCoverage {
            status: CoverageStatus::Partial,
            static_surface_items: 200,
            pending_item_ceiling: 195,
            pending_scope_ceiling: 2,
            pending_scopes: vec!["predicate-source".into(), "command-source".into()],
        };
        let catalog = ApiCatalog::from_registrations_with_coverage(
            "0.1.0",
            configuration(1),
            [&REGISTRATION],
            coverage,
        )
        .unwrap();
        assert_eq!(
            catalog.coverage.pending_scopes,
            ["command-source", "predicate-source"]
        );
        assert!(
            catalog
                .to_json_pretty()
                .unwrap()
                .contains("\"status\": \"partial\"")
        );

        let error = ApiCatalog::from_registrations_with_coverage(
            "0.1.0",
            configuration(1),
            [&REGISTRATION],
            ApiCoverage {
                status: CoverageStatus::Complete,
                static_surface_items: 1,
                pending_item_ceiling: 1,
                pending_scope_ceiling: 0,
                pending_scopes: Vec::new(),
            },
        )
        .unwrap_err();
        assert!(matches!(error, CatalogError::InvalidCoverage(_)));
    }

    #[test]
    fn rustdoc_prose_excludes_code_and_preserves_abbreviations_and_issue_references() {
        let documentation = "Short summary.\n\n```rust\nuse sand_core::Hidden;\n```\n\nConvention: prefix with # (e.g. #fake).\n#273). This continues the issue reference.\n\n# API Contract\nignored";
        assert_eq!(
            rustdoc_prose_paragraphs(documentation),
            [
                "Short summary.",
                "Convention: prefix with # (e.g. #fake). #273). This continues the issue reference."
            ]
        );
        assert!(rustdoc_prose_paragraphs("```rust\nlet specialized_name = 1;\n```").is_empty());
        assert!(!rustdoc_has_specific_semantics(
            "```rust\nlet specialized_name = 1;\n```"
        ));
        assert!(!rustdoc_has_specific_semantics(
            "use sand::feature::SpecializedType;"
        ));
        assert!(rustdoc_has_specific_semantics(
            "Copies the entity snapshot into durable command storage."
        ));
        assert_eq!(
            rustdoc_prose_paragraphs(
                "**API Contract:** Run `sand api show sand::data::NbtPath::as_str` for the canonical contract.\nBorrows the rendered NBT path text without allocating."
            ),
            ["Borrows the rendered NBT path text without allocating."]
        );
        assert!(
            rustdoc_prose_paragraphs("# Heading\n\nActual prose.")
                .first()
                .is_some_and(|paragraph| paragraph == "Actual prose.")
        );
    }
}
