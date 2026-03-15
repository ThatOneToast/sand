//! Typed Minecraft command builders.
//!
//! Each Minecraft command is represented as a Rust struct that can be chained
//! with builder methods and serializes to the correct command string via
//! [`std::fmt::Display`].
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

mod blocks;
mod cooldown;
mod coord;
mod data;
mod display;
mod execute;
mod inventory;
mod macros;
mod objective;
mod particles;
mod selector;
mod sound;
mod types;

pub use blocks::{
    BlockState, CloneBlocks, CloneMaskMode, CloneMode, Fill, FillMode, SetBlock, SetBlockMode,
};
pub use cooldown::Cooldown;
pub use coord::{BlockPos, Coord, Rotation, Vec2, Vec3};
pub use data::{DataModify, DataTarget, NbtValue, Storage, StorageKind, data_modify};
pub use display::{Actionbar, Bossbar, BossbarColor, BossbarStyle, Title};
pub use execute::Execute;
pub use inventory::{Inventory, InventorySlot, SlotPattern};
pub use macros::{function_with, macro_line, macro_var};
pub use objective::Objective;
pub use particles::{ParticleEffect, ParticleSpread};
pub use selector::{Selector, SortOrder};
pub use sound::{Sound, SoundSource};
pub use types::{Anchor, ChatColor, GameMode, ScoreHolder, ScoreOp, Swizzle, TextComponent};

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
