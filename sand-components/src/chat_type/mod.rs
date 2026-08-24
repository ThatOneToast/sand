//! Builder for `data/<namespace>/chat_type/` JSON files (Minecraft 1.21+).
//!
//! Chat types define how chat messages are decorated and displayed in-game.
//! Each chat type has a `chat` decoration (shown in chat) and an optional
//! `narration` decoration (used by screen readers / narrator).
//!
//! # Typed authoring
//!
//! Normal decoration parameters are authored through [`ChatDecorationParameter`]
//! and normal style overrides through [`ChatStyle`] — both are validated before
//! export. [`RawJson`] style objects remain available through the explicit
//! [`ChatDecoration::style_raw`] escape hatch for shapes typed helpers don't
//! cover yet.
//!
//! ```
//! use sand_components::chat_type::{ChatDecoration, ChatDecorationParameter, ChatStyle, ChatType};
//! use sand_components::{DatapackComponent, ResourceLocation};
//! use sand_commands::ChatColor;
//!
//! let chat = ChatDecoration::new("chat.type.text")
//!     .parameters([
//!         ChatDecorationParameter::Sender,
//!         ChatDecorationParameter::Content,
//!     ])
//!     .style(ChatStyle::new().color(ChatColor::Gray).italic(true));
//!
//! let chat_type = ChatType::new(ResourceLocation::new("example", "system").unwrap(), chat);
//! assert!(chat_type.validate().is_ok());
//! ```

use std::fmt;

use serde_json::Value;

use crate::component::DatapackComponent;
use crate::error::SandError;
use crate::raw::RawJson;
use crate::resource_location::ResourceLocation;

// ── ChatDecorationParameter ────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter` for the canonical contract."]
/// A parameter substituted into a chat decoration's translation format string.
///
/// Vanilla only recognizes `sender`, `target`, and `content`. Use
/// [`ChatDecorationParameter::Custom`] as an explicit, visually distinct escape
/// hatch for future/unknown/modded parameter names — note that unknown custom
/// values still fail [`ChatDecoration`] validation unless they match a known
/// vanilla parameter, since Minecraft itself only understands the three above.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatDecorationParameter {
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::Sender` for the canonical contract."]
    /// The message sender's display name.
    Sender,
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::Target` for the canonical contract."]
    /// The message target (used by e.g. `/msg`-style decorations).
    Target,
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::Content` for the canonical contract."]
    /// The message content itself.
    Content,
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::Custom` for the canonical contract."]
    /// Escape hatch for a raw/unknown parameter name.
    Custom(
        #[doc = "The `Custom` variant carries the value described by its variant semantics: Escape hatch for a raw/unknown parameter name."]
        #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::Custom::0` for the canonical contract."]
        String,
    ),
}

impl ChatDecorationParameter {
    /// The vanilla wire string for this parameter.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecorationParameter::as_str` for the canonical contract."]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Sender => "sender",
            Self::Target => "target",
            Self::Content => "content",
            Self::Custom(s) => s,
        }
    }

    fn is_known(&self) -> bool {
        matches!(self, Self::Sender | Self::Target | Self::Content)
    }
}

impl fmt::Display for ChatDecorationParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ChatDecorationParameter {
    fn from(value: &str) -> Self {
        match value {
            "sender" => Self::Sender,
            "target" => Self::Target,
            "content" => Self::Content,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl From<String> for ChatDecorationParameter {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

// ── ChatStyle ───────────────────────────────────────────────────────────────────

/// A color for [`ChatStyle`] — either a typed [`sand_commands::ChatColor`] or an
/// explicit `#RRGGBB` hex escape hatch.
#[derive(Debug, Clone, PartialEq)]
enum ChatStyleColor {
    Named(sand_commands::ChatColor),
    Hex(String),
}

#[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle` for the canonical contract."]
/// Typed style overrides for a [`ChatDecoration`].
///
/// Covers the common text-style fields accepted in chat type JSON without
/// requiring callers to hand-write `serde_json::json!` objects. For style
/// shapes this doesn't cover, use [`ChatDecoration::style_raw`].
///
/// ```
/// use sand_components::chat_type::ChatStyle;
/// use sand_commands::ChatColor;
///
/// let style = ChatStyle::new().color(ChatColor::Yellow).bold(true);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatStyle {
    color: Option<ChatStyleColor>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    insertion: Option<String>,
}

impl ChatStyle {
    /// Creates an empty style with no overrides set.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::new` for the canonical contract."]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a named vanilla text color.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::color` for the canonical contract."]
    pub fn color(mut self, color: sand_commands::ChatColor) -> Self {
        self.color = Some(ChatStyleColor::Named(color));
        self
    }

    /// Sets a raw `#RRGGBB` hex color (validated before export).
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::color_hex` for the canonical contract."]
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(ChatStyleColor::Hex(hex.into()));
        self
    }

    /// Sets whether the decorated text is bold.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::bold` for the canonical contract."]
    pub fn bold(mut self, value: bool) -> Self {
        self.bold = Some(value);
        self
    }

    /// Sets whether the decorated text is italic.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::italic` for the canonical contract."]
    pub fn italic(mut self, value: bool) -> Self {
        self.italic = Some(value);
        self
    }

    /// Sets whether the decorated text is underlined.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::underlined` for the canonical contract."]
    pub fn underlined(mut self, value: bool) -> Self {
        self.underlined = Some(value);
        self
    }

    /// Sets whether the decorated text is struck through.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::strikethrough` for the canonical contract."]
    pub fn strikethrough(mut self, value: bool) -> Self {
        self.strikethrough = Some(value);
        self
    }

    /// Sets whether the decorated text is obfuscated (matrix effect).
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::obfuscated` for the canonical contract."]
    pub fn obfuscated(mut self, value: bool) -> Self {
        self.obfuscated = Some(value);
        self
    }

    /// Sets the shift-click chat insertion text.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatStyle::insertion` for the canonical contract."]
    pub fn insertion(mut self, text: impl Into<String>) -> Self {
        self.insertion = Some(text.into());
        self
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(color) = &self.color {
            let value = match color {
                ChatStyleColor::Named(c) => c.to_string(),
                ChatStyleColor::Hex(h) => h.clone(),
            };
            map.insert("color".to_string(), Value::String(value));
        }
        if let Some(v) = self.bold {
            map.insert("bold".to_string(), Value::Bool(v));
        }
        if let Some(v) = self.italic {
            map.insert("italic".to_string(), Value::Bool(v));
        }
        if let Some(v) = self.underlined {
            map.insert("underlined".to_string(), Value::Bool(v));
        }
        if let Some(v) = self.strikethrough {
            map.insert("strikethrough".to_string(), Value::Bool(v));
        }
        if let Some(v) = self.obfuscated {
            map.insert("obfuscated".to_string(), Value::Bool(v));
        }
        if let Some(ins) = &self.insertion {
            map.insert("insertion".to_string(), Value::String(ins.clone()));
        }
        Value::Object(map)
    }

    fn validate(&self, owner: &ResourceLocation, path: &str) -> crate::error::Result<()> {
        if let Some(ChatStyleColor::Hex(hex)) = &self.color
            && !is_valid_hex_color(hex)
        {
            return Err(SandError::ComponentValidation {
                location: owner.clone(),
                kind: "chat_type".to_string(),
                field: format!("{path}.color"),
                message: format!(
                    "error[SAND-TEXT-COLOR] invalid hex color `{hex}`: expected `#RRGGBB`"
                ),
            });
        }
        Ok(())
    }
}

fn is_valid_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Internal storage for a decoration's style — either the typed [`ChatStyle`]
/// normal path or the raw [`serde_json::Value`] escape hatch.
#[derive(Debug, Clone)]
enum ChatDecorationStyle {
    Typed(ChatStyle),
    Raw(Value),
}

// ── ChatDecoration ────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration` for the canonical contract."]
/// Controls how a chat message is decorated (wrapped with sender/target text).
///
/// The `translation_key` maps to a format string in the language file.
/// Parameters list the values substituted into the format string in order —
/// author them with [`ChatDecorationParameter`] (typed) or plain strings
/// (converted automatically; unknown strings become
/// [`ChatDecorationParameter::Custom`] and fail validation, same as before).
#[derive(Clone)]
pub struct ChatDecoration {
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::translation_key` for the canonical contract."]
    /// The translation key for the format string.
    pub translation_key: String,
    style: Option<ChatDecorationStyle>,
    parameters: Vec<ChatDecorationParameter>,
}

impl ChatDecoration {
    /// Creates a new decoration with the given translation key.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::new` for the canonical contract."]
    pub fn new(translation_key: impl Into<String>) -> Self {
        Self {
            translation_key: translation_key.into(),
            style: None,
            parameters: Vec::new(),
        }
    }

    /// Adds a parameter to the decoration.
    ///
    /// ```
    /// use sand_components::chat_type::{ChatDecoration, ChatDecorationParameter};
    /// let deco = ChatDecoration::new("chat.type.text").parameter(ChatDecorationParameter::Sender);
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::parameter` for the canonical contract."]
    pub fn parameter(mut self, param: impl Into<ChatDecorationParameter>) -> Self {
        self.parameters.push(param.into());
        self
    }

    /// Sets multiple parameters at once.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::parameters` for the canonical contract."]
    pub fn parameters(
        mut self,
        params: impl IntoIterator<Item = impl Into<ChatDecorationParameter>>,
    ) -> Self {
        self.parameters = params.into_iter().map(Into::into).collect();
        self
    }

    /// Sets typed style overrides (color, bold, italic, ...).
    ///
    /// ```
    /// use sand_components::chat_type::{ChatDecoration, ChatStyle};
    /// use sand_commands::ChatColor;
    ///
    /// let deco = ChatDecoration::new("chat.type.text")
    ///     .style(ChatStyle::new().color(ChatColor::Yellow).bold(true));
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::style` for the canonical contract."]
    pub fn style(mut self, style: ChatStyle) -> Self {
        self.style = Some(ChatDecorationStyle::Typed(style));
        self
    }

    /// Sets a raw JSON style object (e.g. `{"color":"yellow","bold":true}`).
    ///
    /// Escape hatch for style shapes [`ChatStyle`] doesn't cover. Prefer
    /// [`ChatDecoration::style`] for normal authoring.
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatDecoration::style_raw` for the canonical contract."]
    pub fn style_raw(mut self, style: RawJson) -> Self {
        self.style = Some(ChatDecorationStyle::Raw(style.into_value()));
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
                    .map(|p| Value::String(p.as_str().to_string()))
                    .collect(),
            ),
        );
        if let Some(style) = &self.style {
            let style_json = match style {
                ChatDecorationStyle::Typed(s) => s.to_json(),
                ChatDecorationStyle::Raw(v) => v.clone(),
            };
            map.insert("style".to_string(), style_json);
        }
        Value::Object(map)
    }

    fn validate(&self, owner: &ResourceLocation, path: &str) -> crate::error::Result<()> {
        if self.translation_key.trim().is_empty()
            || self.translation_key.chars().any(char::is_control)
        {
            return Err(SandError::ComponentValidation {
                location: owner.clone(),
                kind: "chat_type".to_string(),
                field: format!("{path}.translation_key"),
                message: "error[SAND-TEXT-TRANSLATE] translation keys must be non-empty and contain no control characters".to_string(),
            });
        }
        let mut seen = std::collections::HashSet::new();
        for (index, parameter) in self.parameters.iter().enumerate() {
            if !parameter.is_known() {
                return Err(SandError::ComponentValidation {
                    location: owner.clone(),
                    kind: "chat_type".to_string(),
                    field: format!("{path}.parameters[{index}]"),
                    message: format!(
                        "expected `sender`, `target`, or `content`, got `{parameter}`"
                    ),
                });
            }
            if !seen.insert(parameter.as_str()) {
                return Err(SandError::ComponentValidation {
                    location: owner.clone(),
                    kind: "chat_type".to_string(),
                    field: format!("{path}.parameters[{index}]"),
                    message: format!("duplicate parameter `{parameter}`"),
                });
            }
        }
        if let Some(style) = &self.style {
            match style {
                ChatDecorationStyle::Typed(s) => s.validate(owner, &format!("{path}.style"))?,
                ChatDecorationStyle::Raw(value) => {
                    if !value.is_object() {
                        return Err(SandError::ComponentValidation {
                            location: owner.clone(),
                            kind: "chat_type".to_string(),
                            field: format!("{path}.style"),
                            message: format!("style must be a JSON object, got `{value}`"),
                        });
                    }
                    sand_commands::text::validate_json_text(
                        value,
                        &sand_commands::CommandProfile::unprofiled(),
                        &format!("{path}.style"),
                    )
                    .map_err(|error| SandError::ComponentValidation {
                        location: owner.clone(),
                        kind: "chat_type".to_string(),
                        field: error.field,
                        message: format!("error[{}] {}", error.code, error.message),
                    })?;
                }
            }
        }
        Ok(())
    }
}

// ── ChatType ──────────────────────────────────────────────────────────────────

#[doc = "**API Contract:** Run `sand api show sand::component::ChatType` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatType::new` for the canonical contract."]
    pub fn new(location: ResourceLocation, chat: ChatDecoration) -> Self {
        Self {
            location,
            chat,
            narration: None,
        }
    }

    /// Sets the narration decoration (used by the narrator / screen readers).
    #[doc = "**API Contract:** Run `sand api show sand::component::ChatType::narration` for the canonical contract."]
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

    fn validate(&self) -> crate::error::Result<()> {
        self.chat.validate(&self.location, "chat")?;
        if let Some(narration) = &self.narration {
            narration.validate(&self.location, "narration")?;
        }
        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        "chat_type"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::ChatTypes]
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> ResourceLocation {
        ResourceLocation::new("test", "chat").unwrap()
    }

    #[test]
    fn typed_parameters_serialize_in_order() {
        let deco = ChatDecoration::new("chat.type.text").parameters([
            ChatDecorationParameter::Sender,
            ChatDecorationParameter::Content,
        ]);
        let json = deco.to_json();
        assert_eq!(json["parameters"], serde_json::json!(["sender", "content"]));
        assert!(deco.validate(&loc(), "chat").is_ok());
    }

    #[test]
    fn raw_string_parameters_still_accepted() {
        let deco = ChatDecoration::new("chat.type.text").parameters(["sender", "content"]);
        assert!(deco.validate(&loc(), "chat").is_ok());
        assert_eq!(
            deco.to_json()["parameters"],
            serde_json::json!(["sender", "content"])
        );
    }

    #[test]
    fn unknown_parameter_rejected() {
        let deco = ChatDecoration::new("chat.type.text").parameter("bogus");
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("bogus"), "{err}");
        assert!(err.contains("chat.parameters[0]"), "{err}");
    }

    #[test]
    fn custom_variant_rejected_same_as_unknown_string() {
        let deco = ChatDecoration::new("chat.type.text")
            .parameter(ChatDecorationParameter::Custom("modded_param".to_string()));
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("modded_param"), "{err}");
    }

    #[test]
    fn duplicate_parameter_rejected() {
        let deco = ChatDecoration::new("chat.type.text").parameters([
            ChatDecorationParameter::Sender,
            ChatDecorationParameter::Sender,
        ]);
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("duplicate"), "{err}");
        assert!(err.contains("chat.parameters[1]"), "{err}");
    }

    #[test]
    fn empty_translation_key_rejected() {
        let deco = ChatDecoration::new("   ");
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("SAND-TEXT-TRANSLATE"), "{err}");
    }

    #[test]
    fn control_character_translation_key_rejected() {
        let deco = ChatDecoration::new("chat.type\u{0007}.text");
        assert!(deco.validate(&loc(), "chat").is_err());
    }

    #[test]
    fn typed_style_serializes_expected_fields() {
        let style = ChatStyle::new()
            .color(sand_commands::ChatColor::Gold)
            .bold(true)
            .italic(false);
        let deco = ChatDecoration::new("chat.type.text").style(style);
        let json = deco.to_json();
        assert_eq!(json["style"]["color"], "gold");
        assert_eq!(json["style"]["bold"], true);
        assert_eq!(json["style"]["italic"], false);
        assert!(json["style"].get("underlined").is_none());
        assert!(deco.validate(&loc(), "chat").is_ok());
    }

    #[test]
    fn typed_style_hex_color_valid() {
        let style = ChatStyle::new().color_hex("#AABBCC");
        let deco = ChatDecoration::new("chat.type.text").style(style);
        assert!(deco.validate(&loc(), "chat").is_ok());
        assert_eq!(deco.to_json()["style"]["color"], "#AABBCC");
    }

    #[test]
    fn typed_style_invalid_hex_color_rejected() {
        let style = ChatStyle::new().color_hex("#ZZZZZZ");
        let deco = ChatDecoration::new("chat.type.text").style(style);
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("SAND-TEXT-COLOR"), "{err}");
    }

    #[test]
    fn style_raw_object_accepted() {
        let deco = ChatDecoration::new("chat.type.text").style_raw(RawJson::new(
            serde_json::json!({"color": "yellow", "bold": true}),
        ));
        assert!(deco.validate(&loc(), "chat").is_ok());
        assert_eq!(deco.to_json()["style"]["color"], "yellow");
    }

    #[test]
    fn style_raw_non_object_rejected() {
        let deco = ChatDecoration::new("chat.type.text")
            .style_raw(RawJson::new(serde_json::json!(["array"])));
        let err = deco.validate(&loc(), "chat").unwrap_err().to_string();
        assert!(err.contains("must be a JSON object"), "{err}");
    }

    #[test]
    fn style_raw_invalid_named_color_rejected() {
        let deco = ChatDecoration::new("chat.type.text")
            .style_raw(RawJson::new(serde_json::json!({"color": "nope"})));
        assert!(deco.validate(&loc(), "chat").is_err());
    }

    #[test]
    fn chat_type_valid_end_to_end() {
        let chat = ChatDecoration::new("chat.type.text")
            .parameters([
                ChatDecorationParameter::Sender,
                ChatDecorationParameter::Content,
            ])
            .style(ChatStyle::new().color(sand_commands::ChatColor::Gray));
        let narration = ChatDecoration::new("chat.type.text.narrate").parameters([
            ChatDecorationParameter::Sender,
            ChatDecorationParameter::Content,
        ]);
        let chat_type = ChatType::new(ResourceLocation::new("example", "system").unwrap(), chat)
            .narration(narration);
        assert!(chat_type.validate().is_ok());
    }

    #[test]
    fn chat_type_invalid_narration_reports_narration_path() {
        let chat = ChatDecoration::new("chat.type.text").parameter(ChatDecorationParameter::Sender);
        let narration = ChatDecoration::new("chat.type.text").parameter("bogus");
        let chat_type = ChatType::new(ResourceLocation::new("example", "system").unwrap(), chat)
            .narration(narration);
        let err = chat_type.validate().unwrap_err().to_string();
        assert!(err.contains("narration.parameters[0]"), "{err}");
        assert!(err.contains("example:system"), "{err}");
    }

    #[test]
    fn golden_chat_type_json() {
        let chat = ChatDecoration::new("chat.type.text")
            .parameters([
                ChatDecorationParameter::Sender,
                ChatDecorationParameter::Content,
            ])
            .style(
                ChatStyle::new()
                    .color(sand_commands::ChatColor::Gray)
                    .italic(true),
            );
        let chat_type = ChatType::new(ResourceLocation::new("example", "system").unwrap(), chat);
        let json = chat_type.to_json();
        assert_eq!(
            json,
            serde_json::json!({
                "chat": {
                    "translation_key": "chat.type.text",
                    "parameters": ["sender", "content"],
                    "style": {"color": "gray", "italic": true}
                }
            })
        );
        assert_eq!(chat_type.component_dir(), "chat_type");
        assert_eq!(
            chat_type.required_features(),
            &[sand_version::ComponentFeature::ChatTypes]
        );
    }
}
