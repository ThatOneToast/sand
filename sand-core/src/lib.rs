// `SandEvent::dispatch` impls returning the concrete `SandEventDispatch` (rather than
// the trait's `impl Into<SandEventDispatch>`) is the expected, common case and is
// intentionally more specific — not a hazard here since `SandEvent::dispatch` is never
// called through a generic bound that relies on the opaque return type being non-refinable.
#![allow(refining_impl_trait)]

//! # sand-core
//!
//! This is an implementation crate. Datapack authors should depend on the
//! `sand` facade; only items reachable through a supported `sand::` path are
//! part of Sand's author-facing compatibility contract.
//!
//! Core types, traits, command builders, and datapack components for the
//! [Sand](https://github.com/ThatOneToast/sand) Minecraft datapack toolkit.
//!
//! This crate provides everything needed to define Minecraft datapack elements
//! in Rust:
//!
//! - [`ResourceLocation`] — validated `namespace:path` identifiers
//! - [`DatapackComponent`] — trait implemented by all datapack element types
//! - [`cmd`] — typed command builders (`Execute`, `Target`, `SetBlock`, etc.)
//!   plus auto-generated enums for `Item`, `Block`, `EntityType`, and more
//! - [`components`] — advancements, recipes, loot tables, predicates, item
//!   modifiers, tags, and custom items
//! - [`mcfunction!`] — advanced macro for command grouping and migration
//!
//! # Usage
//!
//! This crate is used alongside [`sand_macros`](https://docs.rs/sand-macros)
//! for the `#[function]` and `#[datapack_component]` proc macros:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_macros::{datapack_component, function};
//!
//! #[function]
//! pub fn greet() {
//!     cmd::tellraw(
//!         Target::players(),
//!         Text::new("Hello from Sand!").gold().bold(true),
//!     );
//! }
//!
//! #[datapack_component(Load)]
//! pub fn on_load() {
//!     cmd::say("Datapack loaded");
//! }
//! ```
//!
//! # API tiers
//!
//! - [`prelude`] is the default user-facing import for datapack authors.
//! - [`advanced`] contains the supported version-aware custom export hook.
//! - `#[doc(hidden)]` items are for macro expansion and internal wiring, not
//!   a stable authoring surface.

pub mod advanced;
pub mod build;
pub mod cmd;
pub mod component;
pub mod components;
pub mod condition;
pub mod custom_item_ext;
pub mod entity;
pub mod error;
pub mod event;
pub mod events;
pub mod execute_when;
pub mod function;
pub mod ir;
pub mod item;
pub mod mc_version;
pub mod participant;
pub mod prelude;
pub mod resource_location;
pub mod resource_ref;
pub mod state;
pub mod systems;
pub mod version;
pub mod vfx;

// ── Re-export the sand-components crate itself ────────────────────────────────

/// The `sand-components` crate — typed JSON builders for every datapack component type.
pub use sand_components;

// ── Dialog callback registry ──────────────────────────────────────────────────

/// Drain all registered dialog callbacks (id, path) pairs.
///
/// Called by the export pipeline to generate `__sand_dialog_tick` and
/// `__sand_dialog_init` infrastructure. End users do not call this directly.
pub use sand_components::dialog::drain_dialog_callbacks;

/// The scoreboard trigger objective used by `DialogAction::callback(...)`.
pub use sand_components::dialog::SAND_DIALOG_TRIGGER;

// ── Core infrastructure ───────────────────────────────────────────────────────

pub use cmd::{
    Actionbar, BlockState, Bossbar, BossbarColor, BossbarStyle, CloneBlocks, CloneMaskMode,
    CloneMode, Command, ConditionedExecute, Cooldown, ExecuteExt, Fill, FillMode, ItemSlot,
    NbtStoreKind, NbtValue, Objective, ObjectiveName, ParticleEffect, ParticleSpread, RawCommand,
    RenderCommand, ScoreCmp, ScoreHolder, SetBlock, SetBlockMode, Sound, SoundSource, Storage,
    Target, Title, TypedExecute, Validate,
};
pub use component::try_export_components;
pub use component::try_export_components_for_version;
pub use component::try_export_components_json;
pub use component::try_export_components_json_for_version;
pub use component::{
    ComponentContent, ComponentExportError, ComponentRecord, DatapackComponent, ExportResult,
    IntoDatapack,
};
pub use error::{Result, SandError};
pub use event::handle::EventHandle;
pub use event::{
    AdvancementEvent, DamageAdvancementEvent, DamageEvent, Event, EventId, EventReset,
    EventVisibility, IntoEventId,
};
pub use events::{
    // Equipment events
    ArmorEquipEvent,
    ArmorUnequipEvent,
    BeeNestDestroyedEvent,
    // Block / world events
    BlockPlaceEvent,
    BreedAnimalsEvent,
    BrewPotionEvent,
    BucketEmptyEvent,
    BucketFillEvent,
    // Player state events
    ChangeDimensionEvent,
    ChanneledLightningEvent,
    ConstructBeaconEvent,
    CureZombieVillagerEvent,
    CurrentlyWearingEvent,
    EffectsChangedEvent,
    EnterBlockEvent,
    EntityDamagePlayerEvent,
    // Kill / combat
    EntityKillEvent,
    // Trait + dispatch
    EventSetup,
    FallFromHeightEvent,
    FirstJoinEvent,
    FishingEvent,
    HeroOfTheVillageEvent,
    HoldingItemEvent,
    InteractWithEntityEvent,
    // Item events
    ItemConsumeEvent,
    ItemCraftEvent,
    ItemDurabilityChangeEvent,
    ItemEnchantEvent,
    ItemPickedUpEvent,
    LightningStrikeEvent,
    LootContainerOpenEvent,
    OnDeathEvent,
    // Session events
    OnJoinEvent,
    OnRespawnEvent,
    PersistentEventCondition,
    PersistentSandEvent,
    PlayerDamageEntityEvent,
    PlayerFlyingEvent,
    PlayerInAdventureEvent,
    PlayerInCreativeEvent,
    PlayerInSpectatorEvent,
    PlayerKillEvent,
    PlayerLevelUpEvent,
    PlayerOnFireEvent,
    PlayerSleepEvent,
    // Tick-poll state events
    PlayerSneakEvent,
    PlayerSprintEvent,
    PlayerStartSneakingEvent,
    PlayerStopSneakingEvent,
    PlayerSwimmingEvent,
    RecipeUnlockEvent,
    SandEvent,
    SandEventDispatch,
    SandEventParticipants,
    ShotCrossbowEvent,
    SlideDownBlockEvent,
    StartRidingEvent,
    SummonEntityEvent,
    TameAnimalEvent,
    TargetHitEvent,
    TickEventDispatch,
    TickScope,
    TotemActivateEvent,
    UseEnderEyeEvent,
    VillagerTradeEvent,
};
pub use function::{
    ArmorEventDescriptor, ArmorEventKind, ArmorSlot, ComponentFactory, EventDescriptor,
    EventDispatch, EventPathEntry, FunctionDescriptor, FunctionPointerEntry,
    FunctionPointerTypeEntry, FunctionTagDescriptor, IntoFunctionRef, ScheduleDescriptor,
    ScoreThresholdComparator, TrackedSource, TrackedTransition, TransitionKind, drain_dyn_fns,
    register_dyn_fn, register_dyn_fn_dedup,
};

mod compiler;
mod transition;
pub use mc_version::McVersion;
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};
pub use state::{
    BlockNbt, EntityNbt, NbtLocation, NbtPath, SnbtCompound, SnbtValue, StorageField,
    StorageLocation, StorageSchema, StorageVar,
};
pub use state::{GameState, GameStateRef, TypedGameState};
pub use state::{
    StateCleanup, StateInit, StateLifecycle, StateMigrate, StateProvision, StateReconcile,
    StateTick,
};
#[doc(hidden)]
pub use state::{StateDescriptor, StateHookDescriptor, StateLifecycleDescriptor, StateScope};
pub use vfx::{Vfx, VfxParticle, VfxParticleVisibility, VfxSound, VfxStep};

// ── McFunction (sand-core-specific component) ─────────────────────────────────

pub use components::mc_function::{IntoCommands, McFunction};

// ── Custom item typed extensions ──────────────────────────────────────────────

pub use custom_item_ext::{CustomItemExt, CustomItemId};

// ── Datapack component builders (all from sand-components) ───────────────────

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{
    Dialog, DialogAction, DialogBody, DialogButton, DialogKind, DialogTag,
};

pub use sand_components::{
    // Advancement
    Advancement,
    AdvancementDisplay,
    AdvancementFrame,
    AdvancementIcon,
    AdvancementId,
    AdvancementRewards,
    AdvancementTrigger,
    // Custom item
    AttributeId,
    AttributeModifier,
    AttributeOperation,
    AttributeType,
    // Banner / Painting / Chat
    BannerPattern,
    // Worldgen
    Biome,
    BiomeEffects,
    // Typed registry identifiers
    BiomeId,
    BiomeSelector,
    BlockId,
    // Typed predicate model
    BlockPredicate,
    // Worldgen
    CarverFloatRange,
    CarvingStep,
    CaveCarverConfig,
    ChatDecoration,
    ChatType,
    // Animal variants
    ChickenVariant,
    ChickenVariantId,
    ConfiguredCarver,
    ConfiguredCarverId,
    ConfiguredFeature,
    ConfiguredFeatureId,
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    CowVariant,
    CowVariantId,
    Criterion,
    CustomData,
    CustomItem,
    // Damage
    DamageEffects,
    DamagePredicate,
    DamageScaling,
    DamageSourcePredicate,
    DamageType,
    DamageTypeId,
    DeathMessageType,
    // Worldgen density functions
    DensityFunction,
    DensityFunctionBinaryOp,
    DensityFunctionExpr,
    DensityFunctionId,
    DensityFunctionUnaryOp,
    DialogId,
    Dimension,
    DimensionId,
    DimensionType,
    DimensionTypeId,
    DistancePredicate,
    DyedColor,
    EffectId,
    EffectPredicate,
    // Enchantment
    Enchantment,
    EnchantmentCost,
    EnchantmentEffectComponentId,
    EnchantmentEntry,
    EnchantmentId,
    EnchantmentOrTag,
    EnchantmentProvider,
    EnchantmentProviderInt,
    EnchantmentSelection,
    // Loot table enchantment selector (ID or tag reference)
    EnchantmentSelector,
    EnchantmentValueOperation,
    EntityEquipment,
    EntityFlags,
    // Item predicates
    EntityPredicate,
    // Standalone predicate authoring
    EntityPredicateTarget,
    EntityTypeId,
    EquipmentModelId,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    // Structure sets
    ExclusionZone,
    FloatRange,
    FoodProperties,
    FrequencyReductionMethod,
    FunctionId,
    GenerationStep,
    HeightProvider,
    Heightmap,
    Ingredient,
    // Instrument / Jukebox
    Instrument,
    IntRange,
    // Item stack (#229)
    IntoItemStack,
    IntoRecipeItemId,
    ItemComponent,
    ItemId,
    // Item modifier
    ItemModifier,
    ItemOrTag,
    ItemPredicate,
    ItemRarity,
    ItemStack,
    ItemStackComponents,
    JigsawConfig,
    JukeboxSong,
    LevelBasedValue,
    LocationPredicate,
    // Loot table
    LootCondition,
    LootEntry,
    LootFunction,
    LootPool,
    LootTable,
    LootTableId,
    LootTableType,
    LootText,
    MobCategory,
    MonsterSpawnLightLevel,
    Noise,
    NoiseId,
    NoiseSettings,
    NumberProvider,
    OreConfig,
    OreTarget,
    PaintingVariant,
    PigVariant,
    PigVariantId,
    PlacedFeature,
    // Structure/template pool
    PoolElement,
    PoolEntry,
    PotionContents,
    PotionId,
    PotionRegistryId,
    // Predicate
    Predicate,
    PredicateId,
    PredicateRoot,
    // Processor list
    Processor,
    ProcessorList,
    ProcessorListId,
    ProcessorRule,
    ProcessorsRef,
    Projection,
    // Villager trades (26.1+)
    RandomSequenceId,
    Rarity,
    // Raw escape hatch types
    RawComponent,
    RawJson,
    RawSnbt,
    RecipeId,
    RecipeResult,
    RuleTest,
    ShapedRecipe,
    ShapelessRecipe,
    SmithingTransformRecipe,
    SmithingTrimRecipe,
    SoundEventId,
    SpawnBoundingBox,
    // Animal variants (shared spawn-condition model)
    SpawnCondition,
    SpawnEntry,
    SpawnOverride,
    SpreadType,
    StatusEffectId,
    StatusEffectInstance,
    StonecuttingRecipe,
    Structure,
    StructureEntry,
    StructureId,
    StructurePlacement,
    StructureSet,
    StructureSetId,
    StructureTemplate,
    StructureTemplateId,
    StructureTypeId,
    SuspiciousStewEffect,
    // Tag
    Tag,
    TagEntry,
    TagId,
    TagRegistry,
    TemplatePool,
    TemplatePoolId,
    TerrainAdaptation,
    Ticks,
    ToolProperties,
    ToolRule,
    TradeItem,
    TradeSet,
    TradeSetId,
    // Trim
    TrimAssetName,
    TrimMaterial,
    TrimPattern,
    TypedTag,
    VerticalAnchor,
    VillagerLevel,
    VillagerProfession,
    VillagerTrade,
    VillagerTradeId,
    VillagerTradePool,
    VillagerTradePoolPatch,
    VillagerTradeRef,
    WanderingTraderPool,
    // Weather
    WeatherPredicate,
    // Wolf
    WolfVariant,
};

/// High-level typed damage command builder.
pub use sand_commands::Damage;

/// Re-exported so proc macros can write `::sand_core::inventory::submit!`
/// without requiring users to add `inventory` as a direct dependency.
#[doc(hidden)]
pub use inventory;

/// Re-exported so proc-macro generated code can use `::sand_core::serde_json::json!`
/// without requiring users to add `serde_json` as a direct dependency.
#[doc(hidden)]
pub use serde_json;

/// Build a `Vec<String>` of Minecraft commands.
///
/// Accepts semicolon-separated expressions. String literals are used as-is;
/// any value implementing [`std::fmt::Display`] (including command builders
/// from [`crate::cmd`]) is serialized via `.to_string()`.
///
/// # Examples
/// ```
/// use sand_core::mcfunction;
/// let cmds = mcfunction!["say hello world"; r#"give @a diamond 1"#];
/// assert_eq!(cmds[0], "say hello world");
/// assert_eq!(cmds.len(), 2);
/// ```
///
/// With command builders:
/// ```rust,ignore
/// use sand_core::{mcfunction, cmd::Target};
/// let cmds = mcfunction![
///     sand_core::cmd::say("Welcome!");
///     sand_core::cmd::kill(Target::entities().tag("enemy"));
/// ];
/// ```
#[macro_export]
macro_rules! mcfunction {
    ($($cmd:expr);* $(;)?) => {{
        let mut _commands: Vec<String> = Vec::new();
        $(
            _commands.extend($crate::IntoCommands::into_commands($cmd));
        )*
        _commands
    }};
}

/// Compose a typed [`Condition`](crate::condition::Condition) requiring **all** sub-conditions.
///
/// Sugar for [`Condition::all`](crate::condition::Condition::all).
///
/// # Example
/// ```rust,ignore
/// use sand_core::{all, state::ScoreVar};
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
/// let cond = all![MANA.of("@s").gte(25), MANA.of("@s").lte(100)];
/// ```
#[macro_export]
macro_rules! all {
    ($($c:expr),+ $(,)?) => {
        $crate::condition::Condition::all([$($c),+])
    };
}

/// Compose a typed [`Condition`](crate::condition::Condition) requiring **any** sub-condition.
///
/// Sugar for [`Condition::any`](crate::condition::Condition::any).
/// Generates one execute command per sub-condition.
///
/// # Example
/// ```rust,ignore
/// use sand_core::{any, state::ScoreVar};
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
/// static RAGE: ScoreVar<i32> = ScoreVar::new("rage");
/// let cond = any![MANA.of("@s").gte(25), RAGE.of("@s").gte(10)];
/// ```
#[macro_export]
macro_rules! any {
    ($($c:expr),+ $(,)?) => {
        $crate::condition::Condition::any([$($c),+])
    };
}

/// Generated Minecraft registry enums (`Item`, `Block`, `EntityType`, etc.).
///
/// Populated at build time by `sand-build` for the Minecraft version specified
/// in the `SAND_MC_VERSION` environment variable (default:
/// `sand_version::DEFAULT_CODEGEN_VERSION`, currently `1.21.11`).
///
/// # Example
/// ```rust,ignore
/// use sand_core::generated::Item;
/// let item = Item::OakLog;
/// println!("{}", item.resource_location()); // "minecraft:oak_log"
/// ```
#[allow(warnings)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/registries.rs"));
}

impl sand_components::recipe::IntoRecipeItemId for generated::Item {
    fn into_recipe_item_id(self) -> sand_components::registry::ItemId {
        self.resource_location()
            .parse()
            .expect("generated vanilla item IDs are valid resource locations")
    }
}

impl sand_commands::IntoTextEntityType for generated::EntityType {
    fn into_text_entity_type(self) -> String {
        self.resource_location().to_owned()
    }
}

impl sand_commands::selector::IntoEntityType for generated::EntityType {
    fn into_entity_type(self) -> String {
        self.resource_location().to_owned()
    }
}

/// Compiler and facade wiring that is deliberately outside Sand's supported
/// author-facing API surface.
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use crate::entity::query::{
        QueryableStateScope, StateQueryHandle, StateQueryScope, StateQuerySpec,
        lower_state_query_current, lower_state_query_each,
    };
    #[doc(hidden)]
    pub use crate::entity::state::{
        ArchetypeStateScope, EntityStateScope, GlobalStateBundleScope, GlobalStateScope,
        LivingStateScope, PlayerStateScope, SameStateScope, StateBundleMember, StateBundleTarget,
        StateBundleTree, StateDataFieldDescriptor, StateScopeMarker, resolve_state_objective,
        state_attach_commands, state_attached_condition, state_bundle_trees_overlap,
        state_detach_commands, state_presence_predicate,
    };
    #[doc(hidden)]
    pub use crate::function::StateSystemDescriptor;
    #[doc(hidden)]
    pub use crate::state::StateMigrationDescriptor;
    /// Extracts an advancement trigger for generated proc-macro wiring.
    #[doc(hidden)]
    pub fn event_dispatch_advancement(
        dispatch: crate::events::SandEventDispatch,
    ) -> Option<crate::AdvancementTrigger> {
        dispatch.into_advancement()
    }

    /// Extracts a legacy raw tick condition for generated proc-macro wiring.
    #[doc(hidden)]
    pub fn event_dispatch_tick_condition(
        dispatch: crate::events::SandEventDispatch,
    ) -> Option<String> {
        dispatch.into_tick_condition()
    }

    /// Extracts a typed tick dispatch for generated proc-macro wiring.
    #[doc(hidden)]
    pub fn event_dispatch_tick(
        dispatch: crate::events::SandEventDispatch,
    ) -> Option<crate::events::TickEventDispatch> {
        dispatch.into_tick()
    }

    /// Extracts a chain dispatch for generated proc-macro wiring.
    #[doc(hidden)]
    pub fn event_dispatch_chain(
        dispatch: crate::events::SandEventDispatch,
    ) -> Option<crate::events::ChainEventDispatch> {
        dispatch.into_chain()
    }

    /// Extracts compiler-owned tracked dispatch state for proc-macro wiring.
    #[doc(hidden)]
    pub fn event_dispatch_tracked(
        dispatch: crate::events::SandEventDispatch,
    ) -> Option<crate::TrackedTransition> {
        dispatch.into_tracked()
    }

    /// Exact generated contract providers selected by this `sand-core` build.
    ///
    /// Keeping these as embedded build outputs lets the installed CLI inspect
    /// its own command and vanilla-registry APIs without source parsing,
    /// network access, or a second version-selection mechanism.
    pub const GENERATED_API_PROVIDER_CATALOGS: &[&str] = &[
        include_str!(concat!(env!("OUT_DIR"), "/commands.api.json")),
        include_str!(concat!(env!("OUT_DIR"), "/registries.api.json")),
        include_str!(concat!(env!("OUT_DIR"), "/registry_ids.api.json")),
    ];

    /// Proc-macro bridge for the built-in sneaking transition source.
    #[doc(hidden)]
    pub const fn player_sneaking_tracked_source() -> crate::TrackedSource {
        crate::events::PLAYER_SNEAKING_TRACKED_SOURCE
    }

    /// Constructs compiler-owned version and dirty entity-score fields for
    /// Sand's entity-state derive expansion.
    pub const fn entity_score_new<T: 'static>(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        kind: crate::entity::StateFieldKind,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> crate::entity::EntityScore<T> {
        crate::entity::EntityScore::__new(namespace, schema, name, kind, default, bounds)
    }

    /// Constructs compiler-owned fixed-point fields for State derive output.
    #[doc(hidden)]
    pub const fn fixed_score_new(
        namespace: &'static str,
        schema: &'static str,
        name: &'static str,
        scale: i32,
        default: i32,
        bounds: Option<(i32, i32)>,
    ) -> crate::entity::FixedScore {
        crate::entity::FixedScore::__new(namespace, schema, name, scale, default, bounds)
    }
}

impl From<generated::Item> for sand_components::registry::ItemId {
    fn from(item: generated::Item) -> Self {
        item.resource_location()
            .parse()
            .expect("generated vanilla item IDs are valid resource locations")
    }
}

impl From<generated::Block> for sand_components::registry::BlockId {
    fn from(block: generated::Block) -> Self {
        block
            .resource_location()
            .parse()
            .expect("generated vanilla block IDs are valid resource locations")
    }
}

impl From<generated::EntityType> for sand_components::registry::EntityTypeId {
    fn from(entity_type: generated::EntityType) -> Self {
        entity_type
            .resource_location()
            .parse()
            .expect("generated vanilla entity type IDs are valid resource locations")
    }
}

/// Generated block state property types.
///
/// Each block with configurable state properties gets a typed `*Properties`
/// struct. Shared property enums (e.g. `Facing`, `Half`) are generated once
/// and reused across blocks.
#[allow(warnings)]
pub mod block_states {
    include!(concat!(env!("OUT_DIR"), "/block_states.rs"));
}
