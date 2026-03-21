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
    ConsumableAnimation,
    ConsumableProperties,
    // Recipes
    CookingRecipe,
    CookingType,
    Criterion,
    CustomItem,
    DyedColor,
    // Item predicates
    EntityPredicate,
    EquipmentSlot,
    EquipmentSlotGroup,
    EquippableProperties,
    FoodProperties,
    Ingredient,
    InventorySlots,
    // Item modifier
    ItemModifier,
    ItemPredicate,
    ItemRarity,
    // Loot table
    LootCondition,
    LootEntry,
    LootFunction,
    LootPool,
    LootTable,
    LootTableType,
    NumberProvider,
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
};
