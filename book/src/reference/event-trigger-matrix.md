# Built-in event matrix

Every built-in event Sand ships (`sand::events::*`), usable as `Event<E>` in an
`#[event]` handler. This list is kept in sync with
`sand_core::events::BUILTIN_EVENT_NAMES` by a compile-time test
(`sand-example/src/lib.rs`) — every name here must appear on this page.

See [Events](../09-events.md) and [Event Composition](../10-event-composition.md)
for how to use these; see [Vanilla Limitations](vanilla-limitations.md) for
what backs each one (advancement trigger vs. tick-poll approximation).

## Session

| Event | Fires when |
|---|---|
| `OnJoinEvent` | A player joins (including reconnects). |
| `FirstJoinEvent` | A player joins this server for the very first time. |
| `OnDeathEvent` | A player dies. |
| `OnRespawnEvent` | A player respawns after death. |

## Equipment

| Event | Fires when |
|---|---|
| `ArmorEquipEvent` | A tracked armor slot changes from empty/other to a watched item. |
| `ArmorUnequipEvent` | A tracked armor slot changes away from a watched item. |
| `HoldingItemEvent` | The player's main hand holds a watched item. |
| `CurrentlyWearingEvent` | Tick-poll: the player currently wears a watched item. |

## Kill / combat

| Event | Fires when |
|---|---|
| `EntityKillEvent` | The player kills any entity. |
| `PlayerKillEvent` | The player kills another player. |
| `PlayerDamageEntityEvent` | The player damages an entity. |
| `EntityDamagePlayerEvent` | An entity damages the player. |
| `ShotCrossbowEvent` | The player fires a crossbow. |
| `ChanneledLightningEvent` | The player channels lightning with a trident. |

## Items

| Event | Fires when |
|---|---|
| `ItemConsumeEvent` | The player consumes an item (food, potion, etc.). |
| `ItemCraftEvent` | The player crafts an item. |
| `ItemEnchantEvent` | The player enchants an item. |
| `BucketFillEvent` | The player fills a bucket. |
| `BucketEmptyEvent` | The player empties a bucket. |
| `FishingEvent` | The player catches something while fishing. |
| `ItemPickedUpEvent` | The player picks up an item. |
| `ItemDurabilityChangeEvent` | A held/worn item's durability changes. |
| `BrewPotionEvent` | The player brews a potion. |
| `TotemActivateEvent` | A totem of undying saves the player. |
| `RecipeUnlockEvent` | The player unlocks a recipe. |

## Block / world

| Event | Fires when |
|---|---|
| `BlockPlaceEvent` | The player places a watched block. |
| `EnterBlockEvent` | The player enters a watched block's space. |
| `SlideDownBlockEvent` | The player slides down a block (e.g. powder snow). |
| `TargetHitEvent` | The player hits a target block. |
| `BeeNestDestroyedEvent` | The player destroys a bee nest/hive. |

## Player state

| Event | Fires when |
|---|---|
| `ChangeDimensionEvent` | The player changes dimension. |
| `PlayerSleepEvent` | The player sleeps in a bed. |
| `FallFromHeightEvent` | The player falls from at least a configured height. |
| `PlayerLevelUpEvent` | The player's XP level increases (tick-poll, see [Vanilla Limitations](vanilla-limitations.md)). |
| `EffectsChangedEvent` | The player's active status effects change. |
| `StartRidingEvent` | The player starts riding an entity. |
| `UseEnderEyeEvent` | The player uses an eye of ender. |
| `HeroOfTheVillageEvent` | The player receives Hero of the Village. |
| `LightningStrikeEvent` | Lightning strikes near the player. |

## Entity / interaction

| Event | Fires when |
|---|---|
| `TameAnimalEvent` | The player tames an animal. |
| `BreedAnimalsEvent` | The player breeds animals. |
| `SummonEntityEvent` | The player summons an entity. |
| `InteractWithEntityEvent` | The player interacts with an entity. |
| `VillagerTradeEvent` | The player trades with a villager. |
| `ConstructBeaconEvent` | The player constructs a beacon. |
| `CureZombieVillagerEvent` | The player cures a zombie villager. |
| `LootContainerOpenEvent` | The player opens a loot-generating container. |

## Tick-poll / continuous state

These have no vanilla trigger and are detected by polling player state every
tick — see [Vanilla Limitations](vanilla-limitations.md) for what that implies
for precision.

| Event | Fires when |
|---|---|
| `PlayerStartSneakingEvent` | The player begins sneaking. |
| `PlayerStopSneakingEvent` | The player stops sneaking. |
| `PlayerSneakEvent` | The player is currently sneaking (level-triggered). |
| `PlayerSprintEvent` | The player is currently sprinting. |
| `PlayerStartSprintingEvent` | The player begins sprinting. |
| `PlayerStopSprintingEvent` | The player stops sprinting. |
| `PlayerSwimmingEvent` | The player is currently swimming. |
| `PlayerStartSwimmingEvent` | The player begins swimming. |
| `PlayerStopSwimmingEvent` | The player stops swimming. |
| `PlayerFlyingEvent` | The player is currently flying (creative/spectator). |
| `PlayerStartFlyingEvent` | The player begins actively flying. |
| `PlayerStopFlyingEvent` | The player stops actively flying. |
| `PlayerOnFireEvent` | The player is currently on fire. |
| `PlayerCaughtFireEvent` | The player catches fire. |
| `PlayerExtinguishedEvent` | The player stops being on fire. |
| `PlayerInCreativeEvent` | The player is currently in creative mode. |
| `PlayerInAdventureEvent` | The player is currently in adventure mode. |
| `PlayerInSpectatorEvent` | The player is currently in spectator mode. |
| `PlayerEnteredSurvivalEvent` | The player switches into survival mode. |
| `PlayerExitedSurvivalEvent` | The player switches out of survival mode. |
| `PlayerEnteredCreativeEvent` | The player switches into creative mode. |
| `PlayerExitedCreativeEvent` | The player switches out of creative mode. |
| `PlayerEnteredAdventureEvent` | The player switches into adventure mode. |
| `PlayerExitedAdventureEvent` | The player switches out of adventure mode. |
| `PlayerEnteredSpectatorEvent` | The player switches into spectator mode. |
| `PlayerExitedSpectatorEvent` | The player switches out of spectator mode. |
| `PlayerHealthChangedEvent` | The player's health value changes (gain or loss). |
| `PlayerHealthLostEvent` | The player's health value decreases. |
| `PlayerHealthGainedEvent` | The player's health value increases. |
| `PlayerLowHealthEvent<HALF_HEARTS>` | The player's health drops to or below `HALF_HEARTS` half-hearts. |
| `PlayerRecoveredHealthEvent<HALF_HEARTS>` | The player's health rises back above `HALF_HEARTS` half-hearts. |
| `EffectStarted<E>` | The player gains status effect `E` (see [`StatusEffectMarker`](../09-events.md) markers). |
| `EffectStopped<E>` | The player loses status effect `E`. |

Freezing and drowning start/stop events are intentionally not provided — see
[Vanilla Limitations](vanilla-limitations.md).
