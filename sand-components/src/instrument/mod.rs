//! Builders for `data/<namespace>/instrument/` JSON files.
//!
//! Instruments define custom goat horn sounds and their properties.
//!
//! # Example
//! ```rust,ignore
//! let horn = Instrument::new(rl)
//!     .sound_event("minecraft:item.goat_horn.sound.0")
//!     .use_duration(7)
//!     .range(256.0)
//!     .description(serde_json::json!({"translate": "instrument.minecraft.ponder_goat_horn"}));
//! ```

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

/// A goat horn instrument definition (`data/<namespace>/instrument/<id>.json`).
pub struct Instrument {
    location: ResourceLocation,
    /// Sound event ID played when this instrument is used.
    sound_event: String,
    /// How long (in seconds) the item is used when playing.
    use_duration: f32,
    /// How far (in blocks) the sound carries.
    range: f32,
    /// Optional text component for the instrument's display name.
    description: Option<Value>,
}

impl Instrument {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            sound_event: String::new(),
            use_duration: 7.0,
            range: 256.0,
            description: None,
        }
    }

    pub fn sound_event(mut self, event: impl Into<String>) -> Self {
        self.sound_event = event.into();
        self
    }

    pub fn use_duration(mut self, seconds: f32) -> Self {
        self.use_duration = seconds;
        self
    }

    pub fn range(mut self, blocks: f32) -> Self {
        self.range = blocks;
        self
    }

    pub fn description(mut self, desc: Value) -> Self {
        self.description = Some(desc);
        self
    }
}

impl DatapackComponent for Instrument {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        // In 1.21+ the sound_event may be an object with id + range,
        // but the simpler string form is also accepted.
        map.insert(
            "sound_event".to_string(),
            Value::String(self.sound_event.clone()),
        );
        map.insert(
            "use_duration".to_string(),
            serde_json::json!(self.use_duration),
        );
        map.insert("range".to_string(), serde_json::json!(self.range));
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "instrument"
    }
}
