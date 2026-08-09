//! Contract metadata emitted from semantic `registry_id!` declarations.
//!
//! A registry wrapper opts into a contract partition with `#[registry_api]`
//! inside the same invocation that emits its Rust type. The declaration owns
//! domain meaning while this module owns the exact shared behavior of the
//! three methods generated for every wrapper.

use std::path::Path;

use sand_api_contract::{ApiEntry, ApiKind, ApiParameter};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, LitStr, Token};

use crate::api_provider::{ApiProviderCatalog, GeneratedProviderEntry};
use crate::error::Result;

struct RegistryInvocation {
    contract: Option<proc_macro2::TokenStream>,
    _attributes: Vec<Attribute>,
    name: syn::Ident,
}

impl Parse for RegistryInvocation {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let contract = if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let keyword: syn::Ident = input.parse()?;
            if keyword != "contract" {
                return Err(syn::Error::new_spanned(keyword, "expected `contract`"));
            }
            let content;
            syn::parenthesized!(content in input);
            let tokens = content.parse()?;
            input.parse::<Token![;]>()?;
            Some(tokens)
        } else {
            None
        };
        Ok(Self {
            contract,
            _attributes: Attribute::parse_outer(input)?,
            name: input.parse()?,
        })
    }
}

#[derive(Default)]
struct RegistryContract {
    path: Option<LitStr>,
    aliases: Option<Vec<LitStr>>,
    subject: Option<LitStr>,
    minecraft: Option<LitStr>,
    use_when: Option<Vec<LitStr>>,
    avoid_when: Option<Vec<LitStr>>,
    example_namespace: Option<LitStr>,
    example_path: Option<LitStr>,
}

/// Build installed contract metadata for registry-ID declarations that opt
/// into a semantic contract partition.
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
        let invocation =
            syn::parse2::<RegistryInvocation>(item.mac.tokens).map_err(std::io::Error::other)?;
        let Some(contract_tokens) = invocation.contract else {
            continue;
        };
        let contract = parse_contract(contract_tokens).map_err(std::io::Error::other)?;
        entries.extend(entries_for(&invocation.name.to_string(), contract)?);
    }

    Ok(ApiProviderCatalog::new(
        "generated_registry_id_contracts",
        minecraft_version,
        entries,
    ))
}

fn parse_contract(tokens: proc_macro2::TokenStream) -> syn::Result<RegistryContract> {
    let mut contract = RegistryContract::default();
    let parser = syn::meta::parser(|meta| {
        let name = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("registry contract keys must be identifiers"))?
            .to_string();
        match name.as_str() {
            "path" => set_once(&mut contract.path, meta.value()?.parse()?, &meta, &name),
            "aliases" => set_once(
                &mut contract.aliases,
                parse_strings(meta.value()?.parse()?)?,
                &meta,
                &name,
            ),
            "subject" => set_once(&mut contract.subject, meta.value()?.parse()?, &meta, &name),
            "minecraft" => set_once(
                &mut contract.minecraft,
                meta.value()?.parse()?,
                &meta,
                &name,
            ),
            "use_when" => set_once(
                &mut contract.use_when,
                parse_strings(meta.value()?.parse()?)?,
                &meta,
                &name,
            ),
            "avoid_when" => set_once(
                &mut contract.avoid_when,
                parse_strings(meta.value()?.parse()?)?,
                &meta,
                &name,
            ),
            "example_namespace" => set_once(
                &mut contract.example_namespace,
                meta.value()?.parse()?,
                &meta,
                &name,
            ),
            "example_path" => set_once(
                &mut contract.example_path,
                meta.value()?.parse()?,
                &meta,
                &name,
            ),
            _ => Err(meta.error(format!("unknown registry contract field `{name}`"))),
        }
    });
    syn::parse::Parser::parse2(parser, tokens)?;
    Ok(contract)
}

fn parse_strings(array: syn::ExprArray) -> syn::Result<Vec<LitStr>> {
    array
        .elems
        .into_iter()
        .map(|element| match element {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(value),
                ..
            }) => Ok(value),
            other => Err(syn::Error::new_spanned(other, "expected a string literal")),
        })
        .collect()
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
) -> syn::Result<()> {
    if slot.replace(value).is_some() {
        Err(meta.error(format!("duplicate registry contract field `{name}`")))
    } else {
        Ok(())
    }
}

fn required(slot: Option<LitStr>, name: &str) -> Result<String> {
    let value = slot.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("missing registry contract field `{name}`"),
        )
    })?;
    let value = value.value();
    if value.trim().is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("registry contract field `{name}` cannot be empty"),
        )
        .into());
    }
    Ok(value)
}

fn required_list(slot: Option<Vec<LitStr>>, name: &str) -> Result<Vec<String>> {
    let values = slot.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("missing registry contract field `{name}`"),
        )
    })?;
    if values.is_empty() || values.iter().any(|value| value.value().trim().is_empty()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("registry contract field `{name}` must contain non-empty values"),
        )
        .into());
    }
    Ok(values.into_iter().map(|value| value.value()).collect())
}

fn entries_for(name: &str, contract: RegistryContract) -> Result<Vec<GeneratedProviderEntry>> {
    let path = required(contract.path, "path")?;
    if path.rsplit("::").next() != Some(name) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("registry contract path `{path}` does not end in `{name}`"),
        )
        .into());
    }
    let module = path
        .rsplit_once("::")
        .map(|(module, _)| module.to_owned())
        .ok_or_else(|| std::io::Error::other("registry contract path has no module"))?;
    let aliases = contract
        .aliases
        .unwrap_or_default()
        .into_iter()
        .map(|alias| alias.value())
        .collect::<Vec<_>>();
    let subject = required(contract.subject, "subject")?;
    let minecraft = required(contract.minecraft, "minecraft")?;
    let use_when = required_list(contract.use_when, "use_when")?;
    let avoid_when = required_list(contract.avoid_when, "avoid_when")?;
    let example_namespace = required(contract.example_namespace, "example_namespace")?;
    let example_path = required(contract.example_path, "example_path")?;
    let identity = format!("sand_components::registry::{name}");
    let availability = vec!["all configurations".to_owned()];

    let type_entry = ApiEntry {
        canonical_path: path.clone(),
        aliases: aliases.clone(),
        canonical_module: module.clone(),
        kind: ApiKind::Struct,
        signature: format!("pub struct {name}(ResourceLocation)"),
        summary: format!("Identifies a {subject} with a validated Minecraft resource location."),
        context: format!(
            "{name} keeps the namespace and path of a {subject} distinct from unrelated registry identifiers."
        ),
        minecraft: minecraft.clone(),
        use_when: use_when.clone(),
        avoid_when: avoid_when.clone(),
        parameters: Vec::new(),
        returns: None,
        example: format!(
            "let id = {name}::custom(ResourceLocation::new(\"{example_namespace}\", \"{example_path}\")?);"
        ),
        availability: availability.clone(),
    };

    let method = |method: &str,
                  signature: String,
                  summary: String,
                  context: String,
                  method_minecraft: String,
                  parameters: Vec<ApiParameter>,
                  returns: &str,
                  example: String| {
        GeneratedProviderEntry {
            definition_identity: format!("{identity}::{method}"),
            definition_kind: ApiKind::Method,
            parent_identity: Some(identity.clone()),
            member_name: Some(method.to_owned()),
            contract: ApiEntry {
                canonical_path: format!("{path}::{method}"),
                aliases: aliases
                    .iter()
                    .map(|alias| format!("{alias}::{method}"))
                    .collect(),
                canonical_module: path.clone(),
                kind: ApiKind::Method,
                signature,
                summary,
                context,
                minecraft: method_minecraft,
                use_when: use_when.clone(),
                avoid_when: avoid_when.clone(),
                parameters,
                returns: Some(returns.to_owned()),
                example,
                availability: availability.clone(),
            },
        }
    };

    Ok(vec![
        GeneratedProviderEntry {
            definition_identity: identity.clone(),
            definition_kind: ApiKind::Struct,
            parent_identity: None,
            member_name: None,
            contract: type_entry,
        },
        method(
            "minecraft",
            "pub fn minecraft(path: impl AsRef<str>) -> Result<Self>".to_owned(),
            format!("Creates an identifier for a {subject} in the minecraft namespace."),
            format!("Use this constructor for a vanilla {subject} rather than spelling the minecraft namespace repeatedly."),
            format!("Validates the path and emits minecraft:<path> when the {subject} identifier is serialized."),
            vec![ApiParameter { name: "path".to_owned(), description: format!("The resource path of the {subject} inside the minecraft namespace.") }],
            "The validated typed identifier, or an error when the resource path is invalid.",
            format!("let id = {name}::minecraft(\"{example_path}\")?;"),
        ),
        method(
            "custom",
            "pub fn custom(rl: ResourceLocation) -> Self".to_owned(),
            format!("Wraps a validated custom resource location as an identifier for a {subject}."),
            format!("This preserves the namespace chosen by a datapack or mod while retaining the registry-specific {name} type."),
            format!("Serializes the supplied namespace:path unchanged wherever Minecraft expects the {subject}."),
            vec![ApiParameter { name: "rl".to_owned(), description: format!("The validated namespaced location of the {subject}.") }],
            "The registry-specific typed identifier.",
            format!("let id = {name}::custom(ResourceLocation::new(\"{example_namespace}\", \"{example_path}\")?);"),
        ),
        method(
            "as_resource_location",
            "pub fn as_resource_location(&self) -> &ResourceLocation".to_owned(),
            format!("Borrows the resource location stored by this {subject} identifier."),
            "Use the shared ResourceLocation view when an API accepts identifiers from multiple Minecraft registries.".to_owned(),
            format!("Does not change serialization; it exposes the validated namespace and path Minecraft uses for the {subject}."),
            Vec::new(),
            "A borrowed view of the identifier's validated namespace and path.",
            format!("let id = {name}::custom(ResourceLocation::new(\"{example_namespace}\", \"{example_path}\")?); let location = id.as_resource_location();"),
        ),
    ])
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
