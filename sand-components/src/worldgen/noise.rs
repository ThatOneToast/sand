//! Builder for `data/<namespace>/worldgen/noise/<id>.json`.
//!
//! Noise parameters are one of the few genuinely stable worldgen shapes: a
//! `firstOctave` integer plus a list of octave `amplitudes`. [`Noise::new`]
//! models exactly that shape with validated numerics, while
//! [`Noise::raw_field`] is an explicit escape hatch for modded or
//! version-specific additions.
//!
//! Emitted noise parameters are referenced from density functions and noise
//! routers through [`NoiseId`]; see [`Noise::id`] and
//! [`crate::worldgen::density_function`].

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::component::DatapackComponent;
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::NoiseId;
use crate::resource_location::ResourceLocation;
use crate::validation;

const KIND: &str = "worldgen/noise";

const TYPED_FIELDS: &[&str] = &["firstOctave", "amplitudes"];

/// Vanilla's `PerlinNoise` rejects octave sets outside this range, so the
/// builder refuses to emit a file the game would reject at world load.
const MIN_FIRST_OCTAVE: i32 = -32;
/// See [`MIN_FIRST_OCTAVE`].
const MAX_FIRST_OCTAVE: i32 = 32;
/// Vanilla caps the total number of octaves a noise may declare.
const MAX_AMPLITUDES: usize = 32;

/// A noise-parameter definition (`data/<namespace>/worldgen/noise/<id>.json`).
///
/// ```
/// use sand_components::{DatapackComponent, Noise, ResourceLocation};
///
/// let noise = Noise::new(
///     ResourceLocation::new("example", "ridges").unwrap(),
///     -7,
///     [1.0, 2.0, 1.0],
/// );
/// noise.validate().unwrap();
/// assert_eq!(noise.component_dir(), "worldgen/noise");
/// assert_eq!(noise.to_json()["firstOctave"], -7);
/// ```
#[derive(Debug, Clone)]
pub struct Noise {
    location: ResourceLocation,
    first_octave: i32,
    amplitudes: Vec<f64>,
    raw_fields: BTreeMap<String, RawJson>,
}

impl Noise {
    /// Create a noise-parameter definition from its first octave and
    /// amplitudes.
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::new` for the canonical contract."]
    pub fn new(
        location: ResourceLocation,
        first_octave: i32,
        amplitudes: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            location,
            first_octave,
            amplitudes: amplitudes.into_iter().collect(),
            raw_fields: BTreeMap::new(),
        }
    }

    /// The typed registry ID other worldgen files use to reference this noise.
    ///
    /// ```
    /// use sand_components::{Noise, ResourceLocation};
    ///
    /// let noise = Noise::new(
    ///     ResourceLocation::new("example", "ridges").unwrap(),
    ///     -7,
    ///     [1.0],
    /// );
    /// assert_eq!(noise.id().to_string(), "example:ridges");
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::id` for the canonical contract."]
    pub fn id(&self) -> NoiseId {
        NoiseId::custom(self.location.clone())
    }

    /// Set the first (lowest) octave.
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::first_octave` for the canonical contract."]
    pub fn first_octave(mut self, first_octave: i32) -> Self {
        self.first_octave = first_octave;
        self
    }

    /// Replace the octave amplitudes.
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::amplitudes` for the canonical contract."]
    pub fn amplitudes(mut self, amplitudes: impl IntoIterator<Item = f64>) -> Self {
        self.amplitudes = amplitudes.into_iter().collect();
        self
    }

    /// Append a single octave amplitude.
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::amplitude` for the canonical contract."]
    pub fn amplitude(mut self, amplitude: f64) -> Self {
        self.amplitudes.push(amplitude);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[doc = "**API Contract:** Run `sand api show sand::component::Noise::raw_field` for the canonical contract."]
    pub fn raw_field(mut self, key: impl Into<String>, value: RawJson) -> Self {
        self.raw_fields.insert(key.into(), value);
        self
    }
}

impl DatapackComponent for Noise {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        if !(MIN_FIRST_OCTAVE..=MAX_FIRST_OCTAVE).contains(&self.first_octave) {
            return Err(validation::error(
                &self.location,
                KIND,
                "firstOctave",
                &format!(
                    "firstOctave must be in {MIN_FIRST_OCTAVE}..={MAX_FIRST_OCTAVE}; received {}",
                    self.first_octave
                ),
            ));
        }
        validation::require_non_empty_collection(
            &self.location,
            KIND,
            "amplitudes",
            self.amplitudes.len(),
        )?;
        if self.amplitudes.len() > MAX_AMPLITUDES {
            return Err(validation::error(
                &self.location,
                KIND,
                "amplitudes",
                &format!(
                    "amplitudes must contain at most {MAX_AMPLITUDES} octaves; received {}",
                    self.amplitudes.len()
                ),
            ));
        }
        for amplitude in &self.amplitudes {
            if !amplitude.is_finite() {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "amplitudes",
                    &format!("amplitudes must be finite; received {amplitude}"),
                ));
            }
        }
        for key in self.raw_fields.keys() {
            validation::require_non_empty(&self.location, KIND, "raw_field", key)?;
            validation::reject_whitespace_only(&self.location, KIND, "raw_field", key)?;
            validation::reject_control_chars(&self.location, KIND, "raw_field", key)?;
            if TYPED_FIELDS.contains(&key.as_str()) {
                return Err(validation::error(
                    &self.location,
                    KIND,
                    "raw_field",
                    &format!("raw field `{key}` would override a typed field"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("firstOctave".into(), self.first_octave.into());
        map.insert(
            "amplitudes".into(),
            Value::Array(
                self.amplitudes
                    .iter()
                    .map(|amplitude| serde_json::json!(amplitude))
                    .collect(),
            ),
        );
        for (key, value) in &self.raw_fields {
            map.insert(key.clone(), value.as_value().clone());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "worldgen/noise"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "ridges").unwrap()
    }

    #[test]
    fn typed_shape_serializes_and_validates() {
        let noise = Noise::new(location(), -7, [1.0, 1.0]).amplitude(0.5);
        noise.validate().unwrap();
        let json = noise.to_json();
        assert_eq!(json["firstOctave"], -7);
        assert_eq!(json["amplitudes"], serde_json::json!([1.0, 1.0, 0.5]));
        assert_eq!(noise.component_dir(), "worldgen/noise");
    }

    #[test]
    fn typed_registry_id_round_trips() {
        let noise = Noise::new(location(), -7, [1.0]);
        assert_eq!(noise.id().to_string(), "test:ridges");
        assert_eq!(noise.id(), NoiseId::custom(location()));
    }

    #[test]
    fn non_finite_amplitudes_are_rejected() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = Noise::new(location(), 0, [value]).validate().unwrap_err();
            assert!(err.to_string().contains("amplitudes"), "{err}");
        }
    }

    #[test]
    fn empty_and_oversized_amplitudes_are_rejected() {
        assert!(Noise::new(location(), 0, []).validate().is_err());
        assert!(
            Noise::new(location(), 0, std::iter::repeat_n(1.0, MAX_AMPLITUDES + 1))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn out_of_range_first_octave_is_rejected() {
        let err = Noise::new(location(), -33, [1.0]).validate().unwrap_err();
        assert!(err.to_string().contains("firstOctave"), "{err}");
        assert!(Noise::new(location(), 33, [1.0]).validate().is_err());
    }

    #[test]
    fn malformed_resource_ids_are_rejected_at_construction() {
        assert!("example:bad path".parse::<NoiseId>().is_err());
        assert!(ResourceLocation::new("example", "Bad Path").is_err());
    }

    #[test]
    fn raw_escape_hatch_emits_stable_json_and_rejects_typed_overrides() {
        let noise = Noise::new(location(), -7, [1.0])
            .raw_field("modded:smoothing", RawJson::new(serde_json::json!(true)));
        noise.validate().unwrap();
        assert_eq!(noise.to_json()["modded:smoothing"], true);

        assert!(
            Noise::new(location(), -7, [1.0])
                .raw_field("amplitudes", RawJson::new(serde_json::json!([9.0])))
                .validate()
                .is_err()
        );
        assert!(
            Noise::new(location(), -7, [1.0])
                .raw_field("  ", RawJson::new(serde_json::json!(1)))
                .validate()
                .is_err()
        );
    }
}
