//! Contract metadata emitted from semantic `registry_id!` declarations.
//!
//! A registry wrapper opts into a contract partition with `@contract(...)`
//! inside the same invocation that emits its Rust type. The shared generator
//! derives provider shapes from that emitted AST rather than restating them.

use std::path::Path;

use crate::api_provider::{ApiProviderCatalog, GeneratedProviderEntry};
use crate::error::Result;

/// Build installed contract metadata from the same expansion that emits each
/// registry wrapper's Rust type and inherent methods.
pub fn registry_id_contract_provider(
    source: &Path,
    minecraft_version: &str,
) -> Result<ApiProviderCatalog> {
    let text = std::fs::read_to_string(source)?;
    let file = syn::parse_file(&text).map_err(std::io::Error::other)?;
    let mut entries = Vec::new();

    for item in file.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        if !item.mac.path.is_ident("registry_id") {
            continue;
        }
        let expansion = sand_api_contract::syntax::registry_id::expand(item.mac.tokens)
            .map_err(std::io::Error::other)?;
        entries.extend(expansion.definitions.into_iter().map(|definition| {
            GeneratedProviderEntry {
                definition_identity: definition.definition_identity,
                definition_kind: definition.definition_kind,
                parent_identity: definition.parent_identity,
                member_name: definition.member_name,
                contract: definition.contract,
            }
        }));
    }

    Ok(ApiProviderCatalog::new(
        "generated_registry_id_contracts",
        minecraft_version,
        entries,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository_provider() -> ApiProviderCatalog {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        registry_id_contract_provider(
            &workspace.join("sand-components/src/registry.rs"),
            "test-version",
        )
        .unwrap()
    }

    #[test]
    fn predicate_registry_id_emits_four_meaningful_contracts() {
        let provider = repository_provider();
        assert_eq!(provider.provider, "generated_registry_id_contracts");
        assert_eq!(provider.entries.len(), 4);
        assert_eq!(
            provider.entries[0].contract.canonical_module,
            "sand::predicate"
        );
        assert!(
            provider.entries[1..]
                .iter()
                .all(|entry| { entry.contract.canonical_module == "sand::predicate::PredicateId" })
        );
        provider.validate().unwrap();

        let paths = provider
            .entries
            .iter()
            .map(|entry| entry.contract.canonical_path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                "sand::predicate::PredicateId",
                "sand::predicate::PredicateId::as_resource_location",
                "sand::predicate::PredicateId::custom",
                "sand::predicate::PredicateId::minecraft",
            ]
        );
        for entry in &provider.entries {
            assert!(
                entry.contract.summary.contains("predicate resource")
                    && entry.contract.minecraft.contains("predicate")
                    && !entry.contract.use_when.is_empty()
                    && !entry.contract.avoid_when.is_empty()
                    && entry.contract.example.contains("PredicateId"),
                "incomplete predicate contract: {entry:#?}"
            );
        }
        assert_eq!(
            provider.entries[0].contract.aliases,
            ["sand::component::PredicateId", "sand::prelude::PredicateId"]
        );
        let minecraft = provider
            .entries
            .iter()
            .find(|entry| entry.member_name.as_deref() == Some("minecraft"))
            .unwrap();
        assert_eq!(minecraft.contract.kind, sand_api_contract::ApiKind::Method);
        assert!(
            minecraft
                .contract
                .signature
                .contains("path : impl AsRef < str >")
        );
        assert_eq!(minecraft.contract.parameters[0].name, "path");
        let custom = provider
            .entries
            .iter()
            .find(|entry| entry.member_name.as_deref() == Some("custom"))
            .unwrap();
        assert_eq!(custom.contract.parameters[0].name, "rl");
    }

    #[test]
    fn registry_contract_provider_writes_deterministic_bytes() {
        let provider = repository_provider();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let first_path = first.path().join("provider.json");
        let second_path = second.path().join("provider.json");
        provider.write_json(&first_path).unwrap();
        provider.write_json(&second_path).unwrap();
        assert_eq!(
            std::fs::read(first_path).unwrap(),
            std::fs::read(second_path).unwrap()
        );
    }

    #[test]
    fn malformed_semantic_contract_fails_closed() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("registry.rs");
        std::fs::write(
            &source,
            r#"
            registry_id! {
                @contract(
                    path = "sand::predicate::BrokenId",
                    aliases = [],
                    subject = "a predicate",
                    minecraft = "A predicate ID.",
                    use_when = ["Referring to it"],
                    avoid_when = ["Building it"],
                    example_namespace = "demo"
                );
                BrokenId
            }
            "#,
        )
        .unwrap();
        let error = registry_id_contract_provider(&source, "test")
            .unwrap_err()
            .to_string();
        assert!(error.contains("missing registry contract field `example_path`"));
    }
}
