use std::collections::BTreeSet;
use std::path::PathBuf;

use sand_api_enforce::{SurfaceRoot, enforce_with_contracts};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/prelude.rs");
    println!("cargo:rerun-if-changed=api-surface.txt");

    let contracted = std::fs::read_to_string("api-surface.txt")
        .expect("read sand/api-surface.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    enforce_with_contracts(
        &[
            SurfaceRoot {
                source: PathBuf::from("src/lib.rs"),
                canonical_module: "sand".to_owned(),
            },
            SurfaceRoot {
                source: PathBuf::from("src/prelude.rs"),
                canonical_module: "sand::prelude".to_owned(),
            },
        ],
        &contracted,
    );
}
