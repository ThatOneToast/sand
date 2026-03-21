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

use std::fmt;

// ── ChatColor ─────────────────────────────────────────────────────────────────

/// The 16 standard Minecraft text colors for chat, titles, and JSON text components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
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

// ── TextComponent internals ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum TextContent {
    Literal(String),
    Score {
        name: String,
        objective: String,
    },
    Selector(String),
    Translate {
        key: String,
        with: Vec<TextComponent>,
    },
    Keybind(String),
}

// ── TextComponent ─────────────────────────────────────────────────────────────

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
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    extra: Vec<TextComponent>,
}

impl TextComponent {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// `{"text": "..."}` — render a plain string literal.
    pub fn literal(text: impl Into<String>) -> Self {
        Self::new(TextContent::Literal(text.into()))
    }

    /// `{"score": {"name": "...", "objective": "..."}}` — render a scoreboard value inline.
    pub fn score(name: impl Into<String>, objective: impl Into<String>) -> Self {
        Self::new(TextContent::Score {
            name: name.into(),
            objective: objective.into(),
        })
    }

    /// `{"selector": "..."}` — render the display name(s) of matched entities.
    pub fn selector(selector: impl Into<String>) -> Self {
        Self::new(TextContent::Selector(selector.into()))
    }

    /// `{"translate": "..."}` — a localization key from Minecraft's language files.
    pub fn translate(key: impl Into<String>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with: vec![],
        })
    }

    /// `{"translate": "...", "with": [...]}` — localization key with interpolation arguments.
    pub fn translate_with(key: impl Into<String>, with: Vec<TextComponent>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with,
        })
    }

    /// `{"keybind": "..."}` — display the key currently bound to a Minecraft action.
    pub fn keybind(key: impl Into<String>) -> Self {
        Self::new(TextContent::Keybind(key.into()))
    }

    fn new(content: TextContent) -> Self {
        Self {
            content,
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            extra: vec![],
        }
    }

    // ── Formatting ────────────────────────────────────────────────────────────

    /// Apply a standard Minecraft named color.
    pub fn color(mut self, color: ChatColor) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Apply an arbitrary hex color code (Minecraft 1.16+), e.g. `"#FF5733"`.
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(hex.into());
        self
    }

    /// Set bold formatting.
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }

    /// Set italic formatting.
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }

    /// Set underline formatting.
    pub fn underlined(mut self, v: bool) -> Self {
        self.underlined = Some(v);
        self
    }

    /// Set strikethrough formatting.
    pub fn strikethrough(mut self, v: bool) -> Self {
        self.strikethrough = Some(v);
        self
    }

    /// Set obfuscated (scrambled) text.
    pub fn obfuscated(mut self, v: bool) -> Self {
        self.obfuscated = Some(v);
        self
    }

    /// Append a sibling component in the `"extra"` array.
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
            TextContent::Selector(sel) => serde_json::json!({ "selector": sel }),
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
        if !self.extra.is_empty() {
            let extras: Vec<_> = self.extra.iter().map(|e| e.to_json_value()).collect();
            obj["extra"] = serde_json::json!(extras);
        }
        obj
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
}
