use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_contract::ApiKind;
use sand_api_enforce::{
    CfgSet, ContractIdentity, GeneratedApi, ReachableKind, ScopeManifest, ScopeState, SurfaceGraph,
    contract_declarations_from_files, discover_facade_feature_union, discover_local_source_crates,
    event_generated_type_provider, registry_id_provider, resolve_contract_identities,
    resource_ref_provider, validate_contract_lookup_namespace, vanilla_registry_enum_provider,
};

fn main() {
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
        "api-surface-baseline.txt",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-env-changed=DEP_SAND_CORE_API_PROVIDER_DIR");

    let manifest = ScopeManifest::from_path("api-scopes.toml")
        .unwrap_or_else(|error| panic!("invalid Sand API scope manifest: {error}"));
    let provider_dir = PathBuf::from(
        env::var_os("DEP_SAND_CORE_API_PROVIDER_DIR").unwrap_or_else(|| {
            panic!(
                "sand-core did not expose generated API providers; its build must emit api_provider_dir metadata"
            )
        }),
    );
    let (generated, generated_contracts) = generated_providers(&provider_dir)
        .unwrap_or_else(|error| panic!("invalid generated API provider: {error}"));
    let mut generated = generated;
    generated.extend(
        resource_ref_provider(&workspace.join("sand-core/src/resource_ref.rs"))
            .unwrap_or_else(|error| panic!("invalid resource_ref! API provider: {error}")),
    );
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
    let graph = SurfaceGraph::load_with_cfg(
        source_crates.clone(),
        cargo_cfg(enabled_features.clone()),
        generated,
    )
    .unwrap_or_else(|error| panic!("failed to construct Sand public facade graph: {error}"));
    let reachable = graph
        .reachable_from("sand")
        .unwrap_or_else(|error| panic!("failed to extract Sand public facade: {error}"));

    let source_declarations =
        contract_declarations_from_files(contract_source_files(&source_crates))
            .unwrap_or_else(|error| panic!("invalid build-time API contract source: {error}"));
    let mut contracts = resolve_contract_identities(&reachable, &source_declarations)
        .unwrap_or_else(|errors| panic_errors("invalid source API contract identities", &errors));
    contracts.extend(validate_generated_contracts(
        &reachable,
        generated_contracts,
    ));
    reject_duplicate_contract_identities(&contracts);

    let report = manifest
        .evaluate(&reachable, &contracts, &enabled_features)
        .unwrap_or_else(|errors| panic_errors("Sand API scope enforcement failed", &errors));
    if reachable.len() != manifest.static_surface_items {
        panic!(
            "Sand static public surface contains {} items, but api-scopes.toml records {}; classify the change and ratchet the committed baseline",
            reachable.len(),
            manifest.static_surface_items
        );
    }

    write_coverage(&manifest, &report, &reachable);
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

fn cargo_cfg(features: BTreeSet<String>) -> CfgSet {
    let mut flags = BTreeMap::from([("test".to_owned(), false)]);
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

fn generated_providers(
    directory: &Path,
) -> Result<(Vec<GeneratedApi>, Vec<ContractIdentity>), String> {
    let mut generated = Vec::new();
    let mut contracts = Vec::new();
    for filename in ["commands.api.json", "registries.api.json"] {
        let path = directory.join(filename);
        let catalog = sand_build::read_api_provider(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
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
    generated.sort_by(|left, right| left.identity.cmp(&right.identity));
    contracts.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok((generated, contracts))
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
        .filter_map(|contract| {
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
            Some(contract)
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

fn write_coverage(
    manifest: &ScopeManifest,
    report: &sand_api_enforce::ScopeReport,
    reachable: &[sand_api_enforce::ReachableApi],
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
    generated.push_str(
        "pub fn installed_surface_report() -> &'static str { include_str!(concat!(env!(\"OUT_DIR\"), \"/api_surface_report.txt\")) }\n",
    );

    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo provides OUT_DIR"));
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
        "schema_version=1\nconfiguration=all-supported-features,current-target\ntotal={}\n",
        reachable.len()
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
    let baseline = fs::read_to_string("api-surface-baseline.txt")
        .unwrap_or_else(|error| panic!("failed to read api-surface-baseline.txt: {error}"));
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
            "Sand API aggregate surface differs from api-surface-baseline.txt; classify the scope-level change and update the deterministic baseline ({difference})"
        );
    }
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
