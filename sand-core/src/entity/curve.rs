//! Deterministic derived-stat curves.
//!
//! Curves in this module are a pure intermediate representation. They neither
//! inspect Minecraft nor emit commands. An archetype compiler can validate and
//! evaluate the same curve that it later lowers to scoreboard arithmetic,
//! balanced branches, a storage-backed table, or a registered callback.
//!
//! Values use signed fixed-point integers. [`FixedPoint::default`] uses three
//! decimal places (`1.0 == 1000`). Every multiply and divide applies the
//! configured [`RoundingPolicy`], and every conversion and arithmetic operation
//! applies the configured [`OverflowPolicy`]. This makes results independent of
//! the host platform and avoids floating-point work in generated functions.
//!
//! # Level-derived health
//!
//! ```
//! use sand_core::entity::{CurveInputs, FixedPoint, StatCurve};
//!
//! let fixed = FixedPoint::default();
//! let health = StatCurve::clamped_linear(
//!     StatCurve::input_raw("rpg_level"),
//!     2.5,
//!     17.5,
//!     20.0,
//!     100.0,
//! );
//! let mut inputs = CurveInputs::new();
//! inputs.insert_score("rpg_level", 10, fixed, "rpg:mob", "health")?;
//! let result = health.evaluate(&inputs, fixed, "rpg:mob", "health")?;
//!
//! assert_eq!(result.as_f64(fixed), 42.5);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

use super::{EntityDiagnostic, EntityStateField};

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::DEFAULT_FIXED_POINT_SCALE",
    aliases = ["sand::prelude::DEFAULT_FIXED_POINT_SCALE"],
    module = "sand::entity",
    summary = "Default number of fixed-point units in one whole value.",
    context = "Default number of fixed-point units in one whole value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::DEFAULT_FIXED_POINT_SCALE;",
)]
/// Default number of fixed-point units in one whole value.
pub const DEFAULT_FIXED_POINT_SCALE: i64 = 1_000;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::FixedPoint",
    aliases = ["sand::prelude::FixedPoint"],
    module = "sand::entity",
    summary = "Fixed-point representation settings used by a curve.",
    context = "Fixed-point representation settings used by a curve. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::FixedPoint;",
)]
/// Fixed-point representation settings used by a curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedPoint {
    scale: i64,
    rounding: RoundingPolicy,
    overflow: OverflowPolicy,
}

impl Default for FixedPoint {
    fn default() -> Self {
        Self {
            scale: DEFAULT_FIXED_POINT_SCALE,
            rounding: RoundingPolicy::NearestTiesAwayFromZero,
            overflow: OverflowPolicy::Error,
        }
    }
}

impl FixedPoint {
    /// Creates settings with a positive scale.
    ///
    /// A scale of `1000` stores three decimal places. A zero or negative scale
    /// returns an [`EntityDiagnostic::InvalidRange`] before export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::new",
        aliases = ["sand::prelude::FixedPoint::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates settings with a positive scale. A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export.",
        context = "Creates settings with a positive scale. A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(scale = "`scale` supplies the scale value used to create settings with a positive scale. A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export.", rounding = "`rounding` supplies the rounding value used to create settings with a positive scale. A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export.", overflow = "`overflow` supplies the overflow value used to create settings with a positive scale. A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export."),
        returns = "A scale of `1000` stores three decimal places. A zero or negative scale returns an [`EntityDiagnostic::InvalidRange`] before export.",
        example = "use sand::prelude::*;\n\nfn demonstrate(scale: i64, rounding: sand::entity::RoundingPolicy, overflow: sand::entity::OverflowPolicy)  {\n    let fixed_point_result = sand::entity::FixedPoint::new(scale, rounding, overflow);\n}",
    )]
    pub fn new(
        scale: i64,
        rounding: RoundingPolicy,
        overflow: OverflowPolicy,
    ) -> Result<Self, EntityDiagnostic> {
        if scale <= 0 {
            return Err(EntityDiagnostic::InvalidRange {
                schema: "entity-curve".into(),
                field: "fixed_point_scale".into(),
                range: scale.to_string(),
            });
        }
        Ok(Self {
            scale,
            rounding,
            overflow,
        })
    }

    /// Returns the number of stored units representing `1.0`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::scale",
        aliases = ["sand::prelude::FixedPoint::scale"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the number of stored units representing `1.0`.",
        context = "Returns the number of stored units representing `1.0`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns the number of stored units representing `1.0`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint)  {\n    let scale = fixed_point_value.scale();\n}",
    )]
    #[must_use]
    pub const fn scale(self) -> i64 {
        self.scale
    }

    /// Returns the rounding rule for lossy operations.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::rounding",
        aliases = ["sand::prelude::FixedPoint::rounding"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the rounding rule for lossy operations.",
        context = "Returns the rounding rule for lossy operations. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns the rounding rule for lossy operations.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint)  {\n    let rounding = fixed_point_value.rounding();\n}",
    )]
    #[must_use]
    pub const fn rounding(self) -> RoundingPolicy {
        self.rounding
    }

    /// Returns the overflow behavior for conversion and arithmetic.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::overflow",
        aliases = ["sand::prelude::FixedPoint::overflow"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the overflow behavior for conversion and arithmetic.",
        context = "Returns the overflow behavior for conversion and arithmetic. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns the overflow behavior for conversion and arithmetic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint)  {\n    let overflow = fixed_point_value.overflow();\n}",
    )]
    #[must_use]
    pub const fn overflow(self) -> OverflowPolicy {
        self.overflow
    }

    /// Converts a finite host value into deterministic fixed-point units.
    ///
    /// Floating point is accepted only at definition time. Runtime evaluation
    /// and generated Minecraft arithmetic use the resulting integer.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::encode",
        aliases = ["sand::prelude::FixedPoint::encode"],
        module = "sand::entity",
        kind = "method",
        summary = "Converts a finite host value into deterministic fixed-point units.",
        context = "Converts a finite host value into deterministic fixed-point units. Floating point is accepted only at definition time. Runtime evaluation and generated Minecraft arithmetic use the resulting integer.",
        minecraft = "Floating point is accepted only at definition time. Runtime evaluation and generated Minecraft arithmetic use the resulting integer.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to convert a finite host value into deterministic fixed-point units.", archetype = "`archetype` provides the entity archetype supplying the property used to convert a finite host value into deterministic fixed-point units.", derivation = "`derivation` provides the derived-stat selector used to convert a finite host value into deterministic fixed-point units."),
        returns = "On success, the value produced to convert a finite host value into deterministic fixed-point units; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint, value: f64, archetype: & str, derivation: & str)  {\n    let encode = fixed_point_value.encode(value, archetype, derivation);\n}",
    )]
    pub fn encode(
        self,
        value: f64,
        archetype: &str,
        derivation: &str,
    ) -> Result<FixedValue, EntityDiagnostic> {
        if !value.is_finite() {
            return Err(non_finite(archetype, derivation, value));
        }
        let scaled = value * self.scale as f64;
        if !scaled.is_finite() || scaled > i64::MAX as f64 || scaled < i64::MIN as f64 {
            return match self.overflow {
                OverflowPolicy::Error => Err(overflow(
                    archetype,
                    derivation,
                    format!("`{value}` cannot be represented at scale {}", self.scale),
                )),
                OverflowPolicy::Saturate => Ok(FixedValue(if scaled.is_sign_negative() {
                    i64::MIN
                } else {
                    i64::MAX
                })),
            };
        }
        Ok(FixedValue(round_float(scaled, self.rounding) as i64))
    }

    /// Converts a whole scoreboard value to fixed-point units.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::encode_score",
        aliases = ["sand::prelude::FixedPoint::encode_score"],
        module = "sand::entity",
        kind = "method",
        summary = "Converts a whole scoreboard value to fixed-point units.",
        context = "Converts a whole scoreboard value to fixed-point units. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to convert a whole scoreboard value to fixed-point units.", archetype = "`archetype` provides the entity archetype supplying the property used to convert a whole scoreboard value to fixed-point units.", derivation = "`derivation` provides the derived-stat selector used to convert a whole scoreboard value to fixed-point units."),
        returns = "On success, the value produced to convert a whole scoreboard value to fixed-point units; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint, value: i64, archetype: & str, derivation: & str)  {\n    let encode_score = fixed_point_value.encode_score(value, archetype, derivation);\n}",
    )]
    pub fn encode_score(
        self,
        value: i64,
        archetype: &str,
        derivation: &str,
    ) -> Result<FixedValue, EntityDiagnostic> {
        checked_i128(
            i128::from(value) * i128::from(self.scale),
            self.overflow,
            archetype,
            derivation,
            "score conversion overflowed",
        )
    }

    /// Converts fixed-point units to a whole scoreboard value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedPoint::decode_score",
        aliases = ["sand::prelude::FixedPoint::decode_score"],
        module = "sand::entity",
        kind = "method",
        summary = "Converts fixed-point units to a whole scoreboard value.",
        context = "Converts fixed-point units to a whole scoreboard value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to convert fixed-point units to a whole scoreboard value.", archetype = "`archetype` provides the entity archetype supplying the property used to convert fixed-point units to a whole scoreboard value.", derivation = "`derivation` provides the derived-stat selector used to convert fixed-point units to a whole scoreboard value."),
        returns = "On success, the value produced to convert fixed-point units to a whole scoreboard value; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_point_value: sand::entity::FixedPoint, value: sand::entity::FixedValue, archetype: & str, derivation: & str)  {\n    let decode_score = fixed_point_value.decode_score(value, archetype, derivation);\n}",
    )]
    pub fn decode_score(
        self,
        value: FixedValue,
        archetype: &str,
        derivation: &str,
    ) -> Result<i64, EntityDiagnostic> {
        divide_rounded(
            i128::from(value.0),
            i128::from(self.scale),
            self.rounding,
            self.overflow,
            archetype,
            derivation,
        )
        .map(FixedValue::units)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::RoundingPolicy",
    aliases = ["sand::prelude::RoundingPolicy"],
    module = "sand::entity",
    summary = "Rounding applied when fixed-point multiplication or division loses a fractional remainder.",
    context = "Rounding applied when fixed-point multiplication or division loses a fractional remainder. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::RoundingPolicy;",
    variants(Ceiling = "Round toward positive infinity.", Floor = "Round toward negative infinity.", NearestTiesAwayFromZero = "Round to the nearest integer, with exact halves away from zero.", NearestTiesToEven = "Round to the nearest integer, with exact halves to an even integer.", TowardZero = "Discard the remainder toward zero."),
)]
/// Rounding applied when fixed-point multiplication or division loses a
/// fractional remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RoundingPolicy {
    /// Discard the remainder toward zero.
    TowardZero,
    /// Round toward negative infinity.
    Floor,
    /// Round toward positive infinity.
    Ceiling,
    /// Round to the nearest integer, with exact halves away from zero.
    NearestTiesAwayFromZero,
    /// Round to the nearest integer, with exact halves to an even integer.
    NearestTiesToEven,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::OverflowPolicy",
    aliases = ["sand::prelude::OverflowPolicy"],
    module = "sand::entity",
    summary = "Behavior when a fixed-point result does not fit in a signed 64-bit value.",
    context = "Behavior when a fixed-point result does not fit in a signed 64-bit value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::OverflowPolicy;",
    variants(Error = "Stop validation/evaluation with a structured diagnostic.", Saturate = "Clamp the result to [`i64::MIN`] or [`i64::MAX`]."),
)]
/// Behavior when a fixed-point result does not fit in a signed 64-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverflowPolicy {
    /// Stop validation/evaluation with a structured diagnostic.
    Error,
    /// Clamp the result to [`i64::MIN`] or [`i64::MAX`].
    Saturate,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::FixedValue",
    aliases = ["sand::prelude::FixedValue"],
    module = "sand::entity",
    summary = "A signed fixed-point value. The scale is supplied by [`FixedPoint`]. Keeping the raw representation explicit prevents accidental interchange with whole scoreboard values.",
    context = "A signed fixed-point value. The scale is supplied by [`FixedPoint`]. Keeping the raw representation explicit prevents accidental interchange with whole scoreboard values. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "The scale is supplied by [`FixedPoint`]. Keeping the raw representation explicit prevents accidental interchange with whole scoreboard values.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::FixedValue;",
)]
/// A signed fixed-point value.
///
/// The scale is supplied by [`FixedPoint`]. Keeping the raw representation
/// explicit prevents accidental interchange with whole scoreboard values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedValue(i64);

impl FixedValue {
    /// Creates a value from already-scaled integer units.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedValue::from_units",
        aliases = ["sand::prelude::FixedValue::from_units"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a value from already-scaled integer units.",
        context = "Creates a value from already-scaled integer units. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(units = "`units` supplies the units value used to create a value from already-scaled integer units."),
        returns = "A newly constructed `FixedValue` configured to create a value from already-scaled integer units.",
        example = "use sand::prelude::*;\n\nfn demonstrate(units: i64)  {\n    let fixed_value = sand::entity::FixedValue::from_units(units);\n}",
    )]
    #[must_use]
    pub const fn from_units(units: i64) -> Self {
        Self(units)
    }

    /// Returns the already-scaled integer representation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedValue::units",
        aliases = ["sand::prelude::FixedValue::units"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the already-scaled integer representation.",
        context = "Returns the already-scaled integer representation. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns the already-scaled integer representation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_value_value: sand::entity::FixedValue)  {\n    let units = fixed_value_value.units();\n}",
    )]
    #[must_use]
    pub const fn units(self) -> i64 {
        self.0
    }

    /// Returns this value as a host floating-point number for inspection.
    ///
    /// Exported arithmetic should use [`Self::units`] instead.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::FixedValue::as_f64",
        aliases = ["sand::prelude::FixedValue::as_f64"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns this value as a host floating-point number for inspection.",
        context = "Returns this value as a host floating-point number for inspection. Exported arithmetic should use [`Self::units`] instead.",
        minecraft = "Exported arithmetic should use [`Self::units`] instead.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(fixed = "`fixed` provides the fixed-value inputs used to return this value as a host floating-point number for inspection."),
        returns = "Returns this value as a host floating-point number for inspection.",
        example = "use sand::prelude::*;\n\nfn demonstrate(fixed_value_value: sand::entity::FixedValue, fixed: sand::entity::FixedPoint)  {\n    let as_f64 = fixed_value_value.as_f64(fixed);\n}",
    )]
    #[must_use]
    pub fn as_f64(self, fixed: FixedPoint) -> f64 {
        self.0 as f64 / fixed.scale as f64
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::CurveInputs",
    aliases = ["sand::prelude::CurveInputs"],
    module = "sand::entity",
    summary = "Deterministic named values supplied to [`StatCurve::evaluate`].",
    context = "Deterministic named values supplied to [`StatCurve::evaluate`]. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::CurveInputs;",
)]
/// Deterministic named values supplied to [`StatCurve::evaluate`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurveInputs {
    values: BTreeMap<String, FixedValue>,
}

impl CurveInputs {
    /// Creates an empty input set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::CurveInputs::new",
        aliases = ["sand::prelude::CurveInputs::new"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates an empty input set.",
        context = "Creates an empty input set. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "A newly constructed `CurveInputs` configured to create an empty input set.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let curve_inputs = sand::entity::CurveInputs::new();\n}",
    )]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Inserts an already-scaled value, replacing a value with the same name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::CurveInputs::insert",
        aliases = ["sand::prelude::CurveInputs::insert"],
        module = "sand::entity",
        kind = "method",
        summary = "Inserts an already-scaled value, replacing a value with the same name.",
        context = "Inserts an already-scaled value, replacing a value with the same name. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "`name` provides the author-visible text value used to insert an already-scaled value, replacing a value with the same name.", value = "`value` provides the value being applied or compared used to insert an already-scaled value, replacing a value with the same name."),
        returns = "The matching value used to insert an already-scaled value, replacing a value with the same name, or `None` when that value is unavailable.",
        example = "use sand::prelude::*;\n\nfn demonstrate(curve_inputs_value: &mut sand::entity::CurveInputs, name: impl Into < String >, value: sand::entity::FixedValue)  {\n    let insert = curve_inputs_value.insert(name, value);\n}",
    )]
    pub fn insert(&mut self, name: impl Into<String>, value: FixedValue) -> Option<FixedValue> {
        self.values.insert(name.into(), value)
    }

    /// Inserts a whole scoreboard value after applying `fixed`'s scale.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::CurveInputs::insert_score",
        aliases = ["sand::prelude::CurveInputs::insert_score"],
        module = "sand::entity",
        kind = "method",
        summary = "Inserts a whole scoreboard value after applying `fixed`'s scale.",
        context = "Inserts a whole scoreboard value after applying `fixed`'s scale. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "`name` provides the author-visible text value used to insert a whole scoreboard value after applying `fixed`'s scale.", value = "`value` provides the value being applied or compared used to insert a whole scoreboard value after applying `fixed`'s scale.", fixed = "Inserts a whole scoreboard value after applying `fixed`'s scale.", archetype = "`archetype` provides the entity archetype supplying the property used to insert a whole scoreboard value after applying `fixed`'s scale.", derivation = "`derivation` provides the derived-stat selector used to insert a whole scoreboard value after applying `fixed`'s scale."),
        returns = "On success, the value produced to insert a whole scoreboard value after applying `fixed`'s scale; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(curve_inputs_value: &mut sand::entity::CurveInputs, name: impl Into < String >, value: i64, fixed: sand::entity::FixedPoint, archetype: & str, derivation: & str)  {\n    let insert_score = curve_inputs_value.insert_score(name, value, fixed, archetype, derivation);\n}",
    )]
    pub fn insert_score(
        &mut self,
        name: impl Into<String>,
        value: i64,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<Option<FixedValue>, EntityDiagnostic> {
        let value = fixed.encode_score(value, archetype, derivation)?;
        Ok(self.insert(name, value))
    }

    /// Returns the fixed curve input registered under `name`, when present.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::CurveInputs::get",
        aliases = ["sand::prelude::CurveInputs::get"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns the fixed curve input registered under `name`, when present.",
        context = "Returns the fixed curve input registered under `name`, when present. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "Returns the fixed curve input registered under `name`, when present."),
        returns = "Returns the fixed curve input registered under `name`, when present.",
        example = "use sand::prelude::*;\n\nfn demonstrate(curve_inputs_value: &sand::entity::CurveInputs, name: & str)  {\n    let get = curve_inputs_value.get(name);\n}",
    )]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<FixedValue> {
        self.values.get(name).copied()
    }

    /// Iterates in lexical key order.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::CurveInputs::iter",
        aliases = ["sand::prelude::CurveInputs::iter"],
        module = "sand::entity",
        kind = "method",
        summary = "Iterates in lexical key order.",
        context = "Iterates in lexical key order. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "The `impl Iterator < Item = (& str , FixedValue) >` value produced to iterate in lexical key order.",
        example = "use sand::prelude::*;\n\nfn demonstrate(curve_inputs_value: &sand::entity::CurveInputs)  {\n    let iter = curve_inputs_value.iter();\n}",
    )]
    pub fn iter(&self) -> impl Iterator<Item = (&str, FixedValue)> {
        self.values
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::CurveEvaluationError",
    aliases = ["sand::prelude::CurveEvaluationError"],
    module = "sand::entity",
    summary = "Failure while evaluating an otherwise structurally valid curve.",
    context = "Failure while evaluating an otherwise structurally valid curve. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
    minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::CurveEvaluationError;",
    variants(Custom = "A custom callback rejected its inputs.", Diagnostic = "A standard entity compilation diagnostic.", DivisionByZero = "A ratio attempted to divide by zero.", MissingInput = "A referenced state input was not supplied."),
    variant_fields(Custom(callback = "Stable registered callback identifier.", message = "Callback-provided failure detail."), Diagnostic = ["A standard entity compilation diagnostic."], DivisionByZero(archetype = "Archetype resource identifier.", derivation = "Derivation identifier."), MissingInput(archetype = "Archetype resource identifier.", derivation = "Derivation identifier.", input = "Missing state/input name.")),
)]
/// Failure while evaluating an otherwise structurally valid curve.
#[derive(Debug, Clone, PartialEq, Error)]
#[non_exhaustive]
pub enum CurveEvaluationError {
    /// A referenced state input was not supplied.
    #[error("curve `{derivation}` for `{archetype}` is missing input `{input}`")]
    MissingInput {
        /// Archetype resource identifier.
        archetype: String,
        /// Derivation identifier.
        derivation: String,
        /// Missing state/input name.
        input: String,
    },
    /// A ratio attempted to divide by zero.
    #[error("curve `{derivation}` for `{archetype}` divided by zero")]
    DivisionByZero {
        /// Archetype resource identifier.
        archetype: String,
        /// Derivation identifier.
        derivation: String,
    },
    /// A standard entity compilation diagnostic.
    #[error(transparent)]
    Diagnostic(
        #[doc = "A standard entity compilation diagnostic."]
        #[from]
        EntityDiagnostic,
    ),
    /// A custom callback rejected its inputs.
    #[error("custom curve `{callback}` failed: {message}")]
    Custom {
        /// Stable registered callback identifier.
        callback: String,
        /// Callback-provided failure detail.
        message: String,
    },
}

/// Backend family preferred by a curve's shape.
///
/// This is introspection for an exporter, not a promise that commands have
/// already been generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum LoweringStrategy {
    /// Direct scoreboard constants and arithmetic.
    ScoreboardArithmetic,
    /// A balanced decision tree for ranges or discrete mappings.
    BalancedBranches,
    /// A generated namespaced storage table plus bounded lookup helper.
    StorageLookupTable,
    /// A user-registered typed callback/function.
    CustomCallback,
}

/// Exporter-facing fixed-point operation produced by
/// [`StatCurve::lower_scoreboard`].
///
/// Operations are ordered and refer to entity score objectives on `@s`.
/// Keeping rounding and overflow explicit lets the Minecraft backend expand
/// the small arithmetic operations according to the selected version profile
/// instead of baking host arithmetic into command strings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum LoweredCurveOperation {
    /// Set an objective to an already-scaled fixed-point constant.
    SetConstant {
        /// Destination score objective.
        destination: String,
        /// Constant fixed-point units.
        value: FixedValue,
    },
    /// Copy one `@s` score to another.
    Copy {
        /// Destination score objective.
        destination: String,
        /// Source score objective.
        source: String,
    },
    /// Convert a whole entity score to fixed-point units.
    ScoreToFixed {
        /// Destination fixed-point scratch objective.
        destination: String,
        /// Existing whole-score objective.
        source: String,
        /// Positive fixed-point scale.
        scale: i64,
        /// Overflow behavior required by the definition.
        overflow: OverflowPolicy,
    },
    /// Add one score into another.
    Add {
        /// Score modified in place.
        destination: String,
        /// Score added to the destination.
        source: String,
        /// Overflow behavior required by the definition.
        overflow: OverflowPolicy,
    },
    /// Multiply two fixed-point scores and divide by the scale.
    MultiplyFixed {
        /// Score modified in place.
        destination: String,
        /// Fixed-point factor.
        factor: String,
        /// Scale used by both operands.
        scale: i64,
        /// Rounding required after division by `scale`.
        rounding: RoundingPolicy,
        /// Overflow behavior required by the definition.
        overflow: OverflowPolicy,
    },
    /// Compute `numerator × scale ÷ denominator`.
    RatioFixed {
        /// Destination score objective.
        destination: String,
        /// Numerator score objective.
        numerator: String,
        /// Denominator score objective.
        denominator: String,
        /// Fixed-point scale.
        scale: i64,
        /// Rounding required after division.
        rounding: RoundingPolicy,
        /// Overflow behavior required by the definition.
        overflow: OverflowPolicy,
    },
    /// Clamp a score to inclusive fixed-point bounds.
    Clamp {
        /// Score modified in place.
        destination: String,
        /// Inclusive lower bound.
        minimum: FixedValue,
        /// Inclusive upper bound.
        maximum: FixedValue,
    },
    /// Select a constant using inclusive minimum bands.
    ///
    /// A backend should lower this to a balanced range tree, not one branch
    /// for every possible level.
    SelectStepped {
        /// Destination score objective.
        destination: String,
        /// Input score objective.
        input: String,
        /// Sorted `(inclusive minimum, output)` pairs.
        bands: Vec<(FixedValue, FixedValue)>,
        /// Value below the first band.
        below: FixedValue,
    },
    /// Select one precomputed branch using inclusive upper bounds.
    ///
    /// The operation represents a balanced decision tree even though the
    /// branch list is stored in sorted semantic order.
    SelectPiecewise {
        /// Destination score objective.
        destination: String,
        /// Input score objective.
        input: String,
        /// Sorted `(inclusive maximum, branch result objective)` pairs.
        branches: Vec<(FixedValue, String)>,
        /// Result objective used above the last bound.
        fallback: String,
    },
    /// Read a bounded table keyed by a whole scoreboard value.
    ///
    /// The Minecraft backend should materialize this as one namespaced storage
    /// table and a generated lookup helper/function macro where supported.
    LookupTable {
        /// Destination score objective.
        destination: String,
        /// Input score objective.
        input: String,
        /// Sorted whole-score keys and fixed-point outputs.
        entries: Vec<(i64, FixedValue)>,
        /// Value used when the key is absent.
        fallback: FixedValue,
    },
    /// Select a value from stable typed enum encodings.
    SelectEnum {
        /// Destination score objective.
        destination: String,
        /// Input score objective.
        input: String,
        /// Sorted enum-score encodings and fixed-point outputs.
        entries: Vec<(i32, FixedValue)>,
        /// Value used for an unknown encoding.
        fallback: FixedValue,
    },
    /// Select between disabled and enabled values.
    SelectFlag {
        /// Destination score objective.
        destination: String,
        /// Input score objective; zero is disabled and nonzero is enabled.
        input: String,
        /// Disabled output.
        disabled: FixedValue,
        /// Enabled output.
        enabled: FixedValue,
    },
    /// Invoke a registered typed custom curve callback/helper.
    Custom {
        /// Destination score objective.
        destination: String,
        /// Stable callback identifier.
        callback: String,
        /// Named input objectives in lexical order.
        inputs: Vec<String>,
    },
}

/// Deterministic structured plan for lowering a curve to entity scoreboards.
///
/// `scratch_objectives` must be provisioned as dummy objectives at load. The
/// ordered operations execute as the entity bound to `@s`; no global scratch
/// score holder or storage compound is shared between entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoweredCurve {
    target_objective: String,
    scratch_objectives: Vec<String>,
    operations: Vec<LoweredCurveOperation>,
    strategy: LoweringStrategy,
}

impl LoweredCurve {
    /// Existing objective that receives the final fixed-point result.
    #[must_use]
    pub(crate) fn target_objective(&self) -> &str {
        &self.target_objective
    }

    /// Generated dummy objectives required at load, in lexical order.
    #[must_use]
    pub(crate) fn scratch_objectives(&self) -> &[String] {
        &self.scratch_objectives
    }

    /// Ordered entity-scoped operations.
    #[must_use]
    pub(crate) fn operations(&self) -> &[LoweredCurveOperation] {
        &self.operations
    }
}

type CurveCallback =
    dyn Fn(&CurveInputs, FixedPoint) -> Result<FixedValue, CurveEvaluationError> + Send + Sync;

#[derive(Clone)]
struct CustomCurve {
    id: String,
    inputs: BTreeSet<String>,
    callback: Arc<CurveCallback>,
}

impl fmt::Debug for CustomCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomCurve")
            .field("id", &self.id)
            .field("inputs", &self.inputs)
            .finish()
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::entity::StatCurve",
    aliases = ["sand::prelude::StatCurve"],
    module = "sand::entity",
    summary = "Pure typed IR for a derived numeric or discrete entity property.",
    context = "Pure typed IR for a derived numeric or discrete entity property. Constructors intentionally accept typed curves and fixed-point constants, rather than command strings. Call [`Self::validate`] before export; Sand chooses the compact Minecraft backend internally.",
    minecraft = "Constructors intentionally accept typed curves and fixed-point constants, rather than command strings. Call [`Self::validate`] before export; Sand chooses the compact Minecraft backend internally.",
    use_when = ["Defining or using typed entity behavior in a Sand datapack"],
    avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
    example = "use sand::entity::StatCurve;",
)]
/// Pure typed IR for a derived numeric or discrete entity property.
///
/// Constructors intentionally accept typed curves and fixed-point constants,
/// rather than command strings. Call [`Self::validate`] before export; Sand
/// chooses the compact Minecraft backend internally.
#[derive(Clone, Debug)]
pub struct StatCurve {
    kind: CurveKind,
}

#[derive(Clone, Debug)]
enum CurveKind {
    Constant(f64),
    Input(String),
    Linear {
        input: Box<StatCurve>,
        slope: f64,
        intercept: f64,
        clamp: Option<(f64, f64)>,
    },
    Add(Vec<StatCurve>),
    Multiply(Vec<StatCurve>),
    Ratio {
        numerator: Box<StatCurve>,
        denominator: Box<StatCurve>,
    },
    Stepped {
        input: Box<StatCurve>,
        bands: Vec<(f64, f64)>,
        below: f64,
    },
    Piecewise {
        input: Box<StatCurve>,
        branches: Vec<(f64, StatCurve)>,
        fallback: Box<StatCurve>,
    },
    Lookup {
        input: String,
        entries: Vec<(i64, f64)>,
        fallback: f64,
    },
    EnumMap {
        input: String,
        entries: Vec<(i32, f64)>,
        fallback: f64,
    },
    FlagMap {
        input: String,
        disabled: f64,
        enabled: f64,
    },
    Custom(CustomCurve),
}

impl StatCurve {
    /// Creates a fixed derived value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::constant",
        aliases = ["sand::prelude::StatCurve::constant"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a fixed derived value.",
        context = "Creates a fixed derived value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(value = "`value` provides the value being applied or compared used to create a fixed derived value."),
        returns = "A newly constructed `StatCurve` configured to create a fixed derived value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: f64)  {\n    let stat_curve = sand::entity::StatCurve::constant(value);\n}",
    )]
    #[must_use]
    pub fn constant(value: f64) -> Self {
        Self {
            kind: CurveKind::Constant(value),
        }
    }

    /// References a typed entity-state input.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::state",
        aliases = ["sand::prelude::StatCurve::state"],
        module = "sand::entity",
        kind = "method",
        summary = "References a typed entity-state input.",
        context = "References a typed entity-state input. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(field = "`field` supplies the field value used to reference a typed entity-state input."),
        returns = "A newly constructed `StatCurve` configured to reference a typed entity-state input.",
        example = "use sand::prelude::*;\n\nfn demonstrate(field: impl sand::entity::EntityStateField)  {\n    let stat_curve = sand::entity::StatCurve::state(field);\n}",
    )]
    #[must_use]
    pub fn state(field: impl super::EntityStateField) -> Self {
        Self {
            kind: CurveKind::Input(field.objective()),
        }
    }

    /// References an explicitly raw objective name.
    ///
    /// Prefer [`Self::state`] for schema fields.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::input_raw",
        aliases = ["sand::prelude::StatCurve::input_raw"],
        module = "sand::entity",
        kind = "method",
        summary = "References an explicitly raw objective name. Prefer [`Self::state`] for schema fields.",
        context = "References an explicitly raw objective name. Prefer [`Self::state`] for schema fields. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Prefer [`Self::state`] for schema fields."],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(name = "`name` provides the author-visible text value used to reference an explicitly raw objective name. Prefer [`Self::state`] for schema fields."),
        returns = "A newly constructed `StatCurve` configured to reference an explicitly raw objective name. Prefer [`Self::state`] for schema fields.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & str)  {\n    let stat_curve = sand::entity::StatCurve::input_raw(name);\n}",
    )]
    #[must_use]
    pub fn input_raw(name: &str) -> Self {
        Self {
            kind: CurveKind::Input(name.to_owned()),
        }
    }

    /// Creates `input × slope + intercept`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::linear",
        aliases = ["sand::prelude::StatCurve::linear"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates `input × slope + intercept`.",
        context = "Creates `input × slope + intercept`. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create `input × slope + intercept`.", slope = "`slope` supplies the slope value used to create `input × slope + intercept`.", intercept = "`intercept` supplies the intercept value used to create `input × slope + intercept`."),
        returns = "A newly constructed `StatCurve` configured to create `input × slope + intercept`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::entity::StatCurve, slope: f64, intercept: f64)  {\n    let stat_curve = sand::entity::StatCurve::linear(input, slope, intercept);\n}",
    )]
    #[must_use]
    pub fn linear(input: Self, slope: f64, intercept: f64) -> Self {
        Self {
            kind: CurveKind::Linear {
                input: Box::new(input),
                slope,
                intercept,
                clamp: None,
            },
        }
    }

    /// Creates an affine curve clamped to the inclusive `[minimum, maximum]`.
    ///
    /// [`Self::validate`] rejects inverted bounds.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::clamped_linear",
        aliases = ["sand::prelude::StatCurve::clamped_linear"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates an affine curve clamped to the inclusive `[minimum, maximum]`.",
        context = "Creates an affine curve clamped to the inclusive `[minimum, maximum]`. [`Self::validate`] rejects inverted bounds.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create an affine curve clamped to the inclusive `[minimum, maximum]`.", slope = "`slope` supplies the slope value used to create an affine curve clamped to the inclusive `[minimum, maximum]`.", intercept = "`intercept` supplies the intercept value used to create an affine curve clamped to the inclusive `[minimum, maximum]`.", minimum = "`minimum` supplies the minimum value used to create an affine curve clamped to the inclusive `[minimum, maximum]`.", maximum = "`maximum` supplies the maximum value used to create an affine curve clamped to the inclusive `[minimum, maximum]`."),
        returns = "A newly constructed `StatCurve` configured to create an affine curve clamped to the inclusive `[minimum, maximum]`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::entity::StatCurve, slope: f64, intercept: f64, minimum: f64, maximum: f64)  {\n    let stat_curve = sand::entity::StatCurve::clamped_linear(input, slope, intercept, minimum, maximum);\n}",
    )]
    #[must_use]
    pub fn clamped_linear(
        input: Self,
        slope: f64,
        intercept: f64,
        minimum: f64,
        maximum: f64,
    ) -> Self {
        Self {
            kind: CurveKind::Linear {
                input: Box::new(input),
                slope,
                intercept,
                clamp: Some((minimum, maximum)),
            },
        }
    }

    /// Adds all modifiers. An empty sum evaluates to zero.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::add",
        aliases = ["sand::prelude::StatCurve::add"],
        module = "sand::entity",
        kind = "method",
        summary = "Adds all modifiers. An empty sum evaluates to zero.",
        context = "Adds all modifiers. An empty sum evaluates to zero. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(terms = "`terms` supplies the terms value used to add all modifiers. An empty sum evaluates to zero."),
        returns = "A newly constructed `StatCurve` configured to add all modifiers. An empty sum evaluates to zero.",
        example = "use sand::prelude::*;\n\nfn demonstrate(terms: impl IntoIterator < Item = sand::entity::StatCurve >)  {\n    let stat_curve = sand::entity::StatCurve::add(terms);\n}",
    )]
    #[must_use]
    pub fn add(terms: impl IntoIterator<Item = Self>) -> Self {
        Self {
            kind: CurveKind::Add(terms.into_iter().collect()),
        }
    }

    /// Multiplies fixed-point factors. An empty product evaluates to one.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::multiply",
        aliases = ["sand::prelude::StatCurve::multiply"],
        module = "sand::entity",
        kind = "method",
        summary = "Multiplies fixed-point factors. An empty product evaluates to one.",
        context = "Multiplies fixed-point factors. An empty product evaluates to one. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(factors = "`factors` supplies the factors value used to multiplie fixed-point factors. An empty product evaluates to one."),
        returns = "A newly constructed `StatCurve` configured to multiplie fixed-point factors. An empty product evaluates to one.",
        example = "use sand::prelude::*;\n\nfn demonstrate(factors: impl IntoIterator < Item = sand::entity::StatCurve >)  {\n    let stat_curve = sand::entity::StatCurve::multiply(factors);\n}",
    )]
    #[must_use]
    pub fn multiply(factors: impl IntoIterator<Item = Self>) -> Self {
        Self {
            kind: CurveKind::Multiply(factors.into_iter().collect()),
        }
    }

    /// Divides one fixed-point curve by another while preserving the scale.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::ratio",
        aliases = ["sand::prelude::StatCurve::ratio"],
        module = "sand::entity",
        kind = "method",
        summary = "Divides one fixed-point curve by another while preserving the scale.",
        context = "Divides one fixed-point curve by another while preserving the scale. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(numerator = "`numerator` supplies the numerator value used to divide one fixed-point curve by another while preserving the scale.", denominator = "`denominator` supplies the denominator value used to divide one fixed-point curve by another while preserving the scale."),
        returns = "A newly constructed `StatCurve` configured to divide one fixed-point curve by another while preserving the scale.",
        example = "use sand::prelude::*;\n\nfn demonstrate(numerator: sand::entity::StatCurve, denominator: sand::entity::StatCurve)  {\n    let stat_curve = sand::entity::StatCurve::ratio(numerator, denominator);\n}",
    )]
    #[must_use]
    pub fn ratio(numerator: Self, denominator: Self) -> Self {
        Self {
            kind: CurveKind::Ratio {
                numerator: Box::new(numerator),
                denominator: Box::new(denominator),
            },
        }
    }

    /// Creates a level-band curve.
    ///
    /// Each pair is `(inclusive minimum input, output)` in strictly increasing
    /// order. [`Self::validate`] rejects duplicate or descending bounds.
    /// `below` is used before the first band.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::stepped",
        aliases = ["sand::prelude::StatCurve::stepped"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a level-band curve. Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band.",
        context = "Creates a level-band curve. Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create a level-band curve. Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band.", bands = "`bands` supplies the bands value used to create a level-band curve. Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band.", below = "Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band."),
        returns = "A newly constructed `StatCurve` configured to create a level-band curve. Each pair is `(inclusive minimum input, output)` in strictly increasing order. [`Self::validate`] rejects duplicate or descending bounds. `below` is used before the first band.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::entity::StatCurve, bands: Vec < (f64 , f64) >, below: f64)  {\n    let stat_curve = sand::entity::StatCurve::stepped(input, bands, below);\n}",
    )]
    #[must_use]
    pub fn stepped(input: Self, bands: Vec<(f64, f64)>, below: f64) -> Self {
        Self {
            kind: CurveKind::Stepped {
                input: Box::new(input),
                bands,
                below,
            },
        }
    }

    /// Creates a piecewise curve selected by inclusive upper bounds.
    ///
    /// Each pair is `(maximum input, branch)`. The fallback handles values
    /// above the final bound.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::piecewise",
        aliases = ["sand::prelude::StatCurve::piecewise"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a piecewise curve selected by inclusive upper bounds.",
        context = "Creates a piecewise curve selected by inclusive upper bounds. Each pair is `(maximum input, branch)`. The fallback handles values above the final bound.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create a piecewise curve selected by inclusive upper bounds.", branches = "`branches` supplies the branches value used to create a piecewise curve selected by inclusive upper bounds.", fallback = "`fallback` supplies the fallback value used to create a piecewise curve selected by inclusive upper bounds."),
        returns = "A newly constructed `StatCurve` configured to create a piecewise curve selected by inclusive upper bounds.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::entity::StatCurve, branches: Vec < (f64 , sand::entity::StatCurve) >, fallback: sand::entity::StatCurve)  {\n    let stat_curve = sand::entity::StatCurve::piecewise(input, branches, fallback);\n}",
    )]
    #[must_use]
    pub fn piecewise(input: Self, branches: Vec<(f64, Self)>, fallback: Self) -> Self {
        Self {
            kind: CurveKind::Piecewise {
                input: Box::new(input),
                branches,
                fallback: Box::new(fallback),
            },
        }
    }

    /// Creates a table keyed by whole scoreboard values.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::lookup",
        aliases = ["sand::prelude::StatCurve::lookup"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a table keyed by whole scoreboard values.",
        context = "Creates a table keyed by whole scoreboard values. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create a table keyed by whole scoreboard values.", entries = "`entries` supplies the entries value used to create a table keyed by whole scoreboard values.", fallback = "`fallback` supplies the fallback value used to create a table keyed by whole scoreboard values."),
        returns = "A newly constructed `StatCurve` configured to create a table keyed by whole scoreboard values.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: impl sand::entity::EntityStateField, entries: impl IntoIterator < Item = (i64 , f64) >, fallback: f64)  {\n    let stat_curve = sand::entity::StatCurve::lookup(input, entries, fallback);\n}",
    )]
    #[must_use]
    pub fn lookup(
        input: impl super::EntityStateField,
        entries: impl IntoIterator<Item = (i64, f64)>,
        fallback: f64,
    ) -> Self {
        Self::lookup_raw(&input.objective(), entries, fallback)
    }

    /// Creates a lookup table from an explicitly raw objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::lookup_raw",
        aliases = ["sand::prelude::StatCurve::lookup_raw"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a lookup table from an explicitly raw objective.",
        context = "Creates a lookup table from an explicitly raw objective. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to create a lookup table from an explicitly raw objective.", entries = "`entries` supplies the entries value used to create a lookup table from an explicitly raw objective.", fallback = "`fallback` supplies the fallback value used to create a lookup table from an explicitly raw objective."),
        returns = "A newly constructed `StatCurve` configured to create a lookup table from an explicitly raw objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: & str, entries: impl IntoIterator < Item = (i64 , f64) >, fallback: f64)  {\n    let stat_curve = sand::entity::StatCurve::lookup_raw(input, entries, fallback);\n}",
    )]
    #[must_use]
    pub fn lookup_raw(
        input: &str,
        entries: impl IntoIterator<Item = (i64, f64)>,
        fallback: f64,
    ) -> Self {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by_key(|(key, _)| *key);
        Self {
            kind: CurveKind::Lookup {
                input: input.to_owned(),
                entries,
                fallback,
            },
        }
    }

    /// Maps a stable [`super::EntityEnum`] integer encoding to a numeric value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::enum_mapping",
        aliases = ["sand::prelude::StatCurve::enum_mapping"],
        module = "sand::entity",
        kind = "method",
        summary = "Maps a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value.",
        context = "Maps a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to map a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value.", entries = "`entries` supplies the entries value used to map a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value.", fallback = "`fallback` supplies the fallback value used to map a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value."),
        returns = "A newly constructed `StatCurve` configured to map a stable [`sand::entity::EntityEnum`] integer encoding to a numeric value.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T : sand::entity::EntityEnumValue + 'static>(input: sand::entity::EntityEnum < T >, entries: impl IntoIterator < Item = (T , f64) >, fallback: f64)  {\n    let stat_curve = sand::entity::StatCurve::enum_mapping::<T>(input, entries, fallback);\n}",
    )]
    #[must_use]
    pub fn enum_mapping<T: super::EntityEnumValue>(
        input: super::EntityEnum<T>,
        entries: impl IntoIterator<Item = (T, f64)>,
        fallback: f64,
    ) -> Self {
        Self::enum_mapping_raw(
            &input.objective(),
            entries
                .into_iter()
                .map(|(value, output)| (value.encode(), output)),
            fallback,
        )
    }

    /// Maps raw enum encodings from an explicitly raw objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::enum_mapping_raw",
        aliases = ["sand::prelude::StatCurve::enum_mapping_raw"],
        module = "sand::entity",
        kind = "method",
        summary = "Maps raw enum encodings from an explicitly raw objective.",
        context = "Maps raw enum encodings from an explicitly raw objective. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to map raw enum encodings from an explicitly raw objective.", entries = "`entries` supplies the entries value used to map raw enum encodings from an explicitly raw objective.", fallback = "`fallback` supplies the fallback value used to map raw enum encodings from an explicitly raw objective."),
        returns = "A newly constructed `StatCurve` configured to map raw enum encodings from an explicitly raw objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: & str, entries: impl IntoIterator < Item = (i32 , f64) >, fallback: f64)  {\n    let stat_curve = sand::entity::StatCurve::enum_mapping_raw(input, entries, fallback);\n}",
    )]
    #[must_use]
    pub fn enum_mapping_raw(
        input: &str,
        entries: impl IntoIterator<Item = (i32, f64)>,
        fallback: f64,
    ) -> Self {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by_key(|(encoding, _)| *encoding);
        Self {
            kind: CurveKind::EnumMap {
                input: input.to_owned(),
                entries,
                fallback,
            },
        }
    }

    /// Maps a zero/one [`super::EntityFlag`] input to numeric values.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::flag_mapping",
        aliases = ["sand::prelude::StatCurve::flag_mapping"],
        module = "sand::entity",
        kind = "method",
        summary = "Maps a zero/one [`sand::entity::EntityFlag`] input to numeric values.",
        context = "Maps a zero/one [`sand::entity::EntityFlag`] input to numeric values. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to map a zero/one [`sand::entity::EntityFlag`] input to numeric values.", disabled = "`disabled` supplies the disabled value used to map a zero/one [`sand::entity::EntityFlag`] input to numeric values.", enabled = "`enabled` supplies the enabled value used to map a zero/one [`sand::entity::EntityFlag`] input to numeric values."),
        returns = "A newly constructed `StatCurve` configured to map a zero/one [`sand::entity::EntityFlag`] input to numeric values.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::entity::EntityFlag, disabled: f64, enabled: f64)  {\n    let stat_curve = sand::entity::StatCurve::flag_mapping(input, disabled, enabled);\n}",
    )]
    #[must_use]
    pub fn flag_mapping(input: super::EntityFlag, disabled: f64, enabled: f64) -> Self {
        Self::flag_mapping_raw(&input.objective(), disabled, enabled)
    }

    /// Maps a flag stored in an explicitly raw objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::flag_mapping_raw",
        aliases = ["sand::prelude::StatCurve::flag_mapping_raw"],
        module = "sand::entity",
        kind = "method",
        summary = "Maps a flag stored in an explicitly raw objective.",
        context = "Maps a flag stored in an explicitly raw objective. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(input = "`input` supplies the input value used to map a flag stored in an explicitly raw objective.", disabled = "`disabled` supplies the disabled value used to map a flag stored in an explicitly raw objective.", enabled = "`enabled` supplies the enabled value used to map a flag stored in an explicitly raw objective."),
        returns = "A newly constructed `StatCurve` configured to map a flag stored in an explicitly raw objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: & str, disabled: f64, enabled: f64)  {\n    let stat_curve = sand::entity::StatCurve::flag_mapping_raw(input, disabled, enabled);\n}",
    )]
    #[must_use]
    pub fn flag_mapping_raw(input: &str, disabled: f64, enabled: f64) -> Self {
        Self {
            kind: CurveKind::FlagMap {
                input: input.to_owned(),
                disabled,
                enabled,
            },
        }
    }

    /// Creates a typed custom evaluator with a stable registration identifier.
    ///
    /// Custom callbacks run while compiling/testing the definition. Sand
    /// registers a matching generated function during lowering. The
    /// identifier, not a function pointer address, supplies deterministic
    /// identity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::custom",
        aliases = ["sand::prelude::StatCurve::custom"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a typed custom evaluator with a stable registration identifier.",
        context = "Creates a typed custom evaluator with a stable registration identifier. Custom callbacks run while compiling/testing the definition. Sand registers a matching generated function during lowering. The identifier, not a function pointer address, supplies deterministic identity.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(function = "`function` provides the callback invoked by this operation used to create a typed custom evaluator with a stable registration identifier.", callback = "`callback` provides the callback invoked by this operation used to create a typed custom evaluator with a stable registration identifier."),
        returns = "A newly constructed `StatCurve` configured to create a typed custom evaluator with a stable registration identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function: sand::resource_ref::FunctionId, callback: impl Fn (& sand::entity::CurveInputs , sand::entity::FixedPoint) -> std::result::Result < sand::entity::FixedValue , sand::entity::CurveEvaluationError > + Send + Sync + 'static)  {\n    let stat_curve = sand::entity::StatCurve::custom(function, callback);\n}",
    )]
    #[must_use]
    pub fn custom(
        function: crate::resource_ref::FunctionId,
        callback: impl Fn(&CurveInputs, FixedPoint) -> Result<FixedValue, CurveEvaluationError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self::custom_with_raw_inputs(function, std::iter::empty::<String>(), callback)
    }

    /// Creates a typed custom evaluator with explicit state dependencies.
    ///
    /// Declaring input objective names lets dirty propagation and exporter
    /// lowering provision the callback deterministically. Use [`Self::custom`]
    /// only for callbacks that genuinely have no state inputs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::custom_with_raw_inputs",
        aliases = ["sand::prelude::StatCurve::custom_with_raw_inputs"],
        module = "sand::entity",
        kind = "method",
        summary = "Creates a typed custom evaluator with explicit state dependencies.",
        context = "Creates a typed custom evaluator with explicit state dependencies. Declaring input objective names lets dirty propagation and exporter lowering provision the callback deterministically. Use [`Self::custom`] only for callbacks that genuinely have no state inputs.",
        minecraft = "Declaring input objective names lets dirty propagation and exporter lowering provision the callback deterministically. Use [`Self::custom`] only for callbacks that genuinely have no state inputs.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(function = "`function` provides the callback invoked by this operation used to create a typed custom evaluator with explicit state dependencies.", inputs = "`inputs` provides the runtime score inputs used to create a typed custom evaluator with explicit state dependencies.", callback = "`callback` provides the callback invoked by this operation used to create a typed custom evaluator with explicit state dependencies."),
        returns = "A newly constructed `StatCurve` configured to create a typed custom evaluator with explicit state dependencies.",
        example = "use sand::prelude::*;\n\nfn demonstrate(function: sand::resource_ref::FunctionId, inputs: impl IntoIterator < Item = impl Into < String > >, callback: impl Fn (& sand::entity::CurveInputs , sand::entity::FixedPoint) -> std::result::Result < sand::entity::FixedValue , sand::entity::CurveEvaluationError > + Send + Sync + 'static)  {\n    let stat_curve = sand::entity::StatCurve::custom_with_raw_inputs(function, inputs, callback);\n}",
    )]
    #[must_use]
    pub fn custom_with_raw_inputs(
        function: crate::resource_ref::FunctionId,
        inputs: impl IntoIterator<Item = impl Into<String>>,
        callback: impl Fn(&CurveInputs, FixedPoint) -> Result<FixedValue, CurveEvaluationError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            kind: CurveKind::Custom(CustomCurve {
                id: function.to_string(),
                inputs: inputs.into_iter().map(Into::into).collect(),
                callback: Arc::new(callback),
            }),
        }
    }

    /// Validates finite constants, ordered bounds, and fixed-point
    /// representability.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::validate",
        aliases = ["sand::prelude::StatCurve::validate"],
        module = "sand::entity",
        kind = "method",
        summary = "Validates finite constants, ordered bounds, and fixed-point representability.",
        context = "Validates finite constants, ordered bounds, and fixed-point representability. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(fixed = "`fixed` provides the fixed-value inputs used to validate finite constants, ordered bounds, and fixed-point representability.", archetype = "`archetype` provides the entity archetype supplying the property used to validate finite constants, ordered bounds, and fixed-point representability.", derivation = "`derivation` provides the derived-stat selector used to validate finite constants, ordered bounds, and fixed-point representability."),
        returns = "On success, the value produced to validate finite constants, ordered bounds, and fixed-point representability; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stat_curve_value: &sand::entity::StatCurve, fixed: sand::entity::FixedPoint, archetype: & str, derivation: & str)  {\n    let validate = stat_curve_value.validate(fixed, archetype, derivation);\n}",
    )]
    pub fn validate(
        &self,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<(), EntityDiagnostic> {
        self.validate_inner(fixed, archetype, derivation)
    }

    /// Evaluates the curve using deterministic integer fixed-point arithmetic.
    ///
    /// `inputs` supplies the named state values referenced by the curve, while
    /// `fixed` selects the scale, rounding, and overflow policy. `archetype`
    /// and `derivation` name the owning definition and derived stat in any
    /// validation or evaluation diagnostic.
    ///
    /// # Example
    ///
    /// ```
    /// use sand_core::entity::{CurveInputs, FixedPoint, StatCurve};
    ///
    /// let fixed = FixedPoint::default();
    /// let curve = StatCurve::linear(StatCurve::input_raw("level"), 2.0, 10.0);
    /// let mut inputs = CurveInputs::new();
    /// inputs.insert_score("level", 5, fixed, "rpg:mob", "health")?;
    /// let value = curve.evaluate(&inputs, fixed, "rpg:mob", "health")?;
    /// assert_eq!(value.as_f64(fixed), 20.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::evaluate",
        aliases = ["sand::prelude::StatCurve::evaluate"],
        module = "sand::entity",
        kind = "method",
        summary = "Evaluates the curve using deterministic integer fixed-point arithmetic.",
        context = "Evaluates the curve using deterministic integer fixed-point arithmetic. `inputs` supplies the named state values referenced by the curve, while `fixed` selects the scale, rounding, and overflow policy. `archetype` and `derivation` name the owning definition and derived stat in any validation or evaluation diagnostic.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        params(inputs = "`inputs` supplies the named state values referenced by the curve, while `fixed` selects the scale, rounding, and overflow policy. `archetype` and `derivation` name the owning definition and derived stat in any validation or evaluation diagnostic.", fixed = "`inputs` supplies the named state values referenced by the curve, while `fixed` selects the scale, rounding, and overflow policy. `archetype` and `derivation` name the owning definition and derived stat in any validation or evaluation diagnostic.", archetype = "`inputs` supplies the named state values referenced by the curve, while `fixed` selects the scale, rounding, and overflow policy. `archetype` and `derivation` name the owning definition and derived stat in any validation or evaluation diagnostic.", derivation = "`inputs` supplies the named state values referenced by the curve, while `fixed` selects the scale, rounding, and overflow policy. `archetype` and `derivation` name the owning definition and derived stat in any validation or evaluation diagnostic."),
        returns = "On success, the value produced to evaluate the curve using deterministic integer fixed-point arithmetic; otherwise, the documented validation or export diagnostic.",
        example = "use {sand::entity::CurveInputs, sand::entity::FixedPoint, sand::entity::StatCurve};\nlet fixed = FixedPoint::default();\nlet curve = StatCurve::linear(StatCurve::input_raw(\"level\"), 2.0, 10.0);\nlet mut inputs = CurveInputs::new();\ninputs.insert_score(\"level\", 5, fixed, \"rpg:mob\", \"health\")?;\nlet value = curve.evaluate(&inputs, fixed, \"rpg:mob\", \"health\")?;\nassert_eq!(value.as_f64(fixed), 20.0);",
    )]
    pub fn evaluate(
        &self,
        inputs: &CurveInputs,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<FixedValue, CurveEvaluationError> {
        self.validate(fixed, archetype, derivation)?;
        self.evaluate_inner(inputs, fixed, archetype, derivation)
    }

    /// Returns the most capable lowering backend required by this curve.
    #[must_use]
    pub(crate) fn lowering_strategy(&self) -> LoweringStrategy {
        self.strategy_inner()
    }

    /// Produces a deterministic entity-scoreboard lowering plan.
    ///
    /// `target_objective` is an existing derived-state objective.
    /// `scratch_prefix` is a logical, archetype-namespaced prefix; generated
    /// objective tokens are collision-safe and at most 16 characters through
    /// [`sand_commands::ObjectiveName::logical`]. Input names in the curve are
    /// interpreted as existing objective names.
    ///
    /// The returned plan is execution-scoped to the entity at `@s`. It does
    /// not allocate global score holders or persistent selector references.
    pub(crate) fn lower_scoreboard(
        &self,
        target_objective: &str,
        scratch_prefix: &str,
        fixed: FixedPoint,
    ) -> Result<LoweredCurve, EntityDiagnostic> {
        self.validate(fixed, scratch_prefix, target_objective)?;
        if target_objective.trim().is_empty() || scratch_prefix.trim().is_empty() {
            return Err(EntityDiagnostic::InvalidRawExtension {
                archetype: scratch_prefix.into(),
                extension: target_objective.into(),
                detail: "curve target objective and scratch prefix must be non-empty".into(),
            });
        }
        let mut builder = CurveLoweringBuilder {
            fixed,
            scratch_prefix,
            next_scratch: 0,
            scratch_objectives: BTreeSet::new(),
            operations: Vec::new(),
        };
        let result = builder.lower(self)?;
        builder.operations.push(LoweredCurveOperation::Copy {
            destination: target_objective.into(),
            source: result,
        });
        Ok(LoweredCurve {
            target_objective: target_objective.into(),
            scratch_objectives: builder.scratch_objectives.into_iter().collect(),
            operations: builder.operations,
            strategy: self.lowering_strategy(),
        })
    }

    /// Returns all referenced named inputs in lexical order.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::entity::StatCurve::inputs",
        aliases = ["sand::prelude::StatCurve::inputs"],
        module = "sand::entity",
        kind = "method",
        summary = "Returns all referenced named inputs in lexical order.",
        context = "Returns all referenced named inputs in lexical order. This declaration belongs to Sand's typed entity model. Semantic definitions are public; selector rendering, validation bookkeeping, and compiler lowering remain internal.",
        minecraft = "Sand validates this definition and lowers it to entity-scoped selectors, scoreboards, NBT operations, and generated lifecycle functions as required.",
        use_when = ["Defining or using typed entity behavior in a Sand datapack"],
        avoid_when = ["Inspecting generated objectives, functions, or compiler lowering plans"],
        returns = "Returns all referenced named inputs in lexical order.",
        example = "use sand::prelude::*;\n\nfn demonstrate(stat_curve_value: &sand::entity::StatCurve)  {\n    let inputs = stat_curve_value.inputs();\n}",
    )]
    #[must_use]
    pub fn inputs(&self) -> BTreeSet<String> {
        let mut inputs = BTreeSet::new();
        self.collect_inputs(&mut inputs);
        inputs
    }

    fn validate_inner(
        &self,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<(), EntityDiagnostic> {
        match &self.kind {
            CurveKind::Constant(value) => {
                fixed.encode(*value, archetype, derivation)?;
            }
            CurveKind::Input(_) => {}
            CurveKind::Linear {
                input,
                slope,
                intercept,
                clamp,
            } => {
                input.validate_inner(fixed, archetype, derivation)?;
                fixed.encode(*slope, archetype, derivation)?;
                fixed.encode(*intercept, archetype, derivation)?;
                if let Some((minimum, maximum)) = clamp {
                    fixed.encode(*minimum, archetype, derivation)?;
                    fixed.encode(*maximum, archetype, derivation)?;
                    if minimum > maximum {
                        return Err(EntityDiagnostic::InvalidRange {
                            schema: archetype.into(),
                            field: derivation.into(),
                            range: format!("{minimum}..={maximum}"),
                        });
                    }
                }
            }
            CurveKind::Add(curves) | CurveKind::Multiply(curves) => {
                for curve in curves {
                    curve.validate_inner(fixed, archetype, derivation)?;
                }
            }
            CurveKind::Ratio {
                numerator,
                denominator,
            } => {
                numerator.validate_inner(fixed, archetype, derivation)?;
                denominator.validate_inner(fixed, archetype, derivation)?;
            }
            CurveKind::Stepped {
                input,
                bands,
                below,
            } => {
                input.validate_inner(fixed, archetype, derivation)?;
                fixed.encode(*below, archetype, derivation)?;
                validate_pairs(
                    bands.iter().map(|(bound, value)| (*bound, Some(*value))),
                    fixed,
                    archetype,
                    derivation,
                )?;
            }
            CurveKind::Piecewise {
                input,
                branches,
                fallback,
            } => {
                input.validate_inner(fixed, archetype, derivation)?;
                let mut previous = None;
                for (bound, branch) in branches {
                    fixed.encode(*bound, archetype, derivation)?;
                    if previous.is_some_and(|value| *bound <= value) {
                        return Err(EntityDiagnostic::InvalidRange {
                            schema: archetype.into(),
                            field: derivation.into(),
                            range: "piecewise upper bounds must be strictly increasing".into(),
                        });
                    }
                    previous = Some(*bound);
                    branch.validate_inner(fixed, archetype, derivation)?;
                }
                fallback.validate_inner(fixed, archetype, derivation)?;
            }
            CurveKind::Lookup {
                entries, fallback, ..
            } => {
                fixed.encode(*fallback, archetype, derivation)?;
                validate_unique_table_keys(
                    entries.iter().map(|(key, _)| *key),
                    archetype,
                    derivation,
                    "lookup key",
                )?;
                for (_, value) in entries {
                    fixed.encode(*value, archetype, derivation)?;
                }
            }
            CurveKind::EnumMap {
                entries, fallback, ..
            } => {
                fixed.encode(*fallback, archetype, derivation)?;
                if let Some(duplicate) = adjacent_duplicate(entries.iter().map(|(key, _)| *key)) {
                    return Err(EntityDiagnostic::InvalidEnumEncoding {
                        schema: archetype.into(),
                        field: derivation.into(),
                        detail: format!("encoding `{duplicate}` is mapped more than once"),
                    });
                }
                for (_, value) in entries {
                    fixed.encode(*value, archetype, derivation)?;
                }
            }
            CurveKind::FlagMap {
                disabled, enabled, ..
            } => {
                fixed.encode(*disabled, archetype, derivation)?;
                fixed.encode(*enabled, archetype, derivation)?;
            }
            CurveKind::Custom(custom) => {
                if custom.id.trim().is_empty() {
                    return Err(EntityDiagnostic::InvalidRawExtension {
                        archetype: archetype.into(),
                        extension: derivation.into(),
                        detail: "custom curve callback id cannot be empty".into(),
                    });
                }
            }
        }
        Ok(())
    }

    fn evaluate_inner(
        &self,
        inputs: &CurveInputs,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<FixedValue, CurveEvaluationError> {
        match &self.kind {
            CurveKind::Constant(value) => Ok(fixed.encode(*value, archetype, derivation)?),
            CurveKind::Input(name) => {
                inputs
                    .get(name)
                    .ok_or_else(|| CurveEvaluationError::MissingInput {
                        archetype: archetype.into(),
                        derivation: derivation.into(),
                        input: name.clone(),
                    })
            }
            CurveKind::Linear {
                input,
                slope,
                intercept,
                clamp,
            } => {
                let input = input.evaluate_inner(inputs, fixed, archetype, derivation)?;
                let slope = fixed.encode(*slope, archetype, derivation)?;
                let intercept = fixed.encode(*intercept, archetype, derivation)?;
                let product = multiply_fixed(input, slope, fixed, archetype, derivation)?;
                let mut result =
                    add_fixed(product, intercept, fixed.overflow, archetype, derivation)?;
                if let Some((minimum, maximum)) = clamp {
                    result = result.clamp(
                        fixed.encode(*minimum, archetype, derivation)?,
                        fixed.encode(*maximum, archetype, derivation)?,
                    );
                }
                Ok(result)
            }
            CurveKind::Add(curves) => curves.iter().try_fold(FixedValue(0), |sum, curve| {
                add_fixed(
                    sum,
                    curve.evaluate_inner(inputs, fixed, archetype, derivation)?,
                    fixed.overflow,
                    archetype,
                    derivation,
                )
                .map_err(Into::into)
            }),
            CurveKind::Multiply(curves) => {
                curves
                    .iter()
                    .try_fold(FixedValue(fixed.scale), |product, curve| {
                        multiply_fixed(
                            product,
                            curve.evaluate_inner(inputs, fixed, archetype, derivation)?,
                            fixed,
                            archetype,
                            derivation,
                        )
                        .map_err(Into::into)
                    })
            }
            CurveKind::Ratio {
                numerator,
                denominator,
            } => {
                let numerator = numerator.evaluate_inner(inputs, fixed, archetype, derivation)?;
                let denominator =
                    denominator.evaluate_inner(inputs, fixed, archetype, derivation)?;
                if denominator.0 == 0 {
                    return Err(CurveEvaluationError::DivisionByZero {
                        archetype: archetype.into(),
                        derivation: derivation.into(),
                    });
                }
                divide_rounded(
                    i128::from(numerator.0) * i128::from(fixed.scale),
                    i128::from(denominator.0),
                    fixed.rounding,
                    fixed.overflow,
                    archetype,
                    derivation,
                )
                .map_err(Into::into)
            }
            CurveKind::Stepped {
                input,
                bands,
                below,
            } => {
                let input = input.evaluate_inner(inputs, fixed, archetype, derivation)?;
                let mut value = *below;
                for (minimum, band_value) in bands {
                    if input < fixed.encode(*minimum, archetype, derivation)? {
                        break;
                    }
                    value = *band_value;
                }
                Ok(fixed.encode(value, archetype, derivation)?)
            }
            CurveKind::Piecewise {
                input,
                branches,
                fallback,
            } => {
                let input = input.evaluate_inner(inputs, fixed, archetype, derivation)?;
                let mut selected = None;
                for (maximum, branch) in branches {
                    if input <= fixed.encode(*maximum, archetype, derivation)? {
                        selected = Some(branch);
                        break;
                    }
                }
                let branch = selected.unwrap_or(fallback);
                branch.evaluate_inner(inputs, fixed, archetype, derivation)
            }
            CurveKind::Lookup {
                input,
                entries,
                fallback,
            } => {
                let value = required_input(inputs, input, archetype, derivation)?;
                let key = fixed.decode_score(value, archetype, derivation)?;
                Ok(fixed.encode(
                    entries
                        .binary_search_by_key(&key, |(entry_key, _)| *entry_key)
                        .ok()
                        .map_or(*fallback, |index| entries[index].1),
                    archetype,
                    derivation,
                )?)
            }
            CurveKind::EnumMap {
                input,
                entries,
                fallback,
            } => {
                let value = required_input(inputs, input, archetype, derivation)?;
                let key = fixed.decode_score(value, archetype, derivation)?;
                let mapped = i32::try_from(key)
                    .ok()
                    .and_then(|key| {
                        entries
                            .binary_search_by_key(&key, |(encoding, _)| *encoding)
                            .ok()
                            .map(|index| entries[index].1)
                    })
                    .unwrap_or(*fallback);
                Ok(fixed.encode(mapped, archetype, derivation)?)
            }
            CurveKind::FlagMap {
                input,
                disabled,
                enabled,
            } => {
                let value = required_input(inputs, input, archetype, derivation)?;
                Ok(fixed.encode(
                    if value.0 == 0 { *disabled } else { *enabled },
                    archetype,
                    derivation,
                )?)
            }
            CurveKind::Custom(custom) => {
                (custom.callback)(inputs, fixed).map_err(|error| match error {
                    CurveEvaluationError::Custom { .. } => error,
                    other => CurveEvaluationError::Custom {
                        callback: custom.id.clone(),
                        message: other.to_string(),
                    },
                })
            }
        }
    }

    fn strategy_inner(&self) -> LoweringStrategy {
        match &self.kind {
            CurveKind::Custom(_) => LoweringStrategy::CustomCallback,
            CurveKind::Lookup { .. } => LoweringStrategy::StorageLookupTable,
            CurveKind::Stepped { input, .. } => {
                combine_strategy(LoweringStrategy::BalancedBranches, input.strategy_inner())
            }
            CurveKind::Piecewise {
                input,
                branches,
                fallback,
            } => branches.iter().fold(
                combine_strategy(
                    LoweringStrategy::BalancedBranches,
                    combine_strategy(input.strategy_inner(), fallback.strategy_inner()),
                ),
                |strategy, (_, branch)| combine_strategy(strategy, branch.strategy_inner()),
            ),
            CurveKind::EnumMap { .. } | CurveKind::FlagMap { .. } => {
                LoweringStrategy::BalancedBranches
            }
            CurveKind::Linear { input, .. } => input.strategy_inner(),
            CurveKind::Add(curves) | CurveKind::Multiply(curves) => curves
                .iter()
                .fold(LoweringStrategy::ScoreboardArithmetic, |strategy, curve| {
                    combine_strategy(strategy, curve.strategy_inner())
                }),
            CurveKind::Ratio {
                numerator,
                denominator,
            } => combine_strategy(numerator.strategy_inner(), denominator.strategy_inner()),
            CurveKind::Constant(_) | CurveKind::Input(_) => LoweringStrategy::ScoreboardArithmetic,
        }
    }

    fn collect_inputs(&self, inputs: &mut BTreeSet<String>) {
        match &self.kind {
            CurveKind::Input(input)
            | CurveKind::Lookup { input, .. }
            | CurveKind::EnumMap { input, .. }
            | CurveKind::FlagMap { input, .. } => {
                inputs.insert(input.clone());
            }
            CurveKind::Linear { input, .. } | CurveKind::Stepped { input, .. } => {
                input.collect_inputs(inputs);
            }
            CurveKind::Add(curves) | CurveKind::Multiply(curves) => {
                for curve in curves {
                    curve.collect_inputs(inputs);
                }
            }
            CurveKind::Ratio {
                numerator,
                denominator,
            } => {
                numerator.collect_inputs(inputs);
                denominator.collect_inputs(inputs);
            }
            CurveKind::Piecewise {
                input,
                branches,
                fallback,
            } => {
                input.collect_inputs(inputs);
                for (_, branch) in branches {
                    branch.collect_inputs(inputs);
                }
                fallback.collect_inputs(inputs);
            }
            CurveKind::Custom(custom) => inputs.extend(custom.inputs.iter().cloned()),
            CurveKind::Constant(_) => {}
        }
    }
}

struct CurveLoweringBuilder<'a> {
    fixed: FixedPoint,
    scratch_prefix: &'a str,
    next_scratch: usize,
    scratch_objectives: BTreeSet<String>,
    operations: Vec<LoweredCurveOperation>,
}

impl CurveLoweringBuilder<'_> {
    fn scratch(&mut self) -> String {
        let logical = format!("{}.curve.{}", self.scratch_prefix, self.next_scratch);
        self.next_scratch += 1;
        let objective = sand_commands::ObjectiveName::logical(logical)
            .as_str()
            .to_string();
        self.scratch_objectives.insert(objective.clone());
        objective
    }

    fn constant(&mut self, value: f64, derivation: &str) -> Result<String, EntityDiagnostic> {
        let destination = self.scratch();
        let value = self.fixed.encode(value, self.scratch_prefix, derivation)?;
        self.operations.push(LoweredCurveOperation::SetConstant {
            destination: destination.clone(),
            value,
        });
        Ok(destination)
    }

    fn lower(&mut self, curve: &StatCurve) -> Result<String, EntityDiagnostic> {
        let destination = self.scratch();
        match &curve.kind {
            CurveKind::Constant(value) => {
                let value = self.fixed.encode(*value, self.scratch_prefix, "constant")?;
                self.operations.push(LoweredCurveOperation::SetConstant {
                    destination: destination.clone(),
                    value,
                });
            }
            CurveKind::Input(source) => {
                self.operations.push(LoweredCurveOperation::ScoreToFixed {
                    destination: destination.clone(),
                    source: source.clone(),
                    scale: self.fixed.scale(),
                    overflow: self.fixed.overflow(),
                });
            }
            CurveKind::Linear {
                input,
                slope,
                intercept,
                clamp,
            } => {
                let input = self.lower(input)?;
                self.operations.push(LoweredCurveOperation::Copy {
                    destination: destination.clone(),
                    source: input,
                });
                let slope = self.constant(*slope, "linear_slope")?;
                self.operations.push(LoweredCurveOperation::MultiplyFixed {
                    destination: destination.clone(),
                    factor: slope,
                    scale: self.fixed.scale,
                    rounding: self.fixed.rounding,
                    overflow: self.fixed.overflow,
                });
                let intercept = self.constant(*intercept, "linear_intercept")?;
                self.operations.push(LoweredCurveOperation::Add {
                    destination: destination.clone(),
                    source: intercept,
                    overflow: self.fixed.overflow,
                });
                if let Some((minimum, maximum)) = clamp {
                    self.operations.push(LoweredCurveOperation::Clamp {
                        destination: destination.clone(),
                        minimum: self.fixed.encode(
                            *minimum,
                            self.scratch_prefix,
                            "linear_minimum",
                        )?,
                        maximum: self.fixed.encode(
                            *maximum,
                            self.scratch_prefix,
                            "linear_maximum",
                        )?,
                    });
                }
            }
            CurveKind::Add(curves) => {
                self.operations.push(LoweredCurveOperation::SetConstant {
                    destination: destination.clone(),
                    value: FixedValue(0),
                });
                for curve in curves {
                    let source = self.lower(curve)?;
                    self.operations.push(LoweredCurveOperation::Add {
                        destination: destination.clone(),
                        source,
                        overflow: self.fixed.overflow,
                    });
                }
            }
            CurveKind::Multiply(curves) => {
                self.operations.push(LoweredCurveOperation::SetConstant {
                    destination: destination.clone(),
                    value: FixedValue(self.fixed.scale),
                });
                for curve in curves {
                    let factor = self.lower(curve)?;
                    self.operations.push(LoweredCurveOperation::MultiplyFixed {
                        destination: destination.clone(),
                        factor,
                        scale: self.fixed.scale,
                        rounding: self.fixed.rounding,
                        overflow: self.fixed.overflow,
                    });
                }
            }
            CurveKind::Ratio {
                numerator,
                denominator,
            } => {
                let numerator = self.lower(numerator)?;
                let denominator = self.lower(denominator)?;
                self.operations.push(LoweredCurveOperation::RatioFixed {
                    destination: destination.clone(),
                    numerator,
                    denominator,
                    scale: self.fixed.scale,
                    rounding: self.fixed.rounding,
                    overflow: self.fixed.overflow,
                });
            }
            CurveKind::Stepped {
                input,
                bands,
                below,
            } => {
                let input = self.lower(input)?;
                let bands = bands
                    .iter()
                    .map(|(minimum, value)| {
                        Ok((
                            self.fixed
                                .encode(*minimum, self.scratch_prefix, "step_minimum")?,
                            self.fixed
                                .encode(*value, self.scratch_prefix, "step_value")?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EntityDiagnostic>>()?;
                self.operations.push(LoweredCurveOperation::SelectStepped {
                    destination: destination.clone(),
                    input,
                    bands,
                    below: self
                        .fixed
                        .encode(*below, self.scratch_prefix, "step_below")?,
                });
            }
            CurveKind::Piecewise {
                input,
                branches,
                fallback,
            } => {
                let input = self.lower(input)?;
                let mut lowered_branches = Vec::with_capacity(branches.len());
                for (maximum, branch) in branches {
                    lowered_branches.push((
                        self.fixed
                            .encode(*maximum, self.scratch_prefix, "piecewise_maximum")?,
                        self.lower(branch)?,
                    ));
                }
                let fallback = self.lower(fallback)?;
                self.operations
                    .push(LoweredCurveOperation::SelectPiecewise {
                        destination: destination.clone(),
                        input,
                        branches: lowered_branches,
                        fallback,
                    });
            }
            CurveKind::Lookup {
                input,
                entries,
                fallback,
            } => {
                let entries = entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            *key,
                            self.fixed
                                .encode(*value, self.scratch_prefix, "lookup_value")?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EntityDiagnostic>>()?;
                self.operations.push(LoweredCurveOperation::LookupTable {
                    destination: destination.clone(),
                    input: input.clone(),
                    entries,
                    fallback: self.fixed.encode(
                        *fallback,
                        self.scratch_prefix,
                        "lookup_fallback",
                    )?,
                });
            }
            CurveKind::EnumMap {
                input,
                entries,
                fallback,
            } => {
                let entries = entries
                    .iter()
                    .map(|(encoding, value)| {
                        Ok((
                            *encoding,
                            self.fixed
                                .encode(*value, self.scratch_prefix, "enum_value")?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EntityDiagnostic>>()?;
                self.operations.push(LoweredCurveOperation::SelectEnum {
                    destination: destination.clone(),
                    input: input.clone(),
                    entries,
                    fallback: self
                        .fixed
                        .encode(*fallback, self.scratch_prefix, "enum_fallback")?,
                });
            }
            CurveKind::FlagMap {
                input,
                disabled,
                enabled,
            } => {
                self.operations.push(LoweredCurveOperation::SelectFlag {
                    destination: destination.clone(),
                    input: input.clone(),
                    disabled: self
                        .fixed
                        .encode(*disabled, self.scratch_prefix, "flag_disabled")?,
                    enabled: self
                        .fixed
                        .encode(*enabled, self.scratch_prefix, "flag_enabled")?,
                });
            }
            CurveKind::Custom(custom) => {
                self.operations.push(LoweredCurveOperation::Custom {
                    destination: destination.clone(),
                    callback: custom.id.clone(),
                    inputs: curve.inputs().into_iter().collect(),
                });
            }
        }
        Ok(destination)
    }
}

/// Directed source-to-dependent graph for state and derived outputs.
///
/// Names are logical identifiers owned by one archetype compiler; no global
/// registry is used. All traversal uses sorted collections, so independent
/// exports and Rust tests produce the same order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DependencyGraph {
    edges: BTreeMap<String, BTreeSet<String>>,
}

impl DependencyGraph {
    /// Creates an empty graph.
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    /// Registers a source or output even when it has no edges.
    pub(crate) fn add_node(&mut self, node: impl Into<String>) {
        self.edges.entry(node.into()).or_default();
    }

    /// Records that changing `source` dirties `dependent`.
    ///
    /// Duplicate edges are ignored, which deduplicates shared observations
    /// before refresh scheduling.
    pub(crate) fn add_dependency(
        &mut self,
        source: impl Into<String>,
        dependent: impl Into<String>,
    ) {
        let source = source.into();
        let dependent = dependent.into();
        self.edges.entry(dependent.clone()).or_default();
        self.edges.entry(source).or_default().insert(dependent);
    }

    /// Computes a stable source-before-dependent order.
    ///
    /// Cycles return [`EntityDiagnostic::DerivationCycle`] with a deterministic
    /// closed path suitable for an export diagnostic.
    pub(crate) fn topological_order(
        &self,
        archetype: &str,
    ) -> Result<Vec<String>, EntityDiagnostic> {
        if let Some(cycle) = self.find_cycle() {
            return Err(EntityDiagnostic::DerivationCycle {
                archetype: archetype.into(),
                cycle: cycle.join(" -> "),
            });
        }
        let mut indegree: BTreeMap<&str, usize> =
            self.edges.keys().map(|node| (node.as_str(), 0)).collect();
        for dependents in self.edges.values() {
            for dependent in dependents {
                *indegree.entry(dependent).or_default() += 1;
            }
        }
        let mut ready: BTreeSet<&str> = indegree
            .iter()
            .filter_map(|(node, degree)| (*degree == 0).then_some(*node))
            .collect();
        let mut order = Vec::with_capacity(indegree.len());
        while let Some(node) = ready.pop_first() {
            order.push(node.to_string());
            if let Some(dependents) = self.edges.get(node) {
                for dependent in dependents {
                    let degree = indegree
                        .get_mut(dependent.as_str())
                        .expect("registered dependency targets are graph nodes");
                    *degree -= 1;
                    if *degree == 0 {
                        ready.insert(dependent);
                    }
                }
            }
        }
        Ok(order)
    }

    fn find_cycle(&self) -> Option<Vec<String>> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Mark {
            Active,
            Done,
        }

        fn visit(
            graph: &DependencyGraph,
            node: &str,
            marks: &mut BTreeMap<String, Mark>,
            stack: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            marks.insert(node.to_string(), Mark::Active);
            stack.push(node.to_string());
            if let Some(dependents) = graph.edges.get(node) {
                for dependent in dependents {
                    match marks.get(dependent) {
                        Some(Mark::Active) => {
                            let start = stack
                                .iter()
                                .position(|entry| entry == dependent)
                                .unwrap_or(0);
                            let mut cycle = stack[start..].to_vec();
                            cycle.push(dependent.clone());
                            return Some(cycle);
                        }
                        Some(Mark::Done) => {}
                        None => {
                            if let Some(cycle) = visit(graph, dependent, marks, stack) {
                                return Some(cycle);
                            }
                        }
                    }
                }
            }
            stack.pop();
            marks.insert(node.to_string(), Mark::Done);
            None
        }

        let mut marks = BTreeMap::new();
        let mut stack = Vec::new();
        for node in self.edges.keys() {
            if !marks.contains_key(node)
                && let Some(cycle) = visit(self, node, &mut marks, &mut stack)
            {
                return Some(cycle);
            }
        }
        None
    }
}

fn required_input(
    inputs: &CurveInputs,
    input: &str,
    archetype: &str,
    derivation: &str,
) -> Result<FixedValue, CurveEvaluationError> {
    inputs
        .get(input)
        .ok_or_else(|| CurveEvaluationError::MissingInput {
            archetype: archetype.into(),
            derivation: derivation.into(),
            input: input.into(),
        })
}

fn validate_pairs(
    pairs: impl IntoIterator<Item = (f64, Option<f64>)>,
    fixed: FixedPoint,
    archetype: &str,
    derivation: &str,
) -> Result<(), EntityDiagnostic> {
    let mut previous = None;
    for (bound, value) in pairs {
        fixed.encode(bound, archetype, derivation)?;
        if let Some(value) = value {
            fixed.encode(value, archetype, derivation)?;
        }
        if previous.is_some_and(|value| bound <= value) {
            return Err(EntityDiagnostic::InvalidRange {
                schema: archetype.into(),
                field: derivation.into(),
                range: "step bounds must be strictly increasing".into(),
            });
        }
        previous = Some(bound);
    }
    Ok(())
}

fn validate_unique_table_keys(
    keys: impl IntoIterator<Item = i64>,
    archetype: &str,
    derivation: &str,
    label: &str,
) -> Result<(), EntityDiagnostic> {
    if let Some(duplicate) = adjacent_duplicate(keys) {
        return Err(EntityDiagnostic::DuplicateStateField {
            schema: archetype.into(),
            field: derivation.into(),
            detail: format!("{label} `{duplicate}` is declared more than once"),
        });
    }
    Ok(())
}

fn adjacent_duplicate<T: Copy + PartialEq>(values: impl IntoIterator<Item = T>) -> Option<T> {
    let mut previous = None;
    for value in values {
        if previous == Some(value) {
            return Some(value);
        }
        previous = Some(value);
    }
    None
}

fn combine_strategy(left: LoweringStrategy, right: LoweringStrategy) -> LoweringStrategy {
    use LoweringStrategy::{
        BalancedBranches, CustomCallback, ScoreboardArithmetic, StorageLookupTable,
    };
    match (left, right) {
        (CustomCallback, _) | (_, CustomCallback) => CustomCallback,
        (StorageLookupTable, _) | (_, StorageLookupTable) => StorageLookupTable,
        (BalancedBranches, _) | (_, BalancedBranches) => BalancedBranches,
        _ => ScoreboardArithmetic,
    }
}

fn multiply_fixed(
    left: FixedValue,
    right: FixedValue,
    fixed: FixedPoint,
    archetype: &str,
    derivation: &str,
) -> Result<FixedValue, EntityDiagnostic> {
    divide_rounded(
        i128::from(left.0) * i128::from(right.0),
        i128::from(fixed.scale),
        fixed.rounding,
        fixed.overflow,
        archetype,
        derivation,
    )
}

fn add_fixed(
    left: FixedValue,
    right: FixedValue,
    policy: OverflowPolicy,
    archetype: &str,
    derivation: &str,
) -> Result<FixedValue, EntityDiagnostic> {
    checked_i128(
        i128::from(left.0) + i128::from(right.0),
        policy,
        archetype,
        derivation,
        "addition overflowed",
    )
}

fn divide_rounded(
    numerator: i128,
    denominator: i128,
    rounding: RoundingPolicy,
    overflow_policy: OverflowPolicy,
    archetype: &str,
    derivation: &str,
) -> Result<FixedValue, EntityDiagnostic> {
    debug_assert_ne!(denominator, 0);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let adjusted = if remainder == 0 {
        quotient
    } else {
        let sign = if (numerator < 0) == (denominator < 0) {
            1
        } else {
            -1
        };
        match rounding {
            RoundingPolicy::TowardZero => quotient,
            RoundingPolicy::Floor => quotient - i128::from(sign < 0),
            RoundingPolicy::Ceiling => quotient + i128::from(sign > 0),
            RoundingPolicy::NearestTiesAwayFromZero => {
                if remainder.abs() * 2 >= denominator.abs() {
                    quotient + sign
                } else {
                    quotient
                }
            }
            RoundingPolicy::NearestTiesToEven => {
                let doubled = remainder.abs() * 2;
                if doubled > denominator.abs()
                    || (doubled == denominator.abs() && quotient % 2 != 0)
                {
                    quotient + sign
                } else {
                    quotient
                }
            }
        }
    };
    checked_i128(
        adjusted,
        overflow_policy,
        archetype,
        derivation,
        "arithmetic result overflowed",
    )
}

fn checked_i128(
    value: i128,
    policy: OverflowPolicy,
    archetype: &str,
    derivation: &str,
    detail: &str,
) -> Result<FixedValue, EntityDiagnostic> {
    match i64::try_from(value) {
        Ok(value) => Ok(FixedValue(value)),
        Err(_) if policy == OverflowPolicy::Saturate => {
            Ok(FixedValue(if value < 0 { i64::MIN } else { i64::MAX }))
        }
        Err(_) => Err(overflow(archetype, derivation, detail)),
    }
}

fn round_float(value: f64, rounding: RoundingPolicy) -> f64 {
    match rounding {
        RoundingPolicy::TowardZero => value.trunc(),
        RoundingPolicy::Floor => value.floor(),
        RoundingPolicy::Ceiling => value.ceil(),
        RoundingPolicy::NearestTiesAwayFromZero => value.round(),
        RoundingPolicy::NearestTiesToEven => value.round_ties_even(),
    }
}

fn non_finite(archetype: &str, derivation: &str, value: f64) -> EntityDiagnostic {
    EntityDiagnostic::NonFiniteCurve {
        archetype: archetype.into(),
        derivation: derivation.into(),
        value: value.to_string(),
    }
}

fn overflow(archetype: &str, derivation: &str, detail: impl Into<String>) -> EntityDiagnostic {
    EntityDiagnostic::FixedPointOverflow {
        archetype: archetype.into(),
        derivation: derivation.into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed() -> FixedPoint {
        FixedPoint::default()
    }

    fn inputs(values: &[(&str, i64)]) -> CurveInputs {
        let mut inputs = CurveInputs::new();
        for (name, value) in values {
            inputs
                .insert_score(*name, *value, fixed(), "rpg:mob", "test")
                .unwrap();
        }
        inputs
    }

    fn eval(curve: &StatCurve, values: &[(&str, i64)]) -> f64 {
        curve
            .evaluate(&inputs(values), fixed(), "rpg:mob", "test")
            .unwrap()
            .as_f64(fixed())
    }

    #[test]
    fn constants_linear_clamping_and_composition() {
        let health =
            StatCurve::clamped_linear(StatCurve::input_raw("level"), 2.5, 17.5, 20.0, 100.0);
        assert_eq!(eval(&health, &[("level", 1)]), 20.0);
        assert_eq!(eval(&health, &[("level", 10)]), 42.5);
        assert_eq!(eval(&health, &[("level", 100)]), 100.0);

        let final_health = StatCurve::add([
            StatCurve::multiply([
                health,
                StatCurve::constant(1.5),
                StatCurve::flag_mapping_raw("sick", 1.0, 0.5),
            ]),
            StatCurve::constant(3.0),
        ]);
        assert_eq!(eval(&final_health, &[("level", 10), ("sick", 1)]), 34.875);
    }

    #[test]
    fn ratio_and_rounding_are_integer_deterministic() {
        let third = StatCurve::ratio(StatCurve::constant(1.0), StatCurve::constant(3.0));
        assert_eq!(
            third
                .evaluate(&CurveInputs::new(), fixed(), "rpg:mob", "ratio")
                .unwrap()
                .units(),
            333
        );
        let even =
            FixedPoint::new(1, RoundingPolicy::NearestTiesToEven, OverflowPolicy::Error).unwrap();
        assert_eq!(
            StatCurve::constant(2.5)
                .evaluate(&CurveInputs::new(), even, "rpg:mob", "round")
                .unwrap()
                .units(),
            2
        );
        assert_eq!(
            StatCurve::constant(3.5)
                .evaluate(&CurveInputs::new(), even, "rpg:mob", "round")
                .unwrap()
                .units(),
            4
        );

        for (policy, expected) in [
            (RoundingPolicy::TowardZero, -3),
            (RoundingPolicy::Floor, -4),
            (RoundingPolicy::Ceiling, -3),
            (RoundingPolicy::NearestTiesAwayFromZero, -3),
            (RoundingPolicy::NearestTiesToEven, -3),
        ] {
            let settings = FixedPoint::new(10, policy, OverflowPolicy::Error).unwrap();
            assert_eq!(
                StatCurve::ratio(StatCurve::constant(-1.0), StatCurve::constant(3.0))
                    .evaluate(&CurveInputs::new(), settings, "rpg:mob", "negative_ratio")
                    .unwrap()
                    .units(),
                expected
            );
        }
    }

    #[test]
    fn stepped_piecewise_lookup_enum_and_flag_mapping() {
        let stepped = StatCurve::stepped(
            StatCurve::input_raw("level"),
            vec![(1.0, 10.0), (5.0, 20.0), (10.0, 40.0)],
            1.0,
        );
        assert_eq!(eval(&stepped, &[("level", 0)]), 1.0);
        assert_eq!(eval(&stepped, &[("level", 7)]), 20.0);

        let piecewise = StatCurve::piecewise(
            StatCurve::input_raw("level"),
            vec![
                (4.0, StatCurve::constant(1.0)),
                (
                    9.0,
                    StatCurve::linear(StatCurve::input_raw("level"), 2.0, 0.0),
                ),
            ],
            StatCurve::constant(99.0),
        );
        assert_eq!(eval(&piecewise, &[("level", 6)]), 12.0);
        assert_eq!(eval(&piecewise, &[("level", 10)]), 99.0);

        let table = StatCurve::lookup_raw("level", [(1, 2.0), (6, 8.0)], -1.0);
        assert_eq!(eval(&table, &[("level", 6)]), 8.0);
        assert_eq!(eval(&table, &[("level", 7)]), -1.0);

        let enum_map = StatCurve::enum_mapping_raw("rarity", [(1, 1.5), (2, 2.0)], 1.0);
        assert_eq!(eval(&enum_map, &[("rarity", 2)]), 2.0);
        let flag = StatCurve::flag_mapping_raw("sick", 1.0, 0.75);
        assert_eq!(eval(&flag, &[("sick", 1)]), 0.75);
    }

    #[test]
    fn validation_reports_non_finite_ranges_and_overflow() {
        let non_finite = StatCurve::constant(f64::NAN)
            .validate(fixed(), "rpg:mob", "health")
            .unwrap_err();
        assert_eq!(non_finite.code(), "SAND-ENTITY-CURVE-NON-FINITE");

        let range = StatCurve::clamped_linear(StatCurve::input_raw("level"), 1.0, 0.0, 10.0, 1.0)
            .validate(fixed(), "rpg:mob", "health")
            .unwrap_err();
        assert_eq!(range.code(), "SAND-ENTITY-RANGE");

        let overflow = StatCurve::constant(i64::MAX as f64)
            .validate(fixed(), "rpg:mob", "health")
            .unwrap_err();
        assert_eq!(overflow.code(), "SAND-ENTITY-FIXED-OVERFLOW");
    }

    #[test]
    fn tables_reject_duplicate_keys_instead_of_silently_overwriting() {
        let lookup = StatCurve::lookup_raw("level", [(1, 2.0), (1, 3.0)], 0.0)
            .validate(fixed(), "rpg:mob", "equipment_tier")
            .unwrap_err();
        assert_eq!(lookup.code(), "SAND-ENTITY-STATE-DUPLICATE");

        let enum_map = StatCurve::enum_mapping_raw("rarity", [(2, 1.5), (2, 2.0)], 1.0)
            .validate(fixed(), "rpg:mob", "rarity_multiplier")
            .unwrap_err();
        assert_eq!(enum_map.code(), "SAND-ENTITY-ENUM");
    }

    #[test]
    fn saturation_is_explicit() {
        let saturating =
            FixedPoint::new(1_000, RoundingPolicy::TowardZero, OverflowPolicy::Saturate).unwrap();
        assert_eq!(
            StatCurve::constant(i64::MAX as f64)
                .evaluate(&CurveInputs::new(), saturating, "rpg:mob", "huge")
                .unwrap()
                .units(),
            i64::MAX
        );
        let sum = StatCurve::add([
            StatCurve::constant(i64::MAX as f64 / 1_000.0),
            StatCurve::constant(10.0),
        ]);
        assert_eq!(
            sum.evaluate(&CurveInputs::new(), saturating, "rpg:mob", "sum")
                .unwrap()
                .units(),
            i64::MAX
        );

        let mut large = CurveInputs::new();
        large.insert("large", FixedValue::from_units(i64::MAX));
        let overflow =
            StatCurve::multiply([StatCurve::input_raw("large"), StatCurve::constant(2.0)])
                .evaluate(&large, fixed(), "rpg:mob", "multiply")
                .unwrap_err();
        assert!(matches!(
            overflow,
            CurveEvaluationError::Diagnostic(EntityDiagnostic::FixedPointOverflow { .. })
        ));
    }

    #[test]
    fn strategy_and_inputs_describe_compact_lowering() {
        let arithmetic = StatCurve::linear(StatCurve::input_raw("level"), 2.0, 10.0);
        assert_eq!(
            arithmetic.lowering_strategy(),
            LoweringStrategy::ScoreboardArithmetic
        );
        let branch = StatCurve::flag_mapping_raw("sick", 1.0, 0.5);
        assert_eq!(
            branch.lowering_strategy(),
            LoweringStrategy::BalancedBranches
        );
        let lookup = StatCurve::add([arithmetic, StatCurve::lookup_raw("rarity", [(1, 1.5)], 1.0)]);
        assert_eq!(
            lookup.lowering_strategy(),
            LoweringStrategy::StorageLookupTable
        );
        assert_eq!(
            lookup.inputs(),
            BTreeSet::from(["level".to_string(), "rarity".to_string()])
        );
        let custom = StatCurve::custom(
            "rpg:custom"
                .parse::<crate::resource_ref::FunctionId>()
                .unwrap(),
            |_inputs, fixed| Ok(FixedValue::from_units(fixed.scale())),
        );
        assert_eq!(custom.lowering_strategy(), LoweringStrategy::CustomCallback);
    }

    #[test]
    fn scoreboard_lowering_is_deterministic_and_entity_scoped() {
        let curve = StatCurve::clamped_linear(
            StatCurve::add([StatCurve::input_raw("rpg_level"), StatCurve::constant(2.0)]),
            1.5,
            10.0,
            20.0,
            100.0,
        );
        let first = curve
            .lower_scoreboard("rpg_health", "rpg:mob.health", fixed())
            .unwrap();
        let second = curve
            .lower_scoreboard("rpg_health", "rpg:mob.health", fixed())
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.strategy, LoweringStrategy::ScoreboardArithmetic);
        assert!(
            first
                .scratch_objectives()
                .iter()
                .all(|name| name.len() <= 16)
        );
        assert!(first.operations().iter().any(|operation| matches!(
            operation,
            LoweredCurveOperation::MultiplyFixed {
                scale: DEFAULT_FIXED_POINT_SCALE,
                ..
            }
        )));
        assert!(matches!(
            first.operations().last(),
            Some(LoweredCurveOperation::Copy { destination, .. })
                if destination == "rpg_health"
        ));
    }

    #[test]
    fn discrete_lowering_uses_balanced_and_table_operations() {
        let stepped = StatCurve::stepped(
            StatCurve::input_raw("rpg_level"),
            vec![(1.0, 10.0), (10.0, 30.0), (20.0, 60.0)],
            5.0,
        )
        .lower_scoreboard("rpg_tier", "rpg:mob.tier", fixed())
        .unwrap();
        assert!(stepped.operations().iter().any(|operation| matches!(
            operation,
            LoweredCurveOperation::SelectStepped { bands, .. } if bands.len() == 3
        )));

        let table = StatCurve::lookup_raw("rpg_level", [(1, 2.0), (100, 40.0)], 1.0)
            .lower_scoreboard("rpg_loot", "rpg:mob.loot", fixed())
            .unwrap();
        assert_eq!(table.strategy, LoweringStrategy::StorageLookupTable);
        assert!(table.operations().iter().any(|operation| matches!(
            operation,
            LoweredCurveOperation::LookupTable { entries, .. } if entries.len() == 2
        )));
    }

    #[test]
    fn graph_order_is_stable_and_deduplicated() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("level", "health");
        graph.add_dependency("level", "damage");
        graph.add_dependency("rarity", "damage");
        graph.add_dependency("health", "name");
        graph.add_dependency("level", "health");

        assert_eq!(
            graph.topological_order("rpg:mob").unwrap(),
            ["level", "health", "name", "rarity", "damage"]
        );
    }

    #[test]
    fn graph_cycle_is_actionable_and_deterministic() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("health", "name");
        graph.add_dependency("name", "health");
        let error = graph.topological_order("rpg:mob").unwrap_err();
        assert_eq!(error.code(), "SAND-ENTITY-DERIVATION-CYCLE");
        assert!(error.to_string().contains("health -> name -> health"));
    }

    #[test]
    fn custom_callback_is_typed_and_missing_inputs_are_reported() {
        let custom = StatCurve::custom(
            "rpg:double_level"
                .parse::<crate::resource_ref::FunctionId>()
                .unwrap(),
            |inputs, fixed| {
                let level = inputs
                    .get("level")
                    .ok_or_else(|| CurveEvaluationError::Custom {
                        callback: "rpg:double_level".into(),
                        message: "missing level".into(),
                    })?;
                Ok(FixedValue::from_units(
                    level.units() * 2 / fixed.scale() * fixed.scale(),
                ))
            },
        );
        assert_eq!(eval(&custom, &[("level", 3)]), 6.0);

        let error = StatCurve::input_raw("missing")
            .evaluate(&CurveInputs::new(), fixed(), "rpg:mob", "damage")
            .unwrap_err();
        assert!(matches!(
            error,
            CurveEvaluationError::MissingInput { input, .. } if input == "missing"
        ));
    }
}
