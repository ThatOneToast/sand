//! Typed scoreboard variable — wraps a scoreboard objective for clean access.
//!
//! # API hierarchy (see [#146](https://github.com/ThatOneToast/sand/issues/146))
//!
//! 1. **Typed normal API** — `try_*` methods (e.g. [`ScoreVar::try_set`],
//!    [`ScoreVar::try_of`]) take a typed [`sand_commands::ScoreHolder`]
//!    (an entity selector, `@s`, a literal player name, or a `#`-prefixed
//!    fake player) and validate it before generating any command text.
//!    Prefer these in new code.
//! 2. **Validated compatibility adapter** — [`sand_commands::scoreboard::Objective`]
//!    offers the same validated surface directly against
//!    [`sand_commands::ObjectiveName`]/[`sand_commands::ScoreHolder`] for
//!    callers that don't need `ScoreVar`'s condition-builder ergonomics.
//! 3. **Raw escape hatch** — the plain (infallible) methods (e.g.
//!    [`ScoreVar::set`], [`ScoreVar::of`]) accept any `impl Display`/`&str`
//!    and interpolate it into command text without validation. They remain
//!    available for compatibility and for advanced/modded selector syntax
//!    Sand cannot yet type-check.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
use crate::condition::ConditionKind;
use crate::condition::{Condition, ScoreCompareOp, ScoreRange};
use crate::execute_when::Conditional;

/// One owned scoreboard entry used by score operations and comparisons.
///
/// Callers normally obtain an operand from [`ScoreRef::operand`] or
/// [`ScoreConst::ref_`] rather than constructing one directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreOperand {
    pub(crate) selector: String,
    pub(crate) objective: String,
}
use crate::state::storage::StorageField;

// ── Name utilities ────────────────────────────────────────────────────────────

/// Minecraft scoreboard objective names are limited to 16 characters.
///
/// Delegates to [`sand_commands::scoreboard::ObjectiveName::logical`] so
/// `ScoreVar`'s generated objective names use the exact same deterministic
/// hashing algorithm as `sand_commands::scoreboard::Objective` (see
/// [#146](https://github.com/ThatOneToast/sand/issues/146) — "do not leave
/// `sand-commands` and `sand-core` with unrelated validation and hashing
/// behavior"). If `name` fits and is already a valid direct objective token,
/// it is used verbatim; otherwise (too long, or containing characters an
/// objective name cannot use, such as whitespace) it is hashed to a stable,
/// always-valid ≤16-character name prefixed with `"s"`.
pub(super) fn objective_name(name: &str) -> String {
    sand_commands::ObjectiveName::logical(name)
        .as_str()
        .to_string()
}

fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 14_695_981_039_346_656_037;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

/// Vanilla scoreboard-player operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOperation {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Min,
    Max,
    Swap,
}

impl ScoreOperation {
    /// Render this operation as vanilla command syntax.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Add => "+=",
            Self::Sub => "-=",
            Self::Mul => "*=",
            Self::Div => "/=",
            Self::Mod => "%=",
            Self::Min => "<",
            Self::Max => ">",
            Self::Swap => "><",
        }
    }
}

/// A namespace for reusable fake-player score constants.
#[derive(Debug, Clone, Copy)]
pub struct ScoreConstants {
    objective: &'static str,
}

impl ScoreConstants {
    /// Create a constant namespace. Its objective is created automatically in
    /// Sand's generated load function when one of its constants is used.
    pub const fn new(objective: &'static str) -> Self {
        Self { objective }
    }

    /// Define a typed integer constant.
    pub const fn i32(&self, name: &'static str, value: i32) -> ScoreConst<i32> {
        ScoreConst {
            objective: self.objective,
            name,
            value,
            _marker: PhantomData,
        }
    }
}

/// A typed fake-player constant for scoreboard operations and comparisons.
#[derive(Debug, Clone, Copy)]
pub struct ScoreConst<T = i32> {
    objective: &'static str,
    name: &'static str,
    value: i32,
    _marker: PhantomData<T>,
}

impl<T> ScoreConst<T> {
    /// Construct a constant in Sand's default `sand_consts` objective.
    pub const fn new(name: &'static str, value: i32) -> Self {
        Self {
            objective: "sand_consts",
            name,
            value,
            _marker: PhantomData,
        }
    }

    /// Return this constant as a score operand and register its deterministic
    /// load-time setup. Reusing the same name/value is deduplicated; a
    /// conflicting definition panics during pack generation rather than
    /// silently changing scoreboard math.
    pub fn ref_(self) -> ScoreOperand {
        let objective = objective_name(self.objective);
        let holder = constant_holder(self.name);
        register_constant(&objective, &holder, self.value);
        ScoreOperand {
            selector: holder,
            objective,
        }
    }
}

fn constant_holder(name: &str) -> String {
    let clean: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        .collect();
    let clean = if clean.is_empty() { "const" } else { &clean };
    let prefix: String = clean.chars().take(28).collect();
    format!("#sand_{prefix}_{:06x}", fnv1a(name) & 0xFF_FFFF)
}

fn constants_registry() -> &'static Mutex<BTreeMap<(String, String), i32>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<(String, String), i32>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn register_constant(objective: &str, holder: &str, value: i32) {
    let mut registry = constants_registry()
        .lock()
        .expect("score constant registry poisoned");
    let key = (objective.to_string(), holder.to_string());
    if let Some(existing) = registry.insert(key.clone(), value)
        && existing != value
    {
        panic!(
            "conflicting Sand score constant `{}` in objective `{}`: {existing} versus {value}",
            holder, objective
        );
    }
}

fn score_constant_operand(prefix: &str, value: i32) -> ScoreOperand {
    let objective = objective_name("sand_consts");
    let holder = constant_holder(&format!("{prefix}_{value}"));
    register_constant(&objective, &holder, value);
    ScoreOperand {
        selector: holder,
        objective,
    }
}

fn score_set_command(target: &ScoreOperand, value: i32) -> String {
    format!(
        "scoreboard players set {} {} {value}",
        target.selector, target.objective
    )
}

fn score_operation_command(
    target: &ScoreOperand,
    op: ScoreOperation,
    source: &ScoreOperand,
) -> String {
    format!(
        "scoreboard players operation {} {} {} {} {}",
        target.selector,
        target.objective,
        op.as_str(),
        source.selector,
        source.objective
    )
}

/// Drain constant setup commands after all user command factories have run.
/// This is consumed by the export pipeline, not by user code.
#[doc(hidden)]
pub fn drain_constant_setup() -> Vec<String> {
    let mut registry = constants_registry()
        .lock()
        .expect("score constant registry poisoned");
    let entries = std::mem::take(&mut *registry);
    let mut objectives = entries
        .keys()
        .map(|(objective, _)| objective.clone())
        .collect::<Vec<_>>();
    objectives.sort();
    objectives.dedup();
    let mut commands: Vec<String> = objectives
        .into_iter()
        .map(|objective| format!("scoreboard objectives add {objective} dummy"))
        .collect();
    commands.extend(entries.into_iter().map(|((objective, holder), value)| {
        format!("scoreboard players set {holder} {objective} {value}")
    }));
    commands
}

pub(crate) fn request_expression_temp() {
    crate::function::request_internal_score_temp();
}

/// Drain all internally managed score setup. Used by the exporter.
#[doc(hidden)]
pub fn drain_internal_score_setup() -> Vec<String> {
    let mut commands = drain_constant_setup();
    if crate::function::take_internal_score_temp_request() {
        commands.insert(
            0,
            format!("scoreboard objectives add {SCORE_EXPRESSION_TEMP_OBJECTIVE} dummy"),
        );
    }
    commands
}

/// Sand's compiler-managed temporary objective used by score expressions.
pub(crate) const SCORE_EXPRESSION_TEMP_OBJECTIVE: &str = "__sand_tmp";

// ── ScoreVar ──────────────────────────────────────────────────────────────────

/// A typed scoreboard variable backed by a single scoreboard objective.
///
/// Declare once as a `static` and use throughout your datapack:
///
/// ```rust,ignore
/// use sand_core::state::ScoreVar;
///
/// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
///
/// let cmds = vec![
///     MANA.define(),
///     MANA.set("@s", 100),
///     MANA.add("@s", 5),
/// ];
/// ```
///
/// # API Contract
///
/// `sand api show sand::state::ScoreVar`
pub struct ScoreVar<T = i32> {
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> ScoreVar<T> {
    /// Create a new `ScoreVar` with the given objective name.
    ///
    /// Names longer than 16 characters are automatically hashed to a stable
    /// 16-character objective name (see [`ScoreVar::objective_name`]).
    ///
    /// # API Contract
    ///
    /// `sand api show sand::state::ScoreVar::new`
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    /// Return the actual scoreboard objective name used in commands.
    ///
    /// This is either `name` directly (≤16 chars) or a stable hash (>16 chars).
    pub fn objective_name(&self) -> String {
        objective_name(self.name)
    }

    /// `scoreboard objectives add <obj> dummy` — register the objective.
    ///
    /// Call this in your `load` function.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.objective_name())
    }

    /// `scoreboard players set <selector> <obj> <value>`
    pub fn set(&self, selector: impl std::fmt::Display, value: i32) -> String {
        format!(
            "scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            value
        )
    }

    /// `scoreboard players add <selector> <obj> <amount>`
    pub fn add(&self, selector: impl std::fmt::Display, amount: i32) -> String {
        format!(
            "scoreboard players add {} {} {}",
            selector,
            self.objective_name(),
            amount
        )
    }

    /// `scoreboard players remove <selector> <obj> <amount>`
    pub fn remove(&self, selector: impl std::fmt::Display, amount: i32) -> String {
        format!(
            "scoreboard players remove {} {} {}",
            selector,
            self.objective_name(),
            amount
        )
    }

    /// `scoreboard players reset <selector> <obj>`
    pub fn reset(&self, selector: impl std::fmt::Display) -> String {
        format!(
            "scoreboard players reset {} {}",
            selector,
            self.objective_name()
        )
    }

    /// Clamp the score for `selector` to `[min, max]`.
    ///
    /// Returns two commands: one to enforce the lower bound and one for the upper.
    ///
    /// Generated commands:
    /// ```text
    /// execute if score <selector> <obj> matches ..<min-1> run scoreboard players set <selector> <obj> <min>
    /// execute if score <selector> <obj> matches <max+1>.. run scoreboard players set <selector> <obj> <max>
    /// ```
    ///
    /// # `min > max`
    ///
    /// This infallible method does not reject `min > max` — it stays
    /// available so existing valid call sites keep byte-identical output.
    /// Two contradictory commands are emitted in that case (see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146)). Prefer
    /// [`ScoreVar::try_clamp`], which rejects `min > max` before returning
    /// any command text.
    ///
    /// # API Contract
    ///
    /// `sand api show sand::state::ScoreVar::clamp`
    pub fn clamp(&self, selector: impl std::fmt::Display, min: i32, max: i32) -> Vec<String> {
        let selector = selector.to_string();
        let obj = self.objective_name();
        vec![
            format!(
                "execute if score {selector} {obj} matches ..{} run scoreboard players set {selector} {obj} {min}",
                min.saturating_sub(1)
            ),
            format!(
                "execute if score {selector} {obj} matches {}.. run scoreboard players set {selector} {obj} {max}",
                max.saturating_add(1)
            ),
        ]
    }

    /// Validated counterpart to [`ScoreVar::clamp`] — rejects `min > max`
    /// before generating any mcfunction output instead of emitting two
    /// contradictory `execute if score ... matches` commands.
    pub fn try_clamp(
        &self,
        selector: impl std::fmt::Display,
        min: i32,
        max: i32,
    ) -> sand_commands::CommandResult<Vec<String>> {
        if min > max {
            return Err(sand_commands::CommandError::new(
                "ScoreVar::try_clamp",
                "min_max",
                format!("min ({min}) must not be greater than max ({max})"),
            )
            .with_code("SAND-SCORE-RANGE"));
        }
        Ok(self.clamp(selector, min, max))
    }

    /// Bind this variable to a selector to produce a condition builder.
    ///
    /// Compatibility/raw path: `selector` is an unvalidated string,
    /// interpolated directly into generated commands. Prefer
    /// [`ScoreVar::try_of`] in normal code — see
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25);
    /// ```
    pub fn of<'a>(&'a self, selector: &str) -> ScoreRef<'a, T> {
        ScoreRef {
            objective: self.name,
            selector: selector.to_string(),
            _marker: PhantomData,
        }
    }

    /// Validated counterpart to [`ScoreVar::of`] — takes a typed
    /// [`sand_commands::ScoreHolder`] (an entity selector or a fake player)
    /// and validates it before producing a condition builder, instead of
    /// interpolating an unvalidated selector string.
    ///
    /// ```
    /// use sand_core::state::ScoreVar;
    /// use sand_commands::ScoreHolder;
    ///
    /// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    ///
    /// let cond = MANA.try_of(ScoreHolder::self_()).unwrap().gte(25);
    /// assert!(MANA.try_of(ScoreHolder::fake("bad holder")).is_err());
    /// ```
    pub fn try_of<'a>(
        &'a self,
        holder: impl Into<sand_commands::ScoreHolder>,
    ) -> sand_commands::CommandResult<ScoreRef<'a, T>> {
        let holder = holder.into();
        // `execute if/unless score <holder> ...` requires exactly one score
        // holder — a wildcard or a multi-entity selector is not legal here,
        // even though it is legal for `set`/`add`/`remove`/`reset`/`init`.
        holder.validate_single(&sand_commands::CommandProfile::unprofiled())?;
        Ok(ScoreRef {
            objective: self.name,
            selector: holder.to_string(),
            _marker: PhantomData,
        })
    }

    /// Validated counterpart to [`ScoreVar::set`] — takes a typed
    /// [`sand_commands::ScoreHolder`] and validates it before generating the
    /// `scoreboard players set` command, instead of interpolating an
    /// unvalidated `Display` value. See
    /// [#146](https://github.com/ThatOneToast/sand/issues/146).
    ///
    /// ```
    /// use sand_core::state::ScoreVar;
    /// use sand_commands::ScoreHolder;
    ///
    /// static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    ///
    /// assert_eq!(
    ///     MANA.try_set(ScoreHolder::self_(), 100).unwrap(),
    ///     "scoreboard players set @s mana 100"
    /// );
    /// assert!(MANA.try_set(ScoreHolder::fake("bad holder"), 100).is_err());
    /// ```
    pub fn try_set(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
        value: i32,
    ) -> sand_commands::CommandResult<String> {
        let holder = holder.into();
        sand_commands::Validate::validate(&holder, &sand_commands::CommandProfile::unprofiled())?;
        Ok(self.set(holder.to_string(), value))
    }

    /// Validated counterpart to [`ScoreVar::add`] — see [`ScoreVar::try_set`].
    pub fn try_add(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
        amount: i32,
    ) -> sand_commands::CommandResult<String> {
        let holder = holder.into();
        sand_commands::Validate::validate(&holder, &sand_commands::CommandProfile::unprofiled())?;
        Ok(self.add(holder.to_string(), amount))
    }

    /// Validated counterpart to [`ScoreVar::remove`] — see [`ScoreVar::try_set`].
    pub fn try_remove(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
        amount: i32,
    ) -> sand_commands::CommandResult<String> {
        let holder = holder.into();
        sand_commands::Validate::validate(&holder, &sand_commands::CommandProfile::unprofiled())?;
        Ok(self.remove(holder.to_string(), amount))
    }

    /// Validated counterpart to [`ScoreVar::reset`] — see [`ScoreVar::try_set`].
    pub fn try_reset(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
    ) -> sand_commands::CommandResult<String> {
        let holder = holder.into();
        sand_commands::Validate::validate(&holder, &sand_commands::CommandProfile::unprofiled())?;
        Ok(self.reset(holder.to_string()))
    }

    /// Validated counterpart to [`ScoreVar::init`] — see [`ScoreVar::try_set`].
    pub fn try_init(
        &self,
        holder: impl Into<sand_commands::ScoreHolder>,
        value: i32,
    ) -> sand_commands::CommandResult<String> {
        let holder = holder.into();
        sand_commands::Validate::validate(&holder, &sand_commands::CommandProfile::unprofiled())?;
        Ok(self.init(holder.to_string(), value))
    }

    /// Validated counterpart to [`ScoreVar::copy_within`] — see
    /// [`ScoreVar::try_set`]. Both the source and destination holders are
    /// validated before any command text is generated.
    ///
    /// `scoreboard players operation <dst> <obj> = <src> <obj>` allows
    /// `dst` to resolve to multiple holders (the operation runs once per
    /// holder) but requires `src` — the copy source — to resolve to exactly
    /// one holder, matching vanilla's `scoreboard players operation`
    /// semantics. `src_holder` is validated with
    /// [`sand_commands::ScoreHolder::validate_single`] accordingly; a
    /// wildcard or multi-entity selector source is rejected.
    pub fn try_copy_within(
        &self,
        src_holder: impl Into<sand_commands::ScoreHolder>,
        dst_holder: impl Into<sand_commands::ScoreHolder>,
    ) -> sand_commands::CommandResult<String> {
        let src_holder = src_holder.into();
        let dst_holder = dst_holder.into();
        src_holder.validate_single(&sand_commands::CommandProfile::unprofiled())?;
        sand_commands::Validate::validate(
            &dst_holder,
            &sand_commands::CommandProfile::unprofiled(),
        )?;
        Ok(self.copy_within(src_holder.to_string(), dst_holder.to_string()))
    }

    /// Initialize the score to `value` only if the selector has no existing score entry.
    ///
    /// Uses `unless score … matches -2147483648..` to detect a missing score.
    ///
    /// Generated command:
    /// ```text
    /// execute unless score <selector> <obj> matches -2147483648.. run scoreboard players set <selector> <obj> <value>
    /// ```
    pub fn init(&self, selector: impl std::fmt::Display, value: i32) -> String {
        format!(
            "execute unless score {} {} matches -2147483648.. run scoreboard players set {} {} {}",
            selector,
            self.objective_name(),
            selector,
            self.objective_name(),
            value
        )
    }

    /// Copy this score from `src_selector` to `dst_selector` within the same objective.
    ///
    /// Generated command:
    /// ```text
    /// scoreboard players operation <dst> <obj> = <src> <obj>
    /// ```
    pub fn copy_within(
        &self,
        src_selector: impl std::fmt::Display,
        dst_selector: impl std::fmt::Display,
    ) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            dst_selector,
            self.objective_name(),
            src_selector,
            self.objective_name()
        )
    }

    /// Copy a score from another `ScoreVar` into this one for `selector`.
    ///
    /// Generated command:
    /// ```text
    /// scoreboard players operation <dst_sel> <self_obj> = <src_sel> <src_obj>
    /// ```
    pub fn copy_from<U>(
        &self,
        dst_selector: impl std::fmt::Display,
        src: &ScoreVar<U>,
        src_selector: impl std::fmt::Display,
    ) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            dst_selector,
            self.objective_name(),
            src_selector,
            src.objective_name()
        )
    }

    /// Copy this score into another `ScoreVar`.
    ///
    /// Generated command:
    /// ```text
    /// scoreboard players operation <dst_sel> <dst_obj> = <src_sel> <self_obj>
    /// ```
    pub fn copy_to<U>(
        &self,
        src_selector: impl std::fmt::Display,
        dst: &ScoreVar<U>,
        dst_selector: impl std::fmt::Display,
    ) -> String {
        format!(
            "scoreboard players operation {} {} = {} {}",
            dst_selector,
            dst.objective_name(),
            src_selector,
            self.objective_name()
        )
    }

    /// Set this score to the minimum of itself and another variable.
    ///
    /// Generated command:
    /// ```text
    /// scoreboard players operation <sel> <self_obj> < <other_sel> <other_obj>
    /// ```
    pub fn min_op<U>(
        &self,
        selector: impl std::fmt::Display,
        other: &ScoreVar<U>,
        other_selector: impl std::fmt::Display,
    ) -> String {
        format!(
            "scoreboard players operation {} {} < {} {}",
            selector,
            self.objective_name(),
            other_selector,
            other.objective_name()
        )
    }

    /// Set this score to the maximum of itself and another variable.
    ///
    /// Generated command:
    /// ```text
    /// scoreboard players operation <sel> <self_obj> > <other_sel> <other_obj>
    /// ```
    pub fn max_op<U>(
        &self,
        selector: impl std::fmt::Display,
        other: &ScoreVar<U>,
        other_selector: impl std::fmt::Display,
    ) -> String {
        format!(
            "scoreboard players operation {} {} > {} {}",
            selector,
            self.objective_name(),
            other_selector,
            other.objective_name()
        )
    }

    /// Condition: score equals zero.
    pub fn is_zero(&self, selector: &str) -> crate::condition::Condition {
        self.of(selector).is_zero()
    }

    /// Condition: score is not zero.
    pub fn is_nonzero(&self, selector: &str) -> crate::condition::Condition {
        self.of(selector).is_nonzero()
    }

    /// Condition: score is strictly positive (> 0).
    pub fn positive(&self, selector: &str) -> crate::condition::Condition {
        self.of(selector).positive()
    }

    /// Condition: score is strictly negative (< 0).
    pub fn negative(&self, selector: &str) -> crate::condition::Condition {
        self.of(selector).negative()
    }
}

// ── ScoreRef ──────────────────────────────────────────────────────────────────

/// A [`ScoreVar`] bound to a selector — used to build [`Condition`]s.
///
/// Produced by [`ScoreVar::of`].
pub struct ScoreRef<'a, T = i32> {
    /// The underlying `ScoreVar` name (used to derive the objective name).
    objective: &'a str,
    selector: String,
    _marker: PhantomData<T>,
}

impl<'a, T> ScoreRef<'a, T> {
    fn obj(&self) -> String {
        objective_name(self.objective)
    }

    /// `execute store result storage <field> int 1 run scoreboard players get
    /// <selector> <obj>` — copy this score into a typed storage field.
    ///
    /// The typed counterpart to reading storage into a score
    /// ([`StorageField::set`] combined with [`ScoreVar::set`]/`.add`); use
    /// this whenever a handler needs to snapshot a scoreboard value into NBT
    /// storage (e.g. for machine-readable evidence) without hand-writing an
    /// `execute store result storage ...` command.
    ///
    /// ```
    /// use sand_core::state::{ScoreVar, StorageSchema};
    ///
    /// static SEQ: ScoreVar = ScoreVar::new("seq");
    /// static AUDIT: StorageSchema<()> = StorageSchema::new("pack:audit", "audit");
    ///
    /// let cmd = SEQ.of("@s").store_into(AUDIT.field::<i32>("sequence"));
    /// assert_eq!(
    ///     cmd,
    ///     "execute store result storage pack:audit audit.sequence int 1 run scoreboard players get @s seq"
    /// );
    /// ```
    pub fn store_into<Schema, U>(&self, field: StorageField<Schema, U>) -> String {
        format!(
            "execute store result storage {} {} int 1 run scoreboard players get {} {}",
            field.storage(),
            field.full_path(),
            self.selector,
            self.obj()
        )
    }

    /// Return the typed scoreboard entry represented by this reference.
    pub fn operand(&self) -> ScoreOperand {
        ScoreOperand {
            selector: self.selector.clone(),
            objective: self.obj(),
        }
    }

    fn operation<O: Into<ScoreOperand>>(self, op: ScoreOperation, other: O) -> String {
        let left = self.operand();
        let right = other.into();
        score_operation_command(&left, op, &right)
    }

    /// Assign this score from another score entry (`=`).
    pub fn assign<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Assign, other)
    }

    /// Add another score entry (`+=`).
    pub fn add_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Add, other)
    }

    /// Subtract another score entry (`-=`).
    pub fn sub_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Sub, other)
    }

    /// Multiply by another score entry (`*=`).
    pub fn mul_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Mul, other)
    }

    /// Divide by another score entry (`/=`). Scoreboard math is integer-only;
    /// division by zero remains a vanilla runtime error.
    pub fn div_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Div, other)
    }

    /// Modulo another score entry (`%=`). Modulo by zero remains a vanilla
    /// runtime error.
    pub fn mod_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Mod, other)
    }

    /// Keep the minimum of this and another score entry (`<`).
    pub fn min_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Min, other)
    }

    /// Keep the maximum of this and another score entry (`>`).
    pub fn max_score<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Max, other)
    }

    /// Swap this score entry with another (`><`).
    pub fn swap<O: Into<ScoreOperand>>(self, other: O) -> String {
        self.operation(ScoreOperation::Swap, other)
    }

    /// Set this score to `value * percent / 100`.
    ///
    /// Scoreboard math is integer-only, so the final division truncates toward
    /// zero. Sand registers the generated percentage constants automatically.
    pub fn set_percent<O: Into<ScoreOperand>>(self, value: O, percent: i32) -> Vec<String> {
        let target = self.operand();
        let value = value.into();
        let percent = score_constant_operand("score_percent", percent);
        let hundred = score_constant_operand("score_percent_denominator", 100);
        vec![
            score_operation_command(&target, ScoreOperation::Assign, &value),
            score_operation_command(&target, ScoreOperation::Mul, &percent),
            score_operation_command(&target, ScoreOperation::Div, &hundred),
        ]
    }

    /// Scale this score in place by `percent / 100`.
    ///
    /// Scoreboard math is integer-only, so the final division truncates toward
    /// zero. Use [`ScoreRef::set_percent`] when the source and destination are
    /// different scores.
    pub fn scale_percent(self, percent: i32) -> Vec<String> {
        let target = self.operand();
        let percent = score_constant_operand("score_percent", percent);
        let hundred = score_constant_operand("score_percent_denominator", 100);
        vec![
            score_operation_command(&target, ScoreOperation::Mul, &percent),
            score_operation_command(&target, ScoreOperation::Div, &hundred),
        ]
    }

    /// Set this score to `current * scale / max`.
    ///
    /// This emits the direct vanilla operations and does not hide division by
    /// zero. Use [`ScoreRef::safe_divide`] when `max` may be zero.
    pub fn set_ratio<N: Into<ScoreOperand>, D: Into<ScoreOperand>>(
        self,
        current: N,
        max: D,
        scale: i32,
    ) -> Vec<String> {
        let target = self.operand();
        let current = current.into();
        let max = max.into();
        let scale = score_constant_operand("score_ratio_scale", scale);
        vec![
            score_operation_command(&target, ScoreOperation::Assign, &current),
            score_operation_command(&target, ScoreOperation::Mul, &scale),
            score_operation_command(&target, ScoreOperation::Div, &max),
        ]
    }

    /// Divide this score by another score only when the divisor is non-zero.
    ///
    /// If the divisor is zero, the target is set to `fallback` instead. This
    /// keeps generated output explicit about the branch that avoids vanilla's
    /// division-by-zero runtime failure.
    pub fn safe_divide<O: Into<ScoreOperand>>(self, divisor: O, fallback: i32) -> Vec<String> {
        let target = self.operand();
        let divisor = divisor.into();
        vec![
            format!(
                "execute unless score {} {} matches 0 run {}",
                divisor.selector,
                divisor.objective,
                score_operation_command(&target, ScoreOperation::Div, &divisor)
            ),
            format!(
                "execute if score {} {} matches 0 run {}",
                divisor.selector,
                divisor.objective,
                score_set_command(&target, fallback)
            ),
        ]
    }

    /// Clamp this score between two other score entries.
    ///
    /// The first command enforces the lower bound with vanilla's `>` operation
    /// (`max`), and the second enforces the upper bound with `<` (`min`).
    pub fn clamp_score<L: Into<ScoreOperand>, U: Into<ScoreOperand>>(
        self,
        min: L,
        max: U,
    ) -> Vec<String> {
        let target = self.operand();
        let min = min.into();
        let max = max.into();
        vec![
            score_operation_command(&target, ScoreOperation::Max, &min),
            score_operation_command(&target, ScoreOperation::Min, &max),
        ]
    }

    /// Add `amount` and then clamp to the literal `[min, max]` range.
    pub fn saturating_add(self, amount: i32, min: i32, max: i32) -> Vec<String> {
        let target = self.operand();
        vec![
            format!(
                "scoreboard players add {} {} {amount}",
                target.selector, target.objective
            ),
            format!(
                "execute if score {} {} matches ..{} run {}",
                target.selector,
                target.objective,
                min.saturating_sub(1),
                score_set_command(&target, min)
            ),
            format!(
                "execute if score {} {} matches {}.. run {}",
                target.selector,
                target.objective,
                max.saturating_add(1),
                score_set_command(&target, max)
            ),
        ]
    }

    /// Subtract `amount` and then clamp to the literal `[min, max]` range.
    pub fn saturating_sub(self, amount: i32, min: i32, max: i32) -> Vec<String> {
        let target = self.operand();
        vec![
            format!(
                "scoreboard players remove {} {} {amount}",
                target.selector, target.objective
            ),
            format!(
                "execute if score {} {} matches ..{} run {}",
                target.selector,
                target.objective,
                min.saturating_sub(1),
                score_set_command(&target, min)
            ),
            format!(
                "execute if score {} {} matches {}.. run {}",
                target.selector,
                target.objective,
                max.saturating_add(1),
                score_set_command(&target, max)
            ),
        ]
    }

    fn compare<O: Into<ScoreOperand>>(self, op: ScoreCompareOp, other: O) -> Condition {
        Condition::score_compare(self.operand(), op, other.into())
    }

    /// Compare this score to another score entry (`=`).
    pub fn eq_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        self.compare(ScoreCompareOp::Eq, other)
    }

    /// Compare this score as not equal to another score entry.
    pub fn ne_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        !self.eq_score(other)
    }

    /// Compare this score as greater than another score entry (`>`).
    pub fn gt_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        self.compare(ScoreCompareOp::Gt, other)
    }

    /// Compare this score as greater than or equal to another score entry (`>=`).
    pub fn gte_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        self.compare(ScoreCompareOp::Gte, other)
    }

    /// Compare this score as less than another score entry (`<`).
    pub fn lt_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        self.compare(ScoreCompareOp::Lt, other)
    }

    /// Compare this score as less than or equal to another score entry (`<=`).
    pub fn lte_score<O: Into<ScoreOperand>>(self, other: O) -> Condition {
        self.compare(ScoreCompareOp::Lte, other)
    }

    /// Begin an integer-only score expression. Its setup commands are emitted
    /// before the final branch condition is evaluated.
    pub fn expr(&self) -> ScoreExpr<T> {
        ScoreExpr {
            base: self.operand(),
            steps: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// `if score <sel> <obj> matches <n>` — equal to `n`.
    pub fn eq(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Eq(n))
    }

    /// `unless score <sel> <obj> matches <n>` — not equal to `n`.
    pub fn ne(self, n: i32) -> Condition {
        let objective = self.obj();
        !Condition::score(self.selector, objective, ScoreRange::Eq(n))
    }

    /// `if score <sel> <obj> matches <n+1>..` — strictly greater than `n`.
    pub fn gt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Gt(n))
    }

    /// `if score <sel> <obj> matches <n>..` — greater than or equal to `n`.
    pub fn gte(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Gte(n))
    }

    /// `if score <sel> <obj> matches ..<n-1>` — strictly less than `n`.
    pub fn lt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Lt(n))
    }

    /// Condition: score equals zero.
    pub fn is_zero(self) -> Condition {
        self.eq(0)
    }

    /// Condition: score is not zero.
    pub fn is_nonzero(self) -> Condition {
        self.ne(0)
    }

    /// Condition: score is strictly positive (`matches 1..`).
    pub fn positive(self) -> Condition {
        self.gt(0)
    }

    /// Condition: score is strictly negative (`matches ..-1`).
    pub fn negative(self) -> Condition {
        self.lt(0)
    }

    /// `if score <sel> <obj> matches ..<n>` — less than or equal to `n`.
    pub fn lte(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Lte(n))
    }

    /// `if score <sel> <obj> matches <min>..<max>` — inside an inclusive range.
    pub fn between(self, min: i32, max: i32) -> Condition {
        let objective = self.obj();
        Condition::score(
            self.selector,
            objective,
            ScoreRange::Between(Some(min), Some(max)),
        )
    }

    /// `unless score <sel> <obj> matches <min>..<max>` — outside an inclusive range.
    pub fn outside(self, min: i32, max: i32) -> Condition {
        !self.between(min, max)
    }

    /// Validated `matches <n+1>..` — strictly greater than `n`.
    ///
    /// Rejects `n == i32::MAX`, which describes a range no `i32` score can
    /// satisfy; for example, no `i32` score can be greater than `i32::MAX`.
    pub fn try_gt(self, n: i32) -> sand_commands::CommandResult<Condition> {
        ScoreRange::Gt(n).validate()?;
        Ok(self.gt(n))
    }

    /// Validated `matches ..<n-1>` — strictly less than `n`.
    ///
    /// Rejects `n == i32::MIN`, which describes a range no `i32` score can
    /// satisfy.
    pub fn try_lt(self, n: i32) -> sand_commands::CommandResult<Condition> {
        ScoreRange::Lt(n).validate()?;
        Ok(self.lt(n))
    }

    /// Validated inclusive range — rejects `min > max` instead of emitting an
    /// always-false `matches` fragment.
    pub fn try_between(self, min: i32, max: i32) -> sand_commands::CommandResult<Condition> {
        ScoreRange::Between(Some(min), Some(max)).validate()?;
        Ok(self.between(min, max))
    }

    /// `if score <sel> <obj> matches <lo>..<hi>` — within an inclusive range.
    ///
    /// Accepts any `RangeBounds<i32>`: `1..=100`, `0..`, `..100`, etc.
    pub fn matches(self, range: impl RangeBounds<i32>) -> Condition {
        use std::ops::Bound;
        let lo = match range.start_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n + 1),
            Bound::Unbounded => None,
        };
        let hi = match range.end_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n - 1),
            Bound::Unbounded => None,
        };
        let objective = self.obj();
        Condition::score(self.selector, objective, ScoreRange::Between(lo, hi))
    }

    /// Validated counterpart to [`ScoreRef::matches`] — rejects a range whose
    /// resolved bounds have `lo > hi` instead of emitting an always-false
    /// `matches` fragment.
    pub fn try_matches(
        self,
        range: impl RangeBounds<i32>,
    ) -> sand_commands::CommandResult<Condition> {
        use std::ops::Bound;
        let lo = match range.start_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n + 1),
            Bound::Unbounded => None,
        };
        let hi = match range.end_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n - 1),
            Bound::Unbounded => None,
        };
        ScoreRange::Between(lo, hi).validate()?;
        let objective = self.obj();
        Ok(Condition::score(
            self.selector,
            objective,
            ScoreRange::Between(lo, hi),
        ))
    }
}

impl<'a, T> From<ScoreRef<'a, T>> for ScoreOperand {
    fn from(value: ScoreRef<'a, T>) -> Self {
        value.operand()
    }
}

/// A compiler-managed sequence of vanilla scoreboard operations.
pub struct ScoreExpr<T = i32> {
    base: ScoreOperand,
    steps: Vec<(ScoreOperation, ScoreOperand)>,
    _marker: PhantomData<T>,
}

impl<T> ScoreExpr<T> {
    fn operation<O: Into<ScoreOperand>>(mut self, op: ScoreOperation, other: O) -> Self {
        self.steps.push((op, other.into()));
        self
    }

    /// Adds another score operand to this expression.
    pub fn plus<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Add, other)
    }
    /// Subtracts another score operand from this expression.
    pub fn minus<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Sub, other)
    }
    /// Multiplies this expression by another score operand.
    #[allow(clippy::should_implement_trait)]
    pub fn mul<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Mul, other)
    }
    /// Divides this expression by another score operand using scoreboard arithmetic.
    #[allow(clippy::should_implement_trait)]
    pub fn div<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Div, other)
    }
    /// Applies scoreboard remainder arithmetic with another score operand.
    pub fn modulo<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Mod, other)
    }
    /// Clamps this expression to the lesser of itself and another score operand.
    pub fn min<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Min, other)
    }
    /// Clamps this expression to the greater of itself and another score operand.
    pub fn max<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Max, other)
    }

    fn lowered(self, condition: Condition) -> Conditional {
        request_expression_temp();
        let temp = ScoreOperand {
            selector: self.base.selector.clone(),
            objective: SCORE_EXPRESSION_TEMP_OBJECTIVE.to_string(),
        };
        let mut setup = vec![format!(
            "scoreboard players operation {} {} = {} {}",
            temp.selector, temp.objective, self.base.selector, self.base.objective
        )];
        setup.extend(self.steps.into_iter().map(|(op, right)| {
            format!(
                "scoreboard players operation {} {} {} {} {}",
                temp.selector,
                temp.objective,
                op.as_str(),
                right.selector,
                right.objective
            )
        }));
        Conditional::with_setup(setup, condition)
    }

    fn temp(&self) -> ScoreOperand {
        ScoreOperand {
            selector: self.base.selector.clone(),
            objective: SCORE_EXPRESSION_TEMP_OBJECTIVE.to_string(),
        }
    }

    /// Builds a condition requiring this expression to equal an integer.
    pub fn eq(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Eq(n),
        ))
    }
    /// Builds a condition requiring this expression to exceed an integer.
    pub fn gt(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Gt(n),
        ))
    }
    /// Builds a condition requiring this expression to be at least an integer.
    pub fn gte(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Gte(n),
        ))
    }
    /// Builds a condition requiring this expression to be below an integer.
    pub fn lt(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Lt(n),
        ))
    }
    /// Builds a condition requiring this expression to be at most an integer.
    pub fn lte(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Lte(n),
        ))
    }
    /// Builds a condition requiring this expression to fall within a score range.
    pub fn matches(self, range: impl RangeBounds<i32>) -> Conditional {
        use std::ops::Bound;
        let lo = match range.start_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n + 1),
            Bound::Unbounded => None,
        };
        let hi = match range.end_bound() {
            Bound::Included(&n) => Some(n),
            Bound::Excluded(&n) => Some(n - 1),
            Bound::Unbounded => None,
        };
        let temp = self.temp();
        self.lowered(Condition::score(
            temp.selector,
            temp.objective,
            ScoreRange::Between(lo, hi),
        ))
    }
    /// Builds a condition requiring this expression to equal another score operand.
    pub fn eq_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::score_compare(
            left,
            ScoreCompareOp::Eq,
            other.into(),
        ))
    }
    /// Builds a condition requiring this expression to exceed another score operand.
    pub fn gt_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::score_compare(
            left,
            ScoreCompareOp::Gt,
            other.into(),
        ))
    }
    /// Builds a condition requiring this expression to be at least another score operand.
    pub fn gte_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::score_compare(
            left,
            ScoreCompareOp::Gte,
            other.into(),
        ))
    }
    /// Builds a condition requiring this expression to be below another score operand.
    pub fn lt_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::score_compare(
            left,
            ScoreCompareOp::Lt,
            other.into(),
        ))
    }
    /// Builds a condition requiring this expression to be at most another score operand.
    pub fn lte_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::score_compare(
            left,
            ScoreCompareOp::Lte,
            other.into(),
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static LONG: ScoreVar<i32> = ScoreVar::new("this_is_a_very_long_name_that_exceeds_limit");

    #[test]
    fn short_name_unchanged() {
        assert_eq!(MANA.objective_name(), "mana");
    }

    #[test]
    fn long_name_hashed_stable() {
        let a = LONG.objective_name();
        let b = LONG.objective_name();
        assert_eq!(a, b, "hash must be deterministic");
        assert!(
            a.len() <= 16,
            "hashed name must be ≤16 chars, got {}",
            a.len()
        );
        assert!(a.starts_with('s'), "hashed name must start with 's'");
    }

    #[test]
    fn long_name_hashing_matches_sand_commands_canonical_algorithm() {
        // #146: sand-core's ScoreVar and sand-commands' Objective/ObjectiveName
        // must share one hashing algorithm, not two independently-maintained
        // ones. Prove ScoreVar's emitted name equals the canonical
        // ObjectiveName::logical output directly.
        static LOCAL: ScoreVar<i32> = ScoreVar::new("this_is_a_very_long_name_that_exceeds_limit");
        assert_eq!(
            LOCAL.objective_name(),
            sand_commands::ObjectiveName::logical("this_is_a_very_long_name_that_exceeds_limit")
                .as_str()
        );
    }

    #[test]
    fn short_invalid_name_is_hashed_rather_than_emitted_verbatim() {
        // Previously a bug: a short (<=16 char) name with a space would be
        // emitted verbatim (invalid `scoreboard objectives add` syntax).
        // objective_name now routes through ObjectiveName::logical, which
        // falls back to hashing instead of emitting an invalid direct token.
        static BAD: ScoreVar<i32> = ScoreVar::new("bad name");
        let emitted = BAD.objective_name();
        assert_ne!(emitted, "bad name");
        assert!(emitted.starts_with('s'));
        assert!(emitted.len() <= 16);
        assert!(!emitted.contains(' '));
    }

    #[test]
    fn define_cmd() {
        assert_eq!(MANA.define(), "scoreboard objectives add mana dummy");
    }

    #[test]
    fn set_cmd() {
        assert_eq!(MANA.set("@s", 100), "scoreboard players set @s mana 100");
    }

    #[test]
    fn add_cmd() {
        assert_eq!(MANA.add("@s", 5), "scoreboard players add @s mana 5");
    }

    #[test]
    fn remove_cmd() {
        assert_eq!(
            MANA.remove("@s", 10),
            "scoreboard players remove @s mana 10"
        );
    }

    #[test]
    fn reset_cmd() {
        assert_eq!(MANA.reset("@s"), "scoreboard players reset @s mana");
    }

    #[test]
    fn clamp_cmds() {
        let cmds = MANA.clamp("@s", 0, 100);
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("matches ..-1"), "got: {}", cmds[0]);
        assert!(cmds[1].contains("matches 101.."), "got: {}", cmds[1]);
    }

    #[test]
    fn try_clamp_matches_clamp_for_valid_range() {
        assert_eq!(
            MANA.try_clamp("@s", 0, 100).unwrap(),
            MANA.clamp("@s", 0, 100)
        );
    }

    #[test]
    fn try_clamp_rejects_min_greater_than_max() {
        assert!(MANA.try_clamp("@s", 10, 5).is_err());
    }

    #[test]
    fn try_clamp_diagnostic_code_is_stable() {
        let err = MANA.try_clamp("@s", 10, 5).unwrap_err();
        assert_eq!(err.code, "SAND-SCORE-RANGE");
    }

    #[test]
    fn try_gt_rejects_i32_max() {
        assert!(MANA.of("@s").try_gt(i32::MAX).is_err());
        assert!(MANA.of("@s").try_gt(10).is_ok());
    }

    #[test]
    fn try_lt_rejects_i32_min() {
        assert!(MANA.of("@s").try_lt(i32::MIN).is_err());
        assert!(MANA.of("@s").try_lt(10).is_ok());
    }

    #[test]
    fn try_between_rejects_min_greater_than_max() {
        assert!(MANA.of("@s").try_between(10, 5).is_err());
        assert!(MANA.of("@s").try_between(5, 10).is_ok());
    }

    #[test]
    fn try_matches_rejects_impossible_range() {
        #[allow(clippy::reversed_empty_ranges)]
        let bad = MANA.of("@s").try_matches(10..=5);
        assert!(bad.is_err());
        assert!(MANA.of("@s").try_matches(1..=100).is_ok());
    }

    #[test]
    fn condition_gte() {
        let cond = MANA.of("@s").gte(25);
        match cond.kind() {
            ConditionKind::Score {
                selector,
                objective,
                range: ScoreRange::Gte(25),
            } => {
                assert_eq!(selector, "@s");
                assert_eq!(objective, "mana");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_lte() {
        let cond = MANA.of("@s").lte(100);
        match cond.kind() {
            ConditionKind::Score {
                range: ScoreRange::Lte(100),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_ne_wraps_not() {
        let cond = MANA.of("@s").ne(0);
        assert!(matches!(cond.kind(), ConditionKind::Not(_)));
    }

    #[test]
    fn condition_matches_range() {
        let cond = MANA.of("@s").matches(1..=100);
        match cond.kind() {
            ConditionKind::Score {
                range: ScoreRange::Between(Some(1), Some(100)),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_between() {
        let cond = MANA.of("@s").between(10, 100);
        match cond.kind() {
            ConditionKind::Score {
                range: ScoreRange::Between(Some(10), Some(100)),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn init_cmd_uses_unless() {
        let cmd = MANA.init("@s", 100);
        assert!(
            cmd.contains("unless score @s mana matches -2147483648.."),
            "got: {cmd}"
        );
        assert!(cmd.contains("set @s mana 100"), "got: {cmd}");
    }

    #[test]
    fn copy_within_cmd() {
        let cmd = MANA.copy_within("@s", "@p");
        assert_eq!(cmd, "scoreboard players operation @p mana = @s mana");
    }

    #[test]
    fn copy_to_cmd() {
        static OTHER: ScoreVar<i32> = ScoreVar::new("other");
        let cmd = MANA.copy_to("@s", &OTHER, "@p");
        assert_eq!(cmd, "scoreboard players operation @p other = @s mana");
    }

    #[test]
    fn copy_from_cmd() {
        static SRC: ScoreVar<i32> = ScoreVar::new("src");
        let cmd = MANA.copy_from("@s", &SRC, "@p");
        assert_eq!(cmd, "scoreboard players operation @s mana = @p src");
    }

    #[test]
    fn min_op_cmd() {
        static CAP: ScoreVar<i32> = ScoreVar::new("cap");
        let cmd = MANA.min_op("@s", &CAP, "@s");
        assert_eq!(cmd, "scoreboard players operation @s mana < @s cap");
    }

    #[test]
    fn max_op_cmd() {
        static FLOOR: ScoreVar<i32> = ScoreVar::new("floor");
        let cmd = MANA.max_op("@s", &FLOOR, "@s");
        assert_eq!(cmd, "scoreboard players operation @s mana > @s floor");
    }

    #[test]
    fn is_zero_condition() {
        let cond = MANA.is_zero("@s");
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn is_nonzero_condition() {
        let cond = MANA.is_nonzero("@s");
        assert!(matches!(cond.kind(), ConditionKind::Not(_)));
    }

    #[test]
    fn positive_condition() {
        let cond = MANA.positive("@s");
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
                range: ScoreRange::Gt(0),
                ..
            }
        ));
    }

    #[test]
    fn negative_condition() {
        let cond = MANA.negative("@s");
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
                range: ScoreRange::Lt(0),
                ..
            }
        ));
    }

    #[test]
    fn scoreref_is_zero() {
        let cond = MANA.of("@s").is_zero();
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn scoreref_positive() {
        let cond = MANA.of("@s").positive();
        assert!(matches!(
            cond.kind(),
            ConditionKind::Score {
                range: ScoreRange::Gt(0),
                ..
            }
        ));
    }

    #[test]
    fn condition_outside_wraps_not() {
        let cond = MANA.of("@s").outside(10, 100);
        assert!(matches!(cond.kind(), ConditionKind::Not(_)));
    }

    #[test]
    fn condition_matches_open_end() {
        let cond = MANA.of("@s").matches(25..);
        match cond.kind() {
            ConditionKind::Score {
                range: ScoreRange::Between(Some(25), None),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn score_comparisons_render_all_vanilla_operators() {
        static COST: ScoreVar<i32> = ScoreVar::new("cost");
        let cases = [
            (MANA.of("@s").eq_score(COST.of("@s")), "="),
            (MANA.of("@s").gt_score(COST.of("@s")), ">"),
            (MANA.of("@s").gte_score(COST.of("@p")), ">="),
            (MANA.of("@s").lt_score(COST.of("@p")), "<"),
            (MANA.of("@s").lte_score(COST.of("@p")), "<="),
        ];
        for (condition, operator) in cases {
            assert_eq!(
                condition.execute_commands(false, "say ok")[0],
                format!(
                    "execute if score @s mana {operator} {} cost run say ok",
                    if operator == "=" || operator == ">" {
                        "@s"
                    } else {
                        "@p"
                    }
                )
            );
        }
    }

    #[test]
    fn score_operations_render_all_symbols() {
        static OTHER: ScoreVar<i32> = ScoreVar::new("other");
        let actual = [
            MANA.of("@s").assign(OTHER.of("@p")),
            MANA.of("@s").add_score(OTHER.of("@p")),
            MANA.of("@s").sub_score(OTHER.of("@p")),
            MANA.of("@s").mul_score(OTHER.of("@p")),
            MANA.of("@s").div_score(OTHER.of("@p")),
            MANA.of("@s").mod_score(OTHER.of("@p")),
            MANA.of("@s").min_score(OTHER.of("@p")),
            MANA.of("@s").max_score(OTHER.of("@p")),
            MANA.of("@s").swap(OTHER.of("@p")),
        ];
        for (command, operator) in actual
            .iter()
            .zip(["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"])
        {
            assert_eq!(
                command,
                &format!("scoreboard players operation @s mana {operator} @p other")
            );
        }
    }

    #[test]
    fn constants_register_setup_and_support_negative_values() {
        let constant = ScoreConst::<i32>::new("negative scale", -2);
        let command = MANA.of("@s").mul_score(constant.ref_());
        assert!(command.contains("#sand_negative"));
        let setup = drain_internal_score_setup();
        assert!(
            setup
                .iter()
                .any(|line| line == "scoreboard objectives add sand_consts dummy")
        );
        assert!(setup.iter().any(|line| line.ends_with(" sand_consts -2")));
    }

    #[test]
    fn set_percent_generates_integer_math_pipeline() {
        static DAMAGE: ScoreVar<i32> = ScoreVar::new("damage");
        let cmds = MANA.of("@s").set_percent(DAMAGE.of("@s"), 150);

        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "scoreboard players operation @s mana = @s damage");
        assert!(
            cmds[1].starts_with("scoreboard players operation @s mana *= #sand_score_percent_150_")
        );
        assert!(cmds[1].ends_with(" sand_consts"));
        assert!(
            cmds[2].starts_with(
                "scoreboard players operation @s mana /= #sand_score_percent_denominator_"
            ),
            "got: {}",
            cmds[2]
        );
        assert!(cmds[2].ends_with(" sand_consts"));
    }

    #[test]
    fn scale_percent_updates_target_in_place() {
        let cmds = MANA.of("@s").scale_percent(75);

        assert_eq!(cmds.len(), 2);
        assert!(
            cmds[0].starts_with("scoreboard players operation @s mana *= #sand_score_percent_75_")
        );
        assert!(cmds[0].ends_with(" sand_consts"));
        assert!(
            cmds[1].starts_with(
                "scoreboard players operation @s mana /= #sand_score_percent_denominator_"
            ),
            "got: {}",
            cmds[1]
        );
    }

    #[test]
    fn set_ratio_generates_current_scale_divide_pipeline() {
        static HEALTH: ScoreVar<i32> = ScoreVar::new("health");
        static MAX_HEALTH: ScoreVar<i32> = ScoreVar::new("max_health");

        let cmds = MANA
            .of("@s")
            .set_ratio(HEALTH.of("@s"), MAX_HEALTH.of("@s"), 100);

        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "scoreboard players operation @s mana = @s health");
        assert!(
            cmds[1].starts_with(
                "scoreboard players operation @s mana *= #sand_score_ratio_scale_100_"
            )
        );
        assert!(cmds[1].ends_with(" sand_consts"));
        assert_eq!(
            cmds[2],
            "scoreboard players operation @s mana /= @s max_health"
        );
    }

    #[test]
    fn safe_divide_guards_zero_divisor() {
        static MAX_HEALTH: ScoreVar<i32> = ScoreVar::new("max_health");

        let cmds = MANA.of("@s").safe_divide(MAX_HEALTH.of("@s"), 0);

        assert_eq!(
            cmds,
            vec![
                "execute unless score @s max_health matches 0 run scoreboard players operation @s mana /= @s max_health",
                "execute if score @s max_health matches 0 run scoreboard players set @s mana 0",
            ]
        );
    }

    #[test]
    fn safe_divide_supports_same_target_and_divisor() {
        let cmds = MANA.of("@s").safe_divide(MANA.of("@s"), 42);

        assert_eq!(
            cmds,
            vec![
                "execute unless score @s mana matches 0 run scoreboard players operation @s mana /= @s mana",
                "execute if score @s mana matches 0 run scoreboard players set @s mana 42",
            ]
        );
    }

    #[test]
    fn clamp_score_uses_vanilla_min_max_operations() {
        static MIN_MANA: ScoreVar<i32> = ScoreVar::new("min_mana");
        static MAX_MANA: ScoreVar<i32> = ScoreVar::new("max_mana");

        let cmds = MANA
            .of("@s")
            .clamp_score(MIN_MANA.of("@s"), MAX_MANA.of("@s"));

        assert_eq!(
            cmds,
            vec![
                "scoreboard players operation @s mana > @s min_mana",
                "scoreboard players operation @s mana < @s max_mana",
            ]
        );
    }

    #[test]
    fn saturating_add_and_sub_emit_literal_clamps() {
        let add = MANA.of("@s").saturating_add(5, 0, 100);
        assert_eq!(
            add,
            vec![
                "scoreboard players add @s mana 5",
                "execute if score @s mana matches ..-1 run scoreboard players set @s mana 0",
                "execute if score @s mana matches 101.. run scoreboard players set @s mana 100",
            ]
        );

        let sub = MANA.of("@s").saturating_sub(20, 0, 100);
        assert_eq!(
            sub,
            vec![
                "scoreboard players remove @s mana 20",
                "execute if score @s mana matches ..-1 run scoreboard players set @s mana 0",
                "execute if score @s mana matches 101.. run scoreboard players set @s mana 100",
            ]
        );
    }

    // ── #146: typed ScoreHolder-validated API ─────────────────────────────

    use sand_commands::ScoreHolder;

    #[test]
    fn try_set_matches_infallible_set_for_valid_holder() {
        assert_eq!(
            MANA.try_set(ScoreHolder::self_(), 100).unwrap(),
            MANA.set("@s", 100)
        );
    }

    #[test]
    fn try_add_try_remove_try_reset_match_infallible_variants() {
        assert_eq!(
            MANA.try_add(ScoreHolder::self_(), 5).unwrap(),
            MANA.add("@s", 5)
        );
        assert_eq!(
            MANA.try_remove(ScoreHolder::self_(), 5).unwrap(),
            MANA.remove("@s", 5)
        );
        assert_eq!(
            MANA.try_reset(ScoreHolder::self_()).unwrap(),
            MANA.reset("@s")
        );
    }

    #[test]
    fn try_init_matches_infallible_init() {
        assert_eq!(
            MANA.try_init(ScoreHolder::self_(), 100).unwrap(),
            MANA.init("@s", 100)
        );
    }

    #[test]
    fn try_copy_within_matches_infallible_copy_within() {
        assert_eq!(
            MANA.try_copy_within(ScoreHolder::self_(), ScoreHolder::player("Notch"))
                .unwrap(),
            MANA.copy_within("@s", "Notch")
        );
    }

    #[test]
    fn try_of_matches_infallible_of() {
        let cond = MANA.try_of(ScoreHolder::self_()).unwrap().gte(25);
        assert_eq!(cond, MANA.of("@s").gte(25));
    }

    #[test]
    fn typed_holder_apis_reject_invalid_fake_player_holders() {
        // A fake-player holder with whitespace is not valid vanilla
        // score-holder syntax; the typed path must reject it rather than
        // silently emit malformed mcfunction output.
        let bad = ScoreHolder::fake("bad holder");
        assert!(MANA.try_set(bad.clone(), 1).is_err());
        assert!(MANA.try_add(bad.clone(), 1).is_err());
        assert!(MANA.try_remove(bad.clone(), 1).is_err());
        assert!(MANA.try_reset(bad.clone()).is_err());
        assert!(MANA.try_init(bad.clone(), 1).is_err());
        assert!(
            MANA.try_copy_within(bad.clone(), ScoreHolder::self_())
                .is_err()
        );
        assert!(
            MANA.try_copy_within(ScoreHolder::self_(), bad.clone())
                .is_err()
        );
        assert!(MANA.try_of(bad).is_err());
    }

    #[test]
    fn typed_holder_apis_accept_fake_players_and_wildcards() {
        assert!(MANA.try_set(ScoreHolder::fake("#total_kills"), 0).is_ok());
        assert!(MANA.try_reset(ScoreHolder::wildcard()).is_ok());
        assert!(MANA.try_set(ScoreHolder::player("Notch"), 1).is_ok());
    }

    #[test]
    fn typed_holder_apis_accept_entity_selectors() {
        use sand_commands::selector::Selector;
        assert!(
            MANA.try_set(ScoreHolder::entity(Selector::all_players()), 1)
                .is_ok()
        );
        assert!(
            MANA.try_add(ScoreHolder::entity(Selector::self_()), 1)
                .is_ok()
        );
    }

    #[test]
    fn try_of_rejects_wildcard_and_multi_entity_holders() {
        // `execute if/unless score <holder> ...` requires exactly one score
        // holder; unlike `set`/`add`/`remove`/`reset`, a wildcard or a
        // selector matching more than one entity must be rejected rather
        // than silently producing `execute if score * mana matches ...` /
        // `execute if score @a mana matches ...`, which Minecraft refuses to
        // parse.
        use sand_commands::selector::Selector;
        assert!(MANA.try_of(ScoreHolder::wildcard()).is_err());
        assert!(
            MANA.try_of(ScoreHolder::entity(Selector::all_players()))
                .is_err()
        );
        assert!(
            MANA.try_of(ScoreHolder::entity(Selector::all_entities()))
                .is_err()
        );
        // Single-holder cases remain accepted.
        assert!(MANA.try_of(ScoreHolder::self_()).is_ok());
        assert!(MANA.try_of(ScoreHolder::player("Notch")).is_ok());
    }

    #[test]
    fn try_copy_within_rejects_wildcard_or_multi_entity_source() {
        // `scoreboard players operation <dst> <obj> = <src> <obj>` allows a
        // multi-holder `dst` but requires a single-holder `src`.
        use sand_commands::selector::Selector;
        assert!(
            MANA.try_copy_within(ScoreHolder::wildcard(), ScoreHolder::self_())
                .is_err()
        );
        assert!(
            MANA.try_copy_within(
                ScoreHolder::entity(Selector::all_players()),
                ScoreHolder::self_()
            )
            .is_err()
        );
        // A multi-holder destination is fine — only the source must be single.
        assert!(
            MANA.try_copy_within(ScoreHolder::self_(), ScoreHolder::wildcard())
                .is_ok()
        );
        assert!(
            MANA.try_copy_within(
                ScoreHolder::self_(),
                ScoreHolder::entity(Selector::all_players())
            )
            .is_ok()
        );
    }
}
