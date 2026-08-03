//! Ratcheted rollout of complete reachable-surface enforcement.
//!
//! Scopes are module-sized migration units. An enforced scope admits no
//! per-item exemptions: every reachable identity in it must have a contract.
//! Pending scopes remain visible in the deterministic report and count toward
//! a committed ceiling, preventing silent enforced-to-pending regressions.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::{ContractIdentity, ReachableApi, ReachableOrigin};

const SCOPE_SCHEMA_VERSION: u32 = 1;

/// Machine-readable rollout manifest loaded by a facade build script.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeManifest {
    pub schema_version: u32,
    /// Maximum number of reachable items allowed to remain in active pending
    /// scopes. Lower this when a scope becomes enforced.
    pub pending_item_ceiling: usize,
    #[serde(rename = "scope")]
    pub scopes: Vec<ApiScope>,
}

/// One intentional canonical module, optionally owned by a code generator.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiScope {
    /// Stable review-facing name for this migration unit.
    pub id: String,
    pub canonical_module: String,
    pub state: ScopeState,
    pub tier: String,
    /// `source` or `generator:<provider>`. Generated and handwritten APIs can
    /// therefore be ratcheted separately within the same facade module.
    pub provider: String,
    /// Whether this scope owns descendants or only direct children.
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeState {
    Pending,
    Enforced,
}

/// Deterministically sorted status for one configured scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeReportEntry {
    pub id: String,
    pub canonical_module: String,
    pub state: ScopeState,
    pub tier: String,
    pub aliases: Vec<String>,
    pub features: Vec<String>,
    pub provider: String,
    pub recursive: bool,
    pub active: bool,
    pub reachable_items: usize,
    pub contracted_items: usize,
}

/// Complete deterministic rollout status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeReport {
    pub entries: Vec<ScopeReportEntry>,
    pub pending_items: usize,
    pub enforced_items: usize,
    pub pending_item_ceiling: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeFailure {
    Io(String),
    Toml(String),
    UnsupportedSchema(u32),
    InvalidCanonicalModule(String),
    InvalidScopeId(String),
    DuplicateScopeId(String),
    DuplicateCanonicalScope(String),
    OverlappingScopes {
        left: String,
        right: String,
    },
    InvalidAlias {
        scope: String,
        alias: String,
    },
    AliasOverlapsScope {
        alias: String,
        scope: String,
    },
    DuplicateAlias(String),
    InvalidTier(String),
    InvalidFeature {
        scope: String,
        feature: String,
    },
    InvalidProvider {
        scope: String,
        provider: String,
    },
    UnscopedItems(Vec<String>),
    AmbiguousScope {
        identity: String,
        scopes: Vec<String>,
    },
    MissingContracts {
        scope: String,
        identities: Vec<String>,
    },
    PendingCeilingExceeded {
        actual: usize,
        ceiling: usize,
    },
}

impl fmt::Display for ScopeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "failed to read API scope manifest: {message}"),
            Self::Toml(message) => write!(formatter, "invalid API scope TOML: {message}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported API scope schema version {version}")
            }
            Self::InvalidCanonicalModule(path) => {
                write!(formatter, "invalid canonical API scope `{path}`")
            }
            Self::InvalidScopeId(id) => write!(formatter, "invalid API scope id `{id}`"),
            Self::DuplicateScopeId(id) => write!(formatter, "duplicate API scope id `{id}`"),
            Self::DuplicateCanonicalScope(path) => {
                write!(formatter, "duplicate canonical API scope `{path}`")
            }
            Self::OverlappingScopes { left, right } => {
                write!(
                    formatter,
                    "canonical API scopes `{left}` and `{right}` overlap"
                )
            }
            Self::InvalidAlias { scope, alias } => {
                write!(formatter, "scope `{scope}` has invalid alias `{alias}`")
            }
            Self::AliasOverlapsScope { alias, scope } => {
                write!(formatter, "API scope alias `{alias}` overlaps `{scope}`")
            }
            Self::DuplicateAlias(alias) => write!(formatter, "duplicate API scope alias `{alias}`"),
            Self::InvalidTier(tier) => {
                write!(formatter, "invalid empty API stability tier `{tier}`")
            }
            Self::InvalidFeature { scope, feature } => {
                write!(formatter, "scope `{scope}` has invalid feature `{feature}`")
            }
            Self::InvalidProvider { scope, provider } => {
                write!(
                    formatter,
                    "scope `{scope}` has invalid provider `{provider}`"
                )
            }
            Self::UnscopedItems(identities) => write!(
                formatter,
                "reachable APIs are not assigned to a contract scope: {}",
                identities.join(", ")
            ),
            Self::AmbiguousScope { identity, scopes } => write!(
                formatter,
                "reachable API `{identity}` belongs to multiple scopes: {}",
                scopes.join(", ")
            ),
            Self::MissingContracts { scope, identities } => write!(
                formatter,
                "enforced API scope `{scope}` has missing contracts: {}",
                identities.join(", ")
            ),
            Self::PendingCeilingExceeded { actual, ceiling } => write!(
                formatter,
                "pending API surface grew to {actual} items, above committed ceiling {ceiling}"
            ),
        }
    }
}

impl std::error::Error for ScopeFailure {}

impl ScopeManifest {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ScopeFailure> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|error| ScopeFailure::Io(format!("{}: {error}", path.display())))?;
        Self::from_toml(&source)
    }

    pub fn from_toml(source: &str) -> Result<Self, ScopeFailure> {
        let manifest = toml::from_str::<Self>(source)
            .map_err(|error| ScopeFailure::Toml(error.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Evaluate only active scopes for the selected feature set. Items outside
    /// configured scopes are intentionally untouched during incremental rollout.
    pub fn evaluate(
        &self,
        reachable: &[ReachableApi],
        contracts: &[ContractIdentity],
        enabled_features: &BTreeSet<String>,
    ) -> Result<ScopeReport, Vec<ScopeFailure>> {
        if let Err(error) = self.validate() {
            return Err(vec![error]);
        }
        let contracted = contracts
            .iter()
            .map(|contract| contract.identity.as_str())
            .collect::<BTreeSet<_>>();
        let mut entries = Vec::with_capacity(self.scopes.len());
        let mut failures = Vec::new();
        let mut pending_items = 0;
        let mut enforced_items = 0;

        let mut unscoped = Vec::new();
        for item in reachable {
            let owners = self
                .scopes
                .iter()
                .filter(|scope| scope.matches(item))
                .collect::<Vec<_>>();
            match owners.as_slice() {
                [] => unscoped.push(item.identity.clone()),
                [_] => {}
                _ => failures.push(ScopeFailure::AmbiguousScope {
                    identity: item.identity.clone(),
                    scopes: owners.iter().map(|scope| scope.id.clone()).collect(),
                }),
            }
        }
        unscoped.sort();
        if !unscoped.is_empty() {
            failures.push(ScopeFailure::UnscopedItems(unscoped));
        }

        for scope in &self.scopes {
            let active = scope
                .features
                .iter()
                .all(|feature| enabled_features.contains(feature));
            let mut matched = if active {
                reachable
                    .iter()
                    .filter(|item| scope.matches(item))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            matched.sort_by(|left, right| left.identity.cmp(&right.identity));
            let contracted_items = matched
                .iter()
                .filter(|item| contracted.contains(item.identity.as_str()))
                .count();
            match scope.state {
                ScopeState::Pending => pending_items += matched.len(),
                ScopeState::Enforced => {
                    enforced_items += matched.len();
                    let missing = matched
                        .iter()
                        .filter(|item| !contracted.contains(item.identity.as_str()))
                        .map(|item| item.identity.clone())
                        .collect::<Vec<_>>();
                    if !missing.is_empty() {
                        failures.push(ScopeFailure::MissingContracts {
                            scope: scope.canonical_module.clone(),
                            identities: missing,
                        });
                    }
                }
            }
            let mut aliases = scope.aliases.clone();
            aliases.sort();
            let mut features = scope.features.clone();
            features.sort();
            entries.push(ScopeReportEntry {
                id: scope.id.clone(),
                canonical_module: scope.canonical_module.clone(),
                state: scope.state,
                tier: scope.tier.clone(),
                aliases,
                features,
                provider: scope.provider.clone(),
                recursive: scope.recursive,
                active,
                reachable_items: matched.len(),
                contracted_items,
            });
        }
        entries.sort_by(|left, right| left.id.cmp(&right.id));
        if pending_items > self.pending_item_ceiling {
            failures.push(ScopeFailure::PendingCeilingExceeded {
                actual: pending_items,
                ceiling: self.pending_item_ceiling,
            });
        }
        if failures.is_empty() {
            Ok(ScopeReport {
                entries,
                pending_items,
                enforced_items,
                pending_item_ceiling: self.pending_item_ceiling,
            })
        } else {
            Err(failures)
        }
    }

    fn validate(&self) -> Result<(), ScopeFailure> {
        if self.schema_version != SCOPE_SCHEMA_VERSION {
            return Err(ScopeFailure::UnsupportedSchema(self.schema_version));
        }
        let mut ids = BTreeSet::new();
        for scope in &self.scopes {
            if !valid_label(&scope.id) {
                return Err(ScopeFailure::InvalidScopeId(scope.id.clone()));
            }
            if !ids.insert(scope.id.as_str()) {
                return Err(ScopeFailure::DuplicateScopeId(scope.id.clone()));
            }
            if !valid_api_path(&scope.canonical_module) {
                return Err(ScopeFailure::InvalidCanonicalModule(
                    scope.canonical_module.clone(),
                ));
            }
            if scope.tier.trim().is_empty() {
                return Err(ScopeFailure::InvalidTier(scope.tier.clone()));
            }
            for feature in &scope.features {
                if !valid_label(feature) {
                    return Err(ScopeFailure::InvalidFeature {
                        scope: scope.canonical_module.clone(),
                        feature: feature.clone(),
                    });
                }
            }
            if !valid_provider(&scope.provider) {
                return Err(ScopeFailure::InvalidProvider {
                    scope: scope.canonical_module.clone(),
                    provider: scope.provider.clone(),
                });
            }
        }
        for (index, left) in self.scopes.iter().enumerate() {
            for right in self.scopes.iter().skip(index + 1) {
                if scope_selectors_overlap(left, right) {
                    if left.canonical_module == right.canonical_module {
                        return Err(ScopeFailure::DuplicateCanonicalScope(
                            left.canonical_module.clone(),
                        ));
                    }
                    return Err(ScopeFailure::OverlappingScopes {
                        left: left.canonical_module.clone(),
                        right: right.canonical_module.clone(),
                    });
                }
            }
        }
        let paths = self
            .scopes
            .iter()
            .map(|scope| scope.canonical_module.as_str())
            .collect::<Vec<_>>();
        let mut aliases = BTreeMap::<&str, BTreeSet<&str>>::new();
        for scope in &self.scopes {
            let mut scope_aliases = BTreeSet::new();
            for alias in &scope.aliases {
                if !valid_api_path(alias) || alias == &scope.canonical_module {
                    return Err(ScopeFailure::InvalidAlias {
                        scope: scope.canonical_module.clone(),
                        alias: alias.clone(),
                    });
                }
                if !scope_aliases.insert(alias.as_str()) {
                    return Err(ScopeFailure::DuplicateAlias(alias.clone()));
                }
                aliases
                    .entry(alias)
                    .or_default()
                    .insert(&scope.canonical_module);
                for path in &paths {
                    if alias == path {
                        return Err(ScopeFailure::AliasOverlapsScope {
                            alias: alias.clone(),
                            scope: (*path).to_owned(),
                        });
                    }
                }
            }
        }
        let aliases = aliases.keys().copied().collect::<Vec<_>>();
        for (index, left) in aliases.iter().enumerate() {
            for right in aliases.iter().skip(index + 1) {
                if overlaps(left, right) {
                    return Err(ScopeFailure::DuplicateAlias((*right).to_owned()));
                }
            }
        }
        Ok(())
    }
}

impl ApiScope {
    fn matches(&self, item: &ReachableApi) -> bool {
        provider_matches(&self.provider, &item.origin)
            && item.paths.iter().any(|path| {
                if self.recursive {
                    within(path, &self.canonical_module)
                } else {
                    direct_child(path, &self.canonical_module)
                }
            })
    }
}

impl fmt::Display for ScopeReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            let state = match entry.state {
                ScopeState::Pending => "pending",
                ScopeState::Enforced => "enforced",
            };
            let aliases = joined_or_dash(&entry.aliases);
            let features = joined_or_dash(&entry.features);
            writeln!(
                formatter,
                "{} module={} state={} tier={} provider={} recursive={} active={} items={} contracted={} aliases={} features={}",
                entry.id,
                entry.canonical_module,
                state,
                entry.tier,
                entry.provider,
                entry.recursive,
                entry.active,
                entry.reachable_items,
                entry.contracted_items,
                aliases,
                features
            )?;
        }
        write!(
            formatter,
            "totals pending={} enforced={} pending_ceiling={}",
            self.pending_items, self.enforced_items, self.pending_item_ceiling
        )
    }
}

fn joined_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_owned()
    } else {
        values.join(",")
    }
}

fn valid_api_path(path: &str) -> bool {
    let mut segments = path.split("::");
    matches!(segments.next(), Some("sand")) && segments.all(valid_api_segment)
}

fn default_recursive() -> bool {
    true
}

fn valid_provider(provider: &str) -> bool {
    provider == "source" || provider.strip_prefix("generator:").is_some_and(valid_label)
}

fn provider_matches(provider: &str, origin: &ReachableOrigin) -> bool {
    match (provider, origin) {
        ("source", ReachableOrigin::Source) => true,
        (provider, ReachableOrigin::Generator(generator)) => {
            provider.strip_prefix("generator:") == Some(generator.as_str())
        }
        _ => false,
    }
}

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character == '-'
                || character.is_ascii_alphanumeric() && (index > 0 || !character.is_ascii_digit())
        })
}

fn valid_api_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character.is_ascii_alphanumeric() && (index > 0 || !character.is_ascii_digit())
        })
}

fn within(path: &str, scope: &str) -> bool {
    path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|rest| rest.starts_with("::"))
}

fn direct_child(path: &str, scope: &str) -> bool {
    path.strip_prefix(scope)
        .and_then(|rest| rest.strip_prefix("::"))
        .is_some_and(|rest| !rest.is_empty() && !rest.contains("::"))
}

fn overlaps(left: &str, right: &str) -> bool {
    within(left, right) || within(right, left)
}

fn scope_selectors_overlap(left: &ApiScope, right: &ApiScope) -> bool {
    if left.provider != right.provider {
        return false;
    }
    match (left.recursive, right.recursive) {
        (true, true) => overlaps(&left.canonical_module, &right.canonical_module),
        (true, false) => within(&right.canonical_module, &left.canonical_module),
        (false, true) => within(&left.canonical_module, &right.canonical_module),
        (false, false) => left.canonical_module == right.canonical_module,
    }
}
