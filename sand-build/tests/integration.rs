/// Full pipeline integration test.
///
/// Requires network access and Java 21+ on PATH.
/// Run with: `cargo test -p sand-build --features integration-tests`
#[cfg(feature = "integration-tests")]
mod pipeline {
    #[test]
    fn full_pipeline_1_21_4() {
        let out_dir = tempfile::tempdir().unwrap();
        sand_build::generate_to_dir("1.21.4", out_dir.path())
            .expect("full pipeline should succeed");

        let registries_rs = out_dir.path().join("registries.rs");
        let block_states_rs = out_dir.path().join("block_states.rs");
        assert!(registries_rs.exists(), "registries.rs should be generated");
        assert!(
            block_states_rs.exists(),
            "block_states.rs should be generated"
        );

        let reg = std::fs::read_to_string(&registries_rs).unwrap();
        assert!(reg.contains("pub enum Item {"), "Item enum expected");
        assert!(reg.contains("pub enum Block {"), "Block enum expected");
        assert!(
            reg.contains("\"minecraft:air\""),
            "minecraft:air resource location expected"
        );

        let bs = std::fs::read_to_string(&block_states_rs).unwrap();
        assert!(
            bs.contains("pub struct OakDoorProperties {"),
            "OakDoorProperties struct expected"
        );
    }

    /// End-to-end regression for #118: verifies the default codegen target
    /// (`sand_version::DEFAULT_CODEGEN_VERSION`) is codegen-available — downloads
    /// the jar, runs the data generator, and asserts non-placeholder output.
    ///
    /// **Maintenance-only — `#[ignore]` so it is NOT part of deterministic
    /// workspace tests.** Requires network access and Java 21+ on PATH. Run
    /// the explicit maintenance command:
    /// `cargo test -p sand-build --features integration-tests \
    ///     default_codegen_target_is_codegen_available -- --ignored`
    /// The deterministic code contract is covered by
    /// `sand_core::version::tests::default_codegen_version_contract`.
    #[test]
    #[ignore = "network/Java/Mojang-dependent: run explicitly with --ignored"]
    fn default_codegen_target_is_codegen_available() {
        let out_dir = tempfile::tempdir().unwrap();
        let version = sand_version::DEFAULT_CODEGEN_VERSION;
        sand_build::generate_to_dir(version, out_dir.path())
            .unwrap_or_else(|e| panic!("default codegen target {version} should succeed: {e}"));

        for name in ["registries.rs", "block_states.rs", "commands.rs"] {
            let path = out_dir.path().join(name);
            assert!(
                path.exists(),
                "{name} should be generated for default target {version}"
            );
            let contents =
                std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));
            assert!(
                !contents.trim().is_empty(),
                "{name} for default target {version} must not be empty"
            );
            assert!(
                !contents.contains("Generation failed"),
                "{name} for default target {version} must not be the non-strict placeholder"
            );
        }

        let reg = std::fs::read_to_string(out_dir.path().join("registries.rs")).unwrap();
        assert!(
            reg.contains("pub enum Item {"),
            "default target {version} must generate the Item enum"
        );
    }
}
