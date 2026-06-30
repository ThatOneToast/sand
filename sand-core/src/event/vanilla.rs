//! Built-in vanilla Sand event types with concise names.
//!
//! These are the preferred names for use in new code. The longer `*Event` suffixed
//! names in [`crate::events`] are kept as deprecated aliases.
//!
//! # Re-exports
//!
//! This module re-exports the event marker structs from [`crate::events`] under
//! shorter, suffix-free names.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::event::vanilla::OnJoin;
//! use sand_core::event::Event;
//! use sand_macros::event;
//!
//! #[event]
//! pub fn on_join(event: Event<OnJoin>) {
//!     cmd::tellraw(event.player(), Text::new("Welcome!").gold());
//! }
//! ```

// ── Session ───────────────────────────────────────────────────────────────────

/// Fires on the first tick after server start, `/reload`, or new player mid-session.
///
/// **Vanilla limitation:** mid-session reconnect does not re-fire (scoreboard
/// persisted). True per-login detection requires a mod or plugin.
pub use crate::events::OnJoinEvent as OnJoin;

/// Fires the very first time a player ever joins. Never fires again.
pub use crate::events::FirstJoinEvent as FirstJoin;

/// Fires on the tick a player dies (any cause).
pub use crate::events::OnDeathEvent as OnDeath;

/// Fires on the tick after a player respawns from death.
pub use crate::events::OnRespawnEvent as OnRespawn;

// ── Kill / combat ─────────────────────────────────────────────────────────────

/// Fires when the player kills any entity.
pub use crate::events::EntityKillEvent as EntityKill;

/// Fires when any entity kills the player.
pub use crate::events::PlayerKillEvent as PlayerKill;

/// Fires when the player deals damage to any entity.
pub use crate::events::PlayerDamageEntityEvent as PlayerDamagesEntity;

/// Fires when any entity deals damage to the player.
pub use crate::events::EntityDamagePlayerEvent as EntityDamagesPlayer;

// ── Items ─────────────────────────────────────────────────────────────────────

/// Fires when the player consumes any item (food, potion, etc.).
pub use crate::events::ItemConsumeEvent as AnyItemConsumed;

/// Fires when the player crafts any item.
pub use crate::events::ItemCraftEvent as AnyItemCrafted;

/// Fires when the player enchants any item.
pub use crate::events::ItemEnchantEvent as AnyItemEnchanted;

/// Fires when the player shoots a crossbow.
pub use crate::events::ShotCrossbowEvent as CrossbowShot;

// ── World ─────────────────────────────────────────────────────────────────────

/// Fires when the player places any block.
pub use crate::events::BlockPlaceEvent as AnyBlockPlaced;

/// Fires when the player changes dimension.
pub use crate::events::ChangeDimensionEvent as DimensionChanged;

/// Fires when a player breeds animals.
pub use crate::events::BreedAnimalsEvent as AnimalsBreed;

/// Fires when the player tames an animal.
pub use crate::events::TameAnimalEvent as AnimalTamed;

/// Fires when the player summons an entity.
pub use crate::events::SummonEntityEvent as EntitySummoned;

/// Fires when a player's XP level increases (tick-backed; no advancement).
///
/// Vanilla Minecraft has no `minecraft:leveled_up` advancement trigger.
/// Sand implements this event via a generated scoreboard/tick system.
/// See [`crate::events::PlayerLevelUpEvent`] for full documentation.
pub use crate::events::PlayerLevelUpEvent as PlayerLevelsUp;

/// Fires when the player brews a potion.
pub use crate::events::BrewPotionEvent as PotionBrewed;

// ── Tick-poll (continuous) ───────────────────────────────────────────────────

/// Fires every tick the player is sneaking.
pub use crate::events::PlayerSneakEvent as PlayerSneaking;

/// Fires every tick the player is sprinting.
pub use crate::events::PlayerSprintEvent as PlayerSprinting;

/// Fires every tick the player is swimming.
pub use crate::events::PlayerSwimmingEvent as PlayerSwimming;

/// Fires every tick the player is on fire.
pub use crate::events::PlayerOnFireEvent as PlayerOnFire;
