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

use sand_commands::{CommandResult, DataModify, DataTarget, NbtValue, Validate};

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
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::global` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::per_player` for the canonical contract."]
    pub const fn per_player(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id),
            kind: StorageKind::PerPlayer,
        }
    }

    /// Dynamic constructor for runtime-determined IDs.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::new` for the canonical contract."]
    pub fn new(id: impl Into<String>, kind: StorageKind) -> Self {
        Self {
            id: Cow::Owned(id.into()),
            kind,
        }
    }

    /// The resource-location string for this storage namespace.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::id` for the canonical contract."]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The declared scope of this storage namespace.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::kind` for the canonical contract."]
    pub fn kind(&self) -> StorageKind {
        self.kind
    }

    fn target(&self) -> DataTarget {
        DataTarget::Storage(self.id.as_ref().to_owned())
    }

    fn profile() -> sand_commands::CommandProfile {
        sand_commands::CommandProfile::unprofiled()
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::insert` for the canonical contract."]
    pub fn insert(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key.into()).set(value)
    }

    /// Delete `key` from storage.
    ///
    /// Equivalent to `HashMap::remove`.
    ///
    /// Raw/unchecked: hand-formats the command without routing the storage
    /// id or NBT path through the typed [`DataTarget`]/[`NbtPath`](sand_commands::NbtPath)
    /// validators. Prefer [`Storage::try_remove`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::remove` for the canonical contract."]
    pub fn remove(&self, key: impl Into<String>) -> String {
        format!("data remove storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::remove`].
    ///
    /// Routes through the same [`DataTarget`]/NBT-path validation as
    /// [`sand_commands::DataCommand`]: the storage id must be a valid
    /// `namespace:path` resource location and `key` must be a
    /// structurally valid NBT path.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_remove` for the canonical contract."]
    pub fn try_remove(&self, key: impl Into<String>) -> CommandResult<String> {
        self.target()
            .path(key.into())
            .remove()
            .try_render(&Self::profile())
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::get` for the canonical contract."]
    pub fn get(&self, key: impl Into<String>) -> String {
        format!("data get storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::get`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_get` for the canonical contract."]
    pub fn try_get(&self, key: impl Into<String>) -> CommandResult<String> {
        self.target()
            .path(key.into())
            .get()
            .try_render(&Self::profile())
    }

    /// Like [`get`](Self::get) but scales the numeric result by `scale`.
    ///
    /// Useful when piping float NBT (e.g. `Health`) into integer scoreboards.
    ///
    /// Raw/unchecked: accepts a non-finite `scale`. Prefer
    /// [`Storage::try_get_scaled`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::get_scaled` for the canonical contract."]
    pub fn get_scaled(&self, key: impl Into<String>, scale: f64) -> String {
        format!("data get storage {} {} {scale}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::get_scaled`]. Rejects a
    /// non-finite `scale` in addition to the storage id/path validation
    /// shared with [`Storage::try_get`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_get_scaled` for the canonical contract."]
    pub fn try_get_scaled(&self, key: impl Into<String>, scale: f64) -> CommandResult<String> {
        self.target()
            .path(key.into())
            .get_scaled(scale)
            .try_render(&Self::profile())
    }

    // ── Existence / defaults ──────────────────────────────────────────────

    /// Returns a condition fragment for use with `execute if data storage …`.
    ///
    /// Equivalent to `HashMap::contains_key`. Use in `Execute::if_` to branch
    /// on whether `key` is present.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::contains` for the canonical contract."]
    pub fn contains(&self, key: impl Into<String>) -> String {
        format!("data storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::contains`].
    ///
    /// `data storage <id> <key>` is a condition fragment, not a standalone
    /// `DataCommand`, so this validates the storage id and NBT path through
    /// the same [`sand_commands`] validators (via a throwaway `data get`
    /// read-shaped check) without changing the emitted fragment's syntax.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_contains` for the canonical contract."]
    pub fn try_contains(&self, key: impl Into<String>) -> CommandResult<String> {
        let key = key.into();
        self.target()
            .path(key.clone())
            .get()
            .validate(&Self::profile())?;
        Ok(format!("data storage {} {}", self.id, key))
    }

    /// Set `key` to `default` only if it is not already present.
    ///
    /// Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single
    /// `execute unless data storage … run data modify …` command.
    ///
    /// Raw/unchecked: prefer [`Storage::try_get_or_insert`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::get_or_insert` for the canonical contract."]
    pub fn get_or_insert(&self, key: impl Into<String>, default: impl Into<NbtValue>) -> String {
        let key = key.into();
        let val = default.into();
        format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            self.id, key, self.id, key, val
        )
    }

    /// Validated counterpart to [`Storage::get_or_insert`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_get_or_insert` for the canonical contract."]
    pub fn try_get_or_insert(
        &self,
        key: impl Into<String>,
        default: impl Into<NbtValue>,
    ) -> CommandResult<String> {
        let key = key.into();
        let contains = self.try_contains(key.clone())?;
        let set = self
            .target()
            .path(key)
            .set(default)
            .try_render(&Self::profile())?;
        Ok(format!("execute unless {contains} run {set}"))
    }

    // ── List operations ───────────────────────────────────────────────────

    /// Append `value` to the end of the list at `key`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::push` for the canonical contract."]
    pub fn push(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key.into()).append(value)
    }

    /// Prepend `value` to the front of the list at `key`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::push_front` for the canonical contract."]
    pub fn push_front(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key.into()).prepend(value)
    }

    // ── Merge ─────────────────────────────────────────────────────────────

    /// `data merge storage <id> <nbt>` — merge a compound into the root.
    ///
    /// Raw/unchecked: prefer [`Storage::try_merge`].
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::merge` for the canonical contract."]
    pub fn merge(&self, value: impl Into<NbtValue>) -> String {
        format!("data merge storage {} {}", self.id, value.into())
    }

    /// Validated counterpart to [`Storage::merge`].
    ///
    /// Validates the storage id through the same resource-location shape
    /// check used by [`DataTarget::Storage`]. `value`'s NBT structure is not
    /// re-validated here: [`sand_commands::DataCommand::Merge`] requires a
    /// structured `NbtCompound`, while this compatibility API keeps
    /// accepting any [`NbtValue`] (including [`NbtValue::raw`] escape
    /// hatches) for the merge payload.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::try_merge` for the canonical contract."]
    pub fn try_merge(&self, value: impl Into<NbtValue>) -> CommandResult<String> {
        sand_commands::validate::resource_location_shape(&self.id, "Storage::try_merge", "id")?;
        Ok(format!("data merge storage {} {}", self.id, value.into()))
    }

    // ── Copy from other locations ─────────────────────────────────────────

    /// Copy a value from entity NBT into this storage.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::copy_from_entity` for the canonical contract."]
    pub fn copy_from_entity(
        &self,
        key: impl Into<String>,
        entity: sand_commands::Selector,
        src_path: impl Into<String>,
    ) -> String {
        DataModify::new(self.target(), key.into())
            .set_from(DataTarget::Entity(entity), src_path.into())
    }

    /// Copy a value from another storage namespace.
    #[doc = "**API Contract:** Run `sand api show sand::command::Storage::copy_from_storage` for the canonical contract."]
    pub fn copy_from_storage(
        &self,
        key: impl Into<String>,
        src_id: impl Into<String>,
        src_path: impl Into<String>,
    ) -> String {
        DataModify::new(self.target(), key.into())
            .set_from(DataTarget::Storage(src_id.into()), src_path.into())
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

    // ── Validated try_* Storage API ─────────────────────────────────────────

    #[test]
    fn try_methods_match_infallible_output_for_valid_input() {
        assert_eq!(
            WORLD.try_remove("boss_phase").unwrap(),
            WORLD.remove("boss_phase")
        );
        assert_eq!(
            WORLD.try_get("boss_phase").unwrap(),
            WORLD.get("boss_phase")
        );
        assert_eq!(
            WORLD.try_get_scaled("boss_phase", 10.0).unwrap(),
            WORLD.get_scaled("boss_phase", 10.0)
        );
        assert_eq!(
            WORLD.try_contains("boss_phase").unwrap(),
            WORLD.contains("boss_phase")
        );
        assert_eq!(
            WORLD.try_get_or_insert("boss_phase", 1_i32).unwrap(),
            WORLD.get_or_insert("boss_phase", 1_i32)
        );
        assert_eq!(
            WORLD
                .try_merge(NbtValue::raw("{phase:2,active:1b}"))
                .unwrap(),
            WORLD.merge(NbtValue::raw("{phase:2,active:1b}"))
        );
    }

    #[test]
    fn try_methods_reject_invalid_storage_id() {
        let bad = Storage::new("not_a_resource_location", StorageKind::Global);
        assert!(bad.try_get("key").is_err());
        assert!(bad.try_remove("key").is_err());
        assert!(bad.try_contains("key").is_err());
        assert!(bad.try_merge(1_i32).is_err());
    }

    #[test]
    fn try_methods_reject_invalid_path() {
        assert!(WORLD.try_get("").is_err());
        assert!(WORLD.try_remove("bad..path").is_err());
    }

    #[test]
    fn try_get_scaled_rejects_non_finite_scale() {
        assert!(WORLD.try_get_scaled("boss_phase", f64::NAN).is_err());
        assert!(WORLD.try_get_scaled("boss_phase", f64::INFINITY).is_err());
    }
}
