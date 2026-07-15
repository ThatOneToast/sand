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
    pub fn abs(v: impl Into<f64>) -> Self {
        Coord::Absolute(v.into())
    }
    /// Relative coordinate at the executor's position (`~`).
    pub fn rel() -> Self {
        Coord::Relative(0.0)
    }
    /// Relative coordinate with an offset (`~N`).
    pub fn rel_n(v: impl Into<f64>) -> Self {
        Coord::Relative(v.into())
    }
    /// Local coordinate (along executor's facing direction, `^`).
    pub fn local() -> Self {
        Coord::Local(0.0)
    }
    /// Local coordinate with an offset (`^N`).
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

/// Integer/relative block position used in commands like `setblock`, `fill`.
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

impl BlockPos {
    /// Create a block position from three coordinates.
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact block coordinates (`X Y Z`).
    pub fn absolute(x: i32, y: i32, z: i32) -> Self {
        Self::new(Coord::abs(x), Coord::abs(y), Coord::abs(z))
    }
    /// Position N blocks above current (`~ ~N ~`).
    pub fn above(n: i32) -> Self {
        Self::new(Coord::rel(), Coord::rel_n(n), Coord::rel())
    }
    /// Position N blocks below current (`~ ~-N ~`).
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
#[derive(Debug, Clone)]
#[must_use = "positions do nothing until passed to a command"]
pub struct Vec3 {
    pub x: Coord,
    pub y: Coord,
    pub z: Coord,
}

impl Vec3 {
    /// Create a 3D position from three coordinates.
    pub fn new(x: impl Into<Coord>, y: impl Into<Coord>, z: impl Into<Coord>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }
    /// Current position (`~ ~ ~`).
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel(), Coord::rel())
    }
    /// Exact world coordinates (`X Y Z`).
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
    pub fn new(yaw: impl Into<Coord>, pitch: impl Into<Coord>) -> Self {
        Self {
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }
    /// Current rotation (`~ ~`).
    pub fn here() -> Self {
        Self::new(Coord::rel(), Coord::rel())
    }
    /// Absolute yaw and pitch angles.
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
}
