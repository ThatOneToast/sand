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

/// A single coordinate value: absolute, relative (`~`), or local (`^`).
#[derive(Debug, Clone, PartialEq)]
#[must_use = "coordinates do nothing until passed to a command"]
pub enum Coord {
    /// Absolute world coordinate.
    Absolute(f64),
    /// Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`.
    Relative(f64),
    /// Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`.
    Local(f64),
}

impl Coord {
    /// Absolute coordinate.
    #[doc = "**API Contract:** Run `sand api show sand::command::Coord::abs` for the canonical contract."]
    pub fn abs(v: impl Into<f64>) -> Self {
        Coord::Absolute(v.into())
    }
    /// Relative coordinate at the executor's position (`~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Coord::rel` for the canonical contract."]
    pub fn rel() -> Self {
        Coord::Relative(0.0)
    }
    /// Relative coordinate with an offset (`~N`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Coord::rel_n` for the canonical contract."]
    pub fn rel_n(v: impl Into<f64>) -> Self {
        Coord::Relative(v.into())
    }
    /// Local coordinate (along executor's facing direction, `^`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Coord::local` for the canonical contract."]
    pub fn local() -> Self {
        Coord::Local(0.0)
    }
    /// Local coordinate with an offset (`^N`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Coord::local_n` for the canonical contract."]
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

/// Integer/relative block position used in commands like `setblock`, `fill`,
/// `clone`, and block-targeted `data`.
///
/// # Contract (see [#169](https://github.com/ThatOneToast/sand/issues/169))
///
/// `BlockPos` models Minecraft's block-position grammar, which is stricter
/// than the generic position grammar accepted by [`Vec3`]:
///
/// - absolute coordinates (`10 64 -5`) must be **integers** — fractional
///   absolute values are rejected by [`Validate::validate`];
/// - relative coordinates (`~N`) must also be integral offsets;
/// - local (`^`) coordinates are **rejected** — block-position commands do
///   not accept them in vanilla Minecraft. Use [`Vec3`] (which does accept
///   local coordinates) for commands with generic position grammar, such as
///   `execute positioned`, `particle`, `summon`, or `tp`.
///
/// Do not reuse `BlockPos` as a generic fractional/local position — use
/// [`Vec3`] for that instead so the type itself documents the grammar a
/// command accepts.
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
    pub x: Coord,
    pub y: Coord,
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
    #[doc = "**API Contract:** Run `sand api show sand::command::BlockPos::new` for the canonical contract."]
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::BlockPos::here` for the canonical contract."]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact block coordinates (`X Y Z`).
    #[doc = "**API Contract:** Run `sand api show sand::command::BlockPos::absolute` for the canonical contract."]
    pub fn absolute(x: i32, y: i32, z: i32) -> Self {
        Self::new(Coord::abs(x), Coord::abs(y), Coord::abs(z))
    }
    /// Position N blocks above current (`~ ~N ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::BlockPos::above` for the canonical contract."]
    pub fn above(n: i32) -> Self {
        Self::new(Coord::rel(), Coord::rel_n(n), Coord::rel())
    }
    /// Position N blocks below current (`~ ~-N ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::BlockPos::below` for the canonical contract."]
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
            let value = coord_value(coord);
            if value.fract() != 0.0 {
                return Err(CommandError::new(
                    "BlockPos",
                    field,
                    format!("integer block coordinates cannot contain fractional value `{value}`"),
                ));
            }
            if matches!(coord, Coord::Local(_)) {
                return Err(CommandError::new(
                    "BlockPos",
                    field,
                    "block-position commands (`setblock`, `fill`, `clone`, block-targeted `data`) do not accept local (`^`) coordinates; use absolute or relative (`~`) coordinates, or a generic `Vec3` position for commands that support local coordinates",
                )
                .with_code("SAND-COORD-BLOCKPOS-LOCAL"));
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
///
/// # API Contract
///
/// `sand api show sand::command::Vec3`
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct Vec3 {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

impl Vec3 {
    /// Create a 3D position from three coordinates.
    ///
    /// # API Contract
    ///
    /// `sand api show sand::command::Vec3::new`
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Vec3::here` for the canonical contract."]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact world coordinates (`X Y Z`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Vec3::absolute` for the canonical contract."]
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

/// 2D column position (X Z), used in `locatebiome` etc.
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct Vec2 {
    pub x: Coord,
    pub z: Coord,
}

impl Vec2 {
    /// Create a 2D position (column) from X and Z coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Vec2::new` for the canonical contract."]
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

/// Yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.
#[derive(Debug, Clone)]
#[must_use = "rotations do nothing until passed to a command"]
pub struct Rotation {
    pub yaw: Coord,
    pub pitch: Coord,
}

impl Rotation {
    /// Create a rotation from yaw and pitch coordinates.
    #[doc = "**API Contract:** Run `sand api show sand::command::Rotation::new` for the canonical contract."]
    pub fn new(yaw: impl Into<Coord>, pitch: impl Into<Coord>) -> Self {
        Self {
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }
    /// Current rotation (`~ ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Rotation::here` for the canonical contract."]
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel())
    }
    /// Absolute yaw and pitch angles.
    #[doc = "**API Contract:** Run `sand api show sand::command::Rotation::absolute` for the canonical contract."]
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

fn coord_value(coord: &Coord) -> f64 {
    match coord {
        Coord::Absolute(value) | Coord::Relative(value) | Coord::Local(value) => *value,
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
    fn block_positions_require_integral_values() {
        assert!(
            BlockPos::new(Coord::abs(1.5), Coord::abs(2), Coord::abs(3))
                .try_build()
                .is_err()
        );
        assert_eq!(BlockPos::absolute(1, 2, 3).try_build().unwrap(), "1 2 3");
    }

    #[test]
    fn block_positions_reject_local_coordinates() {
        let err = BlockPos::new(Coord::local_n(1), Coord::local_n(2), Coord::local_n(3))
            .try_build()
            .unwrap_err();
        assert!(err.to_string().contains("local"), "{err}");
    }

    #[test]
    fn block_positions_local_rejection_uses_stable_diagnostic_code() {
        let err = BlockPos::new(Coord::local_n(1), Coord::local_n(2), Coord::local_n(3))
            .try_build()
            .unwrap_err();
        assert_eq!(err.code, "SAND-COORD-BLOCKPOS-LOCAL");
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
