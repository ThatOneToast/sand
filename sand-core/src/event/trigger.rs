//! Typed trigger builders for Minecraft advancement triggers.
//!
//! Each trigger type has a builder with typed methods. All builders implement
//! `Into<AdvancementTrigger>` so they work directly with
//! [`crate::event::AdvancementEvent`]'s `Trigger` associated type.
//!
//! # Typed predicates
//!
//! Use [`ItemPredicate`] and [`EntityPredicate`] from the prelude for type-safe
//! trigger filters.  To pass raw JSON as an escape hatch, wrap it with
//! `ItemPredicate::raw(RawJson::new(json!({...})))`.
//!
//! ```rust,ignore
//! use sand_core::event::trigger::ConsumeItemTrigger;
//! use sand_core::{ItemPredicate, RawJson};
//! use serde_json::json;
//!
//! // Typed (preferred):
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::id("minecraft:golden_apple"))
//!     .build();
//!
//! // Raw JSON escape hatch:
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::raw(RawJson::new(json!({"items": "minecraft:golden_apple"}))))
//!     .build();
//! ```

use crate::AdvancementTrigger;
use sand_components::advancement::InventorySlotsPredicate;
use sand_components::predicates::{DamagePredicate, EntityPredicate, IntRange, ItemPredicate};

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
    item: Option<ItemPredicate>,
}

impl ConsumeItemTrigger {
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the consumed item.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
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
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl PlayerKilledEntityTrigger {
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the killed entity's properties.
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by how the entity was killed (damage type, etc.).
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
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
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl EntityKilledPlayerTrigger {
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the attacking entity's properties.
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by the killing blow (damage type, etc.).
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
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
        AdvancementTrigger::RecipeUnlocked {
            recipe: self.recipe,
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
    slots: Option<InventorySlotsPredicate>,
    items: Vec<ItemPredicate>,
}

impl InventoryChangedTrigger {
    pub fn new() -> Self {
        Self {
            slots: None,
            items: Vec::new(),
        }
    }

    /// Filter by occupied/empty slot ranges.
    pub fn slots(mut self, slots: InventorySlotsPredicate) -> Self {
        self.slots = Some(slots);
        self
    }

    /// Add an item filter. Can be called multiple times.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.items.push(predicate);
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

/// Fires when the player crafts an item.
///
/// Maps to `minecraft:crafted_item`; it does not fire for smelting or other item acquisition.
#[derive(Clone, Debug, Default)]
pub struct ItemObtainedTrigger {
    item: Option<ItemPredicate>,
}

impl ItemObtainedTrigger {
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the crafted item.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
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
    item: Option<ItemPredicate>,
    levels: Option<IntRange>,
}

impl ItemEnchantTrigger {
    pub fn new() -> Self {
        Self {
            item: None,
            levels: None,
        }
    }

    /// Filter by the enchanted item.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by experience levels spent.
    pub fn levels(mut self, levels: IntRange) -> Self {
        self.levels = Some(levels);
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

// ─── UsingItemTrigger ─────────────────────────────────────────────────────

/// Fires when the player is actively using (holding right-click) an item.
#[derive(Clone, Debug, Default)]
pub struct UsingItemTrigger {
    item: Option<ItemPredicate>,
}

impl UsingItemTrigger {
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the item being used.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::UsingItem { item: self.item }
    }
}

impl From<UsingItemTrigger> for AdvancementTrigger {
    fn from(t: UsingItemTrigger) -> Self {
        t.build()
    }
}

// ─── MultiKillTrigger (KilledByCrossbow) ────────────────────────────────────

/// Fires when the player kills multiple unique entity types with a crossbow.
#[derive(Clone, Debug, Default)]
pub struct MultiKillTrigger {
    unique_entity_types: Option<IntRange>,
    victims: Option<Vec<EntityPredicate>>,
}

impl MultiKillTrigger {
    pub fn new() -> Self {
        Self {
            unique_entity_types: None,
            victims: None,
        }
    }

    /// Number of unique entity types that must be killed.
    pub fn unique_entity_types(mut self, count: IntRange) -> Self {
        self.unique_entity_types = Some(count);
        self
    }

    /// Filter by victim entity predicates.
    pub fn victim(mut self, predicate: EntityPredicate) -> Self {
        self.victims.get_or_insert_with(Vec::new).push(predicate);
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

// ── PlayerInteractedWithEntityTrigger ─────────────────────────────────────────

/// Fires when the player right-clicks an entity.
///
/// Use this with `interaction` entities for custom clickable objects.
#[derive(Clone, Debug, Default)]
pub struct PlayerInteractedWithEntityTrigger {
    item: Option<ItemPredicate>,
    entity: Option<EntityPredicate>,
}

impl PlayerInteractedWithEntityTrigger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the item held during the interaction.
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by the entity that was interacted with.
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::PlayerInteractedWithEntity {
            item: self.item,
            entity: self.entity,
        }
    }
}

impl From<PlayerInteractedWithEntityTrigger> for AdvancementTrigger {
    fn from(t: PlayerInteractedWithEntityTrigger) -> Self {
        t.build()
    }
}

// ── SummonedEntityTrigger ─────────────────────────────────────────────────────

/// Fires when the player summons an entity (via a spawn egg, totem, etc.).
#[derive(Clone, Debug, Default)]
pub struct SummonedEntityTrigger {
    entity: Option<EntityPredicate>,
}

impl SummonedEntityTrigger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the summoned entity's properties.
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::SummonedEntity {
            entity: self.entity,
        }
    }
}

impl From<SummonedEntityTrigger> for AdvancementTrigger {
    fn from(t: SummonedEntityTrigger) -> Self {
        t.build()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_item_typed_predicate() {
        let trigger = ConsumeItemTrigger::new()
            .item(ItemPredicate::id("minecraft:golden_apple"))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["item"]["items"], "minecraft:golden_apple");
    }

    #[test]
    fn consume_item_raw_json_escape_hatch() {
        use sand_components::RawJson;
        let trigger = ConsumeItemTrigger::new()
            .item(ItemPredicate::raw(RawJson::new(
                serde_json::json!({"items": "minecraft:honey_bottle"}),
            )))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["item"]["items"], "minecraft:honey_bottle");
    }

    #[test]
    fn player_killed_entity_typed_predicate() {
        let trigger = PlayerKilledEntityTrigger::new()
            .entity(EntityPredicate::type_("minecraft:zombie"))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["entity"]["type"], "minecraft:zombie");
    }

    #[test]
    fn inventory_changed_typed_item_predicate() {
        let trigger = InventoryChangedTrigger::new()
            .item(ItemPredicate::id("minecraft:diamond"))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        let items = &v["conditions"]["items"];
        assert_eq!(items[0]["items"], "minecraft:diamond");
    }

    #[test]
    fn tick_trigger_builds() {
        let t: AdvancementTrigger = TickTrigger::new().into();
        assert!(matches!(t, AdvancementTrigger::Tick));
    }

    #[test]
    fn impossible_trigger_builds() {
        let t: AdvancementTrigger = ImpossibleTrigger::new().into();
        assert!(matches!(t, AdvancementTrigger::Impossible));
    }

    #[test]
    fn recipe_unlocked_uses_typed_variant() {
        let t = RecipeUnlockedTrigger::new("minecraft:crafting_table").build();
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:recipe_unlocked");
        assert_eq!(v["conditions"]["recipe"], "minecraft:crafting_table");
    }
}
