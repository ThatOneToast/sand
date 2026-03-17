//! Particle effect builders for Minecraft datapacks.
//!
//! # Quick-start
//!
//! ```rust,ignore
//! use sand_core::cmd::{ParticleBuilder, Particle, ParticleSpread};
//!
//! // Colored dust ring
//! let cmds = ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.5))
//!     .circle(2.0, 1.0, 32);
//!
//! // Flame helix — build once, use for multiple shapes
//! let builder = ParticleBuilder::new(Particle::named("minecraft:flame"))
//!     .speed(0.02);
//! let bottom = builder.circle(1.5, 0.0, 24);
//! let top    = builder.helix(1.0, 3.0, 2.0, 48);
//!
//! // Arbitrary point list
//! let cmds = ParticleBuilder::new(Particle::named("minecraft:end_rod"))
//!     .points_at(&[[0.0,0.0,0.0],[1.0,1.0,0.0],[2.0,0.0,0.0]]);
//! ```

// ── Particle ──────────────────────────────────────────────────────────────────

/// A Minecraft particle type with its parameters.
///
/// Covers all particle types that require extra arguments in the
/// `particle` command (dust color, block state, item, etc.).
/// Use [`Particle::named`] for particles that take no extra parameters.
#[derive(Debug, Clone)]
pub enum Particle {
    /// A named particle with no extra parameters, e.g. `"minecraft:flame"`.
    Named(String),

    /// Colored `minecraft:dust` particle.
    ///
    /// RGB values are in `0.0–1.0`. `scale` controls the particle size (1.0 is default).
    /// Use [`Particle::dust`], [`Particle::dust_hex`], or [`Particle::dust_u8`] to construct.
    Dust { r: f32, g: f32, b: f32, scale: f32 },

    /// `minecraft:dust_color_transition` — animates from one color to another.
    ///
    /// Use [`Particle::dust_transition`] or [`Particle::dust_transition_hex`].
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
}

impl Particle {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// A named particle with no extra parameters (e.g. `"minecraft:flame"`).
    pub fn named(name: impl Into<String>) -> Self {
        Particle::Named(name.into())
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

    // ── Internal ──────────────────────────────────────────────────────────────

    /// Produce the particle name + params section of the `particle` command.
    fn command_token(&self) -> String {
        match self {
            Particle::Named(n) => n.clone(),
            Particle::Dust { r, g, b, scale } => {
                // 1.21+ format: minecraft:dust{color:[r,g,b],scale:s}
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
                // 1.21+ format: minecraft:dust_color_transition{from_color:[r,g,b],to_color:[r,g,b],scale:s}
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
        }
    }
}

// ── ParticleSpread ─────────────────────────────────────────────────────────────

/// Spread/dispersion of a particle from its spawn position.
///
/// `(dx, dy, dz)` are half-extents of the random offset box.
/// `ParticleSpread::POINT` means all particles appear exactly at the
/// specified coordinates.
#[derive(Debug, Clone)]
pub struct ParticleSpread {
    /// Half-extent spread along the X axis.
    pub dx: f64,
    /// Half-extent spread along the Y axis.
    pub dy: f64,
    /// Half-extent spread along the Z axis.
    pub dz: f64,
}

impl ParticleSpread {
    /// No spread — particles appear exactly at the specified position.
    pub const POINT: Self = Self { dx: 0.0, dy: 0.0, dz: 0.0 };

    /// Uniform spread in all three directions.
    pub fn uniform(v: f64) -> Self {
        Self { dx: v, dy: v, dz: v }
    }

    /// Custom per-axis spread.
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self { dx, dy, dz }
    }
}

// ── ParticleBuilder ────────────────────────────────────────────────────────────

/// Fluent builder for generating `particle` commands.
///
/// Configure the particle type, appearance, and behavior with chained methods,
/// then call a shape method to generate the commands.
///
/// # Examples
///
/// ```rust,ignore
/// // Colored ring
/// let ring = ParticleBuilder::new(Particle::dust_hex(0x00AAFF, 1.0))
///     .circle(2.5, 0.0, 48);
///
/// // Reuse the same builder for multiple shapes
/// let b = ParticleBuilder::new(Particle::named("minecraft:soul_fire_flame"))
///     .speed(0.02)
///     .force(true);
/// let mut cmds = b.helix(1.0, 4.0, 3.0, 64);
/// cmds.extend(b.disc(1.0, 0.0, 6));
/// ```
#[derive(Debug, Clone)]
pub struct ParticleBuilder {
    particle: Particle,
    spread: ParticleSpread,
    /// Speed of each particle after spawning. 0 = stationary.
    speed: f64,
    /// Number of particles Minecraft spawns per command invocation.
    ///
    /// Note: for colored `dust` particles, keep this at `1` and `spread` at
    /// `POINT` to preserve exact color. Higher counts with spread will randomize hues.
    particles_per_point: u32,
    /// `force` makes particles visible at any distance; `normal` respects the
    /// client's particle setting.
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

    // ── Configuration ─────────────────────────────────────────────────────────

    /// Set the random spread box around each particle's spawn position.
    pub fn spread(mut self, spread: ParticleSpread) -> Self {
        self.spread = spread;
        self
    }

    /// Set the initial speed of each particle after spawning.
    ///
    /// `0.0` = stationary (recommended for colored dust).
    /// Higher values make particles drift in random directions.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Number of particles Minecraft spawns per command call.
    ///
    /// Defaults to `1`. Increase for denser effects per point.
    pub fn particles_per_point(mut self, n: u32) -> Self {
        self.particles_per_point = n;
        self
    }

    /// Whether to use `force` visibility mode (ignores client particle settings).
    ///
    /// `true` (default) = always visible; `false` = respects client settings.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    // ── Shape generators ──────────────────────────────────────────────────────

    /// A horizontal ring of particles at `y_offset` above the executor.
    ///
    /// ```rust,ignore
    /// ParticleBuilder::new(Particle::named("minecraft:flame"))
    ///     .circle(2.0, 0.0, 32);
    /// ```
    pub fn circle(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let a = TAU * i as f64 / points as f64;
                self.cmd(radius * a.cos(), y_offset, radius * a.sin())
            })
            .collect()
    }

    /// A partial arc of a circle, from `start_deg` to `end_deg` (degrees).
    ///
    /// ```rust,ignore
    /// // Top half of a circle
    /// builder.arc(2.0, 0.0, 0.0, 180.0, 24);
    /// ```
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

    /// A regular polygon (triangle, square, hexagon, …) at `y_offset`.
    ///
    /// `points_per_side` particles are placed along each edge.
    ///
    /// ```rust,ignore
    /// // Hexagon with 8 particles per side
    /// builder.polygon(6, 2.0, 0.0, 8);
    /// ```
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

    /// A star shape with `arms` points, alternating between outer and inner radii.
    ///
    /// ```rust,ignore
    /// // 5-pointed star
    /// builder.star(5, 2.0, 0.8, 0.0);
    /// ```
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
                let r = if i % 2 == 0 { outer_radius } else { inner_radius };
                self.cmd(r * a.cos(), y_offset, r * a.sin())
            })
            .collect()
    }

    /// A sphere surface using the Fibonacci lattice for even distribution.
    ///
    /// ```rust,ignore
    /// builder.sphere(3.0, 0.0, 200);
    /// ```
    pub fn sphere(&self, radius: f64, y_offset: f64, points: usize) -> Vec<String> {
        let gr = (1.0 + 5.0_f64.sqrt()) / 2.0;
        (0..points)
            .map(|i| {
                let theta = TAU * i as f64 / gr;
                let phi =
                    ((1.0 - 2.0 * (i as f64 + 0.5) / points as f64).clamp(-1.0, 1.0)).acos();
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
                // Keep particles whose y component is non-negative relative to y_offset.
                // Extract the ~y token — it's the third `~...` field in the command.
                let y_val = extract_relative_y(cmd);
                y_val >= y_offset - 1e-9
            })
            .take(points)
            .collect()
    }

    /// A rising spiral helix.
    ///
    /// ```rust,ignore
    /// builder.helix(1.5, 4.0, 2.0, 64);
    /// ```
    pub fn helix(&self, radius: f64, height: f64, turns: f64, points: usize) -> Vec<String> {
        (0..points)
            .map(|i| {
                let t = i as f64 / (points as f64 - 1.0).max(1.0);
                let a = TAU * turns * t;
                self.cmd(radius * a.cos(), height * t, radius * a.sin())
            })
            .collect()
    }

    /// Two interleaved helices rising upward (double-helix / DNA shape).
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
    ///
    /// ```rust,ignore
    /// builder.line([0.0, 0.0, 0.0], [3.0, 2.0, 1.0], 20);
    /// ```
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

    /// A filled disc of concentric rings.
    ///
    /// `density` controls ring count per unit radius (higher = denser).
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
    ///
    /// `major_radius` = distance from the torus center to the tube center.
    /// `minor_radius` = radius of the tube itself.
    ///
    /// ```rust,ignore
    /// builder.torus(2.0, 0.5, 1.0, 16, 12);
    /// ```
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
    ///
    /// `rings` controls the number of horizontal cross-sections.
    ///
    /// ```rust,ignore
    /// builder.cone(2.0, 3.0, 0.0, 8);
    /// ```
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
    ///
    /// ```rust,ignore
    /// builder.burst(2.5, 1.0, 64);
    /// ```
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
    ///
    /// `length` = total length, `cycles` = number of full wave cycles.
    ///
    /// ```rust,ignore
    /// builder.wave(6.0, 1.0, 2.0, 0.5, 48);
    /// ```
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
    ///
    /// Centered on the executor. `cols` × `rows` points spanning `width` × `depth` blocks.
    ///
    /// ```rust,ignore
    /// builder.grid(4.0, 4.0, 9, 9, 0.0);
    /// ```
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
    ///
    /// ```rust,ignore
    /// builder.points_at(&[
    ///     [0.0, 0.0, 0.0],
    ///     [1.0, 0.5, 0.0],
    ///     [2.0, 1.0, 0.0],
    /// ]);
    /// ```
    pub fn points_at(&self, pts: &[[f64; 3]]) -> Vec<String> {
        pts.iter().map(|[x, y, z]| self.cmd(*x, *y, *z)).collect()
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn cmd(&self, x: f64, y: f64, z: f64) -> String {
        let mode = if self.force { "force" } else { "normal" };
        format!(
            "particle {} ~{} ~{} ~{} {} {} {} {} {} {mode}",
            self.particle.command_token(),
            fmt_f(x),
            fmt_f(y),
            fmt_f(z),
            fmt_f(self.spread.dx),
            fmt_f(self.spread.dy),
            fmt_f(self.spread.dz),
            fmt_f(self.speed),
            self.particles_per_point,
        )
    }
}

// ── ParticleEffect (backwards-compatible static helpers) ──────────────────────

/// Static particle geometry generators.
///
/// These are thin wrappers around [`ParticleBuilder`] kept for backwards
/// compatibility. New code should use [`ParticleBuilder`] directly for full
/// control over colors, speed, and particle types.
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

/// Format a f64 for position/spread/speed in a particle command.
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

/// Format a f32 color component for dust particles (2 decimal places).
fn fmt_c(v: f32) -> String {
    let rounded = (v * 100.0).round() / 100.0;
    format!("{:.2}", rounded)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// Extract the relative Y offset from a particle command string for hemisphere filtering.
fn extract_relative_y(cmd: &str) -> f64 {
    // Particle command: "particle <name+params> ~x ~y ~z ..."
    // We need the 3rd `~` prefixed token.
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
        assert!(cmds[0].contains("~2 ~0 ~0"), "first point at radius: {}", cmds[0]);
    }

    #[test]
    fn arc_partial() {
        let cmds = builder("minecraft:flame").arc(2.0, 0.0, 0.0, 90.0, 4);
        assert_eq!(cmds.len(), 4);
        // First point should be at angle 0 → (radius, 0)
        assert!(cmds[0].contains("~2 ~0 ~0"), "arc start: {}", cmds[0]);
        // Last point at 90° → (0, 0, radius)
        assert!(cmds[3].contains("~0 ~0 ~2"), "arc end: {}", cmds[3]);
    }

    #[test]
    fn polygon_sides() {
        // Square: 4 sides × 4 pts each = 16 commands
        let cmds = builder("minecraft:crit").polygon(4, 2.0, 0.0, 4);
        assert_eq!(cmds.len(), 16);
    }

    #[test]
    fn star_arms() {
        // 5-pointed star = 10 vertices
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
        // All apex particles should be near (0, 3, 0) → "~0 ~3 ~0"
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
        let cmds = ParticleBuilder::new(Particle::dust(1.0, 0.0, 0.0, 1.0))
            .circle(1.0, 0.0, 4);
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
}
