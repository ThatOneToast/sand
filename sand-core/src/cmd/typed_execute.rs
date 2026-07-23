//! Condition-aware execute builder.
//!
//! Extends the low-level [`Execute`] builder from `sand-commands` with typed
//! [`Condition`](crate::condition::Condition) support.  `Any`-expansion into
//! multiple commands is handled automatically.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::cmd::{ExecuteExt, TypedExecute};
//! use sand_core::state::ScoreVar;
//! use sand_core::{all, any};
//! use sand_commands::Selector;
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//!
//! // Single command
//! let cmds: Vec<String> = TypedExecute::as_players()
//!     .at(Selector::self_())
//!     .when(MANA.of("@s").gte(25))
//!     .run("say enough mana");
//!
//! // any! expansion → 2 commands
//! let cmds: Vec<String> = TypedExecute::as_players()
//!     .when(any![MANA.of("@s").gte(25), MANA.of("@s").gte(50)])
//!     .run("say ok");
//! ```

use std::fmt;

use sand_commands::{Execute, Selector};

use crate::condition::Condition;

// ── ConditionedExecute ────────────────────────────────────────────────────────

/// An execute chain paired with a typed [`Condition`].
///
/// Created by [`ExecuteExt::when`] or [`ExecuteExt::unless`].
/// Call [`run`](ConditionedExecute::run) to finalize into `Vec<String>`.
pub struct ConditionedExecute {
    prefix: Execute,
    cond: Condition,
    negated: bool,
}

impl ConditionedExecute {
    /// Add another AND condition (Cartesian-product expansion).
    pub fn and_when(self, cond: Condition) -> Self {
        let combined = Condition::all([self.cond, cond]);
        Self {
            prefix: self.prefix,
            cond: combined,
            negated: self.negated,
        }
    }

    /// Finalize the execute chain.
    ///
    /// Returns one command per expanded plan.  A simple score condition gives
    /// one string; `any![...]` gives N strings.
    ///
    /// Accepts any `Display` value — raw `&str`, owned `String`, or any
    /// command builder.
    pub fn run(self, cmd: impl fmt::Display) -> Vec<String> {
        let cmd_str = cmd.to_string();
        self.cond
            .to_ir_plans(self.negated)
            .into_iter()
            .map(|clauses| {
                clauses
                    .into_iter()
                    .fold(self.prefix.clone(), |execute, clause| {
                        execute.with_operation(clause.into_operation())
                    })
                    .run(&cmd_str)
            })
            .collect()
    }
}

// ── ExecuteExt ────────────────────────────────────────────────────────────────

/// Extension trait — adds `when` and `unless` to the low-level [`Execute`] builder.
///
/// Import this trait to access the typed condition methods.
///
/// ```rust,ignore
/// use sand_core::cmd::ExecuteExt;
/// use sand_commands::Execute;
///
/// let cmds = Execute::new()
///     .as_(Selector::all_players())
///     .when(MANA.of("@s").gte(25))
///     .run("say enough");
/// ```
pub trait ExecuteExt: Sized {
    /// Attach a typed condition — returns a [`ConditionedExecute`] whose
    /// [`run`](ConditionedExecute::run) produces `Vec<String>`.
    fn when(self, cond: Condition) -> ConditionedExecute;

    /// Attach a negated typed condition.
    fn unless(self, cond: Condition) -> ConditionedExecute;
}

impl ExecuteExt for Execute {
    fn when(self, cond: Condition) -> ConditionedExecute {
        ConditionedExecute {
            prefix: self,
            cond,
            negated: false,
        }
    }

    fn unless(self, cond: Condition) -> ConditionedExecute {
        ConditionedExecute {
            prefix: self,
            cond,
            negated: true,
        }
    }
}

// ── TypedExecute ──────────────────────────────────────────────────────────────

/// Convenience constructors for common `execute` patterns.
///
/// Each method returns a bare [`Execute`] so you can chain standard sub-commands
/// before calling [`when`](ExecuteExt::when) or terminating with [`Execute::run`].
pub struct TypedExecute;

impl TypedExecute {
    /// `execute as @a` — run as every player.
    pub fn as_players() -> Execute {
        Execute::new().as_(Selector::all_players())
    }

    /// `execute as @e` — run as every entity.
    pub fn as_entities() -> Execute {
        Execute::new().as_(Selector::all_entities())
    }

    /// `execute as @s at @s` — run as self, at self's position.
    pub fn as_self_at_self() -> Execute {
        Execute::new().as_(Selector::self_()).at(Selector::self_())
    }

    /// `execute as @a at @s` — run as every player at their own position.
    pub fn as_players_at_self() -> Execute {
        Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use sand_commands::{Build, Execute};

    use super::*;
    use crate::state::{Flag, ScoreVar};
    use crate::{all, any};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static CASTING: Flag = Flag::new("casting");

    #[test]
    fn when_single_condition() {
        let cmds = Execute::new()
            .as_(Selector::all_players())
            .when(MANA.of("@s").gte(25))
            .run("say enough mana");
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute as @a if score @s mana matches 25.. run say enough mana"
        );
    }

    #[test]
    fn unless_condition() {
        let cmds = Execute::new()
            .unless(CASTING.of("@s").is_true())
            .run("say not casting");
        assert_eq!(cmds.len(), 1);
        assert!(
            cmds[0].contains("unless score @s casting matches 1"),
            "got: {}",
            cmds[0]
        );
    }

    #[test]
    fn when_any_expands() {
        let cmds = Execute::new()
            .as_(Selector::all_players())
            .when(any![MANA.of("@s").gte(25), MANA.of("@s").gte(50),])
            .run("say ok");
        assert_eq!(cmds.len(), 2, "any! should produce 2 commands");
    }

    #[test]
    fn when_all_macro() {
        let cmds = Execute::new()
            .as_(Selector::all_players())
            .at(Selector::self_())
            .when(all![MANA.of("@s").gte(25), CASTING.of("@s").is_false(),])
            .run("say ready");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s casting"), "got: {}", cmds[0]);
    }

    #[test]
    fn nested_any_in_all_via_execute() {
        let cmds = Execute::new()
            .when(all![
                MANA.of("@s").gte(25),
                any![CASTING.of("@s").is_false(), CASTING.of("@s").is_true(),],
            ])
            .run("say ok");
        assert_eq!(cmds.len(), 2, "all![a, any![b,c]] gives 2 commands");
    }

    #[test]
    fn and_when_chaining() {
        let cmds = Execute::new()
            .when(MANA.of("@s").gte(25))
            .and_when(CASTING.of("@s").is_false())
            .run("say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s casting"), "got: {}", cmds[0]);
    }

    #[test]
    fn as_players_shorthand() {
        let exec = TypedExecute::as_players();
        assert!(exec.build().contains("as @a"), "got: {}", exec.build());
    }

    #[test]
    fn as_players_at_self_shorthand() {
        let exec = TypedExecute::as_players_at_self();
        let s = exec.build();
        assert!(s.contains("as @a"), "got: {s}");
        assert!(s.contains("at @s"), "got: {s}");
    }

    #[test]
    fn golden_spell_execute() {
        // Matches the documented spell system pattern exactly
        let cmds = TypedExecute::as_players_at_self()
            .when(all![MANA.of("@s").gte(25), CASTING.of("@s").is_false(),])
            .run("function example:dash");
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute as @a at @s if score @s mana matches 25.. if score @s casting matches 0 run function example:dash"
        );
    }
}
