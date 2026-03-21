//! Typed Minecraft command builders.
//!
//! Every command type implements [`Build`], which provides:
//! - `build(&self) -> String` — the canonical serialization method
//! - `Clone` — required supertrait, derived on every type
//! - `ToString` — automatic via `Display`, which each type also implements
//! - `Into<String>` — each concrete type has its own `impl From<T> for String`
//!   delegating to `build()`; this is NOT a blanket supertrait to avoid orphan-rule
//!   violations when implementing `Build for &T`
//! - `&T` support — `impl Build for &T where T: Build` is provided below

pub mod blocks;
pub mod builtins;
pub mod coord;
pub mod display;
pub mod execute;
pub mod execute_args;
pub mod inventory;
pub mod nbt;
pub mod particles;
pub mod scoreboard;
pub mod selector;
pub mod sound;
pub mod text;

pub use blocks::{
    BlockState, CloneBlocks, CloneMaskMode, CloneMode, Fill, FillMode, SetBlock, SetBlockMode,
};
pub use builtins::*;
pub use coord::{BlockPos, Coord, Rotation, Vec2, Vec3};
pub use display::{Actionbar, Bossbar, BossbarColor, BossbarStyle, Title};
pub use execute::Execute;
pub use execute_args::{Anchor, ItemSlot, NbtStoreKind, Swizzle};
pub use inventory::{Inventory, InventorySlot, SlotPattern};
pub use nbt::{DataModify, DataTarget, NbtValue, data_modify};
pub use particles::{Particle, ParticleBuilder, ParticleEffect, ParticleSpread};
pub use scoreboard::{
    DisplaySlot, Objective, ScoreCmp, ScoreHolder, ScoreOp, ScoreboardPlayersOperation,
    scoreboard_players_operation,
};
pub use selector::{GameMode, Selector, SortOrder, TargetBase};
pub use sound::{Sound, SoundSource};
pub use text::{ChatColor, TextComponent};

// ── Build trait ───────────────────────────────────────────────────────────────

/// The core trait for all Minecraft command builders.
///
/// Every type that represents a Minecraft command or produces a command string
/// implements this trait. Implementors are required to satisfy:
///
/// - [`Clone`] — required supertrait, derived via `#[derive(Clone)]`
///
/// # Into\<String\> — not a supertrait, but always available
///
/// Rust's orphan rules prevent a blanket `impl<T: Build> From<&T> for String`
/// (both `From` and `String` are foreign). Instead, every concrete `Build` type
/// ships its own `impl From<T> for String` that delegates to `build()`, so
/// `value.into()` works on any owned command value. For references, call
/// `.build()` directly or `.clone().into()`.
///
/// # Display and ToString
///
/// Every `Build` type additionally provides `impl Display`, which forwards to
/// `build()`. This gives `ToString` for free.
///
/// # References
///
/// `&T` automatically implements `Build` whenever `T: Build`:
///
/// ```rust,ignore
/// let cmd = Execute::new().as_(Selector::all_players()).run_raw("say hi");
/// let s = (&cmd).build(); // works via the &T impl
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use sand_commands::{Build, Execute, Selector};
///
/// let cmd = Execute::new()
///     .as_(Selector::all_players())
///     .run_raw("say hello");
///
/// // Three equivalent ways to get the String:
/// let s1 = cmd.build();
/// let s2 = cmd.to_string();
/// let s3: String = cmd.into();   // works because each type has From<T> for String
/// ```
pub trait Build: Clone {
    /// Serialize this command to its Minecraft string representation.
    fn build(&self) -> String;
}

/// `&T` implements `Build` whenever `T` does.
///
/// Allows passing references to commands wherever `build()` is needed,
/// avoiding needless clones in tight loops.
impl<T: Build> Build for &T {
    fn build(&self) -> String {
        (*self).build()
    }
}
// NOTE: a blanket `impl<T: Build> From<&T> for String` is not possible —
// both `From` and `String` are foreign types (orphan rule E0210).
// Each concrete type provides its own `impl From<T> for String` instead.
