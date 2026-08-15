use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use sand_api_enforce::{
    CfgSet, ContractIdentity, ScopeManifest, SourceCrate, SurfaceGraph,
    resourcepack_macro_provider,
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
    let cfg = CfgSet {
        features: ["resourcepack".to_owned()].into_iter().collect(),
        ..CfgSet::default()
    };
    let generated = resourcepack_macro_provider(Path::new("src/lib.rs"), "sand", &cfg)
        .expect("resourcepack macros must provide generated HUD handle metadata");
    let graph = SurfaceGraph::load_with_cfg(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        cfg,
        generated,
    )
    .expect("extract resourcepack fixture surface")
    .bind_item_macro_provider("sand", "hud_bar", "resourcepack_macros")
    .expect("bind hud_bar provider")
    .bind_item_macro_provider("sand", "hud_element", "resourcepack_macros")
    .expect("bind hud_element provider")
    .bind_item_macro_provider("sand", "texture", "resourcepack_macros")
    .expect("bind texture provider");
    let reachable = graph.reachable_from("sand").expect("resolve fixture facade");
    let mut contracts = vec![contract("sand::HEALTH")];
    if env::var_os("CARGO_FEATURE_COMPLETE_PROVIDER").is_some() {
        contracts.push(contract("sand::STATUS_ICON"));
    }
    let manifest = ScopeManifest::from_path("api-scopes.toml").expect("parse scope manifest");
    let connected = BTreeSet::from(["resourcepack-generated".to_owned()]);
    let diagnostic = manifest
        .evaluate_with_provider_audits(
            &reachable,
            &contracts,
            &BTreeSet::from(["resourcepack".to_owned()]),
            &connected,
        )
        .err()
        .map(|failures| failures.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("api_enforcement.rs");
    fs::write(
        output,
        diagnostic.map_or_else(String::new, |diagnostic| format!("compile_error!({diagnostic:?});")),
    )
    .expect("write enforcement result");
}
