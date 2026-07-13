#![allow(deprecated)] // Compatibility exports intentionally retain deprecated APIs.

//! # sand-core
//!
//! Core types, traits, command builders, and datapack components for the
//! [Sand](https://github.com/ThatOneToast/sand) Minecraft datapack toolkit.
//!
//! This crate provides everything needed to define Minecraft datapack elements
//! in Rust:
//!
//! - [`ResourceLocation`] — validated `namespace:path` identifiers
//! - [`DatapackComponent`] — trait implemented by all datapack element types
//! - [`cmd`] — typed command builders (`Execute`, `Selector`, `SetBlock`, etc.)
//!   plus auto-generated enums for `Item`, `Block`, `EntityType`, and more
//! - [`components`] — advancements, recipes, loot tables, predicates, item
//!   modifiers, tags, and custom items
//! - [`mcfunction!`] — advanced macro for command grouping and migration
//!
//! # Usage
//!
//! This crate is used alongside [`sand_macros`](https://docs.rs/sand-macros)
//! for the `#[function]` and `#[component]` proc macros:
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//! use sand_macros::{component, function};
//!
//! #[function]
//! pub fn greet() {
//!     cmd::tellraw(
//!         Selector::all_players(),
//!         Text::new("Hello from Sand!").gold().bold(true),
//!     );
//! }
//!
//! #[component(Load)]
//! pub fn on_load() {
//!     cmd::say("Datapack loaded");
//! }
//! ```
//!
//! # API tiers
//!
//! - [`prelude`] is the default user-facing import for datapack authors.
//! - [`advanced`] contains supported lower-level export hooks and raw escape
//!   hatches for custom integrations.
//! - [`compat`] names backwards-compatible exports that remain available while
//!   newer docs and examples avoid relying on them.
//! - `#[doc(hidden)]` items are for macro expansion and internal wiring, not
//!   a stable authoring surface.

pub mod advanced;
pub mod cmd;
pub mod compat;
pub mod component;
pub mod components;
pub mod condition;
pub mod custom_item_ext;
pub mod error;
pub mod event;
pub mod events;
pub mod execute_when;
pub mod function;
pub mod ir;
pub mod mc_version;
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
    CloneMode, Command, ConditionedExecute, Cooldown, EntityTargets, ExecuteExt, Fill, FillMode,
    ItemSlot, NbtStoreKind, NbtValue, Objective, ParticleEffect, ParticleSpread, PlayerTargets,
    ScoreCmp, SetBlock, SetBlockMode, SingleEntity, SinglePlayer, Sound, SoundSource, Storage,
    Title, TypedExecute,
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
pub use event::handle::{EventHandle, RawEventHandle};
pub use event::{
    AdvancementEvent,
    DamageAdvancementEvent,
    DamageEvent,
    Event,
    // Kept for backward compat; prefer Event<E> as handler context
    Event as TypedEvent,
    EventAdvancement,
    EventBuilder,
    EventConfig,
    EventId,
    EventPlayer,
    EventReset,
    EventVisibility,
    IntoEventAdvancement,
    IntoEventId,
};
#[allow(deprecated)]
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
    // Trait + dispatch
    SandEvent,
    SandEventDispatch,
    ShotCrossbowEvent,
    SlideDownBlockEvent,
    StartRidingEvent,
    SummonEntityEvent,
    TameAnimalEvent,
    TargetHitEvent,
    TotemActivateEvent,
    UseEnderEyeEvent,
    VillagerTradeEvent,
};
pub use function::{
    ArmorEventDescriptor, ArmorEventKind, ArmorSlot, ComponentFactory, EventDescriptor,
    EventDispatch, EventPathEntry, FunctionDescriptor, FunctionPointerEntry,
    FunctionPointerTypeEntry, FunctionTagDescriptor, IntoFunctionRef, ScheduleDescriptor,
    TempScoreboard, TrackedSource, TrackedTransition, TransitionKind, drain_dyn_fns,
    register_dyn_fn, register_dyn_fn_dedup,
};

mod transition;
pub use mc_version::McVersion;
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};
pub use state::{
    BlockNbt, EntityNbt, NbtLocation, NbtPath, SnbtCompound, SnbtValue, StorageField,
    StorageLocation, StorageSchema, StorageVar,
};
pub use state::{GameState, GameStateRef, TypedGameState};
pub use state::{
    StateDescriptor, StateLifecycle, drain_load_commands, drain_tick_commands,
    register_load_objective, register_tick_handler,
};
pub use vfx::{
    IntoParticleStep, IntoSoundStep, IntoVfxSelector, Vfx, VfxParticle, VfxSound, VfxStep,
};

/// Declare a typed state value and submit its automatic lifecycle descriptor.
///
/// Every entry uses the same [`StateLifecycle`] model available to callers of
/// [`inventory::submit!`]. Timer and cooldown ticking is opt-in through
/// `auto_tick`; defaults are initialized only for players missing a score.
///
/// ```rust,ignore
/// sand_core::sand_state! {
///     pub static MANA: ScoreVar<i32> = ScoreVar::new("mana") =>
///         MANA.lifecycle().default(100);
///     pub static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3)) =>
///         DASH.lifecycle().default(0).auto_tick();
/// }
/// ```
#[macro_export]
macro_rules! sand_state {
    ($(
        $(#[$meta:meta])*
        $vis:vis static $name:ident : $ty:ty = $value:expr => $lifecycle:expr;
    )+) => {
        $(
            $(#[$meta])*
            $vis static $name: $ty = $value;
            const _: () = {
                $crate::inventory::submit! {
                    $crate::StateDescriptor::new($lifecycle)
                }
            };
        )+
    };
}

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
    BlockId,
    // Typed predicate model
    BlockPredicate,
    ChatDecoration,
    ChatType,
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    Criterion,
    CustomData,
    CustomItem,
    // Damage
    DamageEffects,
    DamagePredicate,
    DamageScaling,
    DamageSourcePredicate,
    DamageTagEntry,
    DamageType,
    DamageTypeId,
    DeathMessageType,
    Dimension,
    DimensionId,
    DistancePredicate,
    DyedColor,
    EffectId,
    EffectPredicate,
    // Enchantment
    Enchantment,
    EnchantmentCost,
    EnchantmentEffect,
    EnchantmentEntry,
    EnchantmentId,
    EntityEquipment,
    EntityFlags,
    // Item predicates
    EntityPredicate,
    EntityTypeId,
    EntityTypeMatch,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    FloatRange,
    FoodProperties,
    FunctionId,
    Ingredient,
    // Instrument / Jukebox
    Instrument,
    IntRange,
    IntoRecipeItemId,
    InventorySlots,
    ItemComponent,
    ItemId,
    // Item modifier
    ItemModifier,
    ItemPredicate,
    ItemRarity,
    ItemStackComponents,
    JukeboxSong,
    LocationPredicate,
    // Loot table
    LootCondition,
    LootEntry,
    LootFunction,
    LootPool,
    LootTable,
    LootTableType,
    NoiseSettings,
    NumberProvider,
    PaintingVariant,
    PlacedFeature,
    PotionContents,
    PotionId,
    PotionRegistryId,
    // Predicate
    Predicate,
    Range,
    Rarity,
    // Raw escape hatch types
    RawCommand,
    RawComponent,
    RawJson,
    RawSnbt,
    RecipeResult,
    ShapedRecipe,
    ShapelessRecipe,
    SmithingTransformRecipe,
    SmithingTrimRecipe,
    StatusEffectId,
    StatusEffectInstance,
    StonecuttingRecipe,
    StructureId,
    StructureTemplate,
    SuspiciousStewEffect,
    // Tag
    Tag,
    TagEntry,
    TagId,
    TagRegistry,
    Ticks,
    ToolProperties,
    ToolRule,
    // Trim
    TrimMaterial,
    TrimPattern,
    TypedTag,
    // Wolf
    WolfVariant,
};

/// High-level typed damage command builder.
pub use sand_commands::Damage;

/// Register a temporary scoreboard objective that Sand creates automatically on load.
///
/// Eliminates the need to manually add `scoreboard objectives add` to your
/// `#[component(Load)]` function.
///
/// # Syntax
///
/// ```rust,ignore
/// temp_score!(my_obj);                         // criterion: dummy
/// temp_score!(my_obj, "playerKillCount");      // custom criterion
/// temp_score!(my_obj, "dummy", "My Display");  // criterion + display name
/// ```
///
/// Sand collects all registered objectives and emits them in a generated
/// `__sand_temp_scores` mcfunction injected into `minecraft:load`.
#[macro_export]
macro_rules! temp_score {
    ($name:ident) => {
        ::sand_core::inventory::submit!(::sand_core::TempScoreboard {
            name: stringify!($name),
            criteria: "dummy",
            display_name: ::std::option::Option::None,
        });
    };
    ($name:ident, $criteria:literal) => {
        ::sand_core::inventory::submit!(::sand_core::TempScoreboard {
            name: stringify!($name),
            criteria: $criteria,
            display_name: ::std::option::Option::None,
        });
    };
    ($name:ident, $criteria:literal, $display:literal) => {
        ::sand_core::inventory::submit!(::sand_core::TempScoreboard {
            name: stringify!($name),
            criteria: $criteria,
            display_name: ::std::option::Option::Some($display),
        });
    };
}

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
/// from [`sand_core::cmd`]) is serialized via `.to_string()`.
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
/// use sand_core::{mcfunction, cmd::Selector};
/// let cmds = mcfunction![
///     sand_core::cmd::say("Welcome!");
///     sand_core::cmd::kill(Selector::all_entities().tag("enemy"));
/// ];
/// ```
#[macro_export]
macro_rules! mcfunction {
    ($($cmd:expr);* $(;)?) => {{
        let mut _commands: Vec<String> = Vec::new();
        $(
            _commands.extend($crate::components::mc_function::IntoCommands::into_commands($cmd));
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

/// Generated block state property types.
///
/// Each block with configurable state properties gets a typed `*Properties`
/// struct. Shared property enums (e.g. `Facing`, `Half`) are generated once
/// and reused across blocks.
#[allow(warnings)]
pub mod block_states {
    include!(concat!(env!("OUT_DIR"), "/block_states.rs"));
}
