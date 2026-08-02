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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rigid => "rigid",
            Self::TerrainMatching => "terrain_matching",
        }
    }
}

/// The processor list a pool element uses, as either a typed reference or an
/// inline anonymous processor list.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorsRef {
    /// A reference to a `worldgen/processor_list` entry.
    Named(ProcessorListId),
    /// An inline anonymous processor list object (`{"processors": [...]}`).
    Inline(RawJson),
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

/// One jigsaw pool element (the `element` payload inside a weighted entry).
#[derive(Debug, Clone, PartialEq)]
pub enum PoolElement {
    /// `minecraft:single_pool_element` — a single `.nbt` structure template.
    Single {
        location: StructureTemplateId,
        processors: ProcessorsRef,
        projection: Projection,
    },
    /// `minecraft:legacy_single_pool_element` — like `Single`, but preserves
    /// pre-1.13 block/data behavior for legacy structure templates.
    LegacySingle {
        location: StructureTemplateId,
        processors: ProcessorsRef,
        projection: Projection,
    },
    /// `minecraft:empty_pool_element` — a placeholder that generates nothing.
    Empty,
    /// `minecraft:feature_pool_element` — places a configured feature.
    Feature {
        feature: ResourceLocation,
        projection: Projection,
    },
    /// `minecraft:list_pool_element` — an ordered list of alternative
    /// elements, the first of which that can be placed is used.
    List {
        elements: Vec<PoolElement>,
        projection: Projection,
    },
    /// An explicitly raw pool element object for unsupported or modded types.
    Raw(RawJson),
}

impl PoolElement {
    /// A `minecraft:single_pool_element` with vanilla `minecraft:empty` processors and rigid projection.
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

/// A weighted pool element entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolEntry {
    element: PoolElement,
    weight: u32,
}

impl PoolEntry {
    /// `weight` must be at least 1; checked on export.
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

    pub fn fallback(mut self, fallback: TemplatePoolId) -> Self {
        self.fallback = fallback;
        self
    }

    pub fn element(mut self, entry: PoolEntry) -> Self {
        self.elements.push(entry);
        self
    }

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
