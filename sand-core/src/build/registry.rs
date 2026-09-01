//! Static vanilla registry data used for build-time validation of world
//! resources (issue #317 §3.4).
//!
//! This is a **bundled, hand-maintained list**, not auto-generated from a
//! live Mojang data download the way `sand-build`'s block/item registries
//! are (`sand-build/src/codegen/registries.rs`). Extending the codegen
//! pipeline to fetch and generate a full biome/structure registry per
//! `VersionProfile` is tracked as a follow-up (see the PR that introduced
//! this module for the linked issue) — this module exists so
//! [`super::validate::validate`] can catch a real class of authoring
//! mistakes (a misspelled or non-existent vanilla biome ID) today, at the
//! cost of not being auto-refreshed when vanilla adds biomes in a future
//! release.
//!
//! Only `minecraft:`-namespaced references are checked against this list —
//! a custom mod or datapack biome (any other namespace) is accepted
//! unconditionally, since Sand cannot know about registry content it
//! didn't generate.

/// Every vanilla biome ID (`minecraft:<path>`) as of Minecraft Java 1.20+
/// through 26.2, Sand's supported version range. Sourced from vanilla's
/// `worldgen/biome` registry; stable across that range (no biome removals,
/// and additions like `cherry_grove`/`deep_dark` are already included).
pub(crate) const VANILLA_BIOMES: &[&str] = &[
    "ocean",
    "deep_ocean",
    "frozen_ocean",
    "deep_frozen_ocean",
    "cold_ocean",
    "deep_cold_ocean",
    "lukewarm_ocean",
    "deep_lukewarm_ocean",
    "warm_ocean",
    "deep_warm_ocean",
    "river",
    "frozen_river",
    "beach",
    "snowy_beach",
    "forest",
    "flower_forest",
    "birch_forest",
    "old_growth_birch_forest",
    "dark_forest",
    "old_growth_pine_taiga",
    "old_growth_spruce_taiga",
    "taiga",
    "snowy_taiga",
    "savanna",
    "savanna_plateau",
    "windswept_hills",
    "windswept_gravelly_hills",
    "windswept_forest",
    "windswept_savanna",
    "jungle",
    "sparse_jungle",
    "bamboo_jungle",
    "badlands",
    "eroded_badlands",
    "wooded_badlands",
    "meadow",
    "grove",
    "snowy_slopes",
    "frozen_peaks",
    "jagged_peaks",
    "stony_peaks",
    "stony_shore",
    "desert",
    "swamp",
    "mangrove_swamp",
    "plains",
    "sunflower_plains",
    "snowy_plains",
    "ice_spikes",
    "the_void",
    "nether_wastes",
    "crimson_forest",
    "warped_forest",
    "soul_sand_valley",
    "basalt_deltas",
    "the_end",
    "end_highlands",
    "end_midlands",
    "small_end_islands",
    "end_barrens",
    "dripstone_caves",
    "lush_caves",
    "deep_dark",
    "cherry_grove",
];

/// Whether `path` is a known vanilla biome. Only meaningful for
/// `minecraft:`-namespaced references — see the module docs.
pub(crate) fn is_known_vanilla_biome(path: &str) -> bool {
    VANILLA_BIOMES.contains(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_biomes() {
        assert!(is_known_vanilla_biome("plains"));
        assert!(is_known_vanilla_biome("the_void"));
        assert!(is_known_vanilla_biome("cherry_grove"));
    }

    #[test]
    fn rejects_a_misspelled_biome() {
        assert!(!is_known_vanilla_biome("dessert"));
        assert!(!is_known_vanilla_biome("plainss"));
    }

    #[test]
    fn list_has_no_duplicates() {
        let mut sorted = VANILLA_BIOMES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), VANILLA_BIOMES.len());
    }
}
