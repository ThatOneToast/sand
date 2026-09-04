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
//! use sand_commands::Target;
//!
//! static MANA: ScoreVar<i32> = ScoreVar::new("mana");
//!
//! // Single command
//! let cmds: Vec<String> = TypedExecute::as_players()
//!     .at(Target::self_())
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ConditionedExecute",
    aliases = ["sand::cmd::ConditionedExecute", "sand::prelude::ConditionedExecute", "sand::prelude::cmd::ConditionedExecute"],
    module = "sand::command",
    summary = "An execute chain paired with a typed [`Condition`].",
    context = "An execute chain paired with a typed [`Condition`]. Created by [`ExecuteExt::when`] or [`ExecuteExt::unless`]. Call [`run`](ConditionedExecute::run) to finalize into `Vec<String>`.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ConditionedExecute;",
)]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ConditionedExecute::and_when",
        aliases = ["sand::cmd::ConditionedExecute::and_when", "sand::prelude::ConditionedExecute::and_when", "sand::prelude::cmd::ConditionedExecute::and_when"],
        module = "sand::command",
        kind = "method",
        summary = "Add another AND condition (Cartesian-product expansion).",
        context = "Add another AND condition (Cartesian-product expansion). This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cond = "`cond` provides the condition that gates the operation used to add another AND condition (Cartesian-product expansion)."),
        returns = "The `ConditionedExecute` value with the documented change applied to add another AND condition (Cartesian-product expansion).",
        example = "use sand::prelude::*;\n\nfn demonstrate(conditioned_execute_value: sand::command::ConditionedExecute, cond: sand::condition::Condition)  {\n    let updated_conditioned_execute = conditioned_execute_value.and_when(cond);\n}",
    )]
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
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ConditionedExecute::run",
        aliases = ["sand::cmd::ConditionedExecute::run", "sand::prelude::ConditionedExecute::run", "sand::prelude::cmd::ConditionedExecute::run"],
        module = "sand::command",
        kind = "method",
        summary = "Finalize the execute chain. Returns one command per expanded plan.  A simple score condition gives one string; `any![...]` gives N strings.",
        context = "Finalize the execute chain. Returns one command per expanded plan.  A simple score condition gives one string; `any![...]` gives N strings. Accepts any `Display` value — raw `&str`, owned `String`, or any command builder.",
        minecraft = "Returns one command per expanded plan.  A simple score condition gives one string; `any![...]` gives N strings.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cmd = "`cmd` is used to finalize the execute chain. Returns one command per expanded plan. A simple score condition gives one string; `any![...]` gives N strings."),
        returns = "Returns one command per expanded plan.  A simple score condition gives one string; `any![...]` gives N strings.",
        example = "use std::fmt;\nuse sand::prelude::*;\n\nfn demonstrate(conditioned_execute_value: sand::command::ConditionedExecute, cmd: impl fmt::Display)  {\n    let values = conditioned_execute_value.run(cmd);\n}",
    )]
    pub fn run(self, cmd: impl fmt::Display) -> Vec<String> {
        let cmd_str = cmd.to_string();
        self.cond
            .to_ir_plans(self.negated)
            .into_iter()
            .map(|clauses| {
                clauses
                    .into_iter()
                    .fold(self.prefix.clone(), |execute, clause| {
                        sand_commands::__private::execute_with_operation(
                            execute,
                            clause.into_operation(),
                        )
                    })
                    .run(&cmd_str)
            })
            .collect()
    }
}

// ── ExecuteExt ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::ExecuteExt",
    aliases = ["sand::cmd::ExecuteExt", "sand::prelude::ExecuteExt", "sand::prelude::cmd::ExecuteExt"],
    module = "sand::command",
    summary = "Extension trait — adds `when` and `unless` to the low-level [`Execute`] builder.",
    context = "Extension trait — adds `when` and `unless` to the low-level [`Execute`] builder. Import this trait to access the typed condition methods.",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::ExecuteExt;",
)]
/// Extension trait — adds `when` and `unless` to the low-level [`Execute`] builder.
///
/// Import this trait to access the typed condition methods.
///
/// ```rust,ignore
/// use sand_core::cmd::ExecuteExt;
/// use sand_commands::Execute;
///
/// let cmds = Execute::new()
///     .as_(Target::players())
///     .when(MANA.of("@s").gte(25))
///     .run("say enough");
/// ```
pub trait ExecuteExt: Sized {
    /// Attach a typed condition — returns a [`ConditionedExecute`] whose
    /// [`run`](ConditionedExecute::run) produces `Vec<String>`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ExecuteExt::when",
        aliases = ["sand::cmd::ExecuteExt::when", "sand::prelude::ExecuteExt::when", "sand::prelude::cmd::ExecuteExt::when"],
        module = "sand::command",
        summary = "Attach a typed condition — returns a [`ConditionedExecute`] whose [`run`](ConditionedExecute::run) produces `Vec<String>`.",
        context = "Attach a typed condition — returns a [`ConditionedExecute`] whose [`run`](ConditionedExecute::run) produces `Vec<String>`. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cond = "`cond` provides the condition that gates the operation used to attach a typed condition — returns a [`ConditionedExecute`] whose [`run`](ConditionedExecute::run) produces `Vec<String>`."),
        returns = "Attach a typed condition — returns a [`ConditionedExecute`] whose [`run`](ConditionedExecute::run) produces `Vec<String>`.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::ExecuteExt>(execute_ext_value: T, cond: sand::condition::Condition)  {\n    let when = execute_ext_value.when(cond);\n}",
    )]
    fn when(self, cond: Condition) -> ConditionedExecute;

    /// Attach a negated typed condition.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::ExecuteExt::unless",
        aliases = ["sand::cmd::ExecuteExt::unless", "sand::prelude::ExecuteExt::unless", "sand::prelude::cmd::ExecuteExt::unless"],
        module = "sand::command",
        summary = "Attach a negated typed condition.",
        context = "Attach a negated typed condition. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        params(cond = "`cond` provides the condition that gates the operation used to attach a negated typed condition."),
        returns = "The `ConditionedExecute` value produced to attach a negated typed condition.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::command::ExecuteExt>(execute_ext_value: T, cond: sand::condition::Condition)  {\n    let unless = execute_ext_value.unless(cond);\n}",
    )]
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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::TypedExecute",
    aliases = ["sand::cmd::TypedExecute", "sand::prelude::TypedExecute", "sand::prelude::cmd::TypedExecute"],
    module = "sand::command",
    summary = "Convenience constructors for common `execute` patterns.",
    context = "Convenience constructors for common `execute` patterns. Each method returns a bare [`Execute`] so you can chain standard sub-commands before calling [`when`](ExecuteExt::when) or terminating with [`Execute::run`].",
    minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
    use_when = ["Constructing Minecraft commands through Sand's typed command model"],
    avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
    example = "use sand::command::TypedExecute;",
)]
/// Convenience constructors for common `execute` patterns.
///
/// Each method returns a bare [`Execute`] so you can chain standard sub-commands
/// before calling [`when`](ExecuteExt::when) or terminating with [`Execute::run`].
pub struct TypedExecute;

impl TypedExecute {
    /// `execute as @a` — run as every player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TypedExecute::as_players",
        aliases = ["sand::cmd::TypedExecute::as_players", "sand::prelude::TypedExecute::as_players", "sand::prelude::cmd::TypedExecute::as_players"],
        module = "sand::command",
        kind = "method",
        summary = "`execute as @a` — run as every player.",
        context = "`execute as @a` — run as every player. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Execute` value produced to emit the documented `execute as @a` — run as every player form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let as_players = sand::command::TypedExecute::as_players();\n}",
    )]
    pub fn as_players() -> Execute {
        Execute::new().as_(Selector::all_players())
    }

    /// `execute as @e` — run as every entity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TypedExecute::as_entities",
        aliases = ["sand::cmd::TypedExecute::as_entities", "sand::prelude::TypedExecute::as_entities", "sand::prelude::cmd::TypedExecute::as_entities"],
        module = "sand::command",
        kind = "method",
        summary = "`execute as @e` — run as every entity.",
        context = "`execute as @e` — run as every entity. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Execute` value produced to emit the documented `execute as @e` — run as every entity form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let as_entities = sand::command::TypedExecute::as_entities();\n}",
    )]
    pub fn as_entities() -> Execute {
        Execute::new().as_(Selector::all_entities())
    }

    /// `execute as @s at @s` — run as self, at self's position.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TypedExecute::as_self_at_self",
        aliases = ["sand::cmd::TypedExecute::as_self_at_self", "sand::prelude::TypedExecute::as_self_at_self", "sand::prelude::cmd::TypedExecute::as_self_at_self"],
        module = "sand::command",
        kind = "method",
        summary = "`execute as @s at @s` — run as self, at self's position.",
        context = "`execute as @s at @s` — run as self, at self's position. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Execute` value produced to emit the documented `execute as @s at @s` — run as self, at self's position form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let as_self_at_self = sand::command::TypedExecute::as_self_at_self();\n}",
    )]
    pub fn as_self_at_self() -> Execute {
        Execute::new().as_(Selector::self_()).at(Selector::self_())
    }

    /// `execute as @a at @s` — run as every player at their own position.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::command::TypedExecute::as_players_at_self",
        aliases = ["sand::cmd::TypedExecute::as_players_at_self", "sand::prelude::TypedExecute::as_players_at_self", "sand::prelude::cmd::TypedExecute::as_players_at_self"],
        module = "sand::command",
        kind = "method",
        summary = "`execute as @a at @s` — run as every player at their own position.",
        context = "`execute as @a at @s` — run as every player at their own position. This handwritten command API complements the generated command catalog with typed selectors, coordinates, execute chains, score holders, NBT, text, and validated command builders.",
        minecraft = "Builders validate domain values and render one or more command lines for the active Minecraft profile; methods explicitly named raw are deliberate advanced escape hatches.",
        use_when = ["Constructing Minecraft commands through Sand's typed command model"],
        avoid_when = ["Passing unvalidated command fragments when a typed builder or validated try_* entry point exists"],
        returns = "The `Execute` value produced to emit the documented `execute as @a at @s` — run as every player at their own position form.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let as_players_at_self = sand::command::TypedExecute::as_players_at_self();\n}",
    )]
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
