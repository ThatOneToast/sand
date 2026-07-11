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
use crate::resource_location::ResourceLocation;

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
/// instead of raw `serde_json::Value`.  The [`Custom`](AdvancementTrigger::Custom)
/// variant provides a named escape hatch for modded triggers.
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
    /// Validate stable predicate/range invariants for typed trigger conditions.
    /// Raw/custom trigger conditions remain an explicit escape hatch.
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        let conditions = format!("{path}.conditions");
        match self {
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
                        predicate.validate_at(&format!("{conditions}.effects.{effect}"))?;
                    }
                }
                if let Some(entity) = source {
                    entity.validate_at(&format!("{conditions}.source"))?;
                }
            }
            Self::Custom { .. } => {}
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
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), serde_json::to_value(k).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlayerHurtEntity { entity, damage }
            | AdvancementTrigger::EntityHurtPlayer { entity, damage } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(d) = damage {
                    cond.insert("damage".into(), serde_json::to_value(d).unwrap());
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
                    cond.insert(
                        "unique_entity_types".into(),
                        serde_json::to_value(u).unwrap(),
                    );
                }
                if let Some(v) = victims {
                    cond.insert("victims".into(), serde_json::to_value(v).unwrap());
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
                    cond.insert("lightning".into(), serde_json::to_value(l).unwrap());
                }
                if let Some(b) = bystander {
                    cond.insert("bystander".into(), serde_json::to_value(b).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::InventoryChanged { slots, items } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = slots {
                    cond.insert("slots".into(), serde_json::to_value(s).unwrap());
                }
                if !items.is_empty() {
                    cond.insert("items".into(), serde_json::to_value(items).unwrap());
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
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(l) = location {
                    cond.insert("location".into(), serde_json::to_value(l).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::FishingRodHooked { rod, entity, item } => {
                let mut cond = serde_json::Map::new();
                if let Some(r) = rod {
                    cond.insert("rod".into(), serde_json::to_value(r).unwrap());
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ThrownItemPickedUp { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
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
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(d) = delta {
                    cond.insert("delta".into(), serde_json::to_value(d).unwrap());
                }
                if let Some(d) = durability {
                    cond.insert("durability".into(), serde_json::to_value(d).unwrap());
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
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(n) = num_bees_inside {
                    cond.insert("num_bees_inside".into(), serde_json::to_value(n).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::EnchantedItem { item, levels } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(l) = levels {
                    cond.insert("levels".into(), serde_json::to_value(l).unwrap());
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
                    cond.insert("parent".into(), serde_json::to_value(p).unwrap());
                }
                if let Some(p) = partner {
                    cond.insert("partner".into(), serde_json::to_value(p).unwrap());
                }
                if let Some(c) = child {
                    cond.insert("child".into(), serde_json::to_value(c).unwrap());
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
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::VillagerTrade { item, villager } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(v) = villager {
                    cond.insert("villager".into(), serde_json::to_value(v).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::CuredZombieVillager { villager, zombie } => {
                let mut cond = serde_json::Map::new();
                if let Some(v) = villager {
                    cond.insert("villager".into(), serde_json::to_value(v).unwrap());
                }
                if let Some(z) = zombie {
                    cond.insert("zombie".into(), serde_json::to_value(z).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlacedBlock {
                block,
                item,
                location,
                state,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(l) = location {
                    cond.insert("location".into(), serde_json::to_value(l).unwrap());
                }
                if let Some(s) = state {
                    cond.insert("state".into(), serde_json::to_value(s).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::EnterBlock { block, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(s) = state {
                    cond.insert("state".into(), serde_json::to_value(s).unwrap());
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
                    cond.insert("entered".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(e) = exited {
                    cond.insert("exited".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), serde_json::to_value(d).unwrap());
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
                    cond.insert("distance".into(), serde_json::to_value(d).unwrap());
                }
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), serde_json::to_value(s).unwrap());
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
                    cond.insert("effects".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(s) = source {
                    cond.insert("source".into(), serde_json::to_value(s).unwrap());
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
                    cond.insert("signal_strength".into(), serde_json::to_value(s).unwrap());
                }
                if let Some(p) = projectile {
                    cond.insert("projectile".into(), serde_json::to_value(p).unwrap());
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
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(l) = location {
                    cond.insert("location".into(), serde_json::to_value(l).unwrap());
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
                    cond.insert("entity".into(), serde_json::to_value(e).unwrap());
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), serde_json::to_value(k).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ItemUsedOnBlock { item, location } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), serde_json::to_value(i).unwrap());
                }
                if let Some(l) = location {
                    cond.insert("location".into(), serde_json::to_value(l).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::RideEntityInLava {
                start_position,
                distance,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), serde_json::to_value(s).unwrap());
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), serde_json::to_value(d).unwrap());
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
}

impl DatapackComponent for Advancement {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> crate::error::Result<()> {
        for (name, criterion) in &self.criteria {
            let path = format!("criteria.{name}");
            criterion.trigger.validate_at(&path).map_err(|message| {
                crate::error::SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "advancement".to_string(),
                    field: path,
                    message,
                }
            })?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        self.try_to_json()
            .unwrap_or_else(|error| panic!("advancement serialization failed: {error}"))
    }

    fn try_content(&self) -> crate::error::Result<ComponentContent> {
        self.validate()?;
        self.try_to_json()
            .map(ComponentContent::Json)
            .map_err(crate::error::SandError::Serialization)
    }

    fn component_dir(&self) -> &'static str {
        "advancement"
    }
}

impl Advancement {
    fn try_to_json(&self) -> Result<Value, serde_json::Error> {
        let mut map = serde_json::Map::new();

        if let Some(ref p) = self.parent {
            map.insert("parent".into(), Value::String(p.clone()));
        }
        if let Some(ref d) = self.display {
            map.insert("display".into(), serde_json::to_value(d)?);
        }

        let mut criteria_map = serde_json::Map::new();
        for (name, criterion) in &self.criteria {
            criteria_map.insert(name.clone(), serde_json::to_value(criterion)?);
        }
        map.insert("criteria".into(), Value::Object(criteria_map));

        if let Some(ref reqs) = self.requirements {
            map.insert("requirements".into(), serde_json::to_value(reqs)?);
        }
        if let Some(ref r) = self.rewards {
            map.insert("rewards".into(), serde_json::to_value(r)?);
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
        assert_eq!(v["conditions"]["item"]["items"], "minecraft:golden_apple");
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
        assert_eq!(v["conditions"]["items"][0]["items"], "minecraft:diamond");
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
}
