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
}

impl std::ops::Not for Condition {
    type Output = Condition;
    fn not(self) -> Self::Output {
        Condition::Not(Box::new(self))
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
    fn storage_exists_execute() {
        let c = Condition::storage_exists("ex:state", "mana");
        let cmds = c.execute_commands(false, "say has mana");
        assert_eq!(
            cmds,
            vec!["execute if data storage ex:state mana run say has mana"]
        );
    }
}
