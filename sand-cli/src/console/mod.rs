//! Interactive Minecraft server console: process lifecycle, log
//! classification, datapack diagnostics, and rendering.
//!
//! Kept as separate, independently testable pieces:
//! - [`log_record`] parses raw lines into a prefix + message.
//! - [`classify`] is a stateless, single-line classifier.
//! - [`diagnostic`] groups multi-line records and extracts datapack diagnostics.
//! - [`render`] turns classified/grouped events into what's printed.
//! - [`process`] owns the child process and event loop tying the above together.

pub mod classify;
pub mod correlate;
pub mod dedup;
pub mod diagnostic;
pub mod health;
pub mod log_record;
pub mod phase;
pub mod process;
pub mod render;

pub use health::RunHealth;
pub use process::{RunOutcome, run_server};
pub use render::OutputMode;
