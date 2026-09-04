//! The default authoring import.
//!
//! ```rust,ignore
//! use sand::prelude::*;
//! ```
//!
//! Nearly every Sand datapack file starts with this glob import; it is the
//! recommended default unless you have a specific reason to import narrower
//! topic modules instead (e.g. avoiding a name collision).
//!
//! # What it exports
//!
//! - **Macros** — the attribute macros (`#[function]`, `#[datapack_component]`,
//!   `#[on_event]`, `#[custom_item]`, `#[armor_event]`, `#[schedule]`) and declarative
//!   expression macros (`all!`, `any!`, `mcfunction!`, `run_fn!`) that
//!   drive datapack authoring.
//! - **Commands** — the [`crate::cmd`] module (as `cmd::...`) plus typed
//!   command builders: `Execute`/`TypedExecute`, the canonical `Target`,
//!   `Actionbar`, `Bossbar`, `Title`, particles, and `Damage`.
//! - **Conditions** — [`Condition`] and the grouped-branch
//!   helpers `if_`/`unless`/`when` from [`crate::execute_when`].
//! - **State** — `ScoreVar`, `Flag`, `Timer`, `Cooldown`, `GameState`/
//!   `TypedGameState`, and storage types (`StorageVar`, `StorageSchema`,
//!   `SnbtValue`, …).
//! - **Entities** — target execution capabilities and execution-scoped
//!   contexts from [`crate::entity`].
//! - **Events** — the typed event model: `Event`, `AdvancementEvent`,
//!   trigger builders (`InventoryChangedTrigger`, …), and `EventHandle`;
//!   plus, for custom event authoring (#273), bare-marker `SandEvent`/
//!   `SandEventDispatch` and the `SandEventParticipants` extension trait
//!   that gives bare `SandEvent` handlers the same `.entity`/`.attacker`/
//!   `.killer`/`.victim`/`.weapon` accessor sugar `Event<E>` has; and
//!   `EventParticipantPlan`/`ParticipantBuilder` for declaring what a plan
//!   observes or inherits.
//! - **Components** — item/advancement/recipe/loot-table/dialog/chat-type
//!   builders from [`mod@crate::component`] (advancements, recipes, loot
//!   tables, predicates, dialogs, chat types, tags, item components) plus
//!   raw escape hatches (`RawComponent`, `RawJson`, `RawSnbt`) and typed
//!   registry identifiers
//!   (`ItemId`, `EntityTypeId`, `EffectId`, …).
//! - **Text** — `Text`, `TextComponent`, `ChatColor`, click/hover events.
//! - **Resource refs** — `ResourceLocation` and typed refs (`FunctionId`,
//!   `DialogId`, `AdvancementId`, `LootTableId`, `PredicateId`).
//! - **Version** — `MinecraftVersion`, `VersionProfile`, and typed `VersionFeature` gates.
//! - **Vanilla** — the [`crate::vanilla`] module path (not its individual
//!   variants) is brought into scope, so `vanilla::Item::Diamond` /
//!   `vanilla::Block::WhiteWool` / `vanilla::EntityType::Marker` work
//!   directly after `use sand::prelude::*;`.
//! - **Optional systems** — gameplay building blocks gated behind their
//!   Cargo feature (e.g. `DamageTracker`/`DamageThreshold` behind
//!   `systems-damage`, `PlayerDataSchema`/`PlayerSchema` behind
//!   `systems-player-data`); see
//!   [`crate::systems`].
//!
//! Anything not listed above — VFX ([`crate::vfx`]), the event dispatch
//! *graph* internals ([`crate::events`]'s `EventGraph` and chain/compose
//! builder plumbing — as opposed to `SandEvent`/`SandEventDispatch`
//! themselves, which are in the prelude), storage/NBT modeling details
//! ([`crate::data`]), and low-level export hooks ([`crate::advanced`]) —
//! stays in its topic module; import it explicitly when you need it (e.g.
//! `use sand::vfx::Vfx;` or `use sand::event::vanilla::OnDeath;`).

// Attribute + declarative macros.
pub use crate::{
    EntityStateEnum, SandStorage, State, StateBundle, StateEnum, StateQuery, all, any, api,
    armor_event, custom_item, datapack_component, entity_archetype, function, mcfunction, on_event,
    run_fn, schedule, state_lifecycle, system,
};

// The `cmd` module itself, so `cmd::say(...)` works from the prelude.
pub use crate::cmd;

// The `vanilla` module path itself (not a glob of its variants), so
// `vanilla::Item::Diamond` etc. work from the prelude without flattening
// thousands of generated variants into it.
pub use crate::vanilla;

// The curated implementation prelude (commands, selectors, conditions, state,
// entities, events, components, dialogs, text, resource refs, raw escape
// hatches). Compiler-facing symbols are excluded at the source.
pub use sand_core::prelude::*;
