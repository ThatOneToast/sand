//! 🌍 World (datapack) — dimension definitions.
//!
//! Every [`Dimension`] lowers to a `data/<namespace>/dimension/<id>.json`
//! resource (plus, for custom dimension *types*, a companion
//! `data/<namespace>/dimension_type/<id>.json`). Both ship inside the
//! exported datapack.

use sand_components::resource_location::ResourceLocation;
use sand_macros::api;
use serde_json::{Value, json};

use super::generator::Generator;

/// Which vanilla dimension a [`Dimension`] replaces, or a custom one.
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::DimensionSlot",
    module = "sand::build",
    summary = "DimensionSlot names which vanilla dimension a Dimension replaces, or declares a custom one.",
    context = "A World's Dimensions collection keys each Dimension by its slot's resource location.",
    minecraft = "Lowers to the dimension resource's data/<namespace>/dimension/<id>.json path.",
    use_when = ["Overriding the Overworld/Nether/End's generator", "Declaring a brand-new custom dimension"],
    avoid_when = ["Referencing a dimension type; use DimensionType instead"],
    variants(Overworld = "The vanilla Overworld slot (minecraft:overworld).", Nether = "The vanilla Nether slot (minecraft:the_nether).", End = "The vanilla End slot (minecraft:the_end).", Custom = "A project-defined dimension at data/<namespace>/dimension/<path>.json."),
    variant_fields(Custom = ["The custom dimension's resource location."]),
    example = "Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld);"
)]
pub enum DimensionSlot {
    Overworld,
    Nether,
    End,
    /// A custom dimension at `data/<namespace>/dimension/<path>.json`.
    Custom(ResourceLocation),
}

impl DimensionSlot {
    pub(crate) fn resource_location(&self) -> ResourceLocation {
        match self {
            DimensionSlot::Overworld => {
                ResourceLocation::new("minecraft", "overworld").expect("valid built-in id")
            }
            DimensionSlot::Nether => {
                ResourceLocation::new("minecraft", "the_nether").expect("valid built-in id")
            }
            DimensionSlot::End => {
                ResourceLocation::new("minecraft", "the_end").expect("valid built-in id")
            }
            DimensionSlot::Custom(rl) => rl.clone(),
        }
    }
}

/// The dimension type reference a [`Dimension`] uses — vanilla's Overworld,
/// Nether, or End type, or a custom `data/<namespace>/dimension_type/<id>.json`
/// this project authors separately.
#[derive(Debug, Clone, PartialEq, Eq)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::DimensionType",
    module = "sand::build",
    summary = "DimensionType selects the dimension_type resource a Dimension uses (skylight, height limits, coordinate scale, and similar physics).",
    context = "Paired with a DimensionSlot inside Dimension to fully describe one dimension resource.",
    minecraft = "Lowers to the dimension resource's top-level \"type\" reference.",
    use_when = ["Choosing vanilla Overworld/Nether/End physics for a dimension"],
    avoid_when = ["Naming which slot the dimension occupies; use DimensionSlot instead"],
    variants(Overworld = "Vanilla Overworld dimension type (skylight, normal height limits).", OverworldCaves = "Vanilla Overworld Caves dimension type variant.", Nether = "Vanilla Nether dimension type (no skylight, ultrawarm).", End = "Vanilla End dimension type.", Custom = "A project-authored data/<namespace>/dimension_type/<id>.json resource."),
    variant_fields(Custom = ["The custom dimension type's resource location."]),
    example = "Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld);"
)]
pub enum DimensionType {
    Overworld,
    OverworldCaves,
    Nether,
    End,
    Custom(ResourceLocation),
}

impl DimensionType {
    pub(crate) fn resource_location(&self) -> ResourceLocation {
        match self {
            DimensionType::Overworld => {
                ResourceLocation::new("minecraft", "overworld").expect("valid built-in id")
            }
            DimensionType::OverworldCaves => {
                ResourceLocation::new("minecraft", "overworld_caves").expect("valid built-in id")
            }
            DimensionType::Nether => {
                ResourceLocation::new("minecraft", "the_nether").expect("valid built-in id")
            }
            DimensionType::End => {
                ResourceLocation::new("minecraft", "the_end").expect("valid built-in id")
            }
            DimensionType::Custom(rl) => rl.clone(),
        }
    }
}

/// One dimension: which slot it occupies, its dimension type, and its chunk
/// generator.
///
/// 🌍 World (datapack).
///
/// ```
/// use sand_core::build::{Dimension, DimensionSlot, DimensionType, FlatGenerator, FlatLayer, Generator};
/// use sand_components::resource_location::ResourceLocation;
///
/// let overworld = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld)
///     .generator(Generator::Flat(FlatGenerator::new(vec![
///         FlatLayer::new(ResourceLocation::new("minecraft", "grass_block").unwrap(), 1),
///     ])));
/// ```
#[derive(Debug, Clone)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Dimension",
    module = "sand::build",
    summary = "Dimension pairs a DimensionSlot, DimensionType, and Generator into one dimension resource.",
    context = "A World's Dimensions collection holds the Dimension entries a build script configures.",
    minecraft = "Lowers to one data/<namespace>/dimension/<id>.json resource in the exported datapack.",
    use_when = ["Configuring the Overworld/Nether/End or a custom dimension's generator"],
    avoid_when = ["Configuring server-only view/simulation distance; use ServerConfig"],
    example = "Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(Generator::Void);"
)]
pub struct Dimension {
    pub(crate) slot: DimensionSlot,
    pub(crate) dimension_type: DimensionType,
    pub(crate) generator: Generator,
}

impl Dimension {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimension::new",
        module = "sand::build",
        summary = "Creates a dimension for the given slot and type, defaulting to a void generator.",
        context = "Call .generator(...) afterward to choose flat/void/noise/custom generation.",
        minecraft = "Produces the dimension resource's \"type\" field; \"generator\" defaults to void until overridden.",
        use_when = ["Starting a new dimension definition"],
        avoid_when = ["Modifying an existing Dimension's generator; use .generator(...)"],
        params(slot = "Which dimension slot this occupies.", dimension_type = "The dimension_type reference this dimension uses."),
        returns = "A new Dimension with a void generator.",
        example = "Dimension::new(DimensionSlot::Nether, DimensionType::Nether);"
    )]
    pub fn new(slot: DimensionSlot, dimension_type: DimensionType) -> Self {
        Self {
            slot,
            dimension_type,
            generator: Generator::Void,
        }
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimension::generator",
        module = "sand::build",
        summary = "Sets this dimension's chunk generator.",
        context = "Chooses which Generator variant (flat, void, noise, or a custom reference) this dimension lowers to.",
        minecraft = "Populates the dimension resource's \"generator\" object.",
        use_when = ["Choosing flat/void/noise generation for a dimension"],
        avoid_when = ["Choosing the dimension_type; use Dimension::new"],
        params(generator = "The generator this dimension uses."),
        returns = "This dimension with its generator set.",
        example = "Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(Generator::Void);"
    )]
    pub fn generator(mut self, generator: Generator) -> Self {
        self.generator = generator;
        self
    }

    /// The resource location this dimension is written to
    /// (`data/<namespace>/dimension/<path>.json`).
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimension::resource_location",
        module = "sand::build",
        summary = "Returns the resource location this dimension writes to.",
        context = "Used by validation and lowering to key dimension_json output and detect duplicate slots.",
        minecraft = "Matches the data/<namespace>/dimension/<path>.json path Sand writes for this dimension.",
        use_when = ["Inspecting which resource location a Dimension will occupy"],
        avoid_when = ["Constructing a new Dimension"],
        returns = "The dimension's resource location.",
        example = "assert_eq!(Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).resource_location().to_string(), \"minecraft:overworld\");"
    )]
    pub fn resource_location(&self) -> ResourceLocation {
        self.slot.resource_location()
    }

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "type": self.dimension_type.resource_location().to_string(),
            "generator": self.generator.to_json(),
        })
    }
}

/// The set of dimensions a [`super::world::World`] contains.
///
/// 🌍 World (datapack) — defaults to empty; a build script normally adds at
/// least an Overworld entry. Sand does not require all three vanilla
/// dimensions to be present — omitted vanilla dimensions keep their default
/// vanilla generation (Sand only overrides what is explicitly configured).
#[derive(Debug, Clone, Default)]
#[api(
    registry = sand_api_contract,
    path = "sand::build::Dimensions",
    module = "sand::build",
    summary = "Dimensions is the ordered collection of Dimension entries a World contains.",
    context = "Defaults to empty; a build script normally adds at least an Overworld entry via .with(...).",
    minecraft = "Lowers to one dimension resource per entry; omitted vanilla dimensions keep default vanilla generation.",
    use_when = ["Assembling the set of dimensions a World configures"],
    avoid_when = ["Configuring a single dimension's generator; use Dimension::generator"],
    example = "Dimensions::new().with(Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld));"
)]
pub struct Dimensions {
    pub(crate) entries: Vec<Dimension>,
}

impl Dimensions {
    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimensions::new",
        module = "sand::build",
        summary = "Creates an empty Dimensions collection.",
        context = "Starting point before chaining .with(...) for each configured dimension.",
        minecraft = "Produces no dimension resources until entries are added.",
        use_when = ["Starting a World's dimension configuration"],
        avoid_when = ["Adding a dimension to an existing collection; use .with(...)"],
        returns = "An empty Dimensions collection.",
        example = "Dimensions::new();"
    )]
    pub fn new() -> Self {
        Self::default()
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimensions::with",
        module = "sand::build",
        summary = "Appends one dimension to this collection.",
        context = "Called once per dimension a build script wants to configure.",
        minecraft = "Each entry becomes one dimension resource in the exported datapack.",
        use_when = ["Adding a configured Dimension to a World"],
        avoid_when = ["Removing or replacing an existing entry"],
        params(dimension = "The dimension to append."),
        returns = "This collection with the dimension appended.",
        example = "Dimensions::new().with(Dimension::new(DimensionSlot::Nether, DimensionType::Nether));"
    )]
    pub fn with(mut self, dimension: Dimension) -> Self {
        self.entries.push(dimension);
        self
    }

    #[api(
        registry = sand_api_contract,
        path = "sand::build::Dimensions::entries",
        module = "sand::build",
        summary = "Returns the configured dimensions in insertion order.",
        context = "Used by lowering and validation to iterate every configured dimension.",
        minecraft = "Iteration order matches the order dimension resources are generated in.",
        use_when = ["Inspecting or iterating a World's configured dimensions"],
        avoid_when = ["Adding a new dimension; use .with(...)"],
        returns = "A slice of the configured Dimension entries.",
        example = "assert!(Dimensions::new().entries().is_empty());"
    )]
    pub fn entries(&self) -> &[Dimension] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::generator::{FlatGenerator, FlatLayer};

    #[test]
    fn dimension_json_names_its_type_and_generator() {
        let d = Dimension::new(DimensionSlot::Overworld, DimensionType::Overworld).generator(
            Generator::Flat(FlatGenerator::new(vec![FlatLayer::new(
                ResourceLocation::new("minecraft", "stone").unwrap(),
                64,
            )])),
        );
        let json = d.to_json();
        assert_eq!(json["type"], "minecraft:overworld");
        assert_eq!(json["generator"]["type"], "minecraft:flat");
    }

    #[test]
    fn custom_dimension_slot_uses_its_own_resource_location() {
        let rl = ResourceLocation::new("my_pack", "sky_realm").unwrap();
        let d = Dimension::new(
            DimensionSlot::Custom(rl.clone()),
            DimensionType::Custom(ResourceLocation::new("my_pack", "sky_realm_type").unwrap()),
        );
        assert_eq!(d.resource_location(), rl);
    }

    #[test]
    fn dimensions_collection_preserves_insertion_order() {
        let dims = Dimensions::new()
            .with(Dimension::new(
                DimensionSlot::Overworld,
                DimensionType::Overworld,
            ))
            .with(Dimension::new(DimensionSlot::Nether, DimensionType::Nether));
        assert_eq!(dims.entries().len(), 2);
        assert_eq!(dims.entries()[0].slot, DimensionSlot::Overworld);
        assert_eq!(dims.entries()[1].slot, DimensionSlot::Nether);
    }
}
