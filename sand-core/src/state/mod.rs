//! Typed datapack state variables.
//!
//! Provides high-level wrappers around scoreboard objectives so users do not
//! need to know scoreboard command syntax for common patterns.
//!
//! # Types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`ScoreVar<T>`] | Integer variable backed by a scoreboard objective |
//! | [`Flag`] | Boolean flag (score = 0 or 1) |
//! | [`Timer`] | Countdown timer |
//! | [`Cooldown`] | Ability cooldown with ready/active conditions |
//! | [`Ticks`] | Tick-based duration |
//!
//! # Example — manual `.define()`
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//! static CASTING: Flag = Flag::new("casting");
//! static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
//!
//! fn load() -> Vec<String> {
//!     vec![MANA.define(), CASTING.define(), DASH.define()]
//! }
//!
//! fn tick() -> Vec<String> {
//!     vec![DASH.tick_all_players()]
//! }
//! ```
//!
//! # Example — automatic lifecycle
//! ```rust,ignore
//! sand_core::sand_state! {
//!     static MANA: ScoreVar<i32> = ScoreVar::new("mana") =>
//!         MANA.lifecycle().default(100);
//!     static CASTING: Flag = Flag::new("casting") =>
//!         CASTING.lifecycle().default(0);
//!     static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60)) =>
//!         DASH.lifecycle().default(0).auto_tick();
//! }
//! ```

pub mod cooldown;
pub mod flag;
pub mod registry;
pub mod score;
pub mod storage;
pub mod timer;
pub mod typed_state;

pub use cooldown::Cooldown;
pub use flag::{Flag, FlagRef};
pub use registry::{
    StateDescriptor, StateLifecycle, define_registered_state, drain_load_commands,
    drain_tick_commands, register_load_objective, register_tick_handler,
};
pub use score::{ScoreConst, ScoreConstants, ScoreExpr, ScoreOperation, ScoreRef, ScoreVar};
pub use storage::{
    BlockNbt, EntityNbt, NbtLocation, NbtPath, SnbtCompound, SnbtValue, StorageField,
    StorageLocation, StorageSchema, StorageVar,
};
pub use timer::{Ticks, Timer};
pub use typed_state::{GameState, GameStateRef, TypedGameState};
