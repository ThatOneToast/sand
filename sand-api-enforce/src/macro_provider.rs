//! Providers for checked-in declarative macros that emit facade API families.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser};

use crate::reachable::{
    CfgSet, cfg_enabled, effective_attribute_metas, module_path, module_search_directory,
};
use crate::{GeneratedApi, GeneratedProducer, ReachableKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MacroProviderError {
    Io(String),
    Parse(String),
    MissingGenerator,
    MissingGeneratedTypes,
    MissingPublicMethods,
    MissingNamedGenerator(String),
    MissingGeneratedVariants,
    MissingGeneratedEvents,
    UnsupportedGeneratedShape(String),
    MissingConsumerInvocation(String),
}

/// Audit a real downstream use of a Sand macro whose expansion deliberately
/// preserves the annotated public declaration's shape.
///
/// `function`, `datapack_component`, `on_event`, `armor_event`, and
/// `schedule` only add private descriptor factories and inventory wiring; the
/// author's function remains the sole supported Rust identity. Likewise,
/// `EntityStateEnum` adds a trait implementation only. This audit refuses an
/// empty fixture, preventing a consumer-build scope from being promoted merely
/// because it happens to have no finite checked-in declarations.
pub fn shape_preserving_consumer_provider(
    path: &Path,
    macro_name: &str,
    cfg: &CfgSet,
) -> Result<(), MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let found = contains_shape_preserving_invocation(
        &file.items,
        macro_name,
        path,
        &module_search_directory(path),
        path.parent().unwrap_or_else(|| Path::new(".")),
        cfg,
    )?;
    if found {
        Ok(())
    } else {
        Err(MacroProviderError::MissingConsumerInvocation(
            macro_name.to_owned(),
        ))
    }
}

fn contains_shape_preserving_invocation(
    items: &[syn::Item],
    macro_name: &str,
    source_file: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
) -> Result<bool, MacroProviderError> {
    for item in items {
        if !provider_item_enabled(item, cfg, source_file)? {
            continue;
        }
        if let Some((path, file)) = parse_provider_include(item, source_file)? {
            if contains_shape_preserving_invocation(
                &file.items,
                macro_name,
                &path,
                &module_search_directory(&path),
                path.parent().unwrap_or_else(|| Path::new(".")),
                cfg,
            )? {
                return Ok(true);
            }
            continue;
        }
        let found = match item {
            syn::Item::Fn(function) => {
                is_public(&function.vis)
                    && provider_has_attribute_named(&function.attrs, macro_name, cfg, source_file)?
            }
            syn::Item::Enum(enumeration) if macro_name == "EntityStateEnum" => {
                is_public(&enumeration.vis)
                    && provider_derives_named(&enumeration.attrs, macro_name, cfg, source_file)?
            }
            syn::Item::Mod(module) => {
                let name = module.ident.unraw().to_string();
                if let Some((_, items)) = &module.content {
                    let child_directory = default_directory.join(name);
                    contains_shape_preserving_invocation(
                        items,
                        macro_name,
                        source_file,
                        &child_directory,
                        &child_directory,
                        cfg,
                    )?
                } else {
                    let (path, file) = parse_provider_module(
                        module,
                        source_file,
                        default_directory,
                        path_directory,
                        cfg,
                    )?;
                    contains_shape_preserving_invocation(
                        &file.items,
                        macro_name,
                        &path,
                        &module_search_directory(&path),
                        path.parent().unwrap_or_else(|| Path::new(".")),
                        cfg,
                    )?
                }
            }
            _ => false,
        };
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Describe the public HUD handle constants emitted by feature-gated
/// `hud_bar!` and `hud_element!` calls. `texture!` intentionally produces no
/// Rust-visible API: it only registers a pack asset. Parsing the same literal
/// `name` input the macros use for their uppercased handle keeps consumer
/// enforcement exact without pretending raw texture registrations are APIs.
pub fn resourcepack_macro_provider(
    path: &Path,
    identity_module: &str,
    cfg: &CfgSet,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let mut generated = Vec::new();
    let saw_resourcepack_macro = collect_resourcepack_macros(
        &file.items,
        identity_module,
        path,
        &module_search_directory(path),
        path.parent().unwrap_or_else(|| Path::new(".")),
        cfg,
        &mut generated,
    )?;
    if !saw_resourcepack_macro {
        return Err(MacroProviderError::MissingConsumerInvocation(
            "resourcepack macro".into(),
        ));
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    generated.dedup_by(|left, right| left.identity == right.identity);
    Ok(generated)
}

fn collect_resourcepack_macros(
    items: &[syn::Item],
    identity_module: &str,
    path: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
    generated: &mut Vec<GeneratedApi>,
) -> Result<bool, MacroProviderError> {
    let mut saw_resourcepack_macro = false;
    for item in items {
        if !provider_item_enabled(item, cfg, path)? {
            continue;
        }
        if let Some((included_path, file)) = parse_provider_include(item, path)? {
            saw_resourcepack_macro |= collect_resourcepack_macros(
                &file.items,
                identity_module,
                &included_path,
                &module_search_directory(&included_path),
                included_path.parent().unwrap_or_else(|| Path::new(".")),
                cfg,
                generated,
            )?;
            continue;
        }
        if let syn::Item::Mod(module) = item {
            let nested_module = format!("{identity_module}::{}", module.ident.unraw());
            if let Some((_, items)) = &module.content {
                let child_directory = default_directory.join(module.ident.unraw().to_string());
                saw_resourcepack_macro |= collect_resourcepack_macros(
                    items,
                    &nested_module,
                    path,
                    &child_directory,
                    &child_directory,
                    cfg,
                    generated,
                )?;
            } else {
                let (nested_path, file) =
                    parse_provider_module(module, path, default_directory, path_directory, cfg)?;
                saw_resourcepack_macro |= collect_resourcepack_macros(
                    &file.items,
                    &nested_module,
                    &nested_path,
                    &module_search_directory(&nested_path),
                    nested_path.parent().unwrap_or_else(|| Path::new(".")),
                    cfg,
                    generated,
                )?;
            }
            continue;
        }
        let syn::Item::Macro(item) = item else {
            continue;
        };
        let Some(name) = item
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            continue;
        };
        match name.as_str() {
            "hud_bar" | "hud_element" => {
                saw_resourcepack_macro = true;
                let handle = resourcepack_handle_name(item.mac.tokens.clone(), path)?;
                generated.push(GeneratedApi {
                    identity: format!("{identity_module}::{handle}"),
                    provider: "resourcepack_macros".into(),
                    producer: None,
                    kind: ReachableKind::Constant,
                    members: Vec::new(),
                    excluded: false,
                });
            }
            "texture" => saw_resourcepack_macro = true,
            _ => {}
        }
    }
    Ok(saw_resourcepack_macro)
}

fn resourcepack_handle_name(
    tokens: TokenStream,
    path: &Path,
) -> Result<String, MacroProviderError> {
    let fields = syn::punctuated::Punctuated::<syn::ExprAssign, syn::Token![,]>::parse_terminated
        .parse2(tokens)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let name = fields
        .iter()
        .rev()
        .find_map(|field| match field.left.as_ref() {
            syn::Expr::Path(path) if path.path.is_ident("name") => match field.right.as_ref() {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }) => Some(value.value()),
                _ => None,
            },
            _ => None,
        })
        .ok_or_else(|| {
            MacroProviderError::Parse(format!(
                "{}: resourcepack HUD macro needs a literal name = \"...\"",
                path.display()
            ))
        })?;
    let handle = name.to_uppercase().replace(['-', ' '], "_");
    if syn::parse_str::<syn::Ident>(&handle).is_err() {
        return Err(MacroProviderError::Parse(format!(
            "{}: resourcepack HUD name `{name}` does not produce a valid public Rust handle",
            path.display()
        )));
    }
    Ok(handle)
}

/// Verify that a `texture!` invocation has the exact semantic inputs needed
/// by the macro while acknowledging that it emits no supported Rust item.
pub(crate) fn audit_resourcepack_texture_invocation(
    tokens: &TokenStream,
) -> Result<(), MacroProviderError> {
    let fields = syn::punctuated::Punctuated::<syn::ExprAssign, syn::Token![,]>::parse_terminated
        .parse2(tokens.clone())
        .map_err(|error| MacroProviderError::Parse(error.to_string()))?;
    let literal = |key: &str| {
        fields
            .iter()
            .rev()
            .find_map(|field| match field.left.as_ref() {
                syn::Expr::Path(path) if path.path.is_ident(key) => match field.right.as_ref() {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }) => Some(Ok(value.value())),
                    _ => Some(Err(MacroProviderError::Parse(format!(
                        "`{key}` must be a string literal in texture!"
                    )))),
                },
                _ => None,
            })
            .unwrap_or_else(|| {
                Err(MacroProviderError::Parse(format!(
                    "`{key}` is required in texture!"
                )))
            })
    };
    let id = literal("id")?;
    let _path = literal("path")?;
    if id.split_once(':').is_none() {
        return Err(MacroProviderError::Parse(format!(
            "`id` must be a resource location `namespace:path`, got `{id}`"
        )));
    }
    Ok(())
}

fn parse_provider_module(
    module: &syn::ItemMod,
    source_file: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
) -> Result<(std::path::PathBuf, syn::File), MacroProviderError> {
    let path = module_path(
        &module.attrs,
        default_directory,
        path_directory,
        &module.ident.unraw().to_string(),
        cfg,
        source_file,
    )
    .map_err(|error| MacroProviderError::Parse(error.to_string()))?;
    let source = fs::read_to_string(&path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    Ok((path, file))
}

fn parse_provider_include(
    item: &syn::Item,
    source_file: &Path,
) -> Result<Option<(std::path::PathBuf, syn::File)>, MacroProviderError> {
    let syn::Item::Macro(item) = item else {
        return Ok(None);
    };
    if !item.mac.path.is_ident("include") {
        return Ok(None);
    }
    // Follow checked-in source includes, but leave generated includes to the
    // surface graph's named-provider enforcement. Consumer fixtures commonly
    // include their generated enforcement shim with `concat!(env!(...))` in a
    // hidden module; that file is not provider source and is unavailable until
    // the fixture build script runs.
    let Ok(relative) = syn::parse2::<syn::LitStr>(item.mac.tokens.clone()) else {
        return Ok(None);
    };
    let path = source_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(relative.value());
    let source = fs::read_to_string(&path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    Ok(Some((path, file)))
}

fn provider_item_enabled(
    item: &syn::Item,
    cfg: &CfgSet,
    source_file: &Path,
) -> Result<bool, MacroProviderError> {
    cfg_enabled(crate::item_attrs(item), cfg, source_file)
        .map_err(|error| MacroProviderError::Parse(error.to_string()))
}

pub(crate) fn provider_effective_attributes(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source_file: &Path,
) -> Result<Vec<syn::Attribute>, MacroProviderError> {
    effective_attribute_metas(attrs, cfg, source_file)
        .map_err(|error| MacroProviderError::Parse(error.to_string()))
        .map(|metas| {
            metas
                .into_iter()
                .map(|meta| syn::parse_quote!(#[#meta]))
                .collect()
        })
}

fn provider_has_attribute_named(
    attrs: &[syn::Attribute],
    name: &str,
    cfg: &CfgSet,
    source_file: &Path,
) -> Result<bool, MacroProviderError> {
    Ok(provider_effective_attributes(attrs, cfg, source_file)?
        .iter()
        .any(|attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == name)
        }))
}

fn provider_derives_named(
    attrs: &[syn::Attribute],
    name: &str,
    cfg: &CfgSet,
    source_file: &Path,
) -> Result<bool, MacroProviderError> {
    Ok(provider_effective_attributes(attrs, cfg, source_file)?
        .iter()
        .any(|attribute| derives_named(std::slice::from_ref(attribute), name)))
}

pub(crate) fn provider_derive_input(
    structure: &syn::ItemStruct,
    cfg: &CfgSet,
    source_file: &Path,
) -> Result<syn::DeriveInput, MacroProviderError> {
    let mut input =
        syn::parse2::<syn::DeriveInput>(structure.to_token_stream()).map_err(|error| {
            MacroProviderError::Parse(format!("{}: {error}", source_file.display()))
        })?;
    input.attrs = provider_effective_attributes(&input.attrs, cfg, source_file)?;
    if let syn::Data::Struct(data) = &mut input.data {
        let fields = match &mut data.fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return Ok(input),
        };
        let mut enabled = syn::punctuated::Punctuated::new();
        for mut field in std::mem::take(fields) {
            if cfg_enabled(&field.attrs, cfg, source_file)
                .map_err(|error| MacroProviderError::Parse(error.to_string()))?
            {
                field.attrs = provider_effective_attributes(&field.attrs, cfg, source_file)?;
                enabled.push(field);
            }
        }
        *fields = enabled;
    }
    Ok(input)
}

/// Prove that a local `macro_rules!` transcriber cannot add an API identity.
///
/// Every expansion arm is inspected. The deliberately conservative grammar
/// accepts private items and trait implementations, but rejects public items,
/// inherent implementations, item-position helper macros, and repetitions.
/// Those rejected forms can grow the reachable surface and therefore require
/// a real generated-API provider instead of an inert classification.
pub(crate) fn audit_inert_macro_transcriber(
    tokens: &TokenStream,
) -> Result<(), MacroProviderError> {
    let bodies = expansion_bodies(tokens.clone());
    if bodies.is_empty() {
        return Err(unsupported(
            "inert declarative macro has no auditable `=> <delimited tokens>` expansion arm",
        ));
    }
    for body in bodies {
        audit_inert_item_sequence(body, "transcriber top level")?;
    }
    Ok(())
}

/// Prove that an `inventory::collect!` invocation contains exactly one type.
///
/// The external macro is compiler/linker wiring, but its invocation still has
/// to be checked: classifying only its spelling would let arbitrary tokens
/// piggyback on the inert binding if the upstream macro ever accepted them.
pub(crate) fn audit_inventory_collection_invocation(
    tokens: &TokenStream,
) -> Result<(), MacroProviderError> {
    syn::parse2::<syn::Type>(tokens.clone())
        .map(|_| ())
        .map_err(|error| {
            unsupported(format!(
                "inventory::collect! must contain exactly one type: {error}"
            ))
        })
}

struct ThreadLocalDeclarations {
    visibilities: Vec<syn::Visibility>,
}

impl Parse for ThreadLocalDeclarations {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut visibilities = Vec::new();
        while !input.is_empty() {
            let _attrs = input.call(syn::Attribute::parse_outer)?;
            let visibility: syn::Visibility = input.parse()?;
            input.parse::<syn::Token![static]>()?;
            input.parse::<syn::Ident>()?;
            input.parse::<syn::Token![:]>()?;
            input.parse::<syn::Type>()?;
            input.parse::<syn::Token![=]>()?;
            input.parse::<syn::Expr>()?;
            input.parse::<syn::Token![;]>()?;
            visibilities.push(visibility);
        }
        if visibilities.is_empty() {
            return Err(input.error("thread_local! must declare at least one static"));
        }
        Ok(Self { visibilities })
    }
}

/// Prove that a `thread_local!` invocation consists solely of non-public
/// static declarations. Restricted visibility remains internal to the source
/// crate and therefore cannot create a supported facade identity.
pub(crate) fn audit_thread_local_invocation(
    tokens: &TokenStream,
) -> Result<(), MacroProviderError> {
    let declarations = syn::parse2::<ThreadLocalDeclarations>(tokens.clone()).map_err(|error| {
        unsupported(format!(
            "thread_local! payload is not structurally understood: {error}"
        ))
    })?;
    if declarations
        .visibilities
        .iter()
        .any(|visibility| matches!(visibility, syn::Visibility::Public(_)))
    {
        return Err(unsupported(
            "thread_local! inert wiring cannot declare a public static",
        ));
    }
    Ok(())
}

fn audit_inert_item_sequence(
    stream: TokenStream,
    position: &str,
) -> Result<(), MacroProviderError> {
    let tokens = stream.into_iter().collect::<Vec<_>>();
    reject_any_repetition(&tokens, position)?;
    reject_item_position_macro_invocations(&tokens, position)?;

    let mut index = 0;
    while index < tokens.len() {
        if is_ident(tokens.get(index), "pub") {
            return Err(unsupported(format!(
                "inert macro emits a public declaration at {position}"
            )));
        }
        if is_ident(tokens.get(index), "impl") {
            let body_index = tokens[index + 1..]
                .iter()
                .position(|token| {
                    matches!(token, TokenTree::Group(group) if group.delimiter() == proc_macro2::Delimiter::Brace)
                })
                .map(|offset| index + 1 + offset)
                .ok_or_else(|| unsupported("inert macro contains an impl without a body"))?;
            if !tokens[index + 1..body_index]
                .iter()
                .any(|token| is_ident(Some(token), "for"))
            {
                return Err(unsupported(
                    "inert macro emits an inherent impl; associated API requires a provider",
                ));
            }
            let TokenTree::Group(body) = &tokens[body_index] else {
                unreachable!()
            };
            audit_inert_trait_impl_body(body.stream())?;
            index = body_index;
        } else if is_ident(tokens.get(index), "mod")
            && let Some(TokenTree::Group(body)) = tokens[index + 1..].iter().find(|token| {
                matches!(token, TokenTree::Group(group) if group.delimiter() == proc_macro2::Delimiter::Brace)
            })
        {
            audit_inert_item_sequence(body.stream(), "nested module")?;
        }
        index += 1;
    }
    Ok(())
}

fn audit_inert_trait_impl_body(stream: TokenStream) -> Result<(), MacroProviderError> {
    let tokens = stream.into_iter().collect::<Vec<_>>();
    reject_any_repetition(&tokens, "trait impl")?;
    reject_item_position_macro_invocations(&tokens, "trait impl")?;
    if tokens.iter().any(|token| is_ident(Some(token), "pub")) {
        return Err(unsupported(
            "inert macro emits explicit visibility inside a trait impl",
        ));
    }
    Ok(())
}

fn reject_any_repetition(tokens: &[TokenTree], position: &str) -> Result<(), MacroProviderError> {
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token, TokenTree::Punct(punct) if punct.as_char() == '$')
            && matches!(tokens.get(index + 1), Some(TokenTree::Group(_)))
            && matches!(tokens.get(index + 2), Some(TokenTree::Punct(punct)) if matches!(punct.as_char(), '*' | '+' | '?'))
        {
            return Err(unsupported(format!(
                "inert macro contains an unaudited repetition at {position}"
            )));
        }
    }
    Ok(())
}

impl fmt::Display for MacroProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Parse(message) => formatter.write_str(message),
            Self::MissingGenerator => {
                formatter.write_str("declarative type-family generator is missing")
            }
            Self::MissingGeneratedTypes => {
                formatter.write_str("declarative type-family has no checked-in type declarations")
            }
            Self::MissingPublicMethods => {
                formatter.write_str("declarative type-family does not generate public methods")
            }
            Self::MissingNamedGenerator(name) => {
                write!(formatter, "{name}! generator is missing")
            }
            Self::MissingGeneratedVariants => {
                formatter.write_str("vanilla_registry_enum! has no generated variants")
            }
            Self::MissingGeneratedEvents => formatter.write_str(
                "gamemode_transition! and status_effect_marker! have no generated event types",
            ),
            Self::MissingConsumerInvocation(name) => {
                write!(formatter, "consumer fixture does not exercise #[{name}]")
            }
            Self::UnsupportedGeneratedShape(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for MacroProviderError {}

/// Test helper for auditing a small declarative type family without executing
/// its macro or duplicating its generated members.
///
/// Production families must use a declaration-specific provider such as
/// [`registry_id_provider`]. This helper is retained only for isolated parser
/// fixtures that exercise the fail-closed structural grammar.
#[doc(hidden)]
pub fn declarative_type_family_fixture_provider(
    path: &Path,
    macro_name: &str,
    identity_module: &str,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    declarative_type_family_provider(
        path,
        macro_name,
        identity_module,
        "fixture_declarative_type_family",
    )
}

/// Expand the typed registry-ID wrapper family from `registry_id!`.
pub fn registry_id_provider(path: &Path) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let bindings = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Use(item) => Some(item),
            _ => None,
        })
        .flat_map(registry_id_bindings)
        .collect::<Vec<_>>();
    let exact_imports = bindings
        .iter()
        .filter(|binding| binding.target == "sand_macros::registry_id")
        .collect::<Vec<_>>();
    if exact_imports.len() != 1 || !exact_imports[0].unconditional {
        return Err(MacroProviderError::MissingNamedGenerator(
            "unconditional sand_macros::registry_id".into(),
        ));
    }
    if bindings.len() != 1 {
        return Err(unsupported(
            "a competing import also binds `registry_id` in the macro namespace",
        ));
    }
    if file.items.iter().any(|item| {
        matches!(item, syn::Item::Macro(item) if item.ident.as_ref().is_some_and(|ident| ident == "registry_id"))
    }) {
        return Err(unsupported(
            "local registry_id! shadows the audited sand_macros generator",
        ));
    }
    let mut generated = Vec::new();
    for item in file.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        if !item.mac.path.is_ident("registry_id") {
            continue;
        }
        let expansion = sand_api_contract::syntax::registry_id::expand(item.mac.tokens)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let expanded = syn::parse2::<syn::File>(expansion.rust)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        generated.push(registry_api_from_expansion(&expanded)?);
    }
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
}

#[derive(Clone, Debug)]
struct RegistryImportBinding {
    target: String,
    unconditional: bool,
}

fn registry_id_bindings(item: &syn::ItemUse) -> Vec<RegistryImportBinding> {
    fn visit(
        tree: &syn::UseTree,
        prefix: &mut Vec<String>,
        unconditional: bool,
        bindings: &mut Vec<RegistryImportBinding>,
    ) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                visit(&path.tree, prefix, unconditional, bindings);
                prefix.pop();
            }
            syn::UseTree::Name(name) if name.ident == "registry_id" => {
                let mut target = prefix.clone();
                target.push(name.ident.to_string());
                bindings.push(RegistryImportBinding {
                    target: target.join("::"),
                    unconditional,
                });
            }
            syn::UseTree::Rename(rename) if rename.rename == "registry_id" => {
                let mut target = prefix.clone();
                target.push(rename.ident.to_string());
                bindings.push(RegistryImportBinding {
                    target: target.join("::"),
                    unconditional,
                });
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    visit(tree, prefix, unconditional, bindings);
                }
            }
            syn::UseTree::Glob(_) => bindings.push(RegistryImportBinding {
                target: format!("{}::*", prefix.join("::")),
                unconditional,
            }),
            _ => {}
        }
    }

    let mut bindings = Vec::new();
    visit(
        &item.tree,
        &mut Vec::new(),
        item.attrs.is_empty(),
        &mut bindings,
    );
    bindings
}

fn registry_api_from_expansion(file: &syn::File) -> Result<GeneratedApi, MacroProviderError> {
    let mut root = None::<(String, ReachableKind)>;
    for item in &file.items {
        let candidate = match item {
            syn::Item::Struct(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                let public_fields = item
                    .fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| matches!(field.vis, syn::Visibility::Public(_)))
                    .map(|(index, field)| {
                        field
                            .ident
                            .as_ref()
                            .map_or_else(|| index.to_string(), ToString::to_string)
                    })
                    .collect::<Vec<_>>();
                if !public_fields.is_empty() {
                    return Err(unsupported(format!(
                        "registry_id! expansion emits unsupported public fields: {}",
                        public_fields.join(", ")
                    )));
                }
                Some((item.ident.to_string(), ReachableKind::Struct))
            }
            syn::Item::Enum(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                return Err(unsupported(format!(
                    "registry_id! expansion changed public root `{}` to an enum with variants: {}",
                    item.ident,
                    item.variants
                        .iter()
                        .map(|variant| variant.ident.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
            syn::Item::Union(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
                return Err(unsupported(format!(
                    "registry_id! expansion changed public root `{}` to a union",
                    item.ident
                )));
            }
            item if public_top_level_identity(item).is_some() => {
                return Err(unsupported(format!(
                    "registry_id! expansion emits unsupported extra public identity `{}`",
                    public_top_level_identity(item).unwrap()
                )));
            }
            syn::Item::Macro(item)
                if item
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("macro_export")) =>
            {
                return Err(unsupported(
                    "registry_id! expansion emits an unsupported exported macro",
                ));
            }
            _ => None,
        };
        if let Some(candidate) = candidate
            && root.replace(candidate).is_some()
        {
            return Err(unsupported(
                "registry_id! expansion emits more than one public root type",
            ));
        }
    }
    let Some((name, kind)) = root else {
        return Err(unsupported(
            "registry_id! expansion does not emit one supported public root type",
        ));
    };
    let mut members = BTreeSet::new();
    for item in &file.items {
        let syn::Item::Impl(block) = item else {
            continue;
        };
        let owns_generated_type = matches!(
            block.self_ty.as_ref(),
            syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident(&name)
        );
        if block.trait_.is_some() || !owns_generated_type {
            continue;
        }
        for member in &block.items {
            match member {
                syn::ImplItem::Fn(method) if matches!(method.vis, syn::Visibility::Public(_)) => {
                    members.insert((method.sig.ident.to_string(), ReachableKind::Method));
                }
                syn::ImplItem::Const(constant)
                    if matches!(constant.vis, syn::Visibility::Public(_)) =>
                {
                    members.insert((constant.ident.to_string(), ReachableKind::AssociatedConst));
                }
                syn::ImplItem::Type(ty) if matches!(ty.vis, syn::Visibility::Public(_)) => {
                    members.insert((ty.ident.to_string(), ReachableKind::AssociatedType));
                }
                syn::ImplItem::Macro(_) => {
                    return Err(unsupported(
                        "registry_id! inherent impl contains an unauditable macro invocation",
                    ));
                }
                _ => {}
            }
        }
    }
    if members.is_empty() {
        return Err(MacroProviderError::MissingPublicMethods);
    }
    Ok(GeneratedApi {
        identity: format!("sand_components::registry::{name}"),
        provider: "generated_registry_ids".into(),
        producer: None,
        kind,
        members: members.into_iter().collect(),
        excluded: false,
    })
}

fn public_top_level_identity(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Const(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::Fn(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.sig.ident.to_string())
        }
        syn::Item::ExternCrate(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(format!("extern crate {}", item.ident))
        }
        syn::Item::ForeignMod(item)
            if item.items.iter().any(|foreign| match foreign {
                syn::ForeignItem::Fn(item) => matches!(item.vis, syn::Visibility::Public(_)),
                syn::ForeignItem::Static(item) => matches!(item.vis, syn::Visibility::Public(_)),
                syn::ForeignItem::Type(item) => matches!(item.vis, syn::Visibility::Public(_)),
                _ => false,
            }) =>
        {
            Some("public foreign item".into())
        }
        syn::Item::Mod(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::Static(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::Trait(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::TraitAlias(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::Type(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.ident.to_string())
        }
        syn::Item::Use(item) if matches!(item.vis, syn::Visibility::Public(_)) => {
            Some(item.tree.to_token_stream().to_string())
        }
        item if item.to_token_stream().to_string().starts_with("pub ") => {
            Some("unclassified public item".into())
        }
        _ => None,
    }
}

#[cfg(test)]
mod registry_expansion_tests {
    use super::*;

    fn error(source: TokenStream) -> String {
        let file = syn::parse2::<syn::File>(source).unwrap();
        registry_api_from_expansion(&file).unwrap_err().to_string()
    }

    #[test]
    fn public_struct_field_growth_fails_closed() {
        let error = error(quote::quote! {
            pub struct ExampleId(pub ResourceLocation);
            impl ExampleId { pub fn minecraft() {} }
        });
        assert!(error.contains("unsupported public fields: 0"));
    }

    #[test]
    fn enum_variant_growth_fails_closed() {
        let error = error(quote::quote! {
            pub enum ExampleId { Vanilla, Custom(ResourceLocation) }
            impl ExampleId { pub fn minecraft() {} }
        });
        assert!(error.contains("enum with variants: Vanilla, Custom"));
    }

    #[test]
    fn extra_public_top_level_identity_fails_closed() {
        let error = error(quote::quote! {
            pub struct ExampleId(ResourceLocation);
            impl ExampleId { pub fn minecraft() {} }
            pub fn leaked_helper() {}
        });
        assert!(error.contains("extra public identity `leaked_helper`"));
    }

    #[test]
    fn public_extern_crate_fails_closed() {
        let error = error(quote::quote! {
            pub struct ExampleId(ResourceLocation);
            impl ExampleId { pub fn minecraft() {} }
            pub extern crate serde as leaked_serde;
        });
        assert!(error.contains("extra public identity `extern crate serde`"));
    }
}

/// Expand the checked-in vanilla registry enums in `effect.rs`.
///
/// Enum names and vanilla variants come from each invocation. Methods and
/// macro-owned variants (currently `Custom`) come from the generator body, so
/// either side of the declaration changes the provider without a parallel
/// hand-maintained identity list.
pub fn vanilla_registry_enum_provider(
    path: &Path,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let generator = named_generator(&file, "vanilla_registry_enum")?;
    let shape = generated_type_shape(&generator, "name", &["name"], GeneratedTypeKind::Enum)?;
    if shape.members.is_empty() {
        return Err(MacroProviderError::MissingPublicMethods);
    }
    let macro_members = generated_enum_members(generator);
    if !macro_members
        .iter()
        .any(|(_, kind)| *kind == ReachableKind::Variant)
    {
        return Err(MacroProviderError::MissingGeneratedVariants);
    }

    let mut generated = Vec::new();
    for item in &file.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        if !item.mac.path.is_ident("vanilla_registry_enum") {
            continue;
        }
        let (name, body) = invocation_name_and_body(item.mac.tokens.clone(), path)?;
        let mut members = invocation_variants(body)
            .into_iter()
            .map(|variant| (variant, ReachableKind::Variant))
            .chain(macro_members.iter().cloned())
            .chain(shape.members.iter().cloned())
            .collect::<Vec<_>>();
        members.sort();
        members.dedup();
        generated.push(GeneratedApi {
            identity: format!("sand_components::effect::{name}"),
            provider: "generated_effect_registry_enums".into(),
            producer: None,
            kind: ReachableKind::Enum,
            members,
            excluded: false,
        });
    }
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
}

/// Expand public event marker types emitted by the two checked-in event
/// declaration families. Trait implementations emitted by adjacent macros do
/// not add separately named facade APIs and therefore need no provider entry.
pub fn event_generated_type_provider(path: &Path) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let gamemode_generator = named_generator(&file, "gamemode_transition")?;
    let gamemode_types = public_struct_metavariables(gamemode_generator.clone());
    if !["enter", "exit"]
        .into_iter()
        .all(|name| gamemode_types.contains(name))
    {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }
    let enter_shape = generated_type_shape(
        &gamemode_generator,
        "enter",
        &["enter", "exit"],
        GeneratedTypeKind::Struct,
    )?;
    let exit_shape = generated_type_shape(
        &gamemode_generator,
        "exit",
        &["enter", "exit"],
        GeneratedTypeKind::Struct,
    )?;
    let status_generator = named_generator(&file, "status_effect_marker")?;
    if !public_struct_metavariables(status_generator.clone()).contains("ty") {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }
    let status_shape =
        generated_type_shape(&status_generator, "ty", &["ty"], GeneratedTypeKind::Struct)?;

    let mut generated = Vec::new();
    for item in &file.items {
        let syn::Item::Macro(item) = item else {
            continue;
        };
        let identifiers = item
            .mac
            .tokens
            .clone()
            .into_iter()
            .filter_map(|token| match token {
                TokenTree::Ident(ident) => Some(ident.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if item.mac.path.is_ident("gamemode_transition") {
            if identifiers.len() < 2 {
                return Err(MacroProviderError::Parse(format!(
                    "{}: gamemode_transition! must declare enter and exit types",
                    path.display()
                )));
            }
            for (name, shape) in identifiers[identifiers.len() - 2..]
                .iter()
                .zip([&enter_shape, &exit_shape])
            {
                generated.push(generated_event(name, shape));
            }
        } else if item.mac.path.is_ident("status_effect_marker") {
            let name = identifiers.first().ok_or_else(|| {
                MacroProviderError::Parse(format!(
                    "{}: status_effect_marker! must declare a marker type",
                    path.display()
                ))
            })?;
            generated.push(generated_event(name, &status_shape));
        }
    }
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    generated.dedup_by(|left, right| left.identity == right.identity);
    Ok(generated)
}

fn generated_event(name: &str, shape: &GeneratedTypeShape) -> GeneratedApi {
    GeneratedApi {
        identity: format!("sand_core::events::{name}"),
        provider: "generated_event_markers".into(),
        producer: None,
        kind: ReachableKind::Struct,
        members: shape.members.clone(),
        excluded: false,
    }
}

/// Describe public associated items emitted by the real `SandStorage` derive
/// from the same annotated struct declaration consumed by macro expansion.
/// Field accessor names come directly from named fields; no parallel member
/// manifest is maintained by the fixture or consuming build script.
pub fn sand_storage_derive_provider(
    path: &Path,
    identity_module: &str,
    cfg: &CfgSet,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let mut generated = Vec::new();
    collect_sand_storage_derives(
        &file.items,
        identity_module,
        path,
        &module_search_directory(path),
        path.parent().unwrap_or_else(|| Path::new(".")),
        cfg,
        &mut generated,
    )?;
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
}

fn collect_sand_storage_derives(
    items: &[syn::Item],
    identity_module: &str,
    path: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
    generated: &mut Vec<GeneratedApi>,
) -> Result<(), MacroProviderError> {
    for item in items {
        if !provider_item_enabled(item, cfg, path)? {
            continue;
        }
        if let Some((included_path, file)) = parse_provider_include(item, path)? {
            collect_sand_storage_derives(
                &file.items,
                identity_module,
                &included_path,
                &module_search_directory(&included_path),
                included_path.parent().unwrap_or_else(|| Path::new(".")),
                cfg,
                generated,
            )?;
            continue;
        }
        if let syn::Item::Mod(module) = item {
            let nested_module = format!("{identity_module}::{}", module.ident.unraw());
            if let Some((_, items)) = &module.content {
                let child_directory = default_directory.join(module.ident.unraw().to_string());
                collect_sand_storage_derives(
                    items,
                    &nested_module,
                    path,
                    &child_directory,
                    &child_directory,
                    cfg,
                    generated,
                )?;
            } else {
                let (nested_path, file) =
                    parse_provider_module(module, path, default_directory, path_directory, cfg)?;
                collect_sand_storage_derives(
                    &file.items,
                    &nested_module,
                    &nested_path,
                    &module_search_directory(&nested_path),
                    nested_path.parent().unwrap_or_else(|| Path::new(".")),
                    cfg,
                    generated,
                )?;
            }
            continue;
        }
        let syn::Item::Struct(structure) = item else {
            continue;
        };
        if !provider_derives_named(&structure.attrs, "SandStorage", cfg, path)? {
            continue;
        }
        let derive_input = provider_derive_input(structure, cfg, path)?;
        let members = sand_api_contract::syntax::sand_storage_generated_member_names(&derive_input)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let owner = format!("{identity_module}::{}", structure.ident.unraw());
        for (index, name) in members.into_iter().enumerate() {
            generated.push(GeneratedApi {
                identity: format!("{owner}::{name}"),
                provider: "storage_derive".into(),
                producer: Some(GeneratedProducer {
                    owner: owner.clone(),
                    name: "SandStorage".into(),
                }),
                kind: if index == 0 {
                    ReachableKind::AssociatedConst
                } else {
                    ReachableKind::Method
                },
                members: Vec::new(),
                excluded: false,
            });
        }
    }
    Ok(())
}

/// Describe the public bound-view type and inherent APIs emitted by a real
/// `State` derive. The surface model is shared with reachability so a new
/// State field or a scope change cannot be accepted by a consumer build until
/// its exact generated declarations are contracted.
pub fn state_derive_provider(
    path: &Path,
    identity_module: &str,
    cfg: &CfgSet,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let mut generated = Vec::new();
    collect_state_derives(
        &file.items,
        identity_module,
        path,
        &module_search_directory(path),
        path.parent().unwrap_or_else(|| Path::new(".")),
        cfg,
        &mut generated,
    )?;
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
}

fn collect_state_derives(
    items: &[syn::Item],
    identity_module: &str,
    path: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
    generated: &mut Vec<GeneratedApi>,
) -> Result<(), MacroProviderError> {
    for item in items {
        if !provider_item_enabled(item, cfg, path)? {
            continue;
        }
        if let Some((included_path, file)) = parse_provider_include(item, path)? {
            collect_state_derives(
                &file.items,
                identity_module,
                &included_path,
                &module_search_directory(&included_path),
                included_path.parent().unwrap_or_else(|| Path::new(".")),
                cfg,
                generated,
            )?;
            continue;
        }
        if let syn::Item::Mod(module) = item {
            let nested_module = format!("{identity_module}::{}", module.ident.unraw());
            if let Some((_, items)) = &module.content {
                let child_directory = default_directory.join(module.ident.unraw().to_string());
                collect_state_derives(
                    items,
                    &nested_module,
                    path,
                    &child_directory,
                    &child_directory,
                    cfg,
                    generated,
                )?;
            } else {
                let (nested_path, file) =
                    parse_provider_module(module, path, default_directory, path_directory, cfg)?;
                collect_state_derives(
                    &file.items,
                    &nested_module,
                    &nested_path,
                    &module_search_directory(&nested_path),
                    nested_path.parent().unwrap_or_else(|| Path::new(".")),
                    cfg,
                    generated,
                )?;
            }
            continue;
        }
        let syn::Item::Struct(structure) = item else {
            continue;
        };
        if !provider_derives_named(&structure.attrs, "State", cfg, path)?
            || !is_public(&structure.vis)
        {
            continue;
        }
        let derive_input = provider_derive_input(structure, cfg, path)?;
        let surface = sand_api_contract::syntax::state_generated_surface(&derive_input)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let owner = format!("{identity_module}::{}", structure.ident.unraw());
        generated.push(GeneratedApi {
            identity: format!("{identity_module}::{}", surface.bound_type),
            provider: "state_derive".into(),
            producer: Some(GeneratedProducer {
                owner: owner.clone(),
                name: "State".into(),
            }),
            kind: ReachableKind::Struct,
            members: surface
                .bound_fields
                .into_iter()
                .map(|name| (name, ReachableKind::Field))
                .collect(),
            excluded: false,
        });
        generated.extend(surface.associated.into_iter().map(|member| GeneratedApi {
            identity: format!("{owner}::{}", member.name),
            provider: "state_derive".into(),
            producer: Some(GeneratedProducer {
                owner: owner.clone(),
                name: "State".into(),
            }),
            kind: match member.kind {
                sand_api_contract::syntax::StateGeneratedAssociatedKind::Const => {
                    ReachableKind::AssociatedConst
                }
                sand_api_contract::syntax::StateGeneratedAssociatedKind::Method => {
                    ReachableKind::Method
                }
            },
            members: Vec::new(),
            excluded: false,
        }));
    }
    Ok(())
}

/// Describe the sibling typed item reference and helpers emitted by one real
/// `#[custom_item]` invocation. Literal name extraction lives in the shared
/// syntax model, so consumer enforcement fails closed if the macro gains a
/// new naming form that has not received a provider implementation.
pub fn custom_item_provider(
    path: &Path,
    identity_module: &str,
    cfg: &CfgSet,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let mut generated = Vec::new();
    collect_custom_items(
        &file.items,
        identity_module,
        path,
        &module_search_directory(path),
        path.parent().unwrap_or_else(|| Path::new(".")),
        cfg,
        &mut generated,
    )?;
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
}

fn collect_custom_items(
    items: &[syn::Item],
    identity_module: &str,
    path: &Path,
    default_directory: &Path,
    path_directory: &Path,
    cfg: &CfgSet,
    generated: &mut Vec<GeneratedApi>,
) -> Result<(), MacroProviderError> {
    for item in items {
        if !provider_item_enabled(item, cfg, path)? {
            continue;
        }
        if let Some((included_path, file)) = parse_provider_include(item, path)? {
            collect_custom_items(
                &file.items,
                identity_module,
                &included_path,
                &module_search_directory(&included_path),
                included_path.parent().unwrap_or_else(|| Path::new(".")),
                cfg,
                generated,
            )?;
            continue;
        }
        if let syn::Item::Mod(module) = item {
            let nested_module = format!("{identity_module}::{}", module.ident.unraw());
            if let Some((_, items)) = &module.content {
                let child_directory = default_directory.join(module.ident.unraw().to_string());
                collect_custom_items(
                    items,
                    &nested_module,
                    path,
                    &child_directory,
                    &child_directory,
                    cfg,
                    generated,
                )?;
            } else {
                let (nested_path, file) =
                    parse_provider_module(module, path, default_directory, path_directory, cfg)?;
                collect_custom_items(
                    &file.items,
                    &nested_module,
                    &nested_path,
                    &module_search_directory(&nested_path),
                    nested_path.parent().unwrap_or_else(|| Path::new(".")),
                    cfg,
                    generated,
                )?;
            }
            continue;
        }
        let syn::Item::Fn(function) = item else {
            continue;
        };
        if !provider_has_attribute_named(&function.attrs, "custom_item", cfg, path)?
            || !is_public(&function.vis)
        {
            continue;
        }
        let mut function = function.clone();
        function.attrs = provider_effective_attributes(&function.attrs, cfg, path)?;
        let surface = sand_api_contract::syntax::custom_item_generated_surface(&function)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let owner = format!("{identity_module}::{}", function.sig.ident.unraw());
        let type_identity = format!("{identity_module}::{}", surface.type_name);
        generated.push(GeneratedApi {
            identity: type_identity.clone(),
            provider: "item_macro".into(),
            producer: Some(GeneratedProducer {
                owner: owner.clone(),
                name: "custom_item".into(),
            }),
            kind: ReachableKind::Struct,
            members: surface
                .constants
                .into_iter()
                .map(|name| (name, ReachableKind::AssociatedConst))
                .chain(
                    surface
                        .methods
                        .into_iter()
                        .map(|name| (name, ReachableKind::Method)),
                )
                .collect(),
            excluded: false,
        });
    }
    Ok(())
}

fn derives_named(attributes: &[syn::Attribute], name: &str) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("derive")
            && attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|derives| {
                    derives.iter().any(|derive| {
                        derive
                            .segments
                            .last()
                            .is_some_and(|segment| segment.ident == name)
                    })
                })
    })
}

fn is_public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn named_generator(file: &syn::File, macro_name: &str) -> Result<TokenStream, MacroProviderError> {
    file.items
        .iter()
        .find_map(|item| match item {
            syn::Item::Macro(item)
                if item.ident.as_ref().is_some_and(|ident| ident == macro_name) =>
            {
                Some(item.mac.tokens.clone())
            }
            _ => None,
        })
        .ok_or_else(|| MacroProviderError::MissingNamedGenerator(macro_name.into()))
}

fn invocation_name_and_body(
    tokens: TokenStream,
    path: &Path,
) -> Result<(String, TokenStream), MacroProviderError> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        let TokenTree::Group(group) = token else {
            continue;
        };
        if group.delimiter() != proc_macro2::Delimiter::Brace {
            continue;
        }
        let name = tokens[..index]
            .iter()
            .rev()
            .find_map(|token| match token {
                TokenTree::Ident(ident) => Some(ident.to_string()),
                _ => None,
            })
            .ok_or_else(|| {
                MacroProviderError::Parse(format!(
                    "{}: enum generator invocation has no type name",
                    path.display()
                ))
            })?;
        return Ok((name, group.stream()));
    }
    Err(MacroProviderError::Parse(format!(
        "{}: enum generator invocation has no variant body",
        path.display()
    )))
}

fn invocation_variants(tokens: TokenStream) -> BTreeSet<String> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    tokens
        .windows(2)
        .filter_map(|tokens| match tokens {
            [TokenTree::Ident(ident), TokenTree::Punct(punct)] if punct.as_char() == '=' => {
                Some(ident.to_string())
            }
            _ => None,
        })
        .collect()
}

fn generated_enum_members(tokens: TokenStream) -> BTreeSet<(String, ReachableKind)> {
    let Some(body) = find_enum_body(tokens) else {
        return BTreeSet::new();
    };
    let tokens = body.into_iter().collect::<Vec<_>>();
    let mut members = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        let TokenTree::Ident(ident) = token else {
            continue;
        };
        let preceded_by_dollar = index
            .checked_sub(1)
            .and_then(|previous| tokens.get(previous))
            .is_some_and(
                |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '$'),
            );
        if preceded_by_dollar {
            continue;
        }
        let starts_variant = index == 0
            || matches!(tokens.get(index - 1), Some(TokenTree::Punct(punct)) if matches!(punct.as_char(), ',' | '+' | '*' | '?'));
        if !starts_variant {
            continue;
        }
        let variant = ident.to_string();
        members.insert((variant.clone(), ReachableKind::Variant));
        let Some(TokenTree::Group(fields)) = tokens.get(index + 1) else {
            continue;
        };
        match fields.delimiter() {
            proc_macro2::Delimiter::Parenthesis => {
                let field_count = fields
                    .stream()
                    .into_iter()
                    .filter(
                        |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','),
                    )
                    .count()
                    + 1;
                for field in 0..field_count {
                    members.insert((format!("{variant}::{field}"), ReachableKind::Field));
                }
            }
            proc_macro2::Delimiter::Brace => {
                let field_tokens = fields.stream().into_iter().collect::<Vec<_>>();
                for segment in field_tokens.split(
                    |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','),
                ) {
                    if let Some(name) = segment.iter().find_map(token_ident) {
                        members.insert((format!("{variant}::{name}"), ReachableKind::Field));
                    }
                }
            }
            _ => {}
        }
    }
    members
}

fn find_enum_body(tokens: TokenStream) -> Option<TokenStream> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    for (index, window) in tokens.windows(2).enumerate() {
        if matches!(&window[0], TokenTree::Ident(ident) if ident == "pub")
            && matches!(&window[1], TokenTree::Ident(ident) if ident == "enum")
            && let Some(TokenTree::Group(group)) = tokens[index + 2..]
                .iter()
                .find(|token| matches!(token, TokenTree::Group(group) if group.delimiter() == proc_macro2::Delimiter::Brace))
        {
            return Some(group.stream());
        }
    }
    tokens.into_iter().find_map(|token| match token {
        TokenTree::Group(group) => find_enum_body(group.stream()),
        _ => None,
    })
}

fn declarative_type_family_provider(
    path: &Path,
    macro_name: &str,
    identity_module: &str,
    provider: &str,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let generator = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Macro(item)
                if item.ident.as_ref().is_some_and(|ident| ident == macro_name) =>
            {
                Some(item.mac.tokens.clone())
            }
            _ => None,
        })
        .ok_or(MacroProviderError::MissingGenerator)?;
    let shape = generated_type_shape(&generator, "name", &["name"], GeneratedTypeKind::Struct)?;
    if shape.members.is_empty() {
        return Err(MacroProviderError::MissingPublicMethods);
    }
    let mut names = BTreeSet::new();
    for item in &file.items {
        if let syn::Item::Macro(item) = item
            && item.mac.path.is_ident(macro_name)
        {
            let name = item
                .mac
                .tokens
                .clone()
                .into_iter()
                .filter_map(|token| match token {
                    TokenTree::Ident(ident) => Some(ident.to_string()),
                    _ => None,
                })
                .last()
                .ok_or_else(|| {
                    MacroProviderError::Parse(format!(
                        "{}: {macro_name}! invocation has no type name",
                        path.display()
                    ))
                })?;
            names.insert(name);
        }
    }
    if names.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    Ok(names
        .into_iter()
        .map(|name| GeneratedApi {
            identity: format!("{identity_module}::{name}"),
            provider: provider.into(),
            producer: None,
            kind: ReachableKind::Struct,
            members: shape.members.clone(),
            excluded: false,
        })
        .collect())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GeneratedTypeKind {
    Struct,
    Enum,
}

#[derive(Clone, Debug)]
struct GeneratedTypeShape {
    members: Vec<(String, ReachableKind)>,
}

/// Inspect the expansion arm itself. This deliberately fails closed when a
/// generator starts emitting another public top-level item: provider support
/// must land in the same change as the new Rust-visible shape.
fn generated_type_shape(
    generator: &TokenStream,
    metavariable: &str,
    allowed_metavariables: &[&str],
    expected: GeneratedTypeKind,
) -> Result<GeneratedTypeShape, MacroProviderError> {
    let body = expansion_body(generator.clone())?;
    let tokens = body.into_iter().collect::<Vec<_>>();
    reject_item_position_macro_invocations(&tokens, "top level")?;
    let mut saw_type = false;
    let mut members = BTreeSet::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == '$')
            && let Some(TokenTree::Group(group)) = tokens.get(index + 1)
            && repetition_emits_api_item(group.stream())
        {
            return Err(unsupported(
                "generator emits API items inside a repetition that the provider cannot identify",
            ));
        }
        if is_ident(tokens.get(index), "pub") {
            if matches!(tokens.get(index + 1), Some(TokenTree::Group(_))) {
                index += 1;
                continue;
            }
            let kind = tokens
                .get(index + 1)
                .and_then(token_ident)
                .unwrap_or_default();
            match kind.as_str() {
                "struct" | "enum" => {
                    let actual = if kind == "struct" {
                        GeneratedTypeKind::Struct
                    } else {
                        GeneratedTypeKind::Enum
                    };
                    let (name, after_name) = generated_name(&tokens, index + 2);
                    let is_allowed = name
                        .as_deref()
                        .is_some_and(|name| allowed_metavariables.contains(&name));
                    if !is_allowed || actual != expected {
                        return Err(unsupported(format!(
                            "generator emits unsupported public {kind} `{}`; expected only `${metavariable}`",
                            name.unwrap_or_else(|| "<unknown>".into())
                        )));
                    }
                    let is_target = name.as_deref() == Some(metavariable);
                    saw_type |= is_target;
                    if is_target && expected == GeneratedTypeKind::Struct {
                        collect_public_fields(&tokens, after_name, &mut members)?;
                    }
                }
                _ => {
                    return Err(unsupported(format!(
                        "generator emits unsupported public top-level `{kind}` item"
                    )));
                }
            }
        } else if is_ident(tokens.get(index), "impl") {
            let Some(body_index) = tokens[index + 1..]
                .iter()
                .position(|token| matches!(token, TokenTree::Group(group) if group.delimiter() == proc_macro2::Delimiter::Brace))
                .map(|offset| index + 1 + offset)
            else {
                return Err(unsupported("generator contains an impl without a body"));
            };
            let header = &tokens[index + 1..body_index];
            let TokenTree::Group(group) = &tokens[body_index] else {
                unreachable!()
            };
            // A trait implementation makes the trait's already-existing items
            // callable for this type, but it does not declare new public API
            // identities on the concrete type. The source graph owns the
            // trait and its associated items once; providers only add members
            // declared by an inherent implementation.
            let is_trait_impl = header.iter().any(|token| is_ident(Some(token), "for"));
            if !is_trait_impl && header_mentions_metavariable(header, metavariable) {
                collect_public_associated_items(group.stream(), &mut members)?;
            }
            index = body_index;
        }
        index += 1;
    }
    if !saw_type {
        return Err(unsupported(format!(
            "generator does not emit public `${metavariable}`"
        )));
    }
    Ok(GeneratedTypeShape {
        members: members.into_iter().collect(),
    })
}

fn unsupported(message: impl Into<String>) -> MacroProviderError {
    MacroProviderError::UnsupportedGeneratedShape(message.into())
}

fn expansion_body(tokens: TokenStream) -> Result<TokenStream, MacroProviderError> {
    let bodies = expansion_bodies(tokens);
    match bodies.len() {
        0 => Err(unsupported(
            "declarative generator has no auditable `=> <delimited tokens>` expansion arm",
        )),
        1 => Ok(bodies.into_iter().next().expect("one expansion body")),
        count => Err(unsupported(format!(
            "declarative generator has {count} expansion arms; multi-arm generators are not auditable by this provider"
        ))),
    }
}

fn expansion_bodies(tokens: TokenStream) -> Vec<TokenStream> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    let bodies = tokens
        .windows(3)
        .filter_map(|window| {
            if matches!(&window[0], TokenTree::Punct(punct) if punct.as_char() == '=')
                && matches!(&window[1], TokenTree::Punct(punct) if punct.as_char() == '>')
                && let TokenTree::Group(group) = &window[2]
            {
                Some(group.stream())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if !bodies.is_empty() {
        return bodies;
    }
    tokens
        .into_iter()
        .filter_map(|token| match token {
            TokenTree::Group(group) => Some(expansion_bodies(group.stream())),
            _ => None,
        })
        .flatten()
        .collect()
}

fn generated_name(tokens: &[TokenTree], index: usize) -> (Option<String>, usize) {
    match (tokens.get(index), tokens.get(index + 1)) {
        (Some(TokenTree::Punct(dollar)), Some(TokenTree::Ident(name)))
            if dollar.as_char() == '$' =>
        {
            (Some(name.to_string()), index + 2)
        }
        (Some(TokenTree::Ident(name)), _) => (Some(name.to_string()), index + 1),
        _ => (None, index),
    }
}

fn collect_public_fields(
    tokens: &[TokenTree],
    after_name: usize,
    members: &mut BTreeSet<(String, ReachableKind)>,
) -> Result<(), MacroProviderError> {
    let fields = tokens[after_name..]
        .iter()
        .take_while(|token| !matches!(token, TokenTree::Punct(punct) if punct.as_char() == ';'))
        .find_map(|token| match token {
            TokenTree::Group(group)
                if matches!(
                    group.delimiter(),
                    proc_macro2::Delimiter::Brace | proc_macro2::Delimiter::Parenthesis
                ) =>
            {
                Some(group)
            }
            _ => None,
        });
    let Some(fields) = fields else {
        return Ok(());
    };
    let delimiter = fields.delimiter();
    let field_tokens = fields.stream().into_iter().collect::<Vec<_>>();
    let field_segments = field_tokens
        .split(|token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ','));
    for (field_index, field) in field_segments.enumerate() {
        let Some(public_index) = field.iter().position(|token| is_ident(Some(token), "pub")) else {
            continue;
        };
        if matches!(field.get(public_index + 1), Some(TokenTree::Group(_))) {
            continue;
        }
        let name = if delimiter == proc_macro2::Delimiter::Parenthesis {
            field_index.to_string()
        } else {
            field
                .get(public_index + 1)
                .and_then(token_ident)
                .ok_or_else(|| unsupported("generated public named field has no stable name"))?
        };
        members.insert((name, ReachableKind::Field));
    }
    Ok(())
}

fn collect_public_associated_items(
    tokens: TokenStream,
    members: &mut BTreeSet<(String, ReachableKind)>,
) -> Result<(), MacroProviderError> {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    reject_item_position_macro_invocations(&tokens, "inherent impl")?;
    for index in 0..tokens.len() {
        if matches!(tokens.get(index), Some(TokenTree::Punct(punct)) if punct.as_char() == '$')
            && let Some(TokenTree::Group(group)) = tokens.get(index + 1)
            && repetition_emits_api_item(group.stream())
        {
            return Err(unsupported(
                "generator emits associated items inside an unmodeled repetition",
            ));
        }
        if !is_ident(tokens.get(index), "pub") {
            continue;
        }
        if matches!(tokens.get(index + 1), Some(TokenTree::Group(_))) {
            continue;
        }
        let item = tokens
            .get(index + 1)
            .and_then(token_ident)
            .unwrap_or_default();
        let kind = match item.as_str() {
            "fn" => ReachableKind::Method,
            "const" => ReachableKind::AssociatedConst,
            "type" => ReachableKind::AssociatedType,
            _ => {
                return Err(unsupported(format!(
                    "generator emits unsupported public associated `{item}` item"
                )));
            }
        };
        let name = tokens
            .get(index + 2)
            .and_then(token_ident)
            .ok_or_else(|| unsupported(format!("generated public `{item}` has no stable name")))?;
        members.insert((name, kind));
    }
    Ok(())
}

fn header_mentions_metavariable(tokens: &[TokenTree], name: &str) -> bool {
    tokens.windows(2).any(|window| {
        matches!(&window[0], TokenTree::Punct(punct) if punct.as_char() == '$')
            && matches!(&window[1], TokenTree::Ident(ident) if ident == name)
    })
}

fn repetition_emits_api_item(tokens: TokenStream) -> bool {
    let flattened = flatten(tokens);
    flattened.iter().any(|token| {
        matches!(token, TokenTree::Ident(ident) if matches!(ident.to_string().as_str(), "pub" | "fn" | "const" | "type" | "struct" | "enum" | "union" | "trait" | "static" | "mod" | "use"))
    }) || flattened.windows(2).enumerate().any(|(index, window)| {
        matches!(&window[0], TokenTree::Ident(_))
            && matches!(&window[1], TokenTree::Punct(punct) if punct.as_char() == '!')
            && !is_api_contract_inventory_submit(&flattened, index)
            && !inert_expression_macro(&flattened, index)
    })
}

/// Reject macros in positions where their expansion can add facade API.
///
/// Groups are intentionally opaque here. A macro nested in an attribute,
/// function body, field type, or expression cannot add an item alongside the
/// generated type. Direct calls at the transcriber top level can emit types,
/// functions, modules, or re-exports, while direct calls in an inherent impl
/// can emit associated items. Those shapes require a dedicated provider rather
/// than trusting an arbitrary helper expansion.
fn reject_item_position_macro_invocations(
    tokens: &[TokenTree],
    position: &str,
) -> Result<(), MacroProviderError> {
    for (index, window) in tokens.windows(3).enumerate() {
        let [
            TokenTree::Ident(name),
            TokenTree::Punct(bang),
            TokenTree::Group(_),
        ] = window
        else {
            continue;
        };
        if bang.as_char() == '!'
            && macro_path_starts_at_item_boundary(tokens, index)
            && !is_api_contract_inventory_submit(tokens, index)
        {
            return Err(unsupported(format!(
                "generator invokes unmodeled `{name}!` macro at {position}; helper macros in item-producing positions are not auditable"
            )));
        }
    }
    Ok(())
}

/// `inventory::submit!` is the inert link-time registration mechanism used by
/// generated API contracts. It cannot introduce a Rust-visible declaration;
/// accepting only this fully-qualified spelling lets generators emit contract
/// metadata beside a repeated public family without creating a macro escape
/// hatch in the public-surface extractor.
fn is_api_contract_inventory_submit(tokens: &[TokenTree], submit_index: usize) -> bool {
    matches!(
        tokens.get(submit_index.saturating_sub(6)..submit_index),
        Some([
            TokenTree::Ident(contract),
            TokenTree::Punct(colon_one),
            TokenTree::Punct(colon_two),
            TokenTree::Ident(inventory),
            TokenTree::Punct(colon_three),
            TokenTree::Punct(colon_four),
        ]) if contract == "sand_api_contract"
            && inventory == "inventory"
            && colon_one.as_char() == ':'
            && colon_two.as_char() == ':'
            && colon_three.as_char() == ':'
            && colon_four.as_char() == ':'
    ) && matches!(tokens.get(submit_index), Some(TokenTree::Ident(submit)) if submit == "submit")
}

/// Built-in literal macros are expressions only and cannot create an item in
/// a generator transcriber. They are common in static contract registrations.
fn inert_expression_macro(tokens: &[TokenTree], index: usize) -> bool {
    matches!(tokens.get(index), Some(TokenTree::Ident(name)) if matches!(name.to_string().as_str(), "concat" | "stringify"))
}

fn macro_path_starts_at_item_boundary(tokens: &[TokenTree], name_index: usize) -> bool {
    let mut path_start = name_index;
    while path_start >= 3
        && matches!(&tokens[path_start - 1], TokenTree::Punct(colon) if colon.as_char() == ':')
        && matches!(&tokens[path_start - 2], TokenTree::Punct(colon) if colon.as_char() == ':')
        && matches!(&tokens[path_start - 3], TokenTree::Ident(_))
    {
        path_start -= 3;
    }
    if path_start == 0 {
        return true;
    }
    match &tokens[path_start - 1] {
        TokenTree::Punct(punct) => punct.as_char() == ';',
        TokenTree::Group(group) => {
            group.delimiter() == proc_macro2::Delimiter::Brace
                || (group.delimiter() == proc_macro2::Delimiter::Bracket
                    && path_start >= 2
                    && matches!(&tokens[path_start - 2], TokenTree::Punct(hash) if hash.as_char() == '#'))
        }
        _ => false,
    }
}

fn token_ident(token: &TokenTree) -> Option<String> {
    match token {
        TokenTree::Ident(ident) => Some(ident.to_string()),
        _ => None,
    }
}

fn is_ident(token: Option<&TokenTree>, expected: &str) -> bool {
    matches!(token, Some(TokenTree::Ident(ident)) if ident == expected)
}

fn public_struct_metavariables(tokens: TokenStream) -> BTreeSet<String> {
    let flattened = flatten(tokens);
    flattened
        .windows(4)
        .filter_map(|tokens| match tokens {
            [
                TokenTree::Ident(public),
                TokenTree::Ident(structure),
                TokenTree::Punct(dollar),
                TokenTree::Ident(name),
            ] if public == "pub" && structure == "struct" && dollar.as_char() == '$' => {
                Some(name.to_string())
            }
            _ => None,
        })
        .collect()
}

fn flatten(tokens: TokenStream) -> Vec<TokenTree> {
    let mut flattened = Vec::new();
    for token in tokens {
        match token {
            TokenTree::Group(group) => flattened.extend(flatten(group.stream())),
            other => flattened.push(other),
        }
    }
    flattened
}
