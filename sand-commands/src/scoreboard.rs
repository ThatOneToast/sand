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
use crate::selector::Selector;
use crate::text::TextComponent;
use crate::validate;

// ── ScoreHolder ───────────────────────────────────────────────────────────────

/// A scoreboard score holder — an entity selector or a named fake player.
///
/// # Examples
/// ```
/// use sand_commands::scoreboard::ScoreHolder;
/// use sand_commands::selector::Selector;
///
/// let self_holder = ScoreHolder::entity(Selector::self_());
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
#[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder` for the canonical contract."]
#[derive(Debug, Clone)]
#[must_use = "score holders do nothing until passed to a scoreboard command"]
pub struct ScoreHolder(ScoreHolderKind);

impl ScoreHolder {
    /// Create a score holder from an entity selector.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::entity` for the canonical contract."]
    pub fn entity(selector: Selector) -> Self {
        ScoreHolder(ScoreHolderKind::Entity(selector))
    }

    /// Create a score holder from a named fake player.
    ///
    /// Convention: prefix with `#` (e.g. `"#const"`, `"#zero"`) to distinguish
    /// from real player names.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::fake` for the canonical contract."]
    pub fn fake(name: impl Into<String>) -> Self {
        ScoreHolder(ScoreHolderKind::Fake(name.into()))
    }

    /// `*` — all score holders with any score in this objective.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::all` for the canonical contract."]
    pub fn all() -> Self {
        ScoreHolder(ScoreHolderKind::All)
    }

    /// Alias for [`ScoreHolder::all`]: `*`, every score holder with any score
    /// in the objective. Named to match #146's requested canonical
    /// constructor set (`entity`/`player`/`fake`/`wildcard`/`raw`).
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::wildcard` for the canonical contract."]
    pub fn wildcard() -> Self {
        Self::all()
    }

    /// A literal online-player name (e.g. `"Notch"`), independent of a
    /// selector or fake-player holder.
    ///
    /// Validated by [`Selector::player`]'s player-name rules (1..=16 ASCII
    /// letters, digits, or `_`) — the same shape Minecraft accepts for a
    /// literal player-name score holder, kept distinct from
    /// [`ScoreHolder::fake`] so a real player name and a `#`-prefixed fake
    /// player are never confused.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::player` for the canonical contract."]
    pub fn player(name: impl Into<String>) -> Self {
        ScoreHolder::entity(Selector::player(name))
    }

    /// `@s` — score holder for the entity executing the command.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::self_` for the canonical contract."]
    pub fn self_() -> Self {
        ScoreHolder::entity(Selector::self_())
    }

    /// Explicit unchecked score-holder syntax.
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::raw` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreHolder::validate_single` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::new` for the canonical contract."]
    pub const fn new(name: &'static str) -> Self {
        Self {
            emitted: Cow::Borrowed(name),
            logical: None,
        }
    }

    /// Alias for [`ObjectiveName::new`]: an already-valid emitted name.
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::minecraft` for the canonical contract."]
    pub const fn minecraft(name: &'static str) -> Self {
        Self::new(name)
    }

    /// Construct and immediately validate a runtime *exact* name.
    ///
    /// Unlike [`ObjectiveName::logical`], this never falls back to hashing —
    /// an invalid name is rejected outright, per #146's requirement that
    /// "invalid short exact names are rejected rather than silently treated
    /// as logical names."
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::try_dynamic` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::logical` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::as_str` for the canonical contract."]
    pub fn as_str(&self) -> &str {
        &self.emitted
    }

    /// The original logical name passed to [`ObjectiveName::logical`], for
    /// diagnostics. Returns the emitted name itself for
    /// [`ObjectiveName::new`]/[`ObjectiveName::try_dynamic`] values, which
    /// have no separate logical identifier.
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::logical_name` for the canonical contract."]
    pub fn logical_name(&self) -> &str {
        self.logical.as_deref().unwrap_or(&self.emitted)
    }

    /// `true` if this name was constructed via [`ObjectiveName::logical`]
    /// and its logical identifier differs from its emitted name (i.e. it
    /// was hashed rather than used verbatim).
    #[doc = "**API Contract:** Run `sand api show sand::command::ObjectiveName::is_hashed` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Add` for the canonical contract."]
    /// `+=` — add source to target.
    Add,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Sub` for the canonical contract."]
    /// `-=` — subtract source from target.
    Sub,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Mul` for the canonical contract."]
    /// `*=` — multiply target by source. Truncates toward zero.
    Mul,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Div` for the canonical contract."]
    /// `/=` — divide target by source. Truncates toward zero.
    Div,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Mod` for the canonical contract."]
    /// `%=` — target becomes `target mod source`.
    Mod,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Set` for the canonical contract."]
    /// `=` — assign source's value to target.
    Set,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Min` for the canonical contract."]
    /// `<` — target becomes `min(target, source)`.
    Min,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Max` for the canonical contract."]
    /// `>` — target becomes `max(target, source)`.
    Max,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreOp::Swap` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp::Eq` for the canonical contract."]
    /// `=` — left equals right.
    Eq,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp::Lt` for the canonical contract."]
    /// `<` — left is strictly less than right.
    Lt,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp::Le` for the canonical contract."]
    /// `<=` — left is less than or equal to right.
    Le,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp::Gt` for the canonical contract."]
    /// `>` — left is strictly greater than right.
    Gt,
    #[doc = "**API Contract:** Run `sand api show sand::command::ScoreCmp::Ge` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::ScoreboardPlayersOperation` for the canonical contract."]
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
#[doc = "**API Contract:** Run `sand api show sand::command::scoreboard_players_operation` for the canonical contract."]
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

#[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot` for the canonical contract."]
/// The display slot for `scoreboard objectives setdisplay`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplaySlot {
    #[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot::List` for the canonical contract."]
    /// `list` — player tab-list.
    List,
    #[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot::Sidebar` for the canonical contract."]
    /// `sidebar` — right-hand scoreboard sidebar.
    Sidebar,
    #[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot::BelowName` for the canonical contract."]
    /// `belowname` — shown below the player name tag.
    BelowName,
    #[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot::TeamSidebar` for the canonical contract."]
    /// `sidebar.team.<color>` — team-colored sidebar.
    TeamSidebar(
        #[doc = "The `TeamSidebar` variant carries the value described by its variant semantics: `sidebar.team.<color>` — team-colored sidebar."]
        #[doc = "**API Contract:** Run `sand api show sand::command::DisplaySlot::TeamSidebar::0` for the canonical contract."]
        String,
    ),
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

#[doc = "**API Contract:** Run `sand api show sand::command::Objective` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::new` for the canonical contract."]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name: ObjectiveName::new(name),
        }
    }

    /// Compatibility constructor for a runtime-determined name.
    ///
    /// Validation is deferred until fallible rendering/export. Prefer
    /// [`try_dynamic`](Self::try_dynamic) when handling user input.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::dynamic` for the canonical contract."]
    pub fn dynamic(name: impl Into<String>) -> Self {
        Self {
            name: ObjectiveName {
                emitted: Cow::Owned(name.into()),
                logical: None,
            },
        }
    }

    /// Fallible runtime constructor for normal user-provided objective names.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_dynamic` for the canonical contract."]
    pub fn try_dynamic(name: impl Into<String>) -> CommandResult<Self> {
        Ok(Self {
            name: ObjectiveName::try_dynamic(name)?,
        })
    }

    /// Return the objective name as a string.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::name` for the canonical contract."]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Validate this objective's name against Minecraft's scoreboard-objective
    /// grammar (non-empty, no whitespace/control characters, ≤16 characters).
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_validate` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_create` for the canonical contract."]
    pub fn try_create(&self, criterion: impl Into<String>) -> CommandResult<String> {
        self.try_validate()?;
        Ok(self.create(criterion))
    }

    /// Validated `scoreboard players set <holder> <obj> <value>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_set` for the canonical contract."]
    pub fn try_set(&self, holder: ScoreHolder, value: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.set(holder, value))
    }

    /// Validated `scoreboard players get <holder> <obj>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_get` for the canonical contract."]
    pub fn try_get(&self, holder: ScoreHolder) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.get(holder))
    }

    /// Validated `scoreboard players add <holder> <obj> <amount>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_add` for the canonical contract."]
    pub fn try_add(&self, holder: ScoreHolder, amount: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.add(holder, amount))
    }

    /// Validated `scoreboard players remove <holder> <obj> <amount>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_subtract` for the canonical contract."]
    pub fn try_subtract(&self, holder: ScoreHolder, amount: i32) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.subtract(holder, amount))
    }

    /// Validated `scoreboard players reset <holder> <obj>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_reset` for the canonical contract."]
    pub fn try_reset(&self, holder: ScoreHolder) -> CommandResult<String> {
        self.try_validate()?;
        holder.validate(&CommandProfile::unprofiled())?;
        Ok(self.reset(holder))
    }

    /// Validated `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`.
    ///
    /// Reuses [`ScoreboardPlayersOperation::validate`], which additionally
    /// requires `rhs` to resolve to exactly one score holder.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::try_operation` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::load_from` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::load_from_scaled` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::create` for the canonical contract."]
    pub fn create(&self, criterion: impl Into<String>) -> String {
        format!(
            "scoreboard objectives add {} {}",
            self.name,
            criterion.into()
        )
    }

    /// `scoreboard objectives add <name> <criterion> <displayName>` — create with a display name.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::create_with_display` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::remove` for the canonical contract."]
    pub fn remove(&self) -> String {
        format!("scoreboard objectives remove {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot> <name>` — show in a display slot.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::set_display` for the canonical contract."]
    pub fn set_display(&self, slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot} {}", self.name)
    }

    /// `scoreboard objectives setdisplay <slot>` — clear the given display slot.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::clear_display` for the canonical contract."]
    pub fn clear_display(slot: DisplaySlot) -> String {
        format!("scoreboard objectives setdisplay {slot}")
    }

    /// `scoreboard objectives modify <name> displayname <text>` — change the display name.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::modify_display_name` for the canonical contract."]
    pub fn modify_display_name(&self, display: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} displayname {}",
            self.name,
            display.into()
        )
    }

    /// `scoreboard objectives modify <name> rendertype <type>` — change render type.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::modify_render_type` for the canonical contract."]
    pub fn modify_render_type(&self, render_type: impl Into<String>) -> String {
        format!(
            "scoreboard objectives modify {} rendertype {}",
            self.name,
            render_type.into()
        )
    }

    // ── Direct manipulation ────────────────────────────────────────────────

    /// `scoreboard players set <holder> <obj> <value>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::set` for the canonical contract."]
    pub fn set(&self, holder: ScoreHolder, value: i32) -> String {
        format!("scoreboard players set {} {} {}", holder, self.name, value)
    }

    /// `scoreboard players get <holder> <obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::get` for the canonical contract."]
    pub fn get(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players get {} {}", holder, self.name)
    }

    /// `scoreboard players add <holder> <obj> <amount>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::add` for the canonical contract."]
    pub fn add(&self, holder: ScoreHolder, amount: i32) -> String {
        format!("scoreboard players add {} {} {}", holder, self.name, amount)
    }

    /// `scoreboard players remove <holder> <obj> <amount>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::subtract` for the canonical contract."]
    pub fn subtract(&self, holder: ScoreHolder, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            holder, self.name, amount
        )
    }

    /// `scoreboard players reset <holder> <obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::reset` for the canonical contract."]
    pub fn reset(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players reset {} {}", holder, self.name)
    }

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> <op> <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::operation` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::enable` for the canonical contract."]
    pub fn enable(&self, holder: ScoreHolder) -> String {
        format!("scoreboard players enable {} {}", holder, self.name)
    }

    /// `scoreboard players set * <obj> <value>` — set score for ALL tracked players.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::set_all` for the canonical contract."]
    pub fn set_all(&self, value: i32) -> String {
        format!("scoreboard players set * {} {}", self.name, value)
    }

    /// `scoreboard players reset * <obj>` — reset scores for ALL tracked players.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::reset_all` for the canonical contract."]
    pub fn reset_all(&self) -> String {
        format!("scoreboard players reset * {}", self.name)
    }

    // ── Named operation shortcuts ──────────────────────────────────────────

    /// `scoreboard players operation <lhs> <obj> += <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::add_from` for the canonical contract."]
    pub fn add_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Add, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> -= <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::sub_from` for the canonical contract."]
    pub fn sub_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Sub, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> *= <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::mul_from` for the canonical contract."]
    pub fn mul_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mul, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> /= <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::div_from` for the canonical contract."]
    pub fn div_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Div, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> %= <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::mod_from` for the canonical contract."]
    pub fn mod_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Mod, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> = <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::copy_from` for the canonical contract."]
    pub fn copy_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Set, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> < <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::min_from` for the canonical contract."]
    pub fn min_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Min, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> > <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::max_from` for the canonical contract."]
    pub fn max_from(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Max, rhs, rhs_obj)
    }

    /// `scoreboard players operation <lhs> <obj> >< <rhs> <rhs_obj>`
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::swap_with` for the canonical contract."]
    pub fn swap_with(&self, lhs: ScoreHolder, rhs: ScoreHolder, rhs_obj: &Objective) -> String {
        self.operation(lhs, ScoreOp::Swap, rhs, rhs_obj)
    }

    // ── Execute conditions ─────────────────────────────────────────────────

    /// Return a condition fragment `if score <holder> <obj> matches <range>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::if_matches` for the canonical contract."]
    pub fn if_matches(&self, holder: ScoreHolder, range: impl Into<String>) -> String {
        format!("if score {} {} matches {}", holder, self.name, range.into())
    }

    /// Return a condition fragment `unless score <holder> <obj> matches <range>`.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::unless_matches` for the canonical contract."]
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
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::as_text` for the canonical contract."]
    pub fn as_text(&self, selector: Selector) -> TextComponent {
        TextComponent::score(selector.to_string(), self.name())
    }

    /// Create a `TextComponent` displaying a fake player's score in this objective.
    #[doc = "**API Contract:** Run `sand api show sand::command::Objective::as_text_fake` for the canonical contract."]
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
