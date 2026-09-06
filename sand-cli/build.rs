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
    let revision = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&workspace_root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_owned())
        .filter(|revision| !revision.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=SAND_CLI_GIT_REVISION={revision}");
    println!("cargo:rerun-if-changed={workspace_root}/.git/HEAD");
    println!("cargo:rerun-if-changed=build.rs");
}
