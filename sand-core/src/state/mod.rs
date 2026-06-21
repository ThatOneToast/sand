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
//! # Example
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
//!     vec![DASH.tick("@a")]
//! }
//! ```

pub mod cooldown;
pub mod flag;
pub mod score;
pub mod storage;
pub mod timer;

pub use cooldown::Cooldown;
pub use flag::{Flag, FlagRef};
pub use score::{ScoreRef, ScoreVar};
pub use storage::{NbtPath, StorageVar};
pub use timer::{Ticks, Timer};
