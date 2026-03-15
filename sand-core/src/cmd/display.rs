/// Builders for player-facing display commands: `title`, `actionbar`, and
/// `bossbar`.
///
/// # Example
/// ```rust,ignore
/// let cmds = Title::of(Selector::all_players())
///     .title(TextComponent::literal("Welcome!").bold(true).color(ChatColor::Gold))
///     .subtitle(TextComponent::literal("to the server"))
///     .times(10, 60, 20);
///
/// Actionbar::show(Selector::self_(), TextComponent::literal("Cooldown: 3s"));
///
/// let mut cmds = Vec::new();
/// cmds.push(Bossbar::add("my_pack:hp", TextComponent::literal("Health")));
/// cmds.push(Bossbar::set_value("my_pack:hp", 80));
/// cmds.push(Bossbar::set_max("my_pack:hp", 100));
/// cmds.push(Bossbar::set_players("my_pack:hp", Selector::all_players()));
/// ```
use std::fmt::Display;

use super::selector::Selector;
use super::types::TextComponent;

// ── Title ─────────────────────────────────────────────────────────────────────

/// Builder for `title` commands targeting a selector.
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
    /// Create a new `Title` for the given selector.
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

    /// Set fade-in / stay / fade-out tick durations.
    pub fn times(mut self, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        self.fade_in = fade_in;
        self.stay = stay;
        self.fade_out = fade_out;
        self
    }

    /// Generate the ordered list of commands needed to display this title.
    ///
    /// Always emits a `title ... times` first, then title/subtitle/actionbar
    /// in the correct order.
    pub fn build(self) -> Vec<String> {
        let sel = &self.selector;
        let mut cmds = Vec::new();

        cmds.push(format!(
            "title {} times {} {} {}",
            sel, self.fade_in, self.stay, self.fade_out
        ));

        if let Some(sub) = self.subtitle {
            let json = sub.to_string();
            cmds.push(format!("title {} subtitle {}", sel, json));
        }

        if let Some(t) = self.title {
            let json = t.to_string();
            cmds.push(format!("title {} title {}", sel, json));
        }

        if let Some(ab) = self.actionbar {
            let json = ab.to_string();
            cmds.push(format!("title {} actionbar {}", sel, json));
        }

        cmds
    }

    /// Clear the title display for the selector.
    pub fn clear(selector: Selector) -> String {
        format!("title {} clear", selector)
    }

    /// Reset title display settings to defaults.
    pub fn reset(selector: Selector) -> String {
        format!("title {} reset", selector)
    }
}

// ── Actionbar ─────────────────────────────────────────────────────────────────

pub struct Actionbar;

impl Actionbar {
    /// Show `text` in the action bar for `selector`.
    pub fn show(selector: impl Display, text: TextComponent) -> String {
        format!("title {} actionbar {}", selector, text)
    }

    /// Show a raw JSON string in the action bar.
    pub fn show_raw(selector: impl Display, json: impl Display) -> String {
        format!("title {} actionbar {}", selector, json)
    }
}

// ── BossbarColor ─────────────────────────────────────────────────────────────

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

/// Static helpers for `bossbar` commands.
pub struct Bossbar;

impl Bossbar {
    /// `bossbar add <id> <name>` — create the bossbar.
    pub fn add(id: impl Display, name: TextComponent) -> String {
        format!("bossbar add {} {}", id, name)
    }

    /// `bossbar remove <id>`
    pub fn remove(id: impl Display) -> String {
        format!("bossbar remove {}", id)
    }

    /// `bossbar set <id> value <n>`
    pub fn set_value(id: impl Display, value: u32) -> String {
        format!("bossbar set {} value {}", id, value)
    }

    /// `bossbar set <id> max <n>`
    pub fn set_max(id: impl Display, max: u32) -> String {
        format!("bossbar set {} max {}", id, max)
    }

    /// `bossbar set <id> players <selector>`
    pub fn set_players(id: impl Display, selector: impl Display) -> String {
        format!("bossbar set {} players {}", id, selector)
    }

    /// `bossbar set <id> color <color>`
    pub fn set_color(id: impl Display, color: BossbarColor) -> String {
        format!("bossbar set {} color {}", id, color)
    }

    /// `bossbar set <id> style <style>`
    pub fn set_style(id: impl Display, style: BossbarStyle) -> String {
        format!("bossbar set {} style {}", id, style)
    }

    /// `bossbar set <id> name <name>`
    pub fn set_name(id: impl Display, name: TextComponent) -> String {
        format!("bossbar set {} name {}", id, name)
    }

    /// `bossbar set <id> visible <bool>`
    pub fn set_visible(id: impl Display, visible: bool) -> String {
        format!("bossbar set {} visible {}", id, visible)
    }

    /// `bossbar get <id> value`
    pub fn get_value(id: impl Display) -> String {
        format!("bossbar get {} value", id)
    }

    /// `bossbar get <id> max`
    pub fn get_max(id: impl Display) -> String {
        format!("bossbar get {} max", id)
    }

    /// `bossbar get <id> players`
    pub fn get_players(id: impl Display) -> String {
        format!("bossbar get {} players", id)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::selector::Selector;
    use crate::cmd::types::TextComponent;

    #[test]
    fn title_build_order() {
        let cmds = Title::of(Selector::all_players())
            .title(TextComponent::literal("Hello"))
            .subtitle(TextComponent::literal("World"))
            .times(5, 40, 10)
            .build();

        // times first, then subtitle, then title
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
