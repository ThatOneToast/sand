//! Coordinate types for Minecraft commands.
//!
//! Minecraft uses three coordinate modes:
//! - **Absolute**: `X Y Z` — exact world coordinates
//! - **Relative** (`~`): `~X ~Y ~Z` — offset from the executor's position
//! - **Local** (`^`): `^X ^Y ^Z` — offset along the executor's facing direction

use std::fmt;

// ── Coord ─────────────────────────────────────────────────────────────────────

/// A single coordinate value: absolute, relative (`~`), or local (`^`).
#[derive(Debug, Clone, PartialEq)]
pub enum Coord {
    Absolute(f64),
    /// Relative (`~`) coordinate. `0.0` renders as `~`, otherwise `~N`.
    Relative(f64),
    /// Local (`^`) coordinate. `0.0` renders as `^`, otherwise `^N`.
    Local(f64),
}

impl Coord {
    /// Create an absolute coordinate.
    pub fn abs(v: impl Into<f64>) -> Self {
        Coord::Absolute(v.into())
    }
    /// Create a relative coordinate at the executor's position (`~`).
    pub fn rel() -> Self {
        Coord::Relative(0.0)
    }
    /// Create a relative coordinate with an offset (`~N`).
    pub fn rel_n(v: impl Into<f64>) -> Self {
        Coord::Relative(v.into())
    }
    /// Create a local coordinate (along executor's facing direction, `^`).
    pub fn local() -> Self {
        Coord::Local(0.0)
    }
    /// Create a local coordinate with an offset (`^N`).
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
        // Display as integer if it's a whole number
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

// ── BlockPos ──────────────────────────────────────────────────────────────────

/// Integer/relative block position used in commands like `setblock`, `fill`.
///
/// # Examples
/// ```
/// use sand_core::cmd::{BlockPos, Coord};
///
/// assert_eq!(BlockPos::absolute(10, 64, -5).to_string(), "10 64 -5");
/// assert_eq!(BlockPos::here().to_string(), "~ ~ ~");
/// assert_eq!(BlockPos::above(3).to_string(), "~ ~3 ~");
/// ```
#[derive(Debug, Clone)]
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

// ── Vec3 ──────────────────────────────────────────────────────────────────────

/// Floating-point position used in commands like `tp`, `summon`, `particle`.
#[derive(Debug, Clone)]
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

// ── Vec2 ──────────────────────────────────────────────────────────────────────

/// 2D column position (X Z), used in `locatebiome` etc.
#[derive(Debug, Clone)]
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

// ── Rotation ──────────────────────────────────────────────────────────────────

/// Yaw + pitch rotation (`yaw pitch`), used in `tp` and `execute rotated`.
#[derive(Debug, Clone)]
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
}
