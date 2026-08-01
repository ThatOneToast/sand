//! Worldgen builders for `data/<namespace>/worldgen/` JSON files (Minecraft 1.21+).
//!
//! This module provides builders for the most commonly customized worldgen types.
//! More complex worldgen (noise settings, processor lists, etc.) can be supplied
//! via the `raw` constructors that accept arbitrary JSON.

pub mod biome;
pub mod configured_feature;
pub mod dimension;
pub mod dimension_type;
pub mod noise_settings;
pub mod placed_feature;
pub mod providers;

pub use biome::Biome;
pub use configured_feature::{ConfiguredFeature, OreConfig, OreTarget, RuleTest};
pub use dimension::Dimension;
pub use dimension_type::{DimensionType, MonsterSpawnLightLevel};
pub use noise_settings::NoiseSettings;
pub use placed_feature::PlacedFeature;
pub use providers::{BlockState, BlockStateProvider, WeightedBlockState};
