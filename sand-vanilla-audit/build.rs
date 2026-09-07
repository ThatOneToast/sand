fn main() {
    let version = std::env::var("SAND_MC_VERSION")
        .unwrap_or_else(|_| sand_version::LATEST_KNOWN.to_string());
    println!("cargo:rustc-check-cfg=cfg(sand_audit_dialogs)");
    if version.starts_with("26.") {
        println!("cargo:rustc-cfg=sand_audit_dialogs");
    }
    sand_build::generate(&version).expect("vanilla audit codegen failed");
}
