use std::collections::BTreeSet;
use std::path::Path;

use sand_api_enforce::{SurfaceGraph, discover_local_source_crates};

#[test]
fn local_path_dependency_reexport_enters_the_reachable_surface() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local-path-reexport/Cargo.toml");
    let crates = discover_local_source_crates(&fixture).unwrap();
    assert_eq!(
        crates
            .iter()
            .map(|source| source.name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["facade", "newly_added"])
    );

    let graph = SurfaceGraph::load(crates, [], []).unwrap();
    let reachable = graph.reachable_from("facade").unwrap();
    let paths = |identity: &str| {
        reachable
            .iter()
            .find(|api| api.identity == identity)
            .unwrap_or_else(|| panic!("missing reachable identity {identity}"))
            .paths
            .clone()
    };
    assert_eq!(
        paths("newly_added::AddedThroughPathDependency"),
        BTreeSet::from(["facade::AddedThroughPathDependency".into()])
    );
    assert_eq!(
        paths("newly_added::AddedThroughPathDependency::newly_reachable"),
        BTreeSet::from(["facade::AddedThroughPathDependency::newly_reachable".into()])
    );
}
