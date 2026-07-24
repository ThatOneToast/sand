//! Structured builders for `title`, actionbar, and `bossbar` commands.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;
use crate::text::TextComponent;

/// Builder for a title payload and its timing command.
#[derive(Debug, Clone)]
pub struct Title {
    selector: Selector,
    title: Option<TextComponent>,
    subtitle: Option<TextComponent>,
    actionbar: Option<TextComponent>,
    fade_in: u32,
    stay: u32,
    fade_out: u32,
}

impl Title {
    /// Create a payload-oriented title builder.
    pub fn of(selector: Selector) -> Self {
        Self {
            selector,
            title: None,
            subtitle: None,
            actionbar: None,
            fade_in: 10,
            stay: 70,
            fade_out: 20,
        }
    }

    pub fn title(mut self, text: TextComponent) -> Self {
        self.title = Some(text);
        self
    }

    pub fn subtitle(mut self, text: TextComponent) -> Self {
        self.subtitle = Some(text);
        self
    }

    pub fn actionbar(mut self, text: TextComponent) -> Self {
        self.actionbar = Some(text);
        self
    }

    pub fn times(mut self, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        self.fade_in = fade_in;
        self.stay = stay;
        self.fade_out = fade_out;
        self
    }

    /// Validate and render all commands. Empty payload builders are rejected.
    pub fn try_build(&self) -> CommandResult<Vec<String>> {
        self.validate(&CommandProfile::unprofiled())?;
        Ok(self.render_lines(true))
    }

    /// Compatibility renderer. Lines retain their typed node for export-time validation.
    pub fn build(self) -> Vec<String> {
        let lines = self.render_lines(true);
        for line in &lines {
            register_line(line, DisplayCommand::Title(Box::new(self.clone())));
        }
        lines
    }

    fn render_lines(&self, include_times: bool) -> Vec<String> {
        let mut lines = Vec::new();
        if include_times {
            lines.push(format!(
                "title {} times {} {} {}",
                self.selector, self.fade_in, self.stay, self.fade_out
            ));
        }
        if let Some(text) = &self.subtitle {
            lines.push(format!("title {} subtitle {}", self.selector, text));
        }
        if let Some(text) = &self.title {
            lines.push(format!("title {} title {}", self.selector, text));
        }
        if let Some(text) = &self.actionbar {
            lines.push(format!("title {} actionbar {}", self.selector, text));
        }
        lines
    }

    pub fn clear(selector: Selector) -> String {
        TitleCommand::Clear(selector).build_registered()
    }

    pub fn reset(selector: Selector) -> String {
        TitleCommand::Reset(selector).build_registered()
    }
}

impl Validate for Title {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.selector
            .validate(profile)
            .map_err(|error| display_error("SAND-DISPLAY-TARGET", "target", error.to_string()))?;
        if self.title.is_none() && self.subtitle.is_none() && self.actionbar.is_none() {
            return Err(display_error(
                "SAND-DISPLAY-TITLE-EMPTY",
                "payload",
                "Title requires a title, subtitle, or actionbar payload; use `TitleTimes` for timing-only commands",
            ));
        }
        for (field, text) in [
            ("title", self.title.as_ref()),
            ("subtitle", self.subtitle.as_ref()),
            ("actionbar", self.actionbar.as_ref()),
        ] {
            if let Some(text) = text {
                text.validate_at_path(profile, field)?;
            }
        }
        Ok(())
    }
}

/// Explicit timing-only title command.
#[derive(Debug, Clone)]
pub struct TitleTimes {
    selector: Selector,
    fade_in: u32,
    stay: u32,
    fade_out: u32,
}

impl TitleTimes {
    pub fn new(selector: Selector, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        Self {
            selector,
            fade_in,
            stay,
            fade_out,
        }
    }

    pub fn build(self) -> String {
        TitleCommand::Times(self).build_registered()
    }
}

#[derive(Debug, Clone)]
enum TitleCommand {
    Times(TitleTimes),
    Clear(Selector),
    Reset(Selector),
    Actionbar {
        selector: Selector,
        text: Box<TextComponent>,
    },
    RawActionbar {
        selector: String,
        json: String,
    },
}

impl TitleCommand {
    fn build_registered(self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, DisplayCommand::TitleCommand(Box::new(self)));
        line
    }
}

impl Validate for TitleCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Times(times) => times.selector.validate(profile),
            Self::Clear(selector) | Self::Reset(selector) => selector.validate(profile),
            Self::Actionbar { selector, text } => {
                selector.validate(profile)?;
                text.validate_at_path(profile, "actionbar")
            }
            Self::RawActionbar { .. } => Ok(()),
        }
        .map_err(|error| {
            if error.code.starts_with("SAND-TEXT-") {
                error
            } else {
                display_error("SAND-DISPLAY-TARGET", error.field, error.message)
            }
        })
    }
}

impl RenderCommand for TitleCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        match self {
            Self::Times(times) => format!(
                "title {} times {} {} {}",
                times.selector, times.fade_in, times.stay, times.fade_out
            ),
            Self::Clear(selector) => format!("title {selector} clear"),
            Self::Reset(selector) => format!("title {selector} reset"),
            Self::Actionbar { selector, text } => format!("title {selector} actionbar {text}"),
            Self::RawActionbar { selector, json } => {
                format!("title {selector} actionbar {json}")
            }
        }
    }
}

/// Actionbar command helpers.
pub struct Actionbar;

impl Actionbar {
    pub fn show(selector: Selector, text: TextComponent) -> String {
        TitleCommand::Actionbar {
            selector,
            text: Box::new(text),
        }
        .build_registered()
    }

    /// Opaque selector/JSON escape hatch.
    pub fn show_raw(selector: impl fmt::Display, json: impl fmt::Display) -> String {
        TitleCommand::RawActionbar {
            selector: selector.to_string(),
            json: json.to_string(),
        }
        .build_registered()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarColor {
    Blue,
    Green,
    Pink,
    Purple,
    Red,
    White,
    Yellow,
}

impl fmt::Display for BossbarColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Pink => "pink",
            Self::Purple => "purple",
            Self::Red => "red",
            Self::White => "white",
            Self::Yellow => "yellow",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarStyle {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

impl fmt::Display for BossbarStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Progress => "progress",
            Self::Notched6 => "notched_6",
            Self::Notched10 => "notched_10",
            Self::Notched12 => "notched_12",
            Self::Notched20 => "notched_20",
        })
    }
}

/// Canonical validated bossbar resource location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BossbarId {
    value: String,
    raw: bool,
}

impl BossbarId {
    pub fn parse(value: impl Into<String>) -> CommandResult<Self> {
        let value = value.into();
        crate::validate::resource_location_shape(&value, "BossbarId", "id")
            .map_err(|error| display_error("SAND-BOSSBAR-ID", "id", error.message))?;
        Ok(Self { value, raw: false })
    }

    pub fn raw(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            raw: true,
        }
    }

    fn compatibility(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            raw: false,
        }
    }
}

impl fmt::Display for BossbarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl From<&str> for BossbarId {
    fn from(value: &str) -> Self {
        Self::compatibility(value)
    }
}

impl From<String> for BossbarId {
    fn from(value: String) -> Self {
        Self::compatibility(value)
    }
}

/// Conversion into a bossbar resource-location token.
pub trait IntoBossbarId {
    fn into_bossbar_id(self) -> BossbarId;
}

impl IntoBossbarId for BossbarId {
    fn into_bossbar_id(self) -> BossbarId {
        self
    }
}

impl IntoBossbarId for String {
    fn into_bossbar_id(self) -> BossbarId {
        BossbarId::compatibility(self)
    }
}

impl IntoBossbarId for &str {
    fn into_bossbar_id(self) -> BossbarId {
        BossbarId::compatibility(self)
    }
}

/// Typed bossbar terminal command.
#[derive(Debug, Clone)]
pub enum BossbarCommand {
    Add { id: BossbarId, name: TextComponent },
    Remove { id: BossbarId },
    List,
    SetName { id: BossbarId, name: TextComponent },
    SetColor { id: BossbarId, color: BossbarColor },
    SetStyle { id: BossbarId, style: BossbarStyle },
    SetValue { id: BossbarId, value: u32 },
    SetMax { id: BossbarId, max: u32 },
    SetVisible { id: BossbarId, visible: bool },
    SetPlayers { id: BossbarId, players: Selector },
    Get { id: BossbarId, field: &'static str },
}

impl BossbarCommand {
    fn build_registered(self) -> String {
        let line = self.render_unchecked(&CommandProfile::unprofiled());
        register_line(&line, DisplayCommand::Bossbar(Box::new(self)));
        line
    }

    fn id(&self) -> Option<&BossbarId> {
        match self {
            Self::Add { id, .. }
            | Self::Remove { id }
            | Self::SetName { id, .. }
            | Self::SetColor { id, .. }
            | Self::SetStyle { id, .. }
            | Self::SetValue { id, .. }
            | Self::SetMax { id, .. }
            | Self::SetVisible { id, .. }
            | Self::SetPlayers { id, .. }
            | Self::Get { id, .. } => Some(id),
            Self::List => None,
        }
    }
}

impl Validate for BossbarCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        if let Some(id) = self.id()
            && !id.raw
        {
            crate::validate::resource_location_shape(&id.value, "BossbarCommand", "id")
                .map_err(|error| display_error("SAND-BOSSBAR-ID", "id", error.message))?;
        }
        match self {
            Self::Add { name, .. } | Self::SetName { name, .. } => {
                name.validate_at_path(profile, "bossbar.name")?
            }
            Self::SetMax { max: 0, .. } => {
                return Err(display_error(
                    "SAND-BOSSBAR-MAX",
                    "max",
                    "bossbar maximum must be greater than zero",
                ));
            }
            Self::SetPlayers { players, .. } => players.validate(profile).map_err(|error| {
                display_error("SAND-BOSSBAR-PLAYERS", "players", error.to_string())
            })?,
            _ => {}
        }
        Ok(())
    }
}

impl RenderCommand for BossbarCommand {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        match self {
            Self::Add { id, name } => format!("bossbar add {id} {name}"),
            Self::Remove { id } => format!("bossbar remove {id}"),
            Self::List => "bossbar list".to_string(),
            Self::SetName { id, name } => format!("bossbar set {id} name {name}"),
            Self::SetColor { id, color } => format!("bossbar set {id} color {color}"),
            Self::SetStyle { id, style } => format!("bossbar set {id} style {style}"),
            Self::SetValue { id, value } => format!("bossbar set {id} value {value}"),
            Self::SetMax { id, max } => format!("bossbar set {id} max {max}"),
            Self::SetVisible { id, visible } => format!("bossbar set {id} visible {visible}"),
            Self::SetPlayers { id, players } => format!("bossbar set {id} players {players}"),
            Self::Get { id, field } => format!("bossbar get {id} {field}"),
        }
    }
}

pub struct Bossbar;

impl Bossbar {
    pub fn add(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::Add {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    pub fn remove(id: impl IntoBossbarId) -> String {
        BossbarCommand::Remove {
            id: id.into_bossbar_id(),
        }
        .build_registered()
    }
    pub fn list() -> String {
        BossbarCommand::List.build_registered()
    }
    pub fn set_value(id: impl IntoBossbarId, value: u32) -> String {
        BossbarCommand::SetValue {
            id: id.into_bossbar_id(),
            value,
        }
        .build_registered()
    }
    pub fn set_max(id: impl IntoBossbarId, max: u32) -> String {
        BossbarCommand::SetMax {
            id: id.into_bossbar_id(),
            max,
        }
        .build_registered()
    }
    pub fn set_players(id: impl IntoBossbarId, players: Selector) -> String {
        BossbarCommand::SetPlayers {
            id: id.into_bossbar_id(),
            players,
        }
        .build_registered()
    }
    pub fn set_color(id: impl IntoBossbarId, color: BossbarColor) -> String {
        BossbarCommand::SetColor {
            id: id.into_bossbar_id(),
            color,
        }
        .build_registered()
    }
    pub fn set_style(id: impl IntoBossbarId, style: BossbarStyle) -> String {
        BossbarCommand::SetStyle {
            id: id.into_bossbar_id(),
            style,
        }
        .build_registered()
    }
    pub fn set_name(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::SetName {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    pub fn set_visible(id: impl IntoBossbarId, visible: bool) -> String {
        BossbarCommand::SetVisible {
            id: id.into_bossbar_id(),
            visible,
        }
        .build_registered()
    }
    pub fn get_value(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "value",
        }
        .build_registered()
    }
    pub fn get_max(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "max",
        }
        .build_registered()
    }
    pub fn get_players(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "players",
        }
        .build_registered()
    }
}

#[derive(Debug, Clone)]
enum DisplayCommand {
    Title(Box<Title>),
    TitleCommand(Box<TitleCommand>),
    Bossbar(Box<BossbarCommand>),
}

impl Validate for DisplayCommand {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match self {
            Self::Title(command) => command.validate(profile),
            Self::TitleCommand(command) => command.validate(profile),
            Self::Bossbar(command) => command.validate(profile),
        }
    }
}

fn display_error(
    code: &'static str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> CommandError {
    CommandError::new("DisplayCommand", field, message).with_code(code)
}

fn registered_lines() -> &'static Mutex<BTreeMap<String, DisplayCommand>> {
    static LINES: OnceLock<Mutex<BTreeMap<String, DisplayCommand>>> = OnceLock::new();
    LINES.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_line(line: &str, command: DisplayCommand) {
    registered_lines()
        .lock()
        .expect("display command registry mutex poisoned")
        .insert(line.to_owned(), command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    registered_lines()
        .lock()
        .expect("display command registry mutex poisoned")
        .get(line)
        .cloned()
        .map_or(Ok(()), |command| command.validate(profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_build_order_and_exact_output() {
        let lines = Title::of(Selector::all_players())
            .title(TextComponent::literal("Hello"))
            .subtitle(TextComponent::literal("World"))
            .times(5, 40, 10)
            .try_build()
            .unwrap();
        assert_eq!(lines[0], "title @a times 5 40 10");
        assert_eq!(lines[1], r#"title @a subtitle {"text":"World"}"#);
        assert_eq!(lines[2], r#"title @a title {"text":"Hello"}"#);
    }

    #[test]
    fn empty_title_is_rejected_but_times_is_explicit() {
        assert!(Title::of(Selector::self_()).try_build().is_err());
        assert_eq!(
            TitleTimes::new(Selector::self_(), 10, 70, 20).build(),
            "title @s times 10 70 20"
        );
    }

    #[test]
    fn actionbar_exact_output() {
        assert_eq!(
            Actionbar::show(Selector::self_(), TextComponent::literal("5 HP left")),
            r#"title @s actionbar {"text":"5 HP left"}"#
        );
    }

    #[test]
    fn bossbar_full_surface() {
        let id = BossbarId::parse("my_pack:boss").unwrap();
        assert_eq!(
            Bossbar::add(id.clone(), TextComponent::literal("Boss")),
            r#"bossbar add my_pack:boss {"text":"Boss"}"#
        );
        assert_eq!(
            Bossbar::set_max(id.clone(), 100),
            "bossbar set my_pack:boss max 100"
        );
        assert_eq!(
            Bossbar::set_players(id.clone(), Selector::all_players()),
            "bossbar set my_pack:boss players @a"
        );
        assert_eq!(Bossbar::remove(id), "bossbar remove my_pack:boss");
        assert_eq!(Bossbar::list(), "bossbar list");
    }

    #[test]
    fn malformed_bossbar_id_and_nested_text_are_rejected() {
        let bad = BossbarCommand::Remove {
            id: BossbarId::compatibility("Boss Bar"),
        };
        assert_eq!(
            bad.validate(&CommandProfile::unprofiled())
                .unwrap_err()
                .code,
            "SAND-BOSSBAR-ID"
        );
        let bad_name = BossbarCommand::Add {
            id: BossbarId::parse("pack:boss").unwrap(),
            name: TextComponent::literal("bad").color_hex("#12FG00"),
        };
        assert_eq!(
            bad_name
                .validate(&CommandProfile::unprofiled())
                .unwrap_err()
                .code,
            "SAND-TEXT-COLOR"
        );
    }
}
