//! Structured builders for `title`, actionbar, and `bossbar` commands.

use std::collections::BTreeMap;
use std::fmt;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::Selector;
use crate::text::TextComponent;

#[doc = "**API Contract:** Run `sand api show sand::command::Title` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::of` for the canonical contract."]
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

    /// Sets the title text emitted by this title-command builder.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::title` for the canonical contract."]
    pub fn title(mut self, text: TextComponent) -> Self {
        self.title = Some(text);
        self
    }

    /// Sets the subtitle text emitted by this title-command builder.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::subtitle` for the canonical contract."]
    pub fn subtitle(mut self, text: TextComponent) -> Self {
        self.subtitle = Some(text);
        self
    }

    /// Sets the actionbar text emitted by this title-command builder.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::actionbar` for the canonical contract."]
    pub fn actionbar(mut self, text: TextComponent) -> Self {
        self.actionbar = Some(text);
        self
    }

    /// Sets the fade-in, display, and fade-out timings for this title sequence.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::times` for the canonical contract."]
    pub fn times(mut self, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        self.fade_in = fade_in;
        self.stay = stay;
        self.fade_out = fade_out;
        self
    }

    /// Validate and render all commands. Empty payload builders are rejected.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::try_build` for the canonical contract."]
    pub fn try_build(&self) -> CommandResult<Vec<String>> {
        self.validate(&CommandProfile::unprofiled())?;
        Ok(self.render_lines(true))
    }

    /// Compatibility renderer. Lines retain their typed node for export-time validation.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::build` for the canonical contract."]
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

    /// Renders the Minecraft clear command for the selected title.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::clear` for the canonical contract."]
    pub fn clear(selector: Selector) -> String {
        TitleCommand::Clear(selector).build_registered()
    }

    /// Renders the Minecraft reset command for the selected title.
    #[doc = "**API Contract:** Run `sand api show sand::command::Title::reset` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::TitleTimes` for the canonical contract."]
/// Explicit timing-only title command.
#[derive(Debug, Clone)]
pub struct TitleTimes {
    selector: Selector,
    fade_in: u32,
    stay: u32,
    fade_out: u32,
}

impl TitleTimes {
    /// Creates a typed title times command builder from the supplied command inputs.
    #[doc = "**API Contract:** Run `sand api show sand::command::TitleTimes::new` for the canonical contract."]
    pub fn new(selector: Selector, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        Self {
            selector,
            fade_in,
            stay,
            fade_out,
        }
    }

    /// Renders the configured title times as validated Minecraft command text.
    #[doc = "**API Contract:** Run `sand api show sand::command::TitleTimes::build` for the canonical contract."]
    pub fn build(self) -> String {
        TitleCommand::Times(self).build_registered()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TitleCommand {
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

#[doc = "**API Contract:** Run `sand api show sand::command::Actionbar` for the canonical contract."]
/// Actionbar command helpers.
pub struct Actionbar;

impl Actionbar {
    /// Renders the Minecraft show command for the selected actionbar.
    ///
    /// `selector` identifies the players whose actionbar is updated. `text` is
    /// the typed text component serialized into the command's JSON payload.
    ///
    /// # Example
    ///
    /// ```
    /// use sand_commands::{Actionbar, Selector, Text};
    ///
    /// let command = Actionbar::show(Selector::self_(), Text::new("Ready").green());
    /// assert!(command.starts_with("title @s actionbar "));
    /// assert!(command.contains("Ready"));
    /// ```
    ///
    /// # API Contract
    ///
    /// Inspect the complete contract with
    /// `sand api show sand::command::Actionbar::show`.
    pub fn show(selector: Selector, text: TextComponent) -> String {
        TitleCommand::Actionbar {
            selector,
            text: Box::new(text),
        }
        .build_registered()
    }

    /// Renders an actionbar command from an unchecked selector and raw JSON text.
    ///
    /// `selector` is inserted verbatim as the command target and `json` is inserted
    /// verbatim as the text-component payload. Use [`Actionbar::show`] when typed
    /// selector and text validation are available.
    ///
    /// Returns the rendered `title <selector> actionbar <json>` command line.
    ///
    /// ```rust
    /// use sand_commands::Actionbar;
    ///
    /// let command = Actionbar::show_raw("@s", r#"{"text":"Ready"}"#);
    /// assert_eq!(command, r#"title @s actionbar {"text":"Ready"}"#);
    /// ```
    #[doc = "**API Contract:** Run `sand api show sand::command::Actionbar::show_raw` for the canonical contract."]
    pub fn show_raw(selector: impl fmt::Display, json: impl fmt::Display) -> String {
        TitleCommand::RawActionbar {
            selector: selector.to_string(),
            json: json.to_string(),
        }
        .build_registered()
    }
}

#[doc = "Defines the supported bossbar color forms for typed Minecraft commands."]
#[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor` for the canonical contract."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarColor {
    #[doc = "Selects the blue form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Blue` for the canonical contract."]
    Blue,
    #[doc = "Selects the green form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Green` for the canonical contract."]
    Green,
    #[doc = "Selects the pink form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Pink` for the canonical contract."]
    Pink,
    #[doc = "Selects the purple form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Purple` for the canonical contract."]
    Purple,
    #[doc = "Selects the red form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Red` for the canonical contract."]
    Red,
    #[doc = "Selects the white form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::White` for the canonical contract."]
    White,
    #[doc = "Selects the yellow form of the bossbar color Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarColor::Yellow` for the canonical contract."]
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

#[doc = "Defines the supported bossbar style forms for typed Minecraft commands."]
#[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle` for the canonical contract."]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarStyle {
    #[doc = "Selects the progress form of the bossbar style Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle::Progress` for the canonical contract."]
    Progress,
    #[doc = "Selects the notched6 form of the bossbar style Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle::Notched6` for the canonical contract."]
    Notched6,
    #[doc = "Selects the notched10 form of the bossbar style Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle::Notched10` for the canonical contract."]
    Notched10,
    #[doc = "Selects the notched12 form of the bossbar style Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle::Notched12` for the canonical contract."]
    Notched12,
    #[doc = "Selects the notched20 form of the bossbar style Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarStyle::Notched20` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::BossbarId` for the canonical contract."]
/// Canonical validated bossbar resource location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BossbarId {
    value: String,
    raw: bool,
}

impl BossbarId {
    /// Parses and validates a typed bossbar id identifier.
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarId::parse` for the canonical contract."]
    pub fn parse(value: impl Into<String>) -> CommandResult<Self> {
        let value = value.into();
        crate::validate::resource_location_shape(&value, "BossbarId", "id")
            .map_err(|error| display_error("SAND-BOSSBAR-ID", "id", error.message))?;
        Ok(Self { value, raw: false })
    }

    /// Creates an unchecked bossbar identifier for advanced command interop.
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarId::raw` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::IntoBossbarId` for the canonical contract."]
/// Conversion into a bossbar resource-location token.
pub trait IntoBossbarId {
    /// Converts a value into the validated bossbar identifier accepted by command builders.
    #[doc = "**API Contract:** Run `sand api show sand::command::IntoBossbarId::into_bossbar_id` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand` for the canonical contract."]
/// Typed bossbar terminal command.
#[derive(Debug, Clone)]
pub enum BossbarCommand {
    #[doc = "Selects the add form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Add` for the canonical contract."]
    Add {
        #[doc = "`id` provides the identifier when the variant selects the add form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Add::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`name` provides the name when the variant selects the add form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Add::name` for the canonical contract."]
        name: TextComponent,
    },
    #[doc = "Selects the remove form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Remove` for the canonical contract."]
    Remove {
        #[doc = "`id` provides the identifier when the variant selects the remove form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Remove::id` for the canonical contract."]
        id: BossbarId,
    },
    #[doc = "Selects the list form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::List` for the canonical contract."]
    List,
    #[doc = "Selects the set name form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetName` for the canonical contract."]
    SetName {
        #[doc = "`id` provides the identifier when the variant selects the set name form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetName::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`name` provides the name when the variant selects the set name form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetName::name` for the canonical contract."]
        name: TextComponent,
    },
    #[doc = "Selects the set color form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetColor` for the canonical contract."]
    SetColor {
        #[doc = "`id` provides the identifier when the variant selects the set color form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetColor::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`color` provides the color when the variant selects the set color form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetColor::color` for the canonical contract."]
        color: BossbarColor,
    },
    #[doc = "Selects the set style form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetStyle` for the canonical contract."]
    SetStyle {
        #[doc = "`id` provides the identifier when the variant selects the set style form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetStyle::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`style` provides the style when the variant selects the set style form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetStyle::style` for the canonical contract."]
        style: BossbarStyle,
    },
    #[doc = "Selects the set value form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetValue` for the canonical contract."]
    SetValue {
        #[doc = "`id` provides the identifier when the variant selects the set value form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetValue::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`value` provides the value when the variant selects the set value form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetValue::value` for the canonical contract."]
        value: u32,
    },
    #[doc = "Selects the set max form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetMax` for the canonical contract."]
    SetMax {
        #[doc = "`id` provides the identifier when the variant selects the set max form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetMax::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`max` provides the maximum value when the variant selects the set max form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetMax::max` for the canonical contract."]
        max: u32,
    },
    #[doc = "Selects the set visible form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetVisible` for the canonical contract."]
    SetVisible {
        #[doc = "`id` provides the identifier when the variant selects the set visible form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetVisible::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`visible` provides the visible when the variant selects the set visible form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetVisible::visible` for the canonical contract."]
        visible: bool,
    },
    #[doc = "Selects the set players form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetPlayers` for the canonical contract."]
    SetPlayers {
        #[doc = "`id` provides the identifier when the variant selects the set players form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetPlayers::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`players` provides the players when the variant selects the set players form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::SetPlayers::players` for the canonical contract."]
        players: Selector,
    },
    #[doc = "Selects the get form of the bossbar command Minecraft command value."]
    #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Get` for the canonical contract."]
    Get {
        #[doc = "`id` provides the identifier when the variant selects the get form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Get::id` for the canonical contract."]
        id: BossbarId,
        #[doc = "`field` provides the field when the variant selects the get form of the bossbar command Minecraft command value."]
        #[doc = "**API Contract:** Run `sand api show sand::command::BossbarCommand::Get::field` for the canonical contract."]
        field: &'static str,
    },
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

#[doc = "Builds or represents the typed bossbar Minecraft command value."]
#[doc = "**API Contract:** Run `sand api show sand::command::Bossbar` for the canonical contract."]
pub struct Bossbar;

impl Bossbar {
    /// Renders the Minecraft add command for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::add` for the canonical contract."]
    pub fn add(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::Add {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    /// Renders the Minecraft remove command for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::remove` for the canonical contract."]
    pub fn remove(id: impl IntoBossbarId) -> String {
        BossbarCommand::Remove {
            id: id.into_bossbar_id(),
        }
        .build_registered()
    }
    /// Renders the Minecraft list command for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::list` for the canonical contract."]
    pub fn list() -> String {
        BossbarCommand::List.build_registered()
    }
    /// Renders the Minecraft command that sets value for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_value` for the canonical contract."]
    pub fn set_value(id: impl IntoBossbarId, value: u32) -> String {
        BossbarCommand::SetValue {
            id: id.into_bossbar_id(),
            value,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets max for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_max` for the canonical contract."]
    pub fn set_max(id: impl IntoBossbarId, max: u32) -> String {
        BossbarCommand::SetMax {
            id: id.into_bossbar_id(),
            max,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets players for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_players` for the canonical contract."]
    pub fn set_players(id: impl IntoBossbarId, players: Selector) -> String {
        BossbarCommand::SetPlayers {
            id: id.into_bossbar_id(),
            players,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets color for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_color` for the canonical contract."]
    pub fn set_color(id: impl IntoBossbarId, color: BossbarColor) -> String {
        BossbarCommand::SetColor {
            id: id.into_bossbar_id(),
            color,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets style for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_style` for the canonical contract."]
    pub fn set_style(id: impl IntoBossbarId, style: BossbarStyle) -> String {
        BossbarCommand::SetStyle {
            id: id.into_bossbar_id(),
            style,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets name for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_name` for the canonical contract."]
    pub fn set_name(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::SetName {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets visible for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::set_visible` for the canonical contract."]
    pub fn set_visible(id: impl IntoBossbarId, visible: bool) -> String {
        BossbarCommand::SetVisible {
            id: id.into_bossbar_id(),
            visible,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries value for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::get_value` for the canonical contract."]
    pub fn get_value(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "value",
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries max for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::get_max` for the canonical contract."]
    pub fn get_max(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "max",
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries players for the selected bossbar.
    #[doc = "**API Contract:** Run `sand api show sand::command::Bossbar::get_players` for the canonical contract."]
    pub fn get_players(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "players",
        }
        .build_registered()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DisplayCommand {
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

/// Export-scoped registry family holding this module's rendered
/// display command lines and their originating typed nodes.
///
/// State lives in [`crate::export_registry`]'s active layer, so it is
/// per-thread, scoped to whichever [`crate::export_registry::ExportRegistryGuard`]
/// is open, and discarded when that guard drops — including on an early
/// `Err` return or an unwind. There is no process-global map and no
/// per-family reset to remember to call.
pub(crate) struct DisplayLines;

impl crate::export_registry::RegistryFamily for DisplayLines {
    type State = BTreeMap<String, DisplayCommand>;
}

fn register_line(line: &str, command: DisplayCommand) {
    crate::export_registry::register_line::<DisplayLines, _>(line, command);
}

pub(crate) fn validate_registered_line(line: &str, profile: &CommandProfile) -> CommandResult<()> {
    crate::export_registry::validate_registered_line::<DisplayLines, _>(
        line,
        profile,
        |command, profile| command.validate(profile),
    )
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
