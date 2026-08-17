//! Builder for the `playsound` command.
//!
//! # Example
//! ```rust,ignore
//! let cmd = Sound::play("minecraft:entity.experience_orb.pickup")
//!     .to(Selector::self_())
//!     .source(SoundSource::Player)
//!     .volume(1.0)
//!     .pitch(1.2)
//!     .build();
//! // → "playsound minecraft:entity.experience_orb.pickup player @s ~ ~ ~ 1 1.2"
//!
//! let cmd = Sound::stop_all(Selector::all_players());
//! // → "stopsound @a"
//! ```

use std::collections::BTreeMap;
use std::fmt;

use crate::Build;
use crate::coord::Vec3;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

// ── SoundSource ───────────────────────────────────────────────────────────────

/// Conversion into a sound-event resource-location token.
pub trait IntoSoundEvent {
    /// Converts a typed or validated value into a Minecraft sound-event identifier.
    fn into_sound_event(self) -> String;
}

impl IntoSoundEvent for String {
    fn into_sound_event(self) -> String {
        self
    }
}

impl IntoSoundEvent for &str {
    fn into_sound_event(self) -> String {
        self.to_string()
    }
}

/// Minecraft audio channel/category for sound playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ui,
    Ambient,
    Voice,
}

impl fmt::Display for SoundSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SoundSource::Master => "master",
            SoundSource::Music => "music",
            SoundSource::Record => "record",
            SoundSource::Weather => "weather",
            SoundSource::Block => "block",
            SoundSource::Hostile => "hostile",
            SoundSource::Neutral => "neutral",
            SoundSource::Player => "player",
            SoundSource::Ui => "ui",
            SoundSource::Ambient => "ambient",
            SoundSource::Voice => "voice",
        };
        f.write_str(s)
    }
}

// ── Sound ─────────────────────────────────────────────────────────────────────

/// Builder for `playsound` commands.
#[derive(Debug, Clone)]
pub struct Sound {
    event: String,
    raw_event: bool,
    source: SoundSource,
    target: Option<Selector>,
    pos: Option<Vec3>,
    volume: f64,
    pitch: f64,
    min_volume: Option<f64>,
}

impl Sound {
    /// Begin building a `playsound` command for the given sound event ID.
    pub fn play(event: impl IntoSoundEvent) -> Self {
        Self {
            event: event.into_sound_event(),
            raw_event: false,
            source: SoundSource::Master,
            target: None,
            pos: None,
            volume: 1.0,
            pitch: 1.0,
            min_volume: None,
        }
    }

    /// Begin building a sound command with an intentionally opaque event token.
    pub fn play_raw(event: impl IntoSoundEvent) -> Self {
        Self {
            raw_event: true,
            ..Self::play(event)
        }
    }

    /// Set the target entity/player who hears the sound (default: `@s`).
    pub fn to(mut self, selector: Selector) -> Self {
        self.target = Some(selector);
        self
    }

    /// Set the sound source/channel category (default: `master`).
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set the position in the world where the sound originates (default: `~ ~ ~`).
    pub fn at(mut self, pos: Vec3) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Set the volume multiplier (default: `1.0`).
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier (default: `1.0`).
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    // ── stopsound helpers ─────────────────────────────────────────────────────

    /// `stopsound <selector>` — stop all sounds playing for the target.
    pub fn stop_all(target: Selector) -> String {
        StopSoundCommand::All { target }.build_registered()
    }

    /// `stopsound <selector> <source>` — stop all sounds in a specific category.
    pub fn stop_source(target: Selector, source: SoundSource) -> String {
        StopSoundCommand::Source { target, source }.build_registered()
    }

    /// `stopsound <selector> <source> <event>` — stop a specific sound for the target.
    pub fn stop_event(target: Selector, source: SoundSource, event: impl Into<String>) -> String {
        StopSoundCommand::Event {
            target,
            source,
            event: event.into(),
            raw_event: false,
        }
        .build_registered()
    }

    /// Compatibility alias for [`Sound::stop_event`].
    pub fn stop(target: Selector, source: SoundSource, event: impl Into<String>) -> String {
        Self::stop_event(target, source, event)
    }

    /// Stop a sound with an intentionally opaque event token.
    pub fn stop_event_raw(
        target: Selector,
        source: SoundSource,
        event: impl Into<String>,
    ) -> String {
        StopSoundCommand::Event {
            target,
            source,
            event: event.into(),
            raw_event: true,
        }
        .build_registered()
    }
}

impl Build for Sound {
    /// Build the complete `playsound` command string.
    ///
    /// Defaults: target=`@s`, position=`~ ~ ~`.
    fn build(&self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, SoundCommand::Play(self.clone()));
        line
    }
}

impl Validate for Sound {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        if !self.raw_event {
            crate::validate::resource_location_shape(&self.event, "SoundCommand", "event")
                .map_err(|error| sound_error("SAND-SOUND-ID", error.field, error.message))?;
        }
        self.target
            .as_ref()
            .unwrap_or(&Selector::self_())
            .validate(profile)
            .map_err(|error| sound_error("SAND-SOUND-TARGET", "target", error.to_string()))?;
        validate_non_negative(self.volume, "volume")?;
        validate_positive(self.pitch, "pitch")?;
        if let Some(minimum) = self.min_volume {
            validate_non_negative(minimum, "minimum_volume")?;
        }
        if let Some(position) = &self.pos {
            for (field, value) in [
                ("position.x", &position.x),
                ("position.y", &position.y),
                ("position.z", &position.z),
            ] {
                let text = value.to_string();
                if text.contains("NaN") || text.contains("inf") {
                    return Err(sound_error(
                        "SAND-SOUND-NUMERIC",
                        field,
                        format!("coordinates must be finite, got `{text}`"),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl RenderCommand for Sound {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        let target = self.target.clone().unwrap_or_else(Selector::self_);
        let pos = self.pos.clone().unwrap_or_else(Vec3::here);

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
}

/// Structured forms of `stopsound`.
#[derive(Debug, Clone)]
pub enum StopSoundCommand {
    All {
        target: Selector,
    },
    Source {
        target: Selector,
        source: SoundSource,
    },
    Event {
        target: Selector,
        source: SoundSource,
        event: String,
        raw_event: bool,
    },
}

impl StopSoundCommand {
    fn build_registered(self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, SoundCommand::Stop(self));
        line
    }
}

impl Validate for StopSoundCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        let target = match self {
            Self::All { target } | Self::Source { target, .. } | Self::Event { target, .. } => {
                target
            }
        };
        target
            .validate(profile)
            .map_err(|error| sound_error("SAND-SOUND-TARGET", "target", error.to_string()))?;
        if let Self::Event {
            event,
            raw_event: false,
            ..
        } = self
        {
            crate::validate::resource_location_shape(event, "SoundCommand", "event")
                .map_err(|error| sound_error("SAND-SOUND-ID", error.field, error.message))?;
        }
        Ok(())
    }
}

impl RenderCommand for StopSoundCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        match self {
            Self::All { target } => format!("stopsound {target}"),
            Self::Source { target, source } => format!("stopsound {target} {source}"),
            Self::Event {
                target,
                source,
                event,
                ..
            } => format!("stopsound {target} {source} {event}"),
        }
    }
}

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<Sound> for String {
    fn from(v: Sound) -> Self {
        v.build()
    }
}

fn format_float(v: f64) -> String {
    if v == v.trunc() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SoundCommand {
    Play(Sound),
    Stop(StopSoundCommand),
}

impl Validate for SoundCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Play(command) => command.validate(profile),
            Self::Stop(command) => command.validate(profile),
        }
    }
}

fn sound_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("SoundCommand", field, message).with_code(code)
}

fn validate_non_negative(value: f64, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || value < 0.0 {
        Err(sound_error(
            "SAND-SOUND-NUMERIC",
            field,
            format!("must be finite and non-negative, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

fn validate_positive(value: f64, field: &'static str) -> CommandResult<()> {
    if !value.is_finite() || value <= 0.0 {
        Err(sound_error(
            "SAND-SOUND-NUMERIC",
            field,
            format!("must be finite and greater than zero, got `{value}`"),
        ))
    } else {
        Ok(())
    }
}

/// Export-scoped registry family holding this module's rendered
/// sound command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct SoundLines;

impl crate::export_registry::RegistryFamily for SoundLines {
    type State = BTreeMap<String, SoundCommand>;
}

fn register_line(line: &str, command: SoundCommand) {
    crate::export_registry::register_line::<SoundLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<SoundLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
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

    #[test]
    fn validates_ids_and_numeric_domains() {
        assert!(Sound::play("modded:custom.event").try_build().is_ok());
        assert_eq!(
            Sound::play("Bad Event").try_build().unwrap_err().code,
            "SAND-SOUND-ID"
        );
        for sound in [
            Sound::play("minecraft:test").volume(f64::NAN),
            Sound::play("minecraft:test").volume(-1.0),
            Sound::play("minecraft:test").pitch(0.0),
            Sound::play("minecraft:test").min_volume(-0.1),
        ] {
            assert_eq!(sound.try_build().unwrap_err().code, "SAND-SOUND-NUMERIC");
        }
    }

    #[test]
    fn raw_sound_event_is_opaque() {
        assert!(Sound::play_raw("modded payload").try_build().is_ok());
    }
}
