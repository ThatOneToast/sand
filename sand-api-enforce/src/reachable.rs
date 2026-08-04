//! A stable-Rust proof for auditing the API *reachable through a facade*.
//!
//! This deliberately operates on source owned by the workspace. Rust source
//! alone cannot reveal arbitrary proc-macro expansion, so controlled
//! generators must provide [`GeneratedApi`] records from the same data that
//! emits their Rust items. This is the hybrid boundary: source declarations
//! and re-export reachability are discovered, while generated declarations
//! are supplied by their generator rather than guessed after expansion.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::spanned::Spanned;

/// Explicit cfg environment used while parsing the selected Cargo target.
#[derive(Clone, Debug, Default)]
pub struct CfgSet {
    pub features: BTreeSet<String>,
    pub flags: BTreeMap<String, bool>,
    pub key_values: BTreeMap<String, BTreeSet<String>>,
}

/// One local crate whose source can participate in facade re-exports.
#[derive(Clone, Debug)]
pub struct SourceCrate {
    pub name: String,
    pub root: PathBuf,
}

/// Discover the facade crate and the transitive closure of its local path
/// dependencies directly from Cargo manifests.
///
/// Keeping this list in source would let a newly added workspace crate be
/// publicly re-exported without ever entering the reachability graph. Only
/// normal dependencies participate: build dependencies cannot be re-exported
/// by the library, and dev dependencies are not part of a normal build.
pub fn discover_local_source_crates(
    facade_manifest: &Path,
) -> Result<Vec<SourceCrate>, ReachabilityError> {
    let facade_manifest = facade_manifest.canonicalize().map_err(|error| {
        ReachabilityError::Io(format!("{}: {error}", facade_manifest.display()))
    })?;
    let mut pending = vec![(None, facade_manifest)];
    let mut manifests = BTreeMap::<String, PathBuf>::new();
    let mut crates = BTreeMap::<String, SourceCrate>::new();

    while let Some((dependency_name, manifest_path)) = pending.pop() {
        let text = fs::read_to_string(&manifest_path).map_err(|error| {
            ReachabilityError::Io(format!("{}: {error}", manifest_path.display()))
        })?;
        let manifest = text.parse::<toml::Value>().map_err(|error| {
            ReachabilityError::Parse(format!("{}: {error}", manifest_path.display()))
        })?;
        let crate_name = dependency_name.unwrap_or_else(|| manifest_crate_name(&manifest));
        let manifest_directory = manifest_path.parent().expect("manifest has a parent");
        let root = manifest
            .get("lib")
            .and_then(|lib| lib.get("path"))
            .and_then(toml::Value::as_str)
            .map_or_else(
                || manifest_directory.join("src/lib.rs"),
                |path| manifest_directory.join(path),
            );
        let root = root
            .canonicalize()
            .map_err(|error| ReachabilityError::Io(format!("{}: {error}", root.display())))?;

        if let Some(previous) = manifests.insert(crate_name.clone(), manifest_path.clone()) {
            if previous != manifest_path {
                return Err(ReachabilityError::Parse(format!(
                    "local dependency crate name `{crate_name}` maps to both {} and {}; rename one dependency explicitly",
                    previous.display(),
                    manifest_path.display()
                )));
            }
            continue;
        }
        crates.insert(
            crate_name.clone(),
            SourceCrate {
                name: crate_name,
                root,
            },
        );

        for (name, path) in local_dependency_paths(&manifest, manifest_directory)? {
            let dependency_manifest = path.join("Cargo.toml").canonicalize().map_err(|error| {
                ReachabilityError::Io(format!("{}: {error}", path.join("Cargo.toml").display()))
            })?;
            pending.push((Some(name), dependency_manifest));
        }
    }

    Ok(crates.into_values().collect())
}

fn manifest_crate_name(manifest: &toml::Value) -> String {
    manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(toml::Value::as_str)
        .or_else(|| {
            manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str)
        })
        .expect("Cargo manifest must have a package or library name")
        .replace('-', "_")
}

fn local_dependency_paths(
    manifest: &toml::Value,
    manifest_directory: &Path,
) -> Result<Vec<(String, PathBuf)>, ReachabilityError> {
    let mut dependencies = BTreeMap::new();
    collect_local_dependencies(
        manifest.get("dependencies"),
        manifest_directory,
        &mut dependencies,
    )?;
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            collect_local_dependencies(
                target.get("dependencies"),
                manifest_directory,
                &mut dependencies,
            )?;
        }
    }
    Ok(dependencies.into_iter().collect())
}

fn collect_local_dependencies(
    table: Option<&toml::Value>,
    manifest_directory: &Path,
    dependencies: &mut BTreeMap<String, PathBuf>,
) -> Result<(), ReachabilityError> {
    let Some(table) = table.and_then(toml::Value::as_table) else {
        return Ok(());
    };
    for (dependency_key, specification) in table {
        let Some(specification) = specification.as_table() else {
            continue;
        };
        if specification
            .get("workspace")
            .and_then(toml::Value::as_bool)
            == Some(true)
        {
            return Err(ReachabilityError::Parse(format!(
                "local dependency `{dependency_key}` uses `workspace = true`; give the facade dependency an explicit path so source auditing cannot be ambiguous"
            )));
        }
        let Some(path) = specification.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        let crate_name = dependency_key.replace('-', "_");
        let path = manifest_directory.join(path);
        if let Some(previous) = dependencies.insert(crate_name.clone(), path.clone())
            && previous != path
        {
            return Err(ReachabilityError::Parse(format!(
                "local dependency crate name `{crate_name}` has conflicting paths {} and {}",
                previous.display(),
                path.display()
            )));
        }
    }
    Ok(())
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
    Union,
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
    Static,
    Macro,
    FunctionLikeMacro,
    AttributeMacro,
    DeriveMacro,
}

/// One underlying item and every public path by which the facade exposes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReachableApi {
    pub identity: String,
    pub kind: ReachableKind,
    pub origin: ReachableOrigin,
    pub paths: BTreeSet<String>,
    /// Exact source declaration that produced this identity. Generated APIs
    /// deliberately have no source declaration and are audited by providers.
    pub definition: Option<SourceDefinition>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceDefinition {
    pub source: PathBuf,
    pub line: usize,
    pub column: usize,
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
    UnknownCfg {
        source: PathBuf,
        line: usize,
        predicate: String,
    },
    UnresolvedReexport {
        source: PathBuf,
        line: usize,
        facade_path: String,
        target: String,
    },
    UnresolvedImplOwner {
        module: String,
        self_type: String,
    },
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
    DuplicateContractIdentity(String),
    ConflictingReachableDefinition {
        identity: String,
        first_kind: ReachableKind,
        first_origin: ReachableOrigin,
        second_kind: ReachableKind,
        second_origin: ReachableOrigin,
    },
}

impl fmt::Display for ReachabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) | Self::Parse(message) => formatter.write_str(message),
            Self::UnknownFacade(facade) => write!(formatter, "unknown facade crate `{facade}`"),
            Self::UnknownCfg {
                source,
                line,
                predicate,
            } => write!(
                formatter,
                "{}:{line}: cfg predicate `{predicate}` was not supplied by the target configuration",
                source.display()
            ),
            Self::UnresolvedReexport {
                source,
                line,
                facade_path,
                target,
            } => write!(
                formatter,
                "{}:{line}: public facade edge `{facade_path}` cannot resolve mapped workspace target `{target}`",
                source.display()
            ),
            Self::UnresolvedImplOwner { module, self_type } => write!(
                formatter,
                "inherent impl in `{module}` has public associated API but its owner `{self_type}` cannot be resolved to a local declaration"
            ),
            Self::MissingContract { identity, paths } => write!(
                formatter,
                "reachable API `{identity}` ({}) is missing a contract",
                paths.join(", ")
            ),
            Self::ContractNotReachable(identity) => {
                write!(formatter, "contract identity `{identity}` is not reachable")
            }
            Self::CanonicalPathNotReachable { identity, path } => write!(
                formatter,
                "contract `{identity}` selects unreachable canonical path `{path}`"
            ),
            Self::AliasSetMismatch {
                identity,
                expected,
                actual,
            } => write!(
                formatter,
                "contract `{identity}` aliases differ: expected [{}], actual [{}]",
                expected.join(", "),
                actual.join(", ")
            ),
            Self::DuplicateCanonicalPath(path) => {
                write!(formatter, "duplicate canonical API path `{path}`")
            }
            Self::DuplicateContractIdentity(identity) => {
                write!(formatter, "duplicate contract identity `{identity}`")
            }
            Self::ConflictingReachableDefinition {
                identity,
                first_kind,
                first_origin,
                second_kind,
                second_origin,
            } => write!(
                formatter,
                "reachable identity `{identity}` has conflicting definitions: {first_kind:?} from {first_origin:?} and {second_kind:?} from {second_origin:?}"
            ),
        }
    }
}

impl std::error::Error for ReachabilityError {}

/// Parsed local workspace graph and its selected Cargo features.
pub struct SurfaceGraph {
    crates: BTreeMap<String, CrateIndex>,
    generated: Vec<GeneratedApi>,
}

#[derive(Default)]
struct CrateIndex {
    modules: BTreeMap<String, Module>,
    declarations: BTreeMap<String, Declaration>,
    extern_aliases: BTreeMap<String, String>,
    type_aliases: BTreeMap<String, String>,
    pending_impls: Vec<PendingImpl>,
}

struct PendingImpl {
    module_id: String,
    self_ty: syn::Type,
    members: Vec<MemberRecord>,
    member_definitions: BTreeMap<String, SourceDefinition>,
    excluded: bool,
}

#[derive(Default)]
struct Module {
    declarations: BTreeMap<String, String>,
    modules: BTreeMap<String, String>,
    uses: Vec<UseRecord>,
    excluded: bool,
    definition: Option<SourceDefinition>,
}

struct Declaration {
    kind: ReachableKind,
    members: Vec<MemberRecord>,
    excluded: bool,
    alias_target: Option<String>,
    definition: SourceDefinition,
    member_definitions: BTreeMap<String, SourceDefinition>,
}

type MemberRecord = (String, ReachableKind, bool);
type DeclarationParts = (String, ReachableKind, Vec<MemberRecord>);

struct UseRecord {
    prefix: Vec<String>,
    leaf: UseLeaf,
    source: PathBuf,
    line: usize,
    public: bool,
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
        let cfg = CfgSet {
            features: features.into_iter().collect(),
            ..CfgSet::default()
        };
        Self::load_with_cfg(crates, cfg, generated)
    }

    pub fn load_with_cfg(
        crates: impl IntoIterator<Item = SourceCrate>,
        cfg: CfgSet,
        generated: impl IntoIterator<Item = GeneratedApi>,
    ) -> Result<Self, ReachabilityError> {
        let mut indices = BTreeMap::new();
        let mut unresolved_impls = Vec::new();
        for spec in crates {
            let mut index = CrateIndex::default();
            parse_module_file(&spec.name, &spec.root, &spec.name, false, &cfg, &mut index)?;
            unresolved_impls.extend(resolve_pending_impls(&spec.name, &mut index));
            indices.insert(spec.name, index);
        }
        let mut generated = generated.into_iter().collect::<Vec<_>>();
        for (owner, pending_impl) in unresolved_impls {
            if let Some(provider) = generated
                .iter_mut()
                .find(|generated| generated.identity == owner && !generated.excluded)
            {
                provider.members.extend(
                    pending_impl
                        .members
                        .into_iter()
                        .filter(|(_, _, excluded)| !excluded)
                        .map(|(name, kind, _)| (name, kind)),
                );
            } else if !pending_impl.excluded && !pending_impl.members.is_empty() {
                return Err(ReachabilityError::UnresolvedImplOwner {
                    module: pending_impl.module_id,
                    self_type: pending_impl.self_ty.to_token_stream().to_string(),
                });
            }
        }
        let mut providers = BTreeMap::<&str, (&str, ReachableKind)>::new();
        for item in &generated {
            if let Some((provider, kind)) =
                providers.insert(item.identity.as_str(), (item.provider.as_str(), item.kind))
            {
                return Err(ReachabilityError::ConflictingReachableDefinition {
                    identity: item.identity.clone(),
                    first_kind: kind,
                    first_origin: ReachableOrigin::Generator(provider.to_owned()),
                    second_kind: item.kind,
                    second_origin: ReachableOrigin::Generator(item.provider.clone()),
                });
            }
        }
        Ok(Self {
            crates: indices,
            generated,
        })
    }

    /// Extract all supported items reachable through `facade`, including
    /// aliases introduced by explicit and glob re-exports.
    ///
    /// `#[doc(hidden)]` is documentation presentation, not a support-boundary
    /// escape hatch. Only an explicitly named `__private` module is excluded;
    /// public hidden items elsewhere remain visible to scope enforcement.
    pub fn reachable_from(&self, facade: &str) -> Result<Vec<ReachableApi>, ReachabilityError> {
        if !self.crates.contains_key(facade) {
            return Err(ReachabilityError::UnknownFacade(facade.to_owned()));
        }
        let mut found = BTreeMap::<String, ReachableApi>::new();
        let mut visiting = BTreeSet::new();
        self.walk_module(facade, facade, &mut visiting, &mut found)?;
        let mut values = found.into_values().collect::<Vec<_>>();
        values.sort_by(|left, right| left.identity.cmp(&right.identity));
        Ok(values)
    }

    fn walk_module(
        &self,
        module_id: &str,
        public_path: &str,
        visiting: &mut BTreeSet<(String, String)>,
        found: &mut BTreeMap<String, ReachableApi>,
    ) -> Result<(), ReachabilityError> {
        if !visiting.insert((module_id.to_owned(), public_path.to_owned())) {
            return Ok(());
        }
        let Some(module) = self.module(module_id) else {
            return Ok(());
        };
        if module.excluded {
            return Ok(());
        }
        let defining_crate = module_id
            .split("::")
            .next()
            .expect("module identities always start with a crate name");

        for (name, identity) in &module.declarations {
            self.expose_declaration(identity, &format!("{public_path}::{name}"), found)?;
        }
        for generated in self.generated.iter().filter(|generated| {
            !generated.excluded && generated_parent(&generated.identity) == Some(module_id)
        }) {
            let name = generated
                .identity
                .rsplit("::")
                .next()
                .expect("generated identities contain an item name");
            self.expose_declaration(
                &generated.identity,
                &format!("{public_path}::{name}"),
                found,
            )?;
        }
        for (name, child) in &module.modules {
            let child_path = format!("{public_path}::{name}");
            self.expose_module(child, &child_path, found)?;
            self.walk_module(child, &child_path, visiting, found)?;
        }
        for use_record in module.uses.iter().filter(|record| record.public) {
            match &use_record.leaf {
                UseLeaf::Name { source, exported } => {
                    let target = self.resolve_named_use_target(
                        defining_crate,
                        module_id,
                        &use_record.prefix,
                        source,
                    );
                    let alias_path = format!("{public_path}::{exported}");
                    let resolved_target = self
                        .resolve_export(&target, &mut BTreeSet::new())
                        .unwrap_or_else(|| target.clone());
                    if self.module(&resolved_target).is_some() {
                        self.expose_module(&resolved_target, &alias_path, found)?;
                        self.walk_module(&resolved_target, &alias_path, visiting, found)?;
                    } else {
                        let exposed =
                            self.expose_declaration(&resolved_target, &alias_path, found)?;
                        if !exposed
                            && !self.known_excluded(&resolved_target)
                            && self.is_mapped_target(&target)
                        {
                            return Err(ReachabilityError::UnresolvedReexport {
                                source: use_record.source.clone(),
                                line: use_record.line,
                                facade_path: alias_path,
                                target,
                            });
                        }
                    }
                }
                UseLeaf::Glob => {
                    let resolved =
                        self.resolve_use_target(defining_crate, module_id, &use_record.prefix);
                    if let Some((declaration, false)) = self.declaration(&resolved) {
                        if declaration.kind != ReachableKind::Enum {
                            return Err(ReachabilityError::UnresolvedReexport {
                                source: use_record.source.clone(),
                                line: use_record.line,
                                facade_path: public_path.to_owned(),
                                target: format!("{resolved}::*"),
                            });
                        }
                        for (name, kind, excluded) in &declaration.members {
                            if *kind == ReachableKind::Variant && !excluded {
                                insert_path(
                                    found,
                                    &format!("{resolved}::{name}"),
                                    *kind,
                                    ReachableOrigin::Source,
                                    &format!("{public_path}::{name}"),
                                    declaration.member_definitions.get(name).cloned(),
                                )?;
                            }
                        }
                        continue;
                    }
                    if self.module(&resolved).is_none() && self.is_mapped_target(&resolved) {
                        return Err(ReachabilityError::UnresolvedReexport {
                            source: use_record.source.clone(),
                            line: use_record.line,
                            facade_path: public_path.to_owned(),
                            target: format!("{resolved}::*"),
                        });
                    }
                    self.walk_module(&resolved, public_path, visiting, found)?;
                }
            }
        }
        visiting.remove(&(module_id.to_owned(), public_path.to_owned()));
        Ok(())
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

    fn known_excluded(&self, identity: &str) -> bool {
        self.declaration(identity)
            .is_some_and(|(_, excluded)| excluded)
            || self
                .generated
                .iter()
                .any(|generated| generated.identity == identity && generated.excluded)
    }

    fn expose_module(
        &self,
        identity: &str,
        path: &str,
        found: &mut BTreeMap<String, ReachableApi>,
    ) -> Result<(), ReachabilityError> {
        if self.module(identity).is_some_and(|module| !module.excluded) {
            insert_path(
                found,
                identity,
                ReachableKind::Module,
                ReachableOrigin::Source,
                path,
                self.module(identity)
                    .and_then(|module| module.definition.clone()),
            )?;
        }
        Ok(())
    }

    fn expose_declaration(
        &self,
        identity: &str,
        path: &str,
        found: &mut BTreeMap<String, ReachableApi>,
    ) -> Result<bool, ReachabilityError> {
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
                Some(declaration.definition.clone()),
            )?;
            for (name, kind, excluded) in &declaration.members {
                if !excluded {
                    insert_path(
                        found,
                        &format!("{resolved}::{name}"),
                        *kind,
                        ReachableOrigin::Source,
                        &format!("{path}::{name}"),
                        declaration.member_definitions.get(name).cloned(),
                    )?;
                }
            }
            for generated in self.generated.iter().filter(|generated| {
                !generated.excluded && generated_parent(&generated.identity) == Some(&resolved)
            }) {
                let name = generated
                    .identity
                    .rsplit("::")
                    .next()
                    .expect("generated member identities contain a member name");
                let origin = ReachableOrigin::Generator(generated.provider.clone());
                insert_path(
                    found,
                    &generated.identity,
                    generated.kind,
                    origin.clone(),
                    &format!("{path}::{name}"),
                    None,
                )?;
                for (member, kind) in &generated.members {
                    insert_path(
                        found,
                        &format!("{}::{member}", generated.identity),
                        *kind,
                        origin.clone(),
                        &format!("{path}::{name}::{member}"),
                        None,
                    )?;
                }
            }
            if declaration.kind == ReachableKind::TypeAlias
                && let Some(target) = &declaration.alias_target
                && let Some(target_identity) = self.resolve_export(target, &mut BTreeSet::new())
                && let Some((target, false)) = self.declaration(&target_identity)
            {
                for (name, kind, excluded) in &target.members {
                    if !excluded {
                        insert_path(
                            found,
                            &format!("{target_identity}::{name}"),
                            *kind,
                            ReachableOrigin::Source,
                            &format!("{path}::{name}"),
                            target.member_definitions.get(name).cloned(),
                        )?;
                    }
                }
            }
            for generated in self
                .generated
                .iter()
                .filter(|generated| generated.identity == resolved && !generated.excluded)
            {
                insert_path(
                    found,
                    &resolved,
                    generated.kind,
                    ReachableOrigin::Generator(generated.provider.clone()),
                    path,
                    None,
                )?;
            }
            return Ok(true);
        }
        let mut exposed = false;
        for generated in &self.generated {
            if generated.identity == resolved && !generated.excluded {
                let origin = ReachableOrigin::Generator(generated.provider.clone());
                exposed = true;
                insert_path(found, &resolved, generated.kind, origin.clone(), path, None)?;
                for (name, kind) in &generated.members {
                    insert_path(
                        found,
                        &format!("{resolved}::{name}"),
                        *kind,
                        origin.clone(),
                        &format!("{path}::{name}"),
                        None,
                    )?;
                }
            }
        }
        Ok(exposed)
    }

    fn resolve_export(&self, identity: &str, seen: &mut BTreeSet<String>) -> Option<String> {
        if self.module(identity).is_some()
            || self.declaration(identity).is_some()
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
            let candidate = match &use_record.leaf {
                UseLeaf::Name { source, exported } if exported == name => {
                    Some(self.resolve_named_use_target(
                        owner_crate,
                        module_id,
                        &use_record.prefix,
                        source,
                    ))
                }
                UseLeaf::Glob => {
                    let resolved =
                        self.resolve_use_target(owner_crate, module_id, &use_record.prefix);
                    Some(join_path(&resolved, &[name.to_owned()]))
                }
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
        let mut prefix = prefix.to_vec();
        if let Some(first) = prefix.first_mut()
            && let Some(alias) = self
                .crates
                .get(crate_name)
                .and_then(|index| index.extern_aliases.get(first))
        {
            *first = alias.clone();
        }
        let resolved = resolve_qualified_use_target(crate_name, module_id, &prefix);
        let first = resolved.split("::").next().unwrap_or_default();
        if self.crates.contains_key(first) {
            resolved
        } else if self.module(&format!("{module_id}::{resolved}")).is_some() {
            format!("{module_id}::{resolved}")
        } else {
            format!("{crate_name}::{resolved}")
        }
    }

    fn resolve_named_use_target(
        &self,
        crate_name: &str,
        module_id: &str,
        prefix: &[String],
        source: &str,
    ) -> String {
        if prefix.is_empty() {
            if self.crates.contains_key(source) {
                return source.to_owned();
            }
            let local = format!("{module_id}::{source}");
            if self.module(&local).is_some() || self.declaration(&local).is_some() {
                return local;
            }
            return format!("{crate_name}::{source}");
        }
        join_path(
            &self.resolve_use_target(crate_name, module_id, prefix),
            &[source.to_owned()],
        )
    }

    fn is_mapped_target(&self, target: &str) -> bool {
        target
            .split("::")
            .next()
            .is_some_and(|name| self.crates.contains_key(name))
    }
}

fn generated_parent(identity: &str) -> Option<&str> {
    identity.rsplit_once("::").map(|(parent, _)| parent)
}

/// Require exactly one central contract record per reachable identity and
/// require its aliases to equal the re-export graph, not a hand-picked subset.
pub fn audit_reachable_surface(
    reachable: &[ReachableApi],
    contracts: &[ContractIdentity],
) -> Result<(), Vec<ReachabilityError>> {
    let mut errors = Vec::new();
    let mut by_identity = BTreeMap::new();
    for contract in contracts {
        if by_identity
            .insert(contract.identity.as_str(), contract)
            .is_some()
        {
            errors.push(ReachabilityError::DuplicateContractIdentity(
                contract.identity.clone(),
            ));
        }
    }
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
    definition: Option<SourceDefinition>,
) -> Result<(), ReachabilityError> {
    let tagged_identity = disambiguated_identity(identity, kind);
    if let Some(entry) = found.get_mut(&tagged_identity) {
        if entry.kind != kind || entry.origin != origin {
            return Err(conflicting_definition(identity, entry, kind, origin));
        }
        if entry.definition != definition {
            return Err(conflicting_definition(identity, entry, kind, origin));
        }
        entry.paths.insert(path.to_owned());
        return Ok(());
    }

    if let Some(existing) = found.get(identity)
        && existing.kind != kind
        && existing.origin == origin
        && member_namespaces_may_overlap(existing.kind, kind)
    {
        let mut existing = found
            .remove(identity)
            .expect("the conflicting base identity was just observed");
        let existing_identity = disambiguated_identity(identity, existing.kind);
        existing.identity.clone_from(&existing_identity);
        found.insert(existing_identity, existing);
        found.insert(
            tagged_identity.clone(),
            ReachableApi {
                identity: tagged_identity,
                kind,
                origin,
                paths: BTreeSet::from([path.to_owned()]),
                definition,
            },
        );
        return Ok(());
    }

    // Once a collision has split an identity, later aliases must join the
    // kind-specific record rather than recreating an ambiguous base record.
    let prior_collision = found.values().find(|candidate| {
        candidate
            .identity
            .strip_prefix(identity)
            .is_some_and(|suffix| suffix.starts_with('#'))
    });
    if !found.contains_key(identity)
        && let Some(existing) = prior_collision
    {
        if existing.origin != origin || !member_namespaces_may_overlap(existing.kind, kind) {
            return Err(conflicting_definition(identity, existing, kind, origin));
        }
        found.insert(
            tagged_identity.clone(),
            ReachableApi {
                identity: tagged_identity,
                kind,
                origin,
                paths: BTreeSet::from([path.to_owned()]),
                definition,
            },
        );
        return Ok(());
    }

    let entry = found
        .entry(identity.to_owned())
        .or_insert_with(|| ReachableApi {
            identity: identity.to_owned(),
            kind,
            origin: origin.clone(),
            paths: BTreeSet::new(),
            definition,
        });
    if entry.kind != kind || entry.origin != origin {
        return Err(conflicting_definition(identity, entry, kind, origin));
    }
    entry.paths.insert(path.to_owned());
    Ok(())
}

fn disambiguated_identity(identity: &str, kind: ReachableKind) -> String {
    format!("{identity}#{}", kind_identity_tag(kind))
}

fn kind_identity_tag(kind: ReachableKind) -> &'static str {
    match kind {
        ReachableKind::Field => "field",
        ReachableKind::Method => "method",
        ReachableKind::AssociatedConst => "associated-const",
        ReachableKind::AssociatedType => "associated-type",
        _ => "item",
    }
}

fn member_namespaces_may_overlap(left: ReachableKind, right: ReachableKind) -> bool {
    matches!(left, ReachableKind::Field)
        && matches!(
            right,
            ReachableKind::Method | ReachableKind::AssociatedConst | ReachableKind::AssociatedType
        )
        || matches!(right, ReachableKind::Field)
            && matches!(
                left,
                ReachableKind::Method
                    | ReachableKind::AssociatedConst
                    | ReachableKind::AssociatedType
            )
}

fn conflicting_definition(
    identity: &str,
    existing: &ReachableApi,
    kind: ReachableKind,
    origin: ReachableOrigin,
) -> ReachabilityError {
    ReachabilityError::ConflictingReachableDefinition {
        identity: identity.to_owned(),
        first_kind: existing.kind,
        first_origin: existing.origin.clone(),
        second_kind: kind,
        second_origin: origin,
    }
}

fn parse_module_file(
    crate_name: &str,
    file: &Path,
    module_id: &str,
    excluded_parent: bool,
    cfg: &CfgSet,
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
        cfg,
        index,
    )
}

fn resolve_pending_impls(crate_name: &str, index: &mut CrateIndex) -> Vec<(String, PendingImpl)> {
    let pending = std::mem::take(&mut index.pending_impls);
    let mut unresolved = Vec::new();
    for pending_impl in pending {
        let owner = resolve_index_type_identity(
            crate_name,
            &pending_impl.module_id,
            &pending_impl.self_ty,
            index,
        );
        let owner =
            resolve_index_export(crate_name, &owner, index, &mut BTreeSet::new()).unwrap_or(owner);
        if let Some(declaration) = index.declarations.get_mut(&owner) {
            declaration.members.extend(pending_impl.members);
            declaration
                .member_definitions
                .extend(pending_impl.member_definitions);
        } else {
            unresolved.push((owner, pending_impl));
        }
    }
    unresolved
}

fn resolve_index_export(
    crate_name: &str,
    identity: &str,
    index: &CrateIndex,
    seen: &mut BTreeSet<String>,
) -> Option<String> {
    if index.declarations.contains_key(identity) {
        return Some(identity.to_owned());
    }
    if !seen.insert(identity.to_owned()) {
        return None;
    }
    let (module_id, name) = identity.rsplit_once("::")?;
    let module = index.modules.get(module_id)?;
    for record in &module.uses {
        let resolved = resolve_index_use_target(crate_name, module_id, &record.prefix, index);
        let candidate = match &record.leaf {
            UseLeaf::Name { source, exported } if exported == name => {
                Some(format!("{resolved}::{source}"))
            }
            UseLeaf::Glob => Some(format!("{resolved}::{name}")),
            _ => None,
        };
        if let Some(candidate) = candidate
            && let Some(target) = resolve_index_export(crate_name, &candidate, index, seen)
        {
            return Some(target);
        }
    }
    None
}

fn resolve_index_type_identity(
    crate_name: &str,
    module_id: &str,
    ty: &syn::Type,
    index: &CrateIndex,
) -> String {
    let raw = ty.to_token_stream().to_string().replace(' ', "");
    let base = raw.split('<').next().unwrap_or(&raw);
    let mut identity =
        if base.starts_with("crate::") || base.starts_with("self::") || base.starts_with("super::")
        {
            resolve_qualified_use_target(
                crate_name,
                module_id,
                &base.split("::").map(str::to_owned).collect::<Vec<_>>(),
            )
        } else {
            let (first, rest) = base.split_once("::").unwrap_or((base, ""));
            let imported = index.modules.get(module_id).and_then(|module| {
                module.uses.iter().find_map(|record| match &record.leaf {
                    UseLeaf::Name { source, exported } if exported == first => {
                        let mut target =
                            resolve_index_use_target(crate_name, module_id, &record.prefix, index);
                        target.push_str("::");
                        target.push_str(source);
                        if !rest.is_empty() {
                            target.push_str("::");
                            target.push_str(rest);
                        }
                        Some(target)
                    }
                    UseLeaf::Glob => {
                        let target = format!(
                            "{}::{base}",
                            resolve_index_use_target(crate_name, module_id, &record.prefix, index)
                        );
                        (index.declarations.contains_key(&target)
                            || index.type_aliases.contains_key(&target))
                        .then_some(target)
                    }
                    _ => None,
                })
            });
            imported.unwrap_or_else(|| {
                if base.contains("::") && base.split("::").next() == Some(crate_name) {
                    base.to_owned()
                } else {
                    format!("{module_id}::{base}")
                }
            })
        };
    let mut seen = BTreeSet::new();
    while seen.insert(identity.clone()) {
        if !index.declarations.contains_key(&identity)
            && !index.type_aliases.contains_key(&identity)
            && let Some((owner_module, name)) = identity.rsplit_once("::")
            && let Some(imported) = resolve_imported_name(crate_name, owner_module, name, index)
        {
            identity = imported;
            continue;
        }
        let Some(target) = index.type_aliases.get(&identity) else {
            break;
        };
        identity = target.clone();
    }
    identity
}

fn resolve_imported_name(
    crate_name: &str,
    module_id: &str,
    name: &str,
    index: &CrateIndex,
) -> Option<String> {
    index
        .modules
        .get(module_id)?
        .uses
        .iter()
        .find_map(|record| match &record.leaf {
            UseLeaf::Name { source, exported } if exported == name => Some(format!(
                "{}::{source}",
                resolve_index_use_target(crate_name, module_id, &record.prefix, index)
            )),
            UseLeaf::Glob => {
                let candidate = format!(
                    "{}::{name}",
                    resolve_index_use_target(crate_name, module_id, &record.prefix, index)
                );
                (index.declarations.contains_key(&candidate)
                    || index.type_aliases.contains_key(&candidate))
                .then_some(candidate)
            }
            _ => None,
        })
}

fn resolve_index_use_target(
    crate_name: &str,
    module_id: &str,
    prefix: &[String],
    index: &CrateIndex,
) -> String {
    let mut prefix = prefix.to_vec();
    if let Some(first) = prefix.first_mut()
        && let Some(alias) = index.extern_aliases.get(first)
    {
        *first = alias.clone();
    }
    let resolved = resolve_qualified_use_target(crate_name, module_id, &prefix);
    if resolved.split("::").next() == Some(crate_name)
        || prefix
            .first()
            .is_some_and(|first| first == "crate" || first == "self" || first == "super")
        || index
            .extern_aliases
            .values()
            .any(|name| name == resolved.split("::").next().unwrap_or_default())
    {
        resolved
    } else if index
        .modules
        .contains_key(&format!("{module_id}::{resolved}"))
    {
        format!("{module_id}::{resolved}")
    } else {
        format!("{crate_name}::{resolved}")
    }
}

fn parse_items(
    crate_name: &str,
    source_file: &Path,
    module_id: &str,
    items: &[syn::Item],
    excluded_parent: bool,
    cfg: &CfgSet,
    index: &mut CrateIndex,
) -> Result<(), ReachabilityError> {
    index
        .modules
        .entry(module_id.to_owned())
        .or_default()
        .excluded |= excluded_parent;
    for item in items {
        let attrs = super::item_attrs(item);
        if !cfg_enabled(attrs, cfg, source_file)? {
            continue;
        }
        let excluded = excluded_parent || module_id.ends_with("::__private");
        match item {
            syn::Item::Fn(value) if proc_macro_declaration(value, cfg, source_file)?.is_some() => {
                let (name, kind) = proc_macro_declaration(value, cfg, source_file)?
                    .expect("the match guard established a proc-macro declaration");
                let identity = format!("{crate_name}::{name}");
                index
                    .modules
                    .entry(crate_name.to_owned())
                    .or_default()
                    .declarations
                    .insert(name, identity.clone());
                index.declarations.insert(
                    identity,
                    Declaration {
                        kind,
                        members: Vec::new(),
                        excluded,
                        alias_target: None,
                        definition: source_definition(source_file, value.span().start()),
                        member_definitions: BTreeMap::new(),
                    },
                );
            }
            syn::Item::Macro(value)
                if has_attr(&value.attrs, "macro_export", cfg, source_file)? =>
            {
                let Some(name) = value.ident.as_ref().map(ident_name) else {
                    continue;
                };
                let identity = format!("{crate_name}::{name}");
                index
                    .modules
                    .entry(crate_name.to_owned())
                    .or_default()
                    .declarations
                    .insert(name, identity.clone());
                index.declarations.insert(
                    identity,
                    Declaration {
                        kind: ReachableKind::Macro,
                        members: Vec::new(),
                        excluded,
                        alias_target: None,
                        definition: source_definition(source_file, value.span().start()),
                        member_definitions: BTreeMap::new(),
                    },
                );
            }
            syn::Item::ExternCrate(value) => {
                let source = ident_name(&value.ident);
                let alias = value
                    .rename
                    .as_ref()
                    .map_or_else(|| source.clone(), |(_, alias)| ident_name(alias));
                index.extern_aliases.insert(alias, source);
            }
            syn::Item::Mod(value) => {
                let name = ident_name(&value.ident);
                let child_id = format!("{module_id}::{name}");
                index
                    .modules
                    .entry(child_id.clone())
                    .or_default()
                    .definition = Some(source_definition(source_file, value.span().start()));
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
                        cfg,
                        index,
                    )?;
                } else {
                    let directory = module_search_directory(source_file);
                    let path = module_path(&value.attrs, &directory, &name, cfg, source_file)?;
                    parse_module_file(crate_name, &path, &child_id, excluded, cfg, index)?;
                }
            }
            syn::Item::Use(value) if !excluded => {
                flatten_use(
                    Vec::new(),
                    &value.tree,
                    source_file,
                    value.span().start().line,
                    super::is_public(&value.vis),
                    &mut index.modules.entry(module_id.to_owned()).or_default().uses,
                );
            }
            syn::Item::Impl(value) if value.trait_.is_none() => {
                let mut members = Vec::new();
                let mut member_definitions = BTreeMap::new();
                for child in &value.items {
                    let member = match child {
                        syn::ImplItem::Fn(method)
                            if super::is_public(&method.vis)
                                && cfg_enabled(&method.attrs, cfg, source_file)? =>
                        {
                            Some((
                                ident_name(&method.sig.ident),
                                ReachableKind::Method,
                                excluded,
                            ))
                        }
                        syn::ImplItem::Const(item)
                            if super::is_public(&item.vis)
                                && cfg_enabled(&item.attrs, cfg, source_file)? =>
                        {
                            Some((
                                ident_name(&item.ident),
                                ReachableKind::AssociatedConst,
                                excluded,
                            ))
                        }
                        syn::ImplItem::Type(item)
                            if super::is_public(&item.vis)
                                && cfg_enabled(&item.attrs, cfg, source_file)? =>
                        {
                            Some((
                                ident_name(&item.ident),
                                ReachableKind::AssociatedType,
                                excluded,
                            ))
                        }
                        _ => None,
                    };
                    if let Some(member) = member {
                        let location = match child {
                            syn::ImplItem::Const(value) => value.span().start(),
                            syn::ImplItem::Fn(value) => value.span().start(),
                            syn::ImplItem::Type(value) => value.span().start(),
                            _ => unreachable!("only public associated items produce members"),
                        };
                        member_definitions
                            .insert(member.0.clone(), source_definition(source_file, location));
                        members.push(member);
                    }
                }
                index.pending_impls.push(PendingImpl {
                    module_id: module_id.to_owned(),
                    self_ty: (*value.self_ty).clone(),
                    members,
                    member_definitions,
                    excluded,
                });
            }
            _ => {
                if let Some((name, kind, members)) = declaration_parts(item, cfg, source_file)? {
                    let identity = format!("{module_id}::{name}");
                    if public_item(item, cfg, source_file)? {
                        index
                            .modules
                            .entry(module_id.to_owned())
                            .or_default()
                            .declarations
                            .insert(name, identity.clone());
                    }
                    let declaration =
                        index
                            .declarations
                            .entry(identity.clone())
                            .or_insert(Declaration {
                                kind,
                                members: Vec::new(),
                                excluded,
                                alias_target: None,
                                definition: source_definition(source_file, item.span().start()),
                                member_definitions: BTreeMap::new(),
                            });
                    declaration.kind = kind;
                    declaration.excluded |= excluded;
                    declaration.members.extend(members);
                    declaration
                        .member_definitions
                        .extend(member_definitions(item, source_file));
                    if let syn::Item::Type(alias) = item {
                        let target = resolve_type_identity(crate_name, module_id, &alias.ty);
                        declaration.alias_target = Some(target.clone());
                        index.type_aliases.insert(identity, target);
                    }
                }
            }
        }
    }
    Ok(())
}

fn declaration_parts(
    item: &syn::Item,
    cfg: &CfgSet,
    source: &Path,
) -> Result<Option<DeclarationParts>, ReachabilityError> {
    let plain = |name: String, kind| Some((name, kind, Vec::new()));
    Ok(match item {
        syn::Item::Struct(value) => {
            let mut fields = Vec::new();
            for (index, field) in value.fields.iter().enumerate() {
                if super::is_public(&field.vis) && cfg_enabled(&field.attrs, cfg, source)? {
                    let name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), ident_name);
                    fields.push((name, ReachableKind::Field, false));
                }
            }
            Some((ident_name(&value.ident), ReachableKind::Struct, fields))
        }
        syn::Item::Union(value) => {
            let mut fields = Vec::new();
            for field in &value.fields.named {
                if super::is_public(&field.vis) && cfg_enabled(&field.attrs, cfg, source)? {
                    fields.push((
                        ident_name(field.ident.as_ref().expect("union field is named")),
                        ReachableKind::Field,
                        false,
                    ));
                }
            }
            Some((ident_name(&value.ident), ReachableKind::Union, fields))
        }
        syn::Item::Enum(value) => {
            let mut members = Vec::new();
            for variant in &value.variants {
                if !cfg_enabled(&variant.attrs, cfg, source)? {
                    continue;
                }
                let variant_name = ident_name(&variant.ident);
                members.push((variant_name.clone(), ReachableKind::Variant, false));
                for (index, field) in variant.fields.iter().enumerate() {
                    if cfg_enabled(&field.attrs, cfg, source)? {
                        let field_name = field
                            .ident
                            .as_ref()
                            .map_or_else(|| index.to_string(), ident_name);
                        members.push((
                            format!("{variant_name}::{field_name}"),
                            ReachableKind::Field,
                            false,
                        ));
                    }
                }
            }
            Some((ident_name(&value.ident), ReachableKind::Enum, members))
        }
        syn::Item::Trait(value) => {
            let mut members = Vec::new();
            for child in &value.items {
                let member = match child {
                    syn::TraitItem::Fn(method) if cfg_enabled(&method.attrs, cfg, source)? => {
                        Some((
                            ident_name(&method.sig.ident),
                            ReachableKind::TraitMethod,
                            false,
                        ))
                    }
                    syn::TraitItem::Const(item) if cfg_enabled(&item.attrs, cfg, source)? => {
                        Some((
                            ident_name(&item.ident),
                            ReachableKind::AssociatedConst,
                            false,
                        ))
                    }
                    syn::TraitItem::Type(item) if cfg_enabled(&item.attrs, cfg, source)? => Some((
                        ident_name(&item.ident),
                        ReachableKind::AssociatedType,
                        false,
                    )),
                    _ => None,
                };
                if let Some(member) = member {
                    members.push(member);
                }
            }
            Some((ident_name(&value.ident), ReachableKind::Trait, members))
        }
        syn::Item::Fn(value) => plain(ident_name(&value.sig.ident), ReachableKind::Function),
        syn::Item::Type(value) => plain(ident_name(&value.ident), ReachableKind::TypeAlias),
        syn::Item::Const(value) => plain(ident_name(&value.ident), ReachableKind::Constant),
        syn::Item::Static(value) => plain(ident_name(&value.ident), ReachableKind::Static),
        _ => None,
    })
}

fn member_definitions(item: &syn::Item, source: &Path) -> BTreeMap<String, SourceDefinition> {
    let mut definitions = BTreeMap::new();
    match item {
        syn::Item::Struct(value) => {
            for (index, field) in value.fields.iter().enumerate() {
                let name = field
                    .ident
                    .as_ref()
                    .map_or_else(|| index.to_string(), ident_name);
                definitions.insert(name, source_definition(source, field.span().start()));
            }
        }
        syn::Item::Union(value) => {
            for field in &value.fields.named {
                let name = ident_name(field.ident.as_ref().expect("union field is named"));
                definitions.insert(name, source_definition(source, field.span().start()));
            }
        }
        syn::Item::Enum(value) => {
            for variant in &value.variants {
                let variant_name = ident_name(&variant.ident);
                definitions.insert(
                    variant_name.clone(),
                    source_definition(source, variant.span().start()),
                );
                for (index, field) in variant.fields.iter().enumerate() {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), ident_name);
                    definitions.insert(
                        format!("{variant_name}::{field_name}"),
                        source_definition(source, field.span().start()),
                    );
                }
            }
        }
        syn::Item::Trait(value) => {
            for member in &value.items {
                let (name, location) = match member {
                    syn::TraitItem::Const(value) => {
                        (ident_name(&value.ident), value.span().start())
                    }
                    syn::TraitItem::Fn(value) => {
                        (ident_name(&value.sig.ident), value.span().start())
                    }
                    syn::TraitItem::Type(value) => (ident_name(&value.ident), value.span().start()),
                    _ => continue,
                };
                definitions.insert(name, source_definition(source, location));
            }
        }
        _ => {}
    }
    definitions
}

fn source_definition(source: &Path, location: proc_macro2::LineColumn) -> SourceDefinition {
    SourceDefinition {
        source: fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf()),
        line: location.line,
        column: location.column,
    }
}

fn public_item(item: &syn::Item, cfg: &CfgSet, source: &Path) -> Result<bool, ReachabilityError> {
    Ok(match item {
        syn::Item::Struct(v) => super::is_public(&v.vis),
        syn::Item::Union(v) => super::is_public(&v.vis),
        syn::Item::Enum(v) => super::is_public(&v.vis),
        syn::Item::Trait(v) => super::is_public(&v.vis),
        syn::Item::Fn(v) => super::is_public(&v.vis),
        syn::Item::Type(v) => super::is_public(&v.vis),
        syn::Item::Const(v) => super::is_public(&v.vis),
        syn::Item::Static(v) => super::is_public(&v.vis),
        syn::Item::Macro(v) => has_attr(&v.attrs, "macro_export", cfg, source)?,
        _ => false,
    })
}

fn proc_macro_declaration(
    function: &syn::ItemFn,
    cfg: &CfgSet,
    source: &Path,
) -> Result<Option<(String, ReachableKind)>, ReachabilityError> {
    let mut declaration = None;
    for attr in effective_attributes(&function.attrs, cfg, source)? {
        let candidate = if attr.meta.path().is_ident("proc_macro") {
            Some((
                ident_name(&function.sig.ident),
                ReachableKind::FunctionLikeMacro,
            ))
        } else if attr.meta.path().is_ident("proc_macro_attribute") {
            Some((
                ident_name(&function.sig.ident),
                ReachableKind::AttributeMacro,
            ))
        } else if attr.meta.path().is_ident("proc_macro_derive") {
            let syn::Meta::List(list) = &attr.meta else {
                return Err(ReachabilityError::Parse(format!(
                    "{}:{}: malformed proc_macro_derive declaration",
                    source.display(),
                    attr.line
                )));
            };
            let arguments = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .map_err(|error| {
                    ReachabilityError::Parse(format!(
                        "{}:{}: malformed proc_macro_derive declaration: {error}",
                        source.display(),
                        attr.line
                    ))
                })?;
            let Some(syn::Meta::Path(name)) = arguments.first() else {
                return Err(ReachabilityError::Parse(format!(
                    "{}:{}: proc_macro_derive requires an exported derive name",
                    source.display(),
                    attr.line
                )));
            };
            let Some(name) = name.segments.last() else {
                return Err(ReachabilityError::Parse(format!(
                    "{}:{}: proc_macro_derive has an empty exported name",
                    source.display(),
                    attr.line
                )));
            };
            Some((ident_name(&name.ident), ReachableKind::DeriveMacro))
        } else {
            None
        };
        if let Some(candidate) = candidate {
            if declaration.is_some() {
                return Err(ReachabilityError::Parse(format!(
                    "{}:{}: a function cannot declare more than one proc-macro export",
                    source.display(),
                    function.span().start().line
                )));
            }
            declaration = Some(candidate);
        }
    }
    Ok(declaration)
}

fn flatten_use(
    prefix: Vec<String>,
    tree: &syn::UseTree,
    source: &Path,
    line: usize,
    public: bool,
    output: &mut Vec<UseRecord>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut next = prefix;
            next.push(ident_name(&path.ident));
            flatten_use(next, &path.tree, source, line, public, output);
        }
        syn::UseTree::Name(name) => {
            let name = ident_name(&name.ident);
            if name == "self" && !prefix.is_empty() {
                let mut parent = prefix;
                let exported = parent.pop().expect("checked nonempty prefix");
                output.push(UseRecord {
                    prefix: parent,
                    leaf: UseLeaf::Name {
                        source: exported.clone(),
                        exported,
                    },
                    source: source.to_path_buf(),
                    line,
                    public,
                });
            } else {
                output.push(UseRecord {
                    prefix,
                    leaf: UseLeaf::Name {
                        source: name.clone(),
                        exported: name,
                    },
                    source: source.to_path_buf(),
                    line,
                    public,
                });
            }
        }
        syn::UseTree::Rename(rename) => output.push(UseRecord {
            prefix,
            leaf: UseLeaf::Name {
                source: ident_name(&rename.ident),
                exported: ident_name(&rename.rename),
            },
            source: source.to_path_buf(),
            line,
            public,
        }),
        syn::UseTree::Glob(_) => output.push(UseRecord {
            prefix,
            leaf: UseLeaf::Glob,
            source: source.to_path_buf(),
            line,
            public,
        }),
        syn::UseTree::Group(group) => {
            for child in &group.items {
                flatten_use(prefix.clone(), child, source, line, public, output);
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
    if base.is_empty() {
        rest.join("::")
    } else if rest.is_empty() {
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

fn module_path(
    attrs: &[syn::Attribute],
    directory: &Path,
    name: &str,
    cfg: &CfgSet,
    source: &Path,
) -> Result<PathBuf, ReachabilityError> {
    if let Some(relative) = effective_attributes(attrs, cfg, source)?
        .iter()
        .find_map(|attr| {
            if !attr.meta.path().is_ident("path") {
                return None;
            }
            match &attr.meta {
                syn::Meta::NameValue(value) => match &value.value {
                    syn::Expr::Lit(value) => match &value.lit {
                        syn::Lit::Str(path) => Some(path.value()),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            }
        })
    {
        return Ok(directory.join(relative));
    }
    let sibling = directory.join(format!("{name}.rs"));
    let nested = directory.join(name).join("mod.rs");
    Ok(if sibling.exists() { sibling } else { nested })
}

fn module_search_directory(source_file: &Path) -> PathBuf {
    let parent = source_file.parent().unwrap_or_else(|| Path::new("."));
    match source_file.file_name().and_then(|name| name.to_str()) {
        Some("lib.rs" | "main.rs" | "mod.rs") => parent.to_path_buf(),
        _ => source_file
            .file_stem()
            .map_or_else(|| parent.to_path_buf(), |stem| parent.join(stem)),
    }
}

fn cfg_enabled(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
) -> Result<bool, ReachabilityError> {
    for attr in effective_attributes(attrs, cfg, source)?
        .iter()
        .filter(|attr| attr.meta.path().is_ident("cfg"))
    {
        let syn::Meta::List(list) = &attr.meta else {
            return Err(ReachabilityError::Parse(format!(
                "{}:{}: malformed cfg attribute",
                source.display(),
                attr.line
            )));
        };
        let meta = list.parse_args::<syn::Meta>().map_err(|error| {
            ReachabilityError::Parse(format!("{}:{}: {error}", source.display(), attr.line))
        })?;
        match eval_cfg(&meta, cfg) {
            Ok(true) => {}
            Ok(false) => return Ok(false),
            Err(predicate) => {
                return Err(ReachabilityError::UnknownCfg {
                    source: source.to_path_buf(),
                    line: attr.line,
                    predicate,
                });
            }
        }
    }
    Ok(true)
}

#[derive(Clone)]
struct EffectiveAttribute {
    meta: syn::Meta,
    line: usize,
}

fn effective_attributes(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
) -> Result<Vec<EffectiveAttribute>, ReachabilityError> {
    let mut output = Vec::new();
    for attr in attrs {
        expand_attribute_meta(
            attr.meta.clone(),
            attr.span().start().line,
            cfg,
            source,
            &mut output,
        )?;
    }
    Ok(output)
}

fn expand_attribute_meta(
    meta: syn::Meta,
    line: usize,
    cfg: &CfgSet,
    source: &Path,
    output: &mut Vec<EffectiveAttribute>,
) -> Result<(), ReachabilityError> {
    if !meta.path().is_ident("cfg_attr") {
        output.push(EffectiveAttribute { meta, line });
        return Ok(());
    }
    let syn::Meta::List(list) = &meta else {
        return Err(ReachabilityError::Parse(format!(
            "{}:{line}: malformed cfg_attr attribute",
            source.display()
        )));
    };
    let arguments = list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        .map_err(|error| {
            ReachabilityError::Parse(format!(
                "{}:{line}: malformed cfg_attr attribute: {error}",
                source.display()
            ))
        })?;
    let mut arguments = arguments.into_iter();
    let Some(predicate) = arguments.next() else {
        return Err(ReachabilityError::Parse(format!(
            "{}:{line}: cfg_attr requires a predicate and an attribute",
            source.display()
        )));
    };
    let nested = arguments.collect::<Vec<_>>();
    if nested.is_empty() {
        return Err(ReachabilityError::Parse(format!(
            "{}:{line}: cfg_attr requires an attribute",
            source.display()
        )));
    }
    let enabled = eval_cfg(&predicate, cfg).map_err(|predicate| ReachabilityError::UnknownCfg {
        source: source.to_path_buf(),
        line,
        predicate,
    })?;
    if enabled {
        for nested in nested {
            expand_attribute_meta(nested, line, cfg, source, output)?;
        }
    }
    Ok(())
}

fn has_attr(
    attrs: &[syn::Attribute],
    name: &str,
    cfg: &CfgSet,
    source: &Path,
) -> Result<bool, ReachabilityError> {
    Ok(effective_attributes(attrs, cfg, source)?
        .iter()
        .any(|attr| attr.meta.path().is_ident(name)))
}

fn eval_cfg(meta: &syn::Meta, cfg: &CfgSet) -> Result<bool, String> {
    match meta {
        syn::Meta::NameValue(value) if value.path.is_ident("feature") => match &value.value {
            syn::Expr::Lit(expr) => match &expr.lit {
                syn::Lit::Str(value) => Ok(cfg.features.contains(&value.value())),
                _ => Err(meta.to_token_stream().to_string()),
            },
            _ => Err(meta.to_token_stream().to_string()),
        },
        syn::Meta::NameValue(value) => {
            let Some(key) = value.path.get_ident().map(ident_name) else {
                return Err(meta.to_token_stream().to_string());
            };
            let syn::Expr::Lit(expr) = &value.value else {
                return Err(meta.to_token_stream().to_string());
            };
            let syn::Lit::Str(value) = &expr.lit else {
                return Err(meta.to_token_stream().to_string());
            };
            let Some(values) = cfg.key_values.get(&key) else {
                return Err(meta.to_token_stream().to_string());
            };
            Ok(values.contains(&value.value()))
        }
        syn::Meta::Path(path) => {
            let Some(flag) = path.get_ident().map(ident_name) else {
                return Err(meta.to_token_stream().to_string());
            };
            cfg.flags.get(&flag).copied().ok_or(flag)
        }
        syn::Meta::List(list) if list.path.is_ident("all") => list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .map_err(|_| meta.to_token_stream().to_string())?
            .iter()
            .try_fold(true, |enabled, item| {
                eval_cfg(item, cfg).map(|item_enabled| enabled && item_enabled)
            }),
        syn::Meta::List(list) if list.path.is_ident("any") => list
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )
            .map_err(|_| meta.to_token_stream().to_string())?
            .iter()
            .try_fold(false, |enabled, item| {
                eval_cfg(item, cfg).map(|item_enabled| enabled || item_enabled)
            }),
        syn::Meta::List(list) if list.path.is_ident("not") => list
            .parse_args::<syn::Meta>()
            .map_err(|_| meta.to_token_stream().to_string())
            .and_then(|item| eval_cfg(&item, cfg).map(|enabled| !enabled)),
        _ => Err(meta.to_token_stream().to_string()),
    }
}

fn ident_name(ident: &syn::Ident) -> String {
    let name = ident.to_string();
    name.strip_prefix("r#").unwrap_or(&name).to_owned()
}
