//! Typed predicate model — shared across advancements, loot tables, and commands.
//!
//! Every predicate type has an explicit [`RawJson`] escape hatch
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
//! use sand_components::EntityTypeId;
//! use serde_json::json;
//!
//! // Typed (preferred):
//! let ep = EntityPredicate::type_(EntityTypeId::minecraft("zombie")?);
//!
//! // Raw escape hatch (for modded entities or unsupported fields):
//! let raw = EntityPredicate::raw(RawJson::new(json!({"type": "mymod:dragon", "nbt": "{Phase:1b}"})));
//! # Ok::<(), sand_components::SandError>(())
//! ```

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use std::collections::BTreeMap;

use crate::effect::EffectId;
use crate::raw::{RawJson, RawSnbt};
use crate::registry::{BiomeId, BlockId, DamageTypeId, DimensionId, EntityTypeId, ItemId, TagId};

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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::IntRange",
    summary = "Represents an exact, bounded, or one-sided integer predicate range.",
    context = "Counts, durations, amplifiers, and game times share this validated integer range.",
    minecraft = "Serializes an exact integer or an object with min and max members.",
    use_when = ["Constraining discrete Minecraft values"],
    avoid_when = ["A floating-point damage or distance range is required"],
    example = "IntRange::between(1, 5)",
)]
pub struct IntRange {
    min: Option<i64>,
    max: Option<i64>,
}

impl IntRange {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        if let (Some(min), Some(max)) = (self.min, self.max)
            && min > max
        {
            return Err(format!("{path}: minimum {min} exceeds maximum {max}"));
        }
        Ok(())
    }
    /// Match exactly `n`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::IntRange::exact",
        summary = "Matches one exact integer value.",
        context = "Exact integer ranges are compact and avoid repeating equal minimum and maximum bounds.",
        minecraft = "Serializes directly as one integer.",
        use_when = ["Matching one discrete count, duration, or level"],
        avoid_when = ["More than one value should be accepted"],
        params(
            n = "The only accepted value."
        ),
        returns = "An exact IntRange.",
        example = "IntRange::exact(5)",
    )]
    pub fn exact(n: i64) -> Self {
        Self {
            min: Some(n),
            max: Some(n),
        }
    }

    /// Match at least `min`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::IntRange::at_least",
        summary = "Creates a range matching values at or above the bound.",
        context = "IntRange provides a shared typed bound for Minecraft predicate fields.",
        minecraft = "Serializes only the min member of the range object.",
        use_when = ["Expressing a one-sided predicate bound"],
        avoid_when = ["Both a lower and upper bound are required"],
        params(
            min = "The inclusive lower bound."
        ),
        returns = "A one-sided IntRange.",
        example = "IntRange::at_least(5)",
    )]
    pub fn at_least(min: i64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    /// Match at most `max`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::IntRange::at_most",
        summary = "Creates a range matching values at or below the bound.",
        context = "IntRange provides a shared typed bound for Minecraft predicate fields.",
        minecraft = "Serializes only the max member of the range object.",
        use_when = ["Expressing a one-sided predicate bound"],
        avoid_when = ["Both a lower and upper bound are required"],
        params(
            max = "The inclusive upper bound."
        ),
        returns = "A one-sided IntRange.",
        example = "IntRange::at_most(5)",
    )]
    pub fn at_most(max: i64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    /// Match between `min` and `max` (inclusive).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::IntRange::between",
        summary = "Creates a range matching values between two inclusive bounds.",
        context = "IntRange validates bound order before predicate export.",
        minecraft = "Serializes min and max members in the range object.",
        use_when = ["Expressing a closed predicate interval"],
        avoid_when = ["Only one side of the interval is bounded"],
        params(
            min = "The inclusive lower bound.",
            max = "The inclusive upper bound."
        ),
        returns = "A bounded IntRange.",
        example = "IntRange::between(2, 8)",
    )]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::FloatRange",
    summary = "Represents a bounded or one-sided floating-point predicate range.",
    context = "Damage and distance conditions share this range representation and validate finite ordered bounds.",
    minecraft = "Serializes min and max members while omitting unbounded sides.",
    use_when = ["Constraining damage amounts or distances"],
    avoid_when = ["An exact integer count is required"],
    example = "FloatRange::between(1.5, 4.0)",
)]
pub struct FloatRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl FloatRange {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::FloatRange::at_least",
        summary = "Creates a range matching values at or above the bound.",
        context = "FloatRange provides a shared typed bound for Minecraft predicate fields.",
        minecraft = "Serializes only the min member of the range object.",
        use_when = ["Expressing a one-sided predicate bound"],
        avoid_when = ["Both a lower and upper bound are required"],
        params(
            min = "The inclusive lower bound."
        ),
        returns = "A one-sided FloatRange.",
        example = "FloatRange::at_least(5.0)",
    )]
    pub fn at_least(min: f64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    /// Match at most `max`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::FloatRange::at_most",
        summary = "Creates a range matching values at or below the bound.",
        context = "FloatRange provides a shared typed bound for Minecraft predicate fields.",
        minecraft = "Serializes only the max member of the range object.",
        use_when = ["Expressing a one-sided predicate bound"],
        avoid_when = ["Both a lower and upper bound are required"],
        params(
            max = "The inclusive upper bound."
        ),
        returns = "A one-sided FloatRange.",
        example = "FloatRange::at_most(5.0)",
    )]
    pub fn at_most(max: f64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    /// Match between `min` and `max` (inclusive).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::FloatRange::between",
        summary = "Creates a range matching values between two inclusive bounds.",
        context = "FloatRange validates bound order before predicate export.",
        minecraft = "Serializes min and max members in the range object.",
        use_when = ["Expressing a closed predicate interval"],
        avoid_when = ["Only one side of the interval is bounded"],
        params(
            min = "The inclusive lower bound.",
            max = "The inclusive upper bound."
        ),
        returns = "A bounded FloatRange.",
        example = "FloatRange::between(2.0, 8.0)",
    )]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::DistancePredicate",
    aliases = ["sand::prelude::DistancePredicate"],
    summary = "Constrains displacement along axes and by horizontal or absolute distance.",
    context = "Advancement and entity predicates use this model to compare a subject with a context reference point.",
    minecraft = "Serializes vanilla x, y, z, horizontal, and absolute distance ranges.",
    use_when = ["Restricting a trigger by relative distance"],
    avoid_when = ["Selecting entities around a command position"],
    example = "DistancePredicate::horizontal_at_most(16.0)",
)]
pub struct DistancePredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    z: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    horizontal: Option<FloatRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    absolute: Option<FloatRange>,
}

impl DistancePredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        for (name, range) in [
            ("x", &self.x),
            ("y", &self.y),
            ("z", &self.z),
            ("horizontal", &self.horizontal),
            ("absolute", &self.absolute),
        ] {
            if let Some(range) = range {
                range.validate_at(&format!("{path}.{name}"))?;
            }
        }
        Ok(())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::new",
        aliases = ["sand::prelude::DistancePredicate::new"],
        summary = "Creates an unconstrained DistancePredicate.",
        context = "Builder methods add only the DistancePredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty DistancePredicate builder.",
        example = "DistancePredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Require horizontal distance to be at most `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::horizontal_at_most",
        aliases = ["sand::prelude::DistancePredicate::horizontal_at_most"],
        summary = "Caps horizontal displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Sets horizontal to an inclusive maximum.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            max = "Greatest horizontal distance in blocks."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::horizontal_at_most(16.0)",
    )]
    pub fn horizontal_at_most(max: f64) -> Self {
        Self {
            horizontal: Some(FloatRange::at_most(max)),
            ..Default::default()
        }
    }

    /// Require absolute 3D distance to be at most `max` blocks.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::absolute_at_most",
        aliases = ["sand::prelude::DistancePredicate::absolute_at_most"],
        summary = "Caps three-dimensional displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Sets absolute to an inclusive maximum.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            max = "Greatest absolute distance in blocks."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::absolute_at_most(16.0)",
    )]
    pub fn absolute_at_most(max: f64) -> Self {
        Self {
            absolute: Some(FloatRange::at_most(max)),
            ..Default::default()
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::x",
        aliases = ["sand::prelude::DistancePredicate::x"],
        summary = "Constrains x-axis displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the x range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted x-axis distance range."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::new().x(FloatRange::at_most(8.0))",
    )]
    pub fn x(mut self, r: FloatRange) -> Self {
        self.x = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::y",
        aliases = ["sand::prelude::DistancePredicate::y"],
        summary = "Constrains y-axis displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the y range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted y-axis distance range."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::new().y(FloatRange::at_most(8.0))",
    )]
    pub fn y(mut self, r: FloatRange) -> Self {
        self.y = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::z",
        aliases = ["sand::prelude::DistancePredicate::z"],
        summary = "Constrains z-axis displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the z range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted z-axis distance range."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::new().z(FloatRange::at_most(8.0))",
    )]
    pub fn z(mut self, r: FloatRange) -> Self {
        self.z = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::horizontal",
        aliases = ["sand::prelude::DistancePredicate::horizontal"],
        summary = "Constrains horizontal displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the horizontal range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted horizontal distance range."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::new().horizontal(FloatRange::at_most(8.0))",
    )]
    pub fn horizontal(mut self, r: FloatRange) -> Self {
        self.horizontal = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DistancePredicate::absolute",
        aliases = ["sand::prelude::DistancePredicate::absolute"],
        summary = "Constrains three-dimensional displacement.",
        context = "Adds one typed DistancePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the absolute range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted three-dimensional distance range."
        ),
        returns = "The updated DistancePredicate predicate.",
        example = "DistancePredicate::new().absolute(FloatRange::at_most(8.0))",
    )]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::EffectPredicate",
    summary = "Constrains one active status effect's amplifier, duration, and display flags.",
    context = "Entity predicates map an EffectId to this value to describe the required live effect state.",
    minecraft = "Serializes one entry in the vanilla effects predicate object.",
    use_when = ["Matching the strength or duration of an active effect"],
    avoid_when = ["Applying or removing an effect"],
    example = "EffectPredicate::new().amplifier(IntRange::at_least(1))",
)]
pub struct EffectPredicate {
    amplifier: Option<IntRange>,
    duration: Option<IntRange>,
    ambient: Option<bool>,
    visible: Option<bool>,
}

impl EffectPredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        if let Some(range) = &self.amplifier {
            range.validate_at(&format!("{path}.amplifier"))?;
        }
        if let Some(range) = &self.duration {
            range.validate_at(&format!("{path}.duration"))?;
        }
        Ok(())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EffectPredicate::new",
        summary = "Creates an unconstrained EffectPredicate.",
        context = "Builder methods add only the EffectPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty EffectPredicate builder.",
        example = "EffectPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EffectPredicate::amplifier",
        summary = "Constrains the effect amplifier.",
        context = "Adds one typed EffectPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the amplifier requirement.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Required effect amplifier."
        ),
        returns = "The updated EffectPredicate predicate.",
        example = "EffectPredicate::new().amplifier(IntRange::at_least(1))",
    )]
    pub fn amplifier(mut self, r: IntRange) -> Self {
        self.amplifier = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EffectPredicate::duration",
        summary = "Constrains the remaining duration.",
        context = "Adds one typed EffectPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the duration requirement.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Required remaining duration."
        ),
        returns = "The updated EffectPredicate predicate.",
        example = "EffectPredicate::new().duration(IntRange::at_least(1))",
    )]
    pub fn duration(mut self, r: IntRange) -> Self {
        self.duration = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EffectPredicate::ambient",
        summary = "Constrains the ambient state.",
        context = "Adds one typed EffectPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the ambient requirement.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required ambient state."
        ),
        returns = "The updated EffectPredicate predicate.",
        example = "EffectPredicate::new().ambient(true)",
    )]
    pub fn ambient(mut self, v: bool) -> Self {
        self.ambient = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EffectPredicate::visible",
        summary = "Constrains the particle visibility.",
        context = "Adds one typed EffectPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the visible requirement.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required particle visibility."
        ),
        returns = "The updated EffectPredicate predicate.",
        example = "EffectPredicate::new().visible(true)",
    )]
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = Some(v);
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
        self.serialize_fields(serializer)
    }
}

// ── DamageSourcePredicate ─────────────────────────────────────────────────────

/// Describes what caused damage — used inside [`DamagePredicate`].
#[derive(Debug, Clone, Default, Serialize)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::DamageSourcePredicate",
    aliases = ["sand::prelude::DamageSourcePredicate"],
    summary = "Matches the typed cause and participating entities of a damage event.",
    context = "The source model separates damage-type tags, the responsible entity, and the immediate damaging entity.",
    minecraft = "Serializes vanilla damage-source properties nested in a damage predicate.",
    use_when = ["Distinguishing projectile, environmental, or entity-caused damage"],
    avoid_when = ["Issuing a damage command"],
    example = "DamageSourcePredicate::new().requires_tag(tag)",
)]
pub struct DamageSourcePredicate {
    #[serde(skip_serializing_if = "Option::is_none")]
    source_entity: Option<Box<EntityPredicate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direct_entity: Option<Box<EntityPredicate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<DamageTagEntry>>,
}

/// Serialized representation of a typed damage-tag membership check.
#[derive(Debug, Clone, Serialize)]
struct DamageTagEntry {
    id: TagId<DamageTypeId>,
    expected: bool,
}

impl DamageTagEntry {
    fn required(id: TagId<DamageTypeId>) -> Self {
        Self { id, expected: true }
    }

    fn excluded(id: TagId<DamageTypeId>) -> Self {
        Self {
            id,
            expected: false,
        }
    }
}

impl DamageSourcePredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        if let Some(entity) = &self.source_entity {
            entity.validate_at(&format!("{path}.source_entity"))?;
        }
        if let Some(entity) = &self.direct_entity {
            entity.validate_at(&format!("{path}.direct_entity"))?;
        }
        Ok(())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamageSourcePredicate::new",
        aliases = ["sand::prelude::DamageSourcePredicate::new"],
        summary = "Creates an unconstrained DamageSourcePredicate.",
        context = "Builder methods add only the DamageSourcePredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty DamageSourcePredicate builder.",
        example = "DamageSourcePredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }
    /// Require the damage type to belong to `tag`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamageSourcePredicate::requires_tag",
        aliases = ["sand::prelude::DamageSourcePredicate::requires_tag"],
        summary = "Requires a damage-type tag.",
        context = "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.",
        minecraft = "Adds a tag predicate expected to be true.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            tag = "Damage-type tag tested against the event."
        ),
        returns = "The updated DamageSourcePredicate predicate.",
        example = "DamageSourcePredicate::new().requires_tag(tag)",
    )]
    pub fn requires_tag(mut self, tag: TagId<DamageTypeId>) -> Self {
        self.tags
            .get_or_insert_with(Vec::new)
            .push(DamageTagEntry::required(tag));
        self
    }
    /// Require the damage type not to belong to `tag`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamageSourcePredicate::excludes_tag",
        aliases = ["sand::prelude::DamageSourcePredicate::excludes_tag"],
        summary = "Excludes a damage-type tag.",
        context = "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.",
        minecraft = "Adds a tag predicate expected to be false.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            tag = "Damage-type tag tested against the event."
        ),
        returns = "The updated DamageSourcePredicate predicate.",
        example = "DamageSourcePredicate::new().excludes_tag(tag)",
    )]
    pub fn excludes_tag(mut self, tag: TagId<DamageTypeId>) -> Self {
        self.tags
            .get_or_insert_with(Vec::new)
            .push(DamageTagEntry::excluded(tag));
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamageSourcePredicate::source_entity",
        aliases = ["sand::prelude::DamageSourcePredicate::source_entity"],
        summary = "Constrains the responsible entity.",
        context = "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.",
        minecraft = "Nests the entity predicate in source_entity.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            ep = "Required properties of the responsible entity."
        ),
        returns = "The updated DamageSourcePredicate predicate.",
        example = "DamageSourcePredicate::new().source_entity(EntityPredicate::new())",
    )]
    pub fn source_entity(mut self, ep: EntityPredicate) -> Self {
        self.source_entity = Some(Box::new(ep));
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamageSourcePredicate::direct_entity",
        aliases = ["sand::prelude::DamageSourcePredicate::direct_entity"],
        summary = "Constrains the immediate damaging entity.",
        context = "Adds one typed DamageSourcePredicate constraint without disturbing its other requirements.",
        minecraft = "Nests the entity predicate in direct_entity.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            ep = "Required properties of the immediate damaging entity."
        ),
        returns = "The updated DamageSourcePredicate predicate.",
        example = "DamageSourcePredicate::new().direct_entity(EntityPredicate::new())",
    )]
    pub fn direct_entity(mut self, ep: EntityPredicate) -> Self {
        self.direct_entity = Some(Box::new(ep));
        self
    }

    pub(crate) fn render_for_advancement(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<Value, String> {
        let mut value = serde_json::to_value(self).map_err(|error| error.to_string())?;
        let object = value
            .as_object_mut()
            .expect("typed damage-source predicates serialize as objects");
        if let Some(entity) = &self.source_entity {
            object.insert("source_entity".into(), entity.render_for_advancement(caps)?);
        }
        if let Some(entity) = &self.direct_entity {
            object.insert("direct_entity".into(), entity.render_for_advancement(caps)?);
        }
        Ok(value)
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::DamagePredicate",
    aliases = ["sand::prelude::DamagePredicate"],
    summary = "Matches the amount, source, and blocking state of a damage event.",
    context = "Damage-sensitive advancements and entity conditions combine dealt and taken amounts with a typed source model.",
    minecraft = "Serializes vanilla damage requirements for damage-related triggers.",
    use_when = ["Constraining a trigger by how damage occurred"],
    avoid_when = ["Applying damage or tracking mutable health"],
    example = "DamagePredicate::new().taken(FloatRange::at_least(4.0))",
)]
pub struct DamagePredicate {
    dealt: Option<FloatRange>,
    taken: Option<FloatRange>,
    blocked: Option<bool>,
    source_entity: Option<EntityPredicate>,
    type_: Option<DamageSourcePredicate>,
    _raw: Option<RawJson>,
}

impl DamagePredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        if let Some(range) = &self.dealt {
            range.validate_at(&format!("{path}.dealt"))?;
        }
        if let Some(range) = &self.taken {
            range.validate_at(&format!("{path}.taken"))?;
        }
        if let Some(entity) = &self.source_entity {
            entity.validate_at(&format!("{path}.source_entity"))?;
        }
        if let Some(source) = &self.type_ {
            source.validate_at(&format!("{path}.type"))?;
        }
        Ok(())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::new",
        aliases = ["sand::prelude::DamagePredicate::new"],
        summary = "Creates an unconstrained DamagePredicate.",
        context = "Builder methods add only the DamagePredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty DamagePredicate builder.",
        example = "DamagePredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON as this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::raw",
        aliases = ["sand::prelude::DamagePredicate::raw"],
        summary = "Creates a DamagePredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw DamagePredicate.",
        example = "DamagePredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::dealt",
        aliases = ["sand::prelude::DamagePredicate::dealt"],
        summary = "Constrains raw damage dealt.",
        context = "Adds one typed DamagePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the dealt range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted range for raw damage dealt."
        ),
        returns = "The updated DamagePredicate predicate.",
        example = "DamagePredicate::new().dealt(FloatRange::at_least(2.0))",
    )]
    pub fn dealt(mut self, r: FloatRange) -> Self {
        self.dealt = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::taken",
        aliases = ["sand::prelude::DamagePredicate::taken"],
        summary = "Constrains damage taken after mitigation.",
        context = "Adds one typed DamagePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the taken range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted range for damage taken after mitigation."
        ),
        returns = "The updated DamagePredicate predicate.",
        example = "DamagePredicate::new().taken(FloatRange::at_least(2.0))",
    )]
    pub fn taken(mut self, r: FloatRange) -> Self {
        self.taken = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::blocked",
        aliases = ["sand::prelude::DamagePredicate::blocked"],
        summary = "Requires a shield-blocking state.",
        context = "Adds one typed DamagePredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the blocked boolean.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Whether the event must be blocked."
        ),
        returns = "The updated DamagePredicate predicate.",
        example = "DamagePredicate::new().blocked(true)",
    )]
    pub fn blocked(mut self, v: bool) -> Self {
        self.blocked = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::source_entity",
        aliases = ["sand::prelude::DamagePredicate::source_entity"],
        summary = "Constrains the responsible entity.",
        context = "Adds one typed DamagePredicate constraint without disturbing its other requirements.",
        minecraft = "Nests source_entity.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            ep = "Required responsible-entity properties."
        ),
        returns = "The updated DamagePredicate predicate.",
        example = "DamagePredicate::new().source_entity(EntityPredicate::new())",
    )]
    pub fn source_entity(mut self, ep: EntityPredicate) -> Self {
        self.source_entity = Some(ep);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::DamagePredicate::type_",
        aliases = ["sand::prelude::DamagePredicate::type_"],
        summary = "Constrains the damage source.",
        context = "Adds one typed DamagePredicate constraint without disturbing its other requirements.",
        minecraft = "Nests the type predicate.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            dsp = "Required damage-source properties."
        ),
        returns = "The updated DamagePredicate predicate.",
        example = "DamagePredicate::new().type_(DamageSourcePredicate::new())",
    )]
    pub fn type_(mut self, dsp: DamageSourcePredicate) -> Self {
        self.type_ = Some(dsp);
        self
    }

    pub(crate) fn render_for_advancement(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<Value, String> {
        if let Some(raw) = &self._raw {
            return Ok(raw.as_value().clone());
        }
        let mut value = serde_json::to_value(self).map_err(|error| error.to_string())?;
        let object = value
            .as_object_mut()
            .expect("typed damage predicates serialize as objects");
        if let Some(entity) = &self.source_entity {
            object.insert("source_entity".into(), entity.render_for_advancement(caps)?);
        }
        if let Some(source) = &self.type_ {
            object.insert("type".into(), source.render_for_advancement(caps)?);
        }
        Ok(value)
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
/// use sand_components::{BiomeId, DimensionId};
/// use sand_components::predicates::LocationPredicate;
///
/// let lp = LocationPredicate::new()
///     .biome(BiomeId::minecraft("plains")?)
///     .dimension(DimensionId::minecraft("overworld")?);
/// # Ok::<(), sand_components::SandError>(())
/// ```
#[derive(Debug, Clone, Default)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::LocationPredicate",
    aliases = ["sand::prelude::LocationPredicate"],
    summary = "Matches biome, dimension, block, smokey state, and world coordinates.",
    context = "World-sensitive entity and standalone predicates compose these location properties in one typed value.",
    minecraft = "Serializes the vanilla location predicate object.",
    use_when = ["Restricting a condition by world position or environment"],
    avoid_when = ["Moving an entity or changing the world"],
    example = "LocationPredicate::new().dimension(DimensionId::minecraft(\"overworld\")?)",
)]
pub struct LocationPredicate {
    biome: Option<BiomeId>,
    dimension: Option<DimensionId>,
    smokey: Option<bool>,
    block: Option<BlockPredicate>,
    x: Option<FloatRange>,
    y: Option<FloatRange>,
    z: Option<FloatRange>,
    _raw: Option<RawJson>,
}

impl LocationPredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::new",
        aliases = ["sand::prelude::LocationPredicate::new"],
        summary = "Creates an unconstrained LocationPredicate.",
        context = "Builder methods add only the LocationPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty LocationPredicate builder.",
        example = "LocationPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON as this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::raw",
        aliases = ["sand::prelude::LocationPredicate::raw"],
        summary = "Creates a LocationPredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw LocationPredicate.",
        example = "LocationPredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::biome",
        aliases = ["sand::prelude::LocationPredicate::biome"],
        summary = "Requires one biome.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the typed biome identifier.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            biome = "Required biome identifier."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().biome(id)",
    )]
    pub fn biome(mut self, biome: BiomeId) -> Self {
        self.biome = Some(biome);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::dimension",
        aliases = ["sand::prelude::LocationPredicate::dimension"],
        summary = "Requires one dimension.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the typed dimension identifier.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            dimension = "Required dimension identifier."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().dimension(id)",
    )]
    pub fn dimension(mut self, dimension: DimensionId) -> Self {
        self.dimension = Some(dimension);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::smokey",
        aliases = ["sand::prelude::LocationPredicate::smokey"],
        summary = "Requires the bee-smokey state.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the smokey boolean.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Whether the position must be smokey."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().smokey(true)",
    )]
    pub fn smokey(mut self, v: bool) -> Self {
        self.smokey = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::block",
        aliases = ["sand::prelude::LocationPredicate::block"],
        summary = "Constrains the block at the position.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Nests a block predicate.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            bp = "Required block properties."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().block(BlockPredicate::new())",
    )]
    pub fn block(mut self, bp: BlockPredicate) -> Self {
        self.block = Some(bp);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::x",
        aliases = ["sand::prelude::LocationPredicate::x"],
        summary = "Constrains the x-coordinate.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes position.x.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted x-coordinate range."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().x(FloatRange::between(0.0, 16.0))",
    )]
    pub fn x(mut self, r: FloatRange) -> Self {
        self.x = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::y",
        aliases = ["sand::prelude::LocationPredicate::y"],
        summary = "Constrains the y-coordinate.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes position.y.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted y-coordinate range."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().y(FloatRange::between(0.0, 16.0))",
    )]
    pub fn y(mut self, r: FloatRange) -> Self {
        self.y = Some(r);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::LocationPredicate::z",
        aliases = ["sand::prelude::LocationPredicate::z"],
        summary = "Constrains the z-coordinate.",
        context = "Adds one typed LocationPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes position.z.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted z-coordinate range."
        ),
        returns = "The updated LocationPredicate predicate.",
        example = "LocationPredicate::new().z(FloatRange::between(0.0, 16.0))",
    )]
    pub fn z(mut self, r: FloatRange) -> Self {
        self.z = Some(r);
        self
    }

    /// True if this predicate has no fields set (matches any location).
    ///
    /// Used by version-aware advancement trigger rendering to decide whether
    /// a `minecraft:location_check` wrapper condition should be emitted at all.
    pub(crate) fn is_empty(&self) -> bool {
        self._raw.is_none()
            && self.biome.is_none()
            && self.dimension.is_none()
            && self.smokey.is_none()
            && self.block.is_none()
            && self.x.is_none()
            && self.y.is_none()
            && self.z.is_none()
    }

    /// True if a `block` sub-predicate is already set (typed or raw).
    pub(crate) fn has_block(&self) -> bool {
        self._raw.is_some() || self.block.is_some()
    }

    pub(crate) fn is_raw(&self) -> bool {
        self._raw.is_some()
    }

    /// Render the location-predicate codec used by advancement consumers.
    ///
    /// The compatibility `Serialize` shape predates the current vanilla codec:
    /// current profiles use `biomes` and nest coordinates under `position`.
    /// Raw predicates remain verbatim and are therefore user-owned.
    pub(crate) fn render_for_advancement(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<Value, String> {
        if let Some(raw) = &self._raw {
            return Ok(raw.as_value().clone());
        }
        if let Some(caps) = caps
            && !caps.is_at_least(1, 21, 4)
        {
            return Err(format!(
                "typed location filters have no verified advancement-predicate lowering for target {}; target Minecraft 1.21.4+ or use LocationPredicate::raw(...) with profile-verified JSON",
                caps.requested_version()
            ));
        }
        let mut object = serde_json::Map::new();
        if let Some(biome) = &self.biome {
            object.insert("biomes".into(), Value::String(biome.to_string()));
        }
        if let Some(dimension) = &self.dimension {
            object.insert("dimension".into(), Value::String(dimension.to_string()));
        }
        if let Some(smokey) = self.smokey {
            object.insert("smokey".into(), Value::Bool(smokey));
        }
        if let Some(block) = &self.block {
            object.insert(
                "block".into(),
                serde_json::to_value(block).map_err(|error| error.to_string())?,
            );
        }
        let mut position = serde_json::Map::new();
        for (name, range) in [("x", &self.x), ("y", &self.y), ("z", &self.z)] {
            if let Some(range) = range {
                position.insert(
                    name.into(),
                    serde_json::to_value(range).map_err(|error| error.to_string())?,
                );
            }
        }
        if !position.is_empty() {
            object.insert("position".into(), Value::Object(position));
        }
        Ok(Value::Object(object))
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
/// use sand_components::{BlockId, predicates::BlockPredicate};
///
/// let bp = BlockPredicate::new()
///     .blocks(vec![BlockId::minecraft("oak_log")?, BlockId::minecraft("birch_log")?]);
/// # Ok::<(), sand_components::SandError>(())
/// ```
#[derive(Debug, Clone, Default)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::BlockPredicate",
    aliases = ["sand::prelude::BlockPredicate"],
    summary = "Matches a block by typed identity, tag, state, or block-entity data.",
    context = "Block conditions in location predicates need one composable description of the block at the tested position.",
    minecraft = "Serializes the vanilla block predicate nested under a location check.",
    use_when = ["Restricting a location by the block occupying it"],
    avoid_when = ["Testing or placing a block through commands"],
    example = "BlockPredicate::new().blocks(vec![BlockId::minecraft(\"stone\")?])",
)]
pub struct BlockPredicate {
    blocks: Option<Vec<BlockId>>,
    tag: Option<TagId<BlockId>>,
    nbt: Option<RawSnbt>,
    state: Option<BTreeMap<String, String>>,
    _raw: Option<RawJson>,
}

impl BlockPredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        if self._raw.is_some() {
            return Ok(());
        }
        if self.blocks.as_ref().is_some_and(Vec::is_empty) {
            return Err(format!("{path}.blocks: matcher list must not be empty"));
        }
        Ok(())
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::new",
        aliases = ["sand::prelude::BlockPredicate::new"],
        summary = "Creates an unconstrained BlockPredicate.",
        context = "Builder methods add only the BlockPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty BlockPredicate builder.",
        example = "BlockPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::raw",
        aliases = ["sand::prelude::BlockPredicate::raw"],
        summary = "Creates a BlockPredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw BlockPredicate.",
        example = "BlockPredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::blocks",
        aliases = ["sand::prelude::BlockPredicate::blocks"],
        summary = "Matches typed block identifiers.",
        context = "Adds one typed BlockPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the blocks array.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            ids = "Accepted block identifiers."
        ),
        returns = "The updated BlockPredicate predicate.",
        example = "BlockPredicate::new().blocks(vec![block])",
    )]
    pub fn blocks(mut self, ids: Vec<BlockId>) -> Self {
        self.blocks = Some(ids);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::tag",
        aliases = ["sand::prelude::BlockPredicate::tag"],
        summary = "Matches a typed block tag.",
        context = "Adds one typed BlockPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the block tag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            tag = "Tag whose members are accepted."
        ),
        returns = "The updated BlockPredicate predicate.",
        example = "BlockPredicate::new().tag(tag)",
    )]
    pub fn tag(mut self, tag: TagId<BlockId>) -> Self {
        self.tag = Some(tag);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::nbt",
        aliases = ["sand::prelude::BlockPredicate::nbt"],
        summary = "Matches block-entity data.",
        context = "Adds one typed BlockPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the SNBT fragment.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            nbt = "Block-entity data that must match."
        ),
        returns = "The updated BlockPredicate predicate.",
        example = "BlockPredicate::new().nbt(nbt)",
    )]
    pub fn nbt(mut self, nbt: RawSnbt) -> Self {
        self.nbt = Some(nbt);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::BlockPredicate::state",
        aliases = ["sand::prelude::BlockPredicate::state"],
        summary = "Matches block-state properties.",
        context = "Adds one typed BlockPredicate constraint without disturbing its other requirements.",
        minecraft = "Writes the state property map.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            state = "Exact property names and values."
        ),
        returns = "The updated BlockPredicate predicate.",
        example = "BlockPredicate::new().state(properties)",
    )]
    pub fn state(mut self, state: BTreeMap<String, String>) -> Self {
        self.state = Some(state);
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
            map.serialize_entry("nbt", v.as_str())?;
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
/// use sand_components::{ItemId, predicates::ItemPredicate};
/// use sand_components::raw::RawJson;
/// use serde_json::json;
///
/// // Fully typed:
/// let pred = ItemPredicate::id(ItemId::minecraft("diamond_sword")?)
///     .count_min(1)
///     .custom_data_key("my_sword");
///
/// // Raw escape hatch for unsupported component predicates:
/// let raw_pred = ItemPredicate::id(ItemId::minecraft("bow")?)
///     .raw_predicates(RawJson::new(json!({"minecraft:enchantments": {"levels": {"min": 1}}})));
/// # Ok::<(), sand_components::SandError>(())
/// ```
#[derive(Debug, Clone, Default)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::ItemPredicate",
    aliases = ["sand::prelude::ItemPredicate", "sand::component::ItemPredicate"],
    summary = "Describes typed properties required of a Minecraft item stack.",
    context = "Item predicates express the item identity, count, components, enchantments, and related constraints consumed by vanilla condition formats.",
    minecraft = "Serializes an item predicate nested in loot, advancement, equipment, or execute-if-items conditions.",
    use_when = ["Matching equipment or inventory contents", "Constraining an item-sensitive trigger"],
    avoid_when = ["Constructing a new item stack rather than matching one"],
    example = "ItemPredicate::new().item(ItemId::minecraft(\"diamond\")?)",
)]
pub struct ItemPredicate {
    items: Option<Vec<ItemId>>,
    count: Option<IntRange>,
    /// Named custom_data keys that must be truthy (emits as component check).
    custom_data_keys: Vec<String>,
    /// Raw component JSON for unsupported predicates.
    raw_components: Option<RawJson>,
    raw_predicates: Option<RawJson>,
    _raw: Option<RawJson>,
}

impl ItemPredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
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
        if let Some(raw) = &self.raw_predicates
            && !raw.as_value().is_object()
        {
            return Err(format!(
                "{path}.predicates: raw sub-predicates must be a JSON object"
            ));
        }
        if !self.custom_data_keys.is_empty()
            && self.raw_predicates.as_ref().is_some_and(|raw| {
                raw.as_value()
                    .as_object()
                    .is_some_and(|object| object.contains_key("minecraft:custom_data"))
            })
        {
            return Err(format!(
                "{path}.predicates.minecraft:custom_data: raw predicate collides with typed custom_data_key filters; combine the partial match in one representation instead of overwriting a requested filter"
            ));
        }
        Ok(())
    }
    /// Match any item.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::new",
        aliases = ["sand::component::ItemPredicate::new", "sand::prelude::ItemPredicate::new"],
        summary = "Creates an unconstrained ItemPredicate.",
        context = "Builder methods add only the ItemPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty ItemPredicate builder.",
        example = "ItemPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON verbatim as this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::raw",
        aliases = ["sand::component::ItemPredicate::raw", "sand::prelude::ItemPredicate::raw"],
        summary = "Creates a ItemPredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw ItemPredicate.",
        example = "ItemPredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    /// Match a specific item ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::id",
        aliases = ["sand::component::ItemPredicate::id","sand::prelude::ItemPredicate::id"],
        summary = "Creates a predicate for one typed item.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Initializes the item identity.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            id = "Item identifier to match."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::id(item)",
    )]
    pub fn id(id: impl Into<ItemId>) -> Self {
        Self::new().item(id)
    }

    /// Add a required item ID (creates an `items` array).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::item",
        aliases = ["sand::component::ItemPredicate::item","sand::prelude::ItemPredicate::item"],
        summary = "Requires one typed item identity.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes the item identifier.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            id = "Item identifier to match."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().item(item)",
    )]
    pub fn item(mut self, id: impl Into<ItemId>) -> Self {
        self.items.get_or_insert_with(Vec::new).push(id.into());
        self
    }

    /// Require at least `min` items in the slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::count_min",
        aliases = ["sand::component::ItemPredicate::count_min","sand::prelude::ItemPredicate::count_min"],
        summary = "Sets the inclusive minimum stack count.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes count.min.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            min = "Inclusive minimum stack count."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().count_min(1)",
    )]
    pub fn count_min(mut self, min: i64) -> Self {
        self.count = Some(IntRange::at_least(min));
        self
    }

    /// Require at most `max` items in the slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::count_max",
        aliases = ["sand::component::ItemPredicate::count_max","sand::prelude::ItemPredicate::count_max"],
        summary = "Sets the inclusive maximum stack count.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes count.max.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            max = "Inclusive maximum stack count."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().count_max(1)",
    )]
    pub fn count_max(mut self, max: i64) -> Self {
        self.count = Some(IntRange::at_most(max));
        self
    }

    /// Require between `min` and `max` items in the slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::count_range",
        aliases = ["sand::component::ItemPredicate::count_range","sand::prelude::ItemPredicate::count_range"],
        summary = "Sets an inclusive stack-count interval.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes both count bounds.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            min = "Inclusive minimum stack count.",
            max = "Inclusive maximum stack count."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().count_range(1, 16)",
    )]
    pub fn count_range(mut self, min: i64, max: i64) -> Self {
        self.count = Some(IntRange::between(min, max));
        self
    }

    /// Set the count predicate directly.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::count",
        aliases = ["sand::component::ItemPredicate::count","sand::prelude::ItemPredicate::count"],
        summary = "Sets a typed stack-count range.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes the count range.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            r = "Accepted stack counts."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().count(IntRange::at_least(1))",
    )]
    pub fn count(mut self, r: IntRange) -> Self {
        self.count = Some(r);
        self
    }

    /// Require a named key in the item's `custom_data` component to be present and truthy.
    ///
    /// This is the primary way to detect Sand custom items tagged with `.custom_data("key")`.
    ///
    /// This lowers to a **partial** NBT match against `minecraft:custom_data` under
    /// the vanilla `predicates` bag (`predicates.minecraft:custom_data = "{key:1b,...}"`),
    /// not an exact `components` equality check. Partial matching means the item may
    /// carry additional custom-data keys added by other packs/mods and still match —
    /// this is the correct semantics for "is this a Sand custom item of kind X", since
    /// exact `components` equality would reject any item whose `custom_data` differs by
    /// even one unrelated key.
    ///
    /// Calling this multiple times ANDs the keys together into one partial-match compound.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::custom_data_key",
        aliases = ["sand::component::ItemPredicate::custom_data_key","sand::prelude::ItemPredicate::custom_data_key"],
        summary = "Requires a key in item custom data.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes a custom_data presence predicate.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            key = "Exact custom-data key required."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().custom_data_key(\"quest_item\")",
    )]
    pub fn custom_data_key(mut self, key: impl Into<String>) -> Self {
        self.custom_data_keys.push(key.into());
        self
    }

    /// Add raw **exact-match** component predicates as an explicit escape hatch.
    ///
    /// Values under `components` must equal the item's component data exactly.
    /// Prefer [`custom_data_key`](Self::custom_data_key) or
    /// [`raw_predicates`](Self::raw_predicates) for partial matching.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::raw_components",
        aliases = ["sand::component::ItemPredicate::raw_components","sand::prelude::ItemPredicate::raw_components"],
        summary = "Supplies unsupported raw item component values.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Merges JSON into the item components section.",
        use_when = ["Minecraft supports item data Sand does not yet model"],
        avoid_when = ["A typed item method expresses the requirement"],
        params(
            v = "JSON object containing component values."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().raw_components(raw)",
    )]
    pub fn raw_components(mut self, v: RawJson) -> Self {
        self.raw_components = Some(v);
        self
    }

    /// Add raw **partial-match** sub-predicates as an explicit escape hatch.
    ///
    /// Merged into the same `predicates` bag as [`custom_data_key`](Self::custom_data_key).
    /// The value must be a JSON object mapping predicate condition IDs
    /// (e.g. `"minecraft:enchantments"`) to their predicate payload.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::ItemPredicate::raw_predicates",
        aliases = ["sand::component::ItemPredicate::raw_predicates","sand::prelude::ItemPredicate::raw_predicates"],
        summary = "Supplies unsupported raw item component predicate tests.",
        context = "Adds one domain-specific ItemPredicate requirement without disturbing its other constraints.",
        minecraft = "Merges JSON into the item predicates section.",
        use_when = ["Minecraft supports item data Sand does not yet model"],
        avoid_when = ["A typed item method expresses the requirement"],
        params(
            v = "JSON object containing component predicate tests."
        ),
        returns = "The updated ItemPredicate predicate.",
        example = "ItemPredicate::new().raw_predicates(raw)",
    )]
    pub fn raw_predicates(mut self, v: RawJson) -> Self {
        self.raw_predicates = Some(v);
        self
    }

    pub(crate) fn is_raw(&self) -> bool {
        self._raw.is_some()
    }

    pub(crate) fn has_component_constraints(&self) -> bool {
        !self.custom_data_keys.is_empty()
            || self.raw_components.is_some()
            || self.raw_predicates.is_some()
    }

    pub(crate) fn render_for_advancement(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<Value, String> {
        self.validate_at("item predicate")?;
        if !self.is_raw()
            && self.has_component_constraints()
            && caps
                .is_some_and(|caps| !caps.supports(sand_version::ComponentFeature::ItemComponents))
        {
            return Err(
                "item-component matching is unavailable for this target profile; remove the component constraint, target Minecraft 1.20.5+, or use ItemPredicate::raw(...) with manually verified legacy JSON"
                    .into(),
            );
        }
        serde_json::to_value(self).map_err(|error| error.to_string())
    }
}

/// Render custom-data marker keys as a partial-match SNBT compound,
/// e.g. `["elevator"]` → `{elevator:1b}`. Namespaced marker keys (e.g.
/// `arcane:dash_wand`) are quoted via `crate::item::snbt_compound_key` — the
/// same helper `sand_components::item` uses for exact-match `custom_data`
/// SNBT — so marker keys round-trip as valid SNBT instead of producing an
/// ambiguous bare `:` token.
fn custom_data_partial_snbt(keys: &[String]) -> String {
    let body = keys
        .iter()
        .map(|k| format!("{}:1b", crate::item::snbt_compound_key(k)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}")
}

impl Serialize for ItemPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let mut map = serializer.serialize_map(None)?;
        if let Some(ref v) = self.items {
            map.serialize_entry("items", v)?;
        }
        if let Some(ref v) = self.count {
            map.serialize_entry("count", v)?;
        }
        if let Some(ref raw_c) = self.raw_components {
            map.serialize_entry("components", raw_c)?;
        }
        // Build the partial-match `predicates` bag from typed custom-data keys + raw fallback.
        let has_custom_data = !self.custom_data_keys.is_empty();
        let has_raw_predicates = self.raw_predicates.is_some();
        if has_custom_data || has_raw_predicates {
            let mut pred_map: serde_json::Map<String, Value> = serde_json::Map::new();
            if has_custom_data {
                let snbt = custom_data_partial_snbt(&self.custom_data_keys);
                pred_map.insert("minecraft:custom_data".to_string(), Value::String(snbt));
            }
            if let Some(ref raw_p) = self.raw_predicates
                && let Value::Object(obj) = raw_p.as_value()
            {
                for (k, v) in obj {
                    pred_map.insert(k.clone(), v.clone());
                }
            }
            map.serialize_entry("predicates", &Value::Object(pred_map))?;
        }
        map.end()
    }
}

// ── EntityPredicate ───────────────────────────────────────────────────────────

/// Typed entity predicate — used in kill/hurt triggers, loot conditions, and more.
///
/// # Example
/// ```rust
/// use sand_components::{EntityTypeId, RawSnbt, predicates::EntityPredicate};
///
/// let ep = EntityPredicate::type_(EntityTypeId::minecraft("zombie")?)
///     .nbt(RawSnbt::new("{IsBaby:1b}"));
/// # Ok::<(), sand_components::SandError>(())
/// ```
#[derive(Debug, Clone, Default)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::EntityPredicate",
    aliases = ["sand::prelude::EntityPredicate", "sand::component::EntityPredicate"],
    summary = "Describes typed properties required of a Minecraft entity.",
    context = "Entity predicates combine identity, flags, equipment, effects, location, distance, and nested relationships for vanilla condition evaluation.",
    minecraft = "Serializes the entity predicate object nested inside advancement, loot, or standalone predicate conditions.",
    use_when = ["Checking entity equipment or flags", "Restricting an event or loot condition by entity properties"],
    avoid_when = ["Selecting live command targets without a predicate context"],
    example = "EntityPredicate::new().equipment(EntityEquipment::new())",
)]
pub struct EntityPredicate {
    entity_type: Option<EntityTypeMatch>,
    nbt: Option<RawSnbt>,
    location: Option<LocationPredicate>,
    flags: Option<EntityFlags>,
    equipment: Option<EntityEquipment>,
    effects: Option<BTreeMap<String, EffectPredicate>>,
    _raw: Option<RawJson>,
}

/// How to match entity types — single type or any of a list.
#[derive(Debug, Clone)]
enum EntityTypeMatch {
    Single(EntityTypeId),
    AnyOf(Vec<EntityTypeId>),
}

impl Serialize for EntityTypeMatch {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            EntityTypeMatch::Single(t) => t.serialize(s),
            EntityTypeMatch::AnyOf(types) => types.serialize(s),
        }
    }
}

/// Boolean entity flags checked in predicates.
#[derive(Debug, Clone, Default, Serialize)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::EntityFlags",
    aliases = ["sand::prelude::EntityFlags"],
    summary = "Matches boolean runtime flags exposed by vanilla entity predicates.",
    context = "Flags describe observable entity state such as fire, movement stance, swimming, or age.",
    minecraft = "Serializes the vanilla flags object nested in an entity predicate.",
    use_when = ["Restricting a condition by entity state flags"],
    avoid_when = ["Changing those flags or selecting unrelated entity properties"],
    example = "EntityFlags::new().sneaking(true)",
)]
pub struct EntityFlags {
    #[serde(skip_serializing_if = "Option::is_none")]
    is_on_fire: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_sneaking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_sprinting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_swimming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_baby: Option<bool>,
}

impl EntityFlags {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::new",
        aliases = ["sand::prelude::EntityFlags::new"],
        summary = "Creates an unconstrained EntityFlags.",
        context = "Builder methods add only the EntityFlags requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty EntityFlags builder.",
        example = "EntityFlags::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::on_fire",
        aliases = ["sand::prelude::EntityFlags::on_fire"],
        summary = "Requires a specific burning state.",
        context = "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.",
        minecraft = "Writes the on_fire flag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required burning state."
        ),
        returns = "The updated EntityFlags predicate.",
        example = "EntityFlags::new().on_fire(true)",
    )]
    pub fn on_fire(mut self, v: bool) -> Self {
        self.is_on_fire = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::sneaking",
        aliases = ["sand::prelude::EntityFlags::sneaking"],
        summary = "Requires a specific sneaking state.",
        context = "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.",
        minecraft = "Writes the sneaking flag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required sneaking state."
        ),
        returns = "The updated EntityFlags predicate.",
        example = "EntityFlags::new().sneaking(true)",
    )]
    pub fn sneaking(mut self, v: bool) -> Self {
        self.is_sneaking = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::sprinting",
        aliases = ["sand::prelude::EntityFlags::sprinting"],
        summary = "Requires a specific sprinting state.",
        context = "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.",
        minecraft = "Writes the sprinting flag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required sprinting state."
        ),
        returns = "The updated EntityFlags predicate.",
        example = "EntityFlags::new().sprinting(true)",
    )]
    pub fn sprinting(mut self, v: bool) -> Self {
        self.is_sprinting = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::swimming",
        aliases = ["sand::prelude::EntityFlags::swimming"],
        summary = "Requires a specific swimming state.",
        context = "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.",
        minecraft = "Writes the swimming flag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required swimming state."
        ),
        returns = "The updated EntityFlags predicate.",
        example = "EntityFlags::new().swimming(true)",
    )]
    pub fn swimming(mut self, v: bool) -> Self {
        self.is_swimming = Some(v);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityFlags::baby",
        aliases = ["sand::prelude::EntityFlags::baby"],
        summary = "Requires a specific baby-age state.",
        context = "Adds one domain-specific EntityFlags requirement without disturbing its other constraints.",
        minecraft = "Writes the baby flag.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Required baby-age state."
        ),
        returns = "The updated EntityFlags predicate.",
        example = "EntityFlags::new().baby(true)",
    )]
    pub fn baby(mut self, v: bool) -> Self {
        self.is_baby = Some(v);
        self
    }
}

/// Equipment slot predicates for entity equipment checks.
#[derive(Debug, Clone, Default, Serialize)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::EntityEquipment",
    aliases = ["sand::prelude::EntityEquipment"],
    summary = "Matches item predicates in the six vanilla entity equipment slots.",
    context = "Keeping each slot typed makes equipment conditions composable within EntityPredicate.",
    minecraft = "Serializes head, chest, legs, feet, mainhand, and offhand item requirements.",
    use_when = ["Checking worn armor or held items"],
    avoid_when = ["Addressing inventory slots for mutation"],
    example = "EntityEquipment::new().head(ItemPredicate::id(item))",
)]
pub struct EntityEquipment {
    #[serde(skip_serializing_if = "Option::is_none")]
    head: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chest: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    legs: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    feet: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mainhand: Option<ItemPredicate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offhand: Option<ItemPredicate>,
}

impl EntityEquipment {
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::new",
        aliases = ["sand::prelude::EntityEquipment::new"],
        summary = "Creates an unconstrained EntityEquipment.",
        context = "Builder methods add only the EntityEquipment requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty EntityEquipment builder.",
        example = "EntityEquipment::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::head",
        aliases = ["sand::prelude::EntityEquipment::head"],
        summary = "Constrains the entity's head slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under head.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the head slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().head(ItemPredicate::new())",
    )]
    pub fn head(mut self, p: ItemPredicate) -> Self {
        self.head = Some(p);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::chest",
        aliases = ["sand::prelude::EntityEquipment::chest"],
        summary = "Constrains the entity's chest slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under chest.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the chest slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().chest(ItemPredicate::new())",
    )]
    pub fn chest(mut self, p: ItemPredicate) -> Self {
        self.chest = Some(p);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::legs",
        aliases = ["sand::prelude::EntityEquipment::legs"],
        summary = "Constrains the entity's legs slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under legs.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the legs slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().legs(ItemPredicate::new())",
    )]
    pub fn legs(mut self, p: ItemPredicate) -> Self {
        self.legs = Some(p);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::feet",
        aliases = ["sand::prelude::EntityEquipment::feet"],
        summary = "Constrains the entity's feet slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under feet.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the feet slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().feet(ItemPredicate::new())",
    )]
    pub fn feet(mut self, p: ItemPredicate) -> Self {
        self.feet = Some(p);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::mainhand",
        aliases = ["sand::prelude::EntityEquipment::mainhand"],
        summary = "Constrains the entity's mainhand slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under mainhand.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the mainhand slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().mainhand(ItemPredicate::new())",
    )]
    pub fn mainhand(mut self, p: ItemPredicate) -> Self {
        self.mainhand = Some(p);
        self
    }
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityEquipment::offhand",
        aliases = ["sand::prelude::EntityEquipment::offhand"],
        summary = "Constrains the entity's offhand slot.",
        context = "Adds one domain-specific EntityEquipment requirement without disturbing its other constraints.",
        minecraft = "Writes an item predicate under offhand.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            p = "Item requirements for the offhand slot."
        ),
        returns = "The updated EntityEquipment predicate.",
        example = "EntityEquipment::new().offhand(ItemPredicate::new())",
    )]
    pub fn offhand(mut self, p: ItemPredicate) -> Self {
        self.offhand = Some(p);
        self
    }
}

impl EntityPredicate {
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
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
        if let Some(effects) = &self.effects {
            for (effect, predicate) in effects {
                predicate.validate_at(&format!("{path}.effects.{effect}"))?;
            }
        }
        Ok(())
    }
    /// Match any entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::new",
        aliases = ["sand::component::EntityPredicate::new", "sand::prelude::EntityPredicate::new"],
        summary = "Creates an unconstrained EntityPredicate.",
        context = "Builder methods add only the EntityPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty EntityPredicate builder.",
        example = "EntityPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON verbatim as this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::raw",
        aliases = ["sand::component::EntityPredicate::raw", "sand::prelude::EntityPredicate::raw"],
        summary = "Creates a EntityPredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw EntityPredicate.",
        example = "EntityPredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    /// Match a specific entity type ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::type_",
        aliases = ["sand::component::EntityPredicate::type_","sand::prelude::EntityPredicate::type_"],
        summary = "Creates a predicate for one entity type.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Initializes the type field.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            entity_type = "Entity type identifier to match."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::type_(entity_type)",
    )]
    pub fn type_(entity_type: impl Into<EntityTypeId>) -> Self {
        Self::new().with_type(entity_type)
    }

    /// Set (or override) the entity type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::with_type",
        aliases = ["sand::component::EntityPredicate::with_type","sand::prelude::EntityPredicate::with_type"],
        summary = "Requires one entity type.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes one typed entity identifier.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            entity_type = "Entity type identifier to match."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().with_type(entity_type)",
    )]
    pub fn with_type(mut self, entity_type: impl Into<EntityTypeId>) -> Self {
        self.entity_type = Some(EntityTypeMatch::Single(entity_type.into()));
        self
    }

    /// Match any of the given entity type IDs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::with_type_any",
        aliases = ["sand::component::EntityPredicate::with_type_any","sand::prelude::EntityPredicate::with_type_any"],
        summary = "Accepts any entity type in a typed list.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes entity-type alternatives.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            types = "Non-empty accepted entity identifiers."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().with_type_any(vec![zombie, skeleton])",
    )]
    pub fn with_type_any(mut self, types: Vec<EntityTypeId>) -> Self {
        self.entity_type = Some(EntityTypeMatch::AnyOf(types));
        self
    }

    /// Require the entity to match this SNBT string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::nbt",
        aliases = ["sand::component::EntityPredicate::nbt","sand::prelude::EntityPredicate::nbt"],
        summary = "Constrains entity NBT data.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes the SNBT fragment.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            nbt = "Entity data fragment that must match."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().nbt(nbt)",
    )]
    pub fn nbt(mut self, nbt: RawSnbt) -> Self {
        self.nbt = Some(nbt);
        self
    }

    /// Require the entity to be at a location matching this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::location",
        aliases = ["sand::component::EntityPredicate::location","sand::prelude::EntityPredicate::location"],
        summary = "Constrains the entity's location.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Nests a location predicate.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            lp = "World-location requirements."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().location(LocationPredicate::new())",
    )]
    pub fn location(mut self, lp: LocationPredicate) -> Self {
        self.location = Some(lp);
        self
    }

    /// Require specific boolean entity flags.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::flags",
        aliases = ["sand::component::EntityPredicate::flags","sand::prelude::EntityPredicate::flags"],
        summary = "Constrains entity state flags.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Nests the flags object.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            flags = "Required fire, movement, and age flags."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().flags(EntityFlags::new())",
    )]
    pub fn flags(mut self, flags: EntityFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Require the entity to wear/hold specific equipment.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::equipment",
        aliases = ["sand::component::EntityPredicate::equipment","sand::prelude::EntityPredicate::equipment"],
        summary = "Constrains worn or held items.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Nests slot-specific item predicates.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            eq = "Required equipment by slot."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().equipment(EntityEquipment::new())",
    )]
    pub fn equipment(mut self, eq: EntityEquipment) -> Self {
        self.equipment = Some(eq);
        self
    }

    /// Require an active status effect (by effect ID).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::EntityPredicate::effect",
        aliases = ["sand::component::EntityPredicate::effect","sand::prelude::EntityPredicate::effect"],
        summary = "Constrains one active status effect.",
        context = "Adds one domain-specific EntityPredicate requirement without disturbing its other constraints.",
        minecraft = "Adds a typed effect and requirements to effects.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            effect_id = "Status effect that must be active.",
            pred = "Required amplifier, duration, and flags."
        ),
        returns = "The updated EntityPredicate predicate.",
        example = "EntityPredicate::new().effect(effect_id, EffectPredicate::new())",
    )]
    pub fn effect(mut self, effect_id: impl Into<EffectId>, pred: EffectPredicate) -> Self {
        self.effects
            .get_or_insert_with(BTreeMap::new)
            .insert(effect_id.into().to_string(), pred);
        self
    }

    /// Render an entity predicate for an advancement entity consumer.
    ///
    /// Minecraft 26.2 moved typed entity sub-predicates to namespaced keys
    /// (`minecraft:entity_type`, `minecraft:location`, ...). Earlier active
    /// profiles use the historical unnamespaced keys. Raw predicates are
    /// preserved verbatim because their compatibility is user-owned.
    pub(crate) fn render_for_advancement(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<Value, String> {
        if let Some(raw) = &self._raw {
            return Ok(raw.as_value().clone());
        }
        let namespaced = caps.is_none_or(|caps| caps.is_at_least(26, 2, 0));
        let key = |legacy: &'static str, modern: &'static str| {
            if namespaced { modern } else { legacy }
        };
        let mut object = serde_json::Map::new();
        if let Some(entity_type) = &self.entity_type {
            object.insert(
                key("type", "minecraft:entity_type").into(),
                serde_json::to_value(entity_type).map_err(|error| error.to_string())?,
            );
        }
        if let Some(nbt) = &self.nbt {
            object.insert(
                key("nbt", "minecraft:nbt").into(),
                Value::String(nbt.as_str().to_owned()),
            );
        }
        if let Some(location) = &self.location {
            object.insert(
                key("location", "minecraft:location").into(),
                location.render_for_advancement(caps)?,
            );
        }
        if let Some(flags) = &self.flags {
            object.insert(
                key("flags", "minecraft:flags").into(),
                serde_json::to_value(flags).map_err(|error| error.to_string())?,
            );
        }
        if let Some(equipment) = &self.equipment {
            let mut rendered = serde_json::Map::new();
            for (slot, item) in [
                ("head", &equipment.head),
                ("chest", &equipment.chest),
                ("legs", &equipment.legs),
                ("feet", &equipment.feet),
                ("mainhand", &equipment.mainhand),
                ("offhand", &equipment.offhand),
            ] {
                if let Some(item) = item {
                    rendered.insert(slot.into(), item.render_for_advancement(caps)?);
                }
            }
            object.insert(
                key("equipment", "minecraft:equipment").into(),
                Value::Object(rendered),
            );
        }
        if let Some(effects) = &self.effects {
            object.insert(
                key("effects", "minecraft:effects").into(),
                serde_json::to_value(effects).map_err(|error| error.to_string())?,
            );
        }
        Ok(Value::Object(object))
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
            map.serialize_entry("nbt", v.as_str())?;
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

// ── WeatherPredicate ──────────────────────────────────────────────────────────

/// Checks current weather state — used by the standalone `minecraft:weather_check`
/// predicate condition.
///
/// # Example
/// ```rust
/// use sand_components::predicates::WeatherPredicate;
///
/// let wp = WeatherPredicate::new().raining(true);
/// ```
#[derive(Debug, Clone, Default)]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::predicate::WeatherPredicate",
    aliases = ["sand::component::WeatherPredicate", "sand::prelude::WeatherPredicate"],
    summary = "Matches the world's raining and thundering states.",
    context = "Standalone weather roots use this reusable value to keep both vanilla weather flags explicit.",
    minecraft = "Serializes raining and thundering fields in a weather-check condition.",
    use_when = ["Gating a predicate on current weather"],
    avoid_when = ["Changing weather or tracking a forecast"],
    example = "WeatherPredicate::new().raining(true)",
)]
pub struct WeatherPredicate {
    raining: Option<bool>,
    thundering: Option<bool>,
    _raw: Option<RawJson>,
}

impl WeatherPredicate {
    pub(crate) fn validate_at(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::WeatherPredicate::new",
        aliases = ["sand::component::WeatherPredicate::new", "sand::prelude::WeatherPredicate::new"],
        summary = "Creates an unconstrained WeatherPredicate.",
        context = "Builder methods add only the WeatherPredicate requirements relevant to the surrounding condition.",
        minecraft = "Serializes an empty predicate object until constraints are added.",
        use_when = ["Building a typed predicate incrementally"],
        avoid_when = ["No constraints will be added"],
        returns = "An empty WeatherPredicate builder.",
        example = "WeatherPredicate::new()",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raw escape hatch — serialize arbitrary JSON as this predicate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::WeatherPredicate::raw",
        aliases = ["sand::component::WeatherPredicate::raw", "sand::prelude::WeatherPredicate::raw"],
        summary = "Creates a WeatherPredicate from an unsupported raw JSON shape.",
        context = "The explicit escape hatch preserves access to modded or newly introduced fields without weakening typed builder methods.",
        minecraft = "Emits the supplied JSON value in place of the typed predicate object.",
        use_when = ["Minecraft supports a predicate field Sand does not yet model"],
        avoid_when = ["Typed builder methods cover the required fields"],
        params(
            v = "The complete raw JSON predicate value."
        ),
        returns = "A raw WeatherPredicate.",
        example = "WeatherPredicate::raw(RawJson::new(json!({{}})))",
    )]
    pub fn raw(v: RawJson) -> Self {
        Self {
            _raw: Some(v),
            ..Default::default()
        }
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::WeatherPredicate::raining",
        aliases = ["sand::component::WeatherPredicate::raining","sand::prelude::WeatherPredicate::raining"],
        summary = "Requires a specific rain state.",
        context = "Adds one domain-specific WeatherPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes the raining boolean.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Whether rain must be active."
        ),
        returns = "The updated WeatherPredicate predicate.",
        example = "WeatherPredicate::new().raining(true)",
    )]
    pub fn raining(mut self, v: bool) -> Self {
        self.raining = Some(v);
        self
    }

    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::predicate::WeatherPredicate::thundering",
        aliases = ["sand::component::WeatherPredicate::thundering","sand::prelude::WeatherPredicate::thundering"],
        summary = "Requires a specific thunder state.",
        context = "Adds one domain-specific WeatherPredicate requirement without disturbing its other constraints.",
        minecraft = "Writes the thundering boolean.",
        use_when = ["Composing this property into a larger predicate"],
        avoid_when = ["The property should remain unconstrained"],
        params(
            v = "Whether thunder must be active."
        ),
        returns = "The updated WeatherPredicate predicate.",
        example = "WeatherPredicate::new().thundering(true)",
    )]
    pub fn thundering(mut self, v: bool) -> Self {
        self.thundering = Some(v);
        self
    }
}

impl Serialize for WeatherPredicate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(ref raw) = self._raw {
            return raw.serialize(serializer);
        }
        let count = self.raining.is_some() as usize + self.thundering.is_some() as usize;
        let mut map = serializer.serialize_map(Some(count))?;
        if let Some(v) = self.raining {
            map.serialize_entry("raining", &v)?;
        }
        if let Some(v) = self.thundering {
            map.serialize_entry("thundering", &v)?;
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
        p.validate_at("predicate")
            .unwrap_or_else(|e| panic!("predicate validation failed: {e}"));
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

impl From<WeatherPredicate> for Value {
    fn from(p: WeatherPredicate) -> Value {
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

    fn item(path: &str) -> ItemId {
        ItemId::minecraft(path).unwrap()
    }

    fn entity_type(path: &str) -> EntityTypeId {
        EntityTypeId::minecraft(path).unwrap()
    }

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
        let p = ItemPredicate::id(item("diamond"));
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["items"], json!(["minecraft:diamond"]));
    }

    #[test]
    fn item_predicate_with_count() {
        let p = ItemPredicate::id(item("diamond")).count_min(5);
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["count"], json!({"min": 5}));
    }

    #[test]
    fn item_predicate_custom_data_key() {
        let p = ItemPredicate::id(item("diamond_sword")).custom_data_key("my_sword");
        let v = serde_json::to_value(&p).unwrap();
        // Partial NBT match, not exact `components` equality — see #233/#232.
        assert_eq!(v["predicates"]["minecraft:custom_data"], "{my_sword:1b}");
        assert!(v.get("components").is_none());
    }

    #[test]
    fn item_predicate_custom_data_key_multiple_keys_and() {
        let p = ItemPredicate::id(item("white_wool"))
            .custom_data_key("elevator")
            .custom_data_key("floor");
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(
            v["predicates"]["minecraft:custom_data"],
            "{elevator:1b,floor:1b}"
        );
    }

    #[test]
    fn item_predicate_raw_predicates_merges_with_custom_data() {
        let p = ItemPredicate::id(item("bow"))
            .custom_data_key("enchanted_bow")
            .raw_predicates(RawJson::new(
                json!({"minecraft:enchantments": {"levels": {"min": 1}}}),
            ));
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(
            v["predicates"]["minecraft:custom_data"],
            "{enchanted_bow:1b}"
        );
        assert_eq!(
            v["predicates"]["minecraft:enchantments"]["levels"]["min"],
            1
        );
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
        let ep = EntityPredicate::type_(entity_type("zombie"));
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(v["type"], "minecraft:zombie");
    }

    #[test]
    fn entity_predicate_nbt() {
        let ep = EntityPredicate::type_(entity_type("cow")).nbt(RawSnbt::new("{IsBaby:1b}"));
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
        let ep = EntityPredicate::type_(entity_type("player"))
            .equipment(EntityEquipment::new().feet(ItemPredicate::id(item("diamond_boots"))));
        let v = serde_json::to_value(&ep).unwrap();
        assert_eq!(
            v["equipment"]["feet"]["items"],
            json!(["minecraft:diamond_boots"])
        );
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
    fn effect_predicate_serializes_constraints() {
        let pred = EffectPredicate::new()
            .amplifier(IntRange::exact(1))
            .duration(IntRange::at_least(200))
            .ambient(false)
            .visible(true);
        assert_eq!(
            serde_json::to_value(&pred).unwrap(),
            json!({
                "amplifier": 1,
                "duration": {"min": 200},
                "ambient": false,
                "visible": true
            })
        );
    }

    #[test]
    fn entity_predicate_accepts_custom_effect_id() {
        let pred = EntityPredicate::new().effect(
            EffectId::custom("mymod:arcane_burn").unwrap(),
            EffectPredicate::new().duration(IntRange::at_most(100)),
        );
        assert_eq!(
            serde_json::to_value(&pred).unwrap(),
            json!({"effects": {"mymod:arcane_burn": {"duration": {"max": 100}}}})
        );
    }

    #[test]
    fn shared_status_effect_id_uses_existing_typed_predicate_paths() {
        let effect = crate::StatusEffectId::minecraft("speed").unwrap();
        let entity = EntityPredicate::new()
            .effect(effect, EffectPredicate::new().amplifier(IntRange::exact(1)));
        assert_eq!(
            serde_json::to_value(entity).unwrap()["effects"]["minecraft:speed"],
            json!({"amplifier": 1})
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
            .biome(BiomeId::minecraft("plains").unwrap())
            .dimension(DimensionId::minecraft("overworld").unwrap());
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
        let bp = BlockPredicate::new().tag(TagId::minecraft("logs").unwrap());
        let v = serde_json::to_value(&bp).unwrap();
        assert_eq!(v["tag"], "minecraft:logs");
    }

    #[test]
    fn damage_source_predicate_tags() {
        let dsp = DamageSourcePredicate::new()
            .requires_tag(TagId::minecraft("is_fire").unwrap())
            .excludes_tag(TagId::minecraft("bypasses_armor").unwrap());
        let v = serde_json::to_value(&dsp).unwrap();
        assert_eq!(v["tags"][0]["id"], "minecraft:is_fire");
        assert_eq!(v["tags"][0]["expected"], true);
        assert_eq!(v["tags"][1]["id"], "minecraft:bypasses_armor");
        assert_eq!(v["tags"][1]["expected"], false);
    }
}
