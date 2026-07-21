//! Builders for `data/<namespace>/jukebox_song/` JSON files (Minecraft 1.21+).
//!
//! Jukebox songs define custom music disc tracks.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `sound_event` must be non-empty and a valid resource location.
//! - `song_length` must be finite and positive.
//! - `comparator_output` must be in `1..=15`.
//!
//! # Example
//! ```rust,ignore
//! let disc = JukeboxSong::new(rl)
//!     .sound_event("my_pack:music.custom_track")
//!     .song_length(180.0)
//!     .comparator_output(5)
//!     .description(serde_json::json!({"translate": "jukebox_song.my_pack.custom_track"}));
//! ```

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A jukebox song definition (`data/<namespace>/jukebox_song/<id>.json`).
pub struct JukeboxSong {
    location: ResourceLocation,
    sound_event: String,
    song_length: f32,
    /// Redstone comparator output level (1–15).
    comparator_output: u8,
    description: Option<Value>,
}

impl JukeboxSong {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            sound_event: String::new(),
            song_length: 0.0,
            comparator_output: 1,
            description: None,
        }
    }

    pub fn sound_event(mut self, event: impl Into<String>) -> Self {
        self.sound_event = event.into();
        self
    }

    pub fn song_length(mut self, seconds: f32) -> Self {
        self.song_length = seconds;
        self
    }

    /// Set the redstone comparator output level (1–15).
    pub fn comparator_output(mut self, output: u8) -> Self {
        self.comparator_output = output;
        self
    }

    pub fn description(mut self, desc: Value) -> Self {
        self.description = Some(desc);
        self
    }
}

impl DatapackComponent for JukeboxSong {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "jukebox_song";
        validation::require_non_empty(&self.location, kind, "sound_event", &self.sound_event)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "sound_event",
            &self.sound_event,
        )?;
        validation::require_positive_f32(&self.location, kind, "song_length", self.song_length)?;
        validation::require_u32_in_range(
            &self.location,
            kind,
            "comparator_output",
            self.comparator_output as u32,
            1,
            15,
        )?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "sound_event".to_string(),
            Value::String(self.sound_event.clone()),
        );
        map.insert(
            "song_length".to_string(),
            serde_json::json!(self.song_length),
        );
        map.insert(
            "comparator_output".to_string(),
            serde_json::json!(self.comparator_output),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.clone());
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
            .sound_event("minecraft:music.disc.13")
            .song_length(178.0)
            .comparator_output(5)
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
            .sound_event("not valid")
            .song_length(10.0);
        assert!(song.validate().is_err());
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

    #[test]
    fn comparator_output_zero_is_rejected() {
        let song = valid().comparator_output(0);
        assert!(song.validate().is_err());
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
            .sound_event("minecraft:music.disc.cat")
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
        assert_eq!(json["song_length"], 178.0);
        assert_eq!(json["comparator_output"], 5);
    }

    #[test]
    fn invalid_jukebox_song_fails_export() {
        let song = JukeboxSong::new(rl());
        assert!(song.try_content().is_err());
    }
}
