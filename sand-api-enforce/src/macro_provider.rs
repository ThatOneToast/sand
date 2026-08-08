//! Providers for checked-in declarative macros that emit facade API families.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

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
            Self::UnsupportedGeneratedShape(message) => formatter.write_str(message),
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
    }) || flattened.windows(2).any(|window| {
        matches!(&window[0], TokenTree::Ident(_))
            && matches!(&window[1], TokenTree::Punct(punct) if punct.as_char() == '!')
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
        if bang.as_char() == '!' && macro_path_starts_at_item_boundary(tokens, index) {
            return Err(unsupported(format!(
                "generator invokes unmodeled `{name}!` macro at {position}; helper macros in item-producing positions are not auditable"
            )));
        }
    }
    Ok(())
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
