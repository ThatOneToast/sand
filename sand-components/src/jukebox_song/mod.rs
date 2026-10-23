//! Builders for `data/<namespace>/jukebox_song/` JSON files in Sand's 26+ baseline.
//!
//! Jukebox songs define custom music disc tracks.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `sound_event` is required and uses the shared [`SoundEventDefinition`] model.
//!   References and inline identifiers are validated; inline ranges must be finite.
//! - `song_length` must be finite and positive.
//! - `comparator_output` must be in `0..=15`.
//!
//! - `description` is required and uses canonical text validation.
//!
//! # Example
//! ```rust,ignore
//! let disc = JukeboxSong::new(rl)
//!     .sound_event(crate::SoundEventDefinition::reference("my_pack:music.custom_track".parse::<crate::SoundEventId>().unwrap()))
//!     .song_length(180.0)
//!     .comparator_output(5)
//!     .description(TextComponent::translate("jukebox_song.my_pack.custom_track"));
//! ```

use crate::SoundEventDefinition;
use sand_commands::TextComponent;
use serde_json::Value;

use crate::component_text::ComponentText;
use crate::raw::RawJson;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A jukebox song definition (`data/<namespace>/jukebox_song/<id>.json`).
#[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong", module = "sand::component", summary = "A jukebox song with a required sound, positive duration and display description.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;")]
pub struct JukeboxSong {
    location: ResourceLocation,
    sound_event: Option<SoundEventDefinition>,
    song_length: f32,
    /// Redstone comparator output level (0–15).
    comparator_output: u8,
    description: Option<ComponentText>,
}

impl JukeboxSong {
    /// Begin a song definition; set its sound, positive length and description before export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::new", module = "sand::component", summary = "Begin a song definition requiring sound, duration and description.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());", params(location = "The jukebox song resource identifier."), returns = "A song requiring sound, positive duration and description before export.")]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            sound_event: None,
            song_length: 0.0,
            comparator_output: 1,
            description: None,
        }
    }

    /// Set the required registered or inline sound using the shared sound-event model.
    ///
    /// This replaces any earlier sound, including an explicit raw definition.
    /// Export validates the selected definition before writing the resource.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::sound_event", module = "sand::component", summary = "Set the required registered or inline jukebox sound.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());\nlet sound = sand::component::SoundEventDefinition::inline(\"example:chime\".parse().unwrap(), None);\nlet song = song.sound_event(sound);", params(event = "The shared sound definition, replacing any previous sound."), returns = "The song with the selected sound.")]
    pub fn sound_event(mut self, event: SoundEventDefinition) -> Self {
        self.sound_event = Some(event);
        self
    }

    /// Set the finite, positive duration in seconds.
    ///
    /// Minecraft receives this value in its `length_in_seconds` field.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::song_length", module = "sand::component", summary = "Set the finite positive song duration in seconds.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());\nlet song = song.song_length(180.0);", params(seconds = "The song duration, exported as length_in_seconds."), returns = "The song with its requested duration.")]
    pub fn song_length(mut self, seconds: f32) -> Self {
        self.song_length = seconds;
        self
    }

    /// Set the redstone comparator output level (0–15).
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::comparator_output", module = "sand::component", summary = "Set the song redstone comparator signal in the range zero through fifteen.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());\nlet song = song.comparator_output(0);", params(output = "The comparator signal; invalid levels are rejected rather than clamped."), returns = "The song with its requested comparator output.")]
    pub fn comparator_output(mut self, output: u8) -> Self {
        self.comparator_output = output;
        self
    }

    /// Set the required display description using Sand's typed text model.
    ///
    /// Export validates text recursively and reports the resource and field
    /// path on failure. Use [`Self::raw_description`] only for JSON text shapes
    /// that [`TextComponent`] cannot express.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::description", module = "sand::component", summary = "Set the required song display description using typed text.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());\nlet song = song.description(sand::text::TextComponent::literal(\"Song\"));", params(desc = "The text displayed for this song."), returns = "The song with the typed description.")]
    pub fn description(mut self, desc: TextComponent) -> Self {
        self.description = Some(ComponentText::Typed(Box::new(desc)));
        self
    }

    /// Set an explicit JSON text escape hatch for the required description.
    ///
    /// Export still applies the canonical text shape and known-field checks;
    /// unknown extension fields are preserved. Prefer [`Self::description`]
    /// for ordinary literal, translated or styled text.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::JukeboxSong::raw_description", module = "sand::component", summary = "Set an explicit JSON song description with canonical text validation.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::JukeboxSong;\nlet song = JukeboxSong::new(\"example:song\".parse().unwrap());\nlet song = song.raw_description(sand::component::RawJson::new(serde_json::json!(\"Song\")));", params(desc = "Raw text JSON; known fields are still validated at export."), returns = "The song with a raw description.")]
    pub fn raw_description(mut self, desc: RawJson) -> Self {
        self.description = Some(ComponentText::Raw(desc));
        self
    }
}

impl DatapackComponent for JukeboxSong {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "jukebox_song";
        let sound = self.sound_event.as_ref().ok_or_else(|| {
            validation::error(
                &self.location,
                kind,
                "sound_event",
                "sound_event is required",
            )
        })?;
        sound.validate(&self.location, kind, "sound_event")?;
        validation::require_positive_f32(&self.location, kind, "song_length", self.song_length)?;
        validation::require_u32_in_range(
            &self.location,
            kind,
            "comparator_output",
            self.comparator_output as u32,
            0,
            15,
        )?;
        let description = self.description.as_ref().ok_or_else(|| {
            validation::error(
                &self.location,
                kind,
                "description",
                "description is required",
            )
        })?;
        description.validate(&self.location, kind, "description")?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(sound) = &self.sound_event {
            map.insert("sound_event".to_string(), sound.to_json());
        }
        map.insert(
            "length_in_seconds".to_string(),
            serde_json::json!(self.song_length),
        );
        map.insert(
            "comparator_output".to_string(),
            serde_json::json!(self.comparator_output),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.to_json());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "jukebox_song"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::JukeboxSongs]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "theme").unwrap()
    }

    fn valid() -> JukeboxSong {
        JukeboxSong::new(rl())
            .sound_event(crate::SoundEventDefinition::reference(
                "minecraft:music.disc.13"
                    .parse::<crate::SoundEventId>()
                    .unwrap(),
            ))
            .song_length(178.0)
            .comparator_output(5)
            .description(TextComponent::literal("Test song"))
    }

    #[test]
    fn valid_jukebox_song_exports_deterministic_json() {
        let song = valid();
        assert!(song.validate().is_ok());
        let a = serde_json::to_string_pretty(&song.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&song.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn empty_sound_event_is_rejected() {
        let song = JukeboxSong::new(rl()).song_length(10.0);
        let err = song.validate().unwrap_err();
        assert!(err.to_string().contains("sound_event"), "{err}");
    }

    #[test]
    fn invalid_sound_event_is_rejected() {
        let song = JukeboxSong::new(rl())
            .sound_event(crate::SoundEventDefinition::raw(crate::RawJson::new(
                serde_json::json!("not valid"),
            )))
            .song_length(10.0);
        assert!(song.validate().is_err());
    }

    #[test]
    fn tag_prefixed_sound_event_is_rejected() {
        let song = JukeboxSong::new(rl())
            .sound_event(crate::SoundEventDefinition::raw(crate::RawJson::new(
                serde_json::json!("#minecraft:music_disc"),
            )))
            .song_length(10.0);
        let err = song.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("sound_event"), "{msg}");
        assert!(msg.contains("tag"), "{msg}");
    }

    #[test]
    fn dotted_sound_event_is_accepted() {
        let song = valid().sound_event(crate::SoundEventDefinition::reference(
            "minecraft:music_disc.13"
                .parse::<crate::SoundEventId>()
                .unwrap(),
        ));
        assert!(song.validate().is_ok());
    }

    #[test]
    fn nan_song_length_is_rejected() {
        let song = valid().song_length(f32::NAN);
        assert!(song.validate().is_err());
    }

    #[test]
    fn infinite_song_length_is_rejected() {
        let song = valid().song_length(f32::INFINITY);
        assert!(song.validate().is_err());
    }

    #[test]
    fn zero_song_length_is_rejected() {
        let song = valid().song_length(0.0);
        assert!(song.validate().is_err());
    }

    #[test]
    fn negative_song_length_is_rejected() {
        let song = valid().song_length(-1.0);
        assert!(song.validate().is_err());
    }

    /// Vanilla permits silent comparator output for a song.
    #[test]
    fn comparator_output_zero_is_accepted() {
        let song = valid().comparator_output(0);
        assert!(song.validate().is_ok());
    }

    #[test]
    fn comparator_output_sixteen_is_rejected() {
        let song = valid().comparator_output(16);
        assert!(song.validate().is_err());
    }

    #[test]
    fn comparator_output_one_is_accepted() {
        let song = valid().comparator_output(1);
        assert!(song.validate().is_ok());
    }

    #[test]
    fn comparator_output_fifteen_is_accepted() {
        let song = valid().comparator_output(15);
        assert!(song.validate().is_ok());
    }

    #[test]
    fn comparator_output_no_longer_silently_clamped() {
        let song = JukeboxSong::new(rl())
            .sound_event(crate::SoundEventDefinition::reference(
                "minecraft:music.disc.cat"
                    .parse::<crate::SoundEventId>()
                    .unwrap(),
            ))
            .song_length(10.0)
            .comparator_output(20);
        assert_eq!(song.comparator_output, 20);
        assert!(song.validate().is_err());
    }

    #[test]
    fn valid_jukebox_song_json_is_stable() {
        let song = valid();
        let json = song.to_json();
        assert_eq!(json["sound_event"], "minecraft:music.disc.13");
        assert_eq!(json["length_in_seconds"], 178.0);
        assert_eq!(json["comparator_output"], 5);
    }

    #[test]
    fn invalid_jukebox_song_fails_export() {
        let song = JukeboxSong::new(rl());
        assert!(song.try_content().is_err());
    }
}
