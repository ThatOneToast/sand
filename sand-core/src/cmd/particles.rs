/// Compile-time particle geometry — generates multiple `particle` commands
/// from geometric shapes defined in Rust code.
///
/// All positions are *relative* (`~x ~y ~z`) to the executor so the effects
/// can be run from `execute as @a at @s ...` without hard-coding coordinates.
///
/// # Example
/// ```rust,ignore
/// let commands = ParticleEffect::circle("minecraft:flame", 2.0, 16);
/// // → ["particle minecraft:flame ~2 ~ ~ 0 0 0 0 1 force", ...]
/// ```

// ── ParticleSpread ────────────────────────────────────────────────────────────

/// Spread/dispersion parameters for particles.
///
/// Particles are randomly scattered within a bounding box with these half-extents.
/// `(dx, dy, dz)` — dispersion in each axis direction from the particle origin.
pub struct ParticleSpread {
    /// Half-extent in the X axis.
    pub dx: f64,
    /// Half-extent in the Y axis.
    pub dy: f64,
    /// Half-extent in the Z axis.
    pub dz: f64,
}

impl ParticleSpread {
    /// A point particle with no spread (all zeros).
    ///
    /// Particles appear exactly at the specified position.
    pub const POINT: Self = Self {
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
    };

    /// Create a uniform cube of spread in all three directions.
    ///
    /// All axes get the same spread value.
    pub fn uniform(v: f64) -> Self {
        Self {
            dx: v,
            dy: v,
            dz: v,
        }
    }

    /// Create custom spread parameters.
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self { dx, dy, dz }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Format a float for use in a particle command.
/// Truncates to 4 decimal places to keep commands readable.
fn fmt_f(v: f64) -> String {
    let rounded = (v * 10000.0).round() / 10000.0;
    if rounded == rounded.trunc() {
        format!("{}", rounded as i64)
    } else {
        // Strip trailing zeros
        let s = format!("{:.4}", rounded);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn particle_cmd(name: &str, x: f64, y: f64, z: f64, spread: &ParticleSpread) -> String {
    format!(
        "particle {} ~{} ~{} ~{} {} {} {} 0 1 force",
        name,
        fmt_f(x),
        fmt_f(y),
        fmt_f(z),
        fmt_f(spread.dx),
        fmt_f(spread.dy),
        fmt_f(spread.dz),
    )
}

// ── ParticleEffect ────────────────────────────────────────────────────────────

/// Static geometry generators for particle effects.
///
/// Produces `Vec<String>` of `particle` commands that form geometric shapes.
/// All coordinates are relative (`~x ~y ~z`) to the executor's position,
/// so effects work in any location without hard-coded coordinates.
pub struct ParticleEffect;

impl ParticleEffect {
    /// Generate a horizontal ring of particles.
    ///
    /// Creates `count` evenly-spaced particles at the given radius.
    /// Y-offset: positive is up, negative is down from executor.
    ///
    /// # Example
    /// ```rust,ignore
    /// // 32 flame particles in a horizontal ring of radius 2
    /// let cmds = ParticleEffect::circle("minecraft:flame", 2.0, 0.0, 32, &ParticleSpread::POINT);
    /// ```
    pub fn circle(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        (0..count)
            .map(|i| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (count as f64);
                let x = radius * angle.cos();
                let z = radius * angle.sin();
                particle_cmd(particle, x, y_offset, z, spread)
            })
            .collect()
    }

    /// Generate a sphere of particles distributed uniformly.
    ///
    /// Uses the Fibonacci sphere algorithm for even distribution across the surface and interior.
    /// Good for spherical effects like magic orbs or auras.
    pub fn sphere(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        (0..count)
            .map(|i| {
                let theta = 2.0 * std::f64::consts::PI * (i as f64) / golden_ratio;
                let phi = ((1.0 - 2.0 * (i as f64 + 0.5) / count as f64).clamp(-1.0, 1.0)).acos();
                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos() + y_offset;
                let z = radius * phi.sin() * theta.sin();
                particle_cmd(particle, x, y, z, spread)
            })
            .collect()
    }

    /// Generate a spiral helix rising upward.
    ///
    /// Creates a DNA-like spiral that rises `height` blocks while rotating `turns` times.
    /// Good for staff effects, tornadoes, or DNA visualizations.
    pub fn helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        (0..count)
            .map(|i| {
                let t = i as f64 / (count as f64 - 1.0).max(1.0);
                let angle = 2.0 * std::f64::consts::PI * turns * t;
                let x = radius * angle.cos();
                let y = height * t;
                let z = radius * angle.sin();
                particle_cmd(particle, x, y, z, spread)
            })
            .collect()
    }

    /// Generate a straight line of particles between two points.
    ///
    /// Both endpoints are relative to the executor. Good for laser beams or lightning paths.
    /// Handles edge cases: empty list if count=0, single point if count=1.
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
        if count == 0 {
            return vec![];
        }
        if count == 1 {
            return vec![particle_cmd(particle, x1, y1, z1, spread)];
        }
        (0..count)
            .map(|i| {
                let t = i as f64 / (count - 1) as f64;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                let z = z1 + (z2 - z1) * t;
                particle_cmd(particle, x, y, z, spread)
            })
            .collect()
    }

    /// Generate an outward burst of particles radiating from a center.
    ///
    /// Particles spread across a sphere with extra velocity for expansion.
    /// Good for explosions, impact effects, or muzzle flashes.
    pub fn burst(
        particle: &str,
        radius: f64,
        y_offset: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        // Uses the same fibonacci sphere but with added spread for a burst look
        let spread_boost = ParticleSpread::new(
            spread.dx + radius * 0.15,
            spread.dy + radius * 0.15,
            spread.dz + radius * 0.15,
        );
        Self::sphere(particle, radius, y_offset, count, &spread_boost)
    }

    /// Generate two interleaved helices (DNA-style) rising upward.
    ///
    /// Creates a double helix with complementary particles offset by 180 degrees.
    /// Rising `height` blocks over `turns` rotations.
    pub fn double_helix(
        particle: &str,
        radius: f64,
        height: f64,
        turns: f64,
        count: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        let mut cmds = Self::helix(particle, radius, height, turns, count / 2, spread);
        // Second strand is offset by π
        let half = count - count / 2;
        let extra: Vec<String> = (0..half)
            .map(|i| {
                let t = i as f64 / (half as f64 - 1.0).max(1.0);
                let angle = 2.0 * std::f64::consts::PI * turns * t + std::f64::consts::PI;
                let x = radius * angle.cos();
                let y = height * t;
                let z = radius * angle.sin();
                particle_cmd(particle, x, y, z, spread)
            })
            .collect();
        cmds.extend(extra);
        cmds
    }

    /// Generate a filled disc (solid circle with concentric rings).
    ///
    /// Creates a center point plus concentric rings to fill the disk.
    /// Density controls the number of rings; higher values = more particles, better coverage.
    /// Good for beam effects, platform highlights, or dimensional portals.
    pub fn disc(
        particle: &str,
        radius: f64,
        y_offset: f64,
        density: usize,
        spread: &ParticleSpread,
    ) -> Vec<String> {
        let rings = (radius * density as f64).ceil() as usize;
        let mut cmds = Vec::new();
        // Centre point
        cmds.push(particle_cmd(particle, 0.0, y_offset, 0.0, spread));
        for ring in 1..=rings {
            let r = radius * ring as f64 / rings as f64;
            let pts = ((2.0 * std::f64::consts::PI * r * density as f64).ceil() as usize).max(4);
            cmds.extend(Self::circle(particle, r, y_offset, pts, spread));
        }
        cmds
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_count() {
        let cmds = ParticleEffect::circle("minecraft:flame", 2.0, 0.0, 8, &ParticleSpread::POINT);
        assert_eq!(cmds.len(), 8);
        assert!(cmds[0].starts_with("particle minecraft:flame"));
        // First point is at angle 0 → x=radius, z=0
        assert!(cmds[0].contains("~2"), "expected radius on x: {}", cmds[0]);
    }

    #[test]
    fn sphere_count() {
        let cmds =
            ParticleEffect::sphere("minecraft:end_rod", 3.0, 0.0, 20, &ParticleSpread::POINT);
        assert_eq!(cmds.len(), 20);
    }

    #[test]
    fn helix_count() {
        let cmds = ParticleEffect::helix(
            "minecraft:soul_fire_flame",
            1.0,
            5.0,
            2.0,
            30,
            &ParticleSpread::POINT,
        );
        assert_eq!(cmds.len(), 30);
    }

    #[test]
    fn line_endpoints() {
        let cmds = ParticleEffect::line(
            "minecraft:crit",
            0.0,
            0.0,
            0.0,
            3.0,
            0.0,
            0.0,
            4,
            &ParticleSpread::POINT,
        );
        assert_eq!(cmds.len(), 4);
        assert!(cmds[0].contains("~0 ~0 ~0"), "start: {}", cmds[0]);
        assert!(cmds[3].contains("~3 ~0 ~0"), "end: {}", cmds[3]);
    }

    #[test]
    fn burst_has_spread() {
        let cmds =
            ParticleEffect::burst("minecraft:explosion", 2.0, 1.0, 10, &ParticleSpread::POINT);
        assert_eq!(cmds.len(), 10);
    }

    #[test]
    fn fmt_f_precision() {
        assert_eq!(fmt_f(0.0), "0");
        assert_eq!(fmt_f(1.0), "1");
        assert_eq!(fmt_f(1.5), "1.5");
        assert_eq!(fmt_f(1.2345678), "1.2346");
    }
}
