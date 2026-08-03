use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_enforce::{
    ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph, sand_storage_derive_provider,
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

    let generated = sand_storage_derive_provider(Path::new("src/lib.rs"), "sand")
        .expect("derive declaration must provide generated API metadata");
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        generated,
    )
    .expect("extract fixture surface with derive provider");
    let reachable = graph.reachable_from("sand").expect("resolve fixture facade");
    let mut contracts = vec![
        contract("sand::PlayerMagic"),
        contract("sand::PlayerMagic::SCHEMA"),
    ];
    if env::var_os("CARGO_FEATURE_COMPLETE_PROVIDER").is_some() {
        contracts.push(contract("sand::PlayerMagic::mana"));
    }
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    let connected = BTreeSet::from(["sand-storage-generated".to_owned()]);
    let diagnostic = manifest
        .evaluate_with_provider_audits(&reachable, &contracts, &BTreeSet::new(), &connected)
        .err()
        .map(|failures| {
            failures
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        });
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("api_enforcement.rs");
    let rust = diagnostic.map_or_else(String::new, |diagnostic| {
        format!("compile_error!({diagnostic:?});")
    });
    fs::write(output, rust).expect("write enforcement result");
}
