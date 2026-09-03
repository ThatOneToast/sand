//! Typed scoreboard objective — a named integer counter in Minecraft.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_commands::scoreboard::{Objective, ScoreHolder};
//!
//! static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
//!
//! INFERNO_DMG.set(ScoreHolder::self_(), 0);
//! INFERNO_DMG.add(ScoreHolder::self_(), 1);
//! INFERNO_DMG.get(ScoreHolder::self_());
//! ```

use std::borrow::Cow;
use std::fmt;

use crate::Build;
use crate::error::{CommandError, CommandResult};
use crate::render::{CommandProfile, RenderCommand, Validate};
use crate::selector::{Selector, Target, TargetArgument};
use crate::text::TextComponent;
use crate::validate;

// ── ScoreHolder ───────────────────────────────────────────────────────────────

/// A scoreboard score holder — an entity selector or a named fake player.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreHolder;
/// use sand_commands::Target;
///
/// let self_holder = ScoreHolder::entity(Target::self_());
/// assert_eq!(self_holder.to_string(), "@s");
///
/// let global = ScoreHolder::fake("#total_kills");
/// assert_eq!(global.to_string(), "#total_kills");
///
/// let everyone = ScoreHolder::all();
/// assert_eq!(everyone.to_string(), "*");
/// ```
#[derive(Debug, Clone)]
enum ScoreHolderKind {
    Entity(Selector),
    Fake(String),
    All,
    Raw(String),
    Compat(String),
}

#[doc = "Builds or represents the typed score holder Minecraft command value."]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreHolder",
    aliases = ["sand::cmd::ScoreHolder", "sand::prelude::ScoreHolder", "sand::prelude::cmd::ScoreHolder"],
    module = "sand::command",
    summary = "Builds or represents the typed score holder Minecraft command value.",
    context = "Builds or represents the typed score holder Minecraft command value. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreHolder;",
)]
#[derive(Debug, Clone)]
#[must_use = "score holders do nothing until passed to a scoreboard command"]
pub struct ScoreHolder(ScoreHolderKind);

impl ScoreHolder {
    /// Create a score holder from an entity selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::entity",
        aliases = ["sand::cmd::ScoreHolder::entity", "sand::prelude::ScoreHolder::entity", "sand::prelude::cmd::ScoreHolder::entity"],
        module = "sand::command",
        kind = "method",
        summary = "Create a score holder from an entity selector.",
        context = "Create a score holder from an entity selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to create a score holder from an entity selector."),
        returns = "A `ScoreHolder` representing a score holder from an entity selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(selector: sand::command::Target)  {\n    let score_holder = sand::command::ScoreHolder::entity(selector);\n}",
    )]
    pub fn entity(selector: impl TargetArgument) -> Self {
        ScoreHolder(ScoreHolderKind::Entity(selector.into_target_selector()))
    }

    /// Create a score holder from a named fake player.
    ///
    /// Convention: prefix with `#` (e.g. `"#const"`, `"#zero"`) to distinguish
    /// from real player names.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::fake",
        aliases = ["sand::cmd::ScoreHolder::fake", "sand::prelude::ScoreHolder::fake", "sand::prelude::cmd::ScoreHolder::fake"],
        module = "sand::command",
        kind = "method",
        summary = "Create a score holder from a named fake player. Convention: prefix with `#` (e.g. `\"#const\"`, `\"#zero\"`) to distinguish from real player names.",
        context = "Create a score holder from a named fake player. Convention: prefix with `#` (e.g. `\"#const\"`, `\"#zero\"`) to distinguish from real player names. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` is used when creating a score holder from a named fake player. Convention: prefix with `#` (e.g. `\"#const\"`, `\"#zero\"`) to distinguish from real player names."),
        returns = "A `ScoreHolder` representing a score holder from a named fake player. Convention: prefix with `#` (e.g. `\"#const\"`, `\"#zero\"`) to distinguish from real player names.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let score_holder = sand::command::ScoreHolder::fake(name);\n}",
    )]
    pub fn fake(name: impl Into<String>) -> Self {
        ScoreHolder(ScoreHolderKind::Fake(name.into()))
    }

    /// `*` — all score holders with any score in this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::all",
        aliases = ["sand::cmd::ScoreHolder::all", "sand::prelude::ScoreHolder::all", "sand::prelude::cmd::ScoreHolder::all"],
        module = "sand::command",
        kind = "method",
        summary = "`*` — all score holders with any score in this objective.",
        context = "`*` — all score holders with any score in this objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `ScoreHolder` that emits the documented `*` — all score holders with any score in this objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let score_holder = sand::command::ScoreHolder::all();\n}",
    )]
    pub fn all() -> Self {
        ScoreHolder(ScoreHolderKind::All)
    }

    /// Alias for [`ScoreHolder::all`]: `*`, every score holder with any score
    /// in the objective. Named to match #146's requested canonical
    /// constructor set (`entity`/`player`/`fake`/`wildcard`/`raw`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::wildcard",
        aliases = ["sand::cmd::ScoreHolder::wildcard", "sand::prelude::ScoreHolder::wildcard", "sand::prelude::cmd::ScoreHolder::wildcard"],
        module = "sand::command",
        kind = "method",
        summary = "Alias for [`ScoreHolder::all`]: `*`, every score holder with any score in the objective. Named to match #146's requested canonical constructor set (`entity`/`player`/`fake`/`wildcard`/`raw`).",
        context = "Alias for [`ScoreHolder::all`]: `*`, every score holder with any score in the objective. Named to match #146's requested canonical constructor set (`entity`/`player`/`fake`/`wildcard`/`raw`). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `ScoreHolder` configured for alias for [`ScoreHolder::all`]: `*`, every score holder with any score in the objective. Named to match #146's requested canonical constructor set (`entity`/`player`/`fake`/`wildcard`/`raw`).",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let score_holder = sand::command::ScoreHolder::wildcard();\n}",
    )]
    pub fn wildcard() -> Self {
        Self::all()
    }

    /// A literal online-player name (e.g. `"Notch"`), independent of a
    /// selector or fake-player holder.
    ///
    /// Validated by [`Target::named_player`]'s player-name rules (1..=16 ASCII
    /// letters, digits, or `_`) — the same shape Minecraft accepts for a
    /// literal player-name score holder, kept distinct from
    /// [`ScoreHolder::fake`] so a real player name and a `#`-prefixed fake
    /// player are never confused.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::player",
        aliases = ["sand::cmd::ScoreHolder::player", "sand::prelude::ScoreHolder::player", "sand::prelude::cmd::ScoreHolder::player"],
        module = "sand::command",
        kind = "method",
        summary = "A literal online-player name (e.g. `\"Notch\"`), independent of a selector or fake-player holder.",
        context = "A literal online-player name (e.g. `\"Notch\"`), independent of a selector or fake-player holder. Validated by [`Target::named_player`]'s player-name rules (1..=16 ASCII letters, digits, or `_`) — the same shape Minecraft accepts for a literal player-name score holder, kept distinct from [`ScoreHolder::fake`] so a real player name and a `#`-prefixed fake player are never confused.",
        minecraft = "Validated by [`Target::named_player`]'s player-name rules (1..=16 ASCII letters, digits, or `_`) — the same shape Minecraft accepts for a literal player-name score holder, kept distinct from [`ScoreHolder::fake`] so a real player name and a `#`-prefixed fake player are never confused.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for a literal online-player name (e.g. `\"Notch\"`), independent of a selector or fake-player holder."),
        returns = "A `ScoreHolder` configured for a literal online-player name (e.g. `\"Notch\"`), independent of a selector or fake-player holder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let score_holder = sand::command::ScoreHolder::player(name);\n}",
    )]
    pub fn player(name: impl Into<String>) -> Self {
        ScoreHolder::entity(Target::named_player(name))
    }

    /// `@s` — score holder for the entity executing the command.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::self_",
        aliases = ["sand::cmd::ScoreHolder::self_", "sand::prelude::ScoreHolder::self_", "sand::prelude::cmd::ScoreHolder::self_"],
        module = "sand::command",
        kind = "method",
        summary = "`@s` — score holder for the entity executing the command.",
        context = "`@s` — score holder for the entity executing the command. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "A `ScoreHolder` that emits the documented `@s` — score holder for the entity executing the command form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let score_holder = sand::command::ScoreHolder::self_();\n}",
    )]
    pub fn self_() -> Self {
        ScoreHolder::entity(Selector::self_())
    }

    /// Explicit unchecked score-holder syntax.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::raw",
        aliases = ["sand::cmd::ScoreHolder::raw", "sand::prelude::ScoreHolder::raw", "sand::prelude::cmd::ScoreHolder::raw"],
        module = "sand::command",
        kind = "method",
        summary = "Explicit unchecked score-holder syntax.",
        context = "Explicit unchecked score-holder syntax. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to use explicit unchecked score-holder syntax."),
        returns = "A `ScoreHolder` configured for explicit unchecked score-holder syntax.",
        example = "use sand::prelude::*;\n\nfn demonstrate(value: impl Into < String >)  {\n    let score_holder = sand::command::ScoreHolder::raw(value);\n}",
    )]
    pub fn raw(value: impl Into<String>) -> Self {
        ScoreHolder(ScoreHolderKind::Raw(value.into()))
    }

    pub(crate) fn is_single(&self) -> bool {
        match &self.0 {
            ScoreHolderKind::Entity(selector) => selector.is_statically_single(),
            ScoreHolderKind::Raw(_) | ScoreHolderKind::Fake(_) => true,
            ScoreHolderKind::All | ScoreHolderKind::Compat(_) => false,
        }
    }

    /// Validate this holder *and* require it to statically resolve to
    /// exactly one score holder.
    ///
    /// Several vanilla scoreboard command shapes require a single-holder
    /// target even though the general `<targets>` grammar allows multiple:
    /// `execute if/unless score <holder> <obj> matches ...` and the
    /// `<source>` half of `scoreboard players operation`. Plain
    /// [`ScoreHolder::validate`] alone is not sufficient there — a wildcard
    /// (`*`) or a selector matching multiple entities (`@a`, `@e`) passes
    /// ordinary validation but is not legal in those positions. Callers
    /// building those command shapes (in this crate or downstream, e.g.
    /// `sand-core`'s `ScoreVar::try_of`/`PlayerSchema::try_init_player`)
    /// should use this instead of [`ScoreHolder::validate`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ScoreHolder::validate_single",
        aliases = ["sand::cmd::ScoreHolder::validate_single", "sand::prelude::ScoreHolder::validate_single", "sand::prelude::cmd::ScoreHolder::validate_single"],
        module = "sand::command",
        kind = "method",
        summary = "Validate this holder *and* require it to statically resolve to exactly one score holder.",
        context = "Validate this holder *and* require it to statically resolve to exactly one score holder. Several vanilla scoreboard command shapes require a single-holder target even though the general `<targets>` grammar allows multiple: `execute if/unless score <holder> <obj> matches ...` and the `<source>` half of `scoreboard players operation`. Plain [`ScoreHolder::validate`] alone is not sufficient there — a wildcard (`*`) or a selector matching multiple entities (`@a`, `@e`) passes ordinary validation but is not legal in those positions. Callers building those command shapes (in this crate or downstream, e.g. `sand-core`'s `ScoreVar::try_of`/`PlayerSchema::try_init_player`) should use this instead of [`ScoreHolder::validate`].",
        minecraft = "Several vanilla scoreboard command shapes require a single-holder target even though the general `<targets>` grammar allows multiple: `execute if/unless score <holder> <obj> matches ...` and the `<source>` half of `scoreboard players operation`. Plain [`ScoreHolder::validate`] alone is not sufficient there — a wildcard (`*`) or a selector matching multiple entities (`@a`, `@e`) passes ordinary validation but is not legal in those positions. Callers building those command shapes (in this crate or downstream, e.g. `sand-core`'s `ScoreVar::try_of`/`PlayerSchema::try_init_player`) should use this instead of [`ScoreHolder::validate`].",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(profile = "`profile` is the profile checked when validating this holder *and* require it to statically resolve to exactly one score holder."),
        returns = "On success, the value produced to validate this holder *and* require it to statically resolve to exactly one score holder; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(score_holder_value: &sand::command::ScoreHolder, profile: & sand::command::CommandProfile)  {\n    let validate_single = score_holder_value.validate_single(profile);\n}",
    )]
    pub fn validate_single(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.validate(profile)?;
        if self.is_single() {
            Ok(())
        } else {
            Err(CommandError::new(
                "ScoreHolder",
                "holder",
                "score conditions require exactly one holder; use a typed single target, `@s`, or a fake player",
            ))
        }
    }

    /// Convert a compatibility string boundary into the closest canonical holder.
    pub(crate) fn compat(value: String) -> Self {
        match value.as_str() {
            "@s" => Self::entity(Selector::self_()),
            "@p" => Self::entity(Selector::nearest_player()),
            "@r" => Self::entity(Selector::random_player()),
            "@a" => Self::entity(Selector::all_players()),
            "@e" => Self::entity(Selector::all_entities()),
            "*" => Self::all(),
            value if value.starts_with('@') => {
                ScoreHolder(ScoreHolderKind::Compat(value.to_string()))
            }
            _ => Self::fake(value),
        }
    }

    pub(crate) fn from_compat(value: String) -> Self {
        Self::compat(value)
    }
}

impl fmt::Display for ScoreHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ScoreHolderKind::Entity(selector) => selector.fmt(f),
            ScoreHolderKind::Fake(value)
            | ScoreHolderKind::Raw(value)
            | ScoreHolderKind::Compat(value) => f.write_str(value),
            ScoreHolderKind::All => f.write_str("*"),
        }
    }
}

impl Validate for ScoreHolder {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        match &self.0 {
            ScoreHolderKind::Entity(selector) => selector.validate(profile),
            ScoreHolderKind::Fake(value) => {
                validate::no_whitespace_or_control(value, "ScoreHolder", "fake_player")?;
                if value.starts_with('@') || value == "*" {
                    return Err(CommandError::new(
                        "ScoreHolder",
                        "fake_player",
                        format!("`{value}` is selector/wildcard syntax, not a literal fake player"),
                    ));
                }
                if value.len() > 40 {
                    return Err(CommandError::new(
                        "ScoreHolder",
                        "fake_player",
                        format!(
                            "score-holder names cannot exceed 40 characters, got {}",
                            value.len()
                        ),
                    ));
                }
                Ok(())
            }
            ScoreHolderKind::Compat(value) => {
                validate::no_whitespace_or_control(value, "ScoreHolder", "holder")?;
                Err(CommandError::new(
                    "ScoreHolder",
                    "holder",
                    "legacy selector strings cannot prove single-holder cardinality; use a typed target or explicit `ScoreHolder::raw`",
                ))
            }
            ScoreHolderKind::All | ScoreHolderKind::Raw(_) => Ok(()),
        }
    }
}

impl RenderCommand for ScoreHolder {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl From<Selector> for ScoreHolder {
    fn from(selector: Selector) -> Self {
        Self::entity(selector)
    }
}

impl<K, A> From<crate::selector::Target<K, A>> for ScoreHolder {
    fn from(target: crate::selector::Target<K, A>) -> Self {
        Self::entity(target.into_selector())
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ObjectiveName",
    aliases = ["sand::cmd::ObjectiveName", "sand::prelude::ObjectiveName", "sand::prelude::cmd::ObjectiveName"],
    module = "sand::command",
    summary = "Validated Minecraft scoreboard objective name. This is the single canonical objective-name type for Sand (see [#146](https://github.com/ThatOneToast/sand/issues/146)): both `sand::command::Objective` and `sand::state::ScoreVar` route their emitted objective name through [`ObjectiveName::logical`], so both crates share one deterministic hashing algorithm ([`hash_objective_name`]) instead of maintaining separate, potentially-diverging implementations.",
    context = "Validated Minecraft scoreboard objective name. This is the single canonical objective-name type for Sand (see [#146](https://github.com/ThatOneToast/sand/issues/146)): both `sand::command::Objective` and `sand::state::ScoreVar` route their emitted objective name through [`ObjectiveName::logical`], so both crates share one deterministic hashing algorithm ([`hash_objective_name`]) instead of maintaining separate, potentially-diverging implementations. - [`ObjectiveName::minecraft`]/[`ObjectiveName::new`] — an already-valid, const-friendly *exact* emitted name. Validated at the fallible render/export boundary, not at construction (so `static` declarations stay const-friendly); an invalid exact name is rejected there rather than silently hashed. - [`ObjectiveName::logical`] — a *generated* name for an arbitrary logical identifier. Short, already-valid logical names pass through unchanged; anything else (too long, or containing characters an objective name cannot use) is deterministically hashed to a stable ≤16-character token. The original logical name is retained for diagnostics via [`ObjectiveName::logical_name`].",
    minecraft = "This is the single canonical objective-name type for Sand (see [#146](https://github.com/ThatOneToast/sand/issues/146)): both `sand::command::Objective` and `sand::state::ScoreVar` route their emitted objective name through [`ObjectiveName::logical`], so both crates share one deterministic hashing algorithm ([`hash_objective_name`]) instead of maintaining separate, potentially-diverging implementations.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ObjectiveName;",
)]
/// Validated Minecraft scoreboard objective name.
///
/// This is the single canonical objective-name type for Sand (see
/// [#146](https://github.com/ThatOneToast/sand/issues/146)): both
/// `sand_commands::scoreboard::Objective` and
/// `sand_core::state::ScoreVar` route their emitted objective name through
/// [`ObjectiveName::logical`], so both crates share one deterministic
/// hashing algorithm ([`hash_objective_name`]) instead of maintaining
/// separate, potentially-diverging implementations.
///
/// - [`ObjectiveName::minecraft`]/[`ObjectiveName::new`] — an already-valid,
///   const-friendly *exact* emitted name. Validated at the fallible
///   render/export boundary, not at construction (so `static`
///   declarations stay const-friendly); an invalid exact name is rejected
///   there rather than silently hashed.
/// - [`ObjectiveName::logical`] — a *generated* name for an arbitrary
///   logical identifier. Short, already-valid logical names pass through
///   unchanged; anything else (too long, or containing characters an
///   objective name cannot use) is deterministically hashed to a stable
///   ≤16-character token. The original logical name is retained for
///   diagnostics via [`ObjectiveName::logical_name`].
#[derive(Debug, Clone)]
#[must_use = "objective names do nothing until passed to a scoreboard command"]
pub struct ObjectiveName {
    emitted: Cow<'static, str>,
    /// `Some` only for [`ObjectiveName::logical`] names, so diagnostics and
    /// [`ObjectiveName::logical_name`] can report the pre-hash identifier
    /// even when the emitted name is a hash.
    logical: Option<Cow<'static, str>>,
}

impl ObjectiveName {
    /// Const-compatible name used by static objectives. Validation occurs at
    /// the fallible render/export boundary.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::new",
        aliases = ["sand::cmd::ObjectiveName::new", "sand::prelude::ObjectiveName::new", "sand::prelude::cmd::ObjectiveName::new"],
        module = "sand::command",
        kind = "method",
        summary = "Const-compatible name used by static objectives. Validation occurs at the fallible render/export boundary.",
        context = "Const-compatible name used by static objectives. Validation occurs at the fallible render/export boundary. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for const-compatible name used by static objectives. Validation occurs at the fallible render/export boundary."),
        returns = "An `ObjectiveName` configured for const-compatible name used by static objectives. Validation occurs at the fallible render/export boundary.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & 'static str)  {\n    let objective_name = sand::command::ObjectiveName::new(name);\n}",
    )]
    pub const fn new(name: &'static str) -> Self {
        Self {
            emitted: Cow::Borrowed(name),
            logical: None,
        }
    }

    /// Alias for [`ObjectiveName::new`]: an already-valid emitted name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::minecraft",
        aliases = ["sand::cmd::ObjectiveName::minecraft", "sand::prelude::ObjectiveName::minecraft", "sand::prelude::cmd::ObjectiveName::minecraft"],
        module = "sand::command",
        kind = "method",
        summary = "Alias for [`ObjectiveName::new`]: an already-valid emitted name.",
        context = "Alias for [`ObjectiveName::new`]: an already-valid emitted name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for alias for [`ObjectiveName::new`]: an already-valid emitted name."),
        returns = "An `ObjectiveName` configured for alias for [`ObjectiveName::new`]: an already-valid emitted name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & 'static str)  {\n    let objective_name = sand::command::ObjectiveName::minecraft(name);\n}",
    )]
    pub const fn minecraft(name: &'static str) -> Self {
        Self::new(name)
    }

    /// Construct and immediately validate a runtime *exact* name.
    ///
    /// Unlike [`ObjectiveName::logical`], this never falls back to hashing —
    /// an invalid name is rejected outright, per #146's requirement that
    /// "invalid short exact names are rejected rather than silently treated
    /// as logical names."
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::try_dynamic",
        aliases = ["sand::cmd::ObjectiveName::try_dynamic", "sand::prelude::ObjectiveName::try_dynamic", "sand::prelude::cmd::ObjectiveName::try_dynamic"],
        module = "sand::command",
        kind = "method",
        summary = "Construct and immediately validate a runtime *exact* name.",
        context = "Construct and immediately validate a runtime *exact* name. Unlike [`ObjectiveName::logical`], this never falls back to hashing — an invalid name is rejected outright, per #146's requirement that \"invalid short exact names are rejected rather than silently treated as logical names.\"",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` is validated while constructing a runtime *exact* name."),
        returns = "On success, the value produced to construct and immediately validate a runtime *exact* name; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let try_dynamic = sand::command::ObjectiveName::try_dynamic(name);\n}",
    )]
    pub fn try_dynamic(name: impl Into<String>) -> CommandResult<Self> {
        let name = Self {
            emitted: Cow::Owned(name.into()),
            logical: None,
        };
        name.validate(&CommandProfile::unprofiled())?;
        Ok(name)
    }

    /// Construct a deterministically generated name for an arbitrary logical
    /// identifier (e.g. a long, human-readable name).
    ///
    /// If `name` is already a valid direct objective token (non-empty, no
    /// whitespace/control characters, ≤16 characters), it is used verbatim.
    /// Otherwise `name` is deterministically hashed via
    /// [`hash_objective_name`] to a stable, always-valid ≤16-character
    /// token. The result of [`ObjectiveName::validate`] is always `Ok` for a
    /// value constructed this way.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::logical",
        aliases = ["sand::cmd::ObjectiveName::logical", "sand::prelude::ObjectiveName::logical", "sand::prelude::cmd::ObjectiveName::logical"],
        module = "sand::command",
        kind = "method",
        summary = "Construct a deterministically generated name for an arbitrary logical identifier (e.g. a long, human-readable name).",
        context = "Construct a deterministically generated name for an arbitrary logical identifier (e.g. a long, human-readable name). If `name` is already a valid direct objective token (non-empty, no whitespace/control characters, ≤16 characters), it is used verbatim. Otherwise `name` is deterministically hashed via [`hash_objective_name`] to a stable, always-valid ≤16-character token. The result of [`ObjectiveName::validate`] is always `Ok` for a value constructed this way.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "If `name` is already a valid direct objective token (non-empty, no whitespace/control characters, ≤16 characters), it is used verbatim. Otherwise `name` is deterministically hashed via [`hash_objective_name`] to a stable, always-valid ≤16-character token. The result of [`ObjectiveName::validate`] is always `Ok` for a value constructed this way."),
        returns = "An `ObjectiveName` representing a deterministically generated name for an arbitrary logical identifier (e.g. a long, human-readable name).",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let objective_name = sand::command::ObjectiveName::logical(name);\n}",
    )]
    pub fn logical(name: impl Into<String>) -> Self {
        let name = name.into();
        let direct = Self {
            emitted: Cow::Owned(name.clone()),
            logical: None,
        };
        let emitted = if direct.validate(&CommandProfile::unprofiled()).is_ok() {
            name.clone()
        } else {
            hash_objective_name(&name)
        };
        Self {
            emitted: Cow::Owned(emitted),
            logical: Some(Cow::Owned(name)),
        }
    }

    /// The name actually emitted into `scoreboard` command text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::as_str",
        aliases = ["sand::cmd::ObjectiveName::as_str", "sand::prelude::ObjectiveName::as_str", "sand::prelude::cmd::ObjectiveName::as_str"],
        module = "sand::command",
        kind = "method",
        summary = "The name actually emitted into `scoreboard` command text.",
        context = "The name actually emitted into `scoreboard` command text. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The rendered Minecraft command text produced to use the name actually emitted into `scoreboard` command text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_name_value: &sand::command::ObjectiveName)  {\n    let as_str = objective_name_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.emitted
    }

    /// The original logical name passed to [`ObjectiveName::logical`], for
    /// diagnostics. Returns the emitted name itself for
    /// [`ObjectiveName::new`]/[`ObjectiveName::try_dynamic`] values, which
    /// have no separate logical identifier.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::logical_name",
        aliases = ["sand::cmd::ObjectiveName::logical_name", "sand::prelude::ObjectiveName::logical_name", "sand::prelude::cmd::ObjectiveName::logical_name"],
        module = "sand::command",
        kind = "method",
        summary = "The original logical name passed to [`ObjectiveName::logical`], for diagnostics. Returns the emitted name itself for [`ObjectiveName::new`]/[`ObjectiveName::try_dynamic`] values, which have no separate logical identifier.",
        context = "The original logical name passed to [`ObjectiveName::logical`], for diagnostics. Returns the emitted name itself for [`ObjectiveName::new`]/[`ObjectiveName::try_dynamic`] values, which have no separate logical identifier. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The original logical name passed to [`ObjectiveName::logical`], for diagnostics. Returns the emitted name itself for [`ObjectiveName::new`]/[`ObjectiveName::try_dynamic`] values, which have no separate logical identifier.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_name_value: &sand::command::ObjectiveName)  {\n    let logical_name = objective_name_value.logical_name();\n}",
    )]
    pub fn logical_name(&self) -> &str {
        self.logical.as_deref().unwrap_or(&self.emitted)
    }

    /// `true` if this name was constructed via [`ObjectiveName::logical`]
    /// and its logical identifier differs from its emitted name (i.e. it
    /// was hashed rather than used verbatim).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ObjectiveName::is_hashed",
        aliases = ["sand::cmd::ObjectiveName::is_hashed", "sand::prelude::ObjectiveName::is_hashed", "sand::prelude::cmd::ObjectiveName::is_hashed"],
        module = "sand::command",
        kind = "method",
        summary = "`true` if this name was constructed via [`ObjectiveName::logical`] and its logical identifier differs from its emitted name (i.e. it was hashed rather than used verbatim).",
        context = "`true` if this name was constructed via [`ObjectiveName::logical`] and its logical identifier differs from its emitted name (i.e. it was hashed rather than used verbatim). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "`true` when the documented condition holds to emit the documented `true` if this name was constructed via [`ObjectiveName::logical`] and its logical identifier differs from its emitted name (i.e. it was hashed rather than used verbatim) form; otherwise `false`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_name_value: &sand::command::ObjectiveName)  {\n    let is_is_hashed = objective_name_value.is_hashed();\n}",
    )]
    pub fn is_hashed(&self) -> bool {
        self.logical.as_deref().is_some_and(|l| l != self.emitted)
    }
}

/// Deterministically hash a name to a stable ≤16-character scoreboard
/// objective token: an FNV-1a hash formatted as `s` followed by 15 hex
/// digits. This is the single canonical hashing algorithm shared by
/// [`ObjectiveName::logical`] and `sand_core::state::ScoreVar` (see
/// [#146](https://github.com/ThatOneToast/sand/issues/146)) — do not
/// reimplement a second hashing scheme elsewhere for the same purpose.
pub fn hash_objective_name(name: &str) -> String {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in name.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("s{:015x}", hash & 0x0FFF_FFFF_FFFF_FFFF)
}

impl fmt::Display for ObjectiveName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.emitted)
    }
}

impl Validate for ObjectiveName {
    fn validate(&self, _profile: &CommandProfile) -> CommandResult<()> {
        validate::no_whitespace_or_control(&self.emitted, "ObjectiveName", "name")?;
        if !self
            .emitted
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'+'))
        {
            return Err(CommandError::new(
                "ObjectiveName",
                "name",
                format!(
                    "objective names may contain only ASCII letters, digits, `_`, `-`, `.`, or `+`; got {:?}",
                    self.emitted
                ),
            )
            .with_code("SAND-SCORE-OBJECTIVE"));
        }
        if self.emitted.len() > 16 {
            return Err(CommandError::new(
                "ObjectiveName",
                "name",
                format!(
                    "objective names cannot exceed 16 characters, got {} (logical name: {:?})",
                    self.emitted.len(),
                    self.logical_name()
                ),
            )
            .with_code("SAND-SCORE-OBJECTIVE"));
        }
        Ok(())
    }
}

impl RenderCommand for ObjectiveName {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

// ── ScoreOp ───────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreOp",
    aliases = ["sand::cmd::ScoreOp", "sand::prelude::cmd::ScoreOp"],
    module = "sand::command",
    summary = "Arithmetic operation for `scoreboard players operation`.",
    context = "Arithmetic operation for `scoreboard players operation`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreOp;",
    variants(Add = "`+=` — add source to target.", Div = "`/=` — divide target by source. Truncates toward zero.", Max = "`>` — target becomes `max(target, source)`.", Min = "`<` — target becomes `min(target, source)`.", Mod = "`%=` — target becomes `target mod source`.", Mul = "`*=` — multiply target by source. Truncates toward zero.", Set = "`=` — assign source's value to target.", Sub = "`-=` — subtract source from target.", Swap = "`><` — swap: exchange the values of target and source."),
)]
/// Arithmetic operation for `scoreboard players operation`.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreOp;
///
/// assert_eq!(ScoreOp::Add.to_string(), "+=");
/// assert_eq!(ScoreOp::Swap.to_string(), "><");
/// assert_eq!(ScoreOp::Min.to_string(), "<");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOp {
    /// `+=` — add source to target.
    Add,
    /// `-=` — subtract source from target.
    Sub,
    /// `*=` — multiply target by source. Truncates toward zero.
    Mul,
    /// `/=` — divide target by source. Truncates toward zero.
    Div,
    /// `%=` — target becomes `target mod source`.
    Mod,
    /// `=` — assign source's value to target.
    Set,
    /// `<` — target becomes `min(target, source)`.
    Min,
    /// `>` — target becomes `max(target, source)`.
    Max,
    /// `><` — swap: exchange the values of target and source.
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreCmp",
    aliases = ["sand::cmd::ScoreCmp", "sand::prelude::cmd::ScoreCmp"],
    module = "sand::command",
    summary = "Comparison operator for `execute if score <a> <obj> <cmp> <b> <obj>`.",
    context = "Comparison operator for `execute if score <a> <obj> <cmp> <b> <obj>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreCmp;",
    variants(Eq = "`=` — left equals right.", Ge = "`>=` — left is greater than or equal to right.", Gt = "`>` — left is strictly greater than right.", Le = "`<=` — left is less than or equal to right.", Lt = "`<` — left is strictly less than right."),
)]
/// Comparison operator for `execute if score <a> <obj> <cmp> <b> <obj>`.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreCmp;
///
/// assert_eq!(ScoreCmp::Eq.to_string(), "=");
/// assert_eq!(ScoreCmp::Le.to_string(), "<=");
/// assert_eq!(ScoreCmp::Ge.to_string(), ">=");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreCmp {
    /// `=` — left equals right.
    Eq,
    /// `<` — left is strictly less than right.
    Lt,
    /// `<=` — left is less than or equal to right.
    Le,
    /// `>` — left is strictly greater than right.
    Gt,
    /// `>=` — left is greater than or equal to right.
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

// ── ScoreboardPlayersOperation ────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ScoreboardPlayersOperation",
    aliases = ["sand::cmd::ScoreboardPlayersOperation", "sand::prelude::cmd::ScoreboardPlayersOperation"],
    module = "sand::command",
    summary = "Result of [`scoreboard_players_operation`]. Implements [`Build`].",
    context = "Result of [`scoreboard_players_operation`]. Implements [`Build`]. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ScoreboardPlayersOperation;",
)]
/// Result of [`scoreboard_players_operation`]. Implements [`Build`].
#[derive(Debug, Clone)]
#[must_use = "command builders must be rendered or collected"]
pub struct ScoreboardPlayersOperation {
    targets: ScoreHolder,
    target_objective: ObjectiveName,
    op: ScoreOp,
    source: ScoreHolder,
    source_objective: ObjectiveName,
}

impl fmt::Display for ScoreboardPlayersOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scoreboard players operation {} {} {} {} {}",
            self.targets, self.target_objective, self.op, self.source, self.source_objective
        )
    }
}

impl Build for ScoreboardPlayersOperation {
    fn build(&self) -> String {
        self.to_string()
    }
}

impl Validate for ScoreboardPlayersOperation {
    fn validate(&self, profile: &CommandProfile) -> CommandResult<()> {
        self.targets
            .validate(profile)
            .map_err(|e| e.with_context("scoreboard operation target"))?;
        self.target_objective
            .validate(profile)
            .map_err(|e| e.with_context("scoreboard operation target objective"))?;
        self.source
            .validate(profile)
            .map_err(|e| e.with_context("scoreboard operation source"))?;
        self.source_objective
            .validate(profile)
            .map_err(|e| e.with_context("scoreboard operation source objective"))?;
        if !self.source.is_single() {
            return Err(CommandError::new(
                "scoreboard_players_operation",
                "source",
                "the source must resolve to exactly one score holder; use `execute as <targets>` and `@s` for per-entity operations",
            ));
        }
        Ok(())
    }
}

impl RenderCommand for ScoreboardPlayersOperation {
    fn render_unchecked(&self, _profile: &CommandProfile) -> String {
        self.to_string()
    }
}

impl From<ScoreboardPlayersOperation> for String {
    fn from(v: ScoreboardPlayersOperation) -> Self {
        v.build()
    }
}

/// `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>`
///
/// Performs integer arithmetic between two scores in-place. `targets` may
/// address multiple score holders, but vanilla requires `source` to resolve to
/// exactly one holder. For per-player copies or arithmetic, execute as the
/// player set and use `@s` for both operands.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::scoreboard_players_operation",
    aliases = ["sand::cmd::scoreboard_players_operation", "sand::prelude::cmd::scoreboard_players_operation"],
    module = "sand::command",
    summary = "`scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>`",
    context = "`scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>` Performs integer arithmetic between two scores in-place. `targets` may address multiple score holders, but vanilla requires `source` to resolve to exactly one holder. For per-player copies or arithmetic, execute as the player set and use `@s` for both operands.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    params(targets = "Performs integer arithmetic between two scores in-place. `targets` may address multiple score holders, but vanilla requires `source` to resolve to exactly one holder. For per-player copies or arithmetic, execute as the player set and use `@s` for both operands.", target_objective = "`target_objective` supplies the documented `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>` form.", op = "`op` supplies the documented `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>` form.", source = "Performs integer arithmetic between two scores in-place. `targets` may address multiple score holders, but vanilla requires `source` to resolve to exactly one holder. For per-player copies or arithmetic, execute as the player set and use `@s` for both operands.", source_objective = "`source_objective` supplies the documented `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>` form."),
    returns = "The `ScoreboardPlayersOperation` value produced to emit the documented `scoreboard players operation <targets> <targetObjective> <op> <source> <sourceObjective>` form.",
    example = "use sand::prelude::*;\n\nfn demonstrate(targets: sand::command::ScoreHolder, target_objective: sand::command::ObjectiveName, op: sand::command::ScoreOp, source: sand::command::ScoreHolder, source_objective: sand::command::ObjectiveName)  {\n    let scoreboard_players_operation = sand::command::scoreboard_players_operation(targets, target_objective, op, source, source_objective);\n}",
)]
pub fn scoreboard_players_operation(
    targets: ScoreHolder,
    target_objective: ObjectiveName,
    op: ScoreOp,
    source: ScoreHolder,
    source_objective: ObjectiveName,
) -> ScoreboardPlayersOperation {
    ScoreboardPlayersOperation {
        targets,
        target_objective,
        op,
        source,
        source_objective,
    }
}

// ── DisplaySlot ───────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::DisplaySlot",
    aliases = ["sand::cmd::DisplaySlot", "sand::prelude::cmd::DisplaySlot"],
    module = "sand::command",
    summary = "The display slot for `scoreboard objectives setdisplay`.",
    context = "The display slot for `scoreboard objectives setdisplay`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::DisplaySlot;",
    variants(BelowName = "`belowname` — shown below the player name tag.", List = "`list` — player tab-list.", Sidebar = "`sidebar` — right-hand scoreboard sidebar.", TeamSidebar = "`sidebar.team.<color>` — team-colored sidebar."),
    variant_fields(TeamSidebar = ["`sidebar.team.<color>` — team-colored sidebar."]),
)]
/// The display slot for `scoreboard objectives setdisplay`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplaySlot {
    /// `list` — player tab-list.
    List,
    /// `sidebar` — right-hand scoreboard sidebar.
    Sidebar,
    /// `belowname` — shown below the player name tag.
    BelowName,
    /// `sidebar.team.<color>` — team-colored sidebar.
    TeamSidebar(#[doc = "`sidebar.team.<color>` — team-colored sidebar."] String),
}

impl fmt::Display for DisplaySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplaySlot::List => write!(f, "list"),
            DisplaySlot::Sidebar => write!(f, "sidebar"),
            DisplaySlot::BelowName => write!(f, "belowname"),
            DisplaySlot::TeamSidebar(color) => write!(f, "sidebar.team.{color}"),
        }
    }
}

// ── Objective ─────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Objective",
    aliases = ["sand::cmd::Objective", "sand::prelude::Objective", "sand::prelude::cmd::Objective"],
    module = "sand::command",
    summary = "A named Minecraft scoreboard objective.",
    context = "A named Minecraft scoreboard objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::Objective;",
)]
/// A named Minecraft scoreboard objective.
///
/// # Declaration
///
/// ```rust,ignore
/// use sand_commands::scoreboard::Objective;
///
/// static INFERNO_DMG: Objective = Objective::new("inferno_dmg");
/// static COOLDOWN:    Objective = Objective::new("inferno_cd");
/// ```
pub struct Objective {
    name: ObjectiveName,
}

impl Objective {
    /// Const-compatible constructor for `static`/`const` declarations.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::new",
        aliases = ["sand::cmd::Objective::new", "sand::prelude::Objective::new", "sand::prelude::cmd::Objective::new"],
        module = "sand::command",
        kind = "method",
        summary = "Const-compatible constructor for `static`/`const` declarations.",
        context = "Const-compatible constructor for `static`/`const` declarations. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for const-compatible constructor for `static`/`const` declarations."),
        returns = "An `Objective` configured for const-compatible constructor for `static`/`const` declarations.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: & 'static str)  {\n    let objective = sand::command::Objective::new(name);\n}",
    )]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: ObjectiveName::new(name),
        }
    }

    /// Compatibility constructor for a runtime-determined name.
    ///
    /// Validation is deferred until fallible rendering/export. Prefer
    /// [`try_dynamic`](Self::try_dynamic) when handling user input.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::dynamic",
        aliases = ["sand::cmd::Objective::dynamic", "sand::prelude::Objective::dynamic", "sand::prelude::cmd::Objective::dynamic"],
        module = "sand::command",
        kind = "method",
        summary = "Compatibility constructor for a runtime-determined name.",
        context = "Compatibility constructor for a runtime-determined name. Validation is deferred until fallible rendering/export. Prefer [`try_dynamic`](Self::try_dynamic) when handling user input.",
        minecraft = "Validation is deferred until fallible rendering/export. Prefer [`try_dynamic`](Self::try_dynamic) when handling user input.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for compatibility constructor for a runtime-determined name."),
        returns = "An `Objective` configured for compatibility constructor for a runtime-determined name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let objective = sand::command::Objective::dynamic(name);\n}",
    )]
    pub fn dynamic(name: impl Into<String>) -> Self {
        Self {
            name: ObjectiveName {
                emitted: Cow::Owned(name.into()),
                logical: None,
            },
        }
    }

    /// Fallible runtime constructor for normal user-provided objective names.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_dynamic",
        aliases = ["sand::cmd::Objective::try_dynamic", "sand::prelude::Objective::try_dynamic", "sand::prelude::cmd::Objective::try_dynamic"],
        module = "sand::command",
        kind = "method",
        summary = "Fallible runtime constructor for normal user-provided objective names.",
        context = "Fallible runtime constructor for normal user-provided objective names. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(name = "`name` sets the author-visible text for fallible runtime constructor for normal user-provided objective names."),
        returns = "On success, the value produced to use fallible runtime constructor for normal user-provided objective names; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(name: impl Into < String >)  {\n    let try_dynamic = sand::command::Objective::try_dynamic(name);\n}",
    )]
    pub fn try_dynamic(name: impl Into<String>) -> CommandResult<Self> {
        Ok(Self {
            name: ObjectiveName::try_dynamic(name)?,
        })
    }

    /// Return the objective name as a string.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::name",
        aliases = ["sand::cmd::Objective::name", "sand::prelude::Objective::name", "sand::prelude::cmd::Objective::name"],
        module = "sand::command",
        kind = "method",
        summary = "Return the objective name as a string.",
        context = "Return the objective name as a string. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "Return the objective name as a string.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective)  {\n    let name = objective_value.name();\n}",
    )]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Validate this objective's name against Minecraft's scoreboard-objective
    /// grammar (non-empty, no whitespace/control characters, ≤16 characters).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_validate",
        aliases = ["sand::cmd::Objective::try_validate", "sand::prelude::Objective::try_validate", "sand::prelude::cmd::Objective::try_validate"],
        module = "sand::command",
        kind = "method",
        summary = "Validate this objective's name against Minecraft's scoreboard-objective grammar (non-empty, no whitespace/control characters, ≤16 characters).",
        context = "Validate this objective's name against Minecraft's scoreboard-objective grammar (non-empty, no whitespace/control characters, ≤16 characters). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "On success, the value produced to validate this objective's name against Minecraft's scoreboard-objective grammar (non-empty, no whitespace/control characters, ≤16 characters); otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective)  {\n    let try_validate = objective_value.try_validate();\n}",
    )]
    pub fn try_validate(&self) -> CommandResult<()> {
        self.name.validate(&CommandProfile::unprofiled())
    }

    // ── Validated direct manipulation ──────────────────────────────────────
    //
    // These `try_*` methods route through the same `ObjectiveName`/
    // `ScoreHolder` validation as `ScoreboardPlayersOperation`. The plain
    // (infallible) methods below remain a documented compatibility path for
    // callers with already-trusted, statically valid names/holders.

    /// Validated `scoreboard objectives add <name> <criterion>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_create",
        aliases = ["sand::cmd::Objective::try_create", "sand::prelude::Objective::try_create", "sand::prelude::cmd::Objective::try_create"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard objectives add <name> <criterion>`.",
        context = "Validated `scoreboard objectives add <name> <criterion>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(criterion = "`criterion` sets the criterion for validated `scoreboard objectives add <name> <criterion>`."),
        returns = "On success, the value produced to use validated `scoreboard objectives add <name> <criterion>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, criterion: impl Into < String >)  {\n    let try_create = objective_value.try_create(criterion);\n}",
    )]
    pub fn try_create(&self, criterion: impl Into<String>) -> CommandResult<String> {
        self.try_validate()?;
        Ok(self.create(criterion))
    }

    /// Validated `scoreboard players set <holder> <obj> <value>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_set",
        aliases = ["sand::cmd::Objective::try_set", "sand::prelude::Objective::try_set", "sand::prelude::cmd::Objective::try_set"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players set <holder> <obj> <value>`.",
        context = "Validated `scoreboard players set <holder> <obj> <value>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` sets the holder for validated `scoreboard players set <holder> <obj> <value>`.", value = "`value` provides the value being applied or compared used to use validated `scoreboard players set <holder> <obj> <value>`."),
        returns = "On success, the value produced to use validated `scoreboard players set <holder> <obj> <value>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, value: i32)  {\n    let try_set = objective_value.try_set(holder, value);\n}",
    )]
    pub fn try_set(&self, holder: ScoreHolder, value: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.set(holder, value))
    }

    /// Validated `scoreboard players get <holder> <obj>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_get",
        aliases = ["sand::cmd::Objective::try_get", "sand::prelude::Objective::try_get", "sand::prelude::cmd::Objective::try_get"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players get <holder> <obj>`.",
        context = "Validated `scoreboard players get <holder> <obj>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` sets the holder for validated `scoreboard players get <holder> <obj>`."),
        returns = "On success, the value produced to use validated `scoreboard players get <holder> <obj>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder)  {\n    let try_get = objective_value.try_get(holder);\n}",
    )]
    pub fn try_get(&self, holder: ScoreHolder) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.get(holder))
    }

    /// Validated `scoreboard players add <holder> <obj> <amount>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_add",
        aliases = ["sand::cmd::Objective::try_add", "sand::prelude::Objective::try_add", "sand::prelude::cmd::Objective::try_add"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players add <holder> <obj> <amount>`.",
        context = "Validated `scoreboard players add <holder> <obj> <amount>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` sets the holder for validated `scoreboard players add <holder> <obj> <amount>`.", amount = "`amount` provides the requested numeric amount used to use validated `scoreboard players add <holder> <obj> <amount>`."),
        returns = "On success, the value produced to use validated `scoreboard players add <holder> <obj> <amount>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, amount: i32)  {\n    let try_add = objective_value.try_add(holder, amount);\n}",
    )]
    pub fn try_add(&self, holder: ScoreHolder, amount: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.add(holder, amount))
    }

    /// Validated `scoreboard players remove <holder> <obj> <amount>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_subtract",
        aliases = ["sand::cmd::Objective::try_subtract", "sand::prelude::Objective::try_subtract", "sand::prelude::cmd::Objective::try_subtract"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players remove <holder> <obj> <amount>`.",
        context = "Validated `scoreboard players remove <holder> <obj> <amount>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` sets the holder for validated `scoreboard players remove <holder> <obj> <amount>`.", amount = "`amount` provides the requested numeric amount used to use validated `scoreboard players remove <holder> <obj> <amount>`."),
        returns = "On success, the value produced to use validated `scoreboard players remove <holder> <obj> <amount>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, amount: i32)  {\n    let try_subtract = objective_value.try_subtract(holder, amount);\n}",
    )]
    pub fn try_subtract(&self, holder: ScoreHolder, amount: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.subtract(holder, amount))
    }

    /// Validated `scoreboard players reset <holder> <obj>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_reset",
        aliases = ["sand::cmd::Objective::try_reset", "sand::prelude::Objective::try_reset", "sand::prelude::cmd::Objective::try_reset"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players reset <holder> <obj>`.",
        context = "Validated `scoreboard players reset <holder> <obj>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` sets the holder for validated `scoreboard players reset <holder> <obj>`."),
        returns = "On success, the value produced to use validated `scoreboard players reset <holder> <obj>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder)  {\n    let try_reset = objective_value.try_reset(holder);\n}",
    )]
    pub fn try_reset(&self, holder: ScoreHolder) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.reset(holder))
    }

    /// Validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`.
    ///
    /// Reuses [`ScoreboardPlayersOperation::validate`], which additionally
    /// requires `rhs` to resolve to exactly one score holder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::try_operation",
        aliases = ["sand::cmd::Objective::try_operation", "sand::prelude::Objective::try_operation", "sand::prelude::cmd::Objective::try_operation"],
        module = "sand::command",
        kind = "method",
        summary = "Validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`.",
        context = "Validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`. Reuses [`ScoreboardPlayersOperation::validate`], which additionally requires `rhs` to resolve to exactly one score holder.",
        minecraft = "Reuses [`ScoreboardPlayersOperation::validate`], which additionally requires `rhs` to resolve to exactly one score holder.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` sets the lhs for validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`.", op = "`op` sets the op for validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`.", rhs = "Reuses [`ScoreboardPlayersOperation::validate`], which additionally requires `rhs` to resolve to exactly one score holder.", rhs_obj = "`rhs_obj` sets the rhs obj for validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`."),
        returns = "On success, the value produced to use validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, op: sand::command::ScoreOp, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let try_operation = objective_value.try_operation(lhs, op, rhs, rhs_obj);\n}",
    )]
    pub fn try_operation(
        &self,
        lhs: ScoreHolder,
        op: ScoreOp,
        rhs: ScoreHolder,
        rhs_obj: &Objective,
    ) -> CommandResult<String> {
        scoreboard_players_operation(lhs, self.name.clone(), op, rhs, rhs_obj.name.clone())
            .try_build()
    }

    // ── Load from storage ──────────────────────────────────────────────────

    /// `execute store result score <holder> <obj> run data get storage <storage_id> <key>`
    ///
    /// Load an integer value from a storage namespace into this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::load_from",
        aliases = ["sand::cmd::Objective::load_from", "sand::prelude::Objective::load_from", "sand::prelude::cmd::Objective::load_from"],
        module = "sand::command",
        kind = "method",
        summary = "`execute store result score <holder> <obj> run data get storage <storage_id> <key>`",
        context = "`execute store result score <holder> <obj> run data get storage <storage_id> <key>` Load an integer value from a storage namespace into this objective.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key>` form.", storage_id = "`storage_id` supplies the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key>` form.", key = "`key` provides the key that identifies the setting or entry used to emit the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key>` form."),
        returns = "The string value produced to emit the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, storage_id: impl Into < String >, key: impl Into < String >)  {\n    let load_from = objective_value.load_from(holder, storage_id, key);\n}",
    )]
    pub fn load_from(
        &self,
        holder: ScoreHolder,
        storage_id: impl Into<String>,
        key: impl Into<String>,
    ) -> String {
        format!(
            "execute store result score {} {} run data get storage {} {}",
            holder,
            self.name,
            storage_id.into(),
            key.into()
        )
    }

    /// `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>`
    ///
    /// Load a float NBT value, multiplied by `scale`, into this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::load_from_scaled",
        aliases = ["sand::cmd::Objective::load_from_scaled", "sand::prelude::Objective::load_from_scaled", "sand::prelude::cmd::Objective::load_from_scaled"],
        module = "sand::command",
        kind = "method",
        summary = "`execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>`",
        context = "`execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>` Load a float NBT value, multiplied by `scale`, into this objective.",
        minecraft = "Load a float NBT value, multiplied by `scale`, into this objective.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>` form.", storage_id = "`storage_id` supplies the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>` form.", key = "`key` provides the key that identifies the setting or entry used to emit the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>` form.", scale = "Load a float NBT value, multiplied by `scale`, into this objective."),
        returns = "The string value produced to emit the documented `execute store result score <holder> <obj> run data get storage <storage_id> <key> <scale>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, storage_id: impl Into < String >, key: impl Into < String >, scale: f64)  {\n    let load_from_scaled = objective_value.load_from_scaled(holder, storage_id, key, scale);\n}",
    )]
    pub fn load_from_scaled(
        &self,
        holder: ScoreHolder,
        storage_id: impl Into<String>,
        key: impl Into<String>,
        scale: f64,
    ) -> String {
        format!(
            "execute store result score {} {} run data get storage {} {} {scale}",
            holder,
            self.name,
            storage_id.into(),
            key.into()
        )
    }

    // ── Objective lifecycle ────────────────────────────────────────────────

    /// `scoreboard objectives add <name> <criterion>` — create this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::create",
        aliases = ["sand::cmd::Objective::create", "sand::prelude::Objective::create", "sand::prelude::cmd::Objective::create"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives add <name> <criterion>` — create this objective.",
        context = "`scoreboard objectives add <name> <criterion>` — create this objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(criterion = "`criterion` supplies the documented `scoreboard objectives add <name> <criterion>` — create this objective form."),
        returns = "The string value produced to emit the documented `scoreboard objectives add <name> <criterion>` — create this objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, criterion: impl Into < String >)  {\n    let create = objective_value.create(criterion);\n}",
    )]
    pub fn create(&self, criterion: impl Into<String>) -> String {
        format!(
            "scoreboard objectives add {} {}",
            self.name,
            criterion.into()
        )
    }

    /// `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::create_with_display",
        aliases = ["sand::cmd::Objective::create_with_display", "sand::prelude::Objective::create_with_display", "sand::prelude::cmd::Objective::create_with_display"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives add <name> <criterion> <displayName>` — create with a display name.",
        context = "`scoreboard objectives add <name> <criterion> <displayName>` — create with a display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(criterion = "`criterion` supplies the documented `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name form.", display = "`display` supplies the documented `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name form."),
        returns = "The string value produced to emit the documented `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, criterion: impl Into < String >, display: impl Into < String >)  {\n    let create_with_display = objective_value.create_with_display(criterion, display);\n}",
    )]
    pub fn create_with_display(
        &self,
        criterion: impl Into<String>,
        display: impl Into<String>,
    ) -> String {
        format!(
            "scoreboard objectives add {} {} {}",
            self.name,
            criterion.into(),
            display.into()
        )
    }

    /// `scoreboard objectives remove <name>` — delete this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::remove",
        aliases = ["sand::cmd::Objective::remove", "sand::prelude::Objective::remove", "sand::prelude::cmd::Objective::remove"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives remove <name>` — delete this objective.",
        context = "`scoreboard objectives remove <name>` — delete this objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to emit the documented `scoreboard objectives remove <name>` — delete this objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective)  {\n    let remove = objective_value.remove();\n}",
    )]
    pub fn remove(&self) -> String {
        format!("scoreboard objectives remove {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot> <name>` — show in a display slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::set_display",
        aliases = ["sand::cmd::Objective::set_display", "sand::prelude::Objective::set_display", "sand::prelude::cmd::Objective::set_display"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives setdisplay <slot> <name>` — show in a display slot.",
        context = "`scoreboard objectives setdisplay <slot> <name>` — show in a display slot. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(slot = "`slot` supplies the documented `scoreboard objectives setdisplay <slot> <name>` — show in a display slot form."),
        returns = "The string value produced to emit the documented `scoreboard objectives setdisplay <slot> <name>` — show in a display slot form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, slot: sand::command::DisplaySlot)  {\n    let set_display = objective_value.set_display(slot);\n}",
    )]
    pub fn set_display(&self, slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot} {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot>` — clear the given display slot.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::clear_display",
        aliases = ["sand::cmd::Objective::clear_display", "sand::prelude::Objective::clear_display", "sand::prelude::cmd::Objective::clear_display"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives setdisplay <slot>` — clear the given display slot.",
        context = "`scoreboard objectives setdisplay <slot>` — clear the given display slot. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(slot = "`slot` supplies the documented `scoreboard objectives setdisplay <slot>` — clear the given display slot form."),
        returns = "The string value produced to emit the documented `scoreboard objectives setdisplay <slot>` — clear the given display slot form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(slot: sand::command::DisplaySlot)  {\n    let clear_display = sand::command::Objective::clear_display(slot);\n}",
    )]
    pub fn clear_display(slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot}")
    }

    /// `scoreboard objectives modify <name> displayname <text>` — change the display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::modify_display_name",
        aliases = ["sand::cmd::Objective::modify_display_name", "sand::prelude::Objective::modify_display_name", "sand::prelude::cmd::Objective::modify_display_name"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives modify <name> displayname <text>` — change the display name.",
        context = "`scoreboard objectives modify <name> displayname <text>` — change the display name. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(display = "`display` supplies the documented `scoreboard objectives modify <name> displayname <text>` — change the display name form."),
        returns = "The string value produced to emit the documented `scoreboard objectives modify <name> displayname <text>` — change the display name form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, display: impl Into < String >)  {\n    let modify_display_name = objective_value.modify_display_name(display);\n}",
    )]
    pub fn modify_display_name(&self, display: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} displayname {}",
            self.name,
            display.into()
        )
    }

    /// `scoreboard objectives modify <name> rendertype <type>` — change render type.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::modify_render_type",
        aliases = ["sand::cmd::Objective::modify_render_type", "sand::prelude::Objective::modify_render_type", "sand::prelude::cmd::Objective::modify_render_type"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard objectives modify <name> rendertype <type>` — change render type.",
        context = "`scoreboard objectives modify <name> rendertype <type>` — change render type. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(render_type = "`render_type` supplies the documented `scoreboard objectives modify <name> rendertype <type>` — change render type form."),
        returns = "The string value produced to emit the documented `scoreboard objectives modify <name> rendertype <type>` — change render type form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, render_type: impl Into < String >)  {\n    let modify_render_type = objective_value.modify_render_type(render_type);\n}",
    )]
    pub fn modify_render_type(&self, render_type: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} rendertype {}",
            self.name,
            render_type.into()
        )
    }

    // ── Direct manipulation ────────────────────────────────────────────────

    /// `scoreboard players set <holder> <obj> <value>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::set",
        aliases = ["sand::cmd::Objective::set", "sand::prelude::Objective::set", "sand::prelude::cmd::Objective::set"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players set <holder> <obj> <value>`",
        context = "`scoreboard players set <holder> <obj> <value>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players set <holder> <obj> <value>` form.", value = "`value` provides the value being applied or compared used to emit the documented `scoreboard players set <holder> <obj> <value>` form."),
        returns = "The string value produced to emit the documented `scoreboard players set <holder> <obj> <value>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, value: i32)  {\n    let set = objective_value.set(holder, value);\n}",
    )]
    pub fn set(&self, holder: ScoreHolder, value: i32) -> String {
        format!("scoreboard players set {} {} {}", holder, self.name, value)
    }

    /// `scoreboard players get <holder> <obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::get",
        aliases = ["sand::cmd::Objective::get", "sand::prelude::Objective::get", "sand::prelude::cmd::Objective::get"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players get <holder> <obj>`",
        context = "`scoreboard players get <holder> <obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players get <holder> <obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players get <holder> <obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder)  {\n    let get = objective_value.get(holder);\n}",
    )]
    pub fn get(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players get {} {}", holder, self.name)
    }

    /// `scoreboard players add <holder> <obj> <amount>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::add",
        aliases = ["sand::cmd::Objective::add", "sand::prelude::Objective::add", "sand::prelude::cmd::Objective::add"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players add <holder> <obj> <amount>`",
        context = "`scoreboard players add <holder> <obj> <amount>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players add <holder> <obj> <amount>` form.", amount = "`amount` provides the requested numeric amount used to emit the documented `scoreboard players add <holder> <obj> <amount>` form."),
        returns = "The string value produced to emit the documented `scoreboard players add <holder> <obj> <amount>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, amount: i32)  {\n    let add = objective_value.add(holder, amount);\n}",
    )]
    pub fn add(&self, holder: ScoreHolder, amount: i32) -> String {
        format!("scoreboard players add {} {} {}", holder, self.name, amount)
    }

    /// `scoreboard players remove <holder> <obj> <amount>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::subtract",
        aliases = ["sand::cmd::Objective::subtract", "sand::prelude::Objective::subtract", "sand::prelude::cmd::Objective::subtract"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players remove <holder> <obj> <amount>`",
        context = "`scoreboard players remove <holder> <obj> <amount>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players remove <holder> <obj> <amount>` form.", amount = "`amount` provides the requested numeric amount used to emit the documented `scoreboard players remove <holder> <obj> <amount>` form."),
        returns = "The string value produced to emit the documented `scoreboard players remove <holder> <obj> <amount>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, amount: i32)  {\n    let subtract = objective_value.subtract(holder, amount);\n}",
    )]
    pub fn subtract(&self, holder: ScoreHolder, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            holder, self.name, amount
        )
    }

    /// `scoreboard players reset <holder> <obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::reset",
        aliases = ["sand::cmd::Objective::reset", "sand::prelude::Objective::reset", "sand::prelude::cmd::Objective::reset"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players reset <holder> <obj>`",
        context = "`scoreboard players reset <holder> <obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players reset <holder> <obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players reset <holder> <obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder)  {\n    let reset = objective_value.reset(holder);\n}",
    )]
    pub fn reset(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players reset {} {}", holder, self.name)
    }

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::operation",
        aliases = ["sand::cmd::Objective::operation", "sand::prelude::Objective::operation", "sand::prelude::cmd::Objective::operation"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` form.", op = "`op` supplies the documented `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, op: sand::command::ScoreOp, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let operation = objective_value.operation(lhs, op, rhs, rhs_obj);\n}",
    )]
    pub fn operation(
        &self,
        lhs: ScoreHolder,
        op: ScoreOp,
        rhs: ScoreHolder,
        rhs_obj: &Objective,
    ) -> String {
        format!(
            "scoreboard players operation {} {} {} {} {}",
            lhs, self.name, op, rhs, rhs_obj.name
        )
    }

    /// `scoreboard players enable <holder> <obj>` — enable a trigger objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::enable",
        aliases = ["sand::cmd::Objective::enable", "sand::prelude::Objective::enable", "sand::prelude::cmd::Objective::enable"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players enable <holder> <obj>` — enable a trigger objective.",
        context = "`scoreboard players enable <holder> <obj>` — enable a trigger objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` supplies the documented `scoreboard players enable <holder> <obj>` — enable a trigger objective form."),
        returns = "The string value produced to emit the documented `scoreboard players enable <holder> <obj>` — enable a trigger objective form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder)  {\n    let enable = objective_value.enable(holder);\n}",
    )]
    pub fn enable(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players enable {} {}", holder, self.name)
    }

    /// `scoreboard players set * <obj> <value>` — set score for ALL tracked players.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::set_all",
        aliases = ["sand::cmd::Objective::set_all", "sand::prelude::Objective::set_all", "sand::prelude::cmd::Objective::set_all"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players set * <obj> <value>` — set score for ALL tracked players.",
        context = "`scoreboard players set * <obj> <value>` — set score for ALL tracked players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(value = "`value` provides the value being applied or compared used to emit the documented `scoreboard players set * <obj> <value>` — set score for ALL tracked players form."),
        returns = "The string value produced to emit the documented `scoreboard players set * <obj> <value>` — set score for ALL tracked players form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, value: i32)  {\n    let set_all = objective_value.set_all(value);\n}",
    )]
    pub fn set_all(&self, value: i32) -> String {
        format!("scoreboard players set * {} {}", self.name, value)
    }

    /// `scoreboard players reset * <obj>` — reset scores for ALL tracked players.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::reset_all",
        aliases = ["sand::cmd::Objective::reset_all", "sand::prelude::Objective::reset_all", "sand::prelude::cmd::Objective::reset_all"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players reset * <obj>` — reset scores for ALL tracked players.",
        context = "`scoreboard players reset * <obj>` — reset scores for ALL tracked players. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The string value produced to emit the documented `scoreboard players reset * <obj>` — reset scores for ALL tracked players form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective)  {\n    let reset_all = objective_value.reset_all();\n}",
    )]
    pub fn reset_all(&self) -> String {
        format!("scoreboard players reset * {}", self.name)
    }

    // ── Named operation shortcuts ──────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::add_from",
        aliases = ["sand::cmd::Objective::add_from", "sand::prelude::Objective::add_from", "sand::prelude::cmd::Objective::add_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let add_from = objective_value.add_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn add_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Add, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::sub_from",
        aliases = ["sand::cmd::Objective::sub_from", "sand::prelude::Objective::sub_from", "sand::prelude::cmd::Objective::sub_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let sub_from = objective_value.sub_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn sub_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Sub, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::mul_from",
        aliases = ["sand::cmd::Objective::mul_from", "sand::prelude::Objective::mul_from", "sand::prelude::cmd::Objective::mul_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let mul_from = objective_value.mul_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn mul_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mul, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::div_from",
        aliases = ["sand::cmd::Objective::div_from", "sand::prelude::Objective::div_from", "sand::prelude::cmd::Objective::div_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let div_from = objective_value.div_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn div_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Div, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::mod_from",
        aliases = ["sand::cmd::Objective::mod_from", "sand::prelude::Objective::mod_from", "sand::prelude::cmd::Objective::mod_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let mod_from = objective_value.mod_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn mod_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mod, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::copy_from",
        aliases = ["sand::cmd::Objective::copy_from", "sand::prelude::Objective::copy_from", "sand::prelude::cmd::Objective::copy_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let copy_from = objective_value.copy_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn copy_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Set, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::min_from",
        aliases = ["sand::cmd::Objective::min_from", "sand::prelude::Objective::min_from", "sand::prelude::cmd::Objective::min_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let min_from = objective_value.min_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn min_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Min, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::max_from",
        aliases = ["sand::cmd::Objective::max_from", "sand::prelude::Objective::max_from", "sand::prelude::cmd::Objective::max_from"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let max_from = objective_value.max_from(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn max_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Max, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>`
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::swap_with",
        aliases = ["sand::cmd::Objective::swap_with", "sand::prelude::Objective::swap_with", "sand::prelude::cmd::Objective::swap_with"],
        module = "sand::command",
        kind = "method",
        summary = "`scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>`",
        context = "`scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(lhs = "`lhs` supplies the documented `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` form.", rhs = "`rhs` supplies the documented `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` form.", rhs_obj = "`rhs_obj` supplies the documented `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` form."),
        returns = "The string value produced to emit the documented `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>` form.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, lhs: sand::command::ScoreHolder, rhs: sand::command::ScoreHolder, rhs_obj: & sand::command::Objective)  {\n    let swap_with = objective_value.swap_with(lhs, rhs, rhs_obj);\n}",
    )]
    pub fn swap_with(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Swap, rhs, rhs_obj)
    }

    // ── Execute conditions ─────────────────────────────────────────────────

    /// Return a condition fragment `if score <holder> <obj> matches <range>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::if_matches",
        aliases = ["sand::cmd::Objective::if_matches", "sand::prelude::Objective::if_matches", "sand::prelude::cmd::Objective::if_matches"],
        module = "sand::command",
        kind = "method",
        summary = "Return a condition fragment `if score <holder> <obj> matches <range>`.",
        context = "Return a condition fragment `if score <holder> <obj> matches <range>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to return a condition fragment `if score <holder> <obj> matches <range>`.", range = "`range` is used to return a condition fragment `if score <holder> <obj> matches <range>`."),
        returns = "Return a condition fragment `if score <holder> <obj> matches <range>`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, range: impl Into < String >)  {\n    let if_matches = objective_value.if_matches(holder, range);\n}",
    )]
    pub fn if_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!("if score {} {} matches {}", holder, self.name, range.into())
    }

    /// Return a condition fragment `unless score <holder> <obj> matches <range>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::unless_matches",
        aliases = ["sand::cmd::Objective::unless_matches", "sand::prelude::Objective::unless_matches", "sand::prelude::cmd::Objective::unless_matches"],
        module = "sand::command",
        kind = "method",
        summary = "Return a condition fragment `unless score <holder> <obj> matches <range>`.",
        context = "Return a condition fragment `unless score <holder> <obj> matches <range>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(holder = "`holder` is used to return a condition fragment `unless score <holder> <obj> matches <range>`.", range = "`range` is used to return a condition fragment `unless score <holder> <obj> matches <range>`."),
        returns = "Return a condition fragment `unless score <holder> <obj> matches <range>`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, holder: sand::command::ScoreHolder, range: impl Into < String >)  {\n    let unless_matches = objective_value.unless_matches(holder, range);\n}",
    )]
    pub fn unless_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!(
            "unless score {} {} matches {}",
            holder,
            self.name,
            range.into()
        )
    }

    // ── Display ───────────────────────────────────────────────────────────

    /// Create a `TextComponent` displaying this objective's value for an entity selector.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::as_text",
        aliases = ["sand::cmd::Objective::as_text", "sand::prelude::Objective::as_text", "sand::prelude::cmd::Objective::as_text"],
        module = "sand::command",
        kind = "method",
        summary = "Create a `TextComponent` displaying this objective's value for an entity selector.",
        context = "Create a `TextComponent` displaying this objective's value for an entity selector. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(selector = "`selector` provides the Minecraft target selection used to create a `TextComponent` displaying this objective's value for an entity selector."),
        returns = "The `TextComponent` value produced to create a `TextComponent` displaying this objective's value for an entity selector.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, selector: sand::command::Target)  {\n    let as_text = objective_value.as_text(selector);\n}",
    )]
    pub fn as_text(&self, selector: impl TargetArgument) -> TextComponent {
        TextComponent::score(selector.into_target_selector().to_string(), self.name())
    }

    /// Create a `TextComponent` displaying a fake player's score in this objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::Objective::as_text_fake",
        aliases = ["sand::cmd::Objective::as_text_fake", "sand::prelude::Objective::as_text_fake", "sand::prelude::cmd::Objective::as_text_fake"],
        module = "sand::command",
        kind = "method",
        summary = "Create a `TextComponent` displaying a fake player's score in this objective.",
        context = "Create a `TextComponent` displaying a fake player's score in this objective. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(fake_player = "`fake_player` is used when creating a `TextComponent` displaying a fake player's score in this objective."),
        returns = "The `TextComponent` value produced to create a `TextComponent` displaying a fake player's score in this objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(objective_value: &sand::command::Objective, fake_player: impl Into < String >)  {\n    let as_text_fake = objective_value.as_text_fake(fake_player);\n}",
    )]
    pub fn as_text_fake(&self, fake_player: impl Into<String>) -> TextComponent {
        TextComponent::score(fake_player, self.name())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    static DMG: Objective = Objective::new("inferno_dmg");

    #[test]
    fn objective_const() {
        assert_eq!(DMG.name(), "inferno_dmg");
    }

    #[test]
    fn load_from() {
        assert_eq!(
            DMG.load_from(ScoreHolder::self_(), "my_pack:players", "uuid.damage"),
            "execute store result score @s inferno_dmg run data get storage my_pack:players uuid.damage"
        );
    }

    #[test]
    fn load_from_scaled() {
        assert_eq!(
            DMG.load_from_scaled(ScoreHolder::self_(), "my_pack:players", "uuid.damage", 10.0),
            "execute store result score @s inferno_dmg run data get storage my_pack:players uuid.damage 10"
        );
    }

    #[test]
    fn set_get_add_subtract() {
        assert_eq!(
            DMG.set(ScoreHolder::self_(), 0),
            "scoreboard players set @s inferno_dmg 0"
        );
        assert_eq!(
            DMG.get(ScoreHolder::self_()),
            "scoreboard players get @s inferno_dmg"
        );
        assert_eq!(
            DMG.add(ScoreHolder::self_(), 5),
            "scoreboard players add @s inferno_dmg 5"
        );
        assert_eq!(
            DMG.subtract(ScoreHolder::self_(), 2),
            "scoreboard players remove @s inferno_dmg 2"
        );
        assert_eq!(
            DMG.reset(ScoreHolder::self_()),
            "scoreboard players reset @s inferno_dmg"
        );
    }

    #[test]
    fn operation() {
        static OTHER: Objective = Objective::new("other_dmg");
        let cmd = DMG.operation(
            ScoreHolder::self_(),
            ScoreOp::Add,
            ScoreHolder::self_(),
            &OTHER,
        );
        assert_eq!(
            cmd,
            "scoreboard players operation @s inferno_dmg += @s other_dmg"
        );
    }

    #[test]
    fn create_and_lifecycle() {
        assert_eq!(
            DMG.create("dummy"),
            "scoreboard objectives add inferno_dmg dummy"
        );
        assert_eq!(
            DMG.create_with_display("dummy", r#"{"text":"Damage"}"#),
            r#"scoreboard objectives add inferno_dmg dummy {"text":"Damage"}"#
        );
        assert_eq!(DMG.remove(), "scoreboard objectives remove inferno_dmg");
        assert_eq!(
            DMG.set_display(DisplaySlot::Sidebar),
            "scoreboard objectives setdisplay sidebar inferno_dmg"
        );
        assert_eq!(
            DMG.set_display(DisplaySlot::TeamSidebar("red".into())),
            "scoreboard objectives setdisplay sidebar.team.red inferno_dmg"
        );
        assert_eq!(
            Objective::clear_display(DisplaySlot::Sidebar),
            "scoreboard objectives setdisplay sidebar"
        );
    }

    #[test]
    fn enable_and_wildcards() {
        static TRIGGER: Objective = Objective::new("my_trigger");
        assert_eq!(
            TRIGGER.enable(ScoreHolder::entity(Selector::all_players())),
            "scoreboard players enable @a my_trigger"
        );
        assert_eq!(DMG.set_all(0), "scoreboard players set * inferno_dmg 0");
        assert_eq!(DMG.reset_all(), "scoreboard players reset * inferno_dmg");
    }

    #[test]
    fn named_operations() {
        static OTHER: Objective = Objective::new("other");
        assert_eq!(
            DMG.add_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg += @s other"
        );
        assert_eq!(
            DMG.copy_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg = @s other"
        );
        assert_eq!(
            DMG.swap_with(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg >< @s other"
        );
        assert_eq!(
            DMG.min_from(ScoreHolder::self_(), ScoreHolder::self_(), &OTHER),
            "scoreboard players operation @s inferno_dmg < @s other"
        );
    }

    #[test]
    fn if_matches() {
        assert_eq!(
            DMG.if_matches(ScoreHolder::self_(), "1.."),
            "if score @s inferno_dmg matches 1.."
        );
    }

    #[test]
    fn as_text() {
        let t = DMG.as_text(Selector::self_()).to_string();
        assert!(t.contains("\"objective\":\"inferno_dmg\""));
        assert!(t.contains("\"name\":\"@s\""));
    }

    #[test]
    fn scoreboard_players_operation_build() {
        use crate::Build;
        let op = scoreboard_players_operation(
            ScoreHolder::self_(),
            ObjectiveName::new("mana"),
            ScoreOp::Add,
            ScoreHolder::self_(),
            ObjectiveName::new("regen"),
        );
        assert_eq!(
            op.build(),
            "scoreboard players operation @s mana += @s regen"
        );
        let s: String = op.into();
        assert_eq!(s, "scoreboard players operation @s mana += @s regen");
    }

    #[test]
    fn score_holder_player_wildcard_constructors() {
        assert_eq!(ScoreHolder::player("Notch").try_build().unwrap(), "Notch");
        assert!(ScoreHolder::player("has space").try_build().is_err());
        assert_eq!(ScoreHolder::wildcard().try_build().unwrap(), "*");
        assert_eq!(
            ScoreHolder::wildcard().to_string(),
            ScoreHolder::all().to_string()
        );
    }

    #[test]
    fn objective_and_holder_validation() {
        assert!(ObjectiveName::try_dynamic("").is_err());
        assert!(ObjectiveName::try_dynamic("has space").is_err());
        assert!(ObjectiveName::try_dynamic("seventeen_chars_x").is_err());
        assert!(ScoreHolder::fake("fake holder").try_build().is_err());
        assert!(ScoreHolder::fake("@a").try_build().is_err());
        assert_eq!(ScoreHolder::fake("#total").try_build().unwrap(), "#total");
        assert_eq!(
            ScoreHolder::raw("@e[modded_single=true]")
                .try_build()
                .unwrap(),
            "@e[modded_single=true]"
        );
    }

    #[test]
    fn objective_name_logical_short_valid_name_passes_through() {
        let name = ObjectiveName::logical("mana");
        assert_eq!(name.as_str(), "mana");
        assert_eq!(name.logical_name(), "mana");
        assert!(!name.is_hashed());
    }

    #[test]
    fn objective_name_logical_long_name_is_hashed_deterministically() {
        let long = "this_is_a_very_long_logical_name_that_exceeds_the_limit";
        let a = ObjectiveName::logical(long);
        let b = ObjectiveName::logical(long);
        assert_eq!(a.as_str(), b.as_str(), "hash must be deterministic");
        assert!(a.as_str().len() <= 16);
        assert!(a.as_str().starts_with('s'));
        assert_eq!(a.logical_name(), long);
        assert!(a.is_hashed());
        assert!(a.validate(&CommandProfile::unprofiled()).is_ok());
    }

    #[test]
    fn objective_name_logical_short_invalid_name_falls_back_to_hash() {
        // Contains a space: not a valid direct token, so it must be hashed
        // rather than emitted verbatim (that would be invalid
        // `scoreboard objectives add` syntax).
        let name = ObjectiveName::logical("player mana");
        assert_ne!(name.as_str(), "player mana");
        assert!(name.as_str().len() <= 16);
        assert_eq!(name.logical_name(), "player mana");
        assert!(name.is_hashed());

        let namespaced = ObjectiveName::logical("rpg:mob.level");
        assert_ne!(namespaced.as_str(), "rpg:mob.level");
        assert!(namespaced.is_hashed());
    }

    #[test]
    fn objective_name_try_dynamic_rejects_rather_than_hashes_invalid_short_names() {
        // try_dynamic (the "exact" constructor) must reject an invalid short
        // name outright, not silently hash it like `logical` does.
        assert!(ObjectiveName::try_dynamic("player mana").is_err());
    }

    #[test]
    fn objective_name_minecraft_is_new_alias() {
        assert_eq!(ObjectiveName::minecraft("mana").as_str(), "mana");
    }

    #[test]
    fn hash_objective_name_matches_objective_name_logical() {
        let long = "this_is_a_very_long_logical_name_that_exceeds_the_limit";
        assert_eq!(
            hash_objective_name(long),
            ObjectiveName::logical(long).as_str()
        );
    }

    #[test]
    fn objective_try_methods_match_infallible_output_for_valid_input() {
        assert_eq!(DMG.try_create("dummy").unwrap(), DMG.create("dummy"));
        assert_eq!(
            DMG.try_set(ScoreHolder::self_(), 5).unwrap(),
            DMG.set(ScoreHolder::self_(), 5)
        );
        assert_eq!(
            DMG.try_get(ScoreHolder::self_()).unwrap(),
            DMG.get(ScoreHolder::self_())
        );
        assert_eq!(
            DMG.try_add(ScoreHolder::self_(), 5).unwrap(),
            DMG.add(ScoreHolder::self_(), 5)
        );
        assert_eq!(
            DMG.try_subtract(ScoreHolder::self_(), 5).unwrap(),
            DMG.subtract(ScoreHolder::self_(), 5)
        );
        assert_eq!(
            DMG.try_reset(ScoreHolder::self_()).unwrap(),
            DMG.reset(ScoreHolder::self_())
        );
    }

    #[test]
    fn objective_try_methods_reject_invalid_objective_name() {
        let bad = Objective::dynamic("has space");
        assert!(bad.try_create("dummy").is_err());
        assert!(bad.try_set(ScoreHolder::self_(), 1).is_err());
    }

    #[test]
    fn objective_try_methods_reject_invalid_holder() {
        assert!(DMG.try_set(ScoreHolder::fake("bad holder"), 1).is_err());
    }

    #[test]
    fn objective_try_operation_matches_scoreboard_players_operation() {
        static OTHER: Objective = Objective::new("other_dmg");
        let cmd = DMG
            .try_operation(
                ScoreHolder::self_(),
                ScoreOp::Add,
                ScoreHolder::self_(),
                &OTHER,
            )
            .unwrap();
        assert_eq!(
            cmd,
            "scoreboard players operation @s inferno_dmg += @s other_dmg"
        );
    }

    #[test]
    fn operation_rejects_multi_holder_source() {
        let operation = scoreboard_players_operation(
            ScoreHolder::self_(),
            ObjectiveName::new("mana"),
            ScoreOp::Set,
            ScoreHolder::entity(Selector::all_players()),
            ObjectiveName::new("other"),
        );
        let error = operation.try_build().unwrap_err().to_string();
        assert!(error.contains("source"), "{error}");
        assert!(error.contains("exactly one"), "{error}");

        let limit_ten = scoreboard_players_operation(
            ScoreHolder::self_(),
            ObjectiveName::new("mana"),
            ScoreOp::Set,
            ScoreHolder::entity(Selector::all_players().limit(10)),
            ObjectiveName::new("other"),
        );
        assert!(limit_ten.try_build().is_err());
    }
}
