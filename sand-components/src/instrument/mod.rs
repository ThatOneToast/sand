//! Builders for `data/<namespace>/instrument/` JSON files.
//!
//! Instruments define custom goat horn sounds and their properties.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `sound_event` is required and uses the shared [`SoundEventDefinition`] model.
//!   References and inline identifiers are validated; inline ranges must be finite.
//! - `use_duration` must be finite and positive.
//! - `range` must be finite and positive.
//!
//! - `description` is required and uses canonical text validation.
//!
//! # Example
//! ```rust,ignore
//! let horn = Instrument::new(rl)
//!     .sound_event(crate::SoundEventDefinition::reference("minecraft:item.goat_horn.sound.0".parse::<crate::SoundEventId>().unwrap()))
//!     .use_duration(7.0)
//!     .range(256.0)
//!     .description(TextComponent::translate("instrument.minecraft.ponder_goat_horn"));
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

/// A goat horn instrument definition (`data/<namespace>/instrument/<id>.json`).
#[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument", module = "sand::component", summary = "A goat horn instrument with a required sound and display description.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;")]
pub struct Instrument {
    location: ResourceLocation,
    sound_event: Option<SoundEventDefinition>,
    use_duration: f32,
    range: f32,
    description: Option<ComponentText>,
}

impl Instrument {
    /// Begin a resource definition; set its required sound and description before export.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::new", module = "sand::component", summary = "Begin an instrument definition with default duration and range.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());", params(location = "The instrument resource identifier."), returns = "An instrument requiring sound and description before export.")]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            sound_event: None,
            use_duration: 7.0,
            range: 256.0,
            description: None,
        }
    }

    /// Set the required registered or inline sound using the shared sound-event model.
    ///
    /// This replaces any earlier sound, including an explicit raw definition.
    /// Export validates the selected definition before writing the resource.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::sound_event", module = "sand::component", summary = "Set the required registered or inline instrument sound.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());\nlet sound = sand::component::SoundEventDefinition::inline(\"example:chime\".parse().unwrap(), None);\nlet horn = horn.sound_event(sound);", params(event = "The shared sound definition, replacing any previous sound."), returns = "The instrument with the selected sound.")]
    pub fn sound_event(mut self, event: SoundEventDefinition) -> Self {
        self.sound_event = Some(event);
        self
    }

    /// Set the finite, positive horn use duration in seconds.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::use_duration", module = "sand::component", summary = "Set the positive horn use duration in seconds.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());\nlet horn = horn.use_duration(7.0);", params(seconds = "A finite positive duration."), returns = "The instrument with the requested duration.")]
    pub fn use_duration(mut self, seconds: f32) -> Self {
        self.use_duration = seconds;
        self
    }

    /// Set the finite, positive instrument range, independent of an inline sound range.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::range", module = "sand::component", summary = "Set the positive instrument range independently of inline sound range.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());\nlet horn = horn.range(256.0);", params(blocks = "A finite positive instrument range in blocks."), returns = "The instrument with the requested outer range.")]
    pub fn range(mut self, blocks: f32) -> Self {
        self.range = blocks;
        self
    }

    /// Set the required display description using Sand's typed text model.
    ///
    /// Export validates text recursively and reports the resource and field
    /// path on failure. Use [`Self::raw_description`] only for JSON text shapes
    /// that [`TextComponent`] cannot express.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::description", module = "sand::component", summary = "Set the required instrument display description using typed text.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());\nlet horn = horn.description(sand::text::TextComponent::literal(\"Horn\"));", params(desc = "The text displayed for this instrument."), returns = "The instrument with the typed description.")]
    pub fn description(mut self, desc: TextComponent) -> Self {
        self.description = Some(ComponentText::Typed(Box::new(desc)));
        self
    }

    /// Set an explicit JSON text escape hatch for the required description.
    ///
    /// Export still applies the canonical text shape and known-field checks;
    /// unknown extension fields are preserved. Prefer [`Self::description`]
    /// for ordinary literal, translated or styled text.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::Instrument::raw_description", module = "sand::component", summary = "Set an explicit JSON instrument description with canonical text validation.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::Instrument;\nlet horn = Instrument::new(\"example:horn\".parse().unwrap());\nlet horn = horn.raw_description(sand::component::RawJson::new(serde_json::json!(\"Horn\")));", params(desc = "Raw text JSON; known fields are still validated at export."), returns = "The instrument with a raw description.")]
    pub fn raw_description(mut self, desc: RawJson) -> Self {
        self.description = Some(ComponentText::Raw(desc));
        self
    }
}

impl DatapackComponent for Instrument {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "instrument";
        let sound = self.sound_event.as_ref().ok_or_else(|| {
            validation::error(
                &self.location,
                kind,
                "sound_event",
                "sound_event is required",
            )
        })?;
        sound.validate(&self.location, kind, "sound_event")?;
        validation::require_positive_f32(&self.location, kind, "use_duration", self.use_duration)?;
        validation::require_positive_f32(&self.location, kind, "range", self.range)?;
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
            "use_duration".to_string(),
            serde_json::json!(self.use_duration),
        );
        map.insert("range".to_string(), serde_json::json!(self.range));
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.to_json());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "instrument"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "horn").unwrap()
    }

    fn valid() -> Instrument {
        Instrument::new(rl())
            .sound_event(crate::SoundEventDefinition::reference(
                "minecraft:item.goat_horn.sound.0"
                    .parse::<crate::SoundEventId>()
                    .unwrap(),
            ))
            .description(TextComponent::literal("Test horn"))
    }

    #[test]
    fn valid_instrument_exports_deterministic_json() {
        let inst = valid();
        assert!(inst.validate().is_ok());
        let a = serde_json::to_string_pretty(&inst.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&inst.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn empty_sound_event_is_rejected() {
        let inst = Instrument::new(rl());
        let err = inst.validate().unwrap_err();
        assert!(err.to_string().contains("sound_event"), "{err}");
    }

    #[test]
    fn invalid_sound_event_is_rejected() {
        let inst = Instrument::new(rl()).sound_event(crate::SoundEventDefinition::raw(
            crate::RawJson::new(serde_json::json!("not valid!!")),
        ));
        let err = inst.validate().unwrap_err();
        assert!(err.to_string().contains("sound_event"), "{err}");
    }

    #[test]
    fn tag_prefixed_sound_event_is_rejected() {
        let inst = Instrument::new(rl()).sound_event(crate::SoundEventDefinition::raw(
            crate::RawJson::new(serde_json::json!("#minecraft:music_disc")),
        ));
        let err = inst.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("sound_event"), "{msg}");
        assert!(msg.contains("tag"), "{msg}");
    }

    #[test]
    fn dotted_sound_event_is_accepted() {
        let inst = valid().sound_event(crate::SoundEventDefinition::reference(
            "minecraft:music_disc.13"
                .parse::<crate::SoundEventId>()
                .unwrap(),
        ));
        assert!(inst.validate().is_ok());
    }

    #[test]
    fn nan_use_duration_is_rejected() {
        let inst = valid().use_duration(f32::NAN);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn infinite_use_duration_is_rejected() {
        let inst = valid().use_duration(f32::INFINITY);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn zero_use_duration_is_rejected() {
        let inst = valid().use_duration(0.0);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn negative_use_duration_is_rejected() {
        let inst = valid().use_duration(-1.0);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn nan_range_is_rejected() {
        let inst = valid().range(f32::NAN);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn infinite_range_is_rejected() {
        let inst = valid().range(f32::INFINITY);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn zero_range_is_rejected() {
        let inst = valid().range(0.0);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn negative_range_is_rejected() {
        let inst = valid().range(-1.0);
        assert!(inst.validate().is_err());
    }

    #[test]
    fn valid_instrument_json_is_stable() {
        let inst = valid();
        let json = inst.to_json();
        assert_eq!(json["sound_event"], "minecraft:item.goat_horn.sound.0");
        assert_eq!(json["use_duration"], 7.0);
        assert_eq!(json["range"], 256.0);
    }

    #[test]
    fn invalid_instrument_fails_export() {
        let inst = Instrument::new(rl());
        assert!(inst.try_content().is_err());
    }
}
