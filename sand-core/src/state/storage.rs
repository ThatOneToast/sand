//! Typed NBT storage variables backed by `data storage` commands.
#![allow(clippy::result_large_err)]

use std::fmt;
use std::marker::PhantomData;

use crate::condition::Condition;
use sand_commands::{BlockPos, DataTarget, Selector};
use sand_components::{RawSnbt, ResourceLocation};

pub use sand_commands::{
    DataCommand, Nbt, NbtCompound as SnbtCompound, NbtPath, NbtRef, NbtTarget,
    NbtValue as SnbtValue, UntypedNbt,
};

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

impl From<StorageLocation> for String {
    fn from(value: StorageLocation) -> Self {
        value.to_string()
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

    pub fn path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(DataTarget::entity(self.target.clone()), path.into())
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

    pub fn path<T>(&self, path: impl Into<NbtPath>) -> NbtRef<T> {
        NbtRef::new(DataTarget::block(self.pos.clone()), path.into())
    }
}

/// Compatibility name for the canonical command-layer [`DataTarget`].
pub type NbtLocation = DataTarget;

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

    pub fn path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage).path(self.root)
    }

    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageSchema::new requires a valid storage resource location")
    }

    pub fn get(&self) -> String {
        self.path().get().to_string()
    }

    pub fn set(&self, value: impl Into<SnbtValue>) -> String {
        self.path().set(value).to_string()
    }

    pub fn set_raw_snbt(&self, raw: RawSnbt) -> String {
        self.path().set_raw(raw.to_string()).to_string()
    }

    pub fn merge(&self, value: impl Into<SnbtValue>) -> String {
        self.path().merge(value).to_string()
    }

    pub fn remove(&self) -> String {
        self.path().remove().to_string()
    }

    pub fn exists(&self) -> Condition {
        Condition::NbtExists {
            target: DataTarget::storage(self.storage),
            path: NbtPath::new(self.root),
        }
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

    pub fn path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage)
            .path::<T>(self.root)
            .field(self.field)
    }

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
    pub fn field_path(&self) -> String {
        self.full_path()
    }

    pub fn location(&self) -> StorageLocation {
        StorageLocation::parse(self.storage)
            .expect("StorageField requires a valid storage resource location")
    }

    pub fn get(&self) -> String {
        self.path().get().to_string()
    }

    pub fn get_scaled(&self, scale: f64) -> String {
        self.path().get_scaled(scale).to_string()
    }

    pub fn set(&self, value: impl Into<SnbtValue>) -> String {
        self.set_value(value.into())
    }

    pub fn set_value(&self, value: SnbtValue) -> String {
        self.path().set(value).to_string()
    }

    pub fn set_raw_snbt(&self, raw: RawSnbt) -> String {
        self.path().set_raw(raw.to_string()).to_string()
    }

    pub fn remove(&self) -> String {
        self.path().remove().to_string()
    }

    pub fn exists(&self) -> Condition {
        Condition::NbtExists {
            target: DataTarget::storage(self.storage),
            path: NbtPath::new(self.full_path()),
        }
    }

    pub fn copy_from<OtherSchema, U>(&self, source: StorageField<OtherSchema, U>) -> String {
        self.path().copy_from(&source.path()).to_string()
    }

    /// `data modify storage <s> <path> set from entity <entity> <src_path>`
    ///
    /// Copy a value from entity NBT into this field. Takes a typed
    /// [`Selector`] — never build this by stringifying a participant handle
    /// yourself; pass [`Selector::self_()`] from inside an
    /// [`crate::participant::EntityParticipant::execute_at`] callback (or any
    /// other typed selector) instead.
    pub fn copy_from_entity(&self, entity: Selector, src_path: impl Into<String>) -> String {
        let source = Nbt::entity(entity).path::<UntypedNbt>(src_path.into());
        self.path().copy_from(&source).to_string()
    }

    pub fn copy_from_path(&self, source_storage: StorageLocation, source_path: NbtPath) -> String {
        let source = Nbt::storage(source_storage.to_string()).path::<UntypedNbt>(source_path);
        self.path().copy_from(&source).to_string()
    }

    pub fn append(&self, value: impl Into<SnbtValue>) -> String {
        self.path().append(value).to_string()
    }

    pub fn merge(&self, value: impl Into<SnbtValue>) -> String {
        self.path().merge(value).to_string()
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
    pub fn as_path(&self) -> NbtRef<T> {
        Nbt::storage(self.storage).path(self.path)
    }

    // ── Read ──────────────────────────────────────────────────────────────────

    /// `data get storage <storage> <path>` — read the value.
    pub fn get(&self) -> String {
        self.as_path().get().to_string()
    }

    /// `data get storage <storage> <path> <scale>` — read a numeric value with scale.
    pub fn get_scaled(&self, scale: f64) -> String {
        self.as_path().get_scaled(scale).to_string()
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// `data modify storage <storage> <path> set value <snbt>`.
    pub fn set_value(&self, value: impl Into<SnbtValue>) -> String {
        self.as_path().set(value).to_string()
    }

    /// `data modify storage <storage> <path> set value <snbt>` — raw SNBT escape hatch.
    pub fn set_raw_snbt(&self, snbt: RawSnbt) -> String {
        self.as_path().set_raw(snbt.to_string()).to_string()
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
        let source = Nbt::storage(src_storage).path::<UntypedNbt>(src_path);
        self.as_path().copy_from(&source).to_string()
    }

    // ── Delete / exists ───────────────────────────────────────────────────────

    /// `data remove storage <storage> <path>` — remove the tag.
    pub fn remove(&self) -> String {
        self.as_path().remove().to_string()
    }

    /// Build a `Condition` that checks `if data storage <storage> <path>`.
    pub fn exists(&self) -> Condition {
        Condition::NbtExists {
            target: DataTarget::storage(self.storage),
            path: NbtPath::new(self.path),
        }
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
            Condition::NbtExists { target, path } => {
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
        let base = Nbt::storage("sand:data").path::<UntypedNbt>("player");
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
        let p = Nbt::storage("sand:data").path::<UntypedNbt>("player.mana");
        assert_eq!(p.get(), "data get storage sand:data player.mana");
        assert_eq!(p.remove(), "data remove storage sand:data player.mana");
    }

    #[test]
    fn nbt_path_set_bool() {
        let p = Nbt::storage("sand:data")
            .path::<UntypedNbt>("player")
            .key("mana");
        assert_eq!(
            p.set_bool(true),
            "data modify storage sand:data player.mana set value 1b"
        );
    }

    #[test]
    fn nbt_path_raw_snbt_escape_hatch() {
        let p = Nbt::storage("sand:data").path::<UntypedNbt>("player.payload");
        assert_eq!(
            p.set_raw(RawSnbt::new("{custom:1b}").to_string()),
            "data modify storage sand:data player.payload set value {custom:1b}"
        );
    }

    #[test]
    fn nbt_path_exists() {
        let p = Nbt::storage("sand:data").path::<UntypedNbt>("player.mana");
        let cond = Condition::NbtExists {
            target: p.location().clone(),
            path: p.path_value().clone(),
        };
        assert!(matches!(cond, Condition::NbtExists { .. }));
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
        assert!(matches!(MAGIC.exists(), Condition::NbtExists { .. }));
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
        assert!(matches!(MAGIC_MANA.exists(), Condition::NbtExists { .. }));
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
            let command = Nbt::storage("sand:data")
                .path::<UntypedNbt>("value")
                .set(value);
            assert_eq!(
                command
                    .try_render(&sand_commands::CommandProfile::unprofiled())
                    .unwrap_err()
                    .code,
                "SAND-DATA-TARGET"
            );
        }
        let compound = Nbt::storage("sand:data")
            .path::<UntypedNbt>("value")
            .set(SnbtCompound::new().field("key\nwith\nnewline", 1_i32));
        assert!(
            compound
                .try_render(&sand_commands::CommandProfile::unprofiled())
                .is_err()
        );
        let string = Nbt::storage("sand:data")
            .path::<UntypedNbt>("player.name")
            .set_string("line1\nline2");
        assert!(
            string
                .try_render(&sand_commands::CommandProfile::unprofiled())
                .is_err()
        );
    }
}
