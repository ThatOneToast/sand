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
//! use sand_core::{CustomItem, ItemRarity, AttributeType, AttributeOperation, EquipmentSlotGroup};
//! use sand_core::cmd::{self, Selector, TextComponent, ChatColor};
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
//!     cmd::give(Selector::all_players(), inferno_blade());
//! }
//! ```

use std::fmt;

use serde_json::Value;

use crate::cmd::TextComponent;
use crate::{Advancement, AdvancementRewards, AdvancementTrigger, Criterion, ResourceLocation};

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
    /// Create a new attribute modifier with the given type, amount, operation, and slot.
    pub fn new(
        attribute: AttributeType,
        amount: f64,
        operation: AttributeOperation,
        slot: EquipmentSlotGroup,
    ) -> Self {
        Self {
            attribute,
            amount,
            operation,
            slot,
            id: None,
        }
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
    pub fn sound(mut self, sound: impl Into<String>) -> Self {
        self.sound = Some(sound.into());
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
    pub fn equip_sound(mut self, sound: impl Into<String>) -> Self {
        self.equip_sound = Some(sound.into());
        self
    }
    /// Set a custom model override for this equipped item.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    /// Restrict equipping to entities with a specific tag.
    pub fn allowed_entities(mut self, tag: impl Into<String>) -> Self {
        self.allowed_entities = Some(tag.into());
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
    pub fn new(blocks: impl Into<String>) -> Self {
        Self {
            blocks: blocks.into(),
            speed: None,
            correct_for_drops: None,
        }
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

    fn to_decimal(&self) -> i32 {
        ((self.r as i32) << 16) | ((self.g as i32) << 8) | (self.b as i32)
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
    custom_data_key: Option<String>,
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

    // ── Raw escape hatch ──────────────────────────────────────────────────────
    /// Additional raw `key=snbt_value` components appended verbatim.
    extra_components: Vec<(String, String)>,
}

impl CustomItem {
    /// Create a new custom item from a base Minecraft item ID.
    ///
    /// ```
    /// use sand_core::CustomItem;
    /// let item = CustomItem::new("minecraft:diamond_sword");
    /// assert!(item.to_string().starts_with("minecraft:diamond_sword"));
    /// ```
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            custom_data_key: None,
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
            extra_components: Vec::new(),
        }
    }

    // ── Identity ──────────────────────────────────────────────────────────────

    /// Tag this item with a unique key in `custom_data` (e.g. `"inferno_blade"`).
    ///
    /// Emits `custom_data={inferno_blade:1b}` and enables item-predicate helpers
    /// like [`item_predicate`](Self::item_predicate) and
    /// [`on_use_advancement`](Self::on_use_advancement).
    pub fn custom_data(mut self, key: impl Into<String>) -> Self {
        self.custom_data_key = Some(key.into());
        self
    }

    /// Set `custom_model_data` for pairing with resourcepack model overrides.
    ///
    /// Emits `custom_model_data={floats:[N.0f]}` (1.21.4+ format).
    pub fn custom_model_data(mut self, value: i32) -> Self {
        self.custom_model_data = Some(value);
        self
    }

    // ── Display ───────────────────────────────────────────────────────────────

    /// Set the item's custom display name (not italicized).
    pub fn custom_name(mut self, name: TextComponent) -> Self {
        self.custom_name = Some(name.to_string());
        self
    }

    /// Set the item name component (shown italicized in UI). Use `custom_name` for non-italic text.
    pub fn item_name(mut self, name: TextComponent) -> Self {
        self.item_name = Some(name.to_string());
        self
    }

    /// Add a single lore line to the item.
    pub fn lore_line(mut self, line: TextComponent) -> Self {
        self.lore.push(line.to_string());
        self
    }

    /// Add multiple lore lines at once.
    pub fn lore(mut self, lines: Vec<TextComponent>) -> Self {
        self.lore.extend(lines.into_iter().map(|l| l.to_string()));
        self
    }

    /// Set the rarity level (affects item name color).
    pub fn rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = Some(rarity);
        self
    }

    /// Force or suppress the enchantment glint animation.
    pub fn enchantment_glint_override(mut self, glint: bool) -> Self {
        self.enchantment_glint_override = Some(glint);
        self
    }

    /// Hide the additional tooltip section (enchantments, attributes, etc.).
    pub fn hide_additional_tooltip(mut self) -> Self {
        self.hide_additional_tooltip = true;
        self
    }

    /// Hide the entire item tooltip.
    pub fn hide_tooltip(mut self) -> Self {
        self.hide_tooltip = true;
        self
    }

    // ── Stack / durability ────────────────────────────────────────────────────

    /// Set the maximum stack size for this item.
    pub fn max_stack_size(mut self, size: u32) -> Self {
        self.max_stack_size = Some(size);
        self
    }

    /// Set the maximum durability (creates a damageable item).
    pub fn max_damage(mut self, damage: i32) -> Self {
        self.max_damage = Some(damage);
        self
    }

    /// Mark the item as unbreakable.
    ///
    /// `show_in_tooltip` controls whether "Unbreakable" is shown in the tooltip.
    pub fn unbreakable(mut self, show_in_tooltip: bool) -> Self {
        self.unbreakable = Some(show_in_tooltip);
        self
    }

    /// Set the experience cost to repair this item at an anvil.
    pub fn repair_cost(mut self, cost: i32) -> Self {
        self.repair_cost = Some(cost);
        self
    }

    // ── Combat / enchanting ───────────────────────────────────────────────────

    /// Add an enchantment by resource-location ID and level.
    pub fn enchantment(mut self, id: impl Into<String>, level: u32) -> Self {
        self.enchantments.push((id.into(), level));
        self
    }

    /// Add a stored enchantment (for enchanted books).
    pub fn stored_enchantment(mut self, id: impl Into<String>, level: u32) -> Self {
        self.stored_enchantments.push((id.into(), level));
        self
    }

    /// Add a pre-built [`AttributeModifier`].
    pub fn attribute_modifier(mut self, modifier: AttributeModifier) -> Self {
        self.attribute_modifiers.push(modifier);
        self
    }

    /// Convenience shorthand for the common case of a single attribute modifier.
    pub fn attribute(
        mut self,
        attr: AttributeType,
        amount: f64,
        operation: AttributeOperation,
        slot: EquipmentSlotGroup,
    ) -> Self {
        self.attribute_modifiers
            .push(AttributeModifier::new(attr, amount, operation, slot));
        self
    }

    // ── Behaviour ─────────────────────────────────────────────────────────────

    /// Add food properties to this item (makes it edible).
    pub fn food(mut self, food: FoodProperties) -> Self {
        self.food = Some(food);
        self
    }

    /// Add consumable properties to this item.
    pub fn consumable(mut self, consumable: ConsumableProperties) -> Self {
        self.consumable = Some(consumable);
        self
    }

    /// Set a use cooldown (in seconds) between each use.
    pub fn use_cooldown(mut self, seconds: f32) -> Self {
        self.use_cooldown = Some(seconds);
        self
    }

    /// Add tool properties to this item (makes it a tool/weapon).
    pub fn tool(mut self, tool: ToolProperties) -> Self {
        self.tool = Some(tool);
        self
    }

    /// Make this item equippable in a specific slot.
    pub fn equippable(mut self, equippable: EquippableProperties) -> Self {
        self.equippable = Some(equippable);
        self
    }

    /// Make this item function as a glider (like an elytra).
    pub fn glider(mut self) -> Self {
        self.glider = true;
        self
    }

    /// Mark this item as fire-resistant (won't burn in lava or fire).
    pub fn fire_resistant(mut self) -> Self {
        self.fire_resistant = true;
        self
    }

    /// Set a dye color for this item (for leather armor, etc.).
    pub fn dyed_color(mut self, color: DyedColor) -> Self {
        self.dyed_color = Some(color);
        self
    }

    // ── Escape hatch ──────────────────────────────────────────────────────────

    /// Add a raw item component (for features not covered by the typed API).
    ///
    /// Appends `key=snbt_value` verbatim to the component string.
    pub fn raw_component(mut self, key: impl Into<String>, snbt_value: impl Into<String>) -> Self {
        self.extra_components.push((key.into(), snbt_value.into()));
        self
    }

    // ── Item predicate ────────────────────────────────────────────────────────

    /// Build a Minecraft item predicate JSON for matching this item.
    ///
    /// Matches the base item type and, if a [`custom_data`](Self::custom_data) key was set,
    /// also matches the `minecraft:custom_data` component.
    ///
    /// Use the result in advancement criteria, loot table conditions, or predicates.
    pub fn item_predicate(&self) -> Value {
        let mut pred = serde_json::Map::new();
        pred.insert("items".into(), serde_json::json!([self.base]));
        if let Some(ref key) = self.custom_data_key {
            pred.insert(
                "components".into(),
                serde_json::json!({ "minecraft:custom_data": { key: true } }),
            );
        }
        Value::Object(pred)
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

    // ── Component string generation ───────────────────────────────────────────

    fn collect_components(&self) -> Vec<String> {
        let mut parts: Vec<String> = Vec::new();

        // Identity
        if let Some(ref key) = self.custom_data_key {
            parts.push(format!("custom_data={{{key}:1b}}"));
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
        if let Some(show_tooltip) = self.unbreakable {
            parts.push(format!("unbreakable={{show_in_tooltip:{show_tooltip}}}"));
        }
        if let Some(cost) = self.repair_cost {
            parts.push(format!("repair_cost={cost}"));
        }

        // Enchantments
        if !self.enchantments.is_empty() {
            let levels: Vec<String> = self
                .enchantments
                .iter()
                .map(|(id, lvl)| format!("\"{id}\":{lvl}"))
                .collect();
            parts.push(format!("enchantments={{levels:{{{}}}}}", levels.join(",")));
        }
        if !self.stored_enchantments.is_empty() {
            let levels: Vec<String> = self
                .stored_enchantments
                .iter()
                .map(|(id, lvl)| format!("\"{id}\":{lvl}"))
                .collect();
            parts.push(format!(
                "stored_enchantments={{levels:{{{}}}}}",
                levels.join(",")
            ));
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

        // Raw extras
        for (key, value) in &self.extra_components {
            parts.push(format!("{key}={value}"));
        }

        parts
    }
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
impl From<CustomItem> for String {
    fn from(item: CustomItem) -> String {
        item.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::{ChatColor, TextComponent};

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
        assert!(s.contains("enchantments={levels:{"));
        assert!(s.contains("\"minecraft:fire_aspect\":2"));
        assert!(s.contains("\"minecraft:sharpness\":5"));
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
        let pred = item.item_predicate();
        assert_eq!(pred["items"][0], "minecraft:diamond_sword");
        assert!(
            pred["components"]["minecraft:custom_data"]["inferno_blade"]
                .as_bool()
                .unwrap()
        );
    }

    #[test]
    fn item_predicate_without_custom_data() {
        let item = CustomItem::new("minecraft:diamond_sword");
        let pred = item.item_predicate();
        assert_eq!(pred["items"][0], "minecraft:diamond_sword");
        assert!(pred.get("components").is_none());
    }

    #[test]
    fn raw_component_escape_hatch() {
        let item = CustomItem::new("minecraft:bow").raw_component("bundle_contents", "{items:[]}");
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
}
