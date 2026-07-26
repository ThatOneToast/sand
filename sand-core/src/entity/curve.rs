//! Deterministic derived-stat curves and dirty dependency planning.
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
//! [`DependencyGraph`] complements the curve IR. Edges point from a state
//! source to an output that consumes it. A [`DirtyPlan`] first identifies all
//! transitively dirty outputs, then lists each output once in deterministic
//! topological recomputation order.
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

/// Default number of fixed-point units in one whole value.
pub const DEFAULT_FIXED_POINT_SCALE: i64 = 1_000;

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
    #[must_use]
    pub const fn scale(self) -> i64 {
        self.scale
    }

    /// Returns the rounding rule for lossy operations.
    #[must_use]
    pub const fn rounding(self) -> RoundingPolicy {
        self.rounding
    }

    /// Returns the overflow behavior for conversion and arithmetic.
    #[must_use]
    pub const fn overflow(self) -> OverflowPolicy {
        self.overflow
    }

    /// Converts a finite host value into deterministic fixed-point units.
    ///
    /// Floating point is accepted only at definition time. Runtime evaluation
    /// and generated Minecraft arithmetic use the resulting integer.
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

/// Behavior when a fixed-point result does not fit in a signed 64-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OverflowPolicy {
    /// Stop validation/evaluation with a structured diagnostic.
    Error,
    /// Clamp the result to [`i64::MIN`] or [`i64::MAX`].
    Saturate,
}

/// A signed fixed-point value.
///
/// The scale is supplied by [`FixedPoint`]. Keeping the raw representation
/// explicit prevents accidental interchange with whole scoreboard values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedValue(i64);

impl FixedValue {
    /// Creates a value from already-scaled integer units.
    #[must_use]
    pub const fn from_units(units: i64) -> Self {
        Self(units)
    }

    /// Returns the already-scaled integer representation.
    #[must_use]
    pub const fn units(self) -> i64 {
        self.0
    }

    /// Returns this value as a host floating-point number for inspection.
    ///
    /// Exported arithmetic should use [`Self::units`] instead.
    #[must_use]
    pub fn as_f64(self, fixed: FixedPoint) -> f64 {
        self.0 as f64 / fixed.scale as f64
    }
}

/// Deterministic named values supplied to [`StatCurve::evaluate`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CurveInputs {
    values: BTreeMap<String, FixedValue>,
}

impl CurveInputs {
    /// Creates an empty input set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Inserts an already-scaled value, replacing a value with the same name.
    pub fn insert(&mut self, name: impl Into<String>, value: FixedValue) -> Option<FixedValue> {
        self.values.insert(name.into(), value)
    }

    /// Inserts a whole scoreboard value after applying `fixed`'s scale.
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

    /// Returns a named value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<FixedValue> {
        self.values.get(name).copied()
    }

    /// Iterates in lexical key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, FixedValue)> {
        self.values
            .iter()
            .map(|(name, value)| (name.as_str(), *value))
    }
}

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
    Diagnostic(#[from] EntityDiagnostic),
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
pub enum LoweringStrategy {
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
pub enum LoweredCurveOperation {
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
pub struct LoweredCurve {
    target_objective: String,
    scratch_objectives: Vec<String>,
    operations: Vec<LoweredCurveOperation>,
    strategy: LoweringStrategy,
}

impl LoweredCurve {
    /// Existing objective that receives the final fixed-point result.
    #[must_use]
    pub fn target_objective(&self) -> &str {
        &self.target_objective
    }

    /// Generated dummy objectives required at load, in lexical order.
    #[must_use]
    pub fn scratch_objectives(&self) -> &[String] {
        &self.scratch_objectives
    }

    /// Ordered entity-scoped operations.
    #[must_use]
    pub fn operations(&self) -> &[LoweredCurveOperation] {
        &self.operations
    }

    /// Most capable backend family needed by this plan.
    #[must_use]
    pub const fn strategy(&self) -> LoweringStrategy {
        self.strategy
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

/// Pure typed IR for a derived numeric or discrete entity property.
///
/// Constructors intentionally accept typed curves and fixed-point constants,
/// rather than command strings. Call [`Self::validate`] before export and
/// [`Self::lowering_strategy`] to choose a compact backend.
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
    #[must_use]
    pub fn constant(value: f64) -> Self {
        Self {
            kind: CurveKind::Constant(value),
        }
    }

    /// References a typed entity-state input.
    #[must_use]
    pub fn state(field: impl super::EntityStateField) -> Self {
        Self {
            kind: CurveKind::Input(field.objective()),
        }
    }

    /// References an explicitly raw objective name.
    ///
    /// Prefer [`Self::state`] for schema fields.
    #[must_use]
    pub fn input_raw(name: &str) -> Self {
        Self {
            kind: CurveKind::Input(name.to_owned()),
        }
    }

    /// Creates `input × slope + intercept`.
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
    #[must_use]
    pub fn add(terms: impl IntoIterator<Item = Self>) -> Self {
        Self {
            kind: CurveKind::Add(terms.into_iter().collect()),
        }
    }

    /// Multiplies fixed-point factors. An empty product evaluates to one.
    #[must_use]
    pub fn multiply(factors: impl IntoIterator<Item = Self>) -> Self {
        Self {
            kind: CurveKind::Multiply(factors.into_iter().collect()),
        }
    }

    /// Divides one fixed-point curve by another while preserving the scale.
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
    #[must_use]
    pub fn lookup(
        input: impl super::EntityStateField,
        entries: impl IntoIterator<Item = (i64, f64)>,
        fallback: f64,
    ) -> Self {
        Self::lookup_raw(&input.objective(), entries, fallback)
    }

    /// Creates a lookup table from an explicitly raw objective.
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
    #[must_use]
    pub fn flag_mapping(input: super::EntityFlag, disabled: f64, enabled: f64) -> Self {
        Self::flag_mapping_raw(&input.objective(), disabled, enabled)
    }

    /// Maps a flag stored in an explicitly raw objective.
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
    /// Custom callbacks run while compiling/testing the definition. An
    /// exporter must register a matching generated function and reports
    /// [`LoweringStrategy::CustomCallback`]. The identifier, not a function
    /// pointer address, supplies deterministic identity.
    #[must_use]
    pub fn custom(
        function: crate::resource_ref::FunctionRef,
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
    #[must_use]
    pub fn custom_with_raw_inputs(
        function: crate::resource_ref::FunctionRef,
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
    pub fn validate(
        &self,
        fixed: FixedPoint,
        archetype: &str,
        derivation: &str,
    ) -> Result<(), EntityDiagnostic> {
        self.validate_inner(fixed, archetype, derivation)
    }

    /// Evaluates the curve using deterministic integer fixed-point arithmetic.
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
    pub fn lowering_strategy(&self) -> LoweringStrategy {
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
    pub fn lower_scoreboard(
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
pub struct DependencyGraph {
    edges: BTreeMap<String, BTreeSet<String>>,
}

impl DependencyGraph {
    /// Creates an empty graph.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    /// Registers a source or output even when it has no edges.
    pub fn add_node(&mut self, node: impl Into<String>) {
        self.edges.entry(node.into()).or_default();
    }

    /// Records that changing `source` dirties `dependent`.
    ///
    /// Duplicate edges are ignored, which deduplicates shared observations
    /// before refresh scheduling.
    pub fn add_dependency(&mut self, source: impl Into<String>, dependent: impl Into<String>) {
        let source = source.into();
        let dependent = dependent.into();
        self.edges.entry(dependent.clone()).or_default();
        self.edges.entry(source).or_default().insert(dependent);
    }

    /// Returns every registered node in lexical order.
    pub fn nodes(&self) -> impl Iterator<Item = &str> {
        self.edges.keys().map(String::as_str)
    }

    /// Returns direct dependents in lexical order.
    pub fn direct_dependents(&self, source: &str) -> impl Iterator<Item = &str> {
        self.edges
            .get(source)
            .into_iter()
            .flat_map(|values| values.iter().map(String::as_str))
    }

    /// Returns all transitive dependents, excluding `source`.
    #[must_use]
    pub fn transitive_dependents(&self, source: &str) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut pending = vec![source.to_string()];
        while let Some(node) = pending.pop() {
            if let Some(dependents) = self.edges.get(&node) {
                for dependent in dependents.iter().rev() {
                    if found.insert(dependent.clone()) {
                        pending.push(dependent.clone());
                    }
                }
            }
        }
        found.remove(source);
        found
    }

    /// Computes a stable source-before-dependent order.
    ///
    /// Cycles return [`EntityDiagnostic::DerivationCycle`] with a deterministic
    /// closed path suitable for an export diagnostic.
    pub fn topological_order(&self, archetype: &str) -> Result<Vec<String>, EntityDiagnostic> {
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

    /// Builds a two-phase dirty/recompute plan for changed sources.
    ///
    /// Sources can overlap and share dependents; each dirty output appears
    /// exactly once. The recomputation order contains only dirty outputs.
    pub fn dirty_plan(
        &self,
        changed_sources: impl IntoIterator<Item = impl AsRef<str>>,
        archetype: &str,
    ) -> Result<DirtyPlan, EntityDiagnostic> {
        let mut sources = BTreeSet::new();
        let mut dirty = BTreeSet::new();
        for source in changed_sources {
            let source = source.as_ref().to_string();
            dirty.extend(self.transitive_dependents(&source));
            sources.insert(source);
        }
        let recompute = self
            .topological_order(archetype)?
            .into_iter()
            .filter(|node| dirty.contains(node))
            .collect();
        Ok(DirtyPlan {
            changed_sources: sources,
            dirty_outputs: dirty,
            recompute,
        })
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

/// Two-phase result of propagating one or more changed state sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyPlan {
    changed_sources: BTreeSet<String>,
    dirty_outputs: BTreeSet<String>,
    recompute: Vec<String>,
}

impl DirtyPlan {
    /// Changed source names in lexical order.
    #[must_use]
    pub fn changed_sources(&self) -> &BTreeSet<String> {
        &self.changed_sources
    }

    /// Outputs whose generated dirty bits should be set.
    #[must_use]
    pub fn dirty_outputs(&self) -> &BTreeSet<String> {
        &self.dirty_outputs
    }

    /// Dirty outputs in source-before-dependent recomputation order.
    #[must_use]
    pub fn recompute_order(&self) -> &[String] {
        &self.recompute
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
            crate::resource_ref::FunctionRef::new("rpg:custom").unwrap(),
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
        assert_eq!(first.strategy(), LoweringStrategy::ScoreboardArithmetic);
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
        assert_eq!(table.strategy(), LoweringStrategy::StorageLookupTable);
        assert!(table.operations().iter().any(|operation| matches!(
            operation,
            LoweredCurveOperation::LookupTable { entries, .. } if entries.len() == 2
        )));
    }

    #[test]
    fn graph_order_transitive_dependencies_and_dedup_are_stable() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("level", "health");
        graph.add_dependency("level", "damage");
        graph.add_dependency("rarity", "damage");
        graph.add_dependency("health", "name");
        graph.add_dependency("level", "health");

        assert_eq!(
            graph.transitive_dependents("level"),
            BTreeSet::from([
                "damage".to_string(),
                "health".to_string(),
                "name".to_string()
            ])
        );
        assert_eq!(
            graph.topological_order("rpg:mob").unwrap(),
            ["level", "health", "name", "rarity", "damage"]
        );
        let plan = graph.dirty_plan(["level", "rarity"], "rpg:mob").unwrap();
        assert_eq!(plan.dirty_outputs().len(), 3);
        assert_eq!(plan.recompute_order(), ["health", "name", "damage"]);
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
            crate::resource_ref::FunctionRef::new("rpg:double_level").unwrap(),
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
