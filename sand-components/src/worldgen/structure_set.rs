//! Builder for `data/<namespace>/worldgen/structure_set/<id>.json`.
//!
//! [`StructureSet::random_spread`] and [`StructureSet::concentric_rings`]
//! cover the two vanilla placement strategies. All stable fields have typed
//! setters, while [`StructureSet::raw_field`] is an explicit escape hatch for
//! modded or version-specific additions.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::StructureId;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/structure_set";

const TYPED_FIELDS: &[&str] = &["structures", "placement"];

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::StructureEntry",
    aliases = ["sand::prelude::StructureEntry"],
    module = "sand::component",
    summary = "A weighted `worldgen/structure` reference inside a structure set.",
    context = "A weighted `worldgen/structure` reference inside a structure set. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::StructureEntry;",
)]
/// A weighted `worldgen/structure` reference inside a structure set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureEntry {
    structure: StructureId,
    weight: u32,
}

impl StructureEntry {
    /// `weight` must be at least 1; checked on export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureEntry::new",
        aliases = ["sand::prelude::StructureEntry::new"],
        module = "sand::component",
        kind = "method",
        summary = "`weight` must be at least 1; checked on export.",
        context = "`weight` must be at least 1; checked on export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(structure = "`structure` provides the typed Minecraft resource identifier used to emit the documented `weight` must be at least 1; checked on export form.", weight = "`weight` must be at least 1; checked on export."),
        returns = "A `StructureEntry` that emits the documented `weight` must be at least 1; checked on export form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure: sand::registry::StructureId, weight: u32)  {\n    let structure_entry = sand::component::StructureEntry::new(structure, weight);\n}",
    )]
    pub fn new(structure: StructureId, weight: u32) -> Self {
        Self { structure, weight }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "structure": self.structure.to_string(),
            "weight": self.weight,
        })
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            KIND,
            &format!("{field}.structure"),
            &self.structure.to_string(),
        )?;
        if self.weight == 0 {
            return Err(validation::error(
                location,
                KIND,
                &format!("{field}.weight"),
                "structure weight must be at least 1",
            ));
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ExclusionZone",
    aliases = ["sand::prelude::ExclusionZone"],
    module = "sand::component",
    summary = "The exclusion-zone chunk count that another structure set must keep clear around a placement.",
    context = "The exclusion-zone chunk count that another structure set must keep clear around a placement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ExclusionZone;",
)]
/// The exclusion-zone chunk count that another structure set must keep clear
/// around a placement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionZone {
    other_set: String,
    chunk_count: u32,
}

impl ExclusionZone {
    /// `other_set` is the raw resource-location string of the other
    /// structure set (kept as a string since vanilla exclusion zones may
    /// reference sets defined outside the current pack). `chunk_count` must
    /// be at least 1.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ExclusionZone::new",
        aliases = ["sand::prelude::ExclusionZone::new"],
        module = "sand::component",
        kind = "method",
        summary = "`other_set` is the raw resource-location string of the other structure set (kept as a string since vanilla exclusion zones may reference sets defined outside the current pack). `chunk_count` must be at least 1.",
        context = "`other_set` is the raw resource-location string of the other structure set (kept as a string since vanilla exclusion zones may reference sets defined outside the current pack). `chunk_count` must be at least 1. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(other_set = "`other_set` is the raw resource-location string of the other structure set (kept as a string since vanilla exclusion zones may reference sets defined outside the current pack). `chunk_count` must be at least 1.", chunk_count = "`other_set` is the raw resource-location string of the other structure set (kept as a string since vanilla exclusion zones may reference sets defined outside the current pack). `chunk_count` must be at least 1."),
        returns = "An `ExclusionZone` that emits the documented `other_set` is the raw resource-location string of the other structure set (kept as a string since vanilla exclusion zones may reference sets defined outside the current pack). `chunk_count` must be at least 1 form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(other_set: impl Into < String >, chunk_count: u32)  {\n    let exclusion_zone = sand::component::ExclusionZone::new(other_set, chunk_count);\n}",
    )]
    pub fn new(other_set: impl Into<String>, chunk_count: u32) -> Self {
        Self {
            other_set: other_set.into(),
            chunk_count,
        }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "other_set": self.other_set,
            "chunk_count": self.chunk_count,
        })
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            KIND,
            "placement.exclusion_zone.other_set",
            &self.other_set,
        )?;
        if self.chunk_count == 0 {
            return Err(validation::error(
                location,
                KIND,
                "placement.exclusion_zone.chunk_count",
                "chunk_count must be at least 1",
            ));
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::FrequencyReductionMethod",
    aliases = ["sand::prelude::FrequencyReductionMethod"],
    module = "sand::component",
    summary = "The frequency reduction curve for random-spread placement.",
    context = "The frequency reduction curve for random-spread placement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::FrequencyReductionMethod;",
    variants(Default = "Uses Minecraft's default frequency-reduction algorithm.", LegacyType1 = "Uses Minecraft's legacy type1 frequency-reduction algorithm.", LegacyType2 = "Uses Minecraft's legacy type2 frequency-reduction algorithm.", LegacyType3 = "Uses Minecraft's legacy type3 frequency-reduction algorithm."),
)]
/// The frequency reduction curve for random-spread placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyReductionMethod {
    #[doc = "Uses Minecraft's default frequency-reduction algorithm."]
    Default,
    #[doc = "Uses Minecraft's legacy type1 frequency-reduction algorithm."]
    LegacyType1,
    #[doc = "Uses Minecraft's legacy type2 frequency-reduction algorithm."]
    LegacyType2,
    #[doc = "Uses Minecraft's legacy type3 frequency-reduction algorithm."]
    LegacyType3,
}

impl FrequencyReductionMethod {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::FrequencyReductionMethod::as_str",
        aliases = ["sand::prelude::FrequencyReductionMethod::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(frequency_reduction_method_value: &sand::component::FrequencyReductionMethod)  {\n    let as_str = frequency_reduction_method_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::LegacyType1 => "legacy_type_1",
            Self::LegacyType2 => "legacy_type_2",
            Self::LegacyType3 => "legacy_type_3",
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::SpreadType",
    aliases = ["sand::prelude::SpreadType"],
    module = "sand::component",
    summary = "How random-spread candidate chunks are chosen inside each spacing cell.",
    context = "How random-spread candidate chunks are chosen inside each spacing cell. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::SpreadType;",
    variants(Linear = "Distributes candidate chunks with Minecraft's linear spread.", Triangular = "Distributes candidate chunks with Minecraft's triangular spread."),
)]
/// How random-spread candidate chunks are chosen inside each spacing cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpreadType {
    #[doc = "Distributes candidate chunks with Minecraft's linear spread."]
    Linear,
    #[doc = "Distributes candidate chunks with Minecraft's triangular spread."]
    Triangular,
}

impl SpreadType {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::SpreadType::as_str",
        aliases = ["sand::prelude::SpreadType::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(spread_type_value: &sand::component::SpreadType)  {\n    let as_str = spread_type_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Triangular => "triangular",
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::StructurePlacement",
    aliases = ["sand::prelude::StructurePlacement"],
    module = "sand::component",
    summary = "A structure-set placement strategy.",
    context = "A structure-set placement strategy. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::StructurePlacement;",
    variants(ConcentricRings = "`minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.", RandomSpread = "`minecraft:random_spread` — an evenly distributed grid with jitter."),
    variant_fields(ConcentricRings(count = "`count` provides the count when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.", distance = "`distance` provides the distance when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.", preferred_biomes = "`preferred_biomes` optionally provides the preferred biomes when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.", spread = "`spread` provides the spread when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds."), RandomSpread(frequency = "`frequency` optionally supplies the placement frequency for this random-spread strategy.", frequency_reduction_method = "`frequency_reduction_method` optionally provides the frequency reduction method when `minecraft:random_spread` — an evenly distributed grid with jitter.", salt = "`salt` provides the salt when `minecraft:random_spread` — an evenly distributed grid with jitter.", separation = "`separation` provides the separation when `minecraft:random_spread` — an evenly distributed grid with jitter.", spacing = "`spacing` provides the spacing when `minecraft:random_spread` — an evenly distributed grid with jitter.", spread_type = "`spread_type` provides the spread type when `minecraft:random_spread` — an evenly distributed grid with jitter.")),
)]
/// A structure-set placement strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum StructurePlacement {
    /// `minecraft:random_spread` — an evenly distributed grid with jitter.
    RandomSpread {
        /// `spacing` provides the spacing when `minecraft:random_spread` — an evenly distributed grid with jitter.
        spacing: u32,
        /// `separation` provides the separation when `minecraft:random_spread` — an evenly distributed grid with jitter.
        separation: u32,
        /// `salt` provides the salt when `minecraft:random_spread` — an evenly distributed grid with jitter.
        salt: i32,
        /// `spread_type` provides the spread type when `minecraft:random_spread` — an evenly distributed grid with jitter.
        spread_type: SpreadType,
        /// `frequency` optionally supplies the placement frequency for this random-spread strategy.
        frequency: Option<f32>,
        /// `frequency_reduction_method` optionally provides the frequency reduction method when `minecraft:random_spread` — an evenly distributed grid with jitter.
        frequency_reduction_method: Option<FrequencyReductionMethod>,
    },
    /// `minecraft:concentric_rings` — rings of candidate chunks around the
    /// world origin, as used by strongholds.
    ConcentricRings {
        /// `distance` provides the distance when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.
        distance: u32,
        /// `spread` provides the spread when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.
        spread: u32,
        /// `count` provides the count when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.
        count: u32,
        /// `preferred_biomes` optionally provides the preferred biomes when `minecraft:concentric_rings` — rings of candidate chunks around the world origin, as used by strongholds.
        preferred_biomes: Option<Vec<String>>,
    },
}

impl StructurePlacement {
    /// A random-spread placement with vanilla village-like defaults.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructurePlacement::random_spread",
        aliases = ["sand::prelude::StructurePlacement::random_spread"],
        module = "sand::component",
        kind = "method",
        summary = "A random-spread placement with vanilla village-like defaults.",
        context = "A random-spread placement with vanilla village-like defaults. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(spacing = "`spacing` sets the spacing for a random-spread placement with vanilla village-like defaults.", separation = "`separation` sets the separation for a random-spread placement with vanilla village-like defaults.", salt = "`salt` sets the salt for a random-spread placement with vanilla village-like defaults."),
        returns = "A `StructurePlacement` configured for a random-spread placement with vanilla village-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(spacing: u32, separation: u32, salt: i32)  {\n    let structure_placement = sand::component::StructurePlacement::random_spread(spacing, separation, salt);\n}",
    )]
    pub fn random_spread(spacing: u32, separation: u32, salt: i32) -> Self {
        Self::RandomSpread {
            spacing,
            separation,
            salt,
            spread_type: SpreadType::Linear,
            frequency: None,
            frequency_reduction_method: None,
        }
    }

    /// A concentric-rings placement with vanilla stronghold-like defaults.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructurePlacement::concentric_rings",
        aliases = ["sand::prelude::StructurePlacement::concentric_rings"],
        module = "sand::component",
        kind = "method",
        summary = "A concentric-rings placement with vanilla stronghold-like defaults.",
        context = "A concentric-rings placement with vanilla stronghold-like defaults. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(distance = "`distance` sets the distance for a concentric-rings placement with vanilla stronghold-like defaults.", spread = "`spread` sets the spread for a concentric-rings placement with vanilla stronghold-like defaults.", count = "`count` provides the requested numeric amount used to use a concentric-rings placement with vanilla stronghold-like defaults."),
        returns = "A `StructurePlacement` configured for a concentric-rings placement with vanilla stronghold-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(distance: u32, spread: u32, count: u32)  {\n    let structure_placement = sand::component::StructurePlacement::concentric_rings(distance, spread, count);\n}",
    )]
    pub fn concentric_rings(distance: u32, spread: u32, count: u32) -> Self {
        Self::ConcentricRings {
            distance,
            spread,
            count,
            preferred_biomes: None,
        }
    }

    fn common_placement_type(&self) -> &'static str {
        match self {
            Self::RandomSpread { .. } => "minecraft:random_spread",
            Self::ConcentricRings { .. } => "minecraft:concentric_rings",
        }
    }

    fn write_fields(&self, map: &mut Map<String, Value>) {
        match self {
            Self::RandomSpread {
                spacing,
                separation,
                salt,
                spread_type,
                frequency,
                frequency_reduction_method,
            } => {
                map.insert("spacing".into(), (*spacing).into());
                map.insert("separation".into(), (*separation).into());
                map.insert("salt".into(), (*salt).into());
                map.insert(
                    "spread_type".into(),
                    Value::String(spread_type.as_str().to_string()),
                );
                if let Some(frequency) = frequency {
                    map.insert("frequency".into(), serde_json::json!(frequency));
                }
                if let Some(method) = frequency_reduction_method {
                    map.insert(
                        "frequency_reduction_method".into(),
                        Value::String(method.as_str().to_string()),
                    );
                }
            }
            Self::ConcentricRings {
                distance,
                spread,
                count,
                preferred_biomes,
            } => {
                map.insert("distance".into(), (*distance).into());
                map.insert("spread".into(), (*spread).into());
                map.insert("count".into(), (*count).into());
                if let Some(biomes) = preferred_biomes {
                    map.insert(
                        "preferred_biomes".into(),
                        Value::Array(biomes.iter().cloned().map(Value::String).collect()),
                    );
                }
            }
        }
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        match self {
            Self::RandomSpread {
                spacing,
                separation,
                frequency,
                ..
            } => {
                if *separation >= *spacing {
                    return Err(validation::error(
                        location,
                        KIND,
                        "placement.separation",
                        &format!(
                            "separation must be less than spacing; received separation={separation}, spacing={spacing}"
                        ),
                    ));
                }
                validation::require_u32_in_range(
                    location,
                    KIND,
                    "placement.spacing",
                    *spacing,
                    0,
                    4096,
                )?;
                if let Some(frequency) = frequency {
                    validation::require_finite_f32(
                        location,
                        KIND,
                        "placement.frequency",
                        *frequency,
                    )?;
                    if !(0.0..=1.0).contains(frequency) {
                        return Err(validation::error(
                            location,
                            KIND,
                            "placement.frequency",
                            &format!("frequency must be in 0..=1; received {frequency}"),
                        ));
                    }
                }
            }
            Self::ConcentricRings {
                distance,
                spread,
                count,
                preferred_biomes,
            } => {
                if *distance == 0 {
                    return Err(validation::error(
                        location,
                        KIND,
                        "placement.distance",
                        "distance must be at least 1",
                    ));
                }
                if *count == 0 {
                    return Err(validation::error(
                        location,
                        KIND,
                        "placement.count",
                        "count must be at least 1",
                    ));
                }
                let _ = spread;
                if let Some(biomes) = preferred_biomes {
                    validation::require_non_empty_collection(
                        location,
                        KIND,
                        "placement.preferred_biomes",
                        biomes.len(),
                    )?;
                    for (index, biome) in biomes.iter().enumerate() {
                        validation::validate_resource_or_tag_location_str(
                            location,
                            KIND,
                            &format!("placement.preferred_biomes[{index}]"),
                            biome,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::StructureSet",
    aliases = ["sand::prelude::StructureSet"],
    module = "sand::component",
    summary = "A structure set definition (`data/<namespace>/worldgen/structure_set/<id>.json`).",
    context = "A structure set definition (`data/<namespace>/worldgen/structure_set/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::StructureSet;",
)]
/// A structure set definition (`data/<namespace>/worldgen/structure_set/<id>.json`).
///
/// ```
/// use sand_components::{DatapackComponent, ResourceLocation, StructureId};
/// use sand_components::worldgen::structure_set::{StructureEntry, StructurePlacement, StructureSet};
///
/// let set = StructureSet::new(
///     ResourceLocation::new("example", "villages").unwrap(),
///     [StructureEntry::new(StructureId::minecraft("village_plains").unwrap(), 1)],
///     StructurePlacement::random_spread(34, 8, 10387312),
/// );
/// set.validate().unwrap();
/// assert_eq!(set.component_dir(), "worldgen/structure_set");
/// assert_eq!(set.to_json()["placement"]["type"], "minecraft:random_spread");
/// ```
pub struct StructureSet {
    location: ResourceLocation,
    structures: Vec<StructureEntry>,
    placement: StructurePlacement,
    exclusion_zone: Option<ExclusionZone>,
    raw_fields: BTreeMap<String, RawJson>,
}

impl StructureSet {
    /// Create a structure set from an explicit weighted structure list and placement.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::new",
        aliases = ["sand::prelude::StructureSet::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a structure set from an explicit weighted structure list and placement.",
        context = "Create a structure set from an explicit weighted structure list and placement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a structure set from an explicit weighted structure list and placement.", structures = "`structures` is used when creating a structure set from an explicit weighted structure list and placement.", placement = "`placement` is used when creating a structure set from an explicit weighted structure list and placement."),
        returns = "A `StructureSet` representing a structure set from an explicit weighted structure list and placement.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, structures: impl IntoIterator < Item = sand::component::StructureEntry >, placement: sand::component::StructurePlacement)  {\n    let structure_set = sand::component::StructureSet::new(location, structures, placement);\n}",
    )]
    pub fn new(
        location: ResourceLocation,
        structures: impl IntoIterator<Item = StructureEntry>,
        placement: StructurePlacement,
    ) -> Self {
        Self {
            location,
            structures: structures.into_iter().collect(),
            placement,
            exclusion_zone: None,
            raw_fields: BTreeMap::new(),
        }
    }

    /// Convenience constructor for a single-structure random-spread set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::random_spread",
        aliases = ["sand::prelude::StructureSet::random_spread"],
        module = "sand::component",
        kind = "method",
        summary = "Convenience constructor for a single-structure random-spread set.",
        context = "Convenience constructor for a single-structure random-spread set. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use convenience constructor for a single-structure random-spread set.", structure = "`structure` provides the typed Minecraft resource identifier used to use convenience constructor for a single-structure random-spread set.", spacing = "`spacing` sets the spacing for convenience constructor for a single-structure random-spread set.", separation = "`separation` sets the separation for convenience constructor for a single-structure random-spread set.", salt = "`salt` sets the salt for convenience constructor for a single-structure random-spread set."),
        returns = "A `StructureSet` configured for convenience constructor for a single-structure random-spread set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, structure: sand::registry::StructureId, spacing: u32, separation: u32, salt: i32)  {\n    let structure_set = sand::component::StructureSet::random_spread(location, structure, spacing, separation, salt);\n}",
    )]
    pub fn random_spread(
        location: ResourceLocation,
        structure: StructureId,
        spacing: u32,
        separation: u32,
        salt: i32,
    ) -> Self {
        Self::new(
            location,
            [StructureEntry::new(structure, 1)],
            StructurePlacement::random_spread(spacing, separation, salt),
        )
    }

    /// Convenience constructor for a single-structure concentric-rings set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::concentric_rings",
        aliases = ["sand::prelude::StructureSet::concentric_rings"],
        module = "sand::component",
        kind = "method",
        summary = "Convenience constructor for a single-structure concentric-rings set.",
        context = "Convenience constructor for a single-structure concentric-rings set. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use convenience constructor for a single-structure concentric-rings set.", structure = "`structure` provides the typed Minecraft resource identifier used to use convenience constructor for a single-structure concentric-rings set.", distance = "`distance` sets the distance for convenience constructor for a single-structure concentric-rings set.", spread = "`spread` sets the spread for convenience constructor for a single-structure concentric-rings set.", count = "`count` provides the requested numeric amount used to use convenience constructor for a single-structure concentric-rings set."),
        returns = "A `StructureSet` configured for convenience constructor for a single-structure concentric-rings set.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, structure: sand::registry::StructureId, distance: u32, spread: u32, count: u32)  {\n    let structure_set = sand::component::StructureSet::concentric_rings(location, structure, distance, spread, count);\n}",
    )]
    pub fn concentric_rings(
        location: ResourceLocation,
        structure: StructureId,
        distance: u32,
        spread: u32,
        count: u32,
    ) -> Self {
        Self::new(
            location,
            [StructureEntry::new(structure, 1)],
            StructurePlacement::concentric_rings(distance, spread, count),
        )
    }

    /// Sets the Minecraft structures property on this typed structure set definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::structures",
        aliases = ["sand::prelude::StructureSet::structures"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft structures property on this typed structure set definition and returns the updated builder.",
        context = "Sets the Minecraft structures property on this typed structure set definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(structures = "`structures` provides the structures applied when setting the Minecraft structures property on this typed structure set definition and returns the updated builder."),
        returns = "Sets the Minecraft structures property on this typed structure set definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_set_value: sand::component::StructureSet, structures: impl IntoIterator < Item = sand::component::StructureEntry >)  {\n    let updated_structure_set = structure_set_value.structures(structures);\n}",
    )]
    pub fn structures(mut self, structures: impl IntoIterator<Item = StructureEntry>) -> Self {
        self.structures = structures.into_iter().collect();
        self
    }

    /// Sets the Minecraft add structure property on this typed structure set definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::add_structure",
        aliases = ["sand::prelude::StructureSet::add_structure"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft add structure property on this typed structure set definition and returns the updated builder.",
        context = "Sets the Minecraft add structure property on this typed structure set definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entry = "`entry` provides the entry applied when setting the Minecraft add structure property on this typed structure set definition and returns the updated builder."),
        returns = "Sets the Minecraft add structure property on this typed structure set definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_set_value: sand::component::StructureSet, entry: sand::component::StructureEntry)  {\n    let updated_structure_set = structure_set_value.add_structure(entry);\n}",
    )]
    pub fn add_structure(mut self, entry: StructureEntry) -> Self {
        self.structures.push(entry);
        self
    }

    /// Sets the Minecraft placement property on this typed structure set definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::placement",
        aliases = ["sand::prelude::StructureSet::placement"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft placement property on this typed structure set definition and returns the updated builder.",
        context = "Sets the Minecraft placement property on this typed structure set definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(placement = "`placement` provides the placement applied when setting the Minecraft placement property on this typed structure set definition and returns the updated builder."),
        returns = "Sets the Minecraft placement property on this typed structure set definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_set_value: sand::component::StructureSet, placement: sand::component::StructurePlacement)  {\n    let updated_structure_set = structure_set_value.placement(placement);\n}",
    )]
    pub fn placement(mut self, placement: StructurePlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Sets the Minecraft exclusion zone property on this typed structure set definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::exclusion_zone",
        aliases = ["sand::prelude::StructureSet::exclusion_zone"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft exclusion zone property on this typed structure set definition and returns the updated builder.",
        context = "Sets the Minecraft exclusion zone property on this typed structure set definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(zone = "`zone` provides the zone applied when setting the Minecraft exclusion zone property on this typed structure set definition and returns the updated builder."),
        returns = "Sets the Minecraft exclusion zone property on this typed structure set definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_set_value: sand::component::StructureSet, zone: sand::component::ExclusionZone)  {\n    let updated_structure_set = structure_set_value.exclusion_zone(zone);\n}",
    )]
    pub fn exclusion_zone(mut self, zone: ExclusionZone) -> Self {
        self.exclusion_zone = Some(zone);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::StructureSet::raw_field",
        aliases = ["sand::prelude::StructureSet::raw_field"],
        module = "sand::component",
        kind = "method",
        summary = "Add a modded or version-specific field not represented by the typed API.",
        context = "Add a modded or version-specific field not represented by the typed API. Typed field names cannot be overridden through this escape hatch.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to add a modded or version-specific field not represented by the typed API.", value = "`value` provides the value being applied or compared used to add a modded or version-specific field not represented by the typed API."),
        returns = "The `StructureSet` value with the documented change applied to add a modded or version-specific field not represented by the typed API.",
        example = "use sand::prelude::*;\n\nfn demonstrate(structure_set_value: sand::component::StructureSet, key: impl Into < String >, value: sand::component::RawJson)  {\n    let updated_structure_set = structure_set_value.raw_field(key, value);\n}",
    )]
    pub fn raw_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.raw_fields.insert(key.into(), value);
        self
    }
}

impl DatapackComponent for StructureSet {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        validation::require_non_empty_collection(
            &self.location,
            KIND,
            "structures",
            self.structures.len(),
        )?;
        for (index, entry) in self.structures.iter().enumerate() {
            entry.validate(&self.location, &format!("structures[{index}]"))?;
        }
        self.placement.validate(&self.location)?;
        if let Some(zone) = &self.exclusion_zone {
            zone.validate(&self.location)?;
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
            "structures".into(),
            Value::Array(
                self.structures
                    .iter()
                    .map(StructureEntry::to_json)
                    .collect(),
            ),
        );
        let mut placement = Map::new();
        placement.insert(
            "type".into(),
            Value::String(self.placement.common_placement_type().to_string()),
        );
        self.placement.write_fields(&mut placement);
        if let Some(zone) = &self.exclusion_zone {
            placement.insert("exclusion_zone".into(), zone.to_json());
        }
        map.insert("placement".into(), Value::Object(placement));
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/structure_set"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "villages").unwrap()
    }

    fn structure() -> StructureId {
        StructureId::minecraft("village_plains").unwrap()
    }

    #[test]
    fn random_spread_structure_set_matches_vanilla_shape() {
        let set = StructureSet::random_spread(location(), structure(), 34, 8, 10_387_312);
        set.validate().unwrap();
        assert_eq!(
            set.to_json(),
            serde_json::json!({
                "structures": [
                    { "structure": "minecraft:village_plains", "weight": 1 }
                ],
                "placement": {
                    "type": "minecraft:random_spread",
                    "spacing": 34,
                    "separation": 8,
                    "salt": 10387312,
                    "spread_type": "linear",
                }
            })
        );
        assert_eq!(set.component_dir(), "worldgen/structure_set");
    }

    #[test]
    fn concentric_rings_structure_set_matches_vanilla_shape() {
        let set = StructureSet::concentric_rings(
            location(),
            StructureId::minecraft("stronghold").unwrap(),
            32,
            3,
            128,
        );
        set.validate().unwrap();
        let json = set.to_json();
        assert_eq!(json["placement"]["type"], "minecraft:concentric_rings");
        assert_eq!(json["placement"]["distance"], 32);
        assert_eq!(json["placement"]["count"], 128);
    }

    #[test]
    fn full_typed_structure_set_serializes_optional_fields() {
        let set = StructureSet::new(
            location(),
            [
                StructureEntry::new(structure(), 2),
                StructureEntry::new(StructureId::minecraft("village_desert").unwrap(), 1),
            ],
            StructurePlacement::RandomSpread {
                spacing: 34,
                separation: 8,
                salt: 10_387_312,
                spread_type: SpreadType::Triangular,
                frequency: Some(0.5),
                frequency_reduction_method: Some(FrequencyReductionMethod::LegacyType1),
            },
        )
        .exclusion_zone(ExclusionZone::new("minecraft:strongholds", 10))
        .raw_field("example:note", RawJson::new(serde_json::json!("test")));
        set.validate().unwrap();
        let json = set.to_json();
        assert_eq!(json["placement"]["spread_type"], "triangular");
        assert_eq!(json["placement"]["frequency"], 0.5);
        assert_eq!(
            json["placement"]["frequency_reduction_method"],
            "legacy_type_1"
        );
        assert_eq!(
            json["placement"]["exclusion_zone"]["other_set"],
            "minecraft:strongholds"
        );
        assert_eq!(json["example:note"], "test");
    }

    #[test]
    fn empty_structure_list_is_rejected() {
        let set = StructureSet::new(
            location(),
            Vec::new(),
            StructurePlacement::random_spread(34, 8, 0),
        );
        assert!(set.validate().is_err());
    }

    #[test]
    fn zero_weight_entry_is_rejected() {
        let set = StructureSet::new(
            location(),
            [StructureEntry::new(structure(), 0)],
            StructurePlacement::random_spread(34, 8, 0),
        );
        assert!(set.validate().is_err());
    }

    #[test]
    fn separation_must_be_less_than_spacing() {
        let set = StructureSet::random_spread(location(), structure(), 10, 10, 0);
        let err = set.validate().unwrap_err().to_string();
        assert!(err.contains("separation"), "{err}");
    }

    #[test]
    fn zero_distance_or_count_concentric_rings_are_rejected() {
        assert!(
            StructureSet::concentric_rings(location(), structure(), 0, 3, 128)
                .validate()
                .is_err()
        );
        assert!(
            StructureSet::concentric_rings(location(), structure(), 32, 3, 0)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn raw_field_cannot_override_typed_field() {
        assert!(
            StructureSet::random_spread(location(), structure(), 34, 8, 0)
                .raw_field("structures", RawJson::new(serde_json::json!([])))
                .validate()
                .is_err()
        );
    }
}
