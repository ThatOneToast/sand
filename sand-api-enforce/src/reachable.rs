//! A stable-Rust proof for auditing the API *reachable through a facade*.
//!
//! This deliberately operates on source owned by the workspace. Rust source
//! alone cannot reveal arbitrary proc-macro expansion, so controlled
//! generators must provide [`GeneratedApi`] records from the same data that
//! emits their Rust items. This is the hybrid boundary: source declarations
//! and re-export reachability are discovered, while generated declarations
//! are supplied by their generator rather than guessed after expansion.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;

/// One local crate whose source can participate in facade re-exports.
#[derive(Clone, Debug)]
pub struct SourceCrate {
    pub name: String,
    pub root: PathBuf,
}

/// A declaration emitted by a controlled code generator.
#[derive(Clone, Debug)]
pub struct GeneratedApi {
    /// Internal identity, for example `sand_core::cmd::generated::Teleport`.
    pub identity: String,
    /// Stable generator-provider scope, for example `generated_commands`.
    pub provider: String,
    pub kind: ReachableKind,
    /// Associated items emitted with a generated type.
    pub members: Vec<(String, ReachableKind)>,
    /// Whether the generator intentionally emits compiler-only wiring.
    pub excluded: bool,
}

/// Where a reachable declaration came from. Scope enforcement partitions
/// source declarations and generated families even when they share a
/// canonical facade module.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReachableOrigin {
    Source,
    Generator(String),
}

/// Item categories needed to prove complete surface traversal.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReachableKind {
    Module,
    Struct,
    Enum,
    Variant,
    Field,
    Trait,
    Function,
    Method,
    TraitMethod,
    AssociatedConst,
    AssociatedType,
    TypeAlias,
    Constant,
    Macro,
}

/// One underlying item and every public path by which the facade exposes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReachableApi {
    pub identity: String,
    pub kind: ReachableKind,
    pub origin: ReachableOrigin,
    pub paths: BTreeSet<String>,
}

/// The central canonical choice for one reachable identity.
#[derive(Clone, Debug)]
pub struct ContractIdentity {
    pub identity: String,
    pub canonical_path: String,
    pub aliases: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReachabilityError {
    Io(String),
    Parse(String),
    UnknownFacade(String),
    MissingContract {
        identity: String,
        paths: Vec<String>,
    },
    ContractNotReachable(String),
    CanonicalPathNotReachable {
        identity: String,
        path: String,
    },
    AliasSetMismatch {
        identity: String,
        expected: Vec<String>,
        actual: Vec<String>,
    },
    DuplicateCanonicalPath(String),
}

/// Parsed local workspace graph and its selected Cargo features.
pub struct SurfaceGraph {
    crates: BTreeMap<String, CrateIndex>,
    generated: Vec<GeneratedApi>,
}

#[derive(Default)]
struct CrateIndex {
    modules: BTreeMap<String, Module>,
    declarations: BTreeMap<String, Declaration>,
}

#[derive(Default)]
struct Module {
    declarations: BTreeMap<String, String>,
    modules: BTreeMap<String, String>,
    uses: Vec<UseRecord>,
    excluded: bool,
}

struct Declaration {
    kind: ReachableKind,
    members: Vec<MemberRecord>,
    excluded: bool,
}

type MemberRecord = (String, ReachableKind, bool);
type DeclarationParts = (String, ReachableKind, Vec<MemberRecord>);

struct UseRecord {
    prefix: Vec<String>,
    leaf: UseLeaf,
}

enum UseLeaf {
    Name { source: String, exported: String },
    Glob,
}

impl SurfaceGraph {
    pub fn load(
        crates: impl IntoIterator<Item = SourceCrate>,
        features: impl IntoIterator<Item = String>,
        generated: impl IntoIterator<Item = GeneratedApi>,
    ) -> Result<Self, ReachabilityError> {
        let features = features.into_iter().collect::<BTreeSet<_>>();
        let mut indices = BTreeMap::new();
        for spec in crates {
            let mut index = CrateIndex::default();
            parse_module_file(
                &spec.name, &spec.root, &spec.name, false, &features, &mut index,
            )?;
            indices.insert(spec.name, index);
        }
        Ok(Self {
            crates: indices,
            generated: generated.into_iter().collect(),
        })
    }

    /// Extract all non-hidden items reachable through `facade`, including
    /// aliases introduced by explicit and glob re-exports.
    pub fn reachable_from(&self, facade: &str) -> Result<Vec<ReachableApi>, ReachabilityError> {
        if !self.crates.contains_key(facade) {
            return Err(ReachabilityError::UnknownFacade(facade.to_owned()));
        }
        let mut found = BTreeMap::<String, ReachableApi>::new();
        let mut visiting = BTreeSet::new();
        self.walk_module(facade, facade, facade, &mut visiting, &mut found);
        let mut values = found.into_values().collect::<Vec<_>>();
        values.sort_by(|left, right| left.identity.cmp(&right.identity));
        Ok(values)
    }

    fn walk_module(
        &self,
        owner_crate: &str,
        module_id: &str,
        public_path: &str,
        visiting: &mut BTreeSet<(String, String)>,
        found: &mut BTreeMap<String, ReachableApi>,
    ) {
        if !visiting.insert((module_id.to_owned(), public_path.to_owned())) {
            return;
        }
        let Some(module) = self.module(module_id) else {
            return;
        };
        if module.excluded {
            return;
        }

        for (name, identity) in &module.declarations {
            self.expose_declaration(identity, &format!("{public_path}::{name}"), found);
        }
        for (name, child) in &module.modules {
            let child_path = format!("{public_path}::{name}");
            self.expose_module(child, &child_path, found);
            self.walk_module(owner_crate, child, &child_path, visiting, found);
        }
        for use_record in &module.uses {
            let resolved = self.resolve_use_target(owner_crate, module_id, &use_record.prefix);
            match &use_record.leaf {
                UseLeaf::Name { source, exported } => {
                    let target = format!("{resolved}::{source}");
                    let alias_path = format!("{public_path}::{exported}");
                    if self.module(&target).is_some() {
                        self.expose_module(&target, &alias_path, found);
                        self.walk_module(owner_crate, &target, &alias_path, visiting, found);
                    } else {
                        self.expose_declaration(&target, &alias_path, found);
                    }
                }
                UseLeaf::Glob => {
                    self.walk_module(owner_crate, &resolved, public_path, visiting, found);
                }
            }
        }
        visiting.remove(&(module_id.to_owned(), public_path.to_owned()));
    }

    fn module(&self, identity: &str) -> Option<&Module> {
        let crate_name = identity.split("::").next()?;
        self.crates.get(crate_name)?.modules.get(identity)
    }

    fn declaration(&self, identity: &str) -> Option<(&Declaration, bool)> {
        let crate_name = identity.split("::").next()?;
        if let Some(value) = self.crates.get(crate_name)?.declarations.get(identity) {
            return Some((value, value.excluded));
        }
        None
    }

    fn expose_module(
        &self,
        identity: &str,
        path: &str,
        found: &mut BTreeMap<String, ReachableApi>,
    ) {
        if self.module(identity).is_some_and(|module| !module.excluded) {
            insert_path(
                found,
                identity,
                ReachableKind::Module,
                ReachableOrigin::Source,
                path,
            );
        }
    }

    fn expose_declaration(
        &self,
        identity: &str,
        path: &str,
        found: &mut BTreeMap<String, ReachableApi>,
    ) {
        let resolved = self
            .resolve_export(identity, &mut BTreeSet::new())
            .unwrap_or_else(|| identity.to_owned());
        if let Some((declaration, false)) = self.declaration(&resolved) {
            insert_path(
                found,
                &resolved,
                declaration.kind,
                ReachableOrigin::Source,
                path,
            );
            for (name, kind, excluded) in &declaration.members {
                if !excluded {
                    insert_path(
                        found,
                        &format!("{resolved}::{name}"),
                        *kind,
                        ReachableOrigin::Source,
                        &format!("{path}::{name}"),
                    );
                }
            }
            return;
        }
        for generated in &self.generated {
            if generated.identity == resolved && !generated.excluded {
                let origin = ReachableOrigin::Generator(generated.provider.clone());
                insert_path(found, &resolved, generated.kind, origin.clone(), path);
                for (name, kind) in &generated.members {
                    insert_path(
                        found,
                        &format!("{resolved}::{name}"),
                        *kind,
                        origin.clone(),
                        &format!("{path}::{name}"),
                    );
                }
            }
        }
    }

    fn resolve_export(&self, identity: &str, seen: &mut BTreeSet<String>) -> Option<String> {
        if self.declaration(identity).is_some()
            || self
                .generated
                .iter()
                .any(|generated| generated.identity == identity)
        {
            return Some(identity.to_owned());
        }
        if !seen.insert(identity.to_owned()) {
            return None;
        }
        let (module_id, name) = identity.rsplit_once("::")?;
        let module = self.module(module_id)?;
        let owner_crate = module_id.split("::").next()?;
        for use_record in &module.uses {
            let resolved = self.resolve_use_target(owner_crate, module_id, &use_record.prefix);
            let candidate = match &use_record.leaf {
                UseLeaf::Name { source, exported } if exported == name => {
                    Some(format!("{resolved}::{source}"))
                }
                UseLeaf::Glob => Some(format!("{resolved}::{name}")),
                _ => None,
            };
            if let Some(candidate) = candidate
                && let Some(target) = self.resolve_export(&candidate, seen)
            {
                return Some(target);
            }
        }
        None
    }

    fn resolve_use_target(&self, crate_name: &str, module_id: &str, prefix: &[String]) -> String {
        let resolved = resolve_qualified_use_target(crate_name, module_id, prefix);
        let first = resolved.split("::").next().unwrap_or_default();
        if self.crates.contains_key(first) {
            resolved
        } else {
            format!("{crate_name}::{resolved}")
        }
    }
}

/// Require exactly one central contract record per reachable identity and
/// require its aliases to equal the re-export graph, not a hand-picked subset.
pub fn audit_reachable_surface(
    reachable: &[ReachableApi],
    contracts: &[ContractIdentity],
) -> Result<(), Vec<ReachabilityError>> {
    let by_identity = contracts
        .iter()
        .map(|contract| (contract.identity.as_str(), contract))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    let mut canonical = BTreeSet::new();
    for contract in contracts {
        if !canonical.insert(contract.canonical_path.as_str()) {
            errors.push(ReachabilityError::DuplicateCanonicalPath(
                contract.canonical_path.clone(),
            ));
        }
    }
    let reachable_ids = reachable
        .iter()
        .map(|item| item.identity.as_str())
        .collect::<BTreeSet<_>>();
    for item in reachable {
        let Some(contract) = by_identity.get(item.identity.as_str()) else {
            errors.push(ReachabilityError::MissingContract {
                identity: item.identity.clone(),
                paths: item.paths.iter().cloned().collect(),
            });
            continue;
        };
        if !item.paths.contains(&contract.canonical_path) {
            errors.push(ReachabilityError::CanonicalPathNotReachable {
                identity: item.identity.clone(),
                path: contract.canonical_path.clone(),
            });
        }
        let expected = item
            .paths
            .iter()
            .filter(|path| *path != &contract.canonical_path)
            .cloned()
            .collect::<BTreeSet<_>>();
        if expected != contract.aliases {
            errors.push(ReachabilityError::AliasSetMismatch {
                identity: item.identity.clone(),
                expected: expected.into_iter().collect(),
                actual: contract.aliases.iter().cloned().collect(),
            });
        }
    }
    for contract in contracts {
        if !reachable_ids.contains(contract.identity.as_str()) {
            errors.push(ReachabilityError::ContractNotReachable(
                contract.identity.clone(),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn insert_path(
    found: &mut BTreeMap<String, ReachableApi>,
    identity: &str,
    kind: ReachableKind,
    origin: ReachableOrigin,
    path: &str,
) {
    found
        .entry(identity.to_owned())
        .or_insert_with(|| ReachableApi {
            identity: identity.to_owned(),
            kind,
            origin,
            paths: BTreeSet::new(),
        })
        .paths
        .insert(path.to_owned());
}

fn parse_module_file(
    crate_name: &str,
    file: &Path,
    module_id: &str,
    excluded_parent: bool,
    features: &BTreeSet<String>,
    index: &mut CrateIndex,
) -> Result<(), ReachabilityError> {
    let source = fs::read_to_string(file)
        .map_err(|error| ReachabilityError::Io(format!("{}: {error}", file.display())))?;
    let parsed = syn::parse_file(&source)
        .map_err(|error| ReachabilityError::Parse(format!("{}: {error}", file.display())))?;
    parse_items(
        crate_name,
        file,
        module_id,
        &parsed.items,
        excluded_parent,
        features,
        index,
    )
}

fn parse_items(
    crate_name: &str,
    source_file: &Path,
    module_id: &str,
    items: &[syn::Item],
    excluded_parent: bool,
    features: &BTreeSet<String>,
    index: &mut CrateIndex,
) -> Result<(), ReachabilityError> {
    index
        .modules
        .entry(module_id.to_owned())
        .or_default()
        .excluded |= excluded_parent;
    for item in items {
        let attrs = super::item_attrs(item);
        if !cfg_enabled(attrs, features) {
            continue;
        }
        let excluded =
            excluded_parent || super::doc_hidden(attrs) || module_id.ends_with("::__private");
        match item {
            syn::Item::Mod(value) => {
                let name = value.ident.to_string();
                let child_id = format!("{module_id}::{name}");
                if super::is_public(&value.vis) {
                    index
                        .modules
                        .entry(module_id.to_owned())
                        .or_default()
                        .modules
                        .insert(name.clone(), child_id.clone());
                }
                if let Some((_, nested)) = &value.content {
                    parse_items(
                        crate_name,
                        source_file,
                        &child_id,
                        nested,
                        excluded,
                        features,
                        index,
                    )?;
                } else {
                    let directory = source_file.parent().unwrap_or_else(|| Path::new("."));
                    let sibling = directory.join(format!("{name}.rs"));
                    let nested = directory.join(&name).join("mod.rs");
                    let path = if sibling.exists() { sibling } else { nested };
                    parse_module_file(crate_name, &path, &child_id, excluded, features, index)?;
                }
            }
            syn::Item::Use(value) if super::is_public(&value.vis) && !excluded => {
                flatten_use(
                    Vec::new(),
                    &value.tree,
                    &mut index.modules.entry(module_id.to_owned()).or_default().uses,
                );
            }
            syn::Item::Impl(value) if value.trait_.is_none() => {
                let owner = resolve_type_identity(crate_name, module_id, &value.self_ty);
                let declaration = index.declarations.entry(owner).or_insert(Declaration {
                    kind: ReachableKind::Struct,
                    members: Vec::new(),
                    excluded,
                });
                for child in &value.items {
                    let member = match child {
                        syn::ImplItem::Fn(method) if super::is_public(&method.vis) => Some((
                            method.sig.ident.to_string(),
                            ReachableKind::Method,
                            excluded || super::doc_hidden(&method.attrs),
                        )),
                        syn::ImplItem::Const(item) if super::is_public(&item.vis) => Some((
                            item.ident.to_string(),
                            ReachableKind::AssociatedConst,
                            excluded || super::doc_hidden(&item.attrs),
                        )),
                        syn::ImplItem::Type(item) if super::is_public(&item.vis) => Some((
                            item.ident.to_string(),
                            ReachableKind::AssociatedType,
                            excluded || super::doc_hidden(&item.attrs),
                        )),
                        _ => None,
                    };
                    if let Some(member) = member {
                        declaration.members.push(member);
                    }
                }
            }
            _ => {
                if let Some((name, kind, members)) = declaration_parts(item)
                    && public_item(item)
                {
                    let identity = format!("{module_id}::{name}");
                    index
                        .modules
                        .entry(module_id.to_owned())
                        .or_default()
                        .declarations
                        .insert(name, identity.clone());
                    let declaration = index.declarations.entry(identity).or_insert(Declaration {
                        kind,
                        members: Vec::new(),
                        excluded,
                    });
                    declaration.kind = kind;
                    declaration.excluded |= excluded;
                    declaration.members.extend(members);
                }
            }
        }
    }
    Ok(())
}

fn declaration_parts(item: &syn::Item) -> Option<DeclarationParts> {
    let plain = |name: String, kind| Some((name, kind, Vec::new()));
    match item {
        syn::Item::Struct(value) => {
            let fields = value
                .fields
                .iter()
                .filter(|field| super::is_public(&field.vis))
                .enumerate()
                .map(|(index, field)| {
                    let name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), ToString::to_string);
                    (name, ReachableKind::Field, super::doc_hidden(&field.attrs))
                })
                .collect();
            Some((value.ident.to_string(), ReachableKind::Struct, fields))
        }
        syn::Item::Enum(value) => Some((
            value.ident.to_string(),
            ReachableKind::Enum,
            value
                .variants
                .iter()
                .map(|variant| {
                    (
                        variant.ident.to_string(),
                        ReachableKind::Variant,
                        super::doc_hidden(&variant.attrs),
                    )
                })
                .collect(),
        )),
        syn::Item::Trait(value) => Some((
            value.ident.to_string(),
            ReachableKind::Trait,
            value
                .items
                .iter()
                .filter_map(|child| match child {
                    syn::TraitItem::Fn(method) => Some((
                        method.sig.ident.to_string(),
                        ReachableKind::TraitMethod,
                        super::doc_hidden(&method.attrs),
                    )),
                    syn::TraitItem::Const(item) => Some((
                        item.ident.to_string(),
                        ReachableKind::AssociatedConst,
                        super::doc_hidden(&item.attrs),
                    )),
                    syn::TraitItem::Type(item) => Some((
                        item.ident.to_string(),
                        ReachableKind::AssociatedType,
                        super::doc_hidden(&item.attrs),
                    )),
                    _ => None,
                })
                .collect(),
        )),
        syn::Item::Fn(value) => plain(value.sig.ident.to_string(), ReachableKind::Function),
        syn::Item::Type(value) => plain(value.ident.to_string(), ReachableKind::TypeAlias),
        syn::Item::Const(value) => plain(value.ident.to_string(), ReachableKind::Constant),
        syn::Item::Macro(value)
            if value
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("macro_export")) =>
        {
            plain(value.ident.as_ref()?.to_string(), ReachableKind::Macro)
        }
        _ => None,
    }
}

fn public_item(item: &syn::Item) -> bool {
    match item {
        syn::Item::Struct(v) => super::is_public(&v.vis),
        syn::Item::Enum(v) => super::is_public(&v.vis),
        syn::Item::Trait(v) => super::is_public(&v.vis),
        syn::Item::Fn(v) => super::is_public(&v.vis),
        syn::Item::Type(v) => super::is_public(&v.vis),
        syn::Item::Const(v) => super::is_public(&v.vis),
        syn::Item::Macro(v) => v
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("macro_export")),
        _ => false,
    }
}

fn flatten_use(prefix: Vec<String>, tree: &syn::UseTree, output: &mut Vec<UseRecord>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut next = prefix;
            next.push(path.ident.to_string());
            flatten_use(next, &path.tree, output);
        }
        syn::UseTree::Name(name) => output.push(UseRecord {
            prefix,
            leaf: UseLeaf::Name {
                source: name.ident.to_string(),
                exported: name.ident.to_string(),
            },
        }),
        syn::UseTree::Rename(rename) => output.push(UseRecord {
            prefix,
            leaf: UseLeaf::Name {
                source: rename.ident.to_string(),
                exported: rename.rename.to_string(),
            },
        }),
        syn::UseTree::Glob(_) => output.push(UseRecord {
            prefix,
            leaf: UseLeaf::Glob,
        }),
        syn::UseTree::Group(group) => {
            for child in &group.items {
                flatten_use(prefix.clone(), child, output);
            }
        }
    }
}

fn resolve_qualified_use_target(crate_name: &str, module_id: &str, prefix: &[String]) -> String {
    let mut segments = prefix.to_vec();
    if segments.first().is_some_and(|s| s == "crate") {
        segments.remove(0);
        return join_path(crate_name, &segments);
    }
    if segments.first().is_some_and(|s| s == "self") {
        segments.remove(0);
        return join_path(module_id, &segments);
    }
    if segments.first().is_some_and(|s| s == "super") {
        let mut base = module_id
            .rsplit_once("::")
            .map_or(crate_name, |(base, _)| base)
            .to_owned();
        while segments.first().is_some_and(|s| s == "super") {
            segments.remove(0);
            base = base
                .rsplit_once("::")
                .map_or(crate_name, |(parent, _)| parent)
                .to_owned();
        }
        return join_path(&base, &segments);
    }
    if segments.first().is_some_and(|first| first == crate_name) {
        return segments.join("::");
    }
    segments.join("::")
}

fn join_path(base: &str, rest: &[String]) -> String {
    if rest.is_empty() {
        base.to_owned()
    } else {
        format!("{base}::{}", rest.join("::"))
    }
}

fn resolve_type_identity(crate_name: &str, module_id: &str, ty: &syn::Type) -> String {
    let raw = ty.to_token_stream().to_string().replace(' ', "");
    let base = raw.split('<').next().unwrap_or(&raw);
    if let Some(relative) = base.strip_prefix("crate::") {
        format!("{crate_name}::{relative}")
    } else if base.contains("::") {
        base.to_owned()
    } else {
        format!("{module_id}::{base}")
    }
}

fn cfg_enabled(attrs: &[syn::Attribute], features: &BTreeSet<String>) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("cfg"))
        .all(|attr| {
            attr.parse_args::<syn::Meta>()
                .map_or(true, |meta| eval_cfg(&meta, features))
        })
}

fn eval_cfg(meta: &syn::Meta, features: &BTreeSet<String>) -> bool {
    match meta {
        syn::Meta::NameValue(value) if value.path.is_ident("feature") => match &value.value {
            syn::Expr::Lit(expr) => match &expr.lit {
                syn::Lit::Str(value) => features.contains(&value.value()),
                _ => true,
            },
            _ => true,
        },
        syn::Meta::List(list) if list.path.is_ident("all") => list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .map_or(true, |items| {
                items.iter().all(|item| eval_cfg(item, features))
            }),
        syn::Meta::List(list) if list.path.is_ident("any") => list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .map_or(true, |items| {
                items.iter().any(|item| eval_cfg(item, features))
            }),
        syn::Meta::List(list) if list.path.is_ident("not") => list
            .parse_args::<syn::Meta>()
            .map_or(true, |item| !eval_cfg(&item, features)),
        _ => true,
    }
}
