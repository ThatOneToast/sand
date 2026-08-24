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
//!     .item(ItemPredicate::id(sand_core::generated::Item::GoldenApple))
//!     .build();
//!
//! // Raw JSON escape hatch:
//! let trigger = ConsumeItemTrigger::new()
//!     .item(ItemPredicate::raw(RawJson::new(json!({"items": "minecraft:golden_apple"}))))
//!     .build();
//! ```

use crate::AdvancementTrigger;
use sand_components::ItemId;
use sand_components::advancement::InventorySlotsPredicate;
use sand_components::predicates::{DamagePredicate, EntityPredicate, IntRange, ItemPredicate};

// ── TickTrigger ─────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::TickTrigger` for the canonical contract."]
/// Fires every tick (20 times per second).
///
/// Commonly used for join detection (with revoke) or per-tick checks.
#[derive(Clone, Debug, Default)]
pub struct TickTrigger;

impl TickTrigger {
    /// Starts an unconstrained tick trigger builder.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::TickTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self
    }

    /// Converts this tick builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::TickTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::ImpossibleTrigger` for the canonical contract."]
/// Never fires. Useful for placeholder or parent-only advancements.
#[derive(Clone, Debug, Default)]
pub struct ImpossibleTrigger;

impl ImpossibleTrigger {
    /// Starts a never-matching trigger builder.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ImpossibleTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self
    }

    /// Converts this never-matching builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ImpossibleTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::ConsumeItemTrigger` for the canonical contract."]
/// Fires when the player consumes an item (food, potion, honey bottle, etc.).
#[derive(Clone, Debug, Default)]
pub struct ConsumeItemTrigger {
    item: Option<ItemPredicate>,
}

impl ConsumeItemTrigger {
    /// Starts an unconstrained consume-item criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ConsumeItemTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the consumed item.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ConsumeItemTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the consume-item builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ConsumeItemTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerKilledEntityTrigger` for the canonical contract."]
/// Fires when the player kills any entity.
#[derive(Clone, Debug, Default)]
pub struct PlayerKilledEntityTrigger {
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl PlayerKilledEntityTrigger {
    /// Starts an unconstrained player-killed-entity criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerKilledEntityTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the killed entity's properties.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerKilledEntityTrigger::entity` for the canonical contract."]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by how the entity was killed (damage type, etc.).
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerKilledEntityTrigger::killing_blow` for the canonical contract."]
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
        self
    }

    /// Converts the player-kill builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerKilledEntityTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::EntityKilledPlayerTrigger` for the canonical contract."]
/// Fires when any entity kills the player.
#[derive(Clone, Debug, Default)]
pub struct EntityKilledPlayerTrigger {
    entity: Option<EntityPredicate>,
    killing_blow: Option<DamagePredicate>,
}

impl EntityKilledPlayerTrigger {
    /// Starts an unconstrained entity-killed-player criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::EntityKilledPlayerTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self {
            entity: None,
            killing_blow: None,
        }
    }

    /// Filter by the attacking entity's properties.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::EntityKilledPlayerTrigger::entity` for the canonical contract."]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Filter by the killing blow (damage type, etc.).
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::EntityKilledPlayerTrigger::killing_blow` for the canonical contract."]
    pub fn killing_blow(mut self, predicate: DamagePredicate) -> Self {
        self.killing_blow = Some(predicate);
        self
    }

    /// Converts the entity-killed-player builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::EntityKilledPlayerTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::RecipeUnlockedTrigger` for the canonical contract."]
/// Fires when the player unlocks a specific recipe.
#[derive(Clone, Debug)]
pub struct RecipeUnlockedTrigger {
    recipe: String,
}

impl RecipeUnlockedTrigger {
    /// Legacy string compatibility constructor.
    ///
    /// Prefer [`Self::from_id`] for new code so malformed IDs fail before a
    /// trigger value is constructed.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::RecipeUnlockedTrigger::new` for the canonical contract."]
    pub fn new(recipe: impl Into<String>) -> Self {
        Self {
            recipe: recipe.into(),
        }
    }

    /// Create a recipe-unlocked trigger builder from a validated recipe ID.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::RecipeUnlockedTrigger::from_id` for the canonical contract."]
    pub fn from_id(recipe: crate::ResourceLocation) -> Self {
        Self {
            recipe: recipe.to_string(),
        }
    }

    /// Converts the recipe-unlocked builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::RecipeUnlockedTrigger::build` for the canonical contract."]
    pub fn build(self) -> AdvancementTrigger {
        // `from_id` validates at construction; `new` remains a raw compatibility
        // path and is protected by Advancement's fallible export validation.
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::InventoryChangedTrigger` for the canonical contract."]
/// Fires when the player's inventory changes.
#[derive(Clone, Debug, Default)]
pub struct InventoryChangedTrigger {
    slots: Option<InventorySlotsPredicate>,
    items: Vec<ItemPredicate>,
}

impl InventoryChangedTrigger {
    /// Starts an unconstrained inventory-change criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::InventoryChangedTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self {
            slots: None,
            items: Vec::new(),
        }
    }

    /// Filter by occupied/empty slot ranges.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::InventoryChangedTrigger::slots` for the canonical contract."]
    pub fn slots(mut self, slots: InventorySlotsPredicate) -> Self {
        self.slots = Some(slots);
        self
    }

    /// Add an item filter. Can be called multiple times.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::InventoryChangedTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.items.push(predicate);
        self
    }

    /// Converts the inventory-change builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::InventoryChangedTrigger::build` for the canonical contract."]
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

// ─── ItemObtainedTrigger (legacy crafted-result filter) ─────────────────────

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemObtainedTrigger` for the canonical contract."]
/// Source-compatibility builder for the removed `minecraft:crafted_item`
/// trigger. Both filtered and unfiltered forms fail target-aware export on
/// verified current profiles. Use [`AdvancementTrigger::RecipeCrafted`] with
/// a concrete recipe ID for current vanilla.
#[derive(Clone, Debug, Default)]
pub struct ItemObtainedTrigger {
    item: Option<ItemPredicate>,
}

impl ItemObtainedTrigger {
    /// Starts an unconstrained item-obtained criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemObtainedTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the crafted item.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemObtainedTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the item-obtained builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemObtainedTrigger::build` for the canonical contract."]
    pub fn build(self) -> AdvancementTrigger {
        match self.item {
            None => AdvancementTrigger::CraftedItem { item: None },
            Some(item) => AdvancementTrigger::CraftedItem { item: Some(item) },
        }
    }
}

impl From<ItemObtainedTrigger> for AdvancementTrigger {
    fn from(t: ItemObtainedTrigger) -> Self {
        t.build()
    }
}

// ─── ItemEnchantTrigger ─────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemEnchantTrigger` for the canonical contract."]
/// Fires when the player enchants an item.
#[derive(Clone, Debug, Default)]
pub struct ItemEnchantTrigger {
    item: Option<ItemPredicate>,
    levels: Option<IntRange>,
}

impl ItemEnchantTrigger {
    /// Starts an unconstrained item-enchantment criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemEnchantTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self {
            item: None,
            levels: None,
        }
    }

    /// Filter by the enchanted item.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemEnchantTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by experience levels spent.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemEnchantTrigger::levels` for the canonical contract."]
    pub fn levels(mut self, levels: IntRange) -> Self {
        self.levels = Some(levels);
        self
    }

    /// Converts the enchantment builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::ItemEnchantTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::UsingItemTrigger` for the canonical contract."]
/// Fires when the player is actively using (holding right-click) an item.
#[derive(Clone, Debug, Default)]
pub struct UsingItemTrigger {
    item: Option<ItemPredicate>,
}

impl UsingItemTrigger {
    /// Starts an unconstrained using-item criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::UsingItemTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self { item: None }
    }

    /// Filter by the item being used.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::UsingItemTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Converts the using-item builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::UsingItemTrigger::build` for the canonical contract."]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::UsingItem { item: self.item }
    }
}

impl From<UsingItemTrigger> for AdvancementTrigger {
    fn from(t: UsingItemTrigger) -> Self {
        t.build()
    }
}

// ─── MultiKillTrigger (KilledByArrow) ───────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::MultiKillTrigger` for the canonical contract."]
/// Fires when the player kills multiple unique entity types with a crossbow.
#[derive(Clone, Debug, Default)]
pub struct MultiKillTrigger {
    unique_entity_types: Option<IntRange>,
    victims: Option<Vec<EntityPredicate>>,
}

impl MultiKillTrigger {
    /// Starts an unconstrained multi-kill criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::MultiKillTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self {
            unique_entity_types: None,
            victims: None,
        }
    }

    /// Number of unique entity types that must be killed.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::MultiKillTrigger::unique_entity_types` for the canonical contract."]
    pub fn unique_entity_types(mut self, count: IntRange) -> Self {
        self.unique_entity_types = Some(count);
        self
    }

    /// Filter by victim entity predicates.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::MultiKillTrigger::victim` for the canonical contract."]
    pub fn victim(mut self, predicate: EntityPredicate) -> Self {
        self.victims.get_or_insert_with(Vec::new).push(predicate);
        self
    }

    /// Converts the multi-kill builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::MultiKillTrigger::build` for the canonical contract."]
    pub fn build(self) -> AdvancementTrigger {
        AdvancementTrigger::KilledByArrow {
            unique_entity_types: self.unique_entity_types,
            fired_from_weapon: Some(ItemPredicate::id(
                ItemId::minecraft("crossbow").expect("crossbow is a valid vanilla item ID"),
            )),
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerInteractedWithEntityTrigger` for the canonical contract."]
/// Fires when the player right-clicks an entity.
///
/// Use this with `interaction` entities for custom clickable objects.
#[derive(Clone, Debug, Default)]
pub struct PlayerInteractedWithEntityTrigger {
    item: Option<ItemPredicate>,
    entity: Option<EntityPredicate>,
}

impl PlayerInteractedWithEntityTrigger {
    /// Starts an unconstrained player-interaction criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerInteractedWithEntityTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the item held during the interaction.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerInteractedWithEntityTrigger::item` for the canonical contract."]
    pub fn item(mut self, predicate: ItemPredicate) -> Self {
        self.item = Some(predicate);
        self
    }

    /// Filter by the entity that was interacted with.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerInteractedWithEntityTrigger::entity` for the canonical contract."]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Converts the interaction builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::PlayerInteractedWithEntityTrigger::build` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::event::trigger::SummonedEntityTrigger` for the canonical contract."]
/// Fires when the player summons an entity (via a spawn egg, totem, etc.).
#[derive(Clone, Debug, Default)]
pub struct SummonedEntityTrigger {
    entity: Option<EntityPredicate>,
}

impl SummonedEntityTrigger {
    /// Starts an unconstrained summoned-entity criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::SummonedEntityTrigger::new` for the canonical contract."]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by the summoned entity's properties.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::SummonedEntityTrigger::entity` for the canonical contract."]
    pub fn entity(mut self, predicate: EntityPredicate) -> Self {
        self.entity = Some(predicate);
        self
    }

    /// Converts the summoned-entity builder into an advancement criterion.
    #[doc = "**API Contract:** Run `sand api show sand::event::trigger::SummonedEntityTrigger::build` for the canonical contract."]
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
            .item(ItemPredicate::id(crate::generated::Item::GoldenApple))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(
            v["conditions"]["item"]["items"],
            serde_json::json!(["minecraft:golden_apple"])
        );
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
            .entity(EntityPredicate::type_(crate::generated::EntityType::Zombie))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        assert_eq!(v["conditions"]["entity"]["type"], "minecraft:zombie");
    }

    #[test]
    fn inventory_changed_typed_item_predicate() {
        let trigger = InventoryChangedTrigger::new()
            .item(ItemPredicate::id(crate::generated::Item::Diamond))
            .build();
        let v = serde_json::to_value(&trigger).unwrap();
        let items = &v["conditions"]["items"];
        assert_eq!(items[0]["items"], serde_json::json!(["minecraft:diamond"]));
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
        let t = RecipeUnlockedTrigger::from_id("minecraft:crafting_table".parse().unwrap()).build();
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:recipe_unlocked");
        assert_eq!(v["conditions"]["recipe"], "minecraft:crafting_table");
        assert!("bad recipe".parse::<crate::ResourceLocation>().is_err());
    }
}
