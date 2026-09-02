//! Minecraft JSON text component builder and chat color types.
//!
//! Minecraft uses a JSON-based format for all styled text: `tellraw`, `title`,
//! `bossbar`, item names, and more. This module provides a strongly-typed Rust
//! builder ([`TextComponent`]) that serializes to the correct JSON format and
//! the standard [`ChatColor`] palette that Minecraft exposes.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_commands::{TextComponent, ChatColor};
//!
//! // "Score: <score>" in two colors
//! let msg = TextComponent::literal("Score: ")
//!     .color(ChatColor::White)
//!     .then(TextComponent::score("@s", "kills").color(ChatColor::Red));
//!
//! // Emit as a tellraw command
//! let _cmd = format!("tellraw @a {msg}");
//! ```

use std::collections::BTreeMap;
use std::{fmt, str::FromStr};

use crate::Build;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;

// ── ChatColor ─────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::ChatColor",
    aliases = ["sand::cmd::ChatColor", "sand::command::ChatColor", "sand::prelude::ChatColor", "sand::prelude::cmd::ChatColor"],
    module = "sand::text",
    summary = "The 16 standard Minecraft text colors for chat, titles, and JSON text components.",
    context = "The 16 standard Minecraft text colors for chat, titles, and JSON text components. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::ChatColor;",
    variants(Aqua = "Selects the aqua Minecraft text behavior.", Black = "Selects the black Minecraft text behavior.", Blue = "Selects the blue Minecraft text behavior.", DarkAqua = "Selects the dark aqua Minecraft text behavior.", DarkBlue = "Selects the dark blue Minecraft text behavior.", DarkGray = "Selects the dark gray Minecraft text behavior.", DarkGreen = "Selects the dark green Minecraft text behavior.", DarkPurple = "Selects the dark purple Minecraft text behavior.", DarkRed = "Selects the dark red Minecraft text behavior.", Gold = "Selects the gold Minecraft text behavior.", Gray = "Selects the gray Minecraft text behavior.", Green = "Selects the green Minecraft text behavior.", LightPurple = "Selects the light purple Minecraft text behavior.", Red = "Selects the red Minecraft text behavior.", White = "Selects the white Minecraft text behavior.", Yellow = "Selects the yellow Minecraft text behavior."),
)]
/// The 16 standard Minecraft text colors for chat, titles, and JSON text components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatColor {
    #[doc = "Selects the black Minecraft text behavior."]
    Black,
    #[doc = "Selects the dark blue Minecraft text behavior."]
    DarkBlue,
    #[doc = "Selects the dark green Minecraft text behavior."]
    DarkGreen,
    #[doc = "Selects the dark aqua Minecraft text behavior."]
    DarkAqua,
    #[doc = "Selects the dark red Minecraft text behavior."]
    DarkRed,
    #[doc = "Selects the dark purple Minecraft text behavior."]
    DarkPurple,
    #[doc = "Selects the gold Minecraft text behavior."]
    Gold,
    #[doc = "Selects the gray Minecraft text behavior."]
    Gray,
    #[doc = "Selects the dark gray Minecraft text behavior."]
    DarkGray,
    #[doc = "Selects the blue Minecraft text behavior."]
    Blue,
    #[doc = "Selects the green Minecraft text behavior."]
    Green,
    #[doc = "Selects the aqua Minecraft text behavior."]
    Aqua,
    #[doc = "Selects the red Minecraft text behavior."]
    Red,
    #[doc = "Selects the light purple Minecraft text behavior."]
    LightPurple,
    #[doc = "Selects the yellow Minecraft text behavior."]
    Yellow,
    #[doc = "Selects the white Minecraft text behavior."]
    White,
}

impl fmt::Display for ChatColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChatColor::Black => "black",
            ChatColor::DarkBlue => "dark_blue",
            ChatColor::DarkGreen => "dark_green",
            ChatColor::DarkAqua => "dark_aqua",
            ChatColor::DarkRed => "dark_red",
            ChatColor::DarkPurple => "dark_purple",
            ChatColor::Gold => "gold",
            ChatColor::Gray => "gray",
            ChatColor::DarkGray => "dark_gray",
            ChatColor::Blue => "blue",
            ChatColor::Green => "green",
            ChatColor::Aqua => "aqua",
            ChatColor::Red => "red",
            ChatColor::LightPurple => "light_purple",
            ChatColor::Yellow => "yellow",
            ChatColor::White => "white",
        };
        write!(f, "{s}")
    }
}

// ── Click / Hover events ──────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::ClickEvent",
    aliases = ["sand::cmd::ClickEvent", "sand::command::ClickEvent", "sand::prelude::ClickEvent", "sand::prelude::cmd::ClickEvent"],
    module = "sand::text",
    summary = "A click event attached to a [`TextComponent`].",
    context = "A click event attached to a [`TextComponent`]. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::ClickEvent;",
    variants(ChangePage = "Emit `change_page` with a numeric page value (written books only).", CopyToClipboard = "Copy text to the clipboard.", OpenUrl = "Open a URL in the player's browser.", RunCommand = "Execute a command when clicked.", SuggestCommand = "Fill the chat bar with a command suggestion."),
    variant_fields(ChangePage = ["Emit `change_page` with a numeric page value (written books only)."], CopyToClipboard = ["Copy text to the clipboard."], OpenUrl = ["Open a URL in the player's browser."], RunCommand = ["Execute a command when clicked."], SuggestCommand = ["Fill the chat bar with a command suggestion."]),
)]
/// A click event attached to a [`TextComponent`].
#[derive(Debug, Clone)]
pub enum ClickEvent {
    /// Execute a command when clicked.
    RunCommand(#[doc = "Execute a command when clicked."] String),
    /// Fill the chat bar with a command suggestion.
    SuggestCommand(#[doc = "Fill the chat bar with a command suggestion."] String),
    /// Open a URL in the player's browser.
    OpenUrl(#[doc = "Open a URL in the player's browser."] String),
    /// Copy text to the clipboard.
    CopyToClipboard(#[doc = "Copy text to the clipboard."] String),
    /// Emit `change_page` with a numeric page value (written books only).
    ///
    /// The value is serialized unchanged. Minecraft book pages are normally
    /// one-indexed, but Sand retains page `0` for backward compatibility.
    ChangePage(#[doc = "Emit `change_page` with a numeric page value (written books only)."] u32),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::HoverEvent",
    aliases = ["sand::cmd::HoverEvent", "sand::command::HoverEvent", "sand::prelude::HoverEvent", "sand::prelude::cmd::HoverEvent"],
    module = "sand::text",
    summary = "A hover event attached to a [`TextComponent`].",
    context = "A hover event attached to a [`TextComponent`]. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::HoverEvent;",
    variants(ShowEntity = "Show a legacy entity tooltip using raw strings, including a plain name.", ShowItem = "Show an item tooltip using a raw item registry string and optional count.", ShowText = "Show another text component as a tooltip."),
    variant_fields(ShowEntity(entity_type = "`entity_type` provides the entity type when show a legacy entity tooltip using raw strings, including a plain name.", id = "`id` optionally provides the identifier when show a legacy entity tooltip using raw strings, including a plain name.", name = "`name` provides the name when show a legacy entity tooltip using raw strings, including a plain name."), ShowItem(count = "`count` optionally provides the count when show an item tooltip using a raw item registry string and optional count.", id = "`id` provides the identifier when show an item tooltip using a raw item registry string and optional count."), ShowText = ["Show another text component as a tooltip."]),
)]
/// A hover event attached to a [`TextComponent`].
#[derive(Debug, Clone)]
pub enum HoverEvent {
    /// Show another text component as a tooltip.
    ShowText(#[doc = "Show another text component as a tooltip."] Box<TextComponent>),
    /// Show an item tooltip using a raw item registry string and optional count.
    ///
    /// This existing compatibility representation does not validate the item
    /// ID. The count is omitted from JSON when it is [`None`].
    ShowItem {
        #[doc = "`id` provides the identifier when show an item tooltip using a raw item registry string and optional count."]
        id: String,
        #[doc = "`count` optionally provides the count when show an item tooltip using a raw item registry string and optional count."]
        count: Option<u32>,
    },
    /// Show a legacy entity tooltip using raw strings, including a plain name.
    ///
    /// New code should use [`TextComponent::hover_entity`] or
    /// [`TextComponent::hover_entity_with_id`] so the entity type is typed, the
    /// optional UUID is validated, and the name remains a styled component.
    ShowEntity {
        /// `name` provides the name when show a legacy entity tooltip using raw strings, including a plain name.
        name: String,
        /// `entity_type` provides the entity type when show a legacy entity tooltip using raw strings, including a plain name.
        entity_type: String,
        /// `id` optionally provides the identifier when show a legacy entity tooltip using raw strings, including a plain name.
        id: Option<String>,
    },
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::IntoTextEntityType",
    aliases = ["sand::cmd::IntoTextEntityType", "sand::command::IntoTextEntityType", "sand::prelude::IntoTextEntityType", "sand::prelude::cmd::IntoTextEntityType"],
    module = "sand::text",
    summary = "Conversion implemented by Sand's typed entity registry identifiers.",
    context = "Conversion implemented by Sand's typed entity registry identifiers. [`TextComponent::hover_entity`] accepts this trait instead of an arbitrary string. `EntityTypeId` validates manually constructed IDs, while Sand's profile-generated vanilla entity enum supplies built-in IDs when available. Use [`TextComponent::hover_entity_raw`] only when an untyped compatibility escape hatch is required.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::IntoTextEntityType;",
)]
/// Conversion implemented by Sand's typed entity registry identifiers.
///
/// [`TextComponent::hover_entity`] accepts this trait instead of an arbitrary
/// string. `EntityTypeId` validates manually constructed IDs, while Sand's
/// profile-generated vanilla entity enum supplies built-in IDs when available. Use
/// [`TextComponent::hover_entity_raw`] only when an untyped compatibility
/// escape hatch is required.
pub trait IntoTextEntityType {
    /// Convert the validated entity registry identifier to its resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::IntoTextEntityType::into_text_entity_type",
        aliases = ["sand::cmd::IntoTextEntityType::into_text_entity_type", "sand::command::IntoTextEntityType::into_text_entity_type", "sand::prelude::IntoTextEntityType::into_text_entity_type", "sand::prelude::cmd::IntoTextEntityType::into_text_entity_type"],
        module = "sand::text",
        summary = "Convert the validated entity registry identifier to its resource location.",
        context = "Convert the validated entity registry identifier to its resource location. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The string value produced to convert the validated entity registry identifier to its resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::text::IntoTextEntityType>(into_text_entity_type_value: T)  {\n    let into_text_entity_type = into_text_entity_type_value.into_text_entity_type();\n}",
    )]
    fn into_text_entity_type(self) -> String;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::EntityHoverId",
    aliases = ["sand::cmd::EntityHoverId", "sand::command::EntityHoverId", "sand::prelude::EntityHoverId", "sand::prelude::cmd::EntityHoverId"],
    module = "sand::text",
    summary = "A validated UUID for a Minecraft `show_entity` hover tooltip.",
    context = "A validated UUID for a Minecraft `show_entity` hover tooltip. Parsing is fallible and accepts only canonical hyphenated UUID text. It never panics for user input.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::EntityHoverId;",
)]
/// A validated UUID for a Minecraft `show_entity` hover tooltip.
///
/// Parsing is fallible and accepts only canonical hyphenated UUID text. It
/// never panics for user input.
///
/// ```
/// use sand_commands::EntityHoverId;
///
/// let id = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000")?;
/// assert_eq!(id.to_string(), "123e4567-e89b-12d3-a456-426614174000");
/// # Ok::<(), sand_commands::CommandError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityHoverId(String);

impl EntityHoverId {
    /// Parse a canonical hyphenated UUID (`8-4-4-4-12` hexadecimal digits).
    ///
    /// Returns a [`CommandError`] naming the `id` field when the input has the
    /// wrong length, hyphen placement, or contains non-hexadecimal digits.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::EntityHoverId::parse",
        aliases = ["sand::cmd::EntityHoverId::parse", "sand::command::EntityHoverId::parse", "sand::prelude::EntityHoverId::parse", "sand::prelude::cmd::EntityHoverId::parse"],
        module = "sand::text",
        kind = "method",
        summary = "Parse a canonical hyphenated UUID (`8-4-4-4-12` hexadecimal digits).",
        context = "Parse a canonical hyphenated UUID (`8-4-4-4-12` hexadecimal digits). Returns a [`CommandError`] naming the `id` field when the input has the wrong length, hyphen placement, or contains non-hexadecimal digits.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(value = "`value` provides the value being applied or compared used to parse a canonical hyphenated UUID (`8-4-4-4-12` hexadecimal digits)."),
        returns = "Returns a [`CommandError`] naming the `id` field when the input has the wrong length, hyphen placement, or contains non-hexadecimal digits.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: impl Into < String >)  {\n    let entity_hover_id_result = sand::text::EntityHoverId::parse(value);\n}",
    )]
    pub fn parse(value: impl Into<String>) -> CommandResult<Self> {
        let value = value.into();
        let valid = value.len() == 36
            && value.bytes().enumerate().all(|(index, byte)| match index {
                8 | 13 | 18 | 23 => byte == b'-',
                _ => byte.is_ascii_hexdigit(),
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(CommandError::new(
                "EntityHoverId::parse",
                "id",
                format!("must be a canonical hyphenated UUID, got `{value}`"),
            ))
        }
    }
}

impl fmt::Display for EntityHoverId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for EntityHoverId {
    type Err = CommandError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

// ── TextComponent internals ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum TextContent {
    Literal(String),
    Score {
        name: String,
        objective: String,
    },
    Selector {
        value: String,
        raw: bool,
    },
    Translate {
        key: String,
        with: Vec<TextComponent>,
    },
    Keybind(String),
}

#[derive(Debug, Clone)]
enum TextHoverEvent {
    Public(HoverEvent),
    RawShowItem {
        id: String,
        count: Option<u32>,
    },
    ShowEntityText {
        name: Box<TextComponent>,
        entity_type: String,
        id: Option<String>,
        raw: bool,
    },
}

// ── TextComponent ─────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::TextComponent",
    aliases = ["sand::cmd::TextComponent", "sand::command::TextComponent", "sand::prelude::TextComponent", "sand::prelude::cmd::TextComponent"],
    module = "sand::text",
    summary = "A Minecraft JSON text component — the universal format for styled in-game text.",
    context = "A Minecraft JSON text component — the universal format for styled in-game text. Used by commands like `tellraw`, `title`, and `bossbar` to display richly formatted messages. Build with a factory method, chain formatting and extra segments, then convert to JSON via `Display` / `.to_string()`.",
    minecraft = "Used by commands like `tellraw`, `title`, and `bossbar` to display richly formatted messages. Build with a factory method, chain formatting and extra segments, then convert to JSON via `Display` / `.to_string()`.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::TextComponent;",
)]
/// A Minecraft JSON text component — the universal format for styled in-game text.
///
/// Used by commands like `tellraw`, `title`, and `bossbar` to display richly
/// formatted messages. Build with a factory method, chain formatting and extra
/// segments, then convert to JSON via `Display` / `.to_string()`.
///
/// # Examples
///
/// ```
/// use sand_commands::{TextComponent, ChatColor};
///
/// let t = TextComponent::literal("Hello!")
///     .color(ChatColor::Gold)
///     .bold(true);
/// assert!(t.to_string().contains("\"text\":\"Hello!\""));
/// ```
#[derive(Debug, Clone)]
pub struct TextComponent {
    content: TextContent,
    color: Option<String>,
    font: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    insertion: Option<String>,
    click_event: Option<ClickEvent>,
    hover_event: Option<TextHoverEvent>,
    extra: Vec<TextComponent>,
}

// ── Text (ergonomic alias) ────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::text::Text",
    aliases = ["sand::cmd::Text", "sand::command::Text", "sand::prelude::Text", "sand::prelude::cmd::Text"],
    module = "sand::text",
    summary = "Ergonomic alias — `Text::new(\"hi\")` creates a `TextComponent::literal(\"hi\")`.",
    context = "Ergonomic alias — `Text::new(\"hi\")` creates a `TextComponent::literal(\"hi\")`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
    minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
    use_when = ["Building player-visible text with typed styling or interactions"],
    avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
    example = "use sand::text::Text;",
)]
/// Ergonomic alias — `Text::new("hi")` creates a `TextComponent::literal("hi")`.
///
/// ```
/// use sand_commands::Text;
/// let t = Text::new("Hello").gold().bold(true);
/// assert!(t.to_string().contains("\"color\":\"gold\""));
/// ```
pub struct Text;

impl Text {
    /// Create a plain-text component from `s`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::Text::new",
        aliases = ["sand::cmd::Text::new", "sand::command::Text::new", "sand::prelude::Text::new", "sand::prelude::cmd::Text::new"],
        module = "sand::text",
        kind = "method",
        summary = "Create a plain-text component from `s`.",
        context = "Create a plain-text component from `s`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(s = "Create a plain-text component from `s`."),
        returns = "The `TextComponent` value produced to create a plain-text component from `s`.",
        example = "let text = sand::text::Text::new(\"Ready\").gold();",
    )]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(s: impl Into<String>) -> TextComponent {
        TextComponent::literal(s)
    }

    /// Embed a pre-serialized JSON string directly (escape hatch).
    ///
    /// No formatting is applied — the string is returned as-is.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::Text::raw_json",
        aliases = ["sand::cmd::Text::raw_json", "sand::command::Text::raw_json", "sand::prelude::Text::raw_json", "sand::prelude::cmd::Text::raw_json"],
        module = "sand::text",
        kind = "method",
        summary = "Embed a pre-serialized JSON string directly (escape hatch).",
        context = "Embed a pre-serialized JSON string directly (escape hatch). No formatting is applied — the string is returned as-is.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(json = "`json` provides the raw JSON payload used to embed a pre-serialized JSON string directly (escape hatch)."),
        returns = "The string value produced to embed a pre-serialized JSON string directly (escape hatch).",
        example = "use sand::prelude::*;\n\nfn demonstrate(json: impl Into < String >)  {\n    let raw_json = sand::text::Text::raw_json(json);\n}",
    )]
    pub fn raw_json(json: impl Into<String>) -> String {
        json.into()
    }
}

impl TextComponent {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// `{"text": "..."}` — render a plain string literal.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::literal",
        aliases = ["sand::cmd::TextComponent::literal", "sand::command::TextComponent::literal", "sand::prelude::TextComponent::literal", "sand::prelude::cmd::TextComponent::literal"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"text\": \"...\"}` — render a plain string literal.",
        context = "`{\"text\": \"...\"}` — render a plain string literal. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(text = "`text` provides the author-visible text value used to emit the documented `{\"text\": \"...\"}` — render a plain string literal form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"text\": \"...\"}` — render a plain string literal form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::literal(text);\n}",
    )]
    pub fn literal(text: impl Into<String>) -> Self {
        Self::new(TextContent::Literal(text.into()))
    }

    /// `{"score": {"name": "...", "objective": "..."}}` — render a scoreboard value inline.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::score",
        aliases = ["sand::cmd::TextComponent::score", "sand::command::TextComponent::score", "sand::prelude::TextComponent::score", "sand::prelude::cmd::TextComponent::score"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"score\": {\"name\": \"...\", \"objective\": \"...\"}}` — render a scoreboard value inline.",
        context = "`{\"score\": {\"name\": \"...\", \"objective\": \"...\"}}` — render a scoreboard value inline. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(name = "`name` provides the author-visible text value used to emit the documented `{\"score\": {\"name\": \"...\", \"objective\": \"...\"}}` — render a scoreboard value inline form.", objective = "`objective` supplies the objective value used to emit the documented `{\"score\": {\"name\": \"...\", \"objective\": \"...\"}}` — render a scoreboard value inline form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"score\": {\"name\": \"...\", \"objective\": \"...\"}}` — render a scoreboard value inline form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >, objective: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::score(name, objective);\n}",
    )]
    pub fn score(name: impl Into<String>, objective: impl Into<String>) -> Self {
        Self::new(TextContent::Score {
            name: name.into(),
            objective: objective.into(),
        })
    }

    /// `{"selector": "..."}` — render the display name(s) of matched entities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::selector",
        aliases = ["sand::cmd::TextComponent::selector", "sand::command::TextComponent::selector", "sand::prelude::TextComponent::selector", "sand::prelude::cmd::TextComponent::selector"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"selector\": \"...\"}` — render the display name(s) of matched entities.",
        context = "`{\"selector\": \"...\"}` — render the display name(s) of matched entities. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(selector = "`selector` provides the Minecraft target selection used to emit the documented `{\"selector\": \"...\"}` — render the display name(s) of matched entities form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"selector\": \"...\"}` — render the display name(s) of matched entities form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::selector(selector);\n}",
    )]
    pub fn selector(selector: impl Into<String>) -> Self {
        Self::new(TextContent::Selector {
            value: selector.into(),
            raw: false,
        })
    }

    /// Create selector text from Sand's canonical typed selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::selector_typed",
        aliases = ["sand::cmd::TextComponent::selector_typed", "sand::command::TextComponent::selector_typed", "sand::prelude::TextComponent::selector_typed", "sand::prelude::cmd::TextComponent::selector_typed"],
        module = "sand::text",
        kind = "method",
        summary = "Create selector text from Sand's canonical typed selector.",
        context = "Create selector text from Sand's canonical typed selector. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(selector = "`selector` provides the Minecraft target selection used to create selector text from Sand's canonical typed selector."),
        returns = "A newly constructed `TextComponent` configured to create selector text from Sand's canonical typed selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Selector)  {\n    let text_component = sand::text::TextComponent::selector_typed(selector);\n}",
    )]
    pub fn selector_typed(selector: Selector) -> Self {
        Self::selector(selector.to_string())
    }

    /// Create intentionally opaque selector text.
    ///
    /// The value is rendered unchanged and selector compatibility is user-owned.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::selector_raw",
        aliases = ["sand::cmd::TextComponent::selector_raw", "sand::command::TextComponent::selector_raw", "sand::prelude::TextComponent::selector_raw", "sand::prelude::cmd::TextComponent::selector_raw"],
        module = "sand::text",
        kind = "method",
        summary = "Create intentionally opaque selector text. The value is rendered unchanged and selector compatibility is user-owned.",
        context = "Create intentionally opaque selector text. The value is rendered unchanged and selector compatibility is user-owned. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(selector = "`selector` provides the Minecraft target selection used to create intentionally opaque selector text. The value is rendered unchanged and selector compatibility is user-owned."),
        returns = "A newly constructed `TextComponent` configured to create intentionally opaque selector text. The value is rendered unchanged and selector compatibility is user-owned.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::selector_raw(selector);\n}",
    )]
    pub fn selector_raw(selector: impl Into<String>) -> Self {
        Self::new(TextContent::Selector {
            value: selector.into(),
            raw: true,
        })
    }

    /// `{"translate": "..."}` — a localization key from Minecraft's language files.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::translate",
        aliases = ["sand::cmd::TextComponent::translate", "sand::command::TextComponent::translate", "sand::prelude::TextComponent::translate", "sand::prelude::cmd::TextComponent::translate"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"translate\": \"...\"}` — a localization key from Minecraft's language files.",
        context = "`{\"translate\": \"...\"}` — a localization key from Minecraft's language files. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(key = "`key` provides the key that identifies the setting or entry used to emit the documented `{\"translate\": \"...\"}` — a localization key from Minecraft's language files form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"translate\": \"...\"}` — a localization key from Minecraft's language files form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(key: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::translate(key);\n}",
    )]
    pub fn translate(key: impl Into<String>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with: vec![],
        })
    }

    /// `{"translate": "...", "with": [...]}` — localization key with interpolation arguments.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::translate_with",
        aliases = ["sand::cmd::TextComponent::translate_with", "sand::command::TextComponent::translate_with", "sand::prelude::TextComponent::translate_with", "sand::prelude::cmd::TextComponent::translate_with"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"translate\": \"...\", \"with\": [...]}` — localization key with interpolation arguments.",
        context = "`{\"translate\": \"...\", \"with\": [...]}` — localization key with interpolation arguments. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(key = "`key` provides the key that identifies the setting or entry used to emit the documented `{\"translate\": \"...\", \"with\": [...]}` — localization key with interpolation arguments form.", with = "`with` provides the player-visible text value used to emit the documented `{\"translate\": \"...\", \"with\": [...]}` — localization key with interpolation arguments form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"translate\": \"...\", \"with\": [...]}` — localization key with interpolation arguments form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(key: impl Into < String >, with: Vec < sand::text::TextComponent >)  {\n    let text_component = sand::text::TextComponent::translate_with(key, with);\n}",
    )]
    pub fn translate_with(key: impl Into<String>, with: Vec<TextComponent>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with,
        })
    }

    /// `{"keybind": "..."}` — display the key currently bound to a Minecraft action.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::keybind",
        aliases = ["sand::cmd::TextComponent::keybind", "sand::command::TextComponent::keybind", "sand::prelude::TextComponent::keybind", "sand::prelude::cmd::TextComponent::keybind"],
        module = "sand::text",
        kind = "method",
        summary = "`{\"keybind\": \"...\"}` — display the key currently bound to a Minecraft action.",
        context = "`{\"keybind\": \"...\"}` — display the key currently bound to a Minecraft action. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(key = "`key` provides the key that identifies the setting or entry used to emit the documented `{\"keybind\": \"...\"}` — display the key currently bound to a Minecraft action form."),
        returns = "A newly constructed `TextComponent` configured to emit the documented `{\"keybind\": \"...\"}` — display the key currently bound to a Minecraft action form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(key: impl Into < String >)  {\n    let text_component = sand::text::TextComponent::keybind(key);\n}",
    )]
    pub fn keybind(key: impl Into<String>) -> Self {
        Self::new(TextContent::Keybind(key.into()))
    }

    fn new(content: TextContent) -> Self {
        Self {
            content,
            color: None,
            font: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            insertion: None,
            click_event: None,
            hover_event: None,
            extra: vec![],
        }
    }

    // ── Formatting ────────────────────────────────────────────────────────────

    /// Apply a standard Minecraft named color.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::color",
        aliases = ["sand::cmd::TextComponent::color", "sand::command::TextComponent::color", "sand::prelude::TextComponent::color", "sand::prelude::cmd::TextComponent::color"],
        module = "sand::text",
        kind = "method",
        summary = "Apply a standard Minecraft named color.",
        context = "Apply a standard Minecraft named color. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(color = "`color` supplies the color value used to apply a standard Minecraft named color."),
        returns = "The `TextComponent` value with the documented change applied to apply a standard Minecraft named color.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, color: sand::text::ChatColor)  {\n    let updated_text_component = text_component_value.color(color);\n}",
    )]
    pub fn color(mut self, color: ChatColor) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Apply an arbitrary hex color code (Minecraft 1.16+), e.g. `"#FF5733"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::color_hex",
        aliases = ["sand::cmd::TextComponent::color_hex", "sand::command::TextComponent::color_hex", "sand::prelude::TextComponent::color_hex", "sand::prelude::cmd::TextComponent::color_hex"],
        module = "sand::text",
        kind = "method",
        summary = "Apply an arbitrary hex color code (Minecraft 1.16+), e.g. `\"#FF5733\"`.",
        context = "Apply an arbitrary hex color code (Minecraft 1.16+), e.g. `\"#FF5733\"`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(hex = "`hex` supplies the hex value used to apply an arbitrary hex color code (Minecraft 1.16+), e.g. `\"#FF5733\"`."),
        returns = "The `TextComponent` value with the documented change applied to apply an arbitrary hex color code (Minecraft 1.16+), e.g. `\"#FF5733\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, hex: impl Into < String >)  {\n    let updated_text_component = text_component_value.color_hex(hex);\n}",
    )]
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(hex.into());
        self
    }

    /// Set the font resource location used to render this component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::font",
        aliases = ["sand::cmd::TextComponent::font", "sand::command::TextComponent::font", "sand::prelude::TextComponent::font", "sand::prelude::cmd::TextComponent::font"],
        module = "sand::text",
        kind = "method",
        summary = "Set the font resource location used to render this component.",
        context = "Set the font resource location used to render this component. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(font = "`font` supplies the font value used to set the font resource location used to render this component."),
        returns = "The `TextComponent` value with the documented change applied to set the font resource location used to render this component.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, font: impl Into < String >)  {\n    let updated_text_component = text_component_value.font(font);\n}",
    )]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = Some(font.into());
        self
    }

    // ── Ergonomic color shortcuts ─────────────────────────────────────────────

    /// Apply `ChatColor::Black`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::black",
        aliases = ["sand::cmd::TextComponent::black", "sand::command::TextComponent::black", "sand::prelude::TextComponent::black", "sand::prelude::cmd::TextComponent::black"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Black`.",
        context = "Apply `ChatColor::Black`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Black`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.black();\n}",
    )]
    pub fn black(self) -> Self {
        self.color(ChatColor::Black)
    }
    /// Apply `ChatColor::DarkBlue`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_blue",
        aliases = ["sand::cmd::TextComponent::dark_blue", "sand::command::TextComponent::dark_blue", "sand::prelude::TextComponent::dark_blue", "sand::prelude::cmd::TextComponent::dark_blue"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkBlue`.",
        context = "Apply `ChatColor::DarkBlue`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkBlue`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_blue();\n}",
    )]
    pub fn dark_blue(self) -> Self {
        self.color(ChatColor::DarkBlue)
    }
    /// Apply `ChatColor::DarkGreen`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_green",
        aliases = ["sand::cmd::TextComponent::dark_green", "sand::command::TextComponent::dark_green", "sand::prelude::TextComponent::dark_green", "sand::prelude::cmd::TextComponent::dark_green"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkGreen`.",
        context = "Apply `ChatColor::DarkGreen`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkGreen`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_green();\n}",
    )]
    pub fn dark_green(self) -> Self {
        self.color(ChatColor::DarkGreen)
    }
    /// Apply `ChatColor::DarkAqua`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_aqua",
        aliases = ["sand::cmd::TextComponent::dark_aqua", "sand::command::TextComponent::dark_aqua", "sand::prelude::TextComponent::dark_aqua", "sand::prelude::cmd::TextComponent::dark_aqua"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkAqua`.",
        context = "Apply `ChatColor::DarkAqua`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkAqua`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_aqua();\n}",
    )]
    pub fn dark_aqua(self) -> Self {
        self.color(ChatColor::DarkAqua)
    }
    /// Apply `ChatColor::DarkRed`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_red",
        aliases = ["sand::cmd::TextComponent::dark_red", "sand::command::TextComponent::dark_red", "sand::prelude::TextComponent::dark_red", "sand::prelude::cmd::TextComponent::dark_red"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkRed`.",
        context = "Apply `ChatColor::DarkRed`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkRed`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_red();\n}",
    )]
    pub fn dark_red(self) -> Self {
        self.color(ChatColor::DarkRed)
    }
    /// Apply `ChatColor::DarkPurple`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_purple",
        aliases = ["sand::cmd::TextComponent::dark_purple", "sand::command::TextComponent::dark_purple", "sand::prelude::TextComponent::dark_purple", "sand::prelude::cmd::TextComponent::dark_purple"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkPurple`.",
        context = "Apply `ChatColor::DarkPurple`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkPurple`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_purple();\n}",
    )]
    pub fn dark_purple(self) -> Self {
        self.color(ChatColor::DarkPurple)
    }
    /// Apply `ChatColor::Gold`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::gold",
        aliases = ["sand::cmd::TextComponent::gold", "sand::command::TextComponent::gold", "sand::prelude::TextComponent::gold", "sand::prelude::cmd::TextComponent::gold"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Gold`.",
        context = "Apply `ChatColor::Gold`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Gold`.",
        example = "let text = sand::text::Text::new(\"Ready\").gold();",
    )]
    pub fn gold(self) -> Self {
        self.color(ChatColor::Gold)
    }
    /// Apply `ChatColor::Gray`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::gray",
        aliases = ["sand::cmd::TextComponent::gray", "sand::command::TextComponent::gray", "sand::prelude::TextComponent::gray", "sand::prelude::cmd::TextComponent::gray"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Gray`.",
        context = "Apply `ChatColor::Gray`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Gray`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.gray();\n}",
    )]
    pub fn gray(self) -> Self {
        self.color(ChatColor::Gray)
    }
    /// Apply `ChatColor::DarkGray`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::dark_gray",
        aliases = ["sand::cmd::TextComponent::dark_gray", "sand::command::TextComponent::dark_gray", "sand::prelude::TextComponent::dark_gray", "sand::prelude::cmd::TextComponent::dark_gray"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::DarkGray`.",
        context = "Apply `ChatColor::DarkGray`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::DarkGray`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.dark_gray();\n}",
    )]
    pub fn dark_gray(self) -> Self {
        self.color(ChatColor::DarkGray)
    }
    /// Apply `ChatColor::Blue`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::blue",
        aliases = ["sand::cmd::TextComponent::blue", "sand::command::TextComponent::blue", "sand::prelude::TextComponent::blue", "sand::prelude::cmd::TextComponent::blue"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Blue`.",
        context = "Apply `ChatColor::Blue`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Blue`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.blue();\n}",
    )]
    pub fn blue(self) -> Self {
        self.color(ChatColor::Blue)
    }
    /// Apply `ChatColor::Green`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::green",
        aliases = ["sand::cmd::TextComponent::green", "sand::command::TextComponent::green", "sand::prelude::TextComponent::green", "sand::prelude::cmd::TextComponent::green"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Green`.",
        context = "Apply `ChatColor::Green`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Green`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.green();\n}",
    )]
    pub fn green(self) -> Self {
        self.color(ChatColor::Green)
    }
    /// Apply `ChatColor::Aqua`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::aqua",
        aliases = ["sand::cmd::TextComponent::aqua", "sand::command::TextComponent::aqua", "sand::prelude::TextComponent::aqua", "sand::prelude::cmd::TextComponent::aqua"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Aqua`.",
        context = "Apply `ChatColor::Aqua`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Aqua`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.aqua();\n}",
    )]
    pub fn aqua(self) -> Self {
        self.color(ChatColor::Aqua)
    }
    /// Apply `ChatColor::Red`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::red",
        aliases = ["sand::cmd::TextComponent::red", "sand::command::TextComponent::red", "sand::prelude::TextComponent::red", "sand::prelude::cmd::TextComponent::red"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Red`.",
        context = "Apply `ChatColor::Red`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Red`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.red();\n}",
    )]
    pub fn red(self) -> Self {
        self.color(ChatColor::Red)
    }
    /// Apply `ChatColor::LightPurple`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::light_purple",
        aliases = ["sand::cmd::TextComponent::light_purple", "sand::command::TextComponent::light_purple", "sand::prelude::TextComponent::light_purple", "sand::prelude::cmd::TextComponent::light_purple"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::LightPurple`.",
        context = "Apply `ChatColor::LightPurple`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::LightPurple`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.light_purple();\n}",
    )]
    pub fn light_purple(self) -> Self {
        self.color(ChatColor::LightPurple)
    }
    /// Apply `ChatColor::Yellow`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::yellow",
        aliases = ["sand::cmd::TextComponent::yellow", "sand::command::TextComponent::yellow", "sand::prelude::TextComponent::yellow", "sand::prelude::cmd::TextComponent::yellow"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::Yellow`.",
        context = "Apply `ChatColor::Yellow`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::Yellow`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.yellow();\n}",
    )]
    pub fn yellow(self) -> Self {
        self.color(ChatColor::Yellow)
    }
    /// Apply `ChatColor::White`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::white",
        aliases = ["sand::cmd::TextComponent::white", "sand::command::TextComponent::white", "sand::prelude::TextComponent::white", "sand::prelude::cmd::TextComponent::white"],
        module = "sand::text",
        kind = "method",
        summary = "Apply `ChatColor::White`.",
        context = "Apply `ChatColor::White`. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "The `TextComponent` value with the documented change applied to apply `ChatColor::White`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.white();\n}",
    )]
    pub fn white(self) -> Self {
        self.color(ChatColor::White)
    }

    // ── Text formatting ───────────────────────────────────────────────────────

    /// Set bold formatting.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::bold",
        aliases = ["sand::cmd::TextComponent::bold", "sand::command::TextComponent::bold", "sand::prelude::TextComponent::bold", "sand::prelude::cmd::TextComponent::bold"],
        module = "sand::text",
        kind = "method",
        summary = "Set bold formatting.",
        context = "Set bold formatting. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set bold formatting."),
        returns = "The `TextComponent` value with the documented change applied to set bold formatting.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, v: bool)  {\n    let updated_text_component = text_component_value.bold(v);\n}",
    )]
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }

    /// Set italic formatting.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::italic",
        aliases = ["sand::cmd::TextComponent::italic", "sand::command::TextComponent::italic", "sand::prelude::TextComponent::italic", "sand::prelude::cmd::TextComponent::italic"],
        module = "sand::text",
        kind = "method",
        summary = "Set italic formatting.",
        context = "Set italic formatting. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set italic formatting."),
        returns = "The `TextComponent` value with the documented change applied to set italic formatting.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, v: bool)  {\n    let updated_text_component = text_component_value.italic(v);\n}",
    )]
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }

    /// Set underline formatting.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::underlined",
        aliases = ["sand::cmd::TextComponent::underlined", "sand::command::TextComponent::underlined", "sand::prelude::TextComponent::underlined", "sand::prelude::cmd::TextComponent::underlined"],
        module = "sand::text",
        kind = "method",
        summary = "Set underline formatting.",
        context = "Set underline formatting. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set underline formatting."),
        returns = "The `TextComponent` value with the documented change applied to set underline formatting.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, v: bool)  {\n    let updated_text_component = text_component_value.underlined(v);\n}",
    )]
    pub fn underlined(mut self, v: bool) -> Self {
        self.underlined = Some(v);
        self
    }

    /// Set strikethrough formatting.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::strikethrough",
        aliases = ["sand::cmd::TextComponent::strikethrough", "sand::command::TextComponent::strikethrough", "sand::prelude::TextComponent::strikethrough", "sand::prelude::cmd::TextComponent::strikethrough"],
        module = "sand::text",
        kind = "method",
        summary = "Set strikethrough formatting.",
        context = "Set strikethrough formatting. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set strikethrough formatting."),
        returns = "The `TextComponent` value with the documented change applied to set strikethrough formatting.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, v: bool)  {\n    let updated_text_component = text_component_value.strikethrough(v);\n}",
    )]
    pub fn strikethrough(mut self, v: bool) -> Self {
        self.strikethrough = Some(v);
        self
    }

    /// Set obfuscated (scrambled) text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::obfuscated",
        aliases = ["sand::cmd::TextComponent::obfuscated", "sand::command::TextComponent::obfuscated", "sand::prelude::TextComponent::obfuscated", "sand::prelude::cmd::TextComponent::obfuscated"],
        module = "sand::text",
        kind = "method",
        summary = "Set obfuscated (scrambled) text.",
        context = "Set obfuscated (scrambled) text. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set obfuscated (scrambled) text."),
        returns = "The `TextComponent` value with the documented change applied to set obfuscated (scrambled) text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, v: bool)  {\n    let updated_text_component = text_component_value.obfuscated(v);\n}",
    )]
    pub fn obfuscated(mut self, v: bool) -> Self {
        self.obfuscated = Some(v);
        self
    }

    /// Set the `insertion` string — shift-clicking inserts this into the chat bar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::insertion",
        aliases = ["sand::cmd::TextComponent::insertion", "sand::command::TextComponent::insertion", "sand::prelude::TextComponent::insertion", "sand::prelude::cmd::TextComponent::insertion"],
        module = "sand::text",
        kind = "method",
        summary = "Set the `insertion` string — shift-clicking inserts this into the chat bar.",
        context = "Set the `insertion` string — shift-clicking inserts this into the chat bar. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(text = "`text` provides the author-visible text value used to set the `insertion` string — shift-clicking inserts this into the chat bar."),
        returns = "The `TextComponent` value with the documented change applied to set the `insertion` string — shift-clicking inserts this into the chat bar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, text: impl Into < String >)  {\n    let updated_text_component = text_component_value.insertion(text);\n}",
    )]
    pub fn insertion(mut self, text: impl Into<String>) -> Self {
        self.insertion = Some(text.into());
        self
    }

    // ── Click events ──────────────────────────────────────────────────────────

    /// Run a command when this text is clicked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::click_run_command",
        aliases = ["sand::cmd::TextComponent::click_run_command", "sand::command::TextComponent::click_run_command", "sand::prelude::TextComponent::click_run_command", "sand::prelude::cmd::TextComponent::click_run_command"],
        module = "sand::text",
        kind = "method",
        summary = "Run a command when this text is clicked.",
        context = "Run a command when this text is clicked. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(cmd = "`cmd` supplies the cmd value used to run a command when this text is clicked."),
        returns = "The `TextComponent` value with the documented change applied to run a command when this text is clicked.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, cmd: impl Into < String >)  {\n    let updated_text_component = text_component_value.click_run_command(cmd);\n}",
    )]
    pub fn click_run_command(mut self, cmd: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::RunCommand(cmd.into()));
        self
    }

    /// Fill the chat bar with a suggestion when clicked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::click_suggest_command",
        aliases = ["sand::cmd::TextComponent::click_suggest_command", "sand::command::TextComponent::click_suggest_command", "sand::prelude::TextComponent::click_suggest_command", "sand::prelude::cmd::TextComponent::click_suggest_command"],
        module = "sand::text",
        kind = "method",
        summary = "Fill the chat bar with a suggestion when clicked.",
        context = "Fill the chat bar with a suggestion when clicked. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(cmd = "`cmd` supplies the cmd value used to fill the chat bar with a suggestion when clicked."),
        returns = "The `TextComponent` value with the documented change applied to fill the chat bar with a suggestion when clicked.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, cmd: impl Into < String >)  {\n    let updated_text_component = text_component_value.click_suggest_command(cmd);\n}",
    )]
    pub fn click_suggest_command(mut self, cmd: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::SuggestCommand(cmd.into()));
        self
    }

    /// Open a URL when clicked.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::click_open_url",
        aliases = ["sand::cmd::TextComponent::click_open_url", "sand::command::TextComponent::click_open_url", "sand::prelude::TextComponent::click_open_url", "sand::prelude::cmd::TextComponent::click_open_url"],
        module = "sand::text",
        kind = "method",
        summary = "Open a URL when clicked.",
        context = "Open a URL when clicked. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(url = "`url` supplies the url value used to open a URL when clicked."),
        returns = "The `TextComponent` value with the documented change applied to open a URL when clicked.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, url: impl Into < String >)  {\n    let updated_text_component = text_component_value.click_open_url(url);\n}",
    )]
    pub fn click_open_url(mut self, url: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::OpenUrl(url.into()));
        self
    }

    /// Copy text to the clipboard when clicked.
    ///
    /// `text` is the literal clipboard payload; it is not rendered as the
    /// player-visible label of this component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::click_copy",
        aliases = ["sand::cmd::TextComponent::click_copy", "sand::command::TextComponent::click_copy", "sand::prelude::TextComponent::click_copy", "sand::prelude::cmd::TextComponent::click_copy"],
        module = "sand::text",
        kind = "method",
        summary = "Copy text to the clipboard when clicked. `text` is the literal clipboard payload; it is not rendered as the player-visible label of this component.",
        context = "Copy text to the clipboard when clicked. `text` is the literal clipboard payload; it is not rendered as the player-visible label of this component. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(text = "`text` is the literal clipboard payload; it is not rendered as the player-visible label of this component."),
        returns = "The `TextComponent` value with the documented change applied to copy text to the clipboard when clicked. `text` is the literal clipboard payload; it is not rendered as the player-visible label of this component.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, text: impl Into < String >)  {\n    let updated_text_component = text_component_value.click_copy(text);\n}",
    )]
    pub fn click_copy(mut self, text: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::CopyToClipboard(text.into()));
        self
    }

    /// Emit a `change_page` click event for a page inside a written book.
    ///
    /// Minecraft only applies this click action in book contexts and normally
    /// treats pages as one-indexed. The value is serialized unchanged, including
    /// `0`, to preserve the existing event model's compatibility behavior.
    ///
    /// ```
    /// use sand_commands::Text;
    /// let text = Text::new("Next").click_change_page(2);
    /// assert!(text.to_string().contains(r#""action":"change_page""#));
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::click_change_page",
        aliases = ["sand::cmd::TextComponent::click_change_page", "sand::command::TextComponent::click_change_page", "sand::prelude::TextComponent::click_change_page", "sand::prelude::cmd::TextComponent::click_change_page"],
        module = "sand::text",
        kind = "method",
        summary = "Emit a `change_page` click event for a page inside a written book.",
        context = "Emit a `change_page` click event for a page inside a written book. Minecraft only applies this click action in book contexts and normally treats pages as one-indexed. The value is serialized unchanged, including `0`, to preserve the existing event model's compatibility behavior.",
        minecraft = "Minecraft only applies this click action in book contexts and normally treats pages as one-indexed. The value is serialized unchanged, including `0`, to preserve the existing event model's compatibility behavior.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(page = "`page` supplies the page value used to emit a `change_page` click event for a page inside a written book."),
        returns = "The `TextComponent` value with the documented change applied to emit a `change_page` click event for a page inside a written book.",
        example = "use sand::text::Text;\nlet text = Text::new(\"Next\").click_change_page(2);\nassert!(text.to_string().contains(r#\"\"action\":\"change_page\"\"#));",
    )]
    pub fn click_change_page(mut self, page: u32) -> Self {
        self.click_event = Some(ClickEvent::ChangePage(page));
        self
    }

    // ── Hover events ──────────────────────────────────────────────────────────

    /// Show another `TextComponent` as a tooltip on hover.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_text",
        aliases = ["sand::cmd::TextComponent::hover_text", "sand::command::TextComponent::hover_text", "sand::prelude::TextComponent::hover_text", "sand::prelude::cmd::TextComponent::hover_text"],
        module = "sand::text",
        kind = "method",
        summary = "Show another `TextComponent` as a tooltip on hover.",
        context = "Show another `TextComponent` as a tooltip on hover. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(text = "`text` provides the author-visible text value used to show another `TextComponent` as a tooltip on hover."),
        returns = "The `TextComponent` value with the documented change applied to show another `TextComponent` as a tooltip on hover.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, text: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.hover_text(text);\n}",
    )]
    pub fn hover_text(mut self, text: TextComponent) -> Self {
        self.hover_event = Some(TextHoverEvent::Public(HoverEvent::ShowText(Box::new(text))));
        self
    }

    /// Show an item tooltip on hover using the existing raw item-ID path.
    ///
    /// The resulting `show_item` JSON omits `count`. The item registry string is
    /// retained verbatim for compatibility and is not validated by this builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_item",
        aliases = ["sand::cmd::TextComponent::hover_item", "sand::command::TextComponent::hover_item", "sand::prelude::TextComponent::hover_item", "sand::prelude::cmd::TextComponent::hover_item"],
        module = "sand::text",
        kind = "method",
        summary = "Show an item tooltip on hover using the existing raw item-ID path.",
        context = "Show an item tooltip on hover using the existing raw item-ID path. The resulting `show_item` JSON omits `count`. The item registry string is retained verbatim for compatibility and is not validated by this builder.",
        minecraft = "The resulting `show_item` JSON omits `count`. The item registry string is retained verbatim for compatibility and is not validated by this builder.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(item_id = "`item_id` supplies the item id value used to show an item tooltip on hover using the existing raw item-ID path."),
        returns = "The `TextComponent` value with the documented change applied to show an item tooltip on hover using the existing raw item-ID path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, item_id: impl Into < String >)  {\n    let updated_text_component = text_component_value.hover_item(item_id);\n}",
    )]
    pub fn hover_item(mut self, item_id: impl Into<String>) -> Self {
        self.hover_event = Some(TextHoverEvent::Public(HoverEvent::ShowItem {
            id: item_id.into(),
            count: None,
        }));
        self
    }

    /// Show an item tooltip with an explicit stack count on hover.
    ///
    /// The item registry string and count are serialized unchanged. Item-ID and
    /// stack-size validation belong to the broader text validation work tracked
    /// in #152; this builder exposes the count-bearing shape already modeled by
    /// [`HoverEvent::ShowItem`] without changing the count-free output.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_item_with_count",
        aliases = ["sand::cmd::TextComponent::hover_item_with_count", "sand::command::TextComponent::hover_item_with_count", "sand::prelude::TextComponent::hover_item_with_count", "sand::prelude::cmd::TextComponent::hover_item_with_count"],
        module = "sand::text",
        kind = "method",
        summary = "Show an item tooltip with an explicit stack count on hover.",
        context = "Show an item tooltip with an explicit stack count on hover. The item registry string and count are serialized unchanged. Item-ID and stack-size validation belong to the broader text validation work tracked in #152; this builder exposes the count-bearing shape already modeled by [`HoverEvent::ShowItem`] without changing the count-free output.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(item_id = "`item_id` supplies the item id value used to show an item tooltip with an explicit stack count on hover.", count = "`count` provides the requested numeric amount used to show an item tooltip with an explicit stack count on hover."),
        returns = "The `TextComponent` value with the documented change applied to show an item tooltip with an explicit stack count on hover.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, item_id: impl Into < String >, count: u32)  {\n    let updated_text_component = text_component_value.hover_item_with_count(item_id, count);\n}",
    )]
    pub fn hover_item_with_count(mut self, item_id: impl Into<String>, count: u32) -> Self {
        self.hover_event = Some(TextHoverEvent::Public(HoverEvent::ShowItem {
            id: item_id.into(),
            count: Some(count),
        }));
        self
    }

    /// Show an entity tooltip without a UUID on hover.
    ///
    /// The entity type must be one of Sand's typed registry identifiers. The
    /// displayed name remains a full text component, so styling and translation
    /// data are preserved.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_entity",
        aliases = ["sand::cmd::TextComponent::hover_entity", "sand::command::TextComponent::hover_entity", "sand::prelude::TextComponent::hover_entity", "sand::prelude::cmd::TextComponent::hover_entity"],
        module = "sand::text",
        kind = "method",
        summary = "Show an entity tooltip without a UUID on hover. The entity type must be one of Sand's typed registry identifiers. The displayed name remains a full text component, so styling and translation data are preserved.",
        context = "Show an entity tooltip without a UUID on hover. The entity type must be one of Sand's typed registry identifiers. The displayed name remains a full text component, so styling and translation data are preserved. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(entity_type = "`entity_type` provides the player-visible text value used to show an entity tooltip without a UUID on hover. The entity type must be one of Sand's typed registry identifiers. The displayed name remains a full text component, so styling and translation data are preserved.", name = "`name` provides the author-visible text value used to show an entity tooltip without a UUID on hover. The entity type must be one of Sand's typed registry identifiers. The displayed name remains a full text component, so styling and translation data are preserved."),
        returns = "The `TextComponent` value with the documented change applied to show an entity tooltip without a UUID on hover. The entity type must be one of Sand's typed registry identifiers. The displayed name remains a full text component, so styling and translation data are preserved.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, entity_type: impl sand::text::IntoTextEntityType, name: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.hover_entity(entity_type, name);\n}",
    )]
    pub fn hover_entity(
        mut self,
        entity_type: impl IntoTextEntityType,
        name: TextComponent,
    ) -> Self {
        self.hover_event = Some(TextHoverEvent::ShowEntityText {
            name: Box::new(name),
            entity_type: entity_type.into_text_entity_type(),
            id: None,
            raw: false,
        });
        self
    }

    /// Show an entity tooltip with a validated UUID on hover.
    ///
    /// `id` is the entity UUID shown in the hover payload, not a namespaced
    /// Minecraft resource identifier.
    ///
    /// Parse user-provided UUID text with [`EntityHoverId::parse`] first. The
    /// styled `name` is serialized as a complete text component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_entity_with_id",
        aliases = ["sand::cmd::TextComponent::hover_entity_with_id", "sand::command::TextComponent::hover_entity_with_id", "sand::prelude::TextComponent::hover_entity_with_id", "sand::prelude::cmd::TextComponent::hover_entity_with_id"],
        module = "sand::text",
        kind = "method",
        summary = "Show an entity tooltip with a validated UUID on hover.",
        context = "Show an entity tooltip with a validated UUID on hover. `id` is the entity UUID shown in the hover payload, not a namespaced Minecraft resource identifier. Parse user-provided UUID text with [`EntityHoverId::parse`] first. The styled `name` is serialized as a complete text component.",
        minecraft = "`id` is the entity UUID shown in the hover payload, not a namespaced Minecraft resource identifier.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(entity_type = "`entity_type` provides the player-visible text value used to show an entity tooltip with a validated UUID on hover.", id = "`id` is the entity UUID shown in the hover payload, not a namespaced Minecraft resource identifier.", name = "Parse user-provided UUID text with [`EntityHoverId::parse`] first. The styled `name` is serialized as a complete text component."),
        returns = "The `TextComponent` value with the documented change applied to show an entity tooltip with a validated UUID on hover.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, entity_type: impl sand::text::IntoTextEntityType, id: sand::text::EntityHoverId, name: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.hover_entity_with_id(entity_type, id, name);\n}",
    )]
    pub fn hover_entity_with_id(
        mut self,
        entity_type: impl IntoTextEntityType,
        id: EntityHoverId,
        name: TextComponent,
    ) -> Self {
        self.hover_event = Some(TextHoverEvent::ShowEntityText {
            name: Box::new(name),
            entity_type: entity_type.into_text_entity_type(),
            id: Some(id.to_string()),
            raw: false,
        });
        self
    }

    /// Show an entity tooltip using unchecked raw entity type and UUID strings.
    ///
    /// Prefer [`Self::hover_entity`] or [`Self::hover_entity_with_id`]. This is
    /// an explicit compatibility escape hatch for legacy or version-specific
    /// values Sand cannot model. Neither raw string is validated; the styled
    /// `name` is still serialized as a complete text component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_entity_raw",
        aliases = ["sand::cmd::TextComponent::hover_entity_raw", "sand::command::TextComponent::hover_entity_raw", "sand::prelude::TextComponent::hover_entity_raw", "sand::prelude::cmd::TextComponent::hover_entity_raw"],
        module = "sand::text",
        kind = "method",
        summary = "Show an entity tooltip using unchecked raw entity type and UUID strings.",
        context = "Show an entity tooltip using unchecked raw entity type and UUID strings. Prefer [`Self::hover_entity`] or [`Self::hover_entity_with_id`]. This is an explicit compatibility escape hatch for legacy or version-specific values Sand cannot model. Neither raw string is validated; the styled `name` is still serialized as a complete text component.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Prefer [`Self::hover_entity`] or [`Self::hover_entity_with_id`]. This is an explicit compatibility escape hatch for legacy or version-specific values Sand cannot model. Neither raw string is validated; the styled `name` is still serialized as a complete text component."],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(entity_type = "`entity_type` supplies the entity type value used to show an entity tooltip using unchecked raw entity type and UUID strings.", id = "`id` provides the typed resource identifier or location used to show an entity tooltip using unchecked raw entity type and UUID strings.", name = "Prefer [`Self::hover_entity`] or [`Self::hover_entity_with_id`]. This is an explicit compatibility escape hatch for legacy or version-specific values Sand cannot model. Neither raw string is validated; the styled `name` is still serialized as a complete text component."),
        returns = "The `TextComponent` value with the documented change applied to show an entity tooltip using unchecked raw entity type and UUID strings.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, entity_type: impl Into < String >, id: Option < String >, name: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.hover_entity_raw(entity_type, id, name);\n}",
    )]
    pub fn hover_entity_raw(
        mut self,
        entity_type: impl Into<String>,
        id: Option<String>,
        name: TextComponent,
    ) -> Self {
        self.hover_event = Some(TextHoverEvent::ShowEntityText {
            name: Box::new(name),
            entity_type: entity_type.into(),
            id,
            raw: true,
        });
        self
    }

    /// Show an item tooltip with an intentionally opaque item token.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::hover_item_raw",
        aliases = ["sand::cmd::TextComponent::hover_item_raw", "sand::command::TextComponent::hover_item_raw", "sand::prelude::TextComponent::hover_item_raw", "sand::prelude::cmd::TextComponent::hover_item_raw"],
        module = "sand::text",
        kind = "method",
        summary = "Show an item tooltip with an intentionally opaque item token.",
        context = "Show an item tooltip with an intentionally opaque item token. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(item = "`item` provides the item value or item predicate used to show an item tooltip with an intentionally opaque item token.", count = "`count` provides the requested numeric amount used to show an item tooltip with an intentionally opaque item token."),
        returns = "The `TextComponent` value with the documented change applied to show an item tooltip with an intentionally opaque item token.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, item: impl Into < String >, count: Option < u32 >)  {\n    let updated_text_component = text_component_value.hover_item_raw(item, count);\n}",
    )]
    pub fn hover_item_raw(mut self, item: impl Into<String>, count: Option<u32>) -> Self {
        self.hover_event = Some(TextHoverEvent::RawShowItem {
            id: item.into(),
            count,
        });
        self
    }

    /// Append a sibling component in the `"extra"` array.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::then",
        aliases = ["sand::cmd::TextComponent::then", "sand::command::TextComponent::then", "sand::prelude::TextComponent::then", "sand::prelude::cmd::TextComponent::then"],
        module = "sand::text",
        kind = "method",
        summary = "Append a sibling component in the `\"extra\"` array.",
        context = "Append a sibling component in the `\"extra\"` array. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(next = "`next` provides the player-visible text value used to append a sibling component in the `\"extra\"` array."),
        returns = "The `TextComponent` value with the documented change applied to append a sibling component in the `\"extra\"` array.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: sand::text::TextComponent, next: sand::text::TextComponent)  {\n    let updated_text_component = text_component_value.then(next);\n}",
    )]
    pub fn then(mut self, next: TextComponent) -> Self {
        self.extra.push(next);
        self
    }

    // ── Serialization ─────────────────────────────────────────────────────────

    fn to_json_value(&self) -> serde_json::Value {
        let mut obj = match &self.content {
            TextContent::Literal(s) => serde_json::json!({ "text": s }),
            TextContent::Score { name, objective } => {
                serde_json::json!({ "score": { "name": name, "objective": objective } })
            }
            TextContent::Selector { value, .. } => serde_json::json!({ "selector": value }),
            TextContent::Translate { key, with } => {
                if with.is_empty() {
                    serde_json::json!({ "translate": key })
                } else {
                    let with_json: Vec<_> = with.iter().map(|w| w.to_json_value()).collect();
                    serde_json::json!({ "translate": key, "with": with_json })
                }
            }
            TextContent::Keybind(key) => serde_json::json!({ "keybind": key }),
        };
        if let Some(c) = &self.color {
            obj["color"] = serde_json::json!(c);
        }
        if let Some(font) = &self.font {
            obj["font"] = serde_json::json!(font);
        }
        if let Some(v) = self.bold {
            obj["bold"] = serde_json::json!(v);
        }
        if let Some(v) = self.italic {
            obj["italic"] = serde_json::json!(v);
        }
        if let Some(v) = self.underlined {
            obj["underlined"] = serde_json::json!(v);
        }
        if let Some(v) = self.strikethrough {
            obj["strikethrough"] = serde_json::json!(v);
        }
        if let Some(v) = self.obfuscated {
            obj["obfuscated"] = serde_json::json!(v);
        }
        if let Some(ins) = &self.insertion {
            obj["insertion"] = serde_json::json!(ins);
        }
        if let Some(ev) = &self.click_event {
            obj["clickEvent"] = match ev {
                ClickEvent::RunCommand(s) => {
                    serde_json::json!({"action": "run_command", "value": s})
                }
                ClickEvent::SuggestCommand(s) => {
                    serde_json::json!({"action": "suggest_command", "value": s})
                }
                ClickEvent::OpenUrl(s) => serde_json::json!({"action": "open_url", "value": s}),
                ClickEvent::CopyToClipboard(s) => {
                    serde_json::json!({"action": "copy_to_clipboard", "value": s})
                }
                ClickEvent::ChangePage(p) => {
                    serde_json::json!({"action": "change_page", "value": p})
                }
            };
        }
        if let Some(ev) = &self.hover_event {
            obj["hoverEvent"] = match ev {
                TextHoverEvent::Public(HoverEvent::ShowText(t)) => {
                    serde_json::json!({"action": "show_text", "contents": t.to_json_value()})
                }
                TextHoverEvent::Public(HoverEvent::ShowItem { id, count }) => {
                    let mut h = serde_json::json!({"action": "show_item", "id": id});
                    if let Some(c) = count {
                        h["count"] = serde_json::json!(c);
                    }
                    h
                }
                TextHoverEvent::RawShowItem { id, count } => {
                    let mut h = serde_json::json!({"action": "show_item", "id": id});
                    if let Some(c) = count {
                        h["count"] = serde_json::json!(c);
                    }
                    h
                }
                TextHoverEvent::Public(HoverEvent::ShowEntity {
                    name,
                    entity_type,
                    id,
                }) => {
                    let mut h = serde_json::json!({"action": "show_entity", "name": name, "type": entity_type});
                    if let Some(i) = id {
                        h["id"] = serde_json::json!(i);
                    }
                    h
                }
                TextHoverEvent::ShowEntityText {
                    name,
                    entity_type,
                    id,
                    ..
                } => {
                    let mut h = serde_json::json!({
                        "action": "show_entity",
                        "name": name.to_json_value(),
                        "type": entity_type,
                    });
                    if let Some(i) = id {
                        h["id"] = serde_json::json!(i);
                    }
                    h
                }
            };
        }
        if !self.extra.is_empty() {
            let extras: Vec<_> = self.extra.iter().map(|e| e.to_json_value()).collect();
            obj["extra"] = serde_json::json!(extras);
        }
        obj
    }

    /// Validate this component recursively and return its deterministic JSON value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::try_to_json_value",
        aliases = ["sand::cmd::TextComponent::try_to_json_value", "sand::command::TextComponent::try_to_json_value", "sand::prelude::TextComponent::try_to_json_value", "sand::prelude::cmd::TextComponent::try_to_json_value"],
        module = "sand::text",
        kind = "method",
        summary = "Validate this component recursively and return its deterministic JSON value.",
        context = "Validate this component recursively and return its deterministic JSON value. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        returns = "On success, the value produced to validate this component recursively and return its deterministic JSON value; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: &sand::text::TextComponent)  {\n    let try_to_json_value = text_component_value.try_to_json_value();\n}",
    )]
    pub fn try_to_json_value(&self) -> CommandResult<serde_json::Value> {
        self.validate(&CommandProfile::unprofiled())?;
        Ok(self.to_json_value())
    }

    /// Validate recursively while retaining a consumer-provided field path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::text::TextComponent::validate_at_path",
        aliases = ["sand::cmd::TextComponent::validate_at_path", "sand::command::TextComponent::validate_at_path", "sand::prelude::TextComponent::validate_at_path", "sand::prelude::cmd::TextComponent::validate_at_path"],
        module = "sand::text",
        kind = "method",
        summary = "Validate recursively while retaining a consumer-provided field path.",
        context = "Validate recursively while retaining a consumer-provided field path. Sand text values preserve Minecraft's structured JSON component model, including styling and validated click or hover interactions.",
        minecraft = "The component serializes to the JSON text format consumed by tellraw, titles, books, dialogs, and other vanilla text fields.",
        use_when = ["Building player-visible text with typed styling or interactions"],
        avoid_when = ["Passing an unvalidated JSON string when a typed text component can express the same value"],
        params(profile = "`profile` supplies the profile value used to validate recursively while retaining a consumer-provided field path.", path = "`path` provides the typed resource identifier or location used to validate recursively while retaining a consumer-provided field path."),
        returns = "On success, the value produced to validate recursively while retaining a consumer-provided field path; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(text_component_value: &sand::text::TextComponent, profile: & sand::command::CommandProfile, path: impl Into < String >)  {\n    let validate_at_path = text_component_value.validate_at_path(profile, path);\n}",
    )]
    pub fn validate_at_path(
        &self,
        profile: &CommandProfile,
        path: impl Into<String>,
    ) -> CommandResult<()> {
        self.validate_inner(profile, &path.into(), 0)
    }

    fn validate_inner(
        &self,
        profile: &CommandProfile,
        path: &str,
        depth: usize,
    ) -> CommandResult<()> {
        const MAX_DEPTH: usize = 64;
        if depth > MAX_DEPTH {
            return Err(text_error(
                "SAND-TEXT-DEPTH",
                path,
                format!("text nesting exceeds the supported depth of {MAX_DEPTH}"),
            ));
        }

        match &self.content {
            TextContent::Literal(_) => {}
            TextContent::Score { name, objective } => {
                validate_score_holder(name).map_err(|error| {
                    text_error(
                        "SAND-TEXT-SCORE-HOLDER",
                        format!("{path}.score.name"),
                        error.message,
                    )
                })?;
                validate_objective(objective).map_err(|error| {
                    text_error(
                        "SAND-TEXT-SCORE-OBJECTIVE",
                        format!("{path}.score.objective"),
                        error.message,
                    )
                })?;
            }
            TextContent::Selector { value, raw: false } => {
                crate::render::validate_selector_token(value).map_err(|error| {
                    text_error(
                        "SAND-TEXT-SELECTOR",
                        format!("{path}.selector"),
                        format!(
                            "{}; use `TextComponent::selector_raw(...)` for opaque syntax",
                            error.message
                        ),
                    )
                })?;
            }
            TextContent::Selector { raw: true, .. } => {}
            TextContent::Translate { key, with } => {
                validate_non_blank_no_control(key, "translation key").map_err(|message| {
                    text_error("SAND-TEXT-TRANSLATE", format!("{path}.translate"), message)
                })?;
                for (index, argument) in with.iter().enumerate() {
                    argument.validate_inner(
                        profile,
                        &format!("{path}.with[{index}]"),
                        depth + 1,
                    )?;
                }
            }
            TextContent::Keybind(key) => {
                if key.is_empty() || key.chars().any(|c| c.is_whitespace() || c.is_control()) {
                    return Err(text_error(
                        "SAND-TEXT-KEYBIND",
                        format!("{path}.keybind"),
                        format!(
                            "keybind IDs must be non-empty and contain no whitespace/control characters; got `{key}`"
                        ),
                    ));
                }
            }
        }

        if let Some(color) = &self.color
            && !is_named_color(color)
            && !is_hex_color(color)
        {
            return Err(text_error(
                "SAND-TEXT-COLOR",
                format!("{path}.style.color"),
                format!("expected a named Minecraft color or `#RRGGBB`, got `{color}`"),
            ));
        }
        if let Some(font) = &self.font {
            crate::validate::resource_location_shape(font, "TextComponent", "font").map_err(
                |error| {
                    text_error(
                        "SAND-TEXT-FONT",
                        format!("{path}.style.font"),
                        error.message,
                    )
                },
            )?;
        }
        if let Some(click) = &self.click_event {
            validate_click(click, path)?;
        }
        if let Some(hover) = &self.hover_event {
            validate_hover(hover, profile, path, depth + 1)?;
        }
        for (index, child) in self.extra.iter().enumerate() {
            child.validate_inner(profile, &format!("{path}.extra[{index}]"), depth + 1)?;
        }
        Ok(())
    }
}

impl Validate for TextComponent {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.validate_inner(profile, "text", 0)
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::TextCommand",
    aliases = ["sand::cmd::TextCommand", "sand::prelude::cmd::TextCommand"],
    module = "sand::command",
    summary = "Structured `tellraw` command retaining its selector and text component.",
    context = "Structured `tellraw` command retaining its selector and text component. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::TextCommand;",
)]
/// Structured `tellraw` command retaining its selector and text component.
#[derive(Debug, Clone)]
pub struct TextCommand {
    target: Selector,
    text: TextComponent,
}

impl TextCommand {
    /// Builds a typed `tellraw` command for the selected entities and text component.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TextCommand::tellraw",
        aliases = ["sand::cmd::TextCommand::tellraw", "sand::prelude::cmd::TextCommand::tellraw"],
        module = "sand::command",
        kind = "method",
        summary = "Builds a typed `tellraw` command for the selected entities and text component.",
        context = "Builds a typed `tellraw` command for the selected entities and text component. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(target = "`target` provides the entity, block, or command target used to build a typed `tellraw` command for the selected entities and text component.", text = "`text` provides the author-visible text value used to build a typed `tellraw` command for the selected entities and text component."),
        returns = "A newly constructed `TextCommand` configured to build a typed `tellraw` command for the selected entities and text component.",
        example = "use sand::prelude::*;\n\nfn demonstrate(target: sand::command::Selector, text: sand::text::TextComponent)  {\n    let text_command = sand::command::TextCommand::tellraw(target, text);\n}",
    )]
    pub fn tellraw(target: Selector, text: TextComponent) -> Self {
        Self { target, text }
    }
}

impl Validate for TextCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.target
            .validate(profile)
            .map_err(|error| text_error("SAND-TEXT-TARGET", "target", error.to_string()))?;
        self.text.validate_at_path(profile, "tellraw.text")
    }
}

impl RenderCommand for TextCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        format!("tellraw {} {}", self.target, self.text)
    }
}

impl Build for TextCommand {
    fn build(&self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, self.clone());
        line
    }
}

impl fmt::Display for TextCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.build())
    }
}

impl From<TextCommand> for String {
    fn from(command: TextCommand) -> Self {
        command.build()
    }
}

fn text_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("TextComponent", field, message).with_code(code)
}

fn validate_non_blank_no_control(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be empty or whitespace-only"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("{label} must not contain control characters"));
    }
    Ok(())
}

fn validate_score_holder(value: &str) -> CommandResult<()> {
    if value.starts_with('@') {
        crate::render::validate_selector_token(value)?;
    } else if value.is_empty()
        || value.len() > 40
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(CommandError::new(
            "TextComponent",
            "score.name",
            format!(
                "score holder must be a valid selector or a non-empty single token of at most 40 characters; got `{value}`"
            ),
        ));
    }
    Ok(())
}

fn validate_objective(value: &str) -> CommandResult<()> {
    if value.is_empty()
        || value.len() > 16
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(CommandError::new(
            "TextComponent",
            "score.objective",
            format!(
                "objective names must be non-empty single tokens of at most 16 characters; got `{value}`"
            ),
        ));
    }
    Ok(())
}

fn is_named_color(value: &str) -> bool {
    matches!(
        value,
        "black"
            | "dark_blue"
            | "dark_green"
            | "dark_aqua"
            | "dark_red"
            | "dark_purple"
            | "gold"
            | "gray"
            | "dark_gray"
            | "blue"
            | "green"
            | "aqua"
            | "red"
            | "light_purple"
            | "yellow"
            | "white"
            | "reset"
    )
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_click(click: &ClickEvent, path: &str) -> CommandResult<()> {
    match click {
        ClickEvent::RunCommand(value) | ClickEvent::SuggestCommand(value)
            if value.trim().is_empty() =>
        {
            Err(text_error(
                "SAND-TEXT-CLICK-COMMAND",
                format!("{path}.click_event.value"),
                "command click values must not be empty",
            ))
        }
        ClickEvent::OpenUrl(value) => {
            let remainder = value
                .strip_prefix("https://")
                .or_else(|| value.strip_prefix("http://"));
            if remainder.is_none_or(|rest| {
                rest.is_empty()
                    || rest.starts_with('/')
                    || rest.chars().any(|c| c.is_whitespace() || c.is_control())
            }) {
                Err(text_error(
                    "SAND-TEXT-CLICK-URL",
                    format!("{path}.click_event.value"),
                    format!(
                        "open_url requires a non-empty `http://` or `https://` target without whitespace; got `{value}`"
                    ),
                ))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn validate_hover(
    hover: &TextHoverEvent,
    profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    match hover {
        TextHoverEvent::Public(HoverEvent::ShowText(text)) => {
            text.validate_inner(profile, &format!("{path}.hover_event.contents"), depth)
        }
        TextHoverEvent::Public(HoverEvent::ShowItem { id, count }) => {
            crate::validate::resource_location_shape(id, "TextComponent", "hover.item.id")
                .map_err(|error| {
                    text_error(
                        "SAND-TEXT-HOVER-ITEM",
                        format!("{path}.hover_event.contents.id"),
                        error.message,
                    )
                })?;
            if count.is_some_and(|count| count == 0) {
                return Err(text_error(
                    "SAND-TEXT-HOVER-ITEM",
                    format!("{path}.hover_event.contents.count"),
                    "item hover count must be greater than zero when present",
                ));
            }
            Ok(())
        }
        TextHoverEvent::RawShowItem { .. } => Ok(()),
        TextHoverEvent::Public(HoverEvent::ShowEntity {
            name: _,
            entity_type,
            id,
        }) => validate_entity_hover(entity_type, id.as_deref(), path),
        TextHoverEvent::ShowEntityText {
            name,
            entity_type,
            id,
            raw,
        } => {
            if !raw {
                validate_entity_hover(entity_type, id.as_deref(), path)?;
            }
            name.validate_inner(profile, &format!("{path}.hover_event.contents.name"), depth)
        }
    }
}

fn validate_entity_hover(entity_type: &str, id: Option<&str>, path: &str) -> CommandResult<()> {
    crate::validate::resource_location_shape(entity_type, "TextComponent", "hover.entity.type")
        .map_err(|error| {
            text_error(
                "SAND-TEXT-HOVER-ENTITY",
                format!("{path}.hover_event.contents.type"),
                error.message,
            )
        })?;
    if let Some(id) = id {
        EntityHoverId::parse(id).map_err(|error| {
            text_error(
                "SAND-TEXT-HOVER-ENTITY",
                format!("{path}.hover_event.contents.id"),
                error.message,
            )
        })?;
    }
    Ok(())
}

/// Export-scoped registry family holding this module's rendered
/// text command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct TextLines;

impl crate::export_registry::RegistryFamily for TextLines {
    type State = BTreeMap<String, TextCommand>;
}

fn register_line(line: &str, command: TextCommand) {
    crate::export_registry::register_line::<TextLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<TextLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
}

/// Validate a component-owned JSON text value.
///
/// This bridges advancement and chat-style `serde_json::Value` fields into
/// the same recursive validator used by typed command text.
pub fn validate_json_text(
    value: &serde_json::Value,
    profile: &CommandProfile,
    path: &str,
) -> CommandResult<()> {
    validate_json_text_inner(value, profile, path, 0)
}

fn validate_json_text_inner(
    value: &serde_json::Value,
    _profile: &CommandProfile,
    path: &str,
    depth: usize,
) -> CommandResult<()> {
    if depth > 64 {
        return Err(text_error(
            "SAND-TEXT-DEPTH",
            path,
            "JSON text nesting exceeds the supported depth of 64",
        ));
    }
    match value {
        serde_json::Value::String(_) => Ok(()),
        serde_json::Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                validate_json_text_inner(child, _profile, &format!("{path}[{index}]"), depth + 1)?;
            }
            Ok(())
        }
        serde_json::Value::Object(object) => {
            let content_keys = ["text", "translate", "score", "selector", "keybind", "nbt"];
            let present = content_keys
                .iter()
                .filter(|key| object.contains_key(**key))
                .copied()
                .collect::<Vec<_>>();
            if present.len() > 1 {
                return Err(text_error(
                    "SAND-TEXT-CONTENT",
                    path,
                    format!("conflicting text content fields: {}", present.join(", ")),
                ));
            }
            if let Some(serde_json::Value::String(color)) = object.get("color")
                && !is_named_color(color)
                && !is_hex_color(color)
            {
                return Err(text_error(
                    "SAND-TEXT-COLOR",
                    format!("{path}.color"),
                    format!("expected a named Minecraft color or `#RRGGBB`, got `{color}`"),
                ));
            }
            if let Some(serde_json::Value::String(font)) = object.get("font") {
                crate::validate::resource_location_shape(font, "TextComponent", "font").map_err(
                    |error| text_error("SAND-TEXT-FONT", format!("{path}.font"), error.message),
                )?;
            }
            if let Some(serde_json::Value::String(key)) = object.get("translate") {
                validate_non_blank_no_control(key, "translation key").map_err(|message| {
                    text_error("SAND-TEXT-TRANSLATE", format!("{path}.translate"), message)
                })?;
            }
            if let Some(serde_json::Value::String(key)) = object.get("keybind")
                && (key.is_empty() || key.chars().any(|c| c.is_whitespace() || c.is_control()))
            {
                return Err(text_error(
                    "SAND-TEXT-KEYBIND",
                    format!("{path}.keybind"),
                    "keybind IDs must be non-empty and contain no whitespace/control characters",
                ));
            }
            if let Some(serde_json::Value::String(selector)) = object.get("selector") {
                crate::render::validate_selector_token(selector).map_err(|error| {
                    text_error(
                        "SAND-TEXT-SELECTOR",
                        format!("{path}.selector"),
                        error.message,
                    )
                })?;
            }
            if let Some(serde_json::Value::Object(score)) = object.get("score") {
                if let Some(serde_json::Value::String(name)) = score.get("name") {
                    validate_score_holder(name)?;
                }
                if let Some(serde_json::Value::String(objective)) = score.get("objective") {
                    validate_objective(objective)?;
                }
            }
            for key in ["with", "extra"] {
                if let Some(serde_json::Value::Array(children)) = object.get(key) {
                    for (index, child) in children.iter().enumerate() {
                        validate_json_text_inner(
                            child,
                            _profile,
                            &format!("{path}.{key}[{index}]"),
                            depth + 1,
                        )?;
                    }
                }
            }
            if let Some(hover) = object.get("hoverEvent")
                && let Some(contents) = hover.get("contents")
            {
                validate_json_text_inner(
                    contents,
                    _profile,
                    &format!("{path}.hoverEvent.contents"),
                    depth + 1,
                )?;
            }
            Ok(())
        }
        _ => Err(text_error(
            "SAND-TEXT-SHAPE",
            path,
            "text JSON must be a string, object, or array",
        )),
    }
}

impl fmt::Display for TextComponent {
    /// Serialize to a compact JSON string suitable for embedding directly in Minecraft commands.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_json_value())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Zombie;

    impl IntoTextEntityType for Zombie {
        fn into_text_entity_type(self) -> String {
            "minecraft:zombie".to_owned()
        }
    }

    #[test]
    fn chat_color_display() {
        assert_eq!(ChatColor::Gold.to_string(), "gold");
        assert_eq!(ChatColor::DarkBlue.to_string(), "dark_blue");
        assert_eq!(ChatColor::LightPurple.to_string(), "light_purple");
        assert_eq!(ChatColor::White.to_string(), "white");
        assert_eq!(ChatColor::Red.to_string(), "red");
    }

    #[test]
    fn literal_component() {
        let t = TextComponent::literal("Hi!")
            .color(ChatColor::Gold)
            .bold(true);
        let s = t.to_string();
        assert!(s.contains("\"text\":\"Hi!\""));
        assert!(s.contains("\"color\":\"gold\""));
        assert!(s.contains("\"bold\":true"));
    }

    #[test]
    fn score_component() {
        let t = TextComponent::score("@s", "join_count").color(ChatColor::Aqua);
        let s = t.to_string();
        assert!(s.contains("\"score\""));
        assert!(s.contains("\"name\":\"@s\""));
        assert!(s.contains("\"objective\":\"join_count\""));
        assert!(s.contains("\"color\":\"aqua\""));
    }

    #[test]
    fn selector_component() {
        let t = TextComponent::selector("@a");
        assert!(t.to_string().contains("\"selector\":\"@a\""));
    }

    #[test]
    fn translate_component() {
        let t = TextComponent::translate("death.attack.generic");
        assert!(t.to_string().contains("\"translate\""));
        assert!(!t.to_string().contains("\"with\""));
    }

    #[test]
    fn translate_with_component() {
        let t =
            TextComponent::translate_with("chat.type.text", vec![TextComponent::literal("Toast")]);
        let s = t.to_string();
        assert!(s.contains("\"with\""));
    }

    #[test]
    fn keybind_component() {
        let t = TextComponent::keybind("key.jump");
        assert!(t.to_string().contains("\"keybind\":\"key.jump\""));
    }

    #[test]
    fn color_hex() {
        let t = TextComponent::literal("hex!").color_hex("#FF5733");
        assert!(t.to_string().contains("\"color\":\"#FF5733\""));
    }

    #[test]
    fn recursive_validation_rejects_structured_field_errors() {
        let profile = CommandProfile::unprofiled();
        assert_eq!(
            Text::new("bad")
                .color_hex("#12FG00")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-COLOR"
        );
        assert_eq!(
            TextComponent::selector("@e[type=]")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-SELECTOR"
        );
        assert_eq!(
            TextComponent::score("@s", "objective name")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-SCORE-OBJECTIVE"
        );
        assert_eq!(
            TextComponent::translate(" \t ")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-TRANSLATE"
        );
        assert_eq!(
            TextComponent::keybind("key jump")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-KEYBIND"
        );
        assert_eq!(
            Text::new("link")
                .click_open_url("ftp://example.com")
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-CLICK-URL"
        );
        assert_eq!(
            Text::new("item")
                .hover_item_with_count("minecraft:diamond", 0)
                .validate(&profile)
                .unwrap_err()
                .code,
            "SAND-TEXT-HOVER-ITEM"
        );
    }

    #[test]
    fn raw_text_fields_are_opaque_but_nested_typed_text_is_not() {
        assert!(
            TextComponent::selector_raw("@e[type=]")
                .validate(&CommandProfile::unprofiled())
                .is_ok()
        );
        assert!(
            Text::new("raw")
                .hover_item_raw("modded payload", Some(0))
                .validate(&CommandProfile::unprofiled())
                .is_ok()
        );
    }

    #[test]
    fn multi_segment_extra() {
        let msg = TextComponent::literal("Score: ")
            .color(ChatColor::White)
            .then(TextComponent::score("@s", "kills").color(ChatColor::Red));
        let s = msg.to_string();
        assert!(s.contains("\"extra\""));
        assert!(s.contains("\"text\":\"Score: \""));
        assert!(s.contains("\"color\":\"red\""));
    }

    #[test]
    fn all_formatting_flags() {
        let t = TextComponent::literal("x")
            .bold(true)
            .italic(false)
            .underlined(true)
            .strikethrough(false)
            .obfuscated(true);
        let s = t.to_string();
        assert!(s.contains("\"bold\":true"));
        assert!(s.contains("\"italic\":false"));
        assert!(s.contains("\"underlined\":true"));
        assert!(s.contains("\"strikethrough\":false"));
        assert!(s.contains("\"obfuscated\":true"));
    }

    // ── New: color shortcuts ──────────────────────────────────────────────────

    #[test]
    fn color_shortcuts() {
        assert!(
            TextComponent::literal("x")
                .gold()
                .to_string()
                .contains("\"color\":\"gold\"")
        );
        assert!(
            TextComponent::literal("x")
                .aqua()
                .to_string()
                .contains("\"color\":\"aqua\"")
        );
        assert!(
            TextComponent::literal("x")
                .green()
                .to_string()
                .contains("\"color\":\"green\"")
        );
        assert!(
            TextComponent::literal("x")
                .red()
                .to_string()
                .contains("\"color\":\"red\"")
        );
        assert!(
            TextComponent::literal("x")
                .yellow()
                .to_string()
                .contains("\"color\":\"yellow\"")
        );
        assert!(
            TextComponent::literal("x")
                .white()
                .to_string()
                .contains("\"color\":\"white\"")
        );
        assert!(
            TextComponent::literal("x")
                .gray()
                .to_string()
                .contains("\"color\":\"gray\"")
        );
        assert!(
            TextComponent::literal("x")
                .dark_gray()
                .to_string()
                .contains("\"color\":\"dark_gray\"")
        );
    }

    // ── New: Text alias ───────────────────────────────────────────────────────

    #[test]
    fn text_alias_new() {
        let t = Text::new("Hello").gold().bold(true);
        let s = t.to_string();
        assert!(s.contains("\"text\":\"Hello\""));
        assert!(s.contains("\"color\":\"gold\""));
        assert!(s.contains("\"bold\":true"));
    }

    #[test]
    fn text_raw_json() {
        let json = Text::raw_json("{\"text\":\"raw\"}");
        assert_eq!(json, "{\"text\":\"raw\"}");
    }

    // ── New: click events ─────────────────────────────────────────────────────

    #[test]
    fn click_run_command() {
        let t = Text::new("Click me").click_run_command("/say hi");
        let s = t.to_string();
        assert!(s.contains("\"clickEvent\""), "got: {s}");
        assert!(s.contains("\"run_command\""), "got: {s}");
        assert!(s.contains("/say hi"), "got: {s}");
    }

    #[test]
    fn click_suggest_command() {
        let t = Text::new("Suggest").click_suggest_command("/tell @s ");
        let s = t.to_string();
        assert!(s.contains("\"suggest_command\""), "got: {s}");
    }

    #[test]
    fn click_open_url() {
        let t = Text::new("Visit").click_open_url("https://example.com");
        let s = t.to_string();
        assert!(s.contains("\"open_url\""), "got: {s}");
        assert!(s.contains("https://example.com"), "got: {s}");
    }

    #[test]
    fn click_copy() {
        let t = Text::new("Copy").click_copy("some text");
        let s = t.to_string();
        assert!(s.contains("\"copy_to_clipboard\""), "got: {s}");
    }

    #[test]
    fn click_change_page_preserves_page_zero_styling_and_siblings() {
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Next page")
                .gold()
                .click_change_page(0)
                .then(Text::new("!").bold(true))
                .to_string(),
        )
        .unwrap();
        assert_eq!(
            value["clickEvent"],
            serde_json::json!({"action": "change_page", "value": 0})
        );
        assert_eq!(value["color"], "gold");
        assert_eq!(
            value["extra"][0],
            serde_json::json!({"text": "!", "bold": true})
        );
    }

    // ── New: hover events ─────────────────────────────────────────────────────

    #[test]
    fn hover_text() {
        let tooltip = Text::new("Tooltip").gray();
        let t = Text::new("Hover me").hover_text(tooltip);
        let s = t.to_string();
        assert!(s.contains("\"hoverEvent\""), "got: {s}");
        assert!(s.contains("\"show_text\""), "got: {s}");
        assert!(s.contains("Tooltip"), "got: {s}");
    }

    #[test]
    fn hover_item() {
        let t = Text::new("Item").hover_item("minecraft:diamond");
        let s = t.to_string();
        assert!(s.contains("\"show_item\""), "got: {s}");
        assert!(s.contains("minecraft:diamond"), "got: {s}");
    }

    #[test]
    fn hover_item_with_count() {
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Items")
                .hover_item_with_count("minecraft:diamond", 3)
                .to_string(),
        )
        .unwrap();
        assert_eq!(
            value["hoverEvent"],
            serde_json::json!({
                "action": "show_item",
                "id": "minecraft:diamond",
                "count": 3,
            })
        );
    }

    #[test]
    fn hover_item_with_count_preserves_component_fields() {
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Items")
                .blue()
                .hover_item_with_count("example:component_bearing_item", 64)
                .then(Text::new(" available").italic(true))
                .to_string(),
        )
        .unwrap();
        assert_eq!(value["color"], "blue");
        assert_eq!(value["hoverEvent"]["id"], "example:component_bearing_item");
        assert_eq!(value["hoverEvent"]["count"], 64);
        assert_eq!(value["extra"][0]["italic"], true);
    }

    #[test]
    fn hover_entity_uses_typed_id_and_text_name() {
        let id = EntityHoverId::parse("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Zombie")
                .hover_entity_with_id(Zombie, id, Text::new("Undead").red())
                .to_string(),
        )
        .unwrap();
        assert_eq!(
            value["hoverEvent"],
            serde_json::json!({
                "action": "show_entity",
                "type": "minecraft:zombie",
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": {"text": "Undead", "color": "red"},
            })
        );
    }

    #[test]
    fn hover_entity_without_id_omits_id() {
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Zombie")
                .hover_entity(Zombie, Text::new("Undead"))
                .to_string(),
        )
        .unwrap();
        assert_eq!(
            value["hoverEvent"],
            serde_json::json!({
                "action": "show_entity",
                "type": "minecraft:zombie",
                "name": {"text": "Undead"},
            })
        );
    }

    #[test]
    fn entity_hover_id_rejects_malformed_uuid() {
        let error = EntityHoverId::parse("not-a-uuid").unwrap_err();
        assert_eq!(error.helper, "EntityHoverId::parse");
        assert_eq!(error.field, "id");
        assert!(error.message.contains("canonical hyphenated UUID"));
    }

    #[test]
    fn hover_entity_raw_preserves_unchecked_advanced_values() {
        let value: serde_json::Value = serde_json::from_str(
            &Text::new("Custom")
                .hover_entity_raw(
                    "modded:future/entity",
                    Some("version-specific-id".to_owned()),
                    Text::new("Advanced").light_purple(),
                )
                .to_string(),
        )
        .unwrap();
        assert_eq!(value["hoverEvent"]["type"], "modded:future/entity");
        assert_eq!(value["hoverEvent"]["id"], "version-specific-id");
        assert_eq!(value["hoverEvent"]["name"]["color"], "light_purple");
    }

    // ── New: insertion ────────────────────────────────────────────────────────

    #[test]
    fn insertion_field() {
        let t = Text::new("shift+click").insertion("/tell @s hello");
        let s = t.to_string();
        assert!(s.contains("\"insertion\""), "got: {s}");
        assert!(s.contains("/tell @s hello"), "got: {s}");
    }

    // ── Golden output ─────────────────────────────────────────────────────────

    #[test]
    fn golden_clickable_text() {
        let t = Text::new("Click me")
            .green()
            .hover_text(Text::new("Runs a command").gray())
            .click_run_command("/say clicked");
        let s = t.to_string();
        assert!(s.contains("\"text\":\"Click me\""), "got: {s}");
        assert!(s.contains("\"color\":\"green\""), "got: {s}");
        assert!(s.contains("\"hoverEvent\""), "got: {s}");
        assert!(s.contains("\"clickEvent\""), "got: {s}");
    }
}
