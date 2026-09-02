//! Builder for `data/<namespace>/worldgen/processor_list/<id>.json`.
//!
//! [`ProcessorList::new`] models the common vanilla structure processors —
//! block ignore, protected blocks, gravity, jigsaw replacement, and rule
//! processors — with [`Processor::Raw`] as the explicit escape hatch for
//! unsupported or modded processor types.

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{BlockId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;
use crate::worldgen::providers::{BlockState, Heightmap};

const KIND: &str = "worldgen/processor_list";

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ProcessorRule",
    aliases = ["sand::prelude::ProcessorRule"],
    module = "sand::component",
    summary = "One rule of a `minecraft:rule` processor. `input_predicate`, `location_predicate`, and `position_predicate` remain raw JSON: vanilla's block-state/position predicate grammar is broad and is better served by an explicit escape hatch than a partial typed model. `output_state` is typed since it is the common case authors need to get right — a plain replacement block state.",
    context = "One rule of a `minecraft:rule` processor. `input_predicate`, `location_predicate`, and `position_predicate` remain raw JSON: vanilla's block-state/position predicate grammar is broad and is better served by an explicit escape hatch than a partial typed model. `output_state` is typed since it is the common case authors need to get right — a plain replacement block state. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "`input_predicate`, `location_predicate`, and `position_predicate` remain raw JSON: vanilla's block-state/position predicate grammar is broad and is better served by an explicit escape hatch than a partial typed model. `output_state` is typed since it is the common case authors need to get right — a plain replacement block state.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ProcessorRule;",
)]
/// One rule of a `minecraft:rule` processor.
///
/// `input_predicate`, `location_predicate`, and `position_predicate` remain
/// raw JSON: vanilla's block-state/position predicate grammar is broad and is
/// better served by an explicit escape hatch than a partial typed model.
/// `output_state` is typed since it is the common case authors need to get
/// right — a plain replacement block state.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorRule {
    input_predicate: RawJson,
    location_predicate: Option<RawJson>,
    position_predicate: Option<RawJson>,
    output_state: BlockState,
    output_nbt: Option<String>,
}

impl ProcessorRule {
    /// `input_predicate` must be a JSON object matching vanilla's block
    /// predicate grammar (e.g. `{"predicate_type": "minecraft:block_match", "block": "minecraft:stone"}`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorRule::new",
        aliases = ["sand::prelude::ProcessorRule::new"],
        module = "sand::component",
        kind = "method",
        summary = "`input_predicate` must be a JSON object matching vanilla's block predicate grammar (e.g. `{\"predicate_type\": \"minecraft:block_match\", \"block\": \"minecraft:stone\"}`).",
        context = "`input_predicate` must be a JSON object matching vanilla's block predicate grammar (e.g. `{\"predicate_type\": \"minecraft:block_match\", \"block\": \"minecraft:stone\"}`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(input_predicate = "`input_predicate` must be a JSON object matching vanilla's block predicate grammar (e.g. `{\"predicate_type\": \"minecraft:block_match\", \"block\": \"minecraft:stone\"}`).", output_state = "`output_state` supplies the output state value used to emit the documented `input_predicate` must be a JSON object matching vanilla's block predicate grammar (e.g. `{\"predicate_type\": \"minecraft:block_match\", \"block\": \"minecraft:stone\"}`) form."),
        returns = "A newly constructed `ProcessorRule` configured to emit the documented `input_predicate` must be a JSON object matching vanilla's block predicate grammar (e.g. `{\"predicate_type\": \"minecraft:block_match\", \"block\": \"minecraft:stone\"}`) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input_predicate: sand::component::RawJson, output_state: sand::component::BlockState)  {\n    let processor_rule = sand::component::ProcessorRule::new(input_predicate, output_state);\n}",
    )]
    pub fn new(input_predicate: RawJson, output_state: BlockState) -> Self {
        Self {
            input_predicate,
            location_predicate: None,
            position_predicate: None,
            output_state,
            output_nbt: None,
        }
    }

    /// Sets the Minecraft location predicate property on this typed processor rule definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorRule::location_predicate",
        aliases = ["sand::prelude::ProcessorRule::location_predicate"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft location predicate property on this typed processor rule definition and returns the updated builder.",
        context = "Sets the Minecraft location predicate property on this typed processor rule definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(predicate = "`predicate` provides the predicate that must match used to set the Minecraft location predicate property on this typed processor rule definition and returns the updated builder."),
        returns = "Sets the Minecraft location predicate property on this typed processor rule definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(processor_rule_value: sand::component::ProcessorRule, predicate: sand::component::RawJson)  {\n    let updated_processor_rule = processor_rule_value.location_predicate(predicate);\n}",
    )]
    pub fn location_predicate(mut self, predicate: RawJson) -> Self {
        self.location_predicate = Some(predicate);
        self
    }

    /// Sets the Minecraft position predicate property on this typed processor rule definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorRule::position_predicate",
        aliases = ["sand::prelude::ProcessorRule::position_predicate"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft position predicate property on this typed processor rule definition and returns the updated builder.",
        context = "Sets the Minecraft position predicate property on this typed processor rule definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(predicate = "`predicate` provides the predicate that must match used to set the Minecraft position predicate property on this typed processor rule definition and returns the updated builder."),
        returns = "Sets the Minecraft position predicate property on this typed processor rule definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(processor_rule_value: sand::component::ProcessorRule, predicate: sand::component::RawJson)  {\n    let updated_processor_rule = processor_rule_value.position_predicate(predicate);\n}",
    )]
    pub fn position_predicate(mut self, predicate: RawJson) -> Self {
        self.position_predicate = Some(predicate);
        self
    }

    /// Sets the Minecraft output nbt property on this typed processor rule definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorRule::output_nbt",
        aliases = ["sand::prelude::ProcessorRule::output_nbt"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft output nbt property on this typed processor rule definition and returns the updated builder.",
        context = "Sets the Minecraft output nbt property on this typed processor rule definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(nbt = "`nbt` provides the NBT payload used to set the Minecraft output nbt property on this typed processor rule definition and returns the updated builder."),
        returns = "Sets the Minecraft output nbt property on this typed processor rule definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(processor_rule_value: sand::component::ProcessorRule, nbt: impl Into < String >)  {\n    let updated_processor_rule = processor_rule_value.output_nbt(nbt);\n}",
    )]
    pub fn output_nbt(mut self, nbt: impl Into<String>) -> Self {
        self.output_nbt = Some(nbt.into());
        self
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            "input_predicate".into(),
            self.input_predicate.as_value().clone(),
        );
        if let Some(predicate) = &self.location_predicate {
            map.insert("location_predicate".into(), predicate.as_value().clone());
        }
        if let Some(predicate) = &self.position_predicate {
            map.insert("position_predicate".into(), predicate.as_value().clone());
        }
        map.insert("output_state".into(), self.output_state.to_json());
        if let Some(nbt) = &self.output_nbt {
            map.insert("output_nbt".into(), Value::String(nbt.clone()));
        }
        Value::Object(map)
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        validation::require_json_object(
            location,
            KIND,
            &format!("{field}.input_predicate"),
            self.input_predicate.as_value(),
        )?;
        if let Some(predicate) = &self.location_predicate {
            validation::require_json_object(
                location,
                KIND,
                &format!("{field}.location_predicate"),
                predicate.as_value(),
            )?;
        }
        if let Some(predicate) = &self.position_predicate {
            validation::require_json_object(
                location,
                KIND,
                &format!("{field}.position_predicate"),
                predicate.as_value(),
            )?;
        }
        self.output_state
            .validate(location, KIND, &format!("{field}.output_state"))?;
        if let Some(nbt) = &self.output_nbt {
            validation::require_non_empty(location, KIND, &format!("{field}.output_nbt"), nbt)?;
            validation::reject_whitespace_only(
                location,
                KIND,
                &format!("{field}.output_nbt"),
                nbt,
            )?;
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Processor",
    aliases = ["sand::prelude::Processor"],
    module = "sand::component",
    summary = "A single structure-processing step.",
    context = "A single structure-processing step. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Processor;",
    variants(BlockIgnore = "`minecraft:block_ignore` — blocks in this list are skipped entirely.", Gravity = "`minecraft:gravity` — applies gravity to loose blocks relative to a heightmap.", JigsawReplacement = "`minecraft:jigsaw_replacement` — replaces jigsaw marker blocks; no fields.", ProtectedBlocks = "`minecraft:protected_blocks` — blocks matching this tag are preserved.", Raw = "An explicitly raw processor object for unsupported or modded types.", Rule = "`minecraft:rule` — replaces matching block states with typed rules."),
    variant_fields(BlockIgnore = ["`minecraft:block_ignore` — blocks in this list are skipped entirely."], Gravity(heightmap = "`heightmap` provides the heightmap when `minecraft:gravity` — applies gravity to loose blocks relative to a heightmap.", offset = "`offset` provides the offset when `minecraft:gravity` — applies gravity to loose blocks relative to a heightmap."), ProtectedBlocks = ["`minecraft:protected_blocks` — blocks matching this tag are preserved."], Raw = ["An explicitly raw processor object for unsupported or modded types."], Rule = ["`minecraft:rule` — replaces matching block states with typed rules."]),
)]
/// A single structure-processing step.
#[derive(Debug, Clone, PartialEq)]
pub enum Processor {
    /// `minecraft:block_ignore` — blocks in this list are skipped entirely.
    BlockIgnore(
        #[doc = "`minecraft:block_ignore` — blocks in this list are skipped entirely."]
        Vec<BlockId>,
    ),
    /// `minecraft:protected_blocks` — blocks matching this tag are preserved.
    ProtectedBlocks(
        #[doc = "`minecraft:protected_blocks` — blocks matching this tag are preserved."]
        TagId<BlockId>,
    ),
    /// `minecraft:gravity` — applies gravity to loose blocks relative to a heightmap.
    Gravity {
        #[doc = "`heightmap` provides the heightmap when `minecraft:gravity` — applies gravity to loose blocks relative to a heightmap."]
        heightmap: Heightmap,
        #[doc = "`offset` provides the offset when `minecraft:gravity` — applies gravity to loose blocks relative to a heightmap."]
        offset: i32,
    },
    /// `minecraft:jigsaw_replacement` — replaces jigsaw marker blocks; no fields.
    JigsawReplacement,
    /// `minecraft:rule` — replaces matching block states with typed rules.
    Rule(
        #[doc = "`minecraft:rule` — replaces matching block states with typed rules."]
        Vec<ProcessorRule>,
    ),
    /// An explicitly raw processor object for unsupported or modded types.
    Raw(#[doc = "An explicitly raw processor object for unsupported or modded types."] RawJson),
}

impl Processor {
    fn to_json(&self) -> Value {
        match self {
            Self::BlockIgnore(blocks) => serde_json::json!({
                "processor_type": "minecraft:block_ignore",
                "blocks": blocks.iter().map(BlockId::to_string).collect::<Vec<_>>(),
            }),
            Self::ProtectedBlocks(tag) => serde_json::json!({
                "processor_type": "minecraft:protected_blocks",
                "value": tag.to_tag_string(),
            }),
            Self::Gravity { heightmap, offset } => serde_json::json!({
                "processor_type": "minecraft:gravity",
                "heightmap": heightmap.as_str(),
                "offset": offset,
            }),
            Self::JigsawReplacement => serde_json::json!({
                "processor_type": "minecraft:jigsaw_replacement",
            }),
            Self::Rule(rules) => serde_json::json!({
                "processor_type": "minecraft:rule",
                "rules": rules.iter().map(ProcessorRule::to_json).collect::<Vec<_>>(),
            }),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    fn validate(&self, location: &ResourceLocation, index: usize) -> SandResult<()> {
        let field = format!("processors[{index}]");
        match self {
            Self::BlockIgnore(blocks) => {
                validation::require_non_empty_collection(
                    location,
                    KIND,
                    &format!("{field}.blocks"),
                    blocks.len(),
                )?;
                for (block_index, block) in blocks.iter().enumerate() {
                    validation::validate_resource_location_str(
                        location,
                        KIND,
                        &format!("{field}.blocks[{block_index}]"),
                        &block.to_string(),
                    )?;
                }
            }
            Self::ProtectedBlocks(tag) => {
                validation::validate_resource_or_tag_location_str(
                    location,
                    KIND,
                    &format!("{field}.value"),
                    &tag.to_tag_string(),
                )?;
            }
            Self::Gravity { .. } | Self::JigsawReplacement => {}
            Self::Rule(rules) => {
                validation::require_non_empty_collection(
                    location,
                    KIND,
                    &format!("{field}.rules"),
                    rules.len(),
                )?;
                for (rule_index, rule) in rules.iter().enumerate() {
                    rule.validate(location, &format!("{field}.rules[{rule_index}]"))?;
                }
            }
            Self::Raw(raw) => {
                validation::require_json_object(location, KIND, &field, raw.as_value())?;
                let processor_type = raw.as_value().get("processor_type").and_then(Value::as_str);
                match processor_type {
                    Some(ty) if !ty.trim().is_empty() => {
                        validation::validate_resource_location_str(
                            location,
                            KIND,
                            &format!("{field}.processor_type"),
                            ty,
                        )?;
                    }
                    _ => {
                        return Err(validation::error(
                            location,
                            KIND,
                            &format!("{field}.processor_type"),
                            "raw processor must be a JSON object with a non-empty string `processor_type` field",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ProcessorList",
    aliases = ["sand::prelude::ProcessorList"],
    module = "sand::component",
    summary = "A processor list definition (`data/<namespace>/worldgen/processor_list/<id>.json`).",
    context = "A processor list definition (`data/<namespace>/worldgen/processor_list/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ProcessorList;",
)]
/// A processor list definition (`data/<namespace>/worldgen/processor_list/<id>.json`).
///
/// ```
/// use sand_components::{BlockId, DatapackComponent, ResourceLocation};
/// use sand_components::worldgen::processor_list::{Processor, ProcessorList};
///
/// let processors = ProcessorList::new(
///     ResourceLocation::new("example", "mossify").unwrap(),
///     [Processor::BlockIgnore(vec![BlockId::minecraft("air").unwrap()])],
/// );
/// processors.validate().unwrap();
/// assert_eq!(processors.component_dir(), "worldgen/processor_list");
/// assert_eq!(processors.to_json()["processors"][0]["processor_type"], "minecraft:block_ignore");
/// ```
pub struct ProcessorList {
    location: ResourceLocation,
    processors: Vec<Processor>,
}

impl ProcessorList {
    /// Create a processor list from an ordered sequence of processors.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorList::new",
        aliases = ["sand::prelude::ProcessorList::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a processor list from an ordered sequence of processors.",
        context = "Create a processor list from an ordered sequence of processors. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a processor list from an ordered sequence of processors.", processors = "`processors` supplies the processors value used to create a processor list from an ordered sequence of processors."),
        returns = "A newly constructed `ProcessorList` configured to create a processor list from an ordered sequence of processors.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, processors: impl IntoIterator < Item = sand::component::Processor >)  {\n    let processor_list = sand::component::ProcessorList::new(location, processors);\n}",
    )]
    pub fn new(
        location: ResourceLocation,
        processors: impl IntoIterator<Item = Processor>,
    ) -> Self {
        Self {
            location,
            processors: processors.into_iter().collect(),
        }
    }

    /// `minecraft:empty` — the vanilla no-op processor list shape (empty list).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorList::empty",
        aliases = ["sand::prelude::ProcessorList::empty"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:empty` — the vanilla no-op processor list shape (empty list).",
        context = "`minecraft:empty` — the vanilla no-op processor list shape (empty list). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to emit the documented `minecraft:empty` — the vanilla no-op processor list shape (empty list) form."),
        returns = "A newly constructed `ProcessorList` configured to emit the documented `minecraft:empty` — the vanilla no-op processor list shape (empty list) form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let processor_list = sand::component::ProcessorList::empty(location);\n}",
    )]
    pub fn empty(location: ResourceLocation) -> Self {
        Self::new(location, Vec::new())
    }

    /// Sets the Minecraft processor property on this typed processor list definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorList::processor",
        aliases = ["sand::prelude::ProcessorList::processor"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft processor property on this typed processor list definition and returns the updated builder.",
        context = "Sets the Minecraft processor property on this typed processor list definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(processor = "`processor` supplies the processor value used to set the Minecraft processor property on this typed processor list definition and returns the updated builder."),
        returns = "Sets the Minecraft processor property on this typed processor list definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(processor_list_value: sand::component::ProcessorList, processor: sand::component::Processor)  {\n    let updated_processor_list = processor_list_value.processor(processor);\n}",
    )]
    pub fn processor(mut self, processor: Processor) -> Self {
        self.processors.push(processor);
        self
    }

    /// Sets the Minecraft processors property on this typed processor list definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ProcessorList::processors",
        aliases = ["sand::prelude::ProcessorList::processors"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft processors property on this typed processor list definition and returns the updated builder.",
        context = "Sets the Minecraft processors property on this typed processor list definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(processors = "`processors` supplies the processors value used to set the Minecraft processors property on this typed processor list definition and returns the updated builder."),
        returns = "Sets the Minecraft processors property on this typed processor list definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(processor_list_value: sand::component::ProcessorList, processors: impl IntoIterator < Item = sand::component::Processor >)  {\n    let updated_processor_list = processor_list_value.processors(processors);\n}",
    )]
    pub fn processors(mut self, processors: impl IntoIterator<Item = Processor>) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }
}

impl DatapackComponent for ProcessorList {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        for (index, processor) in self.processors.iter().enumerate() {
            processor.validate(&self.location, index)?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "processors": self.processors.iter().map(Processor::to_json).collect::<Vec<_>>(),
        })
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/processor_list"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "mossify").unwrap()
    }

    #[test]
    fn empty_processor_list_matches_vanilla_shape() {
        let list = ProcessorList::empty(location());
        list.validate().unwrap();
        assert_eq!(list.to_json(), serde_json::json!({ "processors": [] }));
        assert_eq!(list.component_dir(), "worldgen/processor_list");
    }

    #[test]
    fn block_ignore_processor_serializes_block_list() {
        let list = ProcessorList::new(
            location(),
            [Processor::BlockIgnore(vec![
                BlockId::minecraft("air").unwrap(),
                BlockId::minecraft("cave_air").unwrap(),
            ])],
        );
        list.validate().unwrap();
        let json = list.to_json();
        assert_eq!(
            json["processors"][0]["processor_type"],
            "minecraft:block_ignore"
        );
        assert_eq!(json["processors"][0]["blocks"][1], "minecraft:cave_air");
    }

    #[test]
    fn gravity_and_jigsaw_replacement_and_protected_blocks_serialize() {
        let list = ProcessorList::new(
            location(),
            [
                Processor::Gravity {
                    heightmap: Heightmap::WorldSurfaceWg,
                    offset: 1,
                },
                Processor::JigsawReplacement,
                Processor::ProtectedBlocks(TagId::minecraft("village_streets").unwrap()),
            ],
        );
        list.validate().unwrap();
        let json = list.to_json();
        assert_eq!(json["processors"][0]["heightmap"], "WORLD_SURFACE_WG");
        assert_eq!(
            json["processors"][1]["processor_type"],
            "minecraft:jigsaw_replacement"
        );
        assert_eq!(json["processors"][2]["value"], "#minecraft:village_streets");
    }

    #[test]
    fn rule_processor_serializes_typed_output_state() {
        let rule = ProcessorRule::new(
            RawJson::new(serde_json::json!({
                "predicate_type": "minecraft:block_match",
                "block": "minecraft:cobblestone",
            })),
            BlockState::new(BlockId::minecraft("mossy_cobblestone").unwrap()),
        );
        let list = ProcessorList::new(location(), [Processor::Rule(vec![rule])]);
        list.validate().unwrap();
        let json = list.to_json();
        assert_eq!(json["processors"][0]["processor_type"], "minecraft:rule");
        assert_eq!(
            json["processors"][0]["rules"][0]["output_state"]["Name"],
            "minecraft:mossy_cobblestone"
        );
    }

    #[test]
    fn raw_processor_escape_hatch_requires_object_with_processor_type() {
        let valid = ProcessorList::new(
            location(),
            [Processor::Raw(RawJson::new(serde_json::json!({
                "processor_type": "mymod:custom",
            })))],
        );
        assert!(valid.validate().is_ok());

        let missing_type = ProcessorList::new(
            location(),
            [Processor::Raw(RawJson::new(serde_json::json!({})))],
        );
        assert!(missing_type.validate().is_err());

        let not_object = ProcessorList::new(
            location(),
            [Processor::Raw(RawJson::new(serde_json::json!([1, 2])))],
        );
        assert!(not_object.validate().is_err());
    }

    #[test]
    fn empty_block_ignore_and_rule_lists_are_rejected() {
        assert!(
            ProcessorList::new(location(), [Processor::BlockIgnore(Vec::new())])
                .validate()
                .is_err()
        );
        assert!(
            ProcessorList::new(location(), [Processor::Rule(Vec::new())])
                .validate()
                .is_err()
        );
    }
}
