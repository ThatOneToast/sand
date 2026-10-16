//! Coordinate argument types for Minecraft commands.
//!
//! Minecraft supports three coordinate modes:
//! - **Absolute**: `X Y Z` — exact world coordinates
//! - **Relative** (`~`): `~X ~Y ~Z` — offset from the executor's position
//! - **Local** (`^`): `^X ^Y ^Z` — offset along the executor's facing direction

use std::fmt;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::validate;

// ── Coord ─────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Coord",
    aliases = ["sand::cmd::Coord", "sand::prelude::Coord", "sand::prelude::cmd::Coord"],
    module = "sand::command",
    summary = "A single coordinate value: absolute, relative (`~`), or local (`^`).",
    context = "A single coordinate value: absolute, relative (`~`), or local (`^`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Coord;",
    variants(Absolute = "Absolute world coordinate.", Local = "Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`.", Relative = "Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`."),
    variant_fields(Absolute = ["Absolute world coordinate."], Local = ["Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`."], Relative = ["Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`."]),
)]
/// A single coordinate value: absolute, relative (`~`), or local (`^`).
#[derive(Debug, Clone, PartialEq)]
#[must_use = "coordinates do nothing until passed to a command"]
pub enum Coord {
    /// Absolute world coordinate.
    Absolute(#[doc = "Absolute world coordinate."] f64),
    /// Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`.
    Relative(#[doc = "Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`."] f64),
    /// Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`.
    Local(#[doc = "Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`."] f64),
}

impl Coord {
    /// Absolute coordinate.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Coord::abs",
        aliases = ["sand::cmd::Coord::abs", "sand::prelude::Coord::abs", "sand::prelude::cmd::Coord::abs"],
        module = "sand::command",
        kind = "method",
        summary = "Absolute coordinate.",
        context = "Absolute coordinate. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(v = "`v` sets the v for absolute coordinate."),
        returns = "A `Coord` configured for absolute coordinate.",
        example = "use sand::prelude::*;\n\nfn demonstrate(v: impl Into < f64 >)  {\n    let coord = sand::command::Coord::abs(v);\n}",
    )]
    pub fn abs(v: impl Into<f64>) -> Self {
        Coord::Absolute(v.into())
    }
    /// Relative coordinate at the executor's position (`~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Coord::rel",
        aliases = ["sand::cmd::Coord::rel", "sand::prelude::Coord::rel", "sand::prelude::cmd::Coord::rel"],
        module = "sand::command",
        kind = "method",
        summary = "Relative coordinate at the executor's position (`~`).",
        context = "Relative coordinate at the executor's position (`~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Coord` configured for relative coordinate at the executor's position (`~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let coord = sand::command::Coord::rel();\n}",
    )]
    pub fn rel() -> Self {
        Coord::Relative(0.0)
    }
    /// Relative coordinate with an offset (`~N`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Coord::rel_n",
        aliases = ["sand::cmd::Coord::rel_n", "sand::prelude::Coord::rel_n", "sand::prelude::cmd::Coord::rel_n"],
        module = "sand::command",
        kind = "method",
        summary = "Relative coordinate with an offset (`~N`).",
        context = "Relative coordinate with an offset (`~N`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(v = "`v` sets the v for relative coordinate with an offset (`~N`)."),
        returns = "A `Coord` configured for relative coordinate with an offset (`~N`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(v: impl Into < f64 >)  {\n    let coord = sand::command::Coord::rel_n(v);\n}",
    )]
    pub fn rel_n(v: impl Into<f64>) -> Self {
        Coord::Relative(v.into())
    }
    /// Local coordinate (along executor's facing direction, `^`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Coord::local",
        aliases = ["sand::cmd::Coord::local", "sand::prelude::Coord::local", "sand::prelude::cmd::Coord::local"],
        module = "sand::command",
        kind = "method",
        summary = "Local coordinate (along executor's facing direction, `^`).",
        context = "Local coordinate (along executor's facing direction, `^`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Coord` representing a local coordinate (along executor's facing direction, `^`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let coord = sand::command::Coord::local();\n}",
    )]
    pub fn local() -> Self {
        Coord::Local(0.0)
    }
    /// Local coordinate with an offset (`^N`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Coord::local_n",
        aliases = ["sand::cmd::Coord::local_n", "sand::prelude::Coord::local_n", "sand::prelude::cmd::Coord::local_n"],
        module = "sand::command",
        kind = "method",
        summary = "Local coordinate with an offset (`^N`).",
        context = "Local coordinate with an offset (`^N`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(v = "`v` sets the offset of the local coordinate (`^N`)."),
        returns = "A `Coord` representing a local coordinate with an offset (`^N`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(v: impl Into < f64 >)  {\n    let coord = sand::command::Coord::local_n(v);\n}",
    )]
    pub fn local_n(v: impl Into<f64>) -> Self {
        Coord::Local(v.into())
    }
}

impl From<f64> for Coord {
    fn from(v: f64) -> Self {
        Coord::Absolute(v)
    }
}
impl From<f32> for Coord {
    fn from(v: f32) -> Self {
        Coord::Absolute(v as f64)
    }
}
impl From<i64> for Coord {
    fn from(v: i64) -> Self {
        Coord::Absolute(v as f64)
    }
}
impl From<i32> for Coord {
    fn from(v: i32) -> Self {
        Coord::Absolute(v as f64)
    }
}

fn fmt_coord(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e12 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Coord::Absolute(v) => write!(f, "{}", fmt_coord(*v)),
            Coord::Relative(v) if *v == 0.0 => write!(f, "~"),
            Coord::Relative(v) => write!(f, "~{}", fmt_coord(*v)),
            Coord::Local(v) if *v == 0.0 => write!(f, "^"),
            Coord::Local(v) => write!(f, "^{}", fmt_coord(*v)),
        }
    }
}

impl Validate for Coord {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        let value = match self {
            Self::Absolute(value) | Self::Relative(value) | Self::Local(value) => *value,
        };
        validate::finite(value, "Coord", "value").map(|_| ())
    }
}

impl RenderCommand for Coord {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── BlockPos ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::BlockPos",
    aliases = ["sand::cmd::BlockPos", "sand::prelude::BlockPos", "sand::prelude::cmd::BlockPos"],
    module = "sand::command",
    summary = "Block position with integer absolute coordinates or fractional relative/local offsets.",
    context = "BlockPos models positions for setblock, fill, clone and block-targeted data. Absolute axes must be signed 32-bit integers. Relative offsets may be fractional; local offsets may be fractional when all three axes use local coordinates. Minecraft resolves offsets from the execution position and floors the result to a block position. Use Vec3 for commands that accept fractional absolute positions.",
    minecraft = "Absolute block coordinates use signed integer grammar; relative and local offsets use finite floating-point grammar. Local axes cannot be mixed with absolute or relative axes.",
    use_when = ["Passing a block position to a typed Minecraft command"],
    avoid_when = ["Passing a fractional absolute position to commands that require Vec3"],
    example = "use sand::command::BlockPos;",
    fields(x = "The x coordinate, validated with the other axes as a block position.", y = "The y coordinate, validated with the other axes as a block position.", z = "The z coordinate, validated with the other axes as a block position."),
)]
/// Block position used by `setblock`, `fill`, `clone`, and block-targeted `data`.
///
/// Absolute axes must be signed 32-bit integers. Relative (`~`) offsets may
/// be fractional, and local (`^`) offsets may be fractional when all three
/// axes are local. Local axes cannot be mixed with absolute or relative axes.
/// Minecraft resolves offsets from the execution position and floors the
/// resulting position to the containing block.
///
/// Use [`Vec3`] for commands that accept fractional absolute positions, such
/// as `execute positioned`, `particle`, `summon`, and `tp`. Both position
/// types reject non-finite values through [`Validate::validate`].
///
/// # Examples
/// ```
/// use sand_commands::coord::{BlockPos, Coord};
///
/// assert_eq!(BlockPos::absolute(10, 64, -5).to_string(), "10 64 -5");
/// assert_eq!(BlockPos::here().to_string(), "~ ~ ~");
/// assert_eq!(BlockPos::above(3).to_string(), "~ ~3 ~");
/// ```
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct BlockPos {
    /// The x coordinate, validated with the other axes as a block position.
    pub x: Coord,
    /// The y coordinate, validated with the other axes as a block position.
    pub y: Coord,
    /// The z coordinate, validated with the other axes as a block position.
    pub z: Coord,
}

impl PartialEq for BlockPos {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for BlockPos {}

impl BlockPos {
    /// Create a block position from three coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BlockPos::new",
        aliases = ["sand::cmd::BlockPos::new", "sand::prelude::BlockPos::new", "sand::prelude::cmd::BlockPos::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a block position from three coordinates.",
        context = "Create a block position from three coordinates. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to create a block position from three coordinates.", y = "`y` provides the y-coordinate used to create a block position from three coordinates.", z = "`z` provides the z-coordinate used to create a block position from three coordinates."),
        returns = "A `BlockPos` representing a block position from three coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(x: impl Into < sand::command::Coord >, y: impl Into < sand::command::Coord >, z: impl Into < sand::command::Coord >)  {\n    let block_pos = sand::command::BlockPos::new(x, y, z);\n}",
    )]
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BlockPos::here",
        aliases = ["sand::cmd::BlockPos::here", "sand::prelude::BlockPos::here", "sand::prelude::cmd::BlockPos::here"],
        module = "sand::command",
        kind = "method",
        summary = "Current position (`~ ~ ~`).",
        context = "Current position (`~ ~ ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `BlockPos` representing the current position (`~ ~ ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let block_pos = sand::command::BlockPos::here();\n}",
    )]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact block coordinates (`X Y Z`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BlockPos::absolute",
        aliases = ["sand::cmd::BlockPos::absolute", "sand::prelude::BlockPos::absolute", "sand::prelude::cmd::BlockPos::absolute"],
        module = "sand::command",
        kind = "method",
        summary = "Exact block coordinates (`X Y Z`).",
        context = "Exact block coordinates (`X Y Z`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to use exact block coordinates (`X Y Z`).", y = "`y` provides the y-coordinate used to use exact block coordinates (`X Y Z`).", z = "`z` provides the z-coordinate used to use exact block coordinates (`X Y Z`)."),
        returns = "A `BlockPos` configured for exact block coordinates (`X Y Z`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(x: i32, y: i32, z: i32)  {\n    let block_pos = sand::command::BlockPos::absolute(x, y, z);\n}",
    )]
    pub fn absolute(x: i32, y: i32, z: i32) -> Self {
        Self::new(Coord::abs(x), Coord::abs(y), Coord::abs(z))
    }
    /// Position N blocks above current (`~ ~N ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BlockPos::above",
        aliases = ["sand::cmd::BlockPos::above", "sand::prelude::BlockPos::above", "sand::prelude::cmd::BlockPos::above"],
        module = "sand::command",
        kind = "method",
        summary = "Position N blocks above current (`~ ~N ~`).",
        context = "Position N blocks above current (`~ ~N ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n` is used to position N blocks above current (`~ ~N ~`)."),
        returns = "A `BlockPos` representing a position N blocks above current (`~ ~N ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let block_pos = sand::command::BlockPos::above(n);\n}",
    )]
    pub fn above(n: i32) -> Self {
        Self::new(Coord::rel(), Coord::rel_n(n), Coord::rel())
    }
    /// Position N blocks below current (`~ ~-N ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BlockPos::below",
        aliases = ["sand::cmd::BlockPos::below", "sand::prelude::BlockPos::below", "sand::prelude::cmd::BlockPos::below"],
        module = "sand::command",
        kind = "method",
        summary = "Position N blocks below current (`~ ~-N ~`).",
        context = "Position N blocks below current (`~ ~-N ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(n = "`n` is used to position N blocks below current (`~ ~-N ~`)."),
        returns = "A `BlockPos` representing a position N blocks below current (`~ ~-N ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(n: i32)  {\n    let block_pos = sand::command::BlockPos::below(n);\n}",
    )]
    pub fn below(n: i32) -> Self {
        Self::new(Coord::rel(), Coord::rel_n(-n), Coord::rel())
    }
}

impl fmt::Display for BlockPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl Validate for BlockPos {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        validate_triplet(&self.x, &self.y, &self.z, "BlockPos", profile)?;
        for (field, coord) in [("x", &self.x), ("y", &self.y), ("z", &self.z)] {
            if let Coord::Absolute(value) = coord {
                if value.fract() != 0.0 {
                    return Err(CommandError::new(
                        "BlockPos",
                        field,
                        format!(
                            "absolute block coordinates cannot contain fractional value `{value}`"
                        ),
                    ));
                }
                if *value < i32::MIN as f64 || *value > i32::MAX as f64 {
                    return Err(CommandError::new(
                        "BlockPos",
                        field,
                        "absolute block coordinates must fit a signed 32-bit integer",
                    ));
                }
            }
        }
        Ok(())
    }
}

impl RenderCommand for BlockPos {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── Vec3 ──────────────────────────────────────────────────────────────────────

/// Floating-point position used in commands like `tp`, `summon`, `particle`.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Vec3",
    aliases = ["sand::cmd::Vec3", "sand::prelude::Vec3", "sand::prelude::cmd::Vec3"],
    module = "sand::command",
    summary = "Floating-point position used in commands like `tp`, `summon`, `particle`.",
    context = "Floating-point position used in commands like `tp`, `summon`, `particle`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Vec3;",
    fields(x = "`x` provides the x coordinate when floating-point position used in commands like `tp`, `summon`, `particle`.", y = "`y` provides the y coordinate when floating-point position used in commands like `tp`, `summon`, `particle`.", z = "`z` provides the z coordinate when floating-point position used in commands like `tp`, `summon`, `particle`."),
)]
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct Vec3 {
    /// `x` provides the x coordinate when floating-point position used in commands like `tp`, `summon`, `particle`.
    pub x: Coord,
    /// `y` provides the y coordinate when floating-point position used in commands like `tp`, `summon`, `particle`.
    pub y: Coord,
    /// `z` provides the z coordinate when floating-point position used in commands like `tp`, `summon`, `particle`.
    pub z: Coord,
}

impl Vec3 {
    /// Create a 3D position from three coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Vec3::new",
        aliases = ["sand::cmd::Vec3::new", "sand::prelude::Vec3::new", "sand::prelude::cmd::Vec3::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a 3D position from three coordinates.",
        context = "Create a 3D position from three coordinates. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to create a 3D position from three coordinates.", y = "`y` provides the y-coordinate used to create a 3D position from three coordinates.", z = "`z` provides the z-coordinate used to create a 3D position from three coordinates."),
        returns = "A `Vec3` representing a 3D position from three coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(x: impl Into < sand::command::Coord >, y: impl Into < sand::command::Coord >, z: impl Into < sand::command::Coord >)  {\n    let vec3 = sand::command::Vec3::new(x, y, z);\n}",
    )]
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Vec3::here",
        aliases = ["sand::cmd::Vec3::here", "sand::prelude::Vec3::here", "sand::prelude::cmd::Vec3::here"],
        module = "sand::command",
        kind = "method",
        summary = "Current position (`~ ~ ~`).",
        context = "Current position (`~ ~ ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Vec3` representing the current position (`~ ~ ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let vec3 = sand::command::Vec3::here();\n}",
    )]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact world coordinates (`X Y Z`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Vec3::absolute",
        aliases = ["sand::cmd::Vec3::absolute", "sand::prelude::Vec3::absolute", "sand::prelude::cmd::Vec3::absolute"],
        module = "sand::command",
        kind = "method",
        summary = "Exact world coordinates (`X Y Z`).",
        context = "Exact world coordinates (`X Y Z`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to use exact world coordinates (`X Y Z`).", y = "`y` provides the y-coordinate used to use exact world coordinates (`X Y Z`).", z = "`z` provides the z-coordinate used to use exact world coordinates (`X Y Z`)."),
        returns = "A `Vec3` configured for exact world coordinates (`X Y Z`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(x: f64, y: f64, z: f64)  {\n    let vec3 = sand::command::Vec3::absolute(x, y, z);\n}",
    )]
    pub fn absolute(x: f64, y: f64, z: f64) -> Self {
        Self::new(Coord::abs(x), Coord::abs(y), Coord::abs(z))
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl Validate for Vec3 {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        validate_triplet(&self.x, &self.y, &self.z, "Vec3", profile)
    }
}

impl RenderCommand for Vec3 {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── Vec2 ──────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Vec2",
    aliases = ["sand::cmd::Vec2", "sand::prelude::Vec2", "sand::prelude::cmd::Vec2"],
    module = "sand::command",
    summary = "2D column position (X Z), used in `locatebiome` etc.",
    context = "2D column position (X Z), used in `locatebiome` etc. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Vec2;",
    fields(x = "`x` provides the x coordinate when 2D column position (X Z), used in `locatebiome` etc.", z = "`z` provides the z coordinate when 2D column position (X Z), used in `locatebiome` etc."),
)]
/// 2D column position (X Z), used in `locatebiome` etc.
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct Vec2 {
    /// `x` provides the x coordinate when 2D column position (X Z), used in `locatebiome` etc.
    pub x: Coord,
    /// `z` provides the z coordinate when 2D column position (X Z), used in `locatebiome` etc.
    pub z: Coord,
}

impl Vec2 {
    /// Create a 2D position (column) from X and Z coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Vec2::new",
        aliases = ["sand::cmd::Vec2::new", "sand::prelude::Vec2::new", "sand::prelude::cmd::Vec2::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a 2D position (column) from X and Z coordinates.",
        context = "Create a 2D position (column) from X and Z coordinates. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(x = "`x` provides the x-coordinate used to create a 2D position (column) from X and Z coordinates.", z = "`z` provides the z-coordinate used to create a 2D position (column) from X and Z coordinates."),
        returns = "A `Vec2` representing a 2D position (column) from X and Z coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(x: impl Into < sand::command::Coord >, z: impl Into < sand::command::Coord >)  {\n    let vec2 = sand::command::Vec2::new(x, z);\n}",
    )]
    pub fn new(x: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            z: z.into(),
        }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.x, self.z)
    }
}

impl Validate for Vec2 {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.x
            .validate(profile)
            .map_err(|e| e.with_context("Vec2.x"))?;
        self.z
            .validate(profile)
            .map_err(|e| e.with_context("Vec2.z"))?;
        if matches!(self.x, Coord::Local(_)) || matches!(self.z, Coord::Local(_)) {
            return Err(CommandError::new(
                "Vec2",
                "coordinate_system",
                "two-dimensional column positions do not accept local (`^`) coordinates",
            ));
        }
        Ok(())
    }
}

impl RenderCommand for Vec2 {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── Rotation ──────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Rotation",
    aliases = ["sand::cmd::Rotation", "sand::prelude::Rotation", "sand::prelude::cmd::Rotation"],
    module = "sand::command",
    summary = "Yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.",
    context = "Yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Rotation;",
    fields(pitch = "`pitch` provides the pitch rotation when yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.", yaw = "`yaw` provides the yaw rotation when yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`."),
)]
/// Yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.
#[derive(Debug, Clone)]
#[must_use = "rotations do nothing until passed to a command"]
pub struct Rotation {
    /// `yaw` provides the yaw rotation when yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.
    pub yaw: Coord,
    /// `pitch` provides the pitch rotation when yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.
    pub pitch: Coord,
}

impl Rotation {
    /// Create a rotation from yaw and pitch coordinates.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Rotation::new",
        aliases = ["sand::cmd::Rotation::new", "sand::prelude::Rotation::new", "sand::prelude::cmd::Rotation::new"],
        module = "sand::command",
        kind = "method",
        summary = "Create a rotation from yaw and pitch coordinates.",
        context = "Create a rotation from yaw and pitch coordinates. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(yaw = "`yaw` is used when creating a rotation from yaw and pitch coordinates.", pitch = "`pitch` is used when creating a rotation from yaw and pitch coordinates."),
        returns = "A `Rotation` representing a rotation from yaw and pitch coordinates.",
        example = "use sand::prelude::*;\n\nfn demonstrate(yaw: impl Into < sand::command::Coord >, pitch: impl Into < sand::command::Coord >)  {\n    let rotation = sand::command::Rotation::new(yaw, pitch);\n}",
    )]
    pub fn new(yaw: impl Into<Coord>, pitch: impl Into<Coord>) -> Self {
        Self {
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }
    /// Current rotation (`~ ~`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Rotation::here",
        aliases = ["sand::cmd::Rotation::here", "sand::prelude::Rotation::here", "sand::prelude::cmd::Rotation::here"],
        module = "sand::command",
        kind = "method",
        summary = "Current rotation (`~ ~`).",
        context = "Current rotation (`~ ~`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `Rotation` representing the current rotation (`~ ~`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let rotation = sand::command::Rotation::here();\n}",
    )]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel())
    }
    /// Absolute yaw and pitch angles.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Rotation::absolute",
        aliases = ["sand::cmd::Rotation::absolute", "sand::prelude::Rotation::absolute", "sand::prelude::cmd::Rotation::absolute"],
        module = "sand::command",
        kind = "method",
        summary = "Absolute yaw and pitch angles.",
        context = "Absolute yaw and pitch angles. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(yaw = "`yaw` sets the yaw for absolute yaw and pitch angles.", pitch = "`pitch` sets the pitch for absolute yaw and pitch angles."),
        returns = "A `Rotation` configured for absolute yaw and pitch angles.",
        example = "use sand::prelude::*;\n\nfn demonstrate(yaw: f64, pitch: f64)  {\n    let rotation = sand::command::Rotation::absolute(yaw, pitch);\n}",
    )]
    pub fn absolute(yaw: f64, pitch: f64) -> Self {
        Self::new(Coord::abs(yaw), Coord::abs(pitch))
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.yaw, self.pitch)
    }
}

impl Validate for Rotation {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.yaw
            .validate(profile)
            .map_err(|e| e.with_context("Rotation.yaw"))?;
        self.pitch
            .validate(profile)
            .map_err(|e| e.with_context("Rotation.pitch"))?;
        if matches!(self.yaw, Coord::Local(_)) || matches!(self.pitch, Coord::Local(_)) {
            return Err(CommandError::new(
                "Rotation",
                "coordinate_system",
                "rotations accept absolute or relative (`~`) angles, not local (`^`) coordinates",
            ));
        }
        Ok(())
    }
}

impl RenderCommand for Rotation {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

fn validate_triplet(
    x: &Coord,
    y: &Coord,
    z: &Coord,
    helper: &'static str,
    profile: &CommandProfile,
) -> CommandResult<()> {
    for (field, coord) in [("x", x), ("y", y), ("z", z)] {
        coord
            .validate(profile)
            .map_err(|e| e.with_context(format!("{helper}.{field}")))?;
    }
    let local_count = [x, y, z]
        .into_iter()
        .filter(|coord| matches!(coord, Coord::Local(_)))
        .count();
    if local_count != 0 && local_count != 3 {
        return Err(CommandError::new(
            helper,
            "coordinate_system",
            "local (`^`) coordinates cannot be mixed with absolute or relative (`~`) coordinates",
        ));
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_display() {
        assert_eq!(Coord::abs(10).to_string(), "10");
        assert_eq!(Coord::abs(10.5_f64).to_string(), "10.5");
        assert_eq!(Coord::rel().to_string(), "~");
        assert_eq!(Coord::rel_n(3).to_string(), "~3");
        assert_eq!(Coord::local().to_string(), "^");
        assert_eq!(Coord::local_n(2.5_f64).to_string(), "^2.5");
    }

    #[test]
    fn block_pos() {
        assert_eq!(BlockPos::here().to_string(), "~ ~ ~");
        assert_eq!(BlockPos::absolute(10, 64, -5).to_string(), "10 64 -5");
        assert_eq!(BlockPos::above(3).to_string(), "~ ~3 ~");
        assert_eq!(BlockPos::below(1).to_string(), "~ ~-1 ~");
    }

    #[test]
    fn vec3() {
        assert_eq!(Vec3::here().to_string(), "~ ~ ~");
        assert_eq!(Vec3::absolute(1.5, 64.0, -3.0).to_string(), "1.5 64 -3");
    }

    #[test]
    fn rotation() {
        assert_eq!(Rotation::here().to_string(), "~ ~");
        assert_eq!(Rotation::absolute(90.0, 0.0).to_string(), "90 0");
    }

    #[test]
    fn validation_rejects_non_finite_and_mixed_coordinates() {
        assert!(Coord::abs(f64::NAN).try_build().is_err());
        assert!(
            Vec3::new(Coord::local(), Coord::rel(), Coord::local())
                .try_build()
                .is_err()
        );
        assert!(
            Rotation::new(Coord::local(), Coord::local())
                .try_build()
                .is_err()
        );
    }

    #[test]
    fn block_positions_require_integral_absolute_values() {
        assert!(
            BlockPos::new(Coord::abs(1.5), Coord::abs(2), Coord::abs(3))
                .try_build()
                .is_err()
        );
        assert_eq!(BlockPos::absolute(1, 2, 3).try_build().unwrap(), "1 2 3");
    }

    /// Block-position arguments accept fractional relative and local offsets.
    #[test]
    fn block_positions_accept_relative_fractions_and_local_triplets() {
        for (position, expected) in [
            (
                BlockPos::new(Coord::rel_n(0.5), Coord::rel(), Coord::rel_n(-1.25)),
                "~0.5 ~ ~-1.25",
            ),
            (
                BlockPos::new(Coord::local_n(0.5), Coord::local(), Coord::local_n(1.25)),
                "^0.5 ^ ^1.25",
            ),
            (
                BlockPos::new(Coord::abs(1), Coord::rel_n(-0.5), Coord::abs(3)),
                "1 ~-0.5 3",
            ),
        ] {
            assert_eq!(position.try_build().unwrap(), expected);
        }
    }

    /// Absolute block coordinates must fit Minecraft's signed integer parser.
    #[test]
    fn block_positions_enforce_absolute_integer_bounds() {
        for value in [i32::MIN as f64, i32::MAX as f64] {
            assert!(BlockPos::new(value, 0, 0).try_build().is_ok());
        }
        for value in [i32::MIN as f64 - 1.0, i32::MAX as f64 + 1.0, 0.5] {
            let error = BlockPos::new(value, 0, 0).try_build().unwrap_err();
            assert_eq!(error.field, "x");
        }
    }

    /// Local coordinates remain all-or-nothing and every axis must be finite.
    #[test]
    fn block_positions_reject_mixed_modes_and_nonfinite_offsets() {
        for position in [
            BlockPos::new(Coord::local(), Coord::rel(), Coord::local()),
            BlockPos::new(Coord::local(), Coord::abs(0), Coord::local()),
            BlockPos::new(Coord::rel_n(f64::INFINITY), Coord::rel(), Coord::rel()),
            BlockPos::new(Coord::local(), Coord::local_n(f64::NAN), Coord::local()),
            BlockPos::new(0, 0, f64::NEG_INFINITY),
        ] {
            assert!(position.try_build().is_err());
        }
    }

    #[test]
    fn block_positions_accept_relative_integers() {
        assert_eq!(BlockPos::above(3).try_build().unwrap(), "~ ~3 ~");
        assert_eq!(
            BlockPos::new(Coord::rel_n(1), Coord::rel_n(-2), Coord::rel())
                .try_build()
                .unwrap(),
            "~1 ~-2 ~"
        );
    }

    #[test]
    fn coordinates_reject_infinity() {
        assert!(Coord::abs(f64::INFINITY).try_build().is_err());
        assert!(Coord::abs(f64::NEG_INFINITY).try_build().is_err());
        assert!(Vec3::absolute(f64::NAN, 0.0, 0.0).try_build().is_err());
        assert!(Rotation::absolute(f64::INFINITY, 0.0).try_build().is_err());
    }
}
