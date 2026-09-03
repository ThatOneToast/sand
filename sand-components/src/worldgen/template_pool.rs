//! Builder for `data/<namespace>/worldgen/template_pool/<id>.json`.
//!
//! [`TemplatePool::new`] models the common vanilla jigsaw pool element
//! shapes — single, legacy single, empty, and list — with
//! [`PoolElement::Raw`] (via [`PoolElement::Feature`] or an explicit raw
//! entry) as the escape hatch for unsupported or modded element types.

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{ProcessorListId, StructureTemplateId, TemplatePoolId};
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/template_pool";

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Projection",
    aliases = ["sand::prelude::Projection"],
    module = "sand::component",
    summary = "A jigsaw structure's projection mode against surrounding terrain.",
    context = "A jigsaw structure's projection mode against surrounding terrain. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Projection;",
    variants(Rigid = "The piece is placed exactly as authored, ignoring terrain height.", TerrainMatching = "The piece is translated vertically to match surrounding terrain."),
)]
/// A jigsaw structure's projection mode against surrounding terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    /// The piece is placed exactly as authored, ignoring terrain height.
    Rigid,
    /// The piece is translated vertically to match surrounding terrain.
    TerrainMatching,
}

impl Projection {
    /// The vanilla string written into datapack JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Projection::as_str",
        aliases = ["sand::prelude::Projection::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla string written into datapack JSON.",
        context = "The vanilla string written into datapack JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla string written into datapack JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(projection_value: &sand::component::Projection)  {\n    let as_str = projection_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rigid => "rigid",
            Self::TerrainMatching => "terrain_matching",
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ProcessorsRef",
    aliases = ["sand::prelude::ProcessorsRef"],
    module = "sand::component",
    summary = "The processor list a pool element uses, as either a typed reference or an inline anonymous processor list.",
    context = "The processor list a pool element uses, as either a typed reference or an inline anonymous processor list. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ProcessorsRef;",
    variants(Inline = "An inline anonymous processor list object (`{\"processors\": [...]}`).", Named = "A reference to a `worldgen/processor_list` entry."),
    variant_fields(Inline = ["An inline anonymous processor list object (`{\"processors\": [...]}`)."], Named = ["A reference to a `worldgen/processor_list` entry."]),
)]
/// The processor list a pool element uses, as either a typed reference or an
/// inline anonymous processor list.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorsRef {
    /// A reference to a `worldgen/processor_list` entry.
    Named(#[doc = "A reference to a `worldgen/processor_list` entry."] ProcessorListId),
    /// An inline anonymous processor list object (`{"processors": [...]}`).
    Inline(
        #[doc = "An inline anonymous processor list object (`{\"processors\": [...]}`)."] RawJson,
    ),
}

impl ProcessorsRef {
    fn to_json(&self) -> Value {
        match self {
            Self::Named(id) => Value::String(id.to_string()),
            Self::Inline(raw) => raw.as_value().clone(),
        }
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        match self {
            Self::Named(id) => {
                validation::validate_resource_location_str(location, KIND, field, &id.to_string())
            }
            Self::Inline(raw) => {
                validation::require_json_object(location, KIND, field, raw.as_value())?;
                validation::require_json_array(
                    location,
                    KIND,
                    &format!("{field}.processors"),
                    raw.as_value().get("processors").unwrap_or(&Value::Null),
                )
            }
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::PoolElement",
    aliases = ["sand::prelude::PoolElement"],
    module = "sand::component",
    summary = "One jigsaw pool element (the `element` payload inside a weighted entry).",
    context = "One jigsaw pool element (the `element` payload inside a weighted entry). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::PoolElement;",
    variants(Empty = "`minecraft:empty_pool_element` — a placeholder that generates nothing.", Feature = "`minecraft:feature_pool_element` — places a configured feature.", LegacySingle = "`minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.", List = "`minecraft:list_pool_element` — an ordered list of alternative elements, the first of which that can be placed is used.", Raw = "An explicitly raw pool element object for unsupported or modded types.", Single = "`minecraft:single_pool_element` — a single `.nbt` structure template."),
    variant_fields(Feature(feature = "`feature` provides the feature identifier when `minecraft:feature_pool_element` — places a configured feature.", projection = "`projection` provides the projection when `minecraft:feature_pool_element` — places a configured feature."), LegacySingle(location = "`location` provides the location identifier when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.", processors = "`processors` provides the processors when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.", projection = "`projection` provides the projection when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates."), List(elements = "`elements` provides the elements when `minecraft:list_pool_element` — an ordered list of alternative elements, the first of which that can be placed is used.", projection = "`projection` provides the projection when `minecraft:list_pool_element` — an ordered list of alternative elements, the first of which that can be placed is used."), Raw = ["An explicitly raw pool element object for unsupported or modded types."], Single(location = "`location` provides the location identifier when `minecraft:single_pool_element` — a single `.nbt` structure template.", processors = "`processors` provides the processors when `minecraft:single_pool_element` — a single `.nbt` structure template.", projection = "`projection` provides the projection when `minecraft:single_pool_element` — a single `.nbt` structure template.")),
)]
/// One jigsaw pool element (the `element` payload inside a weighted entry).
#[derive(Debug, Clone, PartialEq)]
pub enum PoolElement {
    /// `minecraft:single_pool_element` — a single `.nbt` structure template.
    Single {
        /// `location` provides the location identifier when `minecraft:single_pool_element` — a single `.nbt` structure template.
        location: StructureTemplateId,
        /// `processors` provides the processors when `minecraft:single_pool_element` — a single `.nbt` structure template.
        processors: ProcessorsRef,
        /// `projection` provides the projection when `minecraft:single_pool_element` — a single `.nbt` structure template.
        projection: Projection,
    },
    /// `minecraft:legacy_single_pool_element` — like `Single`, but preserves
    /// pre-1.13 block/data behavior for legacy structure templates.
    LegacySingle {
        /// `location` provides the location identifier when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.
        location: StructureTemplateId,
        /// `processors` provides the processors when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.
        processors: ProcessorsRef,
        /// `projection` provides the projection when `minecraft:legacy_single_pool_element` — like `Single`, but preserves pre-1.13 block/data behavior for legacy structure templates.
        projection: Projection,
    },
    /// `minecraft:empty_pool_element` — a placeholder that generates nothing.
    Empty,
    /// `minecraft:feature_pool_element` — places a configured feature.
    Feature {
        /// `feature` provides the feature identifier when `minecraft:feature_pool_element` — places a configured feature.
        feature: ResourceLocation,
        /// `projection` provides the projection when `minecraft:feature_pool_element` — places a configured feature.
        projection: Projection,
    },
    /// `minecraft:list_pool_element` — an ordered list of alternative
    /// elements, the first of which that can be placed is used.
    List {
        /// `elements` provides the elements when `minecraft:list_pool_element` — an ordered list of alternative elements, the first of which that can be placed is used.
        elements: Vec<PoolElement>,
        /// `projection` provides the projection when `minecraft:list_pool_element` — an ordered list of alternative elements, the first of which that can be placed is used.
        projection: Projection,
    },
    /// An explicitly raw pool element object for unsupported or modded types.
    Raw(#[doc = "An explicitly raw pool element object for unsupported or modded types."] RawJson),
}

impl PoolElement {
    /// A `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PoolElement::single",
        aliases = ["sand::prelude::PoolElement::single"],
        module = "sand::component",
        kind = "method",
        summary = "A `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection.",
        context = "A `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to use a `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection."),
        returns = "A `PoolElement` configured for a `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::registry::StructureTemplateId)  {\n    let pool_element = sand::component::PoolElement::single(location);\n}",
    )]
    pub fn single(location: StructureTemplateId) -> Self {
        Self::Single {
            location,
            processors: ProcessorsRef::Named(ProcessorListId::empty()),
            projection: Projection::Rigid,
        }
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Single {
                location,
                processors,
                projection,
            } => serde_json::json!({
                "element_type": "minecraft:single_pool_element",
                "location": location.to_string(),
                "processors": processors.to_json(),
                "projection": projection.as_str(),
            }),
            Self::LegacySingle {
                location,
                processors,
                projection,
            } => serde_json::json!({
                "element_type": "minecraft:legacy_single_pool_element",
                "location": location.to_string(),
                "processors": processors.to_json(),
                "projection": projection.as_str(),
            }),
            Self::Empty => serde_json::json!({
                "element_type": "minecraft:empty_pool_element",
            }),
            Self::Feature {
                feature,
                projection,
            } => serde_json::json!({
                "element_type": "minecraft:feature_pool_element",
                "feature": feature.to_string(),
                "projection": projection.as_str(),
            }),
            Self::List {
                elements,
                projection,
            } => serde_json::json!({
                "element_type": "minecraft:list_pool_element",
                "elements": elements.iter().map(PoolElement::to_json).collect::<Vec<_>>(),
                "projection": projection.as_str(),
            }),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        match self {
            Self::Single {
                location: template,
                processors,
                ..
            }
            | Self::LegacySingle {
                location: template,
                processors,
                ..
            } => {
                validation::validate_resource_location_str(
                    location,
                    KIND,
                    &format!("{field}.location"),
                    &template.to_string(),
                )?;
                processors.validate(location, &format!("{field}.processors"))?;
            }
            Self::Empty => {}
            Self::Feature { feature, .. } => {
                validation::validate_resource_location_str(
                    location,
                    KIND,
                    &format!("{field}.feature"),
                    &feature.to_string(),
                )?;
            }
            Self::List { elements, .. } => {
                validation::require_non_empty_collection(
                    location,
                    KIND,
                    &format!("{field}.elements"),
                    elements.len(),
                )?;
                for (index, element) in elements.iter().enumerate() {
                    element.validate(location, &format!("{field}.elements[{index}]"))?;
                }
            }
            Self::Raw(raw) => {
                validation::require_json_object(location, KIND, field, raw.as_value())?;
                let element_type = raw.as_value().get("element_type").and_then(Value::as_str);
                match element_type {
                    Some(ty) if !ty.trim().is_empty() => {
                        validation::validate_resource_location_str(
                            location,
                            KIND,
                            &format!("{field}.element_type"),
                            ty,
                        )?;
                    }
                    _ => {
                        return Err(validation::error(
                            location,
                            KIND,
                            &format!("{field}.element_type"),
                            "raw pool element must be a JSON object with a non-empty string `element_type` field",
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
    path = "sand::component::PoolEntry",
    aliases = ["sand::prelude::PoolEntry"],
    module = "sand::component",
    summary = "A weighted pool element entry.",
    context = "A weighted pool element entry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::PoolEntry;",
)]
/// A weighted pool element entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolEntry {
    element: PoolElement,
    weight: u32,
}

impl PoolEntry {
    /// `weight` must be at least 1; checked on export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::PoolEntry::new",
        aliases = ["sand::prelude::PoolEntry::new"],
        module = "sand::component",
        kind = "method",
        summary = "`weight` must be at least 1; checked on export.",
        context = "`weight` must be at least 1; checked on export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(element = "`element` supplies the documented `weight` must be at least 1; checked on export form.", weight = "`weight` must be at least 1; checked on export."),
        returns = "A `PoolEntry` that emits the documented `weight` must be at least 1; checked on export form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(element: sand::component::PoolElement, weight: u32)  {\n    let pool_entry = sand::component::PoolEntry::new(element, weight);\n}",
    )]
    pub fn new(element: PoolElement, weight: u32) -> Self {
        Self { element, weight }
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "element": self.element.to_json(),
            "weight": self.weight,
        })
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        self.element
            .validate(location, &format!("{field}.element"))?;
        if self.weight == 0 {
            return Err(validation::error(
                location,
                KIND,
                &format!("{field}.weight"),
                "pool entry weight must be at least 1",
            ));
        }
        Ok(())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TemplatePool",
    aliases = ["sand::prelude::TemplatePool"],
    module = "sand::component",
    summary = "A template pool definition (`data/<namespace>/worldgen/template_pool/<id>.json`).",
    context = "A template pool definition (`data/<namespace>/worldgen/template_pool/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TemplatePool;",
)]
/// A template pool definition (`data/<namespace>/worldgen/template_pool/<id>.json`).
///
/// ```
/// use sand_components::{DatapackComponent, ResourceLocation, StructureTemplateId, TemplatePoolId};
/// use sand_components::worldgen::template_pool::{PoolElement, PoolEntry, TemplatePool};
///
/// let pool = TemplatePool::new(
///     ResourceLocation::new("example", "town_centers").unwrap(),
///     TemplatePoolId::empty(),
///     [PoolEntry::new(
///         PoolElement::single(StructureTemplateId::minecraft("village/plains/town_centers/1").unwrap()),
///         1,
///     )],
/// );
/// pool.validate().unwrap();
/// assert_eq!(pool.component_dir(), "worldgen/template_pool");
/// assert_eq!(pool.to_json()["fallback"], "minecraft:empty");
/// ```
pub struct TemplatePool {
    location: ResourceLocation,
    fallback: TemplatePoolId,
    elements: Vec<PoolEntry>,
}

impl TemplatePool {
    /// Create a template pool with an explicit fallback and weighted elements.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TemplatePool::new",
        aliases = ["sand::prelude::TemplatePool::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a template pool with an explicit fallback and weighted elements.",
        context = "Create a template pool with an explicit fallback and weighted elements. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a template pool with an explicit fallback and weighted elements.", fallback = "`fallback` provides the typed Minecraft resource identifier used to create a template pool with an explicit fallback and weighted elements.", elements = "`elements` is used when creating a template pool with an explicit fallback and weighted elements."),
        returns = "A `TemplatePool` representing a template pool with an explicit fallback and weighted elements.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, fallback: sand::registry::TemplatePoolId, elements: impl IntoIterator < Item = sand::component::PoolEntry >)  {\n    let template_pool = sand::component::TemplatePool::new(location, fallback, elements);\n}",
    )]
    pub fn new(
        location: ResourceLocation,
        fallback: TemplatePoolId,
        elements: impl IntoIterator<Item = PoolEntry>,
    ) -> Self {
        Self {
            location,
            fallback,
            elements: elements.into_iter().collect(),
        }
    }

    /// Sets the Minecraft fallback property on this typed template pool definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TemplatePool::fallback",
        aliases = ["sand::prelude::TemplatePool::fallback"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft fallback property on this typed template pool definition and returns the updated builder.",
        context = "Sets the Minecraft fallback property on this typed template pool definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(fallback = "`fallback` provides the typed Minecraft resource identifier used to set the Minecraft fallback property on this typed template pool definition and returns the updated builder."),
        returns = "Sets the Minecraft fallback property on this typed template pool definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(template_pool_value: sand::component::TemplatePool, fallback: sand::registry::TemplatePoolId)  {\n    let updated_template_pool = template_pool_value.fallback(fallback);\n}",
    )]
    pub fn fallback(mut self, fallback: TemplatePoolId) -> Self {
        self.fallback = fallback;
        self
    }

    /// Sets the Minecraft element property on this typed template pool definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TemplatePool::element",
        aliases = ["sand::prelude::TemplatePool::element"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft element property on this typed template pool definition and returns the updated builder.",
        context = "Sets the Minecraft element property on this typed template pool definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entry = "`entry` provides the entry applied when setting the Minecraft element property on this typed template pool definition and returns the updated builder."),
        returns = "Sets the Minecraft element property on this typed template pool definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(template_pool_value: sand::component::TemplatePool, entry: sand::component::PoolEntry)  {\n    let updated_template_pool = template_pool_value.element(entry);\n}",
    )]
    pub fn element(mut self, entry: PoolEntry) -> Self {
        self.elements.push(entry);
        self
    }

    /// Sets the Minecraft elements property on this typed template pool definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TemplatePool::elements",
        aliases = ["sand::prelude::TemplatePool::elements"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft elements property on this typed template pool definition and returns the updated builder.",
        context = "Sets the Minecraft elements property on this typed template pool definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(elements = "`elements` provides the elements applied when setting the Minecraft elements property on this typed template pool definition and returns the updated builder."),
        returns = "Sets the Minecraft elements property on this typed template pool definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(template_pool_value: sand::component::TemplatePool, elements: impl IntoIterator < Item = sand::component::PoolEntry >)  {\n    let updated_template_pool = template_pool_value.elements(elements);\n}",
    )]
    pub fn elements(mut self, elements: impl IntoIterator<Item = PoolEntry>) -> Self {
        self.elements = elements.into_iter().collect();
        self
    }
}

impl DatapackComponent for TemplatePool {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        validation::validate_resource_location_str(
            &self.location,
            KIND,
            "fallback",
            &self.fallback.to_string(),
        )?;
        validation::require_non_empty_collection(
            &self.location,
            KIND,
            "elements",
            self.elements.len(),
        )?;
        for (index, entry) in self.elements.iter().enumerate() {
            entry.validate(&self.location, &format!("elements[{index}]"))?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("name".into(), Value::String(self.location.to_string()));
        map.insert("fallback".into(), Value::String(self.fallback.to_string()));
        map.insert(
            "elements".into(),
            Value::Array(self.elements.iter().map(PoolEntry::to_json).collect()),
        );
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/template_pool"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "town_centers").unwrap()
    }

    fn template() -> StructureTemplateId {
        StructureTemplateId::minecraft("village/plains/town_centers/1").unwrap()
    }

    #[test]
    fn minimal_template_pool_matches_vanilla_shape() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [PoolEntry::new(PoolElement::single(template()), 1)],
        );
        pool.validate().unwrap();
        assert_eq!(
            pool.to_json(),
            serde_json::json!({
                "name": "test:town_centers",
                "fallback": "minecraft:empty",
                "elements": [
                    {
                        "element": {
                            "element_type": "minecraft:single_pool_element",
                            "location": "minecraft:village/plains/town_centers/1",
                            "processors": "minecraft:empty",
                            "projection": "rigid",
                        },
                        "weight": 1,
                    }
                ]
            })
        );
        assert_eq!(pool.component_dir(), "worldgen/template_pool");
    }

    #[test]
    fn legacy_single_feature_empty_and_list_elements_serialize() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [
                PoolEntry::new(
                    PoolElement::LegacySingle {
                        location: template(),
                        processors: ProcessorsRef::Named(
                            ProcessorListId::minecraft("mossify").unwrap(),
                        ),
                        projection: Projection::TerrainMatching,
                    },
                    1,
                ),
                PoolEntry::new(PoolElement::Empty, 1),
                PoolEntry::new(
                    PoolElement::Feature {
                        feature: ResourceLocation::minecraft("oak").unwrap(),
                        projection: Projection::Rigid,
                    },
                    1,
                ),
                PoolEntry::new(
                    PoolElement::List {
                        elements: vec![PoolElement::single(template()), PoolElement::Empty],
                        projection: Projection::Rigid,
                    },
                    1,
                ),
            ],
        );
        pool.validate().unwrap();
        let json = pool.to_json();
        assert_eq!(
            json["elements"][0]["element"]["element_type"],
            "minecraft:legacy_single_pool_element"
        );
        assert_eq!(
            json["elements"][0]["element"]["projection"],
            "terrain_matching"
        );
        assert_eq!(
            json["elements"][1]["element"]["element_type"],
            "minecraft:empty_pool_element"
        );
        assert_eq!(json["elements"][2]["element"]["feature"], "minecraft:oak");
        assert_eq!(
            json["elements"][3]["element"]["element_type"],
            "minecraft:list_pool_element"
        );
        assert_eq!(
            json["elements"][3]["element"]["elements"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn inline_processors_and_raw_element_escape_hatches_work() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [
                PoolEntry::new(
                    PoolElement::Single {
                        location: template(),
                        processors: ProcessorsRef::Inline(RawJson::new(serde_json::json!({
                            "processors": [
                                { "processor_type": "minecraft:block_ignore", "blocks": ["minecraft:air"] }
                            ]
                        }))),
                        projection: Projection::Rigid,
                    },
                    1,
                ),
                PoolEntry::new(
                    PoolElement::Raw(RawJson::new(serde_json::json!({
                        "element_type": "mymod:custom_element",
                    }))),
                    1,
                ),
            ],
        );
        pool.validate().unwrap();
    }

    #[test]
    fn empty_elements_list_is_rejected() {
        let pool = TemplatePool::new(location(), TemplatePoolId::empty(), Vec::new());
        assert!(pool.validate().is_err());
    }

    #[test]
    fn zero_weight_entry_is_rejected() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [PoolEntry::new(PoolElement::single(template()), 0)],
        );
        assert!(pool.validate().is_err());
    }

    #[test]
    fn raw_element_without_element_type_is_rejected() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [PoolEntry::new(
                PoolElement::Raw(RawJson::new(serde_json::json!({}))),
                1,
            )],
        );
        assert!(pool.validate().is_err());
    }

    #[test]
    fn empty_nested_list_is_rejected() {
        let pool = TemplatePool::new(
            location(),
            TemplatePoolId::empty(),
            [PoolEntry::new(
                PoolElement::List {
                    elements: Vec::new(),
                    projection: Projection::Rigid,
                },
                1,
            )],
        );
        assert!(pool.validate().is_err());
    }
}
