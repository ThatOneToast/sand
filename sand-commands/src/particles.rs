//! Particle effect builders for Minecraft datapacks.
//!
//! # Quick-start
//!
//! ```rust,ignore
//! use sand_commands::{ParticleBuilder, Particle, ParticleSpread};
//!
//! // Colored dust ring
//! let cmds = ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.5))
//!     .circle(2.0, 1.0, 32);
//!
//! // Arbitrary point list
//! let cmds = ParticleBuilder::new(Particle::named("minecraft:end_rod"))
//!     .points_at(&[[0.0,0.0,0.0],[1.0,1.0,0.0],[2.0,0.0,0.0]]);
//! ```

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};

// ── Particle ──────────────────────────────────────────────────────────────────

/// A Minecraft particle type with its parameters.
#[derive(Debug, Clone)]
pub enum Particle {
    /// A named particle with no extra parameters, e.g. `"minecraft:flame"`.
    Named(String),
    /// Colored `minecraft:dust` particle. RGB values in `0.0–1.0`.
    Dust { r: f32, g: f32, b: f32, scale: f32 },
    /// `minecraft:dust_color_transition` — animates from one color to another.
    DustColorTransition {
        from_r: f32,
        from_g: f32,
        from_b: f32,
        to_r: f32,
        to_g: f32,
        to_b: f32,
        scale: f32,
    },
    /// `minecraft:block` particle showing a block's break texture.
    Block(String),
    /// `minecraft:item` particle showing an item's texture.
    Item(String),
    /// `minecraft:sculk_charge` with a rotation in radians.
    SculkCharge { roll: f32 },
    /// `minecraft:shriek` with a delay in ticks before appearing.
    Shriek { delay: u32 },
    /// Explicit opaque particle token.
    Raw(String),
}

impl Particle {
    /// A named particle with no extra parameters (e.g. `"minecraft:flame"`).
    pub fn named(name: impl Into<String>) -> Self {
        Particle::Named(name.into())
    }

    /// Create an intentionally opaque particle token.
    ///
    /// Sand renders this unchanged and does not apply particle-specific
    /// compatibility checks.
    pub fn raw_token(token: impl Into<String>) -> Self {
        Self::Raw(token.into())
    }

    /// Colored dust particle. RGB values in `0.0–1.0`, scale is size (1.0 = default).
    pub fn dust(r: f32, g: f32, b: f32, scale: f32) -> Self {
        Particle::Dust { r, g, b, scale }
    }

    /// Colored dust from 8-bit RGB (0–255).
    pub fn dust_u8(r: u8, g: u8, b: u8, scale: f32) -> Self {
        Particle::Dust {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            scale,
        }
    }

    /// Colored dust from a hex RGB value, e.g. `0xFF4400` for orange.
    pub fn dust_hex(hex: u32, scale: f32) -> Self {
        Particle::dust_u8(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
            scale,
        )
    }

    /// Color-transitioning dust. RGB values in `0.0–1.0`.
    pub fn dust_transition(
        from_r: f32,
        from_g: f32,
        from_b: f32,
        to_r: f32,
        to_g: f32,
        to_b: f32,
        scale: f32,
    ) -> Self {
        Particle::DustColorTransition {
            from_r,
            from_g,
            from_b,
            to_r,
            to_g,
            to_b,
            scale,
        }
    }

    /// Color-transitioning dust from two hex RGB values.
    pub fn dust_transition_hex(from_hex: u32, to_hex: u32, scale: f32) -> Self {
        let [fr, fg, fb] = hex_to_f32(from_hex);
        let [tr, tg, tb] = hex_to_f32(to_hex);
        Particle::DustColorTransition {
            from_r: fr,
            from_g: fg,
            from_b: fb,
            to_r: tr,
            to_g: tg,
            to_b: tb,
            scale,
        }
    }

    /// Block break texture particle, e.g. `"minecraft:stone"`.
    pub fn block(state: impl Into<String>) -> Self {
        Particle::Block(state.into())
    }

    /// Item texture particle, e.g. `"minecraft:diamond_sword"`.
    pub fn item(item: impl Into<String>) -> Self {
        Particle::Item(item.into())
    }

    /// `minecraft:sculk_charge` with a roll angle in radians.
    pub fn sculk_charge(roll: f32) -> Self {
        Particle::SculkCharge { roll }
    }

    /// `minecraft:shriek` with a delay in ticks.
    pub fn shriek(delay: u32) -> Self {
        Particle::Shriek { delay }
    }

    fn command_token(&self) -> String {
        match self {
            Particle::Named(n) => n.clone(),
            Particle::Dust { r, g, b, scale } => {
                format!(
                    "minecraft:dust{{color:[{},{},{}],scale:{}}}",
                    fmt_c(*r),
                    fmt_c(*g),
                    fmt_c(*b),
                    fmt_c(*scale)
                )
            }
            Particle::DustColorTransition {
                from_r,
                from_g,
                from_b,
                to_r,
                to_g,
                to_b,
                scale,
            } => {
                format!(
                    "minecraft:dust_color_transition{{from_color:[{},{},{}],to_color:[{},{},{}],scale:{}}}",
                    fmt_c(*from_r),
                    fmt_c(*from_g),
                    fmt_c(*from_b),
                    fmt_c(*to_r),
                    fmt_c(*to_g),
                    fmt_c(*to_b),
                    fmt_c(*scale),
                )
            }
            Particle::Block(s) => format!("minecraft:block {s}"),
            Particle::Item(s) => format!("minecraft:item {s}"),
            Particle::SculkCharge { roll } => format!("minecraft:sculk_charge {}", fmt_c(*roll)),
            Particle::Shriek { delay } => format!("minecraft:shriek {delay}"),
            Particle::Raw(token) => token.clone(),
        }
    }
}

impl Validate for Particle {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Named(id) => {
                crate::validate::resource_location_shape(id, "ParticleCommand", "particle.id")
                    .map(|_| ())
                    .map_err(|error| particle_error("SAND-PARTICLE-ID", error.field, error.message))
            }
            Self::Dust { r, g, b, scale } => {
                validate_color(*r, "particle.color.r")?;
                validate_color(*g, "particle.color.g")?;
                validate_color(*b, "particle.color.b")?;
                validate_scale(*scale)
            }
            Self::DustColorTransition {
                from_r,
                from_g,
                from_b,
                to_r,
                to_g,
                to_b,
                scale,
            } => {
                for (field, value) in [
                    ("particle.from_color.r", *from_r),
                    ("particle.from_color.g", *from_g),
                    ("particle.from_color.b", *from_b),
                    ("particle.to_color.r", *to_r),
                    ("particle.to_color.g", *to_g),
                    ("particle.to_color.b", *to_b),
                ] {
                    validate_color(value, field)?;
                }
                validate_scale(*scale)
            }
            Self::Block(state) => validate_particle_payload_id(state, "particle.block"),
            Self::Item(item) => validate_particle_payload_id(item, "particle.item"),
            Self::SculkCharge { roll } => validate_finite(*roll as f64, "particle.roll"),
            Self::Shriek { .. } | Self::Raw(_) => Ok(()),
        }
    }
}

// ── ParticleSpread ─────────────────────────────────────────────────────────────

/// Spread/dispersion of a particle from its spawn position.
#[derive(Debug, Clone)]
pub struct ParticleSpread {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

impl ParticleSpread {
    /// No spread — particles appear exactly at the specified position.
    pub const POINT: Self = Self {
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
    };

    /// Uniform spread in all three directions.
    pub fn uniform(v: f64) -> Self {
        Self {
            dx: v,
            dy: v,
            dz: v,
        }
    }

    /// Custom per-axis spread.
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self { dx, dy, dz }
    }
}

impl Validate for ParticleSpread {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        for (field, value) in [
            ("spread.dx", self.dx),
            ("spread.dy", self.dy),
            ("spread.dz", self.dz),
        ] {
            validate_non_negative(value, field)?;
        }
        Ok(())
    }
}

/// One structured `particle` command retained until validation and rendering.
#[derive(Debug, Clone)]
pub struct ParticleCommand {
    particle: Particle,
    position: [f64; 3],
    spread: ParticleSpread,
    speed: f64,
    count: u32,
    force: bool,
}

impl Validate for ParticleCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.particle.validate(profile)?;
        self.spread.validate(profile)?;
        validate_non_negative(self.speed, "speed")?;
        if self.count == 0 {
            return Err(particle_error(
                "SAND-PARTICLE-COUNT",
                "count",
                "particles_per_point must be greater than zero",
            ));
        }
        for (field, value) in [
            ("position.x", self.position[0]),
            ("position.y", self.position[1]),
            ("position.z", self.position[2]),
        ] {
            validate_finite(value, field)?;
        }
        Ok(())
    }
}

impl RenderCommand for ParticleCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        let mode = if self.force { "force" } else { "normal" };
        format!(
            "particle {} ~{} ~{} ~{} {} {} {} {} {} {mode}",
            self.particle.command_token(),
            fmt_f(self.position[0]),
            fmt_f(self.position[1]),
            fmt_f(self.position[2]),
            fmt_f(self.spread.dx),
            fmt_f(self.spread.dy),
            fmt_f(self.spread.dz),
            fmt_f(self.speed),
            self.count,
        )
    }
}

// ── ParticleBuilder ────────────────────────────────────────────────────────────

/// Fluent builder for generating `particle` commands.
#[derive(Debug, Clone)]
pub struct ParticleBuilder {
    particle: Particle,
    spread: ParticleSpread,
    speed: f64,
    particles_per_point: u32,
    force: bool,
}

impl ParticleBuilder {
    /// Create a new builder for the given particle type.
    ///
    /// Defaults: `spread = POINT`, `speed = 0`, `particles_per_point = 1`, `force = true`.
    pub fn new(particle: Particle) -> Self {
        Self {
            particle,
            spread: ParticleSpread::POINT,
            speed: 0.0,
            particles_per_point: 1,
            force: true,
        }
    }

    /// Set the random spread box around each particle's spawn position.
    pub fn spread(mut self, spread: ParticleSpread) -> Self {
        self.spread = spread;
        self
    }

    /// Set the initial speed of each particle after spawning.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Number of particles Minecraft spawns per command call (default: `1`).
    pub fn particles_per_point(mut self, n: u32) -> Self {
        self.particles_per_point = n;
        self
    }

    /// Whether to use `force` visibility mode (default: `true`).
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Validate the particle and numeric command settings without rendering.
    pub fn validate(&self) -> CommandResult<()> {
        self.command_at([0.0, 0.0, 0.0])
            .validate(&CommandProfile::unprofiled())
    }

    /// Fallible point renderer used by VFX and strict callers.
    pub fn try_points_at(&self, pts: &[[f64; 3]]) -> CommandResult<Vec<String>> {
        if pts.is_empty() {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "points",
                "geometry must contain at least one point",
            ));
        }
        pts.iter()
            .map(|point| self.command_at(*point).try_build())
            .collect()
    }

    /// Fallible circle generator with explicit geometry diagnostics.
    pub fn try_circle(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.circle(radius, y_offset, points))
    }

    pub fn try_arc(
        &self,
        radius: f64,
        y_offset: f64,
        start_deg: f64,
        end_deg: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        for (field, value) in [
            ("geometry.y_offset", y_offset),
            ("geometry.start_degrees", start_deg),
            ("geometry.end_degrees", end_deg),
        ] {
            validate_finite(value, field)?;
        }
        validate_points(points)?;
        Ok(self.arc(radius, y_offset, start_deg, end_deg, points))
    }

    pub fn try_polygon(
        &self,
        sides: usize,
        radius: f64,
        y_offset: f64,
        points_per_side: usize,
    ) -> CommandResult<Vec<String>> {
        if sides < 3 {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.sides",
                "polygon sides must be at least 3",
            ));
        }
        validate_points(points_per_side)?;
        sides.checked_mul(points_per_side).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "polygon output count overflows `usize`",
            )
        })?;
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        Ok(self.polygon(sides, radius, y_offset, points_per_side))
    }

    pub fn try_star(
        &self,
        arms: usize,
        outer_radius: f64,
        inner_radius: f64,
        y_offset: f64,
    ) -> CommandResult<Vec<String>> {
        if arms < 2 || arms.checked_mul(2).is_none() {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.arms",
                "star arms must be at least 2 and small enough to double safely",
            ));
        }
        validate_geometry_non_negative(outer_radius, "geometry.outer_radius")?;
        validate_geometry_non_negative(inner_radius, "geometry.inner_radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        Ok(self.star(arms, outer_radius, inner_radius, y_offset))
    }

    /// Fallible sphere generator with explicit geometry diagnostics.
    pub fn try_sphere(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.sphere(radius, y_offset, points))
    }

    /// Fallible helix generator with explicit geometry diagnostics.
    pub fn try_helix(
        &self,
        radius: f64,
        height: f64,
        turns: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_geometry_non_negative(height, "geometry.height")?;
        validate_non_negative(turns, "geometry.turns")?;
        if turns == 0.0 {
            return Err(particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.turns",
                "helix turns must be greater than zero",
            ));
        }
        validate_points(points)?;
        Ok(self.helix(radius, height, turns, points))
    }

    pub fn try_double_helix(
        &self,
        radius: f64,
        height: f64,
        turns: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        self.try_helix(radius, height, turns, points)?;
        Ok(self.double_helix(radius, height, turns, points))
    }

    pub fn try_line(
        &self,
        from: [f64; 3],
        to: [f64; 3],
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_points(points)?;
        for (index, value) in from.into_iter().chain(to).enumerate() {
            validate_finite(value, format!("geometry.coordinate[{index}]"))?;
        }
        Ok(self.line(from, to, points))
    }

    pub fn try_disc(
        &self,
        radius: f64,
        y_offset: f64,
        density: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(density)?;
        Ok(self.disc(radius, y_offset, density))
    }

    pub fn try_torus(
        &self,
        major_radius: f64,
        minor_radius: f64,
        y_offset: f64,
        rings: usize,
        segments_per_ring: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(major_radius, "geometry.major_radius")?;
        validate_geometry_non_negative(minor_radius, "geometry.minor_radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(rings)?;
        validate_points(segments_per_ring)?;
        rings.checked_mul(segments_per_ring).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "torus output count overflows `usize`",
            )
        })?;
        Ok(self.torus(
            major_radius,
            minor_radius,
            y_offset,
            rings,
            segments_per_ring,
        ))
    }

    pub fn try_cone(
        &self,
        base_radius: f64,
        height: f64,
        y_offset: f64,
        rings: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(base_radius, "geometry.base_radius")?;
        validate_geometry_non_negative(height, "geometry.height")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(rings)?;
        Ok(self.cone(base_radius, height, y_offset, rings))
    }

    pub fn try_burst(
        &self,
        radius: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(radius, "geometry.radius")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.burst(radius, y_offset, points))
    }

    pub fn try_wave(
        &self,
        length: f64,
        amplitude: f64,
        cycles: f64,
        y_offset: f64,
        points: usize,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(length, "geometry.length")?;
        validate_geometry_non_negative(amplitude, "geometry.amplitude")?;
        validate_non_negative(cycles, "geometry.cycles")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(points)?;
        Ok(self.wave(length, amplitude, cycles, y_offset, points))
    }

    pub fn try_grid(
        &self,
        width: f64,
        depth: f64,
        cols: usize,
        rows: usize,
        y_offset: f64,
    ) -> CommandResult<Vec<String>> {
        validate_geometry_non_negative(width, "geometry.width")?;
        validate_geometry_non_negative(depth, "geometry.depth")?;
        validate_finite(y_offset, "geometry.y_offset")?;
        validate_points(cols)?;
        validate_points(rows)?;
        cols.checked_mul(rows).ok_or_else(|| {
            particle_error(
                "SAND-PARTICLE-GEOMETRY",
                "geometry.points",
                "grid output count overflows `usize`",
            )
        })?;
        Ok(self.grid(width, depth, cols, rows, y_offset))
    }

    // ── Shape generators ──────────────────────────────────────────────────────

    /// A horizontal ring of particles at `y_offset` above the executor.
    pub fn circle(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let a = TAU * i as f64 / points as f64;
                self.cmd(radius * a.cos(), y_offset, radius * a.sin())
            })
            .collect()
    }

    /// A partial arc of a circle, from `start_deg` to `end_deg` (degrees).
    pub fn arc(
        &self,
        radius: f64,
        y_offset: f64,
        start_deg: f64,
        end_deg: f64,
        points: usize,
    ) -> Vec<String> {
        if points == 0 {
            return vec![];
        }
        let start = start_deg.to_radians();
        let end = end_deg.to_radians();
        let steps = if points == 1 { 1 } else { points - 1 };
        (0..points)
            .map(|i| {
                let a = start + (end - start) * i as f64 / steps as f64;
                self.cmd(radius * a.cos(), y_offset, radius * a.sin())
            })
            .collect()
    }

    /// A regular polygon at `y_offset`. `points_per_side` particles per edge.
    pub fn polygon(
        &self,
        sides: usize,
        radius: f64,
        y_offset: f64,
        points_per_side: usize,
    ) -> Vec<String> {
        if sides < 3 || points_per_side == 0 {
            return vec![];
        }
        let mut cmds = Vec::new();
        for side in 0..sides {
            let a1 = TAU * side as f64 / sides as f64;
            let a2 = TAU * (side + 1) as f64 / sides as f64;
            let (x1, z1) = (radius * a1.cos(), radius * a1.sin());
            let (x2, z2) = (radius * a2.cos(), radius * a2.sin());
            let steps = points_per_side.max(1);
            for p in 0..steps {
                let t = p as f64 / steps as f64;
                cmds.push(self.cmd(x1 + (x2 - x1) * t, y_offset, z1 + (z2 - z1) * t));
            }
        }
        cmds
    }

    /// A star shape with `arms` points, alternating outer and inner radii.
    pub fn star(
        &self,
        arms: usize,
        outer_radius: f64,
        inner_radius: f64,
        y_offset: f64,
    ) -> Vec<String> {
        if arms < 2 {
            return vec![];
        }
        let total = arms * 2;
        (0..total)
            .map(|i| {
                let a = TAU * i as f64 / total as f64;
                let r = if i % 2 == 0 {
                    outer_radius
                } else {
                    inner_radius
                };
                self.cmd(r * a.cos(), y_offset, r * a.sin())
            })
            .collect()
    }

    /// A sphere surface using the Fibonacci lattice for even distribution.
    pub fn sphere(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        let gr = (1.0 + 5.0_f64.sqrt()) / 2.0;
        (0..points)
            .map(|i| {
                let theta = TAU * i as f64 / gr;
                let phi = ((1.0 - 2.0 * (i as f64 + 0.5) / points as f64).clamp(-1.0, 1.0)).acos();
                self.cmd(
                    radius * phi.sin() * theta.cos(),
                    radius * phi.cos() + y_offset,
                    radius * phi.sin() * theta.sin(),
                )
            })
            .collect()
    }

    /// The upper hemisphere only (y ≥ y_offset).
    pub fn hemisphere(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        self.sphere(radius, y_offset, points * 2)
            .into_iter()
            .filter(|cmd| {
                let y_val = extract_relative_y(cmd);
                y_val >= y_offset - 1e-9
            })
            .take(points)
            .collect()
    }

    /// A rising spiral helix.
    pub fn helix(&self, radius: f64, height: f64, turns: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let t = i as f64 / (points as f64 - 1.0).max(1.0);
                let a = TAU * turns * t;
                self.cmd(radius * a.cos(), height * t, radius * a.sin())
            })
            .collect()
    }

    /// Two interleaved helices (double-helix / DNA shape).
    pub fn double_helix(&self, radius: f64, height: f64, turns: f64, points: usize) -> Vec<String> {
        let half = points / 2;
        let mut cmds = self.helix(radius, height, turns, half);
        let rest = points - half;
        cmds.extend((0..rest).map(|i| {
            let t = i as f64 / (rest as f64 - 1.0).max(1.0);
            let a = TAU * turns * t + std::f64::consts::PI;
            self.cmd(radius * a.cos(), height * t, radius * a.sin())
        }));
        cmds
    }

    /// A straight line from `from` to `to` (both relative to executor).
    pub fn line(&self, from: [f64; 3], to: [f64; 3], points: usize) -> Vec<String> {
        match points {
            0 => vec![],
            1 => vec![self.cmd(from[0], from[1], from[2])],
            _ => (0..points)
                .map(|i| {
                    let t = i as f64 / (points - 1) as f64;
                    self.cmd(
                        from[0] + (to[0] - from[0]) * t,
                        from[1] + (to[1] - from[1]) * t,
                        from[2] + (to[2] - from[2]) * t,
                    )
                })
                .collect(),
        }
    }

    /// A filled disc of concentric rings. `density` controls ring count per unit radius.
    pub fn disc(&self, radius: f64, y_offset: f64, density: usize) -> Vec<String> {
        let rings = (radius * density as f64).ceil() as usize;
        let mut cmds = vec![self.cmd(0.0, y_offset, 0.0)];
        for ring in 1..=rings {
            let r = radius * ring as f64 / rings as f64;
            let pts = ((TAU * r * density as f64).ceil() as usize).max(4);
            cmds.extend(self.circle(r, y_offset, pts));
        }
        cmds
    }

    /// A 3D torus (donut shape).
    pub fn torus(
        &self,
        major_radius: f64,
        minor_radius: f64,
        y_offset: f64,
        rings: usize,
        segments_per_ring: usize,
    ) -> Vec<String> {
        let mut cmds = Vec::new();
        for ring in 0..rings {
            let phi = TAU * ring as f64 / rings as f64;
            let cx = major_radius * phi.cos();
            let cz = major_radius * phi.sin();
            for seg in 0..segments_per_ring {
                let theta = TAU * seg as f64 / segments_per_ring as f64;
                let x = cx + minor_radius * theta.cos() * phi.cos();
                let y = minor_radius * theta.sin() + y_offset;
                let z = cz + minor_radius * theta.cos() * phi.sin();
                cmds.push(self.cmd(x, y, z));
            }
        }
        cmds
    }

    /// A cone rising from the base ring up to an apex.
    pub fn cone(&self, base_radius: f64, height: f64, y_offset: f64, rings: usize) -> Vec<String> {
        let mut cmds = Vec::new();
        let ring_count = rings.max(1);
        for ring in 0..=ring_count {
            let t = ring as f64 / ring_count as f64;
            let r = base_radius * (1.0 - t);
            let y = height * t + y_offset;
            let pts = ((TAU * r * 8.0).ceil() as usize).max(4);
            for i in 0..pts {
                let a = TAU * i as f64 / pts as f64;
                cmds.push(self.cmd(r * a.cos(), y, r * a.sin()));
            }
        }
        cmds
    }

    /// An outward burst (sphere with extra spread, simulates an explosion).
    pub fn burst(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        let boosted = Self {
            spread: ParticleSpread::new(
                self.spread.dx + radius * 0.15,
                self.spread.dy + radius * 0.15,
                self.spread.dz + radius * 0.15,
            ),
            ..self.clone()
        };
        boosted.sphere(radius, y_offset, points)
    }

    /// A horizontal sine wave along the X axis.
    pub fn wave(
        &self,
        length: f64,
        amplitude: f64,
        cycles: f64,
        y_offset: f64,
        points: usize,
    ) -> Vec<String> {
        if points == 0 {
            return vec![];
        }
        (0..points)
            .map(|i| {
                let t = i as f64 / (points as f64 - 1.0).max(1.0);
                let x = length * t - length / 2.0;
                let y = amplitude * (TAU * cycles * t).sin() + y_offset;
                self.cmd(x, y, 0.0)
            })
            .collect()
    }

    /// A flat rectangular grid of particles.
    pub fn grid(
        &self,
        width: f64,
        depth: f64,
        cols: usize,
        rows: usize,
        y_offset: f64,
    ) -> Vec<String> {
        let mut cmds = Vec::new();
        let cols = cols.max(1);
        let rows = rows.max(1);
        for row in 0..rows {
            for col in 0..cols {
                let x = if cols > 1 {
                    -width / 2.0 + width * col as f64 / (cols - 1) as f64
                } else {
                    0.0
                };
                let z = if rows > 1 {
                    -depth / 2.0 + depth * row as f64 / (rows - 1) as f64
                } else {
                    0.0
                };
                cmds.push(self.cmd(x, y_offset, z));
            }
        }
        cmds
    }

    /// Spawn a particle at each point in the given list (relative offsets from executor).
    pub fn points_at(&self, pts: &[[f64; 3]]) -> Vec<String> {
        pts.iter().map(|[x, y, z]| self.cmd(*x, *y, *z)).collect()
    }

    fn cmd(&self, x: f64, y: f64, z: f64) -> String {
        let command = self.command_at([x, y, z]);
        let line = command.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, command);
        line
    }

    fn command_at(&self, position: [f64; 3]) -> ParticleCommand {
        ParticleCommand {
            particle: self.particle.clone(),
            position,
            spread: self.spread.clone(),
            speed: self.speed,
            count: self.particles_per_point,
            force: self.force,
        }
    }
}

// ── ParticleEffect ────────────────────────────────────────────────────────────

/// Static particle geometry generators (thin wrappers around [`ParticleBuilder`]).
pub struct ParticleEffect;

impl ParticleEffect {
    /// Horizontal ring of particles.
    pub fn circle(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .circle(radius, y_offset, count)
    }

    /// Sphere surface (Fibonacci distribution).
    pub fn sphere(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .sphere(radius, y_offset, count)
    }

    /// Rising spiral helix.
    pub fn helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .helix(radius, height, turns, count)
    }

    /// Straight line between two relative points.
    #[allow(clippy::too_many_arguments)]
    pub fn line(
        particle: &str,
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .line([x1, y1, z1], [x2, y2, z2], count)
    }

    /// Outward burst (sphere with boosted spread).
    pub fn burst(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .burst(radius, y_offset, count)
    }

    /// Two interleaved helices.
    pub fn double_helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .double_helix(radius, height, turns, count)
    }

    /// Filled disc of concentric rings.
    pub fn disc(
        particle: &str,
        radius: f64,
        y_offset: f64,
        density: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        ParticleBuilder::new(Particle::named(particle))
            .spread(spread.clone())
            .disc(radius, y_offset, density)
    }
}

// ── Private helpers ────────────────────────────────────────────────────────────

const TAU: f64 = std::f64::consts::TAU;

fn hex_to_f32(hex: u32) -> [f32; 3] {
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    ]
}

fn fmt_f(v: f64) -> String {
    let rounded = (v * 10000.0).round() / 10000.0;
    if rounded == rounded.trunc() && rounded.abs() < 1e15 {
        format!("{}", rounded as i64)
    } else {
        let s = format!("{:.4}", rounded);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn fmt_c(v: f32) -> String {
    let rounded = (v * 100.0).round() / 100.0;
    format!("{:.2}", rounded)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn extract_relative_y(cmd: &str) -> f64 {
    let mut tilde_count = 0;
    for token in cmd.split_whitespace() {
        if let Some(rest) = token.strip_prefix('~') {
            tilde_count += 1;
            if tilde_count == 2 {
                return rest.parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    0.0
}

fn particle_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("ParticleCommand", field, message).with_code(code)
}

fn validate_finite(value: f64, field: impl Into<String>) -> CommandResult<()> {
    let field = field.into();
    if value.is_finite() {
        Ok(())
    } else {
        Err(particle_error(
            "SAND-PARTICLE-NUMERIC",
            field,
            format!("must be finite, got `{value}`"),
        ))
    }
}

fn validate_non_negative(value: f64, field: impl Into<String>) -> CommandResult<()> {
    let field = field.into();
    validate_finite(value, field.clone())?;
    if value < 0.0 {
        Err(particle_error(
            "SAND-PARTICLE-NUMERIC",
            field,
            format!("must be non-negative, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_geometry_non_negative(value: f64, field: &'static str) -> CommandResult<()> {
    validate_non_negative(value, field)
        .map_err(|error| particle_error("SAND-PARTICLE-GEOMETRY", error.field, error.message))
}

fn validate_points(points: usize) -> CommandResult<()> {
    if points == 0 {
        Err(particle_error(
            "SAND-PARTICLE-GEOMETRY",
            "geometry.points",
            "point count must be greater than zero",
        ))
    } else {
        Ok(())
    }
}

fn validate_color(value: f32, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        Err(particle_error(
            "SAND-PARTICLE-COLOR",
            field,
            format!("RGB channels must be finite and within 0.0..=1.0, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_scale(scale: f32) -> CommandResult<()> {
    if !scale.is_finite() || scale <= 0.0 {
        Err(particle_error(
            "SAND-PARTICLE-SCALE",
            "particle.scale",
            format!("dust scale must be finite and greater than zero, got `{scale}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_particle_payload_id(value: &str, field: &'static str) -> CommandResult<()> {
    let id = value
        .split_once(['[', '{', ' '])
        .map_or(value, |(id, _)| id);
    crate::validate::resource_location_shape(id, "ParticleCommand", field)
        .map(|_| ())
        .map_err(|error| particle_error("SAND-PARTICLE-ID", field, error.message))
}

fn registered_lines() -> &'static Mutex<BTreeMap<String, ParticleCommand>> {
    static LINES: OnceLock<Mutex<BTreeMap<String, ParticleCommand>>> = OnceLock::new();
    LINES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_line(line: &str, command: ParticleCommand) {
    registered_lines()
        .lock()
        .expect("particle command registry mutex poisoned")
        .insert(line.to_owned(), command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    let command = registered_lines()
        .lock()
        .expect("particle command registry mutex poisoned")
        .get(line)
        .cloned();
    match command {
        Some(command) => command.validate(profile),
        None => Ok(()),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn builder(name: &str) -> ParticleBuilder {
        ParticleBuilder::new(Particle::named(name))
    }

    #[test]
    fn circle_count() {
        let cmds = builder("minecraft:flame").circle(2.0, 0.0, 8);
        assert_eq!(cmds.len(), 8);
        assert!(cmds[0].starts_with("particle minecraft:flame"));
        assert!(
            cmds[0].contains("~2 ~0 ~0"),
            "first point at radius: {}",
            cmds[0]
        );
    }

    #[test]
    fn arc_partial() {
        let cmds = builder("minecraft:flame").arc(2.0, 0.0, 0.0, 90.0, 4);
        assert_eq!(cmds.len(), 4);
        assert!(cmds[0].contains("~2 ~0 ~0"), "arc start: {}", cmds[0]);
        assert!(cmds[3].contains("~0 ~0 ~2"), "arc end: {}", cmds[3]);
    }

    #[test]
    fn polygon_sides() {
        let cmds = builder("minecraft:crit").polygon(4, 2.0, 0.0, 4);
        assert_eq!(cmds.len(), 16);
    }

    #[test]
    fn star_arms() {
        let cmds = builder("minecraft:end_rod").star(5, 2.0, 0.8, 0.0);
        assert_eq!(cmds.len(), 10);
    }

    #[test]
    fn sphere_count() {
        let cmds = builder("minecraft:end_rod").sphere(3.0, 0.0, 20);
        assert_eq!(cmds.len(), 20);
    }

    #[test]
    fn helix_count() {
        let cmds = builder("minecraft:soul_fire_flame").helix(1.0, 5.0, 2.0, 30);
        assert_eq!(cmds.len(), 30);
    }

    #[test]
    fn double_helix_count() {
        let cmds = builder("minecraft:flame").double_helix(1.0, 4.0, 2.0, 40);
        assert_eq!(cmds.len(), 40);
    }

    #[test]
    fn line_endpoints() {
        let cmds = builder("minecraft:crit").line([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], 4);
        assert_eq!(cmds.len(), 4);
        assert!(cmds[0].contains("~0 ~0 ~0"), "start: {}", cmds[0]);
        assert!(cmds[3].contains("~3 ~0 ~0"), "end: {}", cmds[3]);
    }

    #[test]
    fn torus_count() {
        let cmds = builder("minecraft:end_rod").torus(3.0, 0.8, 0.0, 16, 8);
        assert_eq!(cmds.len(), 16 * 8);
    }

    #[test]
    fn cone_apex_at_top() {
        let cmds = builder("minecraft:flame").cone(2.0, 3.0, 0.0, 4);
        let apex = cmds.last().unwrap();
        assert!(apex.contains("~0 ~3 ~0"), "apex: {apex}");
    }

    #[test]
    fn wave_count() {
        let cmds = builder("minecraft:witch").wave(8.0, 1.0, 2.0, 0.0, 32);
        assert_eq!(cmds.len(), 32);
    }

    #[test]
    fn grid_count() {
        let cmds = builder("minecraft:flame").grid(4.0, 4.0, 5, 5, 0.0);
        assert_eq!(cmds.len(), 25);
    }

    #[test]
    fn points_at() {
        let pts = [[0.0_f64, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let cmds = builder("minecraft:flash").points_at(&pts);
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn dust_colored_format() {
        let cmds = ParticleBuilder::new(Particle::dust(1.0, 0.0, 0.0, 1.0)).circle(1.0, 0.0, 4);
        assert!(
            cmds[0].starts_with("particle minecraft:dust{color:[1,0,0],scale:1}"),
            "{}",
            cmds[0]
        );
    }

    #[test]
    fn dust_hex_red() {
        let p = Particle::dust_hex(0xFF0000, 1.0);
        if let Particle::Dust { r, g, b, .. } = p {
            assert!((r - 1.0).abs() < 0.01);
            assert!(g < 0.01);
            assert!(b < 0.01);
        } else {
            panic!("not dust");
        }
    }

    #[test]
    fn dust_transition_format() {
        let cmds = ParticleBuilder::new(Particle::dust_transition_hex(0xFF0000, 0x0000FF, 1.0))
            .points_at(&[[0.0, 0.0, 0.0]]);
        assert!(
            cmds[0].starts_with("particle minecraft:dust_color_transition"),
            "{}",
            cmds[0]
        );
    }

    #[test]
    fn force_normal_mode() {
        let force_cmd = builder("minecraft:flame")
            .force(true)
            .points_at(&[[0.0, 0.0, 0.0]]);
        let normal_cmd = builder("minecraft:flame")
            .force(false)
            .points_at(&[[0.0, 0.0, 0.0]]);
        assert!(force_cmd[0].ends_with("force"), "{}", force_cmd[0]);
        assert!(normal_cmd[0].ends_with("normal"), "{}", normal_cmd[0]);
    }

    #[test]
    fn fmt_f_precision() {
        assert_eq!(fmt_f(0.0), "0");
        assert_eq!(fmt_f(1.0), "1");
        assert_eq!(fmt_f(1.5), "1.5");
        assert_eq!(fmt_f(1.2345678), "1.2346");
    }

    #[test]
    fn particle_validation_covers_ids_numbers_and_geometry() {
        let profile = CommandProfile::unprofiled();
        assert!(
            Particle::named("modded:custom_particle")
                .validate(&profile)
                .is_ok()
        );
        assert_eq!(
            Particle::named("Bad Particle")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-ID"
        );
        assert_eq!(
            Particle::dust(f32::NAN, 0.0, 0.0, 1.0)
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-COLOR"
        );
        assert_eq!(
            Particle::dust(1.0, 0.0, 0.0, 0.0)
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-PARTICLE-SCALE"
        );
        let builder = ParticleBuilder::new(Particle::named("minecraft:flame"));
        assert!(builder.try_circle(-1.0, 0.0, 4).is_err());
        assert!(builder.try_helix(1.0, 2.0, 0.0, 4).is_err());
        assert!(builder.try_grid(1.0, 1.0, 0, 2, 0.0).is_err());
        assert!(builder.try_points_at(&[]).is_err());
    }

    #[test]
    fn particle_raw_token_is_opaque() {
        assert!(
            Particle::raw_token("modded payload")
                .validate(&CommandProfile::unprofiled())
                .is_ok()
        );
    }
}
