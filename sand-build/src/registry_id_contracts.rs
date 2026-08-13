//! Contract metadata emitted from semantic `registry_id!` declarations.
//!
//! A registry wrapper opts into a contract partition with `@contract(...)`
//! inside the same invocation that emits its Rust type. The shared generator
//! derives provider shapes from that emitted AST rather than restating them.

use std::collections::BTreeMap;
use std::path::Path;

use crate::api_provider::{ApiProviderCatalog, GeneratedProviderEntry};
use crate::error::Result;
use sand_api_contract::{ApiEntry, ApiKind};

/// Build installed contract metadata from the same expansion that emits each
/// registry wrapper's Rust type and inherent methods.
pub fn registry_id_contract_provider(
    source: &Path,
    minecraft_version: &str,
) -> Result<ApiProviderCatalog> {
    let text = std::fs::read_to_string(source)?;
    let file = syn::parse_file(&text).map_err(std::io::Error::other)?;
    let mut entries = Vec::new();

    for item in &file.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        if !item.mac.path.is_ident("registry_id") {
            continue;
        }
        let expansion = sand_api_contract::syntax::registry_id::expand(item.mac.tokens.clone())
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

    // A handful of registry families expose semantic convenience constructors
    // beside their `registry_id!` invocation (for example `jigsaw()` and
    // `empty()`). They are still associated API of the generated wrapper, so
    // derive their provider entries from the declaration docs instead of
    // leaving a second, handwritten metadata list behind.
    let parents = entries
        .iter()
        .filter(|entry| entry.parent_identity.is_none())
        .map(|entry| (entry.definition_identity.clone(), entry.contract.clone()))
        .collect::<BTreeMap<_, _>>();
    for item in &file.items {
        let syn::Item::Impl(implementation) = item else {
            continue;
        };
        if implementation.trait_.is_some() {
            continue;
        }
        let Some(type_name) = impl_type_name(&implementation.self_ty) else {
            continue;
        };
        let identity = format!("sand_components::registry::{type_name}");
        let Some(parent) = parents.get(&identity) else {
            continue;
        };
        for method in &implementation.items {
            let syn::ImplItem::Fn(method) = method else {
                continue;
            };
            if !matches!(method.vis, syn::Visibility::Public(_)) {
                continue;
            }
            let name = method.sig.ident.to_string();
            entries.push(GeneratedProviderEntry {
                definition_identity: format!("{identity}::{name}"),
                definition_kind: ApiKind::Method,
                parent_identity: Some(identity.clone()),
                member_name: Some(name.clone()),
                contract: documented_convenience_contract(parent, method, &type_name)?,
            });
        }
    }

    Ok(ApiProviderCatalog::new(
        "generated_registry_id_contracts",
        minecraft_version,
        entries,
    ))
}

fn impl_type_name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn documented_convenience_contract(
    parent: &ApiEntry,
    method: &syn::ImplItemFn,
    type_name: &str,
) -> Result<ApiEntry> {
    let documentation = method
        .attrs
        .iter()
        .filter_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(value)
                if value.path.is_ident("doc")
                    && matches!(&value.value, syn::Expr::Lit(expression) if matches!(&expression.lit, syn::Lit::Str(_))) =>
            {
                let syn::Expr::Lit(expression) = &value.value else {
                    unreachable!();
                };
                let syn::Lit::Str(text) = &expression.lit else {
                    unreachable!();
                };
                Some(text.value().trim().to_owned())
            }
            _ => None,
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let summary = documentation
        .split_once('.')
        .map(|(sentence, _)| sentence)
        .unwrap_or(&documentation)
        .trim()
        .to_owned();
    if summary.is_empty() {
        return Err(std::io::Error::other(format!(
            "{type_name}::{} needs Rustdoc because it is a public generated-wrapper convenience API",
            method.sig.ident
        ))
        .into());
    }
    let name = method.sig.ident.to_string();
    Ok(ApiEntry {
        canonical_path: format!("{}::{name}", parent.canonical_path),
        aliases: parent
            .aliases
            .iter()
            .map(|alias| format!("{alias}::{name}"))
            .collect(),
        canonical_module: parent.canonical_path.clone(),
        kind: ApiKind::Method,
        signature: quote::quote!(pub #method).to_string(),
        summary,
        context: format!(
            "This documented convenience constructor specializes {} without requiring callers to repeat a conventional vanilla resource path.",
            parent.canonical_path
        ),
        minecraft: parent.minecraft.clone(),
        use_when: vec![format!(
            "Using the documented conventional {type_name} identifier"
        )],
        avoid_when: vec![format!(
            "Selecting a different or custom {type_name} identifier; use minecraft or custom"
        )],
        parameters: Vec::new(),
        returns: Some(format!("The conventional typed {type_name} identifier.")),
        example: format!("let id = {}::{name}();", parent.canonical_path),
        availability: parent.availability.clone(),
    })
}

/// Validate and install the provider artifact published by the packaged
/// `sand-components` dependency into a consuming crate's output directory.
pub fn install_registry_id_contract_provider(
    source: &Path,
    destination: &Path,
    minecraft_version: &str,
) -> Result<()> {
    let catalog = crate::read_api_provider(source)?;
    if catalog.provider != "generated_registry_id_contracts" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "{} declares provider `{}`, expected `generated_registry_id_contracts`",
                source.display(),
                catalog.provider
            ),
        )
        .into());
    }
    if catalog.minecraft_version != minecraft_version {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "{} targets Minecraft {}, expected {minecraft_version}",
                source.display(),
                catalog.minecraft_version
            ),
        )
        .into());
    }
    catalog.validate().map_err(std::io::Error::other)?;
    catalog.write_json(destination)
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
        assert_eq!(provider.entries.len(), 148);
        let predicate = provider
            .entries
            .iter()
            .filter(|entry| {
                entry
                    .contract
                    .canonical_path
                    .starts_with("sand::predicate::PredicateId")
            })
            .collect::<Vec<_>>();
        assert_eq!(predicate.len(), 4);
        assert_eq!(predicate[0].contract.canonical_module, "sand::predicate");
        assert!(
            predicate[1..]
                .iter()
                .all(|entry| entry.contract.canonical_module == "sand::predicate::PredicateId")
        );
        provider.validate().unwrap();

        let paths = predicate
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
        for entry in &predicate {
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
            predicate[0].contract.aliases,
            [
                "sand::component::PredicateId",
                "sand::prelude::PredicateId",
                "sand::resource_ref::PredicateId"
            ]
        );
        let minecraft = predicate
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
        let custom = predicate
            .iter()
            .find(|entry| entry.member_name.as_deref() == Some("custom"))
            .unwrap();
        assert_eq!(custom.contract.parameters[0].name, "rl");
    }

    #[test]
    fn resource_reference_ids_emit_complete_dialog_capability_contracts() {
        let provider = repository_provider();
        let resource_entries = provider
            .entries
            .iter()
            .filter(|entry| {
                entry
                    .contract
                    .canonical_path
                    .starts_with("sand::resource_ref::")
            })
            .collect::<Vec<_>>();
        assert_eq!(resource_entries.len(), 22);
        let dialog_local = resource_entries
            .iter()
            .find(|entry| entry.contract.canonical_path == "sand::resource_ref::DialogId::local")
            .unwrap();
        assert_eq!(dialog_local.contract.parameters[0].name, "path");
        assert_eq!(
            dialog_local.contract.availability,
            ["Minecraft Java 1.21.6+", "Minecraft Java 26.x"]
        );
        assert!(
            dialog_local
                .contract
                .minecraft
                .contains("namespace sentinel")
        );
        let dialog_try_local = resource_entries
            .iter()
            .find(|entry| {
                entry.contract.canonical_path == "sand::resource_ref::DialogId::try_local"
            })
            .unwrap();
        assert!(
            dialog_try_local
                .contract
                .returns
                .as_deref()
                .unwrap()
                .contains("error")
        );
        assert!(resource_entries.iter().all(|entry| {
            !entry.contract.summary.trim().is_empty()
                && !entry.contract.context.trim().is_empty()
                && !entry.contract.minecraft.trim().is_empty()
                && !entry.contract.use_when.is_empty()
                && !entry.contract.avoid_when.is_empty()
                && !entry.contract.example.trim().is_empty()
        }));
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

    #[test]
    fn packaged_components_provider_installs_without_workspace_sibling_sources() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let package_layout = tempfile::tempdir().unwrap();
        let components = package_layout.path().join("registry-package");
        let core = package_layout.path().join("consumer-package");
        std::fs::create_dir_all(components.join("src")).unwrap();
        std::fs::create_dir_all(core.join("out")).unwrap();
        std::fs::copy(
            workspace.join("sand-components/src/registry.rs"),
            components.join("src/registry.rs"),
        )
        .unwrap();

        let published = components.join("registry_ids.api.json");
        registry_id_contract_provider(&components.join("src/registry.rs"), "package-test")
            .unwrap()
            .write_json(&published)
            .unwrap();
        let installed = core.join("out/registry_ids.api.json");
        install_registry_id_contract_provider(&published, &installed, "package-test").unwrap();

        assert_eq!(
            std::fs::read(published).unwrap(),
            std::fs::read(installed).unwrap()
        );
        assert!(!core.join("../sand-components/src/registry.rs").exists());
    }
}
