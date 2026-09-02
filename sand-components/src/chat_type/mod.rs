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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ChatDecorationParameter",
    aliases = ["sand::prelude::ChatDecorationParameter"],
    module = "sand::component",
    summary = "A parameter substituted into a chat decoration's translation format string.",
    context = "A parameter substituted into a chat decoration's translation format string. Vanilla only recognizes `sender`, `target`, and `content`. Use [`ChatDecorationParameter::Custom`] as an explicit, visually distinct escape hatch for future/unknown/modded parameter names — note that unknown custom values still fail [`ChatDecoration`] validation unless they match a known vanilla parameter, since Minecraft itself only understands the three above.",
    minecraft = "Vanilla only recognizes `sender`, `target`, and `content`. Use [`ChatDecorationParameter::Custom`] as an explicit, visually distinct escape hatch for future/unknown/modded parameter names — note that unknown custom values still fail [`ChatDecoration`] validation unless they match a known vanilla parameter, since Minecraft itself only understands the three above.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ChatDecorationParameter;",
    variants(Content = "The message content itself.", Custom = "Escape hatch for a raw/unknown parameter name.", Sender = "The message sender's display name.", Target = "The message target (used by e.g. `/msg`-style decorations)."),
    variant_fields(Custom = ["Escape hatch for a raw/unknown parameter name."]),
)]
/// A parameter substituted into a chat decoration's translation format string.
///
/// Vanilla only recognizes `sender`, `target`, and `content`. Use
/// [`ChatDecorationParameter::Custom`] as an explicit, visually distinct escape
/// hatch for future/unknown/modded parameter names — note that unknown custom
/// values still fail [`ChatDecoration`] validation unless they match a known
/// vanilla parameter, since Minecraft itself only understands the three above.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatDecorationParameter {
    /// The message sender's display name.
    Sender,
    /// The message target (used by e.g. `/msg`-style decorations).
    Target,
    /// The message content itself.
    Content,
    /// Escape hatch for a raw/unknown parameter name.
    Custom(#[doc = "Escape hatch for a raw/unknown parameter name."] String),
}

impl ChatDecorationParameter {
    /// The vanilla wire string for this parameter.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecorationParameter::as_str",
        aliases = ["sand::prelude::ChatDecorationParameter::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "The vanilla wire string for this parameter.",
        context = "The vanilla wire string for this parameter. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The string value produced to use the vanilla wire string for this parameter.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_decoration_parameter_value: &sand::component::ChatDecorationParameter)  {\n    let as_str = chat_decoration_parameter_value.as_str();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ChatStyle",
    aliases = ["sand::prelude::ChatStyle"],
    module = "sand::component",
    summary = "Typed style overrides for a [`ChatDecoration`]. Covers the common text-style fields accepted in chat type JSON without requiring callers to hand-write `serde_json::json!` objects. For style shapes this doesn't cover, use [`ChatDecoration::style_raw`].",
    context = "Typed style overrides for a [`ChatDecoration`]. Covers the common text-style fields accepted in chat type JSON without requiring callers to hand-write `serde_json::json!` objects. For style shapes this doesn't cover, use [`ChatDecoration::style_raw`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "Covers the common text-style fields accepted in chat type JSON without requiring callers to hand-write `serde_json::json!` objects. For style shapes this doesn't cover, use [`ChatDecoration::style_raw`].",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ChatStyle;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::new",
        aliases = ["sand::prelude::ChatStyle::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates an empty style with no overrides set.",
        context = "Creates an empty style with no overrides set. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "A newly constructed `ChatStyle` configured to create an empty style with no overrides set.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let chat_style = sand::component::ChatStyle::new();\n}",
    )]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a named vanilla text color.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::color",
        aliases = ["sand::prelude::ChatStyle::color"],
        module = "sand::component",
        kind = "method",
        summary = "Sets a named vanilla text color.",
        context = "Sets a named vanilla text color. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(color = "`color` provides the player-visible text value used to set a named vanilla text color."),
        returns = "The `ChatStyle` value with the documented change applied to set a named vanilla text color.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, color: sand::text::ChatColor)  {\n    let updated_chat_style = chat_style_value.color(color);\n}",
    )]
    pub fn color(mut self, color: sand_commands::ChatColor) -> Self {
        self.color = Some(ChatStyleColor::Named(color));
        self
    }

    /// Sets a raw `#RRGGBB` hex color (validated before export).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::color_hex",
        aliases = ["sand::prelude::ChatStyle::color_hex"],
        module = "sand::component",
        kind = "method",
        summary = "Sets a raw `#RRGGBB` hex color (validated before export).",
        context = "Sets a raw `#RRGGBB` hex color (validated before export). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(hex = "`hex` supplies the hex value used to set a raw `#RRGGBB` hex color (validated before export)."),
        returns = "The `ChatStyle` value with the documented change applied to set a raw `#RRGGBB` hex color (validated before export).",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, hex: impl Into < String >)  {\n    let updated_chat_style = chat_style_value.color_hex(hex);\n}",
    )]
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(ChatStyleColor::Hex(hex.into()));
        self
    }

    /// Sets whether the decorated text is bold.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::bold",
        aliases = ["sand::prelude::ChatStyle::bold"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the decorated text is bold.",
        context = "Sets whether the decorated text is bold. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set whether the decorated text is bold."),
        returns = "The `ChatStyle` value with the documented change applied to set whether the decorated text is bold.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, value: bool)  {\n    let updated_chat_style = chat_style_value.bold(value);\n}",
    )]
    pub fn bold(mut self, value: bool) -> Self {
        self.bold = Some(value);
        self
    }

    /// Sets whether the decorated text is italic.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::italic",
        aliases = ["sand::prelude::ChatStyle::italic"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the decorated text is italic.",
        context = "Sets whether the decorated text is italic. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set whether the decorated text is italic."),
        returns = "The `ChatStyle` value with the documented change applied to set whether the decorated text is italic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, value: bool)  {\n    let updated_chat_style = chat_style_value.italic(value);\n}",
    )]
    pub fn italic(mut self, value: bool) -> Self {
        self.italic = Some(value);
        self
    }

    /// Sets whether the decorated text is underlined.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::underlined",
        aliases = ["sand::prelude::ChatStyle::underlined"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the decorated text is underlined.",
        context = "Sets whether the decorated text is underlined. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set whether the decorated text is underlined."),
        returns = "The `ChatStyle` value with the documented change applied to set whether the decorated text is underlined.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, value: bool)  {\n    let updated_chat_style = chat_style_value.underlined(value);\n}",
    )]
    pub fn underlined(mut self, value: bool) -> Self {
        self.underlined = Some(value);
        self
    }

    /// Sets whether the decorated text is struck through.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::strikethrough",
        aliases = ["sand::prelude::ChatStyle::strikethrough"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the decorated text is struck through.",
        context = "Sets whether the decorated text is struck through. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set whether the decorated text is struck through."),
        returns = "The `ChatStyle` value with the documented change applied to set whether the decorated text is struck through.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, value: bool)  {\n    let updated_chat_style = chat_style_value.strikethrough(value);\n}",
    )]
    pub fn strikethrough(mut self, value: bool) -> Self {
        self.strikethrough = Some(value);
        self
    }

    /// Sets whether the decorated text is obfuscated (matrix effect).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::obfuscated",
        aliases = ["sand::prelude::ChatStyle::obfuscated"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether the decorated text is obfuscated (matrix effect).",
        context = "Sets whether the decorated text is obfuscated (matrix effect). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(value = "`value` provides the value being applied or compared used to set whether the decorated text is obfuscated (matrix effect)."),
        returns = "The `ChatStyle` value with the documented change applied to set whether the decorated text is obfuscated (matrix effect).",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, value: bool)  {\n    let updated_chat_style = chat_style_value.obfuscated(value);\n}",
    )]
    pub fn obfuscated(mut self, value: bool) -> Self {
        self.obfuscated = Some(value);
        self
    }

    /// Sets the shift-click chat insertion text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatStyle::insertion",
        aliases = ["sand::prelude::ChatStyle::insertion"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the shift-click chat insertion text.",
        context = "Sets the shift-click chat insertion text. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(text = "`text` provides the author-visible text value used to set the shift-click chat insertion text."),
        returns = "The `ChatStyle` value with the documented change applied to set the shift-click chat insertion text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_style_value: sand::component::ChatStyle, text: impl Into < String >)  {\n    let updated_chat_style = chat_style_value.insertion(text);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ChatDecoration",
    aliases = ["sand::prelude::ChatDecoration"],
    module = "sand::component",
    summary = "Controls how a chat message is decorated (wrapped with sender/target text).",
    context = "Controls how a chat message is decorated (wrapped with sender/target text). The `translation_key` maps to a format string in the language file. Parameters list the values substituted into the format string in order — author them with [`ChatDecorationParameter`] (typed) or plain strings (converted automatically; unknown strings become [`ChatDecorationParameter::Custom`] and fail validation, same as before).",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ChatDecoration;",
    fields(translation_key = "The translation key for the format string."),
)]
/// Controls how a chat message is decorated (wrapped with sender/target text).
///
/// The `translation_key` maps to a format string in the language file.
/// Parameters list the values substituted into the format string in order —
/// author them with [`ChatDecorationParameter`] (typed) or plain strings
/// (converted automatically; unknown strings become
/// [`ChatDecorationParameter::Custom`] and fail validation, same as before).
#[derive(Clone)]
pub struct ChatDecoration {
    /// The translation key for the format string.
    pub translation_key: String,
    style: Option<ChatDecorationStyle>,
    parameters: Vec<ChatDecorationParameter>,
}

impl ChatDecoration {
    /// Creates a new decoration with the given translation key.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecoration::new",
        aliases = ["sand::prelude::ChatDecoration::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new decoration with the given translation key.",
        context = "Creates a new decoration with the given translation key. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(translation_key = "`translation_key` supplies the translation key value used to create a new decoration with the given translation key."),
        returns = "A newly constructed `ChatDecoration` configured to create a new decoration with the given translation key.",
        example = "use sand::prelude::*;\n\nfn demonstrate(translation_key: impl Into < String >)  {\n    let chat_decoration = sand::component::ChatDecoration::new(translation_key);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecoration::parameter",
        aliases = ["sand::prelude::ChatDecoration::parameter"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a parameter to the decoration.",
        context = "Adds a parameter to the decoration. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(param = "`param` supplies the param value used to add a parameter to the decoration."),
        returns = "The `ChatDecoration` value with the documented change applied to add a parameter to the decoration.",
        example = "use {sand::component::ChatDecoration, sand::component::ChatDecorationParameter};\nlet deco = ChatDecoration::new(\"chat.type.text\").parameter(ChatDecorationParameter::Sender);",
    )]
    pub fn parameter(mut self, param: impl Into<ChatDecorationParameter>) -> Self {
        self.parameters.push(param.into());
        self
    }

    /// Sets multiple parameters at once.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecoration::parameters",
        aliases = ["sand::prelude::ChatDecoration::parameters"],
        module = "sand::component",
        kind = "method",
        summary = "Sets multiple parameters at once.",
        context = "Sets multiple parameters at once. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(params = "`params` supplies the params value used to set multiple parameters at once."),
        returns = "The `ChatDecoration` value with the documented change applied to set multiple parameters at once.",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_decoration_value: sand::component::ChatDecoration, params: impl IntoIterator < Item = impl Into < sand::component::ChatDecorationParameter > >)  {\n    let updated_chat_decoration = chat_decoration_value.parameters(params);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecoration::style",
        aliases = ["sand::prelude::ChatDecoration::style"],
        module = "sand::component",
        kind = "method",
        summary = "Sets typed style overrides (color, bold, italic, ...).",
        context = "Sets typed style overrides (color, bold, italic, ...). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(style = "`style` supplies the style value used to set typed style overrides (color, bold, italic, ...)."),
        returns = "The `ChatDecoration` value with the documented change applied to set typed style overrides (color, bold, italic, ...).",
        example = "use {sand::component::ChatDecoration, sand::component::ChatStyle};\nuse sand::text::ChatColor;\nlet deco = ChatDecoration::new(\"chat.type.text\")\n.style(ChatStyle::new().color(ChatColor::Yellow).bold(true));",
    )]
    pub fn style(mut self, style: ChatStyle) -> Self {
        self.style = Some(ChatDecorationStyle::Typed(style));
        self
    }

    /// Sets a raw JSON style object (e.g. `{"color":"yellow","bold":true}`).
    ///
    /// Escape hatch for style shapes [`ChatStyle`] doesn't cover. Prefer
    /// [`ChatDecoration::style`] for normal authoring.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatDecoration::style_raw",
        aliases = ["sand::prelude::ChatDecoration::style_raw"],
        module = "sand::component",
        kind = "method",
        summary = "Sets a raw JSON style object (e.g. `{\"color\":\"yellow\",\"bold\":true}`).",
        context = "Sets a raw JSON style object (e.g. `{\"color\":\"yellow\",\"bold\":true}`). Escape hatch for style shapes [`ChatStyle`] doesn't cover. Prefer [`ChatDecoration::style`] for normal authoring.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(style = "`style` supplies the style value used to set a raw JSON style object (e.g. `{\"color\":\"yellow\",\"bold\":true}`)."),
        returns = "The `ChatDecoration` value with the documented change applied to set a raw JSON style object (e.g. `{\"color\":\"yellow\",\"bold\":true}`).",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_decoration_value: sand::component::ChatDecoration, style: sand::component::RawJson)  {\n    let updated_chat_decoration = chat_decoration_value.style_raw(style);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::ChatType",
    aliases = ["sand::prelude::ChatType"],
    module = "sand::component",
    summary = "A chat type definition (`data/<namespace>/chat_type/<id>.json`).",
    context = "A chat type definition (`data/<namespace>/chat_type/<id>.json`). Chat types control how player and system messages appear in the chat box and are read by the narrator.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::ChatType;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatType::new",
        aliases = ["sand::prelude::ChatType::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new chat type with the given resource location and chat decoration.",
        context = "Creates a new chat type with the given resource location and chat decoration. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new chat type with the given resource location and chat decoration.", chat = "`chat` supplies the chat value used to create a new chat type with the given resource location and chat decoration."),
        returns = "A newly constructed `ChatType` configured to create a new chat type with the given resource location and chat decoration.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation, chat: sand::component::ChatDecoration)  {\n    let chat_type = sand::component::ChatType::new(location, chat);\n}",
    )]
    pub fn new(location: ResourceLocation, chat: ChatDecoration) -> Self {
        Self {
            location,
            chat,
            narration: None,
        }
    }

    /// Sets the narration decoration (used by the narrator / screen readers).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::ChatType::narration",
        aliases = ["sand::prelude::ChatType::narration"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the narration decoration (used by the narrator / screen readers).",
        context = "Sets the narration decoration (used by the narrator / screen readers). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(narration = "`narration` supplies the narration value used to set the narration decoration (used by the narrator / screen readers)."),
        returns = "The `ChatType` value with the documented change applied to set the narration decoration (used by the narrator / screen readers).",
        example = "use sand::prelude::*;\n\nfn demonstrate(chat_type_value: sand::component::ChatType, narration: sand::component::ChatDecoration)  {\n    let updated_chat_type = chat_type_value.narration(narration);\n}",
    )]
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
