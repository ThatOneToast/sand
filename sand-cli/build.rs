fn main() {
    // Embed the workspace root at compile time so generated projects can use
    // path dependencies pointing to the local sand-core and sand-build crates.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .display()
        .to_string();
    println!("cargo:rustc-env=SAND_WORKSPACE_ROOT={workspace_root}");
    println!("cargo:rerun-if-changed=build.rs");
}
