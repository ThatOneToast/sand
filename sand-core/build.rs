fn main() {
    let version = std::env::var("SAND_MC_VERSION").unwrap_or_else(|_| "1.21.11".to_string());

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    println!("cargo:rerun-if-env-changed=SAND_MC_VERSION");

    match sand_build::generate_to_dir(&version, &out_dir) {
        Ok(()) => {}
        Err(e) => {
            // Don't hard-fail the build — write empty placeholders so the
            // include! macros in lib.rs can still compile. A warning is
            // surfaced so the developer knows the types are absent.
            println!(
                "cargo:warning=sand-build failed to generate registry types for \
                 MC {version}: {e}. `sand_core::generated` and \
                 `sand_core::block_states` will be empty."
            );
            let _ = std::fs::write(out_dir.join("registries.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("block_states.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("commands.rs"), "// Generation failed\n");
        }
    }
}
