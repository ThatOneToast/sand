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

#[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep` for the canonical contract."]
/// The world-generation step a structure starts in.
///
/// This is vanilla's shared `GenerationStep.Decoration` enum: the same
/// ordered steps also bucket a biome's per-step `features` list-of-lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationStep {
    #[doc = "Selects the raw generation form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::RawGeneration` for the canonical contract."]
    RawGeneration,
    #[doc = "Selects the lakes form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::Lakes` for the canonical contract."]
    Lakes,
    #[doc = "Selects the local modifications form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::LocalModifications` for the canonical contract."]
    LocalModifications,
    #[doc = "Selects the underground structures form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::UndergroundStructures` for the canonical contract."]
    UndergroundStructures,
    #[doc = "Selects the surface structures form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::SurfaceStructures` for the canonical contract."]
    SurfaceStructures,
    #[doc = "Selects the strongholds form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::Strongholds` for the canonical contract."]
    Strongholds,
    #[doc = "Selects the underground ores form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::UndergroundOres` for the canonical contract."]
    UndergroundOres,
    #[doc = "Selects the underground decoration form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::UndergroundDecoration` for the canonical contract."]
    UndergroundDecoration,
    #[doc = "Selects the fluid springs form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::FluidSprings` for the canonical contract."]
    FluidSprings,
    #[doc = "Selects the vegetal decoration form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::VegetalDecoration` for the canonical contract."]
    VegetalDecoration,
    #[doc = "Selects the top layer modification form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::TopLayerModification` for the canonical contract."]
    TopLayerModification,
}

impl GenerationStep {
    /// The vanilla string written into datapack JSON.
    #[doc = "**API Contract:** Run `sand api show sand::component::GenerationStep::as_str` for the canonical contract."]
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

    /// The vanilla `GenerationStep.Decoration` ordinal, used to index a
    /// biome's per-step `features` list-of-lists array.
    pub(crate) fn index(self) -> usize {
        match self {
            Self::RawGeneration => 0,
            Self::Lakes => 1,
            Self::LocalModifications => 2,
            Self::UndergroundStructures => 3,
            Self::SurfaceStructures => 4,
            Self::Strongholds => 5,
            Self::UndergroundOres => 6,
            Self::UndergroundDecoration => 7,
            Self::FluidSprings => 8,
            Self::VegetalDecoration => 9,
            Self::TopLayerModification => 10,
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation` for the canonical contract."]
/// How terrain is modified around a generated structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdaptation {
    #[doc = "Selects the none form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::None` for the canonical contract."]
    None,
    #[doc = "Selects the beard thin form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::BeardThin` for the canonical contract."]
    BeardThin,
    #[doc = "Selects the beard box form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::BeardBox` for the canonical contract."]
    BeardBox,
    #[doc = "Selects the bury form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::Bury` for the canonical contract."]
    Bury,
    #[doc = "Selects the encapsulate form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::Encapsulate` for the canonical contract."]
    Encapsulate,
}

impl TerrainAdaptation {
    /// The vanilla string written into datapack JSON.
    #[doc = "**API Contract:** Run `sand api show sand::component::TerrainAdaptation::as_str` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::component::MobCategory` for the canonical contract."]
/// A vanilla mob category used as a spawn-override key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MobCategory {
    #[doc = "Selects the monster form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::Monster` for the canonical contract."]
    Monster,
    #[doc = "Selects the creature form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::Creature` for the canonical contract."]
    Creature,
    #[doc = "Selects the ambient form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::Ambient` for the canonical contract."]
    Ambient,
    #[doc = "Selects the axolotls form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::Axolotls` for the canonical contract."]
    Axolotls,
    #[doc = "Selects the underground water creature form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::UndergroundWaterCreature` for the canonical contract."]
    UndergroundWaterCreature,
    #[doc = "Selects the water creature form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::WaterCreature` for the canonical contract."]
    WaterCreature,
    #[doc = "Selects the water ambient form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::WaterAmbient` for the canonical contract."]
    WaterAmbient,
    #[doc = "Selects the misc form in this typed Minecraft component schema."]
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::Misc` for the canonical contract."]
    Misc,
}

impl MobCategory {
    /// The vanilla string written into datapack JSON.
    #[doc = "**API Contract:** Run `sand api show sand::component::MobCategory::as_str` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::component::SpawnBoundingBox` for the canonical contract."]
/// Which part of a structure a spawn override applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnBoundingBox {
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnBoundingBox::Piece` for the canonical contract."]
    /// Only inside individual structure pieces.
    Piece,
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnBoundingBox::Full` for the canonical contract."]
    /// The structure's full bounding box.
    Full,
}

impl SpawnBoundingBox {
    /// The vanilla string written into datapack JSON.
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnBoundingBox::as_str` for the canonical contract."]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Piece => "piece",
            Self::Full => "full",
        }
    }
}

#[doc = "**API Contract:** Run `sand api show sand::component::SpawnEntry` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnEntry::new` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::component::SpawnOverride` for the canonical contract."]
/// A per-mob-category spawn override for a structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnOverride {
    bounding_box: SpawnBoundingBox,
    spawns: Vec<SpawnEntry>,
}

impl SpawnOverride {
    /// An override that suppresses all spawns of its category.
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnOverride::none` for the canonical contract."]
    pub fn none(bounding_box: SpawnBoundingBox) -> Self {
        Self {
            bounding_box,
            spawns: Vec::new(),
        }
    }

    /// An override with an explicit spawn list.
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnOverride::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::SpawnOverride::spawn` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::component::BiomeSelector` for the canonical contract."]
/// The biome constraint of a structure: a biome tag or an explicit list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSelector {
    #[doc = "**API Contract:** Run `sand api show sand::component::BiomeSelector::Tag` for the canonical contract."]
    /// A biome tag reference, emitted as `#namespace:path`.
    Tag(
        #[doc = "The `Tag` variant carries the value described by its variant semantics: A biome tag reference, emitted as `#namespace:path`."]
        #[doc = "**API Contract:** Run `sand api show sand::component::BiomeSelector::Tag::0` for the canonical contract."]
        TagId<BiomeId>,
    ),
    #[doc = "**API Contract:** Run `sand api show sand::component::BiomeSelector::Entries` for the canonical contract."]
    /// An explicit list of biome IDs.
    Entries(
        #[doc = "The `Entries` variant carries the value described by its variant semantics: An explicit list of biome IDs."]
        #[doc = "**API Contract:** Run `sand api show sand::component::BiomeSelector::Entries::0` for the canonical contract."]
        Vec<BiomeId>,
    ),
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

#[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::size` for the canonical contract."]
    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    /// Sets the Minecraft start height property on this typed jigsaw config definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::start_height` for the canonical contract."]
    pub fn start_height(mut self, start_height: HeightProvider) -> Self {
        self.start_height = start_height;
        self
    }

    /// Sets the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::start_jigsaw_name` for the canonical contract."]
    pub fn start_jigsaw_name(mut self, name: impl Into<String>) -> Self {
        self.start_jigsaw_name = Some(name.into());
        self
    }

    /// Sets the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::project_start_to_heightmap` for the canonical contract."]
    pub fn project_start_to_heightmap(mut self, heightmap: Heightmap) -> Self {
        self.project_start_to_heightmap = Some(heightmap);
        self
    }

    /// Maximum horizontal distance pieces may extend from the start (`1..=128`).
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::max_distance_from_center` for the canonical contract."]
    pub fn max_distance_from_center(mut self, blocks: u32) -> Self {
        self.max_distance_from_center = blocks;
        self
    }

    /// Sets the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::JigsawConfig::use_expansion_hack` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::component::Structure` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::new` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::jigsaw` for the canonical contract."]
    pub fn jigsaw(
        location: ResourceLocation,
        start_pool: TemplatePoolId,
        biomes: BiomeSelector,
    ) -> Self {
        Self::new(location, StructureTypeId::jigsaw(), biomes)
            .jigsaw_config(JigsawConfig::new(start_pool))
    }

    /// Sets the Minecraft structure type property on this typed structure definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::structure_type` for the canonical contract."]
    pub fn structure_type(mut self, structure_type: StructureTypeId) -> Self {
        self.structure_type = structure_type;
        self
    }

    /// Sets the Minecraft biomes property on this typed structure definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::biomes` for the canonical contract."]
    pub fn biomes(mut self, biomes: BiomeSelector) -> Self {
        self.biomes = biomes;
        self
    }

    /// Sets the Minecraft step property on this typed structure definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::step` for the canonical contract."]
    pub fn step(mut self, step: GenerationStep) -> Self {
        self.step = step;
        self
    }

    /// Sets the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::terrain_adaptation` for the canonical contract."]
    pub fn terrain_adaptation(mut self, adaptation: TerrainAdaptation) -> Self {
        self.terrain_adaptation = Some(adaptation);
        self
    }

    /// Replace the jigsaw configuration.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::jigsaw_config` for the canonical contract."]
    pub fn jigsaw_config(mut self, config: JigsawConfig) -> Self {
        self.jigsaw = Some(config);
        self
    }

    /// Modify the jigsaw configuration in place, if one is present.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::map_jigsaw_config` for the canonical contract."]
    pub fn map_jigsaw_config(mut self, f: impl FnOnce(JigsawConfig) -> JigsawConfig) -> Self {
        self.jigsaw = self.jigsaw.map(f);
        self
    }

    /// Add or replace a spawn override for one mob category.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::spawn_override` for the canonical contract."]
    pub fn spawn_override(mut self, category: MobCategory, spawns: SpawnOverride) -> Self {
        self.spawn_overrides.insert(category, spawns);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[doc = "**API Contract:** Run `sand api show sand::component::Structure::raw_field` for the canonical contract."]
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
