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
    EquipmentModelId,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    FoodProperties,
    Ingredient,
    InventorySlots,
    // Item modifier
    ItemModifier,
    ItemOrTag,
    ItemPredicate,
    ItemRarity,
    LevelBasedValue,
    // Loot table
    LootCondition,
    LootEntry,
    LootFunction,
    LootPool,
    LootTable,
    LootTableType,
    NumberProvider,
    PigVariant,
    // Predicate
    Predicate,
    RecipeResult,
    ShapedRecipe,
    ShapelessRecipe,
    SmithingTransformRecipe,
    SmithingTrimRecipe,
    SoundEventId,
    StonecuttingRecipe,
    // Tag
    Tag,
    ToolProperties,
    ToolRule,
    TrimAssetName,
    TrimMaterial,
    TrimPattern,
};
