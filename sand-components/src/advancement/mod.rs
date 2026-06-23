use std::collections::HashMap;

use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;

use crate::component::DatapackComponent;
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
    pub fn new() -> Self { Self::default() }
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
    fn trigger_id(&self) -> &str {
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
            AdvancementTrigger::Custom { trigger, .. } => trigger.as_str(),
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

            AdvancementTrigger::PlayerKilledEntity { entity, killing_blow }
            | AdvancementTrigger::EntityKilledPlayer { entity, killing_blow } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity { cond.insert("entity".into(), serde_json::to_value(e).unwrap()); }
                if let Some(k) = killing_blow { cond.insert("killing_blow".into(), serde_json::to_value(k).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::PlayerHurtEntity { entity, damage }
            | AdvancementTrigger::EntityHurtPlayer { entity, damage } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity { cond.insert("entity".into(), serde_json::to_value(e).unwrap()); }
                if let Some(d) = damage { cond.insert("damage".into(), serde_json::to_value(d).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::KilledByCrossbow { unique_entity_types, victims } => {
                let mut cond = serde_json::Map::new();
                if let Some(u) = unique_entity_types { cond.insert("unique_entity_types".into(), serde_json::to_value(u).unwrap()); }
                if let Some(v) = victims { cond.insert("victims".into(), serde_json::to_value(v).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::ChanneledLightning { victims } => {
                if let Some(v) = victims {
                    map.serialize_entry("conditions", &serde_json::json!({ "victims": v }))?;
                }
            }

            AdvancementTrigger::LightningStrike { lightning, bystander } => {
                let mut cond = serde_json::Map::new();
                if let Some(l) = lightning { cond.insert("lightning".into(), serde_json::to_value(l).unwrap()); }
                if let Some(b) = bystander { cond.insert("bystander".into(), serde_json::to_value(b).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::InventoryChanged { slots, items } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = slots { cond.insert("slots".into(), serde_json::to_value(s).unwrap()); }
                if !items.is_empty() { cond.insert("items".into(), serde_json::to_value(items).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
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
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(l) = location { cond.insert("location".into(), serde_json::to_value(l).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::FishingRodHooked { rod, entity, item } => {
                let mut cond = serde_json::Map::new();
                if let Some(r) = rod { cond.insert("rod".into(), serde_json::to_value(r).unwrap()); }
                if let Some(e) = entity { cond.insert("entity".into(), serde_json::to_value(e).unwrap()); }
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::ThrownItemPickedUp { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(e) = entity { cond.insert("entity".into(), serde_json::to_value(e).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::ItemDurabilityChanged { item, delta, durability } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(d) = delta { cond.insert("delta".into(), serde_json::to_value(d).unwrap()); }
                if let Some(d) = durability { cond.insert("durability".into(), serde_json::to_value(d).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::BrewedPotion { potion } => {
                if let Some(p) = potion {
                    map.serialize_entry("conditions", &serde_json::json!({ "potion": p }))?;
                }
            }

            AdvancementTrigger::BeeNestDestroyed { block, item, num_bees_inside } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block { cond.insert("block".into(), Value::String(b.clone())); }
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(n) = num_bees_inside { cond.insert("num_bees_inside".into(), serde_json::to_value(n).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::EnchantedItem { item, levels } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(l) = levels { cond.insert("levels".into(), serde_json::to_value(l).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::BredAnimals { parent, partner, child } => {
                let mut cond = serde_json::Map::new();
                if let Some(p) = parent { cond.insert("parent".into(), serde_json::to_value(p).unwrap()); }
                if let Some(p) = partner { cond.insert("partner".into(), serde_json::to_value(p).unwrap()); }
                if let Some(c) = child { cond.insert("child".into(), serde_json::to_value(c).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::TamedAnimal { entity } | AdvancementTrigger::SummonedEntity { entity } => {
                if let Some(e) = entity {
                    map.serialize_entry("conditions", &serde_json::json!({ "entity": e }))?;
                }
            }

            AdvancementTrigger::PlayerInteractedWithEntity { item, entity }
            | AdvancementTrigger::TamedAnimalInteracted { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(e) = entity { cond.insert("entity".into(), serde_json::to_value(e).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::VillagerTrade { item, villager } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(v) = villager { cond.insert("villager".into(), serde_json::to_value(v).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::CuredZombieVillager { villager, zombie } => {
                let mut cond = serde_json::Map::new();
                if let Some(v) = villager { cond.insert("villager".into(), serde_json::to_value(v).unwrap()); }
                if let Some(z) = zombie { cond.insert("zombie".into(), serde_json::to_value(z).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::PlacedBlock { block, item, location, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block { cond.insert("block".into(), Value::String(b.clone())); }
                if let Some(i) = item { cond.insert("item".into(), serde_json::to_value(i).unwrap()); }
                if let Some(l) = location { cond.insert("location".into(), serde_json::to_value(l).unwrap()); }
                if let Some(s) = state { cond.insert("state".into(), serde_json::to_value(s).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::EnterBlock { block, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block { cond.insert("block".into(), Value::String(b.clone())); }
                if let Some(s) = state { cond.insert("state".into(), serde_json::to_value(s).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::Location { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::NetherTravel { entered, exited, distance } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entered { cond.insert("entered".into(), serde_json::to_value(e).unwrap()); }
                if let Some(e) = exited { cond.insert("exited".into(), serde_json::to_value(e).unwrap()); }
                if let Some(d) = distance { cond.insert("distance".into(), serde_json::to_value(d).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::ChangedDimension { from, to } => {
                let mut cond = serde_json::Map::new();
                if let Some(f) = from { cond.insert("from".into(), Value::String(f.clone())); }
                if let Some(t) = to { cond.insert("to".into(), Value::String(t.clone())); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::SleptInBed { location } | AdvancementTrigger::HeroOfTheVillage { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::FallFromHeight { distance, start_position } => {
                let mut cond = serde_json::Map::new();
                if let Some(d) = distance { cond.insert("distance".into(), serde_json::to_value(d).unwrap()); }
                if let Some(s) = start_position { cond.insert("start_position".into(), serde_json::to_value(s).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::LeveledUp { level } => {
                if let Some(l) = level {
                    map.serialize_entry("conditions", &serde_json::json!({ "level": l }))?;
                }
            }

            AdvancementTrigger::EffectsChanged { effects, source } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = effects { cond.insert("effects".into(), serde_json::to_value(e).unwrap()); }
                if let Some(s) = source { cond.insert("source".into(), serde_json::to_value(s).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
            }

            AdvancementTrigger::SlideDownBlock { block } => {
                if let Some(b) = block {
                    map.serialize_entry("conditions", &serde_json::json!({ "block": b }))?;
                }
            }

            AdvancementTrigger::TargetHit { signal_strength, projectile } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = signal_strength { cond.insert("signal_strength".into(), serde_json::to_value(s).unwrap()); }
                if let Some(p) = projectile { cond.insert("projectile".into(), serde_json::to_value(p).unwrap()); }
                if !cond.is_empty() { map.serialize_entry("conditions", &Value::Object(cond))?; }
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

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        if let Some(ref p) = self.parent {
            map.insert("parent".into(), Value::String(p.clone()));
        }
        if let Some(ref d) = self.display {
            map.insert("display".into(), serde_json::to_value(d).unwrap());
        }

        let criteria_map: serde_json::Map<String, Value> = self
            .criteria
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::to_value(v).unwrap()))
            .collect();
        map.insert("criteria".into(), Value::Object(criteria_map));

        if let Some(ref reqs) = self.requirements {
            map.insert("requirements".into(), serde_json::to_value(reqs).unwrap());
        }
        if let Some(ref r) = self.rewards {
            map.insert("rewards".into(), serde_json::to_value(r).unwrap());
        }
        if self.sends_telemetry_data {
            map.insert("sends_telemetry_data".into(), Value::Bool(true));
        }

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "advancement"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicates::{DamagePredicate, EntityPredicate, FloatRange, IntRange, ItemPredicate, LocationPredicate};

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
            .rewards(AdvancementRewards::new().experience(1000).function("test:reward"));
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["killed_dragon"]["conditions"]["entity"]["type"],
            "minecraft:ender_dragon"
        );
        assert_eq!(json["rewards"]["experience"], 1000);
    }
}
