//! Typed API for Minecraft's `data` command and NBT storage.
//!
//! # Storage — a typed HashMap over Minecraft NBT
//!
//! [`Storage`] wraps a Minecraft NBT storage namespace and exposes a
//! Rust-HashMap–style API. You write typed Rust values; the underlying
//! Minecraft commands are generated automatically.
//!
//! ```rust,ignore
//! use sand_core::cmd::{Storage, NbtValue};
//!
//! static WORLD: Storage = Storage::global("my_pack:world");
//!
//! // Insert typed values — no raw NBT strings needed
//! WORLD.insert("boss_phase", 2_i32)       // → data modify … set value 2
//! WORLD.insert("active",     true)        // → data modify … set value 1b
//! WORLD.insert("name",       "Golem")     // → data modify … set value "Golem"
//!
//! // Check / ensure defaults
//! WORLD.contains("boss_phase")            // condition fragment for `execute if`
//! WORLD.get_or_insert("boss_phase", 1_i32) // sets default only when absent
//!
//! // Read into a scoreboard via execute store
//! WORLD.get("boss_phase")                 // data get storage my_pack:world boss_phase
//!
//! // Remove
//! WORLD.remove("boss_phase")
//!
//! // Lists
//! WORLD.push("kill_log", NbtValue::raw(r#"{type:"zombie"}"#))
//! WORLD.push_front("kill_log", NbtValue::from("Golem"))
//! ```
//!
//! ## Minecraft storage locations
//!
//! | Location | Type | Persists | Best for |
//! |----------|------|----------|----------|
//! | [`DataTarget::Entity`] | per-entity NBT | until entity removed | per-entity state |
//! | [`DataTarget::Block`]  | block-entity NBT | until block removed | chest/furnace contents |
//! | [`DataTarget::Storage`] | named storage | world lifetime | global / cross-function state |

use std::borrow::Cow;
use std::fmt;

use super::{BlockPos, Command, Selector};

// ── NbtValue ──────────────────────────────────────────────────────────────────

/// A typed NBT value for use with [`Storage`] and [`DataModify`].
///
/// Implements `From` for the most common Rust primitives so you rarely need
/// to construct this explicitly:
///
/// ```rust,ignore
/// store.insert("phase",  2_i32);   // i32 → NbtValue::Int
/// store.insert("active", true);    // bool → NbtValue::Bool
/// store.insert("name",   "Golem"); // &str → NbtValue::String
/// store.insert("ratio",  0.5_f64); // f64  → NbtValue::Double
/// ```
///
/// For compound or list values use [`NbtValue::raw`]:
/// ```rust,ignore
/// store.insert("spawn", NbtValue::raw("{x:0,y:64,z:0}"));
/// ```
#[derive(Debug, Clone)]
pub enum NbtValue {
    /// Boolean flag. Serializes as `1b` (true) or `0b` (false).
    Bool(bool),
    /// 8-bit signed integer. Serializes as `<n>b`.
    Byte(i8),
    /// 16-bit signed integer. Serializes as `<n>s`.
    Short(i16),
    /// 32-bit signed integer. Serializes as `<n>`.
    Int(i32),
    /// 64-bit signed integer. Serializes as `<n>L`.
    Long(i64),
    /// 32-bit float. Serializes as `<n>f`.
    Float(f32),
    /// 64-bit float. Serializes as `<n>d`.
    Double(f64),
    /// UTF-8 string. Serializes as `"<escaped>"`.
    String(String),
    /// Raw SNBT — use this for compounds, lists, or any pre-formatted NBT.
    Raw(String),
}

impl NbtValue {
    /// Wrap a pre-formatted SNBT string (e.g. `"{x:0,y:64,z:0}"`).
    pub fn raw(snbt: impl Into<String>) -> Self {
        NbtValue::Raw(snbt.into())
    }
}

impl fmt::Display for NbtValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtValue::Bool(b) => write!(f, "{}", if *b { "1b" } else { "0b" }),
            NbtValue::Byte(n) => write!(f, "{n}b"),
            NbtValue::Short(n) => write!(f, "{n}s"),
            NbtValue::Int(n) => write!(f, "{n}"),
            NbtValue::Long(n) => write!(f, "{n}L"),
            NbtValue::Float(n) => write!(f, "{n}f"),
            NbtValue::Double(n) => write!(f, "{n}d"),
            NbtValue::String(s) => {
                write!(f, "\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            }
            NbtValue::Raw(s) => write!(f, "{s}"),
        }
    }
}

impl From<bool> for NbtValue {
    fn from(v: bool) -> Self {
        NbtValue::Bool(v)
    }
}
impl From<i8> for NbtValue {
    fn from(v: i8) -> Self {
        NbtValue::Byte(v)
    }
}
impl From<i16> for NbtValue {
    fn from(v: i16) -> Self {
        NbtValue::Short(v)
    }
}
impl From<i32> for NbtValue {
    fn from(v: i32) -> Self {
        NbtValue::Int(v)
    }
}
impl From<i64> for NbtValue {
    fn from(v: i64) -> Self {
        NbtValue::Long(v)
    }
}
impl From<f32> for NbtValue {
    fn from(v: f32) -> Self {
        NbtValue::Float(v)
    }
}
impl From<f64> for NbtValue {
    fn from(v: f64) -> Self {
        NbtValue::Double(v)
    }
}
impl From<&str> for NbtValue {
    fn from(v: &str) -> Self {
        NbtValue::String(v.to_owned())
    }
}
impl From<String> for NbtValue {
    fn from(v: String) -> Self {
        NbtValue::String(v)
    }
}

// ── DataTarget ────────────────────────────────────────────────────────────────

/// Where data should be read from or written to.
#[derive(Debug, Clone)]
pub enum DataTarget {
    Entity(Selector),
    Block(BlockPos),
    /// A named NBT storage namespace (e.g. `"my_pack:global"`).
    Storage(String),
}

impl DataTarget {
    pub fn entity(selector: Selector) -> Self {
        DataTarget::Entity(selector)
    }
    pub fn block(pos: BlockPos) -> Self {
        DataTarget::Block(pos)
    }
    pub fn storage(id: impl Into<String>) -> Self {
        DataTarget::Storage(id.into())
    }
}

impl fmt::Display for DataTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataTarget::Entity(sel) => write!(f, "entity {sel}"),
            DataTarget::Block(pos) => write!(f, "block {pos}"),
            DataTarget::Storage(id) => write!(f, "storage {id}"),
        }
    }
}

// ── DataModify ────────────────────────────────────────────────────────────────

/// Builder for `data modify <target> <path> <operation> <source>`.
///
/// # Example
/// ```rust,ignore
/// use sand_core::cmd::{data_modify, DataTarget, NbtValue, Selector};
///
/// // data modify entity @s Custom.ready set value 1b
/// let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.ready")
///     .set(true);
///
/// // data modify entity @s Inventory append value {id:"minecraft:apple",Count:1b}
/// let cmd = data_modify(DataTarget::entity(Selector::self_()), "Inventory")
///     .append(NbtValue::raw(r#"{id:"minecraft:apple",Count:1b}"#));
/// ```
#[derive(Debug, Clone)]
pub struct DataModify {
    target: DataTarget,
    path: String,
}

impl DataModify {
    pub(crate) fn new(target: DataTarget, path: impl Into<String>) -> Self {
        Self {
            target,
            path: path.into(),
        }
    }

    /// `set value <nbt>` — overwrite the path with a typed value.
    pub fn set(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} set value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `set from <source> <source_path>` — copy a value from another location.
    pub fn set_from(self, source: DataTarget, source_path: impl Into<String>) -> String {
        format!(
            "data modify {} {} set from {} {}",
            self.target,
            self.path,
            source,
            source_path.into()
        )
    }

    /// `append value <nbt>` — push to the end of a list.
    pub fn append(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} append value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `prepend value <nbt>` — push to the front of a list.
    pub fn prepend(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} prepend value {}",
            self.target,
            self.path,
            value.into()
        )
    }

    /// `append from <source> <source_path>` — copy-append from another location.
    pub fn append_from(self, source: DataTarget, source_path: impl Into<String>) -> String {
        format!(
            "data modify {} {} append from {} {}",
            self.target,
            self.path,
            source,
            source_path.into()
        )
    }

    /// `insert <index> value <nbt>` — insert into a list at the given index.
    pub fn insert(self, index: i32, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} insert {} value {}",
            self.target,
            self.path,
            index,
            value.into()
        )
    }

    /// `merge value <nbt>` — merge a compound value into the path.
    pub fn merge(self, value: impl Into<NbtValue>) -> String {
        format!(
            "data modify {} {} merge value {}",
            self.target,
            self.path,
            value.into()
        )
    }
}

impl fmt::Display for DataModify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data modify {} {}", self.target, self.path)
    }
}

impl Command for DataModify {}

/// Build a `data modify` command targeting `target` at `path`.
///
/// # Example
/// ```rust,ignore
/// use sand_core::cmd::{data_modify, DataTarget, Selector};
/// let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase")
///     .set(2_i32);
/// assert_eq!(cmd, "data modify entity @s Custom.Phase set value 2");
/// ```
pub fn data_modify(target: DataTarget, path: impl Into<String>) -> DataModify {
    DataModify::new(target, path)
}

// ── StorageKind ───────────────────────────────────────────────────────────────

/// Declares the intended scope of a [`Storage`] namespace.
///
/// This is a semantic annotation — Minecraft does not enforce it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    /// One namespace shared by all players and functions. Use for world state,
    /// boss phases, global flags, server-wide counters.
    Global,

    /// Conceptually per-player. Callers scope paths by player identity
    /// (e.g. `"players.<uuid>.kills"`).
    ///
    /// For simpler per-player data that only needs to exist while the player is
    /// online, entity NBT (`data modify entity @s Custom.<key>`) is easier.
    PerPlayer,
}

// ── Storage ───────────────────────────────────────────────────────────────────

/// A named Minecraft NBT storage namespace — used like a `HashMap<String, NbtValue>`.
///
/// Keys are dot-separated NBT paths (e.g. `"boss_phase"`, `"players.health"`).
/// Values are typed Rust values that are automatically serialized to SNBT.
///
/// # Declaration
///
/// ```rust,ignore
/// use sand_core::cmd::Storage;
///
/// static WORLD:   Storage = Storage::global("my_pack:world");
/// static PLAYERS: Storage = Storage::per_player("my_pack:players");
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// // Write (returns the Minecraft command string)
/// WORLD.insert("boss_phase", 2_i32)   // data modify storage … set value 2
/// WORLD.insert("active",     true)    // data modify storage … set value 1b
/// WORLD.insert("name",       "Boss")  // data modify storage … set value "Boss"
///
/// // Read (for `execute store result`)
/// WORLD.get("boss_phase")             // data get storage my_pack:world boss_phase
///
/// // Existence (condition fragment for `execute if data storage …`)
/// WORLD.contains("boss_phase")        // "data storage my_pack:world boss_phase"
///
/// // Default-initialize (no-op if key already exists)
/// WORLD.get_or_insert("boss_phase", 1_i32)
///
/// // Delete
/// WORLD.remove("boss_phase")
///
/// // Lists
/// WORLD.push("kills",       NbtValue::raw(r#"{type:"zombie"}"#))
/// WORLD.push_front("queue", "Steve")
/// ```
pub struct Storage {
    id: Cow<'static, str>,
    kind: StorageKind,
}

impl Storage {
    /// Const-compatible global storage constructor.
    ///
    /// ```rust,ignore
    /// static WORLD: Storage = Storage::global("my_pack:world");
    /// ```
    pub const fn global(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id),
            kind: StorageKind::Global,
        }
    }

    /// Const-compatible per-player storage constructor.
    ///
    /// ```rust,ignore
    /// static PLAYERS: Storage = Storage::per_player("my_pack:players");
    /// ```
    pub const fn per_player(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id),
            kind: StorageKind::PerPlayer,
        }
    }

    /// Dynamic constructor for runtime-determined IDs.
    pub fn new(id: impl Into<String>, kind: StorageKind) -> Self {
        Self {
            id: Cow::Owned(id.into()),
            kind,
        }
    }

    /// The resource-location string for this storage namespace.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The declared scope of this storage namespace.
    pub fn kind(&self) -> StorageKind {
        self.kind
    }

    fn target(&self) -> DataTarget {
        DataTarget::Storage(self.id.as_ref().to_owned())
    }

    // ── HashMap-like write ────────────────────────────────────────────────

    /// Set `key` to `value`.
    ///
    /// Equivalent to `HashMap::insert`. Overwrites any existing value.
    ///
    /// ```rust,ignore
    /// WORLD.insert("boss_phase", 2_i32)   // → data modify storage … set value 2
    /// WORLD.insert("active",     true)    // → data modify storage … set value 1b
    /// WORLD.insert("name",       "Boss")  // → data modify storage … set value "Boss"
    /// ```
    pub fn insert(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key).set(value)
    }

    /// Delete `key` from storage.
    ///
    /// Equivalent to `HashMap::remove`.
    pub fn remove(&self, key: impl Into<String>) -> String {
        format!("data remove storage {} {}", self.id, key.into())
    }

    // ── HashMap-like read ─────────────────────────────────────────────────

    /// Returns a `data get storage` command that reads `key`.
    ///
    /// Use this as the `run` argument of an `execute store result score` chain
    /// to load the value into a scoreboard objective.
    ///
    /// ```rust,ignore
    /// Execute::new()
    ///     .store_result_score(ScoreHolder::entity(Selector::self_()), "my_obj")
    ///     .run(WORLD.get("boss_phase"))
    /// ```
    pub fn get(&self, key: impl Into<String>) -> String {
        format!("data get storage {} {}", self.id, key.into())
    }

    /// Like [`get`](Self::get) but scales the numeric result by `scale`.
    ///
    /// Useful when piping float NBT (e.g. `Health`) into integer scoreboards.
    pub fn get_scaled(&self, key: impl Into<String>, scale: f64) -> String {
        format!("data get storage {} {} {scale}", self.id, key.into())
    }

    // ── Existence / defaults ──────────────────────────────────────────────

    /// Returns a condition fragment for use with `execute if data storage …`.
    ///
    /// Equivalent to `HashMap::contains_key`. Use in `Execute::if_` to branch
    /// on whether `key` is present:
    ///
    /// ```rust,ignore
    /// // execute if data storage my_pack:world boss_phase run say phase exists
    /// Execute::new()
    ///     .if_(WORLD.contains("boss_phase"))
    ///     .run(cmd::say("phase exists"))
    /// ```
    pub fn contains(&self, key: impl Into<String>) -> String {
        format!("data storage {} {}", self.id, key.into())
    }

    /// Set `key` to `default` only if it is not already present.
    ///
    /// Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single
    /// `execute unless data storage … run data modify …` command.
    ///
    /// ```rust,ignore
    /// WORLD.get_or_insert("boss_phase", 1_i32)
    /// // → execute unless data storage my_pack:world boss_phase
    /// //       run data modify storage my_pack:world boss_phase set value 1
    /// ```
    pub fn get_or_insert(&self, key: impl Into<String>, default: impl Into<NbtValue>) -> String {
        let key = key.into();
        let val = default.into();
        format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            self.id, key, self.id, key, val
        )
    }

    // ── List operations ───────────────────────────────────────────────────

    /// Append `value` to the end of the list at `key`.
    ///
    /// ```rust,ignore
    /// WORLD.push("kill_log", NbtValue::raw(r#"{type:"zombie"}"#))
    /// ```
    pub fn push(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key).append(value)
    }

    /// Prepend `value` to the front of the list at `key`.
    pub fn push_front(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key).prepend(value)
    }

    // ── Merge ─────────────────────────────────────────────────────────────

    /// `data merge storage <id> <nbt>` — merge a compound into the root.
    ///
    /// Use this to set multiple keys at once:
    /// ```rust,ignore
    /// WORLD.merge(NbtValue::raw("{phase:2,active:1b}"))
    /// ```
    pub fn merge(&self, value: impl Into<NbtValue>) -> String {
        format!("data merge storage {} {}", self.id, value.into())
    }

    // ── Copy from other locations ─────────────────────────────────────────

    /// Copy a value from entity NBT into this storage.
    ///
    /// ```rust,ignore
    /// WORLD.copy_from_entity("debug.health", Selector::self_(), "Health")
    /// // → data modify storage my_pack:world debug.health set from entity @s Health
    /// ```
    pub fn copy_from_entity(
        &self,
        key: impl Into<String>,
        entity: Selector,
        src_path: impl Into<String>,
    ) -> String {
        DataModify::new(self.target(), key).set_from(DataTarget::Entity(entity), src_path)
    }

    /// Copy a value from another storage namespace.
    pub fn copy_from_storage(
        &self,
        key: impl Into<String>,
        src_id: impl Into<String>,
        src_path: impl Into<String>,
    ) -> String {
        DataModify::new(self.target(), key).set_from(DataTarget::Storage(src_id.into()), src_path)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::Selector;

    static WORLD: Storage = Storage::global("my_pack:world");
    static PLAYERS: Storage = Storage::per_player("my_pack:players");

    // ── NbtValue serialization ─────────────────────────────────────────────

    #[test]
    fn nbt_value_display() {
        assert_eq!(NbtValue::Bool(true).to_string(), "1b");
        assert_eq!(NbtValue::Bool(false).to_string(), "0b");
        assert_eq!(NbtValue::Byte(42_i8).to_string(), "42b");
        assert_eq!(NbtValue::Short(100_i16).to_string(), "100s");
        assert_eq!(NbtValue::Int(42_i32).to_string(), "42");
        assert_eq!(NbtValue::Long(99_i64).to_string(), "99L");
        assert_eq!(NbtValue::String("hi".into()).to_string(), "\"hi\"");
        assert_eq!(NbtValue::Raw("{x:0}".into()).to_string(), "{x:0}");
    }

    #[test]
    fn nbt_value_string_escaping() {
        let v = NbtValue::from("say \"hello\"");
        assert_eq!(v.to_string(), r#""say \"hello\"""#);
    }

    #[test]
    fn nbt_value_from_primitives() {
        assert_eq!(NbtValue::from(true).to_string(), "1b");
        assert_eq!(NbtValue::from(2_i32).to_string(), "2");
        assert_eq!(NbtValue::from("hi").to_string(), "\"hi\"");
        assert_eq!(NbtValue::from(1.5_f64).to_string(), "1.5d");
    }

    // ── DataModify ─────────────────────────────────────────────────────────

    #[test]
    fn data_modify_set() {
        let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase").set(2_i32);
        assert_eq!(cmd, "data modify entity @s Custom.Phase set value 2");
    }

    #[test]
    fn data_modify_set_bool() {
        let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Active").set(true);
        assert_eq!(cmd, "data modify entity @s Custom.Active set value 1b");
    }

    #[test]
    fn data_modify_append() {
        let cmd = data_modify(DataTarget::storage("my_pack:log"), "kills")
            .append(NbtValue::raw(r#"{type:"zombie"}"#));
        assert_eq!(
            cmd,
            r#"data modify storage my_pack:log kills append value {type:"zombie"}"#
        );
    }

    #[test]
    fn data_modify_set_from_entity() {
        let cmd = data_modify(DataTarget::storage("my_pack:debug"), "health")
            .set_from(DataTarget::entity(Selector::self_()), "Health");
        assert_eq!(
            cmd,
            "data modify storage my_pack:debug health set from entity @s Health"
        );
    }

    // ── Storage static / const ─────────────────────────────────────────────

    #[test]
    fn storage_static_const() {
        assert_eq!(WORLD.id(), "my_pack:world");
        assert_eq!(WORLD.kind(), StorageKind::Global);
        assert_eq!(PLAYERS.kind(), StorageKind::PerPlayer);
    }

    // ── HashMap-like API ───────────────────────────────────────────────────

    #[test]
    fn storage_insert_int() {
        assert_eq!(
            WORLD.insert("boss_phase", 2_i32),
            "data modify storage my_pack:world boss_phase set value 2"
        );
    }

    #[test]
    fn storage_insert_bool() {
        assert_eq!(
            WORLD.insert("active", true),
            "data modify storage my_pack:world active set value 1b"
        );
    }

    #[test]
    fn storage_insert_string() {
        assert_eq!(
            WORLD.insert("name", "Golem"),
            r#"data modify storage my_pack:world name set value "Golem""#
        );
    }

    #[test]
    fn storage_remove() {
        assert_eq!(
            WORLD.remove("boss_phase"),
            "data remove storage my_pack:world boss_phase"
        );
    }

    #[test]
    fn storage_get() {
        assert_eq!(
            WORLD.get("boss_phase"),
            "data get storage my_pack:world boss_phase"
        );
    }

    #[test]
    fn storage_contains() {
        assert_eq!(
            WORLD.contains("boss_phase"),
            "data storage my_pack:world boss_phase"
        );
    }

    #[test]
    fn storage_get_or_insert() {
        assert_eq!(
            WORLD.get_or_insert("boss_phase", 1_i32),
            "execute unless data storage my_pack:world boss_phase run data modify storage my_pack:world boss_phase set value 1"
        );
    }

    #[test]
    fn storage_push() {
        let store = Storage::global("my_pack:log");
        assert_eq!(
            store.push("kills", NbtValue::raw(r#"{type:"zombie"}"#)),
            r#"data modify storage my_pack:log kills append value {type:"zombie"}"#
        );
    }

    #[test]
    fn storage_merge() {
        assert_eq!(
            WORLD.merge(NbtValue::raw("{phase:2,active:1b}")),
            "data merge storage my_pack:world {phase:2,active:1b}"
        );
    }

    #[test]
    fn storage_copy_from_entity() {
        let store = Storage::global("my_pack:debug");
        assert_eq!(
            store.copy_from_entity("last_health", Selector::self_(), "Health"),
            "data modify storage my_pack:debug last_health set from entity @s Health"
        );
    }

    #[test]
    fn data_target_display() {
        assert_eq!(
            DataTarget::entity(Selector::self_()).to_string(),
            "entity @s"
        );
        assert_eq!(DataTarget::storage("ns:key").to_string(), "storage ns:key");
    }
}
