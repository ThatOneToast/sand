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

pub use crate::cmd::{
    Actionbar, Bossbar, BossbarColor, BossbarStyle, Execute, Inventory, InventorySlot, Selector,
    SlotPattern, Title,
};

// ── State variables ───────────────────────────────────────────────────────────

pub use crate::state::{
    Cooldown, Flag, FlagRef, NbtPath, ScoreRef, ScoreVar, StorageVar, Ticks, Timer,
};

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
    AdvancementEvent, Event, EventId, EventPlayer, EventReset, EventVisibility,
    IntoEventAdvancement,
};

// ── Dialog builders ───────────────────────────────────────────────────────────

pub use sand_components::dialog::{Dialog, DialogAction, DialogBody, DialogButton, DialogKind};

// ── Item/component builders ──────────────────────────────────────────────────

pub use sand_components::{
    AttributeModifier, AttributeOperation, AttributeType, CustomItem, EquipmentSlotGroup,
    FoodProperties, ItemPredicate, ItemRarity,
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
