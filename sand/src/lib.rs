#![forbid(unsafe_code)]

//! # Sand — Minecraft datapacks in type-safe Rust
//!
//! Sand lets you author complete Minecraft Java datapacks — functions,
//! commands, events, custom items, recipes, advancements, loot tables,
//! predicates, dialogs, and more — as ordinary Rust code, then compile them
//! to a datapack with the `sand` CLI.
//!
//! This crate is the only dependency a datapack project needs:
//!
//! ```toml
//! [dependencies]
//! sand = "0.1"
//! ```
//!
//! ```rust,ignore
//! use sand::prelude::*;
//!
//! #[function]
//! fn hello() {
//!     cmd::say("Hello from Sand");
//! }
//! ```
//!
//! # Where to look
//!
//! - [`prelude`] — the common authoring vocabulary; `use sand::prelude::*`
//!   covers ordinary datapack development.
//! - Topic modules ([`event`], [`item`], [`state`], [`command`],
//!   [`component`], [`entity`], [`data`], [`text`], [`version`], [`vfx`]) —
//!   the full supported surface for less common needs.
//! - [`advanced`] — supported low-level export hooks and raw escape hatches
//!   for framework integrations.
//! - `__private` is macro/compiler wiring only and carries no compatibility
//!   promise; nothing in it is part of the authoring API.

// ── Procedural macros ─────────────────────────────────────────────────────────

/// Attribute and function-like macros, re-exported so authors never depend on
/// the proc-macro crate directly.
pub use sand_macros::{armor_event, component, event, function, item, run_fn, schedule};

#[cfg(feature = "resourcepack")]
pub use sand_macros::{hud_bar, hud_element, texture};

// ── Declarative macros (defined in the implementation crate) ─────────────────

pub use sand_core::{all, any, mcfunction, sand_state, temp_score};

// ── Prelude ───────────────────────────────────────────────────────────────────

pub mod prelude;

// ── Topic modules ─────────────────────────────────────────────────────────────

/// Typed command builders: `execute`, selectors, scoreboard, effects, sounds,
/// particles, block/NBT operations, and free-function command helpers.
pub use sand_core::cmd as command;

/// Same module as [`command`]; kept under its conventional short name because
/// generated code and examples call helpers as `cmd::say(...)`.
pub use sand_core::cmd;

/// The typed event model: [`event::Event`], advancement/tick-backed events,
/// triggers, composition, and handler contexts.
pub use sand_core::event;

/// Event graph/dispatch surface used by `#[event]` declarations
/// (`SandEvent`, dispatch composition, vanilla event markers).
pub use sand_core::events;

/// Custom items, item stacks, matchers, and item locations.
pub use sand_core::item;

/// Scoreboard-, storage-, and NBT-backed state: scores, flags, timers,
/// cooldowns, typed game state, and storage schemas.
pub use sand_core::state;

/// Entity and player queries plus execution-scoped contexts.
pub use sand_core::entity;

/// Datapack component builders: advancements, recipes, loot tables,
/// predicates, item modifiers, tags, dialogs, and enchantments.
pub use sand_core::components as component;

/// Conditions and `execute`-plan composition (`if_`, `unless`, `when`).
pub use sand_core::condition;
pub use sand_core::execute_when;

/// Storage/NBT data modeling re-exports.
pub use sand_core::resource_ref;

/// Version model: Minecraft versions, profiles, and capability gating.
pub use sand_core::version;

/// Particle/sound VFX sequencing.
pub use sand_core::vfx;

/// Optional gameplay systems (feature-gated).
pub use sand_core::systems;

/// Validated `namespace:path` resource identifiers.
pub use sand_core::ResourceLocation;

/// Text components, chat colors, and click/hover events for `tellraw`,
/// titles, dialogs, and books.
pub mod text {
    pub use sand_core::prelude::{
        ChatColor, ClickEvent, EntityHoverId, HoverEvent, IntoTextEntityType, Text, TextComponent,
    };
}

/// Storage/NBT data authoring: SNBT values, storage locations, and typed
/// storage schemas.
pub mod data {
    pub use sand_core::state::{
        BlockNbt, EntityNbt, NbtLocation, NbtPath, SnbtCompound, SnbtValue, StorageField,
        StorageLocation, StorageSchema, StorageVar,
    };
}

/// Supported low-level hooks: export entry points and raw escape hatches.
pub use sand_core::advanced;

/// Resource-pack authoring (HUD bars/elements, textures).
#[cfg(feature = "resourcepack")]
pub use sand_resourcepack as resourcepack;

// ── Macro/compiler wiring. Not public API. ────────────────────────────────────

#[doc(hidden)]
pub mod __private {
    //! Expansion targets for Sand's procedural macros and wiring for the
    //! compiler/export pipeline. Nothing here is a compatibility promise;
    //! paths exist solely so generated code can reach the implementation
    //! crate through the façade. See docs/architecture/adr-001.
    pub use sand_core::*;
    pub use sand_core::{cmd, condition, event, events, state};

    #[cfg(feature = "resourcepack")]
    pub use sand_resourcepack as rp;
}
