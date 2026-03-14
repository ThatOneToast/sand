use std::collections::HashMap;

use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::Value;

use crate::component::DatapackComponent;
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
    pub components: Option<Value>,
}

impl AdvancementIcon {
    /// Creates a new advancement icon with the specified item ID.
    pub fn new(id: impl std::fmt::Display) -> Self {
        Self { id: id.to_string(), components: None }
    }

    /// Sets the item components (e.g., enchantments, name) for this icon.
    pub fn components(mut self, components: Value) -> Self {
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
/// Each variant corresponds to a specific advancement trigger type in Minecraft datapacks,
/// with optional condition fields for more specific matching.
pub enum AdvancementTrigger {
    Tick,
    Impossible,
    PlayerKilledEntity {
        entity: Option<Value>,
        killing_blow: Option<Value>,
    },
    EntityKilledPlayer {
        entity: Option<Value>,
        killing_blow: Option<Value>,
    },
    InventoryChanged {
        slots: Option<Value>,
        items: Vec<Value>,
    },
    RecipeUnlocked {
        recipe: String,
    },
    UsedItem {
        item: Option<Value>,
    },
    PlacedBlock {
        block: Option<String>,
        item: Option<Value>,
        location: Option<Value>,
        state: Option<HashMap<String, String>>,
    },
    BredAnimals {
        parent: Option<Value>,
        partner: Option<Value>,
        child: Option<Value>,
    },
    ConsumeItem {
        item: Option<Value>,
    },
    EnterBlock {
        block: Option<String>,
        state: Option<HashMap<String, String>>,
    },
    EnchantedItem {
        item: Option<Value>,
        levels: Option<Value>,
    },
    TamedAnimal {
        entity: Option<Value>,
    },
    SummonedEntity {
        entity: Option<Value>,
    },
    Location {
        location: Option<Value>,
    },
    NetherTravel {
        entered: Option<Value>,
        exited: Option<Value>,
        distance: Option<Value>,
    },
    UsingItem {
        item: Option<Value>,
    },
    PlayerInteractedWithEntity {
        item: Option<Value>,
        entity: Option<Value>,
    },
    /// Any trigger not covered by the typed variants.
    ///
    /// Use this to target triggers that were added to or removed from Minecraft
    /// after a given version (e.g. `minecraft:player_joined_world` which was
    /// present in 1.16–1.20.x but removed in 1.21.x).
    ///
    /// ```
    /// use sand_core::AdvancementTrigger;
    /// // Explicitly opt-in to a trigger id:
    /// let t = AdvancementTrigger::Custom {
    ///     trigger: "minecraft:tick".into(),
    ///     conditions: None,
    /// };
    /// ```
    Custom {
        trigger: String,
        conditions: Option<Value>,
    },
}

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
            AdvancementTrigger::Custom { trigger, .. } => trigger.as_str(),
        }
    }
}

impl Serialize for AdvancementTrigger {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("trigger", self.trigger_id())?;

        match self {
            AdvancementTrigger::Tick | AdvancementTrigger::Impossible => {}

            AdvancementTrigger::PlayerKilledEntity { entity, killing_blow }
            | AdvancementTrigger::EntityKilledPlayer { entity, killing_blow } => {
                if entity.is_some() || killing_blow.is_some() {
                    let mut cond = serde_json::Map::new();
                    if let Some(e) = entity {
                        cond.insert("entity".into(), e.clone());
                    }
                    if let Some(kb) = killing_blow {
                        cond.insert("killing_blow".into(), kb.clone());
                    }
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::InventoryChanged { slots, items } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = slots {
                    cond.insert("slots".into(), s.clone());
                }
                if !items.is_empty() {
                    cond.insert("items".into(), Value::Array(items.clone()));
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::RecipeUnlocked { recipe } => {
                map.serialize_entry(
                    "conditions",
                    &serde_json::json!({ "recipe": recipe }),
                )?;
            }

            AdvancementTrigger::UsedItem { item }
            | AdvancementTrigger::ConsumeItem { item }
            | AdvancementTrigger::UsingItem { item } => {
                if let Some(i) = item {
                    map.serialize_entry("conditions", &serde_json::json!({ "item": i }))?;
                }
            }

            AdvancementTrigger::PlacedBlock { block, item, location, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(i) = item {
                    cond.insert("item".into(), i.clone());
                }
                if let Some(l) = location {
                    cond.insert("location".into(), l.clone());
                }
                if let Some(s) = state {
                    cond.insert("state".into(), serde_json::to_value(s).unwrap());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::BredAnimals { parent, partner, child } => {
                let mut cond = serde_json::Map::new();
                if let Some(p) = parent {
                    cond.insert("parent".into(), p.clone());
                }
                if let Some(p) = partner {
                    cond.insert("partner".into(), p.clone());
                }
                if let Some(c) = child {
                    cond.insert("child".into(), c.clone());
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

            AdvancementTrigger::EnchantedItem { item, levels } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), i.clone());
                }
                if let Some(l) = levels {
                    cond.insert("levels".into(), l.clone());
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

            AdvancementTrigger::Location { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::NetherTravel { entered, exited, distance } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entered {
                    cond.insert("entered".into(), e.clone());
                }
                if let Some(e) = exited {
                    cond.insert("exited".into(), e.clone());
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), d.clone());
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlayerInteractedWithEntity { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), i.clone());
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), e.clone());
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
///
/// Represents an advancement with all its components: display info, criteria, requirements, and rewards.
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

    fn component_dir(&self) -> &'static str { "advancement" }
}
