//! # sand-components
//!
//! Typed JSON builders for every Minecraft 1.21.x datapack component type.
//!
//! ## Module Overview
//!
//! | Module              | Component directory                        | Key types |
//! |--------------------|--------------------------------------------|-----------|
//! | [`advancement`]    | `advancement/`                             | [`Advancement`], [`AdvancementTrigger`], … |
//! | [`animal_variant`] | *(shared spawn-condition model)*           | [`animal_variant::SpawnCondition`] |
//! | [`banner_pattern`] | `banner_pattern/`                          | [`BannerPattern`] |
//! | [`chat_type`]      | `chat_type/`                               | [`ChatType`], [`ChatDecoration`] |
//! | [`chicken_variant`] | `chicken_variant/`                        | [`ChickenVariant`] |
//! | [`cow_variant`]    | `cow_variant/`                             | [`CowVariant`] |
//! | [`damage_type`]    | `damage_type/`                             | [`DamageType`], [`DamageScaling`], … |
//! | [`enchantment`]    | `enchantment/`                             | [`Enchantment`], [`EnchantmentCost`] |
//! | [`enchantment_provider`] | `enchantment_provider/`              | [`EnchantmentProvider`], [`EnchantmentSelection`] |
//! | [`instrument`]     | `instrument/`                              | [`Instrument`] |
//! | [`item`]           | *(item component strings)*                 | [`CustomItem`], [`FoodProperties`], … |
//! | [`item_modifier`]  | `item_modifier/`                           | [`ItemModifier`] |
//! | [`jukebox_song`]   | `jukebox_song/`                            | [`JukeboxSong`] |
//! | [`loot_table`]     | `loot_table/`                              | [`LootTable`], [`LootPool`], [`LootEntry`], … |
//! | [`painting_variant`] | `painting_variant/`                      | [`PaintingVariant`] |
//! | [`pig_variant`]    | `pig_variant/`                             | [`PigVariant`] |
//! | [`predicate`]      | `predicate/`                               | [`Predicate`], [`PredicateRoot`], [`EntityPredicateTarget`] |
//! | [`recipe`]         | `recipe/`                                  | [`ShapedRecipe`], [`ShapelessRecipe`], … |
//! | [`structure_template`] | `structure/`                          | [`StructureTemplate`] |
//! | [`tag`]            | `tags/`                                    | [`Tag`] |
//! | [`trim`]           | `trim_material/`, `trim_pattern/`          | [`TrimMaterial`], [`TrimPattern`] |
//! | [`wolf_variant`]   | `wolf_variant/`                            | [`WolfVariant`] |
//! | [`worldgen`]       | `worldgen/biome/`, `dimension/`, …         | [`Biome`], [`Dimension`], … |

pub mod advancement;
pub mod animal_variant;
pub mod banner_pattern;
pub mod chat_type;
pub mod chicken_variant;
pub mod component;
pub mod cow_variant;
pub mod damage_type;
pub mod dialog;
pub mod effect;
pub mod enchantment;
pub mod enchantment_provider;
pub mod error;
pub mod instrument;
pub mod item;
pub mod item_modifier;
pub mod jukebox_song;
pub mod loot_table;
pub mod painting_variant;
pub mod pig_variant;
pub mod predicate;
pub mod predicates;
pub mod raw;
pub mod recipe;
pub mod registry;
pub mod registry_coverage;
pub mod resource_location;
pub mod structure_template;
pub mod tag;
pub mod trim;
pub(crate) mod validation;
pub mod wolf_variant;
pub mod worldgen;

// ── Core traits and types ─────────────────────────────────────────────────────

pub use component::{ComponentContent, DatapackComponent, IntoDatapack};
pub use effect::{
    EffectId, PotionContents, PotionId, StatusEffectInstance, SuspiciousStewEffect, Ticks,
};
pub use raw::{RawCommand, RawComponent, RawJson, RawSnbt};

// ── Shared typed predicate model ──────────────────────────────────────────────

pub use error::{Result, SandError};
pub use predicates::{
    BlockPredicate, DamagePredicate, DamageSourcePredicate, DamageTagEntry, DistancePredicate,
    EffectPredicate, EntityEquipment, EntityFlags, EntityPredicate, EntityTypeMatch, FloatRange,
    IntRange, ItemPredicate, LocationPredicate, Range, WeatherPredicate,
};
pub use registry::{
    AdvancementId, BiomeId, BlockId, ChickenVariantId, ConfiguredFeatureId, CowVariantId,
    DamageTypeId, DensityFunctionId, DimensionId, DimensionTypeId, EnchantmentEffectComponentId,
    EnchantmentId, EntityTypeId, EquipmentModelId, FunctionId, ItemId, LootTableId, NoiseId,
    PigVariantId, PotionRegistryId, PredicateId, ProcessorListId, RecipeId, SoundEventId,
    StatusEffectId, StructureId, StructureSetId, StructureTemplateId, StructureTypeId, TagId,
    TemplatePoolId,
};
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};

// ── Advancement ───────────────────────────────────────────────────────────────

pub use advancement::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, Criterion,
};

// ── Animal Variants ───────────────────────────────────────────────────────────

pub use animal_variant::SpawnCondition;
pub use chicken_variant::ChickenVariant;
pub use cow_variant::CowVariant;
pub use pig_variant::PigVariant;

// ── Banner Pattern ────────────────────────────────────────────────────────────

pub use banner_pattern::BannerPattern;

// ── Chat Type ─────────────────────────────────────────────────────────────────

pub use chat_type::{ChatDecoration, ChatDecorationParameter, ChatStyle, ChatType};

// ── Damage Type ───────────────────────────────────────────────────────────────

pub use damage_type::{DamageEffects, DamageScaling, DamageType, DeathMessageType};

// ── Enchantment ───────────────────────────────────────────────────────────────

pub use enchantment::{
    Enchantment, EnchantmentCost, EnchantmentOrTag, EnchantmentValueOperation, ItemOrTag,
    LevelBasedValue,
};
pub use enchantment_provider::{EnchantmentProvider, EnchantmentProviderInt, EnchantmentSelection};

// ── Instrument ────────────────────────────────────────────────────────────────

pub use instrument::Instrument;

// ── Item ──────────────────────────────────────────────────────────────────────

pub use item::definition::CustomItemDefinition;
pub use item::matcher::{IntoItemMatcher, ItemMatcher, ItemMatcherConsumer, TryIntoItemPredicate};
pub use item::predicates::InventorySlots;
pub use item::stack::{IntoItemStack, ItemStack, MAX_STACK_SIZE};
pub use item::{
    AttributeId, AttributeModifier, AttributeOperation, AttributeType, ConsumableAnimation,
    ConsumableProperties, CustomData, CustomItem, DyedColor, EnchantmentEntry, EquipmentSlot,
    EquipmentSlotGroup, EquippableProperties, FoodProperties, ItemComponent, ItemRarity,
    ItemStackComponents, Rarity, ToolProperties, ToolRule,
};

// ── Item Modifier ─────────────────────────────────────────────────────────────

pub use item_modifier::ItemModifier;

// ── Jukebox Song ──────────────────────────────────────────────────────────────

pub use jukebox_song::JukeboxSong;

// ── Loot Table ────────────────────────────────────────────────────────────────

pub use loot_table::{
    EnchantmentSelector, LootCondition, LootEntry, LootFunction, LootPool, LootTable,
    LootTableType, LootText, NumberProvider,
};

// ── Painting Variant ──────────────────────────────────────────────────────────

pub use painting_variant::PaintingVariant;

// ── Predicate ─────────────────────────────────────────────────────────────────

pub use predicate::{EntityPredicateTarget, Predicate, PredicateRoot};

// ── Recipes ───────────────────────────────────────────────────────────────────

pub use recipe::{
    CookingRecipe, CookingType, Ingredient, IntoRecipeItemId, RecipeResult, ShapedRecipe,
    ShapelessRecipe, SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
    TryIntoIngredient, TryIntoRecipeResult,
};

// ── Dialog ────────────────────────────────────────────────────────────────────

pub use dialog::{Dialog, DialogAction, DialogBody, DialogButton, DialogKind, DialogTag};

// ── Tag ───────────────────────────────────────────────────────────────────────

pub use tag::{Tag, TagEntry, TagRegistry, TypedTag};

// ── Structure Templates ───────────────────────────────────────────────────────

pub use structure_template::StructureTemplate;

// ── Trim ──────────────────────────────────────────────────────────────────────

pub use trim::{TrimAssetName, TrimMaterial, TrimPattern};

// ── Wolf Variant ──────────────────────────────────────────────────────────────

pub use wolf_variant::WolfVariant;

// ── Worldgen ──────────────────────────────────────────────────────────────────

pub use worldgen::biome::BiomeEffects;
pub use worldgen::providers::{BlockState, BlockStateProvider, WeightedBlockState};
pub use worldgen::{
    Biome, BiomeSelector, ConfiguredFeature, DensityFunction, DensityFunctionBinaryOp,
    DensityFunctionExpr, DensityFunctionUnaryOp, Dimension, DimensionType, ExclusionZone,
    FrequencyReductionMethod, GenerationStep, HeightProvider, Heightmap, JigsawConfig, MobCategory,
    MonsterSpawnLightLevel, Noise, NoiseSettings, OreConfig, OreTarget, PlacedFeature, PoolElement,
    PoolEntry, Processor, ProcessorList, ProcessorRule, ProcessorsRef, Projection, RuleTest,
    SpawnBoundingBox, SpawnEntry, SpawnOverride, SpreadType, Structure, StructureEntry,
    StructurePlacement, StructureSet, TemplatePool, TerrainAdaptation, VerticalAnchor,
};
