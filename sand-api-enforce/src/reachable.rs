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

use crate::macro_provider::{
    audit_inert_macro_transcriber, audit_inventory_collection_invocation,
    audit_thread_local_invocation,
};

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
    /// Exact source declaration and macro use which emitted this family.
    ///
    /// Provider names group generated APIs into ratchet scopes; they are not
    /// proof that a particular macro invocation was modeled. API-producing
    /// derives and attributes therefore require this declaration-level edge.
    pub producer: Option<GeneratedProducer>,
    pub kind: ReachableKind,
    /// Associated items emitted with a generated type.
    pub members: Vec<(String, ReachableKind)>,
    /// Whether the generator intentionally emits compiler-only wiring.
    pub excluded: bool,
}

/// The source-side macro invocation modeled by a generated declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedProducer {
    pub owner: String,
    pub name: String,
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
    UnboundInclude {
        source: PathBuf,
        line: usize,
        module: String,
        expression: String,
    },
    InvalidIncludeProvider {
        module: String,
        provider: String,
        dynamic_includes: usize,
    },
    UnboundItemMacro {
        source: PathBuf,
        line: usize,
        module: String,
        macro_path: String,
    },
    InvalidItemMacroProvider {
        module: String,
        macro_path: String,
        provider: String,
        invocations: usize,
    },
    UnboundAssociatedItemMacro {
        source: PathBuf,
        line: usize,
        owner: String,
        macro_path: String,
    },
    InvalidAssociatedItemMacroProvider {
        owner: String,
        macro_path: String,
        provider: String,
        invocations: usize,
    },
    InvalidInertItemMacro {
        module: String,
        macro_path: String,
        classification: InertItemMacroClassification,
        invocations: usize,
        reason: String,
    },
    UnsupportedReachableSyntax {
        source: PathBuf,
        line: usize,
        module: String,
        syntax: &'static str,
    },
    UnboundApiProducer {
        source: PathBuf,
        line: usize,
        producer: String,
        owner: String,
    },
    InvalidApiProducerProvider {
        owner: String,
        producer: String,
        provider: String,
        expected: Vec<String>,
        actual: Vec<String>,
    },
    UnclassifiedApiMacro {
        source: PathBuf,
        line: usize,
        owner: String,
        name: String,
        form: &'static str,
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
                "{}:{line}: public facade edge `{facade_path}` cannot resolve audited source target `{target}`; external dependencies must not be re-exported through the supported facade unless their source is explicitly modeled",
                source.display()
            ),
            Self::UnresolvedImplOwner { module, self_type } => write!(
                formatter,
                "inherent impl in `{module}` has public associated API but its owner `{self_type}` cannot be resolved to a local declaration"
            ),
            Self::UnboundInclude {
                source,
                line,
                module,
                expression,
            } => write!(
                formatter,
                "{}:{line}: reachable module `{module}` contains include!({expression}) that is neither a literal source include nor bound to a named generated API provider",
                source.display()
            ),
            Self::InvalidIncludeProvider {
                module,
                provider,
                dynamic_includes,
            } => write!(
                formatter,
                "generated include binding `{module}` -> `{provider}` requires exactly one opaque include and at least one matching generated API declaration beneath that module (found {dynamic_includes} opaque includes)"
            ),
            Self::UnboundItemMacro {
                source,
                line,
                module,
                macro_path,
            } => write!(
                formatter,
                "{}:{line}: reachable module `{module}` invokes item-position macro `{macro_path}!` without an exact generated API provider binding",
                source.display()
            ),
            Self::InvalidItemMacroProvider {
                module,
                macro_path,
                provider,
                invocations,
            } => write!(
                formatter,
                "item macro binding `({module}, {macro_path}!)` -> `{provider}` requires at least one exact invocation and at least one provider-owned generated declaration beneath that module (found {invocations} invocations)"
            ),
            Self::UnboundAssociatedItemMacro {
                source,
                line,
                owner,
                macro_path,
            } => write!(
                formatter,
                "{}:{line}: reachable API `{owner}` invokes associated-item macro `{macro_path}!` without an exact generated API provider binding",
                source.display()
            ),
            Self::InvalidAssociatedItemMacroProvider {
                owner,
                macro_path,
                provider,
                invocations,
            } => write!(
                formatter,
                "associated item macro binding `({owner}, {macro_path}!)` -> `{provider}` requires at least one exact invocation and provider-owned generated output directly beneath that owner (found {invocations} invocations)"
            ),
            Self::InvalidInertItemMacro {
                module,
                macro_path,
                classification,
                invocations,
                reason,
            } => write!(
                formatter,
                "inert item macro binding `({module}, {macro_path}!)` as {classification:?} is invalid (found {invocations} exact invocations): {reason}"
            ),
            Self::UnsupportedReachableSyntax {
                source,
                line,
                module,
                syntax,
            } => write!(
                formatter,
                "{}:{line}: reachable module `{module}` contains unsupported `{syntax}` syntax whose public surface cannot be modeled",
                source.display()
            ),
            Self::UnboundApiProducer {
                source,
                line,
                producer,
                owner,
            } => write!(
                formatter,
                "{}:{line}: reachable API source `{owner}` uses API-producing Sand macro `{producer}` without a connected generated API provider",
                source.display()
            ),
            Self::InvalidApiProducerProvider {
                owner,
                producer,
                provider,
                expected,
                actual,
            } => write!(
                formatter,
                "API-producing macro `{producer}` on `{owner}` is bound to provider `{provider}`, but its generated identities differ: expected [{}], actual [{}]",
                expected.join(", "),
                actual.join(", ")
            ),
            Self::UnclassifiedApiMacro {
                source,
                line,
                owner,
                name,
                form,
            } => write!(
                formatter,
                "{}:{line}: reachable API source `{owner}` uses unclassified custom {form} `{name}`; classify it as an API producer, shape-preserving inert macro, or trait-only derive",
                source.display()
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
    include_providers: BTreeMap<String, String>,
    item_macro_providers: BTreeMap<(String, String), String>,
    associated_item_macro_providers: BTreeMap<(String, String), String>,
    inert_item_macros: BTreeMap<(String, String), InertItemMacroClassification>,
    api_producer_bindings: BTreeMap<(String, String), String>,
    transcriber_producers: Vec<ApiProducerUse>,
    associated_item_macros: Vec<AssociatedMacroSite>,
}

#[derive(Default)]
struct CrateIndex {
    modules: BTreeMap<String, Module>,
    declarations: BTreeMap<String, Declaration>,
    extern_aliases: BTreeMap<String, String>,
    type_aliases: BTreeMap<String, String>,
    pending_impls: Vec<PendingImpl>,
    transcriber_producers: Vec<ApiProducerUse>,
    associated_item_macros: Vec<AssociatedMacroSite>,
}

struct PendingImpl {
    module_id: String,
    self_ty: syn::Type,
    members: Vec<MemberRecord>,
    member_definitions: BTreeMap<String, SourceDefinition>,
    api_producers: Vec<ApiProducerUse>,
    excluded: bool,
    item_macros: Vec<ItemMacroSite>,
}

#[derive(Default)]
struct Module {
    declarations: BTreeMap<String, String>,
    modules: BTreeMap<String, String>,
    uses: Vec<UseRecord>,
    excluded: bool,
    definition: Option<SourceDefinition>,
    dynamic_includes: Vec<IncludeSite>,
    item_macros: Vec<ItemMacroSite>,
    macro_rules: BTreeMap<String, proc_macro2::TokenStream>,
    unsupported_syntax: Vec<UnsupportedSyntaxSite>,
    api_producers: Vec<ApiProducerUse>,
}

/// A narrow reason why an item-position macro intentionally emits no public
/// API identity and therefore needs no generated declaration provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InertItemMacroClassification {
    /// A local `macro_rules!` family whose expansion arms have been proven to
    /// emit only private items and trait implementations.
    LocalTraitImplOnly,
    /// `inventory::collect!` registers compiler/linker wiring but declares no
    /// Rust item reachable through Sand's facade.
    InventoryCollectionWiring,
    /// `thread_local!` declares internal storage. This classification is only
    /// valid for the exact `thread_local` or `std::thread_local` macro path.
    ThreadLocalStorageWiring,
}

impl InertItemMacroClassification {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::LocalTraitImplOnly => {
                "structurally audited local trait implementations/private items"
            }
            Self::InventoryCollectionWiring => {
                "inventory linker registration with no facade identity"
            }
            Self::ThreadLocalStorageWiring => {
                "internal thread-local compiler wiring with no facade identity"
            }
        }
    }
}

struct IncludeSite {
    source: PathBuf,
    line: usize,
    expression: String,
}

struct ItemMacroSite {
    source: PathBuf,
    line: usize,
    macro_path: String,
    tokens: proc_macro2::TokenStream,
}

struct AssociatedMacroSite {
    owner: String,
    site: ItemMacroSite,
}

struct UnsupportedSyntaxSite {
    source: PathBuf,
    line: usize,
    syntax: &'static str,
}

struct Declaration {
    kind: ReachableKind,
    members: Vec<MemberRecord>,
    excluded: bool,
    alias_target: Option<String>,
    definition: SourceDefinition,
    member_definitions: BTreeMap<String, SourceDefinition>,
    api_producers: Vec<ApiProducerUse>,
}

#[derive(Clone)]
struct ApiProducerUse {
    name: String,
    source: PathBuf,
    line: usize,
    owner: String,
    expected_generated: Option<BTreeSet<(String, ReachableKind)>>,
    unclassified_form: Option<&'static str>,
    requires_binding: bool,
    bare_name: bool,
    qualified_root: Option<String>,
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
        let transcriber_producers = indices
            .values_mut()
            .flat_map(|index| std::mem::take(&mut index.transcriber_producers))
            .collect();
        let associated_item_macros = indices
            .values_mut()
            .flat_map(|index| std::mem::take(&mut index.associated_item_macros))
            .collect();
        Ok(Self {
            crates: indices,
            generated,
            include_providers: BTreeMap::new(),
            item_macro_providers: BTreeMap::new(),
            associated_item_macro_providers: BTreeMap::new(),
            inert_item_macros: BTreeMap::new(),
            api_producer_bindings: BTreeMap::new(),
            transcriber_producers,
            associated_item_macros,
        })
    }

    /// Connect one source declaration using a known Sand macro that can emit
    /// additional public items to the generator provider which models them.
    ///
    /// Built-in/inert attributes and derives that only implement traits do not
    /// need a binding. Binding the same producer on another declaration is not
    /// sufficient: every occurrence is an explicit edge, so adding a new
    /// derived API fails closed. The deliberately small producer set is
    /// recognized by the internal `api_producing_sand_macro` classifier.
    pub fn bind_api_producer(
        mut self,
        owner: impl Into<String>,
        producer: impl Into<String>,
        provider: impl Into<String>,
    ) -> Result<Self, ReachabilityError> {
        let owner = owner.into();
        let producer = producer.into();
        let provider = provider.into();
        let expected = self
            .crates
            .values()
            .flat_map(|index| index.declarations.values())
            .flat_map(|declaration| &declaration.api_producers)
            .chain(self.transcriber_producers.iter())
            .find(|usage| usage.owner == owner && usage.name == producer)
            .and_then(|usage| usage.expected_generated.clone())
            .unwrap_or_default();
        let actual = self
            .generated
            .iter()
            .filter(|item| {
                !item.excluded
                    && item.provider == provider
                    && item.producer.as_ref().is_some_and(|generated| {
                        generated.owner == owner && generated.name == producer
                    })
            })
            .map(|item| (item.identity.clone(), item.kind))
            .collect::<BTreeSet<_>>();
        if expected.is_empty() || actual != expected {
            return Err(ReachabilityError::InvalidApiProducerProvider {
                owner,
                producer,
                provider,
                expected: expected
                    .into_iter()
                    .map(|(identity, kind)| format!("{identity} [{kind:?}]"))
                    .collect(),
                actual: actual
                    .into_iter()
                    .map(|(identity, kind)| format!("{identity} [{kind:?}]"))
                    .collect(),
            });
        }
        self.api_producer_bindings
            .insert((owner, producer), provider);
        Ok(self)
    }

    /// Bind a non-literal `include!` in `module` to the named provider that
    /// owns the generated declarations emitted there.
    ///
    /// This is intentionally an explicit build-graph edge. Merely placing a
    /// provider somewhere in the catalog cannot exempt an opaque include: the
    /// provider must actually own at least one declaration below this module.
    pub fn bind_generated_include(
        mut self,
        module: impl Into<String>,
        provider: impl Into<String>,
    ) -> Result<Self, ReachabilityError> {
        let module = module.into();
        let provider = provider.into();
        let prefix = format!("{module}::");
        let dynamic_includes = self
            .module(&module)
            .map_or(0, |module| module.dynamic_includes.len());
        let owns_declaration = self.generated.iter().any(|item| {
            item.provider == provider && !item.excluded && item.identity.starts_with(&prefix)
        });
        if dynamic_includes != 1 || !owns_declaration {
            return Err(ReachabilityError::InvalidIncludeProvider {
                module,
                provider,
                dynamic_includes,
            });
        }
        self.include_providers.insert(module, provider);
        Ok(self)
    }

    /// Bind ordinary item-position invocations of one exact macro path in a
    /// defining module to the provider which models their generated surface.
    ///
    /// The key is lexical and intentionally narrow: binding `family!` in one
    /// module neither classifies another macro nor the same spelling in a
    /// different module. The provider must own a concrete generated
    /// declaration below the defining module, so an unrelated catalog entry
    /// cannot turn the binding into an exemption.
    pub fn bind_item_macro_provider(
        mut self,
        module: impl Into<String>,
        macro_path: impl Into<String>,
        provider: impl Into<String>,
    ) -> Result<Self, ReachabilityError> {
        let module = module.into();
        let macro_path = macro_path.into();
        let provider = provider.into();
        let invocations = self.module(&module).map_or(0, |parsed| {
            parsed
                .item_macros
                .iter()
                .filter(|site| site.macro_path == macro_path)
                .count()
        });
        let prefix = format!("{module}::");
        let owns_declaration = self.generated.iter().any(|item| {
            item.provider == provider && !item.excluded && item.identity.starts_with(&prefix)
        });
        if invocations == 0 || !owns_declaration {
            return Err(ReachabilityError::InvalidItemMacroProvider {
                module,
                macro_path,
                provider,
                invocations,
            });
        }
        self.item_macro_providers
            .insert((module, macro_path), provider);
        Ok(self)
    }

    /// Bind a macro invocation inside one exact inherent impl or trait to the
    /// generated associated identities it emits. An ordinary module-level
    /// binding cannot cover this site, and output owned by another type or
    /// trait cannot satisfy it.
    pub fn bind_associated_item_macro_provider(
        mut self,
        owner: impl Into<String>,
        macro_path: impl Into<String>,
        provider: impl Into<String>,
    ) -> Result<Self, ReachabilityError> {
        let owner = owner.into();
        let macro_path = macro_path.into();
        let provider = provider.into();
        let invocations = self
            .associated_item_macros
            .iter()
            .filter(|site| site.owner == owner && site.site.macro_path == macro_path)
            .count();
        let owns_direct_output = self.generated.iter().any(|item| {
            item.provider == provider
                && !item.excluded
                && generated_parent(&item.identity) == Some(owner.as_str())
        });
        if invocations == 0 || !owns_direct_output {
            return Err(ReachabilityError::InvalidAssociatedItemMacroProvider {
                owner,
                macro_path,
                provider,
                invocations,
            });
        }
        self.associated_item_macro_providers
            .insert((owner, macro_path), provider);
        Ok(self)
    }

    /// Classify one exact item-position macro invocation family as producing
    /// no facade API identity.
    ///
    /// Unlike [`Self::bind_item_macro_provider`], this creates no generated
    /// declarations. Local macro families are accepted only after their
    /// transcribers pass the structural trait-impl/private-item audit.
    /// External classifications are restricted to two documented compiler
    /// wiring macros and cannot be applied to an arbitrary spelling.
    pub fn bind_inert_item_macro(
        mut self,
        module: impl Into<String>,
        macro_path: impl Into<String>,
        classification: InertItemMacroClassification,
    ) -> Result<Self, ReachabilityError> {
        let module = module.into();
        let macro_path = macro_path.into();
        let invocations = self.module(&module).map_or(0, |parsed| {
            parsed
                .item_macros
                .iter()
                .filter(|site| site.macro_path == macro_path)
                .count()
        });
        let invalid = |reason: String| ReachabilityError::InvalidInertItemMacro {
            module: module.clone(),
            macro_path: macro_path.clone(),
            classification,
            invocations,
            reason,
        };
        if invocations == 0 {
            return Err(invalid("the exact module/path has no invocation".into()));
        }
        if self
            .item_macro_providers
            .contains_key(&(module.clone(), macro_path.clone()))
        {
            return Err(invalid(
                "the same invocation already has a generated API provider".into(),
            ));
        }
        if let Some(existing) = self
            .inert_item_macros
            .get(&(module.clone(), macro_path.clone()))
            && *existing != classification
        {
            return Err(invalid(format!(
                "the same invocation is already classified as {existing:?}"
            )));
        }
        match classification {
            InertItemMacroClassification::LocalTraitImplOnly => {
                if macro_path.contains("::") {
                    return Err(invalid(
                        "local inert families must resolve to a bare macro_rules name in the exact defining module".into(),
                    ));
                }
                let transcriber = self
                    .module(&module)
                    .and_then(|parsed| parsed.macro_rules.get(&macro_path))
                    .ok_or_else(|| {
                        invalid(
                            "no matching local macro_rules definition exists in the exact module"
                                .into(),
                        )
                    })?;
                audit_inert_macro_transcriber(transcriber)
                    .map_err(|error| invalid(error.to_string()))?;
            }
            InertItemMacroClassification::InventoryCollectionWiring => {
                if macro_path != "inventory::collect" {
                    return Err(invalid(
                        "inventory wiring is valid only for `inventory::collect!`".into(),
                    ));
                }
                for site in self
                    .module(&module)
                    .into_iter()
                    .flat_map(|parsed| &parsed.item_macros)
                    .filter(|site| site.macro_path == macro_path)
                {
                    audit_inventory_collection_invocation(&site.tokens)
                        .map_err(|error| invalid(error.to_string()))?;
                }
            }
            InertItemMacroClassification::ThreadLocalStorageWiring => {
                if !matches!(macro_path.as_str(), "thread_local" | "std::thread_local") {
                    return Err(invalid(
                        "thread-local wiring is valid only for `thread_local!` or `std::thread_local!`"
                            .into(),
                    ));
                }
                for site in self
                    .module(&module)
                    .into_iter()
                    .flat_map(|parsed| &parsed.item_macros)
                    .filter(|site| site.macro_path == macro_path)
                {
                    audit_thread_local_invocation(&site.tokens)
                        .map_err(|error| invalid(error.to_string()))?;
                }
            }
        }
        self.inert_item_macros
            .insert((module, macro_path), classification);
        Ok(self)
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
        for producer in &self.transcriber_producers {
            self.require_api_producer_binding(producer)?;
        }
        let mut values = found.into_values().collect::<Vec<_>>();
        values.sort_by(|left, right| left.identity.cmp(&right.identity));
        Ok(values)
    }

    fn require_api_producer_binding(
        &self,
        producer: &ApiProducerUse,
    ) -> Result<(), ReachabilityError> {
        if let Some(form) = producer.unclassified_form {
            return Err(ReachabilityError::UnclassifiedApiMacro {
                source: producer.source.clone(),
                line: producer.line,
                owner: producer.owner.clone(),
                name: producer.name.clone(),
                form,
            });
        }
        if producer.bare_name && self.macro_name_is_shadowed(producer) {
            return Err(ReachabilityError::UnclassifiedApiMacro {
                source: producer.source.clone(),
                line: producer.line,
                owner: producer.owner.clone(),
                name: producer.name.clone(),
                form: "macro imported under an audited bare name",
            });
        }
        if let Some(root) = &producer.qualified_root
            && self.qualified_macro_root_is_shadowed(producer, root)
        {
            return Err(ReachabilityError::UnclassifiedApiMacro {
                source: producer.source.clone(),
                line: producer.line,
                owner: producer.owner.clone(),
                name: producer.name.clone(),
                form: "qualified macro path with a shadowed crate root",
            });
        }
        if !producer.requires_binding {
            return Ok(());
        }
        if self
            .api_producer_bindings
            .contains_key(&(producer.owner.clone(), producer.name.clone()))
        {
            Ok(())
        } else {
            Err(ReachabilityError::UnboundApiProducer {
                source: producer.source.clone(),
                line: producer.line,
                producer: producer.name.clone(),
                owner: producer.owner.clone(),
            })
        }
    }

    fn macro_name_is_shadowed(&self, producer: &ApiProducerUse) -> bool {
        let Some(module_id) = self.producer_defining_module(&producer.owner) else {
            return false;
        };
        let Some(module) = self.module(module_id) else {
            return false;
        };
        module.uses.iter().any(|record| match &record.leaf {
            UseLeaf::Glob => true,
            UseLeaf::Name { source, exported } if exported == &producer.name => {
                let mut path = record.prefix.clone();
                path.push(source.clone());
                !trusted_macro_import(&producer.name, &path)
            }
            _ => false,
        })
    }

    fn qualified_macro_root_is_shadowed(&self, producer: &ApiProducerUse, root: &str) -> bool {
        let Some(module_id) = self.producer_defining_module(&producer.owner) else {
            return false;
        };
        let crate_name = module_id.split("::").next().unwrap_or(module_id);
        let local_module = self.module(&format!("{module_id}::{root}")).is_some();
        let module_shadow = self.module(module_id).is_some_and(|module| {
            local_module
                || module.uses.iter().any(|record| {
                    matches!(&record.leaf, UseLeaf::Name { exported, .. } if exported == root)
                })
        });
        let extern_shadow = self
            .crates
            .get(crate_name)
            .and_then(|index| index.extern_aliases.get(root))
            .is_some_and(|target| target != root);
        module_shadow || extern_shadow
    }

    fn producer_defining_module<'a>(&self, owner: &'a str) -> Option<&'a str> {
        let (mut candidate, _) = owner.rsplit_once("::")?;
        loop {
            if self.module(candidate).is_some() {
                return Some(candidate);
            }
            (candidate, _) = candidate.rsplit_once("::")?;
        }
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
        self.require_audited_includes(module_id, module)?;
        self.require_supported_syntax(module_id, module)?;
        self.require_item_macro_bindings(module_id, module)?;
        for producer in &module.api_producers {
            self.require_api_producer_binding(producer)?;
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
                        if !exposed && !self.known_excluded(&resolved_target) {
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
                    if self.module(&resolved).is_none() {
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

    fn require_audited_includes(
        &self,
        module_id: &str,
        module: &Module,
    ) -> Result<(), ReachabilityError> {
        if module.dynamic_includes.is_empty() || self.include_providers.contains_key(module_id) {
            return Ok(());
        }
        let include = &module.dynamic_includes[0];
        Err(ReachabilityError::UnboundInclude {
            source: include.source.clone(),
            line: include.line,
            module: module_id.to_owned(),
            expression: include.expression.clone(),
        })
    }

    fn require_item_macro_bindings(
        &self,
        module_id: &str,
        module: &Module,
    ) -> Result<(), ReachabilityError> {
        if let Some(site) = module.item_macros.iter().find(|site| {
            let key = (module_id.to_owned(), site.macro_path.clone());
            !self.item_macro_providers.contains_key(&key)
                && !self.inert_item_macros.contains_key(&key)
        }) {
            return Err(ReachabilityError::UnboundItemMacro {
                source: site.source.clone(),
                line: site.line,
                module: module_id.to_owned(),
                macro_path: site.macro_path.clone(),
            });
        }
        Ok(())
    }

    fn require_supported_syntax(
        &self,
        module_id: &str,
        module: &Module,
    ) -> Result<(), ReachabilityError> {
        if let Some(site) = module.unsupported_syntax.first() {
            return Err(ReachabilityError::UnsupportedReachableSyntax {
                source: site.source.clone(),
                line: site.line,
                module: module_id.to_owned(),
                syntax: site.syntax,
            });
        }
        Ok(())
    }

    fn require_associated_item_macro_bindings(&self, owner: &str) -> Result<(), ReachabilityError> {
        if let Some(site) = self.associated_item_macros.iter().find(|site| {
            site.owner == owner
                && !self
                    .associated_item_macro_providers
                    .contains_key(&(owner.to_owned(), site.site.macro_path.clone()))
        }) {
            return Err(ReachabilityError::UnboundAssociatedItemMacro {
                source: site.site.source.clone(),
                line: site.site.line,
                owner: owner.to_owned(),
                macro_path: site.site.macro_path.clone(),
            });
        }
        Ok(())
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
        if let Some((owner, _)) = resolved.rsplit_once("::")
            && let Some(module) = self.module(owner)
            && !module.excluded
        {
            self.require_audited_includes(owner, module)?;
            self.require_supported_syntax(owner, module)?;
            self.require_item_macro_bindings(owner, module)?;
        }
        if let Some((declaration, false)) = self.declaration(&resolved) {
            self.require_associated_item_macro_bindings(&resolved)?;
            for producer in &declaration.api_producers {
                self.require_api_producer_binding(producer)?;
            }
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
        index
            .associated_item_macros
            .extend(
                pending_impl
                    .item_macros
                    .iter()
                    .map(|site| AssociatedMacroSite {
                        owner: owner.clone(),
                        site: ItemMacroSite {
                            source: site.source.clone(),
                            line: site.line,
                            macro_path: site.macro_path.clone(),
                            tokens: site.tokens.clone(),
                        },
                    }),
            );
        if let Some(declaration) = index.declarations.get_mut(&owner) {
            declaration.members.extend(pending_impl.members);
            declaration
                .member_definitions
                .extend(pending_impl.member_definitions);
            declaration
                .api_producers
                .extend(pending_impl.api_producers.into_iter().map(|mut producer| {
                    producer.owner.clone_from(&owner);
                    producer
                }));
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
        .excluded |= excluded_parent || module_id.ends_with("::__private");
    for item in items {
        let attrs = super::item_attrs(item);
        if !cfg_enabled(attrs, cfg, source_file)? {
            continue;
        }
        let excluded = excluded_parent || module_id.ends_with("::__private");
        if !excluded
            && let syn::Item::Macro(value) = item
            && let Some(name) = &value.ident
        {
            let macro_name = ident_name(name);
            let parsed_module = index.modules.entry(module_id.to_owned()).or_default();
            if parsed_module
                .macro_rules
                .insert(macro_name.clone(), value.mac.tokens.clone())
                .is_some()
            {
                return Err(ReachabilityError::Parse(format!(
                    "{}:{}: duplicate or shadowed macro_rules definition `{macro_name}` in `{module_id}` cannot be resolved lexically",
                    source_file.display(),
                    value.span().start().line
                )));
            }
            let owner = format!("{module_id}::{}", ident_name(name));
            if has_attr(&value.attrs, "macro_export", cfg, source_file)? {
                index
                    .transcriber_producers
                    .extend(api_producers_from_macro_transcriber(
                        &value.mac.tokens,
                        source_file,
                        &owner,
                    )?);
            }
        }
        match item {
            syn::Item::Macro(value) if value.mac.path.is_ident("include") => {
                if let Ok(path) = syn::parse2::<syn::LitStr>(value.mac.tokens.clone()) {
                    let included = source_file
                        .parent()
                        .expect("a Rust source file has a parent directory")
                        .join(path.value());
                    parse_module_file(crate_name, &included, module_id, excluded, cfg, index)?;
                } else {
                    index
                        .modules
                        .entry(module_id.to_owned())
                        .or_default()
                        .dynamic_includes
                        .push(IncludeSite {
                            source: source_file.to_owned(),
                            line: value.span().start().line,
                            expression: value.mac.tokens.to_string(),
                        });
                }
            }
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
                    identity.clone(),
                    Declaration {
                        kind,
                        members: Vec::new(),
                        excluded,
                        alias_target: None,
                        definition: source_definition(source_file, value.span().start()),
                        member_definitions: BTreeMap::new(),
                        api_producers: api_producers_from_attrs(
                            &value.attrs,
                            cfg,
                            source_file,
                            &identity,
                            None,
                        )?,
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
                    identity.clone(),
                    Declaration {
                        kind: ReachableKind::Macro,
                        members: Vec::new(),
                        excluded,
                        alias_target: None,
                        definition: source_definition(source_file, value.span().start()),
                        member_definitions: BTreeMap::new(),
                        api_producers: api_producers_from_attrs(
                            &value.attrs,
                            cfg,
                            source_file,
                            &identity,
                            Some(item),
                        )?,
                    },
                );
            }
            syn::Item::Macro(value) if value.ident.is_none() => {
                let macro_path = path_segments(&value.mac.path).join("::");
                index
                    .modules
                    .entry(module_id.to_owned())
                    .or_default()
                    .item_macros
                    .push(ItemMacroSite {
                        source: source_file.to_owned(),
                        line: value.span().start().line,
                        macro_path,
                        tokens: value.mac.tokens.clone(),
                    });
            }
            syn::Item::ForeignMod(value) if !excluded => {
                index
                    .modules
                    .entry(module_id.to_owned())
                    .or_default()
                    .unsupported_syntax
                    .push(UnsupportedSyntaxSite {
                        source: source_file.to_owned(),
                        line: value.span().start().line,
                        syntax: "extern block",
                    });
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
                    .entry(module_id.to_owned())
                    .or_default()
                    .api_producers
                    .extend(api_producers_from_attrs(
                        &value.attrs,
                        cfg,
                        source_file,
                        &child_id,
                        Some(item),
                    )?);
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
                let owner = format!("{module_id}::<use@{}>", value.span().start().line);
                index
                    .modules
                    .entry(module_id.to_owned())
                    .or_default()
                    .api_producers
                    .extend(api_producers_from_attrs(
                        &value.attrs,
                        cfg,
                        source_file,
                        &owner,
                        Some(item),
                    )?);
                flatten_use(
                    Vec::new(),
                    &value.tree,
                    source_file,
                    value.span().start().line,
                    super::is_public(&value.vis),
                    &mut index.modules.entry(module_id.to_owned()).or_default().uses,
                );
            }
            syn::Item::Impl(value) => {
                let mut members = Vec::new();
                let mut member_definitions = BTreeMap::new();
                let mut item_macros = Vec::new();
                let provisional_owner = format!("{module_id}::<impl>");
                let mut api_producers = api_producers_from_attrs(
                    &value.attrs,
                    cfg,
                    source_file,
                    &provisional_owner,
                    None,
                )?;
                for child in &value.items {
                    let child_attrs = impl_item_attrs(child);
                    if cfg_enabled(child_attrs, cfg, source_file)? {
                        api_producers.extend(api_producers_from_attrs(
                            child_attrs,
                            cfg,
                            source_file,
                            &provisional_owner,
                            None,
                        )?);
                        if let syn::ImplItem::Macro(item) = child {
                            item_macros.push(ItemMacroSite {
                                source: source_file.to_owned(),
                                line: item.span().start().line,
                                macro_path: path_segments(&item.mac.path).join("::"),
                                tokens: item.mac.tokens.clone(),
                            });
                        }
                    }
                    let member = if value.trait_.is_some() {
                        None
                    } else {
                        match child {
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
                        }
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
                    api_producers,
                    excluded,
                    item_macros,
                });
            }
            _ => {
                if let Some((name, kind, members)) = declaration_parts(item, cfg, source_file)? {
                    let identity = format!("{module_id}::{name}");
                    let mut api_producers =
                        api_producers_from_attrs(attrs, cfg, source_file, &identity, Some(item))?;
                    api_producers.extend(api_producers_from_nested_declaration_attrs(
                        item,
                        attrs,
                        cfg,
                        source_file,
                        &identity,
                    )?);
                    if let syn::Item::Trait(trait_item) = item {
                        for member in &trait_item.items {
                            let member_attrs = trait_item_attrs(member);
                            if cfg_enabled(member_attrs, cfg, source_file)? {
                                api_producers.extend(api_producers_from_attrs(
                                    member_attrs,
                                    cfg,
                                    source_file,
                                    &identity,
                                    None,
                                )?);
                                if let syn::TraitItem::Macro(item) = member {
                                    index.associated_item_macros.push(AssociatedMacroSite {
                                        owner: identity.clone(),
                                        site: ItemMacroSite {
                                            source: source_file.to_owned(),
                                            line: item.span().start().line,
                                            macro_path: path_segments(&item.mac.path).join("::"),
                                            tokens: item.mac.tokens.clone(),
                                        },
                                    });
                                }
                            }
                        }
                    }
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
                                api_producers,
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

fn api_producers_from_nested_declaration_attrs(
    item: &syn::Item,
    parent_attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
    owner: &str,
) -> Result<Vec<ApiProducerUse>, ReachabilityError> {
    let context = helper_attribute_context(parent_attrs, cfg, source)?;
    let mut producers = Vec::new();
    let mut inspect = |attrs: &[syn::Attribute], nested_owner: String, target| {
        if !cfg_enabled(attrs, cfg, source)? {
            return Ok(());
        }
        producers.extend(api_producers_from_attrs_with_context(
            attrs,
            cfg,
            source,
            &nested_owner,
            None,
            &context,
            target,
        )?);
        Ok::<_, ReachabilityError>(())
    };
    match item {
        syn::Item::Struct(value) => {
            for (index, field) in value.fields.iter().enumerate() {
                let name = field
                    .ident
                    .as_ref()
                    .map_or_else(|| index.to_string(), ident_name);
                inspect(
                    &field.attrs,
                    format!("{owner}::{name}"),
                    AttributeTarget::Field,
                )?;
            }
        }
        syn::Item::Union(value) => {
            for field in &value.fields.named {
                let name = ident_name(field.ident.as_ref().expect("union field is named"));
                inspect(
                    &field.attrs,
                    format!("{owner}::{name}"),
                    AttributeTarget::Field,
                )?;
            }
        }
        syn::Item::Enum(value) => {
            for variant in &value.variants {
                let variant_name = ident_name(&variant.ident);
                inspect(
                    &variant.attrs,
                    format!("{owner}::{variant_name}"),
                    AttributeTarget::Variant,
                )?;
                for (index, field) in variant.fields.iter().enumerate() {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map_or_else(|| index.to_string(), ident_name);
                    inspect(
                        &field.attrs,
                        format!("{owner}::{variant_name}::{field_name}"),
                        AttributeTarget::Field,
                    )?;
                }
            }
        }
        _ => {}
    }
    Ok(producers)
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

fn api_producers_from_attrs(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
    owner: &str,
    item: Option<&syn::Item>,
) -> Result<Vec<ApiProducerUse>, ReachabilityError> {
    let helper_context = helper_attribute_context(attrs, cfg, source)?;
    api_producers_from_attrs_with_context(
        attrs,
        cfg,
        source,
        owner,
        item,
        &helper_context,
        AttributeTarget::Declaration,
    )
}

#[derive(Clone, Copy)]
enum AttributeTarget {
    Declaration,
    Variant,
    Field,
}

#[derive(Default)]
struct HelperAttributeContext {
    derives: BTreeSet<String>,
}

fn helper_attribute_context(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
) -> Result<HelperAttributeContext, ReachabilityError> {
    let mut context = HelperAttributeContext::default();
    for attr in effective_attributes(attrs, cfg, source)? {
        if attr.meta.path().is_ident("derive") {
            let syn::Meta::List(list) = attr.meta else {
                continue;
            };
            let derives = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .map_err(|error| {
                    ReachabilityError::Parse(format!(
                        "{}:{}: malformed derive attribute: {error}",
                        source.display(),
                        attr.line
                    ))
                })?;
            for derive in derives {
                let Some(name) = derive
                    .segments
                    .last()
                    .map(|segment| ident_name(&segment.ident))
                else {
                    continue;
                };
                if derive.segments.len() == 1 || trusted_qualified_derive(&derive) {
                    context.derives.insert(name);
                }
            }
        }
    }
    Ok(context)
}

fn api_producers_from_attrs_with_context(
    attrs: &[syn::Attribute],
    cfg: &CfgSet,
    source: &Path,
    owner: &str,
    item: Option<&syn::Item>,
    helper_context: &HelperAttributeContext,
    target: AttributeTarget,
) -> Result<Vec<ApiProducerUse>, ReachabilityError> {
    let mut producers = Vec::new();
    for attr in effective_attributes(attrs, cfg, source)? {
        if attr.meta.path().is_ident("derive") {
            let syn::Meta::List(list) = &attr.meta else {
                continue;
            };
            let derives = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .map_err(|error| {
                    ReachabilityError::Parse(format!(
                        "{}:{}: malformed derive attribute: {error}",
                        source.display(),
                        attr.line
                    ))
                })?;
            for derive in derives {
                let Some(name) = derive
                    .segments
                    .last()
                    .map(|segment| segment.ident.to_string())
                else {
                    continue;
                };
                let bare_name = derive.segments.len() == 1;
                let qualified_root = (!bare_name).then(|| {
                    derive
                        .segments
                        .first()
                        .expect("a qualified derive has a first segment")
                        .ident
                        .to_string()
                });
                let trusted = bare_name || trusted_qualified_derive(&derive);
                if !trusted || (!api_producing_sand_macro(&name) && !trait_only_derive(&name)) {
                    producers.push(ApiProducerUse {
                        source: source.to_owned(),
                        line: attr.line,
                        owner: owner.to_owned(),
                        name,
                        expected_generated: None,
                        unclassified_form: Some("derive"),
                        requires_binding: false,
                        bare_name,
                        qualified_root,
                    });
                } else if api_producing_sand_macro(&name) {
                    producers.push(ApiProducerUse {
                        expected_generated: expected_generated_identities(&name, owner, item)?,
                        name,
                        source: source.to_owned(),
                        line: attr.line,
                        owner: owner.to_owned(),
                        unclassified_form: None,
                        requires_binding: true,
                        bare_name,
                        qualified_root,
                    });
                } else {
                    // A bare built-in or audited trait-only derive can still
                    // be shadowed by a macro import. Defer that namespace
                    // check until this declaration is proven reachable.
                    producers.push(ApiProducerUse {
                        source: source.to_owned(),
                        line: attr.line,
                        owner: owner.to_owned(),
                        name,
                        expected_generated: None,
                        unclassified_form: None,
                        requires_binding: false,
                        bare_name,
                        qualified_root,
                    });
                }
            }
        } else if let Some(name) = attr
            .meta
            .path()
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        {
            let path = attr.meta.path();
            let bare_name = path.segments.len() == 1;
            let qualified_root = (!bare_name).then(|| {
                path.segments
                    .first()
                    .expect("a qualified attribute has a first segment")
                    .ident
                    .to_string()
            });
            let trusted = bare_name || trusted_qualified_attribute(path);
            let helper = bare_name && valid_derive_helper(&name, helper_context, target);
            let inert = builtin_or_inert_attribute(&name)
                || matches!(path_segments(path).as_slice(), [tool, attribute]
                    if tool == "diagnostic" && attribute == "on_unimplemented");
            if !trusted
                || (!api_producing_sand_macro(&name)
                    && !inert
                    && !shape_preserving_sand_attribute(&name)
                    && !helper)
            {
                producers.push(ApiProducerUse {
                    source: source.to_owned(),
                    line: attr.line,
                    owner: owner.to_owned(),
                    name,
                    expected_generated: None,
                    unclassified_form: Some("attribute"),
                    requires_binding: false,
                    bare_name,
                    qualified_root,
                });
            } else if api_producing_sand_macro(&name) {
                producers.push(ApiProducerUse {
                    expected_generated: expected_generated_identities(&name, owner, item)?,
                    name,
                    source: source.to_owned(),
                    line: attr.line,
                    owner: owner.to_owned(),
                    unclassified_form: None,
                    requires_binding: true,
                    bare_name,
                    qualified_root,
                });
            } else if helper {
                // Derive helpers are inert only because the matching derive
                // declares them. Keep a reachable occurrence so an imported
                // proc macro cannot impersonate that helper name.
                producers.push(ApiProducerUse {
                    source: source.to_owned(),
                    line: attr.line,
                    owner: owner.to_owned(),
                    name,
                    expected_generated: None,
                    unclassified_form: None,
                    requires_binding: false,
                    bare_name: true,
                    qualified_root: None,
                });
            } else if shape_preserving_sand_attribute(&name) {
                producers.push(ApiProducerUse {
                    source: source.to_owned(),
                    line: attr.line,
                    owner: owner.to_owned(),
                    name,
                    expected_generated: None,
                    unclassified_form: None,
                    requires_binding: false,
                    bare_name,
                    qualified_root,
                });
            }
        }
    }
    Ok(producers)
}

fn valid_derive_helper(
    name: &str,
    context: &HelperAttributeContext,
    target: AttributeTarget,
) -> bool {
    let has = |derive: &str| context.derives.contains(derive);
    match name {
        "serde" => has("Serialize") || has("Deserialize"),
        "error" => {
            has("Error")
                && matches!(
                    target,
                    AttributeTarget::Declaration | AttributeTarget::Variant
                )
        }
        "from" | "source" | "backtrace" => has("Error") && matches!(target, AttributeTarget::Field),
        "command" => {
            (has("Args") || has("Parser") || has("Subcommand"))
                && matches!(
                    target,
                    AttributeTarget::Declaration | AttributeTarget::Variant
                )
        }
        "arg" => {
            (has("Args") || has("Parser") || has("Subcommand"))
                && matches!(target, AttributeTarget::Field)
        }
        "group" => {
            (has("Args") || has("Parser"))
                && matches!(
                    target,
                    AttributeTarget::Declaration | AttributeTarget::Field
                )
        }
        "value" => has("ValueEnum") && matches!(target, AttributeTarget::Variant),
        "state" => has("State") && matches!(target, AttributeTarget::Field),
        "sand" => {
            has("SandStorage")
                && matches!(
                    target,
                    AttributeTarget::Declaration | AttributeTarget::Field
                )
        }
        _ => false,
    }
}

// Only macros which introduce additional inherent or sibling public items
// belong here. Trait-only derives (including EntityStateEnum) deliberately do
// not participate, nor do attributes which preserve the annotated item's
// public shape.
fn api_producing_sand_macro(name: &str) -> bool {
    matches!(name, "SandStorage" | "State" | "item")
}

fn expected_generated_identities(
    producer: &str,
    owner: &str,
    item: Option<&syn::Item>,
) -> Result<Option<BTreeSet<(String, ReachableKind)>>, ReachabilityError> {
    if producer != "SandStorage" {
        // State and item emit sibling APIs whose names require their macro's
        // shared semantic parser. Until those providers expose exact claims,
        // the occurrence remains deliberately unbindable and fails closed.
        return Ok(None);
    }
    let Some(item) = item else {
        // A macro_rules! definition is a template, not a concrete derive
        // occurrence. Consumer-side expansion providers must bind actual
        // invocation owners instead of fabricating one here.
        return Ok(None);
    };
    let input = syn::parse2::<syn::DeriveInput>(item.to_token_stream()).map_err(|error| {
        ReachabilityError::Parse(format!(
            "cannot model SandStorage output for `{owner}`: {error}"
        ))
    })?;
    let names = sand_api_contract::syntax::sand_storage_generated_member_names(&input).map_err(
        |error| {
            ReachabilityError::Parse(format!(
                "cannot model SandStorage output for `{owner}`: {error}"
            ))
        },
    )?;
    Ok(Some(
        names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                (
                    format!("{owner}::{name}"),
                    if index == 0 {
                        ReachableKind::AssociatedConst
                    } else {
                        ReachableKind::Method
                    },
                )
            })
            .collect(),
    ))
}

// These derives are known to implement traits only. Adding a derive here is a
// deliberate assertion about its expansion shape, not a general exemption for
// macros from a particular crate.
fn trait_only_derive(name: &str) -> bool {
    matches!(
        name,
        "Args"
            | "Clone"
            | "Copy"
            | "Debug"
            | "Default"
            | "Deserialize"
            | "EntityStateEnum"
            | "Eq"
            | "Error"
            | "Hash"
            | "Ord"
            | "Parser"
            | "PartialEq"
            | "PartialOrd"
            | "Serialize"
            | "Subcommand"
            | "ValueEnum"
    )
}

fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn trusted_qualified_derive(path: &syn::Path) -> bool {
    matches!(
        path_segments(path).as_slice(),
        [crate_name, name]
            if matches!(
                (crate_name.as_str(), name.as_str()),
                ("serde", "Serialize" | "Deserialize")
                    | ("thiserror", "Error")
                    | ("clap", "Args" | "Parser" | "Subcommand" | "ValueEnum")
                    | ("sand", "EntityStateEnum" | "SandStorage" | "State")
                    | ("sand_macros", "EntityStateEnum" | "SandStorage" | "State")
            )
    )
}

fn trusted_qualified_attribute(path: &syn::Path) -> bool {
    matches!(
        path_segments(path).as_slice(),
        [crate_name, name]
            if matches!(
                (crate_name.as_str(), name.as_str()),
                ("diagnostic", "on_unimplemented")
                    | ("sand", "api" | "armor_event" | "component" | "entity_archetype" | "event" | "function" | "item" | "schedule")
                    | ("sand_macros", "api" | "armor_event" | "component" | "entity_archetype" | "event" | "function" | "item" | "schedule")
            )
    )
}

fn shape_preserving_sand_attribute(name: &str) -> bool {
    matches!(
        name,
        "api"
            | "armor_event"
            | "component"
            | "entity_archetype"
            | "event"
            | "function"
            | "item"
            | "schedule"
    )
}

fn trusted_macro_import(name: &str, path: &[String]) -> bool {
    matches!(
        path,
        [crate_name, imported]
            if imported == name
                && matches!(
                    (crate_name.as_str(), name),
                    ("serde", "Serialize" | "Deserialize")
                        | ("thiserror", "Error")
                        | ("clap", "Args" | "Parser" | "Subcommand" | "ValueEnum")
                        | ("sand", "EntityStateEnum" | "SandStorage" | "State" | "api" | "armor_event" | "component" | "entity_archetype" | "event" | "function" | "item" | "schedule")
                        | ("sand_macros", "EntityStateEnum" | "SandStorage" | "State" | "api" | "armor_event" | "component" | "entity_archetype" | "event" | "function" | "item" | "schedule")
                )
    ) || matches!(
        path,
        [std, fmt, imported]
            if std == "std" && fmt == "fmt" && imported == name && name == "Debug"
    )
}

// Language/compiler attributes are inert without relying on a proc-macro
// namespace lookup. Derive helper attributes deliberately do not belong here:
// they are accepted by `valid_derive_helper` only on a form whose parent has
// the derive that declares that helper.
fn builtin_or_inert_attribute(name: &str) -> bool {
    matches!(
        name,
        "allow"
            | "automatically_derived"
            | "cfg"
            | "cold"
            | "default"
            | "deny"
            | "deprecated"
            | "doc"
            | "export_name"
            | "forbid"
            | "inline"
            | "link"
            | "link_name"
            | "link_section"
            | "macro_export"
            | "must_use"
            | "no_mangle"
            | "non_exhaustive"
            | "path"
            | "proc_macro"
            | "proc_macro_attribute"
            | "proc_macro_derive"
            | "repr"
            | "should_panic"
            | "target_feature"
            | "test"
            | "track_caller"
            | "used"
            | "warn"
    )
}

fn api_producers_from_macro_transcriber(
    tokens: &proc_macro2::TokenStream,
    source: &Path,
    owner: &str,
) -> Result<Vec<ApiProducerUse>, ReachabilityError> {
    fn visit(
        tokens: proc_macro2::TokenStream,
        source: &Path,
        owner: &str,
        output: &mut Vec<ApiProducerUse>,
    ) -> Result<(), ReachabilityError> {
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            let proc_macro2::TokenTree::Punct(hash) = token else {
                continue;
            };
            let Some(proc_macro2::TokenTree::Group(attribute)) = tokens.get(index + 1) else {
                continue;
            };
            if hash.as_char() != '#' || attribute.delimiter() != proc_macro2::Delimiter::Bracket {
                continue;
            }
            let attribute_tokens = attribute.stream().into_iter().collect::<Vec<_>>();
            let Some(proc_macro2::TokenTree::Ident(first)) = attribute_tokens.first() else {
                continue;
            };
            if first == "derive" {
                let Some(proc_macro2::TokenTree::Group(arguments)) = attribute_tokens.get(1) else {
                    continue;
                };
                for candidate in arguments.stream() {
                    let proc_macro2::TokenTree::Ident(candidate) = candidate else {
                        continue;
                    };
                    let name = candidate.to_string();
                    if api_producing_sand_macro(&name) {
                        output.push(ApiProducerUse {
                            name,
                            source: source.to_owned(),
                            line: hash.span().start().line,
                            owner: owner.to_owned(),
                            expected_generated: None,
                            unclassified_form: None,
                            requires_binding: true,
                            bare_name: true,
                            qualified_root: None,
                        });
                    } else if !trait_only_derive(&name) {
                        output.push(ApiProducerUse {
                            source: source.to_owned(),
                            line: hash.span().start().line,
                            owner: owner.to_owned(),
                            name,
                            expected_generated: None,
                            unclassified_form: Some("derive in exported macro transcriber"),
                            requires_binding: false,
                            bare_name: true,
                            qualified_root: None,
                        });
                    }
                }
            } else {
                let name = attribute_tokens
                    .iter()
                    .rev()
                    .find_map(|token| match token {
                        proc_macro2::TokenTree::Ident(ident) => Some(ident.to_string()),
                        proc_macro2::TokenTree::Group(_) => None,
                        _ => None,
                    })
                    .unwrap_or_else(|| first.to_string());
                if api_producing_sand_macro(&name) {
                    output.push(ApiProducerUse {
                        name,
                        source: source.to_owned(),
                        line: hash.span().start().line,
                        owner: owner.to_owned(),
                        expected_generated: None,
                        unclassified_form: None,
                        requires_binding: true,
                        bare_name: true,
                        qualified_root: None,
                    });
                } else if !builtin_or_inert_attribute(&name) {
                    output.push(ApiProducerUse {
                        source: source.to_owned(),
                        line: hash.span().start().line,
                        owner: owner.to_owned(),
                        name,
                        expected_generated: None,
                        unclassified_form: Some("attribute in exported macro transcriber"),
                        requires_binding: false,
                        bare_name: true,
                        qualified_root: None,
                    });
                }
            }
        }
        for token in tokens {
            if let proc_macro2::TokenTree::Group(group) = token {
                visit(group.stream(), source, owner, output)?;
            }
        }
        Ok(())
    }

    let mut producers = Vec::new();
    visit(tokens.clone(), source, owner, &mut producers)?;
    producers.sort_by(|left, right| {
        (&left.owner, &left.name, left.line).cmp(&(&right.owner, &right.name, right.line))
    });
    producers.dedup_by(|left, right| {
        left.owner == right.owner && left.name == right.name && left.line == right.line
    });
    Ok(producers)
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

fn impl_item_attrs(item: &syn::ImplItem) -> &[syn::Attribute] {
    match item {
        syn::ImplItem::Const(item) => &item.attrs,
        syn::ImplItem::Fn(item) => &item.attrs,
        syn::ImplItem::Type(item) => &item.attrs,
        syn::ImplItem::Macro(item) => &item.attrs,
        syn::ImplItem::Verbatim(_) | _ => &[],
    }
}

fn trait_item_attrs(item: &syn::TraitItem) -> &[syn::Attribute] {
    match item {
        syn::TraitItem::Const(item) => &item.attrs,
        syn::TraitItem::Fn(item) => &item.attrs,
        syn::TraitItem::Type(item) => &item.attrs,
        syn::TraitItem::Macro(item) => &item.attrs,
        _ => &[],
    }
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
