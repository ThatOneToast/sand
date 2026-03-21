//! Minecraft NBT storage abstraction for datapacks.
//!
//! This module provides only the datapack-level types: [`Storage`] and
//! [`StorageKind`]. The low-level building blocks — [`NbtValue`], [`DataTarget`],
//! [`DataModify`], and [`data_modify`] — live in `sand-commands` and are
//! re-exported from `sand_core::cmd`.
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
//! ## Passing Storage to Objective
//!
//! `Storage` implements `Into<String>` (via `From<&Storage> for String`), so
//! it can be passed directly to `Objective::load_from`:
//!
//! ```rust,ignore
//! INFERNO_DMG.load_from(ScoreHolder::self_(), &PLAYERS, "uuid.damage")
//! ```

use std::borrow::Cow;

use sand_commands::{DataModify, DataTarget, NbtValue};

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
    /// Construct a global storage namespace at compile time.
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

    /// Construct a per-player storage namespace at compile time.
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
    /// on whether `key` is present.
    pub fn contains(&self, key: impl Into<String>) -> String {
        format!("data storage {} {}", self.id, key.into())
    }

    /// Set `key` to `default` only if it is not already present.
    ///
    /// Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single
    /// `execute unless data storage … run data modify …` command.
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
    pub fn push(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key).append(value)
    }

    /// Prepend `value` to the front of the list at `key`.
    pub fn push_front(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key).prepend(value)
    }

    // ── Merge ─────────────────────────────────────────────────────────────

    /// `data merge storage <id> <nbt>` — merge a compound into the root.
    pub fn merge(&self, value: impl Into<NbtValue>) -> String {
        format!("data merge storage {} {}", self.id, value.into())
    }

    // ── Copy from other locations ─────────────────────────────────────────

    /// Copy a value from entity NBT into this storage.
    pub fn copy_from_entity(
        &self,
        key: impl Into<String>,
        entity: sand_commands::Selector,
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

// ── Into<String> bridge ───────────────────────────────────────────────────────

/// Allows `&Storage` to be passed wherever `impl Into<String>` is expected.
///
/// This is the primary integration point between `Storage` and
/// `Objective::load_from` / `Objective::load_from_scaled`:
///
/// ```rust,ignore
/// INFERNO_DMG.load_from(ScoreHolder::self_(), &PLAYERS, "uuid.damage")
/// //                                          ^^^^^^^^^
/// //                          &Storage satisfies impl Into<String>
/// ```
impl From<&Storage> for String {
    fn from(s: &Storage) -> String {
        s.id().to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::{DataTarget, NbtValue, Selector, data_modify};

    static WORLD: Storage = Storage::global("my_pack:world");
    static PLAYERS: Storage = Storage::per_player("my_pack:players");

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
    fn storage_into_string() {
        let s: String = (&PLAYERS).into();
        assert_eq!(s, "my_pack:players");
    }

    // ── data_modify convenience ────────────────────────────────────────────

    #[test]
    fn data_modify_via_sand_commands() {
        let cmd = data_modify(DataTarget::entity(Selector::self_()), "Custom.Phase").set(2_i32);
        assert_eq!(cmd, "data modify entity @s Custom.Phase set value 2");
    }
}
