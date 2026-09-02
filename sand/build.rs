use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_contract::ApiKind;
use sand_api_enforce::{
    CfgSet, ContractIdentity, GeneratedApi, InertItemMacroClassification, ReachableKind,
    ScopeManifest, ScopeState, SurfaceGraph, SurfaceProfileManifest,
    contract_declarations_from_files, discover_facade_feature_union, discover_local_source_crates,
    event_generated_type_provider, registry_id_provider, resolve_contract_identities,
    validate_contract_lookup_namespace, vanilla_registry_enum_provider,
};

const PLACEHOLDER_SURFACE_PROFILE: &str = "placeholder-codegen";

/// Opt-in internal phase profiling for this build script (issue #347 Part
/// 3, "determine the real dependency graph first"). Set
/// `SAND_BUILD_RS_PROFILE=1` to print a phase-by-phase timing breakdown of
/// this build script's own work to stderr. Disabled by default so ordinary
/// builds see no output change; the timing calls themselves
/// (`Instant::now()`) are negligible either way.
struct Profiler {
    enabled: bool,
    start: std::time::Instant,
    last: std::time::Instant,
}

impl Profiler {
    fn new() -> Self {
        let enabled = env::var_os("SAND_BUILD_RS_PROFILE").is_some();
        let now = std::time::Instant::now();
        Self {
            enabled,
            start: now,
            last: now,
        }
    }

    /// Marks the end of a phase, printing its duration since the previous
    /// mark (or since `new()` for the first phase).
    fn mark(&mut self, phase: &str) {
        if !self.enabled {
            return;
        }
        let now = std::time::Instant::now();
        eprintln!(
            "cargo:warning=sand/build.rs profile: {phase} took {:?}",
            now.duration_since(self.last)
        );
        self.last = now;
    }

    fn total(&self) {
        if !self.enabled {
            return;
        }
        eprintln!(
            "cargo:warning=sand/build.rs profile: TOTAL {:?}",
            self.start.elapsed()
        );
    }
}

fn main() {
    let mut profiler = Profiler::new();
    let workspace = Path::new("..");
    for source in [
        "src",
        "../sand-core/src",
        "../sand-commands/src",
        "../sand-components/src",
        "../sand-macros/src",
        "../sand-resourcepack/src",
        "../sand-version/src",
        "api-scopes.toml",
        "api-surface-profiles.toml",
        "api-surface-baseline.txt",
        "api-surface-baseline-1.21.4.txt",
        "api-surface-baseline-placeholder.txt",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-env-changed=DEP_SAND_CORE_API_PROVIDER_DIR");

    let mut manifest = ScopeManifest::from_path("api-scopes.toml")
        .unwrap_or_else(|error| panic!("invalid Sand API scope manifest: {error}"));
    let profiles = SurfaceProfileManifest::from_path(Path::new("api-surface-profiles.toml"))
        .unwrap_or_else(|error| panic!("invalid Sand API surface profiles: {error}"));
    let provider_dir = PathBuf::from(
        env::var_os("DEP_SAND_CORE_API_PROVIDER_DIR").unwrap_or_else(|| {
            panic!(
                "sand-core did not expose generated API providers; its build must emit api_provider_dir metadata"
            )
        }),
    );
    let providers = generated_providers(&provider_dir)
        .unwrap_or_else(|error| panic!("invalid generated API provider: {error}"));
    profiler.mark("load scope/profile manifests + generated providers");
    println!("cargo:rustc-check-cfg=cfg(sand_placeholder_codegen)");
    if providers.placeholder {
        println!("cargo:rustc-cfg=sand_placeholder_codegen");
    }
    let profile = profiles
        .select(&providers.versions)
        .unwrap_or_else(|error| panic!("cannot select Sand API surface profile: {error}"));
    let placeholder_codegen = providers.placeholder;
    let minecraft_version = providers.minecraft_version.clone();
    manifest.static_surface_items = profile.static_surface_items;
    manifest.pending_item_ceiling = profile.pending_item_ceiling;
    let mut generated = providers.apis;
    let generated_contracts = providers.contracts;
    generated.extend(
        registry_id_provider(&workspace.join("sand-components/src/registry.rs"))
            .unwrap_or_else(|error| panic!("invalid registry_id! API provider: {error}")),
    );
    generated.extend(
        vanilla_registry_enum_provider(&workspace.join("sand-components/src/effect.rs"))
            .unwrap_or_else(|error| panic!("invalid vanilla_registry_enum! API provider: {error}")),
    );
    generated.extend(
        event_generated_type_provider(&workspace.join("sand-core/src/events/mod.rs"))
            .unwrap_or_else(|error| panic!("invalid generated event marker provider: {error}")),
    );
    profiler.mark("parse item-macro generated providers (registry/effect/event)");

    // The ratchet is an all-supported-features baseline on every build. Using
    // only the current Cargo selection would leave enough global headroom for
    // a newly added feature-gated API to escape a default `cargo check`.
    let enabled_features = discover_facade_feature_union(Path::new("Cargo.toml"))
        .unwrap_or_else(|error| panic!("failed to discover Sand facade features: {error}"));
    let source_crates = discover_local_source_crates(Path::new("Cargo.toml"))
        .unwrap_or_else(|error| panic!("failed to discover local Sand source crates: {error}"));
    for source_crate in &source_crates {
        println!("cargo:rerun-if-changed={}", source_crate.root.display());
    }
    profiler.mark("discover facade features + local source crates");
    let generated_for_installed = generated.clone();
    let build_graph = |features: BTreeSet<String>, generated: Vec<GeneratedApi>| {
        SurfaceGraph::load_with_cfg(
            source_crates.clone(),
            cargo_cfg(features, placeholder_codegen),
            generated,
        )
        .and_then(|graph| {
            graph.bind_item_macro_provider(
                "sand_components::registry",
                "registry_id",
                "generated_registry_ids",
            )
        })
        .and_then(|graph| {
            graph.bind_item_macro_provider(
                "sand_components::effect",
                "vanilla_registry_enum",
                "generated_effect_registry_enums",
            )
        })
        .and_then(|graph| {
            graph.bind_item_macro_provider(
                "sand_core::events",
                "gamemode_transition",
                "generated_event_markers",
            )
        })
        .and_then(|graph| {
            graph.bind_item_macro_provider(
                "sand_core::events",
                "status_effect_marker",
                "generated_event_markers",
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_commands::nbt",
                "nbt_from",
                InertItemMacroClassification::LocalTraitImplOnly,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_components::tag",
                "tag_registry",
                InertItemMacroClassification::LocalTraitImplOnly,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_core::events",
                "adv_event",
                InertItemMacroClassification::LocalTraitImplOnly,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_commands::export_registry",
                "thread_local",
                InertItemMacroClassification::ThreadLocalStorageWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_core::function",
                "thread_local",
                InertItemMacroClassification::ThreadLocalStorageWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_core::function",
                "inventory::collect",
                InertItemMacroClassification::InventoryCollectionWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_core::state::registry",
                "inventory::collect",
                InertItemMacroClassification::InventoryCollectionWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_core::entity::archetype",
                "inventory::collect",
                InertItemMacroClassification::InventoryCollectionWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_components::dialog",
                "inventory::collect",
                InertItemMacroClassification::InventoryCollectionWiring,
            )
        })
        .and_then(|graph| {
            graph.bind_inert_item_macro(
                "sand_resourcepack::descriptor",
                "inventory::collect",
                InertItemMacroClassification::InventoryCollectionWiring,
            )
        })
        .and_then(|graph| {
            if placeholder_codegen {
                graph.bind_placeholder_generated_include(
                    "sand_core::generated",
                    "generated_registries",
                )
            } else {
                graph.bind_generated_include("sand_core::generated", "generated_registries")
            }
        })
        .and_then(|graph| {
            if placeholder_codegen {
                graph.bind_placeholder_generated_include(
                    "sand_core::cmd::_generated",
                    "generated_commands",
                )
            } else {
                graph.bind_generated_include("sand_core::cmd::_generated", "generated_commands")
            }
        })
        .unwrap_or_else(|error| panic!("failed to construct Sand public facade graph: {error}"))
    };
    // Split so --explain-rebuild-style diagnostics (SAND_BUILD_RS_PROFILE=1)
    // can distinguish "parse + cfg-evaluate + classify + bind macro
    // providers" (build_graph, dominated by SurfaceGraph::load_with_cfg's
    // semantic walk -- see issue #349) from "walk the already-built graph
    // for facade-reachable items" (reachable_from). Per issue #349's
    // profiler-expansion request: this is a coarse two-way split at the
    // sand/build.rs call-site level, not instrumentation inside
    // SurfaceGraph itself (a ~4300-line file in sand-api-enforce) -- finer
    // internal phase timing (file discovery, AST parse, cfg evaluation,
    // item classification, macro-provider binding as separate numbers) is
    // deferred to whoever picks up the deeper per-crate-manifest redesign,
    // since adding instrumentation to reachable.rs's internals is
    // meaningfully more invasive and this split already answers the
    // question that matters here: parsing+classification+binding
    // (build_graph) vs. the reachability walk (reachable_from) as separate
    // costs.
    let graph = build_graph(enabled_features.clone(), generated);
    profiler
        .mark("build surface graph (all-supported-features ratchet): parse+cfg-eval+classify+bind");
    let reachable = graph
        .reachable_from("sand")
        .unwrap_or_else(|error| panic!("failed to extract Sand public facade: {error}"));
    profiler.mark("reachable_from(\"sand\") (all-supported-features ratchet): facade walk");
    let installed_features = enabled_cargo_features(&enabled_features);
    let installed_graph = build_graph(installed_features.clone(), generated_for_installed);
    profiler.mark("build surface graph (installed configuration): parse+cfg-eval+classify+bind");
    let installed_reachable = installed_graph
        .reachable_from("sand")
        .unwrap_or_else(|error| panic!("failed to extract installed Sand public facade: {error}"));
    profiler.mark("reachable_from(\"sand\") (installed configuration): facade walk");

    let source_declarations =
        contract_declarations_from_files(contract_source_files(&source_crates))
            .unwrap_or_else(|error| panic!("invalid build-time API contract source: {error}"));
    profiler.mark("parse API contract declarations from source files (second full-source pass)");
    let mut contracts = resolve_contract_identities(&reachable, &source_declarations)
        .unwrap_or_else(|errors| panic_errors("invalid source API contract identities", &errors));
    contracts.extend(validate_generated_contracts(
        &reachable,
        generated_contracts,
    ));
    contracts.extend(generated_provider_contracts(
        &reachable,
        "generated_effect_registry_enums",
    ));
    contracts.extend(generated_provider_contracts(
        &reachable,
        "generated_event_markers",
    ));
    reject_duplicate_contract_identities(&contracts);
    profiler.mark("resolve contract identities + validate generated providers + dedup check");

    // The command and vanilla-registry providers are structurally compared
    // with their emitted Rust above even when a selected Minecraft profile
    // contains no generated declarations (the explicit placeholder profile).
    // Connecting those audits prevents the empty profile from becoming a
    // vacuous enforcement claim.
    // Only static providers proven against their emitted source in this build
    // participate in the facade report. Parametric consumer expansions are
    // self-audited by their proc macros and do not pretend to be finite items
    // in Sand's installed static surface.
    let connected_provider_audits = BTreeSet::from([
        "generated-commands".to_owned(),
        "generated-vanilla-registries".to_owned(),
    ]);
    let report = manifest
        .evaluate_with_provider_audits(
            &reachable,
            &contracts,
            &enabled_features,
            &connected_provider_audits,
        )
        .unwrap_or_else(|errors| panic_errors("Sand API scope enforcement failed", &errors));
    if reachable.len() != manifest.static_surface_items {
        panic!(
            "Sand static public surface contains {} items, but api-scopes.toml records {}; classify the change and ratchet the committed baseline",
            reachable.len(),
            manifest.static_surface_items
        );
    }
    profiler.mark("evaluate scope manifest + provider audits + static surface count check");

    write_coverage(
        &manifest,
        &report,
        &reachable,
        &installed_reachable,
        &contracts,
        &source_declarations,
        InstalledConfiguration {
            features: &installed_features,
            profile,
            placeholder_codegen,
            minecraft_version: &minecraft_version,
        },
    );
    profiler.mark("write installed API metadata (facade registrations, coverage, surface report)");
    profiler.total();
}

fn enabled_cargo_features(supported: &BTreeSet<String>) -> BTreeSet<String> {
    supported
        .iter()
        .filter(|feature| feature.as_str() != "default")
        .filter(|feature| {
            let variable = format!(
                "CARGO_FEATURE_{}",
                feature.replace('-', "_").to_ascii_uppercase()
            );
            env::var_os(variable).is_some()
        })
        .cloned()
        .collect()
}

fn contract_source_files(source_crates: &[sand_api_enforce::SourceCrate]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for source_crate in source_crates {
        let directory = source_crate
            .root
            .parent()
            .expect("library source root has a parent directory");
        collect_rust_files(directory, &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| {
            entry.unwrap_or_else(|error| panic!("failed to read directory entry: {error}"))
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn cargo_cfg(features: BTreeSet<String>, placeholder_codegen: bool) -> CfgSet {
    let mut flags = BTreeMap::from([("test".to_owned(), false)]);
    flags.insert("sand_placeholder_codegen".to_owned(), placeholder_codegen);
    let mut key_values = BTreeMap::new();
    for (key, value) in env::vars_os() {
        let Some(key) = key.to_str().and_then(|key| key.strip_prefix("CARGO_CFG_")) else {
            continue;
        };
        let key = key.to_ascii_lowercase();
        let value = value.to_string_lossy();
        if value.is_empty() || matches!(value.as_ref(), "1" | "true") {
            flags.insert(key, true);
        } else {
            key_values.insert(
                key,
                value.split(',').map(str::to_owned).collect::<BTreeSet<_>>(),
            );
        }
    }
    CfgSet {
        features,
        flags,
        key_values,
    }
}

struct GeneratedProviders {
    apis: Vec<GeneratedApi>,
    contracts: Vec<ContractIdentity>,
    versions: BTreeMap<String, String>,
    placeholder: bool,
    minecraft_version: String,
}

fn generated_providers(directory: &Path) -> Result<GeneratedProviders, String> {
    let mut generated = Vec::new();
    let mut contracts = Vec::new();
    let mut provider_versions = BTreeMap::new();
    let mut placeholder_modes = BTreeSet::new();
    let mut minecraft_versions = BTreeSet::new();
    for (filename, rust_filename, root_identity, expected_provider) in [
        (
            "commands.api.json",
            "commands.rs",
            "sand_core::cmd::_generated",
            "generated_commands",
        ),
        (
            "registries.api.json",
            "registries.rs",
            "sand_core::generated",
            "generated_registries",
        ),
    ] {
        let path = directory.join(filename);
        let catalog = sand_build::read_api_provider(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if catalog.provider != expected_provider {
            return Err(format!(
                "{} declares provider `{}`, expected `{expected_provider}`",
                path.display(),
                catalog.provider
            ));
        }
        placeholder_modes.insert(catalog.placeholder);
        minecraft_versions.insert(catalog.minecraft_version.clone());
        let surface_profile = if catalog.placeholder {
            PLACEHOLDER_SURFACE_PROFILE.to_owned()
        } else {
            catalog.minecraft_version.clone()
        };
        if provider_versions
            .insert(catalog.provider.clone(), surface_profile)
            .is_some()
        {
            return Err(format!(
                "duplicate generated provider `{}`",
                catalog.provider
            ));
        }
        sand_build::validate_api_provider_source(
            &catalog,
            &directory.join(rust_filename),
            root_identity,
        )?;
        let mut parents = BTreeMap::<String, GeneratedApi>::new();
        for entry in &catalog.entries {
            contracts.push(ContractIdentity {
                identity: entry.definition_identity.clone(),
                canonical_path: entry.contract.canonical_path.clone(),
                aliases: entry.contract.aliases.iter().cloned().collect(),
            });
            if entry.parent_identity.is_none() {
                parents.insert(
                    entry.definition_identity.clone(),
                    GeneratedApi {
                        identity: entry.definition_identity.clone(),
                        provider: catalog.provider.clone(),
                        producer: None,
                        kind: reachable_kind(entry.definition_kind),
                        members: Vec::new(),
                        excluded: false,
                    },
                );
            }
        }
        for entry in &catalog.entries {
            if let (Some(parent), Some(member)) = (&entry.parent_identity, &entry.member_name) {
                let owner = parents.get_mut(parent).ok_or_else(|| {
                    format!(
                        "{}: generated member `{}` names missing parent `{parent}`",
                        path.display(),
                        entry.definition_identity
                    )
                })?;
                owner
                    .members
                    .push((member.clone(), reachable_kind(entry.definition_kind)));
            }
        }
        for parent in parents.values_mut() {
            parent.members.sort();
        }
        generated.extend(parents.into_values());
    }

    let registry_id_path = directory.join("registry_ids.api.json");
    let registry_id_catalog = sand_build::read_api_provider(&registry_id_path)
        .map_err(|error| format!("{}: {error}", registry_id_path.display()))?;
    if registry_id_catalog.provider != "generated_registry_id_contracts" {
        return Err(format!(
            "{} declares provider `{}`, expected `generated_registry_id_contracts`",
            registry_id_path.display(),
            registry_id_catalog.provider
        ));
    }
    if registry_id_catalog.placeholder {
        return Err(format!(
            "{} cannot be a placeholder catalog because registry-ID wrappers are generated from checked-in source",
            registry_id_path.display()
        ));
    }
    minecraft_versions.insert(registry_id_catalog.minecraft_version.clone());
    if placeholder_modes == BTreeSet::from([false])
        && provider_versions
            .values()
            .any(|version| version != &registry_id_catalog.minecraft_version)
    {
        return Err(format!(
            "{} targets Minecraft {}, which does not match the selected generated provider versions {:?}",
            registry_id_path.display(),
            registry_id_catalog.minecraft_version,
            provider_versions.values().collect::<BTreeSet<_>>()
        ));
    }
    for entry in registry_id_catalog.entries {
        contracts.push(ContractIdentity {
            identity: entry.definition_identity,
            canonical_path: entry.contract.canonical_path,
            aliases: entry.contract.aliases.into_iter().collect(),
        });
    }

    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    contracts.sort_by(|left, right| left.identity.cmp(&right.identity));
    if placeholder_modes.len() != 1 {
        return Err(
            "generated API providers mix placeholder and real codegen artifacts".to_owned(),
        );
    }
    if minecraft_versions.len() != 1 {
        return Err(format!(
            "generated API providers disagree on Minecraft version: {minecraft_versions:?}"
        ));
    }
    Ok(GeneratedProviders {
        apis: generated,
        contracts,
        versions: provider_versions,
        placeholder: placeholder_modes
            .into_iter()
            .next()
            .expect("two provider files were read"),
        minecraft_version: minecraft_versions
            .into_iter()
            .next()
            .expect("three provider files were read"),
    })
}

fn validate_generated_contracts(
    reachable: &[sand_api_enforce::ReachableApi],
    contracts: Vec<ContractIdentity>,
) -> Vec<ContractIdentity> {
    let reachable = reachable
        .iter()
        .map(|item| (item.identity.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    contracts
        .into_iter()
        .filter_map(|mut contract| {
            // A generated declaration shadowed by a handwritten item is Rust-
            // public inside the implementation module but is not reachable
            // through the facade glob, so it is outside the supported surface.
            let item = reachable.get(contract.identity.as_str())?;
            if !item.paths.contains(&contract.canonical_path) {
                panic!(
                    "generated contract `{}` selects unreachable path `{}`",
                    contract.identity, contract.canonical_path
                );
            }
            for alias in &contract.aliases {
                if !item.paths.contains(alias) {
                    panic!(
                        "generated contract `{}` selects unreachable alias `{alias}`",
                        contract.identity
                    );
                }
            }
            // Providers own semantic metadata, while the facade graph owns
            // discovery paths. Preserve the provider's canonical spelling but
            // derive every reachable alias here so a newly exported prelude or
            // topic alias cannot silently fall out of strict enforcement.
            contract.aliases = item
                .paths
                .iter()
                .filter(|path| *path != &contract.canonical_path)
                .cloned()
                .collect();
            Some(contract)
        })
        .collect()
}

/// A declaration-backed generator owns both the emitted public shape and its
/// contract registrations. Resolve that provider's contracts through the
/// facade graph here, so a new generated identity or re-export path fails the
/// ordinary build rather than relying on a hand-maintained path list.
fn generated_provider_contracts(
    reachable: &[sand_api_enforce::ReachableApi],
    provider: &str,
) -> Vec<ContractIdentity> {
    reachable
        .iter()
        .filter(|item| {
            matches!(
                &item.origin,
                sand_api_enforce::ReachableOrigin::Generator(origin) if origin == provider
            )
        })
        .map(|item| {
            let canonical_candidates = item
                .paths
                .iter()
                .filter(|path| path.starts_with("sand::") && !path.starts_with("sand::prelude::"))
                .collect::<Vec<_>>();
            let [canonical_path] = canonical_candidates.as_slice() else {
                panic!(
                    "generated provider `{provider}` identity `{}` must expose exactly one sand::* canonical path; found {:?}",
                    item.identity, item.paths
                );
            };
            ContractIdentity {
                identity: item.identity.clone(),
                canonical_path: (*canonical_path).clone(),
                aliases: item
                    .paths
                    .iter()
                    .filter(|path| *path != *canonical_path)
                    .cloned()
                    .collect(),
            }
        })
        .collect()
}

fn reachable_kind(kind: ApiKind) -> ReachableKind {
    match kind {
        ApiKind::Module => ReachableKind::Module,
        ApiKind::Struct => ReachableKind::Struct,
        ApiKind::Enum => ReachableKind::Enum,
        ApiKind::Variant => ReachableKind::Variant,
        ApiKind::Trait => ReachableKind::Trait,
        ApiKind::Function => ReachableKind::Function,
        ApiKind::Method => ReachableKind::Method,
        ApiKind::TraitMethod => ReachableKind::TraitMethod,
        ApiKind::TypeAlias => ReachableKind::TypeAlias,
        ApiKind::Constant => ReachableKind::Constant,
        ApiKind::AssociatedConst => ReachableKind::AssociatedConst,
        ApiKind::AssociatedType => ReachableKind::AssociatedType,
        ApiKind::Field => ReachableKind::Field,
        ApiKind::Macro => ReachableKind::Macro,
    }
}

fn reject_duplicate_contract_identities(contracts: &[ContractIdentity]) {
    let mut identities = BTreeSet::new();
    for contract in contracts {
        if !identities.insert(contract.identity.as_str()) {
            panic!(
                "multiple authoritative API contracts resolve to `{}`",
                contract.identity
            );
        }
    }
    validate_contract_lookup_namespace(contracts)
        .unwrap_or_else(|error| panic!("invalid authoritative API lookup namespace: {error}"));
}

fn panic_errors<T: ToString>(heading: &str, errors: &[T]) -> ! {
    panic!(
        "{heading}:\n{}",
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    )
}

struct InstalledConfiguration<'a> {
    features: &'a BTreeSet<String>,
    profile: &'a sand_api_enforce::SurfaceProfile,
    placeholder_codegen: bool,
    minecraft_version: &'a str,
}

fn write_coverage(
    manifest: &ScopeManifest,
    report: &sand_api_enforce::ScopeReport,
    reachable: &[sand_api_enforce::ReachableApi],
    installed_reachable: &[sand_api_enforce::ReachableApi],
    contracts: &[ContractIdentity],
    source_declarations: &[sand_api_enforce::ContractDeclaration],
    installed: InstalledConfiguration<'_>,
) {
    let mut pending = report
        .entries
        .iter()
        .filter(|scope| scope.state == ScopeState::Pending)
        .map(|scope| scope.id.as_str())
        .collect::<Vec<_>>();
    pending.sort_unstable();
    let status = if pending.is_empty() {
        "Complete"
    } else {
        "Partial"
    };
    let mut generated = String::new();
    writeln!(
        generated,
        "pub fn installed_coverage() -> ApiCoverage {{ ApiCoverage {{"
    )
    .unwrap();
    writeln!(generated, "status: CoverageStatus::{status},").unwrap();
    writeln!(generated, "static_surface_items: {},", reachable.len()).unwrap();
    writeln!(
        generated,
        "pending_item_ceiling: {},",
        manifest.pending_item_ceiling
    )
    .unwrap();
    writeln!(
        generated,
        "pending_scope_ceiling: {},",
        manifest.pending_scope_ceiling
    )
    .unwrap();
    generated.push_str("pending_scopes: vec![\n");
    for id in pending {
        writeln!(generated, "String::from({id:?}),").unwrap();
    }
    generated.push_str("] } }\n");
    generated
        .push_str("pub fn installed_configuration() -> ApiConfiguration { ApiConfiguration {\n");
    writeln!(
        generated,
        "surface_profile: String::from({:?}),",
        installed.profile.minecraft_version
    )
    .unwrap();
    writeln!(
        generated,
        "minecraft_version: String::from({:?}),",
        installed.minecraft_version
    )
    .unwrap();
    generated.push_str("cargo_features: vec![\n");
    for feature in installed.features {
        writeln!(generated, "String::from({feature:?}),").unwrap();
    }
    generated.push_str("],\n");
    writeln!(
        generated,
        "placeholder_codegen: {},",
        installed.placeholder_codegen
    )
    .unwrap();
    writeln!(
        generated,
        "compiled_surface_items: {},",
        installed_reachable.len()
    )
    .unwrap();
    generated.push_str("} }\n");
    let installed_paths = installed_reachable
        .iter()
        .flat_map(|item| item.paths.iter())
        .filter(|path| path.starts_with("sand::"))
        .collect::<BTreeSet<_>>();
    generated.push_str("pub static INSTALLED_API_PATHS: &[&str] = &[\n");
    for path in installed_paths {
        writeln!(generated, "{path:?},").unwrap();
    }
    generated.push_str("];\n");
    let canonical_by_identity = contracts
        .iter()
        .map(|contract| (contract.identity.as_str(), contract.canonical_path.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut facade_path_mappings = installed_reachable
        .iter()
        .filter_map(|item| {
            canonical_by_identity
                .get(item.identity.as_str())
                .map(|canonical_path| (item.identity.clone(), *canonical_path))
        })
        .collect::<BTreeMap<_, _>>();
    let kind_by_identity = installed_reachable
        .iter()
        .map(|item| (item.identity.as_str(), item.kind))
        .collect::<BTreeMap<_, _>>();
    let mut shortened_owners = BTreeMap::<String, BTreeSet<&str>>::new();
    for (identity, canonical_path) in &facade_path_mappings {
        let segments = identity.split("::").collect::<Vec<_>>();
        let end = match kind_by_identity.get(identity.as_str()) {
            Some(
                ReachableKind::Method
                | ReachableKind::TraitMethod
                | ReachableKind::AssociatedConst
                | ReachableKind::AssociatedType
                | ReachableKind::Field
                | ReachableKind::Variant,
            ) => segments.len().saturating_sub(1),
            _ => segments.len(),
        };
        for start in 1..end {
            shortened_owners
                .entry(
                    std::iter::once(segments[0])
                        .chain(segments[start..].iter().copied())
                        .collect::<Vec<_>>()
                        .join("::"),
                )
                .or_default()
                .insert(*canonical_path);
        }
    }
    for (implementation_path, owners) in shortened_owners {
        if let Some(canonical_path) = owners.iter().next().filter(|_| owners.len() == 1) {
            facade_path_mappings
                .entry(implementation_path)
                .or_insert(*canonical_path);
        }
    }
    generated.push_str("pub static INSTALLED_API_PATH_MAPPINGS: &[(&str, &str)] = &[\n");
    for (implementation_path, canonical_path) in &facade_path_mappings {
        writeln!(generated, "({implementation_path:?}, {canonical_path:?}),").unwrap();
    }
    generated.push_str("];\n");
    let mut suffix_owners = BTreeMap::<String, BTreeSet<&str>>::new();
    for canonical_path in facade_path_mappings.values().copied() {
        let segments = canonical_path.split("::").collect::<Vec<_>>();
        for start in 1..segments.len() {
            suffix_owners
                .entry(segments[start..].join("::"))
                .or_default()
                .insert(canonical_path);
        }
    }
    generated.push_str("pub static INSTALLED_API_SUFFIX_MAPPINGS: &[(&str, &str)] = &[\n");
    for (suffix, owners) in suffix_owners {
        if let Some(canonical_path) = owners.iter().next().filter(|_| owners.len() == 1) {
            writeln!(generated, "({suffix:?}, {canonical_path:?}),").unwrap();
        }
    }
    generated.push_str("];\n");
    let mut type_suffix_owners = BTreeMap::<String, BTreeSet<&str>>::new();
    let mut type_path_mappings = BTreeMap::<&str, &str>::new();
    for item in installed_reachable {
        if !matches!(
            item.kind,
            ReachableKind::Struct
                | ReachableKind::Enum
                | ReachableKind::Union
                | ReachableKind::Trait
                | ReachableKind::TypeAlias
        ) {
            continue;
        }
        let Some(canonical_path) = canonical_by_identity.get(item.identity.as_str()) else {
            continue;
        };
        type_path_mappings.insert(item.identity.as_str(), *canonical_path);
        let terminal = canonical_path.rsplit("::").next().unwrap_or(canonical_path);
        type_suffix_owners
            .entry(terminal.to_owned())
            .or_default()
            .insert(*canonical_path);
    }
    generated.push_str("pub static INSTALLED_API_TYPE_SUFFIX_MAPPINGS: &[(&str, &str)] = &[\n");
    for (suffix, owners) in type_suffix_owners {
        if let Some(canonical_path) = owners.iter().next().filter(|_| owners.len() == 1) {
            writeln!(generated, "({suffix:?}, {canonical_path:?}),").unwrap();
        }
    }
    generated.push_str("];\n");
    generated.push_str("pub static INSTALLED_API_TYPE_PATH_MAPPINGS: &[(&str, &str)] = &[\n");
    for (implementation_path, canonical_path) in type_path_mappings {
        writeln!(generated, "({implementation_path:?}, {canonical_path:?}),").unwrap();
    }
    generated.push_str("];\n");
    generated.push_str("pub static INSTALLED_FACADE_CONTRACTS: &[ApiRegistration] = &[\n");
    let mut installed_facades = source_declarations
        .iter()
        .filter_map(|declaration| {
            declaration
                .facade
                .as_ref()
                .map(|facade| (declaration, facade))
        })
        .collect::<Vec<_>>();
    installed_facades.sort_by_key(|(declaration, _)| declaration.canonical_path.as_str());
    let mut facade_registrations = String::new();
    for (declaration, facade) in &installed_facades {
        facade_registrations.push_str(
            "sand_api_contract::inventory::submit! { sand_api_contract::ApiRegistration {\n",
        );
        writeln!(
            facade_registrations,
            "canonical_path: {:?},",
            declaration.canonical_path
        )
        .unwrap();
        facade_registrations.push_str("aliases: &[");
        for alias in &declaration.aliases {
            write!(facade_registrations, "{alias:?},").unwrap();
        }
        writeln!(
            facade_registrations,
            "], canonical_module: {:?},",
            facade.canonical_module
        )
        .unwrap();
        writeln!(
            facade_registrations,
            "kind: sand_api_contract::ApiKind::{:?},",
            facade.kind
        )
        .unwrap();
        writeln!(
            facade_registrations,
            "signature: {:?},",
            facade.runtime_signature
        )
        .unwrap();
        writeln!(facade_registrations, "summary: {:?},", facade.summary).unwrap();
        writeln!(facade_registrations, "context: {:?},", facade.context).unwrap();
        writeln!(facade_registrations, "minecraft: {:?},", facade.minecraft).unwrap();
        facade_registrations.push_str("use_when: &[");
        for value in &facade.use_when {
            write!(facade_registrations, "{value:?},").unwrap();
        }
        facade_registrations.push_str("], avoid_when: &[");
        for value in &facade.avoid_when {
            write!(facade_registrations, "{value:?},").unwrap();
        }
        facade_registrations.push_str("], parameters: &[");
        for (name, description) in &facade.parameter_docs {
            write!(facade_registrations, "sand_api_contract::StaticApiParameter {{ name: {name:?}, description: {description:?} }},").unwrap();
        }
        facade_registrations.push_str("], returns: ");
        match &facade.return_doc {
            Some(value) => write!(facade_registrations, "Some({value:?})").unwrap(),
            None => facade_registrations.push_str("None"),
        }
        writeln!(facade_registrations, ", example: {:?},", facade.example).unwrap();
        facade_registrations.push_str("availability: &[");
        for value in &facade.availability {
            write!(facade_registrations, "{value:?},").unwrap();
        }
        facade_registrations.push_str("], } }\n");

        generated.push_str("ApiRegistration {\n");
        writeln!(
            generated,
            "canonical_path: {:?},",
            declaration.canonical_path
        )
        .unwrap();
        generated.push_str("aliases: &[");
        for alias in &declaration.aliases {
            write!(generated, "{alias:?},").unwrap();
        }
        generated.push_str("],\n");
        writeln!(
            generated,
            "canonical_module: {:?},",
            facade.canonical_module
        )
        .unwrap();
        writeln!(generated, "kind: ApiKind::{:?},", facade.kind).unwrap();
        writeln!(generated, "signature: {:?},", facade.runtime_signature).unwrap();
        writeln!(generated, "summary: {:?},", facade.summary).unwrap();
        writeln!(generated, "context: {:?},", facade.context).unwrap();
        writeln!(generated, "minecraft: {:?},", facade.minecraft).unwrap();
        generated.push_str("use_when: &[");
        for value in &facade.use_when {
            write!(generated, "{value:?},").unwrap();
        }
        generated.push_str("],\navoid_when: &[");
        for value in &facade.avoid_when {
            write!(generated, "{value:?},").unwrap();
        }
        generated.push_str("],\nparameters: &[");
        for (name, description) in &facade.parameter_docs {
            write!(
                generated,
                "StaticApiParameter {{ name: {name:?}, description: {description:?} }},"
            )
            .unwrap();
        }
        generated.push_str("],\n");
        match &facade.return_doc {
            Some(value) => writeln!(generated, "returns: Some({value:?}),").unwrap(),
            None => generated.push_str("returns: None,\n"),
        }
        writeln!(generated, "example: {:?},", facade.example).unwrap();
        generated.push_str("availability: &[");
        for value in &facade.availability {
            write!(generated, "{value:?},").unwrap();
        }
        generated.push_str("],\n},\n");
    }
    generated.push_str("];\n");
    generated.push_str("pub static INSTALLED_FAMILY_API_PATHS: &[&str] = &[\n");
    for (declaration, facade) in installed_facades {
        if facade.family {
            writeln!(generated, "{:?},", declaration.canonical_path).unwrap();
        }
    }
    generated.push_str("];\n");
    generated.push_str(
        "pub type InstalledApiShape = (&'static str, &'static [&'static str], &'static str, &'static [(&'static str, &'static str)], Option<&'static str>, &'static str, bool, Option<&'static str>, Option<&'static str>, Option<&'static str>);\n",
    );
    let all_definition_shapes = sand_api_enforce::definition_shapes(reachable)
        .unwrap_or_else(|error| panic!("failed to derive structural API metadata: {error}"));
    generated.push_str("pub static INSTALLED_API_SHAPES: &[InstalledApiShape] = &[\n");
    let pointer_only_docs = reachable
        .iter()
        .filter_map(|item| {
            let shape = all_definition_shapes.get(&item.identity)?;
            if !shape.documentation.contains("sand api show")
                || source_documentation_is_substantive(&shape.documentation)
            {
                return None;
            }
            let definition = item.definition.as_ref()?;
            let canonical_path = item
                .paths
                .iter()
                .find(|path| path.starts_with("sand::"))
                .map_or(item.identity.as_str(), String::as_str);
            Some(format!(
                "{canonical_path}\t{}\t{}",
                definition.source.display(),
                definition.line
            ))
        })
        .collect::<Vec<_>>();
    if !pointer_only_docs.is_empty() {
        let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));
        let report = output_dir.join("api_contract_doc_gaps.txt");
        fs::write(&report, pointer_only_docs.join("\n"))
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", report.display()));
        panic!(
            "{} public APIs use `sand api show` as their only substantive source Rustdoc; endpoint documentation must explain the API locally (details: {})",
            pointer_only_docs.len(),
            report.display()
        );
    }
    let definition_shapes = sand_api_enforce::definition_shapes(installed_reachable)
        .unwrap_or_else(|error| {
            panic!("failed to derive installed structural API metadata: {error}")
        });
    for item in installed_reachable {
        let Some(shape) = definition_shapes.get(&item.identity) else {
            continue;
        };
        write!(generated, "({:?}, &[", item.identity).unwrap();
        for path in item.paths.iter().filter(|path| path.starts_with("sand::")) {
            write!(generated, "{path:?},").unwrap();
        }
        write!(generated, "], {:?}, &[", shape.signature).unwrap();
        for (name, ty) in &shape.parameters {
            write!(generated, "({name:?},{ty:?}),").unwrap();
        }
        writeln!(
            generated,
            "], {:?}, {:?}, {}, {:?}, {:?}, {:?}),",
            shape.return_type.as_deref(),
            shape.documentation,
            shape.has_receiver,
            shape.impl_self_type.as_deref(),
            shape.impl_generics.as_deref(),
            shape.impl_where_clause.as_deref()
        )
        .unwrap();
    }
    generated.push_str("];\n");
    generated.push_str("pub static INSTALLED_API_IDENTITIES: &[&[&str]] = &[\n");
    for item in installed_reachable {
        generated.push_str("&[");
        for path in item.paths.iter().filter(|path| path.starts_with("sand::")) {
            write!(generated, "{path:?},").unwrap();
        }
        generated.push_str("],\n");
    }
    generated.push_str("];\n");
    generated.push_str(
        "pub fn installed_surface_report() -> &'static str { include_str!(concat!(env!(\"OUT_DIR\"), \"/api_surface_report.txt\")) }\n",
    );

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));
    let facade_output = output_dir.join("api_facade_registrations.rs");
    fs::write(&facade_output, facade_registrations)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", facade_output.display()));
    let output = output_dir.join("api_coverage.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", output.display()));

    let mut kinds = BTreeMap::<&str, usize>::new();
    let mut origins = BTreeMap::<String, usize>::new();
    for item in reachable {
        *kinds.entry(kind_name(item.kind)).or_default() += 1;
        let origin = match &item.origin {
            sand_api_enforce::ReachableOrigin::Source => "source".to_owned(),
            sand_api_enforce::ReachableOrigin::Generator(provider) => {
                format!("generator:{provider}")
            }
        };
        *origins.entry(origin).or_default() += 1;
    }
    let mut surface = format!(
        "schema_version=1\nconfiguration=all-supported-features,current-target\nminecraft_version={}\ntotal={}\n",
        installed.profile.minecraft_version,
        reachable.len(),
    );
    for (kind, count) in kinds {
        writeln!(surface, "kind {kind}={count}").unwrap();
    }
    for (origin, count) in origins {
        writeln!(surface, "origin {origin}={count}").unwrap();
    }
    writeln!(surface, "{report}").unwrap();
    let report_path = output_dir.join("api_surface_report.txt");
    fs::write(&report_path, &surface)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", report_path.display()));
    let baseline = fs::read_to_string(&installed.profile.baseline).unwrap_or_else(|error| {
        panic!(
            "failed to read selected API surface baseline {}: {error}",
            installed.profile.baseline.display()
        )
    });
    if baseline != surface {
        let difference = baseline
            .lines()
            .zip(surface.lines())
            .enumerate()
            .find(|(_, (expected, actual))| expected != actual)
            .map_or_else(
                || "file lengths differ".to_owned(),
                |(index, (expected, actual))| {
                    format!(
                        "first difference at line {}: baseline `{expected}`, measured `{actual}`",
                        index + 1
                    )
                },
            );
        panic!(
            "Sand API aggregate surface differs from selected profile baseline {}; classify the scope-level change and update that deterministic baseline ({difference})",
            installed.profile.baseline.display()
        );
    }
}

fn source_documentation_is_substantive(documentation: &str) -> bool {
    sand_api_contract::rustdoc_has_specific_semantics(documentation)
}

fn kind_name(kind: ReachableKind) -> &'static str {
    match kind {
        ReachableKind::Module => "module",
        ReachableKind::Struct => "struct",
        ReachableKind::Enum => "enum",
        ReachableKind::Union => "union",
        ReachableKind::Variant => "variant",
        ReachableKind::Field => "field",
        ReachableKind::Trait => "trait",
        ReachableKind::Function => "function",
        ReachableKind::Method => "method",
        ReachableKind::TraitMethod => "trait_method",
        ReachableKind::AssociatedConst => "associated_const",
        ReachableKind::AssociatedType => "associated_type",
        ReachableKind::TypeAlias => "type_alias",
        ReachableKind::Constant => "constant",
        ReachableKind::Static => "static",
        ReachableKind::Macro => "macro",
        ReachableKind::FunctionLikeMacro => "function_like_macro",
        ReachableKind::AttributeMacro => "attribute_macro",
        ReachableKind::DeriveMacro => "derive_macro",
    }
}
