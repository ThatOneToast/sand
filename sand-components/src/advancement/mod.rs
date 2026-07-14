pub mod trigger_coverage;

use std::collections::HashMap;

use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::predicates::{
    DamagePredicate, DistancePredicate, EffectPredicate, EntityPredicate, FloatRange, IntRange,
    ItemPredicate, LocationPredicate,
};
use crate::raw::RawJson;
use crate::registry::{BlockId, DimensionId, PotionRegistryId, StatusEffectId};
use crate::resource_location::ResourceLocation;

fn validate_resource_id(value: &str, path: &str) -> Result<(), String> {
    value
        .parse::<ResourceLocation>()
        .map(|_| ())
        .map_err(|_| format!("{path}: `{value}` must be a valid namespaced resource location"))
}

fn json_value<T: Serialize, E: serde::ser::Error>(value: &T) -> Result<Value, E> {
    serde_json::to_value(value).map_err(E::custom)
}

// ── AdvancementFrame ──────────────────────────────────────────────────────────

/// The visual frame style for an advancement in the advancement screen.
///
/// Determines how the advancement appears to the player when completed.
pub enum AdvancementFrame {
    Task,
    Goal,
    Challenge,
}

impl AdvancementFrame {
    fn as_str(&self) -> &'static str {
        match self {
            AdvancementFrame::Task => "task",
            AdvancementFrame::Goal => "goal",
            AdvancementFrame::Challenge => "challenge",
        }
    }
}

// ── AdvancementIcon ───────────────────────────────────────────────────────────

/// The icon displayed for an advancement, with optional item components.
pub struct AdvancementIcon {
    pub id: String,
    pub components: Option<RawJson>,
}

impl AdvancementIcon {
    /// Creates a new advancement icon with the specified item ID.
    pub fn new(id: impl std::fmt::Display) -> Self {
        Self {
            id: id.to_string(),
            components: None,
        }
    }

    /// Sets the item components for this icon using an explicit [`RawJson`] escape hatch.
    ///
    /// Use this for icon component overrides (e.g. enchantments, custom model data)
    /// that are not yet modelled by the typed item component API.
    pub fn components(mut self, components: RawJson) -> Self {
        self.components = Some(components);
        self
    }
}

impl Serialize for AdvancementIcon {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id)?;
        if let Some(ref c) = self.components {
            map.serialize_entry("components", c)?;
        }
        map.end()
    }
}

// ── AdvancementDisplay ────────────────────────────────────────────────────────

/// The display information shown for an advancement in the advancement screen and toast.
pub struct AdvancementDisplay {
    pub icon: AdvancementIcon,
    pub title: Value,
    pub description: Value,
    pub background: Option<String>,
    pub frame: AdvancementFrame,
    pub show_toast: bool,
    pub announce_to_chat: bool,
    pub hidden: bool,
}

impl AdvancementDisplay {
    /// Creates a new advancement display with the specified icon, title, and description.
    ///
    /// `title` and `description` accept any `impl Into<Value>` — pass a plain `&str`
    /// for a string literal title, or a [`TextComponent`](sand_commands::TextComponent)
    /// for rich text (it implements `Into<Value>` via `Into<String>` → `Value::String`).
    pub fn new(
        icon: AdvancementIcon,
        title: impl Into<Value>,
        description: impl Into<Value>,
    ) -> Self {
        Self {
            icon,
            title: title.into(),
            description: description.into(),
            background: None,
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_to_chat: true,
            hidden: false,
        }
    }

    /// Sets the background texture for the advancement tab.
    pub fn background(mut self, bg: impl Into<String>) -> Self {
        self.background = Some(bg.into());
        self
    }

    /// Sets the frame style for this advancement display.
    pub fn frame(mut self, frame: AdvancementFrame) -> Self {
        self.frame = frame;
        self
    }

    /// Sets whether a toast notification is shown when this advancement is completed.
    pub fn show_toast(mut self, v: bool) -> Self {
        self.show_toast = v;
        self
    }

    /// Sets whether this advancement completion is announced in chat.
    pub fn announce_to_chat(mut self, v: bool) -> Self {
        self.announce_to_chat = v;
        self
    }

    /// Sets whether this advancement is hidden until completed.
    pub fn hidden(mut self, v: bool) -> Self {
        self.hidden = v;
        self
    }
}

impl Serialize for AdvancementDisplay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("icon", &self.icon)?;
        map.serialize_entry("title", &self.title)?;
        map.serialize_entry("description", &self.description)?;
        if let Some(ref bg) = self.background {
            map.serialize_entry("background", bg)?;
        }
        map.serialize_entry("frame", self.frame.as_str())?;
        map.serialize_entry("show_toast", &self.show_toast)?;
        map.serialize_entry("announce_to_chat", &self.announce_to_chat)?;
        map.serialize_entry("hidden", &self.hidden)?;
        map.end()
    }
}

// ── AdvancementTrigger ────────────────────────────────────────────────────────

/// Represents a trigger condition for an advancement criterion.
///
/// Each variant uses typed predicate structs from [`sand_components::predicates`]
/// instead of raw `serde_json::Value`. Prefer the typed associated constructors
/// for variants whose public fields remain strings for source compatibility.
/// The [`Custom`](AdvancementTrigger::Custom) variant is the legacy raw shape;
/// [`AdvancementTrigger::custom_trigger`] is the validated normal path for
/// custom/modded triggers.
///
/// # Escape hatch
///
/// ```rust
/// use sand_components::{AdvancementTrigger, RawJson};
/// use serde_json::json;
///
/// let t = AdvancementTrigger::Custom {
///     trigger: "mymod:custom_trigger".into(),
///     conditions: Some(RawJson::new(json!({"level": 5}))),
/// };
/// ```
#[allow(clippy::large_enum_variant)]
pub enum AdvancementTrigger {
    Tick,
    Impossible,

    // ── Kill / combat ─────────────────────────────────────────────────────────
    PlayerKilledEntity {
        entity: Option<EntityPredicate>,
        killing_blow: Option<DamagePredicate>,
    },
    EntityKilledPlayer {
        entity: Option<EntityPredicate>,
        killing_blow: Option<DamagePredicate>,
    },
    /// Player deals damage to an entity.
    PlayerHurtEntity {
        entity: Option<EntityPredicate>,
        damage: Option<DamagePredicate>,
    },
    /// Entity deals damage to the player.
    EntityHurtPlayer {
        entity: Option<EntityPredicate>,
        damage: Option<DamagePredicate>,
    },
    /// Player kills an entity using a crossbow.
    KilledByCrossbow {
        unique_entity_types: Option<IntRange>,
        victims: Option<Vec<EntityPredicate>>,
    },
    /// A lightning bolt hits an entity the player summoned with a trident.
    ChanneledLightning {
        victims: Option<Vec<EntityPredicate>>,
    },
    /// A lightning bolt strikes near the player.
    LightningStrike {
        lightning: Option<EntityPredicate>,
        bystander: Option<EntityPredicate>,
    },

    // ── Inventory / items ─────────────────────────────────────────────────────
    InventoryChanged {
        slots: Option<InventorySlotsPredicate>,
        items: Vec<ItemPredicate>,
    },
    RecipeUnlocked {
        recipe: String,
    },
    UsedItem {
        item: Option<ItemPredicate>,
    },
    ConsumeItem {
        item: Option<ItemPredicate>,
    },
    UsingItem {
        item: Option<ItemPredicate>,
    },
    /// Player crafts an item.
    CraftedItem {
        item: Option<ItemPredicate>,
    },
    /// Player fills a bucket.
    FilledBucket {
        item: Option<ItemPredicate>,
    },
    /// Player empties a bucket.
    EmptiedBucket {
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
    },
    /// Player shoots a crossbow.
    ShotCrossbow {
        item: Option<ItemPredicate>,
    },
    /// Player activates a totem of undying.
    UsedTotem {
        item: Option<ItemPredicate>,
    },
    /// A thrown item is picked up by an entity.
    ThrownItemPickedUp {
        item: Option<ItemPredicate>,
        entity: Option<EntityPredicate>,
    },
    /// An item in the player's inventory loses durability.
    ItemDurabilityChanged {
        item: Option<ItemPredicate>,
        delta: Option<IntRange>,
        durability: Option<IntRange>,
    },
    /// Player brews a potion.
    BrewedPotion {
        potion: Option<String>,
    },
    /// Player destroys a bee nest or beehive.
    BeeNestDestroyed {
        block: Option<String>,
        item: Option<ItemPredicate>,
        num_bees_inside: Option<IntRange>,
    },

    /// Player enchants an item.
    EnchantedItem {
        item: Option<ItemPredicate>,
        levels: Option<IntRange>,
    },

    // ── Entities / interactions ───────────────────────────────────────────────
    BredAnimals {
        parent: Option<EntityPredicate>,
        partner: Option<EntityPredicate>,
        child: Option<EntityPredicate>,
    },
    TamedAnimal {
        entity: Option<EntityPredicate>,
    },
    SummonedEntity {
        entity: Option<EntityPredicate>,
    },
    PlayerInteractedWithEntity {
        item: Option<ItemPredicate>,
        entity: Option<EntityPredicate>,
    },
    /// Player uses a fishing rod and it hooks something.
    FishingRodHooked {
        rod: Option<ItemPredicate>,
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
    },
    TamedAnimalInteracted {
        entity: Option<EntityPredicate>,
        item: Option<ItemPredicate>,
    },
    VillagerTrade {
        item: Option<ItemPredicate>,
        villager: Option<EntityPredicate>,
    },
    CuredZombieVillager {
        villager: Option<EntityPredicate>,
        zombie: Option<EntityPredicate>,
    },

    // ── Location / world ──────────────────────────────────────────────────────
    PlacedBlock {
        block: Option<String>,
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
        state: Option<HashMap<String, String>>,
    },
    EnterBlock {
        block: Option<String>,
        state: Option<HashMap<String, String>>,
    },
    Location {
        location: Option<LocationPredicate>,
    },
    NetherTravel {
        entered: Option<LocationPredicate>,
        exited: Option<LocationPredicate>,
        distance: Option<DistancePredicate>,
    },
    ChangedDimension {
        from: Option<String>,
        to: Option<String>,
    },
    SleptInBed {
        location: Option<LocationPredicate>,
    },
    FallFromHeight {
        distance: Option<DistancePredicate>,
        start_position: Option<LocationPredicate>,
    },
    SlideDownBlock {
        block: Option<String>,
    },
    TargetHit {
        signal_strength: Option<IntRange>,
        projectile: Option<EntityPredicate>,
    },
    HeroOfTheVillage {
        location: Option<LocationPredicate>,
    },
    PlayerGeneratesContainerLoot {
        loot_table: Option<String>,
    },

    // ── Player state ──────────────────────────────────────────────────────────
    LeveledUp {
        level: Option<IntRange>,
    },
    EffectsChanged {
        effects: Option<HashMap<String, EffectPredicate>>,
        source: Option<EntityPredicate>,
    },
    StartedRiding,
    ConstructBeacon {
        level: Option<IntRange>,
    },
    UsedEnderEye {
        distance: Option<FloatRange>,
    },

    // ── 1.19+ triggers ───────────────────────────────────────────────────────
    /// Player causes an allay to drop an item on a block (1.19+).
    AllayDropItemOnBlock {
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
    },
    /// Player avoids triggering a sculk sensor vibration (1.19+).
    AvoidVibration,
    /// Player kills a mob near a sculk catalyst (1.19+).
    KillMobNearSculkCatalyst {
        entity: Option<EntityPredicate>,
        killing_blow: Option<DamagePredicate>,
    },
    /// Player right-clicks on a block while holding an item (1.19.4+).
    ItemUsedOnBlock {
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
    },

    // ── 1.16+ triggers ───────────────────────────────────────────────────────
    /// Player rides an entity in lava (1.16+).
    RideEntityInLava {
        start_position: Option<LocationPredicate>,
        distance: Option<DistancePredicate>,
    },

    // ── Custom (escape hatch) ─────────────────────────────────────────────────
    /// Any trigger not covered by the typed variants.
    ///
    /// Use this to target triggers that were added to or removed from Minecraft
    /// after a given version, or for modded triggers.
    ///
    /// ```rust
    /// use sand_components::AdvancementTrigger;
    /// let t = AdvancementTrigger::Custom {
    ///     trigger: "minecraft:tick".into(),
    ///     conditions: None,
    /// };
    /// ```
    Custom {
        trigger: String,
        /// Raw JSON conditions block.  Use [`RawJson`] to signal intentional
        /// opt-out of the typed predicate API.
        conditions: Option<RawJson>,
    },
}

// ── Inventory slots predicate (used only by InventoryChanged) ─────────────────

/// Slot-count conditions for [`AdvancementTrigger::InventoryChanged`].
///
/// Controls how many inventory slots must be occupied, full, or empty.
/// This is a *count* predicate, not a slot-position selector.
#[derive(Debug, Clone, Default, Serialize)]
pub struct InventorySlotsPredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupied: Option<IntRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<IntRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty: Option<IntRange>,
}

impl InventorySlotsPredicate {
    fn validate_at(&self, path: &str) -> Result<(), String> {
        for (name, range) in [
            ("occupied", &self.occupied),
            ("full", &self.full),
            ("empty", &self.empty),
        ] {
            if let Some(range) = range {
                range.validate_at(&format!("{path}.{name}"))?;
            }
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self::default()
    }
    pub fn occupied_min(mut self, n: i64) -> Self {
        self.occupied = Some(IntRange::at_least(n));
        self
    }
    pub fn occupied_max(mut self, n: i64) -> Self {
        self.occupied = Some(IntRange::at_most(n));
        self
    }
    pub fn empty_min(mut self, n: i64) -> Self {
        self.empty = Some(IntRange::at_least(n));
        self
    }
    pub fn full_min(mut self, n: i64) -> Self {
        self.full = Some(IntRange::at_least(n));
        self
    }
}

// ── AdvancementTrigger::trigger_id helper ─────────────────────────────────────

impl AdvancementTrigger {
    /// Create a recipe-unlocked trigger from a validated recipe reference.
    pub fn recipe_unlocked(recipe: ResourceLocation) -> Self {
        Self::RecipeUnlocked {
            recipe: recipe.to_string(),
        }
    }

    /// Create a brewed-potion trigger using the shared potion registry ID.
    pub fn brewed_potion(potion: impl Into<PotionRegistryId>) -> Self {
        Self::BrewedPotion {
            potion: Some(potion.into().to_string()),
        }
    }

    /// Create an unfiltered brewed-potion trigger.
    pub fn brewed_any_potion() -> Self {
        Self::BrewedPotion { potion: None }
    }

    /// Create a bee-nest-destroyed trigger with typed block identity.
    pub fn bee_nest_destroyed(
        block: Option<BlockId>,
        item: Option<ItemPredicate>,
        num_bees_inside: Option<IntRange>,
    ) -> Self {
        Self::BeeNestDestroyed {
            block: block.map(|id| id.to_string()),
            item,
            num_bees_inside,
        }
    }

    /// Create a placed-block trigger with typed block identity.
    pub fn placed_block(
        block: Option<BlockId>,
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
        state: Option<HashMap<String, String>>,
    ) -> Self {
        Self::PlacedBlock {
            block: block.map(|id| id.to_string()),
            item,
            location,
            state,
        }
    }

    /// Create an enter-block trigger with typed block identity.
    pub fn enter_block(block: Option<BlockId>, state: Option<HashMap<String, String>>) -> Self {
        Self::EnterBlock {
            block: block.map(|id| id.to_string()),
            state,
        }
    }

    /// Create a dimension-change trigger with typed dimension identities.
    pub fn changed_dimension(from: Option<DimensionId>, to: Option<DimensionId>) -> Self {
        Self::ChangedDimension {
            from: from.map(|id| id.to_string()),
            to: to.map(|id| id.to_string()),
        }
    }

    /// Create a slide-down-block trigger with typed block identity.
    pub fn slide_down_block(block: Option<BlockId>) -> Self {
        Self::SlideDownBlock {
            block: block.map(|id| id.to_string()),
        }
    }

    /// Create a container-loot trigger from a validated loot-table reference.
    pub fn player_generates_container_loot(loot_table: Option<ResourceLocation>) -> Self {
        Self::PlayerGeneratesContainerLoot {
            loot_table: loot_table.map(|id| id.to_string()),
        }
    }

    /// Create an effects-changed trigger with typed status-effect map keys.
    pub fn effects_changed<I, E>(effects: I, source: Option<EntityPredicate>) -> Self
    where
        I: IntoIterator<Item = (E, EffectPredicate)>,
        E: Into<StatusEffectId>,
    {
        let effects = effects
            .into_iter()
            .map(|(id, predicate)| (id.into().to_string(), predicate))
            .collect::<HashMap<_, _>>();
        Self::EffectsChanged {
            effects: (!effects.is_empty()).then_some(effects),
            source,
        }
    }

    /// Create an unfiltered effects-changed trigger.
    pub fn effects_changed_any(source: Option<EntityPredicate>) -> Self {
        Self::EffectsChanged {
            effects: None,
            source,
        }
    }

    /// Create a custom/modded trigger with a validated trigger ID.
    ///
    /// The conditions remain an explicit opaque [`RawJson`] escape hatch.
    pub fn custom_trigger(trigger: ResourceLocation, conditions: Option<RawJson>) -> Self {
        Self::Custom {
            trigger: trigger.to_string(),
            conditions,
        }
    }

    /// Validate stable predicate/range invariants for typed trigger conditions.
    /// Raw/custom trigger conditions remain an explicit escape hatch.
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        let conditions = format!("{path}.conditions");
        match self {
            Self::RecipeUnlocked { recipe } => {
                validate_resource_id(recipe, &format!("{conditions}.recipe"))?;
            }
            Self::BrewedPotion {
                potion: Some(potion),
            } => {
                validate_resource_id(potion, &format!("{conditions}.potion"))?;
            }
            Self::BeeNestDestroyed {
                block: Some(block), ..
            } => {
                validate_resource_id(block, &format!("{conditions}.block"))?;
            }
            Self::PlacedBlock {
                block: Some(block), ..
            }
            | Self::EnterBlock {
                block: Some(block), ..
            }
            | Self::SlideDownBlock { block: Some(block) } => {
                validate_resource_id(block, &format!("{conditions}.block"))?;
            }
            Self::ChangedDimension { from, to } => {
                if let Some(from) = from {
                    validate_resource_id(from, &format!("{conditions}.from"))?;
                }
                if let Some(to) = to {
                    validate_resource_id(to, &format!("{conditions}.to"))?;
                }
            }
            Self::PlayerGeneratesContainerLoot {
                loot_table: Some(loot_table),
            } => {
                validate_resource_id(loot_table, &format!("{conditions}.loot_table"))?;
            }
            Self::Custom { trigger, .. } => {
                validate_resource_id(trigger, &format!("{path}.trigger"))?;
            }
            Self::PlayerKilledEntity {
                entity,
                killing_blow,
            }
            | Self::EntityKilledPlayer {
                entity,
                killing_blow,
            }
            | Self::KillMobNearSculkCatalyst {
                entity,
                killing_blow,
            } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(damage) = killing_blow {
                    damage.validate_at(&format!("{conditions}.killing_blow"))?;
                }
            }
            Self::PlayerHurtEntity { entity, damage }
            | Self::EntityHurtPlayer { entity, damage } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(damage) = damage {
                    damage.validate_at(&format!("{conditions}.damage"))?;
                }
            }
            Self::KilledByCrossbow {
                unique_entity_types,
                victims,
            } => {
                if let Some(range) = unique_entity_types {
                    range.validate_at(&format!("{conditions}.unique_entity_types"))?;
                }
                if let Some(victims) = victims {
                    for (index, victim) in victims.iter().enumerate() {
                        victim.validate_at(&format!("{conditions}.victims[{index}]"))?;
                    }
                }
            }
            Self::ChanneledLightning {
                victims: Some(victims),
            } => {
                for (index, victim) in victims.iter().enumerate() {
                    victim.validate_at(&format!("{conditions}.victims[{index}]"))?;
                }
            }
            Self::LightningStrike {
                lightning,
                bystander,
            } => {
                if let Some(entity) = lightning {
                    entity.validate_at(&format!("{conditions}.lightning"))?;
                }
                if let Some(entity) = bystander {
                    entity.validate_at(&format!("{conditions}.bystander"))?;
                }
            }
            Self::InventoryChanged { slots, items } => {
                if let Some(slots) = slots {
                    slots.validate_at(&format!("{conditions}.slots"))?;
                }
                for (index, item) in items.iter().enumerate() {
                    item.validate_at(&format!("{conditions}.items[{index}]"))?;
                }
            }
            Self::LeveledUp { level } | Self::ConstructBeacon { level } => {
                if let Some(level) = level {
                    level.validate_at(&format!("{conditions}.level"))?;
                }
            }
            Self::UsedEnderEye {
                distance: Some(distance),
            } => distance.validate_at(&format!("{conditions}.distance"))?,
            Self::Location { location }
            | Self::SleptInBed { location }
            | Self::HeroOfTheVillage { location } => {
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::UsedItem { item }
            | Self::ConsumeItem { item }
            | Self::UsingItem { item }
            | Self::CraftedItem { item }
            | Self::FilledBucket { item }
            | Self::ShotCrossbow { item }
            | Self::UsedTotem { item } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
            }
            Self::EmptiedBucket { item, location }
            | Self::AllayDropItemOnBlock { item, location }
            | Self::ItemUsedOnBlock { item, location } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::ThrownItemPickedUp { item, entity }
            | Self::PlayerInteractedWithEntity { item, entity }
            | Self::TamedAnimalInteracted { item, entity } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
            }
            Self::ItemDurabilityChanged {
                item,
                delta,
                durability,
            } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = delta {
                    range.validate_at(&format!("{conditions}.delta"))?;
                }
                if let Some(range) = durability {
                    range.validate_at(&format!("{conditions}.durability"))?;
                }
            }
            Self::BeeNestDestroyed {
                item,
                num_bees_inside,
                ..
            } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = num_bees_inside {
                    range.validate_at(&format!("{conditions}.num_bees_inside"))?;
                }
            }
            Self::EnchantedItem { item, levels } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = levels {
                    range.validate_at(&format!("{conditions}.levels"))?;
                }
            }
            Self::BredAnimals {
                parent,
                partner,
                child,
            } => {
                for (name, entity) in [("parent", parent), ("partner", partner), ("child", child)] {
                    if let Some(entity) = entity {
                        entity.validate_at(&format!("{conditions}.{name}"))?;
                    }
                }
            }
            Self::TamedAnimal { entity } | Self::SummonedEntity { entity } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
            }
            Self::FishingRodHooked { rod, entity, item } => {
                if let Some(rod) = rod {
                    rod.validate_at(&format!("{conditions}.rod"))?;
                }
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
            }
            Self::VillagerTrade { item, villager } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(villager) = villager {
                    villager.validate_at(&format!("{conditions}.villager"))?;
                }
            }
            Self::CuredZombieVillager { villager, zombie } => {
                if let Some(entity) = villager {
                    entity.validate_at(&format!("{conditions}.villager"))?;
                }
                if let Some(entity) = zombie {
                    entity.validate_at(&format!("{conditions}.zombie"))?;
                }
            }
            Self::PlacedBlock { item, location, .. } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::NetherTravel {
                entered,
                exited,
                distance,
            } => {
                if let Some(location) = entered {
                    location.validate_at(&format!("{conditions}.entered"))?;
                }
                if let Some(location) = exited {
                    location.validate_at(&format!("{conditions}.exited"))?;
                }
                if let Some(distance) = distance {
                    distance.validate_at(&format!("{conditions}.distance"))?;
                }
            }
            Self::FallFromHeight {
                distance,
                start_position,
            }
            | Self::RideEntityInLava {
                distance,
                start_position,
            } => {
                if let Some(distance) = distance {
                    distance.validate_at(&format!("{conditions}.distance"))?;
                }
                if let Some(location) = start_position {
                    location.validate_at(&format!("{conditions}.start_position"))?;
                }
            }
            Self::TargetHit {
                signal_strength,
                projectile,
            } => {
                if let Some(range) = signal_strength {
                    range.validate_at(&format!("{conditions}.signal_strength"))?;
                }
                if let Some(entity) = projectile {
                    entity.validate_at(&format!("{conditions}.projectile"))?;
                }
            }
            Self::EffectsChanged { effects, source } => {
                if let Some(effects) = effects {
                    for (effect, predicate) in effects {
                        validate_resource_id(effect, &format!("{conditions}.effects.{effect}"))?;
                        predicate.validate_at(&format!("{conditions}.effects.{effect}"))?;
                    }
                }
                if let Some(entity) = source {
                    entity.validate_at(&format!("{conditions}.source"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Return the vanilla trigger ID selected by this typed trigger.
    pub fn trigger_id(&self) -> &str {
        match self {
            AdvancementTrigger::Tick => "minecraft:tick",
            AdvancementTrigger::Impossible => "minecraft:impossible",
            AdvancementTrigger::PlayerKilledEntity { .. } => "minecraft:player_killed_entity",
            AdvancementTrigger::EntityKilledPlayer { .. } => "minecraft:entity_killed_player",
            AdvancementTrigger::InventoryChanged { .. } => "minecraft:inventory_changed",
            AdvancementTrigger::RecipeUnlocked { .. } => "minecraft:recipe_unlocked",
            AdvancementTrigger::UsedItem { .. } => "minecraft:used_item",
            AdvancementTrigger::PlacedBlock { .. } => "minecraft:placed_block",
            AdvancementTrigger::BredAnimals { .. } => "minecraft:bred_animals",
            AdvancementTrigger::ConsumeItem { .. } => "minecraft:consume_item",
            AdvancementTrigger::EnterBlock { .. } => "minecraft:enter_block",
            AdvancementTrigger::EnchantedItem { .. } => "minecraft:enchanted_item",
            AdvancementTrigger::TamedAnimal { .. } => "minecraft:tame_animal",
            AdvancementTrigger::SummonedEntity { .. } => "minecraft:summoned_entity",
            AdvancementTrigger::Location { .. } => "minecraft:location",
            AdvancementTrigger::NetherTravel { .. } => "minecraft:nether_travel",
            AdvancementTrigger::UsingItem { .. } => "minecraft:using_item",
            AdvancementTrigger::PlayerInteractedWithEntity { .. } => {
                "minecraft:player_interacted_with_entity"
            }
            AdvancementTrigger::PlayerHurtEntity { .. } => "minecraft:player_hurt_entity",
            AdvancementTrigger::EntityHurtPlayer { .. } => "minecraft:entity_hurt_player",
            AdvancementTrigger::KilledByCrossbow { .. } => "minecraft:killed_by_crossbow",
            AdvancementTrigger::ChanneledLightning { .. } => "minecraft:channeled_lightning",
            AdvancementTrigger::LightningStrike { .. } => "minecraft:lightning_strike",
            AdvancementTrigger::CraftedItem { .. } => "minecraft:crafted_item",
            AdvancementTrigger::FilledBucket { .. } => "minecraft:filled_bucket",
            AdvancementTrigger::EmptiedBucket { .. } => "minecraft:emptied_bucket",
            AdvancementTrigger::FishingRodHooked { .. } => "minecraft:fishing_rod_hooked",
            AdvancementTrigger::ShotCrossbow { .. } => "minecraft:shot_crossbow",
            AdvancementTrigger::UsedTotem { .. } => "minecraft:used_totem",
            AdvancementTrigger::ThrownItemPickedUp { .. } => "minecraft:thrown_item_picked_up",
            AdvancementTrigger::ItemDurabilityChanged { .. } => "minecraft:item_durability_changed",
            AdvancementTrigger::BrewedPotion { .. } => "minecraft:brewed_potion",
            AdvancementTrigger::BeeNestDestroyed { .. } => "minecraft:bee_nest_destroyed",
            AdvancementTrigger::ChangedDimension { .. } => "minecraft:changed_dimension",
            AdvancementTrigger::SleptInBed { .. } => "minecraft:slept_in_bed",
            AdvancementTrigger::FallFromHeight { .. } => "minecraft:fall_from_height",
            AdvancementTrigger::LeveledUp { .. } => "minecraft:leveled_up",
            AdvancementTrigger::EffectsChanged { .. } => "minecraft:effects_changed",
            AdvancementTrigger::StartedRiding => "minecraft:started_riding",
            AdvancementTrigger::SlideDownBlock { .. } => "minecraft:slide_down_block",
            AdvancementTrigger::TargetHit { .. } => "minecraft:target_hit",
            AdvancementTrigger::ConstructBeacon { .. } => "minecraft:construct_beacon",
            AdvancementTrigger::CuredZombieVillager { .. } => "minecraft:cured_zombie_villager",
            AdvancementTrigger::UsedEnderEye { .. } => "minecraft:used_ender_eye",
            AdvancementTrigger::HeroOfTheVillage { .. } => "minecraft:hero_of_the_village",
            AdvancementTrigger::PlayerGeneratesContainerLoot { .. } => {
                "minecraft:player_generates_container_loot"
            }
            AdvancementTrigger::VillagerTrade { .. } => "minecraft:villager_trade",
            AdvancementTrigger::TamedAnimalInteracted { .. } => {
                "minecraft:player_interacted_with_entity"
            }
            AdvancementTrigger::AllayDropItemOnBlock { .. } => "minecraft:allay_drop_item_on_block",
            AdvancementTrigger::AvoidVibration => "minecraft:avoid_vibration",
            AdvancementTrigger::KillMobNearSculkCatalyst { .. } => {
                "minecraft:kill_mob_near_sculk_catalyst"
            }
            AdvancementTrigger::ItemUsedOnBlock { .. } => "minecraft:item_used_on_block",
            AdvancementTrigger::RideEntityInLava { .. } => "minecraft:ride_entity_in_lava",
            AdvancementTrigger::Custom { trigger, .. } => trigger.as_str(),
        }
    }

    /// Validate this trigger against Sand's supported vanilla target profiles.
    ///
    /// This intentionally fails before an advancement JSON file is emitted for
    /// IDs known to be absent from the vanilla registry.
    pub fn validate_for_target(&self) -> Result<(), String> {
        let metadata = crate::advancement::trigger_coverage::trigger_metadata(self.trigger_id());
        if metadata.supported {
            Ok(())
        } else {
            Err(format!(
                "advancement trigger `{}` is not available for Sand's supported Minecraft targets. {}",
                self.trigger_id(),
                metadata.diagnostic.unwrap_or("choose a supported trigger")
            ))
        }
    }

    // ── Convenience constructors ──────────────────────────────────────────────

    /// Build an `InventoryChanged` trigger matching any of the given item IDs.
    ///
    /// Items are generated registry values implementing `Display`.
    pub fn inventory_changed(items: Vec<impl std::fmt::Display>) -> Self {
        AdvancementTrigger::InventoryChanged {
            slots: None,
            items: items
                .into_iter()
                .map(|i| ItemPredicate::id(i.to_string()))
                .collect(),
        }
    }
}

// ── Serialize ─────────────────────────────────────────────────────────────────

impl Serialize for AdvancementTrigger {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // `PlacedBlock` and `ItemUsedOnBlock` render through the same modern
        // `location_check`/`match_tool` lowering used by `render_for(None)` —
        // see #232/#233. This compatibility `Serialize` impl (used directly by
        // tests, `Criterion`, and any caller that doesn't route through
        // `render_for`) must never fall back to the old unfiltered flat
        // `conditions.block`/`conditions.item` shape, or it would silently
        // reintroduce the bug those issues fixed. The pre-item-component
        // legacy shape remains reachable only through the explicit
        // `render_for(Some(&caps))` profile-gated path.
        match self {
            AdvancementTrigger::PlacedBlock {
                block,
                item,
                location,
                state,
            } => {
                let value = render_placed_block_modern(block, item, location, state)
                    .map_err(serde::ser::Error::custom)?;
                return value.serialize(serializer);
            }
            AdvancementTrigger::ItemUsedOnBlock { item, location } => {
                let value = render_item_used_on_block_modern(item, location)
                    .map_err(serde::ser::Error::custom)?;
                return value.serialize(serializer);
            }
            _ => {}
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("trigger", self.trigger_id())?;

        match self {
            AdvancementTrigger::Tick
            | AdvancementTrigger::Impossible
            | AdvancementTrigger::StartedRiding => {}

            AdvancementTrigger::PlayerKilledEntity {
                entity,
                killing_blow,
            }
            | AdvancementTrigger::EntityKilledPlayer {
                entity,
                killing_blow,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), json_value::<_, S::Error>(k)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlayerHurtEntity { entity, damage }
            | AdvancementTrigger::EntityHurtPlayer { entity, damage } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(d) = damage {
                    cond.insert("damage".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::KilledByCrossbow {
                unique_entity_types,
                victims,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(u) = unique_entity_types {
                    cond.insert("unique_entity_types".into(), json_value::<_, S::Error>(u)?);
                }
                if let Some(v) = victims {
                    cond.insert("victims".into(), json_value::<_, S::Error>(v)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ChanneledLightning { victims } => {
                if let Some(v) = victims {
                    map.serialize_entry("conditions", &serde_json::json!({ "victims": v }))?;
                }
            }

            AdvancementTrigger::LightningStrike {
                lightning,
                bystander,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(l) = lightning {
                    cond.insert("lightning".into(), json_value::<_, S::Error>(l)?);
                }
                if let Some(b) = bystander {
                    cond.insert("bystander".into(), json_value::<_, S::Error>(b)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::InventoryChanged { slots, items } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = slots {
                    cond.insert("slots".into(), json_value::<_, S::Error>(s)?);
                }
                if !items.is_empty() {
                    cond.insert("items".into(), json_value::<_, S::Error>(items)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::RecipeUnlocked { recipe } => {
                map.serialize_entry("conditions", &serde_json::json!({ "recipe": recipe }))?;
            }

            AdvancementTrigger::UsedItem { item }
            | AdvancementTrigger::ConsumeItem { item }
            | AdvancementTrigger::UsingItem { item }
            | AdvancementTrigger::CraftedItem { item }
            | AdvancementTrigger::FilledBucket { item }
            | AdvancementTrigger::ShotCrossbow { item }
            | AdvancementTrigger::UsedTotem { item } => {
                if let Some(i) = item {
                    map.serialize_entry("conditions", &serde_json::json!({ "item": i }))?;
                }
            }

            AdvancementTrigger::EmptiedBucket { item, location } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = location {
                    cond.insert("location".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::FishingRodHooked { rod, entity, item } => {
                let mut cond = serde_json::Map::new();
                if let Some(r) = rod {
                    cond.insert("rod".into(), json_value::<_, S::Error>(r)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ThrownItemPickedUp { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ItemDurabilityChanged {
                item,
                delta,
                durability,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(d) = delta {
                    cond.insert("delta".into(), json_value::<_, S::Error>(d)?);
                }
                if let Some(d) = durability {
                    cond.insert("durability".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::BrewedPotion { potion } => {
                if let Some(p) = potion {
                    map.serialize_entry("conditions", &serde_json::json!({ "potion": p }))?;
                }
            }

            AdvancementTrigger::BeeNestDestroyed {
                block,
                item,
                num_bees_inside,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(n) = num_bees_inside {
                    cond.insert("num_bees_inside".into(), json_value::<_, S::Error>(n)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::EnchantedItem { item, levels } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = levels {
                    cond.insert("levels".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::BredAnimals {
                parent,
                partner,
                child,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(p) = parent {
                    cond.insert("parent".into(), json_value::<_, S::Error>(p)?);
                }
                if let Some(p) = partner {
                    cond.insert("partner".into(), json_value::<_, S::Error>(p)?);
                }
                if let Some(c) = child {
                    cond.insert("child".into(), json_value::<_, S::Error>(c)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::TamedAnimal { entity }
            | AdvancementTrigger::SummonedEntity { entity } => {
                if let Some(e) = entity {
                    map.serialize_entry("conditions", &serde_json::json!({ "entity": e }))?;
                }
            }

            AdvancementTrigger::PlayerInteractedWithEntity { item, entity }
            | AdvancementTrigger::TamedAnimalInteracted { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::VillagerTrade { item, villager } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(v) = villager {
                    cond.insert("villager".into(), json_value::<_, S::Error>(v)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::CuredZombieVillager { villager, zombie } => {
                let mut cond = serde_json::Map::new();
                if let Some(v) = villager {
                    cond.insert("villager".into(), json_value::<_, S::Error>(v)?);
                }
                if let Some(z) = zombie {
                    cond.insert("zombie".into(), json_value::<_, S::Error>(z)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlacedBlock { .. } => {
                unreachable!("PlacedBlock is handled by the early return above")
            }

            AdvancementTrigger::EnterBlock { block, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(s) = state {
                    cond.insert("state".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::Location { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::NetherTravel {
                entered,
                exited,
                distance,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entered {
                    cond.insert("entered".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(e) = exited {
                    cond.insert("exited".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ChangedDimension { from, to } => {
                let mut cond = serde_json::Map::new();
                if let Some(f) = from {
                    cond.insert("from".into(), Value::String(f.clone()));
                }
                if let Some(t) = to {
                    cond.insert("to".into(), Value::String(t.clone()));
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::SleptInBed { location }
            | AdvancementTrigger::HeroOfTheVillage { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::FallFromHeight {
                distance,
                start_position,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::LeveledUp { level } => {
                if let Some(l) = level {
                    map.serialize_entry("conditions", &serde_json::json!({ "level": l }))?;
                }
            }

            AdvancementTrigger::EffectsChanged { effects, source } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = effects {
                    cond.insert("effects".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(s) = source {
                    cond.insert("source".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::SlideDownBlock { block } => {
                if let Some(b) = block {
                    map.serialize_entry("conditions", &serde_json::json!({ "block": b }))?;
                }
            }

            AdvancementTrigger::TargetHit {
                signal_strength,
                projectile,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = signal_strength {
                    cond.insert("signal_strength".into(), json_value::<_, S::Error>(s)?);
                }
                if let Some(p) = projectile {
                    cond.insert("projectile".into(), json_value::<_, S::Error>(p)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ConstructBeacon { level } => {
                if let Some(l) = level {
                    map.serialize_entry("conditions", &serde_json::json!({ "level": l }))?;
                }
            }

            AdvancementTrigger::UsedEnderEye { distance } => {
                if let Some(d) = distance {
                    map.serialize_entry("conditions", &serde_json::json!({ "distance": d }))?;
                }
            }

            AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table } => {
                if let Some(lt) = loot_table {
                    map.serialize_entry("conditions", &serde_json::json!({ "loot_table": lt }))?;
                }
            }

            AdvancementTrigger::AllayDropItemOnBlock { item, location } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = location {
                    cond.insert("location".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::AvoidVibration => {}

            AdvancementTrigger::KillMobNearSculkCatalyst {
                entity,
                killing_blow,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), json_value::<_, S::Error>(k)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ItemUsedOnBlock { .. } => {
                unreachable!("ItemUsedOnBlock is handled by the early return above")
            }

            AdvancementTrigger::RideEntityInLava {
                start_position,
                distance,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), json_value::<_, S::Error>(s)?);
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::Custom { conditions, .. } => {
                if let Some(c) = conditions {
                    map.serialize_entry("conditions", c)?;
                }
            }
        }

        map.end()
    }
}

// ── Schema families (#232) ─────────────────────────────────────────────────────

/// Which vanilla advancement condition/predicate schema a target Minecraft
/// profile expects.
///
/// This is the single place that maps a [`sand_version::VersionCaps`] profile
/// to a rendering strategy — trigger rendering matches on this enum instead
/// of comparing capability flags or version strings inline. See
/// [`AdvancementTrigger::render_for`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementSchemaFamily {
    /// Pre item-component era (pre-1.20.5). [`AdvancementTrigger::PlacedBlock`]
    /// and [`AdvancementTrigger::ItemUsedOnBlock`] render through the
    /// historical flat `conditions.block`/`conditions.item` shape here.
    ///
    /// **Known limitation:** unlike the modern family below, this flat shape
    /// has *not* been verified against a real pre-1.20.5 vanilla server.
    /// Historical research for #231/#232 found no authoritative evidence
    /// that `placed_block`/`item_used_on_block` ever accepted flat
    /// `conditions.block`/`conditions.item` fields at any version — the
    /// `location`/`location_check`/`match_tool` composition these triggers
    /// use predates the 1.20.5 item-component overhaul by years. It is
    /// possible this family has the same "filter silently ignored" defect
    /// #231 fixed for the modern family. This PR does not change legacy
    /// output without verified proof (existing supported-profile output is
    /// preserved per project policy), and does not implement the
    /// pre-component item-predicate schema (`tag`/`nbt`-based matching) that
    /// would be needed to correctly filter `item` on this family — that is
    /// full item-model work owned by #229. Filed as a follow-up: verify
    /// `placed_block`/`item_used_on_block` semantics on a real pre-1.20.5
    /// server and, if broken, apply the same `location`/`match_tool` fix
    /// used for the modern family here.
    Legacy,
    /// 1.20.5+ item-component era (includes every currently-supported 26.x
    /// profile). `PlacedBlock`/`ItemUsedOnBlock` render through
    /// `conditions.location` wrapping `minecraft:location_check` (block) and
    /// `minecraft:match_tool` (item), with item predicates using the
    /// `components` (exact)/`predicates` (partial) keys. Verified against a
    /// real, manually-confirmed-working Minecraft 26.2 JSON document — see
    /// `placed_block_modern_render_matches_vanilla_location_check_and_match_tool`.
    LocationConditionItemComponents,
}

impl AdvancementSchemaFamily {
    /// Map a target profile's capabilities to its advancement schema family.
    ///
    /// `caps` is `None` on the unprofiled compatibility export path, treated
    /// the same as a fully item-component-capable modern profile (matching
    /// the `VersionCaps::all_enabled()` convention used elsewhere in Sand).
    pub fn for_caps(caps: Option<&sand_version::VersionCaps>) -> Self {
        if caps.is_none_or(|c| c.supports(sand_version::ComponentFeature::ItemComponents)) {
            Self::LocationConditionItemComponents
        } else {
            Self::Legacy
        }
    }
}

/// Which advancement trigger/field a rendered [`ItemPredicate`] is being
/// converted for.
///
/// This is a narrowly-scoped, advancement-rendering-internal analog of the
/// consumer-aware matcher conversion the full shared item model (#229) will
/// eventually own. It exists so diagnostics can name the exact trigger/field
/// an unsupported item-predicate conversion was requested for, and so #229
/// has a documented seam to integrate with rather than needing to redesign
/// advancement export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementItemConsumer {
    /// The tool/item filter for [`AdvancementTrigger::PlacedBlock`], rendered
    /// as a `minecraft:match_tool` condition in the modern schema family.
    PlacedBlockTool,
    /// The tool/item filter for [`AdvancementTrigger::ItemUsedOnBlock`],
    /// rendered as a `minecraft:match_tool` condition in the modern schema family.
    ItemUsedOnBlockTool,
}

impl AdvancementItemConsumer {
    /// The vanilla trigger ID this consumer belongs to, for diagnostics.
    pub const fn trigger_id(self) -> &'static str {
        match self {
            Self::PlacedBlockTool => "minecraft:placed_block",
            Self::ItemUsedOnBlockTool => "minecraft:item_used_on_block",
        }
    }
}

// ── Version-aware rendering (#231, #232, #233) ─────────────────────────────────

impl AdvancementTrigger {
    /// Render this trigger's `{"trigger": ..., "conditions": ...}` JSON for a
    /// specific Minecraft version's predicate schema.
    ///
    /// Most trigger variants have one stable JSON representation across every
    /// Sand-supported target and simply delegate to [`Serialize`]. Two
    /// variants — [`AdvancementTrigger::PlacedBlock`] and
    /// [`AdvancementTrigger::ItemUsedOnBlock`] — additionally filter by the
    /// item used to place/interact with the block, and render differently
    /// per [`AdvancementSchemaFamily`]. Minecraft's modern
    /// (1.20.5+ item-component era) schema expresses that filter as a
    /// `conditions.location` array of `minecraft:location_check` /
    /// `minecraft:match_tool` loot conditions, not the direct `block`/`item`
    /// fields this crate used to emit. Emitting the direct fields makes the
    /// generated advancement fire unconditionally in-game — see #231/#233.
    ///
    /// This never silently drops a filter: if a caller supplies both the
    /// trigger-level `block`/`state` shorthand *and* a `location` predicate
    /// that already sets `block`, rendering fails with an actionable
    /// [`SandError`](crate::error::SandError) instead of picking one silently.
    /// Likewise, requesting an item filter on [`AdvancementSchemaFamily::Legacy`]
    /// fails with an actionable error instead of emitting an item-component-era
    /// JSON shape (`components`/`predicates`) that legacy profiles don't
    /// recognize — see [`AdvancementSchemaFamily::Legacy`]'s docs.
    pub fn render_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<Value> {
        let family = AdvancementSchemaFamily::for_caps(caps);

        match (self, family) {
            (
                AdvancementTrigger::PlacedBlock {
                    block,
                    item,
                    location,
                    state,
                },
                AdvancementSchemaFamily::LocationConditionItemComponents,
            ) => render_placed_block_modern(block, item, location, state),

            (
                AdvancementTrigger::ItemUsedOnBlock { item, location },
                AdvancementSchemaFamily::LocationConditionItemComponents,
            ) => render_item_used_on_block_modern(item, location),

            // Pre-item-component targets never had `location_check`/`match_tool`
            // wrapping verified for these two triggers, and this crate has no
            // pre-component item-predicate model — reject an item filter with
            // an actionable diagnostic rather than emit a schema shape the
            // target version doesn't recognize (see `AdvancementSchemaFamily::Legacy`).
            (
                AdvancementTrigger::PlacedBlock { item: Some(_), .. },
                AdvancementSchemaFamily::Legacy,
            ) => Err(unsupported_legacy_item_filter(
                AdvancementItemConsumer::PlacedBlockTool,
            )),
            (
                AdvancementTrigger::ItemUsedOnBlock { item: Some(_), .. },
                AdvancementSchemaFamily::Legacy,
            ) => Err(unsupported_legacy_item_filter(
                AdvancementItemConsumer::ItemUsedOnBlockTool,
            )),

            (
                AdvancementTrigger::PlacedBlock {
                    block,
                    location,
                    state,
                    ..
                },
                AdvancementSchemaFamily::Legacy,
            ) => Ok(render_placed_block_legacy(block, location, state)),

            (
                AdvancementTrigger::ItemUsedOnBlock { location, .. },
                AdvancementSchemaFamily::Legacy,
            ) => Ok(render_item_used_on_block_legacy(location)),

            _ => serde_json::to_value(self).map_err(crate::error::SandError::Serialization),
        }
    }
}

/// Build the actionable diagnostic for requesting an item filter on
/// [`AdvancementSchemaFamily::Legacy`], where this crate has no verified,
/// correct representation.
fn unsupported_legacy_item_filter(consumer: AdvancementItemConsumer) -> crate::error::SandError {
    crate::error::SandError::ComponentValidation {
        location: ResourceLocation::new("sand", "advancement_trigger")
            .expect("static resource location is always valid"),
        kind: consumer.trigger_id().to_string(),
        field: "conditions.item".to_string(),
        message: format!(
            "`{}` item filtering for this target's pre-item-component profile is not \
             implemented — Sand's item predicate model only renders the 1.20.5+ \
             `components`/`predicates` schema, which older profiles do not recognize. \
             Target a supported item-component profile (every currently-supported 1.20.5+ \
             and 26.x profile), drop the item filter and rely on the block/location \
             condition only, or use `AdvancementTrigger::Custom`/raw JSON with a \
             manually-verified legacy predicate shape.",
            consumer.trigger_id()
        ),
    }
}

/// Pre-item-component-era flat rendering for [`AdvancementTrigger::PlacedBlock`],
/// preserved only for targets where `render_for` determines the modern
/// `location_check`/`match_tool` schema is unsupported. Not used by the
/// compatibility `Serialize` impl, which always renders the modern (correct)
/// shape — see the `Serialize for AdvancementTrigger` impl's doc comment.
fn render_placed_block_legacy(
    block: &Option<String>,
    location: &Option<LocationPredicate>,
    state: &Option<HashMap<String, String>>,
) -> Value {
    let mut cond = serde_json::Map::new();
    if let Some(b) = block {
        cond.insert("block".to_string(), Value::String(b.clone()));
    }
    if let Some(l) = location {
        cond.insert(
            "location".to_string(),
            serde_json::to_value(l).unwrap_or(Value::Null),
        );
    }
    if let Some(s) = state {
        cond.insert(
            "state".to_string(),
            serde_json::to_value(s).unwrap_or(Value::Null),
        );
    }

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:placed_block".to_string()),
    );
    if !cond.is_empty() {
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Value::Object(map)
}

/// Pre-item-component-era flat rendering for [`AdvancementTrigger::ItemUsedOnBlock`].
/// See [`render_placed_block_legacy`] for when this is used.
fn render_item_used_on_block_legacy(location: &Option<LocationPredicate>) -> Value {
    let mut cond = serde_json::Map::new();
    if let Some(l) = location {
        cond.insert(
            "location".to_string(),
            serde_json::to_value(l).unwrap_or(Value::Null),
        );
    }

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:item_used_on_block".to_string()),
    );
    if !cond.is_empty() {
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Value::Object(map)
}

/// Build the `minecraft:location_check` / `minecraft:match_tool` condition
/// array shared by [`AdvancementTrigger::PlacedBlock`] and
/// [`AdvancementTrigger::ItemUsedOnBlock`]'s modern rendering.
fn render_location_and_item_conditions(
    consumer: AdvancementItemConsumer,
    location: &Option<LocationPredicate>,
    item: &Option<ItemPredicate>,
    block_shorthand: Option<&String>,
    state_shorthand: &Option<HashMap<String, String>>,
) -> crate::error::Result<Vec<Value>> {
    let mut loc = location.clone().unwrap_or_default();

    if block_shorthand.is_some() || state_shorthand.is_some() {
        if loc.has_block() {
            return Err(crate::error::SandError::ComponentValidation {
                location: ResourceLocation::new("sand", "advancement_trigger")
                    .expect("static resource location is always valid"),
                kind: consumer.trigger_id().to_string(),
                field: "conditions.block".to_string(),
                message: "both the direct `block`/`state` shorthand and an explicit \
                    `location` predicate that may already set `block` (a typed `block`, \
                    or a `LocationPredicate::raw(...)` escape hatch whose contents Sand \
                    cannot inspect) were set; specify the block filter in exactly one place"
                    .to_string(),
            });
        }
        let mut bp = crate::predicates::BlockPredicate::new();
        if let Some(block) = block_shorthand {
            bp = bp.blocks(vec![block.clone()]);
        }
        if let Some(state) = state_shorthand {
            bp = bp.state(state.clone());
        }
        loc = loc.block(bp);
    }

    let mut conditions = Vec::new();
    if !loc.is_empty() {
        conditions.push(serde_json::json!({
            "condition": "minecraft:location_check",
            "predicate": loc,
        }));
    }
    if let Some(item) = item {
        conditions.push(serde_json::json!({
            "condition": "minecraft:match_tool",
            "predicate": item,
        }));
    }
    Ok(conditions)
}

fn render_placed_block_modern(
    block: &Option<String>,
    item: &Option<ItemPredicate>,
    location: &Option<LocationPredicate>,
    state: &Option<HashMap<String, String>>,
) -> crate::error::Result<Value> {
    let conditions = render_location_and_item_conditions(
        AdvancementItemConsumer::PlacedBlockTool,
        location,
        item,
        block.as_ref(),
        state,
    )?;

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:placed_block".to_string()),
    );
    if !conditions.is_empty() {
        let mut cond = serde_json::Map::new();
        cond.insert("location".to_string(), Value::Array(conditions));
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Ok(Value::Object(map))
}

fn render_item_used_on_block_modern(
    item: &Option<ItemPredicate>,
    location: &Option<LocationPredicate>,
) -> crate::error::Result<Value> {
    let conditions = render_location_and_item_conditions(
        AdvancementItemConsumer::ItemUsedOnBlockTool,
        location,
        item,
        None,
        &None,
    )?;

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:item_used_on_block".to_string()),
    );
    if !conditions.is_empty() {
        let mut cond = serde_json::Map::new();
        cond.insert("location".to_string(), Value::Array(conditions));
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Ok(Value::Object(map))
}

// ── Criterion ─────────────────────────────────────────────────────────────────

/// A single criterion for an advancement that must be met for progress.
pub struct Criterion {
    pub trigger: AdvancementTrigger,
}

impl Criterion {
    /// Creates a new criterion with the specified trigger.
    pub fn new(trigger: AdvancementTrigger) -> Self {
        Self { trigger }
    }
}

impl Serialize for Criterion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.trigger.serialize(serializer)
    }
}

// ── AdvancementRewards ────────────────────────────────────────────────────────

/// Rewards granted to the player when an advancement is completed.
pub struct AdvancementRewards {
    pub recipes: Vec<String>,
    pub loot: Vec<String>,
    pub experience: i32,
    pub function: Option<String>,
}

impl AdvancementRewards {
    /// Creates a new advancement rewards container with no rewards set.
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
            loot: Vec::new(),
            experience: 0,
            function: None,
        }
    }

    /// Adds a recipe unlock reward.
    pub fn recipe(mut self, recipe: impl Into<String>) -> Self {
        self.recipes.push(recipe.into());
        self
    }

    /// Adds a loot table reward.
    pub fn loot(mut self, loot: impl Into<String>) -> Self {
        self.loot.push(loot.into());
        self
    }

    /// Sets the experience points awarded.
    pub fn experience(mut self, xp: i32) -> Self {
        self.experience = xp;
        self
    }

    /// Sets a function to execute as a reward.
    pub fn function(mut self, func: impl Into<String>) -> Self {
        self.function = Some(func.into());
        self
    }

    fn validate(&self) -> Result<(), (String, String)> {
        if self.experience < 0 {
            return Err((
                "rewards.experience".into(),
                "experience reward must be non-negative".into(),
            ));
        }
        for (index, recipe) in self.recipes.iter().enumerate() {
            validate_resource_id(recipe, &format!("rewards.recipes[{index}]"))
                .map_err(split_validation_message)?;
        }
        for (index, loot) in self.loot.iter().enumerate() {
            validate_resource_id(loot, &format!("rewards.loot[{index}]"))
                .map_err(split_validation_message)?;
        }
        if let Some(function) = &self.function {
            validate_resource_id(function, "rewards.function").map_err(split_validation_message)?;
        }
        Ok(())
    }
}

fn split_validation_message(message: String) -> (String, String) {
    message
        .split_once(": ")
        .map(|(path, detail)| (path.to_string(), detail.to_string()))
        .unwrap_or_else(|| ("advancement".into(), message))
}

impl Default for AdvancementRewards {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for AdvancementRewards {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        if !self.recipes.is_empty() {
            map.serialize_entry("recipes", &self.recipes)?;
        }
        if !self.loot.is_empty() {
            map.serialize_entry("loot", &self.loot)?;
        }
        if self.experience != 0 {
            map.serialize_entry("experience", &self.experience)?;
        }
        if let Some(ref f) = self.function {
            map.serialize_entry("function", f)?;
        }
        map.end()
    }
}

// ── Advancement ───────────────────────────────────────────────────────────────

/// A complete advancement definition for a Minecraft datapack.
pub struct Advancement {
    pub location: ResourceLocation,
    pub parent: Option<String>,
    pub display: Option<AdvancementDisplay>,
    pub criteria: HashMap<String, Criterion>,
    pub requirements: Option<Vec<Vec<String>>>,
    pub rewards: Option<AdvancementRewards>,
    pub sends_telemetry_data: bool,
}

impl Advancement {
    /// Creates a new advancement with the specified resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            parent: None,
            display: None,
            criteria: HashMap::new(),
            requirements: None,
            rewards: None,
            sends_telemetry_data: false,
        }
    }

    /// Sets the parent advancement.
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Sets the display information for this advancement.
    pub fn display(mut self, display: AdvancementDisplay) -> Self {
        self.display = Some(display);
        self
    }

    /// Adds a criterion with the specified name.
    pub fn criterion(mut self, name: impl Into<String>, criterion: Criterion) -> Self {
        self.criteria.insert(name.into(), criterion);
        self
    }

    /// Sets the requirements specifying how criteria must be completed.
    pub fn requirements(mut self, requirements: Vec<Vec<String>>) -> Self {
        self.requirements = Some(requirements);
        self
    }

    /// Sets the rewards given when this advancement is completed.
    pub fn rewards(mut self, rewards: AdvancementRewards) -> Self {
        self.rewards = Some(rewards);
        self
    }

    /// Sets whether telemetry data is sent for this advancement.
    pub fn sends_telemetry_data(mut self, v: bool) -> Self {
        self.sends_telemetry_data = v;
        self
    }

    fn validation_error(
        &self,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> crate::error::SandError {
        crate::error::SandError::ComponentValidation {
            location: self.location.clone(),
            kind: "advancement".to_string(),
            field: field.into(),
            message: message.into(),
        }
    }
}

impl DatapackComponent for Advancement {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> crate::error::Result<()> {
        if self.criteria.is_empty() {
            return Err(self.validation_error("criteria", "at least one criterion is required"));
        }

        if let Some(parent) = &self.parent {
            validate_resource_id(parent, "parent")
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }
        if let Some(display) = &self.display {
            validate_resource_id(&display.icon.id, "display.icon.id")
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
            if let Some(background) = &display.background {
                validate_resource_id(background, "display.background")
                    .map_err(split_validation_message)
                    .map_err(|(field, message)| self.validation_error(field, message))?;
            }
        }
        if let Some(rewards) = &self.rewards {
            rewards
                .validate()
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }

        let mut criteria = self.criteria.iter().collect::<Vec<_>>();
        criteria.sort_by_key(|(name, _)| *name);
        for (name, criterion) in criteria {
            ResourceLocation::new("sand", name).map_err(|_| {
                self.validation_error(
                    format!("criteria.{name}"),
                    "criterion name must be non-empty and contain only [a-z0-9_./-]",
                )
            })?;
            let path = format!("criteria.{name}");
            criterion
                .trigger
                .validate_at(&path)
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }

        if let Some(requirements) = &self.requirements {
            if requirements.is_empty() {
                return Err(self.validation_error(
                    "requirements",
                    "requirements must contain at least one group",
                ));
            }
            let mut referenced = std::collections::HashSet::new();
            for (group_index, group) in requirements.iter().enumerate() {
                if group.is_empty() {
                    return Err(self.validation_error(
                        format!("requirements[{group_index}]"),
                        "requirement group must contain at least one criterion",
                    ));
                }
                for (criterion_index, name) in group.iter().enumerate() {
                    if !self.criteria.contains_key(name) {
                        return Err(self.validation_error(
                            format!("requirements[{group_index}][{criterion_index}]"),
                            format!("references missing criterion `{name}`"),
                        ));
                    }
                    referenced.insert(name.as_str());
                }
            }
            if let Some(missing) = self
                .criteria
                .keys()
                .filter(|name| !referenced.contains(name.as_str()))
                .min()
            {
                return Err(self.validation_error(
                    "requirements",
                    format!("criterion `{missing}` is not referenced by any requirement group"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        self.try_to_json_for(None)
            .unwrap_or_else(|error| panic!("advancement serialization failed: {error}"))
    }

    fn try_content(&self) -> crate::error::Result<ComponentContent> {
        self.try_content_for(None)
    }

    fn try_content_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<ComponentContent> {
        self.validate()?;
        self.try_to_json_for(caps).map(ComponentContent::Json)
    }

    fn component_dir(&self) -> &'static str {
        "advancement"
    }
}

impl Advancement {
    /// Serialize this advancement's JSON, rendering each criterion's trigger
    /// through [`AdvancementTrigger::render_for`] for the given profile.
    ///
    /// `caps` is `None` on the compatibility path, treated the same as a
    /// fully-capable modern profile — see [`AdvancementTrigger::render_for`].
    fn try_to_json_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<Value> {
        let mut map = serde_json::Map::new();

        if let Some(ref p) = self.parent {
            map.insert("parent".into(), Value::String(p.clone()));
        }
        if let Some(ref d) = self.display {
            map.insert(
                "display".into(),
                serde_json::to_value(d).map_err(crate::error::SandError::Serialization)?,
            );
        }

        let mut criteria_map = serde_json::Map::new();
        for (name, criterion) in &self.criteria {
            let trigger_json = criterion.trigger.render_for(caps).map_err(|error| {
                self.validation_error(format!("criteria.{name}"), error.to_string())
            })?;
            criteria_map.insert(name.clone(), trigger_json);
        }
        map.insert("criteria".into(), Value::Object(criteria_map));

        // Always emit `requirements`. Minecraft treats a missing/empty `requirements`
        // array as "no criteria required", which makes the advancement fire
        // unconditionally regardless of how restrictive the criteria conditions are
        // (see #233). When the caller hasn't supplied an explicit group layout, derive
        // a single AND-group covering every defined criterion — the correct default
        // for the common single- and multi-criterion "all must complete" case.
        let requirements: Vec<Vec<String>> = match &self.requirements {
            Some(reqs) => reqs.clone(),
            None => {
                let mut names: Vec<String> = self.criteria.keys().cloned().collect();
                names.sort();
                // `validate()` rejects zero-criteria advancements, but `to_json()`/
                // `content()` are documented infallible escape hatches that can be
                // called without validating first — don't synthesize a structurally
                // invalid single empty requirement group (`[[]]`) in that case.
                if names.is_empty() {
                    vec![]
                } else {
                    vec![names]
                }
            }
        };
        map.insert(
            "requirements".into(),
            serde_json::to_value(&requirements).map_err(crate::error::SandError::Serialization)?,
        );
        if let Some(ref r) = self.rewards {
            map.insert(
                "rewards".into(),
                serde_json::to_value(r).map_err(crate::error::SandError::Serialization)?,
            );
        }
        if self.sends_telemetry_data {
            map.insert("sends_telemetry_data".into(), Value::Bool(true));
        }

        Ok(Value::Object(map))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicates::{
        DamagePredicate, EntityPredicate, FloatRange, IntRange, ItemPredicate, LocationPredicate,
    };

    #[test]
    fn tick_trigger_serializes() {
        let t = AdvancementTrigger::Tick;
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:tick");
    }

    #[test]
    fn consume_item_typed() {
        let t = AdvancementTrigger::ConsumeItem {
            item: Some(ItemPredicate::id("minecraft:golden_apple")),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:consume_item");
        assert_eq!(
            v["conditions"]["item"]["items"],
            serde_json::json!(["minecraft:golden_apple"])
        );
    }

    #[test]
    fn player_killed_entity_typed() {
        let t = AdvancementTrigger::PlayerKilledEntity {
            entity: Some(EntityPredicate::type_("minecraft:ender_dragon")),
            killing_blow: None,
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:player_killed_entity");
        assert_eq!(v["conditions"]["entity"]["type"], "minecraft:ender_dragon");
    }

    #[test]
    fn player_hurt_entity_with_damage() {
        let t = AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: Some(DamagePredicate::new().dealt(FloatRange::at_least(5.0))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:player_hurt_entity");
        assert_eq!(v["conditions"]["damage"]["dealt"]["min"], 5.0);
    }

    #[test]
    fn leveled_up_typed() {
        let t = AdvancementTrigger::LeveledUp {
            level: Some(IntRange::at_least(30)),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["conditions"]["level"]["min"], 30);
    }

    #[test]
    fn leveled_up_is_rejected_before_advancement_export() {
        let trigger = AdvancementTrigger::LeveledUp { level: None };
        let error = trigger.validate_for_target().unwrap_err();
        assert!(error.contains("minecraft:leveled_up"));
        assert!(error.contains("experience query"));
    }

    #[test]
    fn inventory_changed_items() {
        let t = AdvancementTrigger::InventoryChanged {
            slots: None,
            items: vec![ItemPredicate::id("minecraft:diamond")],
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(
            v["conditions"]["items"][0]["items"],
            serde_json::json!(["minecraft:diamond"])
        );
    }

    #[test]
    fn location_trigger_typed() {
        let t = AdvancementTrigger::Location {
            location: Some(LocationPredicate::new().biome("minecraft:plains")),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["conditions"]["location"]["biome"], "minecraft:plains");
    }

    #[test]
    fn custom_trigger_escape_hatch() {
        use crate::raw::RawJson;
        let t = AdvancementTrigger::Custom {
            trigger: "mymod:do_thing".into(),
            conditions: Some(RawJson::new(serde_json::json!({"count": 5}))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "mymod:do_thing");
        assert_eq!(v["conditions"]["count"], 5);
    }

    #[test]
    fn custom_trigger_no_conditions() {
        let t = AdvancementTrigger::Custom {
            trigger: "minecraft:tick".into(),
            conditions: None,
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:tick");
        assert!(v.get("conditions").is_none());
    }

    #[test]
    fn advancement_full_round_trip() {
        let adv = Advancement::new("test:adv".parse().unwrap())
            .criterion(
                "killed_dragon",
                Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                    entity: Some(EntityPredicate::type_("minecraft:ender_dragon")),
                    killing_blow: None,
                }),
            )
            .rewards(
                AdvancementRewards::new()
                    .experience(1000)
                    .function("test:reward"),
            );
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["killed_dragon"]["conditions"]["entity"]["type"],
            "minecraft:ender_dragon"
        );
        assert_eq!(json["rewards"]["experience"], 1000);
    }

    // ── Trigger ID golden tests ───────────────────────────────────────────────
    // One test per trigger variant asserting the exact vanilla trigger ID.

    fn trigger_id(t: &AdvancementTrigger) -> &str {
        t.trigger_id()
    }

    macro_rules! trigger_id_test {
        ($name:ident, $trigger:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(trigger_id(&$trigger), $expected);
            }
        };
    }

    trigger_id_test!(tick_id, AdvancementTrigger::Tick, "minecraft:tick");
    trigger_id_test!(
        impossible_id,
        AdvancementTrigger::Impossible,
        "minecraft:impossible"
    );
    trigger_id_test!(
        player_killed_entity_id,
        AdvancementTrigger::PlayerKilledEntity {
            entity: None,
            killing_blow: None
        },
        "minecraft:player_killed_entity"
    );
    trigger_id_test!(
        entity_killed_player_id,
        AdvancementTrigger::EntityKilledPlayer {
            entity: None,
            killing_blow: None
        },
        "minecraft:entity_killed_player"
    );
    trigger_id_test!(
        player_hurt_entity_id,
        AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: None
        },
        "minecraft:player_hurt_entity"
    );
    trigger_id_test!(
        entity_hurt_player_id,
        AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None
        },
        "minecraft:entity_hurt_player"
    );
    trigger_id_test!(
        killed_by_crossbow_id,
        AdvancementTrigger::KilledByCrossbow {
            unique_entity_types: None,
            victims: None
        },
        "minecraft:killed_by_crossbow"
    );
    trigger_id_test!(
        channeled_lightning_id,
        AdvancementTrigger::ChanneledLightning { victims: None },
        "minecraft:channeled_lightning"
    );
    trigger_id_test!(
        lightning_strike_id,
        AdvancementTrigger::LightningStrike {
            lightning: None,
            bystander: None
        },
        "minecraft:lightning_strike"
    );
    trigger_id_test!(
        inventory_changed_id,
        AdvancementTrigger::InventoryChanged {
            slots: None,
            items: vec![]
        },
        "minecraft:inventory_changed"
    );
    trigger_id_test!(
        recipe_unlocked_id,
        AdvancementTrigger::RecipeUnlocked {
            recipe: "test:r".into()
        },
        "minecraft:recipe_unlocked"
    );
    trigger_id_test!(
        used_item_id,
        AdvancementTrigger::UsedItem { item: None },
        "minecraft:used_item"
    );
    trigger_id_test!(
        consume_item_id,
        AdvancementTrigger::ConsumeItem { item: None },
        "minecraft:consume_item"
    );
    trigger_id_test!(
        using_item_id,
        AdvancementTrigger::UsingItem { item: None },
        "minecraft:using_item"
    );
    trigger_id_test!(
        crafted_item_id,
        AdvancementTrigger::CraftedItem { item: None },
        "minecraft:crafted_item"
    );
    trigger_id_test!(
        filled_bucket_id,
        AdvancementTrigger::FilledBucket { item: None },
        "minecraft:filled_bucket"
    );
    trigger_id_test!(
        emptied_bucket_id,
        AdvancementTrigger::EmptiedBucket {
            item: None,
            location: None
        },
        "minecraft:emptied_bucket"
    );
    trigger_id_test!(
        shot_crossbow_id,
        AdvancementTrigger::ShotCrossbow { item: None },
        "minecraft:shot_crossbow"
    );
    trigger_id_test!(
        used_totem_id,
        AdvancementTrigger::UsedTotem { item: None },
        "minecraft:used_totem"
    );
    trigger_id_test!(
        thrown_item_picked_up_id,
        AdvancementTrigger::ThrownItemPickedUp {
            item: None,
            entity: None
        },
        "minecraft:thrown_item_picked_up"
    );
    trigger_id_test!(
        item_durability_changed_id,
        AdvancementTrigger::ItemDurabilityChanged {
            item: None,
            delta: None,
            durability: None
        },
        "minecraft:item_durability_changed"
    );
    trigger_id_test!(
        brewed_potion_id,
        AdvancementTrigger::BrewedPotion { potion: None },
        "minecraft:brewed_potion"
    );
    trigger_id_test!(
        bee_nest_destroyed_id,
        AdvancementTrigger::BeeNestDestroyed {
            block: None,
            item: None,
            num_bees_inside: None
        },
        "minecraft:bee_nest_destroyed"
    );
    trigger_id_test!(
        enchanted_item_id,
        AdvancementTrigger::EnchantedItem {
            item: None,
            levels: None
        },
        "minecraft:enchanted_item"
    );
    trigger_id_test!(
        bred_animals_id,
        AdvancementTrigger::BredAnimals {
            parent: None,
            partner: None,
            child: None
        },
        "minecraft:bred_animals"
    );
    trigger_id_test!(
        tamed_animal_id,
        AdvancementTrigger::TamedAnimal { entity: None },
        "minecraft:tame_animal"
    );
    trigger_id_test!(
        summoned_entity_id,
        AdvancementTrigger::SummonedEntity { entity: None },
        "minecraft:summoned_entity"
    );
    trigger_id_test!(
        player_interacted_with_entity_id,
        AdvancementTrigger::PlayerInteractedWithEntity {
            item: None,
            entity: None
        },
        "minecraft:player_interacted_with_entity"
    );
    trigger_id_test!(
        fishing_rod_hooked_id,
        AdvancementTrigger::FishingRodHooked {
            rod: None,
            entity: None,
            item: None
        },
        "minecraft:fishing_rod_hooked"
    );
    trigger_id_test!(
        villager_trade_id,
        AdvancementTrigger::VillagerTrade {
            item: None,
            villager: None
        },
        "minecraft:villager_trade"
    );
    trigger_id_test!(
        cured_zombie_villager_id,
        AdvancementTrigger::CuredZombieVillager {
            villager: None,
            zombie: None
        },
        "minecraft:cured_zombie_villager"
    );
    trigger_id_test!(
        placed_block_id,
        AdvancementTrigger::PlacedBlock {
            block: None,
            item: None,
            location: None,
            state: None
        },
        "minecraft:placed_block"
    );
    trigger_id_test!(
        enter_block_id,
        AdvancementTrigger::EnterBlock {
            block: None,
            state: None
        },
        "minecraft:enter_block"
    );
    trigger_id_test!(
        location_id,
        AdvancementTrigger::Location { location: None },
        "minecraft:location"
    );
    trigger_id_test!(
        nether_travel_id,
        AdvancementTrigger::NetherTravel {
            entered: None,
            exited: None,
            distance: None
        },
        "minecraft:nether_travel"
    );
    trigger_id_test!(
        changed_dimension_id,
        AdvancementTrigger::ChangedDimension {
            from: None,
            to: None
        },
        "minecraft:changed_dimension"
    );
    trigger_id_test!(
        slept_in_bed_id,
        AdvancementTrigger::SleptInBed { location: None },
        "minecraft:slept_in_bed"
    );
    trigger_id_test!(
        fall_from_height_id,
        AdvancementTrigger::FallFromHeight {
            distance: None,
            start_position: None
        },
        "minecraft:fall_from_height"
    );
    trigger_id_test!(
        slide_down_block_id,
        AdvancementTrigger::SlideDownBlock { block: None },
        "minecraft:slide_down_block"
    );
    trigger_id_test!(
        target_hit_id,
        AdvancementTrigger::TargetHit {
            signal_strength: None,
            projectile: None
        },
        "minecraft:target_hit"
    );
    trigger_id_test!(
        hero_of_the_village_id,
        AdvancementTrigger::HeroOfTheVillage { location: None },
        "minecraft:hero_of_the_village"
    );
    trigger_id_test!(
        player_generates_container_loot_id,
        AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table: None },
        "minecraft:player_generates_container_loot"
    );
    trigger_id_test!(
        leveled_up_id,
        AdvancementTrigger::LeveledUp { level: None },
        "minecraft:leveled_up"
    );
    trigger_id_test!(
        effects_changed_id,
        AdvancementTrigger::EffectsChanged {
            effects: None,
            source: None
        },
        "minecraft:effects_changed"
    );
    trigger_id_test!(
        started_riding_id,
        AdvancementTrigger::StartedRiding,
        "minecraft:started_riding"
    );
    trigger_id_test!(
        construct_beacon_id,
        AdvancementTrigger::ConstructBeacon { level: None },
        "minecraft:construct_beacon"
    );
    trigger_id_test!(
        used_ender_eye_id,
        AdvancementTrigger::UsedEnderEye { distance: None },
        "minecraft:used_ender_eye"
    );
    // New 1.19+ triggers
    trigger_id_test!(
        allay_drop_item_on_block_id,
        AdvancementTrigger::AllayDropItemOnBlock {
            item: None,
            location: None
        },
        "minecraft:allay_drop_item_on_block"
    );
    trigger_id_test!(
        avoid_vibration_id,
        AdvancementTrigger::AvoidVibration,
        "minecraft:avoid_vibration"
    );
    trigger_id_test!(
        kill_mob_near_sculk_catalyst_id,
        AdvancementTrigger::KillMobNearSculkCatalyst {
            entity: None,
            killing_blow: None
        },
        "minecraft:kill_mob_near_sculk_catalyst"
    );
    trigger_id_test!(
        item_used_on_block_id,
        AdvancementTrigger::ItemUsedOnBlock {
            item: None,
            location: None
        },
        "minecraft:item_used_on_block"
    );
    trigger_id_test!(
        ride_entity_in_lava_id,
        AdvancementTrigger::RideEntityInLava {
            start_position: None,
            distance: None
        },
        "minecraft:ride_entity_in_lava"
    );

    #[test]
    fn advancement_range_validation_retains_owner_and_criterion_path() {
        let advancement = Advancement::new("test:bad_level".parse().unwrap()).criterion(
            "level_up",
            Criterion::new(AdvancementTrigger::LeveledUp {
                level: Some(IntRange::between(10, 2)),
            }),
        );
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("test:bad_level"));
        assert!(error.contains("criteria.level_up.conditions.level"));
    }

    #[test]
    fn advancement_non_finite_range_is_rejected_before_serialization() {
        let advancement = Advancement::new("test:bad_distance".parse().unwrap()).criterion(
            "eye",
            Criterion::new(AdvancementTrigger::UsedEnderEye {
                distance: Some(FloatRange::at_least(f64::NAN)),
            }),
        );
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("criteria.eye.conditions.distance.min"));
        assert!(error.contains("finite"));

        let nested = Advancement::new("test:bad_damage".parse().unwrap()).criterion(
            "hurt",
            Criterion::new(AdvancementTrigger::PlayerHurtEntity {
                entity: None,
                damage: Some(DamagePredicate::new().dealt(FloatRange::at_most(f64::INFINITY))),
            }),
        );
        let nested_error = nested.try_content().unwrap_err().to_string();
        assert!(nested_error.contains("criteria.hurt.conditions.damage.dealt.max"));
    }

    #[test]
    fn advancement_valid_and_custom_content_remain_compatible() {
        let valid = Advancement::new("test:valid_level".parse().unwrap()).criterion(
            "level",
            Criterion::new(AdvancementTrigger::ConstructBeacon {
                level: Some(IntRange::between(1, 4)),
            }),
        );
        assert_eq!(valid.try_content().unwrap(), valid.content());

        let custom = Advancement::new("test:custom".parse().unwrap()).criterion(
            "custom",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "mymod:trigger".to_string(),
                conditions: Some(RawJson::new(serde_json::json!({"anything": true}))),
            }),
        );
        assert_eq!(custom.try_content().unwrap(), custom.content());
    }

    fn tick_advancement(path: &str) -> Advancement {
        Advancement::new(format!("test:{path}").parse().unwrap())
            .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
    }

    #[test]
    fn advancement_requires_criteria() {
        let advancement = Advancement::new("test:empty".parse().unwrap());
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("test:empty"));
        assert!(error.contains("field: criteria"));
        assert!(error.contains("at least one criterion"));
    }

    #[test]
    fn advancement_criterion_names_must_be_safe_and_nonempty() {
        for name in ["", "has space", "UPPER", "bad\nname"] {
            let advancement = Advancement::new("test:bad_name".parse().unwrap())
                .criterion(name, Criterion::new(AdvancementTrigger::Tick));
            let error = advancement.try_content().unwrap_err().to_string();
            assert!(error.contains("criterion name"), "{error}");
            assert!(error.contains("test:bad_name"), "{error}");
        }
    }

    #[test]
    fn advancement_requirements_must_be_nonempty_and_reference_criteria() {
        let empty = tick_advancement("empty_requirements").requirements(Vec::new());
        assert!(
            empty
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: requirements")
        );

        let empty_group = tick_advancement("empty_group").requirements(vec![Vec::new()]);
        assert!(
            empty_group
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: requirements[0]")
        );

        let missing =
            tick_advancement("missing_requirement").requirements(vec![vec!["missing".into()]]);
        let error = missing.try_content().unwrap_err().to_string();
        assert!(error.contains("field: requirements[0][0]"), "{error}");
        assert!(error.contains("missing criterion `missing`"), "{error}");

        let unreferenced = tick_advancement("unreferenced_requirement")
            .criterion("other", Criterion::new(AdvancementTrigger::Impossible))
            .requirements(vec![vec!["tick".into()]]);
        let error = unreferenced.try_content().unwrap_err().to_string();
        assert!(error.contains("field: requirements"), "{error}");
        assert!(
            error.contains("criterion `other` is not referenced"),
            "{error}"
        );
    }

    #[test]
    fn advancement_rejects_negative_experience_rewards() {
        let advancement =
            tick_advancement("negative_xp").rewards(AdvancementRewards::new().experience(-1));
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("field: rewards.experience"), "{error}");
        assert!(error.contains("non-negative"), "{error}");
    }

    #[test]
    fn advancement_validates_top_level_resource_references() {
        let invalid_parent = tick_advancement("bad_parent").parent("not namespaced");
        assert!(
            invalid_parent
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: parent")
        );

        let display =
            AdvancementDisplay::new(AdvancementIcon::new("bad icon"), "Title", "Description");
        let invalid_icon = tick_advancement("bad_icon").display(display);
        assert!(
            invalid_icon
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: display.icon.id")
        );

        let display = AdvancementDisplay::new(
            AdvancementIcon::new("minecraft:stone"),
            "Title",
            "Description",
        )
        .background("bad background");
        let invalid_background = tick_advancement("bad_background").display(display);
        assert!(
            invalid_background
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: display.background")
        );
    }

    #[test]
    fn advancement_validates_reward_resource_references() {
        let rewards = [
            AdvancementRewards::new().recipe("bad recipe"),
            AdvancementRewards::new().loot("bad loot"),
            AdvancementRewards::new().function("bad function"),
        ];
        let fields = ["rewards.recipes[0]", "rewards.loot[0]", "rewards.function"];
        for (rewards, field) in rewards.into_iter().zip(fields) {
            let error = tick_advancement("bad_reward")
                .rewards(rewards)
                .try_content()
                .unwrap_err()
                .to_string();
            assert!(error.contains(&format!("field: {field}")), "{error}");
        }
    }

    #[test]
    fn advancement_validates_trigger_resource_reference_strings() {
        let triggers = vec![
            AdvancementTrigger::RecipeUnlocked {
                recipe: "bad recipe".into(),
            },
            AdvancementTrigger::BrewedPotion {
                potion: Some("bad potion".into()),
            },
            AdvancementTrigger::BeeNestDestroyed {
                block: Some("bad block".into()),
                item: None,
                num_bees_inside: None,
            },
            AdvancementTrigger::PlacedBlock {
                block: Some("bad block".into()),
                item: None,
                location: None,
                state: None,
            },
            AdvancementTrigger::EnterBlock {
                block: Some("bad block".into()),
                state: None,
            },
            AdvancementTrigger::SlideDownBlock {
                block: Some("bad block".into()),
            },
            AdvancementTrigger::ChangedDimension {
                from: Some("bad dimension".into()),
                to: None,
            },
            AdvancementTrigger::PlayerGeneratesContainerLoot {
                loot_table: Some("bad loot".into()),
            },
            AdvancementTrigger::Custom {
                trigger: "bad trigger".into(),
                conditions: Some(RawJson::new(serde_json::json!({"opaque": true}))),
            },
        ];

        for (index, trigger) in triggers.into_iter().enumerate() {
            let error = Advancement::new(format!("test:bad_trigger_{index}").parse().unwrap())
                .criterion("event", Criterion::new(trigger))
                .try_content()
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("valid namespaced resource location"),
                "{error}"
            );
            assert!(error.contains("criteria.event"), "{error}");
        }
    }

    #[test]
    fn advancement_valid_resource_references_and_raw_conditions_are_preserved() {
        let advancement = Advancement::new("mymod:advancement".parse().unwrap())
            .parent("mymod:parent")
            .display(
                AdvancementDisplay::new(AdvancementIcon::new("mymod:icon"), "Title", "Description")
                    .background("mymod:textures/gui/background.png"),
            )
            .criterion(
                "custom/event",
                Criterion::new(AdvancementTrigger::Custom {
                    trigger: "mymod:custom_trigger".into(),
                    conditions: Some(RawJson::new(serde_json::json!({"future": {"x": 1}}))),
                }),
            )
            .requirements(vec![vec!["custom/event".into()]])
            .rewards(
                AdvancementRewards::new()
                    .recipe("mymod:recipe")
                    .loot("mymod:loot")
                    .function("mymod:reward"),
            );

        assert_eq!(advancement.try_content().unwrap(), advancement.content());
    }

    #[test]
    fn typed_trigger_reference_constructors_preserve_vanilla_json() {
        let typed_and_legacy = [
            (
                AdvancementTrigger::recipe_unlocked("test:recipe".parse().unwrap()),
                AdvancementTrigger::RecipeUnlocked {
                    recipe: "test:recipe".into(),
                },
            ),
            (
                AdvancementTrigger::brewed_potion(crate::PotionId::Swiftness),
                AdvancementTrigger::BrewedPotion {
                    potion: Some("minecraft:swiftness".into()),
                },
            ),
            (
                AdvancementTrigger::bee_nest_destroyed(
                    Some(BlockId::minecraft("bee_nest").unwrap()),
                    None,
                    None,
                ),
                AdvancementTrigger::BeeNestDestroyed {
                    block: Some("minecraft:bee_nest".into()),
                    item: None,
                    num_bees_inside: None,
                },
            ),
            (
                AdvancementTrigger::placed_block(
                    Some(BlockId::minecraft("stone").unwrap()),
                    None,
                    None,
                    None,
                ),
                AdvancementTrigger::PlacedBlock {
                    block: Some("minecraft:stone".into()),
                    item: None,
                    location: None,
                    state: None,
                },
            ),
            (
                AdvancementTrigger::enter_block(Some(BlockId::minecraft("water").unwrap()), None),
                AdvancementTrigger::EnterBlock {
                    block: Some("minecraft:water".into()),
                    state: None,
                },
            ),
            (
                AdvancementTrigger::changed_dimension(
                    Some(DimensionId::minecraft("overworld").unwrap()),
                    Some(DimensionId::minecraft("the_nether").unwrap()),
                ),
                AdvancementTrigger::ChangedDimension {
                    from: Some("minecraft:overworld".into()),
                    to: Some("minecraft:the_nether".into()),
                },
            ),
            (
                AdvancementTrigger::slide_down_block(Some(
                    BlockId::minecraft("honey_block").unwrap(),
                )),
                AdvancementTrigger::SlideDownBlock {
                    block: Some("minecraft:honey_block".into()),
                },
            ),
            (
                AdvancementTrigger::player_generates_container_loot(Some(
                    "test:chests/reward".parse().unwrap(),
                )),
                AdvancementTrigger::PlayerGeneratesContainerLoot {
                    loot_table: Some("test:chests/reward".into()),
                },
            ),
            (
                AdvancementTrigger::custom_trigger(
                    "mymod:future_trigger".parse().unwrap(),
                    Some(RawJson::new(serde_json::json!({"future": true}))),
                ),
                AdvancementTrigger::Custom {
                    trigger: "mymod:future_trigger".into(),
                    conditions: Some(RawJson::new(serde_json::json!({"future": true}))),
                },
            ),
        ];

        for (typed, legacy) in typed_and_legacy {
            assert_eq!(
                serde_json::to_value(typed).unwrap(),
                serde_json::to_value(legacy).unwrap()
            );
        }
    }

    // ── Version-aware placed_block rendering golden tests (#232, #233) ────────

    fn elevator_wool_item_predicate() -> ItemPredicate {
        ItemPredicate::id("minecraft:white_wool").custom_data_key("elevator")
    }

    #[test]
    fn placed_block_modern_render_matches_vanilla_location_check_and_match_tool() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );

        let v = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();

        assert_eq!(v["trigger"], "minecraft:placed_block");
        let location = v["conditions"]["location"]
            .as_array()
            .expect("conditions.location must be an array");
        assert_eq!(location.len(), 2);

        assert_eq!(location[0]["condition"], "minecraft:location_check");
        assert_eq!(
            location[0]["predicate"]["block"]["blocks"],
            serde_json::json!(["minecraft:white_wool"])
        );

        assert_eq!(location[1]["condition"], "minecraft:match_tool");
        assert_eq!(
            location[1]["predicate"]["items"],
            serde_json::json!(["minecraft:white_wool"])
        );
        assert_eq!(
            location[1]["predicate"]["predicates"]["minecraft:custom_data"],
            "{elevator:1b}"
        );

        // Regression guard for #233: the old flat shape must be gone.
        assert!(v["conditions"].get("block").is_none());
        assert!(v["conditions"].get("item").is_none());
    }

    #[test]
    fn placed_block_modern_render_block_only_has_no_match_tool_condition() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 1);
        assert_eq!(location[0]["condition"], "minecraft:location_check");
    }

    #[test]
    fn placed_block_modern_render_item_only_has_no_location_check_condition() {
        let trigger = AdvancementTrigger::placed_block(
            None,
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 1);
        assert_eq!(location[0]["condition"], "minecraft:match_tool");
    }

    #[test]
    fn placed_block_unfiltered_emits_no_conditions() {
        let trigger = AdvancementTrigger::placed_block(None, None, None, None);
        let v = trigger.render_for(None).unwrap();
        assert!(v.get("conditions").is_none());
    }

    #[test]
    fn placed_block_render_for_no_profile_defaults_to_modern() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let no_profile = trigger.render_for(None).unwrap();
        let modern = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();
        assert_eq!(no_profile, modern);
    }

    #[test]
    fn schema_family_for_caps_maps_correctly() {
        assert_eq!(
            AdvancementSchemaFamily::for_caps(None),
            AdvancementSchemaFamily::LocationConditionItemComponents,
            "no profile is treated as the fully-capable modern profile"
        );
        assert_eq!(
            AdvancementSchemaFamily::for_caps(Some(&sand_version::VersionCaps::all_enabled())),
            AdvancementSchemaFamily::LocationConditionItemComponents,
        );
        assert_eq!(
            AdvancementSchemaFamily::for_caps(Some(&sand_version::VersionCaps::all_disabled())),
            AdvancementSchemaFamily::Legacy,
        );
    }

    #[test]
    fn placed_block_render_for_legacy_profile_keeps_flat_shape_for_block_only() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let v = trigger
            .render_for(Some(&sand_version::VersionCaps::all_disabled()))
            .unwrap();
        // Pre-item-component targets never had `location_check`/`match_tool`
        // wrapping for this trigger — output must keep the historical flat shape.
        // Note this intentionally diverges from `Serialize`/`render_for(None)`,
        // which always render the modern (correct) shape by default; the legacy
        // shape is reachable only by explicitly passing pre-item-component caps.
        assert_eq!(v["conditions"]["block"], "minecraft:white_wool");
        assert!(v["conditions"].get("item").is_none());
        assert!(v["conditions"].get("location").is_none());
    }

    #[test]
    fn placed_block_render_for_legacy_profile_rejects_item_filter() {
        // Sand has no verified pre-item-component item-predicate schema (#229
        // territory), so requesting an item filter on a legacy profile must fail
        // with an actionable diagnostic instead of emitting a modern-era
        // `components`/`predicates` shape the target version won't recognize.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let error = trigger
            .render_for(Some(&sand_version::VersionCaps::all_disabled()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:placed_block"));
        assert!(error.contains("pre-item-component"));
    }

    #[test]
    fn item_used_on_block_render_for_legacy_profile_rejects_item_filter() {
        let trigger = AdvancementTrigger::ItemUsedOnBlock {
            item: Some(elevator_wool_item_predicate()),
            location: None,
        };
        let error = trigger
            .render_for(Some(&sand_version::VersionCaps::all_disabled()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:item_used_on_block"));
        assert!(error.contains("pre-item-component"));
    }

    #[test]
    fn placed_block_serialize_never_uses_legacy_flat_shape() {
        // Regression guard for the "Criterion::Serialize latent trap" found in
        // review: the plain `Serialize` impl (used by `Criterion` and any
        // direct `serde_json::to_value` caller) must always render the modern,
        // correct schema — never silently fall back to the pre-#233 shape.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let via_serialize = serde_json::to_value(&trigger).unwrap();
        let via_render_for_none = trigger.render_for(None).unwrap();
        assert_eq!(via_serialize, via_render_for_none);
        assert!(via_serialize["conditions"]["location"].is_array());
        assert!(via_serialize["conditions"].get("block").is_none());
        assert!(via_serialize["conditions"].get("item").is_none());
    }

    #[test]
    fn criterion_serialize_uses_modern_placed_block_shape() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let criterion = Criterion::new(trigger);
        let v = serde_json::to_value(&criterion).unwrap();
        assert!(v["conditions"]["location"].is_array());
    }

    #[test]
    fn item_used_on_block_modern_render_uses_location_check_and_match_tool() {
        let trigger = AdvancementTrigger::ItemUsedOnBlock {
            item: Some(elevator_wool_item_predicate()),
            location: Some(LocationPredicate::new().biome("minecraft:plains")),
        };
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 2);
        assert_eq!(location[0]["condition"], "minecraft:location_check");
        assert_eq!(location[0]["predicate"]["biome"], "minecraft:plains");
        assert_eq!(location[1]["condition"], "minecraft:match_tool");
    }

    #[test]
    fn placed_block_render_rejects_conflicting_block_shorthand_and_location_block() {
        let trigger = AdvancementTrigger::PlacedBlock {
            block: Some("minecraft:white_wool".into()),
            item: None,
            location: Some(LocationPredicate::new().block(
                crate::predicates::BlockPredicate::new().blocks(vec!["minecraft:dirt".into()]),
            )),
            state: None,
        };
        let error = trigger.render_for(None).unwrap_err().to_string();
        assert!(error.contains("block"), "{error}");
    }

    #[test]
    fn placed_block_regression_dirt_and_plain_wool_are_structurally_excluded() {
        // Reproduces the #233 scenario: the generated predicate must only match
        // the exact block id and carry the custom-data partial-match condition,
        // so unrelated placements (dirt) and the un-tagged base item (plain
        // white wool with no `elevator` custom_data) cannot satisfy it.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        let blocks = location[0]["predicate"]["block"]["blocks"]
            .as_array()
            .unwrap();
        assert_eq!(blocks, &[Value::String("minecraft:white_wool".into())]);
        assert_ne!(blocks[0], "minecraft:dirt");
        // The match_tool predicate requires the `elevator` custom_data marker,
        // which plain (untagged) white wool does not carry.
        assert_eq!(
            location[1]["predicate"]["predicates"]["minecraft:custom_data"],
            "{elevator:1b}"
        );
    }

    // ── requirements auto-derivation (#233) ────────────────────────────────────

    #[test]
    fn advancement_requirements_auto_derived_single_criterion() {
        let advancement = Advancement::new("test:single".parse().unwrap())
            .criterion(
                "event",
                Criterion::new(AdvancementTrigger::placed_block(
                    Some(BlockId::minecraft("white_wool").unwrap()),
                    None,
                    None,
                    None,
                )),
            )
            .rewards(AdvancementRewards::new().function("test:reward"));
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["event"]]));
    }

    #[test]
    fn advancement_requirements_auto_derived_multi_criterion_is_one_and_group() {
        let advancement = Advancement::new("test:multi".parse().unwrap())
            .criterion("a", Criterion::new(AdvancementTrigger::Tick))
            .criterion("b", Criterion::new(AdvancementTrigger::Impossible));
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["a", "b"]]));
    }

    #[test]
    fn advancement_explicit_requirements_are_preserved_when_set() {
        let advancement = Advancement::new("test:explicit".parse().unwrap())
            .criterion("a", Criterion::new(AdvancementTrigger::Tick))
            .criterion("b", Criterion::new(AdvancementTrigger::Impossible))
            .requirements(vec![vec!["a".into()], vec!["b".into()]]);
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["a"], ["b"]]));
    }

    #[test]
    fn effects_changed_constructor_uses_typed_status_effect_keys() {
        let typed = AdvancementTrigger::effects_changed(
            [(
                crate::EffectId::Speed,
                EffectPredicate::new().amplifier(IntRange::exact(1)),
            )],
            None,
        );
        assert_eq!(
            serde_json::to_value(typed).unwrap(),
            serde_json::json!({
                "trigger": "minecraft:effects_changed",
                "conditions": {
                    "effects": {
                        "minecraft:speed": {"amplifier": 1}
                    }
                }
            })
        );

        let unfiltered = AdvancementTrigger::effects_changed_any(None);
        assert_eq!(
            serde_json::to_value(unfiltered).unwrap(),
            serde_json::json!({"trigger": "minecraft:effects_changed"})
        );
    }

    #[test]
    fn typed_trigger_ids_reject_malformed_resource_locations_at_construction() {
        assert!("bad recipe".parse::<ResourceLocation>().is_err());
        assert!(BlockId::minecraft("bad block").is_err());
        assert!(DimensionId::minecraft("bad dimension").is_err());
        assert!(PotionRegistryId::minecraft("bad potion").is_err());
        assert!(StatusEffectId::minecraft("bad effect").is_err());
    }
}
