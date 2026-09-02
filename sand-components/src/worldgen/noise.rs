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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Noise",
    aliases = ["sand::prelude::Noise"],
    module = "sand::component",
    summary = "A noise-parameter definition (`data/<namespace>/worldgen/noise/<id>.json`).",
    context = "A noise-parameter definition (`data/<namespace>/worldgen/noise/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Noise;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::new",
        aliases = ["sand::prelude::Noise::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a noise-parameter definition from its first octave and amplitudes.",
        context = "Create a noise-parameter definition from its first octave and amplitudes. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a noise-parameter definition from its first octave and amplitudes.", first_octave = "`first_octave` supplies the first octave value used to create a noise-parameter definition from its first octave and amplitudes.", amplitudes = "`amplitudes` supplies the amplitudes value used to create a noise-parameter definition from its first octave and amplitudes."),
        returns = "A newly constructed `Noise` configured to create a noise-parameter definition from its first octave and amplitudes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, first_octave: i32, amplitudes: impl IntoIterator < Item = f64 >)  {\n    let noise = sand::component::Noise::new(location, first_octave, amplitudes);\n}",
    )]
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
    /// use sand_components::worldgen::Noise;
    /// use sand_components::ResourceLocation;
    ///
    /// let noise = Noise::new(
    ///     ResourceLocation::new("example", "ridges").unwrap(),
    ///     -7,
    ///     [1.0],
    /// );
    /// assert_eq!(noise.id().to_string(), "example:ridges");
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::id",
        aliases = ["sand::prelude::Noise::id"],
        module = "sand::component",
        kind = "method",
        summary = "The typed registry ID other worldgen files use to reference this noise.",
        context = "The typed registry ID other worldgen files use to reference this noise. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `NoiseId` value produced to use the typed registry ID other worldgen files use to reference this noise.",
        example = "use sand::component::Noise;\nuse sand::ResourceLocation;\nlet noise = Noise::new(\nResourceLocation::new(\"example\", \"ridges\").unwrap(),\n-7,\n[1.0],\n);\nassert_eq!(noise.id().to_string(), \"example:ridges\");",
    )]
    pub fn id(&self) -> NoiseId {
        NoiseId::custom(self.location.clone())
    }

    /// Set the first (lowest) octave.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::first_octave",
        aliases = ["sand::prelude::Noise::first_octave"],
        module = "sand::component",
        kind = "method",
        summary = "Set the first (lowest) octave.",
        context = "Set the first (lowest) octave. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(first_octave = "`first_octave` supplies the first octave value used to set the first (lowest) octave."),
        returns = "The `Noise` value with the documented change applied to set the first (lowest) octave.",
        example = "use sand::prelude::*;\n\nfn demonstrate(noise_value: sand::component::Noise, first_octave: i32)  {\n    let updated_noise = noise_value.first_octave(first_octave);\n}",
    )]
    pub fn first_octave(mut self, first_octave: i32) -> Self {
        self.first_octave = first_octave;
        self
    }

    /// Replace the octave amplitudes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::amplitudes",
        aliases = ["sand::prelude::Noise::amplitudes"],
        module = "sand::component",
        kind = "method",
        summary = "Replace the octave amplitudes.",
        context = "Replace the octave amplitudes. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(amplitudes = "`amplitudes` supplies the amplitudes value used to replace the octave amplitudes."),
        returns = "The `Noise` value with the documented change applied to replace the octave amplitudes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(noise_value: sand::component::Noise, amplitudes: impl IntoIterator < Item = f64 >)  {\n    let updated_noise = noise_value.amplitudes(amplitudes);\n}",
    )]
    pub fn amplitudes(mut self, amplitudes: impl IntoIterator<Item = f64>) -> Self {
        self.amplitudes = amplitudes.into_iter().collect();
        self
    }

    /// Append a single octave amplitude.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::amplitude",
        aliases = ["sand::prelude::Noise::amplitude"],
        module = "sand::component",
        kind = "method",
        summary = "Append a single octave amplitude.",
        context = "Append a single octave amplitude. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(amplitude = "`amplitude` supplies the amplitude value used to append a single octave amplitude."),
        returns = "The `Noise` value with the documented change applied to append a single octave amplitude.",
        example = "use sand::prelude::*;\n\nfn demonstrate(noise_value: sand::component::Noise, amplitude: f64)  {\n    let updated_noise = noise_value.amplitude(amplitude);\n}",
    )]
    pub fn amplitude(mut self, amplitude: f64) -> Self {
        self.amplitudes.push(amplitude);
        self
    }

    /// Add a modded or version-specific field not represented by the typed API.
    ///
    /// Typed field names cannot be overridden through this escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Noise::raw_field",
        aliases = ["sand::prelude::Noise::raw_field"],
        module = "sand::component",
        kind = "method",
        summary = "Add a modded or version-specific field not represented by the typed API.",
        context = "Add a modded or version-specific field not represented by the typed API. Typed field names cannot be overridden through this escape hatch.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to add a modded or version-specific field not represented by the typed API.", value = "`value` provides the value being applied or compared used to add a modded or version-specific field not represented by the typed API."),
        returns = "The `Noise` value with the documented change applied to add a modded or version-specific field not represented by the typed API.",
        example = "use sand::prelude::*;\n\nfn demonstrate(noise_value: sand::component::Noise, key: impl Into < String >, value: sand::component::RawJson)  {\n    let updated_noise = noise_value.raw_field(key, value);\n}",
    )]
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
