//! Worldgen builders for `data/<namespace>/worldgen/` JSON files (Minecraft 1.21+).
//!
//! This module provides builders for the most commonly customized worldgen types.
//! More complex worldgen (noise settings, processor lists, etc.) can be supplied
//! via the `raw` constructors that accept arbitrary JSON.

pub mod biome;
pub mod dimension;
pub mod noise_settings;
pub mod placed_feature;

pub use biome::Biome;
pub use dimension::Dimension;
pub use noise_settings::NoiseSettings;
pub use placed_feature::PlacedFeature;
