//! Supporting types for command arguments.

use std::fmt;

// ── GameMode ──────────────────────────────────────────────────────────────────

/// Minecraft player game mode.
///
/// Determines the player's interaction rules (creative building, survival challenges, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// `survival` — normal gameplay with health, hunger, and environmental damage.
    Survival,
    /// `creative` — infinite resources, flight, and block placement without cost.
    Creative,
    /// `adventure` — survival-like but players cannot destroy blocks (for map creators).
    Adventure,
    /// `spectator` — observe-only mode; players can see through walls but cannot interact.
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

/// Standard Minecraft text color for chat, titles, and JSON components.
///
/// These are the 16 legacy colors. For arbitrary colors, use hex with `color_hex()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatColor {
    /// Black text.
    Black,
    /// Dark blue text.
    DarkBlue,
    /// Dark green text.
    DarkGreen,
    /// Dark cyan/aqua text.
    DarkAqua,
    /// Dark red text.
    DarkRed,
    /// Dark purple/magenta text.
    DarkPurple,
    /// Gold/orange text.
    Gold,
    /// Light gray text.
    Gray,
    /// Dark gray text.
    DarkGray,
    /// Bright blue text.
    Blue,
    /// Bright green text.
    Green,
    /// Bright cyan/aqua text.
    Aqua,
    /// Bright red text.
    Red,
    /// Bright pink/magenta text.
    LightPurple,
    /// Bright yellow text.
    Yellow,
    /// White text.
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

    /// `{"text": "..."}` — a plain text literal component.
    ///
    /// Renders as the given string. Use builder methods to add formatting.
    pub fn literal(text: impl Into<String>) -> Self {
        Self::new(TextContent::Literal(text.into()))
    }

    /// `{"score": {"name": "...", "objective": "..."}}` — display a scoreboard value inline.
    ///
    /// The `name` can be a selector (`"@s"`, `"@p"`) or a fake player name.
    /// Renders the integer score stored in the objective for that holder.
    ///
    /// # Example
    /// ```
    /// use sand_core::cmd::TextComponent;
    /// let comp = TextComponent::score("@s", "join_count");
    /// // Renders the player's join_count score where they execute the command
    /// ```
    pub fn score(name: impl Into<String>, objective: impl Into<String>) -> Self {
        Self::new(TextContent::Score {
            name: name.into(),
            objective: objective.into(),
        })
    }

    /// `{"selector": "..."}` — display the name(s) of matched entities.
    ///
    /// The selector string (e.g., `"@s"`, `"@a"`) is evaluated, and the display names
    /// of all matched entities are rendered. Multiple matches appear as a comma-separated list.
    pub fn selector(selector: impl Into<String>) -> Self {
        Self::new(TextContent::Selector(selector.into()))
    }

    /// `{"translate": "..."}` — a localization key from Minecraft's language files.
    ///
    /// Renders the translated string for the player's language. No interpolation arguments.
    /// Use [`translate_with`](Self::translate_with) for translations with placeholders.
    pub fn translate(key: impl Into<String>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with: vec![],
        })
    }

    /// `{"translate": "...", "with": [...]}` — localization key with interpolation arguments.
    ///
    /// The `with` vector provides components that are substituted into `%s`, `%1$s`, etc. in the translation.
    pub fn translate_with(key: impl Into<String>, with: Vec<TextComponent>) -> Self {
        Self::new(TextContent::Translate {
            key: key.into(),
            with,
        })
    }

    /// `{"keybind": "..."}` — display the key bound to a Minecraft action.
    ///
    /// Example: `keybind("key.jump")` renders as `[SPACE]` for default bindings.
    /// Use Minecraft keybind IDs like `"key.attack"`, `"key.sneak"`, etc.
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

    /// Set the text color using a standard Minecraft color.
    ///
    /// Produces `"color":"<color_name>"` in the JSON output.
    pub fn color(mut self, color: ChatColor) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Set the text color using a hex color code (Minecraft 1.16+).
    ///
    /// Example: `"#FF5733"` for orange. Produces `"color":"#FF5733"` in JSON.
    pub fn color_hex(mut self, hex: impl Into<String>) -> Self {
        self.color = Some(hex.into());
        self
    }

    /// Set bold formatting on or off.
    ///
    /// Produces `"bold":true` or `"bold":false`.
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = Some(v);
        self
    }

    /// Set italic formatting on or off.
    ///
    /// Produces `"italic":true` or `"italic":false`.
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = Some(v);
        self
    }

    /// Set underline formatting on or off.
    ///
    /// Produces `"underlined":true` or `"underlined":false`.
    pub fn underlined(mut self, v: bool) -> Self {
        self.underlined = Some(v);
        self
    }

    /// Set strikethrough formatting on or off.
    ///
    /// Produces `"strikethrough":true` or `"strikethrough":false`.
    pub fn strikethrough(mut self, v: bool) -> Self {
        self.strikethrough = Some(v);
        self
    }

    /// Set obfuscated (scrambled) text on or off.
    ///
    /// Text displays as random characters but can still be selected/copied as the original.
    /// Produces `"obfuscated":true` or `"obfuscated":false`.
    pub fn obfuscated(mut self, v: bool) -> Self {
        self.obfuscated = Some(v);
        self
    }

    /// Append another text component in the `extra` array.
    ///
    /// Enables building multi-segment messages without raw JSON. Each `then()` adds
    /// a sibling component that inherits formatting from the parent unless overridden.
    ///
    /// # Example
    /// ```
    /// use sand_core::cmd::{TextComponent, ChatColor};
    /// let msg = TextComponent::literal("Score: ")
    ///     .color(ChatColor::White)
    ///     .then(TextComponent::score("@s", "kills").color(ChatColor::Red));
    /// // Renders as: "Score: " + the player's kills score in red
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

/// Entity anchor point for eye-level or foot-level calculations.
///
/// Used in `execute anchored` and `execute facing entity` to specify which part
/// of an entity is the reference point (e.g., where they look from vs. where they stand).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The entity's eye level (head/face).
    Eyes,
    /// The entity's feet level (bottom).
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

/// Axis combination for `execute align` — specifies which axes to floor to block boundaries.
#[derive(Debug, Clone)]
pub struct Swizzle(String);

impl Swizzle {
    /// `x` — floor the X coordinate only.
    pub fn x() -> Self {
        Swizzle("x".into())
    }

    /// `y` — floor the Y coordinate only.
    pub fn y() -> Self {
        Swizzle("y".into())
    }

    /// `z` — floor the Z coordinate only.
    pub fn z() -> Self {
        Swizzle("z".into())
    }

    /// `xy` — floor both X and Y coordinates.
    pub fn xy() -> Self {
        Swizzle("xy".into())
    }

    /// `xz` — floor both X and Z coordinates.
    pub fn xz() -> Self {
        Swizzle("xz".into())
    }

    /// `yz` — floor both Y and Z coordinates.
    pub fn yz() -> Self {
        Swizzle("yz".into())
    }

    /// `xyz` — floor all three coordinates to the block grid.
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

/// A scoreboard score holder: an entity selector or a fake player name.
///
/// Score holders are the subjects of scoreboard operations (e.g., who gets or increments a score).
/// Can be a single player, all matching entities, or a named fake player for global counters.
#[derive(Debug, Clone)]
pub struct ScoreHolder(String);

impl ScoreHolder {
    /// Create a score holder from an entity selector.
    ///
    /// Example: `ScoreHolder::entity(Selector::self_())` → `"@s"`.
    pub fn entity(selector: super::Selector) -> Self {
        ScoreHolder(selector.to_string())
    }

    /// Create a score holder from a fake player name (for global counters).
    ///
    /// Fake players have no existence in the world; they exist only for scoreboard storage.
    /// Example: `ScoreHolder::fake("total_kills")` → `"total_kills"`.
    pub fn fake(name: impl Into<String>) -> Self {
        ScoreHolder(name.into())
    }

    /// `*` — all score holders that have at least one score in this objective.
    ///
    /// Use in operations to affect all holders at once (e.g., reset all scores).
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

/// Scoreboard arithmetic operation for score comparisons and modifications.
///
/// Used in `execute if score` comparisons and `scoreboard players operation` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOp {
    /// `+=` — add: `a += b` (a becomes a + b).
    Add,
    /// `-=` — subtract: `a -= b` (a becomes a - b).
    Sub,
    /// `*=` — multiply: `a *= b` (a becomes a * b).
    Mul,
    /// `/=` — divide: `a /= b` (a becomes a / b, truncated).
    Div,
    /// `%=` — modulo: `a %= b` (a becomes a mod b).
    Mod,
    /// `=` — assign: `a = b` (a becomes b).
    Set,
    /// `<` — minimum: `a < b` (a becomes min(a, b)).
    Min,
    /// `>` — maximum: `a > b` (a becomes max(a, b)).
    Max,
    /// `><` — swap: `a >< b` (exchange values of a and b).
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

// ── ScoreCmp ──────────────────────────────────────────────────────────────────

/// Comparison operator for `execute if score … <op> … ` conditions.
///
/// **Distinct from [`ScoreOp`]** — `ScoreOp` is for `scoreboard players operation`
/// (arithmetic that modifies scores). `ScoreCmp` is read-only: it tests two scores
/// without changing either.
///
/// # Examples
/// ```rust,ignore
/// Execute::new()
///     .if_score_compare("@s", "mana", ScoreCmp::Ge, "@s", "max_mana")
///     .run_raw("say full mana");
///
/// // Or use the named shorthand:
/// Execute::new()
///     .if_score_lt("@s", "health", "#const", "ten")
///     .run_raw("say low health");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreCmp {
    /// `=` — left score equals right.
    Eq,
    /// `<` — left score is strictly less than right.
    Lt,
    /// `<=` — left score is less than or equal to right.
    Le,
    /// `>` — left score is strictly greater than right.
    Gt,
    /// `>=` — left score is greater than or equal to right.
    Ge,
}

impl fmt::Display for ScoreCmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScoreCmp::Eq => "=",
            ScoreCmp::Lt => "<",
            ScoreCmp::Le => "<=",
            ScoreCmp::Gt => ">",
            ScoreCmp::Ge => ">=",
        };
        write!(f, "{s}")
    }
}

// ── NbtStoreKind ──────────────────────────────────────────────────────────────

/// The NBT data type used when storing a value via `execute store result/success … nbt`.
///
/// Minecraft truncates or rounds the stored value to fit the chosen type.
/// Use `Double` or `Float` when storing fractional values (with a `scale` factor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtStoreKind {
    /// Store as `byte` (`0b`–`127b`). Values outside `i8` range are truncated.
    Byte,
    /// Store as `short` (`0s`–`32767s`). Values outside `i16` range are truncated.
    Short,
    /// Store as `int`. Values outside `i32` range are truncated.
    Int,
    /// Store as `long`. Full `i64` range.
    Long,
    /// Store as `float`. Result is multiplied by scale, then cast.
    Float,
    /// Store as `double`. Highest precision floating-point storage.
    Double,
}

impl fmt::Display for NbtStoreKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NbtStoreKind::Byte => "byte",
            NbtStoreKind::Short => "short",
            NbtStoreKind::Int => "int",
            NbtStoreKind::Long => "long",
            NbtStoreKind::Float => "float",
            NbtStoreKind::Double => "double",
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
