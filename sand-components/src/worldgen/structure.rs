//! Builder for `data/<namespace>/worldgen/structure/<id>.json`.
//!
//! [`Structure::jigsaw`] is the common starting point: it produces a complete
//! jigsaw structure that only needs a biome selector and a start pool. All
//! stable vanilla fields have typed setters, while [`Structure::raw_field`] is
//! an explicit escape hatch for modded or version-specific additions.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{BiomeId, EntityTypeId, StructureTypeId, TagId, TemplatePoolId};
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::providers::{HeightProvider, Heightmap};

const KIND: &str = "worldgen/structure";

const TYPED_FIELDS: &[&str] = &[
    "type",
    "biomes",
    "step",
    "terrain_adaptation",
    "spawn_overrides",
    "start_pool",
    "size",
    "start_height",
    "start_jigsaw_name",
    "project_start_to_heightmap",
    "max_distance_from_center",
    "use_expansion_hack",
];

/// The world-generation step a structure starts in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationStep {
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}

impl GenerationStep {
    /// The vanilla string written into datapack JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RawGeneration => "raw_generation",
            Self::Lakes => "lakes",
            Self::LocalModifications => "local_modifications",
            Self::UndergroundStructures => "underground_structures",
            Self::SurfaceStructures => "surface_structures",
            Self::Strongholds => "strongholds",
            Self::UndergroundOres => "underground_ores",
            Self::UndergroundDecoration => "underground_decoration",
            Self::FluidSprings => "fluid_springs",
            Self::VegetalDecoration => "vegetal_decoration",
            Self::TopLayerModification => "top_layer_modification",
        }
    }
}

/// How terrain is modified around a generated structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdaptation {
    None,
    BeardThin,
    BeardBox,
    Bury,
    Encapsulate,
}

impl TerrainAdaptation {
    /// The vanilla string written into datapack JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BeardThin => "beard_thin",
            Self::BeardBox => "beard_box",
            Self::Bury => "bury",
            Self::Encapsulate => "encapsulate",
        }
    }
}

/// A vanilla mob category used as a spawn-override key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MobCategory {
    Monster,
    Creature,
    Ambient,
    Axolotls,
    UndergroundWaterCreature,
    WaterCreature,
    WaterAmbient,
    Misc,
}

impl MobCategory {
    /// The vanilla string written into datapack JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monster => "monster",
            Self::Creature => "creature",
            Self::Ambient => "ambient",
            Self::Axolotls => "axolotls",
            Self::UndergroundWaterCreature => "underground_water_creature",
            Self::WaterCreature => "water_creature",
            Self::WaterAmbient => "water_ambient",
            Self::Misc => "misc",
        }
    }
}

/// Which part of a structure a spawn override applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnBoundingBox {
    /// Only inside individual structure pieces.
    Piece,
    /// The structure's full bounding box.
    Full,
}

impl SpawnBoundingBox {
    /// The vanilla string written into datapack JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Piece => "piece",
            Self::Full => "full",
        }
    }
}

/// One weighted mob-spawn entry inside a [`SpawnOverride`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnEntry {
    entity_type: EntityTypeId,
    weight: u32,
    min_count: u32,
    max_count: u32,
}

impl SpawnEntry {
    /// Create a spawn entry. `weight` and `min_count` must be at least 1 and
    /// `max_count` must be at least `min_count`; both are checked on export.
    pub fn new(entity_type: EntityTypeId, weight: u32, min_count: u32, max_count: u32) -> Self {
        Self {
            entity_type,
            weight,
            min_count,
            max_count,
        }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "type": self.entity_type.to_string(),
            "weight": self.weight,
            "minCount": self.min_count,
            "maxCount": self.max_count,
        })
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            KIND,
            &format!("{field}.type"),
            &self.entity_type.to_string(),
        )?;
        if self.weight == 0 {
            return Err(validation::error(
                location,
                KIND,
                &format!("{field}.weight"),
                "spawn weight must be at least 1",
            ));
        }
        if self.min_count == 0 {
            return Err(validation::error(
                location,
                KIND,
                &format!("{field}.minCount"),
                "minCount must be at least 1",
            ));
        }
        if self.max_count < self.min_count {
            return Err(validation::error(
                location,
                KIND,
                &format!("{field}.maxCount"),
                &format!(
                    "maxCount must be at least minCount; received {}..={}",
                    self.min_count, self.max_count
                ),
            ));
        }
        Ok(())
    }
}

/// A per-mob-category spawn override for a structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnOverride {
    bounding_box: SpawnBoundingBox,
    spawns: Vec<SpawnEntry>,
}

impl SpawnOverride {
    /// An override that suppresses all spawns of its category.
    pub fn none(bounding_box: SpawnBoundingBox) -> Self {
        Self {
            bounding_box,
            spawns: Vec::new(),
        }
    }

    /// An override with an explicit spawn list.
    pub fn new(
        bounding_box: SpawnBoundingBox,
        spawns: impl IntoIterator<Item = SpawnEntry>,
    ) -> Self {
        Self {
            bounding_box,
            spawns: spawns.into_iter().collect(),
        }
    }

    /// Append a spawn entry.
    pub fn spawn(mut self, entry: SpawnEntry) -> Self {
        self.spawns.push(entry);
        self
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "bounding_box": self.bounding_box.as_str(),
            "spawns": self.spawns.iter().map(SpawnEntry::to_json).collect::<Vec<_>>(),
        })
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        for (index, spawn) in self.spawns.iter().enumerate() {
            spawn.validate(location, &format!("{field}.spawns[{index}]"))?;
        }
        Ok(())
    }
}

/// The biome constraint of a structure: a biome tag or an explicit list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSelector {
    /// A biome tag reference, emitted as `#namespace:path`.
    Tag(TagId<BiomeId>),
    /// An explicit list of biome IDs.
    Entries(Vec<BiomeId>),
}

impl BiomeSelector {
    fn to_json(&self) -> Value {
        match self {
            Self::Tag(tag) => Value::String(tag.to_tag_string()),
            Self::Entries(entries) => Value::Array(
                entries
                    .iter()
                    .map(|id| Value::String(id.to_string()))
                    .collect(),
            ),
        }
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        match self {
            Self::Tag(tag) => validation::validate_resource_or_tag_location_str(
                location,
                KIND,
                "biomes",
                &tag.to_tag_string(),
            ),
            Self::Entries(entries) => {
                validation::require_non_empty_collection(location, KIND, "biomes", entries.len())?;
                for (index, biome) in entries.iter().enumerate() {
                    validation::validate_resource_location_str(
                        location,
                        KIND,
                        &format!("biomes[{index}]"),
                        &biome.to_string(),
                    )?;
                }
                Ok(())
            }
        }
    }
}

/// The jigsaw-specific configuration of a `minecraft:jigsaw` structure.
#[derive(Debug, Clone, PartialEq)]
pub struct JigsawConfig {
    start_pool: TemplatePoolId,
    size: u32,
    start_height: HeightProvider,
    start_jigsaw_name: Option<String>,
    project_start_to_heightmap: Option<Heightmap>,
    max_distance_from_center: u32,
    use_expansion_hack: bool,
}

impl JigsawConfig {
    /// A jigsaw config with vanilla village-like defaults.
    pub fn new(start_pool: TemplatePoolId) -> Self {
        Self {
            start_pool,
            size: 6,
            start_height: HeightProvider::absolute(0),
            start_jigsaw_name: None,
            project_start_to_heightmap: None,
            max_distance_from_center: 80,
            use_expansion_hack: false,
        }
    }

    /// Jigsaw expansion depth (`0..=20`).
    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    pub fn start_height(mut self, start_height: HeightProvider) -> Self {
        self.start_height = start_height;
        self
    }

    pub fn start_jigsaw_name(mut self, name: impl Into<String>) -> Self {
        self.start_jigsaw_name = Some(name.into());
        self
    }

    pub fn project_start_to_heightmap(mut self, heightmap: Heightmap) -> Self {
        self.project_start_to_heightmap = Some(heightmap);
        self
    }

    /// Maximum horizontal distance pieces may extend from the start (`1..=128`).
    pub fn max_distance_from_center(mut self, blocks: u32) -> Self {
        self.max_distance_from_center = blocks;
        self
    }

    pub fn use_expansion_hack(mut self, value: bool) -> Self {
        self.use_expansion_hack = value;
        self
    }

    fn write_into(&self, map: &mut Map<String, Value>) {
        map.insert(
            "start_pool".into(),
            Value::String(self.start_pool.to_string()),
        );
        map.insert("size".into(), self.size.into());
        map.insert("start_height".into(), self.start_height.to_json());
        if let Some(name) = &self.start_jigsaw_name {
            map.insert("start_jigsaw_name".into(), Value::String(name.clone()));
        }
        if let Some(heightmap) = self.project_start_to_heightmap {
            map.insert(
                "project_start_to_heightmap".into(),
                Value::String(heightmap.as_str().to_string()),
            );
        }
        map.insert(
            "max_distance_from_center".into(),
            self.max_distance_from_center.into(),
        );
        map.insert("use_expansion_hack".into(), self.use_expansion_hack.into());
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            KIND,
            "start_pool",
            &self.start_pool.to_string(),
        )?;
        validation::require_u32_in_range(location, KIND, "size", self.size, 0, 20)?;
        validation::require_u32_in_range(
            location,
            KIND,
            "max_distance_from_center",
            self.max_distance_from_center,
            1,
            128,
        )?;
        self.start_height.validate(location, KIND, "start_height")?;
        if let Some(name) = &self.start_jigsaw_name {
            validation::require_non_empty(location, KIND, "start_jigsaw_name", name)?;
            validation::reject_whitespace_only(location, KIND, "start_jigsaw_name", name)?;
            validation::reject_control_chars(location, KIND, "start_jigsaw_name", name)?;
        }
        Ok(())
    }
}

/// A structure definition (`data/<namespace>/worldgen/structure/<id>.json`).
///
/// ```
/// use sand_components::{DatapackComponent, ResourceLocation, Structure, TagId, TemplatePoolId};
/// use sand_components::worldgen::structure::BiomeSelector;
///
/// let structure = Structure::jigsaw(
///     ResourceLocation::new("example", "outpost").unwrap(),
///     TemplatePoolId::minecraft("village/plains/town_centers").unwrap(),
///     BiomeSelector::Tag(TagId::minecraft("has_structure/village_plains").unwrap()),
/// );
/// structure.validate().unwrap();
/// assert_eq!(structure.component_dir(), "worldgen/structure");
/// assert_eq!(structure.to_json()["type"], "minecraft:jigsaw");
/// ```
pub struct Structure {
    location: ResourceLocation,
    structure_type: StructureTypeId,
    biomes: BiomeSelector,
    step: GenerationStep,
    terrain_adaptation: Option<TerrainAdaptation>,
    spawn_overrides: BTreeMap<MobCategory, SpawnOverride>,
    jigsaw: Option<JigsawConfig>,
    raw_fields: BTreeMap<String, RawJson>,
}

impl Structure {
    /// Create a structure of an arbitrary typed structure type.
    ///
    /// Non-jigsaw vanilla structure types carry type-specific fields that Sand
    /// does not model; supply those through [`Structure::raw_field`].
    pub fn new(
        location: ResourceLocation,
        structure_type: StructureTypeId,
        biomes: BiomeSelector,
    ) -> Self {
        Self {
            location,
            structure_type,
            biomes,
            step: GenerationStep::SurfaceStructures,
            terrain_adaptation: None,
            spawn_overrides: BTreeMap::new(),
            jigsaw: None,
            raw_fields: BTreeMap::new(),
        }
    }

    /// Create a complete `minecraft:jigsaw` structure with vanilla-like defaults.
    pub fn jigsaw(
        location: ResourceLocation,
        start_pool: TemplatePoolId,
        biomes: BiomeSelector,
    ) -> Self {
        Self::new(location, StructureTypeId::jigsaw(), biomes)
            .jigsaw_config(JigsawConfig::new(start_pool))
    }

    pub fn structure_type(mut self, structure_type: StructureTypeId) -> Self {
        self.structure_type = structure_type;
        self
    }

    pub fn biomes(mut self, biomes: BiomeSelector) -> Self {
        self.biomes = biomes;
        self
    }

    pub fn step(mut self, step: GenerationStep) -> Self {
        self.step = step;
        self
    }

    pub fn terrain_adaptation(mut self, adaptation: TerrainAdaptation) -> Self {
        self.terrain_adaptation = Some(adaptation);
        self
    }

    /// Replace the jigsaw configuration.
    pub fn jigsaw_config(mut self, config: JigsawConfig) -> Self {
        self.jigsaw = Some(config);
        self
    }

    /// Modify the jigsaw configuration in place, if one is present.
    pub fn map_jigsaw_config(mut self, f: impl FnOnce(JigsawConfig) -> JigsawConfig) -> Self {
        self.jigsaw = self.jigsaw.map(f);
        self
    }

    /// Add or replace a spawn override for one mob category.
    pub fn spawn_override(mut self, category: MobCategory, spawns: SpawnOverride) -> Self {
        self.spawn_overrides.insert(category, spawns);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    pub fn raw_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.raw_fields.insert(key.into(), value);
        self
    }
}

impl DatapackComponent for Structure {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        validation::validate_resource_location_str(
            &self.location,
            KIND,
            "type",
            &self.structure_type.to_string(),
        )?;
        self.biomes.validate(&self.location)?;
        for (category, spawns) in &self.spawn_overrides {
            spawns.validate(
                &self.location,
                &format!("spawn_overrides.{}", category.as_str()),
            )?;
        }
        match &self.jigsaw {
            Some(jigsaw) => jigsaw.validate(&self.location)?,
            None if self.structure_type == StructureTypeId::jigsaw() => {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "start_pool",
                    "minecraft:jigsaw structures require a jigsaw configuration",
                ));
            }
            None => {}
        }
        for key in self.raw_fields.keys() {
            validation::require_non_empty(&self.location, KIND, "raw_field", key)?;
            validation::reject_whitespace_only(&self.location, KIND, "raw_field", key)?;
            validation::reject_control_chars(&self.location, KIND, "raw_field", key)?;
            if TYPED_FIELDS.contains(&key.as_str()) {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "raw_field",
                    &format!("raw field `{key}` would override a typed field"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "type".into(),
            Value::String(self.structure_type.to_string()),
        );
        map.insert("biomes".into(), self.biomes.to_json());
        map.insert("step".into(), Value::String(self.step.as_str().to_string()));
        if let Some(adaptation) = self.terrain_adaptation {
            map.insert(
                "terrain_adaptation".into(),
                Value::String(adaptation.as_str().to_string()),
            );
        }
        if !self.spawn_overrides.is_empty() {
            let overrides: Map<String, Value> = self
                .spawn_overrides
                .iter()
                .map(|(category, spawns)| (category.as_str().to_string(), spawns.to_json()))
                .collect();
            map.insert("spawn_overrides".into(), Value::Object(overrides));
        }
        if let Some(jigsaw) = &self.jigsaw {
            jigsaw.write_into(&mut map);
        }
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/structure"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::providers::VerticalAnchor;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "sky_outpost").unwrap()
    }

    fn biomes() -> BiomeSelector {
        BiomeSelector::Tag(TagId::minecraft("has_structure/village_plains").unwrap())
    }

    fn jigsaw_structure() -> Structure {
        Structure::jigsaw(
            location(),
            TemplatePoolId::minecraft("village/plains/town_centers").unwrap(),
            biomes(),
        )
    }

    #[test]
    fn minimal_jigsaw_structure_matches_vanilla_shape() {
        let structure = jigsaw_structure();
        structure.validate().unwrap();
        assert_eq!(
            structure.to_json(),
            serde_json::json!({
                "type": "minecraft:jigsaw",
                "biomes": "#minecraft:has_structure/village_plains",
                "step": "surface_structures",
                "start_pool": "minecraft:village/plains/town_centers",
                "size": 6,
                "start_height": { "absolute": 0 },
                "max_distance_from_center": 80,
                "use_expansion_hack": false,
            })
        );
        assert_eq!(structure.component_dir(), "worldgen/structure");
    }

    #[test]
    fn full_typed_structure_serializes_overrides_and_jigsaw_options() {
        let structure = jigsaw_structure()
            .step(GenerationStep::UndergroundStructures)
            .terrain_adaptation(TerrainAdaptation::BeardThin)
            .biomes(BiomeSelector::Entries(vec![
                BiomeId::minecraft("plains").unwrap(),
            ]))
            .spawn_override(
                MobCategory::Monster,
                SpawnOverride::new(
                    SpawnBoundingBox::Piece,
                    [SpawnEntry::new(
                        EntityTypeId::minecraft("pillager").unwrap(),
                        1,
                        1,
                        2,
                    )],
                ),
            )
            .map_jigsaw_config(|config| {
                config
                    .size(7)
                    .start_height(HeightProvider::Uniform {
                        min_inclusive: VerticalAnchor::AboveBottom(0),
                        max_inclusive: VerticalAnchor::BelowTop(8),
                    })
                    .start_jigsaw_name("minecraft:bottom")
                    .project_start_to_heightmap(Heightmap::WorldSurfaceWg)
                    .max_distance_from_center(96)
                    .use_expansion_hack(true)
            })
            .raw_field("example:mood", RawJson::new(serde_json::json!("eerie")));
        structure.validate().unwrap();
        let json = structure.to_json();
        assert_eq!(json["biomes"][0], "minecraft:plains");
        assert_eq!(json["terrain_adaptation"], "beard_thin");
        assert_eq!(json["spawn_overrides"]["monster"]["bounding_box"], "piece");
        assert_eq!(
            json["spawn_overrides"]["monster"]["spawns"][0]["type"],
            "minecraft:pillager"
        );
        assert_eq!(json["project_start_to_heightmap"], "WORLD_SURFACE_WG");
        assert_eq!(json["start_jigsaw_name"], "minecraft:bottom");
        assert_eq!(json["example:mood"], "eerie");
    }

    #[test]
    fn jigsaw_type_without_config_is_rejected() {
        let structure = Structure::new(location(), StructureTypeId::jigsaw(), biomes());
        assert!(structure.validate().is_err());
    }

    #[test]
    fn empty_biome_list_and_out_of_range_jigsaw_values_are_rejected() {
        assert!(
            jigsaw_structure()
                .biomes(BiomeSelector::Entries(Vec::new()))
                .validate()
                .is_err()
        );
        assert!(
            jigsaw_structure()
                .map_jigsaw_config(|config| config.size(21))
                .validate()
                .is_err()
        );
        assert!(
            jigsaw_structure()
                .map_jigsaw_config(|config| config.max_distance_from_center(0))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn invalid_spawn_overrides_are_rejected() {
        let zero_weight = jigsaw_structure().spawn_override(
            MobCategory::Monster,
            SpawnOverride::none(SpawnBoundingBox::Full).spawn(SpawnEntry::new(
                EntityTypeId::minecraft("zombie").unwrap(),
                0,
                1,
                1,
            )),
        );
        assert!(zero_weight.validate().is_err());

        let inverted_counts = jigsaw_structure().spawn_override(
            MobCategory::Monster,
            SpawnOverride::none(SpawnBoundingBox::Full).spawn(SpawnEntry::new(
                EntityTypeId::minecraft("zombie").unwrap(),
                1,
                4,
                2,
            )),
        );
        assert!(inverted_counts.validate().is_err());
    }

    #[test]
    fn raw_fields_cannot_override_typed_fields() {
        assert!(
            jigsaw_structure()
                .raw_field("size", RawJson::new(serde_json::json!(3)))
                .validate()
                .is_err()
        );
    }
}
