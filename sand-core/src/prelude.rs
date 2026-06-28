//! Default user-facing Sand API.
//!
//! Bring the whole prelude into scope with:
//! ```rust,ignore
//! use sand_core::prelude::*;
//! ```
//!
//! The prelude is the recommended import for ordinary datapack authoring. It
//! covers typed functions, components, commands, selectors, state, storage,
//! events, common component builders, text, resource references, and deliberate
//! raw escape hatches. Reach for [`crate::advanced`] only when you need
//! lower-level export registries or custom framework integration points.

pub use crate::{all, any, cmd, mcfunction};

// ── Conditions & execute wiring ───────────────────────────────────────────────

pub use crate::cmd::{ConditionedExecute, ExecuteExt, TypedExecute};
pub use crate::condition::{Condition, ExecutePlan};
pub use crate::execute_when::{if_, unless, when};

// ── Command builders ──────────────────────────────────────────────────────────

pub use crate::Damage;
#[allow(deprecated)]
pub use crate::cmd::{
    Actionbar, Bossbar, BossbarColor, BossbarStyle, DamageAmount, DamageBuilder, DamageKind,
    EntityTargets, Execute, Inventory, InventorySlot, PlayerTargets, Selector, SingleEntity,
    SinglePlayer, SlotPattern, Title,
};

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    BlockNbt, Cooldown, EntityNbt, Flag, FlagRef, NbtLocation, NbtPath, ScoreRef, ScoreVar,
    SnbtCompound, SnbtValue, StorageField, StorageLocation, StorageSchema, StorageVar, Ticks,
    Timer,
};

// ── Optional systems ──────────────────────────────────────────────────────────

#[cfg(feature = "systems-damage")]
pub use crate::systems::damage::{DamageThreshold, DamageTracker, recently_damaged};

#[cfg(feature = "systems-player-data")]
pub use crate::systems::player_data::PlayerSchema;

// ── Resource refs ─────────────────────────────────────────────────────────────

pub use crate::ResourceLocation;
pub use crate::resource_ref::{
    AdvancementRef, DialogRef, FunctionRef, LootTableRef, PredicateRef, RecipeRef,
};

// ── Version gating ────────────────────────────────────────────────────────────

pub use crate::version::{MinecraftVersion, VersionProfile};

// ── Function refs (IntoFunctionRef trait) ──────────────────────────────────────

pub use crate::function::IntoFunctionRef;

// ── Typed event model ─────────────────────────────────────────────────────────

pub use crate::event::handle::EventHandle;
pub use crate::event::trigger::{
    ConsumeItemTrigger, EntityKilledPlayerTrigger, ImpossibleTrigger, InventoryChangedTrigger,
    ItemEnchantTrigger, ItemObtainedTrigger, MultiKillTrigger, PlayerInteractedWithEntityTrigger,
    PlayerKilledEntityTrigger, RecipeUnlockedTrigger, SummonedEntityTrigger, TickTrigger,
    UsingItemTrigger,
};
pub use crate::event::{
    AdvancementEvent, DamageAdvancementEvent, DamageEvent, Event, EventBuilder, EventConfig,
    EventId, EventPlayer, EventReset, EventVisibility, IntoEventAdvancement,
};

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{Dialog, DialogAction, DialogBody, DialogButton, DialogKind};

// ── Item/component builders ──────────────────────────────────────────────────

pub use sand_components::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, AttributeId, AttributeModifier, AttributeOperation, AttributeType,
    BannerPattern, BlockPredicate, ConsumableAnimation, ConsumableProperties, Criterion,
    CustomData, CustomItem, DamagePredicate, DamageSourcePredicate, DistancePredicate,
    EnchantmentEntry, EntityEquipment, EntityFlags, EntityPredicate, EquipmentSlot,
    EquipmentSlotGroup, EquippableProperties, FoodProperties, Ingredient, ItemComponent,
    ItemModifier, ItemPredicate, ItemRarity, LocationPredicate, LootCondition, LootEntry,
    LootFunction, LootPool, LootTable, LootTableType, Predicate, Rarity, RecipeResult,
    ShapedRecipe, ShapelessRecipe, SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
    Tag, ToolProperties, ToolRule,
};

// ── Raw escape hatch types ────────────────────────────────────────────────────

pub use sand_components::{RawCommand, RawComponent, RawJson, RawSnbt};

// ── Typed registry identifiers ────────────────────────────────────────────────

pub use sand_components::{
    BiomeId, BlockId, DamageTypeId, DimensionId, EffectId, EnchantmentId, EntityTypeId, ItemId,
    PotionContents, PotionId, Range, StatusEffectInstance, StructureId, StructureTemplate,
    SuspiciousStewEffect, TagId,
};

// ── Text / chat ───────────────────────────────────────────────────────────────

pub use sand_commands::{ChatColor, ClickEvent, HoverEvent, Text, TextComponent};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prelude_exports_typed_command_path() {
        let cmd = cmd::tellraw(
            Selector::all_players(),
            Text::new("Hello from Sand").gold().bold(true),
        )
        .to_string();

        assert!(cmd.starts_with("tellraw @a "));
        assert!(cmd.contains(r#""text":"Hello from Sand""#));
        assert!(cmd.contains(r#""color":"gold""#));
        assert!(cmd.contains(r#""bold":true"#));
    }

    #[test]
    fn prelude_exports_resource_locations_for_function_refs() {
        let id = ResourceLocation::new("example", "start").unwrap();
        assert_eq!(cmd::function(id).to_string(), "function example:start");
    }

    #[cfg(feature = "systems-damage")]
    #[test]
    fn prelude_exports_damage_threshold_with_damage_system() {
        let current: Condition =
            DamageTracker::current_damage_at_least("@s", DamageThreshold::hearts(1.0));
        let last: Condition =
            DamageTracker::last_damage_at_least("@s", DamageThreshold::raw_stat(10));

        assert_eq!(DamageThreshold::hearts(1.0).to_raw_stat(), 10);
        assert!(matches!(current, Condition::Score { .. }));
        assert!(matches!(last, Condition::Score { .. }));
    }

    #[cfg(feature = "systems-player-data")]
    #[test]
    fn prelude_exports_manual_player_schema_contract() {
        static MANA: ScoreVar<i32> = ScoreVar::new("mana");
        static HAS_WAND: Flag = Flag::new("has_wand");
        static CAST_COOLDOWN: Cooldown = Cooldown::new("cast_cd", Ticks::seconds(3));

        let schema = PlayerSchema::new("magic")
            .score(&MANA, 100)
            .flag(&HAS_WAND, false)
            .cooldown(&CAST_COOLDOWN);

        assert_eq!(schema.scoreboard_field_count(), 3);
        assert_eq!(
            schema.define_all(),
            vec![
                "scoreboard objectives add mana dummy",
                "scoreboard objectives add has_wand dummy",
                "scoreboard objectives add cast_cd dummy",
            ]
        );
        assert_eq!(
            schema.init_player("@s"),
            vec![
                "execute unless score @s mana matches -2147483648.. run scoreboard players set @s mana 100",
                "execute unless score @s has_wand matches -2147483648.. run scoreboard players set @s has_wand 0",
            ]
        );
    }
}
