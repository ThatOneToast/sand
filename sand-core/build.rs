// Single source of truth for the default codegen target. Lives in
// `sand-version` so it can be shared by this build script, the
// `generated_api_health` regression tests, and contributor docs without
// pulling `sand-core` into a build-time dependency cycle. See the docs on
// `sand_version::DEFAULT_CODEGEN_VERSION` for the codegen-vs-profile contract.
use sand_version::DEFAULT_CODEGEN_VERSION;

fn main() {
    // The default codegen target. Contributors get a working
    // `cargo test -p sand-core --lib` out of the box when this version is
    // codegen-available (cached server jar or network). Override with
    // `SAND_MC_VERSION=<version>` to target a different Minecraft version.
    let version =
        std::env::var("SAND_MC_VERSION").unwrap_or_else(|_| DEFAULT_CODEGEN_VERSION.to_string());

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    println!("cargo:rerun-if-env-changed=SAND_MC_VERSION");
    println!("cargo:rerun-if-env-changed=SAND_STRICT_CODEGEN");
    println!("cargo:rerun-if-env-changed=SAND_ALLOW_PLACEHOLDER_CODEGEN");

    // SAND_STRICT_CODEGEN is kept for CI backward compatibility. The default
    // behavior is now already a hard failure (see below), so strict mode is
    // functionally equivalent — it just makes the intent explicit in CI.
    let strict = std::env::var("SAND_STRICT_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    // Explicit opt-in placeholder fallback. When codegen fails and this is set
    // (and strict is NOT), write `// Generation failed` placeholder files so
    // the include! macros still compile. The generated_api_health tests then
    // fail on those placeholders — they can never silently pass.
    let allow_placeholders = std::env::var("SAND_ALLOW_PLACEHOLDER_CODEGEN")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false);

    match sand_build::generate_to_dir(&version, &out_dir) {
        Ok(()) => {}
        Err(e) if allow_placeholders && !strict => {
            // Explicitly opted-in lenient mode: write placeholder files so the
            // include! macros still compile. The generated_api_health tests
            // will fail on these placeholders.
            println!(
                "cargo:warning=sand-core codegen FAILED for MC {version}: {e}\n\
                 Placeholder files written because SAND_ALLOW_PLACEHOLDER_CODEGEN=1.\n\
                 The generated_api_health tests will fail until real codegen succeeds.\n\
                 Remove SAND_ALLOW_PLACEHOLDER_CODEGEN for a hard build error."
            );
            let _ = std::fs::write(out_dir.join("registries.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("block_states.rs"), "// Generation failed\n");
            let _ = std::fs::write(out_dir.join("commands.rs"), "// Generation failed\n");
        }
        Err(e) => {
            // Default and strict: fail immediately with an actionable error.
            // Do NOT write placeholders — a secondary test failure from
            // placeholder files is less actionable than a build-time error.
            panic!(
                "sand-build codegen failed for MC {version}: {e}\n\
                 \n\
                 The default codegen target is `sand_version::DEFAULT_CODEGEN_VERSION`\n\
                 (={DEFAULT_CODEGEN_VERSION}); override it with SAND_MC_VERSION=<version>.\n\
                 \n\
                 Codegen requires a Java runtime new enough for the selected Minecraft\n\
                 server (Java 21 for the stable baseline; Java 25 for 26.2) and either\n\
                 network access or a cached jar in ~/.sand/cache/{version}/.\n\
                 \n\
                 To compile sand-core with empty placeholder APIs (generated_api_health\n\
                 tests will fail), set SAND_ALLOW_PLACEHOLDER_CODEGEN=1."
            );
        }
    }
}
