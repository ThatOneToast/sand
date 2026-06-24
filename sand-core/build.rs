fn main() {
    let version = std::env::var("SAND_MC_VERSION").unwrap_or_else(|_| "1.21.11".to_string());

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    println!("cargo:rerun-if-env-changed=SAND_MC_VERSION");
    println!("cargo:rerun-if-env-changed=SAND_STRICT_CODEGEN");

    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    match sand_build::generate_to_dir(&version, &out_dir) {
        Ok(()) => {}
        Err(e) if strict => {
            // In strict mode the build must fail so the developer (or CI) knows
            // that `sand_core::generated` and `sand_core::block_states` would be
            // empty. Set SAND_STRICT_CODEGEN=1 in CI to catch this early.
            panic!(
                "sand-build failed to generate registry types for MC {version}: {e}\n\
                 \n\
                 To allow the build to continue with empty placeholders (local dev),\n\
                 unset SAND_STRICT_CODEGEN or set it to 0."
            );
        }
        Err(e) => {
            // Non-strict: write empty placeholders so the include! macros compile,
            // but emit a prominent warning so the developer knows types are absent.
            println!(
                "cargo:warning=sand-core codegen FAILED for MC {version}: {e}\n\
                 `sand_core::generated` and `sand_core::block_states` will be EMPTY.\n\
                 Fix the issue above, or set SAND_STRICT_CODEGEN=1 to turn this into\n\
                 a hard build error."
            );
            let _ = std::fs::write(out_dir.join("registries.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("block_states.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("commands.rs"), "// Generation failed\n");
        }
    }
}
