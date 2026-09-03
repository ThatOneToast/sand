//! Shared worldgen value types used by more than one worldgen builder.
//!
//! These types model small vanilla worldgen shapes that appear in several
//! registries (structures, processor lists, features). They are deliberately
//! kept here — rather than duplicated per module — so that a block state or a
//! height provider serializes identically everywhere Sand emits one.
//!
//! Every type keeps an explicitly named raw escape hatch where vanilla has
//! more variants than Sand models today.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::BlockId;
use crate::resource_location::ResourceLocation;
use crate::validation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::BlockState",
    module = "sand::component",
    summary = "A concrete block state: a block identifier plus optional property values.",
    context = "A concrete block state: a block identifier plus optional property values. This is the datapack JSON form used by structure processors and feature configs (`{\"Name\": …, \"Properties\": {…}}`). It is deliberately distinct from the command-side `sand::command::BlockState`, which renders `minecraft:stone[facing=north]`.",
    minecraft = "This is the datapack JSON form used by structure processors and feature configs (`{\"Name\": …, \"Properties\": {…}}`). It is deliberately distinct from the command-side `sand::command::BlockState`, which renders `minecraft:stone[facing=north]`.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::BlockState;",
)]
/// A concrete block state: a block identifier plus optional property values.
///
/// This is the datapack JSON form used by structure processors and feature
/// configs (`{"Name": …, "Properties": {…}}`). It is deliberately distinct
/// from the command-side `sand_commands::BlockState`, which renders
/// `minecraft:stone[facing=north]`.
///
/// ```
/// use sand_components::worldgen::providers::BlockState;
/// use sand_components::BlockId;
///
/// let state = BlockState::new(BlockId::minecraft("oak_log").unwrap()).property("axis", "y");
/// assert_eq!(state.block().to_string(), "minecraft:oak_log");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    block: BlockId,
    properties: BTreeMap<String, String>,
}

impl BlockState {
    /// Create a block state with no property overrides.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BlockState::new",
        module = "sand::component",
        kind = "method",
        summary = "Create a block state with no property overrides.",
        context = "Create a block state with no property overrides. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(block = "`block` provides the block value or block predicate used to create a block state with no property overrides."),
        returns = "A `BlockState` representing a block state with no property overrides.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block: sand::registry::BlockId)  {\n    let block_state = sand::component::BlockState::new(block);\n}",
    )]
    pub fn new(block: BlockId) -> Self {
        Self {
            block,
            properties: BTreeMap::new(),
        }
    }

    /// Set a block-state property (deterministically ordered on export).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BlockState::property",
        module = "sand::component",
        kind = "method",
        summary = "Set a block-state property (deterministically ordered on export).",
        context = "Set a block-state property (deterministically ordered on export). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to set a block-state property (deterministically ordered on export).", value = "`value` provides the value being applied or compared used to set a block-state property (deterministically ordered on export)."),
        returns = "The `BlockState` value with the documented change applied to set a block-state property (deterministically ordered on export).",
        example = "use sand::prelude::*;\n\nfn demonstrate(block_state_value: sand::component::BlockState, key: impl Into < String >, value: impl Into < String >)  {\n    let updated_block_state = block_state_value.property(key, value);\n}",
    )]
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// The block this state refers to.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BlockState::block",
        module = "sand::component",
        kind = "method",
        summary = "The block this state refers to.",
        context = "The block this state refers to. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `& BlockId` value produced to use the block this state refers to.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block_state_value: &sand::component::BlockState)  {\n    let block = block_state_value.block();\n}",
    )]
    pub fn block(&self) -> &BlockId {
        &self.block
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("Name".into(), Value::String(self.block.to_string()));
        if !self.properties.is_empty() {
            let properties: Map<String, Value> = self
                .properties
                .iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect();
            map.insert("Properties".into(), Value::Object(properties));
        }
        Value::Object(map)
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        validation::validate_resource_location_str(
            location,
            kind,
            &format!("{field}.Name"),
            &self.block.to_string(),
        )?;
        for (key, value) in &self.properties {
            let property_field = format!("{field}.Properties.{key}");
            validation::require_non_empty(location, kind, &property_field, key)?;
            validation::reject_whitespace_only(location, kind, &property_field, key)?;
            validation::reject_control_chars(location, kind, &property_field, key)?;
            validation::require_non_empty(location, kind, &property_field, value)?;
            validation::reject_whitespace_only(location, kind, &property_field, value)?;
            validation::reject_control_chars(location, kind, &property_field, value)?;
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::WeightedBlockState",
    module = "sand::component",
    summary = "A weighted entry of a [`BlockStateProvider::Weighted`] provider.",
    context = "A weighted entry of a [`BlockStateProvider::Weighted`] provider. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::WeightedBlockState;",
)]
/// A weighted entry of a [`BlockStateProvider::Weighted`] provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedBlockState {
    state: BlockState,
    weight: u32,
}

impl WeightedBlockState {
    /// Create a weighted block-state entry. `weight` must be at least 1.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::WeightedBlockState::new",
        module = "sand::component",
        kind = "method",
        summary = "Create a weighted block-state entry. `weight` must be at least 1.",
        context = "Create a weighted block-state entry. `weight` must be at least 1. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(state = "`state` is used when creating a weighted block-state entry. `weight` must be at least 1.", weight = "Create a weighted block-state entry. `weight` must be at least 1."),
        returns = "A `WeightedBlockState` representing a weighted block-state entry. `weight` must be at least 1.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state: sand::component::BlockState, weight: u32)  {\n    let weighted_block_state = sand::component::WeightedBlockState::new(state, weight);\n}",
    )]
    pub fn new(state: BlockState, weight: u32) -> Self {
        Self { state, weight }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::BlockStateProvider",
    module = "sand::component",
    summary = "A typed block-state provider used by worldgen feature configs.",
    context = "A typed block-state provider used by worldgen feature configs. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::BlockStateProvider;",
    variants(Simple = "`minecraft:simple_state_provider` — always the same block state.", Weighted = "`minecraft:weighted_state_provider` — a weighted random choice."),
    variant_fields(Simple = ["`minecraft:simple_state_provider` — always the same block state."], Weighted = ["`minecraft:weighted_state_provider` — a weighted random choice."]),
)]
/// A typed block-state provider used by worldgen feature configs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStateProvider {
    /// `minecraft:simple_state_provider` — always the same block state.
    Simple(#[doc = "`minecraft:simple_state_provider` — always the same block state."] BlockState),
    /// `minecraft:weighted_state_provider` — a weighted random choice.
    Weighted(
        #[doc = "`minecraft:weighted_state_provider` — a weighted random choice."]
        Vec<WeightedBlockState>,
    ),
}

impl BlockStateProvider {
    /// Convenience constructor for a `minecraft:simple_state_provider`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BlockStateProvider::simple",
        module = "sand::component",
        kind = "method",
        summary = "Convenience constructor for a `minecraft:simple_state_provider`.",
        context = "Convenience constructor for a `minecraft:simple_state_provider`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(state = "`state` sets the state for convenience constructor for a `minecraft:simple_state_provider`."),
        returns = "A `BlockStateProvider` configured for convenience constructor for a `minecraft:simple_state_provider`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(state: sand::component::BlockState)  {\n    let block_state_provider = sand::component::BlockStateProvider::simple(state);\n}",
    )]
    pub fn simple(state: BlockState) -> Self {
        Self::Simple(state)
    }

    /// Convenience constructor for a `minecraft:weighted_state_provider`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BlockStateProvider::weighted",
        module = "sand::component",
        kind = "method",
        summary = "Convenience constructor for a `minecraft:weighted_state_provider`.",
        context = "Convenience constructor for a `minecraft:weighted_state_provider`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entries = "`entries` sets the entries for convenience constructor for a `minecraft:weighted_state_provider`."),
        returns = "A `BlockStateProvider` configured for convenience constructor for a `minecraft:weighted_state_provider`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(entries: impl IntoIterator < Item = sand::component::WeightedBlockState >)  {\n    let block_state_provider = sand::component::BlockStateProvider::weighted(entries);\n}",
    )]
    pub fn weighted(entries: impl IntoIterator<Item = WeightedBlockState>) -> Self {
        Self::Weighted(entries.into_iter().collect())
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Simple(state) => serde_json::json!({
                "type": "minecraft:simple_state_provider",
                "state": state.to_json(),
            }),
            Self::Weighted(entries) => {
                let entries: Vec<Value> = entries
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "weight": entry.weight,
                            "data": entry.state.to_json(),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "type": "minecraft:weighted_state_provider",
                    "entries": entries,
                })
            }
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        match self {
            Self::Simple(state) => state.validate(location, kind, &format!("{field}.state")),
            Self::Weighted(entries) => {
                validation::require_non_empty_collection(
                    location,
                    kind,
                    &format!("{field}.entries"),
                    entries.len(),
                )?;
                for (index, entry) in entries.iter().enumerate() {
                    let entry_field = format!("{field}.entries[{index}]");
                    if entry.weight == 0 {
                        return Err(validation::error(
                            location,
                            kind,
                            &format!("{entry_field}.weight"),
                            "weight must be at least 1; received 0",
                        ));
                    }
                    entry.state.validate(location, kind, &entry_field)?;
                }
                Ok(())
            }
        }
    }
}

/// Inclusive world-height bounds accepted by vanilla vertical anchors.
const MIN_ANCHOR: i32 = -2032;
const MAX_ANCHOR: i32 = 2031;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::VerticalAnchor",
    aliases = ["sand::prelude::VerticalAnchor"],
    module = "sand::component",
    summary = "A vanilla vertical anchor (`{\"absolute\": 0}`, `{\"above_bottom\": 8}`, …).",
    context = "A vanilla vertical anchor (`{\"absolute\": 0}`, `{\"above_bottom\": 8}`, …). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::VerticalAnchor;",
    variants(AboveBottom = "An offset above the dimension's minimum build height.", Absolute = "An absolute Y coordinate.", BelowTop = "An offset below the dimension's maximum build height."),
    variant_fields(AboveBottom = ["An offset above the dimension's minimum build height."], Absolute = ["An absolute Y coordinate."], BelowTop = ["An offset below the dimension's maximum build height."]),
)]
/// A vanilla vertical anchor (`{"absolute": 0}`, `{"above_bottom": 8}`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerticalAnchor {
    /// An absolute Y coordinate.
    Absolute(#[doc = "An absolute Y coordinate."] i32),
    /// An offset above the dimension's minimum build height.
    AboveBottom(#[doc = "An offset above the dimension's minimum build height."] i32),
    /// An offset below the dimension's maximum build height.
    BelowTop(#[doc = "An offset below the dimension's maximum build height."] i32),
}

impl VerticalAnchor {
    /// Serialize to the vanilla single-key anchor object.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::VerticalAnchor::to_json",
        aliases = ["sand::prelude::VerticalAnchor::to_json"],
        module = "sand::component",
        kind = "method",
        summary = "Serialize to the vanilla single-key anchor object.",
        context = "Serialize to the vanilla single-key anchor object. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `Value` value produced to serialize to the vanilla single-key anchor object.",
        example = "use sand::prelude::*;\n\nfn demonstrate(vertical_anchor_value: &sand::component::VerticalAnchor)  {\n    let to_json = vertical_anchor_value.to_json();\n}",
    )]
    pub fn to_json(&self) -> Value {
        let (key, value) = match self {
            Self::Absolute(value) => ("absolute", *value),
            Self::AboveBottom(value) => ("above_bottom", *value),
            Self::BelowTop(value) => ("below_top", *value),
        };
        let mut map = Map::new();
        map.insert(key.into(), value.into());
        Value::Object(map)
    }

    fn value(&self) -> i32 {
        match self {
            Self::Absolute(value) | Self::AboveBottom(value) | Self::BelowTop(value) => *value,
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        let value = self.value();
        if !(MIN_ANCHOR..=MAX_ANCHOR).contains(&value) {
            return Err(validation::error(
                location,
                kind,
                field,
                &format!(
                    "vertical anchor must be in {MIN_ANCHOR}..={MAX_ANCHOR}; received {value}"
                ),
            ));
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::HeightProvider",
    aliases = ["sand::prelude::HeightProvider"],
    module = "sand::component",
    summary = "A vanilla height provider. [`HeightProvider::Raw`] is the explicit escape hatch for provider types Sand does not model yet (for example `weighted_list`).",
    context = "A vanilla height provider. [`HeightProvider::Raw`] is the explicit escape hatch for provider types Sand does not model yet (for example `weighted_list`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::HeightProvider;",
    variants(Constant = "A constant anchor, emitted using the vanilla inline shorthand.", Raw = "An explicitly raw height provider object.", Trapezoid = "A trapezoidal distribution between two anchors.", Uniform = "A uniformly sampled inclusive anchor range."),
    variant_fields(Constant = ["A constant anchor, emitted using the vanilla inline shorthand."], Raw = ["An explicitly raw height provider object."], Trapezoid(max_inclusive = "`max_inclusive` provides the max inclusive when a trapezoidal distribution between two anchors.", min_inclusive = "`min_inclusive` provides the min inclusive when a trapezoidal distribution between two anchors.", plateau = "`plateau` provides the plateau when a trapezoidal distribution between two anchors."), Uniform(max_inclusive = "`max_inclusive` provides the max inclusive when a uniformly sampled inclusive anchor range.", min_inclusive = "`min_inclusive` provides the min inclusive when a uniformly sampled inclusive anchor range.")),
)]
/// A vanilla height provider.
///
/// [`HeightProvider::Raw`] is the explicit escape hatch for provider types
/// Sand does not model yet (for example `weighted_list`).
#[derive(Debug, Clone, PartialEq)]
pub enum HeightProvider {
    /// A constant anchor, emitted using the vanilla inline shorthand.
    Constant(
        #[doc = "A constant anchor, emitted using the vanilla inline shorthand."] VerticalAnchor,
    ),
    /// A uniformly sampled inclusive anchor range.
    Uniform {
        /// `min_inclusive` provides the min inclusive when a uniformly sampled inclusive anchor range.
        min_inclusive: VerticalAnchor,
        /// `max_inclusive` provides the max inclusive when a uniformly sampled inclusive anchor range.
        max_inclusive: VerticalAnchor,
    },
    /// A trapezoidal distribution between two anchors.
    Trapezoid {
        /// `min_inclusive` provides the min inclusive when a trapezoidal distribution between two anchors.
        min_inclusive: VerticalAnchor,
        /// `max_inclusive` provides the max inclusive when a trapezoidal distribution between two anchors.
        max_inclusive: VerticalAnchor,
        /// `plateau` provides the plateau when a trapezoidal distribution between two anchors.
        plateau: i32,
    },
    /// An explicitly raw height provider object.
    Raw(#[doc = "An explicitly raw height provider object."] RawJson),
}

impl HeightProvider {
    /// A constant absolute-Y height provider.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::HeightProvider::absolute",
        aliases = ["sand::prelude::HeightProvider::absolute"],
        module = "sand::component",
        kind = "method",
        summary = "A constant absolute-Y height provider.",
        context = "A constant absolute-Y height provider. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(y = "`y` provides the y-coordinate used to use a constant absolute-Y height provider."),
        returns = "A `HeightProvider` configured for a constant absolute-Y height provider.",
        example = "use sand::prelude::*;\n\nfn demonstrate(y: i32)  {\n    let height_provider = sand::component::HeightProvider::absolute(y);\n}",
    )]
    pub fn absolute(y: i32) -> Self {
        Self::Constant(VerticalAnchor::Absolute(y))
    }

    /// Serialize to the vanilla height provider JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::HeightProvider::to_json",
        aliases = ["sand::prelude::HeightProvider::to_json"],
        module = "sand::component",
        kind = "method",
        summary = "Serialize to the vanilla height provider JSON.",
        context = "Serialize to the vanilla height provider JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `Value` value produced to serialize to the vanilla height provider JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(height_provider_value: &sand::component::HeightProvider)  {\n    let to_json = height_provider_value.to_json();\n}",
    )]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Constant(anchor) => anchor.to_json(),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => serde_json::json!({
                "type": "minecraft:uniform",
                "min_inclusive": min_inclusive.to_json(),
                "max_inclusive": max_inclusive.to_json(),
            }),
            Self::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => serde_json::json!({
                "type": "minecraft:trapezoid",
                "min_inclusive": min_inclusive.to_json(),
                "max_inclusive": max_inclusive.to_json(),
                "plateau": plateau,
            }),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        match self {
            Self::Constant(anchor) => anchor.validate(location, kind, field)?,
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => {
                min_inclusive.validate(location, kind, field)?;
                max_inclusive.validate(location, kind, field)?;
                require_ordered_anchors(location, kind, field, min_inclusive, max_inclusive)?;
            }
            Self::Trapezoid {
                min_inclusive,
                max_inclusive,
                plateau,
            } => {
                min_inclusive.validate(location, kind, field)?;
                max_inclusive.validate(location, kind, field)?;
                require_ordered_anchors(location, kind, field, min_inclusive, max_inclusive)?;
                if *plateau < 0 {
                    return Err(validation::error(
                        location,
                        kind,
                        field,
                        &format!("plateau must be non-negative; received {plateau}"),
                    ));
                }
            }
            Self::Raw(raw) => {
                validation::require_json_object(location, kind, field, raw.as_value())?;
            }
        }
        Ok(())
    }
}

/// Compares two anchors only when they use the same anchor kind; mixed kinds
/// depend on the dimension's height and cannot be ordered at export time.
fn require_ordered_anchors(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    min: &VerticalAnchor,
    max: &VerticalAnchor,
) -> SandResult<()> {
    let comparable = matches!(
        (min, max),
        (VerticalAnchor::Absolute(_), VerticalAnchor::Absolute(_))
            | (
                VerticalAnchor::AboveBottom(_),
                VerticalAnchor::AboveBottom(_)
            )
            | (VerticalAnchor::BelowTop(_), VerticalAnchor::BelowTop(_))
    );
    if comparable && min.value() > max.value() {
        return Err(validation::error(
            location,
            kind,
            field,
            &format!(
                "min_inclusive must not exceed max_inclusive; received {}..={}",
                min.value(),
                max.value()
            ),
        ));
    }
    Ok(())
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Heightmap",
    aliases = ["sand::prelude::Heightmap"],
    module = "sand::component",
    summary = "A vanilla chunk heightmap selector.",
    context = "A vanilla chunk heightmap selector. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Heightmap;",
    variants(MotionBlocking = "Samples Minecraft's motion blocking heightmap.", MotionBlockingNoLeaves = "Samples Minecraft's motion blocking no leaves heightmap.", OceanFloor = "Samples Minecraft's ocean floor heightmap.", OceanFloorWg = "Samples Minecraft's ocean floor wg heightmap.", WorldSurface = "Samples Minecraft's world surface heightmap.", WorldSurfaceWg = "Samples Minecraft's world surface wg heightmap."),
)]
/// A vanilla chunk heightmap selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heightmap {
    #[doc = "Samples Minecraft's world surface wg heightmap."]
    WorldSurfaceWg,
    #[doc = "Samples Minecraft's world surface heightmap."]
    WorldSurface,
    #[doc = "Samples Minecraft's ocean floor wg heightmap."]
    OceanFloorWg,
    #[doc = "Samples Minecraft's ocean floor heightmap."]
    OceanFloor,
    #[doc = "Samples Minecraft's motion blocking heightmap."]
    MotionBlocking,
    #[doc = "Samples Minecraft's motion blocking no leaves heightmap."]
    MotionBlockingNoLeaves,
}

impl Heightmap {
    /// The vanilla uppercase enum name written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Heightmap::as_str",
        aliases = ["sand::prelude::Heightmap::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla uppercase enum name written into datapack JSON.",
        context = "The vanilla uppercase enum name written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla uppercase enum name written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(heightmap_value: &sand::component::Heightmap)  {\n    let as_str = heightmap_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "providers").unwrap()
    }

    #[test]
    fn simple_provider_serializes_block_state() {
        let provider = BlockStateProvider::simple(
            BlockState::new(BlockId::minecraft("oak_log").unwrap()).property("axis", "y"),
        );
        assert_eq!(
            provider.to_json(),
            serde_json::json!({
                "type": "minecraft:simple_state_provider",
                "state": {"Name": "minecraft:oak_log", "Properties": {"axis": "y"}},
            })
        );
        provider.validate(&location(), "kind", "field").unwrap();
    }

    #[test]
    fn weighted_provider_requires_entries_with_positive_weights() {
        let empty = BlockStateProvider::weighted([]);
        assert!(empty.validate(&location(), "kind", "field").is_err());

        let zero_weight = BlockStateProvider::weighted([WeightedBlockState::new(
            BlockState::new(BlockId::minecraft("stone").unwrap()),
            0,
        )]);
        assert!(zero_weight.validate(&location(), "kind", "field").is_err());

        let valid = BlockStateProvider::weighted([WeightedBlockState::new(
            BlockState::new(BlockId::minecraft("stone").unwrap()),
            3,
        )]);
        valid.validate(&location(), "kind", "field").unwrap();
        assert_eq!(valid.to_json()["entries"][0]["weight"], 3);
    }

    #[test]
    fn malformed_block_state_properties_are_rejected() {
        let state = BlockState::new(BlockId::minecraft("stone").unwrap()).property("", "value");
        assert!(state.validate(&location(), "kind", "field").is_err());

        let state = BlockState::new(BlockId::minecraft("stone").unwrap()).property("axis", "  ");
        assert!(state.validate(&location(), "kind", "field").is_err());
    }

    #[test]
    fn block_state_serializes_name_and_sorted_properties() {
        let state = BlockState::new(BlockId::minecraft("oak_log").unwrap())
            .property("axis", "y")
            .property("waterlogged", "false");
        state.validate(&location(), "kind", "block").unwrap();
        let json = state.to_json();
        assert_eq!(json["Name"], "minecraft:oak_log");
        assert_eq!(json["Properties"]["axis"], "y");
        assert_eq!(
            serde_json::to_string(&json).unwrap(),
            r#"{"Name":"minecraft:oak_log","Properties":{"axis":"y","waterlogged":"false"}}"#
        );
    }

    #[test]
    fn constant_height_provider_uses_inline_anchor_shorthand() {
        let provider = HeightProvider::absolute(0);
        provider
            .validate(&location(), "kind", "start_height")
            .unwrap();
        assert_eq!(provider.to_json(), serde_json::json!({ "absolute": 0 }));
    }

    #[test]
    fn uniform_height_provider_serializes_typed_anchors() {
        let provider = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::AboveBottom(0),
            max_inclusive: VerticalAnchor::BelowTop(8),
        };
        provider
            .validate(&location(), "kind", "start_height")
            .unwrap();
        let json = provider.to_json();
        assert_eq!(json["type"], "minecraft:uniform");
        assert_eq!(json["min_inclusive"]["above_bottom"], 0);
        assert_eq!(json["max_inclusive"]["below_top"], 8);
    }

    #[test]
    fn inverted_and_out_of_range_anchors_are_rejected() {
        assert!(
            HeightProvider::Uniform {
                min_inclusive: VerticalAnchor::Absolute(64),
                max_inclusive: VerticalAnchor::Absolute(0),
            }
            .validate(&location(), "kind", "start_height")
            .is_err()
        );
        assert!(
            HeightProvider::absolute(9000)
                .validate(&location(), "kind", "start_height")
                .is_err()
        );
    }

    #[test]
    fn raw_height_provider_must_be_a_json_object() {
        assert!(
            HeightProvider::Raw(RawJson::new(serde_json::json!([1, 2])))
                .validate(&location(), "kind", "start_height")
                .is_err()
        );
        assert!(
            HeightProvider::Raw(RawJson::new(serde_json::json!({ "type": "mymod:custom" })))
                .validate(&location(), "kind", "start_height")
                .is_ok()
        );
    }

    #[test]
    fn heightmap_names_use_vanilla_uppercase_form() {
        assert_eq!(Heightmap::WorldSurfaceWg.as_str(), "WORLD_SURFACE_WG");
        assert_eq!(
            Heightmap::MotionBlockingNoLeaves.as_str(),
            "MOTION_BLOCKING_NO_LEAVES"
        );
    }
}
