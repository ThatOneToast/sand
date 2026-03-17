//! # Particle Effects
//!
//! Demonstrates the `ParticleBuilder` API for creating custom visual effects.
//! All positions are relative to the executing entity (`~x ~y ~z`), so effects
//! can be attached to players, mobs, or locations via `execute at @s ...`.

use sand_core::cmd::{Execute, Particle, ParticleBuilder, ParticleSpread, Selector, TextComponent};
use sand_core::mcfunction;
use sand_macros::{component, function, run_fn};

// ── Sphere around a location ─────────────────────────────────────────────────
// A glowing orb of end_rod particles surrounding the executing entity.
// Pair with: execute at @e[type=armor_stand,tag=orb] run function ns:orb_sphere

#[function]
pub fn orb_sphere() {
    let cmds = ParticleBuilder::new(Particle::named("minecraft:end_rod"))
        .speed(0.01)
        .sphere(1.5, 1.0, 120);

    mcfunction! {
        for cmd in &cmds { cmd; }
    }
}

// ── Colored aura around a player ─────────────────────────────────────────────
// Three concentric dust rings in different colors stacked around the player.
// Pair with: execute as @a at @s run function ns:player_aura

#[function]
pub fn player_aura() {
    let blue = ParticleBuilder::new(Particle::dust_hex(0x00AAFF, 1.2));
    let purple = ParticleBuilder::new(Particle::dust_hex(0xAA00FF, 1.0));
    let cyan = ParticleBuilder::new(Particle::dust_hex(0x00FFDD, 0.8));

    let ring_lo = blue.circle(1.2, 0.1, 32);
    let ring_mid = purple.circle(1.0, 1.0, 28);
    let ring_hi = cyan.circle(0.8, 1.9, 24);

    mcfunction! {
        for cmd in ring_lo.iter().chain(&ring_mid).chain(&ring_hi) { cmd; }
    }
}

// ── Color-transitioning tornado ───────────────────────────────────────────────
// A red-to-blue transitioning dust helix that rises like a whirlwind.

#[function]
pub fn tornado() {
    let cmds = ParticleBuilder::new(Particle::dust_transition_hex(0xFF2200, 0x2200FF, 1.5))
        .double_helix(1.8, 5.0, 3.0, 96);

    mcfunction! {
        for cmd in &cmds { cmd; }
    }
}

// ── Portal ring ───────────────────────────────────────────────────────────────
// A vertical disc of portal-coloured particles — place it at a custom
// location with `execute positioned X Y Z run function ns:portal_ring`.

#[function]
pub fn portal_ring() {
    // Outer glow ring
    let outer = ParticleBuilder::new(Particle::named("minecraft:portal"))
        .speed(0.05)
        .spread(ParticleSpread::uniform(0.05))
        .circle(2.5, 1.5, 64);

    // Inner solid disc
    let inner = ParticleBuilder::new(Particle::dust_hex(0x5500AA, 1.0)).disc(2.3, 1.5, 5);

    mcfunction! {
        for cmd in outer.iter().chain(&inner) { cmd; }
    }
}

// ── Impact burst ─────────────────────────────────────────────────────────────
// A one-shot explosion burst — crit particles expanding outward.
// Good for spell impacts, hammer slams, etc.

#[function]
pub fn impact_burst() {
    let sparks = ParticleBuilder::new(Particle::named("minecraft:crit"))
        .speed(0.3)
        .particles_per_point(2)
        .burst(1.5, 0.8, 48);

    let glow = ParticleBuilder::new(Particle::dust_hex(0xFFAA00, 1.5)).sphere(0.6, 0.6, 24);

    mcfunction! {
        for cmd in sparks.iter().chain(&glow) { cmd; }
    }
}

// ── Magic circle (summoning) ──────────────────────────────────────────────────
// A flat ritual circle: outer hexagon, inner star, and a central flash.
// Place at ground level with `execute positioned X Y Z run function ns:magic_circle`.

#[function]
pub fn magic_circle() {
    let gold = ParticleBuilder::new(Particle::dust_hex(0xFFCC00, 1.2));
    let white = ParticleBuilder::new(Particle::dust_hex(0xFFFFFF, 0.8));
    let orange = ParticleBuilder::new(Particle::named("minecraft:flame")).speed(0.0);

    let hexagon = gold.polygon(6, 2.5, 0.05, 12);
    let star = gold.star(6, 2.0, 0.9, 0.05);
    let inner = white.circle(0.8, 0.05, 20);
    let center = orange.disc(0.3, 0.05, 4);

    mcfunction! {
        for cmd in hexagon.iter().chain(&star).chain(&inner).chain(&center) { cmd; }
    }
}

// ── DNA helix ────────────────────────────────────────────────────────────────
// A classic double helix — complementary green/blue strands rising upward.

#[function]
pub fn dna_helix() {
    let strand_a = ParticleBuilder::new(Particle::dust_hex(0x00FF88, 1.0)).helix(0.8, 6.0, 3.0, 64);

    // Second strand manually offset by π (built into double_helix, but shown separately here)
    let strand_b = ParticleBuilder::new(Particle::dust_hex(0x0088FF, 1.0))
        .helix(0.8, 6.0, 3.0, 64)
        .into_iter()
        // Shift the x/z tokens is not practical here — use double_helix instead:
        .collect::<Vec<_>>();

    let both = ParticleBuilder::new(Particle::dust_transition_hex(0x00FF88, 0x0088FF, 1.0))
        .double_helix(0.8, 6.0, 3.0, 128);

    mcfunction! {
        for cmd in &both { cmd; }
    }
}

// ── Lightning bolt ────────────────────────────────────────────────────────────
// A jagged vertical line using multiple short segments at random-ish offsets.
// Simulates a lightning strike from above the player.

#[function]
pub fn lightning_bolt() {
    let b = ParticleBuilder::new(Particle::named("minecraft:electric_spark"))
        .speed(0.02)
        .particles_per_point(2);

    // Compose segments to create a jagged lightning effect
    let seg1 = b.line([0.0, 8.0, 0.0], [0.3, 6.0, 0.2], 4);
    let seg2 = b.line([0.3, 6.0, 0.2], [-0.2, 4.5, -0.1], 4);
    let seg3 = b.line([-0.2, 4.5, -0.1], [0.4, 3.0, 0.3], 4);
    let seg4 = b.line([0.4, 3.0, 0.3], [0.0, 1.5, 0.0], 4);
    let seg5 = b.line([0.0, 1.5, 0.0], [0.1, 0.0, 0.1], 3);

    // Glow at impact point
    let impact = ParticleBuilder::new(Particle::dust_hex(0xCCDDFF, 1.5)).burst(0.5, 0.1, 16);

    mcfunction! {
        for cmd in seg1.iter().chain(&seg2).chain(&seg3).chain(&seg4).chain(&seg5).chain(&impact) {
            cmd;
        }
    }
}

// ── Torus gate ────────────────────────────────────────────────────────────────
// A glowing 3D donut shape — use as a portal gate or magical ring.

#[function]
pub fn torus_gate() {
    // Outer torus shell (end_rod)
    let shell = ParticleBuilder::new(Particle::named("minecraft:end_rod"))
        .speed(0.0)
        .torus(2.0, 0.4, 1.5, 24, 16);

    // Color ring in the middle
    let ring = ParticleBuilder::new(Particle::dust_transition_hex(0xFF00FF, 0x00FFFF, 1.2))
        .circle(2.0, 1.5, 48);

    mcfunction! {
        for cmd in shell.iter().chain(&ring) { cmd; }
    }
}

// ── Tick-driven rotating ring ─────────────────────────────────────────────────
// This effect runs on the tick function. Uses a scoreboard to track rotation
// angle and spawns a single arc segment per tick to create a smooth spin.
// (Full implementation would require an angle counter in NBT/scoreboards.)
//
// Simplified version: re-spawn the whole ring every tick — fine for small rings.

#[component(Tick)]
pub fn rotating_ring_tick() {
    mcfunction! {
        Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
            .run(run_fn!({
                for cmd in &ParticleBuilder::new(Particle::dust_hex(0xFF4400, 1.0))
                    .speed(0.0)
                    .circle(1.5, 1.0, 24)
                {
                    cmd;
                }
            }));
    }
}
