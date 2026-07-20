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
//! ```rust
//! use sand::prelude::*;
//!
//! #[function]
//! fn hello() {
//!     cmd::say("Hello from Sand");
//! }
//!
//! // `#[function]`-tagged functions return the commands they emit, so
//! // ordinary Rust tests can assert on generated output directly:
//! assert_eq!(hello(), vec!["say Hello from Sand"]);
//! ```
//!
//! # Where to look
//!
//! - [`prelude`] — the common authoring vocabulary; `use sand::prelude::*`
//!   covers ordinary datapack development.
//! - Topic modules ([`mod@event`], [`mod@item`], [`state`], [`command`],
//!   [`mod@component`], [`entity`], [`data`], [`text`], [`version`], [`vfx`]) —
//!   the full supported surface for less common needs.
//! - [`advanced`] — supported low-level export hooks and raw escape hatches
//!   for framework integrations.
//! - `__private` is macro/compiler wiring only and carries no compatibility
//!   promise; nothing in it is part of the authoring API.
//!
//! # Execution-context expectations
//!
//! Attribute macros like `#[function]`, `#[component]`, `#[event]`, and
//! `#[item]` register their targets with Sand's `inventory`-based collector at
//! program load, and the bodies they wrap are only meaningful when compiled
//! and exported through `sand build` (or `sand_export`, the binary that
//! `sand build` generates for your project). Calling a `#[function]`-tagged
//! Rust function directly (e.g. from a unit test) just returns the
//! `Vec<String>` of Minecraft commands it would emit — useful for asserting
//! on generated command output, as `examples/book_project` does — but the
//! function is only wired into the datapack's actual `.mcfunction` files
//! through the export pipeline.

// ── Procedural macros ─────────────────────────────────────────────────────────

/// `#[function]`, `#[component]`, `#[event]`, `#[item]`, `#[armor_event]`,
/// `#[schedule]`, and `run_fn!` — the attribute and function-like macros that
/// turn ordinary Rust functions into datapack functions, lifecycle hooks
/// (`Load`/`Tick`/`Tag`), typed event handlers, custom items with generated
/// predicates, armor equip/unequip watchers, and self-scheduling routines.
/// Re-exported here so authors never depend on the `sand-macros` proc-macro
/// crate directly — `use sand::prelude::*` (or these paths) is the only
/// import needed. See each macro's own docs for attribute syntax and
/// generated code; `#[function]`/`#[component]`/`#[event]` bodies are only
/// meaningful when compiled through `sand build`.
pub use sand_macros::{armor_event, component, event, function, item, run_fn, schedule};

/// `hud_bar!`, `hud_element!`, and `texture!` — declarative resource-pack
/// authoring macros for custom HUD bars/elements and referenced textures.
/// Only available with the `resourcepack` feature, and only useful alongside
/// [`resourcepack`] (the `sand-resourcepack` crate), which provides the types
/// these macros construct.
#[cfg(feature = "resourcepack")]
pub use sand_macros::{hud_bar, hud_element, texture};

// ── Declarative macros (defined in the implementation crate) ─────────────────

/// `all!`/`any!` compose typed [`condition::Condition`]s (all-of / any-of);
/// `mcfunction!` builds a `Vec<String>` of commands from semicolon-separated
/// expressions; `sand_state!` declares a typed state value with an automatic
/// lifecycle (default value, optional auto-ticking); `temp_score!` registers
/// a scoreboard objective that Sand creates for you on load without an
/// explicit `#[component(Load)]` entry. These are `macro_rules!` macros
/// defined in the implementation crate and re-exported here so `sand::` is
/// the only path authors need.
pub use sand_core::{all, any, mcfunction, sand_state, temp_score};

// ── Prelude ───────────────────────────────────────────────────────────────────

pub mod prelude;

// ── Topic modules ─────────────────────────────────────────────────────────────

/// Typed command builders: `execute` chains, selectors (`Selector`,
/// `EntityTargets`), scoreboard operations, effects, sounds, particles,
/// block/NBT operations, and free functions like `cmd::say`/`cmd::tellraw`.
/// Reach for this when the [`prelude`] doesn't already have the command
/// builder you need, or when you want to name the module explicitly (e.g. in
/// generic code taking `impl Fn() -> Vec<String>`). Every command builder
/// implements `Display`, so `.to_string()` (or letting `mcfunction!`/
/// `#[function]` collect it) produces the literal Minecraft command text.
pub use sand_core::cmd as command;

/// Same module as [`command`]; kept under its conventional short name because
/// generated code and examples call helpers as `cmd::say(...)`. Both paths
/// point at the identical module — use whichever reads better at the call
/// site.
pub use sand_core::cmd;

/// The typed event model: [`event::Event`], `AdvancementEvent` (custom
/// advancement-backed triggers), vanilla event markers, and the trigger
/// builders (`InventoryChangedTrigger`, `RecipeUnlockedTrigger`, …) used to
/// describe when an event fires. Use this module when defining your own
/// advancement-backed event type or reading its handler context
/// (`event.player()`); ordinary `#[event]` handlers for built-in vanilla
/// events usually only need the handler parameter type, exported from here
/// (e.g. `sand::event::vanilla::OnDeath`) as shown in the crate-level example.
pub use sand_core::event;

/// The event graph/dispatch surface backing `#[event]` and tick-driven custom
/// events: `SandEvent`, `SandEventDispatch` (tick/chain/after-any/after-all
/// dispatch composition), and vanilla event marker types
/// (`PlayerSprintEvent`, etc.) usable as dispatch parents. Use this module
/// when composing a custom event out of another event's detection logic
/// (`SandEventDispatch::chain::<Parent>()`) instead of writing a fresh
/// tick-poll condition from scratch.
pub use sand_core::events;

/// Custom items: `CustomItem` (the builder passed to `#[item]`), item stack
/// component types, item matchers/predicates, and item location helpers.
/// Use this when building or matching custom items outside a `#[item]`
/// function body — for example, constructing an `ItemPredicate` to gate an
/// event or `execute if items` check.
pub use sand_core::item;

/// Scoreboard-, storage-, and NBT-backed state: `ScoreVar`, `Flag`, `Timer`,
/// `Cooldown`, `GameState`/`TypedGameState`, and storage schemas
/// (`StorageVar`, `StorageSchema`). These are the building blocks behind
/// [`sand_state!`](crate::sand_state) — declare a `static` of one of these
/// types, call `.define()` from `#[component(Load)]`, and read/write it with
/// `.of(selector)` inside function bodies. Most of this module is already in
/// the [`prelude`]; reach for it directly when writing generic helpers over
/// state types.
pub use sand_core::state;

/// Entity and player queries (`Selector`-adjacent typed query builders) and
/// execution-scoped contexts (`EntityContext`, `PlayerContext`) used to model
/// "the entity/player this command executes as" inside typed `execute`
/// chains and event handlers.
pub use sand_core::entity;

/// Datapack component builders: advancements, recipes (shaped/shapeless/
/// smithing/stonecutting), loot tables, predicates, item modifiers, tags,
/// dialogs, and enchantments. Functions returning one of these types and
/// annotated `#[component]` (e.g. `examples/book_project`'s
/// `trailhead_dialog()`, which returns `Dialog`) are exported as generated
/// JSON resources. Most individual builder types (`Advancement`,
/// `LootTable`, `Dialog`, …) are already re-exported from the [`prelude`].
pub use sand_core::components as component;

/// Typed conditions (`Condition`) and `execute`-plan composition helpers.
/// [`condition::Condition`] is what `all!`/`any!` build and what
/// `TypedExecute::when(...)` accepts; see [`execute_when`] for the
/// `if_`/`unless`/`when` grouped-branch API used to express `if/else`-style
/// command logic (see `trail:claim_striders` in `examples/book_project` for
/// a worked example of `if_(...).then_all(...).else_all(...)`).
pub use sand_core::condition;

/// Grouped-branch `execute` composition: `if_(condition)`, `unless(condition)`,
/// and `when(condition)`, each returning a builder with `.then_all(...)`
/// (and, for `if_`, `.else_all(...)`) that accepts command lists built with
/// `mcfunction!`. Use this instead of hand-writing parallel `execute if`/
/// `execute unless` command pairs.
pub use sand_core::execute_when;

/// Typed resource references: `FunctionRef`, `AdvancementRef`, `DialogRef`,
/// `LootTableRef`, `PredicateRef`, and similar `namespace:path`-validated
/// handles used where a command needs to point at another generated
/// resource (e.g. `cmd::show_dialog(selector, DialogRef::local("trailhead"))`).
pub use sand_core::resource_ref;

/// The Minecraft version model: `MinecraftVersion`, `VersionProfile`, and the
/// capability-gating machinery that determines which command/component
/// syntax a given target version supports. Most authors only interact with
/// this indirectly (via `sand.toml`'s `mc_version` and the generated
/// `sand_export` binary); reach for it directly when writing version-aware
/// logic, e.g. checking `resolve_export_caps` output in a custom export hook
/// (see `__sand_export` in `examples/book_project`).
pub use sand_core::version;

/// Particle/sound VFX sequencing: `Vfx`, `VfxParticle`, `VfxSound`, and the
/// `VfxStep` trait used to build a reusable, composable effect
/// (`Vfx::new(name).particle(...).sound(...)`) that emits its commands with
/// `.play_at(selector)`.
pub use sand_core::vfx;

/// Optional, feature-gated gameplay systems (`systems-damage`,
/// `systems-cooldowns`, `systems-lifecycle`, `systems-player-data`,
/// `systems-movement`, `systems-inventory`, `systems-entities`) providing
/// higher-level building blocks — e.g. `DamageTracker`/`DamageThreshold`
/// behind `systems-damage` — on top of the core state/event primitives.
/// Each submodule only compiles when its Cargo feature is enabled; forward
/// the relevant `sand/systems-*` feature from your project's `Cargo.toml`.
pub use sand_core::systems;

/// A validated `namespace:path` resource identifier, used throughout Sand
/// anywhere a datapack resource (function, advancement, item, tag, …) is
/// referenced by name. Construction is fallible and validates both segments
/// at call time: `ResourceLocation::new("trail", "grapple/execute").unwrap()`.
pub use sand_core::ResourceLocation;

/// Text components, chat colors, and click/hover events for `tellraw`,
/// titles, dialogs, and books. [`text::Text`] is the builder used everywhere
/// a chat component is needed (`Text::new("Hello").gold().bold(true)`); it
/// implements `Display`, so it renders directly to the JSON text component
/// Minecraft expects wherever a command takes one.
pub mod text {
    pub use sand_core::prelude::{
        ChatColor, ClickEvent, EntityHoverId, HoverEvent, IntoTextEntityType, Text, TextComponent,
    };
}

/// Storage/NBT data authoring: SNBT values (`SnbtValue`, `SnbtCompound`),
/// command-storage locations (`StorageLocation`, `NbtLocation`, `NbtPath`),
/// and typed storage schemas (`StorageSchema`, `StorageField`,
/// `StorageVar` — also available from [`state`] since storage-backed values
/// are one kind of state). Use this module when working with NBT/storage
/// data directly rather than through a typed state wrapper.
pub mod data {
    pub use sand_core::state::{
        BlockNbt, EntityNbt, NbtLocation, NbtPath, SnbtCompound, SnbtValue, StorageField,
        StorageLocation, StorageSchema, StorageVar,
    };
}

/// Supported low-level hooks for framework integrators: export entry points
/// (e.g. `try_export_components_json_for_version`, used by the generated
/// `__sand_export` binary hook — see `examples/book_project`'s
/// `__sand_export` function) and raw escape hatches for emitting JSON/SNBT
/// Sand doesn't yet model with a typed builder. Ordinary datapack authors
/// should not need this module; it exists for `sand-cli`/`sand-build` and
/// for advanced integrations that need to drive the export pipeline
/// themselves.
pub use sand_core::advanced;

/// Resource-pack authoring (HUD bars/elements, textures), re-exporting the
/// `sand-resourcepack` crate. Only available with the `resourcepack`
/// feature; pair with the [`hud_bar!`](crate::hud_bar),
/// [`hud_element!`](crate::hud_element), and [`texture!`](crate::texture)
/// macros, also feature-gated.
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
