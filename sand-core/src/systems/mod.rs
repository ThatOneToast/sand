//! Optional built-in datapack systems, enabled via Cargo features.
//!
//! Each system is opt-in. Enable only what your pack needs:
//!
//! ```toml
//! [dependencies]
//! sand-core = { version = "...", features = ["systems-damage"] }
//! ```
//!
//! # Available features
//!
//! | Feature | Description |
//! |---|---|
//! | `systems-damage` | Damage tracking via cumulative scoreboard stats |
//! | `systems-cooldowns` | Auto-tick all registered cooldowns |
//! | `systems-lifecycle` | Manual join/death/respawn command-fragment helpers |
//! | `systems-player-data` | Manual `PlayerSchema` builder helpers (implies `systems-lifecycle`) |
//! | `systems-movement` | Typed push, launch, speed boost, and slow helpers |
//! | `systems-inventory` | Typed inventory has/replace/clear/give helpers |
//! | `systems-entities` | Typed interactable entity builder |
//! | `systems-all` | Enable all of the above |

#[cfg(feature = "systems-damage")]
pub mod damage;

#[cfg(feature = "systems-cooldowns")]
pub mod cooldowns;

#[cfg(feature = "systems-lifecycle")]
pub mod lifecycle;

#[cfg(feature = "systems-player-data")]
pub mod player_data;

#[cfg(feature = "systems-movement")]
pub mod movement;

#[cfg(feature = "systems-inventory")]
pub mod inventory;

#[cfg(feature = "systems-entities")]
pub mod entities;
