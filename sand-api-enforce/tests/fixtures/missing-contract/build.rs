use std::path::PathBuf;

use sand_api_enforce::{SurfaceRoot, enforce};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    enforce(&[SurfaceRoot {
        source: PathBuf::from("src/lib.rs"),
        canonical_module: "sand::fixture".to_owned(),
    }]);
}
