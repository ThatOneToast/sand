//! Builders for `data/<namespace>/jukebox_song/` JSON files (Minecraft 1.21+).
//!
//! Jukebox songs define custom music disc tracks.
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

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A jukebox song definition (`data/<namespace>/jukebox_song/<id>.json`).
pub struct JukeboxSong {
    location: ResourceLocation,
    /// Sound event ID that plays when this disc is inserted.
    sound_event: String,
    /// Duration of the song in seconds.
    song_length: f32,
    /// Comparator output value (1–13) when this disc is in a jukebox.
    comparator_output: u8,
    /// Optional text component for the disc description in the tooltip.
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

    /// Set the redstone comparator output level (1–13).
    pub fn comparator_output(mut self, output: u8) -> Self {
        self.comparator_output = output.clamp(1, 15);
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
}
