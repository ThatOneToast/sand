//! Builder for `data/<namespace>/dimension_type/<id>.json`.
//!
//! [`DimensionType::new`] provides a valid, familiar starting
//! point. All stable fields have typed setters, while [`DimensionType::raw_field`]
//! is an explicit escape hatch for modded or version-specific additions.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{BlockId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;

const TYPED_FIELDS: &[&str] = &[
    "fixed_time",
    "has_skylight",
    "has_ceiling",
    "ultrawarm",
    "natural",
    "coordinate_scale",
    "bed_works",
    "respawn_anchor_works",
    "min_y",
    "height",
    "logical_height",
    "infiniburn",
    "effects",
    "ambient_light",
    "piglin_safe",
    "has_raids",
    "monster_spawn_light_level",
    "monster_spawn_block_light_limit",
];

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::MonsterSpawnLightLevel",
    aliases = ["sand::prelude::MonsterSpawnLightLevel"],
    module = "sand::component",
    summary = "The sky-light range in which monsters may spawn.",
    context = "The sky-light range in which monsters may spawn. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::MonsterSpawnLightLevel;",
    variants(Constant = "A single light level.", Uniform = "A uniformly sampled inclusive light-level range."),
    variant_fields(Constant = ["A single light level."], Uniform(max_inclusive = "`max_inclusive` provides the max inclusive when a uniformly sampled inclusive light-level range.", min_inclusive = "`min_inclusive` provides the min inclusive when a uniformly sampled inclusive light-level range.")),
)]
/// The sky-light range in which monsters may spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterSpawnLightLevel {
    /// A single light level.
    Constant(#[doc = "A single light level."] u8),
    /// A uniformly sampled inclusive light-level range.
    Uniform {
        /// `min_inclusive` provides the min inclusive when a uniformly sampled inclusive light-level range.
        min_inclusive: u8,
        /// `max_inclusive` provides the max inclusive when a uniformly sampled inclusive light-level range.
        max_inclusive: u8,
    },
}

impl MonsterSpawnLightLevel {
    fn to_json(&self) -> Value {
        match self {
            Self::Constant(level) => serde_json::json!(level),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => serde_json::json!({
                "type": "minecraft:uniform",
                "value": {
                    "min_inclusive": min_inclusive,
                    "max_inclusive": max_inclusive,
                }
            }),
        }
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        let (min, max) = match self {
            Self::Constant(level) => (*level, *level),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => (*min_inclusive, *max_inclusive),
        };
        if max > 15 {
            return Err(validation::error(
                location,
                "dimension_type",
                "monster_spawn_light_level",
                &format!("light levels must be in 0..=15; received {min}..={max}"),
            ));
        }
        if min > max {
            return Err(validation::error(
                location,
                "dimension_type",
                "monster_spawn_light_level",
                &format!("min_inclusive must not exceed max_inclusive; received {min}..={max}"),
            ));
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DimensionType",
    aliases = ["sand::prelude::DimensionType"],
    module = "sand::component",
    summary = "A Minecraft dimension type definition. The constructor uses overworld-like defaults, producing a complete valid shape without requiring raw JSON:.",
    context = "A Minecraft dimension type definition. The constructor uses overworld-like defaults, producing a complete valid shape without requiring raw JSON:",
    minecraft = "The constructor uses overworld-like defaults, producing a complete valid shape without requiring raw JSON:",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DimensionType;",
)]
/// A Minecraft dimension type definition.
///
/// The constructor uses overworld-like defaults, producing a complete valid
/// shape without requiring raw JSON:
///
/// ```
/// use sand_components::{DatapackComponent, DimensionType, ResourceLocation};
///
/// let ty = DimensionType::new(
///     ResourceLocation::new("example", "bright_overworld").unwrap(),
/// );
/// assert_eq!(ty.component_dir(), "dimension_type");
/// assert_eq!(ty.to_json()["effects"], "minecraft:overworld");
/// ```
pub struct DimensionType {
    location: ResourceLocation,
    fixed_time: Option<i64>,
    has_skylight: bool,
    has_ceiling: bool,
    ultrawarm: bool,
    natural: bool,
    coordinate_scale: f64,
    bed_works: bool,
    respawn_anchor_works: bool,
    min_y: i32,
    height: u32,
    logical_height: u32,
    infiniburn: TagId<BlockId>,
    effects: ResourceLocation,
    ambient_light: f32,
    piglin_safe: bool,
    has_raids: bool,
    monster_spawn_light_level: MonsterSpawnLightLevel,
    monster_spawn_block_light_limit: u8,
    raw_fields: BTreeMap<String, RawJson>,
}

impl DimensionType {
    /// Create a complete dimension type with vanilla overworld-like defaults.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::new",
        aliases = ["sand::prelude::DimensionType::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a complete dimension type with vanilla overworld-like defaults.",
        context = "Create a complete dimension type with vanilla overworld-like defaults. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a complete dimension type with vanilla overworld-like defaults."),
        returns = "A `DimensionType` representing a complete dimension type with vanilla overworld-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let dimension_type = sand::component::DimensionType::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            fixed_time: None,
            has_skylight: true,
            has_ceiling: false,
            ultrawarm: false,
            natural: true,
            coordinate_scale: 1.0,
            bed_works: true,
            respawn_anchor_works: false,
            min_y: -64,
            height: 384,
            logical_height: 384,
            infiniburn: TagId::minecraft("infiniburn_overworld")
                .expect("built-in infiniburn tag is valid"),
            effects: ResourceLocation::minecraft("overworld")
                .expect("built-in dimension effects ID is valid"),
            ambient_light: 0.0,
            piglin_safe: false,
            has_raids: true,
            monster_spawn_light_level: MonsterSpawnLightLevel::Uniform {
                min_inclusive: 0,
                max_inclusive: 7,
            },
            monster_spawn_block_light_limit: 0,
            raw_fields: BTreeMap::new(),
        }
    }

    /// Create a complete dimension type with vanilla overworld-like defaults.
    ///
    /// This named alias is convenient when an example should emphasize which
    /// vanilla behavior the defaults model.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::overworld_like",
        aliases = ["sand::prelude::DimensionType::overworld_like"],
        module = "sand::component",
        kind = "method",
        summary = "Create a complete dimension type with vanilla overworld-like defaults.",
        context = "Create a complete dimension type with vanilla overworld-like defaults. This named alias is convenient when an example should emphasize which vanilla behavior the defaults model.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a complete dimension type with vanilla overworld-like defaults."),
        returns = "A `DimensionType` representing a complete dimension type with vanilla overworld-like defaults.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let dimension_type = sand::component::DimensionType::overworld_like(location);\n}",
    )]
    pub fn overworld_like(location: ResourceLocation) -> Self {
        Self::new(location)
    }

    /// Sets the Minecraft fixed time property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::fixed_time",
        aliases = ["sand::prelude::DimensionType::fixed_time"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft fixed time property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft fixed time property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(time = "`time` provides the time applied when setting the Minecraft fixed time property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft fixed time property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, time: i64)  {\n    let updated_dimension_type = dimension_type_value.fixed_time(time);\n}",
    )]
    pub fn fixed_time(mut self, time: i64) -> Self {
        self.fixed_time = Some(time);
        self
    }

    /// Sets the Minecraft without fixed time property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::without_fixed_time",
        aliases = ["sand::prelude::DimensionType::without_fixed_time"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft without fixed time property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft without fixed time property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Sets the Minecraft without fixed time property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType)  {\n    let updated_dimension_type = dimension_type_value.without_fixed_time();\n}",
    )]
    pub fn without_fixed_time(mut self) -> Self {
        self.fixed_time = None;
        self
    }

    /// Sets the Minecraft has skylight property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::has_skylight",
        aliases = ["sand::prelude::DimensionType::has_skylight"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft has skylight property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft has skylight property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft has skylight property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft has skylight property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.has_skylight(value);\n}",
    )]
    pub fn has_skylight(mut self, value: bool) -> Self {
        self.has_skylight = value;
        self
    }

    /// Sets the Minecraft has ceiling property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::has_ceiling",
        aliases = ["sand::prelude::DimensionType::has_ceiling"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft has ceiling property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft has ceiling property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft has ceiling property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft has ceiling property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.has_ceiling(value);\n}",
    )]
    pub fn has_ceiling(mut self, value: bool) -> Self {
        self.has_ceiling = value;
        self
    }

    /// Sets the Minecraft ultrawarm property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::ultrawarm",
        aliases = ["sand::prelude::DimensionType::ultrawarm"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft ultrawarm property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft ultrawarm property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft ultrawarm property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft ultrawarm property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.ultrawarm(value);\n}",
    )]
    pub fn ultrawarm(mut self, value: bool) -> Self {
        self.ultrawarm = value;
        self
    }

    /// Sets the Minecraft natural property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::natural",
        aliases = ["sand::prelude::DimensionType::natural"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft natural property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft natural property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft natural property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft natural property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.natural(value);\n}",
    )]
    pub fn natural(mut self, value: bool) -> Self {
        self.natural = value;
        self
    }

    /// Sets the Minecraft coordinate scale property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::coordinate_scale",
        aliases = ["sand::prelude::DimensionType::coordinate_scale"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft coordinate scale property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft coordinate scale property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft coordinate scale property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft coordinate scale property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: f64)  {\n    let updated_dimension_type = dimension_type_value.coordinate_scale(value);\n}",
    )]
    pub fn coordinate_scale(mut self, value: f64) -> Self {
        self.coordinate_scale = value;
        self
    }

    /// Sets the Minecraft bed works property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::bed_works",
        aliases = ["sand::prelude::DimensionType::bed_works"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft bed works property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft bed works property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft bed works property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft bed works property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.bed_works(value);\n}",
    )]
    pub fn bed_works(mut self, value: bool) -> Self {
        self.bed_works = value;
        self
    }

    /// Sets the Minecraft respawn anchor works property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::respawn_anchor_works",
        aliases = ["sand::prelude::DimensionType::respawn_anchor_works"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft respawn anchor works property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft respawn anchor works property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft respawn anchor works property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft respawn anchor works property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.respawn_anchor_works(value);\n}",
    )]
    pub fn respawn_anchor_works(mut self, value: bool) -> Self {
        self.respawn_anchor_works = value;
        self
    }

    /// Sets the Minecraft min y property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::min_y",
        aliases = ["sand::prelude::DimensionType::min_y"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft min y property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft min y property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft min y property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft min y property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: i32)  {\n    let updated_dimension_type = dimension_type_value.min_y(value);\n}",
    )]
    pub fn min_y(mut self, value: i32) -> Self {
        self.min_y = value;
        self
    }

    /// Sets the Minecraft height property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::height",
        aliases = ["sand::prelude::DimensionType::height"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft height property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft height property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft height property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft height property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: u32)  {\n    let updated_dimension_type = dimension_type_value.height(value);\n}",
    )]
    pub fn height(mut self, value: u32) -> Self {
        self.height = value;
        self
    }

    /// Sets the Minecraft logical height property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::logical_height",
        aliases = ["sand::prelude::DimensionType::logical_height"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft logical height property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft logical height property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft logical height property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft logical height property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: u32)  {\n    let updated_dimension_type = dimension_type_value.logical_height(value);\n}",
    )]
    pub fn logical_height(mut self, value: u32) -> Self {
        self.logical_height = value;
        self
    }

    /// Sets the Minecraft infiniburn property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::infiniburn",
        aliases = ["sand::prelude::DimensionType::infiniburn"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft infiniburn property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft infiniburn property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft infiniburn property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft infiniburn property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: sand::component::TagId < sand::registry::BlockId >)  {\n    let updated_dimension_type = dimension_type_value.infiniburn(value);\n}",
    )]
    pub fn infiniburn(mut self, value: TagId<BlockId>) -> Self {
        self.infiniburn = value;
        self
    }

    /// Sets the Minecraft effects property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::effects",
        aliases = ["sand::prelude::DimensionType::effects"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft effects property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft effects property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft effects property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft effects property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: sand::ResourceLocation)  {\n    let updated_dimension_type = dimension_type_value.effects(value);\n}",
    )]
    pub fn effects(mut self, value: ResourceLocation) -> Self {
        self.effects = value;
        self
    }

    /// Sets the Minecraft ambient light property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::ambient_light",
        aliases = ["sand::prelude::DimensionType::ambient_light"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft ambient light property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft ambient light property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft ambient light property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft ambient light property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: f32)  {\n    let updated_dimension_type = dimension_type_value.ambient_light(value);\n}",
    )]
    pub fn ambient_light(mut self, value: f32) -> Self {
        self.ambient_light = value;
        self
    }

    /// Sets the Minecraft piglin safe property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::piglin_safe",
        aliases = ["sand::prelude::DimensionType::piglin_safe"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft piglin safe property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft piglin safe property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft piglin safe property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft piglin safe property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.piglin_safe(value);\n}",
    )]
    pub fn piglin_safe(mut self, value: bool) -> Self {
        self.piglin_safe = value;
        self
    }

    /// Sets the Minecraft has raids property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::has_raids",
        aliases = ["sand::prelude::DimensionType::has_raids"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft has raids property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft has raids property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft has raids property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft has raids property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: bool)  {\n    let updated_dimension_type = dimension_type_value.has_raids(value);\n}",
    )]
    pub fn has_raids(mut self, value: bool) -> Self {
        self.has_raids = value;
        self
    }

    /// Sets the Minecraft monster spawn light level property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::monster_spawn_light_level",
        aliases = ["sand::prelude::DimensionType::monster_spawn_light_level"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft monster spawn light level property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft monster spawn light level property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft monster spawn light level property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft monster spawn light level property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: sand::component::MonsterSpawnLightLevel)  {\n    let updated_dimension_type = dimension_type_value.monster_spawn_light_level(value);\n}",
    )]
    pub fn monster_spawn_light_level(mut self, value: MonsterSpawnLightLevel) -> Self {
        self.monster_spawn_light_level = value;
        self
    }

    /// Sets the Minecraft monster spawn block light limit property on this typed dimension type definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::monster_spawn_block_light_limit",
        aliases = ["sand::prelude::DimensionType::monster_spawn_block_light_limit"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft monster spawn block light limit property on this typed dimension type definition and returns the updated builder.",
        context = "Sets the Minecraft monster spawn block light limit property on this typed dimension type definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set the Minecraft monster spawn block light limit property on this typed dimension type definition and returns the updated builder."),
        returns = "Sets the Minecraft monster spawn block light limit property on this typed dimension type definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, value: u8)  {\n    let updated_dimension_type = dimension_type_value.monster_spawn_block_light_limit(value);\n}",
    )]
    pub fn monster_spawn_block_light_limit(mut self, value: u8) -> Self {
        self.monster_spawn_block_light_limit = value;
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DimensionType::raw_field",
        aliases = ["sand::prelude::DimensionType::raw_field"],
        module = "sand::component",
        kind = "method",
        summary = "Add a modded or version-specific field not represented by the typed API.",
        context = "Add a modded or version-specific field not represented by the typed API. Typed field names cannot be overridden through this escape hatch.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to add a modded or version-specific field not represented by the typed API.", value = "`value` provides the value being applied or compared used to add a modded or version-specific field not represented by the typed API."),
        returns = "The `DimensionType` value with the documented change applied to add a modded or version-specific field not represented by the typed API.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dimension_type_value: sand::component::DimensionType, key: impl Into < String >, value: sand::component::RawJson)  {\n    let updated_dimension_type = dimension_type_value.raw_field(key, value);\n}",
    )]
    pub fn raw_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.raw_fields.insert(key.into(), value);
        self
    }
}

impl DatapackComponent for DimensionType {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "dimension_type";
        if !self.coordinate_scale.is_finite()
            || !(0.000_01..=30_000_000.0).contains(&self.coordinate_scale)
        {
            return Err(validation::error(
                &self.location,
                kind,
                "coordinate_scale",
                &format!(
                    "coordinate_scale must be finite and in 0.00001..=30000000; received {}",
                    self.coordinate_scale
                ),
            ));
        }
        validation::require_finite_f32(&self.location, kind, "ambient_light", self.ambient_light)?;
        if !(0.0..=1.0).contains(&self.ambient_light) {
            return Err(validation::error(
                &self.location,
                kind,
                "ambient_light",
                &format!(
                    "ambient_light must be in 0..=1; received {}",
                    self.ambient_light
                ),
            ));
        }
        if !(-2032..=2031).contains(&self.min_y) || self.min_y % 16 != 0 {
            return Err(validation::error(
                &self.location,
                kind,
                "min_y",
                &format!(
                    "min_y must be in -2032..=2031 and a multiple of 16; received {}",
                    self.min_y
                ),
            ));
        }
        if !(16..=4064).contains(&self.height) || !self.height.is_multiple_of(16) {
            return Err(validation::error(
                &self.location,
                kind,
                "height",
                &format!(
                    "height must be in 16..=4064 and a multiple of 16; received {}",
                    self.height
                ),
            ));
        }
        if i64::from(self.min_y) + i64::from(self.height) > 2032 {
            return Err(validation::error(
                &self.location,
                kind,
                "height",
                &format!(
                    "min_y + height must not exceed 2032; received {} + {}",
                    self.min_y, self.height
                ),
            ));
        }
        if self.logical_height == 0 || self.logical_height > self.height {
            return Err(validation::error(
                &self.location,
                kind,
                "logical_height",
                &format!(
                    "logical_height must be in 1..=height ({}); received {}",
                    self.height, self.logical_height
                ),
            ));
        }
        self.monster_spawn_light_level.validate(&self.location)?;
        if self.monster_spawn_block_light_limit > 15 {
            return Err(validation::error(
                &self.location,
                kind,
                "monster_spawn_block_light_limit",
                &format!(
                    "monster_spawn_block_light_limit must be in 0..=15; received {}",
                    self.monster_spawn_block_light_limit
                ),
            ));
        }
        for key in self.raw_fields.keys() {
            validation::require_non_empty(&self.location, kind, "raw_field", key)?;
            validation::reject_whitespace_only(&self.location, kind, "raw_field", key)?;
            validation::reject_control_chars(&self.location, kind, "raw_field", key)?;
            if TYPED_FIELDS.contains(&key.as_str()) {
                return Err(validation::error(
                    &self.location,
                    kind,
                    "raw_field",
                    &format!("raw field `{key}` would override a typed field"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Some(fixed_time) = self.fixed_time {
            map.insert("fixed_time".into(), serde_json::json!(fixed_time));
        }
        map.insert("has_skylight".into(), self.has_skylight.into());
        map.insert("has_ceiling".into(), self.has_ceiling.into());
        map.insert("ultrawarm".into(), self.ultrawarm.into());
        map.insert("natural".into(), self.natural.into());
        map.insert(
            "coordinate_scale".into(),
            serde_json::json!(self.coordinate_scale),
        );
        map.insert("bed_works".into(), self.bed_works.into());
        map.insert(
            "respawn_anchor_works".into(),
            self.respawn_anchor_works.into(),
        );
        map.insert("min_y".into(), self.min_y.into());
        map.insert("height".into(), self.height.into());
        map.insert("logical_height".into(), self.logical_height.into());
        map.insert(
            "infiniburn".into(),
            Value::String(self.infiniburn.to_tag_string()),
        );
        map.insert("effects".into(), Value::String(self.effects.to_string()));
        map.insert(
            "ambient_light".into(),
            serde_json::json!(self.ambient_light),
        );
        map.insert("piglin_safe".into(), self.piglin_safe.into());
        map.insert("has_raids".into(), self.has_raids.into());
        map.insert(
            "monster_spawn_light_level".into(),
            self.monster_spawn_light_level.to_json(),
        );
        map.insert(
            "monster_spawn_block_light_limit".into(),
            self.monster_spawn_block_light_limit.into(),
        );
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "dimension_type"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "skylands").unwrap()
    }

    #[test]
    fn minimal_overworld_like_shape_is_valid() {
        let ty = DimensionType::overworld_like(location());
        ty.validate().unwrap();
        let json = ty.to_json();
        assert_eq!(json["min_y"], -64);
        assert_eq!(json["height"], 384);
        assert_eq!(json["infiniburn"], "#minecraft:infiniburn_overworld");
        assert_eq!(json["effects"], "minecraft:overworld");
        assert!(json.get("fixed_time").is_none());
        assert_eq!(ty.component_dir(), "dimension_type");
    }

    #[test]
    fn full_nether_like_shape_and_raw_extension_serialize() {
        let ty = DimensionType::overworld_like(location())
            .fixed_time(18_000)
            .has_skylight(false)
            .has_ceiling(true)
            .ultrawarm(true)
            .natural(false)
            .coordinate_scale(8.0)
            .bed_works(false)
            .respawn_anchor_works(true)
            .min_y(0)
            .height(256)
            .logical_height(128)
            .infiniburn(TagId::minecraft("infiniburn_nether").unwrap())
            .effects(ResourceLocation::minecraft("the_nether").unwrap())
            .ambient_light(0.1)
            .piglin_safe(true)
            .has_raids(false)
            .monster_spawn_light_level(MonsterSpawnLightLevel::Constant(7))
            .monster_spawn_block_light_limit(15)
            .raw_field("example:weather", RawJson::new(serde_json::json!("ash")));
        ty.validate().unwrap();
        let json = ty.to_json();
        assert_eq!(json["fixed_time"], 18_000);
        assert_eq!(json["monster_spawn_light_level"], 7);
        assert_eq!(json["example:weather"], "ash");
    }

    #[test]
    fn non_finite_numeric_values_are_rejected() {
        assert!(
            DimensionType::overworld_like(location())
                .coordinate_scale(f64::NAN)
                .validate()
                .is_err()
        );
        assert!(
            DimensionType::overworld_like(location())
                .ambient_light(f32::INFINITY)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn malformed_resource_and_tag_ids_are_rejected_at_construction() {
        assert!("minecraft:bad path".parse::<ResourceLocation>().is_err());
        assert!("minecraft:bad tag".parse::<TagId<BlockId>>().is_err());
    }

    #[test]
    fn invalid_height_relationships_are_rejected() {
        for ty in [
            DimensionType::overworld_like(location()).min_y(-63),
            DimensionType::overworld_like(location()).height(15),
            DimensionType::overworld_like(location())
                .min_y(0)
                .height(2048 + 16),
            DimensionType::overworld_like(location()).logical_height(385),
        ] {
            assert!(ty.validate().is_err());
        }
    }

    #[test]
    fn invalid_light_levels_and_typed_raw_overrides_are_rejected() {
        assert!(
            DimensionType::overworld_like(location())
                .monster_spawn_light_level(MonsterSpawnLightLevel::Uniform {
                    min_inclusive: 8,
                    max_inclusive: 7,
                })
                .validate()
                .is_err()
        );
        assert!(
            DimensionType::overworld_like(location())
                .raw_field("height", RawJson::new(serde_json::json!(16)))
                .validate()
                .is_err()
        );
    }
}
