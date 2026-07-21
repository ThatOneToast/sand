//! Builders for `data/<namespace>/instrument/` JSON files.
//!
//! Instruments define custom goat horn sounds and their properties.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `sound_event` must be non-empty and a valid **plain** resource location
//!   (a `#namespace:path` tag reference is rejected — the field is
//!   serialized as a single concrete sound event, not a tag).
//! - `use_duration` must be finite and positive.
//! - `range` must be finite and positive.
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

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A goat horn instrument definition (`data/<namespace>/instrument/<id>.json`).
pub struct Instrument {
    location: ResourceLocation,
    sound_event: String,
    use_duration: f32,
    range: f32,
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

    fn validate(&self) -> SandResult<()> {
        let kind = "instrument";
        validation::require_non_empty(&self.location, kind, "sound_event", &self.sound_event)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "sound_event",
            &self.sound_event,
        )?;
        validation::require_positive_f32(&self.location, kind, "use_duration", self.use_duration)?;
        validation::require_positive_f32(&self.location, kind, "range", self.range)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "horn").unwrap()
    }

    fn valid() -> Instrument {
        Instrument::new(rl()).sound_event("minecraft:item.goat_horn.sound.0")
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
        let inst = Instrument::new(rl()).sound_event("not valid!!");
        let err = inst.validate().unwrap_err();
        assert!(err.to_string().contains("sound_event"), "{err}");
    }

    #[test]
    fn tag_prefixed_sound_event_is_rejected() {
        let inst = Instrument::new(rl()).sound_event("#minecraft:music_disc");
        let err = inst.validate().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("sound_event"), "{msg}");
        assert!(msg.contains("tag"), "{msg}");
    }

    #[test]
    fn dotted_sound_event_is_accepted() {
        let inst = valid().sound_event("minecraft:music_disc.13");
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
