//! Deterministic metadata emitted beside generated Rust APIs.
//!
//! Provider catalogs are generated from the same in-memory declarations as
//! the Rust source. At the consuming facade boundary, Sand also parses the
//! emitted Rust into a structural identity set and requires byte-independent
//! set equality with the provider. The provider remains authoritative for
//! contracts; the structural projection proves it describes every emitted
//! public declaration and no nonexistent one.

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

/// Prove that a provider is an exact structural description of the public
/// declarations in the generated Rust file included beneath `root_identity`.
///
/// Provider metadata and Rust are deliberately parsed as separate artifacts
/// here. Even though generators normally emit both from the same declaration
/// model, this consumer-side comparison prevents a later write, formatter, or
/// generator bug from adding a reachable public item that the provider does
/// not report (or reporting an item that Rust does not actually contain).
pub fn validate_api_provider_source(
    catalog: &ApiProviderCatalog,
    rust_path: &Path,
    root_identity: &str,
) -> std::result::Result<(), String> {
    catalog.validate()?;
    let source = std::fs::read_to_string(rust_path)
        .map_err(|error| format!("failed to read {}: {error}", rust_path.display()))?;
    let syntax = syn::parse_file(&source)
        .map_err(|error| format!("failed to parse {}: {error}", rust_path.display()))?;
    let actual = public_source_declarations(&syntax, root_identity)?;
    let expected = catalog
        .entries
        .iter()
        .map(|entry| (entry.definition_identity.clone(), entry.definition_kind))
        .collect::<BTreeSet<_>>();
    if actual == expected {
        return Ok(());
    }

    let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
    let unreported = actual.difference(&expected).cloned().collect::<Vec<_>>();
    Err(format!(
        "provider `{}` does not exactly match generated Rust {} beneath `{root_identity}`; missing from Rust: {}; public but unreported: {}",
        catalog.provider,
        rust_path.display(),
        format_declarations(&missing),
        format_declarations(&unreported),
    ))
}

fn public_source_declarations(
    syntax: &syn::File,
    root: &str,
) -> std::result::Result<BTreeSet<(String, ApiKind)>, String> {
    let mut declarations = BTreeSet::new();
    collect_public_source_declarations(&syntax.items, root, &mut declarations)?;
    Ok(declarations)
}

fn collect_public_source_declarations(
    items: &[syn::Item],
    root: &str,
    declarations: &mut BTreeSet<(String, ApiKind)>,
) -> std::result::Result<(), String> {
    for item in items {
        match item {
            syn::Item::Mod(item) if public(&item.vis) => {
                let identity = format!("{root}::{}", item.ident);
                declarations.insert((identity.clone(), ApiKind::Module));
                let Some((_, items)) = &item.content else {
                    return Err(format!(
                        "generated public module `{identity}` is out-of-line and cannot be structurally verified"
                    ));
                };
                collect_public_source_declarations(items, &identity, declarations)?;
            }
            syn::Item::Struct(item) if public(&item.vis) => {
                let owner = format!("{root}::{}", item.ident);
                declarations.insert((owner.clone(), ApiKind::Struct));
                for (index, field) in item.fields.iter().enumerate() {
                    if public(&field.vis) {
                        let name = field
                            .ident
                            .as_ref()
                            .map_or_else(|| index.to_string(), ToString::to_string);
                        declarations.insert((format!("{owner}::{name}"), ApiKind::Field));
                    }
                }
            }
            syn::Item::Enum(item) if public(&item.vis) => {
                let owner = format!("{root}::{}", item.ident);
                declarations.insert((owner.clone(), ApiKind::Enum));
                for variant in &item.variants {
                    let variant_owner = format!("{owner}::{}", variant.ident);
                    declarations.insert((variant_owner.clone(), ApiKind::Variant));
                    for (index, field) in variant.fields.iter().enumerate() {
                        let name = field
                            .ident
                            .as_ref()
                            .map_or_else(|| index.to_string(), ToString::to_string);
                        declarations.insert((format!("{variant_owner}::{name}"), ApiKind::Field));
                    }
                }
            }
            syn::Item::Trait(item) if public(&item.vis) => {
                let owner = format!("{root}::{}", item.ident);
                declarations.insert((owner.clone(), ApiKind::Trait));
                for member in &item.items {
                    let (name, kind) = match member {
                        syn::TraitItem::Fn(item) => (&item.sig.ident, ApiKind::TraitMethod),
                        syn::TraitItem::Const(item) => (&item.ident, ApiKind::AssociatedConst),
                        syn::TraitItem::Type(item) => (&item.ident, ApiKind::AssociatedType),
                        _ => continue,
                    };
                    declarations.insert((format!("{owner}::{name}"), kind));
                }
            }
            syn::Item::Fn(item) if public(&item.vis) => {
                declarations.insert((format!("{root}::{}", item.sig.ident), ApiKind::Function));
            }
            syn::Item::Type(item) if public(&item.vis) => {
                declarations.insert((format!("{root}::{}", item.ident), ApiKind::TypeAlias));
            }
            syn::Item::Const(item) if public(&item.vis) => {
                declarations.insert((format!("{root}::{}", item.ident), ApiKind::Constant));
            }
            syn::Item::Static(item) if public(&item.vis) => {
                declarations.insert((format!("{root}::{}", item.ident), ApiKind::Constant));
            }
            syn::Item::Impl(item) if item.trait_.is_none() => {
                let syn::Type::Path(owner) = item.self_ty.as_ref() else {
                    continue;
                };
                let Some(owner) = owner.path.segments.last() else {
                    continue;
                };
                let owner = format!("{root}::{}", owner.ident);
                for member in &item.items {
                    let (visibility, name, kind) = match member {
                        syn::ImplItem::Fn(item) => (&item.vis, &item.sig.ident, ApiKind::Method),
                        syn::ImplItem::Const(item) => {
                            (&item.vis, &item.ident, ApiKind::AssociatedConst)
                        }
                        syn::ImplItem::Type(item) => {
                            (&item.vis, &item.ident, ApiKind::AssociatedType)
                        }
                        _ => continue,
                    };
                    if public(visibility) {
                        declarations.insert((format!("{owner}::{name}"), kind));
                    }
                }
            }
            syn::Item::Union(item) if public(&item.vis) => {
                return Err(format!(
                    "generated public union `{root}::{}` has no API contract kind",
                    item.ident
                ));
            }
            syn::Item::Macro(item)
                if item
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("macro_export")) =>
            {
                let Some(name) = &item.ident else {
                    return Err("generated #[macro_export] declaration has no name".into());
                };
                declarations.insert((format!("{root}::{name}"), ApiKind::Macro));
            }
            syn::Item::Use(item) if public(&item.vis) => {
                return Err(format!(
                    "generated public re-export beneath `{root}` cannot be structurally verified by declaration parity"
                ));
            }
            syn::Item::ExternCrate(item) if public(&item.vis) => {
                return Err(format!(
                    "generated public extern crate `{}` beneath `{root}` cannot be structurally verified by declaration parity",
                    item.ident
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn format_declarations(declarations: &[(String, ApiKind)]) -> String {
    if declarations.is_empty() {
        "none".into()
    } else {
        declarations
            .iter()
            .map(|(identity, kind)| format!("{identity} ({kind:?})"))
            .collect::<Vec<_>>()
            .join(", ")
    }
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

    #[test]
    fn generated_source_must_exactly_match_provider_structure() {
        let directory = tempfile::tempdir().unwrap();
        let rust = directory.path().join("generated.rs");
        std::fs::write(&rust, "pub struct Generated;\n").unwrap();
        let catalog = ApiProviderCatalog::new(
            "fixture",
            "test",
            vec![entry(
                "core::generated::Generated",
                "sand::generated::Generated",
            )],
        );
        validate_api_provider_source(&catalog, &rust, "core::generated").unwrap();

        // This simulates an opaque generator (or a post-generation mutation)
        // emitting an extra reachable item without changing its provider.
        std::fs::write(&rust, "pub struct Generated;\npub fn bypass() {}\n").unwrap();
        let error = validate_api_provider_source(&catalog, &rust, "core::generated")
            .expect_err("unreported public declaration must fail closed");
        assert!(error.contains("public but unreported"), "{error}");
        assert!(
            error.contains("core::generated::bypass (Function)"),
            "{error}"
        );

        std::fs::write(
            &rust,
            "pub struct Generated;\npub mod hidden_bypass { pub struct Extra; }\n",
        )
        .unwrap();
        let error = validate_api_provider_source(&catalog, &rust, "core::generated")
            .expect_err("unreported public module contents must fail closed");
        assert!(
            error.contains("core::generated::hidden_bypass (Module)"),
            "{error}"
        );
        assert!(
            error.contains("core::generated::hidden_bypass::Extra (Struct)"),
            "{error}"
        );
    }
}
