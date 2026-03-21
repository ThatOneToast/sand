//! Typed Minecraft command builders.
//!
//! Each Minecraft command (or family of commands) is represented as a Rust
//! struct or free function that serializes to the correct command string via
//! [`std::fmt::Display`]. All types implement the [`Command`] marker trait.
//!
//! String-building types are provided by [`sand_commands`] and re-exported
//! here. Sand-core-specific modules contain only datapack-level concepts.
//!
//! # Module layout
//!
//! | Source | Contents |
//! |---|---|
//! | `sand_commands` (re-exported) | All command builders: blocks, coordinates, execute, selectors, scoreboard, NBT, sound, display, inventory, particles … |
//! | [`cooldown`] | `Cooldown` — scoreboard-based ability cooldown timer |
//! | [`data`] | `Storage`, `StorageKind` — named NBT namespaces; bridges to `Objective::load_from` via `From<&Storage> for String` |
//! | [`fn_macros`] | `macro_var`, `macro_line`, `function_with` — function macro utilities |
//!
//! # Example
//! ```rust,ignore
//! use sand_core::cmd::{self, Execute, Selector};
//!
//! mcfunction! {
//!     cmd::give(Selector::all_players(), "diamond_sword").count(1);
//!     cmd::kill(Selector::all_entities().tag("enemy"));
//!     Execute::new()
//!         .as_(Selector::all_players())
//!         .if_score_matches("@s", "playtime", "100..")
//!         .run(cmd::say("100 ticks!"));
//! }
//! ```

// ── Internal modules (sand-core-specific) ─────────────────────────────────────

mod cooldown;
mod data;
mod fn_macros;

// ── Re-exports from sand-commands ─────────────────────────────────────────────

/// The `Build` trait: every command builder implements `build(&self) -> String`.
pub use sand_commands::Build;

// Block placement
pub use sand_commands::{
    BlockState, CloneBlocks, CloneMaskMode, CloneMode, Fill, FillMode, SetBlock, SetBlockMode,
};
// Coordinate types
pub use sand_commands::{BlockPos, Coord, Rotation, Vec2, Vec3};
// Player display commands
pub use sand_commands::{Actionbar, Bossbar, BossbarColor, BossbarStyle, Title};
// Execute builder
pub use sand_commands::Execute;
// Execute argument types
pub use sand_commands::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
// Inventory manipulation
pub use sand_commands::{Inventory, InventorySlot, SlotPattern};
// Particle effects
pub use sand_commands::{Particle, ParticleBuilder, ParticleEffect, ParticleSpread};
// Entity/player targeting
pub use sand_commands::{GameMode, Selector, SortOrder, TargetBase};
// Sound
pub use sand_commands::{Sound, SoundSource};
// Text components
pub use sand_commands::{ChatColor, TextComponent};
// NBT types — owned by sand-commands
pub use sand_commands::{DataModify, DataTarget, NbtValue, data_modify};
// Scoreboard types — owned by sand-commands
// Note: &Storage satisfies Objective::load_from's `impl Into<String>` parameter
// via the `From<&Storage> for String` impl in mod data.
pub use sand_commands::{
    DisplaySlot, Objective, ScoreCmp, ScoreHolder, ScoreOp, ScoreboardPlayersOperation,
    scoreboard_players_operation,
};
// NOTE: sand_commands::builtins::* is intentionally NOT re-exported here because
// sand-core provides its own generated command builders (see _generated below)
// that would conflict. Use sand_commands directly for the free-function builders.

// ── Re-exports from internal modules ─────────────────────────────────────────

pub use cooldown::Cooldown;
// Storage and StorageKind are datapack concepts defined only in sand-core.
// All other NBT/scoreboard types come from sand-commands above.
pub use data::{Storage, StorageKind};
pub use fn_macros::{function_with, macro_line, macro_var};

/// A typed Minecraft command that can be serialized to a command string.
///
/// All command builders generated from the Minecraft command tree implement
/// this trait. It is also implemented by [`Execute`] for chaining.
///
/// Since [`Command`] requires [`std::fmt::Display`], you can use command
/// builders directly in [`crate::mcfunction!`]:
/// ```rust,ignore
/// mcfunction! {
///     cmd::kill(Selector::all_entities().tag("mob"));
///     "raw fallback command string";
/// }
/// ```
pub trait Command: std::fmt::Display {}

// Include the generated command builders from commands.json.
#[allow(warnings, clippy::all)]
mod _generated {
    use super::*;
    use crate::ResourceLocation;
    include!(concat!(env!("OUT_DIR"), "/commands.rs"));
}
#[allow(unused)]
pub use _generated::*;
