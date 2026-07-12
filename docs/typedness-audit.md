# Typedness Audit — Sand Datapack Framework

> Phase 1 of the typed-API migration.  
> Last updated: 2026-06-23

This document catalogs every public API across the Sand workspace, classifies its
current level of type safety, and defines the planned migration path.

---

## Classification key

| Label | Meaning |
|---|---|
| **Typed** | Only accepts domain-specific types; no raw strings, `Value`, or SNBT in the normal path |
| **Mostly typed** | Core API is typed but one or two fields still accept `Value` or `impl Into<String>` |
| **Raw hatch** | Has an explicit, named raw/custom escape hatch (`raw_component`, `Custom`, etc.) |
| **Needs redesign** | Normal-user path requires raw JSON, SNBT, or unguarded bare strings |
| **Missing coverage** | Vanilla feature not yet represented in the API |

---

## `sand-components` audit

### `advancement/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `AdvancementFrame` | **Typed** | Clean enum |
| `AdvancementIcon::id` | **Mostly typed** | Accepts `impl Display`, should be a typed item/block ID |
| `AdvancementIcon::components` | **Needs redesign** | `Option<Value>` — accepts raw JSON for item components |
| `AdvancementDisplay::title` | **Needs redesign** | Stores `Value`, accepts `impl Into<Value>` — should be `TextComponent` |
| `AdvancementDisplay::description` | **Needs redesign** | Same as `title` |
| `AdvancementDisplay::background` | **Mostly typed** | Raw `String` for texture path — acceptable as `ResourceLocation` |
| `AdvancementTrigger` (all variants) | **Mostly typed** | Typed predicates plus validated associated constructors for resource-bearing variants; legacy string-field variants remain compatibility/raw paths |
| `AdvancementRewards::recipes` | **Needs redesign** | `Vec<String>` — should be `Vec<ResourceLocation>` |
| `AdvancementRewards::loot` | **Needs redesign** | `Vec<String>` — same |
| `AdvancementRewards::function` | **Needs redesign** | `Option<String>` — should be `Option<ResourceLocation>` |
| `Advancement::parent` | **Needs redesign** | `Option<String>` — should be `Option<ResourceLocation>` |
| `Criterion` | **Typed** | Thin wrapper; clean |

#### `AdvancementTrigger` condition field audit

Every field listed as `Value` means the user must pass raw `serde_json::json!{...}` today.

| Trigger | Raw fields |
|---|---|
| `PlayerKilledEntity` | `entity: Option<Value>`, `killing_blow: Option<Value>` |
| `EntityKilledPlayer` | `entity: Option<Value>`, `killing_blow: Option<Value>` |
| `InventoryChanged` | `slots: Option<Value>`, `items: Vec<Value>` |
| `UsedItem / ConsumeItem / UsingItem` | `item: Option<Value>` |
| `PlacedBlock` | `item: Option<Value>`, `location: Option<Value>` |
| `BredAnimals` | `parent/partner/child: Option<Value>` |
| `EnterBlock` | (block is `Option<String>` — raw ID) |
| `EnchantedItem` | `item: Option<Value>`, `levels: Option<Value>` |
| `TamedAnimal / SummonedEntity` | `entity: Option<Value>` |
| `Location` | `location: Option<Value>` |
| `NetherTravel` | `entered/exited/distance: Option<Value>` |
| `PlayerInteractedWithEntity` | `item: Option<Value>`, `entity: Option<Value>` |
| `PlayerHurtEntity / EntityHurtPlayer` | `entity: Option<Value>`, `damage: Option<Value>` |
| `KilledByCrossbow` | `unique_entity_types: Option<Value>`, `victims: Option<Vec<Value>>` |
| `ChanneledLightning` | `victims: Option<Vec<Value>>` |
| `LightningStrike` | `lightning/bystander: Option<Value>` |
| `CraftedItem / FilledBucket / ShotCrossbow / UsedTotem` | `item: Option<Value>` |
| `EmptiedBucket` | `item: Option<Value>`, `location: Option<Value>` |
| `FishingRodHooked` | `rod/entity/item: Option<Value>` |
| `ThrownItemPickedUp` | `item/entity: Option<Value>` |
| `ItemDurabilityChanged` | `item/delta/durability: Option<Value>` |
| `BeeNestDestroyed` | `item: Option<Value>`, `num_bees_inside: Option<Value>` |
| `SleptInBed / HeroOfTheVillage` | `location: Option<Value>` |
| `FallFromHeight` | `distance/start_position: Option<Value>` |
| `LeveledUp / ConstructBeacon / UsedEnderEye` | `level/distance: Option<Value>` |
| `EffectsChanged` | `effects/source: Option<Value>` |
| `TargetHit` | `signal_strength/projectile: Option<Value>` |
| `CuredZombieVillager` | `villager/zombie: Option<Value>` |
| `VillagerTrade` | `item/villager: Option<Value>` |
| `Custom` | `conditions: Option<Value>` — **this one is correct as-is** |

**Migration target:** Each `Option<Value>` should become a typed predicate struct
(`ItemPredicate`, `EntityPredicate`, `LocationPredicate`, `DamagePredicate`, etc.)
with an explicit `::raw(RawJson)` fallback on each type.

---

### `item/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `ItemRarity` | **Typed** | Clean enum |
| `AttributeType` | **Typed** | Has `Custom(String)` escape hatch — good |
| `AttributeOperation` | **Typed** | Clean enum |
| `EquipmentSlotGroup` | **Typed** | Clean enum |
| `EquipmentSlot` | **Typed** | Clean enum |
| `AttributeModifier` | **Typed** | Serializes to SNBT internally; user never writes SNBT |
| `FoodProperties` | **Typed** | Clean |
| `ConsumableAnimation` | **Typed** | Clean enum |
| `ConsumableProperties::sound` | **Mostly typed** | Accepts displayable IDs; a dedicated sound registry wrapper is still pending |
| `EquippableProperties::equip_sound` | **Mostly typed** | Accepts displayable IDs; a dedicated sound registry wrapper is still pending |
| `EquippableProperties::model` | **Mostly typed** | Accepts displayable resource locations |
| `EquippableProperties::allowed_entities` | **Typed** | Supports `EntityTypeId` and `TagId<EntityTypeId>` helpers |
| `ToolRule::blocks` | **Typed** | Supports `BlockId` and `TagId<BlockId>` helpers |
| `ItemComponent` | **Typed** | Primary CustomItem v2 component model |
| `CustomItem::new(base)` | **Typed** | Accepts displayable typed item IDs such as `ItemId` |
| `CustomItem::id()` | **Typed** | Namespaced `custom_data` marker for stable item identity |
| `CustomItem::component()` | **Typed** | Primary component API |
| `CustomItem::item_predicate()` | **Typed** | Returns typed `ItemPredicate` |
| `CustomItem::raw_component()` | **Raw hatch** | Deprecated string-key compatibility API; prefer `RawComponent` |

---

### `loot_table/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `LootTableType` | **Typed** | Has `Custom(String)` escape hatch — good |
| `NumberProvider::Constant / Uniform / Binomial` | **Typed** | Clean |
| `NumberProvider::Score::target` | **Needs redesign** | `Value` — should be typed `ScoreTarget` or `EntitySelector` |
| `NumberProvider::Score::score` | **Mostly typed** | `String` — should be typed objective name |
| `LootCondition::EntityProperties::predicate` | **Needs redesign** | `Value` — needs typed `EntityPredicate` |
| `LootCondition::EntityScores::scores` | **Needs redesign** | `HashMap<String, Value>` — values should be typed score ranges |
| `LootCondition::MatchTool::predicate` | **Needs redesign** | `Value` — needs typed `ItemPredicate` |
| `LootCondition::TimeCheck::value` | **Needs redesign** | `Value` — needs typed range |
| `LootCondition::Custom::data` | **Raw hatch** | Named `Custom` — good, but `data: Value` is implicit raw |
| `LootFunction::SetName::name` | **Needs redesign** | `Value` — should be `TextComponent` |
| `LootFunction::SetLore::lore` | **Needs redesign** | `Vec<Value>` — should be `Vec<TextComponent>` |
| `LootFunction::Custom::data` | **Raw hatch** | Same as above — implicit |
| `LootEntry::Item / Tag / etc.` | **Mostly typed** | Item/tag names are raw `String` — should be typed IDs |
| `LootTable::random_sequence` | **Mostly typed** | `String` — could be `ResourceLocation` |

---

### `predicate/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| (entire module) | **Missing coverage** | Module exists but appears minimal; typed predicate model needed |

---

### `recipe/` modules

| Symbol | Classification | Notes |
|---|---|---|
| `Ingredient` | **Mostly typed** | Item/tag IDs are raw strings |
| `RecipeResult` | **Mostly typed** | Item ID is raw string |
| `ShapedRecipe / ShapelessRecipe` | **Mostly typed** | Ingredient and result IDs are raw strings |
| `CookingRecipe / SmithingRecipe / StonecuttingRecipe` | **Mostly typed** | Same |

---

### `tag/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `Tag` | **Mostly typed** | Values are `Vec<String>` — not registry-typed |

---

### `worldgen/`

| Symbol | Classification | Notes |
|---|---|---|
| `Biome` | **Needs redesign** | Heavy use of `Value` fields for effects, spawners, features |
| `Dimension` | **Needs redesign** | Generator and type configs are mostly `Value` |
| `NoiseSettings` | **Needs redesign** | Density functions, ore veins etc. are `Value` |
| `PlacedFeature` | **Needs redesign** | Feature config is `Value` |

---

### Other component modules

| Module | Classification | Notes |
|---|---|---|
| `damage_type/` | **Mostly typed** | Enums for scaling/effects; clean |
| `dialog/` | **Mostly typed** | Good typed API; body/button accept `TextComponent` |
| `enchantment/` | **Needs redesign** | Effect values are `Value` |
| `item_modifier/` | **Typed** | Thin wrapper around `LootFunction`; inherits its gaps |
| `trim/` | **Mostly typed** | Asset paths are raw strings |
| `chat_type/` | **Mostly typed** | Decoration parameters are `Value` |

---

## `sand-commands` audit

| Symbol | Classification | Notes |
|---|---|---|
| `TextComponent` | **Typed** | Full builder; no raw JSON needed |
| `Selector` / typed selector variants | **Mostly typed** | Entity type, tag, team, etc. are `impl Into<String>` |
| `Execute` | **Mostly typed** | `run_raw()` is a clear escape hatch; most methods are typed |
| `NbtValue::Raw(String)` | **Raw hatch** | Named `Raw` — explicit; good |
| `DataTarget` | **Typed** | Clean enum |
| `Scoreboard / Objective / ScoreHolder` | **Mostly typed** | Names are raw strings |
| `builtins::*` | **Mostly typed** | Most take `impl Into<String>` for IDs (entity type, item, etc.) |
| `sand_core::cmd::effect_give()` | **Typed** | Uses `EffectId` and builder options for duration, amplifier, particles |
| `sand_commands::effect_give()` | **Raw hatch** | Low-level compatibility helper; prefer `sand_core::cmd::effect_give()` |
| `summon()` | **Mostly typed** | Entity type is `impl Into<String>` |
| `gamemode()` | **Typed** | Uses `GameMode` enum — good |

---

## `sand-core` audit

### `state/storage.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `SnbtValue` / `SnbtCompound` | **Typed** | Primitive, list, compound, and explicit `RawSnbt` escape hatch |
| `NbtPath` | **Typed** | Supports root, field/key segments, list indexes, and quoted keys |
| `StorageLocation` / `NbtLocation` | **Typed** | Storage, entity, and block NBT targets |
| `StorageSchema<T>` | **Typed** | Preferred structured storage API |
| `StorageField<Schema, T>` | **Typed** | Typed field handles with get/set/remove/exists/copy/append/merge |
| `StorageVar<T>` | **Mostly typed** | Kept for simple legacy variables |
| `StorageVar::set_raw()` / `NbtPath::set_raw()` | **Raw hatch** | Deprecated string compatibility; prefer `set_raw_snbt(RawSnbt)` |

### `state/score.rs`, `flag.rs`, `timer.rs`, `cooldown.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `ScoreVar<T>` | **Typed** | Clean generic design |
| `Flag` | **Typed** | Clean |
| `Timer` | **Typed** | Clean |
| `Cooldown` | **Typed** | Clean |

### `condition.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `Condition` | **Mostly typed** | `StorageExists`, `ScoreRange` etc. are typed; some string fields |

### `event/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `AdvancementEvent` trait | **Mostly typed** | Trigger associated type is typed; guard returns `Option<Condition>` |
| `Event<E>` | **Typed** | Zero-cost context; clean |
| `EventId` | **Typed** | Clean enum |
| `EventReset` | **Typed** | Clean (with backward-compat aliases) |
| `EventVisibility` | **Typed** | Clean enum |
| `EventAdvancement` | **Mostly typed** | IDs are raw strings; should use `ResourceLocation` |

### `cmd/mod.rs`

| Symbol | Classification | Notes |
|---|---|---|
| `cmd::raw()` | **Raw hatch** | Explicit name — keep |
| Re-exports from `sand-commands` | See `sand-commands` section |

---

## Summary: raw `Value` / raw string occurrence count by file

| File | `Value` count | `impl Into<String>` / `String` ID fields | Priority |
|---|---|---|---|
| `advancement/mod.rs` | ~60 (trigger conditions) | ~10 (IDs) | **Critical** |
| `loot_table/mod.rs` | ~8 | ~8 | **High** |
| `item/mod.rs` | 1 (`item_predicate` return) | ~6 | **Medium** |
| `worldgen/*.rs` | ~30 | ~10 | **High** |
| `enchantment/mod.rs` | ~5 | ~3 | **Medium** |
| `sand-commands/builtins.rs` | 0 | ~15 | **Low** |
| `sand-core/state/storage.rs` | 0 | ~2 | **Low** |
| `sand-core/event/mod.rs` | 0 | ~2 | **Low** |

---

## Planned migration path

### Phase 2 — Explicit raw escape hatch types
Introduce named wrapper types so raw-data usage is always visible at the call site:
- `RawJson(serde_json::Value)` — for raw JSON objects/values
- `RawSnbt(String)` — for raw SNBT strings
- `RawCommand(String)` — for raw command strings (already named in `cmd::raw`)
- `RawComponent(String)` — for raw item component SNBT

Replace anonymous `Value` / bare `String` parameters with `RawJson` where a typed
replacement is not yet available. This makes the gap visible without breaking callers.

### Phase 3 — Typed resource IDs
`ResourceLocation`, `ItemId`, `BlockId`, `EntityTypeId`, `EffectId`,
`EnchantmentId`, `TagId<T>`, etc. Replace the most common `impl Into<String>` ID
parameters.

### Phase 4 — Shared typed predicate core
`ItemPredicate`, `EntityPredicate`, `LocationPredicate`, `DamagePredicate`,
`BlockPredicate`, `NumberProvider` improvements, `ScoreRange`.  Used by
advancements, loot tables, and commands.

### Phase 5 — Advancement trigger rewrite
Replace `Option<Value>` fields in every `AdvancementTrigger` variant with the
typed predicate structs from Phase 4. `AdvancementDisplay::title/description`
become `TextComponent`.

### Phase 6 — Event builder
Typed `EventBuilder` API wrapping `AdvancementEvent` with guard, state, reset,
and visibility. `EventAdvancement` IDs become `ResourceLocation`.

### Phase 7 — Typed status effects
Complete. `StatusEffectId` and `PotionRegistryId` provide the shared validated
registry path for dynamic/modded IDs. The enum-style `EffectId` and `PotionId`
remain source-compatible vanilla conveniences and convert in both directions.
`StatusEffectInstance`, `PotionContents`, and `SuspiciousStewEffect` cover
structured JSON/SNBT data. `sand_core::cmd`
provides typed `effect_give`, `effect_clear`, and `effect_clear_effect`; raw
effect command syntax is explicit through compatibility/escape hatches.

### Phase 8 — CustomItem v2
Complete. `ItemComponent` is the primary component model for `custom_name`,
`item_name`, `lore`, `rarity`, `custom_model_data`, enchantments, attribute
modifiers, food, consumable, equippable, tool, potion contents, suspicious stew
effects, durability/stacking components, unbreakable, and custom data.
`CustomItem::item_predicate()` returns typed `ItemPredicate`. Modded and future
item components remain explicit through `RawComponent`; the legacy
`CustomItem::raw_component(key, snbt)` string-key helper is deprecated.

### Phase 9 — Typed storage schemas
Complete. `StorageSchema<T>` and `StorageField<Schema, T>` provide typed
structured storage, `SnbtValue`/`SnbtCompound` cover primitive/list/compound
SNBT generation, and `NbtPath` supports field/index builders. EventBuilder can
declare typed storage fields via `.storage_field(...)`. A derive macro remains
future work.

### Phase 10 — Commands and conditions cleanup
Audit `impl Into<String>` across `sand-commands`; replace with typed IDs where
Phase 3 provides coverage.

### Phase 11 — Loot table and predicate cleanup
Wire Phase 4 predicates into loot conditions and functions. `SetName`/`SetLore`
become `TextComponent`.

### Phase 12 — Worldgen and structures
Typed biome/dimension/feature builders replacing `Value` fields.

### Phase 13 — Recipes, tags, and remaining components
`TypedTag<T>` now covers item, block, entity-type, and function tags while the
legacy raw `Tag` remains available. Typed recipe ingredient/result migration is
tracked separately.

### Phase 14–16 — Examples, docs, final cleanup

---

## Existing escape hatches — keep these

The following raw/custom escape hatches are **deliberately kept** as named,
explicit opt-ins. They must never be removed.

| Escape hatch | Location | Reason to keep |
|---|---|---|
| `RawComponent` / `CustomItem::with_raw_component()` | `item/mod.rs` | Modded/future item components |
| `AdvancementTrigger::Custom { trigger, conditions }` | `advancement/mod.rs` | Modded advancement triggers |
| `LootCondition::Custom { condition, data }` | `loot_table/mod.rs` | Modded loot conditions |
| `LootFunction::Custom { function, data }` | `loot_table/mod.rs` | Modded loot functions |
| `SnbtValue::Raw(RawSnbt)` / `set_raw_snbt(RawSnbt)` | `state/storage.rs` | Unsupported/modded/future SNBT |
| `NbtValue::Raw(String)` | `sand-commands/nbt.rs` | Low-level command-builder compatibility |
| `cmd::raw(RawCommand)` (planned rename) | `cmd/mod.rs` | Commands not yet generated |
| `AttributeType::Custom(String)` | `item/mod.rs` | Modded entity attributes |
| `LootTableType::Custom(String)` | `loot_table/mod.rs` | Modded loot table types |
