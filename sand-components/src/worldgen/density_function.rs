//! Builder for `data/<namespace>/worldgen/density_function/<id>.json`.
//!
//! Density functions form a small expression language. Modelling *all* vanilla
//! variants would be brittle and version-sensitive, so [`DensityFunctionExpr`]
//! covers the common, stable, reference-shaped subset — constants, references
//! to other density functions, noise sampling, the arithmetic/unary
//! transforms, `clamp`, and `y_clamped_gradient` — and
//! [`DensityFunctionExpr::raw`] is the explicit escape hatch for everything
//! else.
//!
//! # Connecting to noise settings
//!
//! [`crate::worldgen::NoiseSettings`] still accepts its `noise_router` as raw
//! JSON (see #182). Typed density functions plug straight into it, because
//! [`DensityFunctionExpr::to_json`] is public:
//!
//! ```
//! use sand_components::{
//!     DatapackComponent, DensityFunction, DensityFunctionExpr, DensityFunctionId, Noise,
//!     NoiseSettings, ResourceLocation,
//! };
//!
//! // A reusable noise parameter file...
//! let ridges = Noise::new(
//!     ResourceLocation::new("example", "ridges").unwrap(),
//!     -7,
//!     [1.0, 1.0],
//! );
//!
//! // ...sampled by a reusable density function file...
//! let density = DensityFunction::new(
//!     ResourceLocation::new("example", "ridge_density").unwrap(),
//!     DensityFunctionExpr::noise(ridges.id(), 1.0, 1.0),
//! );
//!
//! // ...referenced by name from a noise router, or inlined as JSON.
//! let settings = NoiseSettings::new(
//!     ResourceLocation::new("example", "custom_overworld").unwrap(),
//! )
//! .noise_router(serde_json::json!({
//!     "final_density": DensityFunctionExpr::reference(density.id()).to_json(),
//!     "vein_toggle": DensityFunctionExpr::constant(0.0).to_json(),
//! }));
//!
//! assert_eq!(
//!     settings.to_json()["noise_router"]["final_density"],
//!     "example:ridge_density"
//! );
//! ```

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{DensityFunctionId, NoiseId};
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/density_function";

/// A single-argument density-function transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityFunctionUnaryOp {
    /// `minecraft:abs`
    Abs,
    /// `minecraft:square`
    Square,
    /// `minecraft:cube`
    Cube,
    /// `minecraft:half_negative`
    HalfNegative,
    /// `minecraft:quarter_negative`
    QuarterNegative,
    /// `minecraft:squeeze`
    Squeeze,
}

impl DensityFunctionUnaryOp {
    fn type_id(self) -> &'static str {
        match self {
            Self::Abs => "minecraft:abs",
            Self::Square => "minecraft:square",
            Self::Cube => "minecraft:cube",
            Self::HalfNegative => "minecraft:half_negative",
            Self::QuarterNegative => "minecraft:quarter_negative",
            Self::Squeeze => "minecraft:squeeze",
        }
    }
}

/// A two-argument density-function combinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityFunctionBinaryOp {
    /// `minecraft:add`
    Add,
    /// `minecraft:mul`
    Mul,
    /// `minecraft:min`
    Min,
    /// `minecraft:max`
    Max,
}

impl DensityFunctionBinaryOp {
    fn type_id(self) -> &'static str {
        match self {
            Self::Add => "minecraft:add",
            Self::Mul => "minecraft:mul",
            Self::Min => "minecraft:min",
            Self::Max => "minecraft:max",
        }
    }
}

/// A density-function expression.
///
/// Construct values through the named helpers rather than the variants
/// directly; they keep boxing and the raw escape hatch visible at call sites.
#[derive(Debug, Clone, PartialEq)]
pub enum DensityFunctionExpr {
    /// A bare constant value, serialized as a JSON number.
    Constant(f64),
    /// A reference to another density-function file, serialized as a string.
    Reference(DensityFunctionId),
    /// `minecraft:noise` — samples a `worldgen/noise` parameter file.
    Noise {
        noise: NoiseId,
        xz_scale: f64,
        y_scale: f64,
    },
    /// A single-argument transform.
    Unary {
        op: DensityFunctionUnaryOp,
        argument: Box<DensityFunctionExpr>,
    },
    /// A two-argument combinator.
    Binary {
        op: DensityFunctionBinaryOp,
        argument1: Box<DensityFunctionExpr>,
        argument2: Box<DensityFunctionExpr>,
    },
    /// `minecraft:clamp`
    Clamp {
        input: Box<DensityFunctionExpr>,
        min: f64,
        max: f64,
    },
    /// `minecraft:y_clamped_gradient`
    YClampedGradient {
        from_y: i32,
        to_y: i32,
        from_value: f64,
        to_value: f64,
    },
    /// An explicit raw escape hatch for modded or version-specific shapes that
    /// the typed variants do not model.
    Raw(RawJson),
}

impl DensityFunctionExpr {
    /// A constant density value.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::constant` for the canonical contract."]
    pub fn constant(value: f64) -> Self {
        Self::Constant(value)
    }

    /// A reference to another `worldgen/density_function` file.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::reference` for the canonical contract."]
    pub fn reference(id: DensityFunctionId) -> Self {
        Self::Reference(id)
    }

    /// Sample a `worldgen/noise` parameter file.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::noise` for the canonical contract."]
    pub fn noise(noise: NoiseId, xz_scale: f64, y_scale: f64) -> Self {
        Self::Noise {
            noise,
            xz_scale,
            y_scale,
        }
    }

    /// Apply a single-argument transform.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::unary` for the canonical contract."]
    pub fn unary(op: DensityFunctionUnaryOp, argument: Self) -> Self {
        Self::Unary {
            op,
            argument: Box::new(argument),
        }
    }

    /// `minecraft:abs`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::abs` for the canonical contract."]
    pub fn abs(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Abs, argument)
    }

    /// `minecraft:square`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::square` for the canonical contract."]
    pub fn square(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Square, argument)
    }

    /// `minecraft:cube`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::cube` for the canonical contract."]
    pub fn cube(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Cube, argument)
    }

    /// `minecraft:half_negative`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::half_negative` for the canonical contract."]
    pub fn half_negative(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::HalfNegative, argument)
    }

    /// `minecraft:quarter_negative`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::quarter_negative` for the canonical contract."]
    pub fn quarter_negative(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::QuarterNegative, argument)
    }

    /// `minecraft:squeeze`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::squeeze` for the canonical contract."]
    pub fn squeeze(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Squeeze, argument)
    }

    /// Apply a two-argument combinator.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::binary` for the canonical contract."]
    pub fn binary(op: DensityFunctionBinaryOp, argument1: Self, argument2: Self) -> Self {
        Self::Binary {
            op,
            argument1: Box::new(argument1),
            argument2: Box::new(argument2),
        }
    }

    /// `minecraft:add`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::sum` for the canonical contract."]
    pub fn sum(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Add, argument1, argument2)
    }

    /// `minecraft:mul`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::product` for the canonical contract."]
    pub fn product(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Mul, argument1, argument2)
    }

    /// `minecraft:min`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::min` for the canonical contract."]
    pub fn min(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Min, argument1, argument2)
    }

    /// `minecraft:max`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::max` for the canonical contract."]
    pub fn max(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Max, argument1, argument2)
    }

    /// `minecraft:clamp`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::clamp` for the canonical contract."]
    pub fn clamp(input: Self, min: f64, max: f64) -> Self {
        Self::Clamp {
            input: Box::new(input),
            min,
            max,
        }
    }

    /// `minecraft:y_clamped_gradient`
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::y_clamped_gradient` for the canonical contract."]
    pub fn y_clamped_gradient(from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> Self {
        Self::YClampedGradient {
            from_y,
            to_y,
            from_value,
            to_value,
        }
    }

    /// An explicit raw escape hatch for shapes the typed variants do not model.
    ///
    /// The value is emitted unchanged; only its outer JSON kind (object,
    /// number, or string) is checked, because those are the only forms
    /// Minecraft accepts for a density function.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::raw` for the canonical contract."]
    pub fn raw(value: RawJson) -> Self {
        Self::Raw(value)
    }

    /// Serialize this expression to the JSON Minecraft expects.
    ///
    /// Public so typed expressions can be embedded in explicit raw
    /// noise-router surfaces that Sand does not yet model as author API.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunctionExpr::to_json` for the canonical contract."]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Constant(value) => serde_json::json!(value),
            Self::Reference(id) => Value::String(id.to_string()),
            Self::Noise {
                noise,
                xz_scale,
                y_scale,
            } => serde_json::json!({
                "type": "minecraft:noise",
                "noise": noise.to_string(),
                "xz_scale": xz_scale,
                "y_scale": y_scale,
            }),
            Self::Unary { op, argument } => {
                let mut map = Map::new();
                map.insert("type".into(), Value::String(op.type_id().into()));
                map.insert("argument".into(), argument.to_json());
                Value::Object(map)
            }
            Self::Binary {
                op,
                argument1,
                argument2,
            } => {
                let mut map = Map::new();
                map.insert("type".into(), Value::String(op.type_id().into()));
                map.insert("argument1".into(), argument1.to_json());
                map.insert("argument2".into(), argument2.to_json());
                Value::Object(map)
            }
            Self::Clamp { input, min, max } => serde_json::json!({
                "type": "minecraft:clamp",
                "input": input.to_json(),
                "min": min,
                "max": max,
            }),
            Self::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => serde_json::json!({
                "type": "minecraft:y_clamped_gradient",
                "from_y": from_y,
                "to_y": to_y,
                "from_value": from_value,
                "to_value": to_value,
            }),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }

    fn validate_at(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        let finite = |name: &str, value: f64| -> SandResult<()> {
            if value.is_finite() {
                return Ok(());
            }
            Err(validation::error(
                location,
                KIND,
                field,
                &format!("{name} must be finite; received {value}"),
            ))
        };
        match self {
            Self::Constant(value) => finite("constant", *value),
            Self::Reference(_) => Ok(()),
            Self::Noise {
                xz_scale, y_scale, ..
            } => {
                finite("xz_scale", *xz_scale)?;
                finite("y_scale", *y_scale)
            }
            Self::Unary { argument, .. } => argument.validate_at(location, field),
            Self::Binary {
                argument1,
                argument2,
                ..
            } => {
                argument1.validate_at(location, field)?;
                argument2.validate_at(location, field)
            }
            Self::Clamp { input, min, max } => {
                finite("min", *min)?;
                finite("max", *max)?;
                if min > max {
                    return Err(validation::error(
                        location,
                        KIND,
                        field,
                        &format!("clamp min must not exceed max; received {min}..={max}"),
                    ));
                }
                input.validate_at(location, field)
            }
            Self::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => {
                finite("from_value", *from_value)?;
                finite("to_value", *to_value)?;
                if from_y > to_y {
                    return Err(validation::error(
                        location,
                        KIND,
                        field,
                        &format!("from_y must not exceed to_y; received {from_y}..={to_y}"),
                    ));
                }
                Ok(())
            }
            Self::Raw(raw) => {
                let value = raw.as_value();
                if value.is_object() || value.is_number() || value.is_string() {
                    Ok(())
                } else {
                    Err(validation::error(
                        location,
                        KIND,
                        field,
                        "raw density functions must be a JSON object, number, or string",
                    ))
                }
            }
        }
    }
}

/// A density-function definition
/// (`data/<namespace>/worldgen/density_function/<id>.json`).
///
/// ```
/// use sand_components::{
///     DatapackComponent, DensityFunction, DensityFunctionExpr, ResourceLocation,
/// };
///
/// let df = DensityFunction::new(
///     ResourceLocation::new("example", "flat").unwrap(),
///     DensityFunctionExpr::constant(0.5),
/// );
/// df.validate().unwrap();
/// assert_eq!(df.component_dir(), "worldgen/density_function");
/// assert_eq!(df.to_json(), serde_json::json!(0.5));
/// ```
#[derive(Debug, Clone)]
pub struct DensityFunction {
    location: ResourceLocation,
    expr: DensityFunctionExpr,
}

impl DensityFunction {
    /// Create a density function from a typed expression.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunction::new` for the canonical contract."]
    pub fn new(location: ResourceLocation, expr: DensityFunctionExpr) -> Self {
        Self { location, expr }
    }

    /// Create a density function from an explicitly raw JSON body.
    ///
    /// Prefer [`DensityFunction::new`]. This escape hatch exists for modded or
    /// version-specific density-function types Sand does not model.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunction::new_raw` for the canonical contract."]
    pub fn new_raw(location: ResourceLocation, body: RawJson) -> Self {
        Self::new(location, DensityFunctionExpr::raw(body))
    }

    /// The typed registry ID other worldgen files use to reference this
    /// density function.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunction::id` for the canonical contract."]
    pub fn id(&self) -> DensityFunctionId {
        DensityFunctionId::custom(self.location.clone())
    }

    /// Replace the density-function expression.
    #[doc = "**API Contract:** Run `sand api show sand::component::DensityFunction::expr` for the canonical contract."]
    pub fn expr(mut self, expr: DensityFunctionExpr) -> Self {
        self.expr = expr;
        self
    }
}

impl DatapackComponent for DensityFunction {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        self.expr.validate_at(&self.location, "expr")
    }

    fn to_json(&self) -> Value {
        self.expr.to_json()
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/density_function"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "ridge_density").unwrap()
    }

    fn df(expr: DensityFunctionExpr) -> DensityFunction {
        DensityFunction::new(location(), expr)
    }

    #[test]
    fn constant_and_reference_shapes_serialize() {
        let constant = df(DensityFunctionExpr::constant(-1.5));
        constant.validate().unwrap();
        assert_eq!(constant.to_json(), serde_json::json!(-1.5));

        let reference = df(DensityFunctionExpr::reference(
            DensityFunctionId::minecraft("overworld/base_3d_noise").unwrap(),
        ));
        reference.validate().unwrap();
        assert_eq!(
            reference.to_json(),
            serde_json::json!("minecraft:overworld/base_3d_noise")
        );
        assert_eq!(reference.component_dir(), "worldgen/density_function");
    }

    #[test]
    fn typed_noise_reference_and_nested_operations_serialize() {
        let expr = DensityFunctionExpr::sum(
            DensityFunctionExpr::square(DensityFunctionExpr::noise(
                NoiseId::minecraft("cave_entrance").unwrap(),
                0.75,
                0.5,
            )),
            DensityFunctionExpr::clamp(DensityFunctionExpr::constant(2.0), -1.0, 1.0),
        );
        let component = df(expr);
        component.validate().unwrap();
        let json = component.to_json();
        assert_eq!(json["type"], "minecraft:add");
        assert_eq!(json["argument1"]["type"], "minecraft:square");
        assert_eq!(
            json["argument1"]["argument"]["noise"],
            "minecraft:cave_entrance"
        );
        assert_eq!(json["argument1"]["argument"]["xz_scale"], 0.75);
        assert_eq!(json["argument2"]["type"], "minecraft:clamp");
        assert_eq!(json["argument2"]["max"], 1.0);
    }

    #[test]
    fn y_clamped_gradient_serializes() {
        let component = df(DensityFunctionExpr::y_clamped_gradient(-64, 320, 1.0, -1.0));
        component.validate().unwrap();
        let json = component.to_json();
        assert_eq!(json["type"], "minecraft:y_clamped_gradient");
        assert_eq!(json["from_y"], -64);
        assert_eq!(json["to_value"], -1.0);
    }

    #[test]
    fn non_finite_numbers_are_rejected_at_any_depth() {
        assert!(
            df(DensityFunctionExpr::constant(f64::NAN))
                .validate()
                .is_err()
        );
        assert!(
            df(DensityFunctionExpr::noise(
                NoiseId::minecraft("cave_entrance").unwrap(),
                f64::INFINITY,
                1.0,
            ))
            .validate()
            .is_err()
        );
        assert!(
            df(DensityFunctionExpr::product(
                DensityFunctionExpr::constant(1.0),
                DensityFunctionExpr::abs(DensityFunctionExpr::constant(f64::NEG_INFINITY)),
            ))
            .validate()
            .is_err()
        );
        assert!(
            df(DensityFunctionExpr::y_clamped_gradient(0, 8, f64::NAN, 1.0))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn inverted_ranges_are_rejected() {
        let err = df(DensityFunctionExpr::clamp(
            DensityFunctionExpr::constant(0.0),
            1.0,
            -1.0,
        ))
        .validate()
        .unwrap_err();
        assert!(err.to_string().contains("clamp"), "{err}");
        assert!(
            df(DensityFunctionExpr::y_clamped_gradient(320, -64, 1.0, -1.0))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn malformed_resource_ids_are_rejected_at_construction() {
        assert!("example:bad path".parse::<DensityFunctionId>().is_err());
        assert!("example:bad path".parse::<NoiseId>().is_err());
    }

    #[test]
    fn raw_escape_hatch_emits_stable_json_for_unsupported_shapes() {
        let raw = serde_json::json!({
            "type": "minecraft:weird_scaled_sampler",
            "rarity_value_mapper": "type_1",
            "noise": "minecraft:temperature",
            "input": {"type": "minecraft:constant", "argument": 0.0},
        });
        let component = DensityFunction::new_raw(location(), RawJson::new(raw.clone()));
        component.validate().unwrap();
        assert_eq!(component.to_json(), raw);

        // Nested raw shapes survive typed wrapping unchanged.
        let nested = df(DensityFunctionExpr::abs(DensityFunctionExpr::raw(
            RawJson::new(raw.clone()),
        )));
        nested.validate().unwrap();
        assert_eq!(nested.to_json()["argument"], raw);
    }

    #[test]
    fn raw_escape_hatch_rejects_impossible_json_kinds() {
        for value in [
            serde_json::json!(null),
            serde_json::json!(true),
            serde_json::json!([1, 2]),
        ] {
            assert!(
                DensityFunction::new_raw(location(), RawJson::new(value))
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn typed_registry_id_round_trips() {
        assert_eq!(
            df(DensityFunctionExpr::constant(0.0)).id().to_string(),
            "test:ridge_density"
        );
    }
}
