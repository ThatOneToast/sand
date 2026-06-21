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

// ── Click / Hover events ──────────────────────────────────────────────────────

/// A click event attached to a [`TextComponent`].
#[derive(Debug, Clone)]
pub enum ClickEvent {
    /// Execute a command when clicked.
    RunCommand(String),
    /// Fill the chat bar with a command suggestion.
    SuggestCommand(String),
    /// Open a URL in the player's browser.
    OpenUrl(String),
    /// Copy text to the clipboard.
    CopyToClipboard(String),
    /// Turn to a book page (book items only).
    ChangePage(u32),
}

/// A hover event attached to a [`TextComponent`].
#[derive(Debug, Clone)]
pub enum HoverEvent {
    /// Show another text component as a tooltip.
    ShowText(Box<TextComponent>),
    /// Show an item tooltip.
    ShowItem { id: String, count: Option<u32> },
    /// Show an entity tooltip.
    ShowEntity {
        name: String,
        entity_type: String,
        id: Option<String>,
    },
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
    insertion: Option<String>,
    click_event: Option<ClickEvent>,
    hover_event: Option<HoverEvent>,
    extra: Vec<TextComponent>,
}

// ── Text (ergonomic alias) ────────────────────────────────────────────────────

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
    #[allow(clippy::new_ret_no_self)]
    pub fn new(s: impl Into<String>) -> TextComponent {
        TextComponent::literal(s)
    }

    /// Embed a pre-serialized JSON string directly (escape hatch).
    ///
    /// No formatting is applied — the string is returned as-is.
    pub fn raw_json(json: impl Into<String>) -> String {
        json.into()
    }
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
            insertion: None,
            click_event: None,
            hover_event: None,
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

    // ── Ergonomic color shortcuts ─────────────────────────────────────────────

    /// Apply `ChatColor::Black`.
    pub fn black(self) -> Self {
        self.color(ChatColor::Black)
    }
    /// Apply `ChatColor::DarkBlue`.
    pub fn dark_blue(self) -> Self {
        self.color(ChatColor::DarkBlue)
    }
    /// Apply `ChatColor::DarkGreen`.
    pub fn dark_green(self) -> Self {
        self.color(ChatColor::DarkGreen)
    }
    /// Apply `ChatColor::DarkAqua`.
    pub fn dark_aqua(self) -> Self {
        self.color(ChatColor::DarkAqua)
    }
    /// Apply `ChatColor::DarkRed`.
    pub fn dark_red(self) -> Self {
        self.color(ChatColor::DarkRed)
    }
    /// Apply `ChatColor::DarkPurple`.
    pub fn dark_purple(self) -> Self {
        self.color(ChatColor::DarkPurple)
    }
    /// Apply `ChatColor::Gold`.
    pub fn gold(self) -> Self {
        self.color(ChatColor::Gold)
    }
    /// Apply `ChatColor::Gray`.
    pub fn gray(self) -> Self {
        self.color(ChatColor::Gray)
    }
    /// Apply `ChatColor::DarkGray`.
    pub fn dark_gray(self) -> Self {
        self.color(ChatColor::DarkGray)
    }
    /// Apply `ChatColor::Blue`.
    pub fn blue(self) -> Self {
        self.color(ChatColor::Blue)
    }
    /// Apply `ChatColor::Green`.
    pub fn green(self) -> Self {
        self.color(ChatColor::Green)
    }
    /// Apply `ChatColor::Aqua`.
    pub fn aqua(self) -> Self {
        self.color(ChatColor::Aqua)
    }
    /// Apply `ChatColor::Red`.
    pub fn red(self) -> Self {
        self.color(ChatColor::Red)
    }
    /// Apply `ChatColor::LightPurple`.
    pub fn light_purple(self) -> Self {
        self.color(ChatColor::LightPurple)
    }
    /// Apply `ChatColor::Yellow`.
    pub fn yellow(self) -> Self {
        self.color(ChatColor::Yellow)
    }
    /// Apply `ChatColor::White`.
    pub fn white(self) -> Self {
        self.color(ChatColor::White)
    }

    // ── Text formatting ───────────────────────────────────────────────────────

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

    /// Set the `insertion` string — shift-clicking inserts this into the chat bar.
    pub fn insertion(mut self, text: impl Into<String>) -> Self {
        self.insertion = Some(text.into());
        self
    }

    // ── Click events ──────────────────────────────────────────────────────────

    /// Run a command when this text is clicked.
    pub fn click_run_command(mut self, cmd: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::RunCommand(cmd.into()));
        self
    }

    /// Fill the chat bar with a suggestion when clicked.
    pub fn click_suggest_command(mut self, cmd: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::SuggestCommand(cmd.into()));
        self
    }

    /// Open a URL when clicked.
    pub fn click_open_url(mut self, url: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::OpenUrl(url.into()));
        self
    }

    /// Copy text to the clipboard when clicked.
    pub fn click_copy(mut self, text: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent::CopyToClipboard(text.into()));
        self
    }

    // ── Hover events ──────────────────────────────────────────────────────────

    /// Show another `TextComponent` as a tooltip on hover.
    pub fn hover_text(mut self, text: TextComponent) -> Self {
        self.hover_event = Some(HoverEvent::ShowText(Box::new(text)));
        self
    }

    /// Show an item tooltip on hover.
    pub fn hover_item(mut self, item_id: impl Into<String>) -> Self {
        self.hover_event = Some(HoverEvent::ShowItem {
            id: item_id.into(),
            count: None,
        });
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
                HoverEvent::ShowText(t) => {
                    serde_json::json!({"action": "show_text", "contents": t.to_json_value()})
                }
                HoverEvent::ShowItem { id, count } => {
                    let mut h = serde_json::json!({"action": "show_item", "id": id});
                    if let Some(c) = count {
                        h["count"] = serde_json::json!(c);
                    }
                    h
                }
                HoverEvent::ShowEntity {
                    name,
                    entity_type,
                    id,
                } => {
                    let mut h = serde_json::json!({"action": "show_entity", "name": name, "type": entity_type});
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
