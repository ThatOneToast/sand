//! Reusable typed visual/audio effects.
//!
//! A [`Vfx`] groups particle, sound, and raw command steps so datapack authors
//! can define an effect once and emit deterministic command lists wherever it
//! is needed.
//!
//! ```rust,ignore
//! use sand_core::prelude::*;
//!
//! fn level_up_vfx() -> Vfx {
//!     Vfx::new("level_up")
//!         .particle(
//!             VfxParticle::named("minecraft:happy_villager")
//!                 .count(20)
//!                 .spread(0.6, 1.0, 0.6),
//!         )
//!         .sound(
//!             VfxSound::new("minecraft:entity.player.levelup")
//!                 .source(SoundSource::Player)
//!                 .volume(1.0)
//!                 .pitch(1.2),
//!         )
//! }
//!
//! #[function]
//! pub fn level_up() {
//!     for cmd in level_up_vfx().play_at(Selector::self_()) {
//!         cmd;
//!     }
//! }
//! ```

use crate::cmd::{
    Build, EntityTargets, Execute, Particle, ParticleBuilder, ParticleSpread, PlayerTargets,
    Selector, SingleEntity, SinglePlayer, Sound, SoundSource, Vec3,
};

/// A reusable group of visual/audio commands.
#[derive(Debug, Clone)]
pub struct Vfx {
    name: String,
    steps: Vec<VfxStep>,
}

impl Vfx {
    /// Create a new named VFX asset.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    /// Stable author-facing name for this effect.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a particle step.
    pub fn particle(mut self, particle: impl IntoParticleStep) -> Self {
        self.steps
            .push(VfxStep::Particle(particle.into_particle_step()));
        self
    }

    /// Add a sound step.
    pub fn sound(mut self, sound: impl IntoSoundStep) -> Self {
        self.steps.push(VfxStep::Sound(sound.into_sound_step()));
        self
    }

    /// Add a raw command step.
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.steps.push(VfxStep::Command(command.into()));
        self
    }

    /// Number of steps in the effect.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether this effect has no steps.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Render commands at the current execution position.
    pub fn play(&self) -> Vec<String> {
        self.steps.iter().flat_map(VfxStep::render).collect()
    }

    /// Render commands at an entity/player selector.
    ///
    /// Particle and raw command steps are wrapped with `execute at <target>`.
    /// Sound steps are also targeted to the same selector.
    pub fn play_at(&self, target: impl IntoVfxSelector) -> Vec<String> {
        let target = target.into_vfx_selector();
        self.steps
            .iter()
            .flat_map(|step| step.render_at(&target))
            .collect()
    }

    /// Render commands for a player audience at the current execution position.
    ///
    /// Particle and raw command steps are emitted unchanged. Sound steps target
    /// the supplied audience.
    pub fn play_for(&self, audience: impl IntoVfxSelector) -> Vec<String> {
        let audience = audience.into_vfx_selector();
        self.steps
            .iter()
            .flat_map(|step| step.render_for(&audience))
            .collect()
    }

    /// Render commands at a specific execution position.
    pub fn play_positioned(&self, position: Vec3) -> Vec<String> {
        self.steps
            .iter()
            .flat_map(|step| step.render_positioned(&position))
            .collect()
    }
}

/// A single step in a [`Vfx`] asset.
#[derive(Debug, Clone)]
pub enum VfxStep {
    /// Particle commands rendered via [`ParticleBuilder`].
    Particle(VfxParticle),
    /// Sound command rendered via [`Sound`].
    Sound(VfxSound),
    /// Raw command string emitted in sequence.
    Command(String),
}

impl VfxStep {
    fn render(&self) -> Vec<String> {
        match self {
            Self::Particle(step) => step.render(),
            Self::Sound(step) => vec![step.render(None, None)],
            Self::Command(command) => vec![command.clone()],
        }
    }

    fn render_at(&self, target: &Selector) -> Vec<String> {
        match self {
            Self::Particle(step) => step
                .render()
                .into_iter()
                .map(|cmd| Execute::new().at(target.clone()).run(cmd))
                .collect(),
            Self::Sound(step) => {
                // Sound audience must NOT be the positional `target`.
                // Using `target` as the audience inside `execute at <target>`
                // would fork the sound once per matched entity and replay it
                // to the entire audience on every fork — e.g.
                //   execute at @a run playsound … @a
                // Instead, use the sound's own configured audience or `@s`
                // (the entity currently executing), which is always safe.
                let cmd = step.render_with_own_audience(Some(Vec3::here()));
                vec![Execute::new().at(target.clone()).run(cmd)]
            }
            Self::Command(command) => vec![Execute::new().at(target.clone()).run(command)],
        }
    }

    fn render_for(&self, audience: &Selector) -> Vec<String> {
        match self {
            Self::Particle(step) => step.render(),
            Self::Sound(step) => vec![step.render(Some(audience), None)],
            Self::Command(command) => vec![command.clone()],
        }
    }

    fn render_positioned(&self, position: &Vec3) -> Vec<String> {
        match self {
            Self::Particle(step) => step
                .render()
                .into_iter()
                .map(|cmd| Execute::new().positioned(position.clone()).run(cmd))
                .collect(),
            Self::Sound(step) => vec![step.render(None, Some(position.clone()))],
            Self::Command(command) => {
                vec![Execute::new().positioned(position.clone()).run(command)]
            }
        }
    }
}

/// A reusable particle step rendered by [`ParticleBuilder`].
#[derive(Debug, Clone)]
pub struct VfxParticle {
    particle: Particle,
    spread: ParticleSpread,
    speed: f64,
    count: u32,
    force: bool,
    points: Vec<[f64; 3]>,
}

impl VfxParticle {
    /// Create a particle step for a concrete particle value.
    pub fn new(particle: Particle) -> Self {
        Self {
            particle,
            spread: ParticleSpread::POINT,
            speed: 0.0,
            count: 1,
            force: true,
            points: vec![[0.0, 0.0, 0.0]],
        }
    }

    /// Create a named particle step.
    pub fn named(name: impl Into<String>) -> Self {
        Self::new(Particle::named(name))
    }

    /// Convenience constructor for `minecraft:happy_villager`.
    pub fn happy_villager() -> Self {
        Self::named("minecraft:happy_villager")
    }

    /// Set the random spread box around each point.
    pub fn spread(mut self, dx: f64, dy: f64, dz: f64) -> Self {
        self.spread = ParticleSpread::new(dx, dy, dz);
        self
    }

    /// Set a uniform random spread around each point.
    pub fn spread_uniform(mut self, value: f64) -> Self {
        self.spread = ParticleSpread::uniform(value);
        self
    }

    /// Set initial particle speed.
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Set the number of particles spawned per point.
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// Set particle visibility mode.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Spawn at one relative offset.
    pub fn offset(mut self, x: f64, y: f64, z: f64) -> Self {
        self.points = vec![[x, y, z]];
        self
    }

    /// Spawn at multiple relative offsets in deterministic order.
    pub fn offsets(mut self, points: impl IntoIterator<Item = [f64; 3]>) -> Self {
        self.points = points.into_iter().collect();
        self
    }

    /// Render this particle step to command strings.
    pub fn render(&self) -> Vec<String> {
        ParticleBuilder::new(self.particle.clone())
            .spread(self.spread.clone())
            .speed(self.speed)
            .particles_per_point(self.count)
            .force(self.force)
            .points_at(&self.points)
    }
}

impl From<Particle> for VfxParticle {
    fn from(particle: Particle) -> Self {
        Self::new(particle)
    }
}

/// Convert a value into a particle step.
pub trait IntoParticleStep {
    /// Convert into a [`VfxParticle`].
    fn into_particle_step(self) -> VfxParticle;
}

impl IntoParticleStep for VfxParticle {
    fn into_particle_step(self) -> VfxParticle {
        self
    }
}

impl IntoParticleStep for Particle {
    fn into_particle_step(self) -> VfxParticle {
        VfxParticle::new(self)
    }
}

/// A reusable sound step rendered by [`Sound`].
#[derive(Debug, Clone)]
pub struct VfxSound {
    event: String,
    source: SoundSource,
    audience: Option<Selector>,
    position: Option<Vec3>,
    volume: f64,
    pitch: f64,
    min_volume: Option<f64>,
}

impl VfxSound {
    /// Begin building a reusable `playsound` step.
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            source: SoundSource::Master,
            audience: None,
            position: None,
            volume: 1.0,
            pitch: 1.0,
            min_volume: None,
        }
    }

    /// Set a default audience for this sound step.
    pub fn to(mut self, audience: impl IntoVfxSelector) -> Self {
        self.audience = Some(audience.into_vfx_selector());
        self
    }

    /// Set the sound source/channel category.
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set a default sound origin.
    pub fn at(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the volume multiplier.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier.
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    fn render(&self, audience: Option<&Selector>, position: Option<Vec3>) -> String {
        let mut sound = Sound::play(self.event.clone())
            .source(self.source)
            .volume(self.volume)
            .pitch(self.pitch);

        if let Some(audience) = audience.cloned().or_else(|| self.audience.clone()) {
            sound = sound.to(audience);
        }

        if let Some(position) = position.or_else(|| self.position.clone()) {
            sound = sound.at(position);
        }

        if let Some(min_volume) = self.min_volume {
            sound = sound.min_volume(min_volume);
        }

        sound.build()
    }

    /// Render using the sound's own configured audience (falling back to `@s`),
    /// never the positional selector passed to `play_at`.
    ///
    /// This prevents `play_at(@a)` from producing
    /// `execute at @a run playsound ... @a`, which would multiply the sound
    /// once per entity fork.
    fn render_with_own_audience(&self, position: Option<Vec3>) -> String {
        let audience = self.audience.clone().unwrap_or_else(Selector::self_);
        self.render(Some(&audience), position)
    }
}

/// Convert a value into a sound step.
pub trait IntoSoundStep {
    /// Convert into a [`VfxSound`].
    fn into_sound_step(self) -> VfxSound;
}

impl IntoSoundStep for VfxSound {
    fn into_sound_step(self) -> VfxSound {
        self
    }
}

/// Convert a target-like value into a [`Selector`] for VFX playback helpers.
pub trait IntoVfxSelector {
    /// Convert into a selector.
    fn into_vfx_selector(self) -> Selector;
}

impl IntoVfxSelector for Selector {
    fn into_vfx_selector(self) -> Selector {
        self
    }
}

impl IntoVfxSelector for &Selector {
    fn into_vfx_selector(self) -> Selector {
        self.clone()
    }
}

impl IntoVfxSelector for SingleEntity {
    fn into_vfx_selector(self) -> Selector {
        self.into_selector()
    }
}

impl IntoVfxSelector for EntityTargets {
    fn into_vfx_selector(self) -> Selector {
        self.into_selector()
    }
}

impl IntoVfxSelector for SinglePlayer {
    fn into_vfx_selector(self) -> Selector {
        self.into_selector()
    }
}

impl IntoVfxSelector for PlayerTargets {
    fn into_vfx_selector(self) -> Selector {
        self.into_selector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vfx_produces_no_commands() {
        let vfx = Vfx::new("empty");
        assert!(vfx.is_empty());
        assert_eq!(vfx.play(), Vec::<String>::new());
    }

    #[test]
    fn single_particle_uses_particle_builder_output() {
        let commands = Vfx::new("spark")
            .particle(
                VfxParticle::named("minecraft:happy_villager")
                    .count(20)
                    .spread(0.6, 1.0, 0.6),
            )
            .play();

        assert_eq!(
            commands,
            vec!["particle minecraft:happy_villager ~0 ~0 ~0 0.6 1 0.6 0 20 force"]
        );
    }

    #[test]
    fn single_sound_uses_sound_builder_output() {
        let commands = Vfx::new("ding")
            .sound(
                VfxSound::new("minecraft:entity.player.levelup")
                    .source(SoundSource::Player)
                    .volume(1.0)
                    .pitch(1.2),
            )
            .play_for(Selector::self_());

        assert_eq!(
            commands,
            vec!["playsound minecraft:entity.player.levelup player @s ~ ~ ~ 1 1.2"]
        );
    }

    #[test]
    fn combined_steps_preserve_order() {
        let commands = Vfx::new("combo")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .command("say done")
            .play_for(Selector::all_players());

        assert_eq!(
            commands,
            vec![
                "particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "playsound minecraft:block.note_block.bell master @a ~ ~ ~ 1 1",
                "say done",
            ]
        );
    }

    #[test]
    fn play_at_targets_expected_selector() {
        let commands = Vfx::new("self")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @s run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
            ]
        );
    }

    #[test]
    fn positioned_playback_wraps_positioned_commands() {
        let commands = Vfx::new("pos")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_positioned(Vec3::absolute(1.0, 2.0, 3.0));

        assert_eq!(
            commands,
            vec![
                "execute positioned 1 2 3 run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "playsound minecraft:block.note_block.bell master @s 1 2 3 1 1",
            ]
        );
    }

    #[test]
    fn command_output_is_deterministic() {
        let vfx = Vfx::new("deterministic")
            .particle(
                VfxParticle::named("minecraft:end_rod").offsets([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
            )
            .sound(VfxSound::new("minecraft:block.note_block.bell"));

        assert_eq!(
            vfx.play_at(Selector::self_()),
            vfx.play_at(Selector::self_())
        );
    }

    // -----------------------------------------------------------------
    // Regression tests for the multi-target sound-duplication bug.
    //
    // `play_at(target)` must NEVER reuse the positional selector as the
    // playsound audience.  With a multi-player selector such as `@a`,
    // Minecraft forks the execute chain once per matched entity; if the
    // playsound audience were also `@a`, every player would hear the
    // sound N times (once per fork).
    // -----------------------------------------------------------------

    #[test]
    fn play_at_all_players_does_not_reuse_selector_as_sound_audience() {
        let commands = Vfx::new("level_up")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_at(Selector::all_players());

        // The playsound audience (third argument to playsound) must NOT be @a.
        // playsound syntax: playsound <sound> <source> <audience> [x y z ...]
        // We check that no command has "playsound" with @a as the audience token
        // (the third whitespace-delimited word after "playsound").
        assert!(
            !commands.iter().any(|cmd| {
                // Find the playsound substring and inspect its audience token.
                if let Some(ps_pos) = cmd.find("playsound ") {
                    let after_ps = &cmd[ps_pos + "playsound ".len()..];
                    // tokens: <sound> <source> <audience> ...
                    let mut tokens = after_ps.split_whitespace();
                    tokens.next(); // skip sound event
                    tokens.next(); // skip source
                    tokens.next() == Some("@a")
                } else {
                    false
                }
            }),
            "play_at must not reuse positional selector as sound audience: {commands:?}"
        );
    }

    #[test]
    fn play_at_all_players_sound_audience_is_self() {
        // When no explicit audience is configured on the VfxSound, play_at
        // must fall back to @s (the entity currently executing), not @a.
        let commands = Vfx::new("level_up")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_at(Selector::all_players());

        assert_eq!(
            commands,
            vec!["execute at @a run playsound minecraft:entity.player.levelup master @s ~ ~ ~ 1 1"]
        );
    }

    #[test]
    fn play_at_self_particle_behavior_unchanged() {
        // play_at("@s") must still emit the expected positional particle
        // command — the fix must not regress the common single-entity case.
        let commands = Vfx::new("spark")
            .particle(VfxParticle::named("minecraft:happy_villager"))
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec!["execute at @s run particle minecraft:happy_villager ~0 ~0 ~0 0 0 0 0 1 force"]
        );
    }

    #[test]
    fn sound_audience_independent_from_positional_selector() {
        // Particle uses the positional target; sound audience is separate.
        let commands = Vfx::new("effect")
            .particle(VfxParticle::named("minecraft:crit"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .play_at(Selector::all_players());

        assert_eq!(
            commands,
            vec![
                "execute at @a run particle minecraft:crit ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @a run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
            ]
        );
    }

    #[test]
    fn explicit_sound_audience_is_preserved_through_play_at() {
        // If VfxSound has an explicit audience set via `.to(...)`, that
        // audience must survive through `play_at`, ignoring the positional
        // selector entirely.
        let commands = Vfx::new("broadcast")
            .sound(
                VfxSound::new("minecraft:ui.toast.challenge_complete").to(Selector::all_players()),
            )
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run playsound minecraft:ui.toast.challenge_complete master @a ~ ~ ~ 1 1"
            ]
        );
    }

    #[test]
    fn play_at_combined_particle_and_sound_preserves_order() {
        // Combined effect: particle first, then sound, deterministic order.
        let commands = Vfx::new("combo_at")
            .particle(VfxParticle::named("minecraft:end_rod"))
            .sound(VfxSound::new("minecraft:block.note_block.bell"))
            .command("say vfx")
            .play_at(Selector::self_());

        assert_eq!(
            commands,
            vec![
                "execute at @s run particle minecraft:end_rod ~0 ~0 ~0 0 0 0 0 1 force",
                "execute at @s run playsound minecraft:block.note_block.bell master @s ~ ~ ~ 1 1",
                "execute at @s run say vfx",
            ]
        );
    }

    #[test]
    fn play_for_all_players_targets_expected_audience() {
        // play_for must still target the supplied audience — regression guard.
        let commands = Vfx::new("announce")
            .sound(VfxSound::new("minecraft:entity.player.levelup"))
            .play_for(Selector::all_players());

        assert_eq!(
            commands,
            vec!["playsound minecraft:entity.player.levelup master @a ~ ~ ~ 1 1"]
        );
    }
}
