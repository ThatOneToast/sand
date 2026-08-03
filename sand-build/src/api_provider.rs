//! Deterministic metadata emitted beside generated Rust APIs.
//!
//! Provider catalogs are generated from the same in-memory declarations as
//! the Rust source. They are therefore suitable input for the facade's public
//! surface audit; parsing generated Rust a second time is neither necessary
//! nor authoritative.

use std::collections::BTreeSet;
use std::path::Path;

use sand_api_contract::{ApiEntry, ApiKind};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Schema version for generator-provider files.
pub const PROVIDER_SCHEMA_VERSION: u32 = 1;

/// A complete deterministic declaration from one API-producing generator.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiProviderCatalog {
    pub schema_version: u32,
    pub provider: String,
    pub minecraft_version: String,
    pub entries: Vec<GeneratedProviderEntry>,
}

/// One generated declaration and the contract for its facade identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GeneratedProviderEntry {
    /// Identity in the implementation graph, before facade re-exports.
    pub definition_identity: String,
    /// Rust item kind independently declared for reachability extraction.
    pub definition_kind: ApiKind,
    /// Owning generated type for a method, field, or variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_identity: Option<String>,
    /// Member name relative to `parent_identity`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_name: Option<String>,
    pub contract: ApiEntry,
}

impl ApiProviderCatalog {
    pub(crate) fn new(
        provider: impl Into<String>,
        minecraft_version: impl Into<String>,
        mut entries: Vec<GeneratedProviderEntry>,
    ) -> Self {
        for entry in &mut entries {
            entry.contract.aliases.sort();
            entry.contract.aliases.dedup();
            entry.contract.availability.sort();
            entry.contract.availability.dedup();
        }
        entries.sort_by(|left, right| {
            left.contract
                .canonical_path
                .cmp(&right.contract.canonical_path)
                .then_with(|| left.definition_identity.cmp(&right.definition_identity))
        });
        Self {
            schema_version: PROVIDER_SCHEMA_VERSION,
            provider: provider.into(),
            minecraft_version: minecraft_version.into(),
            entries,
        }
    }

    /// Validate identity uniqueness before a provider can be consumed.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.schema_version != PROVIDER_SCHEMA_VERSION {
            return Err(format!(
                "unsupported provider schema {}, expected {PROVIDER_SCHEMA_VERSION}",
                self.schema_version
            ));
        }
        let mut paths = BTreeSet::new();
        let mut identities = BTreeSet::new();
        let mut previous_path: Option<&str> = None;
        for entry in &self.entries {
            if previous_path
                .is_some_and(|previous| previous > entry.contract.canonical_path.as_str())
            {
                return Err(format!(
                    "provider `{}` entries are not sorted",
                    self.provider
                ));
            }
            previous_path = Some(&entry.contract.canonical_path);
            if entry.definition_kind != entry.contract.kind {
                return Err(format!(
                    "provider `{}` declaration `{}` kind differs from its contract",
                    self.provider, entry.definition_identity
                ));
            }
            if entry.parent_identity.is_some() != entry.member_name.is_some() {
                return Err(format!(
                    "provider `{}` declaration `{}` has incomplete member ownership",
                    self.provider, entry.definition_identity
                ));
            }
            if let (Some(parent), Some(member)) = (&entry.parent_identity, &entry.member_name)
                && entry.definition_identity != format!("{parent}::{member}")
            {
                return Err(format!(
                    "provider `{}` declaration `{}` is not member `{member}` of `{parent}`",
                    self.provider, entry.definition_identity
                ));
            }
            if !entry.contract.canonical_path.starts_with("sand::")
                || entry.contract.summary.trim().is_empty()
                || entry.contract.context.trim().is_empty()
                || entry.contract.minecraft.trim().is_empty()
                || entry.contract.use_when.is_empty()
                || entry.contract.avoid_when.is_empty()
                || entry.contract.example.trim().is_empty()
            {
                return Err(format!(
                    "provider `{}` declaration `{}` has an incomplete public contract",
                    self.provider, entry.definition_identity
                ));
            }
            if !entry
                .contract
                .aliases
                .windows(2)
                .all(|pair| pair[0] < pair[1])
                || !entry
                    .contract
                    .availability
                    .windows(2)
                    .all(|pair| pair[0] < pair[1])
            {
                return Err(format!(
                    "provider `{}` declaration `{}` has nondeterministic set ordering",
                    self.provider, entry.definition_identity
                ));
            }
            if !identities.insert(entry.definition_identity.as_str()) {
                return Err(format!(
                    "provider `{}` emitted duplicate implementation identity `{}`",
                    self.provider, entry.definition_identity
                ));
            }
            if !paths.insert(entry.contract.canonical_path.as_str()) {
                return Err(format!(
                    "provider `{}` emitted duplicate canonical API `{}`",
                    self.provider, entry.contract.canonical_path
                ));
            }
            for alias in &entry.contract.aliases {
                if !paths.insert(alias.as_str()) {
                    return Err(format!(
                        "provider `{}` emitted duplicate lookup API `{alias}`",
                        self.provider
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn write_json(&self, path: &Path) -> Result<()> {
        self.validate().map_err(std::io::Error::other)?;
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        std::fs::write(path, bytes)?;
        Ok(())
    }
}

/// Read and validate a generated provider catalog.
pub fn read_api_provider(path: &Path) -> Result<ApiProviderCatalog> {
    let catalog: ApiProviderCatalog = serde_json::from_slice(&std::fs::read(path)?)?;
    catalog.validate().map_err(std::io::Error::other)?;
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    use sand_api_contract::ApiKind;

    use super::*;

    fn entry(identity: &str, path: &str) -> GeneratedProviderEntry {
        GeneratedProviderEntry {
            definition_identity: identity.into(),
            definition_kind: ApiKind::Struct,
            parent_identity: None,
            member_name: None,
            contract: ApiEntry {
                canonical_path: path.into(),
                aliases: Vec::new(),
                canonical_module: "sand::generated".into(),
                kind: ApiKind::Struct,
                signature: "pub struct Example".into(),
                summary: "Represents an exact generated Minecraft declaration.".into(),
                context: "Generated from the selected Minecraft data report.".into(),
                minecraft: "Maps to the corresponding Minecraft declaration.".into(),
                use_when: vec!["Using the generated declaration".into()],
                avoid_when: vec!["Using custom content".into()],
                parameters: Vec::new(),
                returns: None,
                example: "let value = Example;".into(),
                availability: vec!["minecraft = test".into()],
            },
        }
    }

    #[test]
    fn provider_rejects_duplicate_definition_identity() {
        let catalog = ApiProviderCatalog::new(
            "fixture",
            "test",
            vec![
                entry("core::Generated", "sand::generated::First"),
                entry("core::Generated", "sand::generated::Second"),
            ],
        );
        assert!(
            catalog
                .validate()
                .unwrap_err()
                .contains("duplicate implementation identity")
        );
    }

    #[test]
    fn provider_rejects_member_owner_drift() {
        let mut member = entry(
            "core::Generated::method",
            "sand::generated::Generated::method",
        );
        member.definition_kind = ApiKind::Method;
        member.contract.kind = ApiKind::Method;
        member.parent_identity = Some("core::Other".into());
        member.member_name = Some("method".into());
        let catalog = ApiProviderCatalog::new("fixture", "test", vec![member]);
        assert!(catalog.validate().unwrap_err().contains("is not member"));
    }
}
