//! Typed builder for Minecraft 1.21+ custom items.
//!
//! `CustomItem` wraps a base item type with any combination of the 1.21 item
//! component system. The resulting value formats as an item-component string
//! (e.g. `minecraft:diamond_sword[custom_name={text:"..."},enchantments={...}]`) that
//! can be passed directly to [`cmd::give`](crate::cmd).
//!
//! # Identifying items
//!
//! Each custom item should set a unique [`custom_data`](CustomItem::custom_data)
//! key (e.g. `"inferno_blade"`). This emits `custom_data={inferno_blade:1b}` in
//! the item-component string and lets you match the item reliably in advancements,
//! predicates, and loot conditions via `minecraft:custom_data`.
//!
//! # Example
//! ```rust,ignore
//! use sand_components::{CustomItem, ItemRarity, AttributeType, AttributeOperation, EquipmentSlotGroup};
//! use sand_commands::{self, Selector, TextComponent, ChatColor};
//!
//! fn inferno_blade() -> CustomItem {
//!     CustomItem::new("minecraft:diamond_sword")
//!         .custom_data("inferno_blade")
//!         .custom_name(TextComponent::literal("Inferno Blade").color(ChatColor::Red))
//!         .lore_line(TextComponent::literal("A weapon of pure flame").color(ChatColor::DarkRed))
//!         .enchantment("minecraft:fire_aspect", 2)
//!         .attribute(AttributeType::AttackDamage, 10.0,
//!                    AttributeOperation::AddValue, EquipmentSlotGroup::Mainhand)
//!         .custom_model_data(1001)
//!         .max_stack_size(1)
//!         .rarity(ItemRarity::Rare)
//! }
//!
//! #[function]
//! fn give_inferno() {
//!     sand_commands::give(Selector::all_players(), inferno_blade());
//! }
//! ```

use std::fmt;

use serde_json::{Map, Value};

use crate::advancement::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion};
use crate::effect::{PotionContents, SuspiciousStewEffect};
use crate::error::{Result as SandResult, SandError};
use crate::predicates::ItemPredicate as TypedItemPredicate;
use crate::raw::{RawComponent, RawSnbt};
use crate::resource_location::ResourceLocation;
use crate::{BlockId, EnchantmentId, EntityTypeId, TagId};
use sand_commands::TextComponent;

pub mod definition;
pub mod matcher;
pub mod predicates;
pub mod stack;

// ── ItemRarity ────────────────────────────────────────────────────────────────

/// Item rarity level — affects the default name color in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    /// White text (default).
    Common,
    /// Yellow text.
    Uncommon,
    /// Cyan text.
    Rare,
    /// Pink/magenta text.
    Epic,
}

/// Alias for the public item component rarity model.
pub type Rarity = ItemRarity;

impl ItemRarity {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemRarity::Common => "common",
            ItemRarity::Uncommon => "uncommon",
            ItemRarity::Rare => "rare",
            ItemRarity::Epic => "epic",
        }
    }
}

// ── AttributeType ─────────────────────────────────────────────────────────────

/// Minecraft entity attribute type for [`AttributeModifier`].
#[derive(Debug, Clone)]
pub enum AttributeType {
    /// Melee damage dealt by the entity.
    AttackDamage,
    /// How fast the entity attacks (lower = faster).
    AttackSpeed,
    /// Knockback applied by the entity's attacks.
    AttackKnockback,
    /// Physical damage reduction.
    Armor,
    /// Extra damage reduction for strong armor.
    ArmorToughness,
    /// Maximum health points.
    MaxHealth,
    /// Speed of walking/running.
    MovementSpeed,
    /// Speed of flying (for flying entities).
    FlyingSpeed,
    /// Resistance to being knocked back.
    KnockbackResistance,
    /// Extra luck for loot tables.
    Luck,
    /// Jump height.
    JumpStrength,
    /// Zombie reinforcement spawning.
    SpawnReinforcements,
    /// Speed at which blocks are mined.
    BlockBreakSpeed,
    /// Duration of burning damage.
    BurningTime,
    /// Resistance to explosion knockback.
    ExplosionKnockbackResistance,
    /// Multiplier for fall damage.
    FallDamageMultiplier,
    /// Gravity multiplier (affects fall speed).
    Gravity,
    /// Bonus underwater breathing time.
    OxygenBonus,
    /// Safe fall distance before damage.
    SafeFallDistance,
    /// Size scale of the entity.
    Scale,
    /// Height of blocks the entity can step on.
    StepHeight,
    /// Mining speed underwater.
    SubmergedMiningSpeed,
    /// Damage dealt by sweep attacks.
    SweepingDamageRatio,
    /// Speed efficiency in water.
    WaterMovementEfficiency,
    /// Any attribute not covered above (namespace:name format).
    Custom(String),
}

/// Alias for the public item attribute identifier model.
pub type AttributeId = AttributeType;

impl AttributeType {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeType::AttackDamage => "minecraft:attack_damage",
            AttributeType::AttackSpeed => "minecraft:attack_speed",
            AttributeType::AttackKnockback => "minecraft:attack_knockback",
            AttributeType::Armor => "minecraft:armor",
            AttributeType::ArmorToughness => "minecraft:armor_toughness",
            AttributeType::MaxHealth => "minecraft:max_health",
            AttributeType::MovementSpeed => "minecraft:movement_speed",
            AttributeType::FlyingSpeed => "minecraft:flying_speed",
            AttributeType::KnockbackResistance => "minecraft:knockback_resistance",
            AttributeType::Luck => "minecraft:luck",
            AttributeType::JumpStrength => "minecraft:jump_strength",
            AttributeType::SpawnReinforcements => "minecraft:spawn_reinforcements",
            AttributeType::BlockBreakSpeed => "minecraft:block_break_speed",
            AttributeType::BurningTime => "minecraft:burning_time",
            AttributeType::ExplosionKnockbackResistance => {
                "minecraft:explosion_knockback_resistance"
            }
            AttributeType::FallDamageMultiplier => "minecraft:fall_damage_multiplier",
            AttributeType::Gravity => "minecraft:gravity",
            AttributeType::OxygenBonus => "minecraft:oxygen_bonus",
            AttributeType::SafeFallDistance => "minecraft:safe_fall_distance",
            AttributeType::Scale => "minecraft:scale",
            AttributeType::StepHeight => "minecraft:step_height",
            AttributeType::SubmergedMiningSpeed => "minecraft:submerged_mining_speed",
            AttributeType::SweepingDamageRatio => "minecraft:sweeping_damage_ratio",
            AttributeType::WaterMovementEfficiency => "minecraft:water_movement_efficiency",
            AttributeType::Custom(s) => s,
        }
    }
}

// ── AttributeOperation ────────────────────────────────────────────────────────

/// How an [`AttributeModifier`] value is applied to the base attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperation {
    /// Flat addition: `base + amount`
    AddValue,
    /// Scaled addition: `base + (base * amount)`
    AddMultipliedBase,
    /// Multiplicative: `total * (1 + amount)`
    AddMultipliedTotal,
}

impl AttributeOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            AttributeOperation::AddValue => "add_value",
            AttributeOperation::AddMultipliedBase => "add_multiplied_base",
            AttributeOperation::AddMultipliedTotal => "add_multiplied_total",
        }
    }
}

// ── EquipmentSlotGroup ────────────────────────────────────────────────────────

/// Which equipment slot(s) an [`AttributeModifier`] is active in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotGroup {
    /// Active in all slots.
    Any,
    /// Main hand slot only.
    Mainhand,
    /// Off hand slot only.
    Offhand,
    /// Both main and off hand slots.
    Hand,
    /// Head armor slot only.
    Head,
    /// Chest armor slot only.
    Chest,
    /// Legs armor slot only.
    Legs,
    /// Feet armor slot only.
    Feet,
    /// Any armor slot (head, chest, legs, or feet).
    Armor,
    /// Body slots (includes all armor and hand slots).
    Body,
}

impl EquipmentSlotGroup {
    pub fn as_str(self) -> &'static str {
        match self {
            EquipmentSlotGroup::Any => "any",
            EquipmentSlotGroup::Mainhand => "mainhand",
            EquipmentSlotGroup::Offhand => "offhand",
            EquipmentSlotGroup::Hand => "hand",
            EquipmentSlotGroup::Head => "head",
            EquipmentSlotGroup::Chest => "chest",
            EquipmentSlotGroup::Legs => "legs",
            EquipmentSlotGroup::Feet => "feet",
            EquipmentSlotGroup::Armor => "armor",
            EquipmentSlotGroup::Body => "body",
        }
    }
}

// ── AttributeModifier ─────────────────────────────────────────────────────────

/// A single entry in the `attribute_modifiers` item component.
#[derive(Debug, Clone)]
pub struct AttributeModifier {
    attribute: AttributeType,
    amount: f64,
    operation: AttributeOperation,
    slot: EquipmentSlotGroup,
    /// Resource-location ID for the modifier (e.g. `"my_pack:sword_damage"`).
    /// Recommended to avoid collisions between datapacks.
    id: Option<String>,
}

impl AttributeModifier {
    /// Create a new attribute modifier for the given attribute.
    pub fn new(attribute: AttributeType) -> Self {
        Self {
            attribute,
            amount: 0.0,
            operation: AttributeOperation::AddValue,
            slot: EquipmentSlotGroup::Any,
            id: None,
        }
    }

    /// Create a fully specified attribute modifier in one call.
    pub fn with_values(
        attribute: AttributeType,
        amount: f64,
        operation: AttributeOperation,
        slot: EquipmentSlotGroup,
    ) -> Self {
        Self::new(attribute)
            .amount(amount)
            .operation(operation)
            .slot(slot)
    }

    /// Set the modifier amount.
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    /// Set the modifier operation.
    pub fn operation(mut self, operation: AttributeOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Set the equipment slot group where this modifier applies.
    pub fn slot(mut self, slot: EquipmentSlotGroup) -> Self {
        self.slot = slot;
        self
    }

    /// Set a unique resource-location identifier for this modifier (e.g. `"my_pack:bonus_damage"`).
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn to_snbt(&self) -> String {
        // Minecraft 1.21+ requires an `id` field on every attribute modifier.
        // Fall back to the attribute type's resource location when none is set.
        let id = self
            .id
            .as_deref()
            .unwrap_or_else(|| self.attribute.as_str());
        format!(
            "{{id:\"{}\",type:\"{}\",amount:{}d,operation:\"{}\",slot:\"{}\"}}",
            id,
            self.attribute.as_str(),
            self.amount,
            self.operation.as_str(),
            self.slot.as_str(),
        )
    }

    /// Structured JSON form of this modifier, for recipe results and similar
    /// JSON-based schemas. Same data as [`to_snbt`](Self::to_snbt), different target.
    fn to_json(&self) -> Value {
        let id = self
            .id
            .as_deref()
            .unwrap_or_else(|| self.attribute.as_str());
        serde_json::json!({
            "id": id,
            "type": self.attribute.as_str(),
            "amount": self.amount,
            "operation": self.operation.as_str(),
            "slot": self.slot.as_str(),
        })
    }
}

// ── EnchantmentEntry ─────────────────────────────────────────────────────────

/// A typed enchantment level entry for item components.
#[derive(Debug, Clone)]
pub struct EnchantmentEntry {
    id: EnchantmentId,
    level: u32,
}

impl EnchantmentEntry {
    /// Create a typed enchantment entry.
    pub fn new(id: EnchantmentId, level: u32) -> Self {
        Self { id, level }
    }
}

// ── CustomData ───────────────────────────────────────────────────────────────

/// Typed wrapper for the `custom_data` item component.
#[derive(Debug, Clone)]
pub enum CustomData {
    /// A common marker emitted as `{key:1b}`.
    Marker(String),
    /// Explicit raw SNBT for complex custom data payloads.
    Raw(RawSnbt),
}

impl CustomData {
    /// Create a marker custom data payload: `{key:1b}`.
    pub fn marker(key: impl Into<String>) -> Self {
        Self::Marker(key.into())
    }

    /// Wrap raw custom data SNBT explicitly.
    pub fn raw(snbt: RawSnbt) -> Self {
        Self::Raw(snbt)
    }

    fn marker_key(&self) -> Option<&str> {
        match self {
            CustomData::Marker(key) => Some(key),
            CustomData::Raw(_) => None,
        }
    }

    fn to_snbt(&self) -> String {
        match self {
            CustomData::Marker(key) => format!("{{{}:1b}}", snbt_compound_key(key)),
            CustomData::Raw(snbt) => snbt.to_string(),
        }
    }

    /// Structured JSON form, when representable.
    ///
    /// `Marker` converts to `{key: true}`. `Raw` wraps arbitrary SNBT that
    /// has no general SNBT→JSON conversion, so it returns `None` — callers
    /// (e.g. recipe result conversion) must surface a clear error instead of
    /// silently dropping or corrupting the raw payload.
    fn to_json(&self) -> Option<Value> {
        match self {
            CustomData::Marker(key) => {
                let mut map = Map::new();
                map.insert(key.clone(), Value::Bool(true));
                Some(Value::Object(map))
            }
            CustomData::Raw(_) => None,
        }
    }
}

impl From<RawSnbt> for CustomData {
    fn from(value: RawSnbt) -> Self {
        Self::Raw(value)
    }
}

// ── ItemComponent ────────────────────────────────────────────────────────────

/// Strongly typed Minecraft item component values used by [`CustomItem`].
#[derive(Debug, Clone)]
pub enum ItemComponent {
    CustomName(TextComponent),
    ItemName(TextComponent),
    Lore(Vec<TextComponent>),
    Rarity(ItemRarity),
    CustomModelData(i32),
    Enchantments(Vec<EnchantmentEntry>),
    StoredEnchantments(Vec<EnchantmentEntry>),
    AttributeModifiers(Vec<AttributeModifier>),
    Food(FoodProperties),
    Consumable(ConsumableProperties),
    Equippable(EquippableProperties),
    Tool(ToolProperties),
    PotionContents(PotionContents),
    SuspiciousStewEffects(Vec<SuspiciousStewEffect>),
    MaxStackSize(u32),
    MaxDamage(i32),
    Damage(i32),
    Unbreakable { show_in_tooltip: bool },
    CustomData(CustomData),
    EnchantmentGlintOverride(bool),
    HideAdditionalTooltip,
    HideTooltip,
    RepairCost(i32),
    UseCooldown(f32),
    Glider,
    FireResistant,
    DyedColor(DyedColor),
    Raw(RawComponent),
}

impl ItemComponent {
    pub fn custom_name(name: TextComponent) -> Self {
        Self::CustomName(name)
    }

    pub fn item_name(name: TextComponent) -> Self {
        Self::ItemName(name)
    }

    pub fn lore(lines: Vec<TextComponent>) -> Self {
        Self::Lore(lines)
    }

    pub fn lore_line(line: TextComponent) -> Self {
        Self::Lore(vec![line])
    }

    pub fn rarity(rarity: ItemRarity) -> Self {
        Self::Rarity(rarity)
    }

    pub fn custom_model_data(value: i32) -> Self {
        Self::CustomModelData(value)
    }

    pub fn enchantment(id: EnchantmentId, level: u32) -> Self {
        Self::Enchantments(vec![EnchantmentEntry::new(id, level)])
    }

    pub fn enchantments(entries: Vec<EnchantmentEntry>) -> Self {
        Self::Enchantments(entries)
    }

    pub fn stored_enchantment(id: EnchantmentId, level: u32) -> Self {
        Self::StoredEnchantments(vec![EnchantmentEntry::new(id, level)])
    }

    pub fn attribute_modifier(modifier: AttributeModifier) -> Self {
        Self::AttributeModifiers(vec![modifier])
    }

    pub fn attribute_modifiers(modifiers: Vec<AttributeModifier>) -> Self {
        Self::AttributeModifiers(modifiers)
    }

    pub fn food(food: FoodProperties) -> Self {
        Self::Food(food)
    }

    pub fn consumable(consumable: ConsumableProperties) -> Self {
        Self::Consumable(consumable)
    }

    pub fn equippable(equippable: EquippableProperties) -> Self {
        Self::Equippable(equippable)
    }

    pub fn tool(tool: ToolProperties) -> Self {
        Self::Tool(tool)
    }

    pub fn potion_contents(contents: PotionContents) -> Self {
        Self::PotionContents(contents)
    }

    pub fn suspicious_stew_effect(effect: SuspiciousStewEffect) -> Self {
        Self::SuspiciousStewEffects(vec![effect])
    }

    pub fn suspicious_stew_effects(effects: Vec<SuspiciousStewEffect>) -> Self {
        Self::SuspiciousStewEffects(effects)
    }

    pub fn max_stack_size(size: u32) -> Self {
        Self::MaxStackSize(size)
    }

    pub fn max_damage(damage: i32) -> Self {
        Self::MaxDamage(damage)
    }

    pub fn damage(damage: i32) -> Self {
        Self::Damage(damage)
    }

    pub fn unbreakable(show_in_tooltip: bool) -> Self {
        Self::Unbreakable { show_in_tooltip }
    }

    pub fn custom_data(data: CustomData) -> Self {
        Self::CustomData(data)
    }

    pub fn custom_data_marker(key: impl Into<String>) -> Self {
        Self::CustomData(CustomData::marker(key))
    }

    pub fn raw_component(component: RawComponent) -> Self {
        Self::Raw(component)
    }
}

// ── FoodProperties ────────────────────────────────────────────────────────────

/// Properties for the `food` item component.
#[derive(Debug, Clone)]
pub struct FoodProperties {
    /// Hunger points restored (1-20).
    pub nutrition: i32,
    /// Saturation restored (usually 0.0-2.0).
    pub saturation: f32,
    /// Whether the food can be eaten even with full hunger.
    pub can_always_eat: bool,
}

impl FoodProperties {
    /// Create food properties with the given nutrition and saturation values.
    pub fn new(nutrition: i32, saturation: f32) -> Self {
        Self {
            nutrition,
            saturation,
            can_always_eat: false,
        }
    }

    /// Set whether this food can be eaten with a full hunger bar.
    pub fn can_always_eat(mut self, v: bool) -> Self {
        self.can_always_eat = v;
        self
    }

    fn to_snbt(&self) -> String {
        format!(
            "{{nutrition:{},saturation:{}f,can_always_eat:{}}}",
            self.nutrition, self.saturation, self.can_always_eat,
        )
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "nutrition": self.nutrition,
            "saturation": self.saturation,
            "can_always_eat": self.can_always_eat,
        })
    }
}

// ── ConsumableAnimation ───────────────────────────────────────────────────────

/// Use animation for the `consumable` item component.
///
/// Specifies the animation played when consuming an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumableAnimation {
    /// No animation.
    None,
    /// Eating animation (like food).
    Eat,
    /// Drinking animation (like potions).
    Drink,
    /// Blocking animation (like shields).
    Block,
    /// Bow drawing animation.
    Bow,
    /// Spear throwing animation.
    Spear,
    /// Crossbow loading animation.
    Crossbow,
    /// Spyglass animation.
    Spyglass,
    /// Horn tooting animation.
    TootHorn,
    /// Brush animation.
    Brush,
}

impl ConsumableAnimation {
    pub fn as_str(self) -> &'static str {
        match self {
            ConsumableAnimation::None => "none",
            ConsumableAnimation::Eat => "eat",
            ConsumableAnimation::Drink => "drink",
            ConsumableAnimation::Block => "block",
            ConsumableAnimation::Bow => "bow",
            ConsumableAnimation::Spear => "spear",
            ConsumableAnimation::Crossbow => "crossbow",
            ConsumableAnimation::Spyglass => "spyglass",
            ConsumableAnimation::TootHorn => "toot_horn",
            ConsumableAnimation::Brush => "brush",
        }
    }
}

// ── ConsumableProperties ──────────────────────────────────────────────────────

/// Properties for the `consumable` item component.
#[derive(Debug, Clone)]
pub struct ConsumableProperties {
    /// Time in seconds to consume the item.
    pub consume_seconds: f32,
    /// Animation to play during consumption.
    pub animation: ConsumableAnimation,
    /// Whether particles appear when consuming.
    pub has_consume_particles: bool,
    /// Optional custom sound to play.
    pub sound: Option<String>,
}

impl ConsumableProperties {
    /// Create consumable properties with the given consumption duration in seconds.
    pub fn new(consume_seconds: f32) -> Self {
        Self {
            consume_seconds,
            animation: ConsumableAnimation::Eat,
            has_consume_particles: true,
            sound: None,
        }
    }

    /// Set the animation to play when consuming this item.
    pub fn animation(mut self, animation: ConsumableAnimation) -> Self {
        self.animation = animation;
        self
    }

    /// Set whether particles appear when consuming.
    pub fn has_consume_particles(mut self, v: bool) -> Self {
        self.has_consume_particles = v;
        self
    }

    /// Set a custom sound to play when consuming (e.g. `"minecraft:entity.player.burp"`).
    pub fn sound(mut self, sound: impl fmt::Display) -> Self {
        self.sound = Some(sound.to_string());
        self
    }

    fn to_snbt(&self) -> String {
        let sound_part = match &self.sound {
            Some(s) => format!(",sound:\"{}\"", s),
            None => String::new(),
        };
        format!(
            "{{consume_seconds:{}f,animation:\"{}\",has_consume_particles:{}{}}}",
            self.consume_seconds,
            self.animation.as_str(),
            self.has_consume_particles,
            sound_part,
        )
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("consume_seconds".into(), Value::from(self.consume_seconds));
        map.insert(
            "animation".into(),
            Value::String(self.animation.as_str().to_string()),
        );
        map.insert(
            "has_consume_particles".into(),
            Value::Bool(self.has_consume_particles),
        );
        if let Some(ref sound) = self.sound {
            map.insert("sound".into(), Value::String(sound.clone()));
        }
        Value::Object(map)
    }
}

// ── EquippableProperties ──────────────────────────────────────────────────────

/// Equipment slot for the `equippable` item component.
///
/// Specifies which equipment slot an item can be equipped into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    /// Head armor slot.
    Head,
    /// Chest armor slot.
    Chest,
    /// Legs armor slot.
    Legs,
    /// Feet armor slot.
    Feet,
    /// Body (all armor slots).
    Body,
    /// Main hand weapon slot.
    Mainhand,
    /// Off hand slot.
    Offhand,
}

impl EquipmentSlot {
    pub fn as_str(self) -> &'static str {
        match self {
            EquipmentSlot::Head => "head",
            EquipmentSlot::Chest => "chest",
            EquipmentSlot::Legs => "legs",
            EquipmentSlot::Feet => "feet",
            EquipmentSlot::Body => "body",
            EquipmentSlot::Mainhand => "mainhand",
            EquipmentSlot::Offhand => "offhand",
        }
    }
}

/// Properties for the `equippable` item component.
///
/// Configures whether an item can be equipped and its behavior when equipped.
#[derive(Debug, Clone)]
pub struct EquippableProperties {
    /// The equipment slot this item occupies.
    pub slot: EquipmentSlot,
    /// Whether dispensers can automatically equip this item.
    pub dispensable: bool,
    /// Whether players can swap this item with existing equipment.
    pub swappable: bool,
    /// Whether the item takes damage when the wearer is hurt.
    pub damage_on_hurt: bool,
    /// Optional sound to play when equipping.
    pub equip_sound: Option<String>,
    /// Optional custom model for the equipped item.
    pub model: Option<String>,
    /// Optional entity tag restricting who can wear this.
    pub allowed_entities: Option<String>,
}

impl EquippableProperties {
    /// Create equippable properties for the given equipment slot.
    pub fn new(slot: EquipmentSlot) -> Self {
        Self {
            slot,
            dispensable: true,
            swappable: true,
            damage_on_hurt: true,
            equip_sound: None,
            model: None,
            allowed_entities: None,
        }
    }

    /// Set whether dispensers can automatically equip this item.
    pub fn dispensable(mut self, v: bool) -> Self {
        self.dispensable = v;
        self
    }
    /// Set whether players can swap this item with existing equipment.
    pub fn swappable(mut self, v: bool) -> Self {
        self.swappable = v;
        self
    }
    /// Set whether the item takes damage when the wearer is hurt.
    pub fn damage_on_hurt(mut self, v: bool) -> Self {
        self.damage_on_hurt = v;
        self
    }
    /// Set a sound to play when equipping (e.g. `"minecraft:item.armor.equip_diamond"`).
    pub fn equip_sound(mut self, sound: impl fmt::Display) -> Self {
        self.equip_sound = Some(sound.to_string());
        self
    }
    /// Set a custom model override for this equipped item.
    pub fn model(mut self, model: impl fmt::Display) -> Self {
        self.model = Some(model.to_string());
        self
    }
    /// Restrict equipping to entities with a specific tag.
    pub fn allowed_entities(mut self, tag: impl fmt::Display) -> Self {
        self.allowed_entities = Some(tag.to_string());
        self
    }
    /// Restrict equipping to a specific entity type.
    pub fn allowed_entity_type(mut self, entity_type: EntityTypeId) -> Self {
        self.allowed_entities = Some(entity_type.to_string());
        self
    }
    /// Restrict equipping to an entity type tag.
    pub fn allowed_entity_tag(mut self, tag: TagId<EntityTypeId>) -> Self {
        self.allowed_entities = Some(tag.to_tag_string());
        self
    }

    fn to_snbt(&self) -> String {
        let mut parts = vec![
            format!("slot:\"{}\"", self.slot.as_str()),
            format!("dispensable:{}", self.dispensable),
            format!("swappable:{}", self.swappable),
            format!("damage_on_hurt:{}", self.damage_on_hurt),
        ];
        if let Some(ref s) = self.equip_sound {
            parts.push(format!("equip_sound:\"{}\"", s));
        }
        if let Some(ref m) = self.model {
            parts.push(format!("model:\"{}\"", m));
        }
        if let Some(ref e) = self.allowed_entities {
            parts.push(format!("allowed_entities:\"{}\"", e));
        }
        format!("{{{}}}", parts.join(","))
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("slot".into(), Value::String(self.slot.as_str().to_string()));
        map.insert("dispensable".into(), Value::Bool(self.dispensable));
        map.insert("swappable".into(), Value::Bool(self.swappable));
        map.insert("damage_on_hurt".into(), Value::Bool(self.damage_on_hurt));
        if let Some(ref s) = self.equip_sound {
            map.insert("equip_sound".into(), Value::String(s.clone()));
        }
        if let Some(ref m) = self.model {
            map.insert("model".into(), Value::String(m.clone()));
        }
        if let Some(ref e) = self.allowed_entities {
            map.insert("allowed_entities".into(), Value::String(e.clone()));
        }
        Value::Object(map)
    }
}

// ── ToolRule ──────────────────────────────────────────────────────────────────

/// A single rule in the `tool` item component.
#[derive(Debug, Clone)]
pub struct ToolRule {
    /// Block or block tag to match (e.g. `"#minecraft:pickaxe_mineable"`).
    pub blocks: String,
    /// Optional mining speed multiplier for this rule.
    pub speed: Option<f32>,
    /// Optional flag for whether the tool drops blocks correctly.
    pub correct_for_drops: Option<bool>,
}

impl ToolRule {
    /// Create a new tool rule for the given block or block tag.
    pub fn new(blocks: impl fmt::Display) -> Self {
        Self {
            blocks: blocks.to_string(),
            speed: None,
            correct_for_drops: None,
        }
    }

    /// Create a new tool rule for one block.
    pub fn block(block: BlockId) -> Self {
        Self::new(block)
    }

    /// Create a new tool rule for a block tag.
    pub fn tag(tag: TagId<BlockId>) -> Self {
        Self::new(tag.to_tag_string())
    }

    /// Set the mining speed multiplier (1.0 = normal, 2.0 = twice as fast).
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }
    /// Set whether this tool is capable of correctly mining the blocks.
    pub fn correct_for_drops(mut self, v: bool) -> Self {
        self.correct_for_drops = Some(v);
        self
    }

    fn to_snbt(&self) -> String {
        let mut parts = vec![format!("blocks:\"{}\"", self.blocks)];
        if let Some(s) = self.speed {
            parts.push(format!("speed:{}f", s));
        }
        if let Some(c) = self.correct_for_drops {
            parts.push(format!("correct_for_drops:{}", c));
        }
        format!("{{{}}}", parts.join(","))
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("blocks".into(), Value::String(self.blocks.clone()));
        if let Some(s) = self.speed {
            map.insert("speed".into(), Value::from(s));
        }
        if let Some(c) = self.correct_for_drops {
            map.insert("correct_for_drops".into(), Value::Bool(c));
        }
        Value::Object(map)
    }
}

// ── ToolProperties ────────────────────────────────────────────────────────────

/// Properties for the `tool` item component.
#[derive(Debug, Clone)]
pub struct ToolProperties {
    /// Rules for specific block types or tags.
    pub rules: Vec<ToolRule>,
    /// Default mining speed for blocks not matching any rule.
    pub default_mining_speed: f32,
    /// Durability damage taken per broken block.
    pub damage_per_block: i32,
}

impl ToolProperties {
    /// Create a new tool with default properties (1.0x speed, 1 damage per block).
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_mining_speed: 1.0,
            damage_per_block: 1,
        }
    }

    /// Add a tool rule for specific block types.
    pub fn rule(mut self, rule: ToolRule) -> Self {
        self.rules.push(rule);
        self
    }
    /// Set the default mining speed for blocks not matching any rule.
    pub fn default_mining_speed(mut self, speed: f32) -> Self {
        self.default_mining_speed = speed;
        self
    }
    /// Set how much durability damage this tool takes per broken block.
    pub fn damage_per_block(mut self, damage: i32) -> Self {
        self.damage_per_block = damage;
        self
    }

    fn to_snbt(&self) -> String {
        let rules: Vec<String> = self.rules.iter().map(|r| r.to_snbt()).collect();
        format!(
            "{{rules:[{}],default_mining_speed:{}f,damage_per_block:{}}}",
            rules.join(","),
            self.default_mining_speed,
            self.damage_per_block,
        )
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "rules": self.rules.iter().map(ToolRule::to_json).collect::<Vec<_>>(),
            "default_mining_speed": self.default_mining_speed,
            "damage_per_block": self.damage_per_block,
        })
    }
}

impl Default for ToolProperties {
    fn default() -> Self {
        Self::new()
    }
}

// ── DyedColor ─────────────────────────────────────────────────────────────────

/// RGB color for the `dyed_color` item component (leather armor, etc.).
#[derive(Debug, Clone, Copy)]
pub struct DyedColor {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
}

impl DyedColor {
    /// Create a color from individual red, green, and blue values (0-255 each).
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Construct a color from a 24-bit hex integer (e.g. `0xFF5733` for orange).
    pub fn hex(rgb: u32) -> Self {
        Self {
            r: ((rgb >> 16) & 0xFF) as u8,
            g: ((rgb >> 8) & 0xFF) as u8,
            b: (rgb & 0xFF) as u8,
        }
    }

    fn to_decimal(self) -> i32 {
        ((self.r as i32) << 16) | ((self.g as i32) << 8) | (self.b as i32)
    }

    fn to_json(self) -> Value {
        serde_json::json!({
            "rgb": self.to_decimal(),
            "show_in_tooltip": true,
        })
    }
}

// ── ItemStackComponents ──────────────────────────────────────────────────────

/// A structured, JSON-serializable snapshot of an item's base ID and data
/// components — Minecraft's *structured* component form (`{"minecraft:key":
/// value}`), as opposed to the SNBT-based command item-stack syntax that
/// [`CustomItem`]'s [`Display`](fmt::Display) impl produces.
///
/// Built once from [`CustomItem::stack_components`] and shared by every
/// consumer that needs JSON components — currently [`RecipeResult`]
/// (`recipe::RecipeResult::custom_item`) — without re-deriving or
/// re-parsing `CustomItem`'s command-syntax string.
///
/// # Duplicate component keys
///
/// [`CustomItem`]'s typed fields can never collide (each typed component
/// contributes at most one JSON key). A user-supplied
/// [`RawComponent`](crate::raw::RawComponent) can collide with a typed
/// component or with another raw component; the later entry wins, replacing
/// the value at the key's original position, so iteration order stays
/// deterministic regardless of which call inserted the winning value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemStackComponents {
    base: String,
    components: Vec<(String, Value)>,
}

impl ItemStackComponents {
    /// The base Minecraft item ID (e.g. `"minecraft:white_wool"`).
    pub fn base_item(&self) -> &str {
        &self.base
    }

    /// The structured components, in deterministic (first-seen-position) order.
    pub fn components(&self) -> &[(String, Value)] {
        &self.components
    }

    /// `true` if this item has no data components — a bare base-item stack.
    pub fn is_component_free(&self) -> bool {
        self.components.is_empty()
    }

    /// Consume this value, returning the base item ID and its components.
    pub fn into_parts(self) -> (String, Vec<(String, Value)>) {
        (self.base, self.components)
    }

    fn insert(&mut self, key: impl Into<String>, value: Value) {
        let key = key.into();
        if let Some(existing) = self.components.iter_mut().find(|(k, _)| *k == key) {
            existing.1 = value;
        } else {
            self.components.push((key, value));
        }
    }
}

/// Sentinel location used for [`SandError::ComponentValidation`] diagnostics
/// raised while building [`ItemStackComponents`] — this conversion happens
/// before the value is attached to any specific recipe/resource location, so
/// the offending item base and component key are named in the message instead.
fn item_stack_components_location() -> ResourceLocation {
    ResourceLocation::new("sand", "custom_item")
        .expect("fixed 'sand:custom_item' sentinel location is valid")
}

fn item_component_error(base: &str, key: &str, message: impl fmt::Display) -> SandError {
    SandError::ComponentValidation {
        location: item_stack_components_location(),
        kind: "custom_item".to_string(),
        field: format!("{base}[{key}]"),
        message: message.to_string(),
    }
}

// ── CustomItem ────────────────────────────────────────────────────────────────

/// A custom item definition using the Minecraft 1.21+ item component system.
///
/// The item formats as `base[component1=val1,component2=val2,...]` and can be
/// passed directly to [`cmd::give`](crate::cmd) since it implements `Into<String>`.
///
/// # Item identity
///
/// Use [`custom_data`](Self::custom_data) to tag the item with a unique key.
/// This is the most reliable way to detect the item in advancements and predicates.
/// Use [`custom_model_data`](Self::custom_model_data) separately for resourcepack
/// model overrides.
#[derive(Debug, Clone)]
pub struct CustomItem {
    base: String,

    // ── Identity ──────────────────────────────────────────────────────────────
    custom_data: Option<CustomData>,
    custom_model_data: Option<i32>,

    // ── Display ───────────────────────────────────────────────────────────────
    /// Pre-serialised JSON string for `custom_name`.
    custom_name: Option<String>,
    /// Pre-serialised JSON string for `item_name`.
    item_name: Option<String>,
    /// Pre-serialised JSON strings for each lore line.
    lore: Vec<String>,
    rarity: Option<ItemRarity>,
    enchantment_glint_override: Option<bool>,
    hide_additional_tooltip: bool,
    hide_tooltip: bool,

    // ── Stack / durability ────────────────────────────────────────────────────
    max_stack_size: Option<u32>,
    max_damage: Option<i32>,
    damage: Option<i32>,
    /// `None` = not unbreakable; `Some(show_in_tooltip)` = unbreakable.
    unbreakable: Option<bool>,
    repair_cost: Option<i32>,

    // ── Combat / enchanting ───────────────────────────────────────────────────
    enchantments: Vec<(String, u32)>,
    /// Enchantments stored without being applied (for enchanted books).
    stored_enchantments: Vec<(String, u32)>,
    attribute_modifiers: Vec<AttributeModifier>,

    // ── Behaviour ─────────────────────────────────────────────────────────────
    food: Option<FoodProperties>,
    consumable: Option<ConsumableProperties>,
    use_cooldown: Option<f32>,
    tool: Option<ToolProperties>,
    equippable: Option<EquippableProperties>,
    glider: bool,
    fire_resistant: bool,
    dyed_color: Option<DyedColor>,
    potion_contents: Option<PotionContents>,
    suspicious_stew_effects: Vec<SuspiciousStewEffect>,

    // ── Raw escape hatch ──────────────────────────────────────────────────────
    /// Additional raw `key=snbt_value` components appended verbatim.
    extra_components: Vec<(String, String)>,
}

impl CustomItem {
    /// Create a new custom item from a base Minecraft item ID.
    ///
    /// ```
    /// use sand_components::CustomItem;
    /// let item = CustomItem::new("minecraft:diamond_sword");
    /// assert!(item.to_string().starts_with("minecraft:diamond_sword"));
    /// ```
    pub fn new(base: impl fmt::Display) -> Self {
        Self {
            base: base.to_string(),
            custom_data: None,
            custom_model_data: None,
            custom_name: None,
            item_name: None,
            lore: Vec::new(),
            rarity: None,
            enchantment_glint_override: None,
            hide_additional_tooltip: false,
            hide_tooltip: false,
            max_stack_size: None,
            max_damage: None,
            damage: None,
            unbreakable: None,
            repair_cost: None,
            enchantments: Vec::new(),
            stored_enchantments: Vec::new(),
            attribute_modifiers: Vec::new(),
            food: None,
            consumable: None,
            use_cooldown: None,
            tool: None,
            equippable: None,
            glider: false,
            fire_resistant: false,
            dyed_color: None,
            potion_contents: None,
            suspicious_stew_effects: Vec::new(),
            extra_components: Vec::new(),
        }
    }

    /// Add or merge a typed item component.
    pub fn component(mut self, component: ItemComponent) -> Self {
        self.apply_component(component);
        self
    }

    fn apply_component(&mut self, component: ItemComponent) {
        match component {
            ItemComponent::CustomName(name) => self.custom_name = Some(name.to_string()),
            ItemComponent::ItemName(name) => self.item_name = Some(name.to_string()),
            ItemComponent::Lore(lines) => {
                self.lore.extend(lines.into_iter().map(|l| l.to_string()))
            }
            ItemComponent::Rarity(rarity) => self.rarity = Some(rarity),
            ItemComponent::CustomModelData(value) => self.custom_model_data = Some(value),
            ItemComponent::Enchantments(entries) => self.enchantments.extend(
                entries
                    .into_iter()
                    .map(|entry| (entry.id.to_string(), entry.level)),
            ),
            ItemComponent::StoredEnchantments(entries) => self.stored_enchantments.extend(
                entries
                    .into_iter()
                    .map(|entry| (entry.id.to_string(), entry.level)),
            ),
            ItemComponent::AttributeModifiers(modifiers) => {
                self.attribute_modifiers.extend(modifiers);
            }
            ItemComponent::Food(food) => self.food = Some(food),
            ItemComponent::Consumable(consumable) => self.consumable = Some(consumable),
            ItemComponent::Equippable(equippable) => self.equippable = Some(equippable),
            ItemComponent::Tool(tool) => self.tool = Some(tool),
            ItemComponent::PotionContents(contents) => self.potion_contents = Some(contents),
            ItemComponent::SuspiciousStewEffects(effects) => {
                self.suspicious_stew_effects.extend(effects);
            }
            ItemComponent::MaxStackSize(size) => self.max_stack_size = Some(size),
            ItemComponent::MaxDamage(damage) => self.max_damage = Some(damage),
            ItemComponent::Damage(damage) => self.damage = Some(damage),
            ItemComponent::Unbreakable { show_in_tooltip } => {
                self.unbreakable = Some(show_in_tooltip);
            }
            ItemComponent::CustomData(data) => self.custom_data = Some(data),
            ItemComponent::EnchantmentGlintOverride(glint) => {
                self.enchantment_glint_override = Some(glint);
            }
            ItemComponent::HideAdditionalTooltip => self.hide_additional_tooltip = true,
            ItemComponent::HideTooltip => self.hide_tooltip = true,
            ItemComponent::RepairCost(cost) => self.repair_cost = Some(cost),
            ItemComponent::UseCooldown(seconds) => self.use_cooldown = Some(seconds),
            ItemComponent::Glider => self.glider = true,
            ItemComponent::FireResistant => self.fire_resistant = true,
            ItemComponent::DyedColor(color) => self.dyed_color = Some(color),
            ItemComponent::Raw(component) => self
                .extra_components
                .push((component.key().to_owned(), component.value().to_owned())),
        }
    }

    // ── Identity ──────────────────────────────────────────────────────────────

    /// Tag this item with a unique key in `custom_data` (e.g. `"inferno_blade"`).
    ///
    /// Emits `custom_data={inferno_blade:1b}` and enables item-predicate helpers
    /// like [`item_predicate`](Self::item_predicate) and
    /// [`on_use_advancement`](Self::on_use_advancement).
    pub fn custom_data(mut self, key: impl Into<String>) -> Self {
        self.custom_data = Some(CustomData::marker(key));
        self
    }

    /// Set this custom item's stable ID as a namespaced `custom_data` marker.
    pub fn id(self, id: impl Into<String>) -> Self {
        self.component(ItemComponent::custom_data_marker(id))
    }

    /// Set typed custom data for this item.
    pub fn typed_custom_data(mut self, data: CustomData) -> Self {
        self.custom_data = Some(data);
        self
    }

    /// Set `custom_model_data` for pairing with resourcepack model overrides.
    ///
    /// Emits `custom_model_data={floats:[N.0f]}` (1.21.4+ format).
    pub fn custom_model_data(self, value: i32) -> Self {
        self.component(ItemComponent::custom_model_data(value))
    }

    // ── Display ───────────────────────────────────────────────────────────────

    /// Set the item's custom display name (not italicized).
    pub fn custom_name(self, name: TextComponent) -> Self {
        self.component(ItemComponent::custom_name(name))
    }

    /// Set the item name component (shown italicized in UI). Use `custom_name` for non-italic text.
    pub fn item_name(self, name: TextComponent) -> Self {
        self.component(ItemComponent::item_name(name))
    }

    /// Add a single lore line to the item.
    pub fn lore_line(self, line: TextComponent) -> Self {
        self.component(ItemComponent::lore_line(line))
    }

    /// Add multiple lore lines at once.
    pub fn lore(self, lines: Vec<TextComponent>) -> Self {
        self.component(ItemComponent::lore(lines))
    }

    /// Set the rarity level (affects item name color).
    pub fn rarity(self, rarity: ItemRarity) -> Self {
        self.component(ItemComponent::rarity(rarity))
    }

    /// Force or suppress the enchantment glint animation.
    pub fn enchantment_glint_override(self, glint: bool) -> Self {
        self.component(ItemComponent::EnchantmentGlintOverride(glint))
    }

    /// Hide the additional tooltip section (enchantments, attributes, etc.).
    pub fn hide_additional_tooltip(self) -> Self {
        self.component(ItemComponent::HideAdditionalTooltip)
    }

    /// Hide the entire item tooltip.
    pub fn hide_tooltip(self) -> Self {
        self.component(ItemComponent::HideTooltip)
    }

    // ── Stack / durability ────────────────────────────────────────────────────

    /// Set the maximum stack size for this item.
    pub fn max_stack_size(self, size: u32) -> Self {
        self.component(ItemComponent::max_stack_size(size))
    }

    /// Set the maximum durability (creates a damageable item).
    pub fn max_damage(self, damage: i32) -> Self {
        self.component(ItemComponent::max_damage(damage))
    }

    /// Set the current damage value for this item.
    pub fn damage(self, damage: i32) -> Self {
        self.component(ItemComponent::damage(damage))
    }

    /// Mark the item as unbreakable.
    ///
    /// `show_in_tooltip` controls whether "Unbreakable" is shown in the tooltip.
    pub fn unbreakable(self, show_in_tooltip: bool) -> Self {
        self.component(ItemComponent::unbreakable(show_in_tooltip))
    }

    /// Set the experience cost to repair this item at an anvil.
    pub fn repair_cost(self, cost: i32) -> Self {
        self.component(ItemComponent::RepairCost(cost))
    }

    // ── Combat / enchanting ───────────────────────────────────────────────────

    /// Add an enchantment by resource-location ID and level.
    pub fn enchantment(mut self, id: impl Into<String>, level: u32) -> Self {
        self.enchantments.push((id.into(), level));
        self
    }

    /// Add a typed enchantment by ID and level.
    pub fn typed_enchantment(self, id: EnchantmentId, level: u32) -> Self {
        self.component(ItemComponent::enchantment(id, level))
    }

    /// Add a stored enchantment (for enchanted books).
    pub fn stored_enchantment(mut self, id: impl Into<String>, level: u32) -> Self {
        self.stored_enchantments.push((id.into(), level));
        self
    }

    /// Add a typed stored enchantment by ID and level.
    pub fn typed_stored_enchantment(self, id: EnchantmentId, level: u32) -> Self {
        self.component(ItemComponent::stored_enchantment(id, level))
    }

    /// Add a pre-built [`AttributeModifier`].
    pub fn attribute_modifier(self, modifier: AttributeModifier) -> Self {
        self.component(ItemComponent::attribute_modifier(modifier))
    }

    /// Convenience shorthand for the common case of a single attribute modifier.
    pub fn attribute(
        self,
        attr: AttributeType,
        amount: f64,
        operation: AttributeOperation,
        slot: EquipmentSlotGroup,
    ) -> Self {
        self.component(ItemComponent::attribute_modifier(
            AttributeModifier::with_values(attr, amount, operation, slot),
        ))
    }

    // ── Behaviour ─────────────────────────────────────────────────────────────

    /// Add food properties to this item (makes it edible).
    pub fn food(self, food: FoodProperties) -> Self {
        self.component(ItemComponent::food(food))
    }

    /// Add consumable properties to this item.
    pub fn consumable(self, consumable: ConsumableProperties) -> Self {
        self.component(ItemComponent::consumable(consumable))
    }

    /// Set a use cooldown (in seconds) between each use.
    pub fn use_cooldown(self, seconds: f32) -> Self {
        self.component(ItemComponent::UseCooldown(seconds))
    }

    /// Add tool properties to this item (makes it a tool/weapon).
    pub fn tool(self, tool: ToolProperties) -> Self {
        self.component(ItemComponent::tool(tool))
    }

    /// Make this item equippable in a specific slot.
    pub fn equippable(self, equippable: EquippableProperties) -> Self {
        self.component(ItemComponent::equippable(equippable))
    }

    /// Make this item function as a glider (like an elytra).
    pub fn glider(self) -> Self {
        self.component(ItemComponent::Glider)
    }

    /// Mark this item as fire-resistant (won't burn in lava or fire).
    pub fn fire_resistant(self) -> Self {
        self.component(ItemComponent::FireResistant)
    }

    /// Set a dye color for this item (for leather armor, etc.).
    pub fn dyed_color(self, color: DyedColor) -> Self {
        self.component(ItemComponent::DyedColor(color))
    }

    /// Set typed `minecraft:potion_contents` component data.
    pub fn potion_contents(self, contents: PotionContents) -> Self {
        self.component(ItemComponent::potion_contents(contents))
    }

    /// Add a typed `minecraft:suspicious_stew_effects` entry.
    pub fn suspicious_stew_effect(self, effect: SuspiciousStewEffect) -> Self {
        self.component(ItemComponent::suspicious_stew_effect(effect))
    }

    /// Add typed `minecraft:suspicious_stew_effects` entries.
    pub fn suspicious_stew_effects(self, effects: Vec<SuspiciousStewEffect>) -> Self {
        self.component(ItemComponent::suspicious_stew_effects(effects))
    }

    // ── Escape hatch ──────────────────────────────────────────────────────────

    /// Add a raw item component from an explicit [`RawComponent`] value
    /// (for features not covered by the typed API).
    ///
    /// The escape hatch is visible at the construction site: the component's
    /// `key=snbt_value` is appended verbatim to the component string.
    pub fn with_raw_component(self, component: RawComponent) -> Self {
        self.component(ItemComponent::raw_component(component))
    }

    // ── Item predicate ────────────────────────────────────────────────────────

    /// Build a Minecraft item predicate JSON for matching this item.
    ///
    /// Matches the base item type and, if a [`custom_data`](Self::custom_data) key was set,
    /// also matches the `minecraft:custom_data` component.
    ///
    /// Use the result in advancement criteria, loot table conditions, or predicates.
    /// The base Minecraft item ID this custom item is built on
    /// (e.g. `"minecraft:white_wool"`), ignoring components.
    pub fn base_id(&self) -> &str {
        &self.base
    }

    pub fn item_predicate(&self) -> TypedItemPredicate {
        let mut pred = TypedItemPredicate::id(&self.base);
        if let Some(key) = self.custom_data.as_ref().and_then(CustomData::marker_key) {
            pred = pred.custom_data_key(key);
        }
        pred
    }

    // ── Advancement helpers ───────────────────────────────────────────────────

    /// Build an advancement that fires when the player right-clicks with this item.
    ///
    /// The advancement uses the `UsingItem` trigger and calls `reward_fn` as its reward.
    /// Register the result with `#[component]`.
    pub fn on_use_advancement(
        &self,
        location: ResourceLocation,
        reward_fn: impl Into<String>,
    ) -> Advancement {
        Advancement::new(location)
            .criterion(
                "used",
                Criterion::new(AdvancementTrigger::UsingItem {
                    item: Some(self.item_predicate()),
                }),
            )
            .rewards(AdvancementRewards::new().function(reward_fn))
    }

    /// Build an advancement that fires when the player kills an entity.
    ///
    /// Note: Minecraft's `PlayerKilledEntity` trigger does not filter by held item.
    /// Verify the mainhand in the reward function using
    /// `execute if items entity @s weapon.mainhand ...`.
    pub fn on_kill_advancement(
        &self,
        location: ResourceLocation,
        reward_fn: impl Into<String>,
    ) -> Advancement {
        Advancement::new(location)
            .criterion(
                "killed",
                Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                    entity: None,
                    killing_blow: None,
                }),
            )
            .rewards(AdvancementRewards::new().function(reward_fn))
    }

    /// Build an advancement with a custom trigger.
    ///
    /// Use this for item interactions not covered by the other helper methods.
    pub fn custom_trigger_advancement(
        &self,
        location: ResourceLocation,
        trigger: AdvancementTrigger,
        reward_fn: impl Into<String>,
    ) -> Advancement {
        Advancement::new(location)
            .criterion("triggered", Criterion::new(trigger))
            .rewards(AdvancementRewards::new().function(reward_fn))
    }

    // ── Validation (#148) ─────────────────────────────────────────────────────

    /// Validate numeric invariants and string/resource-id shape before this
    /// item is formatted into command text.
    ///
    /// `Display`/`Into<String>` remain fully infallible (see their docs) for
    /// backward compatibility, so command-facing call sites that want a Sand
    /// diagnostic instead of silently-malformed SNBT should call
    /// [`try_to_string`](Self::try_to_string) — or this method directly —
    /// before formatting. [`stack_components`](Self::stack_components) also
    /// calls this internally.
    ///
    /// Checks:
    /// - `max_stack_size` is in `1..=99` (vanilla's item stack limit);
    /// - `max_damage`/`damage`/`repair_cost` are non-negative, and
    ///   `damage <= max_damage` when both are set;
    /// - enchantment/stored-enchantment levels are non-zero, and their raw
    ///   string ids (from [`CustomItem::enchantment`]/
    ///   [`CustomItem::stored_enchantment`]) are non-empty and don't contain
    ///   `"`/`\` (which would break the emitted SNBT string);
    /// - `AttributeModifier::amount` is finite, and its `id`/custom
    ///   [`AttributeType::Custom`] string are non-empty and quote/backslash-free;
    /// - `ConsumableProperties::consume_seconds` is finite and non-negative,
    ///   and its `sound` is non-empty and quote/backslash-free;
    /// - `EquippableProperties::{equip_sound,model,allowed_entities}` are
    ///   non-empty and quote/backslash-free;
    /// - `ToolRule::blocks` is non-empty and quote/backslash-free, and
    ///   `ToolRule::speed`/`ToolProperties::default_mining_speed` are finite;
    /// - `use_cooldown` is finite and non-negative;
    /// - `FoodProperties::saturation` is finite;
    /// - `CustomData::Marker` keys are non-empty (empty keys would emit
    ///   invalid SNBT `{:1b}`);
    /// - `PotionContents::custom_color` is a 24-bit RGB value (`0x000000..=0xFFFFFF`).
    pub fn validate(&self) -> SandResult<()> {
        let err = |key: &str, message: &str| item_component_error(&self.base, key, message);
        let finite = |key: &str, v: f32| -> SandResult<()> {
            if v.is_finite() {
                Ok(())
            } else {
                Err(err(key, &format!("must be a finite number, got `{v}`")))
            }
        };
        // `AttributeModifier::amount` is stored and formatted as `f64` (see
        // `AttributeModifier::to_snbt`'s `{amount}d` suffix) — validating it
        // via an `as f32` cast would spuriously reject a finite f64 value
        // outside f32's range (e.g. 1e300) as "non-finite", since the cast
        // itself produces `f32::INFINITY`.
        let finite_f64 = |key: &str, v: f64| -> SandResult<()> {
            if v.is_finite() {
                Ok(())
            } else {
                Err(err(key, &format!("must be a finite number, got `{v}`")))
            }
        };
        let snbt_safe_string = |key: &str, s: &str| -> SandResult<()> {
            if s.is_empty() {
                return Err(err(key, "must not be empty"));
            }
            if s.contains('"') || s.contains('\\') {
                return Err(err(
                    key,
                    &format!(
                        "must not contain `\"` or `\\` (unescaped in emitted SNBT), got `{s}`"
                    ),
                ));
            }
            Ok(())
        };

        if let Some(size) = self.max_stack_size
            && !(1..=99).contains(&size)
        {
            return Err(err(
                "max_stack_size",
                &format!("must be in 1..=99 (vanilla's item stack limit), got {size}"),
            ));
        }
        if let Some(max_damage) = self.max_damage
            && max_damage < 0
        {
            return Err(err(
                "max_damage",
                &format!("must be non-negative, got {max_damage}"),
            ));
        }
        if let Some(damage) = self.damage {
            if damage < 0 {
                return Err(err(
                    "damage",
                    &format!("must be non-negative, got {damage}"),
                ));
            }
            if let Some(max_damage) = self.max_damage
                && damage > max_damage
            {
                return Err(err(
                    "damage",
                    &format!("must not exceed max_damage ({max_damage}), got {damage}"),
                ));
            }
        }
        if let Some(cost) = self.repair_cost
            && cost < 0
        {
            return Err(err(
                "repair_cost",
                &format!("must be non-negative, got {cost}"),
            ));
        }

        for (id, level) in self.enchantments.iter().chain(&self.stored_enchantments) {
            snbt_safe_string("enchantments[id]", id)?;
            if *level == 0 {
                return Err(err(
                    "enchantments[level]",
                    &format!("level must be non-zero, got 0 for `{id}`"),
                ));
            }
        }

        for modifier in &self.attribute_modifiers {
            finite_f64("attribute_modifiers[amount]", modifier.amount)?;
            if let Some(ref id) = modifier.id {
                snbt_safe_string("attribute_modifiers[id]", id)?;
            }
            if let AttributeType::Custom(ref custom) = modifier.attribute {
                snbt_safe_string("attribute_modifiers[attribute]", custom)?;
            }
        }

        if let Some(ref food) = self.food {
            finite("food[saturation]", food.saturation)?;
        }

        if let Some(ref consumable) = self.consumable {
            finite("consumable[consume_seconds]", consumable.consume_seconds)?;
            if consumable.consume_seconds < 0.0 {
                return Err(err(
                    "consumable[consume_seconds]",
                    &format!("must be non-negative, got {}", consumable.consume_seconds),
                ));
            }
            if let Some(ref sound) = consumable.sound {
                snbt_safe_string("consumable[sound]", sound)?;
            }
        }

        if let Some(secs) = self.use_cooldown {
            finite("use_cooldown", secs)?;
            if secs < 0.0 {
                return Err(err(
                    "use_cooldown",
                    &format!("must be non-negative, got {secs}"),
                ));
            }
        }

        if let Some(ref tool) = self.tool {
            finite("tool[default_mining_speed]", tool.default_mining_speed)?;
            for rule in &tool.rules {
                snbt_safe_string("tool[rule.blocks]", &rule.blocks)?;
                if let Some(speed) = rule.speed {
                    finite("tool[rule.speed]", speed)?;
                }
            }
        }

        if let Some(ref equippable) = self.equippable {
            if let Some(ref s) = equippable.equip_sound {
                snbt_safe_string("equippable[equip_sound]", s)?;
            }
            if let Some(ref m) = equippable.model {
                snbt_safe_string("equippable[model]", m)?;
            }
            if let Some(ref e) = equippable.allowed_entities {
                snbt_safe_string("equippable[allowed_entities]", e)?;
            }
        }

        if let Some(CustomData::Marker(ref key)) = self.custom_data
            && key.is_empty()
        {
            return Err(err("custom_data", "marker key must not be empty"));
        }

        if let Some(ref contents) = self.potion_contents
            && let Some(color) = contents.custom_color
            && color > 0x00FF_FFFF
        {
            return Err(err(
                "potion_contents[custom_color]",
                &format!("must be a 24-bit RGB value in 0x000000..=0xFFFFFF, got {color:#x}"),
            ));
        }

        Ok(())
    }

    /// Fallible alternative to [`Display`]/[`Into<String>`] — validates this
    /// item (see [`validate`](Self::validate)) before formatting it as an
    /// item-component command-argument string.
    ///
    /// Prefer this over `.to_string()`/`.into()` at command-generation
    /// boundaries (e.g. before [`cmd::give`](crate::component)-style call
    /// sites) so malformed numeric/string state fails with a Sand diagnostic
    /// instead of silently producing command text Minecraft rejects at
    /// dispatch time.
    pub fn try_to_string(&self) -> SandResult<String> {
        self.validate()?;
        Ok(self.to_string())
    }

    // ── Component string generation ───────────────────────────────────────────

    fn collect_components(&self) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();

        // Identity
        if let Some(ref data) = self.custom_data {
            parts.push(format!("custom_data={}", data.to_snbt()));
        }
        if let Some(cmd) = self.custom_model_data {
            // 1.21.4+ format: custom_model_data={floats:[N.0f]}
            parts.push(format!("custom_model_data={{floats:[{cmd}.0f]}}"));
        }

        // Display — 1.21.4+ requires SNBT compound tags for text components,
        // not JSON strings in single quotes.
        if let Some(ref name) = self.custom_name {
            parts.push(format!("custom_name={}", text_to_snbt(name)));
        }
        if let Some(ref name) = self.item_name {
            parts.push(format!("item_name={}", text_to_snbt(name)));
        }
        if !self.lore.is_empty() {
            let lines: Vec<String> = self.lore.iter().map(|l| text_to_snbt(l)).collect();
            parts.push(format!("lore=[{}]", lines.join(",")));
        }
        if let Some(rarity) = self.rarity {
            parts.push(format!("rarity=\"{}\"", rarity.as_str()));
        }
        if let Some(glint) = self.enchantment_glint_override {
            parts.push(format!("enchantment_glint_override={glint}"));
        }
        if self.hide_additional_tooltip {
            parts.push("hide_additional_tooltip={}".to_string());
        }
        if self.hide_tooltip {
            parts.push("hide_tooltip={}".to_string());
        }

        // Stack / durability
        if let Some(size) = self.max_stack_size {
            parts.push(format!("max_stack_size={size}"));
        }
        if let Some(damage) = self.max_damage {
            parts.push(format!("max_damage={damage}"));
        }
        if let Some(damage) = self.damage {
            parts.push(format!("damage={damage}"));
        }
        if let Some(show_tooltip) = self.unbreakable {
            parts.push(format!("unbreakable={{show_in_tooltip:{show_tooltip}}}"));
        }
        if let Some(cost) = self.repair_cost {
            parts.push(format!("repair_cost={cost}"));
        }

        // Minecraft's item-component command syntax stores each enchantment
        // component as a direct id-to-level map. `levels` was part of an older
        // representation and is parsed as an enchantment id by current targets.
        if !self.enchantments.is_empty() {
            let levels: Vec<String> = self
                .enchantments
                .iter()
                .map(|(id, lvl)| format!("\"{id}\":{lvl}"))
                .collect();
            parts.push(format!("enchantments={{{}}}", levels.join(",")));
        }
        if !self.stored_enchantments.is_empty() {
            let levels: Vec<String> = self
                .stored_enchantments
                .iter()
                .map(|(id, lvl)| format!("\"{id}\":{lvl}"))
                .collect();
            parts.push(format!("stored_enchantments={{{}}}", levels.join(",")));
        }

        // Attributes
        if !self.attribute_modifiers.is_empty() {
            let mods: Vec<String> = self
                .attribute_modifiers
                .iter()
                .map(|m| m.to_snbt())
                .collect();
            parts.push(format!("attribute_modifiers=[{}]", mods.join(",")));
        }

        // Behaviour
        if let Some(ref food) = self.food {
            parts.push(format!("food={}", food.to_snbt()));
        }
        if let Some(ref consumable) = self.consumable {
            parts.push(format!("consumable={}", consumable.to_snbt()));
        }
        if let Some(secs) = self.use_cooldown {
            parts.push(format!("use_cooldown={{seconds:{secs}f}}"));
        }
        if let Some(ref tool) = self.tool {
            parts.push(format!("tool={}", tool.to_snbt()));
        }
        if let Some(ref equippable) = self.equippable {
            parts.push(format!("equippable={}", equippable.to_snbt()));
        }
        if self.glider {
            parts.push("glider={}".to_string());
        }
        if self.fire_resistant {
            parts.push("fire_resistant={}".to_string());
        }
        if let Some(color) = self.dyed_color {
            parts.push(format!(
                "dyed_color={{rgb:{},show_in_tooltip:true}}",
                color.to_decimal()
            ));
        }
        if let Some(ref contents) = self.potion_contents {
            parts.push(format!("potion_contents={}", contents.to_snbt()));
        }
        if !self.suspicious_stew_effects.is_empty() {
            let effects = self
                .suspicious_stew_effects
                .iter()
                .map(SuspiciousStewEffect::to_snbt)
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("suspicious_stew_effects=[{effects}]"));
        }

        // Raw extras
        for (key, value) in &self.extra_components {
            parts.push(format!("{key}={value}"));
        }

        parts
    }

    // ── Structured (JSON) components ─────────────────────────────────────────

    /// Build the structured JSON-component view of this item.
    ///
    /// This mirrors [`collect_components`](Self::collect_components) field for
    /// field, but targets Minecraft's structured component JSON schema
    /// (`{"minecraft:key": value}`) instead of SNBT-based command syntax. It
    /// is built directly from this item's typed state — never by parsing
    /// [`Display`](fmt::Display)'s command item-stack string — so recipe
    /// results, predicates, and commands all share one source of truth.
    ///
    /// # Errors
    ///
    /// Returns [`SandError::ComponentValidation`] if:
    /// - [`custom_data`](Self::custom_data) was set via
    ///   [`CustomItem::typed_custom_data`] with [`CustomData::Raw`] — arbitrary
    ///   SNBT has no general SNBT→JSON conversion.
    /// - A raw component (added via [`with_raw_component`](Self::with_raw_component))
    ///   has a key that is not a valid resource location, or a value that does
    ///   not parse as strict JSON (raw component values are SNBT intended for
    ///   command syntax, and are only accepted here when they also happen to
    ///   be valid JSON).
    ///
    /// Never silently drops or corrupts a component — every failure surfaces
    /// as an `Err` naming the item's base ID and the offending component key.
    pub fn stack_components(&self) -> SandResult<ItemStackComponents> {
        self.validate()?;

        let mut out = ItemStackComponents {
            base: self.base.clone(),
            components: Vec::new(),
        };

        // Identity
        if let Some(ref data) = self.custom_data {
            match data.to_json() {
                Some(value) => out.insert("minecraft:custom_data", value),
                None => {
                    return Err(item_component_error(
                        &self.base,
                        "minecraft:custom_data",
                        "raw SNBT custom_data has no general SNBT-to-JSON conversion; \
                         use CustomItem::custom_data(...) (a marker) instead of \
                         typed_custom_data(CustomData::raw(...)) if this item is used \
                         as a recipe result",
                    ));
                }
            }
        }
        if let Some(cmd) = self.custom_model_data {
            out.insert(
                "minecraft:custom_model_data",
                serde_json::json!({ "floats": [cmd as f64] }),
            );
        }

        // Display
        if let Some(ref name) = self.custom_name {
            out.insert("minecraft:custom_name", text_json_value(&self.base, name)?);
        }
        if let Some(ref name) = self.item_name {
            out.insert("minecraft:item_name", text_json_value(&self.base, name)?);
        }
        if !self.lore.is_empty() {
            let lines = self
                .lore
                .iter()
                .map(|l| text_json_value(&self.base, l))
                .collect::<SandResult<Vec<_>>>()?;
            out.insert("minecraft:lore", Value::Array(lines));
        }
        if let Some(rarity) = self.rarity {
            out.insert(
                "minecraft:rarity",
                Value::String(rarity.as_str().to_string()),
            );
        }
        if let Some(glint) = self.enchantment_glint_override {
            out.insert("minecraft:enchantment_glint_override", Value::Bool(glint));
        }
        if self.hide_additional_tooltip {
            out.insert(
                "minecraft:hide_additional_tooltip",
                Value::Object(Map::new()),
            );
        }
        if self.hide_tooltip {
            out.insert("minecraft:hide_tooltip", Value::Object(Map::new()));
        }

        // Stack / durability
        if let Some(size) = self.max_stack_size {
            out.insert("minecraft:max_stack_size", Value::from(size));
        }
        if let Some(damage) = self.max_damage {
            out.insert("minecraft:max_damage", Value::from(damage));
        }
        if let Some(damage) = self.damage {
            out.insert("minecraft:damage", Value::from(damage));
        }
        if let Some(show_tooltip) = self.unbreakable {
            out.insert(
                "minecraft:unbreakable",
                serde_json::json!({ "show_in_tooltip": show_tooltip }),
            );
        }
        if let Some(cost) = self.repair_cost {
            out.insert("minecraft:repair_cost", Value::from(cost));
        }

        // Combat / enchanting — direct id-to-level maps (matches the command
        // syntax representation; see the note in collect_components above).
        if !self.enchantments.is_empty() {
            let mut map = Map::new();
            for (id, lvl) in &self.enchantments {
                map.insert(id.clone(), Value::from(*lvl));
            }
            out.insert("minecraft:enchantments", Value::Object(map));
        }
        if !self.stored_enchantments.is_empty() {
            let mut map = Map::new();
            for (id, lvl) in &self.stored_enchantments {
                map.insert(id.clone(), Value::from(*lvl));
            }
            out.insert("minecraft:stored_enchantments", Value::Object(map));
        }
        if !self.attribute_modifiers.is_empty() {
            let mods: Vec<Value> = self
                .attribute_modifiers
                .iter()
                .map(AttributeModifier::to_json)
                .collect();
            out.insert("minecraft:attribute_modifiers", Value::Array(mods));
        }

        // Behaviour
        if let Some(ref food) = self.food {
            out.insert("minecraft:food", food.to_json());
        }
        if let Some(ref consumable) = self.consumable {
            out.insert("minecraft:consumable", consumable.to_json());
        }
        if let Some(secs) = self.use_cooldown {
            out.insert(
                "minecraft:use_cooldown",
                serde_json::json!({ "seconds": secs }),
            );
        }
        if let Some(ref tool) = self.tool {
            out.insert("minecraft:tool", tool.to_json());
        }
        if let Some(ref equippable) = self.equippable {
            out.insert("minecraft:equippable", equippable.to_json());
        }
        if self.glider {
            out.insert("minecraft:glider", Value::Object(Map::new()));
        }
        if self.fire_resistant {
            out.insert("minecraft:fire_resistant", Value::Object(Map::new()));
        }
        if let Some(color) = self.dyed_color {
            out.insert("minecraft:dyed_color", color.to_json());
        }
        if let Some(ref contents) = self.potion_contents {
            let value = serde_json::to_value(contents).map_err(SandError::from)?;
            out.insert("minecraft:potion_contents", value);
        }
        if !self.suspicious_stew_effects.is_empty() {
            let value =
                serde_json::to_value(&self.suspicious_stew_effects).map_err(SandError::from)?;
            out.insert("minecraft:suspicious_stew_effects", value);
        }

        // Raw extras — only accepted when the key is a valid resource
        // location and the value round-trips as strict JSON. Command-syntax
        // SNBT (unquoted keys, `1b`/`2.0f` suffixes, etc.) is rejected with a
        // clear error rather than silently dropped or mis-encoded.
        for (key, value) in &self.extra_components {
            let full_key = if key.contains(':') {
                key.clone()
            } else {
                format!("minecraft:{key}")
            };
            let (namespace, path) = full_key.split_once(':').expect("':' just verified present");
            if ResourceLocation::new(namespace, path).is_err() {
                return Err(item_component_error(
                    &self.base,
                    &full_key,
                    "raw component key is not a valid resource location",
                ));
            }
            let parsed: Value = serde_json::from_str(value).map_err(|_| {
                item_component_error(
                    &self.base,
                    &full_key,
                    "raw component value is SNBT-only (command item-stack syntax) and \
                     cannot be safely converted to structured recipe/JSON components; \
                     use a typed ItemComponent, or supply a raw value that is also \
                     valid JSON",
                )
            })?;
            out.insert(full_key, parsed);
        }

        Ok(out)
    }

    /// Build a [`recipe::RecipeResult`](crate::recipe::RecipeResult) that produces
    /// this item, preserving its base ID and every data component. `count` must
    /// be at least 1 (validated by the recipe builder before export).
    ///
    /// Equivalent to [`recipe::RecipeResult::from_custom_item`], provided as a
    /// method for callers that already have a `CustomItem` in hand.
    pub fn recipe_result(&self, count: u32) -> SandResult<crate::recipe::RecipeResult> {
        crate::recipe::RecipeResult::from_custom_item(self, count)
    }
}

/// Parse a `CustomItem`-internal pre-serialized text-component JSON string
/// (produced by `TextComponent`'s `Display`, e.g. `self.custom_name`) back
/// into a `serde_json::Value`.
///
/// This is *not* a parse of `CustomItem::to_string()` (the SNBT-based command
/// item-stack string) — each of these fields is already JSON text, stored
/// verbatim from `TextComponent::to_string()`, which serializes via
/// `to_json_value()` before formatting.
fn text_json_value(base: &str, text_json: &str) -> SandResult<Value> {
    serde_json::from_str(text_json).map_err(|_| {
        item_component_error(
            base,
            "text_component",
            "internal text component JSON failed to parse — this indicates a bug \
             in TextComponent's JSON serialization",
        )
    })
}

/// Convert a JSON text component string to an SNBT compound for use in item components.
///
/// In Minecraft 1.21.4+, text components in item NBT are stored as SNBT compound tags
/// rather than JSON strings. `/give ... custom_name='{"text":"..."}' ` no longer works;
/// the value must be `custom_name={text:"..."}`.
fn text_to_snbt(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(v) => json_val_to_snbt(&v),
        Err(_) => format!("{{text:\"{}\"}}", json_str.replace('"', "\\\"")),
    }
}

fn json_val_to_snbt(v: &Value) -> String {
    match v {
        Value::Object(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}:{}", k, json_val_to_snbt(v)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(json_val_to_snbt).collect();
            format!("[{}]", parts.join(","))
        }
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => "\"\"".to_string(),
    }
}

/// Quote an SNBT compound key if it contains characters outside the
/// unquoted-key charset. Shared with `crate::predicates` so item-component
/// and predicate SNBT rendering don't drift on the same quoting rule.
pub(crate) fn snbt_compound_key(key: &str) -> String {
    if key
        .chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.'))
    {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Formats this item as an item-component command-argument string.
///
/// **This does not validate** — see [`CustomItem::validate`]. Malformed
/// numeric/string state (e.g. a non-finite `AttributeModifier::amount`, an
/// empty enchantment id) is formatted as-is, which can produce SNBT
/// Minecraft rejects at command-dispatch time. Prefer
/// [`CustomItem::try_to_string`] at command-generation boundaries; this
/// infallible impl remains available for callers that accept that risk
/// (e.g. already-validated items, or exploratory/test code).
impl fmt::Display for CustomItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let components = self.collect_components();
        if components.is_empty() {
            write!(f, "{}", self.base)
        } else {
            write!(f, "{}[{}]", self.base, components.join(","))
        }
    }
}

/// Allows passing a `CustomItem` directly to [`cmd::give`](crate::cmd) and any
/// other function accepting `impl Into<String>`.
///
/// **This does not validate** — see [`fmt::Display`]'s doc comment above and
/// [`CustomItem::try_to_string`].
impl From<CustomItem> for String {
    fn from(item: CustomItem) -> String {
        item.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sand_commands::{ChatColor, TextComponent};

    #[test]
    fn base_only() {
        let item = CustomItem::new("minecraft:stick");
        assert_eq!(item.to_string(), "minecraft:stick");
    }

    #[test]
    fn custom_data_key() {
        let item = CustomItem::new("minecraft:diamond_sword").custom_data("inferno_blade");
        assert!(item.to_string().contains("custom_data={inferno_blade:1b}"));
    }

    #[test]
    fn custom_model_data() {
        let item = CustomItem::new("minecraft:diamond_sword").custom_model_data(1001);
        assert!(
            item.to_string()
                .contains("custom_model_data={floats:[1001.0f]}")
        );
    }

    #[test]
    fn typed_potion_contents_component() {
        let item = CustomItem::new("minecraft:potion").potion_contents(
            PotionContents::new()
                .potion(crate::PotionId::Swiftness)
                .effect(crate::StatusEffectInstance::new(crate::EffectId::Haste).seconds(5)),
        );
        assert_eq!(
            item.to_string(),
            "minecraft:potion[potion_contents={potion:\"minecraft:swiftness\",custom_effects:[{id:\"minecraft:haste\",duration:100}]}]"
        );
    }

    #[test]
    fn custom_name() {
        let item = CustomItem::new("minecraft:diamond_sword")
            .custom_name(TextComponent::literal("Inferno").color(ChatColor::Red));
        let s = item.to_string();
        assert!(s.contains("custom_name="));
        assert!(s.contains("Inferno"));
        assert!(s.contains("red"));
    }

    #[test]
    fn lore_lines() {
        let item = CustomItem::new("minecraft:stick")
            .lore_line(TextComponent::literal("Line 1"))
            .lore_line(TextComponent::literal("Line 2"));
        let s = item.to_string();
        assert!(s.contains("lore=["));
        assert!(s.contains("Line 1"));
        assert!(s.contains("Line 2"));
    }

    #[test]
    fn enchantments() {
        let item = CustomItem::new("minecraft:diamond_sword")
            .enchantment("minecraft:fire_aspect", 2)
            .enchantment("minecraft:sharpness", 5);
        let s = item.to_string();
        assert!(s.contains("enchantments={"));
        assert!(!s.contains("enchantments={levels:"));
        assert!(s.contains("\"minecraft:fire_aspect\":2"));
        assert!(s.contains("\"minecraft:sharpness\":5"));
    }

    #[test]
    fn enchantment_component_is_a_direct_map_in_insertion_order() {
        let item =
            CustomItem::new("minecraft:crossbow").component(ItemComponent::Enchantments(vec![
                EnchantmentEntry::new(EnchantmentId::minecraft("quick_charge").unwrap(), 10),
                EnchantmentEntry::new(EnchantmentId::minecraft("infinity").unwrap(), 1),
            ]));
        assert_eq!(
            item.to_string(),
            "minecraft:crossbow[enchantments={\"minecraft:quick_charge\":10,\"minecraft:infinity\":1}]"
        );
    }

    #[test]
    fn stored_enchantment_component_is_a_direct_map() {
        let item = CustomItem::new("minecraft:enchanted_book").component(
            ItemComponent::StoredEnchantments(vec![EnchantmentEntry::new(
                EnchantmentId::minecraft("sharpness").unwrap(),
                5,
            )]),
        );
        assert_eq!(
            item.to_string(),
            "minecraft:enchanted_book[stored_enchantments={\"minecraft:sharpness\":5}]"
        );
    }

    #[test]
    fn custom_item_enchantment_helpers_use_direct_maps() {
        let string_item =
            CustomItem::new("minecraft:crossbow").enchantment("minecraft:quick_charge", 10);
        let typed_item = CustomItem::new("minecraft:crossbow")
            .typed_enchantment(EnchantmentId::minecraft("quick_charge").unwrap(), 10);
        assert_eq!(string_item.to_string(), typed_item.to_string());
        assert_eq!(
            typed_item.to_string(),
            "minecraft:crossbow[enchantments={\"minecraft:quick_charge\":10}]"
        );
    }

    #[test]
    fn custom_item_stored_enchantment_helpers_use_direct_maps() {
        let string_item = CustomItem::new("minecraft:enchanted_book")
            .stored_enchantment("minecraft:sharpness", 5);
        let typed_item = CustomItem::new("minecraft:enchanted_book")
            .typed_stored_enchantment(EnchantmentId::minecraft("sharpness").unwrap(), 5);
        assert_eq!(string_item.to_string(), typed_item.to_string());
        assert_eq!(
            typed_item.to_string(),
            "minecraft:enchanted_book[stored_enchantments={\"minecraft:sharpness\":5}]"
        );
    }

    #[test]
    fn empty_enchantment_components_are_omitted() {
        let item = CustomItem::new("minecraft:crossbow")
            .component(ItemComponent::Enchantments(vec![]))
            .component(ItemComponent::StoredEnchantments(vec![]));
        assert_eq!(item.to_string(), "minecraft:crossbow");
    }

    #[test]
    fn attribute_modifier() {
        let item = CustomItem::new("minecraft:diamond_sword").attribute(
            AttributeType::AttackDamage,
            8.0,
            AttributeOperation::AddValue,
            EquipmentSlotGroup::Mainhand,
        );
        let s = item.to_string();
        assert!(s.contains("attribute_modifiers=["));
        assert!(s.contains("minecraft:attack_damage"));
        assert!(s.contains("add_value"));
        assert!(s.contains("mainhand"));
    }

    #[test]
    fn unbreakable() {
        let item = CustomItem::new("minecraft:diamond_sword").unbreakable(false);
        assert!(
            item.to_string()
                .contains("unbreakable={show_in_tooltip:false}")
        );
    }

    #[test]
    fn into_string_for_give() {
        let item = CustomItem::new("minecraft:diamond_sword").custom_data("my_sword");
        let s: String = item.into();
        assert!(s.contains("minecraft:diamond_sword"));
        assert!(s.contains("my_sword"));
    }

    #[test]
    fn item_predicate_with_custom_data() {
        let item = CustomItem::new("minecraft:diamond_sword").custom_data("inferno_blade");
        let pred = serde_json::to_value(item.item_predicate()).unwrap();
        assert_eq!(
            pred["items"],
            serde_json::json!(["minecraft:diamond_sword"])
        );
        // Partial-match predicate, not exact `components` equality — see #233.
        assert_eq!(
            pred["predicates"]["minecraft:custom_data"],
            "{inferno_blade:1b}"
        );
        assert!(pred.get("components").is_none());
    }

    #[test]
    fn item_predicate_without_custom_data() {
        let item = CustomItem::new("minecraft:diamond_sword");
        let pred = serde_json::to_value(item.item_predicate()).unwrap();
        assert_eq!(
            pred["items"],
            serde_json::json!(["minecraft:diamond_sword"])
        );
        assert!(pred.get("components").is_none());
        assert!(pred.get("predicates").is_none());
    }

    #[test]
    fn raw_component_escape_hatch() {
        let item = CustomItem::new("minecraft:bow")
            .with_raw_component(RawComponent::new("bundle_contents", "{items:[]}"));
        assert!(item.to_string().contains("bundle_contents={items:[]}"));
    }

    #[test]
    fn food_properties() {
        let item = CustomItem::new("minecraft:apple")
            .food(FoodProperties::new(4, 2.4).can_always_eat(true));
        let s = item.to_string();
        assert!(s.contains("food="));
        assert!(s.contains("nutrition:4"));
        assert!(s.contains("can_always_eat:true"));
    }

    #[test]
    fn dyed_color() {
        let item =
            CustomItem::new("minecraft:leather_chestplate").dyed_color(DyedColor::hex(0xFF5733));
        let s = item.to_string();
        assert!(s.contains("dyed_color="));
        assert!(s.contains("rgb:"));
    }

    // ── stack_components (#226) ──────────────────────────────────────────────

    #[test]
    fn stack_components_base_only_is_component_free() {
        let item = CustomItem::new("minecraft:stick");
        let stack = item.stack_components().unwrap();
        assert_eq!(stack.base_item(), "minecraft:stick");
        assert!(stack.is_component_free());
        assert!(stack.components().is_empty());
    }

    #[test]
    fn stack_components_custom_data_marker_is_structured_json() {
        let item = CustomItem::new("minecraft:white_wool").custom_data("elevator_block_item");
        let stack = item.stack_components().unwrap();
        let (_, value) = stack
            .components()
            .iter()
            .find(|(k, _)| k == "minecraft:custom_data")
            .expect("custom_data component present");
        assert_eq!(value, &serde_json::json!({ "elevator_block_item": true }));
    }

    #[test]
    fn stack_components_raw_snbt_custom_data_errors_instead_of_corrupting() {
        let item = CustomItem::new("minecraft:stick")
            .typed_custom_data(CustomData::raw(RawSnbt::new("{level:3}")));
        let err = item
            .stack_components()
            .expect_err("raw SNBT custom_data has no general SNBT-to-JSON conversion");
        assert!(err.to_string().contains("custom_data"));
    }

    #[test]
    fn stack_components_preserve_item_name_glint_and_custom_model_data() {
        let item = CustomItem::new("minecraft:white_wool")
            .custom_data("elevator_block_item")
            .component(ItemComponent::EnchantmentGlintOverride(true))
            .item_name(
                TextComponent::literal("Elevator Block")
                    .bold(true)
                    .color(ChatColor::Aqua),
            )
            .custom_model_data(7);
        let stack = item.stack_components().unwrap();
        assert_eq!(stack.base_item(), "minecraft:white_wool");

        let get = |key: &str| {
            stack
                .components()
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            get("minecraft:custom_data"),
            Some(serde_json::json!({ "elevator_block_item": true }))
        );
        assert_eq!(
            get("minecraft:enchantment_glint_override"),
            Some(serde_json::json!(true))
        );
        assert_eq!(
            get("minecraft:custom_model_data"),
            Some(serde_json::json!({ "floats": [7.0] }))
        );
        let item_name = get("minecraft:item_name").expect("item_name present");
        assert_eq!(item_name["text"], "Elevator Block");
        assert_eq!(item_name["bold"], true);
        assert_eq!(item_name["color"], "aqua");
    }

    #[test]
    fn stack_components_raw_component_valid_json_is_accepted() {
        let item = CustomItem::new("minecraft:bow")
            .with_raw_component(RawComponent::new("modded:widget", "{\"a\":1}"));
        let stack = item.stack_components().unwrap();
        let (_, value) = stack
            .components()
            .iter()
            .find(|(k, _)| k == "modded:widget")
            .expect("raw component present under its own namespaced key");
        assert_eq!(value, &serde_json::json!({ "a": 1 }));
    }

    #[test]
    fn stack_components_raw_component_snbt_only_value_errors_clearly() {
        // `{items:[]}` is valid SNBT (unquoted keys) but not valid JSON — this
        // must fail loudly rather than being silently dropped or mis-encoded.
        let item = CustomItem::new("minecraft:bow")
            .with_raw_component(RawComponent::new("bundle_contents", "{items:[]}"));
        let err = item
            .stack_components()
            .expect_err("SNBT-only raw component values must not silently convert");
        assert!(err.to_string().contains("bundle_contents"));
    }

    #[test]
    fn recipe_result_method_matches_free_function_conversion() {
        let elevator = CustomItem::new("minecraft:white_wool").custom_data("elevator_block_item");
        let via_method = elevator.recipe_result(4).unwrap();
        let via_free_fn = crate::recipe::RecipeResult::from_custom_item(&elevator, 4).unwrap();
        assert_eq!(
            serde_json::to_value(via_method).unwrap(),
            serde_json::to_value(via_free_fn).unwrap()
        );
    }

    // ── #148: CustomItem::validate() / try_to_string() ─────────────────────────

    #[test]
    fn validate_accepts_the_documented_inferno_blade_example() {
        let item = CustomItem::new("minecraft:diamond_sword")
            .custom_data("inferno_blade")
            .custom_name(TextComponent::literal("Inferno Blade").color(ChatColor::Red))
            .enchantment("minecraft:fire_aspect", 2)
            .attribute(
                AttributeType::AttackDamage,
                10.0,
                AttributeOperation::AddValue,
                EquipmentSlotGroup::Mainhand,
            )
            .custom_model_data(1001)
            .max_stack_size(1)
            .rarity(ItemRarity::Rare);
        assert!(item.validate().is_ok());
        assert!(item.try_to_string().is_ok());
    }

    #[test]
    fn validate_rejects_zero_max_stack_size() {
        let item = CustomItem::new("minecraft:stick").max_stack_size(0);
        let err = item.validate().unwrap_err().to_string();
        assert!(err.contains("max_stack_size"), "{err}");
    }

    #[test]
    fn validate_rejects_max_stack_size_above_vanilla_limit() {
        let item = CustomItem::new("minecraft:stick").max_stack_size(100);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_accepts_max_stack_size_at_vanilla_limit() {
        let item = CustomItem::new("minecraft:stick").max_stack_size(99);
        assert!(item.validate().is_ok());
    }

    #[test]
    fn validate_rejects_negative_max_damage_and_damage() {
        assert!(
            CustomItem::new("minecraft:stick")
                .max_damage(-1)
                .validate()
                .is_err()
        );
        assert!(
            CustomItem::new("minecraft:stick")
                .damage(-1)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn validate_rejects_damage_exceeding_max_damage() {
        let item = CustomItem::new("minecraft:stick").max_damage(10).damage(20);
        let err = item.validate().unwrap_err().to_string();
        assert!(err.contains("damage"), "{err}");
    }

    #[test]
    fn validate_rejects_negative_repair_cost() {
        assert!(
            CustomItem::new("minecraft:stick")
                .repair_cost(-1)
                .validate()
                .is_err()
        );
    }

    #[test]
    fn validate_rejects_zero_enchantment_level() {
        let item = CustomItem::new("minecraft:stick").enchantment("minecraft:sharpness", 0);
        let err = item.validate().unwrap_err().to_string();
        assert!(err.contains("level"), "{err}");
    }

    #[test]
    fn validate_rejects_empty_enchantment_id() {
        let item = CustomItem::new("minecraft:stick").enchantment("", 1);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_enchantment_id_with_embedded_quote() {
        let item = CustomItem::new("minecraft:stick").enchantment("mod:sharp\"ness", 1);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_zero_stored_enchantment_level() {
        let item = CustomItem::new("minecraft:enchanted_book")
            .stored_enchantment("minecraft:sharpness", 0);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_attribute_amount() {
        let item = CustomItem::new("minecraft:stick").attribute(
            AttributeType::AttackDamage,
            f64::NAN,
            AttributeOperation::AddValue,
            EquipmentSlotGroup::Mainhand,
        );
        let err = item.validate().unwrap_err().to_string();
        assert!(err.contains("attribute_modifiers"), "{err}");
    }

    #[test]
    fn validate_accepts_large_but_finite_attribute_amount() {
        // Regression: `amount` is stored/formatted as f64 (`{amount}d` SNBT
        // double); validating it via a lossy `as f32` cast would turn a
        // large-but-finite f64 like 1e100 into f32::INFINITY and wrongly
        // reject it. See #148 review follow-up.
        let item = CustomItem::new("minecraft:stick").attribute(
            AttributeType::AttackDamage,
            1e100,
            AttributeOperation::AddValue,
            EquipmentSlotGroup::Mainhand,
        );
        assert!(item.validate().is_ok());
    }

    #[test]
    fn validate_rejects_attribute_modifier_id_with_embedded_quote() {
        let item = CustomItem::new("minecraft:stick").component(ItemComponent::attribute_modifier(
            AttributeModifier::new(AttributeType::AttackDamage)
                .amount(1.0)
                .id("mod:bad\"id"),
        ));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_custom_attribute_type() {
        let item = CustomItem::new("minecraft:stick").attribute(
            AttributeType::Custom(String::new()),
            1.0,
            AttributeOperation::AddValue,
            EquipmentSlotGroup::Mainhand,
        );
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_consume_seconds() {
        let item =
            CustomItem::new("minecraft:stick").consumable(ConsumableProperties::new(f32::NAN));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_negative_consume_seconds() {
        let item = CustomItem::new("minecraft:stick").consumable(ConsumableProperties::new(-1.0));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_consumable_sound() {
        let item =
            CustomItem::new("minecraft:stick").consumable(ConsumableProperties::new(1.0).sound(""));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_use_cooldown() {
        let item = CustomItem::new("minecraft:stick").use_cooldown(f32::INFINITY);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_negative_use_cooldown() {
        let item = CustomItem::new("minecraft:stick").use_cooldown(-1.0);
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_equippable_strings() {
        let item = CustomItem::new("minecraft:stick")
            .equippable(EquippableProperties::new(EquipmentSlot::Head).equip_sound(""));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_tool_rule_speed() {
        let item = CustomItem::new("minecraft:stick").tool(
            ToolProperties::new()
                .rule(ToolRule::new("#minecraft:pickaxe_mineable").speed(f32::NAN)),
        );
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_tool_rule_blocks() {
        let item =
            CustomItem::new("minecraft:stick").tool(ToolProperties::new().rule(ToolRule::new("")));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_default_mining_speed() {
        let item = CustomItem::new("minecraft:stick")
            .tool(ToolProperties::new().default_mining_speed(f32::NAN));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_non_finite_food_saturation() {
        let item = CustomItem::new("minecraft:stick").food(FoodProperties::new(4, f32::NAN));
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_custom_data_marker_key() {
        let item = CustomItem::new("minecraft:stick").custom_data("");
        assert!(item.validate().is_err());
    }

    #[test]
    fn validate_rejects_custom_color_above_24_bit_range() {
        let item = CustomItem::new("minecraft:potion")
            .potion_contents(PotionContents::new().custom_color(0x0100_0000));
        let err = item.validate().unwrap_err().to_string();
        assert!(err.contains("custom_color"), "{err}");
    }

    #[test]
    fn validate_accepts_custom_color_at_24_bit_max() {
        let item = CustomItem::new("minecraft:potion")
            .potion_contents(PotionContents::new().custom_color(0x00FF_FFFF));
        assert!(item.validate().is_ok());
    }

    #[test]
    fn try_to_string_surfaces_validation_error_instead_of_bad_snbt() {
        let item = CustomItem::new("minecraft:stick").max_stack_size(0);
        let err = item.try_to_string().unwrap_err();
        assert!(err.to_string().contains("max_stack_size"));
    }

    #[test]
    fn stack_components_rejects_invalid_numeric_state_before_recipe_output() {
        // Regression: stack_components() previously only checked raw
        // component/custom_data shape, not the numeric invariants validate()
        // now enforces — an invalid CustomItem used as a recipe result must
        // fail before JSON is written, not silently emit `max_stack_size=0`.
        let item = CustomItem::new("minecraft:stick").max_stack_size(0);
        assert!(item.stack_components().is_err());
    }

    #[test]
    fn display_and_into_string_remain_infallible_for_invalid_state() {
        // Documented raw escape hatch: Display/Into<String> never validate,
        // so this must not panic even though the item is invalid.
        let item = CustomItem::new("minecraft:stick").max_stack_size(0);
        let s: String = item.into();
        assert!(s.contains("max_stack_size=0"));
    }
}
