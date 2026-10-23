//! Shared sound-event holders for instrument and jukebox song resources.

use sand_commands::resource::{RegistryReference, SoundEvent};
use serde_json::{Value, json};

use crate::{RawJson, ResourceLocation, SoundEventId, error::Result, validation};

/// A registered sound event or an inline sound definition shared by instruments and songs.
///
/// References resolve an existing sound-event registry entry. Inline definitions
/// name a sound from a resource pack directly, optionally fixing its audible
/// range; they do not register a new sound event or package audio assets.
/// Use [`Self::raw`] only for forms the typed constructors cannot express.
#[derive(Debug, Clone)]
#[sand_macros::api(registry = sand_api_contract, path = "sand::component::SoundEventDefinition", module = "sand::component", summary = "A registered or inline sound definition shared by instruments and jukebox songs.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::SoundEventDefinition;")]
pub struct SoundEventDefinition(SoundEventKind);

#[derive(Debug, Clone)]
enum SoundEventKind {
    Reference(SoundEventId),
    Inline {
        sound_id: ResourceLocation,
        fixed_range: Option<f32>,
    },
    Raw(RawJson),
}

impl SoundEventDefinition {
    /// Refer to an existing event using a generated vanilla sound or validated custom ID.
    ///
    /// The JSON value is a string. Sand validates identifier syntax, but the
    /// target event must exist in Minecraft's registry when the datapack loads.
    /// A custom audio asset alone does not register a sound event; use [`Self::inline`]
    /// to name such an asset directly.
    ///
    /// ```
    /// use sand_components::{SoundEventDefinition, SoundEventId};
    /// let custom = SoundEventId::custom("example:registered_chime".parse().unwrap());
    /// let sound = SoundEventDefinition::reference(custom);
    /// ```
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::SoundEventDefinition::reference", module = "sand::component", summary = "Reference an existing sound-event registry entry.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::SoundEventDefinition;\nlet sound = SoundEventDefinition::reference(sand::registry::SoundEventId::custom(\"example:registered_chime\".parse().unwrap()));", params(event = "A generated vanilla sound or validated SoundEventId."), returns = "A definition serialized as a registry reference string.")]
    pub fn reference(event: impl RegistryReference<SoundEvent>) -> Self {
        Self(SoundEventKind::Reference(
            event
                .registry_id()
                .as_str()
                .parse()
                .expect("validated sound-event registry ID"),
        ))
    }

    /// Define a sound directly from its resource-pack identifier and optional fixed range.
    ///
    /// `None` uses Minecraft's volume-dependent range. `Some` serializes as the
    /// inline object's `range` field. Export rejects non-finite values; finite
    /// negative values and zero are accepted by Minecraft 26.2's sound codec.
    /// This is independent of the positive outer [`crate::Instrument::range`].
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::SoundEventDefinition::inline", module = "sand::component", summary = "Define a resource-pack sound with an optional fixed audible range.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::SoundEventDefinition;\nlet sound = SoundEventDefinition::inline(\"example:chime\".parse().unwrap(), Some(32.0));", params(sound_id = "The resource-pack sound identifier.", fixed_range = "None for volume-dependent range, or any finite f32 for fixed range."), returns = "An inline sound definition, validated when its resource is exported.")]
    pub fn inline(sound_id: ResourceLocation, fixed_range: Option<f32>) -> Self {
        Self(SoundEventKind::Inline {
            sound_id,
            fixed_range,
        })
    }

    /// Preserve an explicit raw reference or inline object, including unknown extension fields.
    ///
    /// Export still requires a valid reference string or an object with a valid
    /// `sound_id`. A present `range` must be a finite, representable f32 number;
    /// null, strings, and overflow are rejected. Unknown fields are retained
    /// verbatim, but their meaning and Minecraft support are the caller's responsibility.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::SoundEventDefinition::raw", module = "sand::component", summary = "Preserve raw sound JSON while validating known sound fields at export.", context = "Instrument and JukeboxSong share sound definitions and validate required resource fields before export.", minecraft = "Exports Minecraft instrument or jukebox_song JSON; inline sound range is independent of instrument range.", use_when = ["Authoring instrument or jukebox song resources"], avoid_when = ["Playing a sound immediately; use typed sound commands"], example = "use sand::component::SoundEventDefinition;\nlet sound = SoundEventDefinition::raw(sand::component::RawJson::new(serde_json::json!({\"sound_id\":\"example:chime\",\"extension\":true})));", params(value = "A reference string or inline object, optionally containing extension fields."), returns = "A raw sound definition retaining unknown fields.")]
    pub fn raw(value: RawJson) -> Self {
        Self(SoundEventKind::Raw(value))
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> Result<()> {
        match &self.0 {
            SoundEventKind::Reference(_) => Ok(()),
            SoundEventKind::Inline { fixed_range, .. } => {
                if let Some(range) = fixed_range {
                    validation::require_finite_f32(
                        location,
                        kind,
                        &format!("{field}.range"),
                        *range,
                    )?;
                }
                Ok(())
            }
            SoundEventKind::Raw(raw) => match raw.as_value() {
                Value::String(id) => {
                    validation::validate_resource_location_str(location, kind, field, id)
                }
                Value::Object(map) => {
                    let id_field = format!("{field}.sound_id");
                    let id = map.get("sound_id").and_then(Value::as_str).ok_or_else(|| {
                        validation::error(
                            location,
                            kind,
                            &id_field,
                            "sound_id is required and must be a resource-location string",
                        )
                    })?;
                    validation::validate_resource_location_str(location, kind, &id_field, id)?;
                    if let Some(range) = map.get("range") {
                        let range_field = format!("{field}.range");
                        if !range
                            .as_f64()
                            .is_some_and(|value| value.is_finite() && (value as f32).is_finite())
                        {
                            return Err(validation::error(
                                location,
                                kind,
                                &range_field,
                                "range must be a finite f32 number",
                            ));
                        }
                    }
                    Ok(())
                }
                _ => Err(validation::error(
                    location,
                    kind,
                    field,
                    "sound event must be a reference string or inline object",
                )),
            },
        }
    }

    pub(crate) fn to_json(&self) -> Value {
        match &self.0 {
            SoundEventKind::Reference(id) => json!(id),
            SoundEventKind::Inline {
                sound_id,
                fixed_range,
            } => {
                let mut value = json!({"sound_id": sound_id});
                if let Some(range) = fixed_range {
                    value["range"] = json!(range);
                }
                value
            }
            SoundEventKind::Raw(value) => value.as_value().clone(),
        }
    }
}
