use std::marker::PhantomData;

use serde_json::{Value, json};

use crate::component::DatapackComponent;
use crate::error::{Result, SandError};
use crate::registry::{BlockId, EntityTypeId, FunctionId, ItemId, TagId, VillagerTradeId};
use crate::resource_location::ResourceLocation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Tag",
    aliases = ["sand::prelude::Tag"],
    module = "sand::component",
    summary = "A Minecraft tag file that groups entities, items, blocks, or other objects together.",
    context = "A Minecraft tag file that groups entities, items, blocks, or other objects together. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Tag;",
    fields(location = "The resource location for this tag.", values = "List of tag entries (item/block/entity IDs or tag references)."),
)]
/// A Minecraft tag file that groups entities, items, blocks, or other objects together.
pub struct Tag {
    /// The resource location for this tag.
    pub location: ResourceLocation,
    /// Whether this tag replaces existing tag definitions.
    replace: bool,
    /// List of tag entries (item/block/entity IDs or tag references).
    pub values: Vec<String>,
}

impl Tag {
    /// Create a new tag with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Tag::new",
        aliases = ["sand::prelude::Tag::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a new tag with the given resource location.",
        context = "Create a new tag with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new tag with the given resource location."),
        returns = "A `Tag` representing a new tag with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let tag = sand::component::Tag::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            replace: false,
            values: Vec::new(),
        }
    }

    /// Add a single entry to this tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Tag::entry",
        aliases = ["sand::prelude::Tag::entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add a single entry to this tag.",
        context = "Add a single entry to this tag. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to add a single entry to this tag."),
        returns = "The `Tag` value with the documented change applied to add a single entry to this tag.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(tag_value: sand::component::Tag, id: impl std::fmt::Display)  {\n    let updated_tag = tag_value.entry(id);\n}",
    )]
    pub fn entry(mut self, id: impl std::fmt::Display) -> Self {
        self.values.push(id.to_string());
        self
    }

    /// Add a reference to another tag (prefixed with `#`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Tag::tag_ref",
        aliases = ["sand::prelude::Tag::tag_ref"],
        module = "sand::component",
        kind = "method",
        summary = "Add a reference to another tag (prefixed with `#`).",
        context = "Add a reference to another tag (prefixed with `#`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tag = "`tag` provides the tag added when building a reference to another tag (prefixed with `#`)."),
        returns = "The `Tag` value with the documented change applied to add a reference to another tag (prefixed with `#`).",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(tag_value: sand::component::Tag, tag: impl std::fmt::Display)  {\n    let updated_tag = tag_value.tag_ref(tag);\n}",
    )]
    pub fn tag_ref(mut self, tag: impl std::fmt::Display) -> Self {
        self.values.push(format!("#{tag}"));
        self
    }

    /// Set whether this tag replaces existing tag definitions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Tag::replace",
        aliases = ["sand::prelude::Tag::replace"],
        module = "sand::component",
        kind = "method",
        summary = "Set whether this tag replaces existing tag definitions.",
        context = "Set whether this tag replaces existing tag definitions. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether this tag replaces existing tag definitions."),
        returns = "The `Tag` value with the documented change applied to set whether this tag replaces existing tag definitions.",
        example = "use sand::prelude::*;\n\nfn demonstrate(tag_value: sand::component::Tag, v: bool)  {\n    let updated_tag = tag_value.replace(v);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TagRegistry",
    aliases = ["sand::prelude::TagRegistry"],
    module = "sand::component",
    summary = "Registry marker implemented by IDs that have a vanilla datapack tag directory.",
    context = "Registry marker implemented by IDs that have a vanilla datapack tag directory. This sealed mapping mirrors `registry_coverage::TAG_COVERAGE`; it prevents an item tag from being exported under `tags/block`, or from accepting a block ID by accident.",
    minecraft = "This sealed mapping mirrors `registry_coverage::TAG_COVERAGE`; it prevents an item tag from being exported under `tags/block`, or from accepting a block ID by accident.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TagRegistry;",
)]
/// Registry marker implemented by IDs that have a vanilla datapack tag directory.
///
/// This sealed mapping mirrors `registry_coverage::TAG_COVERAGE`; it prevents an
/// item tag from being exported under `tags/block`, or from accepting a block ID
/// by accident.
pub trait TagRegistry: sealed::Sealed + Sized + std::fmt::Display {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagRegistry::REGISTRY_KEY",
        aliases = ["sand::prelude::TagRegistry::REGISTRY_KEY"],
        module = "sand::component",
        kind = "associated_const",
        summary = "Registry whose values the tag contains.",
        context = "Registry whose values the tag contains. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        example = "use sand::component::TagRegistry;",
    )]
    /// Registry whose values the tag contains.
    const REGISTRY_KEY: &'static str;
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagRegistry::TAG_DIR",
        aliases = ["sand::prelude::TagRegistry::TAG_DIR"],
        module = "sand::component",
        kind = "associated_const",
        summary = "Directory relative to `data/<namespace>/`.",
        context = "Directory relative to `data/<namespace>/`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        example = "use sand::component::TagRegistry;",
    )]
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
tag_registry!(
    VillagerTradeId,
    "minecraft:villager_trade",
    "tags/villager_trade"
);

#[derive(Debug, Clone, PartialEq, Eq)]
enum EntryKind<T> {
    Value(T),
    Tag(TagId<T>),
    Raw(String),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TagEntry",
    aliases = ["sand::prelude::TagEntry"],
    module = "sand::component",
    summary = "One registry-checked entry in a [`TypedTag`]. Required entries serialize as strings. Optional entries use vanilla's `{ \"id\": ..., \"required\": false }` form. Raw constructors validate the resource location and normalize tag references to exactly one leading `#`.",
    context = "One registry-checked entry in a [`TypedTag`]. Required entries serialize as strings. Optional entries use vanilla's `{ \"id\": ..., \"required\": false }` form. Raw constructors validate the resource location and normalize tag references to exactly one leading `#`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TagEntry;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::value",
        aliases = ["sand::prelude::TagEntry::value"],
        module = "sand::component",
        kind = "method",
        summary = "Create a required registry value entry.",
        context = "Create a required registry value entry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to create a required registry value entry."),
        returns = "A `TagEntry` representing a required registry value entry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(value: T)  {\n    let tag_entry = sand::component::TagEntry ::< T >::value(value);\n}",
    )]
    pub fn value(value: T) -> Self {
        Self {
            kind: EntryKind::Value(value),
            required: true,
        }
    }

    /// Create an optional registry value entry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::optional_value",
        aliases = ["sand::prelude::TagEntry::optional_value"],
        module = "sand::component",
        kind = "method",
        summary = "Create an optional registry value entry.",
        context = "Create an optional registry value entry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to create an optional registry value entry."),
        returns = "A `TagEntry` representing an optional registry value entry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(value: T)  {\n    let tag_entry = sand::component::TagEntry ::< T >::optional_value(value);\n}",
    )]
    pub fn optional_value(value: T) -> Self {
        Self {
            kind: EntryKind::Value(value),
            required: false,
        }
    }

    /// Create a required reference to another tag in the same registry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::tag",
        aliases = ["sand::prelude::TagEntry::tag"],
        module = "sand::component",
        kind = "method",
        summary = "Create a required reference to another tag in the same registry.",
        context = "Create a required reference to another tag in the same registry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tag = "`tag` is used when creating a required reference to another tag in the same registry."),
        returns = "A `TagEntry` representing a required reference to another tag in the same registry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(tag: sand::component::TagId < T >)  {\n    let tag_entry = sand::component::TagEntry ::< T >::tag(tag);\n}",
    )]
    pub fn tag(tag: TagId<T>) -> Self {
        Self {
            kind: EntryKind::Tag(tag),
            required: true,
        }
    }

    /// Create an optional reference to another tag in the same registry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::optional_tag",
        aliases = ["sand::prelude::TagEntry::optional_tag"],
        module = "sand::component",
        kind = "method",
        summary = "Create an optional reference to another tag in the same registry.",
        context = "Create an optional reference to another tag in the same registry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tag = "`tag` is used when creating an optional reference to another tag in the same registry."),
        returns = "A `TagEntry` representing an optional reference to another tag in the same registry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(tag: sand::component::TagId < T >)  {\n    let tag_entry = sand::component::TagEntry ::< T >::optional_tag(tag);\n}",
    )]
    pub fn optional_tag(tag: TagId<T>) -> Self {
        Self {
            kind: EntryKind::Tag(tag),
            required: false,
        }
    }

    /// Validated escape hatch for a value ID or `#`-prefixed tag reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::raw",
        aliases = ["sand::prelude::TagEntry::raw"],
        module = "sand::component",
        kind = "method",
        summary = "Validated escape hatch for a value ID or `#`-prefixed tag reference.",
        context = "Validated escape hatch for a value ID or `#`-prefixed tag reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to use validated escape hatch for a value ID or `#`-prefixed tag reference."),
        returns = "On success, the value produced to use validated escape hatch for a value ID or `#`-prefixed tag reference; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(id: impl AsRef < str >)  {\n    let raw = sand::component::TagEntry ::< T >::raw(id);\n}",
    )]
    pub fn raw(id: impl AsRef<str>) -> Result<Self> {
        Self::raw_with_required(id.as_ref(), true)
    }

    /// Validated optional escape hatch for a value ID or tag reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TagEntry::optional_raw",
        aliases = ["sand::prelude::TagEntry::optional_raw"],
        module = "sand::component",
        kind = "method",
        summary = "Validated optional escape hatch for a value ID or tag reference.",
        context = "Validated optional escape hatch for a value ID or tag reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to use validated optional escape hatch for a value ID or tag reference."),
        returns = "On success, the value produced to use validated optional escape hatch for a value ID or tag reference; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(id: impl AsRef < str >)  {\n    let optional_raw = sand::component::TagEntry ::< T >::optional_raw(id);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TypedTag",
    aliases = ["sand::prelude::TypedTag"],
    module = "sand::component",
    summary = "A tag whose entries and output directory are tied to registry type `T`.",
    context = "A tag whose entries and output directory are tied to registry type `T`. Entries retain insertion order, including duplicates, matching the legacy [`Tag`] behavior and making output deterministic. Empty typed tags are rejected by default; call [`TypedTag::allow_empty`] when an intentionally empty tag is required.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TypedTag;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::new",
        aliases = ["sand::prelude::TypedTag::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create an empty typed tag. Add values or explicitly call `allow_empty(true)`.",
        context = "Create an empty typed tag. Add values or explicitly call `allow_empty(true)`. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create an empty typed tag. Add values or explicitly call `allow_empty(true)`."),
        returns = "A `TypedTag` representing an empty typed tag. Add values or explicitly call `allow_empty(true)`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(location: sand::component::TagId < T >)  {\n    let typed_tag = sand::component::TypedTag ::< T >::new(location);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::entry",
        aliases = ["sand::prelude::TypedTag::entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add a required value.",
        context = "Add a required value. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to add a required value."),
        returns = "The `TypedTag` value with the documented change applied to add a required value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, value: T)  {\n    let updated_typed_tag = typed_tag_value.entry(value);\n}",
    )]
    pub fn entry(mut self, value: T) -> Self {
        self.values.push(TagEntry::value(value));
        self
    }
    /// Add an optional value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::optional_entry",
        aliases = ["sand::prelude::TypedTag::optional_entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add an optional value.",
        context = "Add an optional value. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to add an optional value."),
        returns = "The `TypedTag` value with the documented change applied to add an optional value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, value: T)  {\n    let updated_typed_tag = typed_tag_value.optional_entry(value);\n}",
    )]
    pub fn optional_entry(mut self, value: T) -> Self {
        self.values.push(TagEntry::optional_value(value));
        self
    }
    /// Add a required reference to another tag in this registry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::tag_ref",
        aliases = ["sand::prelude::TypedTag::tag_ref"],
        module = "sand::component",
        kind = "method",
        summary = "Add a required reference to another tag in this registry.",
        context = "Add a required reference to another tag in this registry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tag = "`tag` provides the tag added when building a required reference to another tag in this registry."),
        returns = "The `TypedTag` value with the documented change applied to add a required reference to another tag in this registry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, tag: sand::component::TagId < T >)  {\n    let updated_typed_tag = typed_tag_value.tag_ref(tag);\n}",
    )]
    pub fn tag_ref(mut self, tag: TagId<T>) -> Self {
        self.values.push(TagEntry::tag(tag));
        self
    }
    /// Add an optional reference to another tag in this registry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::optional_tag_ref",
        aliases = ["sand::prelude::TypedTag::optional_tag_ref"],
        module = "sand::component",
        kind = "method",
        summary = "Add an optional reference to another tag in this registry.",
        context = "Add an optional reference to another tag in this registry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tag = "`tag` provides the tag added when building an optional reference to another tag in this registry."),
        returns = "The `TypedTag` value with the documented change applied to add an optional reference to another tag in this registry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, tag: sand::component::TagId < T >)  {\n    let updated_typed_tag = typed_tag_value.optional_tag_ref(tag);\n}",
    )]
    pub fn optional_tag_ref(mut self, tag: TagId<T>) -> Self {
        self.values.push(TagEntry::optional_tag(tag));
        self
    }
    /// Add a validated raw value or tag reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::raw_entry",
        aliases = ["sand::prelude::TypedTag::raw_entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add a validated raw value or tag reference.",
        context = "Add a validated raw value or tag reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to add a validated raw value or tag reference."),
        returns = "On success, the value produced to add a validated raw value or tag reference; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, id: impl AsRef < str >)  {\n    let raw_entry = typed_tag_value.raw_entry(id);\n}",
    )]
    pub fn raw_entry(mut self, id: impl AsRef<str>) -> Result<Self> {
        self.values.push(TagEntry::raw(id)?);
        Ok(self)
    }
    /// Add a validated optional raw value or tag reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::optional_raw_entry",
        aliases = ["sand::prelude::TypedTag::optional_raw_entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add a validated optional raw value or tag reference.",
        context = "Add a validated optional raw value or tag reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to add a validated optional raw value or tag reference."),
        returns = "On success, the value produced to add a validated optional raw value or tag reference; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, id: impl AsRef < str >)  {\n    let optional_raw_entry = typed_tag_value.optional_raw_entry(id);\n}",
    )]
    pub fn optional_raw_entry(mut self, id: impl AsRef<str>) -> Result<Self> {
        self.values.push(TagEntry::optional_raw(id)?);
        Ok(self)
    }
    /// Add an already constructed typed entry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::with_entry",
        aliases = ["sand::prelude::TypedTag::with_entry"],
        module = "sand::component",
        kind = "method",
        summary = "Add an already constructed typed entry.",
        context = "Add an already constructed typed entry. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(entry = "`entry` provides the entry added when building an already constructed typed entry."),
        returns = "The `TypedTag` value with the documented change applied to add an already constructed typed entry.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, entry: sand::component::TagEntry < T >)  {\n    let updated_typed_tag = typed_tag_value.with_entry(entry);\n}",
    )]
    pub fn with_entry(mut self, entry: TagEntry<T>) -> Self {
        self.values.push(entry);
        self
    }

    /// Set vanilla's replacement flag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::replace",
        aliases = ["sand::prelude::TypedTag::replace"],
        module = "sand::component",
        kind = "method",
        summary = "Set vanilla's replacement flag.",
        context = "Set vanilla's replacement flag. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(replace = "`replace` provides the switch that enables or disables the behavior used to set vanilla's replacement flag."),
        returns = "The `TypedTag` value with the documented change applied to set vanilla's replacement flag.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, replace: bool)  {\n    let updated_typed_tag = typed_tag_value.replace(replace);\n}",
    )]
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
    /// Permit or reject an empty values array. Empty tags are rejected by default.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::allow_empty",
        aliases = ["sand::prelude::TypedTag::allow_empty"],
        module = "sand::component",
        kind = "method",
        summary = "Permit or reject an empty values array. Empty tags are rejected by default.",
        context = "Permit or reject an empty values array. Empty tags are rejected by default. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(allow = "`allow` provides the switch that enables or disables the behavior used to permit or reject an empty values array. Empty tags are rejected by default."),
        returns = "The `TypedTag` value with the documented change applied to permit or reject an empty values array. Empty tags are rejected by default.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: sand::component::TypedTag < T >, allow: bool)  {\n    let updated_typed_tag = typed_tag_value.allow_empty(allow);\n}",
    )]
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }
    /// Entries in deterministic insertion order. Duplicates are retained.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TypedTag::values",
        aliases = ["sand::prelude::TypedTag::values"],
        module = "sand::component",
        kind = "method",
        summary = "Entries in deterministic insertion order. Duplicates are retained.",
        context = "Entries in deterministic insertion order. Duplicates are retained. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `& [TagEntry < T >]` value produced to entrie in deterministic insertion order. Duplicates are retained.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::component::TagRegistry + 'static>(typed_tag_value: &sand::component::TypedTag < T >)  {\n    let values = typed_tag_value.values();\n}",
    )]
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
