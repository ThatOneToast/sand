//! Default user-facing Sand API.
//!
//! Bring the whole prelude into scope with:
//! ```rust,ignore
//! use sand_core::prelude::*;
//! ```
//!
//! The prelude is the recommended import for ordinary datapack authoring. It
//! covers typed functions, components, commands, selectors, state, storage,
//! events (both `AdvancementEvent`/`Event<E>` and bare `SandEvent` markers,
//! plus participant declaration via `ParticipantBuilder`), common component
//! builders, text, resource references, and deliberate raw escape hatches.
//! Reach for [`crate::advanced`] only when you need lower-level export
//! registries or custom framework integration points, or `crate::events`
//! directly for the event dispatch *graph* internals (`EventGraph` and
//! friends) that this prelude deliberately does not flatten.

pub use crate::{all, any, cmd, mcfunction};

// ── Conditions & execute wiring ───────────────────────────────────────────────

pub use crate::cmd::{ConditionedExecute, ExecuteExt, TypedExecute};
pub use crate::condition::Condition;
pub use crate::execute_when::{if_, unless, when};

// ── Command builders ──────────────────────────────────────────────────────────

pub use crate::Damage;
pub use crate::cmd::{
    Actionbar, BlockPos, BlockState, Bossbar, BossbarColor, BossbarId, BossbarStyle, Build,
    CloneBlocks, CloneMaskMode, CloneMode, Coord, DamageAmount, DamageBuilder, DamageKind,
    DataCommand, EffectDuration, EntityTargets, Execute, Fill, FillMode, FunctionMacroArg,
    FunctionMacroArgs, GameMode, Inventory, ItemSlot, Nbt, NbtCompound, NbtRef, NbtTarget,
    Objective, ObjectiveName, Particle, ParticleBuilder, ParticleSpread, PlayerTargets, RawCommand,
    RenderCommand, Rotation, ScoreHolder, Selector, SetBlock, SetBlockMode, SingleEntity,
    SinglePlayer, Sound, SoundSource, Title, TitleTimes, UntypedNbt, Validate, Vec2, Vec3,
};
pub use crate::item::{
    BlockInventory, ContainerIndex, EnderChestIndex, EntityInventory, EntityInventorySlot,
    HotbarIndex, InventoryIndex, ItemLocation, MainInventoryIndex,
};
pub use crate::vfx::{Vfx, VfxParticle, VfxParticleVisibility, VfxSound, VfxStep};

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    BlockNbt, Cooldown, EntityNbt, Flag, FlagRef, GameState, GameStateRef, IntoStateCommands,
    NbtLocation, NbtPath, ScoreRef, ScoreVar, SnbtCompound, SnbtValue, StateFlow, StorageField,
    StorageLocation, StorageSchema, StorageVar, Ticks, Timer, TypedGameState,
};

// ── Optional systems ──────────────────────────────────────────────────────────

#[cfg(feature = "systems-damage")]
pub use crate::systems::damage::{DamageThreshold, DamageTracker, recently_damaged};

#[cfg(feature = "systems-player-data")]
pub use crate::systems::player_data::{
    CooldownField, CooldownFieldRef, FlagField, GameStateField, GlobalStorageField,
    PlayerDataSchema, PlayerSchema, ScoreField, TimerField, TimerFieldRef,
};

// ── Version gating ────────────────────────────────────────────────────────────

pub use crate::version::{MinecraftVersion, VersionFeature, VersionProfile};

/// Validated `namespace:path` locations shared by typed resource IDs.
pub use crate::ResourceLocation;

// ── Entity queries and execution-scoped contexts ──────────────────────────────

pub use crate::entity::{
    Adoption, AdoptionSource, AnyEntity, AttributeBinding, AttributeModifierBinding,
    CurrentHealthSync, CurveEvaluationError, CurveInputs, DEFAULT_FIXED_POINT_SCALE,
    DerivedScoreEncoding, EffectBinding, EntityAction, EntityArchetype, EntityContext,
    EntityCooldown, EntityDerivation, EntityDiagnostic, EntityEnum, EntityEnumValue, EntityEventId,
    EntityFlag, EntityKind, EntityNbtBinding, EntityNbtProperty, EntityNbtType, EntityNbtValue,
    EntityQueries, EntityQuery, EntityScope, EntityScore, EntityState, EntityStateField, EntityTag,
    EntityTeam, EntityText, EntityTextSegment, EntityTimer, EntityTransition, EnumEncoding,
    EquipmentBinding, FixedPoint, FixedValue, HealthBinding, HealthResizePolicy, KnownEntityKind,
    LivingEntityKind, MarkerKind, Migration, MutableLivingEntityKind, NameBinding,
    NumericPropertySource, OverflowPolicy, OwnershipPolicy, PlayerContext, PlayerKind,
    PlayerQueries, PlayerQuery, RawEntityProperty, RawEntityStateField, ReconcilePolicy,
    RefreshPolicy, Relation, RelationQuery, RoundingPolicy, SafeEntityDataWriteKind,
    ScopedEntityRef, SingleEntityQuery, SinglePlayerQuery, SpecialEntityPolicy, StatCurve,
    StateFieldDescriptor, StateFieldKind, StatePredicate, StateSchema, TagBinding, TeamBinding,
    ThresholdDirection, ZombieKind,
};

// ── Function refs (IntoFunctionRef trait) ─────────────────────────────────────

pub use crate::function::IntoFunctionRef;

// ── Typed event model ─────────────────────────────────────────────────────────

pub use crate::event::handle::EventHandle;
pub use crate::event::trigger::{
    ConsumeItemTrigger, EntityKilledPlayerTrigger, ImpossibleTrigger, InventoryChangedTrigger,
    ItemEnchantTrigger, ItemObtainedTrigger, MultiKillTrigger, PlayerInteractedWithEntityTrigger,
    PlayerKilledEntityTrigger, RecipeUnlockedTrigger, SummonedEntityTrigger, TickTrigger,
    UsingItemTrigger,
};
pub use crate::event::{
    AdvancementEvent, DamageAdvancementEvent, DamageEvent, Event, EventId, EventReset,
    EventVisibility,
};

// ── Custom SandEvent authoring (#273) ──────────────────────────────────────────
//
// `SandEvent`/`SandEventDispatch` are the bare-marker counterpart to
// `AdvancementEvent`/`Event<E>` above — defining a custom tick-polled,
// chained, or tracked-transition event, and dispatching it as
// `fn handler(event: MarkerType)` rather than `Event<MarkerType>`. Both are
// now part of ordinary event authoring (#273 made bare `SandEvent` handlers
// a first-class, equally-ergonomic alternative to `AdvancementEvent`
// handlers), so they belong in the default authoring surface alongside
// `AdvancementEvent`/`Event<E>` above — unlike the rest of `crate::events`
// (the event dispatch *graph* internals: `EventGraph`, `ChainEventDispatch`
// builder plumbing, `NormalizedEventDispatch`, …), which remain reachable
// only through `crate::events` (`sand::events`) directly, not this glob.
// `SandEventParticipants` is the extension trait providing `.entity`/`.item`/
// `.attacker`/`.killer`/`.victim`/`.interacted_entity`/`.weapon` on bare
// `SandEvent` markers (mirroring `Event<E>`'s inherent methods below) — it
// must be in scope (as it is here) for those methods to resolve; see its own
// doc for why it is a trait rather than inherent methods.

pub use crate::events::{SandEvent, SandEventDispatch, SandEventParticipants};

// ── Participant context vocabulary (#230, #273) ────────────────────────────────
//
// The vocabulary needed to consume `Event<E>::entity`/`.item`/`.attacker`/
// `.victim`/`.weapon` (and the identical bare-`SandEvent` accessors via
// `SandEventParticipants` above) — reliability, availability, unavailable
// reasons, and role enums — plus the two types needed to *declare* a plan
// (`EventParticipantPlan`, the plan type itself, and `ParticipantBuilder`,
// the ordinary-Rust builder that produces one; see its doc for the full
// observe/inherit/advancement-bridge model). Typed handles
// (`EntityParticipant`) and observation-backend internals stay under
// `crate::participant` (`sand::participant`) directly, not this glob — see
// that module's doc for the full API.

pub use crate::participant::{
    BoundedItemSnapshot, EntityParticipantRole, EventParticipantPlan, ItemParticipantRole,
    ParticipantAvailability, ParticipantBuilder, ParticipantHand, ParticipantReliability,
    ParticipantUnavailableReason,
};

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{
    Dialog, DialogAction, DialogBody, DialogButton, DialogKind, DialogTag,
};

// ── Chat type builders ────────────────────────────────────────────────────────

pub use sand_components::{ChatDecoration, ChatDecorationParameter, ChatStyle, ChatType};

// ── Item/component builders ──────────────────────────────────────────────────

pub use sand_components::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, AttributeId, AttributeModifier, AttributeOperation, AttributeType,
    BannerPattern, BiomeSelector, BlockPredicate, CarverFloatRange, CarvingStep, CaveCarverConfig,
    ConfiguredCarver, ConfiguredFeature, ConsumableAnimation, ConsumableProperties, Criterion,
    CustomData, CustomItem, DamagePredicate, DamageSourcePredicate, DensityFunction,
    DensityFunctionBinaryOp, DensityFunctionExpr, DensityFunctionUnaryOp, Dimension, DimensionType,
    DistancePredicate, Enchantment, EnchantmentCost, EnchantmentEntry, EnchantmentOrTag,
    EnchantmentProvider, EnchantmentProviderInt, EnchantmentSelection, EnchantmentValueOperation,
    EntityEquipment, EntityFlags, EntityPredicate, EntityPredicateTarget, EquipmentModelId,
    EquipmentSlot, EquipmentSlotGroup, EquippableProperties, ExclusionZone, FoodProperties,
    FrequencyReductionMethod, GenerationStep, HeightProvider, Heightmap, Ingredient, IntoItemStack,
    ItemComponent, ItemModifier, ItemOrTag, ItemPredicate, ItemRarity, ItemStack,
    ItemStackComponents, JigsawConfig, LevelBasedValue, LocationPredicate, LootCondition,
    LootEntry, LootFunction, LootPool, LootTable, LootTableType, MobCategory,
    MonsterSpawnLightLevel, Noise, OreConfig, OreTarget, PlacedFeature, PoolElement, PoolEntry,
    Predicate, PredicateRoot, Processor, ProcessorList, ProcessorRule, ProcessorsRef, Projection,
    Rarity, RecipeResult, RuleTest, ShapedRecipe, ShapelessRecipe, SmithingTransformRecipe,
    SmithingTrimRecipe, SpawnBoundingBox, SpawnEntry, SpawnOverride, SpreadType,
    StonecuttingRecipe, Structure, StructureEntry, StructurePlacement, StructureSet, Tag, TagEntry,
    TagRegistry, TemplatePool, TerrainAdaptation, ToolProperties, ToolRule, TradeItem, TradeSet,
    TrimAssetName, TrimMaterial, TrimPattern, TypedTag, VerticalAnchor, VillagerLevel,
    VillagerProfession, VillagerTrade, VillagerTradePool, VillagerTradePoolPatch, VillagerTradeRef,
    WanderingTraderPool, WeatherPredicate,
};

// ── Raw escape hatch types ────────────────────────────────────────────────────

pub use sand_components::{RawComponent, RawJson, RawSnbt};

// ── Typed registry identifiers ────────────────────────────────────────────────

pub use sand_components::{
    AdvancementId, BiomeId, BlockId, ConfiguredCarverId, ConfiguredFeatureId, DamageTypeId,
    DensityFunctionId, DialogId, DimensionId, DimensionTypeId, EffectId,
    EnchantmentEffectComponentId, EnchantmentId, EntityTypeId, FunctionId, ItemId, LootTableId,
    NoiseId, PotionContents, PotionId, PotionRegistryId, PredicateId, ProcessorListId,
    RandomSequenceId, RecipeId, SoundEventId, StatusEffectId, StatusEffectInstance, StructureId,
    StructureSetId, StructureTemplate, StructureTemplateId, StructureTypeId, SuspiciousStewEffect,
    TagId, TemplatePoolId, TradeSetId, VillagerTradeId,
};

// ── Text / chat ───────────────────────────────────────────────────────────────

pub use sand_commands::{
    ChatColor, ClickEvent, EntityHoverId, HoverEvent, IntoTextEntityType, Text, TextComponent,
};

#[cfg(test)]
mod tests {
    use super::*;
    use sand_components::DatapackComponent;

    #[test]
    fn prelude_exports_typed_command_path() {
        let cmd = cmd::tellraw(
            Selector::all_players(),
            Text::new("Hello from Sand").gold().bold(true),
        )
        .to_string();

        assert!(cmd.starts_with("tellraw @a "));
        assert!(cmd.contains(r#""text":"Hello from Sand""#));
        assert!(cmd.contains(r#""color":"gold""#));
        assert!(cmd.contains(r#""bold":true"#));
    }

    #[test]
    fn prelude_exports_typed_dimension_reference_path() {
        let location = ResourceLocation::new("test", "skylands").unwrap();
        let _dimension_type = DimensionType::new(location.clone());
        let dimension = Dimension::new(
            location,
            DimensionTypeId::minecraft("overworld").unwrap(),
            serde_json::json!({"type": "minecraft:flat", "settings": {}}),
        );
        assert_eq!(dimension.to_json()["type"], "minecraft:overworld");
    }

    #[test]
    fn prelude_exports_typed_configured_feature_reference_path() {
        let feature = ConfiguredFeature::no_op(
            ResourceLocation::new("test", "worldgen/configured_feature/nothing").unwrap(),
        );
        let placed = PlacedFeature::new(
            ResourceLocation::new("test", "worldgen/placed_feature/nowhere").unwrap(),
            feature.id(),
        );
        assert_eq!(
            placed.to_json()["feature"],
            "test:worldgen/configured_feature/nothing"
        );
    }

    #[test]
    fn prelude_exports_typed_configured_carver_reference_path() {
        let carver = ConfiguredCarver::cave(
            ResourceLocation::new("test", "worldgen/configured_carver/shallow_cave").unwrap(),
            CaveCarverConfig::new(
                0.15,
                HeightProvider::absolute(0),
                CarverFloatRange::new(0.1, 0.9),
                VerticalAnchor::Absolute(-54),
            ),
        );
        let biome = crate::Biome::new(
            ResourceLocation::new("test", "worldgen/biome/shallow_caves").unwrap(),
            crate::BiomeEffects::new(0xC0D8FF, 0x3F76E4, 0x050533, 0x78A7FF),
        )
        .carver_step(CarvingStep::Air, carver.id());
        assert_eq!(
            biome.to_json()["carvers"]["air"][0],
            "test:worldgen/configured_carver/shallow_cave"
        );
    }

    #[test]
    fn prelude_exports_typed_text_event_helpers() {
        let entity_type = EntityTypeId::minecraft("zombie").unwrap();
        let uuid = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let text = Text::new("Inspect")
            .hover_entity_with_id(entity_type, uuid, Text::new("Undead"))
            .click_change_page(1);
        let json = text.to_string();
        assert!(json.contains(r#""action":"show_entity""#));
        assert!(json.contains(r#""action":"change_page""#));
    }

    #[test]
    fn prelude_exports_resource_locations_for_function_refs() {
        let id = ResourceLocation::new("example", "start").unwrap();
        assert_eq!(cmd::function(id).to_string(), "function example:start");
    }

    #[test]
    fn prelude_exports_typed_game_state() {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Phase {
            Idle = 0,
        }

        impl TypedGameState for Phase {
            fn to_score(self) -> i32 {
                self as i32
            }

            fn from_score(score: i32) -> Option<Self> {
                match score {
                    0 => Some(Self::Idle),
                    _ => None,
                }
            }
        }

        static PHASE: GameState<Phase> = GameState::with_default_score("phase", 0);

        let _state_ref: GameStateRef<'_, Phase> = PHASE.of("@s");
        assert_eq!(PHASE.of("@s").reset(), "scoreboard players set @s phase 0");
    }

    #[test]
    fn prelude_exports_vfx_types() {
        let commands = Vfx::new("prelude")
            .particle(VfxParticle::happy_villager().count(2))
            .sound(VfxSound::new("minecraft:block.note_block.bell").source(SoundSource::Player))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:happy_villager ~0 ~0 ~0 0 0 0 0 2 force",
                "execute at @s run playsound minecraft:block.note_block.bell player @s ~ ~ ~ 1 1",
            ]
        );
    }

    #[cfg(feature = "systems-damage")]
    #[test]
    fn prelude_exports_damage_threshold_with_damage_system() {
        let current: Condition =
            DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0));
        let last: Condition =
            DamageTracker::last_damage_at_least("@s", DamageThreshold::raw_stat(10));

        assert_eq!(DamageThreshold::hearts(1.0).to_raw_stat(), 10);
        assert_eq!(
            when(current).then_one("say current"),
            ["execute if score @s sd_dmg_delta matches 10.. run say current"]
        );
        assert_eq!(
            when(last).then_one("say last"),
            ["execute if score @s sd_dmg_last matches 10.. run say last"]
        );
    }

    #[cfg(feature = "systems-player-data")]
    #[test]
    fn prelude_exports_manual_player_schema_contract() {
        static MANA: ScoreVar<i32> = ScoreVar::new("mana");
        static HAS_WAND: Flag = Flag::new("has_wand");
        static REGEN_TIMER: Timer = Timer::new("regen", Ticks::seconds(2));
        static CAST_COOLDOWN: Cooldown = Cooldown::new("cast_cd", Ticks::seconds(3));

        let schema = PlayerDataSchema::new("magic")
            .score(&MANA, 100)
            .flag(&HAS_WAND, false)
            .timer(&REGEN_TIMER)
            .cooldown(&CAST_COOLDOWN);

        assert_eq!(schema.scoreboard_field_count(), 4);
        assert_eq!(
            schema.define_all(),
            vec![
                "scoreboard objectives add mana dummy",
                "scoreboard objectives add has_wand dummy",
                "scoreboard objectives add regen dummy",
                "scoreboard objectives add cast_cd dummy",
            ]
        );
        assert_eq!(
            schema.init_player("@s"),
            vec![
                "execute unless score @s mana matches -2147483648.. run scoreboard players set @s mana 100",
                "execute unless score @s has_wand matches -2147483648.. run scoreboard players set @s has_wand 0",
            ]
        );
    }

    /// Facade-completeness fixture (#175/#168/#169/#146 Phase 8): every
    /// symbol here must be reachable through a single `use super::*;` glob
    /// import of this prelude module (equivalently, `use sand::prelude::*;`
    /// for external consumers, since `sand::prelude` re-exports
    /// `sand_core::prelude::*` verbatim) — selector construction, score
    /// objective/holder, setblock/fill, teleport, tag, gamemode, a
    /// function call, and the raw command escape hatch, without reaching
    /// into `sand_commands`/`sand_core` internals directly.
    #[test]
    fn prelude_only_command_argument_fixture_compiles_and_runs() {
        // Selector construction.
        let scan = Selector::all_entities().limit(5).distance_range(0.0, 16.0);
        assert!(scan.try_build().is_ok());

        // Score objective + holder.
        static SCORE: Objective = Objective::new("prelude_fixture");
        let score_cmd = SCORE.try_set(ScoreHolder::self_(), 1).unwrap();
        assert_eq!(score_cmd, "scoreboard players set @s prelude_fixture 1");

        // Block placement.
        let setblock = SetBlock::new(BlockPos::here(), BlockState::of("minecraft:stone"))
            .try_build()
            .unwrap();
        assert_eq!(setblock, "setblock ~ ~ ~ minecraft:stone");
        let fill = Fill::new(
            BlockPos::absolute(0, 64, 0),
            BlockPos::absolute(1, 65, 1),
            BlockState::of("minecraft:glass"),
        )
        .try_build()
        .unwrap();
        assert_eq!(fill, "fill 0 64 0 1 65 1 minecraft:glass");

        // Teleport (generated typed command builder — `cmd::teleport_4`).
        let tp = cmd::teleport_4(Selector::self_(), Vec3::absolute(0.0, 64.0, 0.0)).to_string();
        assert_eq!(tp, "teleport @s 0 64 0");

        // Tag + gamemode (generated typed command builders).
        let tag = cmd::tag_add(Selector::self_(), "prelude_fixture").to_string();
        assert_eq!(tag, "tag @s add prelude_fixture");
        let gamemode = cmd::gamemode(GameMode::Survival)
            .target(Selector::self_())
            .to_string();
        assert_eq!(gamemode, "gamemode survival @s");

        // Function call.
        let call = cmd::try_function("my_pack:api/do_thing").unwrap();
        assert_eq!(call, "function my_pack:api/do_thing");

        // Raw command escape hatch.
        let raw = cmd::raw("mymod:pulse 5").try_build().unwrap();
        assert_eq!(raw, "mymod:pulse 5");
    }
}
