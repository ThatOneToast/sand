//! Build-time discovery of authoritative contract declarations.
//!
//! Runtime inventory is intentionally not available to a build script.  The
//! build therefore reads the same `#[api]` arguments and facade-owned
//! `register!` declarations that produce runtime registrations.  This is a
//! contract source, not an exemption list: each declaration has a canonical
//! public path and aliases which must resolve to one reachable Rust identity.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use sand_api_contract::ApiKind;
use sand_api_contract::syntax::{
    ContractSemantics, parse_contract_args, validate_contract_semantics,
};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Item, LitStr, Token};

use crate::{ContractIdentity, ReachableApi, SourceDefinition};

/// One contract identity as authored in Rust source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDeclaration {
    pub canonical_path: String,
    pub aliases: BTreeSet<String>,
    pub source: PathBuf,
    /// Source item carrying `#[api]`. Facade-owned `register!` providers have
    /// no attached definition and are resolved by their explicit path.
    pub definition: Option<SourceDefinition>,
    pub facade: Option<FacadeContract>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacadeContract {
    pub kind: ApiKind,
    pub signature: Option<String>,
    pub parameters: Option<BTreeSet<String>>,
    pub returns: Option<bool>,
    pub runtime_signature: String,
    pub summary: String,
    pub context: String,
    pub minecraft: String,
    pub use_when: Vec<String>,
    pub avoid_when: Vec<String>,
    pub parameter_docs: Vec<(String, String)>,
    pub return_doc: Option<String>,
    pub example: String,
    pub availability: Vec<String>,
    pub canonical_module: String,
    pub family: bool,
}

/// Structural facts read from the independently discovered source
/// declaration. Catalog assembly uses these instead of facade-authored shape
/// strings, so signatures cannot drift from the Rust API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionShape {
    pub signature: String,
    pub parameters: Vec<(String, String)>,
    pub return_type: Option<String>,
    pub documentation: String,
    pub has_receiver: bool,
}

pub fn definition_shape(item: &ReachableApi) -> Result<Option<DefinitionShape>, String> {
    let Some(definition) = &item.definition else {
        return Ok(None);
    };
    if let Some(callable) = callable_shape(definition)? {
        let parameters = callable
            .signature
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(argument) => {
                    let name = match argument.pat.as_ref() {
                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                        pattern => pattern.to_token_stream().to_string(),
                    };
                    Some((name, argument.ty.to_token_stream().to_string()))
                }
            })
            .collect();
        let return_type = match &callable.signature.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
        };
        return Ok(Some(DefinitionShape {
            signature: callable_signature(&callable.signature, callable.visibility.as_ref()),
            parameters,
            return_type,
            documentation: String::new(),
            has_receiver: callable
                .signature
                .inputs
                .iter()
                .any(|input| matches!(input, syn::FnArg::Receiver(_))),
        }));
    }
    let source = fs::read_to_string(&definition.source)
        .map_err(|error| format!("failed to read reachable definition: {error}"))?;
    let file = syn::parse_file(&source)
        .map_err(|error| format!("failed to parse reachable definition: {error}"))?;
    Ok(
        find_declaration_signature(&file.items, definition).map(|signature| DefinitionShape {
            signature,
            parameters: Vec::new(),
            return_type: None,
            documentation: String::new(),
            has_receiver: false,
        }),
    )
}

/// Derive structural metadata for a complete reachable surface with each
/// source file parsed exactly once.
pub fn definition_shapes(
    items: &[ReachableApi],
) -> Result<BTreeMap<String, DefinitionShape>, String> {
    let mut by_source = BTreeMap::<PathBuf, Vec<&ReachableApi>>::new();
    for item in items {
        if let Some(definition) = &item.definition {
            by_source
                .entry(definition.source.clone())
                .or_default()
                .push(item);
        }
    }
    let mut result = BTreeMap::new();
    for (source_path, source_items) in by_source {
        let source = fs::read_to_string(&source_path)
            .map_err(|error| format!("failed to read reachable definition: {error}"))?;
        let file = syn::parse_file(&source)
            .map_err(|error| format!("failed to parse reachable definition: {error}"))?;
        let mut locations = BTreeMap::new();
        collect_definition_shapes(&file.items, &mut locations);
        for item in source_items {
            let definition = item.definition.as_ref().expect("grouped by definition");
            if let Some(shape) = locations.get(&(definition.line, definition.column)) {
                result.insert(item.identity.clone(), shape.clone());
            }
        }
    }
    Ok(result)
}

fn collect_definition_shapes(
    items: &[Item],
    shapes: &mut BTreeMap<(usize, usize), DefinitionShape>,
) {
    for item in items {
        let location = item.span().start();
        if let Some(signature) = item_definition_signature(item) {
            shapes.insert(
                (location.line, location.column),
                DefinitionShape {
                    signature,
                    parameters: Vec::new(),
                    return_type: None,
                    documentation: rustdoc(crate::item_attrs(item)),
                    has_receiver: false,
                },
            );
        }
        match item {
            Item::Fn(function) => {
                shapes.insert(
                    (location.line, location.column),
                    definition_shape_from_signature(
                        &function.sig,
                        Some(&function.vis),
                        &function.attrs,
                    ),
                );
            }
            Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_definition_shapes(nested, shapes);
                }
            }
            Item::Impl(block) => {
                for member in &block.items {
                    let location = member.span().start();
                    let shape = match member {
                        syn::ImplItem::Fn(function) => definition_shape_from_signature(
                            &function.sig,
                            Some(&function.vis),
                            &function.attrs,
                        ),
                        _ => DefinitionShape {
                            signature: impl_member_signature(member),
                            parameters: Vec::new(),
                            return_type: None,
                            documentation: rustdoc(impl_member_attrs(member)),
                            has_receiver: false,
                        },
                    };
                    shapes.insert((location.line, location.column), shape);
                }
            }
            Item::Trait(block) => {
                for member in &block.items {
                    let location = member.span().start();
                    let shape = match member {
                        syn::TraitItem::Fn(function) => {
                            definition_shape_from_signature(&function.sig, None, &function.attrs)
                        }
                        _ => DefinitionShape {
                            signature: trait_member_signature(member),
                            parameters: Vec::new(),
                            return_type: None,
                            documentation: rustdoc(trait_member_attrs(member)),
                            has_receiver: false,
                        },
                    };
                    shapes.insert((location.line, location.column), shape);
                }
            }
            Item::Struct(value) => {
                for field in &value.fields {
                    let location = field.span().start();
                    shapes.insert(
                        (location.line, location.column),
                        DefinitionShape {
                            signature: field_signature(field, true),
                            parameters: Vec::new(),
                            return_type: None,
                            documentation: rustdoc(&field.attrs),
                            has_receiver: false,
                        },
                    );
                }
            }
            Item::Enum(value) => {
                for variant in &value.variants {
                    let location = variant.span().start();
                    shapes.insert(
                        (location.line, location.column),
                        DefinitionShape {
                            signature: variant_signature(variant),
                            parameters: Vec::new(),
                            return_type: None,
                            documentation: rustdoc(&variant.attrs),
                            has_receiver: false,
                        },
                    );
                    for field in &variant.fields {
                        let location = field.span().start();
                        shapes.insert(
                            (location.line, location.column),
                            DefinitionShape {
                                signature: field_signature(field, false),
                                parameters: Vec::new(),
                                return_type: None,
                                documentation: rustdoc(&field.attrs),
                                has_receiver: false,
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn impl_member_attrs(member: &syn::ImplItem) -> &[Attribute] {
    match member {
        syn::ImplItem::Const(value) => &value.attrs,
        syn::ImplItem::Fn(value) => &value.attrs,
        syn::ImplItem::Macro(value) => &value.attrs,
        syn::ImplItem::Type(value) => &value.attrs,
        _ => &[],
    }
}

fn trait_member_attrs(member: &syn::TraitItem) -> &[Attribute] {
    match member {
        syn::TraitItem::Const(value) => &value.attrs,
        syn::TraitItem::Fn(value) => &value.attrs,
        syn::TraitItem::Macro(value) => &value.attrs,
        syn::TraitItem::Type(value) => &value.attrs,
        _ => &[],
    }
}

fn impl_member_signature(member: &syn::ImplItem) -> String {
    let mut member = member.clone();
    match &mut member {
        syn::ImplItem::Const(value) => value.attrs.clear(),
        syn::ImplItem::Fn(value) => value.attrs.clear(),
        syn::ImplItem::Macro(value) => value.attrs.clear(),
        syn::ImplItem::Type(value) => value.attrs.clear(),
        _ => {}
    }
    member.to_token_stream().to_string()
}

fn trait_member_signature(member: &syn::TraitItem) -> String {
    let mut member = member.clone();
    match &mut member {
        syn::TraitItem::Const(value) => value.attrs.clear(),
        syn::TraitItem::Fn(value) => value.attrs.clear(),
        syn::TraitItem::Macro(value) => value.attrs.clear(),
        syn::TraitItem::Type(value) => value.attrs.clear(),
        _ => {}
    }
    member.to_token_stream().to_string()
}

fn variant_signature(variant: &syn::Variant) -> String {
    let mut variant = variant.clone();
    variant.attrs.clear();
    for field in &mut variant.fields {
        field.attrs.clear();
    }
    variant.to_token_stream().to_string()
}

fn definition_shape_from_signature(
    signature: &syn::Signature,
    visibility: Option<&syn::Visibility>,
    attrs: &[Attribute],
) -> DefinitionShape {
    let parameters = signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(argument) => {
                let name = match argument.pat.as_ref() {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    pattern => pattern.to_token_stream().to_string(),
                };
                Some((name, argument.ty.to_token_stream().to_string()))
            }
        })
        .collect();
    let return_type = match &signature.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
    };
    DefinitionShape {
        signature: callable_signature(signature, visibility),
        parameters,
        return_type,
        documentation: rustdoc(attrs),
        has_receiver: signature
            .inputs
            .iter()
            .any(|input| matches!(input, syn::FnArg::Receiver(_))),
    }
}

fn callable_signature(signature: &syn::Signature, visibility: Option<&syn::Visibility>) -> String {
    match visibility {
        Some(visibility) => format!(
            "{} {}",
            visibility.to_token_stream(),
            signature.to_token_stream()
        ),
        None => signature.to_token_stream().to_string(),
    }
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn field_signature(field: &syn::Field, include_visibility: bool) -> String {
    let visibility = include_visibility
        .then(|| field.vis.to_token_stream().to_string())
        .unwrap_or_default();
    let declaration = match &field.ident {
        Some(name) => format!("{name}: {}", field.ty.to_token_stream()),
        None => field.ty.to_token_stream().to_string(),
    };
    format!("{visibility} {declaration}")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn rustdoc(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(value) if value.path.is_ident("doc") => match &value.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) => Some(text.value().trim().to_owned()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn item_definition_signature(item: &Item) -> Option<String> {
    let signature = match item {
        Item::Mod(value) => format!("{} mod {}", value.vis.to_token_stream(), value.ident),
        Item::Struct(value) => format!(
            "{} struct {} {}",
            value.vis.to_token_stream(),
            value.ident,
            value.generics.to_token_stream()
        ),
        Item::Enum(value) => format!(
            "{} enum {} {}",
            value.vis.to_token_stream(),
            value.ident,
            value.generics.to_token_stream()
        ),
        Item::Trait(value) => trait_signature(value),
        Item::Type(value) => format!(
            "{} type {} {} = {}",
            value.vis.to_token_stream(),
            value.ident,
            value.generics.to_token_stream(),
            value.ty.to_token_stream()
        ),
        Item::Const(value) => format!(
            "{} const {} : {} = {}",
            value.vis.to_token_stream(),
            value.ident,
            value.ty.to_token_stream(),
            value.expr.to_token_stream()
        ),
        Item::Fn(_) => return None,
        _ => item.to_token_stream().to_string(),
    };
    Some(signature.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn trait_signature(value: &syn::ItemTrait) -> String {
    let generics = if value.generics.params.is_empty() {
        String::new()
    } else {
        format!("<{}>", value.generics.params.to_token_stream())
    };
    let supertraits = if value.supertraits.is_empty() {
        String::new()
    } else {
        format!(": {}", value.supertraits.to_token_stream())
    };
    format!(
        "{} trait {} {} {} {}",
        value.vis.to_token_stream(),
        value.ident,
        generics,
        supertraits,
        value.generics.where_clause.to_token_stream()
    )
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn find_declaration_signature(items: &[Item], definition: &SourceDefinition) -> Option<String> {
    for item in items {
        if same_location(item.span().start(), definition) {
            let signature = match item {
                Item::Mod(value) => format!("{}mod {}", value.vis.to_token_stream(), value.ident),
                Item::Struct(value) => format!(
                    "{} struct {} {}",
                    value.vis.to_token_stream(),
                    value.ident,
                    value.generics.to_token_stream()
                ),
                Item::Enum(value) => format!(
                    "{} enum {} {}",
                    value.vis.to_token_stream(),
                    value.ident,
                    value.generics.to_token_stream()
                ),
                Item::Trait(value) => trait_signature(value),
                Item::Type(value) => format!(
                    "{} type {} {} = {}",
                    value.vis.to_token_stream(),
                    value.ident,
                    value.generics.to_token_stream(),
                    value.ty.to_token_stream()
                ),
                Item::Const(value) => format!(
                    "{} const {} : {} = {}",
                    value.vis.to_token_stream(),
                    value.ident,
                    value.ty.to_token_stream(),
                    value.expr.to_token_stream()
                ),
                _ => item.to_token_stream().to_string(),
            };
            return Some(signature.split_whitespace().collect::<Vec<_>>().join(" "));
        }
        match item {
            Item::Mod(module) => {
                if let Some((_, nested)) = &module.content
                    && let Some(signature) = find_declaration_signature(nested, definition)
                {
                    return Some(signature);
                }
            }
            Item::Struct(value) => {
                for field in &value.fields {
                    if same_location(field.span().start(), definition) {
                        return Some(field_signature(field, true));
                    }
                }
            }
            Item::Enum(value) => {
                for variant in &value.variants {
                    if same_location(variant.span().start(), definition) {
                        return Some(variant_signature(variant));
                    }
                    for field in &variant.fields {
                        if same_location(field.span().start(), definition) {
                            return Some(field_signature(field, false));
                        }
                    }
                }
            }
            Item::Impl(block) => {
                for member in &block.items {
                    if same_location(member.span().start(), definition) {
                        return Some(impl_member_signature(member));
                    }
                }
            }
            Item::Trait(block) => {
                for member in &block.items {
                    if same_location(member.span().start(), definition) {
                        return Some(trait_member_signature(member));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractSourceError {
    Io(String),
    Parse(String),
    MissingCanonicalPath(PathBuf),
    DuplicateCanonicalPath(String),
    DuplicateLookupPath {
        path: String,
        identities: Vec<String>,
    },
    DuplicateIdentity {
        identity: String,
        paths: Vec<String>,
    },
    UnreachablePath(String),
    AmbiguousPath {
        path: String,
        identities: Vec<String>,
    },
    AliasTargetsDifferentIdentity {
        canonical_path: String,
        alias: String,
    },
    ContractAttachedToDifferentItem {
        canonical_path: String,
        annotated: SourceDefinition,
        reachable: Option<SourceDefinition>,
    },
    InvalidFacadeContract {
        path: String,
        message: String,
    },
}

impl fmt::Display for ContractSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Parse(message) => formatter.write_str(message),
            Self::MissingCanonicalPath(source) => write!(
                formatter,
                "{}: facade API contracts used by build enforcement require an explicit `path`",
                source.display()
            ),
            Self::DuplicateCanonicalPath(path) => {
                write!(formatter, "duplicate build-time API contract path `{path}`")
            }
            Self::DuplicateLookupPath { path, identities } => write!(
                formatter,
                "API lookup path `{path}` is claimed by multiple identities: {}",
                identities.join(", ")
            ),
            Self::DuplicateIdentity { identity, paths } => write!(
                formatter,
                "reachable API `{identity}` has multiple contracts: {}",
                paths.join(", ")
            ),
            Self::UnreachablePath(path) => {
                write!(
                    formatter,
                    "API contract path `{path}` is not reachable through `sand`"
                )
            }
            Self::AmbiguousPath { path, identities } => write!(
                formatter,
                "API contract path `{path}` resolves to multiple identities: {}",
                identities.join(", ")
            ),
            Self::AliasTargetsDifferentIdentity {
                canonical_path,
                alias,
            } => write!(
                formatter,
                "API contract alias `{alias}` does not resolve to the same item as `{canonical_path}`"
            ),
            Self::ContractAttachedToDifferentItem {
                canonical_path,
                annotated,
                reachable,
            } => write!(
                formatter,
                "API contract `{canonical_path}` is attached to {}:{}:{} but that path resolves to {}",
                annotated.source.display(),
                annotated.line,
                annotated.column,
                reachable.as_ref().map_or_else(
                    || "a generated API without a source declaration".to_owned(),
                    |definition| format!(
                        "{}:{}:{}",
                        definition.source.display(),
                        definition.line,
                        definition.column
                    ),
                )
            ),
            Self::InvalidFacadeContract { path, message } => {
                write!(formatter, "invalid facade API contract `{path}`: {message}")
            }
        }
    }
}

impl std::error::Error for ContractSourceError {}

/// Reject collisions across the combined canonical-path and alias namespace.
pub fn validate_contract_lookup_namespace(
    contracts: &[ContractIdentity],
) -> Result<(), ContractSourceError> {
    let mut paths = BTreeMap::<&str, &str>::new();
    for contract in contracts {
        for path in std::iter::once(contract.canonical_path.as_str())
            .chain(contract.aliases.iter().map(String::as_str))
        {
            if let Some(existing) = paths.insert(path, contract.identity.as_str()) {
                return Err(ContractSourceError::DuplicateLookupPath {
                    path: path.to_owned(),
                    identities: vec![existing.to_owned(), contract.identity.clone()],
                });
            }
        }
    }
    Ok(())
}

/// Read `#[api]` and facade-owned `register!` declarations from Rust files.
pub fn contract_declarations_from_files(
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<Vec<ContractDeclaration>, ContractSourceError> {
    let mut declarations = Vec::new();
    for path in paths {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|error| ContractSourceError::Io(format!("{}: {error}", path.display())))?;
        let file = syn::parse_file(&source)
            .map_err(|error| ContractSourceError::Parse(format!("{}: {error}", path.display())))?;
        inspect_items(&file.items, path, &mut declarations)?;
    }
    declarations.sort_by(|left, right| left.canonical_path.cmp(&right.canonical_path));
    let mut paths = BTreeSet::new();
    for declaration in &declarations {
        if !paths.insert(declaration.canonical_path.as_str()) {
            return Err(ContractSourceError::DuplicateCanonicalPath(
                declaration.canonical_path.clone(),
            ));
        }
    }
    Ok(declarations)
}

/// Resolve authored public paths to stable source/generator identities.
///
/// Canonical paths and every explicitly declared alias are checked even while
/// their scope remains pending.  Complete alias equality is ratcheted when the
/// owning scope becomes enforced.
pub fn resolve_contract_identities(
    reachable: &[ReachableApi],
    declarations: &[ContractDeclaration],
) -> Result<Vec<ContractIdentity>, Vec<ContractSourceError>> {
    let mut by_path = BTreeMap::<&str, Vec<&ReachableApi>>::new();
    for item in reachable {
        for path in &item.paths {
            by_path.entry(path).or_default().push(item);
        }
    }
    let mut identities = BTreeMap::<&str, Vec<&ContractDeclaration>>::new();
    let mut resolved = Vec::new();
    let mut errors = Vec::new();
    for declaration in declarations {
        let Some(items) = by_path.get(declaration.canonical_path.as_str()) else {
            errors.push(ContractSourceError::UnreachablePath(
                declaration.canonical_path.clone(),
            ));
            continue;
        };
        let [item] = items.as_slice() else {
            errors.push(ContractSourceError::AmbiguousPath {
                path: declaration.canonical_path.clone(),
                identities: items.iter().map(|item| item.identity.clone()).collect(),
            });
            continue;
        };
        if let Some(facade) = &declaration.facade
            && api_kind(item.kind) != Some(facade.kind)
        {
            errors.push(ContractSourceError::InvalidFacadeContract {
                path: declaration.canonical_path.clone(),
                message: format!(
                    "declares kind {:?}, but the reachable definition is {:?}",
                    facade.kind, item.kind
                ),
            });
            continue;
        }
        if let (Some(facade), Some(definition)) = (&declaration.facade, &item.definition)
            && facade.signature.is_some()
            && facade.kind != ApiKind::Macro
            && !facade.family
        {
            match callable_shape(definition) {
                Ok(Some(shape)) => {
                    if facade.parameters.as_ref() != Some(&shape.parameters) {
                        errors.push(ContractSourceError::InvalidFacadeContract {
                            path: declaration.canonical_path.clone(),
                            message: format!(
                                "parameter metadata {:?} does not match reachable parameters {:?}",
                                facade.parameters, shape.parameters
                            ),
                        });
                        continue;
                    }
                    if facade.returns != Some(shape.returns) {
                        errors.push(ContractSourceError::InvalidFacadeContract {
                            path: declaration.canonical_path.clone(),
                            message: format!(
                                "return metadata {:?} does not match reachable return shape {}",
                                facade.returns, shape.returns
                            ),
                        });
                        continue;
                    }
                    let authored = parse_authored_signature(
                        facade.signature.as_deref().expect("checked above"),
                    );
                    match authored {
                        Ok(signature)
                            if normalize_signature(&signature)
                                == normalize_signature(&shape.signature) => {}
                        Ok(signature) => {
                            errors.push(ContractSourceError::InvalidFacadeContract {
                                path: declaration.canonical_path.clone(),
                                message: format!(
                                    "stale signature `{}`; reachable definition is `{}`",
                                    signature.to_token_stream(),
                                    shape.signature.to_token_stream()
                                ),
                            });
                            continue;
                        }
                        Err(message) => {
                            errors.push(ContractSourceError::InvalidFacadeContract {
                                path: declaration.canonical_path.clone(),
                                message,
                            });
                            continue;
                        }
                    }
                }
                Ok(None) => {}
                Err(message) => {
                    errors.push(ContractSourceError::InvalidFacadeContract {
                        path: declaration.canonical_path.clone(),
                        message,
                    });
                    continue;
                }
            }
        }
        if let Some(annotated) = &declaration.definition
            && item.definition.as_ref() != Some(annotated)
        {
            errors.push(ContractSourceError::ContractAttachedToDifferentItem {
                canonical_path: declaration.canonical_path.clone(),
                annotated: annotated.clone(),
                reachable: item.definition.clone(),
            });
            continue;
        }
        let mut aliases_valid = true;
        for alias in &declaration.aliases {
            match by_path.get(alias.as_str()).map(Vec::as_slice) {
                Some([alias_item]) if alias_item.identity == item.identity => {}
                Some([_]) => {
                    aliases_valid = false;
                    errors.push(ContractSourceError::AliasTargetsDifferentIdentity {
                        canonical_path: declaration.canonical_path.clone(),
                        alias: alias.clone(),
                    });
                }
                Some(alias_items) => {
                    aliases_valid = false;
                    errors.push(ContractSourceError::AmbiguousPath {
                        path: alias.clone(),
                        identities: alias_items
                            .iter()
                            .map(|item| item.identity.clone())
                            .collect(),
                    });
                }
                None => {
                    aliases_valid = false;
                    errors.push(ContractSourceError::UnreachablePath(alias.clone()));
                }
            }
        }
        if aliases_valid {
            identities
                .entry(item.identity.as_str())
                .or_default()
                .push(declaration);
            resolved.push(ContractIdentity {
                identity: item.identity.clone(),
                canonical_path: declaration.canonical_path.clone(),
                aliases: declaration.aliases.clone(),
            });
        }
    }
    for (identity, declarations) in identities {
        if declarations.len() > 1 {
            errors.push(ContractSourceError::DuplicateIdentity {
                identity: identity.to_owned(),
                paths: declarations
                    .iter()
                    .map(|declaration| declaration.canonical_path.clone())
                    .collect(),
            });
        }
    }
    resolved.sort_by(|left, right| left.identity.cmp(&right.identity));
    if errors.is_empty() {
        Ok(resolved)
    } else {
        errors.sort_by_key(ToString::to_string);
        Err(errors)
    }
}

struct CallableShape {
    signature: syn::Signature,
    visibility: Option<syn::Visibility>,
    parameters: BTreeSet<String>,
    returns: bool,
}

fn callable_shape(definition: &SourceDefinition) -> Result<Option<CallableShape>, String> {
    let source = fs::read_to_string(&definition.source)
        .map_err(|error| format!("failed to read reachable definition: {error}"))?;
    let file = syn::parse_file(&source)
        .map_err(|error| format!("failed to parse reachable definition: {error}"))?;
    Ok(find_callable(&file.items, definition)
        .map(|(signature, visibility)| shape_from_signature(signature, visibility)))
}

fn find_callable(
    items: &[Item],
    definition: &SourceDefinition,
) -> Option<(syn::Signature, Option<syn::Visibility>)> {
    for item in items {
        if let Item::Fn(function) = item
            && same_location(function.span().start(), definition)
        {
            return Some((function.sig.clone(), Some(function.vis.clone())));
        }
        match item {
            Item::Mod(module) => {
                if let Some((_, nested)) = &module.content
                    && let Some(signature) = find_callable(nested, definition)
                {
                    return Some(signature);
                }
            }
            Item::Impl(block) => {
                for member in &block.items {
                    if let syn::ImplItem::Fn(function) = member
                        && same_location(function.span().start(), definition)
                    {
                        return Some((function.sig.clone(), Some(function.vis.clone())));
                    }
                }
            }
            Item::Trait(block) => {
                for member in &block.items {
                    if let syn::TraitItem::Fn(function) = member
                        && same_location(function.span().start(), definition)
                    {
                        return Some((function.sig.clone(), None));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn same_location(location: proc_macro2::LineColumn, definition: &SourceDefinition) -> bool {
    location.line == definition.line && location.column == definition.column
}

fn shape_from_signature(
    signature: syn::Signature,
    visibility: Option<syn::Visibility>,
) -> CallableShape {
    let parameters = signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(argument) => match argument.pat.as_ref() {
                syn::Pat::Ident(ident) => Some(ident.ident.to_string()),
                _ => Some(argument.pat.to_token_stream().to_string()),
            },
        })
        .collect();
    let returns = !matches!(signature.output, syn::ReturnType::Default);
    CallableShape {
        signature,
        visibility,
        parameters,
        returns,
    }
}

fn parse_authored_signature(signature: &str) -> Result<syn::Signature, String> {
    syn::parse_str::<syn::TraitItemFn>(&format!("{signature};"))
        .map(|function| function.sig)
        .or_else(|_| {
            syn::parse_str::<syn::ItemFn>(&format!("{signature} {{ unreachable!() }}"))
                .map(|function| function.sig)
        })
        .map_err(|error| format!("invalid authored signature: {error}"))
}

fn normalize_signature(signature: &syn::Signature) -> String {
    signature
        .to_token_stream()
        .to_string()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn api_kind(kind: crate::ReachableKind) -> Option<ApiKind> {
    Some(match kind {
        crate::ReachableKind::Module => ApiKind::Module,
        crate::ReachableKind::Struct => ApiKind::Struct,
        crate::ReachableKind::Enum => ApiKind::Enum,
        crate::ReachableKind::Variant => ApiKind::Variant,
        crate::ReachableKind::Field => ApiKind::Field,
        crate::ReachableKind::Trait => ApiKind::Trait,
        crate::ReachableKind::Function => ApiKind::Function,
        crate::ReachableKind::Method => ApiKind::Method,
        crate::ReachableKind::TraitMethod => ApiKind::TraitMethod,
        crate::ReachableKind::AssociatedConst => ApiKind::AssociatedConst,
        crate::ReachableKind::AssociatedType => ApiKind::AssociatedType,
        crate::ReachableKind::TypeAlias => ApiKind::TypeAlias,
        crate::ReachableKind::Constant => ApiKind::Constant,
        crate::ReachableKind::Macro
        | crate::ReachableKind::FunctionLikeMacro
        | crate::ReachableKind::AttributeMacro
        | crate::ReachableKind::DeriveMacro => ApiKind::Macro,
        crate::ReachableKind::Union | crate::ReachableKind::Static => return None,
    })
}

fn inspect_items(
    items: &[Item],
    source: &Path,
    declarations: &mut Vec<ContractDeclaration>,
) -> Result<(), ContractSourceError> {
    for item in items {
        if let Some(attribute) = crate::item_attrs(item)
            .iter()
            .find(|attribute| is_api_path(attribute.path()))
        {
            let declaration = declaration_from_attribute(
                attribute,
                source,
                Some(source_definition(source, item.span().start())),
            )?;
            let parent_path = declaration.canonical_path.clone();
            let parent_aliases = declaration.aliases.clone();
            let args = api_args(attribute, source)?;
            declarations.push(declaration);
            for member in args
                .fields
                .into_iter()
                .flatten()
                .chain(args.variants.into_iter().flatten())
            {
                let name = member.name.to_string();
                declarations.push(ContractDeclaration {
                    canonical_path: format!("{parent_path}::{name}"),
                    aliases: parent_aliases
                        .iter()
                        .map(|alias| format!("{alias}::{name}"))
                        .collect(),
                    source: source.to_owned(),
                    definition: member_source_definition(item, &name, source),
                    facade: None,
                });
            }
            for member in args.variant_fields.into_iter().flatten() {
                let name = format!("{}::{}", member.variant, member.name);
                declarations.push(ContractDeclaration {
                    canonical_path: format!("{parent_path}::{name}"),
                    aliases: parent_aliases
                        .iter()
                        .map(|alias| format!("{alias}::{name}"))
                        .collect(),
                    source: source.to_owned(),
                    definition: member_source_definition(item, &name, source),
                    facade: None,
                });
            }
        }
        match item {
            Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    inspect_items(nested, source, declarations)?;
                }
            }
            Item::Impl(block) => {
                for member in &block.items {
                    let attrs = match member {
                        syn::ImplItem::Const(value) => &value.attrs,
                        syn::ImplItem::Fn(value) => &value.attrs,
                        syn::ImplItem::Type(value) => &value.attrs,
                        syn::ImplItem::Macro(value) => &value.attrs,
                        _ => continue,
                    };
                    inspect_attributes(attrs, source, member.span().start(), declarations, true)?;
                }
            }
            Item::Trait(item) => {
                for member in &item.items {
                    let attrs = match member {
                        syn::TraitItem::Const(value) => &value.attrs,
                        syn::TraitItem::Fn(value) => &value.attrs,
                        syn::TraitItem::Type(value) => &value.attrs,
                        syn::TraitItem::Macro(value) => &value.attrs,
                        _ => continue,
                    };
                    inspect_attributes(attrs, source, member.span().start(), declarations, true)?;
                }
            }
            Item::Macro(item)
                if item.mac.path.is_ident("register")
                    || item.mac.path.is_ident("register_event_marker")
                    || item.mac.path.is_ident("register_event_api")
                    || item.mac.path.is_ident("register_entity_api")
                    || item.mac.path.is_ident("register_state_api")
                    || item.mac.path.is_ident("register_participant_api")
                    || item.mac.path.is_ident("register_text_api")
                    || item.mac.path.is_ident("register_data_api")
                    || item.mac.path.is_ident("register_systems_api")
                    || item.mac.path.is_ident("register_command_api")
                    || item.mac.path.is_ident("register_component_api")
                    || item.mac.path.is_ident("register_version_api")
                    || item.mac.path.is_ident("register_resourcepack_api") =>
            {
                let macro_name = item
                    .mac
                    .path
                    .segments
                    .last()
                    .expect("macro path")
                    .ident
                    .to_string();
                let mut parsed =
                    syn::parse2::<RegisterArgs>(item.mac.tokens.clone()).map_err(|error| {
                        ContractSourceError::Parse(format!("{}: {error}", source.display()))
                    })?;
                parsed.complete_family_semantics(&macro_name);
                parsed.validate(&macro_name).map_err(|message| {
                    ContractSourceError::Parse(format!("{}: {message}", source.display()))
                })?;
                let facade = parsed.facade_contract()?;
                let path = parsed.path.ok_or_else(|| {
                    ContractSourceError::Parse(format!(
                        "{}: facade register! contract is missing `path`",
                        source.display()
                    ))
                })?;
                declarations.push(ContractDeclaration {
                    canonical_path: path,
                    aliases: parsed.aliases,
                    source: source.to_owned(),
                    definition: None,
                    facade: Some(facade),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn inspect_attributes(
    attributes: &[Attribute],
    source: &Path,
    location: proc_macro2::LineColumn,
    declarations: &mut Vec<ContractDeclaration>,
    derive_parent_aliases: bool,
) -> Result<(), ContractSourceError> {
    if let Some(attribute) = attributes
        .iter()
        .find(|attribute| is_api_path(attribute.path()))
    {
        let args = api_args(attribute, source)?;
        let aliases_are_authored = args.aliases.is_some();
        let mut declaration =
            declaration_from_args(args, source, Some(source_definition(source, location)))?;
        if !aliases_are_authored
            && derive_parent_aliases
            && let Some((parent_path, member)) = declaration.canonical_path.rsplit_once("::")
            && let Some(parent_aliases) = declarations
                .iter()
                .rev()
                .find(|parent| parent.canonical_path == parent_path)
                .map(|parent| &parent.aliases)
        {
            declaration.aliases = parent_aliases
                .iter()
                .map(|alias| format!("{alias}::{member}"))
                .collect();
        }
        declarations.push(declaration);
    }
    Ok(())
}

fn is_api_path(path: &syn::Path) -> bool {
    path.is_ident("api")
        || path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "api")
}

fn declaration_from_attribute(
    attribute: &Attribute,
    source: &Path,
    definition: Option<SourceDefinition>,
) -> Result<ContractDeclaration, ContractSourceError> {
    declaration_from_args(api_args(attribute, source)?, source, definition)
}

fn declaration_from_args(
    args: sand_api_contract::syntax::ContractArgs,
    source: &Path,
    definition: Option<SourceDefinition>,
) -> Result<ContractDeclaration, ContractSourceError> {
    let canonical_path = args
        .path
        .map(|path| path.value())
        .ok_or_else(|| ContractSourceError::MissingCanonicalPath(source.to_owned()))?;
    Ok(ContractDeclaration {
        canonical_path,
        aliases: args
            .aliases
            .unwrap_or_default()
            .into_iter()
            .map(|alias| alias.value())
            .collect(),
        source: source.to_owned(),
        definition,
        facade: None,
    })
}

fn source_definition(source: &Path, location: proc_macro2::LineColumn) -> SourceDefinition {
    SourceDefinition {
        source: fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf()),
        line: location.line,
        column: location.column,
    }
}

fn member_source_definition(item: &Item, name: &str, source: &Path) -> Option<SourceDefinition> {
    let location = match item {
        Item::Struct(value) => value.fields.iter().enumerate().find_map(|(index, field)| {
            let field_name = field
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ToString::to_string);
            (field_name == name).then(|| field.span().start())
        }),
        Item::Union(value) => value.fields.named.iter().find_map(|field| {
            (field.ident.as_ref().is_some_and(|ident| ident == name)).then(|| field.span().start())
        }),
        Item::Enum(value) => {
            let (variant_name, field_name) = name.split_once("::").unwrap_or((name, ""));
            let variant = value
                .variants
                .iter()
                .find(|variant| variant.ident == variant_name)?;
            if field_name.is_empty() {
                Some(variant.span().start())
            } else {
                variant
                    .fields
                    .iter()
                    .enumerate()
                    .find_map(|(index, field)| {
                        let name = field
                            .ident
                            .as_ref()
                            .map_or_else(|| index.to_string(), ToString::to_string);
                        (name == field_name).then(|| field.span().start())
                    })
            }
        }
        _ => None,
    }?;
    Some(source_definition(source, location))
}

fn api_args(
    attribute: &Attribute,
    source: &Path,
) -> Result<sand_api_contract::syntax::ContractArgs, ContractSourceError> {
    let tokens = attribute
        .meta
        .require_list()
        .map_err(|error| ContractSourceError::Parse(format!("{}: {error}", source.display())))?
        .tokens
        .clone();
    parse_contract_args(tokens)
        .map_err(|error| ContractSourceError::Parse(format!("{}: {error}", source.display())))
}

#[derive(Default)]
struct RegisterArgs {
    path: Option<String>,
    aliases: BTreeSet<String>,
    kind: Option<ApiKind>,
    signature: Option<String>,
    summary: Option<String>,
    context: Option<String>,
    minecraft: Option<String>,
    use_when: Option<Vec<String>>,
    avoid_when: Option<Vec<String>>,
    parameters: Option<BTreeSet<String>>,
    returns: Option<bool>,
    example: Option<String>,
    availability: Option<Vec<String>>,
    canonical_module: Option<String>,
    parameter_docs: Option<Vec<(String, String)>>,
    return_doc: Option<Option<String>>,
    family: bool,
}

impl RegisterArgs {
    fn complete_family_semantics(&mut self, macro_name: &str) {
        if macro_name == "register" {
            return;
        }
        self.family = true;
        let (module, signature, context, minecraft, use_when, avoid_when, example, availability) =
            match macro_name {
                "register_event_marker" => {
                    self.kind.get_or_insert(ApiKind::Struct);
                    (
                        "sand::events",
                        "pub struct event marker",
                        "This stateless marker selects one built-in Sand event for a typed #[on_event] handler. Use Event<T> for advancement-backed markers and the marker itself for SandEvent-backed dispatches.",
                        self.minecraft.as_deref().unwrap_or_default(),
                        "Registering a handler for this specific Minecraft or Sand runtime occurrence",
                        "Representing mutable event data; read typed handler context or declared participants instead",
                        self.example.as_deref().unwrap_or_default(),
                        Vec::new(),
                    )
                }
                "register_event_api" => (
                    self.canonical_module.as_deref().unwrap_or("sand::events"),
                    self.signature.as_deref().unwrap_or_default(),
                    "This typed event API is part of Sand's author-facing event model; exporter records and generated function wiring remain private.",
                    self.minecraft.as_deref().unwrap_or_default(),
                    "Defining, composing, or handling a typed Sand event",
                    "Inspecting generated advancement or event-graph implementation state",
                    "use sand::prelude::*;",
                    Vec::new(),
                ),
                "register_entity_api" => (
                    "sand::entity",
                    "author-facing entity API",
                    "This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
                    "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
                    "Defining or using typed entity behavior in a Sand datapack",
                    "Inspecting generated objectives, functions, or compiler lowering plans",
                    "use sand::entity::*;",
                    Vec::new(),
                ),
                "register_state_api" => (
                    "sand::state",
                    "author-facing typed state API",
                    "This declaration provides the typed scoreboard or lifecycle primitives used directly and by #[derive(State)]; generated schema registration remains private.",
                    "Operations render validated scoreboard commands or conditions against the selected score holder, with lifecycle setup emitted at load when required.",
                    "Working with typed gameplay state or composing state transitions",
                    "Manually reproducing metadata generated by #[derive(State)]",
                    "use sand::state::*;",
                    Vec::new(),
                ),
                "register_participant_api" => (
                    "sand::participant",
                    "author-facing typed event participant API",
                    "Participants are available only when the event plan declares a real observation or a valid same-cycle inheritance path; the exporter rejects unsupported transport.",
                    "Entity relationships use the matching execute relation, while item snapshots are copied into Sand-owned command storage and cleaned up at the end of their declared lifetime.",
                    "Declaring or reading a typed participant whose lifecycle is guaranteed by the event plan",
                    "Assuming an entity or item remains live beyond its declared invocation, event-cycle, or bounded correlation lifetime",
                    "use sand::participant::*;",
                    Vec::new(),
                ),
                "register_text_api" => (
                    "sand::text",
                    "typed Minecraft text component API",
                    "Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
                    "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
                    "Building player-visible text with typed styling or interactions",
                    "Passing an unvalidated JSON string when a typed text component can express the same value",
                    "let text = sand::text::Text::new(\"Ready\").gold();",
                    Vec::new(),
                ),
                "register_data_api" => (
                    "sand::data",
                    "typed Minecraft NBT and command-storage API",
                    "This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
                    "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
                    "Reading or mutating structured Minecraft NBT through typed paths and values",
                    "A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT",
                    "use sand::data::{NbtPath, StorageLocation};",
                    Vec::new(),
                ),
                "register_systems_api" => {
                    let availability = self
                        .path
                        .as_deref()
                        .and_then(system_feature_for_path)
                        .map(|feature| vec![format!("Cargo feature: {feature}")])
                        .unwrap_or_default();
                    (
                        "sand::systems",
                        "feature-gated author-facing gameplay system API",
                        "This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
                        "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
                        "Opting into the documented higher-level gameplay behavior instead of assembling its commands manually",
                        "Using the API outside its documented system scope or feature configuration",
                        "use sand::systems;",
                        availability,
                    )
                }
                "register_command_api" => (
                    "sand::command",
                    "handwritten typed Minecraft command API",
                    "This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
                    "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
                    "Constructing Minecraft commands through Sand's typed command model",
                    "Passing unvalidated command fragments when a typed builder or validated try_* entry point exists",
                    "use sand::command as cmd;",
                    Vec::new(),
                ),
                "register_component_api" => (
                    "sand::component",
                    "typed datapack component definition API",
                    "This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
                    "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
                    "Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component",
                    "Injecting unchecked JSON when the typed schema can represent the resource",
                    "use sand::component::*;",
                    Vec::new(),
                ),
                "register_version_api" => (
                    "sand::version",
                    "typed Minecraft version capability API",
                    "This version API lets reusable authoring and tooling make the same capability decisions as Sand's profile-aware component exporter.",
                    "Capability checks describe the data-driven features accepted by the selected Minecraft Java Edition target before pack output is written.",
                    "Adapting authored resources or integrations to an explicitly selected Minecraft target",
                    "Ordinary datapack code can rely on the target selected in sand.toml",
                    "let caps = sand::version::VersionCaps::all_enabled();",
                    Vec::new(),
                ),
                "register_resourcepack_api" => (
                    "sand::resourcepack",
                    "feature-gated resource-pack authoring API",
                    "This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
                    "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
                    "Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack",
                    "The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand",
                    "use sand::resourcepack::*;",
                    vec!["Cargo feature: resourcepack".to_owned()],
                ),
                _ => return,
            };
        let module = module.to_owned();
        let signature = signature.to_owned();
        let context = context.to_owned();
        let minecraft = minecraft.to_owned();
        let use_when = use_when.to_owned();
        let avoid_when = avoid_when.to_owned();
        let example = example.to_owned();
        self.canonical_module.get_or_insert(module);
        self.signature.get_or_insert(signature);
        self.context.get_or_insert(context);
        self.minecraft.get_or_insert(minecraft);
        self.use_when.get_or_insert_with(|| vec![use_when]);
        self.avoid_when.get_or_insert_with(|| vec![avoid_when]);
        self.example.get_or_insert(example);
        self.availability.get_or_insert(availability);
        self.parameters.get_or_insert_with(BTreeSet::new);
        self.parameter_docs.get_or_insert_with(Vec::new);
        self.returns.get_or_insert(false);
        self.return_doc.get_or_insert(None);
    }

    fn validate(&self, macro_name: &str) -> Result<(), String> {
        validate_contract_semantics(&ContractSemantics {
            summary: self.summary.as_deref(),
            context: self.context.as_deref(),
            minecraft: self.minecraft.as_deref(),
            use_when: self.use_when.as_deref(),
            avoid_when: self.avoid_when.as_deref(),
            example: self.example.as_deref(),
        })?;
        if self.kind.is_none() {
            return Err(format!("{macro_name}! contract is missing `kind`"));
        }
        if macro_name == "register_systems_api"
            && self.availability.as_ref().is_none_or(Vec::is_empty)
        {
            return Err(format!(
                "register_systems_api! path `{}` does not map to a known Cargo feature",
                self.path.as_deref().unwrap_or("<missing>")
            ));
        }
        if macro_name == "register" {
            if self
                .signature
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err("register! contract field `signature` cannot be empty".into());
            }
            if self.parameters.is_none() {
                return Err("register! contract is missing `params`".into());
            }
            if self.returns.is_none() {
                return Err("register! contract is missing `returns`".into());
            }
        }
        Ok(())
    }

    fn facade_contract(&self) -> Result<FacadeContract, ContractSourceError> {
        Ok(FacadeContract {
            kind: self
                .kind
                .ok_or_else(|| ContractSourceError::Parse("missing kind".into()))?,
            signature: self.signature.clone(),
            parameters: self.parameters.clone(),
            returns: self.returns,
            runtime_signature: self
                .signature
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing runtime signature".into()))?,
            summary: self
                .summary
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing summary".into()))?,
            context: self
                .context
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing context".into()))?,
            minecraft: self
                .minecraft
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing minecraft".into()))?,
            use_when: self
                .use_when
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing use_when".into()))?,
            avoid_when: self
                .avoid_when
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing avoid_when".into()))?,
            parameter_docs: self
                .parameter_docs
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing parameter docs".into()))?,
            return_doc: self
                .return_doc
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing return metadata".into()))?,
            example: self
                .example
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing example".into()))?,
            availability: self.availability.clone().unwrap_or_default(),
            canonical_module: self
                .canonical_module
                .clone()
                .ok_or_else(|| ContractSourceError::Parse("missing canonical module".into()))?,
            family: self.family,
        })
    }
}

fn system_feature_for_path(path: &str) -> Option<&'static str> {
    let family = path.strip_prefix("sand::systems::")?.split("::").next()?;
    match family {
        "cooldowns" => Some("systems-cooldowns"),
        "damage" => Some("systems-damage"),
        "entities" => Some("systems-entities"),
        "inventory" => Some("systems-inventory"),
        "lifecycle" => Some("systems-lifecycle"),
        "movement" => Some("systems-movement"),
        "player_data" => Some("systems-player-data"),
        _ => None,
    }
}

impl Parse for RegisterArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut result = Self::default();
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "path" => {
                    let value = input.parse::<LitStr>()?.value();
                    if result.path.replace(value).is_some() {
                        return Err(syn::Error::new_spanned(key, "duplicate register! path"));
                    }
                }
                "aliases" => {
                    let content;
                    syn::bracketed!(content in input);
                    for value in Punctuated::<LitStr, Token![,]>::parse_terminated(&content)? {
                        result.aliases.insert(value.value());
                    }
                }
                "kind" => {
                    let value: syn::Ident = input.parse()?;
                    result.kind = Some(parse_api_kind(&value)?);
                }
                "signature" => result.signature = Some(input.parse::<LitStr>()?.value()),
                "summary" => result.summary = Some(input.parse::<LitStr>()?.value()),
                "context" => result.context = Some(input.parse::<LitStr>()?.value()),
                "minecraft" => result.minecraft = Some(input.parse::<LitStr>()?.value()),
                "example" => result.example = Some(input.parse::<LitStr>()?.value()),
                "module" => {
                    result.canonical_module = Some(input.parse::<LitStr>()?.value());
                }
                "use_when" => result.use_when = Some(parse_register_strings(input)?),
                "avoid_when" => result.avoid_when = Some(parse_register_strings(input)?),
                "availability" => {
                    result.availability = Some(parse_register_strings(input)?);
                }
                "params" => {
                    let content;
                    syn::bracketed!(content in input);
                    let mut names = BTreeSet::new();
                    let mut docs = Vec::new();
                    while !content.is_empty() {
                        let name = content.parse::<LitStr>()?.value();
                        content.parse::<Token![=>]>()?;
                        let description = content.parse::<LitStr>()?;
                        if description.value().trim().is_empty() {
                            return Err(syn::Error::new_spanned(
                                description,
                                "parameter description cannot be empty",
                            ));
                        }
                        if !names.insert(name.clone()) {
                            return Err(syn::Error::new_spanned(
                                key.clone(),
                                format!("duplicate parameter `{name}`"),
                            ));
                        }
                        docs.push((name, description.value()));
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    result.parameters = Some(names);
                    result.parameter_docs = Some(docs);
                }
                "returns" => {
                    let expression: syn::Expr = input.parse()?;
                    let return_doc = parse_return_doc(&expression)?;
                    result.returns = Some(return_doc.is_some());
                    result.return_doc = Some(return_doc);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        key,
                        format!("unknown facade contract field `{other}`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(result)
    }
}

fn parse_return_doc(expression: &syn::Expr) -> syn::Result<Option<String>> {
    if matches!(expression, syn::Expr::Path(path) if path.path.is_ident("None")) {
        return Ok(None);
    }
    let syn::Expr::Call(call) = expression else {
        return Err(syn::Error::new_spanned(
            expression,
            "returns must be None or Some(\"description\")",
        ));
    };
    let syn::Expr::Path(function) = call.func.as_ref() else {
        return Err(syn::Error::new_spanned(expression, "returns must use Some"));
    };
    if !function.path.is_ident("Some") || call.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            expression,
            "returns must be None or Some(\"description\")",
        ));
    }
    let syn::Expr::Lit(value) = &call.args[0] else {
        return Err(syn::Error::new_spanned(
            &call.args[0],
            "return description must be a string literal",
        ));
    };
    let syn::Lit::Str(value) = &value.lit else {
        return Err(syn::Error::new_spanned(
            &value.lit,
            "return description must be a string literal",
        ));
    };
    Ok(Some(value.value()))
}

fn parse_register_strings(input: ParseStream<'_>) -> syn::Result<Vec<String>> {
    let content;
    syn::bracketed!(content in input);
    Punctuated::<LitStr, Token![,]>::parse_terminated(&content)
        .map(|values| values.into_iter().map(|value| value.value()).collect())
}

fn parse_api_kind(value: &syn::Ident) -> syn::Result<ApiKind> {
    Ok(match value.to_string().as_str() {
        "Module" => ApiKind::Module,
        "Struct" => ApiKind::Struct,
        "Enum" => ApiKind::Enum,
        "Variant" => ApiKind::Variant,
        "Trait" => ApiKind::Trait,
        "Function" => ApiKind::Function,
        "Method" => ApiKind::Method,
        "TraitMethod" => ApiKind::TraitMethod,
        "TypeAlias" => ApiKind::TypeAlias,
        "Constant" => ApiKind::Constant,
        "AssociatedConst" => ApiKind::AssociatedConst,
        "AssociatedType" => ApiKind::AssociatedType,
        "Field" => ApiKind::Field,
        "Macro" => ApiKind::Macro,
        _ => {
            return Err(syn::Error::new_spanned(
                value,
                "unknown facade contract kind",
            ));
        }
    })
}

#[cfg(test)]
mod structural_shape_tests {
    use super::*;

    #[test]
    fn source_shapes_preserve_visibility_tuple_syntax_bounds_and_const_values() {
        let file = syn::parse_file(
            r#"
            pub const RELEASE: &str = "26.2";
            pub trait Render: Validate where Self: Sized {
                /// Names the registry represented by this renderer.
                const REGISTRY_KEY: &'static str;
            }
            pub struct Named { pub path: String }
            pub struct Tuple(pub f64);
            pub enum Coord { Absolute(f64) }
            pub fn make(value: i32) -> i32 { value }
            impl Named { pub fn path(&self) -> &str { &self.path } }
            "#,
        )
        .unwrap();
        let mut shapes = BTreeMap::new();
        collect_definition_shapes(&file.items, &mut shapes);
        let signatures = shapes
            .values()
            .map(|shape| shape.signature.as_str())
            .collect::<BTreeSet<_>>();

        assert!(signatures.contains("pub const RELEASE : & str = \"26.2\""));
        assert!(signatures.contains("pub trait Render : Validate where Self : Sized"));
        assert!(signatures.contains("pub path: String"));
        assert!(signatures.contains("pub f64"));
        assert!(signatures.contains("f64"));
        assert!(signatures.contains("pub fn make (value : i32) -> i32"));
        assert!(signatures.contains("pub fn path (& self) -> & str"));
        assert!(shapes.values().any(|shape| {
            shape.signature.contains("const REGISTRY_KEY")
                && shape
                    .documentation
                    .contains("Names the registry represented")
        }));
        assert!(
            !signatures
                .iter()
                .any(|signature| signature.starts_with("0:"))
        );
    }
}
