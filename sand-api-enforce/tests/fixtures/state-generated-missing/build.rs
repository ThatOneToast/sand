use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_enforce::{
    CfgSet, ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph, state_derive_provider,
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
    let generated = state_derive_provider(Path::new("src/lib.rs"), "sand", &CfgSet::default())
        .expect("State declaration must provide generated API metadata");
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        generated,
    )
    .expect("extract fixture surface with State provider")
    .bind_api_producer("sand::PlayerState", "State", "state_derive")
    .expect("connect State to its generated API provider");
    let reachable = graph.reachable_from("sand").expect("resolve fixture facade");
    let mut contracts = vec![
        contract("sand::PlayerState"),
        contract("sand::PlayerStateBound"),
        contract("sand::PlayerStateBound::mana"),
        contract("sand::PlayerState::FIELDS"),
        contract("sand::PlayerState::on"),
    ];
    if env::var_os("CARGO_FEATURE_COMPLETE_PROVIDER").is_some() {
        contracts.push(contract("sand::PlayerState::mana"));
    }
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    let connected = BTreeSet::from(["state-generated".to_owned()]);
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
