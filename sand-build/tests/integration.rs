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
}
