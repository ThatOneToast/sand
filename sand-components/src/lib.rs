//! # sand-components
//!
//! Typed JSON builders for every Minecraft 1.21.x datapack component type.
//!
//! ## Module Overview
//!
//! | Module              | Component directory                        | Key types |
//! |--------------------|--------------------------------------------|-----------|
//! | [`advancement`]    | `advancement/`                             | [`Advancement`], [`AdvancementTrigger`], … |
//! | [`banner_pattern`] | `banner_pattern/`                          | [`BannerPattern`] |
//! | [`chat_type`]      | `chat_type/`                               | [`ChatType`], [`ChatDecoration`] |
//! | [`damage_type`]    | `damage_type/`                             | [`DamageType`], [`DamageScaling`], … |
//! | [`enchantment`]    | `enchantment/`                             | [`Enchantment`], [`EnchantmentCost`] |
//! | [`instrument`]     | `instrument/`                              | [`Instrument`] |
//! | [`item`]           | *(item component strings)*                 | [`CustomItem`], [`FoodProperties`], … |
//! | [`item_modifier`]  | `item_modifier/`                           | [`ItemModifier`] |
//! | [`jukebox_song`]   | `jukebox_song/`                            | [`JukeboxSong`] |
//! | [`loot_table`]     | `loot_table/`                              | [`LootTable`], [`LootPool`], [`LootEntry`], … |
//! | [`painting_variant`] | `painting_variant/`                      | [`PaintingVariant`] |
//! | [`predicate`]      | `predicate/`                               | [`Predicate`] |
//! | [`recipe`]         | `recipe/`                                  | [`ShapedRecipe`], [`ShapelessRecipe`], … |
//! | [`tag`]            | `tags/`                                    | [`Tag`] |
//! | [`trim`]           | `trim_material/`, `trim_pattern/`          | [`TrimMaterial`], [`TrimPattern`] |
//! | [`wolf_variant`]   | `wolf_variant/`                            | [`WolfVariant`] |
//! | [`worldgen`]       | `worldgen/biome/`, `dimension/`, …         | [`Biome`], [`Dimension`], … |

pub mod advancement;
pub mod banner_pattern;
pub mod chat_type;
pub mod component;
pub mod damage_type;
pub mod enchantment;
pub mod error;
pub mod instrument;
pub mod item;
pub mod item_modifier;
pub mod jukebox_song;
pub mod loot_table;
pub mod painting_variant;
pub mod predicate;
pub mod recipe;
pub mod resource_location;
pub mod tag;
pub mod trim;
pub mod wolf_variant;
pub mod worldgen;

// ── Core traits and types ─────────────────────────────────────────────────────

pub use component::{ComponentContent, DatapackComponent, IntoDatapack};
pub use error::{Result, SandError};
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};

// ── Advancement ───────────────────────────────────────────────────────────────

pub use advancement::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, Criterion,
};

// ── Banner Pattern ────────────────────────────────────────────────────────────

pub use banner_pattern::BannerPattern;

// ── Chat Type ─────────────────────────────────────────────────────────────────

pub use chat_type::{ChatDecoration, ChatType};

// ── Damage Type ───────────────────────────────────────────────────────────────

pub use damage_type::{DamageEffects, DamageScaling, DamageType, DeathMessageType};

// ── Enchantment ───────────────────────────────────────────────────────────────

pub use enchantment::{Enchantment, EnchantmentCost, EnchantmentEffect};

// ── Instrument ────────────────────────────────────────────────────────────────

pub use instrument::Instrument;

// ── Item ──────────────────────────────────────────────────────────────────────

pub use item::predicates::{EntityPredicate, InventorySlots, ItemPredicate};
pub use item::{
    AttributeModifier, AttributeOperation, AttributeType, ConsumableAnimation,
    ConsumableProperties, CustomItem, DyedColor, EquipmentSlot, EquipmentSlotGroup,
    EquippableProperties, FoodProperties, ItemRarity, ToolProperties, ToolRule,
};

// ── Item Modifier ─────────────────────────────────────────────────────────────

pub use item_modifier::ItemModifier;

// ── Jukebox Song ──────────────────────────────────────────────────────────────

pub use jukebox_song::JukeboxSong;

// ── Loot Table ────────────────────────────────────────────────────────────────

pub use loot_table::{
    LootCondition, LootEntry, LootFunction, LootPool, LootTable, LootTableType, NumberProvider,
};

// ── Painting Variant ──────────────────────────────────────────────────────────

pub use painting_variant::PaintingVariant;

// ── Predicate ─────────────────────────────────────────────────────────────────

pub use predicate::Predicate;

// ── Recipes ───────────────────────────────────────────────────────────────────

pub use recipe::{
    CookingRecipe, CookingType, Ingredient, RecipeResult, ShapedRecipe, ShapelessRecipe,
    SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
};

// ── Tag ───────────────────────────────────────────────────────────────────────

pub use tag::Tag;

// ── Trim ──────────────────────────────────────────────────────────────────────

pub use trim::{TrimMaterial, TrimPattern};

// ── Wolf Variant ──────────────────────────────────────────────────────────────

pub use wolf_variant::WolfVariant;

// ── Worldgen ──────────────────────────────────────────────────────────────────

pub use worldgen::biome::BiomeEffects;
pub use worldgen::{Biome, Dimension, NoiseSettings, PlacedFeature};
