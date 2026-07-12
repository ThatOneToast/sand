use std::marker::PhantomData;

use serde_json::{Value, json};

use crate::component::DatapackComponent;
use crate::error::{Result, SandError};
use crate::registry::{BlockId, EntityTypeId, FunctionId, ItemId, TagId};
use crate::resource_location::ResourceLocation;

/// A Minecraft tag file that groups entities, items, blocks, or other objects together.
pub struct Tag {
    /// The resource location for this tag.
    pub location: ResourceLocation,
    /// Whether this tag replaces existing tag definitions.
    pub replace: bool,
    /// List of tag entries (item/block/entity IDs or tag references).
    pub values: Vec<String>,
}

impl Tag {
    /// Create a new tag with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            replace: false,
            values: Vec::new(),
        }
    }

    /// Add a single entry to this tag.
    pub fn entry(mut self, id: impl std::fmt::Display) -> Self {
        self.values.push(id.to_string());
        self
    }

    /// Add a reference to another tag (prefixed with `#`).
    pub fn tag_ref(mut self, tag: impl std::fmt::Display) -> Self {
        self.values.push(format!("#{tag}"));
        self
    }

    /// Set whether this tag replaces existing tag definitions.
    pub fn replace(mut self, v: bool) -> Self {
        self.replace = v;
        self
    }
}

impl DatapackComponent for Tag {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "replace": self.replace,
            "values": self.values,
        })
    }

    fn component_dir(&self) -> &'static str {
        "tags"
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Registry marker implemented by IDs that have a vanilla datapack tag directory.
///
/// This sealed mapping mirrors `registry_coverage::TAG_COVERAGE`; it prevents an
/// item tag from being exported under `tags/block`, or from accepting a block ID
/// by accident.
pub trait TagRegistry: sealed::Sealed + Sized + std::fmt::Display {
    /// Registry whose values the tag contains.
    const REGISTRY_KEY: &'static str;
    /// Directory relative to `data/<namespace>/`.
    const TAG_DIR: &'static str;
}

macro_rules! tag_registry {
    ($ty:ty, $key:literal, $dir:literal) => {
        impl sealed::Sealed for $ty {}
        impl TagRegistry for $ty {
            const REGISTRY_KEY: &'static str = $key;
            const TAG_DIR: &'static str = $dir;
        }
    };
}

tag_registry!(ItemId, "minecraft:item", "tags/item");
tag_registry!(BlockId, "minecraft:block", "tags/block");
tag_registry!(EntityTypeId, "minecraft:entity_type", "tags/entity_type");
tag_registry!(FunctionId, "minecraft:function", "tags/function");

#[derive(Debug, Clone, PartialEq, Eq)]
enum EntryKind<T> {
    Value(T),
    Tag(TagId<T>),
    Raw(String),
}

/// One registry-checked entry in a [`TypedTag`].
///
/// Required entries serialize as strings. Optional entries use vanilla's
/// `{ "id": ..., "required": false }` form. Raw constructors validate the
/// resource location and normalize tag references to exactly one leading `#`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagEntry<T> {
    kind: EntryKind<T>,
    required: bool,
}

impl<T> TagEntry<T> {
    /// Create a required registry value entry.
    pub fn value(value: T) -> Self {
        Self {
            kind: EntryKind::Value(value),
            required: true,
        }
    }

    /// Create an optional registry value entry.
    pub fn optional_value(value: T) -> Self {
        Self {
            kind: EntryKind::Value(value),
            required: false,
        }
    }

    /// Create a required reference to another tag in the same registry.
    pub fn tag(tag: TagId<T>) -> Self {
        Self {
            kind: EntryKind::Tag(tag),
            required: true,
        }
    }

    /// Create an optional reference to another tag in the same registry.
    pub fn optional_tag(tag: TagId<T>) -> Self {
        Self {
            kind: EntryKind::Tag(tag),
            required: false,
        }
    }

    /// Validated escape hatch for a value ID or `#`-prefixed tag reference.
    pub fn raw(id: impl AsRef<str>) -> Result<Self> {
        Self::raw_with_required(id.as_ref(), true)
    }

    /// Validated optional escape hatch for a value ID or tag reference.
    pub fn optional_raw(id: impl AsRef<str>) -> Result<Self> {
        Self::raw_with_required(id.as_ref(), false)
    }

    fn raw_with_required(id: &str, required: bool) -> Result<Self> {
        let (tag, plain) = match id.strip_prefix('#') {
            Some(rest) => (true, rest),
            None => (false, id),
        };
        if plain.starts_with('#') {
            return Err(SandError::InvalidPath(id.to_owned()));
        }
        let parsed: ResourceLocation = plain.parse()?;
        let normalized = if tag {
            format!("#{parsed}")
        } else {
            parsed.to_string()
        };
        Ok(Self {
            kind: EntryKind::Raw(normalized),
            required,
        })
    }
}

impl<T: std::fmt::Display> TagEntry<T> {
    fn id(&self) -> String {
        match &self.kind {
            EntryKind::Value(value) => value.to_string(),
            EntryKind::Tag(tag) => tag.to_tag_string(),
            EntryKind::Raw(id) => id.clone(),
        }
    }

    fn to_json(&self) -> Value {
        let id = self.id();
        if self.required {
            Value::String(id)
        } else {
            json!({"id": id, "required": false})
        }
    }
}

/// A tag whose entries and output directory are tied to registry type `T`.
///
/// Entries retain insertion order, including duplicates, matching the legacy
/// [`Tag`] behavior and making output deterministic. Empty typed tags are
/// rejected by default; call [`TypedTag::allow_empty`] when an intentionally
/// empty tag is required.
///
/// ```compile_fail
/// use sand_components::{BlockId, ItemId, TagId, TypedTag};
/// let tag = TypedTag::<ItemId>::new(TagId::minecraft("example").unwrap())
///     .entry(BlockId::minecraft("stone").unwrap());
/// ```
#[derive(Debug, Clone)]
pub struct TypedTag<T: TagRegistry> {
    location: TagId<T>,
    replace: bool,
    allow_empty: bool,
    values: Vec<TagEntry<T>>,
    _marker: PhantomData<T>,
}

impl<T: TagRegistry> TypedTag<T> {
    /// Create an empty typed tag. Add values or explicitly call `allow_empty(true)`.
    pub fn new(location: TagId<T>) -> Self {
        Self {
            location,
            replace: false,
            allow_empty: false,
            values: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Add a required value.
    pub fn entry(mut self, value: T) -> Self {
        self.values.push(TagEntry::value(value));
        self
    }
    /// Add an optional value.
    pub fn optional_entry(mut self, value: T) -> Self {
        self.values.push(TagEntry::optional_value(value));
        self
    }
    /// Add a required reference to another tag in this registry.
    pub fn tag_ref(mut self, tag: TagId<T>) -> Self {
        self.values.push(TagEntry::tag(tag));
        self
    }
    /// Add an optional reference to another tag in this registry.
    pub fn optional_tag_ref(mut self, tag: TagId<T>) -> Self {
        self.values.push(TagEntry::optional_tag(tag));
        self
    }
    /// Add a validated raw value or tag reference.
    pub fn raw_entry(mut self, id: impl AsRef<str>) -> Result<Self> {
        self.values.push(TagEntry::raw(id)?);
        Ok(self)
    }
    /// Add a validated optional raw value or tag reference.
    pub fn optional_raw_entry(mut self, id: impl AsRef<str>) -> Result<Self> {
        self.values.push(TagEntry::optional_raw(id)?);
        Ok(self)
    }
    /// Add an already constructed typed entry.
    pub fn with_entry(mut self, entry: TagEntry<T>) -> Self {
        self.values.push(entry);
        self
    }

    /// Set vanilla's replacement flag.
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
    /// Permit or reject an empty values array. Empty tags are rejected by default.
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }
    /// Entries in deterministic insertion order. Duplicates are retained.
    pub fn values(&self) -> &[TagEntry<T>] {
        &self.values
    }
}

impl<T: TagRegistry> DatapackComponent for TypedTag<T> {
    fn resource_location(&self) -> &ResourceLocation {
        self.location.as_resource_location()
    }

    fn to_json(&self) -> Value {
        json!({"replace": self.replace, "values": self.values.iter().map(TagEntry::to_json).collect::<Vec<_>>()})
    }

    fn validate(&self) -> Result<()> {
        if self.values.is_empty() && !self.allow_empty {
            return Err(SandError::ComponentValidation {
                location: self.resource_location().clone(),
                kind: T::TAG_DIR.to_owned(),
                field: "values".to_owned(),
                message: "typed tags must contain at least one entry; call allow_empty(true) for an intentional empty tag".to_owned(),
            });
        }
        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        T::TAG_DIR
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use crate::registry_coverage::TAG_COVERAGE;

    #[test]
    fn item_tag_serializes_required_optional_and_refs() {
        let tag = TypedTag::new(TagId::<ItemId>::minecraft("tools").unwrap())
            .entry(ItemId::minecraft("stick").unwrap())
            .optional_entry(ItemId::minecraft("diamond").unwrap())
            .tag_ref(TagId::minecraft("axes").unwrap())
            .optional_tag_ref(TagId::minecraft("hammers").unwrap());
        assert_eq!(tag.component_dir(), "tags/item");
        assert_eq!(
            tag.to_json(),
            json!({"replace": false, "values": [
                "minecraft:stick", {"id":"minecraft:diamond","required":false},
                "#minecraft:axes", {"id":"#minecraft:hammers","required":false}
            ]})
        );
    }

    #[test]
    fn registry_directories_are_correct() {
        assert_eq!(
            TypedTag::<BlockId>::new(TagId::minecraft("x").unwrap()).component_dir(),
            "tags/block"
        );
        assert_eq!(
            TypedTag::<EntityTypeId>::new(TagId::minecraft("x").unwrap()).component_dir(),
            "tags/entity_type"
        );
        assert_eq!(
            TypedTag::<FunctionId>::new(TagId::minecraft("x").unwrap()).component_dir(),
            "tags/function"
        );
    }

    #[test]
    fn typed_registry_mapping_matches_coverage_source() {
        for (key, dir) in [
            (ItemId::REGISTRY_KEY, ItemId::TAG_DIR),
            (BlockId::REGISTRY_KEY, BlockId::TAG_DIR),
            (EntityTypeId::REGISTRY_KEY, EntityTypeId::TAG_DIR),
            (FunctionId::REGISTRY_KEY, FunctionId::TAG_DIR),
        ] {
            assert!(
                TAG_COVERAGE
                    .iter()
                    .any(|row| { row.value_registry == key && row.datapack_dir == dir }),
                "missing TAG_COVERAGE mapping for {key} -> {dir}"
            );
        }
    }

    #[test]
    fn empty_requires_explicit_opt_in() {
        let tag = TypedTag::<ItemId>::new(TagId::minecraft("empty").unwrap());
        assert!(
            matches!(tag.validate(), Err(SandError::ComponentValidation { field, .. }) if field == "values")
        );
        assert!(tag.allow_empty(true).validate().is_ok());
    }

    #[test]
    fn raw_refs_are_validated_and_normalized_once() {
        let tag = TypedTag::<ItemId>::new(TagId::minecraft("raw").unwrap())
            .raw_entry("#modded:tools")
            .unwrap();
        assert_eq!(tag.to_json()["values"][0], "#modded:tools");
        assert!(TagEntry::<ItemId>::raw("##minecraft:tools").is_err());
        assert!(TagEntry::<ItemId>::raw("not valid").is_err());
    }

    #[test]
    fn duplicates_preserve_insertion_order() {
        let tag = TypedTag::new(TagId::<ItemId>::minecraft("dupes").unwrap())
            .entry(ItemId::minecraft("stick").unwrap())
            .entry(ItemId::minecraft("stick").unwrap());
        assert_eq!(
            tag.to_json()["values"],
            json!(["minecraft:stick", "minecraft:stick"])
        );
    }

    #[test]
    fn legacy_tag_output_is_unchanged() {
        let tag = Tag::new("demo:legacy".parse().unwrap())
            .entry("minecraft:stone")
            .tag_ref("minecraft:logs");
        assert_eq!(tag.component_dir(), "tags");
        assert_eq!(
            tag.to_json(),
            json!({"replace":false,"values":["minecraft:stone","#minecraft:logs"]})
        );
    }

    #[test]
    fn fallible_component_export_rejects_empty_typed_tag() {
        let tag = TypedTag::<BlockId>::new(TagId::minecraft("empty").unwrap());
        let error = tag.try_content().unwrap_err().to_string();
        assert!(error.contains("minecraft:empty"));
        assert!(error.contains("tags/block"));
        assert!(error.contains("values"));
    }
}
