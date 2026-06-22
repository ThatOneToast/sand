//! Typed trigger builders for Minecraft advancement triggers.
//!
//! Each trigger type has a builder with typed methods instead of `Option<Value>`.
//! All builders implement `Into<AdvancementTrigger>` so they work directly with
//! [`crate::event::AdvancementEvent`]'s `Trigger` associated type.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_core::ItemPredicate;
//!
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::new().with_id("minecraft:golden_apple"))
//!     .build();
//! ```

use crate::AdvancementTrigger;

// ── TickTrigger ─────────────────────────────────────────────────────────────

/// Fires every tick (20 times per second).
///
/// Commonly used for join detection (with revoke) or per-tick checks.
#[derive(Clone, Debug, Default)]
pub struct TickTrigger;

impl TickTrigger {
    pub fn new() -> Self {
        Self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::Tick
    }
}

impl From<TickTrigger> for AdvancementTrigger {
    fn from(_: TickTrigger) -> Self {
        AdvancementTrigger::Tick
    }
}

// ── ImpossibleTrigger ───────────────────────────────────────────────────────

/// Never fires. Useful for placeholder or parent-only advancements.
#[derive(Clone, Debug, Default)]
pub struct ImpossibleTrigger;

impl ImpossibleTrigger {
    pub fn new() -> Self {
        Self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::Impossible
    }
}

impl From<ImpossibleTrigger> for AdvancementTrigger {
    fn from(_: ImpossibleTrigger) -> Self {
        AdvancementTrigger::Impossible
    }
}

// ── ConsumeItemTrigger ──────────────────────────────────────────────────────

/// Fires when the player consumes an item (food, potion, honey bottle, etc.).
#[derive(Clone, Debug, Default)]
pub struct ConsumeItemTrigger {
    item: Option<::serde_json::Value>,
}

impl ConsumeItemTrigger {
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the consumed item.
    pub fn item(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.item = Some(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::ConsumeItem { item: self.item }
    }
}

impl From<ConsumeItemTrigger> for AdvancementTrigger {
    fn from(t: ConsumeItemTrigger) -> Self {
        t.build()
    }
}

// ── PlayerKilledEntityTrigger ───────────────────────────────────────────────

/// Fires when the player kills any entity.
#[derive(Clone, Debug, Default)]
pub struct PlayerKilledEntityTrigger {
    entity: Option<::serde_json::Value>,
    killing_blow: Option<::serde_json::Value>,
}

impl PlayerKilledEntityTrigger {
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the killed entity's properties.
    pub fn entity(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.entity = Some(predicate.into());
        self
    }

    /// Filter by how the entity was killed (damage type, etc.).
    pub fn killing_blow(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.killing_blow = Some(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::PlayerKilledEntity {
            entity: self.entity,
            killing_blow: self.killing_blow,
        }
    }
}

impl From<PlayerKilledEntityTrigger> for AdvancementTrigger {
    fn from(t: PlayerKilledEntityTrigger) -> Self {
        t.build()
    }
}

// ── EntityKilledPlayerTrigger ───────────────────────────────────────────────

/// Fires when any entity kills the player.
#[derive(Clone, Debug, Default)]
pub struct EntityKilledPlayerTrigger {
    entity: Option<::serde_json::Value>,
    killing_blow: Option<::serde_json::Value>,
}

impl EntityKilledPlayerTrigger {
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the attacking entity's properties.
    pub fn entity(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.entity = Some(predicate.into());
        self
    }

    /// Filter by the killing blow (damage type, etc.).
    pub fn killing_blow(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.killing_blow = Some(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::EntityKilledPlayer {
            entity: self.entity,
            killing_blow: self.killing_blow,
        }
    }
}

impl From<EntityKilledPlayerTrigger> for AdvancementTrigger {
    fn from(t: EntityKilledPlayerTrigger) -> Self {
        t.build()
    }
}

// ── RecipeUnlockedTrigger ───────────────────────────────────────────────────

/// Fires when the player unlocks a specific recipe.
#[derive(Clone, Debug)]
pub struct RecipeUnlockedTrigger {
    recipe: String,
}

impl RecipeUnlockedTrigger {
    pub fn new(recipe: impl Into<String>) -> Self {
        Self {
            recipe: recipe.into(),
        }
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::Custom {
            trigger: "minecraft:recipe_unlocked".into(),
            conditions: Some(::serde_json::json!({ "recipe": self.recipe })),
        }
    }
}

impl From<RecipeUnlockedTrigger> for AdvancementTrigger {
    fn from(t: RecipeUnlockedTrigger) -> Self {
        t.build()
    }
}

// ─── InventoryChangedTrigger ────────────────────────────────────────────────

/// Fires when the player's inventory changes.
#[derive(Clone, Debug, Default)]
pub struct InventoryChangedTrigger {
    slots: Option<::serde_json::Value>,
    items: Vec<::serde_json::Value>,
}

impl InventoryChangedTrigger {
    pub fn new() -> Self {
        Self {
            slots: None,
            items: Vec::new(),
        }
    }

    /// Filter by occupied/empty slot ranges.
    pub fn slots(mut self, slots: impl Into<::serde_json::Value>) -> Self {
        self.slots = Some(slots.into());
        self
    }

    /// Add an item filter. Can be called multiple times.
    pub fn item(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.items.push(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::InventoryChanged {
            slots: self.slots,
            items: self.items,
        }
    }
}

impl From<InventoryChangedTrigger> for AdvancementTrigger {
    fn from(t: InventoryChangedTrigger) -> Self {
        t.build()
    }
}

// ─── ItemObtainedTrigger (CraftedItem) ──────────────────────────────────────

/// Fires when the player obtains an item (crafting, smelting, etc.).
#[derive(Clone, Debug, Default)]
pub struct ItemObtainedTrigger {
    item: Option<::serde_json::Value>,
}

impl ItemObtainedTrigger {
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the obtained item.
    pub fn item(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.item = Some(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::CraftedItem { item: self.item }
    }
}

impl From<ItemObtainedTrigger> for AdvancementTrigger {
    fn from(t: ItemObtainedTrigger) -> Self {
        t.build()
    }
}

// ─── ItemEnchantTrigger ─────────────────────────────────────────────────────

/// Fires when the player enchants an item.
#[derive(Clone, Debug, Default)]
pub struct ItemEnchantTrigger {
    item: Option<::serde_json::Value>,
    levels: Option<::serde_json::Value>,
}

impl ItemEnchantTrigger {
    pub fn new() -> Self {
        Self {
            item: None,
            levels: None,
        }
    }

    /// Filter by the enchanted item.
    pub fn item(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.item = Some(predicate.into());
        self
    }

    /// Filter by experience levels spent.
    pub fn levels(mut self, levels: impl Into<::serde_json::Value>) -> Self {
        self.levels = Some(levels.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::EnchantedItem {
            item: self.item,
            levels: self.levels,
        }
    }
}

impl From<ItemEnchantTrigger> for AdvancementTrigger {
    fn from(t: ItemEnchantTrigger) -> Self {
        t.build()
    }
}

// ─── KillEntityNearStructureTrigger (KilledByCrossbow) ──────────────────────

/// Fires when the player kills multiple unique entity types with a crossbow.
#[derive(Clone, Debug, Default)]
pub struct MultiKillTrigger {
    unique_entity_types: Option<::serde_json::Value>,
    victims: Option<Vec<::serde_json::Value>>,
}

impl MultiKillTrigger {
    pub fn new() -> Self {
        Self {
            unique_entity_types: None,
            victims: None,
        }
    }

    /// Number of unique entity types that must be killed.
    pub fn unique_entity_types(mut self, count: impl Into<::serde_json::Value>) -> Self {
        self.unique_entity_types = Some(count.into());
        self
    }

    /// Filter by victim entity predicates.
    pub fn victim(mut self, predicate: impl Into<::serde_json::Value>) -> Self {
        self.victims
            .get_or_insert_with(Vec::new)
            .push(predicate.into());
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::KilledByCrossbow {
            unique_entity_types: self.unique_entity_types,
            victims: self.victims,
        }
    }
}

impl From<MultiKillTrigger> for AdvancementTrigger {
    fn from(t: MultiKillTrigger) -> Self {
        t.build()
    }
}
