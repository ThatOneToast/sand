//! Shared component-owned text, using the canonical command text validator.

use sand_commands::{CommandProfile, TextComponent};
use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::{RawJson, ResourceLocation, error::Result, validation};

#[derive(Debug, Clone)]
/// Typed and explicit raw text retained until the owning resource is validated.
pub(crate) enum ComponentText {
    Typed(Box<TextComponent>),
    Raw(RawJson),
}

impl ComponentText {
    /// Attach the owning resource and nested field path to canonical text errors.
    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> Result<()> {
        let result = match self {
            Self::Typed(text) => text.validate_at_path(&CommandProfile::unprofiled(), field),
            Self::Raw(text) => sand_commands::text::validate_json_text(
                text.as_value(),
                &CommandProfile::unprofiled(),
                field,
            ),
        };
        result.map_err(|error| {
            validation::error(
                location,
                kind,
                &error.field,
                &format!("error[{}] {}", error.code, error.message),
            )
        })
    }

    /// Render resource JSON after its export path has validated the text.
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Typed(text) => serde_json::from_str(&text.to_string())
                .expect("TextComponent always renders structurally valid JSON"),
            Self::Raw(text) => text.as_value().clone(),
        }
    }
}

impl Serialize for ComponentText {
    /// Serialize either validated authoring representation without changing its JSON shape.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Typed(text) => text
                .try_to_json_value()
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer),
            Self::Raw(text) => text.serialize(serializer),
        }
    }
}
