//! Shared `#[api]` syntax parsing and item-shape validation.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::{ExprArray, FnArg, ItemEnum, ItemStruct, LitStr, Pat, ReturnType, Signature};

/// One explicitly described function parameter or nested API member.
#[derive(Clone)]
pub struct Description {
    pub name: syn::Ident,
    pub text: LitStr,
}

/// Parsed `#[api(...)]` arguments, retaining spans for precise diagnostics.
#[derive(Default)]
pub struct ContractArgs {
    /// Narrow item-kind hint for syntax that is ambiguous outside its enclosing impl.
    pub kind: Option<LitStr>,
    /// Rust path used only to transport the link-time registration.
    ///
    /// Facade users normally omit this and register through Sand's hidden
    /// re-export. API-defining implementation crates set it to their direct
    /// `sand-api-contract` dependency, avoiding a dependency on the facade.
    pub registry: Option<syn::Path>,
    pub path: Option<LitStr>,
    pub module: Option<LitStr>,
    pub aliases: Option<Vec<LitStr>>,
    pub summary: Option<LitStr>,
    pub context: Option<LitStr>,
    pub minecraft: Option<LitStr>,
    pub use_when: Option<Vec<LitStr>>,
    pub avoid_when: Option<Vec<LitStr>>,
    pub params: Option<Vec<Description>>,
    pub returns: Option<LitStr>,
    pub example: Option<LitStr>,
    pub availability: Option<Vec<LitStr>>,
    pub variants: Option<Vec<Description>>,
    pub fields: Option<Vec<Description>>,
}

/// The declaration shape whose contract is being validated.
pub enum ContractTarget<'a> {
    Function {
        ident: &'a syn::Ident,
        signature: &'a Signature,
    },
    Struct(&'a ItemStruct),
    Enum(&'a ItemEnum),
    Plain {
        ident: &'a syn::Ident,
    },
}

impl ContractTarget<'_> {
    pub fn ident(&self) -> &syn::Ident {
        match self {
            Self::Function { ident, .. } | Self::Plain { ident } => ident,
            Self::Struct(item) => &item.ident,
            Self::Enum(item) => &item.ident,
        }
    }
}

/// Parse one attribute's comma-separated argument tokens.
pub fn parse_contract_args(tokens: TokenStream) -> syn::Result<ContractArgs> {
    let mut args = ContractArgs::default();
    let parser = syn::meta::parser(|meta| {
        let key = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("API contract keys must be identifiers"))?
            .to_string();
        match key.as_str() {
            "kind" => set_once(&mut args.kind, meta.value()?.parse()?, &meta, "kind"),
            "registry" => set_once(
                &mut args.registry,
                meta.value()?.parse()?,
                &meta,
                "registry",
            ),
            "path" => set_once(&mut args.path, meta.value()?.parse()?, &meta, "path"),
            "module" => set_once(&mut args.module, meta.value()?.parse()?, &meta, "module"),
            "summary" => set_once(&mut args.summary, meta.value()?.parse()?, &meta, "summary"),
            "context" => set_once(&mut args.context, meta.value()?.parse()?, &meta, "context"),
            "minecraft" => set_once(
                &mut args.minecraft,
                meta.value()?.parse()?,
                &meta,
                "minecraft",
            ),
            "returns" => set_once(&mut args.returns, meta.value()?.parse()?, &meta, "returns"),
            "example" => set_once(&mut args.example, meta.value()?.parse()?, &meta, "example"),
            "aliases" => parse_array_field(&mut args.aliases, &meta, "aliases"),
            "use_when" => parse_array_field(&mut args.use_when, &meta, "use_when"),
            "avoid_when" => parse_array_field(&mut args.avoid_when, &meta, "avoid_when"),
            "availability" => parse_array_field(&mut args.availability, &meta, "availability"),
            "params" => parse_descriptions(&mut args.params, &meta, "params", "parameter"),
            "variants" => parse_descriptions(&mut args.variants, &meta, "variants", "variant"),
            "fields" => parse_descriptions(&mut args.fields, &meta, "fields", "field"),
            _ => Err(meta.error(format!("unknown API contract field `{key}`"))),
        }
    });
    parser.parse2(tokens)?;
    Ok(args)
}

fn parse_array_field(
    slot: &mut Option<Vec<LitStr>>,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
) -> syn::Result<()> {
    let value = parse_string_array(meta)?;
    set_once(slot, value, meta, name)
}

fn parse_descriptions(
    slot: &mut Option<Vec<Description>>,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
    member_kind: &str,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(meta.error(format!("duplicate API contract field `{name}`")));
    }
    let mut descriptions = Vec::new();
    let mut names = BTreeSet::new();
    meta.parse_nested_meta(|member| {
        let ident = member
            .path
            .get_ident()
            .cloned()
            .ok_or_else(|| member.error(format!("{member_kind} names must be identifiers")))?;
        if !names.insert(ident.to_string()) {
            return Err(member.error(format!(
                "duplicate {member_kind} documentation for `{ident}`"
            )));
        }
        let text: LitStr = member.value()?.parse()?;
        if text.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(
                &text,
                format!("{member_kind} description cannot be empty"),
            ));
        }
        descriptions.push(Description { name: ident, text });
        Ok(())
    })?;
    *slot = Some(descriptions);
    Ok(())
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &syn::meta::ParseNestedMeta<'_>,
    name: &str,
) -> syn::Result<()> {
    if slot.replace(value).is_some() {
        Err(meta.error(format!("duplicate API contract field `{name}`")))
    } else {
        Ok(())
    }
}

fn parse_string_array(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Vec<LitStr>> {
    let array = meta.value()?.parse::<ExprArray>()?;
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

/// Validate required prose, paths, parameters, returns, and nested members.
pub fn validate_contract(args: &ContractArgs, target: &ContractTarget<'_>) -> syn::Result<()> {
    let ident = target.ident();
    for (name, value) in [
        ("summary", args.summary.as_ref()),
        ("context", args.context.as_ref()),
        ("minecraft", args.minecraft.as_ref()),
        ("example", args.example.as_ref()),
    ] {
        let value = value.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                format!("missing required API contract field `{name}`"),
            )
        })?;
        if value.value().trim().is_empty() {
            return Err(syn::Error::new_spanned(
                value,
                format!("API contract field `{name}` cannot be empty"),
            ));
        }
    }
    for (name, values) in [
        ("use_when", args.use_when.as_ref()),
        ("avoid_when", args.avoid_when.as_ref()),
    ] {
        let values = values.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                format!("missing required API contract field `{name}`"),
            )
        })?;
        if values.is_empty() || values.iter().any(|value| value.value().trim().is_empty()) {
            return Err(syn::Error::new(
                ident.span(),
                format!("API contract field `{name}` must contain non-empty strings"),
            ));
        }
    }
    if let Some(path) = &args.path {
        validate_path(path, "path")?;
    }
    if let Some(kind) = &args.kind {
        match kind.value().as_str() {
            "associated_const" | "associated_type" => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    kind,
                    "`kind` must be `associated_const` or `associated_type`",
                ));
            }
        }
    }
    if let Some(module) = &args.module {
        validate_path(module, "module")?;
    }
    let mut aliases = BTreeSet::new();
    for alias in args.aliases.as_deref().unwrap_or_default() {
        validate_path(alias, "alias")?;
        if !aliases.insert(alias.value()) {
            return Err(syn::Error::new_spanned(
                alias,
                "duplicate API contract alias",
            ));
        }
    }

    match target {
        ContractTarget::Function { signature, .. } => {
            reject_members(args, ident)?;
            validate_function(args, signature)
        }
        ContractTarget::Struct(item) => {
            reject_variant_docs(args, ident)?;
            validate_struct_fields(args, item)
        }
        ContractTarget::Enum(item) => {
            reject_field_docs(args, ident)?;
            validate_enum_variants(args, item)
        }
        ContractTarget::Plain { .. } => {
            reject_members(args, ident)?;
            reject_parameters_and_returns(args, ident)
        }
    }
}

fn reject_members(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    reject_field_docs(args, ident)?;
    reject_variant_docs(args, ident)
}

fn reject_field_docs(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(fields) = &args.fields {
        return Err(syn::Error::new_spanned(
            fields.first().map_or_else(
                || ident.to_token_stream(),
                |field| field.name.to_token_stream(),
            ),
            "field descriptions are only valid on structs with public named fields",
        ));
    }
    Ok(())
}

fn reject_variant_docs(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(variants) = &args.variants {
        return Err(syn::Error::new_spanned(
            variants.first().map_or_else(
                || ident.to_token_stream(),
                |variant| variant.name.to_token_stream(),
            ),
            "variant descriptions are only valid on enums",
        ));
    }
    Ok(())
}

fn reject_parameters_and_returns(args: &ContractArgs, ident: &syn::Ident) -> syn::Result<()> {
    if let Some(parameters) = &args.params {
        return Err(syn::Error::new_spanned(
            parameters.first().map_or_else(
                || ident.to_token_stream(),
                |parameter| parameter.name.to_token_stream(),
            ),
            "parameter descriptions are only valid on functions and methods",
        ));
    }
    if let Some(returns) = &args.returns {
        return Err(syn::Error::new_spanned(
            returns,
            "`returns` is only valid on functions and methods",
        ));
    }
    Ok(())
}

fn validate_function(args: &ContractArgs, signature: &Signature) -> syn::Result<()> {
    let mut actual = Vec::new();
    for argument in &signature.inputs {
        match argument {
            FnArg::Receiver(_) => {}
            FnArg::Typed(argument) => match argument.pat.as_ref() {
                Pat::Ident(ident) => actual.push(ident.ident.to_string()),
                pattern => {
                    return Err(syn::Error::new_spanned(
                        pattern,
                        "contracted public API parameters must use simple identifier patterns",
                    ));
                }
            },
        }
    }
    let documented = args
        .params
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|description| (description.name.to_string(), &description.name))
        .collect::<BTreeMap<_, _>>();
    for name in &actual {
        if !documented.contains_key(name) {
            return Err(syn::Error::new_spanned(
                signature,
                format!("missing API contract documentation for parameter `{name}`"),
            ));
        }
    }
    for (name, ident) in &documented {
        if !actual.contains(name) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("API contract documents nonexistent parameter `{name}`"),
            ));
        }
    }
    match (&signature.output, &args.returns) {
        (ReturnType::Default, Some(value)) => Err(syn::Error::new_spanned(
            value,
            "`returns` is not valid on a function without a return value",
        )),
        (ReturnType::Type(_, _), None) => Err(syn::Error::new_spanned(
            signature,
            "missing required API contract field `returns`",
        )),
        _ => Ok(()),
    }
}

fn validate_struct_fields(args: &ContractArgs, item: &ItemStruct) -> syn::Result<()> {
    reject_parameters_and_returns(args, &item.ident)?;
    let mut actual = BTreeMap::new();
    for field in &item.fields {
        if matches!(field.vis, syn::Visibility::Public(_)) && !doc_hidden(&field.attrs) {
            let Some(ident) = &field.ident else {
                return Err(syn::Error::new_spanned(
                    field,
                    "contracted public tuple fields are unsupported; use named fields or make them private",
                ));
            };
            actual.insert(ident.to_string(), ident);
        }
    }
    validate_members(
        "field",
        args.fields.as_deref().unwrap_or_default(),
        &actual,
        &item.ident,
    )
}

fn validate_enum_variants(args: &ContractArgs, item: &ItemEnum) -> syn::Result<()> {
    reject_parameters_and_returns(args, &item.ident)?;
    let actual = item
        .variants
        .iter()
        .filter(|variant| !doc_hidden(&variant.attrs))
        .map(|variant| (variant.ident.to_string(), &variant.ident))
        .collect::<BTreeMap<_, _>>();
    validate_members(
        "variant",
        args.variants.as_deref().unwrap_or_default(),
        &actual,
        &item.ident,
    )
}

fn validate_members(
    kind: &str,
    documented: &[Description],
    actual: &BTreeMap<String, &syn::Ident>,
    parent: &syn::Ident,
) -> syn::Result<()> {
    let docs = documented
        .iter()
        .map(|doc| (doc.name.to_string(), &doc.name))
        .collect::<BTreeMap<_, _>>();
    for (name, ident) in actual {
        if !docs.contains_key(name) {
            return Err(syn::Error::new_spanned(
                *ident,
                format!("missing API contract documentation for {kind} `{name}`"),
            ));
        }
    }
    for (name, ident) in docs {
        if !actual.contains_key(&name) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("API contract documents nonexistent {kind} `{name}` on `{parent}`"),
            ));
        }
    }
    Ok(())
}

fn doc_hidden(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "hidden")
    })
}

pub fn validate_path(value: &LitStr, role: &str) -> syn::Result<()> {
    let path = value.value();
    let valid = (role == "module" && path == "sand")
        || path.starts_with("sand::")
            && path.split("::").all(|segment| {
                !segment.is_empty()
                    && segment.chars().enumerate().all(|(index, ch)| {
                        ch == '_'
                            || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit())
                    })
            });
    if valid {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("invalid canonical API {role}; expected a path beginning with `sand::`"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parses_nested_member_descriptions() {
        let args = parse_contract_args(quote!(
            summary = "Value.",
            context = "Context.",
            minecraft = "Behavior.",
            use_when = ["Useful."],
            avoid_when = ["Not useful."],
            example = "Value::A",
            variants(A = "The first state.", B = "The second state.")
        ))
        .unwrap();
        assert_eq!(args.variants.unwrap().len(), 2);
    }
}
