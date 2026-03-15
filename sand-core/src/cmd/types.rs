//! Supporting types for command arguments.

use std::fmt;

// ── GameMode ──────────────────────────────────────────────────────────────────

/// Minecraft game mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameMode::Survival => write!(f, "survival"),
            GameMode::Creative => write!(f, "creative"),
            GameMode::Adventure => write!(f, "adventure"),
            GameMode::Spectator => write!(f, "spectator"),
        }
    }
}

// ── ChatColor ─────────────────────────────────────────────────────────────────

/// Minecraft chat/text color.
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

// ── TextComponent ─────────────────────────────────────────────────────────────

/// The content kind of a [`TextComponent`].
///
/// Minecraft JSON text has several mutually exclusive content sources.
#[derive(Debug, Clone)]
enum TextContent {
    /// `{"text": "..."}` — a raw string literal.
    Literal(String),
    /// `{"score": {"name": "...", "objective": "..."}}` — inline scoreboard value.
    Score { name: String, objective: String },
    /// `{"selector": "..."}` — renders the matched entity's display name(s).
    Selector(String),
    /// `{"translate": "...", "with": [...]}` — localisation key.
    Translate {
        key: String,
        with: Vec<TextComponent>,
    },
    /// `{"keybind": "..."}` — shows the key bound to an action (e.g. `"key.jump"`).
    Keybind(String),
}

/// A Minecraft JSON text component for `tellraw`, `title`, `bossbar`, etc.
///
/// Construct with one of the factory methods, chain formatting, then chain
/// additional segments with [`.then()`](TextComponent::then).
///
/// # Examples
/// ```
/// use sand_core::cmd::{TextComponent, ChatColor};
///
/// // Plain literal
/// let t = TextComponent::literal("Hello!")
///     .color(ChatColor::Gold)
///     .bold(true);
/// assert!(t.to_string().contains("\"text\":\"Hello!\""));
///
/// // Inline scoreboard value
/// let score = TextComponent::score("@s", "join_count")
///     .color(ChatColor::Aqua);
/// assert!(score.to_string().contains("\"score\""));
///
/// // Composite: "You have joined X times!"
/// let msg = TextComponent::literal("You have joined ")
///     .color(ChatColor::Yellow)
///     .then(TextComponent::score("@s", "join_count").color(ChatColor::Aqua))
///     .then(TextComponent::literal(" times!").color(ChatColor::Yellow));
/// assert!(msg.to_string().contains("\"extra\""));
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

    /// `{"text": "..."}` — a plain string.
    pub fn literal(text: impl Into<String>) -> Self {
        Self::new(TextContent::Literal(text.into()))
    }

    /// `{"score": {"name": "...", "objective": "..."}}` — inline scoreboard value.
    ///
    /// `name` accepts a selector (`"@s"`, `"@p"`) or a fake player name.
    /// The rendered value is the integer stored in `objective` for that holder.
    ///
    /// ```
    /// use sand_core::cmd::TextComponent;
    /// let s = TextComponent::score("@s", "join_count").to_string();
    /// assert!(s.contains("\"score\""));
    /// assert!(s.contains("\"name\":\"@s\""));
    /// assert!(s.contains("\"objective\":\"join_count\""));
    /// ```
    pub fn score(name: impl Into<String>, objective: impl Into<String>) -> Self {
        Self::new(TextContent::Score {
            name: name.into(),
            objective: objective.into(),
        })
    }

    /// `{"selector": "..."}` — renders the display name(s) of matched entities.
    ///
    /// `selector` is a raw selector string such as `"@s"` or `"@a"`.
    pub fn selector(selector: impl Into<String>) -> Self {
        Self::new(TextContent::Selector(selector.into()))
    }

    /// `{"translate": "..."}` — a localisation key.
    ///
    /// Use [`TextComponent::translate_with`] when the translation includes
    /// positional arguments.
    pub fn translate(key: impl Into<String>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with: vec![],
        })
    }

    /// `{"translate": "...", "with": [...]}` — a localisation key with arguments.
    pub fn translate_with(key: impl Into<String>, with: Vec<TextComponent>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with,
        })
    }

    /// `{"keybind": "..."}` — shows the key bound to an action.
    ///
    /// Example: `TextComponent::keybind("key.jump")` renders as `[SPACE]` for
    /// a player with the default bindings.
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

    pub fn color(mut self, color: ChatColor) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Hex color, e.g. `"#FF5733"`. Requires Minecraft 1.16+.
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(hex.into());
        self
    }

    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }
    pub fn underlined(mut self, v: bool) -> Self {
        self.underlined = Some(v);
        self
    }
    pub fn strikethrough(mut self, v: bool) -> Self {
        self.strikethrough = Some(v);
        self
    }
    pub fn obfuscated(mut self, v: bool) -> Self {
        self.obfuscated = Some(v);
        self
    }

    /// Append another component in the `extra` array.
    ///
    /// Enables building multi-segment messages without raw JSON strings:
    /// ```
    /// use sand_core::cmd::{TextComponent, ChatColor};
    /// let msg = TextComponent::literal("Score: ")
    ///     .then(TextComponent::score("@s", "kills").color(ChatColor::Red));
    /// assert!(msg.to_string().contains("\"extra\""));
    /// ```
    pub fn then(mut self, next: TextComponent) -> Self {
        self.extra.push(next);
        self
    }

    // ── Serialisation ─────────────────────────────────────────────────────────

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_json_value())
    }
}

// ── Anchor ────────────────────────────────────────────────────────────────────

/// Entity anchor point for `execute anchored` and `execute facing entity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Eyes,
    Feet,
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anchor::Eyes => write!(f, "eyes"),
            Anchor::Feet => write!(f, "feet"),
        }
    }
}

// ── Swizzle ───────────────────────────────────────────────────────────────────

/// Axis combination for `execute align`.
#[derive(Debug, Clone)]
pub struct Swizzle(String);

impl Swizzle {
    pub fn x() -> Self {
        Swizzle("x".into())
    }
    pub fn y() -> Self {
        Swizzle("y".into())
    }
    pub fn z() -> Self {
        Swizzle("z".into())
    }
    pub fn xy() -> Self {
        Swizzle("xy".into())
    }
    pub fn xz() -> Self {
        Swizzle("xz".into())
    }
    pub fn yz() -> Self {
        Swizzle("yz".into())
    }
    pub fn xyz() -> Self {
        Swizzle("xyz".into())
    }
}

impl fmt::Display for Swizzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── ScoreHolder ───────────────────────────────────────────────────────────────

/// A scoreboard score holder: either an entity selector or a fake player name.
#[derive(Debug, Clone)]
pub struct ScoreHolder(String);

impl ScoreHolder {
    pub fn entity(selector: super::Selector) -> Self {
        ScoreHolder(selector.to_string())
    }
    pub fn fake(name: impl Into<String>) -> Self {
        ScoreHolder(name.into())
    }
    /// `*` — all score holders with at least one score.
    pub fn all() -> Self {
        ScoreHolder("*".into())
    }
}

impl fmt::Display for ScoreHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── ScoreOp ───────────────────────────────────────────────────────────────────

/// Scoreboard arithmetic operation (`+=`, `-=`, `*=`, `/=`, `%=`, `=`, `<`, `>`, `><`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Set,
    Min,
    Max,
    Swap,
}

impl fmt::Display for ScoreOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScoreOp::Add => "+=",
            ScoreOp::Sub => "-=",
            ScoreOp::Mul => "*=",
            ScoreOp::Div => "/=",
            ScoreOp::Mod => "%=",
            ScoreOp::Set => "=",
            ScoreOp::Min => "<",
            ScoreOp::Max => ">",
            ScoreOp::Swap => "><",
        };
        write!(f, "{s}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamemode_display() {
        assert_eq!(GameMode::Survival.to_string(), "survival");
        assert_eq!(GameMode::Creative.to_string(), "creative");
    }

    #[test]
    fn chat_color_display() {
        assert_eq!(ChatColor::Gold.to_string(), "gold");
        assert_eq!(ChatColor::DarkBlue.to_string(), "dark_blue");
    }

    #[test]
    fn text_component_json() {
        let t = TextComponent::literal("Hi!")
            .color(ChatColor::Gold)
            .bold(true);
        let s = t.to_string();
        assert!(s.contains("\"text\":\"Hi!\""));
        assert!(s.contains("\"color\":\"gold\""));
        assert!(s.contains("\"bold\":true"));
    }

    #[test]
    fn score_op_display() {
        assert_eq!(ScoreOp::Add.to_string(), "+=");
        assert_eq!(ScoreOp::Swap.to_string(), "><");
    }
}
