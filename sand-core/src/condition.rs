//! Typed condition DSL for `execute if/unless` generation.
//!
//! Conditions can be composed with [`Condition::all`], [`Condition::any`], and
//! [`Condition::not`] without writing any raw execute syntax.
//!
//! Nested `Any` inside `All` is correctly lowered into multiple execute commands
//! via [`Condition::to_execute_plans`] — see that method for the full semantics.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::condition::Condition;
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//! static CASTING: Flag = Flag::new("casting");
//! static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));
//!
//! let cond = Condition::all([
//!     MANA.of("@s").gte(25),
//!     DASH.ready("@s"),
//!     CASTING.of("@s").is_false(),
//! ]);
//! ```

// ── ScoreRange ────────────────────────────────────────────────────────────────

/// A range used in `execute if score … matches <range>`.
///
/// `None` on either end of `Between` means the range is open on that side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreRange {
    /// `matches <n>` — exactly equal.
    Eq(i32),
    /// `matches <n+1>..` — strictly greater than n.
    Gt(i32),
    /// `matches <n>..` — greater than or equal.
    Gte(i32),
    /// `matches ..<n-1>` — strictly less than n.
    Lt(i32),
    /// `matches ..<n>` — less than or equal.
    Lte(i32),
    /// `matches [lo]..[hi]` — inclusive range (either bound may be `None` = open).
    Between(Option<i32>, Option<i32>),
}

/// One scoreboard entry used by score-to-score comparisons.
///
/// This is public so typed score operands can be reused by the expression API,
/// but callers normally obtain one from [`ScoreRef`](crate::state::ScoreRef).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreOperand {
    /// The score holder or entity selector.
    pub(crate) selector: String,
    /// The (already Minecraft-safe) objective name.
    pub(crate) objective: String,
}

/// Vanilla operators accepted by `execute if score <left> <op> <right>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreCompareOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl ScoreCompareOp {
    /// Render the vanilla scoreboard comparison operator.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
        }
    }
}

impl ScoreRange {
    /// Render the range to a Minecraft matches string fragment.
    pub fn render(&self) -> String {
        match self {
            ScoreRange::Eq(n) => n.to_string(),
            ScoreRange::Gt(n) => format!("{}..", n + 1),
            ScoreRange::Gte(n) => format!("{n}.."),
            ScoreRange::Lt(n) => format!("..{}", n - 1),
            ScoreRange::Lte(n) => format!("..{n}"),
            ScoreRange::Between(lo, hi) => {
                let lo_s = lo.map(|n| n.to_string()).unwrap_or_default();
                let hi_s = hi.map(|n| n.to_string()).unwrap_or_default();
                format!("{lo_s}..{hi_s}")
            }
        }
    }
}

// ── Condition ─────────────────────────────────────────────────────────────────

/// A typed datapack condition, suitable for use in `execute if/unless`.
///
/// Produce conditions from [`ScoreVar::of`](crate::state::ScoreVar::of),
/// [`Flag::of`](crate::state::Flag::of), or the static constructors below.
///
/// Use [`when`](crate::execute_when::when) / [`unless`](crate::execute_when::unless)
/// to turn a `Condition` into complete execute commands.
///
/// Nested `Any` inside `All` is automatically distributed into multiple execute
/// commands by [`execute_commands`](Condition::execute_commands).
#[derive(Debug, Clone)]
pub enum Condition {
    /// `if score <selector> <objective> matches <range>`
    Score {
        selector: String,
        objective: String,
        range: ScoreRange,
    },
    /// `if score <left selector> <left objective> <op> <right selector> <right objective>`.
    ScoreCompare {
        left: ScoreOperand,
        op: ScoreCompareOp,
        right: ScoreOperand,
    },
    /// `if score <selector> <flag_objective> matches 1` (or `0` when `value = false`)
    Flag {
        selector: String,
        objective: String,
        value: bool,
    },
    /// `if predicate <namespace:path>`
    Predicate(String),
    /// `if entity <selector>`
    Entity(String),
    /// `if data storage <location> <path>`
    StorageExists { location: String, path: String },
    /// Invert this condition (flips `if` ↔ `unless`).
    Not(Box<Condition>),
    /// All sub-conditions must hold (chained `if … if …`).
    All(Vec<Condition>),
    /// At least one sub-condition must hold (generates one execute per sub-condition).
    Any(Vec<Condition>),
    /// Explicit raw escape hatch: `if/unless <fragment>` verbatim.
    ///
    /// The fragment must be a valid Minecraft `execute if`/`unless` sub-command
    /// *without* the leading `if`/`unless` keyword, e.g. `"score @s sync_jumps < @s jumps"`
    /// or `"predicate my_pack:some_predicate"`.
    ///
    /// This is an intentionally explicit escape hatch — there is no `From<&str>`/
    /// `From<String>` impl for `Condition`, so raw fragments never enter a typed
    /// condition chain silently. Prefer the typed constructors above; reach for
    /// `Condition::raw` only when no typed equivalent exists yet.
    Raw(String),
}

impl Condition {
    /// Invert a condition.
    ///
    /// Also available as the `!` operator via [`std::ops::Not`].
    pub fn negate(cond: Condition) -> Self {
        Condition::Not(Box::new(cond))
    }

    /// All of the given conditions must hold.
    pub fn all(conds: impl IntoIterator<Item = Condition>) -> Self {
        Condition::All(conds.into_iter().collect())
    }

    /// Any of the given conditions must hold.
    pub fn any(conds: impl IntoIterator<Item = Condition>) -> Self {
        Condition::Any(conds.into_iter().collect())
    }

    /// Condition on a named predicate resource.
    ///
    /// ```rust,ignore
    /// let c = Condition::predicate("my_pack:can_cast");
    /// ```
    pub fn predicate(location: impl Into<String>) -> Self {
        Condition::Predicate(location.into())
    }

    /// Condition on an entity selector.
    ///
    /// ```rust,ignore
    /// let c = Condition::entity("@s[tag=ready]");
    /// ```
    pub fn entity(selector: impl Into<String>) -> Self {
        Condition::Entity(selector.into())
    }

    /// Condition on a named storage path existing.
    ///
    /// ```rust,ignore
    /// let c = Condition::storage_exists("example:state", "player.mana");
    /// ```
    pub fn storage_exists(location: impl Into<String>, path: impl Into<String>) -> Self {
        Condition::StorageExists {
            location: location.into(),
            path: path.into(),
        }
    }

    /// Explicit raw `execute if/unless` fragment escape hatch.
    ///
    /// The fragment is used verbatim after the `if`/`unless` keyword — it is
    /// **not** validated beyond being a non-empty string. Use this only when
    /// no typed condition constructor covers your case.
    ///
    /// ```rust,ignore
    /// let c = Condition::raw("score @s sync_jumps < @s jumps");
    /// ```
    pub fn raw(fragment: impl Into<String>) -> Self {
        Condition::Raw(fragment.into())
    }
}

impl std::ops::Not for Condition {
    type Output = Condition;
    fn not(self) -> Self::Output {
        Condition::Not(Box::new(self))
    }
}

// ── Ergonomic chaining ────────────────────────────────────────────────────────

impl Condition {
    /// Both `self` and `other` must hold (`All`).
    ///
    /// Flattens adjacent `All` chains so `a.and(b).and(c)` produces a single
    /// `All([a, b, c])` rather than nested `All([All([a, b]), c])`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25).and(DASH.ready("@s"));
    /// ```
    pub fn and(self, other: Condition) -> Condition {
        match self {
            Condition::All(mut conds) => {
                conds.push(other);
                Condition::All(conds)
            }
            _ => Condition::All(vec![self, other]),
        }
    }

    /// Either `self` or `other` must hold (`Any`).
    ///
    /// Flattens adjacent `Any` chains so `a.or(b).or(c)` produces a single
    /// `Any([a, b, c])`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(100).or(SHIELD.of("@s").is_true());
    /// ```
    pub fn or(self, other: Condition) -> Condition {
        match self {
            Condition::Any(mut conds) => {
                conds.push(other);
                Condition::Any(conds)
            }
            _ => Condition::Any(vec![self, other]),
        }
    }

    /// `self` must hold and `other` must **not** hold.
    ///
    /// Equivalent to `self.and(!other)`.
    ///
    /// ```rust,ignore
    /// let cond = MANA.of("@s").gte(25).and_not(CASTING.of("@s").is_true());
    /// ```
    pub fn and_not(self, other: Condition) -> Condition {
        self.and(!other)
    }

    /// Either `self` holds or `other` must **not** hold.
    ///
    /// Equivalent to `self.or(!other)`.
    pub fn or_not(self, other: Condition) -> Condition {
        self.or(!other)
    }
}

// ── Execute plan lowering ─────────────────────────────────────────────────────

/// A single execute plan — a sequence of `if/unless …` clause strings to chain.
///
/// Multiple plans from the same condition are OR-alternatives (at least one must
/// succeed for the overall condition to hold).
pub type ExecutePlan = Vec<String>;

impl Condition {
    /// Expand this condition into a list of [`ExecutePlan`]s.
    ///
    /// Each plan is a list of `if/unless …` clause strings to chain in a single
    /// `execute … run <cmd>` command. Multiple plans are OR-alternatives — the
    /// command is emitted once per plan.
    ///
    /// | Condition | negated=false | negated=true |
    /// |---|---|---|
    /// | Leaf | `[[if clause]]` | `[[unless clause]]` |
    /// | `Not(c)` | `c.to_execute_plans(true)` | `c.to_execute_plans(false)` |
    /// | `All(cs)` | Cartesian product of children | Union of negated children |
    /// | `Any(cs)` | Union of children | Cartesian product of negated children |
    ///
    /// The Cartesian product of `[[a], [b]]` and `[[c], [d]]` is
    /// `[[a, c], [a, d], [b, c], [b, d]]`.
    pub fn to_execute_plans(&self, negated: bool) -> Vec<ExecutePlan> {
        match self {
            // Leaf nodes → single plan with one clause
            Condition::Score {
                selector,
                objective,
                range,
            } => {
                let kw = if_kw(negated);
                vec![vec![format!(
                    "{kw} score {selector} {objective} matches {}",
                    range.render()
                )]]
            }
            Condition::ScoreCompare { left, op, right } => {
                let kw = if_kw(negated);
                vec![vec![format!(
                    "{kw} score {} {} {} {} {}",
                    left.selector,
                    left.objective,
                    op.as_str(),
                    right.selector,
                    right.objective
                )]]
            }
            Condition::Flag {
                selector,
                objective,
                value,
            } => {
                let kw = if_kw(negated);
                let expected = if *value { "1" } else { "0" };
                vec![vec![format!(
                    "{kw} score {selector} {objective} matches {expected}"
                )]]
            }
            Condition::Predicate(loc) => {
                let kw = if_kw(negated);
                vec![vec![format!("{kw} predicate {loc}")]]
            }
            Condition::Entity(sel) => {
                let kw = if_kw(negated);
                vec![vec![format!("{kw} entity {sel}")]]
            }
            Condition::StorageExists { location, path } => {
                let kw = if_kw(negated);
                vec![vec![format!("{kw} data storage {location} {path}")]]
            }
            Condition::Raw(fragment) => {
                let kw = if_kw(negated);
                vec![vec![format!("{kw} {fragment}")]]
            }

            // Not: flip the negated flag and delegate
            Condition::Not(inner) => inner.to_execute_plans(!negated),

            // All(cs) negated=false → AND  → Cartesian product of each child's plans
            // All(cs) negated=true  → NOT(AND) = OR of NOTs → union of negated children
            Condition::All(conds) => {
                if negated {
                    // NOT(a AND b) = NOT a OR NOT b
                    conds
                        .iter()
                        .flat_map(|c| c.to_execute_plans(true))
                        .collect()
                } else {
                    let sub_plan_sets: Vec<Vec<ExecutePlan>> =
                        conds.iter().map(|c| c.to_execute_plans(false)).collect();
                    cartesian_product_plans(sub_plan_sets)
                }
            }

            // Any(cs) negated=false → OR  → union of children's plans
            // Any(cs) negated=true  → NOT(OR) = AND of NOTs → Cartesian product of negated children
            Condition::Any(conds) => {
                if negated {
                    // NOT(a OR b) = NOT a AND NOT b
                    let sub_plan_sets: Vec<Vec<ExecutePlan>> =
                        conds.iter().map(|c| c.to_execute_plans(true)).collect();
                    cartesian_product_plans(sub_plan_sets)
                } else {
                    conds
                        .iter()
                        .flat_map(|c| c.to_execute_plans(false))
                        .collect()
                }
            }
        }
    }

    /// Build complete `execute … run <cmd>` command strings for this condition.
    ///
    /// Uses [`to_execute_plans`](Condition::to_execute_plans) internally, so
    /// nested `Any` inside `All` correctly expands into multiple commands.
    ///
    /// - Simple conditions and `All`: typically one command.
    /// - `Any`: one command per sub-condition.
    /// - `Not(Any)`: one command with de Morgan–applied `unless` clauses.
    /// - `All([a, Any([b, c])])`: two commands.
    pub fn execute_commands(&self, negated: bool, run: &str) -> Vec<String> {
        self.to_execute_plans(negated)
            .into_iter()
            .map(|clauses| {
                if clauses.is_empty() {
                    run.to_string()
                } else {
                    format!("execute {} run {run}", clauses.join(" "))
                }
            })
            .collect()
    }

    /// Render this condition as a flat list of `if/unless …` clause strings.
    ///
    /// This is a best-effort rendering for simple conditions. For `Any` nested
    /// inside `All`, use [`execute_commands`](Condition::execute_commands) or
    /// [`to_execute_plans`](Condition::to_execute_plans) instead, which correctly
    /// expand OR conditions into multiple commands.
    ///
    /// `negated = true` flips `if` → `unless` (and vice-versa).
    pub fn render_clauses(&self, negated: bool) -> Vec<String> {
        // Delegate to to_execute_plans and flatten — this loses OR structure for
        // nested Any, but is preserved for simple use-cases and backwards compat.
        self.to_execute_plans(negated)
            .into_iter()
            .flatten()
            .collect()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn if_kw(negated: bool) -> &'static str {
    if negated { "unless" } else { "if" }
}

/// Compute the Cartesian product of multiple plan sets.
///
/// Given `[[plan_a1, plan_a2], [plan_b1]]` produces every combination:
/// `[plan_a1 + plan_b1, plan_a2 + plan_b1]`.
fn cartesian_product_plans(plan_sets: Vec<Vec<ExecutePlan>>) -> Vec<ExecutePlan> {
    if plan_sets.is_empty() {
        return vec![vec![]];
    }
    let mut result: Vec<ExecutePlan> = vec![vec![]];
    for plan_set in plan_sets {
        let mut new_result = Vec::new();
        for existing in &result {
            for plan in &plan_set {
                let mut combined = existing.clone();
                combined.extend_from_slice(plan);
                new_result.push(combined);
            }
        }
        result = new_result;
    }
    result
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn score(sel: &str, obj: &str, range: ScoreRange) -> Condition {
        Condition::Score {
            selector: sel.to_string(),
            objective: obj.to_string(),
            range,
        }
    }

    fn flag(sel: &str, obj: &str, value: bool) -> Condition {
        Condition::Flag {
            selector: sel.to_string(),
            objective: obj.to_string(),
            value,
        }
    }

    // ── ScoreRange rendering ──────────────────────────────────────────────────

    #[test]
    fn range_eq() {
        assert_eq!(ScoreRange::Eq(5).render(), "5");
    }

    #[test]
    fn range_gte() {
        assert_eq!(ScoreRange::Gte(25).render(), "25..");
    }

    #[test]
    fn range_lte() {
        assert_eq!(ScoreRange::Lte(100).render(), "..100");
    }

    #[test]
    fn range_gt() {
        assert_eq!(ScoreRange::Gt(10).render(), "11..");
    }

    #[test]
    fn range_lt() {
        assert_eq!(ScoreRange::Lt(10).render(), "..9");
    }

    #[test]
    fn range_between() {
        assert_eq!(ScoreRange::Between(Some(1), Some(100)).render(), "1..100");
    }

    #[test]
    fn range_between_open_end() {
        assert_eq!(ScoreRange::Between(Some(25), None).render(), "25..");
    }

    #[test]
    fn range_between_open_start() {
        assert_eq!(ScoreRange::Between(None, Some(100)).render(), "..100");
    }

    // ── Leaf execute_plans ────────────────────────────────────────────────────

    #[test]
    fn score_plan_if() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let plans = c.to_execute_plans(false);
        assert_eq!(plans, vec![vec!["if score @s mana matches 25.."]]);
    }

    #[test]
    fn score_plan_unless() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let plans = c.to_execute_plans(true);
        assert_eq!(plans, vec![vec!["unless score @s mana matches 25.."]]);
    }

    #[test]
    fn storage_exists_plan() {
        let c = Condition::storage_exists("ex:state", "mana");
        let plans = c.to_execute_plans(false);
        assert_eq!(plans, vec![vec!["if data storage ex:state mana"]]);
        let plans_neg = c.to_execute_plans(true);
        assert_eq!(plans_neg, vec![vec!["unless data storage ex:state mana"]]);
    }

    // ── Condition rendering (backwards compat) ────────────────────────────────

    #[test]
    fn score_if_clause() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s mana matches 25.."]);
    }

    #[test]
    fn score_unless_clause() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let clauses = c.render_clauses(true);
        assert_eq!(clauses, vec!["unless score @s mana matches 25.."]);
    }

    #[test]
    fn flag_true_clause() {
        let c = flag("@s", "casting", true);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s casting matches 1"]);
    }

    #[test]
    fn flag_false_clause() {
        let c = flag("@s", "casting", false);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s casting matches 0"]);
    }

    #[test]
    fn predicate_clause() {
        let c = Condition::predicate("my_pack:can_cast");
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if predicate my_pack:can_cast"]);
    }

    #[test]
    fn entity_clause() {
        let c = Condition::entity("@s[tag=ready]");
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if entity @s[tag=ready]"]);
    }

    #[test]
    fn not_flips_keyword() {
        let c = !(score("@s", "mana", ScoreRange::Gte(25)));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["unless score @s mana matches 25.."]);
    }

    #[test]
    fn not_not_cancels() {
        let c = !(!(score("@s", "mana", ScoreRange::Eq(10))));
        let clauses = c.render_clauses(false);
        assert_eq!(clauses, vec!["if score @s mana matches 10"]);
    }

    #[test]
    fn all_chains_clauses() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            flag("@s", "casting", false),
        ]);
        let clauses = c.render_clauses(false);
        assert_eq!(clauses.len(), 2);
        assert_eq!(clauses[0], "if score @s mana matches 25..");
        assert_eq!(clauses[1], "if score @s casting matches 0");
    }

    // ── execute_commands ──────────────────────────────────────────────────────

    #[test]
    fn execute_score() {
        let c = score("@s", "mana", ScoreRange::Gte(25));
        let cmds = c.execute_commands(false, "say enough mana");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. run say enough mana"]
        );
    }

    #[test]
    fn execute_unless() {
        let c = flag("@s", "casting", true);
        let cmds = c.execute_commands(true, "say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn execute_any_generates_multiple() {
        let c = Condition::any([
            score("@s", "mana", ScoreRange::Gte(25)),
            score("@s", "rage", ScoreRange::Gte(10)),
        ]);
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("mana"), "got: {}", cmds[0]);
        assert!(cmds[1].contains("rage"), "got: {}", cmds[1]);
    }

    #[test]
    fn execute_not_any_de_morgan() {
        // NOT(a OR b) = (NOT a) AND (NOT b) — one chained command
        let c = !(Condition::any([flag("@s", "a", true), flag("@s", "b", true)]));
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(
            cmds.len(),
            1,
            "de Morgan should produce one chained command"
        );
        assert!(cmds[0].contains("unless score @s a"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("unless score @s b"), "got: {}", cmds[0]);
    }

    #[test]
    fn execute_all() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            flag("@s", "casting", false),
            Condition::predicate("my_pack:can_cast"),
        ]);
        let cmds = c.execute_commands(false, "say cast");
        assert_eq!(cmds.len(), 1);
        let cmd = &cmds[0];
        assert!(cmd.contains("if score @s mana matches 25.."), "got: {cmd}");
        assert!(cmd.contains("if score @s casting matches 0"), "got: {cmd}");
        assert!(cmd.contains("if predicate my_pack:can_cast"), "got: {cmd}");
    }

    // ── Nested Any lowering (the key bug fix) ────────────────────────────────

    #[test]
    fn nested_any_inside_all_expands() {
        // all![a, any![b, c]] → 2 commands:
        //   execute if a if b run cmd
        //   execute if a if c run cmd
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let c = flag("@s", "sprinting", true);
        let cond = Condition::all([a, Condition::any([b, c])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "nested Any should expand: got {cmds:?}");
        assert!(
            cmds[0].contains("if score @s mana") && cmds[0].contains("if score @s casting"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[1].contains("if score @s mana") && cmds[1].contains("if score @s sprinting"),
            "got: {}",
            cmds[1]
        );
    }

    #[test]
    fn any_inside_any_flattens() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = score("@s", "rage", ScoreRange::Gte(10));
        let c = score("@s", "ki", ScoreRange::Gte(5));
        let cond = Condition::any([a, Condition::any([b, c])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 3, "Any(Any) should produce 3 commands");
    }

    #[test]
    fn not_all_de_morgan() {
        // NOT(a AND b) = NOT a OR NOT b → 2 separate commands
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", true);
        let cond = !(Condition::all([a, b]));
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "NOT(AND) should give 2 plans");
        assert!(cmds[0].contains("unless"), "got: {}", cmds[0]);
        assert!(cmds[1].contains("unless"), "got: {}", cmds[1]);
    }

    #[test]
    fn all_of_any_cross_product() {
        // all![any![a, b], any![c, d]] → 4 commands
        let a = score("@s", "a", ScoreRange::Eq(1));
        let b = score("@s", "b", ScoreRange::Eq(2));
        let c = score("@s", "c", ScoreRange::Eq(3));
        let d = score("@s", "d", ScoreRange::Eq(4));
        let cond = Condition::all([Condition::any([a, b]), Condition::any([c, d])]);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(
            cmds.len(),
            4,
            "cross product of two Any(2) should be 4: got {cmds:?}"
        );
    }

    #[test]
    fn raw_condition_if() {
        let c = Condition::raw("score @s sync_jumps < @s jumps");
        let plans = c.to_execute_plans(false);
        assert_eq!(plans, vec![vec!["if score @s sync_jumps < @s jumps"]]);
    }

    #[test]
    fn raw_condition_unless() {
        let c = Condition::raw("entity @s[tag=busy]");
        let plans = c.to_execute_plans(true);
        assert_eq!(plans, vec![vec!["unless entity @s[tag=busy]"]]);
    }

    #[test]
    fn raw_condition_composes_with_typed() {
        let c = Condition::all([
            score("@s", "mana", ScoreRange::Gte(25)),
            Condition::raw("score @s sync_jumps < @s jumps"),
        ]);
        let cmds = c.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(
            cmds[0].contains("if score @s sync_jumps < @s jumps"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn storage_exists_execute() {
        let c = Condition::storage_exists("ex:state", "mana");
        let cmds = c.execute_commands(false, "say has mana");
        assert_eq!(
            cmds,
            vec!["execute if data storage ex:state mana run say has mana"]
        );
    }

    // ── Condition chaining ────────────────────────────────────────────────────

    #[test]
    fn and_produces_all() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let cond = a.and(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s casting"), "got: {}", cmds[0]);
    }

    #[test]
    fn and_flattens_chain() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", false);
        let c = flag("@s", "sprinting", true);
        // a.and(b).and(c) should be a flat All([a, b, c]), not All([All([a,b]), c])
        let cond = a.and(b).and(c);
        match &cond {
            Condition::All(v) => assert_eq!(v.len(), 3, "expected flat All([a,b,c])"),
            other => panic!("expected All, got {other:?}"),
        }
    }

    #[test]
    fn or_produces_any() {
        let a = score("@s", "mana", ScoreRange::Gte(100));
        let b = flag("@s", "shield", true);
        let cond = a.or(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "Any should expand to 2 commands");
    }

    #[test]
    fn or_flattens_chain() {
        let a = score("@s", "mana", ScoreRange::Gte(100));
        let b = score("@s", "rage", ScoreRange::Gte(50));
        let c = score("@s", "ki", ScoreRange::Gte(10));
        let cond = a.or(b).or(c);
        match &cond {
            Condition::Any(v) => assert_eq!(v.len(), 3, "expected flat Any([a,b,c])"),
            other => panic!("expected Any, got {other:?}"),
        }
    }

    #[test]
    fn and_not_negates_rhs() {
        let a = score("@s", "mana", ScoreRange::Gte(25));
        let b = flag("@s", "casting", true);
        let cond = a.and_not(b);
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(
            cmds[0].contains("unless score @s casting"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn chained_and_with_nested_or() {
        let mana = score("@s", "mana", ScoreRange::Gte(25));
        let dash = flag("@s", "dash", false);
        let shield = flag("@s", "shield", true);
        // mana AND (dash OR shield) → 2 commands
        let cond = mana.and(dash.or(shield));
        let cmds = cond.execute_commands(false, "say ok");
        assert_eq!(cmds.len(), 2, "AND with nested OR: {cmds:?}");
        assert!(
            cmds.iter().all(|c| c.contains("if score @s mana")),
            "both commands include mana: {cmds:?}"
        );
    }

    #[test]
    fn event_guard_chaining_pattern() {
        static MANA2: crate::state::ScoreVar<i32> = crate::state::ScoreVar::new("mana");
        static DASH2: crate::state::Cooldown =
            crate::state::Cooldown::new("dash", crate::state::Ticks::new(60));
        static CASTING2: crate::state::Flag = crate::state::Flag::new("casting");
        let guard = MANA2
            .of("@s")
            .gte(25)
            .and(DASH2.ready("@s"))
            .and_not(CASTING2.of("@s").is_true());
        let cmds = guard.execute_commands(false, "function ns:handler");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s dash"), "got: {}", cmds[0]);
        assert!(
            cmds[0].contains("unless score @s casting"),
            "got: {}",
            cmds[0]
        );
    }
}
