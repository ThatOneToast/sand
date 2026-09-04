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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::StorageKind",
    aliases = ["sand::cmd::StorageKind", "sand::prelude::cmd::StorageKind"],
    module = "sand::command",
    summary = "Declares the intended scope of a [`Storage`] namespace.",
    context = "Declares the intended scope of a [`Storage`] namespace. This is a semantic annotation — Minecraft does not enforce it.",
    minecraft = "This is a semantic annotation — Minecraft does not enforce it.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::StorageKind;",
    variants(Global = "One namespace shared by all players and functions. Use for world state, boss phases, global flags, server-wide counters.", PerPlayer = "Conceptually per-player. Callers scope paths by player identity (e.g. `\"players.<uuid>.kills\"`)."),
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Storage",
    aliases = ["sand::cmd::Storage", "sand::prelude::cmd::Storage"],
    module = "sand::command",
    summary = "A named Minecraft NBT storage namespace — used like a `HashMap<String, NbtValue>`.",
    context = "A named Minecraft NBT storage namespace — used like a `HashMap<String, NbtValue>`. Keys are dot-separated NBT paths (e.g. `\"boss_phase\"`, `\"players.health\"`). Values are typed Rust values that are automatically serialized to SNBT.",
    minecraft = "Keys are dot-separated NBT paths (e.g. `\"boss_phase\"`, `\"players.health\"`). Values are typed Rust values that are automatically serialized to SNBT.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Storage;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::global",
        aliases = ["sand::cmd::Storage::global", "sand::prelude::cmd::Storage::global"],
        module = "sand::command",
        kind = "method",
        summary = "Construct a global storage namespace at compile time.",
        context = "Construct a global storage namespace at compile time. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to construct a global storage namespace at compile time."),
        returns = "A `Storage` representing a global storage namespace at compile time.",
        example = "static WORLD: Storage = Storage::global(\"my_pack:world\");",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::per_player",
        aliases = ["sand::cmd::Storage::per_player", "sand::prelude::cmd::Storage::per_player"],
        module = "sand::command",
        kind = "method",
        summary = "Construct a per-player storage namespace at compile time.",
        context = "Construct a per-player storage namespace at compile time. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to construct a per-player storage namespace at compile time."),
        returns = "A `Storage` representing a per-player storage namespace at compile time.",
        example = "static PLAYERS: Storage = Storage::per_player(\"my_pack:players\");",
    )]
    pub const fn per_player(id: &'static str) -> Self {
        Self {
            id: Cow::Borrowed(id),
            kind: StorageKind::PerPlayer,
        }
    }

    /// Dynamic constructor for runtime-determined IDs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::new",
        aliases = ["sand::cmd::Storage::new", "sand::prelude::cmd::Storage::new"],
        module = "sand::command",
        kind = "method",
        summary = "Dynamic constructor for runtime-determined IDs.",
        context = "Dynamic constructor for runtime-determined IDs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` supplies the runtime-determined resource identifier.", kind = "`kind` identifies whether the runtime target is a block, entity, or storage value."),
        returns = "A `Storage` for runtime-determined IDs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl Into < String >, kind: sand::command::StorageKind)  {\n    let storage = sand::command::Storage::new(id, kind);\n}",
    )]
    pub fn new(id: impl Into<String>, kind: StorageKind) -> Self {
        Self {
            id: Cow::Owned(id.into()),
            kind,
        }
    }

    /// The resource-location string for this storage namespace.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::id",
        aliases = ["sand::cmd::Storage::id", "sand::prelude::cmd::Storage::id"],
        module = "sand::command",
        kind = "method",
        summary = "The resource-location string for this storage namespace.",
        context = "The resource-location string for this storage namespace. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to use the resource-location string for this storage namespace.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage)  {\n    let id = storage_value.id();\n}",
    )]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The declared scope of this storage namespace.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::kind",
        aliases = ["sand::cmd::Storage::kind", "sand::prelude::cmd::Storage::kind"],
        module = "sand::command",
        kind = "method",
        summary = "The declared scope of this storage namespace.",
        context = "The declared scope of this storage namespace. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `StorageKind` value produced to use the declared scope of this storage namespace.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage)  {\n    let kind = storage_value.kind();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::insert",
        aliases = ["sand::cmd::Storage::insert", "sand::prelude::cmd::Storage::insert"],
        module = "sand::command",
        kind = "method",
        summary = "Set `key` to `value`. Equivalent to `HashMap::insert`. Overwrites any existing value.",
        context = "Set `key` to `value`. Equivalent to `HashMap::insert`. Overwrites any existing value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Set `key` to `value`.", value = "Set `key` to `value`."),
        returns = "The string value produced to set `key` to `value`. Equivalent to `HashMap::insert`. Overwrites any existing value.",
        example = "WORLD.insert(\"boss_phase\", 2_i32)   // → data modify storage … set value 2\nWORLD.insert(\"active\",     true)    // → data modify storage … set value 1b\nWORLD.insert(\"name\",       \"Boss\")  // → data modify storage … set value \"Boss\"",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::remove",
        aliases = ["sand::cmd::Storage::remove", "sand::prelude::cmd::Storage::remove"],
        module = "sand::command",
        kind = "method",
        summary = "Delete `key` from storage. Equivalent to `HashMap::remove`.",
        context = "Delete `key` from storage. Equivalent to `HashMap::remove`. Raw/unchecked: hand-formats the command without routing the storage id or NBT path through the typed [`DataTarget`]/[`NbtPath`](sand::data::NbtPath) validators. Prefer [`Storage::try_remove`].",
        minecraft = "Raw/unchecked: hand-formats the command without routing the storage id or NBT path through the typed [`DataTarget`]/[`NbtPath`](sand::data::NbtPath) validators. Prefer [`Storage::try_remove`].",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Delete `key` from storage."),
        returns = "The string value produced to delete `key` from storage. Equivalent to `HashMap::remove`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >)  {\n    let remove = storage_value.remove(key);\n}",
    )]
    pub fn remove(&self, key: impl Into<String>) -> String {
        format!("data remove storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::remove`].
    ///
    /// Routes through the same [`DataTarget`]/NBT-path validation as
    /// [`sand_commands::DataCommand`]: the storage id must be a valid
    /// `namespace:path` resource location and `key` must be a
    /// structurally valid NBT path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_remove",
        aliases = ["sand::cmd::Storage::try_remove", "sand::prelude::cmd::Storage::try_remove"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::remove`]. Routes through the same [`DataTarget`]/NBT-path validation as [`sand::data::DataCommand`]: the storage id must be a valid `namespace:path` resource location and `key` must be a structurally valid NBT path.",
        context = "Validated counterpart to [`Storage::remove`]. Routes through the same [`DataTarget`]/NBT-path validation as [`sand::data::DataCommand`]: the storage id must be a valid `namespace:path` resource location and `key` must be a structurally valid NBT path. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Routes through the same [`DataTarget`]/NBT-path validation as [`sand::data::DataCommand`]: the storage id must be a valid `namespace:path` resource location and `key` must be a structurally valid NBT path.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Routes through the same [`DataTarget`]/NBT-path validation as [`sand::data::DataCommand`]: the storage id must be a valid `namespace:path` resource location and `key` must be a structurally valid NBT path."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::remove`]. Routes through the same [`DataTarget`]/NBT-path validation as [`sand::data::DataCommand`]: the storage id must be a valid `namespace:path` resource location and `key` must be a structurally valid NBT path; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >)  {\n    let try_remove = storage_value.try_remove(key);\n}",
    )]
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
    ///     .store_result_score(ScoreHolder::entity(Target::self_()), "my_obj")
    ///     .run(WORLD.get("boss_phase"))
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::get",
        aliases = ["sand::cmd::Storage::get", "sand::prelude::cmd::Storage::get"],
        module = "sand::command",
        kind = "method",
        summary = "Returns a `data get storage` command that reads `key`.",
        context = "Returns a `data get storage` command that reads `key`. Use this as the `run` argument of an `execute store result score` chain to load the value into a scoreboard objective.",
        minecraft = "Use this as the `run` argument of an `execute store result score` chain to load the value into a scoreboard objective.",
        use_when = ["Use this as the `run` argument of an `execute store result score` chain to load the value into a scoreboard objective."],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Returns a `data get storage` command that reads `key`."),
        returns = "Returns a `data get storage` command that reads `key`.",
        example = "Execute::new()\n.store_result_score(ScoreHolder::entity(Target::self_()), \"my_obj\")\n.run(WORLD.get(\"boss_phase\"))",
    )]
    pub fn get(&self, key: impl Into<String>) -> String {
        format!("data get storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::get`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_get",
        aliases = ["sand::cmd::Storage::try_get", "sand::prelude::cmd::Storage::try_get"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::get`].",
        context = "Validated counterpart to [`Storage::get`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to use validated counterpart to [`Storage::get`]."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::get`]; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >)  {\n    let try_get = storage_value.try_get(key);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::get_scaled",
        aliases = ["sand::cmd::Storage::get_scaled", "sand::prelude::cmd::Storage::get_scaled"],
        module = "sand::command",
        kind = "method",
        summary = "Like [`get`](Self::get) but scales the numeric result by `scale`.",
        context = "Like [`get`](Self::get) but scales the numeric result by `scale`. Useful when piping float NBT (e.g. `Health`) into integer scoreboards. Raw/unchecked: accepts a non-finite `scale`. Prefer [`Storage::try_get_scaled`].",
        minecraft = "Useful when piping float NBT (e.g. `Health`) into integer scoreboards.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to use like [`get`](Self::get) but scales the numeric result by `scale`.", scale = "Like [`get`](Self::get) but scales the numeric result by `scale`."),
        returns = "The string value produced to use like [`get`](Self::get) but scales the numeric result by `scale`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, scale: f64)  {\n    let get_scaled = storage_value.get_scaled(key, scale);\n}",
    )]
    pub fn get_scaled(&self, key: impl Into<String>, scale: f64) -> String {
        format!("data get storage {} {} {scale}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::get_scaled`]. Rejects a
    /// non-finite `scale` in addition to the storage id/path validation
    /// shared with [`Storage::try_get`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_get_scaled",
        aliases = ["sand::cmd::Storage::try_get_scaled", "sand::prelude::cmd::Storage::try_get_scaled"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::get_scaled`]. Rejects a non-finite `scale` in addition to the storage id/path validation shared with [`Storage::try_get`].",
        context = "Validated counterpart to [`Storage::get_scaled`]. Rejects a non-finite `scale` in addition to the storage id/path validation shared with [`Storage::try_get`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to use validated counterpart to [`Storage::get_scaled`]. Rejects a non-finite `scale` in addition to the storage id/path validation shared with [`Storage::try_get`].", scale = "Validated counterpart to [`Storage::get_scaled`]. Rejects a non-finite `scale` in addition to the storage id/path validation shared with [`Storage::try_get`]."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::get_scaled`]. Rejects a non-finite `scale` in addition to the storage id/path validation shared with [`Storage::try_get`]; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, scale: f64)  {\n    let try_get_scaled = storage_value.try_get_scaled(key, scale);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::contains",
        aliases = ["sand::cmd::Storage::contains", "sand::prelude::cmd::Storage::contains"],
        module = "sand::command",
        kind = "method",
        summary = "Returns a condition fragment for use with `execute if data storage …`.",
        context = "Returns a condition fragment for use with `execute if data storage …`. Equivalent to `HashMap::contains_key`. Use in `Execute::if_` to branch on whether `key` is present.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Equivalent to `HashMap::contains_key`. Use in `Execute::if_` to branch on whether `key` is present."),
        returns = "Returns a condition fragment for use with `execute if data storage …`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >)  {\n    let contains = storage_value.contains(key);\n}",
    )]
    pub fn contains(&self, key: impl Into<String>) -> String {
        format!("data storage {} {}", self.id, key.into())
    }

    /// Validated counterpart to [`Storage::contains`].
    ///
    /// `data storage <id> <key>` is a condition fragment, not a standalone
    /// `DataCommand`, so this validates the storage id and NBT path through
    /// the same [`sand_commands`] validators (via a throwaway `data get`
    /// read-shaped check) without changing the emitted fragment's syntax.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_contains",
        aliases = ["sand::cmd::Storage::try_contains", "sand::prelude::cmd::Storage::try_contains"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::contains`]. `data storage <id> <key>` is a condition fragment, not a standalone `DataCommand`, so this validates the storage id and NBT path through the same [`sand_commands`] validators (via a throwaway `data get` read-shaped check) without changing the emitted fragment's syntax.",
        context = "Validated counterpart to [`Storage::contains`]. `data storage <id> <key>` is a condition fragment, not a standalone `DataCommand`, so this validates the storage id and NBT path through the same [`sand_commands`] validators (via a throwaway `data get` read-shaped check) without changing the emitted fragment's syntax. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "`data storage <id> <key>` is a condition fragment, not a standalone `DataCommand`, so this validates the storage id and NBT path through the same [`sand_commands`] validators (via a throwaway `data get` read-shaped check) without changing the emitted fragment's syntax.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to use validated counterpart to [`Storage::contains`]. `data storage <id> <key>` is a condition fragment, not a standalone `DataCommand`, so this validates the storage id and NBT path through the same [`sand_commands`] validators (via a throwaway `data get` read-shaped check) without changing the emitted fragment's syntax."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::contains`]. `data storage <id> <key>` is a condition fragment, not a standalone `DataCommand`, so this validates the storage id and NBT path through the same [`sand_commands`] validators (via a throwaway `data get` read-shaped check) without changing the emitted fragment's syntax; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >)  {\n    let try_contains = storage_value.try_contains(key);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::get_or_insert",
        aliases = ["sand::cmd::Storage::get_or_insert", "sand::prelude::cmd::Storage::get_or_insert"],
        module = "sand::command",
        kind = "method",
        summary = "Set `key` to `default` only if it is not already present.",
        context = "Set `key` to `default` only if it is not already present. Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single `execute unless data storage … run data modify …` command. Raw/unchecked: prefer [`Storage::try_get_or_insert`].",
        minecraft = "Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single `execute unless data storage … run data modify …` command.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Set `key` to `default` only if it is not already present.", default = "Set `key` to `default` only if it is not already present."),
        returns = "Equivalent to `HashMap::entry(k).or_insert(v)`. Returns a single `execute unless data storage … run data modify …` command.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, default: impl Into < sand::data::NbtValue >)  {\n    let get_or_insert = storage_value.get_or_insert(key, default);\n}",
    )]
    pub fn get_or_insert(&self, key: impl Into<String>, default: impl Into<NbtValue>) -> String {
        let key = key.into();
        let val = default.into();
        format!(
            "execute unless data storage {} {} run data modify storage {} {} set value {}",
            self.id, key, self.id, key, val
        )
    }

    /// Validated counterpart to [`Storage::get_or_insert`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_get_or_insert",
        aliases = ["sand::cmd::Storage::try_get_or_insert", "sand::prelude::cmd::Storage::try_get_or_insert"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::get_or_insert`].",
        context = "Validated counterpart to [`Storage::get_or_insert`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to use validated counterpart to [`Storage::get_or_insert`].", default = "`default` sets the default for validated counterpart to [`Storage::get_or_insert`]."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::get_or_insert`]; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, default: impl Into < sand::data::NbtValue >)  {\n    let try_get_or_insert = storage_value.try_get_or_insert(key, default);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::push",
        aliases = ["sand::cmd::Storage::push", "sand::prelude::cmd::Storage::push"],
        module = "sand::command",
        kind = "method",
        summary = "Append `value` to the end of the list at `key`.",
        context = "Append `value` to the end of the list at `key`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Append `value` to the end of the list at `key`.", value = "Append `value` to the end of the list at `key`."),
        returns = "The string value produced to append `value` to the end of the list at `key`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, value: impl Into < sand::data::NbtValue >)  {\n    let push = storage_value.push(key, value);\n}",
    )]
    pub fn push(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key.into()).append(value)
    }

    /// Prepend `value` to the front of the list at `key`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::push_front",
        aliases = ["sand::cmd::Storage::push_front", "sand::prelude::cmd::Storage::push_front"],
        module = "sand::command",
        kind = "method",
        summary = "Prepend `value` to the front of the list at `key`.",
        context = "Prepend `value` to the front of the list at `key`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "Prepend `value` to the front of the list at `key`.", value = "Prepend `value` to the front of the list at `key`."),
        returns = "The string value produced to prepend `value` to the front of the list at `key`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, value: impl Into < sand::data::NbtValue >)  {\n    let push_front = storage_value.push_front(key, value);\n}",
    )]
    pub fn push_front(&self, key: impl Into<String>, value: impl Into<NbtValue>) -> String {
        DataModify::new(self.target(), key.into()).prepend(value)
    }

    // ── Merge ─────────────────────────────────────────────────────────────

    /// `data merge storage <id> <nbt>` — merge a compound into the root.
    ///
    /// Raw/unchecked: prefer [`Storage::try_merge`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::merge",
        aliases = ["sand::cmd::Storage::merge", "sand::prelude::cmd::Storage::merge"],
        module = "sand::command",
        kind = "method",
        summary = "`data merge storage <id> <nbt>` — merge a compound into the root.",
        context = "`data merge storage <id> <nbt>` — merge a compound into the root. Raw/unchecked: prefer [`Storage::try_merge`].",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to emit the documented `data merge storage <id> <nbt>` — merge a compound into the root form."),
        returns = "The string value produced to emit the documented `data merge storage <id> <nbt>` — merge a compound into the root form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, value: impl Into < sand::data::NbtValue >)  {\n    let merge = storage_value.merge(value);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::try_merge",
        aliases = ["sand::cmd::Storage::try_merge", "sand::prelude::cmd::Storage::try_merge"],
        module = "sand::command",
        kind = "method",
        summary = "Validated counterpart to [`Storage::merge`]. Validates the storage id through the same resource-location shape check used by [`DataTarget::Storage`]. `value`'s NBT structure is not re-validated here: [`sand::data::DataCommand::Merge`] requires a structured `NbtCompound`, while this compatibility API keeps accepting any [`NbtValue`] (including [`NbtValue::raw`] escape hatches) for the merge payload.",
        context = "Validated counterpart to [`Storage::merge`]. Validates the storage id through the same resource-location shape check used by [`DataTarget::Storage`]. `value`'s NBT structure is not re-validated here: [`sand::data::DataCommand::Merge`] requires a structured `NbtCompound`, while this compatibility API keeps accepting any [`NbtValue`] (including [`NbtValue::raw`] escape hatches) for the merge payload. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Validates the storage id through the same resource-location shape check used by [`DataTarget::Storage`]. `value`'s NBT structure is not re-validated here: [`sand::data::DataCommand::Merge`] requires a structured `NbtCompound`, while this compatibility API keeps accepting any [`NbtValue`] (including [`NbtValue::raw`] escape hatches) for the merge payload.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "Validates the storage id through the same resource-location shape check used by [`DataTarget::Storage`]. `value`'s NBT structure is not re-validated here: [`sand::data::DataCommand::Merge`] requires a structured `NbtCompound`, while this compatibility API keeps accepting any [`NbtValue`] (including [`NbtValue::raw`] escape hatches) for the merge payload."),
        returns = "On success, the value produced to use validated counterpart to [`Storage::merge`]. Validates the storage id through the same resource-location shape check used by [`DataTarget::Storage`]. `value`'s NBT structure is not re-validated here: [`sand::data::DataCommand::Merge`] requires a structured `NbtCompound`, while this compatibility API keeps accepting any [`NbtValue`] (including [`NbtValue::raw`] escape hatches) for the merge payload; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, value: impl Into < sand::data::NbtValue >)  {\n    let try_merge = storage_value.try_merge(value);\n}",
    )]
    pub fn try_merge(&self, value: impl Into<NbtValue>) -> CommandResult<String> {
        sand_commands::validate::resource_location_shape(&self.id, "Storage::try_merge", "id")?;
        Ok(format!("data merge storage {} {}", self.id, value.into()))
    }

    // ── Copy from other locations ─────────────────────────────────────────

    /// Copy a value from entity NBT into this storage.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::copy_from_entity",
        aliases = ["sand::cmd::Storage::copy_from_entity", "sand::prelude::cmd::Storage::copy_from_entity"],
        module = "sand::command",
        kind = "method",
        summary = "Copy a value from entity NBT into this storage.",
        context = "Copy a value from entity NBT into this storage. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to copy a value from entity NBT into this storage.", entity = "`entity` provides the entity participant or predicate used to copy a value from entity NBT into this storage.", src_path = "`src_path` is used to copy a value from entity NBT into this storage."),
        returns = "The string value produced to copy a value from entity NBT into this storage.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, entity: sand::command::Target, src_path: impl Into < String >)  {\n    let copy_from_entity = storage_value.copy_from_entity(key, entity, src_path);\n}",
    )]
    pub fn copy_from_entity(
        &self,
        key: impl Into<String>,
        entity: impl sand_commands::TargetArgument,
        src_path: impl Into<String>,
    ) -> String {
        DataModify::new(self.target(), key.into()).set_from(
            DataTarget::Entity(entity.into_target_selector()),
            src_path.into(),
        )
    }

    /// Copy a value from another storage namespace.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Storage::copy_from_storage",
        aliases = ["sand::cmd::Storage::copy_from_storage", "sand::prelude::cmd::Storage::copy_from_storage"],
        module = "sand::command",
        kind = "method",
        summary = "Copy a value from another storage namespace.",
        context = "Copy a value from another storage namespace. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(key = "`key` provides the key that identifies the setting or entry used to copy a value from another storage namespace.", src_id = "`src_id` is used to copy a value from another storage namespace.", src_path = "`src_path` is used to copy a value from another storage namespace."),
        returns = "The string value produced to copy a value from another storage namespace.",
        example = "use sand::prelude::*;\n\nfn demonstrate(storage_value: &sand::command::Storage, key: impl Into < String >, src_id: impl Into < String >, src_path: impl Into < String >)  {\n    let copy_from_storage = storage_value.copy_from_storage(key, src_id, src_path);\n}",
    )]
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
