//! Built-in Sand event types and the [`SandEvent`] trait for custom events.
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
//! function. The event type is passed as the (phantom) function parameter:
//!
//! ```rust,ignore
//! use sand_macros::event;
//! use sand_core::events::{OnJoinEvent, OnDeathEvent, ArmorEquipEvent};
//!
//! #[event]
//! pub fn on_join(event: OnJoinEvent) {
//!     mcfunction! { r#"tellraw @s {"text":"Welcome!","color":"gold"}"# }
//! }
//!
//! #[event]
//! pub fn on_death(event: OnDeathEvent) {
//!     mcfunction! { "scoreboard players add @s total_deaths 1" }
//! }
//!
//! // Slot filter required; item is optional
//! #[event(slot = Head, item = "minecraft:diamond_helmet")]
//! pub fn equipped_diamond_helmet(event: ArmorEquipEvent) {
//!     mcfunction! { "say Diamond helmet on!" }
//! }
//! ```
//!
//! # Custom events
//!
//! Implement [`SandEvent`] on your own type to define a custom event. You
//! choose the dispatch mechanism — advancement-trigger or tick-condition:
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
//! pub fn on_pickup(event: ItemPickupEvent) {
//!     mcfunction! { "say Picked something up!" }
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
///     mcfunction! { r#"playsound minecraft:entity.item.pickup player @s"# }
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
/// Implemented as an `Advancement + Tick` trigger that is immediately
/// revoked — so it fires **once per join session**, not every tick.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_join(event: OnJoinEvent) {
///     mcfunction! { r#"tellraw @s {"text":"Welcome back!","color":"gold"}"# }
/// }
/// ```
pub struct OnJoinEvent;

/// Fires the very first time a player ever joins. Never fires again.
///
/// Implemented as an `Advancement + Tick` trigger **without** revocation.
/// Once the advancement is granted it stays, so the event fires exactly once
/// per player across all sessions.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn first_join(event: FirstJoinEvent) {
///     mcfunction! {
///         r#"tellraw @s {"text":"Welcome for the very first time!","color":"aqua"}"#;
///         "give @s minecraft:diamond 3";
///     }
/// }
/// ```
pub struct FirstJoinEvent;

/// Fires on the tick a player dies (any cause: mob, fall, void, `/kill`, …).
///
/// Implemented via the `deathCount` scoreboard criterion. The handler runs as
/// `@s` = the dying player.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_death(event: OnDeathEvent) {
///     mcfunction! {
///         "scoreboard players add @s total_deaths 1";
///         "playsound minecraft:entity.wither.death player @s ~ ~ ~ 0.5 0.8";
///     }
/// }
/// ```
pub struct OnDeathEvent;

/// Fires on the tick after a player respawns from death.
///
/// Sand tags each dying player with `__sand_was_dead` during the death check.
/// Each tick, any player with that tag who is no longer in spectator mode
/// (i.e. has respawned) triggers this event, then the tag is removed.
///
/// # Example
///
/// ```rust,ignore
/// #[event]
/// pub fn on_respawn(event: OnRespawnEvent) {
///     mcfunction! {
///         r#"tellraw @s {"text":"You respawned!","color":"green"}"#;
///         "effect give @s minecraft:regeneration 5 1 true";
///     }
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
/// // Any item equipped in the feet slot
/// #[event(slot = Feet)]
/// pub fn any_boots_equipped(event: ArmorEquipEvent) {
///     mcfunction! { "say Boots equipped!" }
/// }
///
/// // Specific item with custom NBT
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_equipped(event: ArmorEquipEvent) {
///     mcfunction! { "scoreboard players set @s mana_regen 1" }
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
/// #[event(slot = Feet, item = "minecraft:leather_boots", custom_data = "{mana_boots:1b}")]
/// pub fn mana_boots_removed(event: ArmorUnequipEvent) {
///     mcfunction! { "scoreboard players set @s mana_regen 0" }
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
/// #[event(item = "minecraft:diamond_sword")]
/// pub fn holding_diamond_sword(event: HoldingItemEvent) {
///     mcfunction! { "particle minecraft:crit @s ~ ~1 ~ 0.3 0.3 0.3 0.01 3" }
/// }
///
/// #[event(item = "minecraft:shield", slot = Offhand)]
/// pub fn holding_shield_offhand(event: HoldingItemEvent) {
///     mcfunction! { "scoreboard players set @s blocking 1" }
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
/// pub fn wearing_diamond_helmet(event: CurrentlyWearingEvent) {
///     mcfunction! { "particle minecraft:enchant @s ~ ~1.8 ~ 0.2 0.2 0.2 0.1 2" }
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
/// #[event]
/// pub fn on_kill(event: EntityKillEvent) {
///     mcfunction! { "scoreboard players add @s total_kills 1" }
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
/// pub fn on_killed(event: PlayerKillEvent) {
///     mcfunction! { r#"tellraw @s {"text":"You were slain!","color":"red"}"# }
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
/// pub fn on_eat(event: ItemConsumeEvent) {
///     mcfunction! { "say Yum!" }
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
/// #[event]
/// pub fn on_place(event: BlockPlaceEvent) {
///     mcfunction! { "scoreboard players add @s blocks_placed 1" }
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
/// pub fn on_change_dim(event: ChangeDimensionEvent) {
///     mcfunction! { "say Dimension change!" }
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
/// pub fn on_fall(event: FallFromHeightEvent) {
///     mcfunction! { "playsound minecraft:entity.player.hurt player @s" }
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
/// pub fn while_sneaking(event: PlayerSneakEvent) {
///     mcfunction! { "particle minecraft:smoke @s ~ ~1 ~ 0 0 0 0 1" }
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
