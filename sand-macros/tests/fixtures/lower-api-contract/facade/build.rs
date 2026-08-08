use std::path::PathBuf;

use sand_api_enforce::{
    SourceCrate, SurfaceGraph, contract_declarations_from_files, resolve_contract_identities,
};

fn main() {
    let facade = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let lower = facade.parent().unwrap().join("lower/src/lib.rs");
    let facade_source = facade.join("src/lib.rs");
    let graph = SurfaceGraph::load(
        [
            SourceCrate {
                name: "sand".into(),
                root: facade_source.clone(),
            },
            SourceCrate {
                name: "lower_api_provider".into(),
                root: lower.clone(),
            },
        ],
        [],
        [],
    )
    .unwrap();
    let reachable = graph.reachable_from("sand").unwrap();
    let declarations = contract_declarations_from_files([&lower]).unwrap();
    let contracts = resolve_contract_identities(&reachable, &declarations).unwrap();
    let identities = contracts
        .iter()
        .map(|contract| contract.identity.as_str())
        .collect::<Vec<_>>();
    assert!(identities.contains(&"lower_api_provider::Widget"));
    assert!(identities.contains(&"lower_api_provider::Widget::value"));

    println!("cargo:rerun-if-changed={}", lower.display());
    println!("cargo:rerun-if-changed={}", facade_source.display());
}
