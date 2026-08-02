//! Worldgen builders for `data/<namespace>/worldgen/` JSON files (Minecraft 1.21+).
//!
//! This module provides builders for the most commonly customized worldgen types.
//! More complex worldgen (noise settings, processor lists, etc.) can be supplied
//! via the `raw` constructors that accept arbitrary JSON.

pub mod biome;
pub mod dimension;
pub mod dimension_type;
pub mod noise_settings;
pub mod placed_feature;
pub mod processor_list;
pub mod providers;
pub mod structure;
pub mod structure_set;
pub mod template_pool;

pub use biome::Biome;
pub use dimension::Dimension;
pub use dimension_type::{DimensionType, MonsterSpawnLightLevel};
pub use noise_settings::NoiseSettings;
pub use placed_feature::PlacedFeature;
pub use processor_list::{Processor, ProcessorList, ProcessorRule};
pub use providers::{HeightProvider, Heightmap, VerticalAnchor, WorldgenBlockState};
pub use structure::{
    BiomeSelector, GenerationStep, JigsawConfig, MobCategory, SpawnBoundingBox, SpawnEntry,
    SpawnOverride, Structure, TerrainAdaptation,
};
pub use structure_set::{
    ExclusionZone, FrequencyReductionMethod, SpreadType, StructureEntry, StructurePlacement,
    StructureSet,
};
pub use template_pool::{PoolElement, PoolEntry, ProcessorsRef, Projection, TemplatePool};
