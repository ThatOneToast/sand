//! Structured builders for `title`, actionbar, and `bossbar` commands.

use std::collections::BTreeMap;
use std::fmt;

use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::{Selector, TargetArgument};
use crate::text::TextComponent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Title",
    aliases = ["sand::cmd::Title", "sand::prelude::Title", "sand::prelude::cmd::Title"],
    module = "sand::command",
    summary = "Builder for a title payload and its timing command.",
    context = "Builder for a title payload and its timing command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Title;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::of",
        aliases = ["sand::cmd::Title::of", "sand::prelude::Title::of", "sand::prelude::cmd::Title::of"],
        module = "sand::command",
        kind = "method",
        summary = "Create a payload-oriented title builder.",
        context = "Create a payload-oriented title builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to create a payload-oriented title builder."),
        returns = "A `Title` representing a payload-oriented title builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let title = sand::command::Title::of(selector);\n}",
    )]
    pub fn of(selector: impl TargetArgument) -> Self {
        Self {
            selector: selector.into_target_selector(),
            title: None,
            subtitle: None,
            actionbar: None,
            fade_in: 10,
            stay: 70,
            fade_out: 20,
        }
    }

    /// Sets the title text emitted by this title-command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::title",
        aliases = ["sand::cmd::Title::title", "sand::prelude::Title::title", "sand::prelude::cmd::Title::title"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the title text emitted by this title-command builder.",
        context = "Sets the title text emitted by this title-command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(text = "`text` provides the author-visible text applied when setting the title text emitted by this title-command builder."),
        returns = "The `Title` value with the documented change applied to set the title text emitted by this title-command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: sand::command::Title, text: sand::text::TextComponent)  {\n    let updated_title = title_value.title(text);\n}",
    )]
    pub fn title(mut self, text: TextComponent) -> Self {
        self.title = Some(text);
        self
    }

    /// Sets the subtitle text emitted by this title-command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::subtitle",
        aliases = ["sand::cmd::Title::subtitle", "sand::prelude::Title::subtitle", "sand::prelude::cmd::Title::subtitle"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the subtitle text emitted by this title-command builder.",
        context = "Sets the subtitle text emitted by this title-command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(text = "`text` provides the author-visible text applied when setting the subtitle text emitted by this title-command builder."),
        returns = "The `Title` value with the documented change applied to set the subtitle text emitted by this title-command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: sand::command::Title, text: sand::text::TextComponent)  {\n    let updated_title = title_value.subtitle(text);\n}",
    )]
    pub fn subtitle(mut self, text: TextComponent) -> Self {
        self.subtitle = Some(text);
        self
    }

    /// Sets the actionbar text emitted by this title-command builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::actionbar",
        aliases = ["sand::cmd::Title::actionbar", "sand::prelude::Title::actionbar", "sand::prelude::cmd::Title::actionbar"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the actionbar text emitted by this title-command builder.",
        context = "Sets the actionbar text emitted by this title-command builder. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(text = "`text` provides the author-visible text applied when setting the actionbar text emitted by this title-command builder."),
        returns = "The `Title` value with the documented change applied to set the actionbar text emitted by this title-command builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: sand::command::Title, text: sand::text::TextComponent)  {\n    let updated_title = title_value.actionbar(text);\n}",
    )]
    pub fn actionbar(mut self, text: TextComponent) -> Self {
        self.actionbar = Some(text);
        self
    }

    /// Sets the fade-in, display, and fade-out timings for this title sequence.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::times",
        aliases = ["sand::cmd::Title::times", "sand::prelude::Title::times", "sand::prelude::cmd::Title::times"],
        module = "sand::command",
        kind = "method",
        summary = "Sets the fade-in, display, and fade-out timings for this title sequence.",
        context = "Sets the fade-in, display, and fade-out timings for this title sequence. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(fade_in = "`fade_in` provides the fade in applied when setting the fade-in, display, and fade-out timings for this title sequence.", stay = "`stay` provides the stay applied when setting the fade-in, display, and fade-out timings for this title sequence.", fade_out = "`fade_out` provides the fade out applied when setting the fade-in, display, and fade-out timings for this title sequence."),
        returns = "The `Title` value with the documented change applied to set the fade-in, display, and fade-out timings for this title sequence.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: sand::command::Title, fade_in: u32, stay: u32, fade_out: u32)  {\n    let updated_title = title_value.times(fade_in, stay, fade_out);\n}",
    )]
    pub fn times(mut self, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        self.fade_in = fade_in;
        self.stay = stay;
        self.fade_out = fade_out;
        self
    }

    /// Validate and render all commands. Empty payload builders are rejected.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::try_build",
        aliases = ["sand::cmd::Title::try_build", "sand::prelude::Title::try_build", "sand::prelude::cmd::Title::try_build"],
        module = "sand::command",
        kind = "method",
        summary = "Validate and render all commands. Empty payload builders are rejected.",
        context = "Validate and render all commands. Empty payload builders are rejected. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "On success, the value produced to validate and render all commands. Empty payload builders are rejected; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: &sand::command::Title)  {\n    let try_build = title_value.try_build();\n}",
    )]
    pub fn try_build(&self) -> CommandResult<Vec<String>> {
        self.validate(&CommandProfile::unprofiled())?;
        Ok(self.render_lines(true))
    }

    /// Compatibility renderer. Lines retain their typed node for export-time validation.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::build",
        aliases = ["sand::cmd::Title::build", "sand::prelude::Title::build", "sand::prelude::cmd::Title::build"],
        module = "sand::command",
        kind = "method",
        summary = "Compatibility renderer. Lines retain their typed node for export-time validation.",
        context = "Compatibility renderer. Lines retain their typed node for export-time validation. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The ordered values produced to use compatibility renderer. Lines retain their typed node for export-time validation.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_value: sand::command::Title)  {\n    let values = title_value.build();\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::clear",
        aliases = ["sand::cmd::Title::clear", "sand::prelude::Title::clear", "sand::prelude::cmd::Title::clear"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft clear command for the selected title.",
        context = "Renders the Minecraft clear command for the selected title. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to render the Minecraft clear command for the selected title."),
        returns = "The rendered Minecraft command text produced to render the Minecraft clear command for the selected title.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let command = sand::command::Title::clear(selector);\n}",
    )]
    pub fn clear(selector: impl TargetArgument) -> String {
        TitleCommand::Clear(selector.into_target_selector()).build_registered()
    }

    /// Renders the Minecraft reset command for the selected title.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Title::reset",
        aliases = ["sand::cmd::Title::reset", "sand::prelude::Title::reset", "sand::prelude::cmd::Title::reset"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft reset command for the selected title.",
        context = "Renders the Minecraft reset command for the selected title. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to render the Minecraft reset command for the selected title."),
        returns = "The rendered Minecraft command text produced to render the Minecraft reset command for the selected title.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let command = sand::command::Title::reset(selector);\n}",
    )]
    pub fn reset(selector: impl TargetArgument) -> String {
        TitleCommand::Reset(selector.into_target_selector()).build_registered()
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::TitleTimes",
    aliases = ["sand::cmd::TitleTimes", "sand::prelude::TitleTimes", "sand::prelude::cmd::TitleTimes"],
    module = "sand::command",
    summary = "Explicit timing-only title command.",
    context = "Explicit timing-only title command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::TitleTimes;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TitleTimes::new",
        aliases = ["sand::cmd::TitleTimes::new", "sand::prelude::TitleTimes::new", "sand::prelude::cmd::TitleTimes::new"],
        module = "sand::command",
        kind = "method",
        summary = "Creates a typed title times command builder from the supplied command inputs.",
        context = "Creates a typed title times command builder from the supplied command inputs. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to create a typed title times command builder from the supplied command inputs.", fade_in = "`fade_in` is used when creating a typed title times command builder from the supplied command inputs.", stay = "`stay` is used when creating a typed title times command builder from the supplied command inputs.", fade_out = "`fade_out` is used when creating a typed title times command builder from the supplied command inputs."),
        returns = "A `TitleTimes` representing a typed title times command builder from the supplied command inputs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target, fade_in: u32, stay: u32, fade_out: u32)  {\n    let title_times = sand::command::TitleTimes::new(selector, fade_in, stay, fade_out);\n}",
    )]
    pub fn new(selector: impl TargetArgument, fade_in: u32, stay: u32, fade_out: u32) -> Self {
        Self {
            selector: selector.into_target_selector(),
            fade_in,
            stay,
            fade_out,
        }
    }

    /// Renders the configured title times as validated Minecraft command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TitleTimes::build",
        aliases = ["sand::cmd::TitleTimes::build", "sand::prelude::TitleTimes::build", "sand::prelude::cmd::TitleTimes::build"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the configured title times as validated Minecraft command text.",
        context = "Renders the configured title times as validated Minecraft command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The rendered Minecraft command text produced to render the configured title times as validated Minecraft command text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(title_times_value: sand::command::TitleTimes)  {\n    let command = title_times_value.build();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Actionbar",
    aliases = ["sand::cmd::Actionbar", "sand::prelude::Actionbar", "sand::prelude::cmd::Actionbar"],
    module = "sand::command",
    summary = "Actionbar command helpers.",
    context = "Actionbar command helpers. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Actionbar;",
)]
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
    /// let command = Actionbar::show(Target::self_(), Text::new("Ready").green());
    /// assert!(command.starts_with("title @s actionbar "));
    /// assert!(command.contains("Ready"));
    /// ```
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Actionbar::show",
        aliases = ["sand::cmd::Actionbar::show", "sand::prelude::Actionbar::show", "sand::prelude::cmd::Actionbar::show"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft show command for the selected actionbar.",
        context = "Renders the Minecraft show command for the selected actionbar. `selector` identifies the players whose actionbar is updated. `text` is the typed text component serialized into the command's JSON payload.",
        minecraft = "`selector` identifies the players whose actionbar is updated. `text` is the typed text component serialized into the command's JSON payload.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` identifies the players whose actionbar is updated. `text` is the typed text component serialized into the command's JSON payload.", text = "`selector` identifies the players whose actionbar is updated. `text` is the typed text component serialized into the command's JSON payload."),
        returns = "The rendered Minecraft command text produced to render the Minecraft show command for the selected actionbar.",
        example = "use {sand::command::Actionbar, sand::command::Target, sand::text::Text};\nlet command = Actionbar::show(Target::self_(), Text::new(\"Ready\").green());\nassert!(command.starts_with(\"title @s actionbar \"));\nassert!(command.contains(\"Ready\"));",
    )]
    pub fn show(selector: impl TargetArgument, text: TextComponent) -> String {
        TitleCommand::Actionbar {
            selector: selector.into_target_selector(),
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Actionbar::show_raw",
        aliases = ["sand::cmd::Actionbar::show_raw", "sand::prelude::Actionbar::show_raw", "sand::prelude::cmd::Actionbar::show_raw"],
        module = "sand::command",
        kind = "method",
        summary = "Renders an actionbar command from an unchecked selector and raw JSON text.",
        context = "Renders an actionbar command from an unchecked selector and raw JSON text. `selector` is inserted verbatim as the command target and `json` is inserted verbatim as the text-component payload. Use [`Actionbar::show`] when typed selector and text validation are available. Returns the rendered `title <selector> actionbar <json>` command line.",
        minecraft = "`selector` is inserted verbatim as the command target and `json` is inserted verbatim as the text-component payload. Use [`Actionbar::show`] when typed selector and text validation are available.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` is inserted verbatim as the command target and `json` is inserted verbatim as the text-component payload. Use [`Actionbar::show`] when typed selector and text validation are available.", json = "`selector` is inserted verbatim as the command target and `json` is inserted verbatim as the text-component payload. Use [`Actionbar::show`] when typed selector and text validation are available."),
        returns = "Returns the rendered `title <selector> actionbar <json>` command line.",
        example = "use sand::command::Actionbar;\nlet command = Actionbar::show_raw(\"@s\", r#\"{\"text\":\"Ready\"}\"#);\nassert_eq!(command, r#\"title @s actionbar {\"text\":\"Ready\"}\"#);",
    )]
    pub fn show_raw(selector: impl fmt::Display, json: impl fmt::Display) -> String {
        TitleCommand::RawActionbar {
            selector: selector.to_string(),
            json: json.to_string(),
        }
        .build_registered()
    }
}

#[doc = "Defines the supported bossbar color forms for typed Minecraft commands."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::BossbarColor",
    aliases = ["sand::cmd::BossbarColor", "sand::prelude::BossbarColor", "sand::prelude::cmd::BossbarColor"],
    module = "sand::command",
    summary = "Defines the supported bossbar color forms for typed Minecraft commands.",
    context = "Defines the supported bossbar color forms for typed Minecraft commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::BossbarColor;",
    variants(Blue = "Selects the blue form of the bossbar color Minecraft command value.", Green = "Selects the green form of the bossbar color Minecraft command value.", Pink = "Selects the pink form of the bossbar color Minecraft command value.", Purple = "Selects the purple form of the bossbar color Minecraft command value.", Red = "Selects the red form of the bossbar color Minecraft command value.", White = "Selects the white form of the bossbar color Minecraft command value.", Yellow = "Selects the yellow form of the bossbar color Minecraft command value."),
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarColor {
    #[doc = "Selects the blue form of the bossbar color Minecraft command value."]
    Blue,
    #[doc = "Selects the green form of the bossbar color Minecraft command value."]
    Green,
    #[doc = "Selects the pink form of the bossbar color Minecraft command value."]
    Pink,
    #[doc = "Selects the purple form of the bossbar color Minecraft command value."]
    Purple,
    #[doc = "Selects the red form of the bossbar color Minecraft command value."]
    Red,
    #[doc = "Selects the white form of the bossbar color Minecraft command value."]
    White,
    #[doc = "Selects the yellow form of the bossbar color Minecraft command value."]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::BossbarStyle",
    aliases = ["sand::cmd::BossbarStyle", "sand::prelude::BossbarStyle", "sand::prelude::cmd::BossbarStyle"],
    module = "sand::command",
    summary = "Defines the supported bossbar style forms for typed Minecraft commands.",
    context = "Defines the supported bossbar style forms for typed Minecraft commands. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::BossbarStyle;",
    variants(Notched10 = "Selects the notched10 form of the bossbar style Minecraft command value.", Notched12 = "Selects the notched12 form of the bossbar style Minecraft command value.", Notched20 = "Selects the notched20 form of the bossbar style Minecraft command value.", Notched6 = "Selects the notched6 form of the bossbar style Minecraft command value.", Progress = "Selects the progress form of the bossbar style Minecraft command value."),
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossbarStyle {
    #[doc = "Selects the progress form of the bossbar style Minecraft command value."]
    Progress,
    #[doc = "Selects the notched6 form of the bossbar style Minecraft command value."]
    Notched6,
    #[doc = "Selects the notched10 form of the bossbar style Minecraft command value."]
    Notched10,
    #[doc = "Selects the notched12 form of the bossbar style Minecraft command value."]
    Notched12,
    #[doc = "Selects the notched20 form of the bossbar style Minecraft command value."]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::BossbarId",
    aliases = ["sand::cmd::BossbarId", "sand::prelude::BossbarId", "sand::prelude::cmd::BossbarId"],
    module = "sand::command",
    summary = "Canonical validated bossbar resource location.",
    context = "Canonical validated bossbar resource location. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::BossbarId;",
)]
/// Canonical validated bossbar resource location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BossbarId {
    value: String,
    raw: bool,
}

impl BossbarId {
    /// Parses and validates a typed bossbar id identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BossbarId::parse",
        aliases = ["sand::cmd::BossbarId::parse", "sand::prelude::BossbarId::parse", "sand::prelude::cmd::BossbarId::parse"],
        module = "sand::command",
        kind = "method",
        summary = "Parses and validates a typed bossbar id identifier.",
        context = "Parses and validates a typed bossbar id identifier. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to parse and validates a typed bossbar id identifier."),
        returns = "On success, the value produced to parse and validates a typed bossbar id identifier; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: impl Into < String >)  {\n    let bossbar_id_result = sand::command::BossbarId::parse(value);\n}",
    )]
    pub fn parse(value: impl Into<String>) -> CommandResult<Self> {
        let value = value.into();
        crate::validate::resource_location_shape(&value, "BossbarId", "id")
            .map_err(|error| display_error("SAND-BOSSBAR-ID", "id", error.message))?;
        Ok(Self { value, raw: false })
    }

    /// Creates an unchecked bossbar identifier for advanced command interop.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::BossbarId::raw",
        aliases = ["sand::cmd::BossbarId::raw", "sand::prelude::BossbarId::raw", "sand::prelude::cmd::BossbarId::raw"],
        module = "sand::command",
        kind = "method",
        summary = "Creates an unchecked bossbar identifier for advanced command interop.",
        context = "Creates an unchecked bossbar identifier for advanced command interop. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to create an unchecked bossbar identifier for advanced command interop."),
        returns = "A `BossbarId` representing an unchecked bossbar identifier for advanced command interop.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: impl Into < String >)  {\n    let bossbar_id = sand::command::BossbarId::raw(value);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::IntoBossbarId",
    aliases = ["sand::cmd::IntoBossbarId", "sand::prelude::cmd::IntoBossbarId"],
    module = "sand::command",
    summary = "Conversion into a bossbar resource-location token.",
    context = "Conversion into a bossbar resource-location token. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::IntoBossbarId;",
)]
/// Conversion into a bossbar resource-location token.
pub trait IntoBossbarId {
    /// Converts a value into the validated bossbar identifier accepted by command builders.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::IntoBossbarId::into_bossbar_id",
        aliases = ["sand::cmd::IntoBossbarId::into_bossbar_id", "sand::prelude::cmd::IntoBossbarId::into_bossbar_id"],
        module = "sand::command",
        summary = "Converts a value into the validated bossbar identifier accepted by command builders.",
        context = "Converts a value into the validated bossbar identifier accepted by command builders. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `BossbarId` value produced to convert a value into the validated bossbar identifier accepted by command builders.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::IntoBossbarId>(into_bossbar_id_value: T)  {\n    let into_bossbar_id = into_bossbar_id_value.into_bossbar_id();\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::BossbarCommand",
    aliases = ["sand::cmd::BossbarCommand", "sand::prelude::cmd::BossbarCommand"],
    module = "sand::command",
    summary = "Typed bossbar terminal command.",
    context = "Typed bossbar terminal command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::BossbarCommand;",
    variants(Add = "Selects the add form of the bossbar command Minecraft command value.", Get = "Selects the get form of the bossbar command Minecraft command value.", List = "Selects the list form of the bossbar command Minecraft command value.", Remove = "Selects the remove form of the bossbar command Minecraft command value.", SetColor = "Selects the set color form of the bossbar command Minecraft command value.", SetMax = "Selects the set max form of the bossbar command Minecraft command value.", SetName = "Selects the set name form of the bossbar command Minecraft command value.", SetPlayers = "Selects the set players form of the bossbar command Minecraft command value.", SetStyle = "Selects the set style form of the bossbar command Minecraft command value.", SetValue = "Selects the set value form of the bossbar command Minecraft command value.", SetVisible = "Selects the set visible form of the bossbar command Minecraft command value."),
    variant_fields(Add(id = "`id` supplies the identifier for the `bossbar add` command.", name = "`name` supplies the name for the `bossbar add` command."), Get(field = "`field` supplies the field for the `bossbar get` command.", id = "`id` supplies the identifier for the `bossbar get` command."), Remove(id = "`id` supplies the identifier for the `bossbar remove` command."), SetColor(color = "`color` supplies the color for the `bossbar set color` command.", id = "`id` supplies the identifier for the `bossbar set color` command."), SetMax(id = "`id` supplies the identifier for the `bossbar set max` command.", max = "`max` supplies the maximum value for the `bossbar set max` command."), SetName(id = "`id` supplies the identifier for the `bossbar set name` command.", name = "`name` supplies the name for the `bossbar set name` command."), SetPlayers(id = "`id` supplies the identifier for the `bossbar set players` command.", players = "`players` supplies the players for the `bossbar set players` command."), SetStyle(id = "`id` supplies the identifier for the `bossbar set style` command.", style = "`style` supplies the style for the `bossbar set style` command."), SetValue(id = "`id` supplies the identifier for the `bossbar set value` command.", value = "`value` supplies the value for the `bossbar set value` command."), SetVisible(id = "`id` supplies the identifier for the `bossbar set visible` command.", visible = "`visible` supplies the visible for the `bossbar set visible` command.")),
)]
/// Typed bossbar terminal command.
#[derive(Debug, Clone)]
pub enum BossbarCommand {
    #[doc = "Selects the add form of the bossbar command Minecraft command value."]
    Add {
        #[doc = "`id` supplies the identifier for the `bossbar add` command."]
        id: BossbarId,
        #[doc = "`name` supplies the name for the `bossbar add` command."]
        name: TextComponent,
    },
    #[doc = "Selects the remove form of the bossbar command Minecraft command value."]
    Remove {
        #[doc = "`id` supplies the identifier for the `bossbar remove` command."]
        id: BossbarId,
    },
    #[doc = "Selects the list form of the bossbar command Minecraft command value."]
    List,
    #[doc = "Selects the set name form of the bossbar command Minecraft command value."]
    SetName {
        #[doc = "`id` supplies the identifier for the `bossbar set name` command."]
        id: BossbarId,
        #[doc = "`name` supplies the name for the `bossbar set name` command."]
        name: TextComponent,
    },
    #[doc = "Selects the set color form of the bossbar command Minecraft command value."]
    SetColor {
        #[doc = "`id` supplies the identifier for the `bossbar set color` command."]
        id: BossbarId,
        #[doc = "`color` supplies the color for the `bossbar set color` command."]
        color: BossbarColor,
    },
    #[doc = "Selects the set style form of the bossbar command Minecraft command value."]
    SetStyle {
        #[doc = "`id` supplies the identifier for the `bossbar set style` command."]
        id: BossbarId,
        #[doc = "`style` supplies the style for the `bossbar set style` command."]
        style: BossbarStyle,
    },
    #[doc = "Selects the set value form of the bossbar command Minecraft command value."]
    SetValue {
        #[doc = "`id` supplies the identifier for the `bossbar set value` command."]
        id: BossbarId,
        #[doc = "`value` supplies the value for the `bossbar set value` command."]
        value: u32,
    },
    #[doc = "Selects the set max form of the bossbar command Minecraft command value."]
    SetMax {
        #[doc = "`id` supplies the identifier for the `bossbar set max` command."]
        id: BossbarId,
        #[doc = "`max` supplies the maximum value for the `bossbar set max` command."]
        max: u32,
    },
    #[doc = "Selects the set visible form of the bossbar command Minecraft command value."]
    SetVisible {
        #[doc = "`id` supplies the identifier for the `bossbar set visible` command."]
        id: BossbarId,
        #[doc = "`visible` supplies the visible for the `bossbar set visible` command."]
        visible: bool,
    },
    #[doc = "Selects the set players form of the bossbar command Minecraft command value."]
    SetPlayers {
        #[doc = "`id` supplies the identifier for the `bossbar set players` command."]
        id: BossbarId,
        #[doc = "`players` supplies the players for the `bossbar set players` command."]
        players: Selector,
    },
    #[doc = "Selects the get form of the bossbar command Minecraft command value."]
    Get {
        #[doc = "`id` supplies the identifier for the `bossbar get` command."]
        id: BossbarId,
        #[doc = "`field` supplies the field for the `bossbar get` command."]
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Bossbar",
    aliases = ["sand::cmd::Bossbar", "sand::prelude::Bossbar", "sand::prelude::cmd::Bossbar"],
    module = "sand::command",
    summary = "Builds or represents the typed bossbar Minecraft command value.",
    context = "Builds or represents the typed bossbar Minecraft command value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Bossbar;",
)]
pub struct Bossbar;

impl Bossbar {
    /// Renders the Minecraft add command for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::add",
        aliases = ["sand::cmd::Bossbar::add", "sand::prelude::Bossbar::add", "sand::prelude::cmd::Bossbar::add"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft add command for the selected bossbar.",
        context = "Renders the Minecraft add command for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft add command for the selected bossbar.", name = "`name` provides the author-visible text rendered when the Minecraft add command for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft add command for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, name: sand::text::TextComponent)  {\n    let command = sand::command::Bossbar::add(id, name);\n}",
    )]
    pub fn add(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::Add {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    /// Renders the Minecraft remove command for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::remove",
        aliases = ["sand::cmd::Bossbar::remove", "sand::prelude::Bossbar::remove", "sand::prelude::cmd::Bossbar::remove"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft remove command for the selected bossbar.",
        context = "Renders the Minecraft remove command for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft remove command for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft remove command for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId)  {\n    let command = sand::command::Bossbar::remove(id);\n}",
    )]
    pub fn remove(id: impl IntoBossbarId) -> String {
        BossbarCommand::Remove {
            id: id.into_bossbar_id(),
        }
        .build_registered()
    }
    /// Renders the Minecraft list command for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::list",
        aliases = ["sand::cmd::Bossbar::list", "sand::prelude::Bossbar::list", "sand::prelude::cmd::Bossbar::list"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft list command for the selected bossbar.",
        context = "Renders the Minecraft list command for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The rendered Minecraft command text produced to render the Minecraft list command for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let command = sand::command::Bossbar::list();\n}",
    )]
    pub fn list() -> String {
        BossbarCommand::List.build_registered()
    }
    /// Renders the Minecraft command that sets value for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_value",
        aliases = ["sand::cmd::Bossbar::set_value", "sand::prelude::Bossbar::set_value", "sand::prelude::cmd::Bossbar::set_value"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets value for the selected bossbar.",
        context = "Renders the Minecraft command that sets value for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets value for the selected bossbar.", value = "`value` provides the value being applied or compared used to render the Minecraft command that sets value for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets value for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, value: u32)  {\n    let command = sand::command::Bossbar::set_value(id, value);\n}",
    )]
    pub fn set_value(id: impl IntoBossbarId, value: u32) -> String {
        BossbarCommand::SetValue {
            id: id.into_bossbar_id(),
            value,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets max for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_max",
        aliases = ["sand::cmd::Bossbar::set_max", "sand::prelude::Bossbar::set_max", "sand::prelude::cmd::Bossbar::set_max"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets max for the selected bossbar.",
        context = "Renders the Minecraft command that sets max for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets max for the selected bossbar.", max = "`max` provides the inclusive upper bound used to render the Minecraft command that sets max for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets max for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, max: u32)  {\n    let command = sand::command::Bossbar::set_max(id, max);\n}",
    )]
    pub fn set_max(id: impl IntoBossbarId, max: u32) -> String {
        BossbarCommand::SetMax {
            id: id.into_bossbar_id(),
            max,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets players for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_players",
        aliases = ["sand::cmd::Bossbar::set_players", "sand::prelude::Bossbar::set_players", "sand::prelude::cmd::Bossbar::set_players"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets players for the selected bossbar.",
        context = "Renders the Minecraft command that sets players for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets players for the selected bossbar.", players = "`players` provides the Minecraft target selection used to render the Minecraft command that sets players for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets players for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, players: sand::command::Target)  {\n    let command = sand::command::Bossbar::set_players(id, players);\n}",
    )]
    pub fn set_players(id: impl IntoBossbarId, players: impl TargetArgument) -> String {
        BossbarCommand::SetPlayers {
            id: id.into_bossbar_id(),
            players: players.into_target_selector(),
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets color for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_color",
        aliases = ["sand::cmd::Bossbar::set_color", "sand::prelude::Bossbar::set_color", "sand::prelude::cmd::Bossbar::set_color"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets color for the selected bossbar.",
        context = "Renders the Minecraft command that sets color for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets color for the selected bossbar.", color = "`color` provides the color rendered when the Minecraft command that sets color for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets color for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, color: sand::command::BossbarColor)  {\n    let command = sand::command::Bossbar::set_color(id, color);\n}",
    )]
    pub fn set_color(id: impl IntoBossbarId, color: BossbarColor) -> String {
        BossbarCommand::SetColor {
            id: id.into_bossbar_id(),
            color,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets style for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_style",
        aliases = ["sand::cmd::Bossbar::set_style", "sand::prelude::Bossbar::set_style", "sand::prelude::cmd::Bossbar::set_style"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets style for the selected bossbar.",
        context = "Renders the Minecraft command that sets style for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets style for the selected bossbar.", style = "`style` provides the style rendered when the Minecraft command that sets style for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets style for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, style: sand::command::BossbarStyle)  {\n    let command = sand::command::Bossbar::set_style(id, style);\n}",
    )]
    pub fn set_style(id: impl IntoBossbarId, style: BossbarStyle) -> String {
        BossbarCommand::SetStyle {
            id: id.into_bossbar_id(),
            style,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets name for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_name",
        aliases = ["sand::cmd::Bossbar::set_name", "sand::prelude::Bossbar::set_name", "sand::prelude::cmd::Bossbar::set_name"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets name for the selected bossbar.",
        context = "Renders the Minecraft command that sets name for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets name for the selected bossbar.", name = "`name` provides the author-visible text rendered when the Minecraft command that sets name for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets name for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, name: sand::text::TextComponent)  {\n    let command = sand::command::Bossbar::set_name(id, name);\n}",
    )]
    pub fn set_name(id: impl IntoBossbarId, name: TextComponent) -> String {
        BossbarCommand::SetName {
            id: id.into_bossbar_id(),
            name,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that sets visible for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::set_visible",
        aliases = ["sand::cmd::Bossbar::set_visible", "sand::prelude::Bossbar::set_visible", "sand::prelude::cmd::Bossbar::set_visible"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that sets visible for the selected bossbar.",
        context = "Renders the Minecraft command that sets visible for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that sets visible for the selected bossbar.", visible = "`visible` provides the switch that enables or disables the behavior used to render the Minecraft command that sets visible for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that sets visible for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId, visible: bool)  {\n    let command = sand::command::Bossbar::set_visible(id, visible);\n}",
    )]
    pub fn set_visible(id: impl IntoBossbarId, visible: bool) -> String {
        BossbarCommand::SetVisible {
            id: id.into_bossbar_id(),
            visible,
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries value for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::get_value",
        aliases = ["sand::cmd::Bossbar::get_value", "sand::prelude::Bossbar::get_value", "sand::prelude::cmd::Bossbar::get_value"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that queries value for the selected bossbar.",
        context = "Renders the Minecraft command that queries value for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that queries value for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that queries value for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId)  {\n    let command = sand::command::Bossbar::get_value(id);\n}",
    )]
    pub fn get_value(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "value",
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries max for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::get_max",
        aliases = ["sand::cmd::Bossbar::get_max", "sand::prelude::Bossbar::get_max", "sand::prelude::cmd::Bossbar::get_max"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that queries max for the selected bossbar.",
        context = "Renders the Minecraft command that queries max for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that queries max for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that queries max for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId)  {\n    let command = sand::command::Bossbar::get_max(id);\n}",
    )]
    pub fn get_max(id: impl IntoBossbarId) -> String {
        BossbarCommand::Get {
            id: id.into_bossbar_id(),
            field: "max",
        }
        .build_registered()
    }
    /// Renders the Minecraft command that queries players for the selected bossbar.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Bossbar::get_players",
        aliases = ["sand::cmd::Bossbar::get_players", "sand::prelude::Bossbar::get_players", "sand::prelude::cmd::Bossbar::get_players"],
        module = "sand::command",
        kind = "method",
        summary = "Renders the Minecraft command that queries players for the selected bossbar.",
        context = "Renders the Minecraft command that queries players for the selected bossbar. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(id = "`id` provides the typed resource identifier or location used to render the Minecraft command that queries players for the selected bossbar."),
        returns = "The rendered Minecraft command text produced to render the Minecraft command that queries players for the selected bossbar.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl sand::command::IntoBossbarId)  {\n    let command = sand::command::Bossbar::get_players(id);\n}",
    )]
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
