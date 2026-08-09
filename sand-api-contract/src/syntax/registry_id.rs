//! Shared expansion and contract derivation for `registry_id!`.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, FnArg, Item, LitStr, Pat, Token};

use crate::{ApiEntry, ApiKind, ApiParameter};

/// One provider definition derived from the same AST emitted as Rust code.
pub struct RegistryApiDefinition {
    pub definition_identity: String,
    pub definition_kind: ApiKind,
    pub parent_identity: Option<String>,
    pub member_name: Option<String>,
    pub contract: ApiEntry,
}

/// The compiled Rust expansion and any opted-in API definitions.
pub struct RegistryIdExpansion {
    pub rust: TokenStream,
    pub definitions: Vec<RegistryApiDefinition>,
}

struct Invocation {
    contract: Option<TokenStream>,
    attributes: Vec<Attribute>,
    name: syn::Ident,
}

impl Parse for Invocation {
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
            attributes: Attribute::parse_outer(input)?,
            name: input.parse()?,
        })
    }
}

impl ToTokens for Invocation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(contract) = &self.contract {
            tokens.extend(quote! { @contract(#contract); });
        }
        let attributes = &self.attributes;
        let name = &self.name;
        tokens.extend(quote! {
            #(#attributes)*
            #name
        });
    }
}

#[derive(Default)]
struct Contract {
    path: Option<LitStr>,
    aliases: Option<Vec<LitStr>>,
    subject: Option<LitStr>,
    minecraft: Option<LitStr>,
    use_when: Option<Vec<LitStr>>,
    avoid_when: Option<Vec<LitStr>>,
    example_namespace: Option<LitStr>,
    example_path: Option<LitStr>,
}

/// Expand one `registry_id!` invocation and derive metadata from its emitted AST.
pub fn expand(input: TokenStream) -> syn::Result<RegistryIdExpansion> {
    let invocation = syn::parse2::<Invocation>(input)?;
    let name = &invocation.name;
    let attributes = &invocation.attributes;
    let base = quote! {
        #(#attributes)*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct #name(ResourceLocation);

        impl #name {
            pub fn minecraft(path: impl AsRef<str>) -> Result<Self> {
                Ok(Self(ResourceLocation::minecraft(path)?))
            }

            pub fn custom(rl: ResourceLocation) -> Self {
                Self(rl)
            }

            pub fn as_resource_location(&self) -> &ResourceLocation {
                &self.0
            }
        }

        impl From<ResourceLocation> for #name {
            fn from(rl: ResourceLocation) -> Self { Self(rl) }
        }

        impl From<#name> for ResourceLocation {
            fn from(id: #name) -> Self { id.0 }
        }

        impl fmt::Display for #name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
        }

        impl Serialize for #name {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                self.0.serialize(s)
            }
        }

        impl std::str::FromStr for #name {
            type Err = crate::error::SandError;
            fn from_str(s: &str) -> Result<Self> { Ok(Self(s.parse()?)) }
        }
    };
    let mut file = syn::parse2::<syn::File>(base)?;
    let definitions = match invocation.contract {
        Some(tokens) => definitions(&file, &invocation.name, parse_contract(tokens)?)?,
        None => Vec::new(),
    };
    if definitions.is_empty() {
        add_default_method_docs(&mut file, &invocation.name);
    } else {
        add_contract_docs(&mut file, &invocation.name, &definitions)?;
    }
    Ok(RegistryIdExpansion {
        rust: file.into_token_stream(),
        definitions,
    })
}

fn parse_contract(tokens: TokenStream) -> syn::Result<Contract> {
    let mut contract = Contract::default();
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

fn required(value: Option<LitStr>, name: &str) -> syn::Result<String> {
    let value = value
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("missing registry contract field `{name}`"),
            )
        })?
        .value();
    if value.trim().is_empty() {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("registry contract field `{name}` cannot be empty"),
        ))
    } else {
        Ok(value)
    }
}

fn required_list(value: Option<Vec<LitStr>>, name: &str) -> syn::Result<Vec<String>> {
    let values = value.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("missing registry contract field `{name}`"),
        )
    })?;
    if values.is_empty() || values.iter().any(|value| value.value().trim().is_empty()) {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("registry contract field `{name}` must contain non-empty values"),
        ));
    }
    Ok(values.into_iter().map(|value| value.value()).collect())
}

fn definitions(
    file: &syn::File,
    name: &syn::Ident,
    contract: Contract,
) -> syn::Result<Vec<RegistryApiDefinition>> {
    let path = required(contract.path, "path")?;
    if path.rsplit("::").next() != Some(name.to_string().as_str()) {
        return Err(syn::Error::new_spanned(
            name,
            format!("registry contract path `{path}` does not end in `{name}`"),
        ));
    }
    let module = path
        .rsplit_once("::")
        .map(|(module, _)| module.to_owned())
        .ok_or_else(|| syn::Error::new_spanned(name, "registry contract path has no module"))?;
    let aliases = contract
        .aliases
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.value())
        .collect::<Vec<_>>();
    let subject = required(contract.subject, "subject")?;
    let minecraft = required(contract.minecraft, "minecraft")?;
    let use_when = required_list(contract.use_when, "use_when")?;
    let avoid_when = required_list(contract.avoid_when, "avoid_when")?;
    let example_namespace = required(contract.example_namespace, "example_namespace")?;
    let example_path = required(contract.example_path, "example_path")?;
    let identity = format!("sand_components::registry::{name}");
    let availability = vec!["all configurations".to_owned()];
    let structure = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == *name => Some(item),
            _ => None,
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(name, "registry expansion did not emit its public struct")
        })?;
    let mut structure_shape = structure.clone();
    structure_shape.attrs.clear();
    let type_entry = ApiEntry {
        canonical_path: path.clone(),
        aliases: aliases.clone(),
        canonical_module: module,
        kind: ApiKind::Struct,
        signature: structure_shape.into_token_stream().to_string(),
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
    let implementation = file
        .items
        .iter()
        .find_map(|item| match item {
            Item::Impl(item) if item.trait_.is_none() && self_type_is(&item.self_ty, name) => {
                Some(item)
            }
            _ => None,
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(name, "registry expansion did not emit its inherent impl")
        })?;
    let methods = implementation
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    let find = |method: &str| {
        methods
            .iter()
            .copied()
            .find(|item| item.sig.ident == method)
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    name,
                    format!("registry expansion is missing required method `{method}`"),
                )
            })
    };
    let method_entry = |item: &syn::ImplItemFn,
                        summary: String,
                        context: String,
                        behavior: String,
                        descriptions: &[String],
                        returns: &str,
                        example: String|
     -> syn::Result<RegistryApiDefinition> {
        let parameters = parameter_names(&item.sig)?;
        if parameters.len() != descriptions.len() {
            return Err(syn::Error::new_spanned(
                &item.sig,
                format!(
                    "registry method `{}` has {} parameters but {} semantic descriptions",
                    item.sig.ident,
                    parameters.len(),
                    descriptions.len()
                ),
            ));
        }
        let method = item.sig.ident.to_string();
        let visibility = &item.vis;
        let signature = &item.sig;
        Ok(RegistryApiDefinition {
            definition_identity: format!("{identity}::{method}"),
            definition_kind: ApiKind::Method,
            parent_identity: Some(identity.clone()),
            member_name: Some(method.clone()),
            contract: ApiEntry {
                canonical_path: format!("{path}::{method}"),
                aliases: aliases
                    .iter()
                    .map(|alias| format!("{alias}::{method}"))
                    .collect(),
                canonical_module: path.clone(),
                kind: ApiKind::Method,
                signature: quote!(#visibility #signature).to_string(),
                summary,
                context,
                minecraft: behavior,
                use_when: use_when.clone(),
                avoid_when: avoid_when.clone(),
                parameters: parameters
                    .into_iter()
                    .zip(descriptions.iter().cloned())
                    .map(|(name, description)| ApiParameter { name, description })
                    .collect(),
                returns: Some(returns.to_owned()),
                example,
                availability: availability.clone(),
            },
        })
    };
    let minecraft_method = find("minecraft")?;
    let custom_method = find("custom")?;
    let location_method = find("as_resource_location")?;
    let mut result = vec![RegistryApiDefinition {
        definition_identity: identity.clone(),
        definition_kind: ApiKind::Struct,
        parent_identity: None,
        member_name: None,
        contract: type_entry,
    }];
    result.push(method_entry(minecraft_method,
        format!("Creates an identifier for a {subject} in the minecraft namespace."),
        format!("Use this constructor for a vanilla {subject} rather than spelling the minecraft namespace repeatedly."),
        format!("Validates the path and emits minecraft:<path> when the {subject} identifier is serialized."),
        &[format!("The resource path of the {subject} inside the minecraft namespace.")],
        "The validated typed identifier, or an error when the resource path is invalid.", format!("let id = {name}::minecraft(\"{example_path}\")?;"))?);
    result.push(method_entry(custom_method,
        format!("Wraps a validated custom resource location as an identifier for a {subject}."),
        format!("This preserves the namespace chosen by a datapack or mod while retaining the registry-specific {name} type."),
        format!("Serializes the supplied namespace:path unchanged wherever Minecraft expects the {subject}."),
        &[format!("The validated namespaced location of the {subject}.")], "The registry-specific typed identifier.",
        format!("let id = {name}::custom(ResourceLocation::new(\"{example_namespace}\", \"{example_path}\")?);"))?);
    result.push(method_entry(location_method,
        format!("Borrows the resource location stored by this {subject} identifier."),
        "Use the shared ResourceLocation view when an API accepts identifiers from multiple Minecraft registries.".to_owned(),
        format!("Does not change serialization; it exposes the validated namespace and path Minecraft uses for the {subject}."),
        &[], "A borrowed view of the identifier's validated namespace and path.",
        format!("let id = {name}::custom(ResourceLocation::new(\"{example_namespace}\", \"{example_path}\")?); let location = id.as_resource_location();"))?);
    Ok(result)
}

fn parameter_names(signature: &syn::Signature) -> syn::Result<Vec<String>> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Receiver(_) => None,
            FnArg::Typed(argument) => Some(match argument.pat.as_ref() {
                Pat::Ident(ident) => Ok(ident.ident.to_string()),
                pattern => Err(syn::Error::new_spanned(
                    pattern,
                    "registry API parameters must use simple identifier patterns",
                )),
            }),
        })
        .collect()
}

fn add_default_method_docs(file: &mut syn::File, name: &syn::Ident) {
    let Some(implementation) = inherent_impl_mut(file, name) else {
        return;
    };
    for item in &mut implementation.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let text = match method.sig.ident.to_string().as_str() {
            "minecraft" => {
                "Construct a `minecraft:<path>` ID. Returns an error if `path` is invalid."
            }
            "custom" => "Wrap any `ResourceLocation` as this registry ID.",
            "as_resource_location" => "Access the inner `ResourceLocation`.",
            _ => continue,
        };
        method.attrs.push(syn::parse_quote!(#[doc = #text]));
    }
}

fn add_contract_docs(
    file: &mut syn::File,
    name: &syn::Ident,
    definitions: &[RegistryApiDefinition],
) -> syn::Result<()> {
    let type_entry = &definitions[0].contract;
    let structure = file
        .items
        .iter_mut()
        .find_map(|item| match item {
            Item::Struct(item) if item.ident == *name => Some(item),
            _ => None,
        })
        .ok_or_else(|| syn::Error::new_spanned(name, "missing registry struct"))?;
    structure.attrs.extend(doc_attributes(type_entry));
    let implementation = inherent_impl_mut(file, name)
        .ok_or_else(|| syn::Error::new_spanned(name, "missing registry impl"))?;
    for method in &mut implementation.items {
        let syn::ImplItem::Fn(method) = method else {
            continue;
        };
        if let Some(definition) = definitions.iter().find(|definition| {
            definition.member_name.as_deref() == Some(method.sig.ident.to_string().as_str())
        }) {
            method.attrs.extend(doc_attributes(&definition.contract));
        }
    }
    Ok(())
}

fn inherent_impl_mut<'a>(
    file: &'a mut syn::File,
    name: &syn::Ident,
) -> Option<&'a mut syn::ItemImpl> {
    file.items.iter_mut().find_map(|item| match item {
        Item::Impl(item) if item.trait_.is_none() && self_type_is(&item.self_ty, name) => {
            Some(item)
        }
        _ => None,
    })
}

fn self_type_is(ty: &syn::Type, name: &syn::Ident) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident(name))
}

fn doc_attributes(entry: &ApiEntry) -> Vec<Attribute> {
    let mut lines = vec![
        entry.summary.clone(),
        String::new(),
        "# Context".into(),
        entry.context.clone(),
        String::new(),
        "# Minecraft behavior".into(),
        entry.minecraft.clone(),
    ];
    if !entry.parameters.is_empty() {
        lines.extend([String::new(), "# Parameters".into()]);
        lines.extend(
            entry
                .parameters
                .iter()
                .map(|parameter| format!("- `{}` — {}", parameter.name, parameter.description)),
        );
    }
    if let Some(returns) = &entry.returns {
        lines.extend([String::new(), "# Returns".into(), returns.clone()]);
    }
    lines.extend([String::new(), "# Use when".into()]);
    lines.extend(entry.use_when.iter().map(|value| format!("- {value}")));
    lines.extend([String::new(), "# Avoid when".into()]);
    lines.extend(entry.avoid_when.iter().map(|value| format!("- {value}")));
    lines.extend([
        String::new(),
        "# Example".into(),
        "```rust,ignore".into(),
        entry.example.clone(),
        "```".into(),
        String::new(),
        "# API Contract".into(),
        "View this API with:".into(),
        "```text".into(),
        format!("sand api show {}", entry.canonical_path),
        "```".into(),
    ]);
    lines
        .into_iter()
        .map(|line| syn::parse_quote!(#[doc = #line]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invocation(name: &str) -> Invocation {
        syn::parse_str(&format!(
            r#"@contract(
                path = "sand::predicate::{name}",
                aliases = ["sand::prelude::{name}"],
                subject = "standalone predicate resource",
                minecraft = "Serializes a predicate resource identifier.",
                use_when = ["Referring to a predicate"],
                avoid_when = ["Building its condition tree"],
                example_namespace = "demo",
                example_path = "conditions/example"
            );
            {name}"#
        ))
        .unwrap()
    }

    fn docs(attributes: &[Attribute]) -> Vec<String> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("doc"))
            .filter_map(|attribute| match &attribute.meta {
                syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(line),
                            ..
                        }),
                    ..
                }) => Some(line.value()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn method_shape_and_parameter_names_come_from_emitted_ast() {
        let invocation = invocation("ChangedId");
        let contract = parse_contract(invocation.contract.unwrap()).unwrap();
        let file: syn::File = syn::parse_quote! {
            pub struct ChangedId(ResourceLocation);
            impl ChangedId {
                pub fn minecraft(resource_path: impl AsRef<str>) -> Result<Self> { todo!() }
                pub fn custom(location: ResourceLocation) -> Self { todo!() }
                pub fn as_resource_location(&self) -> &ResourceLocation { todo!() }
            }
        };
        let definitions = definitions(&file, &invocation.name, contract).unwrap();
        let minecraft = definitions
            .iter()
            .find(|definition| definition.member_name.as_deref() == Some("minecraft"))
            .unwrap();
        assert!(minecraft.contract.signature.contains("resource_path"));
        assert_eq!(minecraft.contract.parameters[0].name, "resource_path");
        let custom = definitions
            .iter()
            .find(|definition| definition.member_name.as_deref() == Some("custom"))
            .unwrap();
        assert_eq!(custom.contract.parameters[0].name, "location");
    }

    #[test]
    fn unexplained_shape_change_fails_contract_derivation() {
        let invocation = invocation("ChangedId");
        let contract = parse_contract(invocation.contract.unwrap()).unwrap();
        let file: syn::File = syn::parse_quote! {
            pub struct ChangedId(ResourceLocation);
            impl ChangedId {
                pub fn minecraft(path: impl AsRef<str>, strict: bool) -> Result<Self> { todo!() }
                pub fn custom(rl: ResourceLocation) -> Self { todo!() }
                pub fn as_resource_location(&self) -> &ResourceLocation { todo!() }
            }
        };
        let error = definitions(&file, &invocation.name, contract)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("2 parameters but 1 semantic descriptions"));
    }

    #[test]
    fn contracted_expansion_generates_type_and_method_rustdoc() {
        let expansion = expand(invocation("DocumentedId").into_token_stream()).unwrap();
        let file = syn::parse2::<syn::File>(expansion.rust).unwrap();
        let structure = file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Struct(item) if item.ident == "DocumentedId" => Some(item),
                _ => None,
            })
            .unwrap();
        let type_docs = docs(&structure.attrs).join("\n");
        assert!(type_docs.contains("# API Contract"));
        assert!(type_docs.contains("sand api show sand::predicate::DocumentedId"));
        let implementation = file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Impl(item) if item.trait_.is_none() => Some(item),
                _ => None,
            })
            .unwrap();
        for method in ["minecraft", "custom", "as_resource_location"] {
            let item = implementation
                .items
                .iter()
                .find_map(|item| match item {
                    syn::ImplItem::Fn(item) if item.sig.ident == method => Some(item),
                    _ => None,
                })
                .unwrap();
            let method_docs = docs(&item.attrs).join("\n");
            assert!(method_docs.contains("# API Contract"));
            assert!(method_docs.contains(&format!(
                "sand api show sand::predicate::DocumentedId::{method}"
            )));
        }
    }
}
