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
//! Generates success and failure functions plus a dispatcher. The dispatcher
//! snapshots the condition result in Sand's internal temporary scoreboard, so
//! exactly one arm runs even if that arm changes the tested state:
//!
//! ```rust,ignore
//! if_(HAS_CELLS.of("@s").is_true())
//!     .then_all([tellraw(...), cmd::return_fail()])
//!     .else_all([attribute_base_set(...), HAS_CELLS.enable("@s")]);
//! // → function <ns>:sand/branches/3
//! //
//! // Dispatcher sand/branches/3 (paths and holder abbreviated):
//! //   scoreboard players set #sand_if_… __sand_tmp 0
//! //   execute if score @s has_cells matches 1 run scoreboard players set #sand_if_… __sand_tmp 1
//! //   execute if score #sand_if_… __sand_tmp matches 1 run function <ns>:sand/branches/2
//! //   execute if score #sand_if_… __sand_tmp matches 0 run function <ns>:sand/branches/1
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

/// A condition with commands that must run before it is evaluated.
///
/// Plain [`Condition`] values convert into this type with no setup, preserving
/// the existing command output. Score expressions use it to materialize their
/// temporary score before the generated `execute` command.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::Conditional",
    summary = "Carries a typed condition together with commands that prepare it for evaluation.",
    context = "Score expressions may need to materialize temporary values before Minecraft can test their lowered Condition; ordinary conditions convert with no setup commands.",
    minecraft = "Emits setup commands first, then renders the condition as execute-if or execute-unless syntax.",
    use_when = ["Passing a computed condition into when, unless, or if_", "Combining a lowered score expression with a normal Condition"],
    avoid_when = ["The branch decision belongs in Rust generation-time control flow", "Raw execute syntax is being passed through without a typed Condition"],
    example = "let conditional: Conditional = condition.into();"
)]
#[must_use = "a Conditional has no effect until passed to a branch builder"]
pub struct Conditional {
    setup: Vec<String>,
    condition: Condition,
}

impl Conditional {
    #[doc(hidden)]
    pub(crate) fn with_setup(setup: Vec<String>, condition: Condition) -> Self {
        Self { setup, condition }
    }

    /// Combine this lowered condition with a normal condition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::Conditional::and",
        summary = "Requires both the prepared condition and another typed condition to hold.",
        context = "The prepared setup remains attached while boolean composition extends the condition evaluated by a branch builder.",
        minecraft = "Renders the conjunction through Condition's execute-if command plan after running the existing setup commands.",
        use_when = ["Adding a normal typed condition to a computed branch condition"],
        avoid_when = ["The second operand also requires setup commands", "Either condition succeeding should run the branch"],
        params(other = "The additional typed condition that must also succeed."),
        returns = "The combined prepared condition with its original setup commands.",
        example = "conditional.and(READY.of(\"@s\").is_true())"
    )]
    pub fn and(self, other: Condition) -> Self {
        Self {
            setup: self.setup,
            condition: self.condition.and(other),
        }
    }

    /// Combine this lowered condition with a normal condition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::Conditional::or",
        summary = "Allows either the prepared condition or another typed condition to hold.",
        context = "The prepared setup remains attached while boolean composition extends the condition evaluated by a branch builder.",
        minecraft = "Renders the disjunction through Condition's execute command plan after running the existing setup commands.",
        use_when = ["Adding an alternative normal condition to a computed branch condition"],
        avoid_when = ["The second operand also requires setup commands", "Both conditions must succeed"],
        params(other = "The alternative typed condition that may satisfy the branch."),
        returns = "The combined prepared condition with its original setup commands.",
        example = "conditional.or(FALLBACK.of(\"@s\").is_true())"
    )]
    pub fn or(self, other: Condition) -> Self {
        Self {
            setup: self.setup,
            condition: self.condition.or(other),
        }
    }

    fn execute_commands(&self, negated: bool, run: &str) -> Vec<String> {
        let mut commands = self.setup.clone();
        commands.extend(self.condition.execute_commands(negated, run));
        commands
    }
}

impl From<Condition> for Conditional {
    fn from(condition: Condition) -> Self {
        Self {
            setup: Vec::new(),
            condition,
        }
    }
}

/// Reset the branch counter. For use in unit tests only — keeps paths stable.
#[cfg(test)]
fn reset_branch_counter_for_tests() {
    crate::drain_dyn_fns();
}

/// Register commands as an anonymous branch function and return its path.
///
/// Uses `__sand_local:` sentinel so the namespace is resolved at export time.
fn register_branch(commands: Vec<String>) -> String {
    let path = crate::register_dyn_fn_dedup("sand/branches", commands);
    format!("__sand_local:{path}")
}

fn branch_decision_holder(seed: &[String]) -> String {
    let mut hash: u32 = 2_166_136_261;
    for value in seed {
        for byte in value.bytes().chain(std::iter::once(0)) {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16_777_619);
        }
    }
    format!("#sand_if_{hash:08x}")
}

// ── WhenBuilder ───────────────────────────────────────────────────────────────

/// Builder returned by [`when`]. Call [`then_one`](WhenBuilder::then_one),
/// [`then_all`](WhenBuilder::then_all), or build up with [`and_then`](WhenBuilder::and_then).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::WhenBuilder",
    summary = "Builds commands that run when a typed condition succeeds.",
    context = "The builder makes single-command, grouped one-time evaluation, and explicit per-command re-evaluation distinct authoring choices.",
    minecraft = "Emits execute if commands and may register an anonymous helper function for grouped commands.",
    use_when = ["Running one or more commands under a typed positive condition"],
    avoid_when = ["Commands should run when the condition fails", "The choice can be made entirely while generating Rust code"],
    example = "when(condition).then_one(\"say ready\")"
)]
#[must_use = "a WhenBuilder emits no commands until a terminal then method is called"]
pub struct WhenBuilder {
    cond: Conditional,
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::WhenBuilder::and_then",
        summary = "Stages another command for a grouped positive branch.",
        context = "Staging defers emission until then supplies the final command, ensuring the condition is evaluated once for the complete sequence.",
        minecraft = "Stores the rendered command for an anonymous function later invoked by one execute if command.",
        use_when = ["Building a grouped branch incrementally"],
        avoid_when = ["Each command must re-test the condition", "The complete command collection is already available"],
        params(cmd = "The displayable Minecraft command to append to the staged branch."),
        returns = "The builder with the rendered command appended.",
        example = "when(condition).and_then(\"say first\").then(\"say second\")"
    )]
    pub fn and_then(mut self, cmd: impl std::fmt::Display) -> WhenBuilder {
        self.staged.push(cmd.to_string());
        self
    }

    /// Finish the chain.
    ///
    /// - With no prior `.and_then(...)`: emits a single `execute if … run <cmd>` line.
    /// - With prior `.and_then(...)` calls: creates a grouped branch function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::WhenBuilder::then",
        summary = "Finishes a positive branch as either one direct command or one grouped helper call.",
        context = "The presence of previously staged commands selects grouped one-time evaluation; an otherwise empty builder keeps the direct single-command form.",
        minecraft = "Emits setup commands followed by execute if ... run, targeting either the supplied command or a generated function.",
        use_when = ["Finishing an incrementally staged branch", "Letting the builder choose direct versus grouped output"],
        avoid_when = ["A helper function must be generated even for one command", "Every command must re-test the condition"],
        params(cmd = "The final displayable Minecraft command in the branch."),
        returns = "The complete command lines to emit in the parent function.",
        example = "when(condition).then(\"say ready\")"
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::WhenBuilder::then_one",
        summary = "Wraps one command directly in a positive execute condition.",
        context = "The explicit single-command operation avoids registering an anonymous branch function.",
        minecraft = "Emits setup commands followed by one execute if ... run <command> line.",
        use_when = ["Running exactly one command when a condition succeeds"],
        avoid_when = ["Several commands must share one condition evaluation", "A named or generated helper function is required"],
        params(cmd = "The displayable Minecraft command to run on success."),
        returns = "The setup and conditional command lines for the parent function.",
        example = "when(condition).then_one(\"say ready\")"
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::WhenBuilder::then_all",
        summary = "Runs a command collection in one grouped positive branch.",
        context = "Grouping evaluates the condition once before entering an anonymous helper, so commands that mutate the condition do not suppress later commands.",
        minecraft = "Registers the rendered commands as a generated function and emits execute if ... run function for it.",
        use_when = ["Several commands must run in order under one successful check", "Branch commands may mutate the tested state"],
        avoid_when = ["Each command intentionally needs a fresh condition check", "Only one direct command is needed"],
        params(cmds = "The displayable Minecraft commands to place in the grouped branch."),
        returns = "The setup and conditional helper-call lines for the parent function.",
        example = "when(condition).then_all([\"say first\", \"say second\"])"
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::WhenBuilder::then_each",
        summary = "Re-evaluates a positive condition independently for every command.",
        context = "This explicit alternative preserves per-command gating when earlier commands are intended to affect whether later commands run.",
        minecraft = "Emits one execute if ... run line per supplied command, repeating any condition command plan each time.",
        use_when = ["Every command intentionally requires a fresh condition evaluation"],
        avoid_when = ["All commands must run after one successful check", "Repeated evaluation or setup would be wasteful"],
        params(cmds = "The displayable commands to wrap with separate condition checks."),
        returns = "The independently conditioned command lines in input order.",
        example = "when(condition).then_each([\"say first\", \"say second\"])"
    )]
    pub fn then_each(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        cmds.into_iter()
            .flat_map(|cmd| self.cond.execute_commands(false, &cmd.to_string()))
            .collect()
    }
}

// ── UnlessBuilder ─────────────────────────────────────────────────────────────

/// Builder returned by [`unless`]. Call [`then_one`](UnlessBuilder::then_one),
/// [`then_all`](UnlessBuilder::then_all), or build up with [`and_then`](UnlessBuilder::and_then).
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::UnlessBuilder",
    summary = "Builds commands that run when a typed condition fails.",
    context = "The negative builder mirrors WhenBuilder while preserving explicit choices between grouped and per-command evaluation.",
    minecraft = "Emits execute unless commands and may register an anonymous helper function for grouped commands.",
    use_when = ["Running one or more commands under a typed negative condition"],
    avoid_when = ["Commands should run when the condition succeeds", "The choice can be made entirely while generating Rust code"],
    example = "unless(condition).then_one(\"say unavailable\")"
)]
#[must_use = "an UnlessBuilder emits no commands until a terminal then method is called"]
pub struct UnlessBuilder {
    cond: Conditional,
    /// Commands accumulated via `.and_then(...)`.
    staged: Vec<String>,
}

impl UnlessBuilder {
    /// Accumulate a command to run unless the condition holds.
    ///
    /// Calling `.then(cmd)` afterwards creates a **grouped branch function**.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::UnlessBuilder::and_then",
        summary = "Stages another command for a grouped negative branch.",
        context = "Staging defers emission until then supplies the final command, ensuring the failed-condition branch is entered only once.",
        minecraft = "Stores the rendered command for an anonymous function later invoked by one execute unless command.",
        use_when = ["Building a grouped negative branch incrementally"],
        avoid_when = ["Each command must re-test the condition", "The complete command collection is already available"],
        params(cmd = "The displayable Minecraft command to append to the staged branch."),
        returns = "The builder with the rendered command appended.",
        example = "unless(condition).and_then(\"say first\").then(\"say second\")"
    )]
    pub fn and_then(mut self, cmd: impl std::fmt::Display) -> UnlessBuilder {
        self.staged.push(cmd.to_string());
        self
    }

    /// Finish the chain.
    ///
    /// - With no prior `.and_then(...)`: emits a single `execute unless … run <cmd>` line.
    /// - With prior `.and_then(...)` calls: creates a grouped branch function.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::UnlessBuilder::then",
        summary = "Finishes a negative branch as either one direct command or one grouped helper call.",
        context = "Previously staged commands select grouped one-time evaluation; an otherwise empty builder keeps the direct single-command form.",
        minecraft = "Emits setup commands followed by execute unless ... run, targeting either the supplied command or a generated function.",
        use_when = ["Finishing an incrementally staged negative branch", "Letting the builder choose direct versus grouped output"],
        avoid_when = ["A helper function must be generated even for one command", "Every command must re-test the condition"],
        params(cmd = "The final displayable Minecraft command in the branch."),
        returns = "The complete command lines to emit in the parent function.",
        example = "unless(condition).then(\"say unavailable\")"
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::UnlessBuilder::then_one",
        summary = "Wraps one command directly in a negative execute condition.",
        context = "The explicit single-command operation avoids registering an anonymous branch function.",
        minecraft = "Emits setup commands followed by one execute unless ... run <command> line.",
        use_when = ["Running exactly one command when a condition fails"],
        avoid_when = ["Several commands must share one condition evaluation", "A named or generated helper function is required"],
        params(cmd = "The displayable Minecraft command to run when the condition fails."),
        returns = "The setup and conditional command lines for the parent function.",
        example = "unless(condition).then_one(\"say unavailable\")"
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::UnlessBuilder::then_all",
        summary = "Runs a command collection in one grouped negative branch.",
        context = "Grouping evaluates the condition once before entering an anonymous helper, so commands that change it do not suppress later commands.",
        minecraft = "Registers the rendered commands as a generated function and emits execute unless ... run function for it.",
        use_when = ["Several commands must run in order after one failed check", "Branch commands may mutate the tested state"],
        avoid_when = ["Each command intentionally needs a fresh condition check", "Only one direct command is needed"],
        params(cmds = "The displayable Minecraft commands to place in the grouped negative branch."),
        returns = "The setup and conditional helper-call lines for the parent function.",
        example = "unless(condition).then_all([\"say first\", \"say second\"])"
    )]
    pub fn then_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        let commands: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        let branch_ref = register_branch(commands);
        self.cond
            .execute_commands(true, &format!("function {branch_ref}"))
    }

    /// Wrap **each** command in the condition separately (old per-command behavior).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::UnlessBuilder::then_each",
        summary = "Re-evaluates a negative condition independently for every command.",
        context = "This explicit alternative preserves per-command gating when earlier commands are intended to affect whether later commands run.",
        minecraft = "Emits one execute unless ... run line per supplied command, repeating any condition command plan each time.",
        use_when = ["Every command intentionally requires a fresh failed-condition evaluation"],
        avoid_when = ["All commands must run after one failed check", "Repeated evaluation or setup would be wasteful"],
        params(cmds = "The displayable commands to wrap with separate negative checks."),
        returns = "The independently conditioned command lines in input order.",
        example = "unless(condition).then_each([\"say first\", \"say second\"])"
    )]
    pub fn then_each(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        cmds.into_iter()
            .flat_map(|cmd| self.cond.execute_commands(true, &cmd.to_string()))
            .collect()
    }
}

// ── IfBuilder / IfThenBuilder (if/else) ──────────────────────────────────────

/// Builder returned by [`if_`]. Supplies a `then_all` arm.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::IfBuilder",
    summary = "Begins a grouped conditional branch from a typed condition.",
    context = "This first stage captures the condition before then_all records the success commands and returns an IfThenBuilder for an optional failure arm.",
    minecraft = "Carries rendered success commands forward until the completed builder registers them as an anonymous function and emits the condition check.",
    use_when = ["Building a grouped success branch that may also need an else arm"],
    avoid_when = ["Only one direct command is needed", "The branch decision belongs in Rust generation-time control flow"],
    example = "if_(condition).then_all([\"say yes\"]).else_all([\"say no\"])"
)]
#[must_use = "an IfBuilder emits no commands until then_all is called"]
pub struct IfBuilder {
    cond: Conditional,
}

impl IfBuilder {
    /// Specify the commands to run when the condition holds.
    ///
    /// Returns an [`IfThenBuilder`] where you can optionally attach an `.else_all(...)`.
    /// Accepts any value implementing [`Display`](std::fmt::Display).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::IfBuilder::then_all",
        summary = "Defines the grouped success arm of a conditional branch.",
        context = "The returned builder can be emitted as a positive-only branch through IntoCommands or completed with a mutually exclusive else_all arm.",
        minecraft = "Renders and stores the commands for an anonymous success function; the condition is evaluated when the returned builder is consumed.",
        use_when = ["Several success commands must share one condition decision", "An else arm may be attached afterward"],
        avoid_when = ["A single direct conditional command is sufficient", "Each success command needs a fresh condition check"],
        params(cmds = "The displayable Minecraft commands for the grouped success arm."),
        returns = "A builder carrying the condition and registered success-arm input, ready for optional else completion.",
        example = "if_(condition).then_all([\"say yes\"])"
    )]
    pub fn then_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> IfThenBuilder {
        let then_cmds: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        IfThenBuilder {
            cond: self.cond,
            then_cmds,
        }
    }
}

/// Returned by [`IfBuilder::then_all`]. Finishes with `.else_all(...)` or used alone.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::IfThenBuilder",
    summary = "Carries a grouped success arm awaiting emission or an optional failure arm.",
    context = "Consuming this builder through IntoCommands emits a positive-only branch; else_all completes one stable, mutually exclusive decision between two grouped arms.",
    minecraft = "Registers grouped branch functions and emits either one execute-if call or one dispatcher that snapshots the condition in an internal score before selecting exactly one arm.",
    use_when = ["Holding the result of IfBuilder::then_all before choosing whether to add an else arm"],
    avoid_when = ["The success branch should be discarded", "Commands need independent condition re-evaluation"],
    example = "if_(condition).then_all([\"say yes\"]).else_all([\"say no\"])"
)]
#[must_use = "an IfThenBuilder must be emitted with IntoCommands or completed with else_all"]
pub struct IfThenBuilder {
    cond: Conditional,
    then_cmds: Vec<String>,
}

impl IfThenBuilder {
    /// Attach an else arm — commands to run when the condition does **not** hold.
    ///
    /// Generates branch helpers and one dispatcher function. The dispatcher
    /// snapshots the condition in an internal score before selecting one arm.
    ///
    /// ```rust,ignore
    /// if_(HAS_CELLS.of("@s").is_true())
    ///     .then_all([tellraw(...), cmd::return_fail()])
    ///     .else_all([attribute_base_set(...), HAS_CELLS.enable("@s")]);
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::execute_when::IfThenBuilder::else_all",
        summary = "Completes a conditional branch with mutually exclusive grouped success and failure arms.",
        context = "A generated dispatcher snapshots one decision before calling either arm, preventing success commands that mutate the condition from subsequently activating the failure arm.",
        minecraft = "Registers success and failure functions, snapshots the typed condition as zero or one in Sand's internal temporary scoreboard, and dispatches exactly one matching branch. A success wrapper marks the decision consumed after nested calls return.",
        use_when = ["Exactly one of two command sequences must run from a typed condition", "Either arm may mutate the state tested by the condition"],
        avoid_when = ["Only a positive or negative branch is required", "The choice belongs in Rust generation-time control flow"],
        params(cmds = "The displayable Minecraft commands for the grouped failure arm."),
        returns = "Setup commands followed by one call to the generated single-decision dispatcher.",
        example = "if_(condition).then_all([\"say yes\"]).else_all([\"say no\"])"
    )]
    pub fn else_all(self, cmds: impl IntoIterator<Item = impl std::fmt::Display>) -> Vec<String> {
        let else_cmds: Vec<String> = cmds.into_iter().map(|c| c.to_string()).collect();
        let then_ref = register_branch(self.then_cmds);
        let else_ref = register_branch(else_cmds);
        crate::state::score::request_expression_temp();

        let mut holder_seed = vec![then_ref.clone(), else_ref.clone()];
        holder_seed.extend(
            self.cond
                .condition
                .execute_commands(false, "scoreboard players set #sand_if_seed __sand_tmp 1"),
        );
        let decision_holder = branch_decision_holder(&holder_seed);
        let objective = crate::state::score::SCORE_EXPRESSION_TEMP_OBJECTIVE;

        // A nested invocation may reuse the same deterministic holder. Marking
        // the successful decision consumed after the arm returns prevents such
        // an invocation from making this dispatch fall through to its else arm.
        let then_wrapper_ref = register_branch(vec![
            format!("function {then_ref}"),
            format!("scoreboard players set {decision_holder} {objective} 2"),
        ]);

        let mut dispatcher = vec![format!(
            "scoreboard players set {decision_holder} {objective} 0"
        )];
        dispatcher.extend(self.cond.condition.execute_commands(
            false,
            &format!("scoreboard players set {decision_holder} {objective} 1"),
        ));
        dispatcher.push(format!(
            "execute if score {decision_holder} {objective} matches 1 run function {then_wrapper_ref}"
        ));
        dispatcher.push(format!(
            "execute if score {decision_holder} {objective} matches 0 run function {else_ref}"
        ));
        let dispatcher_ref = register_branch(dispatcher);
        let mut result = self.cond.setup.clone();
        result.push(format!("function {dispatcher_ref}"));
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::when",
    aliases = ["sand::prelude::when"],
    summary = "Begins a command branch that runs when a typed condition succeeds.",
    context = "The returned builder distinguishes direct single commands, grouped one-time evaluation, and explicit per-command re-evaluation.",
    minecraft = "Carries the condition and any preparation commands into an execute-if branch builder.",
    use_when = ["Gating Minecraft commands on a typed positive condition"],
    avoid_when = ["Commands should run when the condition fails", "The decision is known while generating Rust code"],
    params(cond = "The typed condition, or prepared Conditional, that must succeed."),
    returns = "A positive branch builder awaiting commands.",
    example = "when(condition).then_one(\"say ready\")"
)]
pub fn when(cond: impl Into<Conditional>) -> WhenBuilder {
    WhenBuilder {
        cond: cond.into(),
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::unless",
    aliases = ["sand::prelude::unless"],
    summary = "Begins a command branch that runs when a typed condition fails.",
    context = "The returned builder provides negative counterparts to direct, grouped, and per-command positive branch operations.",
    minecraft = "Carries the condition and any preparation commands into an execute-unless branch builder.",
    use_when = ["Gating Minecraft commands on a typed condition not holding"],
    avoid_when = ["Commands should run when the condition succeeds", "The decision is known while generating Rust code"],
    params(cond = "The typed condition, or prepared Conditional, that must fail."),
    returns = "A negative branch builder awaiting commands.",
    example = "unless(condition).then_one(\"say unavailable\")"
)]
pub fn unless(cond: impl Into<Conditional>) -> UnlessBuilder {
    UnlessBuilder {
        cond: cond.into(),
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
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::execute_when::if_",
    aliases = ["sand::prelude::if_"],
    summary = "Begins a grouped conditional branch with an optional mutually exclusive else arm.",
    context = "The staged builder API records success commands before deciding whether to emit a positive-only branch or complete a two-arm branch.",
    minecraft = "A completed if/else uses one generated dispatcher that snapshots the condition in an internal score before selecting exactly one generated branch function.",
    use_when = ["Choosing exactly one of two grouped Minecraft command sequences", "The selected branch may mutate the tested state"],
    avoid_when = ["Only one direct positive or negative command is needed", "The choice belongs in Rust generation-time control flow"],
    params(cond = "The typed condition, or prepared Conditional, that selects the success arm."),
    returns = "A conditional builder awaiting its grouped success commands.",
    example = "if_(condition).then_all([\"say yes\"]).else_all([\"say no\"])"
)]
pub fn if_(cond: impl Into<Conditional>) -> IfBuilder {
    IfBuilder { cond: cond.into() }
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

    // The dynamic-function registry is thread-local (see
    // `crate::function`), so each test thread already has an isolated
    // view — no cross-test lock is needed, just a per-call reset so
    // assertions on generated branch paths start from a clean slate.
    fn reset_dynamic_branch_registry_for_test() {
        let _ = crate::drain_dyn_fns();
        reset_branch_counter_for_tests();
    }

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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
        let cmds = unless(CASTING.of("@s").is_true()).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn when_and_then_then_creates_branch() {
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
    fn if_else_uses_one_parent_dispatch_call() {
        reset_dynamic_branch_registry_for_test();
        let cmds = if_(CASTING.of("@s").is_true())
            .then_all(["say yes"])
            .else_all(["say no"]);
        assert_eq!(
            cmds.len(),
            1,
            "if/else should make one stable decision in a dispatcher: {cmds:?}"
        );
        assert!(
            cmds[0].starts_with("function __sand_local:sand/branches/"),
            "dispatcher call: {}",
            cmds[0]
        );
    }

    #[test]
    fn if_else_dispatcher_snapshots_exactly_one_arm_without_return_run() {
        reset_dynamic_branch_registry_for_test();
        let flag = Flag::new("active");
        let cmds = if_(flag.of("@s").is_true())
            .then_all(["say active"])
            .else_all(["say inactive"]);

        let registered = crate::drain_dyn_fns();
        assert_eq!(
            registered.len(),
            4,
            "then, else, success wrapper, and dispatcher functions"
        );
        let dispatcher_path = cmds[0]
            .strip_prefix("function __sand_local:")
            .expect("parent calls the dispatcher");
        let dispatcher = registered
            .iter()
            .find(|(path, _)| path == dispatcher_path)
            .map(|(_, commands)| commands)
            .expect("dispatcher is registered");

        assert_eq!(dispatcher.len(), 4);
        assert!(dispatcher[0].starts_with("scoreboard players set #sand_if_"));
        assert!(
            dispatcher[1].contains(
                "execute if score @s active matches 1 run scoreboard players set #sand_if_"
            ),
            "condition result is snapshotted before either arm: {}",
            dispatcher[1]
        );
        assert!(
            dispatcher[2].contains("matches 1 run function __sand_local:"),
            "success uses the snapshotted result: {}",
            dispatcher[2]
        );
        assert!(
            dispatcher[3].contains("matches 0 run function __sand_local:"),
            "failure uses the same snapshotted result: {}",
            dispatcher[3]
        );
        assert!(
            dispatcher
                .iter()
                .all(|command| !command.contains("return run"))
        );

        let profile = sand_commands::CommandProfile::new("1.19.4", false);
        for command in registered.iter().flat_map(|(_, commands)| commands) {
            sand_commands::render::validate_collected_line(command, &profile)
                .unwrap_or_else(|error| panic!("1.19.4 rejected `{command}`: {error}"));
        }
        let score_setup = crate::state::score::drain_internal_score_setup();
        assert!(score_setup.contains(&"scoreboard objectives add __sand_tmp dummy".to_string()));
    }

    #[test]
    fn concurrent_if_else_exports_keep_independent_score_setup_requests() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads = (0..2)
            .map(|index| {
                let barrier = std::sync::Arc::clone(&barrier);
                std::thread::spawn(move || {
                    let _commands = if_(Condition::raw(format!(
                        "score @s concurrent_{index} matches 1"
                    )))
                    .then_all(["say yes"])
                    .else_all(["say no"]);
                    barrier.wait();
                    crate::state::score::drain_internal_score_setup()
                })
            })
            .collect::<Vec<_>>();

        for thread in threads {
            let setup = thread.join().expect("concurrent export thread succeeds");
            assert!(
                setup.contains(&"scoreboard objectives add __sand_tmp dummy".to_string()),
                "each export thread must retain its own score setup request: {setup:?}"
            );
        }
    }

    #[test]
    fn failed_export_scope_cannot_leak_helpers_after_score_setup_was_consumed() {
        {
            let _scope = crate::function::ExportFunctionRegistryScope::enter();
            let commands = if_(Condition::raw("score @s failed_export matches 1"))
                .then_all(["say yes"])
                .else_all(["say no"]);
            assert_eq!(commands.len(), 1);

            // Reproduce the dangerous pipeline ordering: consume score setup,
            // then leave through an error before dynamic helpers are drained.
            let setup = crate::state::score::drain_internal_score_setup();
            assert!(setup.contains(&"scoreboard objectives add __sand_tmp dummy".to_string()));
        }

        assert!(crate::drain_dyn_fns().is_empty());
        assert!(crate::state::score::drain_internal_score_setup().is_empty());
    }

    #[test]
    fn if_else_any_condition_keeps_one_fallback_after_all_success_checks() {
        reset_dynamic_branch_registry_for_test();
        let cmds = if_(Condition::any([
            MANA.of("@s").gte(25),
            CASTING.of("@s").is_true(),
        ]))
        .then_all(["say yes"])
        .else_all(["say no"]);

        let registered = crate::drain_dyn_fns();
        let dispatcher_path = cmds[0]
            .strip_prefix("function __sand_local:")
            .expect("parent calls the dispatcher");
        let dispatcher = registered
            .iter()
            .find(|(path, _)| path == dispatcher_path)
            .map(|(_, commands)| commands)
            .expect("dispatcher is registered");

        assert_eq!(
            dispatcher.len(),
            5,
            "reset, two Any checks, and two score-selected arms"
        );
        assert!(
            dispatcher[1..3]
                .iter()
                .all(|command| command.contains("run scoreboard players set #sand_if_"))
        );
        assert!(dispatcher[3].contains("matches 1 run function __sand_local:"));
        assert!(dispatcher[4].contains("matches 0 run function __sand_local:"));
    }

    // ── return commands ───────────────────────────────────────────────────────

    #[test]
    fn then_all_with_return_fail() {
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
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
        reset_dynamic_branch_registry_for_test();
        let first = when(MANA.of("@s").gte(10)).then_all(["say same"]);
        let second = when(MANA.of("@s").gte(20)).then_all(["say same"]);
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

    #[test]
    fn reset_dynamic_branch_registry_clears_stale_entries() {
        {
            reset_dynamic_branch_registry_for_test();
            let _cmds = when(MANA.of("@s").gte(10)).then_all(["say stale"]);
        }

        reset_dynamic_branch_registry_for_test();
        let fns = crate::drain_dyn_fns();
        assert!(
            fns.is_empty(),
            "expected empty registry after reset, got: {fns:?}"
        );
    }

    #[test]
    fn score_expression_setup_precedes_branch_check_once() {
        reset_dynamic_branch_registry_for_test();
        static COST: ScoreVar<i32> = ScoreVar::new("cost");
        let commands = if_(MANA.of("@s").expr().minus(COST.of("@s")).gte(0))
            .then_all(["say yes"])
            .else_all(["say no"]);
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands[0],
            "scoreboard players operation @s __sand_tmp = @s mana"
        );
        assert_eq!(
            commands[1],
            "scoreboard players operation @s __sand_tmp -= @s cost"
        );
        let dispatcher_path = commands[2]
            .strip_prefix("function __sand_local:")
            .expect("setup is followed by one dispatcher call");
        let registered = crate::drain_dyn_fns();
        let dispatcher = registered
            .iter()
            .find(|(path, _)| path == dispatcher_path)
            .map(|(_, body)| body)
            .expect("dispatcher is registered");
        assert_eq!(dispatcher.len(), 4);
        assert!(dispatcher[1].starts_with(
            "execute if score @s __sand_tmp matches 0.. run scoreboard players set #sand_if_"
        ));
        assert!(dispatcher[2].contains("matches 1 run function"));
        assert!(dispatcher[3].contains("matches 0 run function"));
    }
}
