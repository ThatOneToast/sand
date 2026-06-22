//! Built-in Sand event types and the legacy [`SandEvent`] trait for custom
//! tick-poll or compatibility events.
//!
//! New custom advancement-backed events should implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) and use
//! [`Event<T>`](crate::event::Event) as the handler parameter:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_components::ItemPredicate;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
//!     }
//! }
//!
//! #[event]
//! pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
//!     cmd::say("Golden apple eaten");
//! }
//! ```
//!
//! # Built-in events
//!
//! | Type | When it fires | Required filters |
//! |---|---|---|
//! | [`OnJoinEvent`] | First tick each join session | — |
//! | [`FirstJoinEvent`] | Very first join ever | — |
//! | [`OnDeathEvent`] | Any death (mob, fall, void, `/kill`, …) | — |
//! | [`OnRespawnEvent`] | Tick after respawning from death | — |
//! | [`ArmorEquipEvent`] | Item equipped in an equipment slot | `slot` |
//! | [`ArmorUnequipEvent`] | Item removed from an equipment slot | `slot` |
//! | [`HoldingItemEvent`] | Holding item (every tick) | `item` |
//! | [`CurrentlyWearingEvent`] | Wearing item in armor slot (every tick) | `slot`, `item` |
//!
//! # Usage
//!
//! Use the `#[event]` attribute macro from `sand_macros` on a free-standing
//! function. The primary handler parameter is `Event<T>` where `T` implements
//! [`AdvancementEvent`](crate::event::AdvancementEvent):
//!
//! ```rust,ignore
//! use sand_macros::event;
//! use sand_core::prelude::*;
//! use sand_core::events::{OnJoinEvent, OnDeathEvent, ArmorEquipEvent};
//!
//! static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
//!
//! #[event]
//! pub fn on_join(event: Event<OnJoinEvent>) {
//!     cmd::tellraw(
//!         Selector::self_(),
//!         Text::new("Welcome!").gold(),
//!     );
//! }
//!
//! #[event]
//! pub fn on_death(event: Event<OnDeathEvent>) {
//!     TOTAL_DEATHS.add(event.player(), 1);
//! }
//!
//! // Slot filter required; item is optional
//! #[event(slot = Head, item = "minecraft:diamond_helmet")]
//! pub fn equipped_diamond_helmet(event: Event<ArmorEquipEvent>) {
//!     cmd::say("Diamond helmet on!");
//! }
//! ```
//!
//! # Custom advancement events
//!
//! For custom advancement-backed events, implement
//! [`AdvancementEvent`](crate::event::AdvancementEvent) on a marker struct and
//! handle it with `Event<T>`:
//!
//! ```rust,ignore
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_core::prelude::*;
//! use sand_components::ItemPredicate;
//!
//! pub struct AteGoldenAppleEvent;
//!
//! impl AdvancementEvent for AteGoldenAppleEvent {
//!     type Trigger = ConsumeItemTrigger;
//!     fn trigger() -> Self::Trigger {
//!         ConsumeItemTrigger::new().item(ItemPredicate::id("minecraft:golden_apple"))
//!     }
//! }
//!
//! #[event]
//! pub fn on_ate_golden_apple(event: Event<AteGoldenAppleEvent>) {
//!     cmd::say("Golden apple eaten");
//! }
//! ```
//!
//! # Legacy: `SandEvent` (tick-poll and backward compatibility)
//!
//! Implement [`SandEvent`] only when you need a custom tick-condition dispatch
//! or are migrating existing code. New advancement-backed events should use
//! [`AdvancementEvent`](crate::event::AdvancementEvent) instead.
//!
//! ```rust,ignore
//! use sand_core::events::{SandEvent, SandEventDispatch};
//! use sand_core::AdvancementTrigger;
//!
//! /// Fires when a player picks up an item.
//! pub struct ItemPickupEvent;
//!
//! impl SandEvent for ItemPickupEvent {
//!     fn dispatch() -> SandEventDispatch {
//!         SandEventDispatch::AdvancementTrigger(
//!             AdvancementTrigger::PickedUpItem { item: None }
//!         )
//!     }
//! }
//!
//! #[event]
//! pub fn on_pickup(event: Event<ItemPickupEvent>) {
//!     cmd::say("Picked something up!");
//! }
//! ```

// ── Custom event API ──────────────────────────────────────────────────────────

/// How a custom [`SandEvent`] is dispatched at runtime.
///
/// Returned by [`SandEvent::dispatch`]. Sand inspects this at build time to
/// generate the correct detection mechanism (advancement JSON or tick loop).
pub enum SandEventDispatch {
    /// The event fires when the given advancement trigger criteria are met.
    ///
    /// Sand generates an advancement JSON file and wires the handler function
    /// as its reward. The advancement is revoked after firing (by default) so
    /// it can trigger again next time.
    AdvancementTrigger(crate::AdvancementTrigger),

    /// The event fires every tick an `execute if <condition>` is satisfied,
    /// evaluated as each online player.
    ///
    /// The string must be a valid Minecraft `execute if` sub-command, e.g.:
    ///
    /// - `"items entity @s mainhand minecraft:diamond_sword"` — holding a sword
    /// - `"score @s my_flag matches 1"` — scoreboard flag is set
    /// - `"predicate my_pack:some_predicate"` — custom predicate
    TickCondition(String),
}

/// Implement this trait on your own type to define a custom Sand event.
///
/// Your type is used as the phantom parameter in an `#[event]` handler
/// function. Sand inspects [`dispatch`](Self::dispatch) at build time to
/// emit the appropriate datapack files.
///
/// # Example
///
/// ```rust,ignore
/// use sand_core::events::{SandEvent, SandEventDispatch};
/// use sand_core::prelude::*;
/// use sand_core::AdvancementTrigger;
///
/// /// Fires when a player picks up any item.
/// pub struct ItemPickupEvent;
///
/// impl SandEvent for ItemPickupEvent {
///     fn dispatch() -> SandEventDispatch {
///         SandEventDispatch::AdvancementTrigger(
///             AdvancementTrigger::PickedUpItem { item: None }
///         )
///     }
/// }
///
/// #[event]
/// pub fn on_item_pickup(event: ItemPickupEvent) {
///     cmd::say("Picked something up!");
/// }
/// ```
pub trait SandEvent {
    /// Return the dispatch strategy for this event type.
    fn dispatch() -> SandEventDispatch;

    /// Whether to revoke the advancement after it fires.
    ///
    /// Defaults to `true` — the advancement is revoked immediately so it can
    /// fire again the next time the trigger is satisfied.
    ///
    /// Set to `false` for one-shot events that should fire **only once per
    /// player, ever** (e.g. first-time rewards).
    ///
    /// Only relevant when [`dispatch`](Self::dispatch) returns
    /// [`SandEventDispatch::AdvancementTrigger`].
    fn revoke() -> bool {
        true
    }
}

// ── Built-in event marker types ───────────────────────────────────────────────

/// Fires on the first tick each time a player joins the server.
///
/// The preferred short name is [`sand_core::event::vanilla::OnJoin`](crate::event::vanilla::OnJoin).
///
/// Implemented as an `Advancement + Tick` trigger that is immediately
/// revoked — so it fires **once per join session**, not every tick.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_join(event: Event<OnJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome back!").gold(),
///     );
/// }
/// ```
pub struct OnJoinEvent;

/// Fires the very first time a player ever joins. Never fires again.
///
/// The preferred short name is [`sand_core::event::vanilla::FirstJoin`](crate::event::vanilla::FirstJoin).
///
/// Implemented as an `Advancement + Tick` trigger **without** revocation.
/// Once the advancement is granted it stays, so the event fires exactly once
/// per player across all sessions.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn first_join(event: Event<FirstJoinEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("Welcome for the very first time!").aqua(),
///     );
///     cmd::give(Selector::self_(), "minecraft:diamond").count(3);
/// }
/// ```
pub struct FirstJoinEvent;

/// Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …).
///
/// The preferred short name is [`sand_core::event::vanilla::OnDeath`](crate::event::vanilla::OnDeath).
///
/// Implemented via the `deathCount` scoreboard criterion. The handler runs as
/// `@s` = the dying player.
///
/// # Example
///
/// ```rust,ignore
/// static TOTAL_DEATHS: ScoreVar<i32> = ScoreVar::new("total_deaths");
///
/// #[event]
/// pub fn on_death(event: Event<OnDeathEvent>) {
///     TOTAL_DEATHS.add(event.player(), 1);
/// }
/// ```
pub struct OnDeathEvent;

/// Fires on the tick after a player respawns from death.
///
/// The preferred short name is [`sand_core::event::vanilla::OnRespawn`](crate::event::vanilla::OnRespawn).
///
/// Sand tags each dying player with `__sand_was_dead` during the death check.
/// Each tick, any player with that tag who is no longer in spectator mode
/// (i.e. has respawned) triggers this event, then the tag is removed.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_respawn(event: Event<OnRespawnEvent>) {
///     cmd::tellraw(
///         Selector::self_(),
///         Text::new("You respawned!").green(),
///     );
/// }
/// ```
pub struct OnRespawnEvent;

/// Fires on the tick a player **equips** an item in an equipment slot.
///
/// Uses tick-based state tracking via entity tags — no advancement required.
/// Sand maintains a `__armor_<slot>` tag per player to detect transitions.
///
/// # Required filter
///
/// - `slot = Head | Chest | Legs | Feet | Offhand`
///
/// # Optional filters
///
/// - `item = "namespace:item_id"` — only trigger for this item
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component (SNBT)
///
/// # Example
///
/// ```rust,ignore
/// static MANA_REGEN: Flag = Flag::new("mana_regen");
///
/// // Any item equipped in the feet slot
/// #[event(slot = Feet)]
/// pub fn any_boots_equipped(event: Event<ArmorEquipEvent>) {
///     cmd::say("Boots equipped!");
/// }
///
/// // Specific item with custom NBT
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_equipped(event: Event<ArmorEquipEvent>) {
///     MANA_REGEN.enable(event.player());
/// }
/// ```
pub struct ArmorEquipEvent;

/// Fires on the tick a player **removes** an item from an equipment slot.
///
/// Same filter syntax as [`ArmorEquipEvent`].
///
/// # Example
///
/// ```rust,ignore
/// static MANA_REGEN: Flag = Flag::new("mana_regen");
///
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_removed(event: Event<ArmorUnequipEvent>) {
///     MANA_REGEN.disable(event.player());
/// }
/// ```
pub struct ArmorUnequipEvent;

/// Fires every tick a player is **holding** a specific item.
///
/// Uses `execute if items entity @s <slot> <item>` per tick.
///
/// # Required filter
///
/// - `item = "namespace:item_id"`
///
/// # Optional filters
///
/// - `slot = Mainhand | Offhand` (defaults to `Mainhand`)
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component
///
/// # Example
///
/// ```rust,ignore
/// static BLOCKING: Flag = Flag::new("blocking");
///
/// #[event(item = "minecraft:diamond_sword")]
/// pub fn holding_diamond_sword(event: Event<HoldingItemEvent>) {
///     cmd::particle(Particle::Crit, event.player());
/// }
///
/// #[event(item = "minecraft:shield", slot = Offhand)]
/// pub fn holding_shield_offhand(event: Event<HoldingItemEvent>) {
///     BLOCKING.enable(event.player());
/// }
/// ```
pub struct HoldingItemEvent;

/// Fires every tick a player is **wearing** a specific item in an armor slot.
///
/// Uses `execute if items entity @s armor.<slot> <item>` per tick.
///
/// # Required filters
///
/// - `slot = Head | Chest | Legs | Feet`
/// - `item = "namespace:item_id"`
///
/// # Optional filters
///
/// - `custom_data = "{key:1b}"` — match `minecraft:custom_data` component
///
/// # Example
///
/// ```rust,ignore
/// #[event(slot = Head, item = "minecraft:diamond_helmet")]
/// pub fn wearing_diamond_helmet(event: Event<CurrentlyWearingEvent>) {
///     cmd::particle(Particle::Enchant, event.player());
/// }
/// ```
pub struct CurrentlyWearingEvent;

// ════════════════════════════════════════════════════════════════════════════
// ── Comprehensive built-in event library ────────────────────────────────────
// ════════════════════════════════════════════════════════════════════════════
//
// All events below implement [`SandEvent`] and can be used directly with
// `#[event]`. Most map 1:1 to a Minecraft advancement trigger so they fire
// once per trigger occurrence and revoke themselves (unless noted).
// For filter-level customisation (e.g. specific item/entity), implement your
// own type with [`SandEvent`] using the same trigger and supply conditions.

// ── Kill / combat ─────────────────────────────────────────────────────────

/// Fires when the player kills any entity.
///
/// Maps to `minecraft:player_killed_entity` with no conditions.
/// For entity-type filters, use a custom [`SandEvent`] with the
/// [`crate::AdvancementTrigger::PlayerKilledEntity`] trigger.
///
/// # Example
/// ```rust,ignore
/// static TOTAL_KILLS: ScoreVar<i32> = ScoreVar::new("total_kills");
///
/// #[event]
/// pub fn on_kill(event: Event<EntityKillEvent>) {
///     TOTAL_KILLS.add(event.player(), 1);
/// }
/// ```
pub struct EntityKillEvent;
impl SandEvent for EntityKillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::PlayerKilledEntity {
            entity: None,
            killing_blow: None,
        })
    }
}

/// Fires when any entity kills the player.
///
/// Maps to `minecraft:entity_killed_player` with no conditions.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn on_killed(event: Event<PlayerKillEvent>) {
///     cmd::tellraw(
///         event.player(),
///         Text::new("You were slain!").red(),
///     );
/// }
/// ```
pub struct PlayerKillEvent;
impl SandEvent for PlayerKillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EntityKilledPlayer {
            entity: None,
            killing_blow: None,
        })
    }
}

/// Fires when the player deals damage to any entity.
///
/// Maps to `minecraft:player_hurt_entity`.
pub struct PlayerDamageEntityEvent;
impl SandEvent for PlayerDamageEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: None,
        })
    }
}

/// Fires when any entity deals damage to the player.
///
/// Maps to `minecraft:entity_hurt_player`.
pub struct EntityDamagePlayerEvent;
impl SandEvent for EntityDamagePlayerEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        })
    }
}

/// Fires when the player shoots a crossbow.
pub struct ShotCrossbowEvent;
impl SandEvent for ShotCrossbowEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ShotCrossbow {
            item: None,
        })
    }
}

/// Fires when the player channels a trident's lightning at an entity.
pub struct ChanneledLightningEvent;
impl SandEvent for ChanneledLightningEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ChanneledLightning {
            victims: None,
        })
    }
}

// ── Items ─────────────────────────────────────────────────────────────────

/// Fires when the player consumes any item (food, potion, etc.).
///
/// Maps to `minecraft:consume_item`.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn on_eat(event: Event<ItemConsumeEvent>) {
///     cmd::say("Yum!");
/// }
/// ```
pub struct ItemConsumeEvent;
impl SandEvent for ItemConsumeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ConsumeItem { item: None })
    }
}

/// Fires when the player crafts any item.
///
/// Maps to `minecraft:crafted_item`.
pub struct ItemCraftEvent;
impl SandEvent for ItemCraftEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::CraftedItem { item: None })
    }
}

/// Fires when the player enchants any item.
///
/// Maps to `minecraft:enchanted_item`.
pub struct ItemEnchantEvent;
impl SandEvent for ItemEnchantEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EnchantedItem {
            item: None,
            levels: None,
        })
    }
}

/// Fires when the player fills any bucket.
///
/// Maps to `minecraft:filled_bucket`.
pub struct BucketFillEvent;
impl SandEvent for BucketFillEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FilledBucket {
            item: None,
        })
    }
}

/// Fires when the player empties any bucket. (Added in MC 1.17.)
///
/// Maps to `minecraft:emptied_bucket`.
pub struct BucketEmptyEvent;
impl SandEvent for BucketEmptyEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EmptiedBucket {
            item: None,
            location: None,
        })
    }
}

/// Fires when the player uses a fishing rod and it hooks something.
///
/// Maps to `minecraft:fishing_rod_hooked`.
pub struct FishingEvent;
impl SandEvent for FishingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FishingRodHooked {
            rod: None,
            entity: None,
            item: None,
        })
    }
}

/// Fires when a thrown item is picked up by any entity.
///
/// Maps to `minecraft:thrown_item_picked_up`.
pub struct ItemPickedUpEvent;
impl SandEvent for ItemPickedUpEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ThrownItemPickedUp {
            item: None,
            entity: None,
        })
    }
}

/// Fires when an item in the player's inventory loses durability.
///
/// Maps to `minecraft:item_durability_changed`.
pub struct ItemDurabilityChangeEvent;
impl SandEvent for ItemDurabilityChangeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ItemDurabilityChanged {
            item: None,
            delta: None,
            durability: None,
        })
    }
}

/// Fires when the player brews a potion.
///
/// Maps to `minecraft:brewed_potion`.
pub struct BrewPotionEvent;
impl SandEvent for BrewPotionEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::BrewedPotion {
            potion: None,
        })
    }
}

/// Fires when the player activates a totem of undying.
///
/// Maps to `minecraft:used_totem`.
pub struct TotemActivateEvent;
impl SandEvent for TotemActivateEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::UsedTotem { item: None })
    }
}

/// Fires when the player unlocks a recipe.
///
/// Maps to `minecraft:recipe_unlocked` with no recipe filter.
pub struct RecipeUnlockEvent;
impl SandEvent for RecipeUnlockEvent {
    fn dispatch() -> SandEventDispatch {
        // Use Custom because RecipeUnlocked requires a specific recipe string;
        // the no-filter version just fires for any recipe unlock.
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::Custom {
            trigger: "minecraft:recipe_unlocked".into(),
            conditions: None,
        })
    }
}

// ── World / blocks ────────────────────────────────────────────────────────

/// Fires when the player places any block.
///
/// Maps to `minecraft:placed_block` with no filters.
///
/// # Example
/// ```rust,ignore
/// static BLOCKS_PLACED: ScoreVar<i32> = ScoreVar::new("blocks_placed");
///
/// #[event]
/// pub fn on_place(event: Event<BlockPlaceEvent>) {
///     BLOCKS_PLACED.add(event.player(), 1);
/// }
/// ```
pub struct BlockPlaceEvent;
impl SandEvent for BlockPlaceEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::PlacedBlock {
            block: None,
            item: None,
            location: None,
            state: None,
        })
    }
}

/// Fires when the player enters a block (e.g. water, honey).
///
/// Maps to `minecraft:enter_block` with no block filter.
pub struct EnterBlockEvent;
impl SandEvent for EnterBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EnterBlock {
            block: None,
            state: None,
        })
    }
}

/// Fires when the player slides down a block (e.g. honey block wall).
///
/// Maps to `minecraft:slide_down_block`.
pub struct SlideDownBlockEvent;
impl SandEvent for SlideDownBlockEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::SlideDownBlock {
            block: None,
        })
    }
}

/// Fires when a target block is hit by a projectile near the player.
///
/// Maps to `minecraft:target_hit`.
pub struct TargetHitEvent;
impl SandEvent for TargetHitEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::TargetHit {
            signal_strength: None,
            projectile: None,
        })
    }
}

/// Fires when the player destroys a bee nest or beehive.
///
/// Maps to `minecraft:bee_nest_destroyed`.
pub struct BeeNestDestroyedEvent;
impl SandEvent for BeeNestDestroyedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::BeeNestDestroyed {
            block: None,
            item: None,
            num_bees_inside: None,
        })
    }
}

// ── Player state ──────────────────────────────────────────────────────────

/// Fires when the player changes dimension (e.g. entering the Nether or End).
///
/// Maps to `minecraft:changed_dimension`.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn on_change_dim(event: Event<ChangeDimensionEvent>) {
///     cmd::say("Dimension change!");
/// }
/// ```
pub struct ChangeDimensionEvent;
impl SandEvent for ChangeDimensionEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ChangedDimension {
            from: None,
            to: None,
        })
    }
}

/// Fires when the player sleeps in a bed.
///
/// Maps to `minecraft:slept_in_bed`.
pub struct PlayerSleepEvent;
impl SandEvent for PlayerSleepEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::SleptInBed {
            location: None,
        })
    }
}

/// Fires when the player falls from a height and lands.
///
/// Maps to `minecraft:fall_from_height`.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn on_fall(event: Event<FallFromHeightEvent>) {
///     cmd::playsound(
///         ResourceLocation::new("minecraft", "entity.player.hurt").unwrap(),
///         event.player(),
///     );
/// }
/// ```
pub struct FallFromHeightEvent;
impl SandEvent for FallFromHeightEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::FallFromHeight {
            distance: None,
            start_position: None,
        })
    }
}

/// Fires when the player levels up.
///
/// Maps to `minecraft:leveled_up`.
pub struct PlayerLevelUpEvent;
impl SandEvent for PlayerLevelUpEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::LeveledUp { level: None })
    }
}

/// Fires when the player's status effects change.
///
/// Maps to `minecraft:effects_changed`.
pub struct EffectsChangedEvent;
impl SandEvent for EffectsChangedEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::EffectsChanged {
            effects: None,
            source: None,
        })
    }
}

/// Fires when the player starts riding an entity (horse, boat, etc.).
///
/// Maps to `minecraft:started_riding`.
pub struct StartRidingEvent;
impl SandEvent for StartRidingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::StartedRiding)
    }
}

/// Fires when the player uses an ender eye (to locate a stronghold).
///
/// Maps to `minecraft:used_ender_eye`.
pub struct UseEnderEyeEvent;
impl SandEvent for UseEnderEyeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::UsedEnderEye {
            distance: None,
        })
    }
}

/// Fires when the player tames an animal.
///
/// Maps to `minecraft:tame_animal`.
pub struct TameAnimalEvent;
impl SandEvent for TameAnimalEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::TamedAnimal {
            entity: None,
        })
    }
}

/// Fires when the player breeds two animals.
///
/// Maps to `minecraft:bred_animals`.
pub struct BreedAnimalsEvent;
impl SandEvent for BreedAnimalsEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::BredAnimals {
            parent: None,
            partner: None,
            child: None,
        })
    }
}

/// Fires when the player summons an entity (e.g. Iron Golem, Snow Golem, Wither).
///
/// Maps to `minecraft:summoned_entity`.
pub struct SummonEntityEvent;
impl SandEvent for SummonEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::SummonedEntity {
            entity: None,
        })
    }
}

/// Fires when the player interacts with any entity (right-click).
///
/// Maps to `minecraft:player_interacted_with_entity`.
pub struct InteractWithEntityEvent;
impl SandEvent for InteractWithEntityEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(
            crate::AdvancementTrigger::PlayerInteractedWithEntity {
                item: None,
                entity: None,
            },
        )
    }
}

/// Fires when the player trades with a villager.
///
/// Maps to `minecraft:villager_trade`.
pub struct VillagerTradeEvent;
impl SandEvent for VillagerTradeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::VillagerTrade {
            item: None,
            villager: None,
        })
    }
}

/// Fires when the player constructs or upgrades a beacon.
///
/// Maps to `minecraft:construct_beacon`.
pub struct ConstructBeaconEvent;
impl SandEvent for ConstructBeaconEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::ConstructBeacon {
            level: None,
        })
    }
}

/// Fires when the player cures a zombie villager.
///
/// Maps to `minecraft:cured_zombie_villager`.
pub struct CureZombieVillagerEvent;
impl SandEvent for CureZombieVillagerEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::CuredZombieVillager {
            villager: None,
            zombie: None,
        })
    }
}

/// Fires when the player opens a container that generates loot.
///
/// Maps to `minecraft:player_generates_container_loot`.
pub struct LootContainerOpenEvent;
impl SandEvent for LootContainerOpenEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(
            crate::AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table: None },
        )
    }
}

/// Fires when the player achieves Hero of the Village.
///
/// Maps to `minecraft:hero_of_the_village`. Fires once per raid victory.
pub struct HeroOfTheVillageEvent;
impl SandEvent for HeroOfTheVillageEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::HeroOfTheVillage {
            location: None,
        })
    }
}

/// Fires when a lightning bolt strikes near the player.
///
/// Maps to `minecraft:lightning_strike`.
pub struct LightningStrikeEvent;
impl SandEvent for LightningStrikeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::AdvancementTrigger(crate::AdvancementTrigger::LightningStrike {
            lightning: None,
            bystander: None,
        })
    }
}

// ════════════════════════════════════════════════════════════════════════════
// ── AdvancementEvent impls for all advancement-backed events ──────────────
// ════════════════════════════════════════════════════════════════════════════
//
// These allow using the built-in event types with `Event<E>` and the typed
// trigger builders from `sand_core::event::trigger`.

macro_rules! adv_event {
    ($ty:ty) => {
        impl crate::event::AdvancementEvent for $ty {
            type Trigger = crate::AdvancementTrigger;
            fn trigger() -> Self::Trigger {
                <$ty as SandEvent>::dispatch().into_trigger().unwrap()
            }
        }
        impl crate::event::EventPlayer for $ty {}
    };
}

impl SandEventDispatch {
    /// Extract the advancement trigger from this dispatch, panicking if it's
    /// a tick-condition dispatch.
    fn into_trigger(self) -> Option<crate::AdvancementTrigger> {
        match self {
            SandEventDispatch::AdvancementTrigger(t) => Some(t),
            SandEventDispatch::TickCondition(_) => None,
        }
    }
}

adv_event!(EntityKillEvent);
adv_event!(PlayerKillEvent);
adv_event!(PlayerDamageEntityEvent);
adv_event!(EntityDamagePlayerEvent);
adv_event!(ShotCrossbowEvent);
adv_event!(ChanneledLightningEvent);
adv_event!(ItemConsumeEvent);
adv_event!(ItemCraftEvent);
adv_event!(ItemEnchantEvent);
adv_event!(BucketFillEvent);
adv_event!(BucketEmptyEvent);
adv_event!(FishingEvent);
adv_event!(ItemPickedUpEvent);
adv_event!(ItemDurabilityChangeEvent);
adv_event!(BrewPotionEvent);
adv_event!(TotemActivateEvent);
adv_event!(RecipeUnlockEvent);
adv_event!(BlockPlaceEvent);
adv_event!(EnterBlockEvent);
adv_event!(SlideDownBlockEvent);
adv_event!(TargetHitEvent);
adv_event!(BeeNestDestroyedEvent);
adv_event!(ChangeDimensionEvent);
adv_event!(PlayerSleepEvent);
adv_event!(FallFromHeightEvent);
adv_event!(PlayerLevelUpEvent);
adv_event!(EffectsChangedEvent);
adv_event!(StartRidingEvent);
adv_event!(UseEnderEyeEvent);
adv_event!(TameAnimalEvent);
adv_event!(BreedAnimalsEvent);
adv_event!(SummonEntityEvent);
adv_event!(InteractWithEntityEvent);
adv_event!(VillagerTradeEvent);
adv_event!(ConstructBeaconEvent);
adv_event!(CureZombieVillagerEvent);
adv_event!(LootContainerOpenEvent);
adv_event!(HeroOfTheVillageEvent);
adv_event!(LightningStrikeEvent);

// ── Tick-poll events ──────────────────────────────────────────────────────
//
// These fire every tick the condition is true, checked as each online player.
// They use `TickCondition` dispatch — no advancement file is generated.
//
// ⚠️  NBT-based conditions may vary between Minecraft versions. The strings
//     below are tested against Java Edition 1.20–1.21. If a condition doesn't
//     work for your version, implement a custom [`SandEvent`] with the
//     corrected condition string.

/// Fires every tick the player is sneaking / crouching (Shift held).
///
/// Uses `entity @s[nbt={IsSneaking:1b}]` — runtime NBT selector.
///
/// # Example
/// ```rust,ignore
/// #[event]
/// pub fn while_sneaking(event: Event<PlayerSneakEvent>) {
///     cmd::particle(Particle::Smoke, event.player());
/// }
/// ```
pub struct PlayerSneakEvent;
impl SandEvent for PlayerSneakEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={IsSneaking:1b}]".into())
    }
}

/// Fires every tick the player is sprinting.
///
/// Uses `entity @s[nbt={IsSprinting:1b}]`.
pub struct PlayerSprintEvent;
impl SandEvent for PlayerSprintEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={IsSprinting:1b}]".into())
    }
}

/// Fires every tick the player is swimming (swimming animation active, 1.13+).
///
/// Uses `entity @s[nbt={Swimming:1b}]`.
pub struct PlayerSwimmingEvent;
impl SandEvent for PlayerSwimmingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={Swimming:1b}]".into())
    }
}

/// Fires every tick the player is actively flying (Creative/Spectator flight).
///
/// Uses `entity @s[nbt={abilities:{flying:1b}}]`.
pub struct PlayerFlyingEvent;
impl SandEvent for PlayerFlyingEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={abilities:{flying:1b}}]".into())
    }
}

/// Fires every tick the player is on fire.
///
/// Uses `entity @s[nbt={Fire:1s..}]`. Note: `Fire` is a short — negative
/// values indicate the "cooling down" state from water. Values ≥ 1 mean
/// actively burning.
pub struct PlayerOnFireEvent;
impl SandEvent for PlayerOnFireEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[nbt={Fire:1s..}]".into())
    }
}

/// Fires every tick the player is in a Creative-mode gamemode.
pub struct PlayerInCreativeEvent;
impl SandEvent for PlayerInCreativeEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=creative]".into())
    }
}

/// Fires every tick the player is in Adventure mode.
pub struct PlayerInAdventureEvent;
impl SandEvent for PlayerInAdventureEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=adventure]".into())
    }
}

/// Fires every tick the player is in Spectator mode.
pub struct PlayerInSpectatorEvent;
impl SandEvent for PlayerInSpectatorEvent {
    fn dispatch() -> SandEventDispatch {
        SandEventDispatch::TickCondition("entity @s[gamemode=spectator]".into())
    }
}

// ════════════════════════════════════════════════════════════════════════════
// ── EventPlayer impls for all event types ──────────────────────────────────
// ════════════════════════════════════════════════════════════════════════════
// (Advancement-backed types are covered by the adv_event! macro above.)

macro_rules! player_event {
    ($ty:ty) => {
        impl crate::event::EventPlayer for $ty {}
    };
}

player_event!(OnJoinEvent);
player_event!(FirstJoinEvent);
player_event!(OnDeathEvent);
player_event!(OnRespawnEvent);
player_event!(ArmorEquipEvent);
player_event!(ArmorUnequipEvent);
player_event!(HoldingItemEvent);
player_event!(CurrentlyWearingEvent);
player_event!(PlayerSneakEvent);
player_event!(PlayerSprintEvent);
player_event!(PlayerSwimmingEvent);
player_event!(PlayerFlyingEvent);
player_event!(PlayerOnFireEvent);
player_event!(PlayerInCreativeEvent);
player_event!(PlayerInAdventureEvent);
player_event!(PlayerInSpectatorEvent);
