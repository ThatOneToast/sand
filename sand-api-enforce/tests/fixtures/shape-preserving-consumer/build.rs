use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_enforce::{
    ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph, shape_preserving_consumer_provider,
};

fn contract(identity: &str) -> ContractIdentity {
    ContractIdentity {
        identity: identity.into(),
        canonical_path: identity.into(),
        aliases: BTreeSet::new(),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=api-scopes.toml");
    for macro_name in [
        "function",
        "datapack_component",
        "on_event",
        "armor_event",
        "schedule",
        "EntityStateEnum",
    ] {
        shape_preserving_consumer_provider(Path::new("src/lib.rs"), macro_name)
            .unwrap_or_else(|error| panic!("invalid {macro_name} consumer provider: {error}"));
    }
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        [],
    )
    .expect("extract shape-preserving consumer surface");
    let reachable = graph.reachable_from("sand").expect("resolve fixture facade");
    let mut contracts = vec![
        contract("sand::generated_function"),
        contract("sand::generated_component"),
        contract("sand::generated_event"),
        contract("sand::generated_armor_event"),
        contract("sand::generated_schedule"),
        contract("sand::GeneratedPhase"),
        contract("sand::GeneratedPhase::Idle"),
        contract("sand::GeneratedPhase::Active"),
    ];
    if env::var_os("CARGO_FEATURE_COMPLETE_PROVIDER").is_none() {
        contracts.retain(|contract| contract.identity != "sand::generated_schedule");
    }
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    let connected = BTreeSet::from([
        "function-generated".to_owned(),
        "component-generated".to_owned(),
        "event-generated".to_owned(),
        "armor-event-generated".to_owned(),
        "schedule-generated".to_owned(),
        "entity-state-enum-generated".to_owned(),
    ]);
    let diagnostic = manifest
        .evaluate_with_provider_audits(&reachable, &contracts, &BTreeSet::new(), &connected)
        .err()
        .map(|failures| failures.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("api_enforcement.rs");
    fs::write(
        output,
        diagnostic.map_or_else(String::new, |diagnostic| format!("compile_error!({diagnostic:?});")),
    )
    .expect("write enforcement result");
}
