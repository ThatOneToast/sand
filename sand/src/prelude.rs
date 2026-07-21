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
//! - **Macros** — the attribute macros (`#[function]`, `#[component]`,
//!   `#[event]`, `#[item]`, `#[armor_event]`, `#[schedule]`) and declarative
//!   macros (`all!`, `any!`, `mcfunction!`, `sand_state!`, `run_fn!`) that
//!   drive datapack authoring.
//! - **Commands** — the [`crate::cmd`] module (as `cmd::...`) plus typed
//!   command builders: `Execute`/`TypedExecute`, `Selector` and target types,
//!   `Actionbar`, `Bossbar`, `Title`, particles, and `Damage`.
//! - **Conditions** — [`Condition`], [`ExecutePlan`], and the grouped-branch
//!   helpers `if_`/`unless`/`when` from [`crate::execute_when`].
//! - **State** — `ScoreVar`, `Flag`, `Timer`, `Cooldown`, `GameState`/
//!   `TypedGameState`, and storage types (`StorageVar`, `StorageSchema`,
//!   `SnbtValue`, …).
//! - **Entities** — typed entity/player query builders and execution-scoped
//!   contexts from [`crate::entity`].
//! - **Events** — the typed event model: `Event`, `AdvancementEvent`,
//!   trigger builders (`InventoryChangedTrigger`, …), and `EventHandle`.
//! - **Components** — item/advancement/recipe/loot-table/dialog builders
//!   from [`mod@crate::component`] (advancements, recipes, loot tables,
//!   predicates, dialogs, tags, item components) plus raw escape hatches
//!   (`RawComponent`, `RawJson`, `RawSnbt`) and typed registry identifiers
//!   (`ItemId`, `EntityTypeId`, `EffectId`, …).
//! - **Text** — `Text`, `TextComponent`, `ChatColor`, click/hover events.
//! - **Resource refs** — `ResourceLocation` and typed refs (`FunctionRef`,
//!   `DialogRef`, `AdvancementRef`, `LootTableRef`, `PredicateRef`).
//! - **Version** — `MinecraftVersion`, `VersionProfile`.
//! - **Vanilla** — the [`crate::vanilla`] module path (not its individual
//!   variants) is brought into scope, so `vanilla::Item::Diamond` /
//!   `vanilla::Block::WhiteWool` / `vanilla::EntityType::Marker` work
//!   directly after `use sand::prelude::*;`.
//! - **Optional systems** — gameplay building blocks gated behind their
//!   Cargo feature (e.g. `DamageTracker`/`DamageThreshold` behind
//!   `systems-damage`, `PlayerDataSchema` behind `systems-player-data`); see
//!   [`crate::systems`].
//!
//! Anything not listed above — VFX ([`crate::vfx`]), the event dispatch
//! graph ([`crate::events`]), storage/NBT modeling details ([`crate::data`]),
//! and low-level export hooks ([`crate::advanced`]) — stays in its topic
//! module; import it explicitly when you need it (e.g. `use sand::vfx::Vfx;`
//! or `use sand::event::vanilla::OnDeath;`).

// Attribute + declarative macros.
pub use crate::{
    SandStorage, all, any, armor_event, component, event, function, item, mcfunction, run_fn,
    sand_state, schedule,
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
