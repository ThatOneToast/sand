use std::path::PathBuf;

use sand_api_enforce::{SourceCrate, SurfaceGraph};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");

    let graph = SurfaceGraph::load(
        [SourceCrate {
            name: "sand".into(),
            root: PathBuf::from("src/lib.rs"),
        }],
        [],
        [],
    )
    .expect("extract reachable facade surface");
    graph
        .reachable_from("sand")
        .unwrap_or_else(|error| panic!("public API extraction failed: {error}"));
}
