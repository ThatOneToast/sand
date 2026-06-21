//! Typed `when` / `unless` execute integration.
//!
//! Wraps a [`Condition`] and a command into complete `execute if/unless … run …`
//! command strings without any raw execute syntax.
//!
//! # Example
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::condition::Condition;
//! use sand_core::execute_when::{when, unless};
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//! static CASTING: Flag = Flag::new("casting");
//!
//! // Single condition
//! let cmds = when(MANA.of("@s").gte(25)).then("say enough mana");
//!
//! // Inverted condition
//! let cmds = unless(CASTING.of("@s").is_true()).then("say not casting");
//!
//! // Compound condition (All → single command, Any → multiple commands)
//! let cmds = when(Condition::all([
//!     MANA.of("@s").gte(25),
//!     CASTING.of("@s").is_false(),
//! ])).then("say ready");
//! ```

use crate::condition::Condition;

// ── WhenBuilder ───────────────────────────────────────────────────────────────

/// Builder returned by [`when`]. Call [`then`](WhenBuilder::then) to produce commands.
pub struct WhenBuilder {
    cond: Condition,
}

impl WhenBuilder {
    /// Generate `execute if <condition> run <cmd>` command strings.
    ///
    /// Returns a `Vec<String>` because [`Condition::any`] expands into one
    /// command per sub-condition. For all other condition types exactly one
    /// command is returned.
    ///
    /// The return type implements [`IntoCommands`](crate::components::mc_function::IntoCommands)
    /// so it can be used directly inside [`mcfunction!`](crate::mcfunction):
    ///
    /// ```rust,ignore
    /// let cmds = mcfunction![
    ///     when(MANA.of("@s").gte(25)).then("say enough");
    /// ];
    /// ```
    pub fn then(self, cmd: impl std::fmt::Display) -> Vec<String> {
        self.cond.execute_commands(false, &cmd.to_string())
    }
}

// ── UnlessBuilder ─────────────────────────────────────────────────────────────

/// Builder returned by [`unless`]. Call [`then`](UnlessBuilder::then) to produce commands.
pub struct UnlessBuilder {
    cond: Condition,
}

impl UnlessBuilder {
    /// Generate `execute unless <condition> run <cmd>` command strings.
    pub fn then(self, cmd: impl std::fmt::Display) -> Vec<String> {
        self.cond.execute_commands(true, &cmd.to_string())
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Begin an `execute if <condition> run …` chain.
///
/// ```rust,ignore
/// let cmds = when(MANA.of("@s").gte(25)).then("say enough mana");
/// ```
pub fn when(cond: Condition) -> WhenBuilder {
    WhenBuilder { cond }
}

/// Begin an `execute unless <condition> run …` chain.
///
/// ```rust,ignore
/// let cmds = unless(CASTING.of("@s").is_true()).then("say not casting");
/// ```
pub fn unless(cond: Condition) -> UnlessBuilder {
    UnlessBuilder { cond }
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

    #[test]
    fn when_score() {
        let cmds = when(MANA.of("@s").gte(25)).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute if score @s mana matches 25.. run say ok"]
        );
    }

    #[test]
    fn unless_flag() {
        let cmds = unless(CASTING.of("@s").is_true()).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute unless score @s casting matches 1 run say ok"]
        );
    }

    #[test]
    fn when_all() {
        let cmds = when(Condition::all([
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            CASTING.of("@s").is_false(),
        ]))
        .then("say ready to cast");
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
        .then("say ok");
        assert_eq!(cmds.len(), 2, "Any should expand to two commands");
    }

    #[test]
    fn unless_any_de_morgan() {
        let cmds = unless(Condition::any([
            CASTING.of("@s").is_true(),
            CASTING.of("@s").is_false(),
        ]))
        .then("say ok");
        assert_eq!(cmds.len(), 1, "NOT(a OR b) chains into one command");
        assert!(cmds[0].contains("unless"), "got: {}", cmds[0]);
    }

    #[test]
    fn when_predicate() {
        let cmds = when(Condition::predicate("my_pack:can_cast")).then("say ok");
        assert_eq!(
            cmds,
            vec!["execute if predicate my_pack:can_cast run say ok"]
        );
    }

    #[test]
    fn when_entity() {
        let cmds = when(Condition::entity("@s[tag=ready]")).then("say ok");
        assert_eq!(cmds, vec!["execute if entity @s[tag=ready] run say ok"]);
    }

    #[test]
    fn nested_not() {
        let cmds = when(!(MANA.of("@s").gte(25))).then("say low mana");
        assert_eq!(
            cmds,
            vec!["execute unless score @s mana matches 25.. run say low mana"]
        );
    }

    #[test]
    fn when_cooldown_ready() {
        let cmds = when(DASH.ready("@s")).then("say dash ready");
        assert_eq!(
            cmds,
            vec!["execute if score @s dash matches 0 run say dash ready"]
        );
    }

    #[test]
    fn all_conditions_snapshot() {
        // Integration-style: state + conditions + execute
        let cond = Condition::all([
            MANA.of("@s").gte(25),
            DASH.ready("@s"),
            CASTING.of("@s").is_false(),
        ]);
        let cmds = when(cond).then("say cast");
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            "execute if score @s mana matches 25.. if score @s dash matches 0 if score @s casting matches 0 run say cast"
        );
    }

    #[test]
    fn all_macro_sugar() {
        let cmds = when(all![MANA.of("@s").gte(25), CASTING.of("@s").is_false(),]).then("say ok");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("if score @s mana"), "got: {}", cmds[0]);
        assert!(cmds[0].contains("if score @s casting"), "got: {}", cmds[0]);
    }

    #[test]
    fn any_macro_sugar() {
        let cmds = when(any![MANA.of("@s").gte(25), MANA.of("@s").gte(50),]).then("say ok");
        assert_eq!(cmds.len(), 2, "any! should expand to 2 commands");
    }

    #[test]
    fn nested_any_in_all_via_macros() {
        let cmds = when(all![
            MANA.of("@s").gte(25),
            any![CASTING.of("@s").is_false(), DASH.ready("@s"),],
        ])
        .then("say ready");
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
            when(MANA2.of("@s").gte(25)).then("say enough mana");
        ];
        assert_eq!(cmds[0], "scoreboard objectives add mana2 dummy");
        assert!(
            cmds[1].contains("if score @s mana2 matches 25.."),
            "got: {}",
            cmds[1]
        );
    }
}
