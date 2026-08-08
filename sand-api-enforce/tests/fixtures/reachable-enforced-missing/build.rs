use std::collections::BTreeSet;
use std::path::PathBuf;

use sand_api_enforce::{ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=api-scopes.toml");

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        [],
    )
    .expect("extract reachable facade surface");
    let reachable = graph.reachable_from("sand").expect("resolve facade");
    let contracts = [
        ContractIdentity {
            identity: "sand::predicate".into(),
            canonical_path: "sand::predicate".into(),
            aliases: BTreeSet::new(),
        },
        ContractIdentity {
            identity: "sand::predicate::Builder".into(),
            canonical_path: "sand::predicate::Builder".into(),
            aliases: BTreeSet::new(),
        },
        ContractIdentity {
            identity: "sand::predicate::Choice".into(),
            canonical_path: "sand::predicate::Choice".into(),
            aliases: BTreeSet::new(),
        },
    ];
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    if let Err(failures) = manifest.evaluate(&reachable, &contracts, &BTreeSet::new()) {
        let diagnostics = failures
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        panic!("reachable API contract enforcement failed:\n{diagnostics}");
    }
}
