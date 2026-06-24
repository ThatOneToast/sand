//! Typed scoreboard variable — wraps a scoreboard objective for clean access.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::sync::{Mutex, OnceLock};

use crate::condition::{Condition, ScoreCompareOp, ScoreOperand, ScoreRange};
use crate::execute_when::Conditional;

// ── Name utilities ────────────────────────────────────────────────────────────

/// Minecraft scoreboard objective names are limited to 16 characters.
/// If the requested name exceeds that limit, a stable FNV-1a hash is used
/// to produce a deterministic 16-character name prefixed with `"s"`.
pub(super) fn objective_name(name: &str) -> String {
    if name.len() <= 16 {
        name.to_string()
    } else {
        let hash = fnv1a(name);
        format!("s{:015x}", hash & 0x0FFF_FFFF_FFFF_FFFF)
    }
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

fn expression_temp_requested() -> &'static Mutex<bool> {
    static REQUESTED: OnceLock<Mutex<bool>> = OnceLock::new();
    REQUESTED.get_or_init(|| Mutex::new(false))
}

fn request_expression_temp() {
    *expression_temp_requested()
        .lock()
        .expect("score expression registry poisoned") = true;
}

/// Drain all internally managed score setup. Used by the exporter.
#[doc(hidden)]
pub fn drain_internal_score_setup() -> Vec<String> {
    let mut commands = drain_constant_setup();
    let mut requested = expression_temp_requested()
        .lock()
        .expect("score expression registry poisoned");
    if std::mem::take(&mut *requested) {
        commands.insert(
            0,
            format!("scoreboard objectives add {SCORE_EXPRESSION_TEMP_OBJECTIVE} dummy"),
        );
    }
    commands
}

/// Sand's compiler-managed temporary objective used by score expressions.
pub const SCORE_EXPRESSION_TEMP_OBJECTIVE: &str = "__sand_tmp";

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
pub struct ScoreVar<T = i32> {
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T> ScoreVar<T> {
    /// Create a new `ScoreVar` with the given objective name.
    ///
    /// Names longer than 16 characters are automatically hashed to a stable
    /// 16-character objective name (see [`ScoreVar::objective_name`]).
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

    /// Bind this variable to a selector to produce a condition builder.
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
        format!(
            "scoreboard players operation {} {} {} {} {}",
            left.selector,
            left.objective,
            op.as_str(),
            right.selector,
            right.objective
        )
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

    fn compare<O: Into<ScoreOperand>>(self, op: ScoreCompareOp, other: O) -> Condition {
        Condition::ScoreCompare {
            left: self.operand(),
            op,
            right: other.into(),
        }
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
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Eq(n),
        }
    }

    /// `unless score <sel> <obj> matches <n>` — not equal to `n`.
    pub fn ne(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Not(Box::new(Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Eq(n),
        }))
    }

    /// `if score <sel> <obj> matches <n+1>..` — strictly greater than `n`.
    pub fn gt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Gt(n),
        }
    }

    /// `if score <sel> <obj> matches <n>..` — greater than or equal to `n`.
    pub fn gte(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Gte(n),
        }
    }

    /// `if score <sel> <obj> matches ..<n-1>` — strictly less than `n`.
    pub fn lt(self, n: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Lt(n),
        }
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
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Lte(n),
        }
    }

    /// `if score <sel> <obj> matches <min>..<max>` — inside an inclusive range.
    pub fn between(self, min: i32, max: i32) -> Condition {
        let objective = self.obj();
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Between(Some(min), Some(max)),
        }
    }

    /// `unless score <sel> <obj> matches <min>..<max>` — outside an inclusive range.
    pub fn outside(self, min: i32, max: i32) -> Condition {
        Condition::Not(Box::new(self.between(min, max)))
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
        Condition::Score {
            selector: self.selector,
            objective,
            range: ScoreRange::Between(lo, hi),
        }
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

    pub fn plus<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Add, other)
    }
    pub fn minus<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Sub, other)
    }
    #[allow(clippy::should_implement_trait)]
    pub fn mul<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Mul, other)
    }
    #[allow(clippy::should_implement_trait)]
    pub fn div<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Div, other)
    }
    pub fn modulo<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Mod, other)
    }
    pub fn min<O: Into<ScoreOperand>>(self, other: O) -> Self {
        self.operation(ScoreOperation::Min, other)
    }
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

    pub fn eq(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Eq(n),
        })
    }
    pub fn gt(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Gt(n),
        })
    }
    pub fn gte(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Gte(n),
        })
    }
    pub fn lt(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Lt(n),
        })
    }
    pub fn lte(self, n: i32) -> Conditional {
        let temp = self.temp();
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Lte(n),
        })
    }
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
        self.lowered(Condition::Score {
            selector: temp.selector,
            objective: temp.objective,
            range: ScoreRange::Between(lo, hi),
        })
    }
    pub fn eq_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::ScoreCompare {
            left,
            op: ScoreCompareOp::Eq,
            right: other.into(),
        })
    }
    pub fn gt_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::ScoreCompare {
            left,
            op: ScoreCompareOp::Gt,
            right: other.into(),
        })
    }
    pub fn gte_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::ScoreCompare {
            left,
            op: ScoreCompareOp::Gte,
            right: other.into(),
        })
    }
    pub fn lt_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::ScoreCompare {
            left,
            op: ScoreCompareOp::Lt,
            right: other.into(),
        })
    }
    pub fn lte_score<O: Into<ScoreOperand>>(self, other: O) -> Conditional {
        let left = self.temp();
        self.lowered(Condition::ScoreCompare {
            left,
            op: ScoreCompareOp::Lte,
            right: other.into(),
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;

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
    fn condition_gte() {
        let cond = MANA.of("@s").gte(25);
        match cond {
            Condition::Score {
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
        match cond {
            Condition::Score {
                range: ScoreRange::Lte(100),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_ne_wraps_not() {
        let cond = MANA.of("@s").ne(0);
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn condition_matches_range() {
        let cond = MANA.of("@s").matches(1..=100);
        match cond {
            Condition::Score {
                range: ScoreRange::Between(Some(1), Some(100)),
                ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn condition_between() {
        let cond = MANA.of("@s").between(10, 100);
        match cond {
            Condition::Score {
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
            cond,
            Condition::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn is_nonzero_condition() {
        let cond = MANA.is_nonzero("@s");
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn positive_condition() {
        let cond = MANA.positive("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gt(0),
                ..
            }
        ));
    }

    #[test]
    fn negative_condition() {
        let cond = MANA.negative("@s");
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Lt(0),
                ..
            }
        ));
    }

    #[test]
    fn scoreref_is_zero() {
        let cond = MANA.of("@s").is_zero();
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Eq(0),
                ..
            }
        ));
    }

    #[test]
    fn scoreref_positive() {
        let cond = MANA.of("@s").positive();
        assert!(matches!(
            cond,
            Condition::Score {
                range: ScoreRange::Gt(0),
                ..
            }
        ));
    }

    #[test]
    fn condition_outside_wraps_not() {
        let cond = MANA.of("@s").outside(10, 100);
        assert!(matches!(cond, Condition::Not(_)));
    }

    #[test]
    fn condition_matches_open_end() {
        let cond = MANA.of("@s").matches(25..);
        match cond {
            Condition::Score {
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
}
