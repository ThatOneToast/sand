use std::collections::HashMap;
use std::fmt::Display;

use sand_commands::TextComponent;
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::raw::RawJson;
use crate::registry::{EnchantmentId, ItemId, LootTableId, TagId};
use crate::resource_location::ResourceLocation;

mod validation;

// ── LootTableType ────────────────────────────────────────────────────────────

/// Represents the type of a Minecraft loot table (block, entity, chest, etc.).
///
/// Each variant corresponds to a specific loot table type defined in the Minecraft datapack spec,
/// with a `Custom` variant for extensibility.
pub enum LootTableType {
    /// Empty loot table (returns nothing).
    Empty,
    /// Entity drops (e.g. from `minecraft:bat`).
    Entity,
    /// Block drops (e.g. from `minecraft:stone`).
    Block,
    /// Chest loot.
    Chest,
    /// Equipment drops.
    Equipment,
    /// Fishing rewards.
    Fishing,
    /// Gift drops.
    Gift,
    /// Vault rewards (1.21+).
    VaultReward,
    /// Shearing rewards (e.g. wool from sheep).
    Shearing,
    /// Archaeology loot.
    Archaeology,
    /// Generic/untyped loot.
    Generic,
    /// Bartering with piglins.
    Barter,
    /// Command rewards.
    Command,
    /// Selector-based loot.
    Selector,
    /// Advancement reward loot.
    AdvancementReward,
    /// Advancement entity rewards.
    AdvancementEntity,
    /// Custom or user-defined loot table type.
    Custom(String),
}

impl LootTableType {
    /// Get the Minecraft namespace string for this loot table type.
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
#[derive(Debug, Clone, PartialEq)]
pub enum NumberProvider {
    /// A constant numeric value.
    Constant(f64),
    /// Uniform random distribution between `min` and `max`.
    Uniform {
        /// Minimum value (inclusive).
        min: f64,
        /// Maximum value (inclusive).
        max: f64,
    },
    /// Binomial distribution with `n` trials and probability `p`.
    Binomial {
        /// Number of trials.
        n: i32,
        /// Probability of success per trial.
        p: f64,
    },
    /// Dynamic value from a scoreboard score.
    Score {
        /// The target selector or name.
        target: Value,
        /// The objective name.
        score: String,
        /// Optional scale factor to apply to the score.
        scale: Option<f64>,
    },
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
            NumberProvider::Score {
                target,
                score,
                scale,
            } => {
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
    /// All conditions must be true (AND).
    AllOf {
        /// List of conditions that must all be true.
        terms: Vec<LootCondition>,
    },
    /// At least one condition must be true (OR).
    AnyOf {
        /// List of conditions, at least one must be true.
        terms: Vec<LootCondition>,
    },
    /// Inverted logic (NOT).
    Inverted {
        /// The condition to invert.
        term: Box<LootCondition>,
    },
    /// Random probability check (0.0 to 1.0).
    RandomChance {
        /// Probability of success.
        chance: f64,
    },
    /// True if the entity was killed by a player.
    KilledByPlayer,
    /// Check entity properties/predicates.
    EntityProperties {
        /// The entity selector.
        entity: String,
        /// The predicate to check.
        predicate: Value,
    },
    /// Check entity scoreboard scores.
    EntityScores {
        /// The entity selector.
        entity: String,
        /// Score names and their required values.
        scores: HashMap<String, Value>,
    },
    /// Match the tool used to mine/break the block.
    MatchTool {
        /// Item predicate for the tool.
        predicate: Value,
    },
    /// Block survives explosion (doesn't drop).
    SurvivesExplosion,
    /// Enchantment bonus table (e.g. for fortune).
    TableBonus {
        /// Enchantment ID to check.
        enchantment: String,
        /// Chances per enchantment level.
        chances: Vec<f64>,
    },
    /// Check the current game time.
    TimeCheck {
        /// The time value or range to check.
        value: Value,
        /// Optional period for periodic checking.
        period: Option<i64>,
    },
    /// Check weather conditions.
    WeatherCheck {
        /// Is it raining?
        raining: Option<bool>,
        /// Is it thundering?
        thundering: Option<bool>,
    },
    /// Check block state properties.
    BlockStateProperty {
        /// Block ID.
        block: String,
        /// Properties to match.
        properties: HashMap<String, String>,
    },
    /// Reference to a named predicate file.
    Reference {
        /// Predicate file name/ID.
        name: String,
    },
    /// Custom condition type — explicit raw escape hatch for modded conditions.
    ///
    /// Use [`RawJson`] for `data`.  The named type signals
    /// intentional opt-out of the typed condition API.
    Custom {
        /// Condition type identifier (e.g. `"mymod:custom_condition"`).
        condition: String,
        /// Additional condition data as raw JSON.
        data: RawJson,
    },
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
            LootCondition::TableBonus {
                enchantment,
                chances,
            } => {
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
            LootCondition::WeatherCheck {
                raining,
                thundering,
            } => {
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
                if let Value::Object(obj) = data.as_value() {
                    for (k, v) in obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

// ── EnchantmentSelector ─────────────────────────────────────────────────────────

/// A single enchantment or enchantment-tag reference used by loot functions
/// that select enchantments (`enchant_with_levels`, `enchant_randomly`).
///
/// Serializes as a plain `namespace:path` for [`EnchantmentSelector::Id`] and as
/// a `#namespace:path` tag reference for [`EnchantmentSelector::Tag`].
pub enum EnchantmentSelector {
    /// A single enchantment ID.
    Id(EnchantmentId),
    /// An enchantment tag reference.
    Tag(TagId<EnchantmentId>),
}

impl From<EnchantmentId> for EnchantmentSelector {
    fn from(id: EnchantmentId) -> Self {
        EnchantmentSelector::Id(id)
    }
}

impl From<TagId<EnchantmentId>> for EnchantmentSelector {
    fn from(tag: TagId<EnchantmentId>) -> Self {
        EnchantmentSelector::Tag(tag)
    }
}

impl Display for EnchantmentSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnchantmentSelector::Id(id) => write!(f, "{id}"),
            EnchantmentSelector::Tag(tag) => write!(f, "{}", tag.to_tag_string()),
        }
    }
}

impl Serialize for EnchantmentSelector {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

// ── LootText ─────────────────────────────────────────────────────────────────

/// Text payload used by loot functions that set item names/lore
/// (`set_name`, `set_lore`).
///
/// Prefer [`LootText::Typed`] (via `From<TextComponent>`/`.into()`) so text is
/// validated the same way command text is. [`LootText::Raw`] is an explicit
/// escape hatch for JSON text shapes [`TextComponent`] cannot express.
pub enum LootText {
    /// A typed, validated Minecraft text component.
    Typed(Box<TextComponent>),
    /// Explicit raw JSON text escape hatch.
    Raw(RawJson),
}

impl From<TextComponent> for LootText {
    fn from(text: TextComponent) -> Self {
        LootText::Typed(Box::new(text))
    }
}

impl From<RawJson> for LootText {
    fn from(text: RawJson) -> Self {
        LootText::Raw(text)
    }
}

impl Serialize for LootText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LootText::Typed(text) => {
                let value = text
                    .try_to_json_value()
                    .map_err(serde::ser::Error::custom)?;
                value.serialize(serializer)
            }
            LootText::Raw(text) => text.serialize(serializer),
        }
    }
}

// ── LootFunction ─────────────────────────────────────────────────────────────

/// Modifies loot entries after they are selected (enchanting, naming, damage, etc.).
///
/// Functions transform items with effects like SetCount, SetName, EnchantWithLevels, or custom operations.
pub enum LootFunction {
    SetCount {
        count: NumberProvider,
        add: bool,
    },
    SetDamage {
        damage: NumberProvider,
        add: bool,
    },
    EnchantWithLevels {
        levels: NumberProvider,
        options: Option<EnchantmentSelector>,
    },
    EnchantRandomly {
        options: Option<Vec<EnchantmentSelector>>,
        only_compatible: bool,
    },
    SetName {
        name: LootText,
        entity: Option<String>,
    },
    SetLore {
        lore: Vec<LootText>,
        entity: Option<String>,
    },
    LootingEnchant {
        count: NumberProvider,
        limit: Option<i32>,
    },
    ExplosionDecay,
    FurnaceSmelt,
    FillPlayerHead {
        entity: String,
    },
    CopyComponents {
        source: String,
        include: Vec<String>,
        exclude: Vec<String>,
    },
    Reference {
        name: String,
    },
    /// Custom function — explicit raw escape hatch for modded loot functions.
    ///
    /// Use [`RawJson`] for `data`.
    Custom {
        /// Function type identifier (e.g. `"mymod:custom_function"`).
        function: String,
        /// Additional function data as raw JSON.
        data: RawJson,
    },
}

impl LootFunction {
    /// Creates a `set_name` function from a typed or raw [`LootText`] value.
    ///
    /// Accepts `TextComponent` directly via `.into()`:
    ///
    /// ```
    /// use sand_commands::Text;
    /// use sand_components::LootFunction;
    ///
    /// let function = LootFunction::set_name(Text::new("Legendary Sword").gold());
    /// ```
    pub fn set_name(name: impl Into<LootText>) -> Self {
        LootFunction::SetName {
            name: name.into(),
            entity: None,
        }
    }

    /// Creates a `set_lore` function from typed or raw [`LootText`] lines.
    pub fn set_lore(lore: impl IntoIterator<Item = impl Into<LootText>>) -> Self {
        LootFunction::SetLore {
            lore: lore.into_iter().map(Into::into).collect(),
            entity: None,
        }
    }
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
            LootFunction::EnchantRandomly {
                options,
                only_compatible,
            } => {
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
            LootFunction::CopyComponents {
                source,
                include,
                exclude,
            } => {
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
                if let Value::Object(obj) = data.as_value() {
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
    /// Creates a direct item entry from a typed [`ItemId`] (preferred path).
    ///
    /// ```
    /// use sand_components::{ItemId, LootEntry};
    ///
    /// let entry = LootEntry::item(ItemId::minecraft("diamond").unwrap());
    /// ```
    pub fn item(id: ItemId) -> Self {
        LootEntry::Item {
            name: id.to_string(),
            weight: None,
            quality: None,
            functions: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Creates a direct item entry through the explicit raw compatibility path.
    ///
    /// Use this for modded/unsupported item IDs that cannot be expressed as
    /// [`ItemId`]. Malformed values are still caught by export validation.
    pub fn item_raw(name: impl Display) -> Self {
        LootEntry::Item {
            name: name.to_string(),
            weight: None,
            quality: None,
            functions: Vec::new(),
            conditions: Vec::new(),
        }
    }

    /// Creates an item tag entry from a typed [`TagId<ItemId>`] (preferred path).
    ///
    /// The `TagId` is scoped to `ItemId`, so a block tag cannot be used by
    /// mistake:
    ///
    /// ```compile_fail
    /// use sand_components::{BlockId, LootEntry, TagId};
    ///
    /// let block_tag: TagId<BlockId> = TagId::minecraft("logs").unwrap();
    /// let _entry = LootEntry::tag(block_tag);
    /// ```
    pub fn tag(tag: TagId<ItemId>) -> Self {
        LootEntry::Tag {
            name: tag.to_string(),
            expand: None,
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }

    /// Creates an item tag entry through the explicit raw compatibility path.
    pub fn tag_raw(name: impl Display) -> Self {
        LootEntry::Tag {
            name: name.to_string(),
            expand: None,
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }

    /// Creates a nested loot table reference entry from a typed [`LootTableId`]
    /// (preferred path).
    pub fn loot_table(id: LootTableId) -> Self {
        LootEntry::LootTable {
            value: id.to_string(),
            weight: None,
            quality: None,
            conditions: Vec::new(),
        }
    }

    /// Creates a nested loot table reference entry through the explicit raw
    /// compatibility path.
    pub fn loot_table_raw(value: impl Display) -> Self {
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
            LootEntry::Item {
                name,
                weight,
                quality,
                functions,
                conditions,
            } => {
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
            LootEntry::Tag {
                name,
                expand,
                weight,
                quality,
                conditions,
            } => {
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
            LootEntry::LootTable {
                value,
                weight,
                quality,
                conditions,
            } => {
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
            LootEntry::Group {
                children,
                conditions,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:group")?;
                map.serialize_entry("children", children)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Alternatives {
                children,
                conditions,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "minecraft:alternatives")?;
                map.serialize_entry("children", children)?;
                if !conditions.is_empty() {
                    map.serialize_entry("conditions", conditions)?;
                }
                map.end()
            }
            LootEntry::Sequence {
                children,
                conditions,
            } => {
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
            LootEntry::Empty {
                weight,
                quality,
                conditions,
            } => {
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
    rolls: NumberProvider,
    bonus_rolls: Option<NumberProvider>,
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
    loot_type: Option<LootTableType>,
    random_sequence: Option<String>,
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

    /// Sets the random sequence seed for deterministic loot generation using a
    /// typed [`ResourceLocation`] (preferred path).
    pub fn random_sequence(mut self, seq: ResourceLocation) -> Self {
        self.random_sequence = Some(seq.to_string());
        self
    }

    /// Sets the random sequence seed through the explicit raw compatibility path.
    pub fn random_sequence_raw(mut self, seq: impl Into<String>) -> Self {
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
    pub fn simple_block_drop(location: ResourceLocation, item: impl Display, count: i32) -> Self {
        let entry = LootEntry::item_raw(item.to_string());
        let entry = if let LootEntry::Item {
            name,
            weight,
            quality,
            mut functions,
            conditions,
        } = entry
        {
            functions.push(LootFunction::SetCount {
                count: NumberProvider::Constant(count as f64),
                add: false,
            });
            LootEntry::Item {
                name,
                weight,
                quality,
                functions,
                conditions,
            }
        } else {
            unreachable!()
        };

        Self::new(location).loot_type(LootTableType::Block).pool(
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
    /// // Probability of the entry passing at each Fortune level.
    /// LootTable::fortune_drop(loc, "minecraft:coal", "minecraft:fortune", &[0.25, 0.5, 0.75, 1.0])
    /// ```
    pub fn fortune_drop(
        location: ResourceLocation,
        item: impl Display,
        enchantment: impl Display,
        chances: &[f64],
    ) -> Self {
        let entry = LootEntry::item_raw(item.to_string());
        let entry = if let LootEntry::Item {
            name,
            weight,
            quality,
            mut functions,
            mut conditions,
        } = entry
        {
            functions.push(LootFunction::SetCount {
                count: NumberProvider::Constant(1.0),
                add: false,
            });
            conditions.push(LootCondition::TableBonus {
                enchantment: enchantment.to_string(),
                chances: chances.to_vec(),
            });
            LootEntry::Item {
                name,
                weight,
                quality,
                functions,
                conditions,
            }
        } else {
            unreachable!()
        };

        Self::new(location).loot_type(LootTableType::Block).pool(
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
            NumberProvider::Uniform {
                min: min_count as f64,
                max: max_count as f64,
            }
        };

        let entry = LootEntry::item_raw(item.to_string());
        let entry = if let LootEntry::Item {
            name,
            weight,
            quality,
            mut functions,
            conditions,
        } = entry
        {
            functions.push(LootFunction::SetCount {
                count: count_provider,
                add: false,
            });
            if let Some(bonus) = looting_bonus {
                functions.push(LootFunction::LootingEnchant {
                    count: NumberProvider::Uniform {
                        min: 0.0,
                        max: bonus as f64,
                    },
                    limit: None,
                });
            }
            LootEntry::Item {
                name,
                weight,
                quality,
                functions,
                conditions,
            }
        } else {
            unreachable!()
        };

        Self::new(location).loot_type(LootTableType::Entity).pool(
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
                NumberProvider::Uniform {
                    min: min_count as f64,
                    max: max_count as f64,
                }
            };

            let entry = LootEntry::item_raw(item.to_string());
            let entry = if let LootEntry::Item {
                name,
                quality,
                mut functions,
                conditions,
                ..
            } = entry
            {
                functions.push(LootFunction::SetCount {
                    count: count_provider,
                    add: false,
                });
                LootEntry::Item {
                    name,
                    weight: Some(weight),
                    quality,
                    functions,
                    conditions,
                }
            } else {
                unreachable!()
            };

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

    fn validate(&self) -> crate::error::Result<()> {
        self.validate_table()
            .map_err(|failure| crate::error::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "loot_table".to_string(),
                field: failure.path,
                message: failure.message,
            })
    }

    fn to_json(&self) -> Value {
        self.try_to_json()
            .unwrap_or_else(|error| panic!("loot table serialization failed: {error}"))
    }

    fn try_content(&self) -> crate::error::Result<ComponentContent> {
        self.validate()?;
        self.try_to_json()
            .map(ComponentContent::Json)
            .map_err(crate::error::SandError::Serialization)
    }

    fn component_dir(&self) -> &'static str {
        "loot_table"
    }
}

impl LootTable {
    fn try_to_json(&self) -> Result<Value, serde_json::Error> {
        let mut map = serde_json::Map::new();

        if let Some(ref lt) = self.loot_type {
            map.insert("type".to_string(), Value::String(lt.type_str()));
        }
        if let Some(ref rs) = self.random_sequence {
            map.insert("random_sequence".to_string(), Value::String(rs.clone()));
        }
        if !self.pools.is_empty() {
            map.insert("pools".to_string(), serde_json::to_value(&self.pools)?);
        }
        if !self.functions.is_empty() {
            map.insert(
                "functions".to_string(),
                serde_json::to_value(&self.functions)?,
            );
        }
        if !self.conditions.is_empty() {
            map.insert(
                "conditions".to_string(),
                serde_json::to_value(&self.conditions)?,
            );
        }

        Ok(Value::Object(map))
    }
}

#[cfg(test)]
mod validation_tests {
    use super::{
        LootCondition, LootEntry, LootFunction, LootPool, LootTable, LootTableType, LootText,
        NumberProvider,
    };
    use crate::component::DatapackComponent;
    use crate::raw::RawJson;
    use crate::resource_location::ResourceLocation;

    #[test]
    fn probability_bounds_are_enforced_with_paths() {
        for value in [0.0, 0.5, 1.0] {
            assert!(
                LootCondition::RandomChance { chance: value }
                    .validate_at("conditions[1]")
                    .is_ok()
            );
        }
        for value in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = LootCondition::RandomChance { chance: value }
                .validate_at("pools[0].conditions[1]")
                .unwrap_err();
            assert!(err.path.contains("pools[0].conditions[1].chance"));
        }
    }

    #[test]
    fn recursive_entry_failure_retains_owner_and_indices() {
        let invalid = LootEntry::Item {
            name: "minecraft:diamond".to_string(),
            weight: None,
            quality: None,
            functions: Vec::new(),
            conditions: vec![LootCondition::RandomChance { chance: -0.1 }],
        };
        let table = LootTable::new("test:recursive".parse().unwrap())
            .pool(LootPool::new().entry(LootEntry::group(vec![invalid])));
        let error = table.try_content().unwrap_err().to_string();
        assert!(error.contains("test:recursive"));
        assert!(error.contains("pools[0].entries[0].children[0].conditions[0].chance"));
        assert!(error.contains("probability"));
    }

    #[test]
    fn valid_recursive_loot_output_is_unchanged() {
        let table = LootTable::new("test:valid_recursive".parse().unwrap()).pool(
            LootPool::new().entry(LootEntry::Item {
                name: "minecraft:diamond".to_string(),
                weight: None,
                quality: None,
                functions: Vec::new(),
                conditions: vec![LootCondition::RandomChance { chance: 0.5 }],
            }),
        );
        assert_eq!(table.try_content().unwrap(), table.content());
    }

    #[test]
    fn nested_function_failure_retains_owner_and_full_path() {
        let invalid = LootEntry::Item {
            name: "minecraft:diamond".to_string(),
            weight: None,
            quality: None,
            functions: vec![LootFunction::SetCount {
                count: NumberProvider::Constant(f64::NAN),
                add: false,
            }],
            conditions: Vec::new(),
        };
        let table = LootTable::new("test:function_path".parse().unwrap())
            .pool(LootPool::new().entry(invalid));
        let error = table.try_content().unwrap_err().to_string();
        assert!(error.contains("test:function_path"));
        assert!(error.contains("pools[0].entries[0].functions[0].count"));
        assert!(error.contains("finite"));
    }

    #[test]
    fn explicit_empty_and_representative_helpers_preserve_valid_json() {
        let empty = LootTable::new("test:empty".parse().unwrap()).loot_type(LootTableType::Empty);
        assert_eq!(empty.try_content().unwrap(), empty.content());

        let examples = [
            LootTable::simple_block_drop("test:block".parse().unwrap(), "minecraft:stone", 1),
            LootTable::entity_drop(
                "test:entity".parse().unwrap(),
                "minecraft:leather",
                1,
                2,
                Some(1),
            ),
            LootTable::chest_loot(
                "test:chest".parse().unwrap(),
                [("minecraft:diamond", 2, 1, 3)],
            ),
            LootTable::fortune_drop(
                "test:fortune".parse().unwrap(),
                "minecraft:coal",
                "minecraft:fortune",
                &[0.5, 0.75, 1.0],
            ),
        ];
        for table in examples {
            assert_eq!(table.try_content().unwrap(), table.content());
        }
    }

    #[test]
    fn helper_invalid_counts_and_ranges_fail_at_owner_boundary() {
        let zero = LootTable::simple_block_drop("test:zero".parse().unwrap(), "minecraft:stone", 0);
        assert!(
            zero.try_content()
                .unwrap_err()
                .to_string()
                .contains("greater than zero")
        );

        let inverted = LootTable::entity_drop(
            "test:inverted".parse().unwrap(),
            "minecraft:leather",
            3,
            1,
            None,
        );
        assert!(
            inverted
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("minimum")
        );

        let bad_weight = LootTable::chest_loot(
            "test:weight".parse().unwrap(),
            [("minecraft:diamond", 0, 1, 1)],
        );
        assert!(
            bad_weight
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("weight")
        );
    }

    // ── Golden tests: typed constructors match the old raw string path ───────

    #[test]
    fn typed_item_entry_matches_raw_item_entry_json() {
        let typed = LootEntry::item(crate::registry::ItemId::minecraft("diamond").unwrap());
        let raw = LootEntry::item_raw("minecraft:diamond");
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::to_value(&raw).unwrap()
        );
    }

    #[test]
    fn typed_tag_entry_matches_raw_tag_entry_json() {
        let typed = LootEntry::tag(crate::registry::TagId::minecraft("logs").unwrap());
        let raw = LootEntry::tag_raw("minecraft:logs");
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::to_value(&raw).unwrap()
        );
    }

    #[test]
    fn typed_loot_table_entry_matches_raw_loot_table_entry_json() {
        let typed = LootEntry::loot_table(
            crate::registry::LootTableId::minecraft("chests/simple_dungeon").unwrap(),
        );
        let raw = LootEntry::loot_table_raw("minecraft:chests/simple_dungeon");
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::to_value(&raw).unwrap()
        );
    }

    #[test]
    fn typed_random_sequence_matches_raw_random_sequence_json() {
        let typed = LootTable::new("test:typed_seq".parse().unwrap())
            .loot_type(LootTableType::Empty)
            .random_sequence(ResourceLocation::new("test", "sequence").unwrap());
        let raw = LootTable::new("test:typed_seq".parse().unwrap())
            .loot_type(LootTableType::Empty)
            .random_sequence_raw("test:sequence");
        assert_eq!(typed.to_json(), raw.to_json());
    }

    #[test]
    fn typed_enchant_with_levels_matches_raw_string_json() {
        let typed = LootFunction::EnchantWithLevels {
            levels: NumberProvider::Constant(10.0),
            options: Some(super::EnchantmentSelector::Id(
                crate::registry::EnchantmentId::minecraft("sharpness").unwrap(),
            )),
        };
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::json!({
                "function": "minecraft:enchant_with_levels",
                "levels": 10.0,
                "options": "minecraft:sharpness",
            })
        );
    }

    #[test]
    fn typed_enchant_randomly_tag_serializes_hash_prefixed_string() {
        let typed = LootFunction::EnchantRandomly {
            options: Some(vec![super::EnchantmentSelector::Tag(
                crate::registry::TagId::minecraft("in_enchanting_table").unwrap(),
            )]),
            only_compatible: true,
        };
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::json!({
                "function": "minecraft:enchant_randomly",
                "options": ["#minecraft:in_enchanting_table"],
                "only_compatible": true,
            })
        );
    }

    #[test]
    fn typed_set_name_matches_raw_json_text_for_plain_string() {
        use sand_commands::Text;

        let typed = LootFunction::set_name(Text::new("Legendary Sword"));
        let raw = LootFunction::SetName {
            name: LootText::Raw(RawJson::new(serde_json::json!({"text": "Legendary Sword"}))),
            entity: None,
        };
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::to_value(&raw).unwrap()
        );
    }

    #[test]
    fn typed_set_lore_matches_raw_json_text_for_plain_strings() {
        use sand_commands::Text;

        let typed = LootFunction::set_lore(vec![Text::new("Line one"), Text::new("Line two")]);
        let raw = LootFunction::SetLore {
            lore: vec![
                LootText::Raw(RawJson::new(serde_json::json!({"text": "Line one"}))),
                LootText::Raw(RawJson::new(serde_json::json!({"text": "Line two"}))),
            ],
            entity: None,
        };
        assert_eq!(
            serde_json::to_value(&typed).unwrap(),
            serde_json::to_value(&raw).unwrap()
        );
    }

    #[test]
    fn raw_escape_hatches_still_fail_export_validation_on_bad_ids() {
        let bad_item = LootTable::new("test:bad_item".parse().unwrap())
            .pool(LootPool::new().entry(LootEntry::item_raw("not a valid id")));
        assert!(bad_item.try_content().is_err());

        let bad_tag = LootTable::new("test:bad_tag".parse().unwrap())
            .pool(LootPool::new().entry(LootEntry::tag_raw("not a valid id")));
        assert!(bad_tag.try_content().is_err());

        let bad_loot_table = LootTable::new("test:bad_loot_table".parse().unwrap())
            .pool(LootPool::new().entry(LootEntry::loot_table_raw("not a valid id")));
        assert!(bad_loot_table.try_content().is_err());

        let bad_sequence = LootTable::new("test:bad_sequence".parse().unwrap())
            .loot_type(LootTableType::Empty)
            .random_sequence_raw("not a valid id");
        assert!(bad_sequence.try_content().is_err());
    }
}
