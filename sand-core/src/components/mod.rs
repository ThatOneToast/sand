//! Datapack component builders.
//!
//! All types except [`McFunction`] come directly from `sand-components`.
//! This module re-exports them so that `sand-core` provides a single
//! import surface for downstream users.

pub mod mc_function;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use mc_function::{IntoCommands, McFunction};

pub use sand_components::{
    // Advancement
    Advancement,
    AdvancementDisplay,
    AdvancementFrame,
    AdvancementIcon,
    AdvancementRewards,
    AdvancementTrigger,
    // Item / custom item
    AttributeModifier,
    AttributeOperation,
    AttributeType,
    // Worldgen: structures
    BiomeSelector,
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    Criterion,
    CustomItem,
    DyedColor,
    // Enchantment
    Enchantment,
    EnchantmentCost,
    EnchantmentEffectComponentId,
    EnchantmentOrTag,
    EnchantmentProvider,
    EnchantmentProviderInt,
    EnchantmentSelection,
    EnchantmentValueOperation,
    // Item predicates
    EntityPredicate,
    EquipmentModelId,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    ExclusionZone,
    FoodProperties,
    FrequencyReductionMethod,
    GenerationStep,
    HeightProvider,
    // Worldgen: shared providers
    Heightmap,
    BlockState,
    BlockStateProvider,
    WeightedBlockState,
    Ingredient,
    InventorySlots,
    // Item modifier
    ItemModifier,
    ItemOrTag,
    ItemPredicate,
    ItemRarity,
    JigsawConfig,
    LevelBasedValue,
    // Loot table
    LootCondition,
    LootEntry,
    LootFunction,
    LootPool,
    LootTable,
    LootTableType,
    MobCategory,
    NumberProvider,
    // Worldgen: template pools
    PoolElement,
    PoolEntry,
    // Predicate
    Predicate,
    // Worldgen: processor lists
    Processor,
    ProcessorList,
    ProcessorRule,
    ProcessorsRef,
    Projection,
    RecipeResult,
    ShapedRecipe,
    ShapelessRecipe,
    SmithingTransformRecipe,
    SmithingTrimRecipe,
    SoundEventId,
    SpawnBoundingBox,
    SpawnEntry,
    SpawnOverride,
    SpreadType,
    StonecuttingRecipe,
    // Worldgen: structure sets
    Structure,
    StructureEntry,
    StructurePlacement,
    StructureSet,
    // Tag
    Tag,
    TemplatePool,
    TerrainAdaptation,
    ToolProperties,
    ToolRule,
    TrimAssetName,
    TrimMaterial,
    TrimPattern,
    VerticalAnchor,
};
