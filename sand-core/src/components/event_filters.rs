//! Typed filter builders for `#[event]` advancement conditions.
//!
//! These types replace raw JSON strings with IDE-completable Rust structs.
//! They all implement [`serde::Serialize`] so the macro serializes them via
//! `serde_json::to_value(...)`.
//!
//! # Slot detection note
//!
//! `InventoryChanged` fires when *any* inventory slot changes. Its `slots`
//! field tracks **slot counts** (how many slots are occupied/full/empty) —
//! not which specific slot changed. You cannot filter "item must be in the
//! feet slot" via this trigger.
//!
//! To detect equipped armor, use a tick-based scoreboard check instead:
//!
//! ```text
//! execute as @a if entity @s[nbt={Inventory:[{Slot:100b,id:"minecraft:leather_boots"}]}]
//!     run function my_pack:on_boots_equip
//! ```
//!
//! Slot IDs: `100b`=feet, `101b`=legs, `102b`=chest, `103b`=head,
//! `-106b`=offhand. Hotbar slots 0–8, inventory slots 9–35.

use serde::Serialize;
use serde_json::Value;

// ── ItemPredicate ──────────────────────────────────────────────────────────────

/// A Minecraft item predicate for use in `#[event]` filters.
///
/// Describes properties that an item must match. Used with
/// `InventoryChanged { items = [...] }`, `ItemUsed`, `ItemConsumed`, etc.
///
/// # Common usage
///
/// ```rust,ignore
/// // Match by item ID
/// ItemPredicate::id("minecraft:leather_boots")
///
/// // Match a custom item by its custom_data tag
/// ItemPredicate::id("minecraft:diamond_sword")
///     .with_custom_data(serde_json::json!({"mana_sword": true}))
///
/// // Match an enchanted item
/// ItemPredicate::id("minecraft:bow")
///     .with_enchantment("minecraft:infinity")
///
/// // Match a stack of at least 5 diamonds
/// ItemPredicate::id("minecraft:diamond").with_count_min(5)
/// ```
#[derive(Serialize, Default, Clone)]
pub struct ItemPredicate {
    /// Item resource location, e.g. `"minecraft:leather_boots"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Stack size condition. Set via [`with_count_min`], [`with_count_max`],
    /// or [`with_count_range`].
    ///
    /// [`with_count_min`]: ItemPredicate::with_count_min
    /// [`with_count_max`]: ItemPredicate::with_count_max
    /// [`with_count_range`]: ItemPredicate::with_count_range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<Value>,

    /// Data components the item must have.
    ///
    /// Populated by [`with_custom_data`], [`with_enchantment`], and
    /// [`with_component`]. Rarely needed directly.
    ///
    /// [`with_custom_data`]: ItemPredicate::with_custom_data
    /// [`with_enchantment`]: ItemPredicate::with_enchantment
    /// [`with_component`]: ItemPredicate::with_component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Value>,

    /// Additional item sub-predicates. Set via [`with_predicates`].
    ///
    /// [`with_predicates`]: ItemPredicate::with_predicates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicates: Option<Value>,
}

impl ItemPredicate {
    /// Create a blank predicate (matches any item).
    pub fn new() -> Self {
        Self::default()
    }

    /// Match a specific item ID.
    ///
    /// ```rust,ignore
    /// ItemPredicate::id("minecraft:leather_boots")
    /// ```
    pub fn id(id: impl Into<String>) -> Self {
        Self::new().with_id(id)
    }

    /// Set (or override) the item ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    // ── Count ─────────────────────────────────────────────────────────────────

    /// Require at least `min` items in the slot.
    pub fn with_count_min(mut self, min: i32) -> Self {
        self.count = Some(serde_json::json!({ "min": min }));
        self
    }

    /// Require at most `max` items in the slot.
    pub fn with_count_max(mut self, max: i32) -> Self {
        self.count = Some(serde_json::json!({ "max": max }));
        self
    }

    /// Require between `min` and `max` items in the slot (inclusive).
    pub fn with_count_range(mut self, min: i32, max: i32) -> Self {
        self.count = Some(serde_json::json!({ "min": min, "max": max }));
        self
    }

    // ── Custom data ───────────────────────────────────────────────────────────

    /// Require the item to have matching `minecraft:custom_data` component
    /// entries.
    ///
    /// Only the keys you specify need to be present — other custom data on the
    /// item is ignored. This is the primary way to detect custom items.
    ///
    /// ```rust,ignore
    /// // Match any item tagged as a "mana_sword"
    /// ItemPredicate::id("minecraft:diamond_sword")
    ///     .with_custom_data(serde_json::json!({"mana_sword": true}))
    ///
    /// // Match a custom item with a string tag
    /// ItemPredicate::new()
    ///     .with_custom_data(serde_json::json!({"sand_item_type": "mana_potion"}))
    /// ```
    pub fn with_custom_data(self, data: Value) -> Self {
        self.with_component("minecraft:custom_data", data)
    }

    // ── Enchantments ──────────────────────────────────────────────────────────

    /// Require the item to have a specific enchantment (any level).
    ///
    /// ```rust,ignore
    /// ItemPredicate::id("minecraft:bow").with_enchantment("minecraft:infinity")
    /// ```
    pub fn with_enchantment(self, enchantment_id: impl Into<String>) -> Self {
        let id = enchantment_id.into();
        let enc = serde_json::json!({ "levels": {}, "enchantments": [id] });
        self.with_component("minecraft:enchantments", enc)
    }

    /// Require the item to have a specific enchantment at minimum `min_level`.
    ///
    /// ```rust,ignore
    /// ItemPredicate::id("minecraft:diamond_sword")
    ///     .with_enchantment_min("minecraft:sharpness", 3)
    /// ```
    pub fn with_enchantment_min(self, enchantment_id: impl Into<String>, min_level: i32) -> Self {
        let id = enchantment_id.into();
        let enc = serde_json::json!({ "levels": { "min": min_level }, "enchantments": [id] });
        self.with_component("minecraft:enchantments", enc)
    }

    // ── Generic component ─────────────────────────────────────────────────────

    /// Require a specific data component with the given key and value.
    ///
    /// Use [`with_custom_data`] for `minecraft:custom_data` and
    /// [`with_enchantment`] for `minecraft:enchantments` — they are more
    /// ergonomic than calling this directly.
    ///
    /// ```rust,ignore
    /// ItemPredicate::new()
    ///     .with_component("minecraft:damage", serde_json::json!({"min": 0, "max": 5}))
    /// ```
    ///
    /// [`with_custom_data`]: ItemPredicate::with_custom_data
    /// [`with_enchantment`]: ItemPredicate::with_enchantment
    pub fn with_component(mut self, key: impl Into<String>, value: Value) -> Self {
        let map = self
            .components
            .get_or_insert_with(|| Value::Object(Default::default()));
        if let Value::Object(m) = map {
            m.insert(key.into(), value);
        }
        self
    }

    // ── Raw predicates ────────────────────────────────────────────────────────

    /// Set raw item sub-predicates JSON (for advanced cases not covered by the
    /// builder methods).
    ///
    /// ```rust,ignore
    /// ItemPredicate::new().with_predicates(serde_json::json!({
    ///     "minecraft:enchantments": {"enchantments": "minecraft:sharpness"}
    /// }))
    /// ```
    pub fn with_predicates(mut self, predicates: Value) -> Self {
        self.predicates = Some(predicates);
        self
    }
}

// ── InventorySlots ─────────────────────────────────────────────────────────────

/// Slot-count conditions for `InventoryChanged { slots = ... }`.
///
/// Controls how many inventory slots must be occupied, full, or empty.
/// This is **not** a slot-position selector — it is a count predicate.
///
/// # What this is NOT
///
/// This does **not** let you say "the item must be in the feet slot". The
/// `inventory_changed` trigger has no per-position slot filter. See the
/// [module-level note](crate::components::event_filters) for armor-slot
/// detection alternatives.
///
/// # Example
///
/// ```rust,ignore
/// // Fire when inventory has at least 1 occupied slot
/// #[event(InventoryChanged {
///     slots = InventorySlots::new().with_occupied_min(1),
///     items = [ItemPredicate::id("minecraft:diamond")],
/// }, revoke = true)]
/// pub fn on_diamond_pickup() { }
/// ```
#[derive(Serialize, Default, Clone)]
pub struct InventorySlots {
    /// How many slots must be occupied (contain any item).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupied: Option<Value>,

    /// How many slots must be completely full (stack at max size).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<Value>,

    /// How many slots must be empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty: Option<Value>,
}

impl InventorySlots {
    pub fn new() -> Self {
        Self::default()
    }

    /// Require at least `min` occupied slots.
    pub fn with_occupied_min(mut self, min: i32) -> Self {
        self.occupied = Some(serde_json::json!({ "min": min }));
        self
    }

    /// Require at most `max` occupied slots.
    pub fn with_occupied_max(mut self, max: i32) -> Self {
        self.occupied = Some(serde_json::json!({ "max": max }));
        self
    }

    /// Require between `min` and `max` occupied slots.
    pub fn with_occupied_range(mut self, min: i32, max: i32) -> Self {
        self.occupied = Some(serde_json::json!({ "min": min, "max": max }));
        self
    }

    /// Require at least `min` empty slots.
    pub fn with_empty_min(mut self, min: i32) -> Self {
        self.empty = Some(serde_json::json!({ "min": min }));
        self
    }

    /// Require at most `max` empty slots.
    pub fn with_empty_max(mut self, max: i32) -> Self {
        self.empty = Some(serde_json::json!({ "max": max }));
        self
    }
}

// ── EntityPredicate ────────────────────────────────────────────────────────────

/// A Minecraft entity predicate for use in `#[event]` filters.
///
/// Used for `entity` and `killing_blow` fields in `Death`, `Kill`,
/// `TamedAnimal`, `SummonedEntity`, and similar events.
///
/// # Examples
///
/// ```rust,ignore
/// // Match any zombie
/// EntityPredicate::type_("minecraft:zombie")
///
/// // Match a skeleton with specific NBT
/// EntityPredicate::type_("minecraft:skeleton").with_nbt("{HasSaddle:1b}")
///
/// // Match a player wearing diamond boots (for Kill { entity = ... })
/// EntityPredicate::new().with_equipment(serde_json::json!({
///     "feet": {"id": "minecraft:diamond_boots"}
/// }))
/// ```
#[derive(Serialize, Default, Clone)]
pub struct EntityPredicate {
    /// Entity type, e.g. `"minecraft:zombie"`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<Value>,

    /// SNBT the entity's data must match, e.g. `"{OnGround:1b}"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbt: Option<String>,

    /// Location predicate the entity must be at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Value>,

    /// Entity flags (is_on_fire, is_sneaking, is_sprinting, is_swimming,
    /// is_baby). Set via [`with_flags`].
    ///
    /// [`with_flags`]: EntityPredicate::with_flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Value>,

    /// Equipment slots. Keys: `head`, `chest`, `legs`, `feet`,
    /// `mainhand`, `offhand`. Values are item predicates (JSON objects).
    /// Set via [`with_equipment`].
    ///
    /// [`with_equipment`]: EntityPredicate::with_equipment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<Value>,

    /// Active potion effects the entity must have.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Value>,
}

impl EntityPredicate {
    /// Create a blank predicate (matches any entity).
    pub fn new() -> Self {
        Self::default()
    }

    /// Match a specific entity type.
    ///
    /// ```rust,ignore
    /// EntityPredicate::type_("minecraft:skeleton")
    /// ```
    pub fn type_(entity_type: impl Into<String>) -> Self {
        Self::new().with_type(entity_type)
    }

    /// Set (or override) the entity type.
    pub fn with_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(Value::String(entity_type.into()));
        self
    }

    /// Match any of the given entity types.
    ///
    /// ```rust,ignore
    /// EntityPredicate::new().with_type_any(&["minecraft:zombie", "minecraft:skeleton"])
    /// ```
    pub fn with_type_any(mut self, types: &[&str]) -> Self {
        self.entity_type = Some(serde_json::json!(types));
        self
    }

    /// Require the entity to match this SNBT string.
    ///
    /// ```rust,ignore
    /// EntityPredicate::type_("minecraft:cow").with_nbt("{IsBaby:1b}")
    /// ```
    pub fn with_nbt(mut self, nbt: impl Into<String>) -> Self {
        self.nbt = Some(nbt.into());
        self
    }

    /// Require the entity to be at a location matching this predicate JSON.
    pub fn with_location(mut self, location: Value) -> Self {
        self.location = Some(location);
        self
    }

    /// Require specific boolean entity flags.
    ///
    /// ```rust,ignore
    /// EntityPredicate::new().with_flags(serde_json::json!({
    ///     "is_on_fire": true,
    ///     "is_sneaking": false,
    /// }))
    /// ```
    pub fn with_flags(mut self, flags: Value) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Require the entity to wear/hold specific equipment.
    ///
    /// Slot keys: `head`, `chest`, `legs`, `feet`, `mainhand`, `offhand`.
    /// Values are item predicate objects (id + optional components/predicates).
    ///
    /// ```rust,ignore
    /// // Kill a player wearing diamond boots
    /// EntityPredicate::type_("minecraft:player").with_equipment(serde_json::json!({
    ///     "feet": {"id": "minecraft:diamond_boots"}
    /// }))
    ///
    /// // Kill a player holding a specific custom item
    /// EntityPredicate::type_("minecraft:player").with_equipment(serde_json::json!({
    ///     "mainhand": {
    ///         "id": "minecraft:diamond_sword",
    ///         "components": {
    ///             "minecraft:custom_data": {"mana_sword": true}
    ///         }
    ///     }
    /// }))
    /// ```
    pub fn with_equipment(mut self, equipment: Value) -> Self {
        self.equipment = Some(equipment);
        self
    }

    /// Require the entity to have specific active effects.
    ///
    /// ```rust,ignore
    /// EntityPredicate::new().with_effects(serde_json::json!({
    ///     "minecraft:speed": {"amplifier": {"min": 1}}
    /// }))
    /// ```
    pub fn with_effects(mut self, effects: Value) -> Self {
        self.effects = Some(effects);
        self
    }
}
