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
//! - [`mcfunction!`] — macro for building command lists
//! - [`cmd`] — typed command builders (`Execute`, `Selector`, `SetBlock`, etc.)
//!   plus auto-generated enums for `Item`, `Block`, `EntityType`, and more
//! - [`components`] — advancements, recipes, loot tables, predicates, item
//!   modifiers, tags, and custom items
//!
//! # Usage
//!
//! This crate is used alongside [`sand_macros`](https://docs.rs/sand-macros)
//! for the `#[function]` and `#[component]` proc macros:
//!
//! ```rust,ignore
//! use sand_core::mcfunction;
//! use sand_macros::{component, function};
//!
//! #[function]
//! pub fn greet() {
//!     mcfunction! {
//!         r#"tellraw @a {"text":"Hello!","color":"gold"}"#;
//!     }
//! }
//! ```

pub mod cmd;
pub mod component;
pub mod components;
pub mod error;
pub mod events;
pub mod function;
pub mod mc_version;
pub mod resource_location;

// ── Re-export the sand-components crate itself ────────────────────────────────

/// The `sand-components` crate — typed JSON builders for every datapack component type.
pub use sand_components;

// ── Core infrastructure ───────────────────────────────────────────────────────

pub use cmd::{
    Actionbar, BlockState, Bossbar, BossbarColor, BossbarStyle, CloneBlocks, CloneMaskMode,
    CloneMode, Command, Cooldown, Fill, FillMode, ItemSlot, NbtStoreKind, NbtValue, Objective,
    ParticleEffect, ParticleSpread, ScoreCmp, SetBlock, SetBlockMode, Sound, SoundSource, Storage,
    Title,
};
pub use component::export_components_json;
pub use component::{ComponentContent, ComponentRecord, DatapackComponent, IntoDatapack};
pub use error::{Result, SandError};
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
    EventDispatch, FunctionDescriptor, FunctionTagDescriptor, ScheduleDescriptor, TempScoreboard,
    drain_dyn_fns, register_dyn_fn,
};
pub use mc_version::McVersion;
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};

// ── McFunction (sand-core-specific component) ─────────────────────────────────

pub use components::mc_function::{IntoCommands, McFunction};

// ── Datapack component builders (all from sand-components) ───────────────────

pub use sand_components::{
    // Advancement
    Advancement,
    AdvancementDisplay,
    AdvancementFrame,
    AdvancementIcon,
    AdvancementRewards,
    AdvancementTrigger,
    // Custom item
    AttributeModifier,
    AttributeOperation,
    AttributeType,
    // Banner / Painting / Chat
    BannerPattern,
    // Worldgen
    Biome,
    BiomeEffects,
    ChatDecoration,
    ChatType,
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    Criterion,
    CustomItem,
    // Damage
    DamageEffects,
    DamageScaling,
    DamageType,
    DeathMessageType,
    Dimension,
    DyedColor,
    // Enchantment
    Enchantment,
    EnchantmentCost,
    EnchantmentEffect,
    // Item predicates
    EntityPredicate,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    FoodProperties,
    Ingredient,
    // Instrument / Jukebox
    Instrument,
    InventorySlots,
    // Item modifier
    ItemModifier,
    ItemPredicate,
    ItemRarity,
    JukeboxSong,
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
    // Predicate
    Predicate,
    RecipeResult,
    ShapedRecipe,
    ShapelessRecipe,
    SmithingTransformRecipe,
    SmithingTrimRecipe,
    StonecuttingRecipe,
    // Tag
    Tag,
    ToolProperties,
    ToolRule,
    // Trim
    TrimMaterial,
    TrimPattern,
    // Wolf
    WolfVariant,
};

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

/// Generated Minecraft registry enums (`Item`, `Block`, `EntityType`, etc.).
///
/// Populated at build time by `sand-build` for the Minecraft version specified
/// in the `SAND_MC_VERSION` environment variable (default: `1.21.11`).
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

/// Generated block state property types.
///
/// Each block with configurable state properties gets a typed `*Properties`
/// struct. Shared property enums (e.g. `Facing`, `Half`) are generated once
/// and reused across blocks.
#[allow(warnings)]
pub mod block_states {
    include!(concat!(env!("OUT_DIR"), "/block_states.rs"));
}
