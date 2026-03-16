/// Builder for the `playsound` command.
///
/// # Example
/// ```rust,ignore
/// let cmd = Sound::play("minecraft:entity.experience_orb.pickup")
///     .to(Selector::self_())
///     .source(SoundSource::Player)
///     .volume(1.0)
///     .pitch(1.2)
///     .build();
/// // → "playsound minecraft:entity.experience_orb.pickup player @s ~ ~ ~ 1 1.2"
///
/// let cmd = Sound::stop_all(Selector::all_players());
/// // → "stopsound @a"
/// ```
use std::fmt::Display;

use super::coord::Vec3;
use super::selector::Selector;

// ── SoundSource ───────────────────────────────────────────────────────────────

/// Minecraft audio channel/category for sound playback.
///
/// Determines which volume slider in settings affects the sound and enables selective audio control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    /// Master volume channel — controls overall volume.
    Master,
    /// Music volume — background music and melodies.
    Music,
    /// Record volume — music discs and note blocks.
    Record,
    /// Weather volume — rain and thunderstorms.
    Weather,
    /// Block volume — mining, placing, and block interactions.
    Block,
    /// Hostile volume — hostile mob sounds (zombies, creepers, etc.).
    Hostile,
    /// Neutral volume — passive mob sounds (cows, sheep, etc.).
    Neutral,
    /// Player volume — player sounds (footsteps, damage, death, etc.).
    Player,
    /// Ambient volume — cave sounds and ambient effects.
    Ambient,
    /// Voice volume — voice chat and spoken text.
    Voice,
}

impl Display for SoundSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SoundSource::Master => "master",
            SoundSource::Music => "music",
            SoundSource::Record => "record",
            SoundSource::Weather => "weather",
            SoundSource::Block => "block",
            SoundSource::Hostile => "hostile",
            SoundSource::Neutral => "neutral",
            SoundSource::Player => "player",
            SoundSource::Ambient => "ambient",
            SoundSource::Voice => "voice",
        };
        f.write_str(s)
    }
}

// ── Sound ─────────────────────────────────────────────────────────────────────

/// Builder for `playsound` commands.
///
/// Chain methods to configure the sound, then call `build()` to generate the command string.
pub struct Sound {
    event: String,
    source: SoundSource,
    target: Option<Selector>,
    pos: Option<Vec3>,
    volume: f64,
    pitch: f64,
    min_volume: Option<f64>,
}

impl Sound {
    /// Begin building a `playsound` command for the given sound event ID.
    ///
    /// Sound event ID example: `"minecraft:entity.experience_orb.pickup"`.
    /// Chain builder methods to customize, then call `build()`.
    pub fn play(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            source: SoundSource::Master,
            target: None,
            pos: None,
            volume: 1.0,
            pitch: 1.0,
            min_volume: None,
        }
    }

    /// Set the target entity/player who hears the sound.
    ///
    /// If not set, defaults to `@s`. The sound is positioned relative to the target.
    pub fn to(mut self, selector: Selector) -> Self {
        self.target = Some(selector);
        self
    }

    /// Set the sound source/channel category (default: `master`).
    ///
    /// Determines which volume slider in settings affects this sound.
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set the position in the world where the sound originates (default: `~ ~ ~`).
    ///
    /// Affects the direction and distance the sound appears to come from for the listener.
    pub fn at(mut self, pos: Vec3) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Set the volume multiplier (default: `1.0`).
    ///
    /// `0.5` is half volume, `2.0` is double volume. Higher values can exceed normal limits.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier (default: `1.0`).
    ///
    /// `0.5` is half pitch (lower), `2.0` is double pitch (higher).
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    ///
    /// Ensures the sound stays audible even at distance. Useful for important sounds.
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    /// Build and return the `playsound` command string.
    ///
    /// Defaults: target=`@s`, position=`~ ~ ~`.
    /// Produces: `playsound <event> <source> <target> <pos> <volume> <pitch> [min_volume]`
    pub fn build(self) -> String {
        let target = self.target.unwrap_or_else(Selector::self_);
        let pos = self.pos.unwrap_or_else(Vec3::here);

        let mut s = format!(
            "playsound {} {} {} {} {} {}",
            self.event, self.source, target, pos, self.volume, self.pitch
        );

        if let Some(mv) = self.min_volume {
            s.push(' ');
            s.push_str(&format_float(mv));
        }

        s
    }

    // ── stopsound helpers ─────────────────────────────────────────────────────

    /// `stopsound <selector>` — stop all sounds playing for the target.
    pub fn stop_all(target: impl Display) -> String {
        format!("stopsound {}", target)
    }

    /// `stopsound <selector> <source>` — stop all sounds in a specific category for the target.
    ///
    /// Useful for silencing all music or all ambient sounds without affecting others.
    pub fn stop_source(target: impl Display, source: SoundSource) -> String {
        format!("stopsound {} {}", target, source)
    }

    /// `stopsound <selector> <source> <event>` — stop a specific sound for the target.
    ///
    /// The most precise stopsound — affects only this one sound event.
    pub fn stop(target: impl Display, source: SoundSource, event: impl Display) -> String {
        format!("stopsound {} {} {}", target, source, event)
    }
}

fn format_float(v: f64) -> String {
    if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_playsound() {
        let cmd = Sound::play("minecraft:entity.experience_orb.pickup")
            .to(Selector::self_())
            .source(SoundSource::Player)
            .build();
        assert_eq!(
            cmd,
            "playsound minecraft:entity.experience_orb.pickup player @s ~ ~ ~ 1 1"
        );
    }

    #[test]
    fn custom_volume_pitch() {
        let cmd = Sound::play("minecraft:block.note_block.bell")
            .to(Selector::all_players())
            .volume(2.0)
            .pitch(0.5)
            .build();
        assert!(cmd.contains("2 0.5"), "{}", cmd);
    }

    #[test]
    fn min_volume() {
        let cmd = Sound::play("minecraft:ambient.cave")
            .to(Selector::self_())
            .min_volume(0.3)
            .build();
        assert!(cmd.ends_with("0.3"), "{}", cmd);
    }

    #[test]
    fn stopsound() {
        assert_eq!(Sound::stop_all(Selector::all_players()), "stopsound @a");
        assert_eq!(
            Sound::stop_source(Selector::all_players(), SoundSource::Music),
            "stopsound @a music"
        );
        assert_eq!(
            Sound::stop(
                Selector::all_players(),
                SoundSource::Block,
                "minecraft:block.stone.hit"
            ),
            "stopsound @a block minecraft:block.stone.hit"
        );
    }
}
