//! Shared generators live in sand-components so resource and build authoring use one model.
pub use sand_components::worldgen::generator::{
    BiomeSource, FlatGenerator, FlatLayer, Generator, NoiseGenerator, NoiseSettingsRef,
    VanillaNoiseSettings,
};
