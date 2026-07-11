//! Typed predicate model — shared across advancements, loot tables, and commands.
//!
//! Every predicate type has an explicit [`RawJson`](crate::raw::RawJson) escape hatch
//! via a `::raw(RawJson)` constructor so modded or unsupported conditions can
//! still be expressed.
//!
//! # Type overview
//!
//! | Type | Used in |
//! |---|---|
//! | [`IntRange`] | Score ranges, item counts, level counts |
//! | [`FloatRange`] | Damage amounts, distances |
//! | [`ItemPredicate`] | Item slots, loot conditions, advancement criteria |
//! | [`EntityPredicate`] | Entity conditions in kill/hurt triggers, loot |
//! | [`LocationPredicate`] | Block/biome/dimension location filters |
//! | [`DamagePredicate`] | Damage amount and type filters |
//! | [`DamageSourcePredicate`] | Who/what caused damage |
//! | [`EffectPredicate`] | Active status effect checks |
//! | [`DistancePredicate`] | Distance from a reference point |
//!
//! # Escape hatches
//!
//! Each predicate type implements a `::raw(RawJson)` constructor and
//! serializes the `RawJson` verbatim.  Use it only when no typed alternative
//! exists.
//!
//! ```rust
//! use sand_components::predicates::EntityPredicate;
//! use sand_components::raw::RawJson;
//! use serde_json::json;
//!
//! // Typed (preferred):
//! let ep = EntityPredicate::type_("minecraft:zombie");
//!
//! // Raw escape hatch (for modded entities or unsupported fields):
//! let raw = EntityPredicate::raw(RawJson::new(json!({"type": "mymod:dragon", "nbt": "{Phase:1b}"})));
//! ```

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use crate::effect::EffectId;
use crate::raw::RawJson;

/// Alias for integer predicate ranges in fluent examples.
pub type Range = IntRange;

// ── IntRange ──────────────────────────────────────────────────────────────────

/// An integer range predicate used in item counts, XP levels, signal strengths, etc.
///
/// Serializes as:
/// - an integer when `min == max`
/// - `{"min": N}` / `{"max": N}` / `{"min": A, "max": B}` otherwise
///
/// # Example
/// ```rust
/// use sand_components::predicates::IntRange;
/// use serde_json::json;
///
/// let r = IntRange::at_least(5);
/// assert_eq!(serde_json::to_value(&r).unwrap(), json!({"min": 5}));
///
/// let exact = IntRange::exact(3);
/// assert_eq!(serde_json::to_value(&exact).unwrap(), json!(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl IntRange {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(format!("{path}: minimum {min} exceeds maximum {max}"));
        }
        Ok(())
    }
    /// Match exactly `n`.
    pub fn exact(n: i64) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// Match at least `min`.
    pub fn at_least(min: i64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    /// Match at most `max`.
    pub fn at_most(max: i64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    /// Match between `min` and `max` (inclusive).
    pub fn between(min: i64, max: i64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl Serialize for IntRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match (self.min, self.max) {
            (Some(a), Some(b)) if a == b => serializer.serialize_i64(a),
            _ => {
                let count = self.min.is_some() as usize + self.max.is_some() as usize;
                let mut map = serializer.serialize_map(Some(count))?;
                if let Some(n) = self.min {
                    map.serialize_entry("min", &n)?;
                }
                if let Some(n) = self.max {
                    map.serialize_entry("max", &n)?;
                }
                map.end()
            }
        }
    }
}

// ── FloatRange ───────────────────────────────────────────────────────────────

/// A floating-point range predicate used in damage amounts, distances, etc.
///
/// Serializes as `{"min": f, "max": f}` (omits unbounded sides).
///
/// # Example
/// ```rust
/// use sand_components::predicates::FloatRange;
/// use serde_json::json;
///
/// let r = FloatRange::at_least(1.5);
/// assert_eq!(serde_json::to_value(&r).unwrap(), json!({"min": 1.5}));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl FloatRange {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        for (name, value) in [("min", self.min), ("max", self.max)] {
            if let Some(value) = value
                && !value.is_finite()
            {
                return Err(format!("{path}.{name}: value must be finite"));
            }
        }
        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(format!("{path}: minimum {min} exceeds maximum {max}"));
        }
        Ok(())
    }
    /// Match at least `min`.
    pub fn at_least(min: f64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    /// Match at most `max`.
    pub fn at_most(max: f64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    /// Match between `min` and `max` (inclusive).
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl Serialize for FloatRange {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.min.is_some() as usize + self.max.is_some() as usize;
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(n) = self.min {
            map.serialize_entry("min", &n)?;
        }
        if let Some(n) = self.max {
            map.serialize_entry("max", &n)?;
        }
        map.end()
    }
}

// ── DistancePredicate ─────────────────────────────────────────────────────────

/// Distance predicate — used in advancement triggers to check how far away something is.
///
/// # Example
/// ```rust
/// use sand_components::predicates::DistancePredicate;
/// let d = DistancePredicate::horizontal_at_most(16.0);
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct DistancePredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute: Option<FloatRange>,
}

impl DistancePredicate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Require horizontal distance to be at most `max` blocks.
    pub fn horizontal_at_most(max: f64) -> Self {
        Self {
            horizontal: Some(FloatRange::at_most(max)),
            ..Default::default()
        }
    }

    /// Require absolute 3D distance to be at most `max` blocks.
    pub fn absolute_at_most(max: f64) -> Self {
        Self {
            absolute: Some(FloatRange::at_most(max)),
            ..Default::default()
        }
    }

    pub fn x(mut self, r: FloatRange) -> Self {
        self.x = Some(r);
        self
    }
    pub fn y(mut self, r: FloatRange) -> Self {
        self.y = Some(r);
        self
    }
    pub fn z(mut self, r: FloatRange) -> Self {
        self.z = Some(r);
        self
    }
    pub fn horizontal(mut self, r: FloatRange) -> Self {
        self.horizontal = Some(r);
        self
    }
    pub fn absolute(mut self, r: FloatRange) -> Self {
        self.absolute = Some(r);
        self
    }
}

// ── EffectPredicate ───────────────────────────────────────────────────────────

/// Checks a single active status effect on an entity.
///
/// # Example
/// ```rust
/// use sand_components::predicates::EffectPredicate;
/// let ep = EffectPredicate::new().amplifier(IntRange::at_least(1));
/// # use sand_components::predicates::IntRange;
/// ```
#[derive(Debug, Clone, Default)]
pub struct EffectPredicate {
    pub effect: Option<EffectId>,
    pub amplifier: Option<IntRange>,
    pub duration: Option<IntRange>,
    pub ambient: Option<bool>,
    pub visible: Option<bool>,
}

impl EffectPredicate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(effect: EffectId) -> Self {
        Self {
            effect: Some(effect),
            ..Default::default()
        }
    }

    pub fn amplifier(mut self, r: IntRange) -> Self {
        self.amplifier = Some(r);
        self
    }
    pub fn duration(mut self, r: IntRange) -> Self {
        self.duration = Some(r);
        self
    }
    pub fn ambient(mut self, v: bool) -> Self {
        self.ambient = Some(v);
        self
    }
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = Some(v);
        self
    }

    fn without_effect(mut self) -> Self {
        self.effect = None;
        self
    }

    fn serialize_fields<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.amplifier.is_some() as usize
            + self.duration.is_some() as usize
            + self.ambient.is_some() as usize
            + self.visible.is_some() as usize;
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(ref v) = self.amplifier {
            map.serialize_entry("amplifier", v)?;
        }
        if let Some(ref v) = self.duration {
            map.serialize_entry("duration", v)?;
        }
        if let Some(ref v) = self.ambient {
            map.serialize_entry("ambient", v)?;
        }
        if let Some(ref v) = self.visible {
            map.serialize_entry("visible", v)?;
        }
        map.end()
    }
}

impl Serialize for EffectPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref effect) = self.effect {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry(&effect.to_string(), &self.clone().without_effect())?;
            map.end()
        } else {
            self.serialize_fields(serializer)
        }
    }
}

// ── DamageSourcePredicate ─────────────────────────────────────────────────────

/// Describes what caused damage — used inside [`DamagePredicate`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct DamageSourcePredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_explosion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_fire: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_magic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_projectile: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_lightning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypasses_armor: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypasses_invulnerability: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypasses_magic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_entity: Option<Box<EntityPredicate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct_entity: Option<Box<EntityPredicate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<DamageTagEntry>>,
}

/// A tag membership check used in [`DamageSourcePredicate::tags`].
#[derive(Debug, Clone, Serialize)]
pub struct DamageTagEntry {
    pub id: String,
    pub expected: bool,
}

impl DamageTagEntry {
    pub fn is(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            expected: true,
        }
    }

    pub fn is_not(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            expected: false,
        }
    }
}

impl DamageSourcePredicate {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_explosion(mut self, v: bool) -> Self {
        self.is_explosion = Some(v);
        self
    }
    pub fn is_fire(mut self, v: bool) -> Self {
        self.is_fire = Some(v);
        self
    }
    pub fn is_magic(mut self, v: bool) -> Self {
        self.is_magic = Some(v);
        self
    }
    pub fn is_projectile(mut self, v: bool) -> Self {
        self.is_projectile = Some(v);
        self
    }
    pub fn is_lightning(mut self, v: bool) -> Self {
        self.is_lightning = Some(v);
        self
    }
    pub fn bypasses_armor(mut self, v: bool) -> Self {
        self.bypasses_armor = Some(v);
        self
    }
    pub fn tag(mut self, entry: DamageTagEntry) -> Self {
        self.tags.get_or_insert_with(Vec::new).push(entry);
        self
    }
    pub fn source_entity(mut self, ep: EntityPredicate) -> Self {
        self.source_entity = Some(Box::new(ep));
        self
    }
    pub fn direct_entity(mut self, ep: EntityPredicate) -> Self {
        self.direct_entity = Some(Box::new(ep));
        self
    }
}

// ── DamagePredicate ───────────────────────────────────────────────────────────

/// Checks properties of a damage event — used in `PlayerHurtEntity`,
/// `EntityHurtPlayer`, and `PlayerKilledEntity` triggers.
///
/// # Example
/// ```rust
/// use sand_components::predicates::{DamagePredicate, FloatRange};
///
/// let dp = DamagePredicate::new()
///     .dealt(FloatRange::at_least(5.0))
///     .blocked(false);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DamagePredicate {
    pub dealt: Option<FloatRange>,
    pub taken: Option<FloatRange>,
    pub blocked: Option<bool>,
    pub source_entity: Option<EntityPredicate>,
    pub type_: Option<DamageSourcePredicate>,
    _raw: Option<RawJson>,
}

impl DamagePredicate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON as this predicate.
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    pub fn dealt(mut self, r: FloatRange) -> Self {
        self.dealt = Some(r);
        self
    }
    pub fn taken(mut self, r: FloatRange) -> Self {
        self.taken = Some(r);
        self
    }
    pub fn blocked(mut self, v: bool) -> Self {
        self.blocked = Some(v);
        self
    }
    pub fn source_entity(mut self, ep: EntityPredicate) -> Self {
        self.source_entity = Some(ep);
        self
    }
    pub fn type_(mut self, dsp: DamageSourcePredicate) -> Self {
        self.type_ = Some(dsp);
        self
    }
}

impl Serialize for DamagePredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.dealt {
            map.serialize_entry("dealt", v)?;
        }
        if let Some(ref v) = self.taken {
            map.serialize_entry("taken", v)?;
        }
        if let Some(v) = self.blocked {
            map.serialize_entry("blocked", &v)?;
        }
        if let Some(ref v) = self.source_entity {
            map.serialize_entry("source_entity", v)?;
        }
        if let Some(ref v) = self.type_ {
            map.serialize_entry("type", v)?;
        }
        map.end()
    }
}

// ── LocationPredicate ─────────────────────────────────────────────────────────

/// Checks location properties — block, biome, dimension, position ranges.
///
/// # Example
/// ```rust
/// use sand_components::predicates::LocationPredicate;
///
/// let lp = LocationPredicate::new()
///     .biome("minecraft:plains")
///     .dimension("minecraft:overworld");
/// ```
#[derive(Debug, Clone, Default)]
pub struct LocationPredicate {
    pub biome: Option<String>,
    pub dimension: Option<String>,
    pub feature: Option<String>,
    pub smokey: Option<bool>,
    pub block: Option<BlockPredicate>,
    pub x: Option<FloatRange>,
    pub y: Option<FloatRange>,
    pub z: Option<FloatRange>,
    _raw: Option<RawJson>,
}

impl LocationPredicate {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        for (name, range) in [("x", &self.x), ("y", &self.y), ("z", &self.z)] {
            if let Some(range) = range {
                range.validate_at(&format!("{path}.{name}"))?;
            }
        }
        if let Some(block) = &self.block {
            block.validate_at(&format!("{path}.block"))?;
        }
        Ok(())
    }
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON as this predicate.
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    pub fn biome(mut self, b: impl Into<String>) -> Self {
        self.biome = Some(b.into());
        self
    }
    pub fn dimension(mut self, d: impl Into<String>) -> Self {
        self.dimension = Some(d.into());
        self
    }
    pub fn feature(mut self, f: impl Into<String>) -> Self {
        self.feature = Some(f.into());
        self
    }
    pub fn smokey(mut self, v: bool) -> Self {
        self.smokey = Some(v);
        self
    }
    pub fn block(mut self, bp: BlockPredicate) -> Self {
        self.block = Some(bp);
        self
    }
    pub fn x(mut self, r: FloatRange) -> Self {
        self.x = Some(r);
        self
    }
    pub fn y(mut self, r: FloatRange) -> Self {
        self.y = Some(r);
        self
    }
    pub fn z(mut self, r: FloatRange) -> Self {
        self.z = Some(r);
        self
    }
}

impl Serialize for LocationPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.biome {
            map.serialize_entry("biome", v)?;
        }
        if let Some(ref v) = self.dimension {
            map.serialize_entry("dimension", v)?;
        }
        if let Some(ref v) = self.feature {
            map.serialize_entry("feature", v)?;
        }
        if let Some(v) = self.smokey {
            map.serialize_entry("smokey", &v)?;
        }
        if let Some(ref v) = self.block {
            map.serialize_entry("block", v)?;
        }
        if let Some(ref v) = self.x {
            map.serialize_entry("x", v)?;
        }
        if let Some(ref v) = self.y {
            map.serialize_entry("y", v)?;
        }
        if let Some(ref v) = self.z {
            map.serialize_entry("z", v)?;
        }
        map.end()
    }
}

// ── BlockPredicate ────────────────────────────────────────────────────────────

/// Checks a block at a specific position.
///
/// # Example
/// ```rust
/// use sand_components::predicates::BlockPredicate;
///
/// let bp = BlockPredicate::new()
///     .blocks(vec!["minecraft:oak_log".to_string(), "minecraft:birch_log".to_string()]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct BlockPredicate {
    pub blocks: Option<Vec<String>>,
    pub tag: Option<String>,
    pub nbt: Option<String>,
    pub state: Option<std::collections::HashMap<String, String>>,
    _raw: Option<RawJson>,
}

impl BlockPredicate {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        if self.blocks.as_ref().is_some_and(Vec::is_empty) {
            return Err(format!("{path}.blocks: matcher list must not be empty"));
        }
        Ok(())
    }
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch.
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    pub fn blocks(mut self, ids: Vec<String>) -> Self {
        self.blocks = Some(ids);
        self
    }
    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.tag = Some(t.into());
        self
    }
    pub fn nbt(mut self, n: impl Into<String>) -> Self {
        self.nbt = Some(n.into());
        self
    }
    pub fn state(mut self, s: std::collections::HashMap<String, String>) -> Self {
        self.state = Some(s);
        self
    }
}

impl Serialize for BlockPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.blocks {
            map.serialize_entry("blocks", v)?;
        }
        if let Some(ref v) = self.tag {
            map.serialize_entry("tag", v)?;
        }
        if let Some(ref v) = self.nbt {
            map.serialize_entry("nbt", v)?;
        }
        if let Some(ref v) = self.state {
            map.serialize_entry("state", v)?;
        }
        map.end()
    }
}

// ── ItemPredicate ─────────────────────────────────────────────────────────────

/// Typed item predicate — used in advancement triggers, loot conditions, and commands.
///
/// All internal `Value` fields from the previous design are now either
/// typed (count, custom_data key) or accessed via explicit [`RawJson`] escape hatches.
///
/// # Example
/// ```rust
/// use sand_components::predicates::ItemPredicate;
/// use sand_components::raw::RawJson;
/// use serde_json::json;
///
/// // Fully typed:
/// let pred = ItemPredicate::id("minecraft:diamond_sword")
///     .count_min(1)
///     .custom_data_key("my_sword");
///
/// // Raw escape hatch for unsupported component predicates:
/// let raw_pred = ItemPredicate::id("minecraft:bow")
///     .raw_predicates(RawJson::new(json!({"minecraft:enchantments": {"levels": {"min": 1}}})));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ItemPredicate {
    pub items: Option<Vec<String>>,
    pub count: Option<IntRange>,
    /// Named custom_data keys that must be truthy (emits as component check).
    custom_data_keys: Vec<String>,
    /// Raw component JSON for unsupported predicates.
    raw_components: Option<RawJson>,
    raw_predicates: Option<RawJson>,
    _raw: Option<RawJson>,
}

impl ItemPredicate {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        if self.items.as_ref().is_some_and(Vec::is_empty) {
            return Err(format!("{path}.items: matcher list must not be empty"));
        }
        if let Some(count) = &self.count {
            count.validate_at(&format!("{path}.count"))?;
        }
        if let Some(raw) = &self.raw_components
            && !raw.as_value().is_object()
        {
            return Err(format!(
                "{path}.components: raw component predicates must be a JSON object"
            ));
        }
        Ok(())
    }
    /// Match any item.
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON verbatim as this predicate.
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    /// Match a specific item ID.
    pub fn id(id: impl Into<String>) -> Self {
        Self::new().item(id)
    }

    /// Add a required item ID (creates an `items` array).
    pub fn item(mut self, id: impl Into<String>) -> Self {
        self.items.get_or_insert_with(Vec::new).push(id.into());
        self
    }

    /// Require at least `min` items in the slot.
    pub fn count_min(mut self, min: i64) -> Self {
        self.count = Some(IntRange::at_least(min));
        self
    }

    /// Require at most `max` items in the slot.
    pub fn count_max(mut self, max: i64) -> Self {
        self.count = Some(IntRange::at_most(max));
        self
    }

    /// Require between `min` and `max` items in the slot.
    pub fn count_range(mut self, min: i64, max: i64) -> Self {
        self.count = Some(IntRange::between(min, max));
        self
    }

    /// Set the count predicate directly.
    pub fn count(mut self, r: IntRange) -> Self {
        self.count = Some(r);
        self
    }

    /// Require a named key in the item's `custom_data` component to be truthy.
    ///
    /// This is the primary way to detect Sand custom items tagged with `.custom_data("key")`.
    pub fn custom_data_key(mut self, key: impl Into<String>) -> Self {
        self.custom_data_keys.push(key.into());
        self
    }

    /// Add raw component predicates as an explicit escape hatch.
    pub fn raw_components(mut self, v: RawJson) -> Self {
        self.raw_components = Some(v);
        self
    }

    /// Add raw sub-predicates as an explicit escape hatch.
    pub fn raw_predicates(mut self, v: RawJson) -> Self {
        self.raw_predicates = Some(v);
        self
    }
}

impl Serialize for ItemPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.items {
            if v.len() == 1 {
                map.serialize_entry("items", &v[0])?;
            } else {
                map.serialize_entry("items", v)?;
            }
        }
        if let Some(ref v) = self.count {
            map.serialize_entry("count", v)?;
        }
        // Build components object from typed data + raw fallback
        let has_custom_data = !self.custom_data_keys.is_empty();
        let has_raw_components = self.raw_components.is_some();
        if has_custom_data || has_raw_components {
            let mut comp_map: serde_json::Map<String, Value> = serde_json::Map::new();
            if has_custom_data {
                let mut cd = serde_json::Map::new();
                for key in &self.custom_data_keys {
                    cd.insert(key.clone(), Value::Bool(true));
                }
                comp_map.insert("minecraft:custom_data".to_string(), Value::Object(cd));
            }
            if let Some(ref raw_c) = self.raw_components
                && let Value::Object(obj) = raw_c.as_value()
            {
                for (k, v) in obj {
                    comp_map.insert(k.clone(), v.clone());
                }
            }
            map.serialize_entry("components", &Value::Object(comp_map))?;
        }
        if let Some(ref v) = self.raw_predicates {
            map.serialize_entry("predicates", v)?;
        }
        map.end()
    }
}

// ── EntityPredicate ───────────────────────────────────────────────────────────

/// Typed entity predicate — used in kill/hurt triggers, loot conditions, and more.
///
/// # Example
/// ```rust
/// use sand_components::predicates::EntityPredicate;
///
/// let ep = EntityPredicate::type_("minecraft:zombie")
///     .nbt("{IsBaby:1b}");
/// ```
#[derive(Debug, Clone, Default)]
pub struct EntityPredicate {
    pub entity_type: Option<EntityTypeMatch>,
    pub nbt: Option<String>,
    pub location: Option<LocationPredicate>,
    pub flags: Option<EntityFlags>,
    pub equipment: Option<EntityEquipment>,
    pub effects: Option<std::collections::BTreeMap<String, EffectPredicate>>,
    _raw: Option<RawJson>,
}

/// How to match entity types — single type or any of a list.
#[derive(Debug, Clone)]
pub enum EntityTypeMatch {
    Single(String),
    AnyOf(Vec<String>),
}

impl Serialize for EntityTypeMatch {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            EntityTypeMatch::Single(t) => s.serialize_str(t),
            EntityTypeMatch::AnyOf(types) => types.serialize(s),
        }
    }
}

/// Boolean entity flags checked in predicates.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EntityFlags {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_on_fire: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sneaking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sprinting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_swimming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_baby: Option<bool>,
}

impl EntityFlags {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_fire(mut self, v: bool) -> Self {
        self.is_on_fire = Some(v);
        self
    }
    pub fn sneaking(mut self, v: bool) -> Self {
        self.is_sneaking = Some(v);
        self
    }
    pub fn sprinting(mut self, v: bool) -> Self {
        self.is_sprinting = Some(v);
        self
    }
    pub fn swimming(mut self, v: bool) -> Self {
        self.is_swimming = Some(v);
        self
    }
    pub fn baby(mut self, v: bool) -> Self {
        self.is_baby = Some(v);
        self
    }
}

/// Equipment slot predicates for entity equipment checks.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EntityEquipment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chest: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feet: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mainhand: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offhand: Option<ItemPredicate>,
}

impl EntityEquipment {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn head(mut self, p: ItemPredicate) -> Self {
        self.head = Some(p);
        self
    }
    pub fn chest(mut self, p: ItemPredicate) -> Self {
        self.chest = Some(p);
        self
    }
    pub fn legs(mut self, p: ItemPredicate) -> Self {
        self.legs = Some(p);
        self
    }
    pub fn feet(mut self, p: ItemPredicate) -> Self {
        self.feet = Some(p);
        self
    }
    pub fn mainhand(mut self, p: ItemPredicate) -> Self {
        self.mainhand = Some(p);
        self
    }
    pub fn offhand(mut self, p: ItemPredicate) -> Self {
        self.offhand = Some(p);
        self
    }
}

impl EntityPredicate {
    pub fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        if matches!(&self.entity_type, Some(EntityTypeMatch::AnyOf(types)) if types.is_empty()) {
            return Err(format!("{path}.type: matcher list must not be empty"));
        }
        if let Some(location) = &self.location {
            location.validate_at(&format!("{path}.location"))?;
        }
        if let Some(equipment) = &self.equipment {
            for (name, item) in [
                ("head", &equipment.head),
                ("chest", &equipment.chest),
                ("legs", &equipment.legs),
                ("feet", &equipment.feet),
                ("mainhand", &equipment.mainhand),
                ("offhand", &equipment.offhand),
            ] {
                if let Some(item) = item {
                    item.validate_at(&format!("{path}.equipment.{name}"))?;
                }
            }
        }
        Ok(())
    }
    /// Match any entity.
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON verbatim as this predicate.
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    /// Match a specific entity type ID.
    pub fn type_(entity_type: impl Into<String>) -> Self {
        Self::new().with_type(entity_type)
    }

    /// Set (or override) the entity type.
    pub fn with_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(EntityTypeMatch::Single(entity_type.into()));
        self
    }

    /// Match any of the given entity type IDs.
    pub fn with_type_any(mut self, types: Vec<String>) -> Self {
        self.entity_type = Some(EntityTypeMatch::AnyOf(types));
        self
    }

    /// Require the entity to match this SNBT string.
    pub fn nbt(mut self, nbt: impl Into<String>) -> Self {
        self.nbt = Some(nbt.into());
        self
    }

    /// Require the entity to be at a location matching this predicate.
    pub fn location(mut self, lp: LocationPredicate) -> Self {
        self.location = Some(lp);
        self
    }

    /// Require specific boolean entity flags.
    pub fn flags(mut self, flags: EntityFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Require the entity to wear/hold specific equipment.
    pub fn equipment(mut self, eq: EntityEquipment) -> Self {
        self.equipment = Some(eq);
        self
    }

    /// Require an active status effect (by effect ID).
    pub fn effect(mut self, effect_id: EffectId, pred: EffectPredicate) -> Self {
        self.effects
            .get_or_insert_with(std::collections::BTreeMap::new)
            .insert(effect_id.to_string(), pred.without_effect());
        self
    }

    /// Require the entity to have the effect named by [`EffectPredicate::has`].
    pub fn effect_predicate(mut self, pred: EffectPredicate) -> Self {
        if let Some(effect) = pred.effect.clone() {
            self.effects
                .get_or_insert_with(std::collections::BTreeMap::new)
                .insert(effect.to_string(), pred.without_effect());
        }
        self
    }
}

impl Serialize for EntityPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.entity_type {
            map.serialize_entry("type", v)?;
        }
        if let Some(ref v) = self.nbt {
            map.serialize_entry("nbt", v)?;
        }
        if let Some(ref v) = self.location {
            map.serialize_entry("location", v)?;
        }
        if let Some(ref v) = self.flags {
            map.serialize_entry("flags", v)?;
        }
        if let Some(ref v) = self.equipment {
            map.serialize_entry("equipment", v)?;
        }
        if let Some(ref v) = self.effects {
            map.serialize_entry("effects", v)?;
        }
        map.end()
    }
}

// ── From impls for use in trigger builders ────────────────────────────────────

impl From<ItemPredicate> for Value {
    fn from(p: ItemPredicate) -> Value {
        p.validate_at("predicate")
            .unwrap_or_else(|e| panic!("predicate validation failed: {e}"));
        serde_json::to_value(p).unwrap_or_else(|e| panic!("predicate serialization failed: {e}"))
    }
}

impl From<EntityPredicate> for Value {
    fn from(p: EntityPredicate) -> Value {
        p.validate_at("predicate")
            .unwrap_or_else(|e| panic!("predicate validation failed: {e}"));
        serde_json::to_value(p).unwrap_or_else(|e| panic!("predicate serialization failed: {e}"))
    }
}

impl From<DamagePredicate> for Value {
    fn from(p: DamagePredicate) -> Value {
        serde_json::to_value(p).unwrap_or_else(|e| panic!("predicate serialization failed: {e}"))
    }
}

impl From<LocationPredicate> for Value {
    fn from(p: LocationPredicate) -> Value {
        p.validate_at("predicate")
            .unwrap_or_else(|e| panic!("predicate validation failed: {e}"));
        serde_json::to_value(p).unwrap_or_else(|e| panic!("predicate serialization failed: {e}"))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn int_range_exact() {
        let r = IntRange::exact(5);
        assert_eq!(serde_json::to_value(r).unwrap(), json!(5));
    }

    #[test]
    fn ranges_reject_inverted_and_non_finite_bounds() {
        assert!(
            IntRange {
                min: None,
                max: None
            }
            .validate_at("count")
            .is_ok()
        );
        assert!(IntRange::exact(-3).validate_at("count").is_ok());
        assert!(IntRange::at_most(4).validate_at("count").is_ok());
        assert!(IntRange::between(2, 1).validate_at("count").is_err());
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(FloatRange::at_least(value).validate_at("distance").is_err());
        }
        assert!(
            FloatRange::between(2.0, 1.0)
                .validate_at("distance")
                .is_err()
        );
        assert!(IntRange::at_least(-2).validate_at("count").is_ok());
    }

    #[test]
    fn nested_predicates_report_their_field_path() {
        let predicate = EntityPredicate::new()
            .location(LocationPredicate::new().x(FloatRange::between(3.0, 1.0)));
        let err = predicate
            .validate_at("criteria.foo.conditions.player")
            .unwrap_err();
        assert!(err.contains("criteria.foo.conditions.player.location.x"));
    }

    #[test]
    fn typed_empty_matchers_and_bad_raw_component_shape_fail() {
        assert!(
            BlockPredicate::new()
                .blocks(vec![])
                .validate_at("block")
                .is_err()
        );
        assert!(
            EntityPredicate::new()
                .with_type_any(vec![])
                .validate_at("entity")
                .is_err()
        );
        assert!(
            ItemPredicate::new()
                .raw_components(RawJson::new(json!("not-an-object")))
                .validate_at("item")
                .is_err()
        );
    }

    #[test]
    fn int_range_at_least() {
        let r = IntRange::at_least(3);
        assert_eq!(serde_json::to_value(r).unwrap(), json!({"min": 3}));
    }

    #[test]
    fn int_range_between() {
        let r = IntRange::between(2, 8);
        assert_eq!(
            serde_json::to_value(r).unwrap(),
            json!({"min": 2, "max": 8})
        );
    }

    #[test]
    fn float_range_at_most() {
        let r = FloatRange::at_most(10.5);
        assert_eq!(serde_json::to_value(r).unwrap(), json!({"max": 10.5}));
    }

    #[test]
    fn item_predicate_id_only() {
        let p = ItemPredicate::id("minecraft:diamond");
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["items"], "minecraft:diamond");
    }

    #[test]
    fn item_predicate_with_count() {
        let p = ItemPredicate::id("minecraft:diamond").count_min(5);
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["count"], json!({"min": 5}));
    }

    #[test]
    fn item_predicate_custom_data_key() {
        let p = ItemPredicate::id("minecraft:diamond_sword").custom_data_key("my_sword");
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["components"]["minecraft:custom_data"]["my_sword"], true);
    }

    #[test]
    fn item_predicate_raw() {
        let raw = ItemPredicate::raw(RawJson::new(
            json!({"items": "minecraft:bow", "tag": "foo"}),
        ));
        let v = serde_json::to_value(&raw).unwrap();
        assert_eq!(v["items"], "minecraft:bow");
    }

    #[test]
    fn entity_predicate_type() {
        let ep = EntityPredicate::type_("minecraft:zombie");
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(v["type"], "minecraft:zombie");
    }

    #[test]
    fn entity_predicate_nbt() {
        let ep = EntityPredicate::type_("minecraft:cow").nbt("{IsBaby:1b}");
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(v["nbt"], "{IsBaby:1b}");
    }

    #[test]
    fn entity_predicate_flags() {
        let ep = EntityPredicate::new().flags(EntityFlags::new().on_fire(true).sneaking(false));
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(v["flags"]["is_on_fire"], true);
        assert_eq!(v["flags"]["is_sneaking"], false);
    }

    #[test]
    fn entity_predicate_equipment() {
        let ep = EntityPredicate::type_("minecraft:player")
            .equipment(EntityEquipment::new().feet(ItemPredicate::id("minecraft:diamond_boots")));
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(v["equipment"]["feet"]["items"], "minecraft:diamond_boots");
    }

    #[test]
    fn entity_predicate_raw() {
        let raw = EntityPredicate::raw(RawJson::new(json!({"type": "mymod:boss"})));
        let v = serde_json::to_value(&raw).unwrap();
        assert_eq!(v["type"], "mymod:boss");
    }

    #[test]
    fn entity_predicate_effects() {
        let ep = EntityPredicate::new().effect(
            EffectId::Speed,
            EffectPredicate::new().amplifier(IntRange::at_least(1)),
        );
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(
            v["effects"]["minecraft:speed"]["amplifier"],
            json!({"min": 1})
        );
    }

    #[test]
    fn effect_predicate_has_vanilla() {
        let pred = EffectPredicate::has(EffectId::Speed)
            .amplifier(Range::exact(1))
            .duration(Range::at_least(200))
            .ambient(false)
            .visible(true);
        assert_eq!(
            serde_json::to_value(&pred).unwrap(),
            json!({
                "minecraft:speed": {
                    "amplifier": 1,
                    "duration": {"min": 200},
                    "ambient": false,
                    "visible": true
                }
            })
        );
    }

    #[test]
    fn effect_predicate_has_custom() {
        let pred = EffectPredicate::has(EffectId::custom("mymod:arcane_burn").unwrap())
            .duration(Range::at_most(100));
        assert_eq!(
            serde_json::to_value(&pred).unwrap(),
            json!({"mymod:arcane_burn": {"duration": {"max": 100}}})
        );
    }

    #[test]
    fn damage_predicate_blocked() {
        let dp = DamagePredicate::new().blocked(false);
        let v = serde_json::to_value(&dp).unwrap();
        assert_eq!(v["blocked"], false);
    }

    #[test]
    fn damage_predicate_dealt() {
        let dp = DamagePredicate::new().dealt(FloatRange::at_least(5.0));
        let v = serde_json::to_value(&dp).unwrap();
        assert_eq!(v["dealt"], json!({"min": 5.0}));
    }

    #[test]
    fn damage_predicate_raw() {
        let raw = DamagePredicate::raw(RawJson::new(json!({"dealt": {"min": 10}})));
        let v = serde_json::to_value(&raw).unwrap();
        assert_eq!(v["dealt"]["min"], 10);
    }

    #[test]
    fn location_predicate_biome_dimension() {
        let lp = LocationPredicate::new()
            .biome("minecraft:plains")
            .dimension("minecraft:overworld");
        let v = serde_json::to_value(&lp).unwrap();
        assert_eq!(v["biome"], "minecraft:plains");
        assert_eq!(v["dimension"], "minecraft:overworld");
    }

    #[test]
    fn distance_predicate_horizontal() {
        let dp = DistancePredicate::horizontal_at_most(16.0);
        let v = serde_json::to_value(&dp).unwrap();
        assert_eq!(v["horizontal"]["max"], 16.0);
    }

    #[test]
    fn block_predicate_tag() {
        let bp = BlockPredicate::new().tag("minecraft:logs");
        let v = serde_json::to_value(&bp).unwrap();
        assert_eq!(v["tag"], "minecraft:logs");
    }

    #[test]
    fn damage_source_predicate_tags() {
        let dsp = DamageSourcePredicate::new()
            .is_fire(true)
            .tag(DamageTagEntry::is("minecraft:is_fire"));
        let v = serde_json::to_value(&dsp).unwrap();
        assert_eq!(v["is_fire"], true);
        assert_eq!(v["tags"][0]["id"], "minecraft:is_fire");
        assert_eq!(v["tags"][0]["expected"], true);
    }
}
