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
    EntityTargets, Execute, Inventory, InventorySlot, ObjectiveName, Particle, ParticleSpread,
    PlayerTargets, RawCommand, RenderCommand, ScoreHolder, Selector, SingleEntity, SinglePlayer,
    SlotPattern, SoundSource, Title, Validate,
};
pub use crate::vfx::{
    IntoParticleStep, IntoSoundStep, IntoVfxSelector, Vfx, VfxParticle, VfxSound, VfxStep,
};

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    BlockNbt, Cooldown, EntityNbt, Flag, FlagRef, GameState, GameStateRef, NbtLocation, NbtPath,
    ScoreRef, ScoreVar, SnbtCompound, SnbtValue, StorageField, StorageLocation, StorageSchema,
    StorageVar, Ticks, Timer, TypedGameState,
};

// ── Lifecycle registry ────────────────────────────────────────────────────────

pub use crate::state::define_registered_state;

// ── Optional systems ──────────────────────────────────────────────────────────

#[cfg(feature = "systems-damage")]
pub use crate::systems::damage::{DamageThreshold, DamageTracker, recently_damaged};

#[cfg(feature = "systems-player-data")]
pub use crate::systems::player_data::{PlayerDataSchema, PlayerSchema};

// ── Resource refs ─────────────────────────────────────────────────────────────

pub use crate::ResourceLocation;
pub use crate::resource_ref::{
    AdvancementRef, DialogRef, FunctionRef, LootTableRef, PredicateRef, RecipeRef,
};

// ── Version gating ────────────────────────────────────────────────────────────

pub use crate::version::{MinecraftVersion, VersionProfile};

// ── Entity queries and execution-scoped contexts ──────────────────────────────

pub use crate::entity::{
    AnyEntity, EntityContext, EntityKind, EntityQueries, EntityQuery, EntityScope, PlayerContext,
    PlayerKind, PlayerQueries, PlayerQuery, Relation, RelationQuery, ScopedEntityRef,
    SingleEntityQuery, SinglePlayerQuery,
};

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

pub use sand_components::dialog::{
    Dialog, DialogAction, DialogBody, DialogButton, DialogKind, DialogTag,
};

// ── Item/component builders ──────────────────────────────────────────────────

pub use sand_components::{
    Advancement, AdvancementDisplay, AdvancementFrame, AdvancementIcon, AdvancementRewards,
    AdvancementTrigger, AttributeId, AttributeModifier, AttributeOperation, AttributeType,
    BannerPattern, BlockPredicate, ConsumableAnimation, ConsumableProperties, Criterion,
    CustomData, CustomItem, DamagePredicate, DamageSourcePredicate, DistancePredicate,
    EnchantmentEntry, EntityEquipment, EntityFlags, EntityPredicate, EquipmentSlot,
    EquipmentSlotGroup, EquippableProperties, FoodProperties, Ingredient, ItemComponent,
    ItemModifier, ItemPredicate, ItemRarity, ItemStackComponents, LocationPredicate, LootCondition,
    LootEntry, LootFunction, LootPool, LootTable, LootTableType, Predicate, Rarity, RecipeResult,
    ShapedRecipe, ShapelessRecipe, SmithingTransformRecipe, SmithingTrimRecipe, StonecuttingRecipe,
    Tag, TagEntry, TagRegistry, ToolProperties, ToolRule, TypedTag,
};

// ── Raw escape hatch types ────────────────────────────────────────────────────

pub use sand_components::{RawComponent, RawJson, RawSnbt};

// ── Typed registry identifiers ────────────────────────────────────────────────

pub use sand_components::{
    BiomeId, BlockId, DamageTypeId, DimensionId, EffectId, EnchantmentId, EntityTypeId, FunctionId,
    ItemId, PotionContents, PotionId, PotionRegistryId, Range, StatusEffectId,
    StatusEffectInstance, StructureId, StructureTemplate, SuspiciousStewEffect, TagId,
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

    #[test]
    fn prelude_exports_define_registered_state() {
        let _drain: fn() -> Vec<String> = define_registered_state;
    }

    #[test]
    fn prelude_exports_typed_game_state() {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Phase {
            Idle = 0,
        }

        impl TypedGameState for Phase {
            fn to_score(self) -> i32 {
                self as i32
            }

            fn from_score(score: i32) -> Option<Self> {
                match score {
                    0 => Some(Self::Idle),
                    _ => None,
                }
            }
        }

        static PHASE: GameState<Phase> = GameState::with_default_score("phase", 0);

        let _state_ref: GameStateRef<'_, Phase> = PHASE.of("@s");
        assert_eq!(PHASE.of("@s").reset(), "scoreboard players set @s phase 0");
    }

    #[test]
    fn prelude_exports_vfx_types() {
        let commands = Vfx::new("prelude")
            .particle(VfxParticle::happy_villager().count(2))
            .sound(VfxSound::new("minecraft:block.note_block.bell").source(SoundSource::Player))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:happy_villager ~0 ~0 ~0 0 0 0 0 2 force",
                "execute at @s run playsound minecraft:block.note_block.bell player @s ~ ~ ~ 1 1",
            ]
        );
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
        static REGEN_TIMER: Timer = Timer::new("regen", Ticks::seconds(2));
        static CAST_COOLDOWN: Cooldown = Cooldown::new("cast_cd", Ticks::seconds(3));

        let schema = PlayerDataSchema::new("magic")
            .score(&MANA, 100)
            .flag(&HAS_WAND, false)
            .timer(&REGEN_TIMER)
            .cooldown(&CAST_COOLDOWN);

        assert_eq!(schema.scoreboard_field_count(), 4);
        assert_eq!(
            schema.define_all(),
            vec![
                "scoreboard objectives add mana dummy",
                "scoreboard objectives add has_wand dummy",
                "scoreboard objectives add regen dummy",
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
