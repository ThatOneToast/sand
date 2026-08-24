//! Datapack component builders.
//!
//! All types except [`McFunction`] come directly from `sand-components`.
//! This module re-exports them so that `sand-core` provides a single
//! import surface for downstream users.

pub(crate) mod mc_function;

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
    BlockState,
    BlockStateProvider,
    // Animal variants
    ChickenVariant,
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    CowVariant,
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
    // Standalone predicate authoring
    EntityPredicateTarget,
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
    Ingredient,
    // Item modifier
    ItemModifier,
    ItemOrTag,
    ItemPredicate,
    ItemRarity,
    // Item stack
    ItemStack,
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
    PigVariant,
    // Worldgen: template pools
    PoolElement,
    PoolEntry,
    // Predicate
    Predicate,
    PredicateId,
    PredicateRoot,
    // Worldgen: processor lists
    Processor,
    ProcessorList,
    ProcessorRule,
    ProcessorsRef,
    Projection,
    // Villager trades
    RandomSequenceId,
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
    TagId,
    TemplatePool,
    TerrainAdaptation,
    ToolProperties,
    ToolRule,
    TradeItem,
    TradeSet,
    TradeSetId,
    TrimAssetName,
    TrimMaterial,
    TrimPattern,
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
    WeightedBlockState,
};
