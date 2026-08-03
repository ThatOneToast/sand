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

use sand_api_contract::syntax::parse_contract_args;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Item, LitStr, Token};

use crate::{ContractIdentity, ReachableApi};

/// One contract identity as authored in Rust source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractDeclaration {
    pub canonical_path: String,
    pub aliases: BTreeSet<String>,
    pub source: PathBuf,
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

fn inspect_items(
    items: &[Item],
    source: &Path,
    declarations: &mut Vec<ContractDeclaration>,
) -> Result<(), ContractSourceError> {
    for item in items {
        if let Some(attribute) = crate::item_attrs(item)
            .iter()
            .find(|attribute| attribute.path().is_ident("api"))
        {
            let declaration = declaration_from_attribute(attribute, source)?;
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
                    inspect_attributes(attrs, source, declarations)?;
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
                    inspect_attributes(attrs, source, declarations)?;
                }
            }
            Item::Macro(item) if item.mac.path.is_ident("register") => {
                let parsed =
                    syn::parse2::<RegisterArgs>(item.mac.tokens.clone()).map_err(|error| {
                        ContractSourceError::Parse(format!("{}: {error}", source.display()))
                    })?;
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
    declarations: &mut Vec<ContractDeclaration>,
) -> Result<(), ContractSourceError> {
    if let Some(attribute) = attributes
        .iter()
        .find(|attribute| attribute.path().is_ident("api"))
    {
        declarations.push(declaration_from_attribute(attribute, source)?);
    }
    Ok(())
}

fn declaration_from_attribute(
    attribute: &Attribute,
    source: &Path,
) -> Result<ContractDeclaration, ContractSourceError> {
    let args = api_args(attribute, source)?;
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
    })
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
                _ => {
                    while !input.is_empty() && !input.peek(Token![,]) {
                        input.parse::<proc_macro2::TokenTree>()?;
                    }
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(result)
    }
}
