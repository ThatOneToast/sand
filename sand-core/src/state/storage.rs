//! Typed NBT storage variables backed by `data storage` commands.

use std::fmt;
use std::marker::PhantomData;

use crate::condition::Condition;
use sand_commands::{BlockPos, Selector};
use sand_components::{RawSnbt, ResourceLocation};

// ── SnbtValue ────────────────────────────────────────────────────────────────

/// A typed SNBT value for storage and data commands.
#[derive(Debug, Clone, PartialEq)]
pub enum SnbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Bool(bool),
    List(Vec<SnbtValue>),
    Compound(SnbtCompound),
    Raw(RawSnbt),
}

impl SnbtValue {
    pub fn list(values: impl IntoIterator<Item = SnbtValue>) -> Self {
        Self::List(values.into_iter().collect())
    }

    pub fn compound(compound: SnbtCompound) -> Self {
        Self::Compound(compound)
    }

    pub fn raw(raw: RawSnbt) -> Self {
        Self::Raw(raw)
    }
}

impl fmt::Display for SnbtValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnbtValue::Byte(v) => write!(f, "{v}b"),
            SnbtValue::Short(v) => write!(f, "{v}s"),
            SnbtValue::Int(v) => write!(f, "{v}"),
            SnbtValue::Long(v) => write!(f, "{v}L"),
            SnbtValue::Float(v) => write!(f, "{v}f"),
            SnbtValue::Double(v) => write!(f, "{v}d"),
            SnbtValue::String(v) => write!(f, "\"{}\"", escape_snbt_string(v)),
            SnbtValue::Bool(v) => f.write_str(if *v { "1b" } else { "0b" }),
            SnbtValue::List(values) => {
                let values = values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                write!(f, "[{values}]")
            }
            SnbtValue::Compound(compound) => compound.fmt(f),
            SnbtValue::Raw(raw) => raw.fmt(f),
        }
    }
}

impl From<i8> for SnbtValue {
    fn from(value: i8) -> Self {
        Self::Byte(value)
    }
}

impl From<i16> for SnbtValue {
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<i32> for SnbtValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for SnbtValue {
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<f32> for SnbtValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for SnbtValue {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<bool> for SnbtValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for SnbtValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for SnbtValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<SnbtCompound> for SnbtValue {
    fn from(value: SnbtCompound) -> Self {
        Self::Compound(value)
    }
}

impl From<RawSnbt> for SnbtValue {
    fn from(value: RawSnbt) -> Self {
        Self::Raw(value)
    }
}

impl<T> From<Vec<T>> for SnbtValue
where
    T: Into<SnbtValue>,
{
    fn from(value: Vec<T>) -> Self {
        Self::List(value.into_iter().map(Into::into).collect())
    }
}

/// A typed SNBT compound preserving insertion order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SnbtCompound {
    entries: Vec<(String, SnbtValue)>,
}

impl SnbtCompound {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<SnbtValue>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<SnbtValue>) {
        self.entries.push((key.into(), value.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl fmt::Display for SnbtCompound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self
            .entries
            .iter()
            .map(|(key, value)| format!("{}:{value}", snbt_compound_key(key)))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{{{entries}}}")
    }
}

// ── NbtPath ───────────────────────────────────────────────────────────────────

/// A typed NBT path string for navigating storage/entity/block NBT.
///
/// # Example
/// ```rust,ignore
/// use sand_core::state::NbtPath;
///
/// let p = NbtPath::new("sand:data", "player.mana");
/// assert_eq!(p.as_str(), "player.mana");
/// assert_eq!(p.storage(), "sand:data");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NbtPath {
    storage: String,
    path: String,
}

impl NbtPath {
    /// Create a path for a storage location.
    pub fn new(storage: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            storage: storage.into(),
            path: path.into(),
        }
    }

    /// Create a root path without binding it to a storage location.
    pub fn root(path: impl Into<String>) -> Self {
        Self {
            storage: String::new(),
            path: path.into(),
        }
    }

    /// The storage namespace (e.g. `"sand:data"`).
    pub fn storage(&self) -> &str {
        &self.storage
    }

    /// The path within the storage (e.g. `"player.mana"`).
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Append a field/key segment, returning a new path.
    pub fn field(&self, key: impl AsRef<str>) -> Self {
        Self {
            storage: self.storage.clone(),
            path: append_path_field(&self.path, key.as_ref()),
        }
    }

    /// Append a sub-key, returning a new path.
    pub fn key(&self, key: impl AsRef<str>) -> Self {
        self.field(key)
    }

    /// Index into an array, returning a new path.
    pub fn index(&self, i: usize) -> Self {
        Self {
            storage: self.storage.clone(),
            path: format!("{}[{i}]", self.path),
        }
    }

    /// `data get storage <storage> <path>` — read the NBT value.
    pub fn get(&self) -> String {
        format!("data get storage {} {}", self.storage, self.path)
    }

    /// `data remove storage <storage> <path>` — remove the NBT tag.
    pub fn remove(&self) -> String {
        format!("data remove storage {} {}", self.storage, self.path)
    }

    /// `data modify storage <storage> <path> set value <snbt>`.
    pub fn set_value(&self, value: impl Into<SnbtValue>) -> String {
        format!(
            "data modify storage {} {} set value {}",
            self.storage,
            self.path,
            value.into()
        )
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch.
    pub fn set_raw_snbt(&self, snbt: RawSnbt) -> String {
        self.set_value(SnbtValue::Raw(snbt))
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT.
    #[deprecated(since = "0.1.0", note = "use set_raw_snbt(RawSnbt::new(...))")]
    pub fn set_raw(&self, snbt: &str) -> String {
        self.set_raw_snbt(RawSnbt::new(snbt))
    }

    /// Set an integer value at this path.
    pub fn set_int(&self, v: i32) -> String {
        self.set_value(v)
    }

    /// Set a boolean as a byte (`0b` or `1b`) at this path.
    pub fn set_bool(&self, v: bool) -> String {
        self.set_value(v)
    }

    /// Set a string value at this path.
    pub fn set_string(&self, v: &str) -> String {
        self.set_value(v)
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    pub fn exists(&self) -> Condition {
        Condition::StorageExists {
            location: self.storage.clone(),
            path: self.path.clone(),
        }
    }
}

// ── Storage locations ────────────────────────────────────────────────────────

/// A typed `data storage <id>` target.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageLocation {
    id: ResourceLocation,
}

impl StorageLocation {
    pub fn new(id: ResourceLocation) -> Self {
        Self { id }
    }

    pub fn parse(id: impl AsRef<str>) -> sand_components::Result<Self> {
        Ok(Self::new(id.as_ref().parse()?))
    }

    pub fn as_resource_location(&self) -> &ResourceLocation {
        &self.id
    }
}

impl fmt::Display for StorageLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}

/// A typed entity NBT target.
#[derive(Debug, Clone)]
pub struct EntityNbt {
    target: Selector,
}

impl EntityNbt {
    pub fn target(target: Selector) -> Self {
        Self { target }
    }
}

/// A typed block entity NBT target.
#[derive(Debug, Clone)]
pub struct BlockNbt {
    pos: BlockPos,
}

impl BlockNbt {
    pub fn pos(pos: BlockPos) -> Self {
        Self { pos }
    }
}

/// A typed target for Minecraft `data` commands.
#[derive(Debug, Clone)]
pub enum NbtLocation {
    Storage(StorageLocation),
    Entity(EntityNbt),
    Block(BlockNbt),
}

impl NbtLocation {
    pub fn storage(storage: StorageLocation) -> Self {
        Self::Storage(storage)
    }

    pub fn entity(target: Selector) -> Self {
        Self::Entity(EntityNbt::target(target))
    }

    pub fn block(pos: BlockPos) -> Self {
        Self::Block(BlockNbt::pos(pos))
    }
}

impl fmt::Display for NbtLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtLocation::Storage(storage) => write!(f, "storage {storage}"),
            NbtLocation::Entity(entity) => write!(f, "entity {}", entity.target),
            NbtLocation::Block(block) => write!(f, "block {}", block.pos),
        }
    }
}

// ── StorageSchema / StorageField ─────────────────────────────────────────────

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
    pub const fn new(storage: &'static str, root: &'static str) -> Self {
        Self {
            storage,
            root,
            _marker: PhantomData,
        }
    }

    pub const fn storage(&self) -> &'static str {
        self.storage
    }

    pub const fn root_path(&self) -> &'static str {
        self.root
    }

    pub const fn field<U>(&self, field: &'static str) -> StorageField<T, U> {
        StorageField {
            storage: self.storage,
            root: self.root,
            field,
            _schema: PhantomData,
            _value: PhantomData,
        }
    }

    pub fn path(&self) -> NbtPath {
        NbtPath::new(self.storage, self.root)
    }

    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageSchema::new requires a valid storage resource location")
    }
}

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
    pub const fn new(schema: &StorageSchema<Schema>, field: &'static str) -> Self {
        schema.field(field)
    }

    pub const fn storage(&self) -> &'static str {
        self.storage
    }

    pub const fn root_path(&self) -> &'static str {
        self.root
    }

    pub const fn field_name(&self) -> &'static str {
        self.field
    }

    pub fn path(&self) -> NbtPath {
        NbtPath::root(self.root).field(self.field)
    }

    pub fn full_path(&self) -> String {
        self.path().as_str().to_string()
    }

    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageField requires a valid storage resource location")
    }

    pub fn get(&self) -> String {
        format!("data get storage {} {}", self.storage, self.full_path())
    }

    pub fn get_scaled(&self, scale: f64) -> String {
        format!(
            "data get storage {} {} {scale}",
            self.storage,
            self.full_path()
        )
    }

    pub fn set(&self, value: impl Into<SnbtValue>) -> String {
        self.set_value(value.into())
    }

    pub fn set_value(&self, value: SnbtValue) -> String {
        format!(
            "data modify storage {} {} set value {}",
            self.storage,
            self.full_path(),
            value
        )
    }

    pub fn set_raw_snbt(&self, raw: RawSnbt) -> String {
        self.set_value(SnbtValue::Raw(raw))
    }

    pub fn remove(&self) -> String {
        format!("data remove storage {} {}", self.storage, self.full_path())
    }

    pub fn exists(&self) -> Condition {
        Condition::StorageExists {
            location: self.storage.to_string(),
            path: self.full_path(),
        }
    }

    pub fn copy_from<OtherSchema, U>(&self, source: StorageField<OtherSchema, U>) -> String {
        format!(
            "data modify storage {} {} set from storage {} {}",
            self.storage,
            self.full_path(),
            source.storage,
            source.full_path()
        )
    }

    pub fn copy_from_path(&self, source_storage: StorageLocation, source_path: NbtPath) -> String {
        format!(
            "data modify storage {} {} set from storage {} {}",
            self.storage,
            self.full_path(),
            source_storage,
            source_path.as_str()
        )
    }

    pub fn append(&self, value: impl Into<SnbtValue>) -> String {
        format!(
            "data modify storage {} {} append value {}",
            self.storage,
            self.full_path(),
            value.into()
        )
    }

    pub fn merge(&self, value: impl Into<SnbtValue>) -> String {
        format!(
            "data modify storage {} {} merge value {}",
            self.storage,
            self.full_path(),
            value.into()
        )
    }
}

// ── StorageVar ────────────────────────────────────────────────────────────────

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
    pub const fn new(storage: &'static str, path: &'static str) -> Self {
        Self {
            storage,
            path,
            _marker: PhantomData,
        }
    }

    /// The storage namespace string (e.g. `"sand:data"`).
    pub fn storage(&self) -> &'static str {
        self.storage
    }

    /// The path string (e.g. `"player.mana"`).
    pub fn path(&self) -> &'static str {
        self.path
    }

    /// Build an [`NbtPath`] for this variable.
    pub fn as_path(&self) -> NbtPath {
        NbtPath::new(self.storage, self.path)
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// `data get storage <storage> <path>` — read the value.
    pub fn get(&self) -> String {
        format!("data get storage {} {}", self.storage, self.path)
    }

    /// `data get storage <storage> <path> <scale>` — read a numeric value with scale.
    pub fn get_scaled(&self, scale: f64) -> String {
        format!("data get storage {} {} {scale}", self.storage, self.path)
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// `data modify storage <storage> <path> set value <snbt>`.
    pub fn set_value(&self, value: impl Into<SnbtValue>) -> String {
        format!(
            "data modify storage {} {} set value {}",
            self.storage,
            self.path,
            value.into()
        )
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch.
    pub fn set_raw_snbt(&self, snbt: RawSnbt) -> String {
        self.set_value(SnbtValue::Raw(snbt))
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT.
    #[deprecated(since = "0.1.0", note = "use set_raw_snbt(RawSnbt::new(...))")]
    pub fn set_raw(&self, snbt: &str) -> String {
        self.set_raw_snbt(RawSnbt::new(snbt))
    }

    /// Set an integer value.
    pub fn set_int(&self, v: i32) -> String {
        self.set_value(v)
    }

    /// Set a long value (`<v>L` SNBT).
    pub fn set_long(&self, v: i64) -> String {
        self.set_value(v)
    }

    /// Set a float value (`<v>f` SNBT).
    pub fn set_float(&self, v: f32) -> String {
        self.set_value(v)
    }

    /// Set a double value (`<v>d` SNBT).
    pub fn set_double(&self, v: f64) -> String {
        self.set_value(v)
    }

    /// Set a string value (auto-quoted, backslash-escaping inner quotes).
    pub fn set_string(&self, v: &str) -> String {
        self.set_value(v)
    }

    /// Set a boolean as a byte (0b or 1b SNBT).
    pub fn set_bool(&self, v: bool) -> String {
        self.set_value(v)
    }

    /// `data modify storage <storage> <path> set from storage <src> <src_path>` — copy.
    pub fn copy_from(&self, src_storage: &str, src_path: &str) -> String {
        format!(
            "data modify storage {} {} set from storage {} {}",
            self.storage, self.path, src_storage, src_path
        )
    }

    // ── Delete / exists ───────────────────────────────────────────────────────

    /// `data remove storage <storage> <path>` — remove the tag.
    pub fn remove(&self) -> String {
        format!("data remove storage {} {}", self.storage, self.path)
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    pub fn exists(&self) -> Condition {
        Condition::StorageExists {
            location: self.storage.to_string(),
            path: self.path.to_string(),
        }
    }
}

fn escape_snbt_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn is_bare_snbt_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.'))
}

fn snbt_compound_key(key: &str) -> String {
    if is_bare_snbt_key(key) {
        key.to_string()
    } else {
        format!("\"{}\"", escape_snbt_string(key))
    }
}

fn nbt_path_key(key: &str) -> String {
    if is_bare_snbt_key(key) {
        key.to_string()
    } else {
        format!("\"{}\"", escape_snbt_string(key))
    }
}

fn append_path_field(path: &str, field: &str) -> String {
    if path.is_empty() {
        nbt_path_key(field)
    } else {
        format!("{path}.{}", nbt_path_key(field))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

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
        match &cond {
            Condition::StorageExists { location, path } => {
                assert_eq!(location, "sand:data");
                assert_eq!(path, "player.mana");
            }
            other => panic!("unexpected: {other:?}"),
        }
        let cmds = cond.execute_commands(false, "run say exists");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if data storage sand:data player.mana"));
    }

    #[test]
    fn nbt_path_navigate() {
        let base = NbtPath::new("sand:data", "player");
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
        let p = NbtPath::new("sand:data", "player.mana");
        assert_eq!(p.get(), "data get storage sand:data player.mana");
        assert_eq!(p.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn nbt_path_set_bool() {
        let p = NbtPath::new("sand:data", "player").key("mana");
        assert_eq!(
            p.set_bool(true),
            "data modify storage sand:data player.mana set value 1b"
        );
    }

    #[test]
    fn nbt_path_raw_snbt_escape_hatch() {
        let p = NbtPath::new("sand:data", "player.payload");
        assert_eq!(
            p.set_raw_snbt(RawSnbt::new("{custom:1b}")),
            "data modify storage sand:data player.payload set value {custom:1b}"
        );
    }

    #[test]
    fn nbt_path_exists() {
        let p = NbtPath::new("sand:data", "player.mana");
        let cond = p.exists();
        assert!(matches!(cond, Condition::StorageExists { .. }));
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
            MAGIC_MANA.exists(),
            Condition::StorageExists { .. }
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
}
