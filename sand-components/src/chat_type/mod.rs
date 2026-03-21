//! Builder for `data/<namespace>/chat_type/` JSON files (Minecraft 1.21+).
//!
//! Chat types define how chat messages are decorated and displayed in-game.
//! Each chat type has a `chat` decoration (shown in chat) and an optional
//! `narration` decoration (used by screen readers / narrator).

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

// ── ChatDecoration ────────────────────────────────────────────────────────────

/// Controls how a chat message is decorated (wrapped with sender/target text).
///
/// The `translation_key` maps to a format string in the language file.
/// `parameters` lists the values substituted into the format string in order
/// (valid values: `"sender"`, `"target"`, `"content"`).
#[derive(Clone)]
pub struct ChatDecoration {
    /// The translation key for the format string.
    pub translation_key: String,
    /// Style overrides (`bold`, `italic`, `color`, etc.) as a raw JSON object.
    pub style: Option<Value>,
    /// Parameter list — ordered substitution into the translation format.
    pub parameters: Vec<String>,
}

impl ChatDecoration {
    /// Creates a new decoration with the given translation key.
    pub fn new(translation_key: impl Into<String>) -> Self {
        Self {
            translation_key: translation_key.into(),
            style: None,
            parameters: Vec::new(),
        }
    }

    /// Adds a parameter to the decoration (e.g. `"sender"`, `"content"`).
    pub fn parameter(mut self, param: impl Into<String>) -> Self {
        self.parameters.push(param.into());
        self
    }

    /// Sets multiple parameters at once.
    pub fn parameters(mut self, params: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.parameters = params.into_iter().map(|p| p.into()).collect();
        self
    }

    /// Sets a raw JSON style object (e.g. `{"color":"yellow","bold":true}`).
    pub fn style(mut self, style: Value) -> Self {
        self.style = Some(style);
        self
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "translation_key".to_string(),
            Value::String(self.translation_key.clone()),
        );
        map.insert(
            "parameters".to_string(),
            Value::Array(
                self.parameters
                    .iter()
                    .map(|p| Value::String(p.clone()))
                    .collect(),
            ),
        );
        if let Some(ref style) = self.style {
            map.insert("style".to_string(), style.clone());
        }
        Value::Object(map)
    }
}

// ── ChatType ──────────────────────────────────────────────────────────────────

/// A chat type definition (`data/<namespace>/chat_type/<id>.json`).
///
/// Chat types control how player and system messages appear in the chat box
/// and are read by the narrator.
pub struct ChatType {
    location: ResourceLocation,
    /// Decoration applied to messages shown in the chat HUD.
    chat: ChatDecoration,
    /// Decoration applied when the narrator reads the message aloud.
    narration: Option<ChatDecoration>,
}

impl ChatType {
    /// Creates a new chat type with the given resource location and chat decoration.
    pub fn new(location: ResourceLocation, chat: ChatDecoration) -> Self {
        Self {
            location,
            chat,
            narration: None,
        }
    }

    /// Sets the narration decoration (used by the narrator / screen readers).
    pub fn narration(mut self, narration: ChatDecoration) -> Self {
        self.narration = Some(narration);
        self
    }
}

impl DatapackComponent for ChatType {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("chat".to_string(), self.chat.to_json());
        if let Some(ref narration) = self.narration {
            map.insert("narration".to_string(), narration.to_json());
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "chat_type"
    }
}
