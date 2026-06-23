//! Typed `when` / `unless` execute integration.
//!
//! Wraps a [`Condition`] and commands into complete `execute if/unless … run …`
//! command strings without any raw execute syntax.
//!
//! # Branch semantics
//!
//! ## Single-command branch — `.then_one(cmd)` or `.then(cmd)` alone
//!
//! Emits one `execute if/unless … run <cmd>` line directly in the parent function.
//!
//! ```rust,ignore
//! when(MANA.of("@s").gte(25)).then_one("say enough mana");
//! // → execute if score @s mana matches 25.. run say enough mana
//! ```
//!
//! ## Grouped branch — `.then_all([...])` or `.and_then(…).then(…)`
//!
//! Collects all commands into an anonymous helper function and emits a single
//! parent `execute if/unless … run function <branch>`. The condition is evaluated
//! **once**; all commands run in order under that one check. Later commands in
//! the branch are not re-tested against the condition, so mutating the condition
//! inside the branch does not prevent later branch commands from running.
//!
//! ```rust,ignore
//! when(HAS_CELLS.of("@s").is_true()).then_all([
//!     tellraw(Selector::self_(), Text::new("Already granted")),
//!     cmd::return_fail(),
//! ]);
//! // → execute if score @s has_cells matches 1 run function <ns>:sand/branches/0
//! //
//! // Branch function sand/branches/0:
//! //   tellraw @s {"text":"Already granted"}
//! //   return fail
//! ```
//!
//! ## Per-command wrapping — `.then_each([...])`
//!
//! Wraps **each** command in the condition separately (old behavior, explicit opt-in).
//! Use only when you intentionally want each command re-tested.
//!
//! ```rust,ignore
//! when(MANA.of("@s").gte(25)).then_each(["say a", "say b"]);
//! // → execute if score @s mana matches 25.. run say a
//! //   execute if score @s mana matches 25.. run say b
//! ```
//!
//! # If/else — `if_(cond).then_all([...]).else_all([...])`
//!
//! Generates two named branch functions and emits two parent execute lines:
//!
//! ```rust,ignore
//! if_(HAS_CELLS.of("@s").is_true())
//!     .then_all([tellraw(...), cmd::return_fail()])
//!     .else_all([attribute_base_set(...), HAS_CELLS.enable("@s")]);
//! // → execute if   score @s has_cells matches 1 run function <ns>:sand/branches/0
//! //   execute unless score @s has_cells matches 1 run function <ns>:sand/branches/1
//! ```
//!
//! # Example
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::condition::Condition;
//! use sand_core::execute_when::{when, unless, if_};
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//! static CASTING: Flag = Flag::new("casting");
//!
//! // Single condition
//! let cmds = when(MANA.of("@s").gte(25)).then_one("say enough mana");
//!
//! // Inverted condition
//! let cmds = unless(CASTING.of("@s").is_true()).then_one("say not casting");
//!
//! // Grouped branch (safe when branch mutates the condition)
//! let cmds = unless(CASTING.of("@s").is_true()).then_all([
//!     "say starting cast".to_string(),
//!     CASTING.enable("@s"),
//! ]);
//! ```

use crate::condition::Condition;

/// Reset the branch counter. For use in unit tests only — keeps paths stable.
#[doc(hidden)]
pub fn reset_branch_counter_for_tests() {
    crate::drain_dyn_fns();
}

/// Register commands as an anonymous branch function and return its path.
///
/// Uses `__sand_local:` sentinel so the namespace is resolved at export time.
fn register_branch(commands: Vec<String>) -> String {
    let path = crate::register_dyn_fn_dedup("sand/branches", commands);
    format!("__sand_local:{path}")
}

// ── WhenBuilder ───────────────────────────────────────────────────────────────

/// Builder returned by [`when`]. Call [`then_one`](WhenBuilder::then_one),
/// [`then_all`](WhenBuilder::then_all), or build up with [`and_then`](WhenBuilder::and_then).
pub struct WhenBuilder {
    cond: Condition,
    /// Commands accumulated via `.and_then(...)` — when non-empty, `.then()` creates a branch.
    staged: Vec<String>,
}

impl WhenBuilder {
    /// Accumulate a command to run if the condition holds.
    ///
    /// Calling `.then(cmd)` afterwards creates a **grouped branch function** that
    /// runs all accumulated commands in order under the condition once.
    ///
    /// ```rust,ignore
    /// let cmds = when(MANA.of("@s").gte(25))
    ///     .and_then("say first")
    ///     .and_then("say second")
    ///     .then("say third");
    /// // → one execute line that calls a branch function containing all 3 commands
    /// ```
    pub fn and_then(mut self, cmd: impl std::fmt::Display) -> WhenBuilder {
        self.staged.push(cmd.to_string());
        self
    }

    /// Finish the chain.
    ///
    /// - With no prior `.and_then(...)`: emits a single `execute if … run <cmd>` line.
    /// - With prior `.and_then(...)` calls: creates a grouped branch function.
    pub fn then(self, cmd: impl std::fmt::Display) -> Vec<String> {
        let mut all_cmds = self.staged;
        all_cmds.push(cmd.to_string());
        if all_cmds.len() == 1 {
            self.cond.execute_commands(false, &all_cmds[0])
        } else {
            let branch_ref = register_branch(all_cmds);
            self.cond
                .execute_commands(false, &format!("function {branch_ref}"))
        }
    }

    /// Always emit a single `execute if … run <cmd>` line (no branch function).
    ///
    /// Use when you want one command wrapped in the condition, with no grouping.
    pub fn then_one(self, cmd: impl std::fmt::Display) -> Vec<String> {
        self.cond.execute_commands(false, &cmd.to_string())
    }

    /// Collect all commands into a branch function, always (even for one command).
    ///
    /// The branch function is called once under the condition. All commands run
    /// in order, regardless of whether they mutate the condition.
    ///
    /// Accepts any value implementing [`Display`](std::fmt::Display) — use raw strings,
    /// [`cmd`](crate::cmd) builders, or any other display-able command type.
    ///
    /// ```rust,ignore
    /// when(HAS_CELLS.of("@s").is_true()).then_all([
    ///     cmd::tellraw(Selector::self_(), Text::new("Already granted")),
    ///     cmd::return_fail(),
    /// ]);
    /// ```
    pub fn then_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        let commands: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        let branch_ref = register_branch(commands);
        self.cond
            .execute_commands(false, &format!("function {branch_ref}"))
    }

    /// Wrap **each** command in the condition separately (old per-command behavior).
    ///
    /// Each command is independently `execute if … run <cmd>`. If a command mutates
    /// the condition, later commands may not run. Prefer [`then_all`](WhenBuilder::then_all)
    /// for most multi-command branches.
    pub fn then_each(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        cmds.into_iter()
            .flat_map(|cmd| self.cond.execute_commands(false, &cmd.to_string()))
            .collect()
    }
}

// ── UnlessBuilder ─────────────────────────────────────────────────────────────

/// Builder returned by [`unless`]. Call [`then_one`](UnlessBuilder::then_one),
/// [`then_all`](UnlessBuilder::then_all), or build up with [`and_then`](UnlessBuilder::and_then).
pub struct UnlessBuilder {
    cond: Condition,
    /// Commands accumulated via `.and_then(...)`.
    staged: Vec<String>,
}

impl UnlessBuilder {
    /// Accumulate a command to run unless the condition holds.
    ///
    /// Calling `.then(cmd)` afterwards creates a **grouped branch function**.
    pub fn and_then(mut self, cmd: impl std::fmt::Display) -> UnlessBuilder {
        self.staged.push(cmd.to_string());
        self
    }

    /// Finish the chain.
    ///
    /// - With no prior `.and_then(...)`: emits a single `execute unless … run <cmd>` line.
    /// - With prior `.and_then(...)` calls: creates a grouped branch function.
    pub fn then(self, cmd: impl std::fmt::Display) -> Vec<String> {
        let mut all_cmds = self.staged;
        all_cmds.push(cmd.to_string());
        if all_cmds.len() == 1 {
            self.cond.execute_commands(true, &all_cmds[0])
        } else {
            let branch_ref = register_branch(all_cmds);
            self.cond
                .execute_commands(true, &format!("function {branch_ref}"))
        }
    }

    /// Always emit a single `execute unless … run <cmd>` line (no branch function).
    pub fn then_one(self, cmd: impl std::fmt::Display) -> Vec<String> {
        self.cond.execute_commands(true, &cmd.to_string())
    }

    /// Collect all commands into a branch function called once under `unless`.
    ///
    /// ```rust,ignore
    /// unless(HAS_CELLS.of("@s").is_true()).then_all([
    ///     cmd::attribute_base_set(Selector::self_(), AttributeType::MaxHealth.as_str(), 40.0),
    ///     HAS_CELLS.enable("@s"),
    ///     cmd::return_cmd(0),
    /// ]);
    /// ```
    pub fn then_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        let commands: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        let branch_ref = register_branch(commands);
        self.cond
            .execute_commands(true, &format!("function {branch_ref}"))
    }

    /// Wrap **each** command in the condition separately (old per-command behavior).
    pub fn then_each(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        cmds.into_iter()
            .flat_map(|cmd| self.cond.execute_commands(true, &cmd.to_string()))
            .collect()
    }
}

// ── IfBuilder / IfThenBuilder (if/else) ──────────────────────────────────────

/// Builder returned by [`if_`]. Supplies a `then_all` arm.
pub struct IfBuilder {
    cond: Condition,
}

impl IfBuilder {
    /// Specify the commands to run when the condition holds.
    ///
    /// Returns an [`IfThenBuilder`] where you can optionally attach an `.else_all(...)`.
    /// Accepts any value implementing [`Display`](std::fmt::Display).
    pub fn then_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> IfThenBuilder {
        let then_cmds: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        IfThenBuilder {
            cond: self.cond,
            then_cmds,
        }
    }
}

/// Returned by [`IfBuilder::then_all`]. Finishes with `.else_all(...)` or used alone.
pub struct IfThenBuilder {
    cond: Condition,
    then_cmds: Vec<String>,
}

impl IfThenBuilder {
    /// Attach an else arm — commands to run when the condition does **not** hold.
    ///
    /// Generates two branch functions and two parent execute lines (if + unless).
    ///
    /// ```rust,ignore
    /// if_(HAS_CELLS.of("@s").is_true())
    ///     .then_all([tellraw(...), cmd::return_fail()])
    ///     .else_all([attribute_base_set(...), HAS_CELLS.enable("@s")]);
    /// ```
    pub fn else_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        let else_cmds: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        let then_ref = register_branch(self.then_cmds);
        let else_ref = register_branch(else_cmds);
        let mut result = self
            .cond
            .execute_commands(false, &format!("function {then_ref}"));
        result.extend(
            self.cond
                .execute_commands(true, &format!("function {else_ref}")),
        );
        result
    }
}

impl crate::components::mc_function::IntoCommands for IfThenBuilder {
    fn into_commands(self) -> Vec<String> {
        let then_ref = register_branch(self.then_cmds);
        self.cond
            .execute_commands(false, &format!("function {then_ref}"))
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Begin an `execute if <condition> run …` chain.
///
/// ```rust,ignore
/// // Single command — no branch function generated:
/// let cmds = when(MANA.of("@s").gte(25)).then_one("say enough mana");
///
/// // Grouped branch — condition evaluated once, all commands run in order:
/// let cmds = when(HAS_CELLS.of("@s").is_true()).then_all([
///     tellraw(Selector::self_(), Text::new("Already granted")),
///     cmd::return_fail(),
/// ]);
/// ```
pub fn when(cond: Condition) -> WhenBuilder {
    WhenBuilder {
        cond,
        staged: Vec::new(),
    }
}

/// Begin an `execute unless <condition> run …` chain.
///
/// ```rust,ignore
/// // Single command:
/// let cmds = unless(CASTING.of("@s").is_true()).then_one("say not casting");
///
/// // Grouped branch:
/// let cmds = unless(HAS_CELLS.of("@s").is_true()).then_all([
///     attribute_base_set(Selector::self_(), "minecraft:max_health", 40.0),
///     HAS_CELLS.enable("@s"),
///     cmd::return_cmd(0),
/// ]);
/// ```
pub fn unless(cond: Condition) -> UnlessBuilder {
    UnlessBuilder {
        cond,
        staged: Vec::new(),
    }
}

/// Begin an if/else branch.
///
/// ```rust,ignore
/// if_(HAS_CELLS.of("@s").is_true())
///     .then_all([
///         tellraw(Selector::self_(), Text::new("Already have enhanced cells")),
///         cmd::return_fail(),
///     ])
///     .else_all([
///         attribute_base_set(Selector::self_(), "minecraft:max_health", 40.0),
///         tellraw(Selector::self_(), Text::new("Granted enhanced cells!")),
///         HAS_CELLS.enable("@s"),
///         cmd::return_cmd(0),
///     ]);
/// ```
pub fn if_(cond: Condition) -> IfBuilder {
    IfBuilder { cond }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Condition;
    use crate::state::{Cooldown, Flag, ScoreVar, Ticks};
    use crate::{all, any};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static CASTING: Flag = Flag::new("casting");
    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));

    // ── then_one (direct single-command behavior) ─────────────────────────────

    #[test]
    fn when_score_then_one() {
        let cmds = when(MANA.of("@s").gte(25)).then_one("say ok");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. run say ok"]
        );
    }

    #[test]
    fn unless_flag_then_one() {
        let cmds = unless(CASTING.of("@s").is_true()).then_one("say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn when_then_one_is_direct() {
        let cmds = when(MANA.of("@s").gte(25)).then_one("say enough mana");
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("execute if score @s mana"),
            "got: {}",
            cmds[0]
        );
        assert!(
            !cmds[0].contains("function"),
            "should not call branch fn: {}",
            cmds[0]
        );
    }

    // ── then (single: direct, chained: branch) ────────────────────────────────

    #[test]
    fn when_then_alone_is_direct() {
        reset_branch_counter_for_tests();
        let cmds = when(MANA.of("@s").gte(25)).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. run say ok"]
        );
        assert!(
            !cmds[0].contains("function"),
            "single .then() should be direct"
        );
    }

    #[test]
    fn unless_then_alone_is_direct() {
        reset_branch_counter_for_tests();
        let cmds = unless(CASTING.of("@s").is_true()).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn when_and_then_then_creates_branch() {
        reset_branch_counter_for_tests();
        let cmds = when(MANA.of("@s").gte(25))
            .and_then("say first")
            .and_then("say second")
            .then("say third");
        // Should be one execute line calling a branch function
        assert_eq!(
            cmds.len(),
            1,
            "grouped branch should produce one parent command: {cmds:?}"
        );
        assert!(
            cmds[0].contains("execute if score @s mana"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[0].contains("function"),
            "should call branch fn: {}",
            cmds[0]
        );
    }

    #[test]
    fn unless_and_then_then_creates_branch() {
        reset_branch_counter_for_tests();
        let cmds = unless(CASTING.of("@s").is_true())
            .and_then("say a")
            .then("say b");
        assert_eq!(cmds.len(), 1, "grouped unless branch: {cmds:?}");
        assert!(
            cmds[0].contains("execute unless score @s casting"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[0].contains("function"),
            "should call branch fn: {}",
            cmds[0]
        );
    }

    // ── then_all (always branch) ──────────────────────────────────────────────

    #[test]
    fn when_then_all_creates_branch() {
        reset_branch_counter_for_tests();
        let cmds = when(MANA.of("@s").gte(25)).then_all(["say a", "say b"]);
        assert_eq!(
            cmds.len(),
            1,
            "then_all should produce one parent command: {cmds:?}"
        );
        assert!(
            cmds[0].contains("execute if score @s mana"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[0].contains("function __sand_local:sand/branches/"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn unless_then_all_emits_unless() {
        reset_branch_counter_for_tests();
        let cmds = unless(CASTING.of("@s").is_true()).then_all(["say a", "say b"]);
        assert_eq!(cmds.len(), 1, "unless then_all: {cmds:?}");
        assert!(
            cmds[0].contains("execute unless score @s casting matches 1"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[0].contains("function __sand_local:sand/branches/"),
            "got: {}",
            cmds[0]
        );
    }

    // ── unless polarity regression ─────────────────────────────────────────────

    #[test]
    fn unless_flag_polarity() {
        let cmds = unless(CASTING.of("@s").is_true()).then_one("say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn unless_any_de_morgan() {
        let cmds = unless(Condition::any([
            CASTING.of("@s").is_true(),
            CASTING.of("@s").is_false(),
        ]))
        .then_one("say ok");
        assert_eq!(cmds.len(), 1, "NOT(a OR b) chains into one command");
        assert!(cmds[0].contains("unless"), "got: {}", cmds[0]);
    }

    // ── then_each (per-command, explicit opt-in) ──────────────────────────────

    #[test]
    fn when_then_each_wraps_each() {
        let cmds = when(MANA.of("@s").gte(25)).then_each(["say first", "say second", "say third"]);
        assert_eq!(cmds.len(), 3, "then_each wraps each separately: {cmds:?}");
        assert!(
            cmds[0].contains("execute if score @s mana matches 25.. run say first"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[1].contains("execute if score @s mana matches 25.. run say second"),
            "got: {}",
            cmds[1]
        );
        assert!(
            cmds[2].contains("execute if score @s mana matches 25.. run say third"),
            "got: {}",
            cmds[2]
        );
    }

    #[test]
    fn unless_then_each_wraps_each() {
        let cmds = unless(CASTING.of("@s").is_true()).then_each(["say a", "say b"]);
        assert_eq!(cmds.len(), 2);
        assert!(
            cmds[0].contains("execute unless score @s casting"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[1].contains("execute unless score @s casting"),
            "got: {}",
            cmds[1]
        );
    }

    // ── any/all conditions ────────────────────────────────────────────────────

    #[test]
    fn when_all() {
        let cmds = when(Condition::all([
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            CASTING.of("@s").is_false(),
        ]))
        .then_one("say ready to cast");
        assert_eq!(cmds.len(), 1);
        let cmd = &cmds[0];
        assert!(cmd.starts_with("execute "), "got: {cmd}");
        assert!(cmd.contains("if score @s mana matches 25.."), "got: {cmd}");
        assert!(cmd.contains("if score @s dash matches 0"), "got: {cmd}");
        assert!(cmd.contains("if score @s casting matches 0"), "got: {cmd}");
        assert!(cmd.ends_with("run say ready to cast"), "got: {cmd}");
    }

    #[test]
    fn when_any_expands() {
        let cmds = when(Condition::any([
            MANA.of("@s").gte(25),
            MANA.of("@s").gte(50),
        ]))
        .then_one("say ok");
        assert_eq!(cmds.len(), 2, "Any should expand to two commands");
    }

    #[test]
    fn when_predicate() {
        let cmds = when(Condition::predicate("my_pack:can_cast")).then_one("say ok");
        assert_eq!(
            cmds,
            vec!["execute if predicate my_pack:can_cast run say ok"]
        );
    }

    #[test]
    fn when_entity() {
        let cmds = when(Condition::entity("@s[tag=ready]")).then_one("say ok");
        assert_eq!(cmds, vec!["execute if entity @s[tag=ready] run say ok"]);
    }

    #[test]
    fn nested_not() {
        let cmds = when(!(MANA.of("@s").gte(25))).then_one("say low mana");
        assert_eq!(
            cmds,
            vec!["execute unless score @s mana matches 25.. run say low mana"]
        );
    }

    #[test]
    fn when_cooldown_ready() {
        let cmds = when(DASH.ready("@s")).then_one("say dash ready");
        assert_eq!(
            cmds,
            vec!["execute if score @s dash matches 0 run say dash ready"]
        );
    }

    #[test]
    fn all_conditions_snapshot() {
        let cond = Condition::all([
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            CASTING.of("@s").is_false(),
        ]);
        let cmds = when(cond).then_one("say cast");
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute if score @s mana matches 25.. if score @s dash matches 0 if score @s casting matches 0 run say cast"
        );
    }

    #[test]
    fn all_macro_sugar() {
        let cmds =
            when(all![MANA.of("@s").gte(25), CASTING.of("@s").is_false(),]).then_one("say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s casting"), "got: {}", cmds[0]);
    }

    #[test]
    fn any_macro_sugar() {
        let cmds = when(any![MANA.of("@s").gte(25), MANA.of("@s").gte(50),]).then_one("say ok");
        assert_eq!(cmds.len(), 2, "any! should expand to 2 commands");
    }

    #[test]
    fn nested_any_in_all_via_macros() {
        let cmds = when(all![
            MANA.of("@s").gte(25),
            any![CASTING.of("@s").is_false(), DASH.ready("@s"),],
        ])
        .then_one("say ready");
        assert_eq!(cmds.len(), 2, "all![a, any![b,c]] should give 2 commands");
        assert!(
            cmds.iter().all(|c| c.contains("if score @s mana")),
            "both commands should include mana check: {cmds:?}"
        );
    }

    #[test]
    fn mcfunction_with_state_and_when() {
        use crate::mcfunction;
        static MANA2: ScoreVar<i32> = ScoreVar::new("mana2");
        let cmds = mcfunction![
            MANA2.define();
            when(MANA2.of("@s").gte(25)).then_one("say enough mana");
        ];
        assert_eq!(cmds[0], "scoreboard objectives add mana2 dummy");
        assert!(
            cmds[1].contains("if score @s mana2 matches 25.."),
            "got: {}",
            cmds[1]
        );
    }

    // ── if_ / IfBuilder ───────────────────────────────────────────────────────

    #[test]
    fn if_then_all_creates_branch() {
        use crate::components::mc_function::IntoCommands;
        reset_branch_counter_for_tests();
        let cmds = if_(CASTING.of("@s").is_true())
            .then_all(["say already casting"])
            .into_commands();
        assert_eq!(cmds.len(), 1, "if_ with no else: one parent command");
        assert!(
            cmds[0].contains("execute if score @s casting matches 1"),
            "got: {}",
            cmds[0]
        );
        assert!(
            cmds[0].contains("function __sand_local:sand/branches/"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn if_else_creates_two_branches() {
        reset_branch_counter_for_tests();
        let cmds = if_(CASTING.of("@s").is_true())
            .then_all(["say yes"])
            .else_all(["say no"]);
        assert_eq!(
            cmds.len(),
            2,
            "if/else should produce 2 parent commands: {cmds:?}"
        );
        assert!(
            cmds[0].contains("execute if score @s casting"),
            "then branch: {}",
            cmds[0]
        );
        assert!(
            cmds[1].contains("execute unless score @s casting"),
            "else branch: {}",
            cmds[1]
        );
    }

    #[test]
    fn if_else_polarity() {
        reset_branch_counter_for_tests();
        let flag = Flag::new("active");
        let cmds = if_(flag.of("@s").is_true())
            .then_all(["say active"])
            .else_all(["say inactive"]);
        assert!(
            cmds[0].starts_with("execute if"),
            "then should be if: {}",
            cmds[0]
        );
        assert!(
            cmds[1].starts_with("execute unless"),
            "else should be unless: {}",
            cmds[1]
        );
    }

    // ── return commands ───────────────────────────────────────────────────────

    #[test]
    fn then_all_with_return_fail() {
        reset_branch_counter_for_tests();
        let cmds = when(CASTING.of("@s").is_true())
            .then_all(["say already casting".to_string(), crate::cmd::return_fail()]);
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("function __sand_local:sand/branches/"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn then_all_with_return_cmd() {
        reset_branch_counter_for_tests();
        let cmds = unless(CASTING.of("@s").is_true())
            .then_all(["say starting cast".to_string(), crate::cmd::return_cmd(0)]);
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("execute unless score @s casting"),
            "got: {}",
            cmds[0]
        );
    }

    // ── branch function registration ──────────────────────────────────────────

    #[test]
    fn branch_is_registered_in_dyn_fn_registry() {
        let _ = crate::drain_dyn_fns();
        reset_branch_counter_for_tests();
        let _cmds = when(MANA.of("@s").gte(10)).then_all(["say registered"]);
        let fns = crate::drain_dyn_fns();
        assert!(
            fns.iter().any(|(path, cmds)| {
                path.contains("sand/branches/") && cmds.contains(&"say registered".to_string())
            }),
            "branch fn not found in registry: {fns:?}"
        );
    }

    #[test]
    fn identical_branch_bodies_reuse_generated_helper() {
        let _ = crate::drain_dyn_fns();
        reset_branch_counter_for_tests();
        let first = when(MANA.of("@s").gte(10)).then_all(["say same"]);
        let second = when(MANA.of("@s").gte(20)).then_all(["say same"]);
        let fns = crate::drain_dyn_fns();

        assert_eq!(fns.len(), 1, "identical branch bodies dedupe: {fns:?}");
        let first_path = first[0]
            .split("function ")
            .nth(1)
            .expect("first branch function path");
        let second_path = second[0]
            .split("function ")
            .nth(1)
            .expect("second branch function path");
        assert_eq!(first_path, second_path);
    }
}
