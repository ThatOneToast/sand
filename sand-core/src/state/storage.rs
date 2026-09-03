//! Typed NBT storage variables backed by `data storage` commands.
#![allow(clippy::result_large_err)]

use std::fmt;
use std::marker::PhantomData;

use crate::condition::Condition;
use sand_commands::{BlockPos, DataTarget, Selector, TargetArgument};
use sand_components::{RawSnbt, ResourceLocation};

pub use sand_commands::{
    DataCommand, Nbt, NbtCompound as SnbtCompound, NbtPath, NbtRef, NbtTarget,
    NbtValue as SnbtValue, UntypedNbt,
};

// ── Storage locations ────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::StorageLocation",
    aliases = ["sand::prelude::StorageLocation", "sand::state::StorageLocation"],
    module = "sand::data",
    summary = "A typed `data storage <id>` target.",
    context = "A typed `data storage <id>` target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::StorageLocation;",
)]
/// A typed `data storage <id>` target.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageLocation {
    id: ResourceLocation,
}

impl StorageLocation {
    /// Creates a command-storage location from a validated resource identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageLocation::new",
        aliases = ["sand::prelude::StorageLocation::new", "sand::state::StorageLocation::new"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a command-storage location from a validated resource identifier.",
        context = "Creates a command-storage location from a validated resource identifier. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(id = "`id` provides the typed resource identifier or location used to create a command-storage location from a validated resource identifier."),
        returns = "A `StorageLocation` representing a command-storage location from a validated resource identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::ResourceLocation)  {\n    let storage_location = sand::data::StorageLocation::new(id);\n}",
    )]
    pub fn new(id: ResourceLocation) -> Self {
        Self { id }
    }

    /// Parses and validates a namespaced command-storage identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageLocation::parse",
        aliases = ["sand::prelude::StorageLocation::parse", "sand::state::StorageLocation::parse"],
        module = "sand::data",
        kind = "method",
        summary = "Parses and validates a namespaced command-storage identifier.",
        context = "Parses and validates a namespaced command-storage identifier. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(id = "`id` provides the typed resource identifier or location used to parse and validates a namespaced command-storage identifier."),
        returns = "The `sand :: component :: Result < Self >` value produced to parse and validates a namespaced command-storage identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl AsRef < str >)  {\n    let storage_location_result = sand::data::StorageLocation::parse(id);\n}",
    )]
    pub fn parse(id: impl AsRef<str>) -> sand_components::Result<Self> {
        Ok(Self::new(id.as_ref().parse()?))
    }

    /// Borrows the validated resource identifier for this storage location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageLocation::as_resource_location",
        aliases = ["sand::prelude::StorageLocation::as_resource_location", "sand::state::StorageLocation::as_resource_location"],
        module = "sand::data",
        kind = "method",
        summary = "Borrows the validated resource identifier for this storage location.",
        context = "Borrows the validated resource identifier for this storage location. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `& ResourceLocation` value produced to borrow the validated resource identifier for this storage location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_location_value: &sand::data::StorageLocation)  {\n    let as_resource_location = storage_location_value.as_resource_location();\n}",
    )]
    pub fn as_resource_location(&self) -> &ResourceLocation {
        &self.id
    }
}

impl fmt::Display for StorageLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

impl From<StorageLocation> for String {
    fn from(value: StorageLocation) -> Self {
        value.to_string()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::EntityNbt",
    aliases = ["sand::prelude::EntityNbt", "sand::state::EntityNbt"],
    module = "sand::data",
    summary = "A typed entity NBT target.",
    context = "A typed entity NBT target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::EntityNbt;",
)]
/// A typed entity NBT target.
#[derive(Debug, Clone)]
pub struct EntityNbt {
    target: Selector,
}

impl EntityNbt {
    /// Creates an entity NBT root bound to the supplied selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::EntityNbt::target",
        aliases = ["sand::prelude::EntityNbt::target", "sand::state::EntityNbt::target"],
        module = "sand::data",
        kind = "method",
        summary = "Creates an entity NBT root bound to the supplied selector.",
        context = "Creates an entity NBT root bound to the supplied selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(target = "`target` provides the entity, block, or command target used to create an entity NBT root bound to the supplied selector."),
        returns = "An `EntityNbt` representing an entity NBT root bound to the supplied selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Target)  {\n    let entity_nbt = sand::data::EntityNbt::target(target);\n}",
    )]
    pub fn target(target: impl TargetArgument) -> Self {
        Self {
            target: target.into_target_selector(),
        }
    }

    /// Extends this typed NBT reference with the supplied path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::EntityNbt::path",
        aliases = ["sand::prelude::EntityNbt::path", "sand::state::EntityNbt::path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied path selector.",
        context = "Extends this typed NBT reference with the supplied path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to extend this typed NBT reference with the supplied path selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(entity_nbt_value: &sand::data::EntityNbt, path: impl Into < sand::data::NbtPath >)  {\n    let path = entity_nbt_value.path::<T>(path);\n}",
    )]
    pub fn path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(DataTarget::entity(self.target.clone()), path.into())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::BlockNbt",
    aliases = ["sand::prelude::BlockNbt", "sand::state::BlockNbt"],
    module = "sand::data",
    summary = "A typed block entity NBT target.",
    context = "A typed block entity NBT target. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::BlockNbt;",
)]
/// A typed block entity NBT target.
#[derive(Debug, Clone)]
pub struct BlockNbt {
    pos: BlockPos,
}

impl BlockNbt {
    /// Creates a block NBT root bound to the supplied coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::BlockNbt::pos",
        aliases = ["sand::prelude::BlockNbt::pos", "sand::state::BlockNbt::pos"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a block NBT root bound to the supplied coordinates.",
        context = "Creates a block NBT root bound to the supplied coordinates. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(pos = "`pos` is used when creating a block NBT root bound to the supplied coordinates."),
        returns = "A `BlockNbt` representing a block NBT root bound to the supplied coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(pos: sand::command::BlockPos)  {\n    let block_nbt = sand::data::BlockNbt::pos(pos);\n}",
    )]
    pub fn pos(pos: BlockPos) -> Self {
        Self { pos }
    }

    /// Extends this typed NBT reference with the supplied path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::BlockNbt::path",
        aliases = ["sand::prelude::BlockNbt::path", "sand::state::BlockNbt::path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied path selector.",
        context = "Extends this typed NBT reference with the supplied path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(path = "`path` provides the typed resource identifier or location used to extend this typed NBT reference with the supplied path selector."),
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(block_nbt_value: &sand::data::BlockNbt, path: impl Into < sand::data::NbtPath >)  {\n    let path = block_nbt_value.path::<T>(path);\n}",
    )]
    pub fn path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(DataTarget::block(self.pos.clone()), path.into())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::NbtLocation",
    aliases = ["sand::prelude::NbtLocation", "sand::state::NbtLocation"],
    module = "sand::data",
    summary = "Compatibility name for the canonical command-layer [`DataTarget`].",
    context = "Compatibility name for the canonical command-layer [`DataTarget`]. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::NbtLocation;",
)]
/// Compatibility name for the canonical command-layer [`DataTarget`].
pub type NbtLocation = DataTarget;

// ── StorageSchema / StorageField ─────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::StorageSchema",
    aliases = ["sand::prelude::StorageSchema", "sand::state::StorageSchema"],
    module = "sand::data",
    summary = "A typed schema rooted at a datapack storage location and NBT path.",
    context = "A typed schema rooted at a datapack storage location and NBT path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::StorageSchema;",
)]
/// A typed schema rooted at a datapack storage location and NBT path.
#[derive(Debug)]
pub struct StorageSchema<T> {
    storage: &'static str,
    root: &'static str,
    _marker: PhantomData<T>,
}

impl<T> Clone for StorageSchema<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for StorageSchema<T> {}

impl<T> StorageSchema<T> {
    /// Defines a typed schema at a command-storage resource and root NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::new",
        aliases = ["sand::prelude::StorageSchema::new", "sand::state::StorageSchema::new"],
        module = "sand::data",
        kind = "method",
        summary = "Defines a typed schema at a command-storage resource and root NBT path.",
        context = "Defines a typed schema at a command-storage resource and root NBT path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(storage = "`storage` provides the storage used when defining a typed schema at a command-storage resource and root NBT path.", root = "`root` provides the root used when defining a typed schema at a command-storage resource and root NBT path."),
        returns = "A `StorageSchema` defining a typed schema at a command-storage resource and root NBT path.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage: & 'static str, root: & 'static str)  {\n    let storage_schema = sand::data::StorageSchema ::< T >::new(storage, root);\n}",
    )]
    pub const fn new(storage: &'static str, root: &'static str) -> Self {
        Self {
            storage,
            root,
            _marker: PhantomData,
        }
    }

    /// Returns the namespaced command-storage identifier used by this schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::storage",
        aliases = ["sand::prelude::StorageSchema::storage", "sand::state::StorageSchema::storage"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the namespaced command-storage identifier used by this schema.",
        context = "Returns the namespaced command-storage identifier used by this schema. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the namespaced command-storage identifier used by this schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let storage = storage_schema_value.storage();\n}",
    )]
    pub const fn storage(&self) -> &'static str {
        self.storage
    }

    /// Returns the schema's root NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::root_path",
        aliases = ["sand::prelude::StorageSchema::root_path", "sand::state::StorageSchema::root_path"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the schema's root NBT path.",
        context = "Returns the schema's root NBT path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the schema's root NBT path.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let root_path = storage_schema_value.root_path();\n}",
    )]
    pub const fn root_path(&self) -> &'static str {
        self.root
    }

    /// Extends this typed NBT reference with the supplied field selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::field",
        aliases = ["sand::prelude::StorageSchema::field", "sand::state::StorageSchema::field"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied field selector.",
        context = "Extends this typed NBT reference with the supplied field selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(field = "`field` is used to extend this typed NBT reference with the supplied field selector."),
        returns = "The `StorageField < T , U >` value produced to extend this typed NBT reference with the supplied field selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static, U: 'static>(storage_schema_value: &sand::data::StorageSchema < T >, field: & 'static str)  {\n    let field = storage_schema_value.field::<U>(field);\n}",
    )]
    pub const fn field<U>(&self, field: &'static str) -> StorageField<T, U> {
        StorageField {
            storage: self.storage,
            root: self.root,
            field,
            _schema: PhantomData,
            _value: PhantomData,
        }
    }

    /// Extends this typed NBT reference with the supplied path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::path",
        aliases = ["sand::prelude::StorageSchema::path", "sand::state::StorageSchema::path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied path selector.",
        context = "Extends this typed NBT reference with the supplied path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let path = storage_schema_value.path();\n}",
    )]
    pub fn path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage).typed_path(self.root)
    }

    /// Returns the typed NBT location targeted by this reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::location",
        aliases = ["sand::prelude::StorageSchema::location", "sand::state::StorageSchema::location"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the typed NBT location targeted by this reference.",
        context = "Returns the typed NBT location targeted by this reference. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the typed NBT location targeted by this reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let location = storage_schema_value.location();\n}",
    )]
    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageSchema::new requires a valid storage resource location")
    }

    /// Builds the typed Minecraft data query for get.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::get",
        aliases = ["sand::prelude::StorageSchema::get", "sand::state::StorageSchema::get"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for get.",
        context = "Builds the typed Minecraft data query for get. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to build the typed Minecraft data query for get.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let get = storage_schema_value.get();\n}",
    )]
    pub fn get(&self) -> String {
        self.path().get().to_string()
    }

    /// Builds the typed Minecraft data modification for set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::set",
        aliases = ["sand::prelude::StorageSchema::set", "sand::state::StorageSchema::set"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set.",
        context = "Builds the typed Minecraft data modification for set. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set."),
        returns = "The string value produced to build the typed Minecraft data modification for set.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >, value: impl Into < SnbtValue >)  {\n    let set = storage_schema_value.set(value);\n}",
    )]
    pub fn set(&self, value: impl Into<SnbtValue>) -> String {
        self.path().set(value).to_string()
    }

    /// Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::set_raw_snbt",
        aliases = ["sand::prelude::StorageSchema::set_raw_snbt", "sand::state::StorageSchema::set_raw_snbt"],
        module = "sand::data",
        kind = "method",
        summary = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        context = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(raw = "`raw` is used to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility."),
        returns = "The string value produced to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >, raw: sand::component::RawSnbt)  {\n    let set_raw_snbt = storage_schema_value.set_raw_snbt(raw);\n}",
    )]
    pub fn set_raw_snbt(&self, raw: RawSnbt) -> String {
        self.path().set_raw(raw.to_string()).to_string()
    }

    /// Builds the typed Minecraft data modification for merge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::merge",
        aliases = ["sand::prelude::StorageSchema::merge", "sand::state::StorageSchema::merge"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge.",
        context = "Builds the typed Minecraft data modification for merge. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for merge."),
        returns = "The string value produced to build the typed Minecraft data modification for merge.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >, value: impl Into < SnbtValue >)  {\n    let merge = storage_schema_value.merge(value);\n}",
    )]
    pub fn merge(&self, value: impl Into<SnbtValue>) -> String {
        self.path().merge(value).to_string()
    }

    /// Builds the typed Minecraft data modification for remove.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::remove",
        aliases = ["sand::prelude::StorageSchema::remove", "sand::state::StorageSchema::remove"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for remove.",
        context = "Builds the typed Minecraft data modification for remove. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to build the typed Minecraft data modification for remove.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let remove = storage_schema_value.remove();\n}",
    )]
    pub fn remove(&self) -> String {
        self.path().remove().to_string()
    }

    /// Builds the typed Minecraft data query for exists.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageSchema::exists",
        aliases = ["sand::prelude::StorageSchema::exists", "sand::state::StorageSchema::exists"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for exists.",
        context = "Builds the typed Minecraft data query for exists. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `Condition` value produced to build the typed Minecraft data query for exists.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_schema_value: &sand::data::StorageSchema < T >)  {\n    let exists = storage_schema_value.exists();\n}",
    )]
    pub fn exists(&self) -> Condition {
        Condition::nbt_exists(DataTarget::storage(self.storage), NbtPath::new(self.root))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::StorageField",
    aliases = ["sand::prelude::StorageField", "sand::state::StorageField"],
    module = "sand::data",
    summary = "A typed field inside a [`StorageSchema`].",
    context = "A typed field inside a [`StorageSchema`]. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::StorageField;",
)]
/// A typed field inside a [`StorageSchema`].
#[derive(Debug)]
pub struct StorageField<Schema, T> {
    storage: &'static str,
    root: &'static str,
    field: &'static str,
    _schema: PhantomData<Schema>,
    _value: PhantomData<T>,
}

impl<Schema, T> Clone for StorageField<Schema, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, T> Copy for StorageField<Schema, T> {}

impl<Schema, T> StorageField<Schema, T> {
    /// Creates a typed field belonging to the supplied storage schema.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::new",
        aliases = ["sand::prelude::StorageField::new", "sand::state::StorageField::new"],
        module = "sand::data",
        kind = "method",
        summary = "Creates a typed field belonging to the supplied storage schema.",
        context = "Creates a typed field belonging to the supplied storage schema. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(schema = "`schema` is used when creating a typed field belonging to the supplied storage schema.", field = "`field` is used when creating a typed field belonging to the supplied storage schema."),
        returns = "A `StorageField` representing a typed field belonging to the supplied storage schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(schema: & sand::data::StorageSchema < Schema >, field: & 'static str)  {\n    let storage_field = sand::data::StorageField ::< Schema , T >::new(schema, field);\n}",
    )]
    pub const fn new(schema: &StorageSchema<Schema>, field: &'static str) -> Self {
        schema.field(field)
    }

    /// Returns the namespaced command-storage identifier containing this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::storage",
        aliases = ["sand::prelude::StorageField::storage", "sand::state::StorageField::storage"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the namespaced command-storage identifier containing this field.",
        context = "Returns the namespaced command-storage identifier containing this field. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the namespaced command-storage identifier containing this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let storage = storage_field_value.storage();\n}",
    )]
    pub const fn storage(&self) -> &'static str {
        self.storage
    }

    /// Returns the containing schema's root NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::root_path",
        aliases = ["sand::prelude::StorageField::root_path", "sand::state::StorageField::root_path"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the containing schema's root NBT path.",
        context = "Returns the containing schema's root NBT path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the containing schema's root NBT path.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let root_path = storage_field_value.root_path();\n}",
    )]
    pub const fn root_path(&self) -> &'static str {
        self.root
    }

    /// Returns this field's name relative to its schema root.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::field_name",
        aliases = ["sand::prelude::StorageField::field_name", "sand::state::StorageField::field_name"],
        module = "sand::data",
        kind = "method",
        summary = "Returns this field's name relative to its schema root.",
        context = "Returns this field's name relative to its schema root. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns this field's name relative to its schema root.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let field_name = storage_field_value.field_name();\n}",
    )]
    pub const fn field_name(&self) -> &'static str {
        self.field
    }

    /// Extends this typed NBT reference with the supplied path selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::path",
        aliases = ["sand::prelude::StorageField::path", "sand::state::StorageField::path"],
        module = "sand::data",
        kind = "method",
        summary = "Extends this typed NBT reference with the supplied path selector.",
        context = "Extends this typed NBT reference with the supplied path selector. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `NbtRef < T >` value produced to extend this typed NBT reference with the supplied path selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let path = storage_field_value.path();\n}",
    )]
    pub fn path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage)
            .typed_path::<T>(self.root)
            .field(self.field)
    }

    /// Returns the complete rendered NBT path to this field.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::full_path",
        aliases = ["sand::prelude::StorageField::full_path", "sand::state::StorageField::full_path"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the complete rendered NBT path to this field.",
        context = "Returns the complete rendered NBT path to this field. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the complete rendered NBT path to this field.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let full_path = storage_field_value.full_path();\n}",
    )]
    pub fn full_path(&self) -> String {
        self.path().path_value().as_str().to_string()
    }

    /// The dot-separated NBT path for this field (`root.field`).
    ///
    /// Alias for [`full_path`](Self::full_path). Useful when passing the path
    /// to a player-scoped command manually, since Minecraft storage is global
    /// and does not have automatic per-player keying.
    ///
    /// ```text
    /// // Manually build a per-player storage write:
    /// let path = PlayerMagic::mana().field_path();
    /// let cmd  = format!("data modify storage powers:players {path} set value 100");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::field_path",
        aliases = ["sand::prelude::StorageField::field_path", "sand::state::StorageField::field_path"],
        module = "sand::data",
        kind = "method",
        summary = "The dot-separated NBT path for this field (`root.field`).",
        context = "The dot-separated NBT path for this field (`root.field`). Alias for [`full_path`](Self::full_path). Useful when passing the path to a player-scoped command manually, since Minecraft storage is global and does not have automatic per-player keying.",
        minecraft = "Alias for [`full_path`](Self::full_path). Useful when passing the path to a player-scoped command manually, since Minecraft storage is global and does not have automatic per-player keying.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to use the dot-separated NBT path for this field (`root.field`).",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let field_path = storage_field_value.field_path();\n}",
    )]
    pub fn field_path(&self) -> String {
        self.full_path()
    }

    /// Returns the typed NBT location targeted by this reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::location",
        aliases = ["sand::prelude::StorageField::location", "sand::state::StorageField::location"],
        module = "sand::data",
        kind = "method",
        summary = "Returns the typed NBT location targeted by this reference.",
        context = "Returns the typed NBT location targeted by this reference. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "Returns the typed NBT location targeted by this reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let location = storage_field_value.location();\n}",
    )]
    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageField requires a valid storage resource location")
    }

    /// Builds the typed Minecraft data query for get.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::get",
        aliases = ["sand::prelude::StorageField::get", "sand::state::StorageField::get"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for get.",
        context = "Builds the typed Minecraft data query for get. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to build the typed Minecraft data query for get.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let get = storage_field_value.get();\n}",
    )]
    pub fn get(&self) -> String {
        self.path().get().to_string()
    }

    /// Builds the typed Minecraft data query for get scaled.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::get_scaled",
        aliases = ["sand::prelude::StorageField::get_scaled", "sand::state::StorageField::get_scaled"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for get scaled.",
        context = "Builds the typed Minecraft data query for get scaled. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(scale = "`scale` provides the scale used to build the typed Minecraft data query for get scaled."),
        returns = "The string value produced to build the typed Minecraft data query for get scaled.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, scale: f64)  {\n    let get_scaled = storage_field_value.get_scaled(scale);\n}",
    )]
    pub fn get_scaled(&self, scale: f64) -> String {
        self.path().get_scaled(scale).to_string()
    }

    /// Builds the typed Minecraft data modification for set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::set",
        aliases = ["sand::prelude::StorageField::set", "sand::state::StorageField::set"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set.",
        context = "Builds the typed Minecraft data modification for set. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set."),
        returns = "The string value produced to build the typed Minecraft data modification for set.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, value: impl Into < SnbtValue >)  {\n    let set = storage_field_value.set(value);\n}",
    )]
    pub fn set(&self, value: impl Into<SnbtValue>) -> String {
        self.set_value(value.into())
    }

    /// Builds the typed Minecraft data modification for set value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::set_value",
        aliases = ["sand::prelude::StorageField::set_value", "sand::state::StorageField::set_value"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for set value.",
        context = "Builds the typed Minecraft data modification for set value. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for set value."),
        returns = "The string value produced to build the typed Minecraft data modification for set value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, value: SnbtValue)  {\n    let set_value = storage_field_value.set_value(value);\n}",
    )]
    pub fn set_value(&self, value: SnbtValue) -> String {
        self.path().set(value).to_string()
    }

    /// Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::set_raw_snbt",
        aliases = ["sand::prelude::StorageField::set_raw_snbt", "sand::state::StorageField::set_raw_snbt"],
        module = "sand::data",
        kind = "method",
        summary = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        context = "Provides the explicit raw SNBT escape hatch after the caller accepts validation responsibility. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(raw = "`raw` is used to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility."),
        returns = "The string value produced to provide the explicit raw SNBT escape hatch after the caller accepts validation responsibility.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, raw: sand::component::RawSnbt)  {\n    let set_raw_snbt = storage_field_value.set_raw_snbt(raw);\n}",
    )]
    pub fn set_raw_snbt(&self, raw: RawSnbt) -> String {
        self.path().set_raw(raw.to_string()).to_string()
    }

    /// Builds the typed Minecraft data modification for remove.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::remove",
        aliases = ["sand::prelude::StorageField::remove", "sand::state::StorageField::remove"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for remove.",
        context = "Builds the typed Minecraft data modification for remove. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to build the typed Minecraft data modification for remove.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let remove = storage_field_value.remove();\n}",
    )]
    pub fn remove(&self) -> String {
        self.path().remove().to_string()
    }

    /// Builds the typed Minecraft data query for exists.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::exists",
        aliases = ["sand::prelude::StorageField::exists", "sand::state::StorageField::exists"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data query for exists.",
        context = "Builds the typed Minecraft data query for exists. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `Condition` value produced to build the typed Minecraft data query for exists.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >)  {\n    let exists = storage_field_value.exists();\n}",
    )]
    pub fn exists(&self) -> Condition {
        Condition::nbt_exists(
            DataTarget::storage(self.storage),
            NbtPath::new(self.full_path()),
        )
    }

    /// Builds the typed Minecraft data modification for copy from.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::copy_from",
        aliases = ["sand::prelude::StorageField::copy_from", "sand::state::StorageField::copy_from"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for copy from.",
        context = "Builds the typed Minecraft data modification for copy from. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source = "`source` provides the source used to build the typed Minecraft data modification for copy from."),
        returns = "The string value produced to build the typed Minecraft data modification for copy from.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static, OtherSchema: 'static, U: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, source: sand::data::StorageField < OtherSchema , U >)  {\n    let copy_from = storage_field_value.copy_from::<OtherSchema, U>(source);\n}",
    )]
    pub fn copy_from<OtherSchema, U>(&self, source: StorageField<OtherSchema, U>) -> String {
        self.path().copy_from(&source.path()).to_string()
    }

    /// `data modify storage <s> <path> set from entity <entity> <src_path>`
    ///
    /// Copy a value from entity NBT into this field. Takes a typed
    /// [`Target`] — never build this by stringifying a participant handle
    /// yourself; pass [`Target::self_()`] from inside an
    /// [`crate::participant::EntityParticipant::execute_at`] callback (or any
    /// other typed target) instead.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::copy_from_entity",
        aliases = ["sand::prelude::StorageField::copy_from_entity", "sand::state::StorageField::copy_from_entity"],
        module = "sand::data",
        kind = "method",
        summary = "`data modify storage <s> <path> set from entity <entity> <src_path>`",
        context = "`data modify storage <s> <path> set from entity <entity> <src_path>` Copy a value from entity NBT into this field. Takes a typed [`Target`] — never build this by stringifying a participant handle yourself; pass [`Target::self_()`] from inside an [`sand::participant::EntityParticipant::execute_at`] callback (or any other typed target) instead.",
        minecraft = "Copy a value from entity NBT into this field. Takes a typed [`Target`] — never build this by stringifying a participant handle yourself; pass [`Target::self_()`] from inside an [`sand::participant::EntityParticipant::execute_at`] callback (or any other typed target) instead.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(entity = "`entity` provides the entity participant or predicate used to emit the documented `data modify storage <s> <path> set from entity <entity> <src_path>` form.", src_path = "`src_path` supplies the documented `data modify storage <s> <path> set from entity <entity> <src_path>` form."),
        returns = "The string value produced to emit the documented `data modify storage <s> <path> set from entity <entity> <src_path>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, entity: sand::command::Target, src_path: impl Into < String >)  {\n    let copy_from_entity = storage_field_value.copy_from_entity(entity, src_path);\n}",
    )]
    pub fn copy_from_entity(
        &self,
        entity: impl TargetArgument,
        src_path: impl Into<String>,
    ) -> String {
        let source = Nbt::entity(entity).path(src_path.into());
        self.path().copy_from(&source).to_string()
    }

    /// Builds the typed Minecraft data modification for copy from path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::copy_from_path",
        aliases = ["sand::prelude::StorageField::copy_from_path", "sand::state::StorageField::copy_from_path"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for copy from path.",
        context = "Builds the typed Minecraft data modification for copy from path. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(source_storage = "`source_storage` provides the source storage used to build the typed Minecraft data modification for copy from path.", source_path = "`source_path` provides the source path used to build the typed Minecraft data modification for copy from path."),
        returns = "The string value produced to build the typed Minecraft data modification for copy from path.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, source_storage: sand::data::StorageLocation, source_path: sand::data::NbtPath)  {\n    let copy_from_path = storage_field_value.copy_from_path(source_storage, source_path);\n}",
    )]
    pub fn copy_from_path(&self, source_storage: StorageLocation, source_path: NbtPath) -> String {
        let source = Nbt::storage(source_storage.to_string()).path(source_path);
        self.path().copy_from(&source).to_string()
    }

    /// Builds the typed Minecraft data modification for append.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::append",
        aliases = ["sand::prelude::StorageField::append", "sand::state::StorageField::append"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for append.",
        context = "Builds the typed Minecraft data modification for append. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for append."),
        returns = "The string value produced to build the typed Minecraft data modification for append.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, value: impl Into < SnbtValue >)  {\n    let append = storage_field_value.append(value);\n}",
    )]
    pub fn append(&self, value: impl Into<SnbtValue>) -> String {
        self.path().append(value).to_string()
    }

    /// Builds the typed Minecraft data modification for merge.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageField::merge",
        aliases = ["sand::prelude::StorageField::merge", "sand::state::StorageField::merge"],
        module = "sand::data",
        kind = "method",
        summary = "Builds the typed Minecraft data modification for merge.",
        context = "Builds the typed Minecraft data modification for merge. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to build the typed Minecraft data modification for merge."),
        returns = "The string value produced to build the typed Minecraft data modification for merge.",
        example = "use sand::prelude::*;\n\nfn demonstrate<Schema: 'static, T: 'static>(storage_field_value: &sand::data::StorageField < Schema , T >, value: impl Into < SnbtValue >)  {\n    let merge = storage_field_value.merge(value);\n}",
    )]
    pub fn merge(&self, value: impl Into<SnbtValue>) -> String {
        self.path().merge(value).to_string()
    }
}

// ── StorageVar ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::data::StorageVar",
    aliases = ["sand::prelude::StorageVar", "sand::state::StorageVar"],
    module = "sand::data",
    summary = "A typed NBT storage variable. Declare once as a `static` and use throughout your datapack. The type parameter `T` is purely documentary — NBT does not carry Rust types at runtime. Use `set_int`, `set_float`, `set_string`, etc. to pick the correct SNBT literal.",
    context = "A typed NBT storage variable. Declare once as a `static` and use throughout your datapack. The type parameter `T` is purely documentary — NBT does not carry Rust types at runtime. Use `set_int`, `set_float`, `set_string`, etc. to pick the correct SNBT literal. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
    minecraft = "Declare once as a `static` and use throughout your datapack. The type parameter `T` is purely documentary — NBT does not carry Rust types at runtime. Use `set_int`, `set_float`, `set_string`, etc. to pick the correct SNBT literal.",
    use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
    avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
    example = "use sand::data::StorageVar;",
)]
/// A typed NBT storage variable.
///
/// Declare once as a `static` and use throughout your datapack. The type
/// parameter `T` is purely documentary — NBT does not carry Rust types at
/// runtime. Use `set_int`, `set_float`, `set_string`, etc. to pick the
/// correct SNBT literal.
///
/// # Example
/// ```rust,ignore
/// use sand_core::state::StorageVar;
///
/// static MANA: StorageVar<i32> = StorageVar::new("sand:data", "player.mana");
/// static NAME: StorageVar<String> = StorageVar::new("sand:data", "player.name");
///
/// fn load() -> Vec<String> {
///     vec![
///         MANA.set_int(100),
///         NAME.set_string("Steve"),
///     ]
/// }
/// ```
pub struct StorageVar<T = serde_json::Value> {
    storage: &'static str,
    path: &'static str,
    _marker: PhantomData<T>,
}

impl<T> StorageVar<T> {
    /// Create a new `StorageVar` pointing at `<storage> <path>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::new",
        aliases = ["sand::prelude::StorageVar::new", "sand::state::StorageVar::new"],
        module = "sand::data",
        kind = "method",
        summary = "Create a new `StorageVar` pointing at `<storage> <path>`.",
        context = "Create a new `StorageVar` pointing at `<storage> <path>`. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(storage = "`storage` is used when creating a new `StorageVar` pointing at `<storage> <path>`.", path = "`path` provides the typed resource identifier or location used to create a new `StorageVar` pointing at `<storage> <path>`."),
        returns = "A `StorageVar` representing a new `StorageVar` pointing at `<storage> <path>`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage: & 'static str, path: & 'static str)  {\n    let storage_var = sand::data::StorageVar ::< T >::new(storage, path);\n}",
    )]
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// The storage namespace string (e.g. `"sand:data"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::storage",
        aliases = ["sand::prelude::StorageVar::storage", "sand::state::StorageVar::storage"],
        module = "sand::data",
        kind = "method",
        summary = "The storage namespace string (e.g. `\"sand:data\"`).",
        context = "The storage namespace string (e.g. `\"sand:data\"`). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to use the storage namespace string (e.g. `\"sand:data\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let storage = storage_var_value.storage();\n}",
    )]
    pub fn storage(&self) -> &'static str {
        self.storage
    }

    /// The path string (e.g. `"player.mana"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::path",
        aliases = ["sand::prelude::StorageVar::path", "sand::state::StorageVar::path"],
        module = "sand::data",
        kind = "method",
        summary = "The path string (e.g. `\"player.mana\"`).",
        context = "The path string (e.g. `\"player.mana\"`). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to use the path string (e.g. `\"player.mana\"`).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let path = storage_var_value.path();\n}",
    )]
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Build an [`NbtPath`] for this variable.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::as_path",
        aliases = ["sand::prelude::StorageVar::as_path", "sand::state::StorageVar::as_path"],
        module = "sand::data",
        kind = "method",
        summary = "Build an [`NbtPath`] for this variable.",
        context = "Build an [`NbtPath`] for this variable. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `NbtRef < T >` value produced to build an [`NbtPath`] for this variable.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let as_path = storage_var_value.as_path();\n}",
    )]
    pub fn as_path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage).typed_path(self.path)
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// `data get storage <storage> <path>` — read the value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::get",
        aliases = ["sand::prelude::StorageVar::get", "sand::state::StorageVar::get"],
        module = "sand::data",
        kind = "method",
        summary = "`data get storage <storage> <path>` — read the value.",
        context = "`data get storage <storage> <path>` — read the value. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to emit the documented `data get storage <storage> <path>` — read the value form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let get = storage_var_value.get();\n}",
    )]
    pub fn get(&self) -> String {
        self.as_path().get().to_string()
    }

    /// `data get storage <storage> <path> <scale>` — read a numeric value with scale.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::get_scaled",
        aliases = ["sand::prelude::StorageVar::get_scaled", "sand::state::StorageVar::get_scaled"],
        module = "sand::data",
        kind = "method",
        summary = "`data get storage <storage> <path> <scale>` — read a numeric value with scale.",
        context = "`data get storage <storage> <path> <scale>` — read a numeric value with scale. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(scale = "`scale` supplies the documented `data get storage <storage> <path> <scale>` — read a numeric value with scale form."),
        returns = "The string value produced to emit the documented `data get storage <storage> <path> <scale>` — read a numeric value with scale form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, scale: f64)  {\n    let get_scaled = storage_var_value.get_scaled(scale);\n}",
    )]
    pub fn get_scaled(&self, scale: f64) -> String {
        self.as_path().get_scaled(scale).to_string()
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// `data modify storage <storage> <path> set value <snbt>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_value",
        aliases = ["sand::prelude::StorageVar::set_value", "sand::state::StorageVar::set_value"],
        module = "sand::data",
        kind = "method",
        summary = "`data modify storage <storage> <path> set value <snbt>`.",
        context = "`data modify storage <storage> <path> set value <snbt>`. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(value = "`value` provides the value being applied or compared used to emit the documented `data modify storage <storage> <path> set value <snbt>` form."),
        returns = "The string value produced to emit the documented `data modify storage <storage> <path> set value <snbt>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, value: impl Into < SnbtValue >)  {\n    let set_value = storage_var_value.set_value(value);\n}",
    )]
    pub fn set_value(&self, value: impl Into<SnbtValue>) -> String {
        self.as_path().set(value).to_string()
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_raw_snbt",
        aliases = ["sand::prelude::StorageVar::set_raw_snbt", "sand::state::StorageVar::set_raw_snbt"],
        module = "sand::data",
        kind = "method",
        summary = "`data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch.",
        context = "`data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(snbt = "`snbt` supplies the documented `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch form."),
        returns = "The string value produced to emit the documented `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, snbt: sand::component::RawSnbt)  {\n    let set_raw_snbt = storage_var_value.set_raw_snbt(snbt);\n}",
    )]
    pub fn set_raw_snbt(&self, snbt: RawSnbt) -> String {
        self.as_path().set_raw(snbt.to_string()).to_string()
    }

    /// Set an integer value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_int",
        aliases = ["sand::prelude::StorageVar::set_int", "sand::state::StorageVar::set_int"],
        module = "sand::data",
        kind = "method",
        summary = "Set an integer value.",
        context = "Set an integer value. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the v applied when setting an integer value."),
        returns = "The string value produced to set an integer value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: i32)  {\n    let set_int = storage_var_value.set_int(v);\n}",
    )]
    pub fn set_int(&self, v: i32) -> String {
        self.set_value(v)
    }

    /// Set a long value (`<v>L` SNBT).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_long",
        aliases = ["sand::prelude::StorageVar::set_long", "sand::state::StorageVar::set_long"],
        module = "sand::data",
        kind = "method",
        summary = "Set a long value (`<v>L` SNBT).",
        context = "Set a long value (`<v>L` SNBT). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the v applied when setting a long value (`<v>L` SNBT)."),
        returns = "The string value produced to set a long value (`<v>L` SNBT).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: i64)  {\n    let set_long = storage_var_value.set_long(v);\n}",
    )]
    pub fn set_long(&self, v: i64) -> String {
        self.set_value(v)
    }

    /// Set a float value (`<v>f` SNBT).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_float",
        aliases = ["sand::prelude::StorageVar::set_float", "sand::state::StorageVar::set_float"],
        module = "sand::data",
        kind = "method",
        summary = "Set a float value (`<v>f` SNBT).",
        context = "Set a float value (`<v>f` SNBT). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the v applied when setting a float value (`<v>f` SNBT)."),
        returns = "The string value produced to set a float value (`<v>f` SNBT).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: f32)  {\n    let set_float = storage_var_value.set_float(v);\n}",
    )]
    pub fn set_float(&self, v: f32) -> String {
        self.set_value(v)
    }

    /// Set a double value (`<v>d` SNBT).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_double",
        aliases = ["sand::prelude::StorageVar::set_double", "sand::state::StorageVar::set_double"],
        module = "sand::data",
        kind = "method",
        summary = "Set a double value (`<v>d` SNBT).",
        context = "Set a double value (`<v>d` SNBT). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the v applied when setting a double value (`<v>d` SNBT)."),
        returns = "The string value produced to set a double value (`<v>d` SNBT).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: f64)  {\n    let set_double = storage_var_value.set_double(v);\n}",
    )]
    pub fn set_double(&self, v: f64) -> String {
        self.set_value(v)
    }

    /// Set a string value (auto-quoted, backslash-escaping inner quotes).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_string",
        aliases = ["sand::prelude::StorageVar::set_string", "sand::state::StorageVar::set_string"],
        module = "sand::data",
        kind = "method",
        summary = "Set a string value (auto-quoted, backslash-escaping inner quotes).",
        context = "Set a string value (auto-quoted, backslash-escaping inner quotes). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the v applied when setting a string value (auto-quoted, backslash-escaping inner quotes)."),
        returns = "The string value produced to set a string value (auto-quoted, backslash-escaping inner quotes).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: & str)  {\n    let set_string = storage_var_value.set_string(v);\n}",
    )]
    pub fn set_string(&self, v: &str) -> String {
        self.set_value(v)
    }

    /// Set a boolean as a byte (0b or 1b SNBT).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::set_bool",
        aliases = ["sand::prelude::StorageVar::set_bool", "sand::state::StorageVar::set_bool"],
        module = "sand::data",
        kind = "method",
        summary = "Set a boolean as a byte (0b or 1b SNBT).",
        context = "Set a boolean as a byte (0b or 1b SNBT). This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set a boolean as a byte (0b or 1b SNBT)."),
        returns = "The string value produced to set a boolean as a byte (0b or 1b SNBT).",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, v: bool)  {\n    let set_bool = storage_var_value.set_bool(v);\n}",
    )]
    pub fn set_bool(&self, v: bool) -> String {
        self.set_value(v)
    }

    /// `data modify storage <storage> <path> set from storage <src> <src_path>` — copy.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::copy_from",
        aliases = ["sand::prelude::StorageVar::copy_from", "sand::state::StorageVar::copy_from"],
        module = "sand::data",
        kind = "method",
        summary = "`data modify storage <storage> <path> set from storage <src> <src_path>` — copy.",
        context = "`data modify storage <storage> <path> set from storage <src> <src_path>` — copy. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        params(src_storage = "`src_storage` supplies the documented `data modify storage <storage> <path> set from storage <src> <src_path>` — copy form.", src_path = "`src_path` supplies the documented `data modify storage <storage> <path> set from storage <src> <src_path>` — copy form."),
        returns = "The string value produced to emit the documented `data modify storage <storage> <path> set from storage <src> <src_path>` — copy form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >, src_storage: & str, src_path: & str)  {\n    let copy_from = storage_var_value.copy_from(src_storage, src_path);\n}",
    )]
    pub fn copy_from(&self, src_storage: &str, src_path: &str) -> String {
        let source = Nbt::storage(src_storage).path(src_path);
        self.as_path().copy_from(&source).to_string()
    }

    // ── Delete / exists ───────────────────────────────────────────────────────

    /// `data remove storage <storage> <path>` — remove the tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::remove",
        aliases = ["sand::prelude::StorageVar::remove", "sand::state::StorageVar::remove"],
        module = "sand::data",
        kind = "method",
        summary = "`data remove storage <storage> <path>` — remove the tag.",
        context = "`data remove storage <storage> <path>` — remove the tag. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The string value produced to emit the documented `data remove storage <storage> <path>` — remove the tag form.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let remove = storage_var_value.remove();\n}",
    )]
    pub fn remove(&self) -> String {
        self.as_path().remove().to_string()
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::data::StorageVar::exists",
        aliases = ["sand::prelude::StorageVar::exists", "sand::state::StorageVar::exists"],
        module = "sand::data",
        kind = "method",
        summary = "Build a `Condition` that checks `if data storage <storage> <path>`.",
        context = "Build a `Condition` that checks `if data storage <storage> <path>`. This API models a typed NBT value, path, target, or data command. Raw SNBT entry points are explicit escape hatches rather than the normal representation.",
        minecraft = "Operations render vanilla data commands against entity, block, or namespaced command-storage targets and validate writable target cardinality.",
        use_when = ["Reading or mutating structured Minecraft NBT through typed paths and values"],
        avoid_when = ["A scoreboard-backed state field is simpler, or the input is untrusted raw SNBT"],
        returns = "The `Condition` value produced to build a `Condition` that checks `if data storage <storage> <path>`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: 'static>(storage_var_value: &sand::data::StorageVar < T >)  {\n    let exists = storage_var_value.exists();\n}",
    )]
    pub fn exists(&self) -> Condition {
        Condition::nbt_exists(DataTarget::storage(self.storage), NbtPath::new(self.path))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{Condition, ConditionKind};

    static MANA: StorageVar<i32> = StorageVar::new("sand:data", "player.mana");
    static NAME: StorageVar<String> = StorageVar::new("sand:data", "player.name");
    #[derive(Debug)]
    struct PlayerMagic;
    static MAGIC: StorageSchema<PlayerMagic> = StorageSchema::new("arcane:players", "player.magic");
    static MAGIC_MANA: StorageField<PlayerMagic, i32> = MAGIC.field("mana");
    static MAGIC_SCHOOL: StorageField<PlayerMagic, String> = MAGIC.field("school");
    static SPELLS: StorageField<PlayerMagic, Vec<String>> = MAGIC.field("unlocked_spells");
    static STATS: StorageSchema<PlayerMagic> = StorageSchema::new("arcane:players", "player.stats");
    static MANA_FIELD: StorageField<PlayerMagic, i32> = STATS.field("mana");

    #[test]
    fn get_command() {
        assert_eq!(MANA.get(), "data get storage sand:data player.mana");
    }

    #[test]
    fn get_scaled() {
        assert_eq!(
            MANA.get_scaled(1.0),
            "data get storage sand:data player.mana 1"
        );
    }

    #[test]
    fn set_int() {
        assert_eq!(
            MANA.set_int(100),
            "data modify storage sand:data player.mana set value 100"
        );
    }

    #[test]
    fn set_string_escaping() {
        assert_eq!(
            NAME.set_string("Steve"),
            r#"data modify storage sand:data player.name set value "Steve""#
        );
        assert_eq!(
            NAME.set_string(r#"say "hi""#),
            r#"data modify storage sand:data player.name set value "say \"hi\"""#
        );
    }

    #[test]
    fn snbt_primitive_formatting() {
        assert_eq!(SnbtValue::Byte(1).to_string(), "1b");
        assert_eq!(SnbtValue::Short(2).to_string(), "2s");
        assert_eq!(SnbtValue::Int(3).to_string(), "3");
        assert_eq!(SnbtValue::Long(4).to_string(), "4L");
        assert_eq!(SnbtValue::Float(1.5).to_string(), "1.5f");
        assert_eq!(SnbtValue::Double(2.5).to_string(), "2.5d");
        assert_eq!(SnbtValue::Bool(true).to_string(), "1b");
        assert_eq!(SnbtValue::Bool(false).to_string(), "0b");
    }

    #[test]
    fn snbt_string_escaping() {
        assert_eq!(
            SnbtValue::from(r#"say "hi" \ now"#).to_string(),
            r#""say \"hi\" \\ now""#
        );
    }

    #[test]
    fn snbt_list_and_compound_formatting() {
        let value = SnbtCompound::new()
            .field("mana", 100)
            .field("school", "pyromancy")
            .field("arcane:rank", 2_i8)
            .field("spells", SnbtValue::from(vec!["dash", "shield"]));

        assert_eq!(
            value.to_string(),
            r#"{mana:100,school:"pyromancy","arcane:rank":2b,spells:["dash","shield"]}"#
        );
    }

    #[test]
    fn set_bool() {
        assert_eq!(
            MANA.set_bool(true),
            "data modify storage sand:data player.mana set value 1b"
        );
        assert_eq!(
            MANA.set_bool(false),
            "data modify storage sand:data player.mana set value 0b"
        );
    }

    #[test]
    fn set_float() {
        assert_eq!(
            MANA.set_float(1.5),
            "data modify storage sand:data player.mana set value 1.5f"
        );
    }

    #[test]
    fn set_long() {
        assert_eq!(
            MANA.set_long(9999),
            "data modify storage sand:data player.mana set value 9999L"
        );
    }

    #[test]
    fn remove_command() {
        assert_eq!(MANA.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn copy_from() {
        assert_eq!(
            MANA.copy_from("other:ns", "foo.bar"),
            "data modify storage sand:data player.mana set from storage other:ns foo.bar"
        );
    }

    #[test]
    fn exists_condition() {
        let cond = MANA.exists();
        match cond.kind() {
            ConditionKind::NbtExists { target, path } => {
                assert_eq!(target.to_string(), "storage sand:data");
                assert_eq!(path.as_str(), "player.mana");
            }
            other => panic!("unexpected: {other:?}"),
        }
        let cmds = cond.execute_commands(false, "run say exists");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if data storage sand:data player.mana"));
    }

    #[test]
    fn nbt_path_navigate() {
        let base = Nbt::storage("sand:data").path("player");
        let mana = base.key("mana");
        assert_eq!(mana.as_str(), "player.mana");
        assert_eq!(mana.storage(), "sand:data");

        let first = mana.index(0);
        assert_eq!(first.as_str(), "player.mana[0]");
    }

    #[test]
    fn nbt_path_root_field_and_quoted_key() {
        let path = NbtPath::root("player")
            .field("magic")
            .index(0)
            .field("arcane:mana");
        assert_eq!(path.as_str(), r#"player.magic[0]."arcane:mana""#);
    }

    #[test]
    fn nbt_path_get_remove() {
        let p = Nbt::storage("sand:data").path("player.mana");
        assert_eq!(p.get(), "data get storage sand:data player.mana");
        assert_eq!(p.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn nbt_path_set_bool() {
        let p = Nbt::storage("sand:data").path("player").key("mana");
        assert_eq!(
            p.set_bool(true),
            "data modify storage sand:data player.mana set value 1b"
        );
    }

    #[test]
    fn nbt_path_raw_snbt_escape_hatch() {
        let p = Nbt::storage("sand:data").path("player.payload");
        assert_eq!(
            p.set_raw(RawSnbt::new("{custom:1b}").to_string()),
            "data modify storage sand:data player.payload set value {custom:1b}"
        );
    }

    #[test]
    fn nbt_path_exists() {
        let p = Nbt::storage("sand:data").path("player.mana");
        let cond = Condition::data_exists(&p);
        assert!(matches!(cond.kind(), ConditionKind::NbtExists { .. }));
    }

    #[test]
    fn golden_mana_system() {
        let init = MANA.set_int(100);
        let check = MANA.exists();
        let drain = MANA.set_int(95);
        let cmds = check.execute_commands(false, &format!("run {drain}"));
        assert_eq!(
            init,
            "data modify storage sand:data player.mana set value 100"
        );
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if data storage sand:data player.mana"));
        assert!(cmds[0].contains("run data modify storage sand:data player.mana set value 95"));
    }

    #[test]
    fn storage_schema_root_paths() {
        assert_eq!(MAGIC.storage(), "arcane:players");
        assert_eq!(MAGIC.root_path(), "player.magic");
        assert_eq!(MAGIC.path().as_str(), "player.magic");
        assert_eq!(MAGIC_MANA.full_path(), "player.magic.mana");
    }

    #[test]
    fn storage_schema_root_commands() {
        assert_eq!(MAGIC.get(), "data get storage arcane:players player.magic");
        assert_eq!(
            MAGIC.set(SnbtCompound::new().field("mana", 100)),
            "data modify storage arcane:players player.magic set value {mana:100}"
        );
        assert_eq!(
            MAGIC.merge(SnbtCompound::new().field("school", "pyromancy")),
            r#"data modify storage arcane:players player.magic merge value {school:"pyromancy"}"#
        );
        assert_eq!(
            MAGIC.remove(),
            "data remove storage arcane:players player.magic"
        );
        assert!(matches!(
            MAGIC.exists().kind(),
            ConditionKind::NbtExists { .. }
        ));
    }

    #[test]
    fn typed_field_set_get_remove_exists() {
        assert_eq!(
            MAGIC_MANA.set(100),
            "data modify storage arcane:players player.magic.mana set value 100"
        );
        assert_eq!(
            MAGIC_SCHOOL.set("pyromancy"),
            r#"data modify storage arcane:players player.magic.school set value "pyromancy""#
        );
        assert_eq!(
            MAGIC_MANA.get(),
            "data get storage arcane:players player.magic.mana"
        );
        assert_eq!(
            MAGIC_MANA.get_scaled(0.5),
            "data get storage arcane:players player.magic.mana 0.5"
        );
        assert_eq!(
            MAGIC_MANA.remove(),
            "data remove storage arcane:players player.magic.mana"
        );
        assert!(matches!(
            MAGIC_MANA.exists().kind(),
            ConditionKind::NbtExists { .. }
        ));
    }

    #[test]
    fn typed_field_copy_append_merge_and_raw() {
        assert_eq!(
            MAGIC_MANA.copy_from(MANA_FIELD),
            "data modify storage arcane:players player.magic.mana set from storage arcane:players player.stats.mana"
        );
        assert_eq!(
            SPELLS.append("dash"),
            r#"data modify storage arcane:players player.magic.unlocked_spells append value "dash""#
        );
        assert_eq!(
            MAGIC_SCHOOL.set_raw_snbt(RawSnbt::new("\"raw_school\"")),
            r#"data modify storage arcane:players player.magic.school set value "raw_school""#
        );
        assert_eq!(
            MAGIC_MANA.merge(SnbtCompound::new().field("bonus", 3)),
            "data modify storage arcane:players player.magic.mana merge value {bonus:3}"
        );
    }

    // ── Issue #99 regression: StorageField::path() must be storage-bound ──────

    #[test]
    fn storage_field_path_retains_storage() {
        let p = MAGIC_MANA.path();
        assert_eq!(
            p.storage(),
            "arcane:players",
            "path() must carry the storage target"
        );
        assert_eq!(p.as_str(), "player.magic.mana");
    }

    #[test]
    fn storage_field_path_commands_are_valid() {
        let p = MAGIC_MANA.path();
        assert_eq!(p.get(), "data get storage arcane:players player.magic.mana");
        assert_eq!(
            p.remove(),
            "data remove storage arcane:players player.magic.mana"
        );
        assert_eq!(
            p.set_value(42_i32),
            "data modify storage arcane:players player.magic.mana set value 42"
        );
        // storage target must not be empty
        assert!(
            !p.get().contains("storage  "),
            "command must not have empty storage target"
        );
    }

    #[test]
    fn storage_field_full_path_unchanged() {
        // full_path() still returns only the dot-separated NBT path (no storage prefix)
        assert_eq!(MAGIC_MANA.full_path(), "player.magic.mana");
        assert_eq!(MAGIC_SCHOOL.full_path(), "player.magic.school");
    }

    // ── Issue #98 regression: control characters must not appear literally ─────

    #[test]
    fn snbt_string_normal_values_unchanged() {
        assert_eq!(
            SnbtValue::from("hello world").to_string(),
            r#""hello world""#
        );
        assert_eq!(SnbtValue::from("123").to_string(), r#""123""#);
    }

    #[test]
    fn snbt_string_quotes_and_backslash() {
        assert_eq!(
            SnbtValue::from(r#"say "hi" \ now"#).to_string(),
            r#""say \"hi\" \\ now""#
        );
    }

    #[test]
    fn typed_snbt_controls_report_structured_errors() {
        for value in ["line1\nline2", "col1\tcol2", "a\rb", "nul\0byte"] {
            let command = Nbt::storage("sand:data").path("value").set(value);
            assert_eq!(
                command
                    .try_render(&sand_commands::CommandProfile::unprofiled())
                    .unwrap_err()
                    .code,
                "SAND-DATA-TARGET"
            );
        }
        let compound = Nbt::storage("sand:data")
            .path("value")
            .set(SnbtCompound::new().field("key\nwith\nnewline", 1_i32));
        assert!(
            compound
                .try_render(&sand_commands::CommandProfile::unprofiled())
                .is_err()
        );
        let string = Nbt::storage("sand:data")
            .path("player.name")
            .set_string("line1\nline2");
        assert!(
            string
                .try_render(&sand_commands::CommandProfile::unprofiled())
                .is_err()
        );
    }
}
