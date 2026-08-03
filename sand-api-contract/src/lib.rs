#![forbid(unsafe_code)]

//! The versioned, serializable model behind Sand's public API contracts.
//!
//! Author-facing items register [`ApiRegistration`] values at compile time.
//! [`ApiCatalog::installed`] converts those static records into an owned,
//! validated, deterministically ordered catalog suitable for the CLI and
//! external tooling.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Current machine-readable catalog schema.
pub const SCHEMA_VERSION: u32 = 1;

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
    Macro,
}

/// Parameter documentation in declaration order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiParameter {
    pub name: String,
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
    pub context: String,
    pub minecraft: String,
    pub use_when: Vec<String>,
    pub avoid_when: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ApiParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    pub example: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub availability: Vec<String>,
}

/// Complete metadata export for one installed Sand version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiCatalog {
    pub schema_version: u32,
    pub sand_version: String,
    pub entries: Vec<ApiEntry>,
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
        }
    }
}

impl std::error::Error for CatalogError {}

impl ApiCatalog {
    /// Builds the authoritative catalog linked into the current executable.
    pub fn installed(sand_version: impl Into<String>) -> Result<Self, CatalogError> {
        Self::from_registrations(sand_version, inventory::iter::<ApiRegistration>)
    }

    /// Builds a catalog from explicit registrations, primarily for generated
    /// catalogs and focused tests.
    pub fn from_registrations<'a>(
        sand_version: impl Into<String>,
        registrations: impl IntoIterator<Item = &'a ApiRegistration>,
    ) -> Result<Self, CatalogError> {
        let mut entries = registrations
            .into_iter()
            .map(ApiEntry::from)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));
        validate_entries(&entries)?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            sand_version: sand_version.into(),
            entries,
        })
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
        let words = query.split_whitespace().collect::<Vec<_>>();
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
                    description: parameter.description.to_owned(),
                })
                .collect(),
            returns: value.returns.map(ToOwned::to_owned),
            example: value.example.to_owned(),
            availability,
        }
    }
}

fn validate_entries(entries: &[ApiEntry]) -> Result<(), CatalogError> {
    let mut lookup = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut previous = None;
    for entry in entries {
        if !valid_path(&entry.canonical_path) {
            return Err(CatalogError::InvalidCanonicalPath(
                entry.canonical_path.clone(),
            ));
        }
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

fn valid_path(path: &str) -> bool {
    let mut segments = path.split("::");
    matches!(segments.next(), Some("sand"))
        && segments.clone().next().is_some()
        && segments.all(|segment| {
            !segment.is_empty()
                && segment.chars().enumerate().all(|(index, ch)| {
                    ch == '_' || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit())
                })
        })
}

fn search_score(entry: &ApiEntry, query: &str, words: &[&str]) -> Option<u32> {
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

    words.iter().try_fold(0_u32, |score, word| {
        let word_score = if path.rsplit("::").next() == Some(*word) {
            500
        } else if path.contains(word) {
            300
        } else if aliases.iter().any(|alias| alias.contains(word)) {
            250
        } else if summary.contains(word) {
            120
        } else if descriptive.contains(word) {
            20
        } else {
            return None;
        };
        Some(score + word_score)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn export_is_deterministic() {
        let first = ApiCatalog::from_registrations("0.1.0", [&REGISTRATION]).unwrap();
        let second = ApiCatalog::from_registrations("0.1.0", [&REGISTRATION]).unwrap();
        assert_eq!(
            first.to_json_pretty().unwrap(),
            second.to_json_pretty().unwrap()
        );
    }

    #[test]
    fn aliases_resolve_and_search_is_stable() {
        let catalog = ApiCatalog::from_registrations("0.1.0", [&REGISTRATION]).unwrap();
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
        assert!(catalog.search("equipment missing").is_empty());
    }
}
