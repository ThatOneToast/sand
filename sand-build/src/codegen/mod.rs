mod blocks;
pub mod cache;
mod commands;
mod registries;

use std::path::Path;

use crate::error::Result;

// Brings `CODEGEN_IMPL_FINGERPRINT` into scope: a SHA-1 fingerprint of this
// crate's entire `codegen/` source tree (recursively, by relative path and
// content -- see `source_tree_fingerprint.rs`), computed automatically at
// compile time by `sand-build/build.rs`. Editing, adding, removing, or
// renaming any generator source file changes this constant with no manual
// step -- see `build.rs` for the full rationale (issue #347 PR #348 review
// items 1 and 2).
include!(concat!(env!("OUT_DIR"), "/codegen_impl_fingerprint.rs"));

/// Explicit salt for generated-code *cache format* changes that don't touch
/// any file under `codegen/` (e.g. changing which files the cache expects
/// to find in a published entry). Ordinary generator logic changes must
/// never depend on this being bumped correctly -- that's what
/// [`CODEGEN_IMPL_FINGERPRINT`] is for.
pub const CODEGEN_CACHE_FORMAT_VERSION: u32 = 1;

/// Generate all source files from the data generator reports.
///
/// Writes to `out_dir` (typically `$OUT_DIR` from Cargo):
/// - `registries.rs` — enums for item, block, entity type, biome, etc.
/// - `block_states.rs` — per-block property structs and shared property enums.
/// - `commands.rs`    — builder structs for Minecraft commands.
///
/// Always regenerates; does not consult the generated-code cache. Callers
/// that want caching should use [`generate_all_cached`].
pub fn generate_all(reports_dir: &Path, out_dir: &Path, minecraft_version: &str) -> Result<()> {
    registries::generate(reports_dir, out_dir, minecraft_version)?;
    blocks::generate(reports_dir, out_dir)?;
    commands::generate(reports_dir, out_dir, minecraft_version)?;
    Ok(())
}

/// Same as [`generate_all`], but consults the fingerprint-addressed
/// generated-code cache first (issue #347 Phase 3): a validated cache hit
/// copies the previously generated files into `out_dir` instead of
/// re-running codegen; a miss generates normally and publishes the result
/// for future builds. Cache population failures never fail the build --
/// see `cache::publish`.
pub fn generate_all_cached(
    reports_dir: &Path,
    out_dir: &Path,
    minecraft_version: &str,
) -> Result<()> {
    generate_all_cached_with_root(
        &crate::cache::cache_dir()?,
        reports_dir,
        out_dir,
        minecraft_version,
    )
}

/// Same as [`generate_all_cached`], but with the cache root passed
/// explicitly instead of resolved from `$HOME`. Exists so tests can point
/// at a temporary directory without mutating global process state.
pub fn generate_all_cached_with_root(
    cache_root: &Path,
    reports_dir: &Path,
    out_dir: &Path,
    minecraft_version: &str,
) -> Result<()> {
    let fp = cache::fingerprint(reports_dir, minecraft_version)?;

    if cache::try_load(cache_root, minecraft_version, &fp, out_dir)? {
        return Ok(());
    }

    generate_all(reports_dir, out_dir, minecraft_version)?;
    cache::publish(cache_root, minecraft_version, &fp, out_dir);
    Ok(())
}

#[cfg(test)]
mod cached_generation_tests {
    use super::*;

    /// Fixture reports mirroring the smallest fixtures each codegen
    /// submodule's own tests already use, kept minimal since this test is
    /// only exercising the cache wiring, not codegen correctness (that's
    /// covered by `registries`/`blocks`/`commands`'s own fixture tests).
    fn write_fixture_reports(dir: &Path) {
        std::fs::write(
            dir.join("registries.json"),
            serde_json::json!({
                "minecraft:item": {
                    "entries": {
                        "minecraft:air": { "protocol_id": 0 },
                        "minecraft:stone": { "protocol_id": 1 }
                    },
                    "protocol_id": 0
                }
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(dir.join("blocks.json"), r#"{}"#).unwrap();
        std::fs::write(
            dir.join("commands.json"),
            serde_json::json!({ "type": "root", "children": {} }).to_string(),
        )
        .unwrap();
    }

    #[test]
    fn cached_path_is_byte_identical_to_uncached_generation() {
        let reports = tempfile::tempdir().unwrap();
        write_fixture_reports(reports.path());

        let uncached_out = tempfile::tempdir().unwrap();
        generate_all(reports.path(), uncached_out.path(), "1.21.4").unwrap();

        let cache_root = tempfile::tempdir().unwrap();
        let cached_out_miss = tempfile::tempdir().unwrap();
        generate_all_cached_with_root(
            cache_root.path(),
            reports.path(),
            cached_out_miss.path(),
            "1.21.4",
        )
        .unwrap();

        for file in [
            "registries.rs",
            "registries.api.json",
            "block_states.rs",
            "commands.rs",
            "commands.api.json",
        ] {
            assert_eq!(
                std::fs::read(uncached_out.path().join(file)).unwrap(),
                std::fs::read(cached_out_miss.path().join(file)).unwrap(),
                "cache-miss path for '{file}' must match uncached generation byte-for-byte"
            );
        }
    }

    #[test]
    fn second_call_with_the_same_inputs_hits_the_cache_and_still_matches() {
        let reports = tempfile::tempdir().unwrap();
        write_fixture_reports(reports.path());
        let cache_root = tempfile::tempdir().unwrap();

        let first_out = tempfile::tempdir().unwrap();
        generate_all_cached_with_root(
            cache_root.path(),
            reports.path(),
            first_out.path(),
            "1.21.4",
        )
        .unwrap();

        // Second call, fresh out_dir, same cache_root/reports/version: must
        // be served from cache and produce identical bytes.
        let second_out = tempfile::tempdir().unwrap();
        generate_all_cached_with_root(
            cache_root.path(),
            reports.path(),
            second_out.path(),
            "1.21.4",
        )
        .unwrap();

        for file in ["registries.rs", "block_states.rs", "commands.rs"] {
            assert_eq!(
                std::fs::read(first_out.path().join(file)).unwrap(),
                std::fs::read(second_out.path().join(file)).unwrap(),
            );
        }
    }

    #[test]
    fn changing_reports_produces_different_output_even_with_a_warm_cache() {
        let cache_root = tempfile::tempdir().unwrap();
        let reports = tempfile::tempdir().unwrap();
        write_fixture_reports(reports.path());

        let first_out = tempfile::tempdir().unwrap();
        generate_all_cached_with_root(
            cache_root.path(),
            reports.path(),
            first_out.path(),
            "1.21.4",
        )
        .unwrap();

        // A different item registry -> different fingerprint -> cache miss
        // -> freshly generated (not stale-cached) output.
        std::fs::write(
            reports.path().join("registries.json"),
            serde_json::json!({
                "minecraft:item": {
                    "entries": {
                        "minecraft:dirt": { "protocol_id": 0 }
                    },
                    "protocol_id": 0
                }
            })
            .to_string(),
        )
        .unwrap();
        let second_out = tempfile::tempdir().unwrap();
        generate_all_cached_with_root(
            cache_root.path(),
            reports.path(),
            second_out.path(),
            "1.21.4",
        )
        .unwrap();

        assert_ne!(
            std::fs::read_to_string(first_out.path().join("registries.rs")).unwrap(),
            std::fs::read_to_string(second_out.path().join("registries.rs")).unwrap(),
            "changed report content must not be served stale from the cache"
        );
    }
}
