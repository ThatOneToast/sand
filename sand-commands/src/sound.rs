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

#[doc = "**API Contract:** Run `sand api show sand::command::IntoSoundEvent` for the canonical contract."]
/// Conversion into a sound-event resource-location token.
pub trait IntoSoundEvent {
    /// Converts a typed or validated value into a Minecraft sound-event identifier.
    #[doc = "**API Contract:** Run `sand api show sand::command::IntoSoundEvent::into_sound_event` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::SoundSource` for the canonical contract."]
/// Minecraft audio channel/category for sound playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    #[doc = "Selects the master form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Master` for the canonical contract."]
    Master,
    #[doc = "Selects the music form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Music` for the canonical contract."]
    Music,
    #[doc = "Selects the record form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Record` for the canonical contract."]
    Record,
    #[doc = "Selects the weather form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Weather` for the canonical contract."]
    Weather,
    #[doc = "Selects the block form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Block` for the canonical contract."]
    Block,
    #[doc = "Selects the hostile form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Hostile` for the canonical contract."]
    Hostile,
    #[doc = "Selects the neutral form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Neutral` for the canonical contract."]
    Neutral,
    #[doc = "Selects the player form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Player` for the canonical contract."]
    Player,
    #[doc = "Selects the ui form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Ui` for the canonical contract."]
    Ui,
    #[doc = "Selects the ambient form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Ambient` for the canonical contract."]
    Ambient,
    #[doc = "Selects the voice form of the sound source Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::SoundSource::Voice` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::Sound` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::play` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::play_raw` for the canonical contract."]
    pub fn play_raw(event: impl IntoSoundEvent) -> Self {
        Self {
            raw_event: true,
            ..Self::play(event)
        }
    }

    /// Set the target entity/player who hears the sound (default: `@s`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::to` for the canonical contract."]
    pub fn to(mut self, selector: Selector) -> Self {
        self.target = Some(selector);
        self
    }

    /// Set the sound source/channel category (default: `master`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::source` for the canonical contract."]
    pub fn source(mut self, source: SoundSource) -> Self {
        self.source = source;
        self
    }

    /// Set the position in the world where the sound originates (default: `~ ~ ~`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::at` for the canonical contract."]
    pub fn at(mut self, pos: Vec3) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Set the volume multiplier (default: `1.0`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::volume` for the canonical contract."]
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Set the pitch multiplier (default: `1.0`).
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::pitch` for the canonical contract."]
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    /// Set minimum volume for players far from the sound origin.
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::min_volume` for the canonical contract."]
    pub fn min_volume(mut self, min: f64) -> Self {
        self.min_volume = Some(min);
        self
    }

    // ── stopsound helpers ─────────────────────────────────────────────────────

    /// `stopsound <selector>` — stop all sounds playing for the target.
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::stop_all` for the canonical contract."]
    pub fn stop_all(target: Selector) -> String {
        StopSoundCommand::All { target }.build_registered()
    }

    /// `stopsound <selector> <source>` — stop all sounds in a specific category.
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::stop_source` for the canonical contract."]
    pub fn stop_source(target: Selector, source: SoundSource) -> String {
        StopSoundCommand::Source { target, source }.build_registered()
    }

    /// `stopsound <selector> <source> <event>` — stop a specific sound for the target.
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::stop_event` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::stop` for the canonical contract."]
    pub fn stop(target: Selector, source: SoundSource, event: impl Into<String>) -> String {
        Self::stop_event(target, source, event)
    }

    /// Stop a sound with an intentionally opaque event token.
    #[doc = "**API Contract:** Run `sand api show sand::command::Sound::stop_event_raw` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand` for the canonical contract."]
/// Structured forms of `stopsound`.
#[derive(Debug, Clone)]
pub enum StopSoundCommand {
    #[doc = "Selects the all form of the stop sound command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::All` for the canonical contract."]
    All {
        /// `target` provides the command target when the variant selects the all form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::All::target` for the canonical contract."]
        target: Selector,
    },
    #[doc = "Selects the source form of the stop sound command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Source` for the canonical contract."]
    Source {
        /// `target` provides the command target when the variant selects the source form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Source::target` for the canonical contract."]
        target: Selector,
        /// `source` provides the source when the variant selects the source form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Source::source` for the canonical contract."]
        source: SoundSource,
    },
    #[doc = "Selects the event form of the stop sound command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Event` for the canonical contract."]
    Event {
        /// `target` provides the command target when the variant selects the event form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Event::target` for the canonical contract."]
        target: Selector,
        /// `source` provides the source when the variant selects the event form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Event::source` for the canonical contract."]
        source: SoundSource,
        /// `event` provides the event when the variant selects the event form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Event::event` for the canonical contract."]
        event: String,
        /// `raw_event` provides the raw event when the variant selects the event form of the stop sound command Minecraft command value.
        #[doc = "**API Contract:** Run `sand api show sand::command::StopSoundCommand::Event::raw_event` for the canonical contract."]
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
