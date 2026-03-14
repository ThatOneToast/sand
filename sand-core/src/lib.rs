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
pub mod function;
pub mod mc_version;
pub mod resource_location;

pub use cmd::{
    Actionbar, BlockState, Bossbar, BossbarColor, BossbarStyle,
    CloneBlocks, CloneMode, CloneMaskMode,
    Command, Cooldown, Fill, FillMode,
    NbtValue, Objective, ParticleEffect, ParticleSpread,
    SetBlock, SetBlockMode, Sound, SoundSource, Storage, Title,
};
pub use component::{ComponentContent, ComponentRecord, DatapackComponent, IntoDatapack};
pub use component::export_components_json;
pub use components::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, Criterion, CookingRecipe, CookingType, Ingredient, ItemModifier,
    LootCondition, LootEntry, LootFunction, LootPool, LootTable, LootTableType, McFunction,
    NumberProvider, Predicate, RecipeResult, ShapedRecipe, ShapelessRecipe,
    SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe, Tag,
    AttributeModifier, AttributeOperation, AttributeType, ConsumableAnimation,
    ConsumableProperties, CustomItem, DyedColor, EquipmentSlot, EquipmentSlotGroup,
    EquippableProperties, FoodProperties, ItemRarity, ToolProperties, ToolRule,
};
pub use error::{Result, SandError};
pub use function::{ComponentFactory, FunctionDescriptor, FunctionTagDescriptor};
pub use mc_version::McVersion;
pub use resource_location::{Identifier, PackNamespace, ResourceLocation};

/// Re-exported so proc macros can write `::sand_core::inventory::submit!`
/// without requiring users to add `inventory` as a direct dependency.
#[doc(hidden)]
pub use inventory;

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
    ($($cmd:expr);* $(;)?) => {
        vec![$($cmd.to_string()),*]
    };
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
