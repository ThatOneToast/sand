//! Convenience re-export of the most commonly used Sand types.
//!
//! Bring the whole prelude into scope with:
//! ```rust,ignore
//! use sand_core::prelude::*;
//! ```

pub use crate::{all, any, cmd, mcfunction};

// ── Conditions & execute wiring ───────────────────────────────────────────────

pub use crate::cmd::{ConditionedExecute, ExecuteExt, TypedExecute};
pub use crate::condition::{Condition, ExecutePlan};
pub use crate::execute_when::{if_, unless, when};

// ── Command builders ──────────────────────────────────────────────────────────

pub use crate::Damage;
pub use crate::cmd::{
    Actionbar, Bossbar, BossbarColor, BossbarStyle, DamageAmount, DamageBuilder, DamageKind,
    EntityTargets, Execute, Inventory, InventorySlot, PlayerTargets, Selector, SingleEntity,
    SinglePlayer, SlotPattern, Title,
};

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    Cooldown, Flag, FlagRef, NbtPath, ScoreRef, ScoreVar, StorageVar, Ticks, Timer,
};

// ── Optional systems ──────────────────────────────────────────────────────────

#[cfg(feature = "systems-damage")]
pub use crate::systems::damage::{DamageTracker, recently_damaged};

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
pub use crate::event::{
    AdvancementEvent, DamageAdvancementEvent, DamageEvent, Event, EventBuilder, EventConfig,
    EventId, EventPlayer, EventReset, EventVisibility, IntoEventAdvancement,
};

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{Dialog, DialogAction, DialogBody, DialogButton, DialogKind};

// ── Item/component builders ──────────────────────────────────────────────────

pub use sand_components::{
    AttributeModifier, AttributeOperation, AttributeType, CustomItem, EquipmentSlotGroup,
    FoodProperties, ItemPredicate, ItemRarity,
};

// ── Raw escape hatch types ────────────────────────────────────────────────────

pub use sand_components::{RawCommand, RawComponent, RawJson, RawSnbt};

// ── Typed registry identifiers ────────────────────────────────────────────────

pub use sand_components::{
    BiomeId, BlockId, DamageTypeId, DimensionId, EffectId, EnchantmentId, EntityTypeId, ItemId,
    StructureId, TagId,
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
}
