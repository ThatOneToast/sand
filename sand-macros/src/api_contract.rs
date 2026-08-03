use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use sand_api_contract::syntax::{
    ContractArgs, ContractTarget, Description, parse_contract_args, validate_contract,
};
use syn::{
    ImplItemConst, ImplItemFn, ImplItemType, Item, LitStr, TraitItemConst, TraitItemFn,
    TraitItemType, parse2,
};

enum Target {
    Item(Item),
    ImplMethod(ImplItemFn),
    ImplConst(ImplItemConst),
    ImplType(ImplItemType),
    TraitMethod(TraitItemFn),
    TraitConst(TraitItemConst),
    TraitType(TraitItemType),
}

struct TargetInfo<'a> {
    ident: &'a syn::Ident,
    kind: &'static str,
    signature: TokenStream,
    contract_target: ContractTarget<'a>,
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = parse_contract_args(attr)?;
    let mut target = parse_target(item.clone(), args.kind.as_ref())?;
    let info = target_info(&target, &args)?;
    validate_contract(&args, &info.contract_target)?;

    let summary = required(&args.summary, "summary")?;
    let context = required(&args.context, "context")?;
    let minecraft = required(&args.minecraft, "minecraft")?;
    let use_when = required(&args.use_when, "use_when")?;
    let avoid_when = required(&args.avoid_when, "avoid_when")?;
    let example = required(&args.example, "example")?;
    let aliases = args.aliases.as_deref().unwrap_or_default();
    let availability = args.availability.as_deref().unwrap_or_default();
    let parameters = args.params.as_deref().unwrap_or_default();
    let parameter_names = parameters
        .iter()
        .map(|parameter| parameter.name.to_string());
    let parameter_docs = parameters.iter().map(|parameter| &parameter.text);
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
    let member_registrations = member_registrations(&args, &target, &path, aliases, &info)?;
    add_member_rustdoc(&mut target, &args);

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

        #member_registrations
    })
}

impl ToTokens for Target {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Item(item) => item.to_tokens(tokens),
            Self::ImplMethod(item) => item.to_tokens(tokens),
            Self::ImplConst(item) => item.to_tokens(tokens),
            Self::ImplType(item) => item.to_tokens(tokens),
            Self::TraitMethod(item) => item.to_tokens(tokens),
            Self::TraitConst(item) => item.to_tokens(tokens),
            Self::TraitType(item) => item.to_tokens(tokens),
        }
    }
}

fn parse_target(tokens: TokenStream, kind: Option<&LitStr>) -> syn::Result<Target> {
    match kind.map(LitStr::value).as_deref() {
        Some("associated_const") => return parse2(tokens).map(Target::ImplConst),
        Some("associated_type") => return parse2(tokens).map(Target::ImplType),
        _ => {}
    }
    if let Ok(item) = parse2::<Item>(tokens.clone())
        && !matches!(item, Item::Verbatim(_))
    {
        return Ok(Target::Item(item));
    }
    if let Ok(item) = parse2::<ImplItemFn>(tokens.clone()) {
        return Ok(Target::ImplMethod(item));
    }
    if let Ok(item) = parse2::<ImplItemConst>(tokens.clone()) {
        return Ok(Target::ImplConst(item));
    }
    if let Ok(item) = parse2::<ImplItemType>(tokens.clone()) {
        return Ok(Target::ImplType(item));
    }
    if let Ok(item) = parse2::<TraitItemFn>(tokens.clone()) {
        return Ok(Target::TraitMethod(item));
    }
    if let Ok(item) = parse2::<TraitItemConst>(tokens.clone()) {
        return Ok(Target::TraitConst(item));
    }
    if let Ok(item) = parse2::<TraitItemType>(tokens.clone()) {
        return Ok(Target::TraitType(item));
    }
    Err(syn::Error::new_spanned(
        tokens,
        "#[api] supports modules, structs, enums, traits, functions, methods, associated constants/types, trait methods/items, type aliases, constants, and macros",
    ))
}

fn target_info<'a>(target: &'a Target, args: &ContractArgs) -> syn::Result<TargetInfo<'a>> {
    if !matches!(target, Target::ImplConst(_) | Target::ImplType(_))
        && let Some(kind) = &args.kind
    {
        return Err(syn::Error::new_spanned(
            kind,
            "`kind` is only valid for inherent associated constants and types",
        ));
    }
    let info = match target {
        Target::ImplMethod(item) => TargetInfo {
            ident: &item.sig.ident,
            kind: "Method",
            signature: item.sig.to_token_stream(),
            contract_target: ContractTarget::Function {
                ident: &item.sig.ident,
                signature: &item.sig,
            },
        },
        Target::ImplConst(item) => TargetInfo {
            ident: &item.ident,
            kind: "AssociatedConst",
            signature: item.to_token_stream(),
            contract_target: ContractTarget::Plain { ident: &item.ident },
        },
        Target::ImplType(item) => TargetInfo {
            ident: &item.ident,
            kind: "AssociatedType",
            signature: item.to_token_stream(),
            contract_target: ContractTarget::Plain { ident: &item.ident },
        },
        Target::TraitMethod(item) => TargetInfo {
            ident: &item.sig.ident,
            kind: "TraitMethod",
            signature: item.sig.to_token_stream(),
            contract_target: ContractTarget::Function {
                ident: &item.sig.ident,
                signature: &item.sig,
            },
        },
        Target::TraitConst(item) => TargetInfo {
            ident: &item.ident,
            kind: "AssociatedConst",
            signature: item.to_token_stream(),
            contract_target: ContractTarget::Plain { ident: &item.ident },
        },
        Target::TraitType(item) => TargetInfo {
            ident: &item.ident,
            kind: "AssociatedType",
            signature: item.to_token_stream(),
            contract_target: ContractTarget::Plain { ident: &item.ident },
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
                contract_target: ContractTarget::Function {
                    ident: &item.sig.ident,
                    signature: &item.sig,
                },
            },
            Item::Mod(item) => {
                let ident = &item.ident;
                TargetInfo {
                    ident,
                    kind: "Module",
                    signature: quote!(pub mod #ident),
                    contract_target: ContractTarget::Plain { ident },
                }
            }
            Item::Struct(item) => TargetInfo {
                ident: &item.ident,
                kind: "Struct",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Struct(item),
            },
            Item::Enum(item) => TargetInfo {
                ident: &item.ident,
                kind: "Enum",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Enum(item),
            },
            Item::Trait(item) => TargetInfo {
                ident: &item.ident,
                kind: "Trait",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Plain { ident: &item.ident },
            },
            Item::Type(item) => TargetInfo {
                ident: &item.ident,
                kind: "TypeAlias",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Plain { ident: &item.ident },
            },
            Item::Const(item) => TargetInfo {
                ident: &item.ident,
                kind: "Constant",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Plain { ident: &item.ident },
            },
            Item::Macro(item) => TargetInfo {
                ident: item.ident.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(item, "contracted macros must have a name")
                })?,
                kind: "Macro",
                signature: item.to_token_stream(),
                contract_target: ContractTarget::Plain {
                    ident: item.ident.as_ref().expect("checked named macro"),
                },
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

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn member_registrations(
    args: &ContractArgs,
    target: &Target,
    parent_path: &TokenStream,
    parent_aliases: &[LitStr],
    info: &TargetInfo<'_>,
) -> syn::Result<TokenStream> {
    let context = required(&args.context, "context")?;
    let minecraft = required(&args.minecraft, "minecraft")?;
    let use_when = required(&args.use_when, "use_when")?;
    let avoid_when = required(&args.avoid_when, "avoid_when")?;
    let example = required(&args.example, "example")?;
    let availability = args.availability.as_deref().unwrap_or_default();
    let mut registrations = Vec::new();

    match target {
        Target::Item(Item::Struct(item)) => {
            for field in &item.fields {
                if !matches!(field.vis, syn::Visibility::Public(_)) || doc_hidden(&field.attrs) {
                    continue;
                }
                let ident = field
                    .ident
                    .as_ref()
                    .expect("shared validation rejects tuple fields");
                let description = description(args.fields.as_deref(), ident)?;
                let ty = &field.ty;
                let signature = quote!(pub #ident: #ty);
                registrations.push(member_registration(
                    info,
                    ident,
                    "Field",
                    signature,
                    description,
                    &member_path(args, info.ident, ident),
                    parent_path,
                    parent_aliases,
                    context,
                    minecraft,
                    use_when,
                    avoid_when,
                    example,
                    availability,
                ));
            }
        }
        Target::Item(Item::Enum(item)) => {
            for variant in &item.variants {
                if doc_hidden(&variant.attrs) {
                    continue;
                }
                let ident = &variant.ident;
                let description = description(args.variants.as_deref(), ident)?;
                let mut signature_variant = variant.clone();
                signature_variant.attrs.clear();
                registrations.push(member_registration(
                    info,
                    ident,
                    "Variant",
                    signature_variant.to_token_stream(),
                    description,
                    &member_path(args, info.ident, ident),
                    parent_path,
                    parent_aliases,
                    context,
                    minecraft,
                    use_when,
                    avoid_when,
                    example,
                    availability,
                ));
            }
        }
        _ => {}
    }

    Ok(quote!(#(#registrations)*))
}

#[allow(clippy::too_many_arguments)]
fn member_registration(
    info: &TargetInfo<'_>,
    ident: &syn::Ident,
    kind: &str,
    signature: TokenStream,
    summary: &LitStr,
    member_path: &TokenStream,
    parent_path: &TokenStream,
    parent_aliases: &[LitStr],
    context: &LitStr,
    minecraft: &LitStr,
    use_when: &[LitStr],
    avoid_when: &[LitStr],
    example: &LitStr,
    availability: &[LitStr],
) -> TokenStream {
    let member_name = ident.to_string();
    let identity = format!("{}::{member_name}", info.ident);
    let registration = syn::Ident::new(
        &format!("__SAND_API_CONTRACT_{:016X}", fnv1a(identity.as_bytes())),
        ident.span(),
    );
    let kind = syn::Ident::new(kind, ident.span());
    let aliases = parent_aliases
        .iter()
        .map(|alias| LitStr::new(&format!("{}::{member_name}", alias.value()), alias.span()));
    quote! {
        #[doc(hidden)]
        const #registration: () = {
            ::sand::__private::api_contract::inventory::submit! {
                ::sand::__private::api_contract::ApiRegistration {
                    canonical_path: #member_path,
                    aliases: &[#(#aliases),*],
                    canonical_module: #parent_path,
                    kind: ::sand::__private::api_contract::ApiKind::#kind,
                    signature: ::std::stringify!(#signature),
                    summary: #summary,
                    context: #context,
                    minecraft: #minecraft,
                    use_when: &[#(#use_when),*],
                    avoid_when: &[#(#avoid_when),*],
                    parameters: &[],
                    returns: ::std::option::Option::None,
                    example: #example,
                    availability: &[#(#availability),*],
                }
            }
        };
    }
}

fn member_path(args: &ContractArgs, parent: &syn::Ident, member: &syn::Ident) -> TokenStream {
    if let Some(path) = &args.path {
        let value = LitStr::new(&format!("{}::{member}", path.value()), member.span());
        quote!(#value)
    } else {
        quote!(::std::concat!(
            ::std::module_path!(),
            "::",
            ::std::stringify!(#parent),
            "::",
            ::std::stringify!(#member)
        ))
    }
}

fn description<'a>(
    descriptions: Option<&'a [Description]>,
    ident: &syn::Ident,
) -> syn::Result<&'a LitStr> {
    descriptions
        .unwrap_or_default()
        .iter()
        .find(|description| description.name == *ident)
        .map(|description| &description.text)
        .ok_or_else(|| syn::Error::new_spanned(ident, "missing nested member description"))
}

fn add_member_rustdoc(target: &mut Target, args: &ContractArgs) {
    match target {
        Target::Item(Item::Struct(item)) => {
            for field in &mut item.fields {
                let Some(ident) = &field.ident else { continue };
                if let Some(description) = args
                    .fields
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .find(|description| description.name == *ident)
                {
                    let text = &description.text;
                    field.attrs.push(syn::parse_quote!(#[doc = #text]));
                    field.attrs.push(syn::parse_quote!(#[doc = ""]));
                    field
                        .attrs
                        .push(member_contract_doc(args, &item.ident, ident));
                }
            }
        }
        Target::Item(Item::Enum(item)) => {
            for variant in &mut item.variants {
                let ident = &variant.ident;
                if let Some(description) = args
                    .variants
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .find(|description| description.name == *ident)
                {
                    let text = &description.text;
                    variant.attrs.push(syn::parse_quote!(#[doc = #text]));
                    variant.attrs.push(syn::parse_quote!(#[doc = ""]));
                    variant
                        .attrs
                        .push(member_contract_doc(args, &item.ident, ident));
                }
            }
        }
        _ => {}
    }
}

fn member_contract_doc(
    args: &ContractArgs,
    parent: &syn::Ident,
    member: &syn::Ident,
) -> syn::Attribute {
    if let Some(path) = &args.path {
        let line = LitStr::new(
            &format!("API Contract: `sand api show {}::{member}`", path.value()),
            member.span(),
        );
        syn::parse_quote!(#[doc = #line])
    } else {
        syn::parse_quote!(#[doc = concat!(
            "API Contract: `sand api show ",
            module_path!(),
            "::",
            stringify!(#parent),
            "::",
            stringify!(#member),
            "`"
        )])
    }
}

fn doc_hidden(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "hidden")
    })
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
                .map(|parameter| format!("- `{}` — {}", parameter.name, parameter.text.value())),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inherent_associated_const_and_type_forms() {
        assert!(matches!(
            parse_target(
                quote!(
                    pub const ENABLED: bool = true;
                ),
                Some(&syn::parse_quote!("associated_const"))
            )
            .unwrap(),
            Target::ImplConst(_)
        ));
        assert!(matches!(
            parse_target(
                quote!(
                    pub type Output = u32;
                ),
                Some(&syn::parse_quote!("associated_type"))
            )
            .unwrap(),
            Target::ImplType(_)
        ));
    }
}
