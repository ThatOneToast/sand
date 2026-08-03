use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::{BTreeMap, BTreeSet};
use syn::parse::Parser;
use syn::{
    ExprArray, FnArg, ImplItemFn, Item, LitStr, Pat, ReturnType, Signature, TraitItemFn, parse2,
};

#[derive(Default)]
struct ContractArgs {
    path: Option<LitStr>,
    module: Option<LitStr>,
    aliases: Option<Vec<LitStr>>,
    summary: Option<LitStr>,
    context: Option<LitStr>,
    minecraft: Option<LitStr>,
    use_when: Option<Vec<LitStr>>,
    avoid_when: Option<Vec<LitStr>>,
    params: Option<Vec<(syn::Ident, LitStr)>>,
    returns: Option<LitStr>,
    example: Option<LitStr>,
    availability: Option<Vec<LitStr>>,
}

enum Target {
    Item(Item),
    ImplMethod(ImplItemFn),
    TraitMethod(TraitItemFn),
}

struct TargetInfo<'a> {
    ident: &'a syn::Ident,
    kind: &'static str,
    signature: TokenStream,
    function: Option<&'a Signature>,
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = parse_args(attr)?;
    let target = parse_target(item.clone())?;
    let info = target_info(&target)?;
    validate(&args, &info)?;

    let summary = required(&args.summary, "summary")?;
    let context = required(&args.context, "context")?;
    let minecraft = required(&args.minecraft, "minecraft")?;
    let use_when = required(&args.use_when, "use_when")?;
    let avoid_when = required(&args.avoid_when, "avoid_when")?;
    let example = required(&args.example, "example")?;
    let aliases = args.aliases.as_deref().unwrap_or_default();
    let availability = args.availability.as_deref().unwrap_or_default();
    let parameters = args.params.as_deref().unwrap_or_default();
    let parameter_names = parameters.iter().map(|(name, _)| name.to_string());
    let parameter_docs = parameters.iter().map(|(_, description)| description);
    let returns = match &args.returns {
        Some(value) => quote!(::std::option::Option::Some(#value)),
        None => quote!(::std::option::Option::None),
    };
    let kind = syn::Ident::new(info.kind, info.ident.span());
    let identity = args
        .path
        .as_ref()
        .map_or_else(|| info.ident.to_string(), LitStr::value);
    let registration = syn::Ident::new(
        &format!("__SAND_API_CONTRACT_{:016X}", fnv1a(identity.as_bytes())),
        info.ident.span(),
    );
    let signature = &info.signature;
    let (path, module) = identity_tokens(&args, info.ident)?;
    let docs = rustdoc(&args, &path, summary, context, minecraft, example);

    Ok(quote! {
        #docs
        #target

        #[doc(hidden)]
        const #registration: () = {
            ::sand::__private::api_contract::inventory::submit! {
                ::sand::__private::api_contract::ApiRegistration {
                    canonical_path: #path,
                    aliases: &[#(#aliases),*],
                    canonical_module: #module,
                    kind: ::sand::__private::api_contract::ApiKind::#kind,
                    signature: ::std::stringify!(#signature),
                    summary: #summary,
                    context: #context,
                    minecraft: #minecraft,
                    use_when: &[#(#use_when),*],
                    avoid_when: &[#(#avoid_when),*],
                    parameters: &[
                        #(
                            ::sand::__private::api_contract::StaticApiParameter {
                                name: #parameter_names,
                                description: #parameter_docs,
                            }
                        ),*
                    ],
                    returns: #returns,
                    example: #example,
                    availability: &[#(#availability),*],
                }
            }
        };
    })
}

impl ToTokens for Target {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Item(item) => item.to_tokens(tokens),
            Self::ImplMethod(item) => item.to_tokens(tokens),
            Self::TraitMethod(item) => item.to_tokens(tokens),
        }
    }
}

fn parse_args(tokens: TokenStream) -> syn::Result<ContractArgs> {
    let mut args = ContractArgs::default();
    let parser = syn::meta::parser(|meta| {
        let key = meta
            .path
            .get_ident()
            .ok_or_else(|| meta.error("API contract keys must be identifiers"))?
            .to_string();
        match key.as_str() {
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
            "aliases" => {
                let value = parse_string_array(&meta)?;
                set_once(&mut args.aliases, value, &meta, "aliases")
            }
            "use_when" => {
                let value = parse_string_array(&meta)?;
                set_once(&mut args.use_when, value, &meta, "use_when")
            }
            "avoid_when" => {
                let value = parse_string_array(&meta)?;
                set_once(&mut args.avoid_when, value, &meta, "avoid_when")
            }
            "availability" => {
                let value = parse_string_array(&meta)?;
                set_once(&mut args.availability, value, &meta, "availability")
            }
            "params" => {
                if args.params.is_some() {
                    return Err(meta.error("duplicate API contract field `params`"));
                }
                let mut parameters = Vec::new();
                let mut names = BTreeSet::new();
                meta.parse_nested_meta(|parameter| {
                    let name =
                        parameter.path.get_ident().cloned().ok_or_else(|| {
                            parameter.error("parameter names must be identifiers")
                        })?;
                    if !names.insert(name.to_string()) {
                        return Err(parameter
                            .error(format!("duplicate parameter documentation for `{name}`")));
                    }
                    parameters.push((name, parameter.value()?.parse()?));
                    Ok(())
                })?;
                args.params = Some(parameters);
                Ok(())
            }
            _ => Err(meta.error(format!("unknown API contract field `{key}`"))),
        }
    });
    parser.parse2(tokens)?;
    Ok(args)
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

fn parse_target(tokens: TokenStream) -> syn::Result<Target> {
    if let Ok(item) = parse2::<Item>(tokens.clone())
        && !matches!(item, Item::Verbatim(_))
    {
        return Ok(Target::Item(item));
    }
    if let Ok(item) = parse2::<ImplItemFn>(tokens.clone()) {
        return Ok(Target::ImplMethod(item));
    }
    if let Ok(item) = parse2::<TraitItemFn>(tokens.clone()) {
        return Ok(Target::TraitMethod(item));
    }
    Err(syn::Error::new_spanned(
        tokens,
        "#[api] supports modules, structs, enums, traits, functions, methods, trait methods, type aliases, constants, and macros",
    ))
}

fn target_info(target: &Target) -> syn::Result<TargetInfo<'_>> {
    let info = match target {
        Target::ImplMethod(item) => TargetInfo {
            ident: &item.sig.ident,
            kind: "Method",
            signature: item.sig.to_token_stream(),
            function: Some(&item.sig),
        },
        Target::TraitMethod(item) => TargetInfo {
            ident: &item.sig.ident,
            kind: "TraitMethod",
            signature: item.sig.to_token_stream(),
            function: Some(&item.sig),
        },
        Target::Item(item) => match item {
            Item::Fn(item) => TargetInfo {
                ident: &item.sig.ident,
                kind: if item.sig.receiver().is_some() {
                    "Method"
                } else {
                    "Function"
                },
                signature: item.sig.to_token_stream(),
                function: Some(&item.sig),
            },
            Item::Mod(item) => {
                let ident = &item.ident;
                TargetInfo {
                    ident,
                    kind: "Module",
                    signature: quote!(pub mod #ident),
                    function: None,
                }
            }
            Item::Struct(item) => TargetInfo {
                ident: &item.ident,
                kind: "Struct",
                signature: item.to_token_stream(),
                function: None,
            },
            Item::Enum(item) => TargetInfo {
                ident: &item.ident,
                kind: "Enum",
                signature: item.to_token_stream(),
                function: None,
            },
            Item::Trait(item) => TargetInfo {
                ident: &item.ident,
                kind: "Trait",
                signature: item.to_token_stream(),
                function: None,
            },
            Item::Type(item) => TargetInfo {
                ident: &item.ident,
                kind: "TypeAlias",
                signature: item.to_token_stream(),
                function: None,
            },
            Item::Const(item) => TargetInfo {
                ident: &item.ident,
                kind: "Constant",
                signature: item.to_token_stream(),
                function: None,
            },
            Item::Macro(item) => TargetInfo {
                ident: item.ident.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(item, "contracted macros must have a name")
                })?,
                kind: "Macro",
                signature: item.to_token_stream(),
                function: None,
            },
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported #[api] item form",
                ));
            }
        },
    };
    Ok(info)
}

fn validate(args: &ContractArgs, info: &TargetInfo<'_>) -> syn::Result<()> {
    for (name, value) in [
        ("summary", args.summary.as_ref()),
        ("context", args.context.as_ref()),
        ("minecraft", args.minecraft.as_ref()),
        ("example", args.example.as_ref()),
    ] {
        let value = value.ok_or_else(|| {
            syn::Error::new(
                info.ident.span(),
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
                info.ident.span(),
                format!("missing required API contract field `{name}`"),
            )
        })?;
        if values.is_empty() || values.iter().any(|value| value.value().trim().is_empty()) {
            return Err(syn::Error::new(
                info.ident.span(),
                format!("API contract field `{name}` must contain non-empty strings"),
            ));
        }
    }
    if let Some(path) = &args.path {
        validate_path(path, "path")?;
    }
    if let Some(module) = &args.module {
        validate_path(module, "module")?;
    }
    for alias in args.aliases.as_deref().unwrap_or_default() {
        validate_path(alias, "alias")?;
    }
    let mut aliases = BTreeSet::new();
    for alias in args.aliases.as_deref().unwrap_or_default() {
        if !aliases.insert(alias.value()) {
            return Err(syn::Error::new_spanned(
                alias,
                "duplicate API contract alias",
            ));
        }
    }

    match info.function {
        Some(signature) => validate_function(args, signature),
        None => {
            if let Some(parameters) = &args.params {
                return Err(syn::Error::new_spanned(
                    parameters.first().map_or_else(
                        || info.ident.to_token_stream(),
                        |parameter| parameter.0.to_token_stream(),
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
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
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
        .map(|(name, _)| (name.to_string(), name))
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

fn validate_path(value: &LitStr, role: &str) -> syn::Result<()> {
    let path = value.value();
    let valid = path.starts_with("sand::")
        && path.split("::").all(|segment| {
            !segment.is_empty()
                && segment.chars().enumerate().all(|(index, ch)| {
                    ch == '_' || ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit())
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

fn identity_tokens(
    args: &ContractArgs,
    ident: &syn::Ident,
) -> syn::Result<(TokenStream, TokenStream)> {
    if let Some(path) = &args.path {
        let value = path.value();
        let module = match &args.module {
            Some(module) => module.clone(),
            None => LitStr::new(
                value.rsplit_once("::").map_or("sand", |(module, _)| module),
                path.span(),
            ),
        };
        Ok((quote!(#path), quote!(#module)))
    } else {
        let module = match &args.module {
            Some(module) => quote!(#module),
            None => quote!(::std::module_path!()),
        };
        Ok((
            quote!(::std::concat!(
                ::std::module_path!(),
                "::",
                ::std::stringify!(#ident)
            )),
            module,
        ))
    }
}

fn required<'a, T>(value: &'a Option<T>, name: &str) -> syn::Result<&'a T> {
    value.as_ref().ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("missing required API contract field `{name}`"),
        )
    })
}

fn rustdoc(
    args: &ContractArgs,
    path: &TokenStream,
    summary: &LitStr,
    context: &LitStr,
    minecraft: &LitStr,
    example: &LitStr,
) -> TokenStream {
    let mut lines = vec![
        summary.value(),
        String::new(),
        "# Context".into(),
        context.value(),
    ];
    lines.extend([
        String::new(),
        "# Minecraft behavior".into(),
        minecraft.value(),
    ]);
    if let Some(parameters) = &args.params {
        lines.extend([String::new(), "# Parameters".into()]);
        lines.extend(
            parameters
                .iter()
                .map(|(name, description)| format!("- `{name}` — {}", description.value())),
        );
    }
    if let Some(returns) = &args.returns {
        lines.extend([String::new(), "# Returns".into(), returns.value()]);
    }
    lines.extend([String::new(), "# Use when".into()]);
    lines.extend(
        args.use_when
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|value| format!("- {}", value.value())),
    );
    lines.extend([String::new(), "# Avoid when".into()]);
    lines.extend(
        args.avoid_when
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|value| format!("- {}", value.value())),
    );
    lines.extend([
        String::new(),
        "# Example".into(),
        "```rust,ignore".into(),
        example.value().trim().into(),
        "```".into(),
        String::new(),
        "# API Contract".into(),
    ]);
    let docs = lines.iter().map(|line| quote!(#[doc = #line]));
    quote! {
        #(#docs)*
        #[doc = ""]
        #[doc = "View this API with:"]
        #[doc = "```text"]
        #[doc = ::std::concat!("sand api show ", #path)]
        #[doc = "```"]
    }
}
