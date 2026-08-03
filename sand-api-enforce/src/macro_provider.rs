//! Providers for checked-in declarative macros that emit facade API families.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use crate::{GeneratedApi, ReachableKind};

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
}

impl fmt::Display for MacroProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Parse(message) => formatter.write_str(message),
            Self::MissingGenerator => formatter.write_str("resource_ref! generator is missing"),
            Self::MissingGeneratedTypes => {
                formatter.write_str("resource_ref! has no checked-in type declarations")
            }
            Self::MissingPublicMethods => {
                formatter.write_str("resource_ref! does not generate public methods")
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
        }
    }
}

impl std::error::Error for MacroProviderError {}

/// Expand the auditable public shape of every checked-in `resource_ref!`
/// invocation without executing a macro or duplicating its family members.
///
/// Public method names are read from the generator body itself. Adding a
/// method or an invocation therefore grows this provider during the same
/// ordinary build that grows the Rust surface.
pub fn resource_ref_provider(path: &Path) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    declarative_type_family_provider(
        path,
        "resource_ref",
        "sand_core::resource_ref",
        "generated_resource_refs",
    )
}

/// Expand the typed registry-ID wrapper family from `registry_id!`.
pub fn registry_id_provider(path: &Path) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    declarative_type_family_provider(
        path,
        "registry_id",
        "sand_components::registry",
        "generated_registry_ids",
    )
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
    let methods = public_function_names(generator.clone());
    if methods.is_empty() {
        return Err(MacroProviderError::MissingPublicMethods);
    }
    let macro_variants = generated_enum_variants(generator);
    if macro_variants.is_empty() {
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
            .chain(macro_variants.iter().cloned())
            .map(|variant| (variant, ReachableKind::Variant))
            .chain(
                methods
                    .iter()
                    .cloned()
                    .map(|method| (method, ReachableKind::Method)),
            )
            .collect::<Vec<_>>();
        members.sort();
        members.dedup();
        generated.push(GeneratedApi {
            identity: format!("sand_components::effect::{name}"),
            provider: "generated_effect_registry_enums".into(),
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
    let gamemode_types = public_struct_metavariables(gamemode_generator);
    if !["enter", "exit"]
        .into_iter()
        .all(|name| gamemode_types.contains(name))
    {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }
    let status_generator = named_generator(&file, "status_effect_marker")?;
    if !public_struct_metavariables(status_generator).contains("ty") {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }

    let mut names = BTreeSet::new();
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
            names.extend(identifiers[identifiers.len() - 2..].iter().cloned());
        } else if item.mac.path.is_ident("status_effect_marker") {
            let name = identifiers.first().ok_or_else(|| {
                MacroProviderError::Parse(format!(
                    "{}: status_effect_marker! must declare a marker type",
                    path.display()
                ))
            })?;
            names.insert(name.clone());
        }
    }
    if names.is_empty() {
        return Err(MacroProviderError::MissingGeneratedEvents);
    }
    Ok(names
        .into_iter()
        .map(|name| GeneratedApi {
            identity: format!("sand_core::events::{name}"),
            provider: "generated_event_markers".into(),
            kind: ReachableKind::Struct,
            members: Vec::new(),
            excluded: false,
        })
        .collect())
}

/// Describe public associated items emitted by the real `SandStorage` derive
/// from the same annotated struct declaration consumed by macro expansion.
/// Field accessor names come directly from named fields; no parallel member
/// manifest is maintained by the fixture or consuming build script.
pub fn sand_storage_derive_provider(
    path: &Path,
    identity_module: &str,
) -> Result<Vec<GeneratedApi>, MacroProviderError> {
    let source = fs::read_to_string(path)
        .map_err(|error| MacroProviderError::Io(format!("{}: {error}", path.display())))?;
    let file = syn::parse_file(&source)
        .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
    let mut generated = Vec::new();
    for item in &file.items {
        let syn::Item::Struct(structure) = item else {
            continue;
        };
        let derives_sand_storage = structure.attrs.iter().any(|attribute| {
            if !attribute.path().is_ident("derive") {
                return false;
            }
            attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|derives| {
                    derives.iter().any(|derive| {
                        derive
                            .segments
                            .last()
                            .is_some_and(|segment| segment.ident == "SandStorage")
                    })
                })
        });
        if !derives_sand_storage {
            continue;
        }
        let derive_input = syn::parse2::<syn::DeriveInput>(structure.to_token_stream())
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let members = sand_api_contract::syntax::sand_storage_generated_member_names(&derive_input)
            .map_err(|error| MacroProviderError::Parse(format!("{}: {error}", path.display())))?;
        let owner = format!("{identity_module}::{}", structure.ident);
        for (index, name) in members.into_iter().enumerate() {
            generated.push(GeneratedApi {
                identity: format!("{owner}::{name}"),
                provider: "sand_storage_derive".into(),
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
    if generated.is_empty() {
        return Err(MacroProviderError::MissingGeneratedTypes);
    }
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(generated)
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

fn generated_enum_variants(tokens: TokenStream) -> BTreeSet<String> {
    let Some(body) = find_enum_body(tokens) else {
        return BTreeSet::new();
    };
    let tokens = body.into_iter().collect::<Vec<_>>();
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let TokenTree::Ident(ident) = token else {
                return None;
            };
            let preceded_by_dollar = index
                .checked_sub(1)
                .and_then(|previous| tokens.get(previous))
                .is_some_and(
                    |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == '$'),
                );
            (!preceded_by_dollar).then(|| ident.to_string())
        })
        .collect()
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
    let methods = public_function_names(generator);
    if methods.is_empty() {
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
            kind: ReachableKind::Struct,
            members: methods
                .iter()
                .map(|method| (method.clone(), ReachableKind::Method))
                .collect(),
            excluded: false,
        })
        .collect())
}

fn public_function_names(tokens: TokenStream) -> BTreeSet<String> {
    let flattened = flatten(tokens);
    flattened
        .windows(3)
        .filter_map(|tokens| match tokens {
            [
                TokenTree::Ident(public),
                TokenTree::Ident(function),
                TokenTree::Ident(name),
            ] if public == "pub" && function == "fn" => Some(name.to_string()),
            _ => None,
        })
        .collect()
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
