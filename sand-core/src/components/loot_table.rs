use std::collections::HashMap;
use std::fmt::Display;

use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;
use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

// ── LootTableType ────────────────────────────────────────────────────────────

/// Represents the type of a Minecraft loot table (block, entity, chest, etc.).
///
/// Each variant corresponds to a specific loot table type defined in the Minecraft datapack spec,
/// with a `Custom` variant for extensibility.
pub enum LootTableType {
    Empty,
    Entity,
    Block,
    Chest,
    Equipment,
    Fishing,
    Gift,
    VaultReward,
    Shearing,
    Archaeology,
    Generic,
    Barter,
    Command,
    Selector,
    AdvancementReward,
    AdvancementEntity,
    Custom(String),
}

impl LootTableType {
    /// Returns the Minecraft namespace string for this loot table type.
    pub fn type_str(&self) -> String {
        match self {
            LootTableType::Empty => "minecraft:empty".to_string(),
            LootTableType::Entity => "minecraft:entity".to_string(),
            LootTableType::Block => "minecraft:block".to_string(),
            LootTableType::Chest => "minecraft:chest".to_string(),
            LootTableType::Equipment => "minecraft:equipment".to_string(),
            LootTableType::Fishing => "minecraft:fishing".to_string(),
            LootTableType::Gift => "minecraft:gift".to_string(),
            LootTableType::VaultReward => "minecraft:vault_reward".to_string(),
            LootTableType::Shearing => "minecraft:shearing".to_string(),
            LootTableType::Archaeology => "minecraft:archaeology".to_string(),
            LootTableType::Generic => "minecraft:generic".to_string(),
            LootTableType::Barter => "minecraft:barter".to_string(),
            LootTableType::Command => "minecraft:command".to_string(),
            LootTableType::Selector => "minecraft:selector".to_string(),
            LootTableType::AdvancementReward => "minecraft:advancement_reward".to_string(),
            LootTableType::AdvancementEntity => "minecraft:advancement_entity".to_string(),
            LootTableType::Custom(s) => s.clone(),
        }
    }
}

// ── NumberProvider ────────────────────────────────────────────────────────────

/// Provides numeric values for loot table operations, supporting constants and dynamic calculations.
///
/// Variants include constant values, uniform random ranges, binomial distributions, and score-based values.
pub enum NumberProvider {
    Constant(f64),
    Uniform { min: f64, max: f64 },
    Binomial { n: i32, p: f64 },
    Score { target: Value, score: String, scale: Option<f64> },
}

impl From<i32> for NumberProvider {
    fn from(v: i32) -> Self {
        NumberProvider::Constant(v as f64)
    }
}

impl From<f64> for NumberProvider {
    fn from(v: f64) -> Self {
        NumberProvider::Constant(v)
    }
}

impl Serialize for NumberProvider {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            NumberProvider::Constant(v) => serializer.serialize_f64(*v),
            NumberProvider::Uniform { min, max } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "minecraft:uniform")?;
                map.serialize_entry("min", min)?;
                map.serialize_entry("max", max)?;
                map.end()
            }
            NumberProvider::Binomial { n, p } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "minecraft:binomial")?;
                map.serialize_entry("n", n)?;
                map.serialize_entry("p", p)?;
                map.end()
            }
            NumberProvider::Score { target, score, scale } => {
                let count = 3 + scale.is_some() as usize;
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("type", "minecraft:score")?;
                map.serialize_entry("target", target)?;
                map.serialize_entry("score", score)?;
                if let Some(s) = scale {
                    map.serialize_entry("scale", s)?;
                }
                map.end()
            }
        }
    }
}

// ── LootCondition ────────────────────────────────────────────────────────────

/// Conditional logic that determines whether loot entries or functions should execute.
///
/// Includes boolean composition (AllOf, AnyOf, Inverted), entity/block checks, probability,
/// and custom conditions for fine-grained control over loot generation.
pub enum LootCondition {
    AllOf { terms: Vec<LootCondition> },
    AnyOf { terms: Vec<LootCondition> },
    Inverted { term: Box<LootCondition> },
    RandomChance { chance: f64 },
    KilledByPlayer,
    EntityProperties { entity: String, predicate: Value },
    EntityScores { entity: String, scores: HashMap<String, Value> },
    MatchTool { predicate: Value },
    SurvivesExplosion,
    TableBonus { enchantment: String, chances: Vec<f64> },
    TimeCheck { value: Value, period: Option<i64> },
    WeatherCheck { raining: Option<bool>, thundering: Option<bool> },
    BlockStateProperty { block: String, properties: HashMap<String, String> },
    Reference { name: String },
    Custom { condition: String, data: Value },
}

impl Serialize for LootCondition {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LootCondition::AllOf { terms } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:all_of")?;
                map.serialize_entry("terms", terms)?;
                map.end()
            }
            LootCondition::AnyOf { terms } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:any_of")?;
                map.serialize_entry("terms", terms)?;
                map.end()
            }
            LootCondition::Inverted { term } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:inverted")?;
                map.serialize_entry("term", term)?;
                map.end()
            }
            LootCondition::RandomChance { chance } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:random_chance")?;
                map.serialize_entry("chance", chance)?;
                map.end()
            }
            LootCondition::KilledByPlayer => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("condition", "minecraft:killed_by_player")?;
                map.end()
            }
            LootCondition::EntityProperties { entity, predicate } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("condition", "minecraft:entity_properties")?;
                map.serialize_entry("entity", entity)?;
                map.serialize_entry("predicate", predicate)?;
                map.end()
            }
            LootCondition::EntityScores { entity, scores } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("condition", "minecraft:entity_scores")?;
                map.serialize_entry("entity", entity)?;
                map.serialize_entry("scores", scores)?;
                map.end()
            }
            LootCondition::MatchTool { predicate } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:match_tool")?;
                map.serialize_entry("predicate", predicate)?;
                map.end()
            }
            LootCondition::SurvivesExplosion => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("condition", "minecraft:survives_explosion")?;
                map.end()
            }
            LootCondition::TableBonus { enchantment, chances } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("condition", "minecraft:table_bonus")?;
                map.serialize_entry("enchantment", enchantment)?;
                map.serialize_entry("chances", chances)?;
                map.end()
            }
            LootCondition::TimeCheck { value, period } => {
                let count = 2 + period.is_some() as usize;
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("condition", "minecraft:time_check")?;
                map.serialize_entry("value", value)?;
                if let Some(p) = period {
                    map.serialize_entry("period", p)?;
                }
                map.end()
            }
            LootCondition::WeatherCheck { raining, thundering } => {
                let count = 1 + raining.is_some() as usize + thundering.is_some() as usize;
                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("condition", "minecraft:weather_check")?;
                if let Some(r) = raining {
                    map.serialize_entry("raining", r)?;
                }
                if let Some(t) = thundering {
                    map.serialize_entry("thundering", t)?;
                }
                map.end()
            }
            LootCondition::BlockStateProperty { block, properties } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("condition", "minecraft:block_state_property")?;
                map.serialize_entry("block", block)?;
                map.serialize_entry("properties", properties)?;
                map.end()
            }
            LootCondition::Reference { name } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:reference")?;
                map.serialize_entry("name", name)?;
                map.end()
            }
            LootCondition::Custom { condition, data } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", condition)?;
                // Merge data fields into the map
                if let Value::Object(obj) = data {
                    for (k, v) in obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

// ── LootFunction ─────────────────────────────────────────────────────────────

/// Modifies loot entries after they are selected (enchanting, naming, damage, etc.).
///
/// Functions transform items with effects like SetCount, SetName, EnchantWithLevels, or custom operations.
pub enum LootFunction {
    SetCount { count: NumberProvider, add: bool },
    SetDamage { damage: NumberProvider, add: bool },
    EnchantWithLevels { levels: NumberProvider, options: Option<String> },
    EnchantRandomly { options: Option<Vec<String>>, only_compatible: bool },
    SetName { name: Value, entity: Option<String> },
    SetLore { lore: Vec<Value>, entity: Option<String> },
    LootingEnchant { count: NumberProvider, limit: Option<i32> },
    ExplosionDecay,
    FurnaceSmelt,
    FillPlayerHead { entity: String },
    CopyComponents { source: String, include: Vec<String>, exclude: Vec<String> },
    Reference { name: String },
    Custom { function: String, data: Value },
}

impl Serialize for LootFunction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LootFunction::SetCount { count, add } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:set_count")?;
                map.serialize_entry("count", count)?;
                map.serialize_entry("add", add)?;
                map.end()
            }
            LootFunction::SetDamage { damage, add } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:set_damage")?;
                map.serialize_entry("damage", damage)?;
                map.serialize_entry("add", add)?;
                map.end()
            }
            LootFunction::EnchantWithLevels { levels, options } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:enchant_with_levels")?;
                map.serialize_entry("levels", levels)?;
                if let Some(opts) = options {
                    map.serialize_entry("options", opts)?;
                }
                map.end()
            }
            LootFunction::EnchantRandomly { options, only_compatible } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:enchant_randomly")?;
                if let Some(opts) = options {
                    map.serialize_entry("options", opts)?;
                }
                map.serialize_entry("only_compatible", only_compatible)?;
                map.end()
            }
            LootFunction::SetName { name, entity } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:set_name")?;
                map.serialize_entry("name", name)?;
                if let Some(e) = entity {
                    map.serialize_entry("entity", e)?;
                }
                map.end()
            }
            LootFunction::SetLore { lore, entity } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:set_lore")?;
                map.serialize_entry("lore", lore)?;
                if let Some(e) = entity {
                    map.serialize_entry("entity", e)?;
                }
                map.end()
            }
            LootFunction::LootingEnchant { count, limit } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:looting_enchant")?;
                map.serialize_entry("count", count)?;
                if let Some(l) = limit {
                    map.serialize_entry("limit", l)?;
                }
                map.end()
            }
            LootFunction::ExplosionDecay => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("function", "minecraft:explosion_decay")?;
                map.end()
            }
            LootFunction::FurnaceSmelt => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("function", "minecraft:furnace_smelt")?;
                map.end()
            }
            LootFunction::FillPlayerHead { entity } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("function", "minecraft:fill_player_head")?;
                map.serialize_entry("entity", entity)?;
                map.end()
            }
            LootFunction::CopyComponents { source, include, exclude } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", "minecraft:copy_components")?;
                map.serialize_entry("source", source)?;
                if !include.is_empty() {
                    map.serialize_entry("include", include)?;
                }
                if !exclude.is_empty() {
                    map.serialize_entry("exclude", exclude)?;
                }
                map.end()
            }
            LootFunction::Reference { name } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("function", "minecraft:reference")?;
                map.serialize_entry("name", name)?;
                map.end()
            }
            LootFunction::Custom { function, data } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("function", function)?;
                if let Value::Object(obj) = data {
                    for (k, v) in obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

// ── LootEntry ────────────────────────────────────────────────────────────────

/// A single entry in a loot pool, representing items, tags, nested tables, or structural groups.
///
/// Variants include direct item drops, item tag selections, nested loot table references,
/// and composition types (Group, Alternatives, Sequence) for organizing multiple entries.
pub enum LootEntry {
    Item {
        name: String,
        weight: Option<i32>,
        quality: Option<i32>,
        functions: Vec<LootFunction>,
        conditions: Vec<LootCondition>,
    },
    Tag {
        name: String,
        expand: Option<bool>,
        weight: Option<i32>,
        quality: Option<i32>,
        conditions: Vec<LootCondition>,
    },
    LootTable {
        value: String,
        weight: Option<i32>,
        quality: Option<i32>,
        conditions: Vec<LootCondition>,
    },
    Group {
        children: Vec<LootEntry>,
        conditions: Vec<LootCondition>,
    },
    Alternatives {
        children: Vec<LootEntry>,
        conditions: Vec<LootCondition>,
    },
    Sequence {
        children: Vec<LootEntry>,
        conditions: Vec<LootCondition>,
    },
    Dynamic {
        name: String,
        conditions: Vec<LootCondition>,
    },
    Empty {
        weight: Option<i32>,
        quality: Option<i32>,
        conditions: Vec<LootCondition>,
    },
}

impl LootEntry {
    /// Creates a direct item entry with the given item name.
    pub fn item(name: impl Display) -> Self {
        LootEntry::Item {
            name: name.to_string(),
            weight: None,
            quality: None,
            functions: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Creates an item tag entry that selects random items from the tag.
    pub fn tag(name: impl Display) -> Self {
        LootEntry::Tag {
            name: name.to_string(),
            expand: None,
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }

    /// Creates a nested loot table reference entry.
    pub fn loot_table(value: impl Display) -> Self {
        LootEntry::LootTable {
            value: value.to_string(),
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }

    /// Creates a group entry that processes all children in sequence.
    pub fn group(children: Vec<LootEntry>) -> Self {
        LootEntry::Group {
            children,
            conditions: Vec::new(),
        }
    }

    /// Creates an alternatives entry that selects the first child whose conditions pass.
    pub fn alternatives(children: Vec<LootEntry>) -> Self {
        LootEntry::Alternatives {
            children,
            conditions: Vec::new(),
        }
    }

    /// Creates a sequence entry that processes children in order and stops at the first success.
    pub fn sequence(children: Vec<LootEntry>) -> Self {
        LootEntry::Sequence {
            children,
            conditions: Vec::new(),
        }
    }

    /// Creates a dynamic entry that references a dynamic loot table.
    pub fn dynamic(name: impl Display) -> Self {
        LootEntry::Dynamic {
            name: name.to_string(),
            conditions: Vec::new(),
        }
    }

    /// Creates an empty entry that produces no items.
    pub fn empty() -> Self {
        LootEntry::Empty {
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }
}

impl Serialize for LootEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LootEntry::Item { name, weight, quality, functions, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:item")?;
                map.serialize_entry("name", name)?;
                if let Some(w) = weight {
                    map.serialize_entry("weight", w)?;
                }
                if let Some(q) = quality {
                    map.serialize_entry("quality", q)?;
                }
                if !functions.is_empty() {
                    map.serialize_entry("functions", functions)?;
                }
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Tag { name, expand, weight, quality, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:tag")?;
                map.serialize_entry("name", name)?;
                if let Some(e) = expand {
                    map.serialize_entry("expand", e)?;
                }
                if let Some(w) = weight {
                    map.serialize_entry("weight", w)?;
                }
                if let Some(q) = quality {
                    map.serialize_entry("quality", q)?;
                }
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::LootTable { value, weight, quality, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:loot_table")?;
                map.serialize_entry("value", value)?;
                if let Some(w) = weight {
                    map.serialize_entry("weight", w)?;
                }
                if let Some(q) = quality {
                    map.serialize_entry("quality", q)?;
                }
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Group { children, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:group")?;
                map.serialize_entry("children", children)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Alternatives { children, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:alternatives")?;
                map.serialize_entry("children", children)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Sequence { children, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:sequence")?;
                map.serialize_entry("children", children)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Dynamic { name, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:dynamic")?;
                map.serialize_entry("name", name)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Empty { weight, quality, conditions } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:empty")?;
                if let Some(w) = weight {
                    map.serialize_entry("weight", w)?;
                }
                if let Some(q) = quality {
                    map.serialize_entry("quality", q)?;
                }
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
        }
    }
}

// ── LootPool ─────────────────────────────────────────────────────────────────

/// A pool of loot entries that are randomly selected based on roll counts.
///
/// Pools define the number of rolls, entries to choose from, conditions to apply, and functions
/// to execute on selected items in a Minecraft loot table.
pub struct LootPool {
    pub rolls: NumberProvider,
    pub bonus_rolls: Option<NumberProvider>,
    pub entries: Vec<LootEntry>,
    pub conditions: Vec<LootCondition>,
    pub functions: Vec<LootFunction>,
}

impl LootPool {
    /// Creates a new loot pool with default settings (1 roll, no bonus rolls).
    pub fn new() -> Self {
        Self {
            rolls: NumberProvider::Constant(1.0),
            bonus_rolls: None,
            entries: Vec::new(),
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// Sets the number of times entries are selected from this pool.
    pub fn rolls(mut self, n: impl Into<NumberProvider>) -> Self {
        self.rolls = n.into();
        self
    }

    /// Sets additional bonus rolls based on conditions like looting enchantment levels.
    pub fn bonus_rolls(mut self, n: impl Into<NumberProvider>) -> Self {
        self.bonus_rolls = Some(n.into());
        self
    }

    /// Adds an entry to this pool's selection options.
    pub fn entry(mut self, entry: LootEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Adds a condition that must be met for this pool to generate loot.
    pub fn condition(mut self, condition: LootCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Adds a function to process entries selected from this pool.
    pub fn function(mut self, function: LootFunction) -> Self {
        self.functions.push(function);
        self
    }
}

impl Default for LootPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for LootPool {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("rolls", &self.rolls)?;
        if let Some(ref br) = self.bonus_rolls {
            map.serialize_entry("bonus_rolls", br)?;
        }
        map.serialize_entry("entries", &self.entries)?;
        if !self.conditions.is_empty() {
            map.serialize_entry("conditions", &self.conditions)?;
        }
        if !self.functions.is_empty() {
            map.serialize_entry("functions", &self.functions)?;
        }
        map.end()
    }
}

// ── LootTable ────────────────────────────────────────────────────────────────

/// Represents a complete Minecraft loot table with pools, functions, and conditions.
///
/// A loot table is a datapack component that defines what items are dropped in specific contexts
/// (blocks, entities, chests, etc.). It consists of pools, global functions, and global conditions.
pub struct LootTable {
    pub location: ResourceLocation,
    pub loot_type: Option<LootTableType>,
    pub random_sequence: Option<String>,
    pub pools: Vec<LootPool>,
    pub functions: Vec<LootFunction>,
    pub conditions: Vec<LootCondition>,
}

impl LootTable {
    /// Creates a new loot table at the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            loot_type: None,
            random_sequence: None,
            pools: Vec::new(),
            functions: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Sets the type of this loot table (block, entity, chest, etc.).
    pub fn loot_type(mut self, loot_type: LootTableType) -> Self {
        self.loot_type = Some(loot_type);
        self
    }

    /// Sets the random sequence seed for deterministic loot generation.
    pub fn random_sequence(mut self, seq: impl Into<String>) -> Self {
        self.random_sequence = Some(seq.into());
        self
    }

    /// Adds a loot pool to this table.
    pub fn pool(mut self, pool: LootPool) -> Self {
        self.pools.push(pool);
        self
    }

    /// Adds a function to apply to all loot generated by this table.
    pub fn function(mut self, function: LootFunction) -> Self {
        self.functions.push(function);
        self
    }

    /// Adds a condition that must be met for this table to generate loot.
    pub fn condition(mut self, condition: LootCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    // ── Shorthand constructors ────────────────────────────────────────────────

    /// A block loot table that drops exactly `count` of `item`.
    ///
    /// Includes `minecraft:survives_explosion` so the drop respects TNT.
    ///
    /// # Example
    /// ```rust,ignore
    /// LootTable::simple_block_drop(loc, "minecraft:oak_log", 1)
    /// ```
    pub fn simple_block_drop(
        location: ResourceLocation,
        item: impl Display,
        count: i32,
    ) -> Self {
        let entry = LootEntry::item(item.to_string());
        let entry = if let LootEntry::Item { name, weight, quality, mut functions, conditions } = entry {
            functions.push(LootFunction::SetCount { count: NumberProvider::Constant(count as f64), add: false });
            LootEntry::Item { name, weight, quality, functions, conditions }
        } else { unreachable!() };

        Self::new(location)
            .loot_type(LootTableType::Block)
            .pool(
                LootPool::new()
                    .rolls(1)
                    .entry(entry)
                    .condition(LootCondition::SurvivesExplosion),
            )
    }

    /// A block loot table with fortune-sensitive drop counts.
    ///
    /// `chances` is a slice of probabilities indexed by fortune level
    /// (0 = no fortune, 1 = Fortune I, 2 = Fortune II, ...).
    ///
    /// # Example
    /// ```rust,ignore
    /// // Coal ore: fortune[0]=1, fortune[1]=1, fortune[2]=2, fortune[3]=3
    /// LootTable::fortune_drop(loc, "minecraft:coal", "minecraft:fortune", &[1.0, 1.0, 2.0, 3.0])
    /// ```
    pub fn fortune_drop(
        location: ResourceLocation,
        item: impl Display,
        enchantment: impl Display,
        chances: &[f64],
    ) -> Self {
        let entry = LootEntry::item(item.to_string());
        let entry = if let LootEntry::Item { name, weight, quality, mut functions, mut conditions } = entry {
            functions.push(LootFunction::SetCount { count: NumberProvider::Constant(1.0), add: false });
            conditions.push(LootCondition::TableBonus {
                enchantment: enchantment.to_string(),
                chances: chances.to_vec(),
            });
            LootEntry::Item { name, weight, quality, functions, conditions }
        } else { unreachable!() };

        Self::new(location)
            .loot_type(LootTableType::Block)
            .pool(
                LootPool::new()
                    .rolls(1)
                    .entry(entry)
                    .condition(LootCondition::SurvivesExplosion),
            )
    }

    /// An entity loot table that drops `item` only when killed by a player,
    /// with an optional looting-enchant bonus.
    ///
    /// # Example
    /// ```rust,ignore
    /// LootTable::entity_drop(loc, "minecraft:leather", 0..=2, Some(1))
    /// ```
    pub fn entity_drop(
        location: ResourceLocation,
        item: impl Display,
        min_count: i32,
        max_count: i32,
        looting_bonus: Option<i32>,
    ) -> Self {
        let count_provider = if min_count == max_count {
            NumberProvider::Constant(min_count as f64)
        } else {
            NumberProvider::Uniform { min: min_count as f64, max: max_count as f64 }
        };

        let entry = LootEntry::item(item.to_string());
        let entry = if let LootEntry::Item { name, weight, quality, mut functions, conditions } = entry {
            functions.push(LootFunction::SetCount { count: count_provider, add: false });
            if let Some(bonus) = looting_bonus {
                functions.push(LootFunction::LootingEnchant {
                    count: NumberProvider::Uniform { min: 0.0, max: bonus as f64 },
                    limit: None,
                });
            }
            LootEntry::Item { name, weight, quality, functions, conditions }
        } else { unreachable!() };

        Self::new(location)
            .loot_type(LootTableType::Entity)
            .pool(
                LootPool::new()
                    .rolls(1)
                    .entry(entry)
                    .condition(LootCondition::KilledByPlayer),
            )
    }

    /// Creates a chest loot table with multiple weighted item entries.
    ///
    /// `items` is an iterator of `(item_id, weight, min_count, max_count)`.
    ///
    /// # Example
    /// ```rust,ignore
    /// LootTable::chest_loot(loc, vec![
    ///     ("minecraft:diamond", 5, 1, 3),
    ///     ("minecraft:gold_ingot", 20, 2, 5),
    /// ])
    /// ```
    pub fn chest_loot<S: Display>(
        location: ResourceLocation,
        items: impl IntoIterator<Item = (S, i32, i32, i32)>,
    ) -> Self {
        let mut pool = LootPool::new().rolls(1);
        for (item, weight, min_count, max_count) in items {
            let count_provider = if min_count == max_count {
                NumberProvider::Constant(min_count as f64)
            } else {
                NumberProvider::Uniform { min: min_count as f64, max: max_count as f64 }
            };

            let entry = LootEntry::item(item.to_string());
            let entry = if let LootEntry::Item { name, quality, mut functions, conditions, .. } = entry {
                functions.push(LootFunction::SetCount { count: count_provider, add: false });
                LootEntry::Item { name, weight: Some(weight), quality, functions, conditions }
            } else { unreachable!() };

            pool = pool.entry(entry);
        }

        Self::new(location)
            .loot_type(LootTableType::Chest)
            .pool(pool)
    }
}

impl DatapackComponent for LootTable {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        if let Some(ref lt) = self.loot_type {
            map.insert("type".to_string(), Value::String(lt.type_str()));
        }
        if let Some(ref rs) = self.random_sequence {
            map.insert("random_sequence".to_string(), Value::String(rs.clone()));
        }
        if !self.pools.is_empty() {
            map.insert("pools".to_string(), serde_json::to_value(&self.pools).unwrap());
        }
        if !self.functions.is_empty() {
            map.insert("functions".to_string(), serde_json::to_value(&self.functions).unwrap());
        }
        if !self.conditions.is_empty() {
            map.insert("conditions".to_string(), serde_json::to_value(&self.conditions).unwrap());
        }

        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str { "loot_table" }
}
