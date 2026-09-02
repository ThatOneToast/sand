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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DensityFunctionUnaryOp",
    aliases = ["sand::prelude::DensityFunctionUnaryOp"],
    module = "sand::component",
    summary = "A single-argument density-function transform.",
    context = "A single-argument density-function transform. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DensityFunctionUnaryOp;",
    variants(Abs = "`minecraft:abs`", Cube = "`minecraft:cube`", HalfNegative = "`minecraft:half_negative`", QuarterNegative = "`minecraft:quarter_negative`", Square = "`minecraft:square`", Squeeze = "`minecraft:squeeze`"),
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DensityFunctionBinaryOp",
    aliases = ["sand::prelude::DensityFunctionBinaryOp"],
    module = "sand::component",
    summary = "A two-argument density-function combinator.",
    context = "A two-argument density-function combinator. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DensityFunctionBinaryOp;",
    variants(Add = "`minecraft:add`", Max = "`minecraft:max`", Min = "`minecraft:min`", Mul = "`minecraft:mul`"),
)]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DensityFunctionExpr",
    aliases = ["sand::prelude::DensityFunctionExpr"],
    module = "sand::component",
    summary = "A density-function expression. Construct values through the named helpers rather than the variants directly; they keep boxing and the raw escape hatch visible at call sites.",
    context = "A density-function expression. Construct values through the named helpers rather than the variants directly; they keep boxing and the raw escape hatch visible at call sites. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DensityFunctionExpr;",
    variants(Binary = "A two-argument combinator.", Clamp = "`minecraft:clamp`", Constant = "A bare constant value, serialized as a JSON number.", Noise = "`minecraft:noise` — samples a `worldgen/noise` parameter file.", Raw = "An explicit raw escape hatch for modded or version-specific shapes that the typed variants do not model.", Reference = "A reference to another density-function file, serialized as a string.", Unary = "A single-argument transform.", YClampedGradient = "`minecraft:y_clamped_gradient`"),
    variant_fields(Binary(argument1 = "`argument1` provides the argument1 when a two-argument combinator.", argument2 = "`argument2` provides the argument2 when a two-argument combinator.", op = "`op` provides the op when a two-argument combinator."), Clamp(input = "`input` provides the input when `minecraft:clamp`.", max = "`max` provides the maximum value when `minecraft:clamp`.", min = "`min` provides the minimum value when `minecraft:clamp`."), Constant = ["A bare constant value, serialized as a JSON number."], Noise(noise = "`noise` provides the noise identifier when `minecraft:noise` — samples a `worldgen/noise` parameter file.", xz_scale = "`xz_scale` provides the xz scale when `minecraft:noise` — samples a `worldgen/noise` parameter file.", y_scale = "`y_scale` provides the y scale when `minecraft:noise` — samples a `worldgen/noise` parameter file."), Raw = ["An explicit raw escape hatch for modded or version-specific shapes that the typed variants do not model."], Reference = ["A reference to another density-function file, serialized as a string."], Unary(argument = "`argument` provides the argument when a single-argument transform.", op = "`op` provides the op when a single-argument transform."), YClampedGradient(from_value = "`from_value` provides the from value when `minecraft:y_clamped_gradient`.", from_y = "`from_y` provides the from y when `minecraft:y_clamped_gradient`.", to_value = "`to_value` provides the to value when `minecraft:y_clamped_gradient`.", to_y = "`to_y` provides the to y when `minecraft:y_clamped_gradient`.")),
)]
/// A density-function expression.
///
/// Construct values through the named helpers rather than the variants
/// directly; they keep boxing and the raw escape hatch visible at call sites.
#[derive(Debug, Clone, PartialEq)]
pub enum DensityFunctionExpr {
    /// A bare constant value, serialized as a JSON number.
    Constant(#[doc = "A bare constant value, serialized as a JSON number."] f64),
    /// A reference to another density-function file, serialized as a string.
    Reference(
        #[doc = "A reference to another density-function file, serialized as a string."]
        DensityFunctionId,
    ),
    /// `minecraft:noise` — samples a `worldgen/noise` parameter file.
    Noise {
        /// `noise` provides the noise identifier when `minecraft:noise` — samples a `worldgen/noise` parameter file.
        noise: NoiseId,
        /// `xz_scale` provides the xz scale when `minecraft:noise` — samples a `worldgen/noise` parameter file.
        xz_scale: f64,
        /// `y_scale` provides the y scale when `minecraft:noise` — samples a `worldgen/noise` parameter file.
        y_scale: f64,
    },
    /// A single-argument transform.
    Unary {
        /// `op` provides the op when a single-argument transform.
        op: DensityFunctionUnaryOp,
        /// `argument` provides the argument when a single-argument transform.
        argument: Box<DensityFunctionExpr>,
    },
    /// A two-argument combinator.
    Binary {
        /// `op` provides the op when a two-argument combinator.
        op: DensityFunctionBinaryOp,
        /// `argument1` provides the argument1 when a two-argument combinator.
        argument1: Box<DensityFunctionExpr>,
        /// `argument2` provides the argument2 when a two-argument combinator.
        argument2: Box<DensityFunctionExpr>,
    },
    /// `minecraft:clamp`
    Clamp {
        /// `input` provides the input when `minecraft:clamp`.
        input: Box<DensityFunctionExpr>,
        /// `min` provides the minimum value when `minecraft:clamp`.
        min: f64,
        /// `max` provides the maximum value when `minecraft:clamp`.
        max: f64,
    },
    /// `minecraft:y_clamped_gradient`
    YClampedGradient {
        /// `from_y` provides the from y when `minecraft:y_clamped_gradient`.
        from_y: i32,
        /// `to_y` provides the to y when `minecraft:y_clamped_gradient`.
        to_y: i32,
        /// `from_value` provides the from value when `minecraft:y_clamped_gradient`.
        from_value: f64,
        /// `to_value` provides the to value when `minecraft:y_clamped_gradient`.
        to_value: f64,
    },
    /// An explicit raw escape hatch for modded or version-specific shapes that
    /// the typed variants do not model.
    Raw(
        #[doc = "An explicit raw escape hatch for modded or version-specific shapes that the typed variants do not model."]
         RawJson,
    ),
}

impl DensityFunctionExpr {
    /// A constant density value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::constant",
        aliases = ["sand::prelude::DensityFunctionExpr::constant"],
        module = "sand::component",
        kind = "method",
        summary = "A constant density value.",
        context = "A constant density value. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to use a constant density value."),
        returns = "A newly constructed `DensityFunctionExpr` configured to use a constant density value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: f64)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::constant(value);\n}",
    )]
    pub fn constant(value: f64) -> Self {
        Self::Constant(value)
    }

    /// A reference to another `worldgen/density_function` file.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::reference",
        aliases = ["sand::prelude::DensityFunctionExpr::reference"],
        module = "sand::component",
        kind = "method",
        summary = "A reference to another `worldgen/density_function` file.",
        context = "A reference to another `worldgen/density_function` file. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to use a reference to another `worldgen/density_function` file."),
        returns = "A newly constructed `DensityFunctionExpr` configured to use a reference to another `worldgen/density_function` file.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::registry::DensityFunctionId)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::reference(id);\n}",
    )]
    pub fn reference(id: DensityFunctionId) -> Self {
        Self::Reference(id)
    }

    /// Sample a `worldgen/noise` parameter file.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::noise",
        aliases = ["sand::prelude::DensityFunctionExpr::noise"],
        module = "sand::component",
        kind = "method",
        summary = "Sample a `worldgen/noise` parameter file.",
        context = "Sample a `worldgen/noise` parameter file. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(noise = "`noise` provides the typed Minecraft resource identifier used to sample a `worldgen/noise` parameter file.", xz_scale = "`xz_scale` supplies the xz scale value used to sample a `worldgen/noise` parameter file.", y_scale = "`y_scale` supplies the y scale value used to sample a `worldgen/noise` parameter file."),
        returns = "A newly constructed `DensityFunctionExpr` configured to sample a `worldgen/noise` parameter file.",
        example = "use sand::prelude::*;\n\nfn demonstrate(noise: sand::registry::NoiseId, xz_scale: f64, y_scale: f64)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::noise(noise, xz_scale, y_scale);\n}",
    )]
    pub fn noise(noise: NoiseId, xz_scale: f64, y_scale: f64) -> Self {
        Self::Noise {
            noise,
            xz_scale,
            y_scale,
        }
    }

    /// Apply a single-argument transform.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::unary",
        aliases = ["sand::prelude::DensityFunctionExpr::unary"],
        module = "sand::component",
        kind = "method",
        summary = "Apply a single-argument transform.",
        context = "Apply a single-argument transform. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(op = "`op` supplies the op value used to apply a single-argument transform.", argument = "`argument` supplies the argument value used to apply a single-argument transform."),
        returns = "A newly constructed `DensityFunctionExpr` configured to apply a single-argument transform.",
        example = "use sand::prelude::*;\n\nfn demonstrate(op: sand::component::DensityFunctionUnaryOp, argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::unary(op, argument);\n}",
    )]
    pub fn unary(op: DensityFunctionUnaryOp, argument: Self) -> Self {
        Self::Unary {
            op,
            argument: Box::new(argument),
        }
    }

    /// `minecraft:abs`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::abs",
        aliases = ["sand::prelude::DensityFunctionExpr::abs"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:abs`",
        context = "`minecraft:abs` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:abs` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:abs` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::abs(argument);\n}",
    )]
    pub fn abs(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Abs, argument)
    }

    /// `minecraft:square`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::square",
        aliases = ["sand::prelude::DensityFunctionExpr::square"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:square`",
        context = "`minecraft:square` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:square` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:square` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::square(argument);\n}",
    )]
    pub fn square(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Square, argument)
    }

    /// `minecraft:cube`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::cube",
        aliases = ["sand::prelude::DensityFunctionExpr::cube"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:cube`",
        context = "`minecraft:cube` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:cube` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:cube` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::cube(argument);\n}",
    )]
    pub fn cube(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Cube, argument)
    }

    /// `minecraft:half_negative`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::half_negative",
        aliases = ["sand::prelude::DensityFunctionExpr::half_negative"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:half_negative`",
        context = "`minecraft:half_negative` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:half_negative` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:half_negative` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::half_negative(argument);\n}",
    )]
    pub fn half_negative(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::HalfNegative, argument)
    }

    /// `minecraft:quarter_negative`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::quarter_negative",
        aliases = ["sand::prelude::DensityFunctionExpr::quarter_negative"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:quarter_negative`",
        context = "`minecraft:quarter_negative` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:quarter_negative` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:quarter_negative` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::quarter_negative(argument);\n}",
    )]
    pub fn quarter_negative(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::QuarterNegative, argument)
    }

    /// `minecraft:squeeze`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::squeeze",
        aliases = ["sand::prelude::DensityFunctionExpr::squeeze"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:squeeze`",
        context = "`minecraft:squeeze` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument = "`argument` supplies the argument value used to emit the documented `minecraft:squeeze` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:squeeze` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::squeeze(argument);\n}",
    )]
    pub fn squeeze(argument: Self) -> Self {
        Self::unary(DensityFunctionUnaryOp::Squeeze, argument)
    }

    /// Apply a two-argument combinator.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::binary",
        aliases = ["sand::prelude::DensityFunctionExpr::binary"],
        module = "sand::component",
        kind = "method",
        summary = "Apply a two-argument combinator.",
        context = "Apply a two-argument combinator. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(op = "`op` supplies the op value used to apply a two-argument combinator.", argument1 = "`argument1` supplies the argument1 value used to apply a two-argument combinator.", argument2 = "`argument2` supplies the argument2 value used to apply a two-argument combinator."),
        returns = "A newly constructed `DensityFunctionExpr` configured to apply a two-argument combinator.",
        example = "use sand::prelude::*;\n\nfn demonstrate(op: sand::component::DensityFunctionBinaryOp, argument1: sand::component::DensityFunctionExpr, argument2: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::binary(op, argument1, argument2);\n}",
    )]
    pub fn binary(op: DensityFunctionBinaryOp, argument1: Self, argument2: Self) -> Self {
        Self::Binary {
            op,
            argument1: Box::new(argument1),
            argument2: Box::new(argument2),
        }
    }

    /// `minecraft:add`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::sum",
        aliases = ["sand::prelude::DensityFunctionExpr::sum"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:add`",
        context = "`minecraft:add` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument1 = "`argument1` supplies the argument1 value used to emit the documented `minecraft:add` form.", argument2 = "`argument2` supplies the argument2 value used to emit the documented `minecraft:add` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:add` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument1: sand::component::DensityFunctionExpr, argument2: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::sum(argument1, argument2);\n}",
    )]
    pub fn sum(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Add, argument1, argument2)
    }

    /// `minecraft:mul`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::product",
        aliases = ["sand::prelude::DensityFunctionExpr::product"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:mul`",
        context = "`minecraft:mul` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument1 = "`argument1` supplies the argument1 value used to emit the documented `minecraft:mul` form.", argument2 = "`argument2` supplies the argument2 value used to emit the documented `minecraft:mul` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:mul` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument1: sand::component::DensityFunctionExpr, argument2: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::product(argument1, argument2);\n}",
    )]
    pub fn product(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Mul, argument1, argument2)
    }

    /// `minecraft:min`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::min",
        aliases = ["sand::prelude::DensityFunctionExpr::min"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:min`",
        context = "`minecraft:min` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument1 = "`argument1` supplies the argument1 value used to emit the documented `minecraft:min` form.", argument2 = "`argument2` supplies the argument2 value used to emit the documented `minecraft:min` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:min` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument1: sand::component::DensityFunctionExpr, argument2: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::min(argument1, argument2);\n}",
    )]
    pub fn min(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Min, argument1, argument2)
    }

    /// `minecraft:max`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::max",
        aliases = ["sand::prelude::DensityFunctionExpr::max"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:max`",
        context = "`minecraft:max` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(argument1 = "`argument1` supplies the argument1 value used to emit the documented `minecraft:max` form.", argument2 = "`argument2` supplies the argument2 value used to emit the documented `minecraft:max` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:max` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(argument1: sand::component::DensityFunctionExpr, argument2: sand::component::DensityFunctionExpr)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::max(argument1, argument2);\n}",
    )]
    pub fn max(argument1: Self, argument2: Self) -> Self {
        Self::binary(DensityFunctionBinaryOp::Max, argument1, argument2)
    }

    /// `minecraft:clamp`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::clamp",
        aliases = ["sand::prelude::DensityFunctionExpr::clamp"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:clamp`",
        context = "`minecraft:clamp` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(input = "`input` supplies the input value used to emit the documented `minecraft:clamp` form.", min = "`min` provides the inclusive lower bound used to emit the documented `minecraft:clamp` form.", max = "`max` provides the inclusive upper bound used to emit the documented `minecraft:clamp` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:clamp` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(input: sand::component::DensityFunctionExpr, min: f64, max: f64)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::clamp(input, min, max);\n}",
    )]
    pub fn clamp(input: Self, min: f64, max: f64) -> Self {
        Self::Clamp {
            input: Box::new(input),
            min,
            max,
        }
    }

    /// `minecraft:y_clamped_gradient`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::y_clamped_gradient",
        aliases = ["sand::prelude::DensityFunctionExpr::y_clamped_gradient"],
        module = "sand::component",
        kind = "method",
        summary = "`minecraft:y_clamped_gradient`",
        context = "`minecraft:y_clamped_gradient` This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(from_y = "`from_y` supplies the from y value used to emit the documented `minecraft:y_clamped_gradient` form.", to_y = "`to_y` supplies the to y value used to emit the documented `minecraft:y_clamped_gradient` form.", from_value = "`from_value` supplies the from value used to emit the documented `minecraft:y_clamped_gradient` form.", to_value = "`to_value` supplies the to value used to emit the documented `minecraft:y_clamped_gradient` form."),
        returns = "A newly constructed `DensityFunctionExpr` configured to emit the documented `minecraft:y_clamped_gradient` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(from_y: i32, to_y: i32, from_value: f64, to_value: f64)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::y_clamped_gradient(from_y, to_y, from_value, to_value);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::raw",
        aliases = ["sand::prelude::DensityFunctionExpr::raw"],
        module = "sand::component",
        kind = "method",
        summary = "An explicit raw escape hatch for shapes the typed variants do not model.",
        context = "An explicit raw escape hatch for shapes the typed variants do not model. The value is emitted unchanged; only its outer JSON kind (object, number, or string) is checked, because those are the only forms Minecraft accepts for a density function.",
        minecraft = "The value is emitted unchanged; only its outer JSON kind (object, number, or string) is checked, because those are the only forms Minecraft accepts for a density function.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to use an explicit raw escape hatch for shapes the typed variants do not model."),
        returns = "A newly constructed `DensityFunctionExpr` configured to use an explicit raw escape hatch for shapes the typed variants do not model.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: sand::component::RawJson)  {\n    let density_function_expr = sand::component::DensityFunctionExpr::raw(value);\n}",
    )]
    pub fn raw(value: RawJson) -> Self {
        Self::Raw(value)
    }

    /// Serialize this expression to the JSON Minecraft expects.
    ///
    /// Public so typed expressions can be embedded in explicit raw
    /// noise-router surfaces that Sand does not yet model as author API.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunctionExpr::to_json",
        aliases = ["sand::prelude::DensityFunctionExpr::to_json"],
        module = "sand::component",
        kind = "method",
        summary = "Serialize this expression to the JSON Minecraft expects.",
        context = "Serialize this expression to the JSON Minecraft expects. Public so typed expressions can be embedded in explicit raw noise-router surfaces that Sand does not yet model as author API.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `Value` value produced to serialize this expression to the JSON Minecraft expects.",
        example = "use sand::prelude::*;\n\nfn demonstrate(density_function_expr_value: &sand::component::DensityFunctionExpr)  {\n    let to_json = density_function_expr_value.to_json();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DensityFunction",
    aliases = ["sand::prelude::DensityFunction"],
    module = "sand::component",
    summary = "A density-function definition (`data/<namespace>/worldgen/density_function/<id>.json`).",
    context = "A density-function definition (`data/<namespace>/worldgen/density_function/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DensityFunction;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunction::new",
        aliases = ["sand::prelude::DensityFunction::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a density function from a typed expression.",
        context = "Create a density function from a typed expression. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a density function from a typed expression.", expr = "`expr` supplies the expr value used to create a density function from a typed expression."),
        returns = "A newly constructed `DensityFunction` configured to create a density function from a typed expression.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, expr: sand::component::DensityFunctionExpr)  {\n    let density_function = sand::component::DensityFunction::new(location, expr);\n}",
    )]
    pub fn new(location: ResourceLocation, expr: DensityFunctionExpr) -> Self {
        Self { location, expr }
    }

    /// Create a density function from an explicitly raw JSON body.
    ///
    /// Prefer [`DensityFunction::new`]. This escape hatch exists for modded or
    /// version-specific density-function types Sand does not model.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunction::new_raw",
        aliases = ["sand::prelude::DensityFunction::new_raw"],
        module = "sand::component",
        kind = "method",
        summary = "Create a density function from an explicitly raw JSON body.",
        context = "Create a density function from an explicitly raw JSON body. Prefer [`DensityFunction::new`]. This escape hatch exists for modded or version-specific density-function types Sand does not model.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Prefer [`DensityFunction::new`]. This escape hatch exists for modded or version-specific density-function types Sand does not model."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a density function from an explicitly raw JSON body.", body = "`body` supplies the body value used to create a density function from an explicitly raw JSON body."),
        returns = "A newly constructed `DensityFunction` configured to create a density function from an explicitly raw JSON body.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, body: sand::component::RawJson)  {\n    let density_function = sand::component::DensityFunction::new_raw(location, body);\n}",
    )]
    pub fn new_raw(location: ResourceLocation, body: RawJson) -> Self {
        Self::new(location, DensityFunctionExpr::raw(body))
    }

    /// The typed registry ID other worldgen files use to reference this
    /// density function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunction::id",
        aliases = ["sand::prelude::DensityFunction::id"],
        module = "sand::component",
        kind = "method",
        summary = "The typed registry ID other worldgen files use to reference this density function.",
        context = "The typed registry ID other worldgen files use to reference this density function. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `DensityFunctionId` value produced to use the typed registry ID other worldgen files use to reference this density function.",
        example = "use sand::prelude::*;\n\nfn demonstrate(density_function_value: &sand::component::DensityFunction)  {\n    let id = density_function_value.id();\n}",
    )]
    pub fn id(&self) -> DensityFunctionId {
        DensityFunctionId::custom(self.location.clone())
    }

    /// Replace the density-function expression.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DensityFunction::expr",
        aliases = ["sand::prelude::DensityFunction::expr"],
        module = "sand::component",
        kind = "method",
        summary = "Replace the density-function expression.",
        context = "Replace the density-function expression. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(expr = "`expr` supplies the expr value used to replace the density-function expression."),
        returns = "The `DensityFunction` value with the documented change applied to replace the density-function expression.",
        example = "use sand::prelude::*;\n\nfn demonstrate(density_function_value: sand::component::DensityFunction, expr: sand::component::DensityFunctionExpr)  {\n    let updated_density_function = density_function_value.expr(expr);\n}",
    )]
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
