use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_enforce::{
    ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph, custom_item_provider,
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
    let generated = custom_item_provider(Path::new("src/lib.rs"), "sand")
        .expect("custom-item declaration must provide generated API metadata");
    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        generated,
    )
    .expect("extract fixture surface with custom-item provider")
    .bind_api_producer("sand::shard_blade", "custom_item", "item_macro")
    .expect("connect custom_item to its generated API provider");
    let reachable = graph.reachable_from("sand").expect("resolve fixture facade");
    let mut contracts = vec![
        contract("sand::shard_blade"),
        contract("sand::ShardBlade"),
        contract("sand::ShardBlade::BASE"),
        contract("sand::ShardBlade::PREDICATE"),
        contract("sand::ShardBlade::CUSTOM_DATA_KEY"),
        contract("sand::ShardBlade::CUSTOM_DATA_SNBT"),
        contract("sand::ShardBlade::if_wearing"),
        contract("sand::ShardBlade::unless_wearing"),
        contract("sand::ShardBlade::item"),
    ];
    if env::var_os("CARGO_FEATURE_COMPLETE_PROVIDER").is_some() {
        contracts.push(contract("sand::ShardBlade::DAMAGE"));
    }
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    let connected = BTreeSet::from(["item-generated".to_owned()]);
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
