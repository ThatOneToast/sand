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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::GenerationStep",
    aliases = ["sand::prelude::GenerationStep"],
    module = "sand::component",
    summary = "The world-generation step a structure starts in.",
    context = "The world-generation step a structure starts in. This is vanilla's shared `GenerationStep.Decoration` enum: the same ordered steps also bucket a biome's per-step `features` list-of-lists.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::GenerationStep;",
    variants(FluidSprings = "Selects the fluid springs form in this typed Minecraft component schema.", Lakes = "Selects the lakes form in this typed Minecraft component schema.", LocalModifications = "Selects the local modifications form in this typed Minecraft component schema.", RawGeneration = "Selects the raw generation form in this typed Minecraft component schema.", Strongholds = "Selects the strongholds form in this typed Minecraft component schema.", SurfaceStructures = "Selects the surface structures form in this typed Minecraft component schema.", TopLayerModification = "Selects the top layer modification form in this typed Minecraft component schema.", UndergroundDecoration = "Selects the underground decoration form in this typed Minecraft component schema.", UndergroundOres = "Selects the underground ores form in this typed Minecraft component schema.", UndergroundStructures = "Selects the underground structures form in this typed Minecraft component schema.", VegetalDecoration = "Selects the vegetal decoration form in this typed Minecraft component schema."),
)]
/// The world-generation step a structure starts in.
///
/// This is vanilla's shared `GenerationStep.Decoration` enum: the same
/// ordered steps also bucket a biome's per-step `features` list-of-lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenerationStep {
    #[doc = "Selects the raw generation form in this typed Minecraft component schema."]
    RawGeneration,
    #[doc = "Selects the lakes form in this typed Minecraft component schema."]
    Lakes,
    #[doc = "Selects the local modifications form in this typed Minecraft component schema."]
    LocalModifications,
    #[doc = "Selects the underground structures form in this typed Minecraft component schema."]
    UndergroundStructures,
    #[doc = "Selects the surface structures form in this typed Minecraft component schema."]
    SurfaceStructures,
    #[doc = "Selects the strongholds form in this typed Minecraft component schema."]
    Strongholds,
    #[doc = "Selects the underground ores form in this typed Minecraft component schema."]
    UndergroundOres,
    #[doc = "Selects the underground decoration form in this typed Minecraft component schema."]
    UndergroundDecoration,
    #[doc = "Selects the fluid springs form in this typed Minecraft component schema."]
    FluidSprings,
    #[doc = "Selects the vegetal decoration form in this typed Minecraft component schema."]
    VegetalDecoration,
    #[doc = "Selects the top layer modification form in this typed Minecraft component schema."]
    TopLayerModification,
}

impl GenerationStep {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::GenerationStep::as_str",
        aliases = ["sand::prelude::GenerationStep::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(generation_step_value: &sand::component::GenerationStep)  {\n    let as_str = generation_step_value.as_str();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TerrainAdaptation",
    aliases = ["sand::prelude::TerrainAdaptation"],
    module = "sand::component",
    summary = "How terrain is modified around a generated structure.",
    context = "How terrain is modified around a generated structure. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TerrainAdaptation;",
    variants(BeardBox = "Selects the beard box form in this typed Minecraft component schema.", BeardThin = "Selects the beard thin form in this typed Minecraft component schema.", Bury = "Selects the bury form in this typed Minecraft component schema.", Encapsulate = "Selects the encapsulate form in this typed Minecraft component schema.", None = "Selects the none form in this typed Minecraft component schema."),
)]
/// How terrain is modified around a generated structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdaptation {
    #[doc = "Selects the none form in this typed Minecraft component schema."]
    None,
    #[doc = "Selects the beard thin form in this typed Minecraft component schema."]
    BeardThin,
    #[doc = "Selects the beard box form in this typed Minecraft component schema."]
    BeardBox,
    #[doc = "Selects the bury form in this typed Minecraft component schema."]
    Bury,
    #[doc = "Selects the encapsulate form in this typed Minecraft component schema."]
    Encapsulate,
}

impl TerrainAdaptation {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TerrainAdaptation::as_str",
        aliases = ["sand::prelude::TerrainAdaptation::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(terrain_adaptation_value: &sand::component::TerrainAdaptation)  {\n    let as_str = terrain_adaptation_value.as_str();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::MobCategory",
    aliases = ["sand::prelude::MobCategory"],
    module = "sand::component",
    summary = "A vanilla mob category used as a spawn-override key.",
    context = "A vanilla mob category used as a spawn-override key. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::MobCategory;",
    variants(Ambient = "Selects the ambient form in this typed Minecraft component schema.", Axolotls = "Selects the axolotls form in this typed Minecraft component schema.", Creature = "Selects the creature form in this typed Minecraft component schema.", Misc = "Selects the misc form in this typed Minecraft component schema.", Monster = "Selects the monster form in this typed Minecraft component schema.", UndergroundWaterCreature = "Selects the underground water creature form in this typed Minecraft component schema.", WaterAmbient = "Selects the water ambient form in this typed Minecraft component schema.", WaterCreature = "Selects the water creature form in this typed Minecraft component schema."),
)]
/// A vanilla mob category used as a spawn-override key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MobCategory {
    #[doc = "Selects the monster form in this typed Minecraft component schema."]
    Monster,
    #[doc = "Selects the creature form in this typed Minecraft component schema."]
    Creature,
    #[doc = "Selects the ambient form in this typed Minecraft component schema."]
    Ambient,
    #[doc = "Selects the axolotls form in this typed Minecraft component schema."]
    Axolotls,
    #[doc = "Selects the underground water creature form in this typed Minecraft component schema."]
    UndergroundWaterCreature,
    #[doc = "Selects the water creature form in this typed Minecraft component schema."]
    WaterCreature,
    #[doc = "Selects the water ambient form in this typed Minecraft component schema."]
    WaterAmbient,
    #[doc = "Selects the misc form in this typed Minecraft component schema."]
    Misc,
}

impl MobCategory {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::MobCategory::as_str",
        aliases = ["sand::prelude::MobCategory::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(mob_category_value: &sand::component::MobCategory)  {\n    let as_str = mob_category_value.as_str();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SpawnBoundingBox",
    aliases = ["sand::prelude::SpawnBoundingBox"],
    module = "sand::component",
    summary = "Which part of a structure a spawn override applies to.",
    context = "Which part of a structure a spawn override applies to. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SpawnBoundingBox;",
    variants(Full = "The structure's full bounding box.", Piece = "Only inside individual structure pieces."),
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnBoundingBox::as_str",
        aliases = ["sand::prelude::SpawnBoundingBox::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(spawn_bounding_box_value: &sand::component::SpawnBoundingBox)  {\n    let as_str = spawn_bounding_box_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Piece => "piece",
            Self::Full => "full",
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SpawnEntry",
    aliases = ["sand::prelude::SpawnEntry"],
    module = "sand::component",
    summary = "One weighted mob-spawn entry inside a [`SpawnOverride`].",
    context = "One weighted mob-spawn entry inside a [`SpawnOverride`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SpawnEntry;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnEntry::new",
        aliases = ["sand::prelude::SpawnEntry::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export.",
        context = "Create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entity_type = "`entity_type` provides the typed Minecraft resource identifier used to create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export.", weight = "Create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export.", min_count = "Create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export.", max_count = "Create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export."),
        returns = "A newly constructed `SpawnEntry` configured to create a spawn entry. `weight` and `min_count` must be at least 1 and `max_count` must be at least `min_count`; both are checked on export.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entity_type: sand::registry::EntityTypeId, weight: u32, min_count: u32, max_count: u32)  {\n    let spawn_entry = sand::component::SpawnEntry::new(entity_type, weight, min_count, max_count);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SpawnOverride",
    aliases = ["sand::prelude::SpawnOverride"],
    module = "sand::component",
    summary = "A per-mob-category spawn override for a structure.",
    context = "A per-mob-category spawn override for a structure. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SpawnOverride;",
)]
/// A per-mob-category spawn override for a structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnOverride {
    bounding_box: SpawnBoundingBox,
    spawns: Vec<SpawnEntry>,
}

impl SpawnOverride {
    /// An override that suppresses all spawns of its category.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnOverride::none",
        aliases = ["sand::prelude::SpawnOverride::none"],
        module = "sand::component",
        kind = "method",
        summary = "An override that suppresses all spawns of its category.",
        context = "An override that suppresses all spawns of its category. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(bounding_box = "`bounding_box` supplies the bounding box value used to use an override that suppresses all spawns of its category."),
        returns = "A newly constructed `SpawnOverride` configured to use an override that suppresses all spawns of its category.",
        example = "use sand::prelude::*;\n\nfn demonstrate(bounding_box: sand::component::SpawnBoundingBox)  {\n    let spawn_override = sand::component::SpawnOverride::none(bounding_box);\n}",
    )]
    pub fn none(bounding_box: SpawnBoundingBox) -> Self {
        Self {
            bounding_box,
            spawns: Vec::new(),
        }
    }

    /// An override with an explicit spawn list.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnOverride::new",
        aliases = ["sand::prelude::SpawnOverride::new"],
        module = "sand::component",
        kind = "method",
        summary = "An override with an explicit spawn list.",
        context = "An override with an explicit spawn list. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(bounding_box = "`bounding_box` supplies the bounding box value used to use an override with an explicit spawn list.", spawns = "`spawns` supplies the spawns value used to use an override with an explicit spawn list."),
        returns = "A newly constructed `SpawnOverride` configured to use an override with an explicit spawn list.",
        example = "use sand::prelude::*;\n\nfn demonstrate(bounding_box: sand::component::SpawnBoundingBox, spawns: impl IntoIterator < Item = sand::component::SpawnEntry >)  {\n    let spawn_override = sand::component::SpawnOverride::new(bounding_box, spawns);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpawnOverride::spawn",
        aliases = ["sand::prelude::SpawnOverride::spawn"],
        module = "sand::component",
        kind = "method",
        summary = "Append a spawn entry.",
        context = "Append a spawn entry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entry = "`entry` supplies the entry value used to append a spawn entry."),
        returns = "The `SpawnOverride` value with the documented change applied to append a spawn entry.",
        example = "use sand::prelude::*;\n\nfn demonstrate(spawn_override_value: sand::component::SpawnOverride, entry: sand::component::SpawnEntry)  {\n    let updated_spawn_override = spawn_override_value.spawn(entry);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::BiomeSelector",
    aliases = ["sand::prelude::BiomeSelector"],
    module = "sand::component",
    summary = "The biome constraint of a structure: a biome tag or an explicit list.",
    context = "The biome constraint of a structure: a biome tag or an explicit list. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::BiomeSelector;",
    variants(Entries = "An explicit list of biome IDs.", Tag = "A biome tag reference, emitted as `#namespace:path`."),
    variant_fields(Entries = ["An explicit list of biome IDs."], Tag = ["A biome tag reference, emitted as `#namespace:path`."]),
)]
/// The biome constraint of a structure: a biome tag or an explicit list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiomeSelector {
    /// A biome tag reference, emitted as `#namespace:path`.
    Tag(#[doc = "A biome tag reference, emitted as `#namespace:path`."] TagId<BiomeId>),
    /// An explicit list of biome IDs.
    Entries(#[doc = "An explicit list of biome IDs."] Vec<BiomeId>),
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::JigsawConfig",
    aliases = ["sand::prelude::JigsawConfig"],
    module = "sand::component",
    summary = "The jigsaw-specific configuration of a `minecraft:jigsaw` structure.",
    context = "The jigsaw-specific configuration of a `minecraft:jigsaw` structure. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::JigsawConfig;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::new",
        aliases = ["sand::prelude::JigsawConfig::new"],
        module = "sand::component",
        kind = "method",
        summary = "A jigsaw config with vanilla village-like defaults.",
        context = "A jigsaw config with vanilla village-like defaults. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(start_pool = "`start_pool` provides the typed Minecraft resource identifier used to use a jigsaw config with vanilla village-like defaults."),
        returns = "A newly constructed `JigsawConfig` configured to use a jigsaw config with vanilla village-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(start_pool: sand::registry::TemplatePoolId)  {\n    let jigsaw_config = sand::component::JigsawConfig::new(start_pool);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::size",
        aliases = ["sand::prelude::JigsawConfig::size"],
        module = "sand::component",
        kind = "method",
        summary = "Jigsaw expansion depth (`0..=20`).",
        context = "Jigsaw expansion depth (`0..=20`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(size = "`size` supplies the size value used to jigsaw expansion depth (`0..=20`)."),
        returns = "The `JigsawConfig` value with the documented change applied to jigsaw expansion depth (`0..=20`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, size: u32)  {\n    let updated_jigsaw_config = jigsaw_config_value.size(size);\n}",
    )]
    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    /// Sets the Minecraft start height property on this typed jigsaw config definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::start_height",
        aliases = ["sand::prelude::JigsawConfig::start_height"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft start height property on this typed jigsaw config definition and returns the updated builder.",
        context = "Sets the Minecraft start height property on this typed jigsaw config definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(start_height = "`start_height` supplies the start height value used to set the Minecraft start height property on this typed jigsaw config definition and returns the updated builder."),
        returns = "Sets the Minecraft start height property on this typed jigsaw config definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, start_height: sand::component::HeightProvider)  {\n    let updated_jigsaw_config = jigsaw_config_value.start_height(start_height);\n}",
    )]
    pub fn start_height(mut self, start_height: HeightProvider) -> Self {
        self.start_height = start_height;
        self
    }

    /// Sets the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::start_jigsaw_name",
        aliases = ["sand::prelude::JigsawConfig::start_jigsaw_name"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder.",
        context = "Sets the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(name = "`name` provides the author-visible text value used to set the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder."),
        returns = "Sets the Minecraft start jigsaw name property on this typed jigsaw config definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, name: impl Into < String >)  {\n    let updated_jigsaw_config = jigsaw_config_value.start_jigsaw_name(name);\n}",
    )]
    pub fn start_jigsaw_name(mut self, name: impl Into<String>) -> Self {
        self.start_jigsaw_name = Some(name.into());
        self
    }

    /// Sets the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::project_start_to_heightmap",
        aliases = ["sand::prelude::JigsawConfig::project_start_to_heightmap"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder.",
        context = "Sets the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(heightmap = "`heightmap` supplies the heightmap value used to set the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder."),
        returns = "Sets the Minecraft project start to heightmap property on this typed jigsaw config definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, heightmap: sand::component::Heightmap)  {\n    let updated_jigsaw_config = jigsaw_config_value.project_start_to_heightmap(heightmap);\n}",
    )]
    pub fn project_start_to_heightmap(mut self, heightmap: Heightmap) -> Self {
        self.project_start_to_heightmap = Some(heightmap);
        self
    }

    /// Maximum horizontal distance pieces may extend from the start (`1..=128`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::max_distance_from_center",
        aliases = ["sand::prelude::JigsawConfig::max_distance_from_center"],
        module = "sand::component",
        kind = "method",
        summary = "Maximum horizontal distance pieces may extend from the start (`1..=128`).",
        context = "Maximum horizontal distance pieces may extend from the start (`1..=128`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(blocks = "`blocks` supplies the blocks value used to maximum horizontal distance pieces may extend from the start (`1..=128`)."),
        returns = "The `JigsawConfig` value with the documented change applied to maximum horizontal distance pieces may extend from the start (`1..=128`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, blocks: u32)  {\n    let updated_jigsaw_config = jigsaw_config_value.max_distance_from_center(blocks);\n}",
    )]
    pub fn max_distance_from_center(mut self, blocks: u32) -> Self {
        self.max_distance_from_center = blocks;
        self
    }

    /// Sets the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::JigsawConfig::use_expansion_hack",
        aliases = ["sand::prelude::JigsawConfig::use_expansion_hack"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder.",
        context = "Sets the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder."),
        returns = "Sets the Minecraft use expansion hack property on this typed jigsaw config definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(jigsaw_config_value: sand::component::JigsawConfig, value: bool)  {\n    let updated_jigsaw_config = jigsaw_config_value.use_expansion_hack(value);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Structure",
    aliases = ["sand::prelude::Structure"],
    module = "sand::component",
    summary = "A structure definition (`data/<namespace>/worldgen/structure/<id>.json`).",
    context = "A structure definition (`data/<namespace>/worldgen/structure/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Structure;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::new",
        aliases = ["sand::prelude::Structure::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a structure of an arbitrary typed structure type.",
        context = "Create a structure of an arbitrary typed structure type. Non-jigsaw vanilla structure types carry type-specific fields that Sand does not model; supply those through [`Structure::raw_field`].",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a structure of an arbitrary typed structure type.", structure_type = "`structure_type` provides the typed Minecraft resource identifier used to create a structure of an arbitrary typed structure type.", biomes = "`biomes` provides the Minecraft target selection used to create a structure of an arbitrary typed structure type."),
        returns = "A newly constructed `Structure` configured to create a structure of an arbitrary typed structure type.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, structure_type: sand::registry::StructureTypeId, biomes: sand::component::BiomeSelector)  {\n    let structure = sand::component::Structure::new(location, structure_type, biomes);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::jigsaw",
        aliases = ["sand::prelude::Structure::jigsaw"],
        module = "sand::component",
        kind = "method",
        summary = "Create a complete `minecraft:jigsaw` structure with vanilla-like defaults.",
        context = "Create a complete `minecraft:jigsaw` structure with vanilla-like defaults. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a complete `minecraft:jigsaw` structure with vanilla-like defaults.", start_pool = "`start_pool` provides the typed Minecraft resource identifier used to create a complete `minecraft:jigsaw` structure with vanilla-like defaults.", biomes = "`biomes` provides the Minecraft target selection used to create a complete `minecraft:jigsaw` structure with vanilla-like defaults."),
        returns = "A newly constructed `Structure` configured to create a complete `minecraft:jigsaw` structure with vanilla-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, start_pool: sand::registry::TemplatePoolId, biomes: sand::component::BiomeSelector)  {\n    let structure = sand::component::Structure::jigsaw(location, start_pool, biomes);\n}",
    )]
    pub fn jigsaw(
        location: ResourceLocation,
        start_pool: TemplatePoolId,
        biomes: BiomeSelector,
    ) -> Self {
        Self::new(location, StructureTypeId::jigsaw(), biomes)
            .jigsaw_config(JigsawConfig::new(start_pool))
    }

    /// Sets the Minecraft structure type property on this typed structure definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::structure_type",
        aliases = ["sand::prelude::Structure::structure_type"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft structure type property on this typed structure definition and returns the updated builder.",
        context = "Sets the Minecraft structure type property on this typed structure definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(structure_type = "`structure_type` provides the typed Minecraft resource identifier used to set the Minecraft structure type property on this typed structure definition and returns the updated builder."),
        returns = "Sets the Minecraft structure type property on this typed structure definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, structure_type: sand::registry::StructureTypeId)  {\n    let updated_structure = structure_value.structure_type(structure_type);\n}",
    )]
    pub fn structure_type(mut self, structure_type: StructureTypeId) -> Self {
        self.structure_type = structure_type;
        self
    }

    /// Sets the Minecraft biomes property on this typed structure definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::biomes",
        aliases = ["sand::prelude::Structure::biomes"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft biomes property on this typed structure definition and returns the updated builder.",
        context = "Sets the Minecraft biomes property on this typed structure definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(biomes = "`biomes` provides the Minecraft target selection used to set the Minecraft biomes property on this typed structure definition and returns the updated builder."),
        returns = "Sets the Minecraft biomes property on this typed structure definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, biomes: sand::component::BiomeSelector)  {\n    let updated_structure = structure_value.biomes(biomes);\n}",
    )]
    pub fn biomes(mut self, biomes: BiomeSelector) -> Self {
        self.biomes = biomes;
        self
    }

    /// Sets the Minecraft step property on this typed structure definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::step",
        aliases = ["sand::prelude::Structure::step"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft step property on this typed structure definition and returns the updated builder.",
        context = "Sets the Minecraft step property on this typed structure definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(step = "`step` supplies the step value used to set the Minecraft step property on this typed structure definition and returns the updated builder."),
        returns = "Sets the Minecraft step property on this typed structure definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, step: sand::component::GenerationStep)  {\n    let updated_structure = structure_value.step(step);\n}",
    )]
    pub fn step(mut self, step: GenerationStep) -> Self {
        self.step = step;
        self
    }

    /// Sets the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::terrain_adaptation",
        aliases = ["sand::prelude::Structure::terrain_adaptation"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder.",
        context = "Sets the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(adaptation = "`adaptation` supplies the adaptation value used to set the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder."),
        returns = "Sets the Minecraft terrain adaptation property on this typed structure definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, adaptation: sand::component::TerrainAdaptation)  {\n    let updated_structure = structure_value.terrain_adaptation(adaptation);\n}",
    )]
    pub fn terrain_adaptation(mut self, adaptation: TerrainAdaptation) -> Self {
        self.terrain_adaptation = Some(adaptation);
        self
    }

    /// Replace the jigsaw configuration.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::jigsaw_config",
        aliases = ["sand::prelude::Structure::jigsaw_config"],
        module = "sand::component",
        kind = "method",
        summary = "Replace the jigsaw configuration.",
        context = "Replace the jigsaw configuration. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(config = "`config` supplies the config value used to replace the jigsaw configuration."),
        returns = "The `Structure` value with the documented change applied to replace the jigsaw configuration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, config: sand::component::JigsawConfig)  {\n    let updated_structure = structure_value.jigsaw_config(config);\n}",
    )]
    pub fn jigsaw_config(mut self, config: JigsawConfig) -> Self {
        self.jigsaw = Some(config);
        self
    }

    /// Modify the jigsaw configuration in place, if one is present.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::map_jigsaw_config",
        aliases = ["sand::prelude::Structure::map_jigsaw_config"],
        module = "sand::component",
        kind = "method",
        summary = "Modify the jigsaw configuration in place, if one is present.",
        context = "Modify the jigsaw configuration in place, if one is present. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(f = "`f` supplies the f value used to modify the jigsaw configuration in place, if one is present."),
        returns = "The `Structure` value with the documented change applied to modify the jigsaw configuration in place, if one is present.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, f: impl FnOnce (sand::component::JigsawConfig) -> sand::component::JigsawConfig)  {\n    let updated_structure = structure_value.map_jigsaw_config(f);\n}",
    )]
    pub fn map_jigsaw_config(mut self, f: impl FnOnce(JigsawConfig) -> JigsawConfig) -> Self {
        self.jigsaw = self.jigsaw.map(f);
        self
    }

    /// Add or replace a spawn override for one mob category.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::spawn_override",
        aliases = ["sand::prelude::Structure::spawn_override"],
        module = "sand::component",
        kind = "method",
        summary = "Add or replace a spawn override for one mob category.",
        context = "Add or replace a spawn override for one mob category. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(category = "`category` supplies the category value used to add or replace a spawn override for one mob category.", spawns = "`spawns` supplies the spawns value used to add or replace a spawn override for one mob category."),
        returns = "The `Structure` value with the documented change applied to add or replace a spawn override for one mob category.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, category: sand::component::MobCategory, spawns: sand::component::SpawnOverride)  {\n    let updated_structure = structure_value.spawn_override(category, spawns);\n}",
    )]
    pub fn spawn_override(mut self, category: MobCategory, spawns: SpawnOverride) -> Self {
        self.spawn_overrides.insert(category, spawns);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Structure::raw_field",
        aliases = ["sand::prelude::Structure::raw_field"],
        module = "sand::component",
        kind = "method",
        summary = "Add a modded or version-specific field not represented by the typed API.",
        context = "Add a modded or version-specific field not represented by the typed API. Typed field names cannot be overridden through this escape hatch.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to add a modded or version-specific field not represented by the typed API.", value = "`value` provides the value being applied or compared used to add a modded or version-specific field not represented by the typed API."),
        returns = "The `Structure` value with the documented change applied to add a modded or version-specific field not represented by the typed API.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_value: sand::component::Structure, key: impl Into < String >, value: sand::component::RawJson)  {\n    let updated_structure = structure_value.raw_field(key, value);\n}",
    )]
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
