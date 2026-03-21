//! Builders for player-facing display commands: `title`, `actionbar`, and `bossbar`.
//!
//! # Example
//! ```rust,ignore
//! let cmds = Title::of(Selector::all_players())
//!     .title(TextComponent::literal("Welcome!").bold(true).color(ChatColor::Gold))
//!     .subtitle(TextComponent::literal("to the server"))
//!     .times(10, 60, 20)
//!     .build();
//!
//! let bar = Actionbar::show(Selector::self_(), TextComponent::literal("Cooldown: 3s"));
//! ```

use std::fmt::Display;

use crate::selector::Selector;
use crate::text::TextComponent;

// ── Title ─────────────────────────────────────────────────────────────────────

/// Builder for title screen display commands (`title` command).
///
/// `build()` returns a `Vec<String>` of commands in the correct order:
/// times first, then subtitle, title, actionbar.
#[derive(Clone)]
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
    /// Create a new Title display for the given selector.
    ///
    /// Defaults: 10 ticks fade-in, 70 ticks stay, 20 ticks fade-out.
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

    /// Set the main title text (large, centered).
    pub fn title(mut self, text: TextComponent) -> Self {
        self.title = Some(text);
        self
    }

    /// Set the subtitle text (smaller, below title).
    pub fn subtitle(mut self, text: TextComponent) -> Self {
        self.subtitle = Some(text);
        self
    }

    /// Set the action bar text (bottom-left, overlays hotbar).
    pub fn actionbar(mut self, text: TextComponent) -> Self {
        self.actionbar = Some(text);
        self
    }

    /// Set animation timings in ticks.
    pub fn times(mut self, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        self.fade_in = fade_in;
        self.stay = stay;
        self.fade_out = fade_out;
        self
    }

    /// Generate the ordered list of commands for this title.
    ///
    /// Commands are in order: times, subtitle, title, actionbar.
    pub fn build(self) -> Vec<String> {
        let sel = &self.selector;
        let mut cmds = Vec::new();

        cmds.push(format!(
            "title {} times {} {} {}",
            sel, self.fade_in, self.stay, self.fade_out
        ));

        if let Some(sub) = self.subtitle {
            cmds.push(format!("title {} subtitle {}", sel, sub));
        }

        if let Some(t) = self.title {
            cmds.push(format!("title {} title {}", sel, t));
        }

        if let Some(ab) = self.actionbar {
            cmds.push(format!("title {} actionbar {}", sel, ab));
        }

        cmds
    }

    /// `title <selector> clear` — hide the current title display.
    pub fn clear(selector: Selector) -> String {
        format!("title {} clear", selector)
    }

    /// `title <selector> reset` — reset title display settings to defaults.
    pub fn reset(selector: Selector) -> String {
        format!("title {} reset", selector)
    }
}

// ── Actionbar ─────────────────────────────────────────────────────────────────

/// Static helpers for action bar display (HUD text above hotbar).
pub struct Actionbar;

impl Actionbar {
    /// `title <selector> actionbar <json>` — show a TextComponent in the action bar.
    pub fn show(selector: impl Display, text: TextComponent) -> String {
        format!("title {} actionbar {}", selector, text)
    }

    /// Show a raw JSON string in the action bar.
    pub fn show_raw(selector: impl Display, json: impl Display) -> String {
        format!("title {} actionbar {}", selector, json)
    }
}

// ── BossbarColor ─────────────────────────────────────────────────────────────

/// Boss bar color/appearance.
#[derive(Debug, Clone, Copy)]
pub enum BossbarColor {
    Blue,
    Green,
    Pink,
    Purple,
    Red,
    White,
    Yellow,
}

impl Display for BossbarColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BossbarColor::Blue => "blue",
            BossbarColor::Green => "green",
            BossbarColor::Pink => "pink",
            BossbarColor::Purple => "purple",
            BossbarColor::Red => "red",
            BossbarColor::White => "white",
            BossbarColor::Yellow => "yellow",
        };
        f.write_str(s)
    }
}

// ── BossbarStyle ─────────────────────────────────────────────────────────────

/// Boss bar segmentation style.
#[derive(Debug, Clone, Copy)]
pub enum BossbarStyle {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

impl Display for BossbarStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BossbarStyle::Progress => "progress",
            BossbarStyle::Notched6 => "notched_6",
            BossbarStyle::Notched10 => "notched_10",
            BossbarStyle::Notched12 => "notched_12",
            BossbarStyle::Notched20 => "notched_20",
        };
        f.write_str(s)
    }
}

// ── Bossbar ───────────────────────────────────────────────────────────────────

/// Static helpers for boss bar display and management (`bossbar` command).
pub struct Bossbar;

impl Bossbar {
    /// `bossbar add <id> <name>` — create a new boss bar.
    pub fn add(id: impl Display, name: TextComponent) -> String {
        format!("bossbar add {} {}", id, name)
    }

    /// `bossbar remove <id>` — delete a boss bar completely.
    pub fn remove(id: impl Display) -> String {
        format!("bossbar remove {}", id)
    }

    /// `bossbar set <id> value <n>` — set the current fill value.
    pub fn set_value(id: impl Display, value: u32) -> String {
        format!("bossbar set {} value {}", id, value)
    }

    /// `bossbar set <id> max <n>` — set the maximum fill value.
    pub fn set_max(id: impl Display, max: u32) -> String {
        format!("bossbar set {} max {}", id, max)
    }

    /// `bossbar set <id> players <selector>` — show the boss bar to matching players.
    pub fn set_players(id: impl Display, selector: impl Display) -> String {
        format!("bossbar set {} players {}", id, selector)
    }

    /// `bossbar set <id> color <color>` — set the boss bar color.
    pub fn set_color(id: impl Display, color: BossbarColor) -> String {
        format!("bossbar set {} color {}", id, color)
    }

    /// `bossbar set <id> style <style>` — set segmentation (progress vs. notched).
    pub fn set_style(id: impl Display, style: BossbarStyle) -> String {
        format!("bossbar set {} style {}", id, style)
    }

    /// `bossbar set <id> name <name>` — change the boss bar title.
    pub fn set_name(id: impl Display, name: TextComponent) -> String {
        format!("bossbar set {} name {}", id, name)
    }

    /// `bossbar set <id> visible <bool>` — show or hide the boss bar.
    pub fn set_visible(id: impl Display, visible: bool) -> String {
        format!("bossbar set {} visible {}", id, visible)
    }

    /// `bossbar get <id> value` — query the current fill value.
    pub fn get_value(id: impl Display) -> String {
        format!("bossbar get {} value", id)
    }

    /// `bossbar get <id> max` — query the maximum fill value.
    pub fn get_max(id: impl Display) -> String {
        format!("bossbar get {} max", id)
    }

    /// `bossbar get <id> players` — query the number of players seeing the boss bar.
    pub fn get_players(id: impl Display) -> String {
        format!("bossbar get {} players", id)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selector::Selector;
    use crate::text::TextComponent;

    #[test]
    fn title_build_order() {
        let cmds = Title::of(Selector::all_players())
            .title(TextComponent::literal("Hello"))
            .subtitle(TextComponent::literal("World"))
            .times(5, 40, 10)
            .build();

        assert_eq!(cmds.len(), 3);
        assert!(cmds[0].starts_with("title @a times 5 40 10"), "{}", cmds[0]);
        assert!(cmds[1].contains("subtitle"), "{}", cmds[1]);
        assert!(cmds[2].contains("title @a title"), "{}", cmds[2]);
    }

    #[test]
    fn actionbar_show() {
        let cmd = Actionbar::show(Selector::self_(), TextComponent::literal("Hi"));
        assert!(cmd.starts_with("title @s actionbar"), "{}", cmd);
    }

    #[test]
    fn bossbar_commands() {
        assert_eq!(
            Bossbar::set_value("foo:bar", 50),
            "bossbar set foo:bar value 50"
        );
        assert_eq!(
            Bossbar::set_max("foo:bar", 100),
            "bossbar set foo:bar max 100"
        );
        assert_eq!(Bossbar::remove("foo:bar"), "bossbar remove foo:bar");
        assert_eq!(
            Bossbar::set_color("foo:bar", BossbarColor::Red),
            "bossbar set foo:bar color red"
        );
    }

    #[test]
    fn title_clear_reset() {
        assert_eq!(Title::clear(Selector::all_players()), "title @a clear");
        assert_eq!(Title::reset(Selector::all_players()), "title @a reset");
    }
}
