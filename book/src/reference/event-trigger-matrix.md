# Event Trigger Matrix

This matrix covers every public built-in event exported from `sand_core`.
Each entry lists the Rust type, canonical import path, the vanilla or Sand
mechanism that fires it, dispatch mode, reliability, and relevant caveats.

**Coverage guarantee:** a workspace test asserts that every name in the
`BUILTIN_EVENT_NAMES` constant appears in this file. CI will fail if a new
built-in event is added without updating both the constant and this matrix.

---

## Dispatch modes

| Mode | Meaning |
|---|---|
| **Advancement (AfterFire)** | Sand generates an advancement JSON; it is revoked after each fire so it can re-arm. |
| **Advancement (OncePerPlayer)** | Advancement is granted once and never revoked — fires exactly once per player, ever. |
| **Sand tick / scoreboard** | Sand polls a scoreboard criterion or custom objective each tick. No advancement file. |
| **Sand tick / entity tag** | Sand tracks transitions via entity tags added/removed each tick. |
| **Sand tick / items check** | Sand runs `execute if items entity @s <slot> <item>` each tick per player. |
| **Sand tick / predicate** | Sand evaluates a generated `execute if predicate` each tick per player. |
| **Sand tick / selector** | Sand runs `execute as @a[gamemode=…]` or a similar selector each tick. |
| **Sand tick / XP scoreboard** | Sand polls `experience query @s levels` via scoreboard and compares ticks. |

---

## Reliability key

| Rating | Meaning |
|---|---|
| **High** | Fires exactly once per vanilla action. |
| **Medium** | Fires reliably but with a tick-boundary delay or minor edge case. |
| **Tick-rate** | Fires every server tick while the condition holds. |

---

## Runtime cost key

| Rating | Meaning |
|---|---|
| **Negligible** | One advancement file; no per-tick work. |
| **Low** | One scoreboard read or selector test per tick per player. |
| **Medium** | One entity-tag or items-predicate check per tick per player. |
| **High** | Multiple scoreboard objectives updated per tick per player. |

---

## Session events

These fire on discrete player lifecycle transitions.

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `OnJoinEvent` | `sand_core::event::vanilla::OnJoin` | Do something when a player connects | Generated scoreboard/tick detection using `__sand_join` | Sand tick / `JoinTick` scoreboard | Medium | Low | Fires on initial join/load detection. Mid-session disconnect/reconnect without `/reload` does not re-fire. |
| `FirstJoinEvent` | `sand_core::event::vanilla::FirstJoin` | One-time welcome / starter kit | `minecraft:tick` advancement, never revoked | Advancement (OncePerPlayer) | High | Negligible | Fires once per player account across all sessions. Cannot un-fire. |
| `OnDeathEvent` | `sand_core::event::vanilla::OnDeath` | React to any player death | `deathCount` scoreboard criterion | Sand tick / scoreboard | High | Low | Fires for any cause: mob, fall, void, `/kill`. Cannot distinguish cause without a separate tracker. |
| `OnRespawnEvent` | `sand_core::event::vanilla::OnRespawn` | React to respawn (after death screen) | Sand tag `__sand_was_dead` + spectator-mode check | Sand tick / entity tag | Medium | Low | One-tick delay after the player leaves spectator. Does not fire for `/gamemode survival` outside of a death cycle. |

---

## Equipment events

These fire on per-tick transitions in equipment slots. All require a `slot` filter.

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `ArmorEquipEvent` | `sand_core::events::ArmorEquipEvent` | Detect when a player puts on armor | Entity tag `__armor_<slot>` transition | Sand tick / entity tag | Medium | Medium | Fires one tick after equip; rapid swap may coalesce. Requires `slot` filter. `item` and `custom_data` optional. |
| `ArmorUnequipEvent` | `sand_core::events::ArmorUnequipEvent` | Detect when a player removes armor | Entity tag `__armor_<slot>` transition | Sand tick / entity tag | Medium | Medium | Same constraints as `ArmorEquipEvent`. Rapid swap-back may skip fire. |
| `HoldingItemEvent` | `sand_core::events::HoldingItemEvent` | React every tick a player holds an item | `execute if items entity @s mainhand/offhand <item>` | Sand tick / items check | Tick-rate | Medium | Fires every tick the item is held. Not a "pickup" event — fires continuously. Requires `item` filter. |
| `CurrentlyWearingEvent` | `sand_core::events::CurrentlyWearingEvent` | React every tick a player wears armor | `execute if items entity @s armor.<slot> <item>` | Sand tick / items check | Tick-rate | Medium | Fires every tick the item is in the slot. Requires both `slot` and `item` filters. |

---

## Kill and combat events

These are advancement-backed and fire once per action.

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `EntityKillEvent` | `sand_core::event::vanilla::EntityKill` | React when the player kills any entity | `minecraft:player_killed_entity` | Advancement (AfterFire) | High | Negligible | No entity-type filter on the built-in; use a custom `AdvancementEvent` with `PlayerKilledEntity { entity: … }` for filtering. |
| `PlayerKillEvent` | `sand_core::event::vanilla::PlayerKill` | React when the player is killed by any entity | `minecraft:entity_killed_player` | Advancement (AfterFire) | High | Negligible | No entity filter; use a custom event for source filtering. |
| `PlayerDamageEntityEvent` | `sand_core::event::vanilla::PlayerDamagesEntity` | Detect when the player deals damage | `minecraft:player_hurt_entity` | Advancement (AfterFire) | High | Negligible | Damage amount is **not** available in the reward function; use `DamageEvent<T>` only for reflected-damage commands, not for reading the amount. |
| `EntityDamagePlayerEvent` | `sand_core::event::vanilla::EntityDamagesPlayer` | Detect when the player takes damage | `minecraft:entity_hurt_player` | Advancement (AfterFire) | High | Negligible | Same limitation: no exact damage amount in the reward function. See `systems-damage` for approximate damage tracking. |
| `ShotCrossbowEvent` | `sand_core::event::vanilla::CrossbowShot` | React when the player fires a crossbow | `minecraft:shot_crossbow` | Advancement (AfterFire) | High | Negligible | — |
| `ChanneledLightningEvent` | `sand_core::events::ChanneledLightningEvent` | React when a trident channels lightning | `minecraft:channeled_lightning` | Advancement (AfterFire) | High | Negligible | Only fires for the trident Channeling enchantment during a thunderstorm. |

---

## Item events

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `ItemConsumeEvent` | `sand_core::event::vanilla::AnyItemConsumed` | React when the player eats or drinks | `minecraft:consume_item` | Advancement (AfterFire) | High | Negligible | Fires for food, potions, milk buckets, etc. No item filter on the built-in type. |
| `ItemCraftEvent` | `sand_core::event::vanilla::AnyItemCrafted` | React when the player crafts anything | `minecraft:crafted_item` | Advancement (AfterFire) | High | Negligible | No item filter on the built-in type. |
| `ItemEnchantEvent` | `sand_core::event::vanilla::AnyItemEnchanted` | React when the player uses an enchanting table | `minecraft:enchanted_item` | Advancement (AfterFire) | High | Negligible | No enchantment or level filter on the built-in type. |
| `BucketFillEvent` | `sand_core::events::BucketFillEvent` | React when the player fills a bucket | `minecraft:filled_bucket` | Advancement (AfterFire) | High | Negligible | — |
| `BucketEmptyEvent` | `sand_core::events::BucketEmptyEvent` | React when the player empties a bucket | `minecraft:emptied_bucket` | Advancement (AfterFire) | High | Negligible | Requires MC 1.17+. |
| `FishingEvent` | `sand_core::events::FishingEvent` | React when a fishing rod hooks something | `minecraft:fishing_rod_hooked` | Advancement (AfterFire) | High | Negligible | Fires on hook, not on reel-in. |
| `ItemPickedUpEvent` | `sand_core::events::ItemPickedUpEvent` | React when a thrown item is picked up | `minecraft:thrown_item_picked_up` | Advancement (AfterFire) | High | Negligible | Only fired for items that were previously thrown (not dropped from breaking blocks). |
| `ItemDurabilityChangeEvent` | `sand_core::events::ItemDurabilityChangeEvent` | React when an item loses durability | `minecraft:item_durability_changed` | Advancement (AfterFire) | High | Negligible | No access to exact delta in the reward function without a custom event + conditions. |
| `BrewPotionEvent` | `sand_core::event::vanilla::PotionBrewed` | React when the player brews a potion | `minecraft:brewed_potion` | Advancement (AfterFire) | High | Negligible | — |
| `TotemActivateEvent` | `sand_core::events::TotemActivateEvent` | React when a totem of undying saves the player | `minecraft:used_totem` | Advancement (AfterFire) | High | Negligible | Only fires when the totem absorbs a killing blow. |
| `RecipeUnlockEvent` | `sand_core::events::RecipeUnlockEvent` | React when the player unlocks any recipe | `minecraft:recipe_unlocked` (Custom trigger, no recipe filter) | Advancement (AfterFire) | High | Negligible | Fires for every recipe unlock. Cannot filter by recipe in the built-in type. |

---

## Block and world events

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `BlockPlaceEvent` | `sand_core::event::vanilla::AnyBlockPlaced` | React when the player places a block | `minecraft:placed_block` | Advancement (AfterFire) | High | Negligible | No block filter on the built-in type. |
| `EnterBlockEvent` | `sand_core::events::EnterBlockEvent` | React when the player enters a block (water, honey) | `minecraft:enter_block` | Advancement (AfterFire) | High | Negligible | Fires when the player's hitbox overlaps a block, not on movement. |
| `SlideDownBlockEvent` | `sand_core::events::SlideDownBlockEvent` | React when the player slides down a honey block wall | `minecraft:slide_down_block` | Advancement (AfterFire) | High | Negligible | — |
| `TargetHitEvent` | `sand_core::events::TargetHitEvent` | React when a target block is hit near the player | `minecraft:target_hit` | Advancement (AfterFire) | High | Negligible | "Near the player" is defined by vanilla; use signal-strength condition for precision. |
| `BeeNestDestroyedEvent` | `sand_core::events::BeeNestDestroyedEvent` | React when the player destroys a bee nest or beehive | `minecraft:bee_nest_destroyed` | Advancement (AfterFire) | High | Negligible | — |

---

## Player state events

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `ChangeDimensionEvent` | `sand_core::event::vanilla::DimensionChanged` | React when the player travels to another dimension | `minecraft:changed_dimension` | Advancement (AfterFire) | High | Negligible | No from/to filter on the built-in type. |
| `PlayerSleepEvent` | `sand_core::events::PlayerSleepEvent` | React when the player enters a bed | `minecraft:slept_in_bed` | Advancement (AfterFire) | High | Negligible | Fires on entering the bed, before the night-skip animation completes. |
| `FallFromHeightEvent` | `sand_core::events::FallFromHeightEvent` | React when the player falls and lands | `minecraft:fall_from_height` | Advancement (AfterFire) | High | Negligible | No minimum distance filter on the built-in type; add a distance condition for fall-damage scenarios. |
| `PlayerLevelUpEvent` | `sand_core::event::vanilla::PlayerLevelsUp` | React when the player gains an XP level | Sand XP scoreboard polling (`__sand_xp_lvl`, `__sand_xp_prev`, `__sand_xp_delta`) | Sand tick / XP scoreboard | Medium | High | **Not backed by a vanilla advancement trigger.** Vanilla has no `minecraft:leveled_up`. Sand detects level increases via per-tick scoreboard comparison. Level decreases do not fire. First tick after join only initializes baseline. `PlayerLevelUpEvent::level_delta("@s")` gives the delta. |
| `EffectsChangedEvent` | `sand_core::events::EffectsChangedEvent` | React when the player's potion effects change | `minecraft:effects_changed` | Advancement (AfterFire) | High | Negligible | Fires on any effect change (add, remove, amplify). No effect filter on the built-in type. |
| `StartRidingEvent` | `sand_core::events::StartRidingEvent` | React when the player starts riding an entity | `minecraft:started_riding` | Advancement (AfterFire) | High | Negligible | No entity filter on the built-in type. |
| `UseEnderEyeEvent` | `sand_core::events::UseEnderEyeEvent` | React when the player throws an ender eye | `minecraft:used_ender_eye` | Advancement (AfterFire) | High | Negligible | — |
| `HeroOfTheVillageEvent` | `sand_core::events::HeroOfTheVillageEvent` | React when the player earns Hero of the Village | `minecraft:hero_of_the_village` | Advancement (AfterFire) | High | Negligible | Fires once per raid victory. |
| `LightningStrikeEvent` | `sand_core::events::LightningStrikeEvent` | React when lightning strikes near the player | `minecraft:lightning_strike` | Advancement (AfterFire) | High | Negligible | Covers naturally spawned lightning, tridents, and lightning rods. |

---

## Entity and interaction events

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `TameAnimalEvent` | `sand_core::event::vanilla::AnimalTamed` | React when the player tames an animal | `minecraft:tame_animal` | Advancement (AfterFire) | High | Negligible | No entity filter on the built-in type. |
| `BreedAnimalsEvent` | `sand_core::event::vanilla::AnimalsBreed` | React when the player breeds animals | `minecraft:bred_animals` | Advancement (AfterFire) | High | Negligible | No entity filter on the built-in type. |
| `SummonEntityEvent` | `sand_core::event::vanilla::EntitySummoned` | React when the player summons an entity | `minecraft:summoned_entity` | Advancement (AfterFire) | High | Negligible | Covers Iron Golem, Snow Golem, Wither. Does not fire for `/summon`. |
| `InteractWithEntityEvent` | `sand_core::events::InteractWithEntityEvent` | React when the player right-clicks an entity | `minecraft:player_interacted_with_entity` | Advancement (AfterFire) | High | Negligible | No entity filter on the built-in type. For interaction-entity patterns, use `systems-entities`. |
| `VillagerTradeEvent` | `sand_core::events::VillagerTradeEvent` | React when the player completes a villager trade | `minecraft:villager_trade` | Advancement (AfterFire) | High | Negligible | — |
| `ConstructBeaconEvent` | `sand_core::events::ConstructBeaconEvent` | React when the player activates or upgrades a beacon | `minecraft:construct_beacon` | Advancement (AfterFire) | High | Negligible | No tier/level filter on the built-in type. |
| `CureZombieVillagerEvent` | `sand_core::events::CureZombieVillagerEvent` | React when the player cures a zombie villager | `minecraft:cured_zombie_villager` | Advancement (AfterFire) | High | Negligible | — |
| `LootContainerOpenEvent` | `sand_core::events::LootContainerOpenEvent` | React when the player opens a loot container | `minecraft:player_generates_container_loot` | Advancement (AfterFire) | High | Negligible | No loot-table filter on the built-in type. |

---

## Tick-poll / continuous state events

These fire **every server tick** while the condition holds. Each one runs an `execute as @a if …` or equivalent check. Author owns the per-tick cost for every registered handler.

| Event type | Preferred import | User intent | Vanilla mechanism | Dispatch | Reliability | Cost | Caveats |
|---|---|---|---|---|---|---|---|
| `PlayerSneakEvent` | `sand_core::event::vanilla::PlayerSneaking` | React every tick a player is crouching | `execute if predicate __sand_local:__sand/player_sneaking` | Sand tick / predicate | Tick-rate | Low | Fires continuously while sneaking. Not an "on sneak start" event. |
| `PlayerSprintEvent` | `sand_core::event::vanilla::PlayerSprinting` | React every tick a player is sprinting | `execute if predicate __sand_local:__sand/player_sprinting` | Sand tick / predicate | Tick-rate | Low | Fires continuously while sprinting. |
| `PlayerSwimmingEvent` | `sand_core::event::vanilla::PlayerSwimming` | React every tick a player is in swimming animation | `execute if predicate __sand_local:__sand/player_swimming` | Sand tick / predicate | Tick-rate | Low | Swimming animation requires 1.13+; triggers on crawl-swim, not on wading in water. |
| `PlayerFlyingEvent` | `sand_core::events::PlayerFlyingEvent` | React every tick a player is flying (Creative/Spectator) | `entity @s[nbt={abilities:{flying:1b}}]` | Sand tick / selector | Tick-rate | Low | Fires for Creative and Spectator flight only. Does not fire for Elytra. |
| `PlayerOnFireEvent` | `sand_core::event::vanilla::PlayerOnFire` | React every tick a player is on fire | `execute if predicate __sand_local:__sand/player_on_fire` | Sand tick / predicate | Tick-rate | Low | Fires for any fire source: lava, fire, flame arrows. |
| `PlayerInCreativeEvent` | `sand_core::events::PlayerInCreativeEvent` | React every tick a player is in Creative mode | `entity @s[gamemode=creative]` | Sand tick / selector | Tick-rate | Low | — |
| `PlayerInAdventureEvent` | `sand_core::events::PlayerInAdventureEvent` | React every tick a player is in Adventure mode | `entity @s[gamemode=adventure]` | Sand tick / selector | Tick-rate | Low | — |
| `PlayerInSpectatorEvent` | `sand_core::events::PlayerInSpectatorEvent` | React every tick a player is in Spectator mode | `entity @s[gamemode=spectator]` | Sand tick / selector | Tick-rate | Low | — |

---

## Intentional exclusions

The following public types are exported from `sand_core` but are excluded from the
matrix because they are traits or infrastructure, not callable event types:

| Name | Reason excluded |
|---|---|
| `SandEvent` | Trait for defining custom events; not a built-in event type. |
| `SandEventDispatch` | Enum used internally by `SandEvent` impls; not callable directly. |

---

## Vanilla limitations and honest caveats

- **No exact damage amount in reward functions.** `PlayerDamageEntityEvent` and
  `EntityDamagePlayerEvent` fire at the right moment but the vanilla advancement
  reward function receives no damage payload. Use the `systems-damage` feature
  (`DamageTracker`) for approximate damage tracking via the
  `minecraft.custom:minecraft.damage_taken` stat.

- **`PlayerLevelUpEvent` is not vanilla-backed.** There is no
  `minecraft:leveled_up` advancement trigger. Sand generates a tick system that
  polls `experience query @s levels` each tick. This has a measurable per-player
  per-tick cost (`High`) and detects only level *increases*; level decreases are
  silent.

- **Tick-poll events have author-owned runtime cost.** Every `PlayerSneakEvent`,
  `HoldingItemEvent`, etc. handler adds one `execute if …` check per player per
  tick. Registering many tick-poll handlers on large servers is expensive.

- **Advancement-backed events have a one-tick minimum latency.** The advancement
  reward function runs on the tick the criterion is met, but the game processes
  advancements after most other game logic in that tick.

- **`OnJoinEvent` vs `FirstJoinEvent`.** `OnJoinEvent` resets on every
  `minecraft:load` (scoreboard reset) and fires once per load for each player.
  `FirstJoinEvent` uses an advancement grant that is never revoked — it fires
  exactly once per player for the lifetime of the datapack.

- **`OnRespawnEvent` uses a tag, not a vanilla trigger.** Sand detects respawn
  via the `__sand_was_dead` entity tag placed on death and removed when the
  player re-enters a non-spectator gamemode. Edge case: `/gamemode survival` on
  a newly spectating player will trigger this.
