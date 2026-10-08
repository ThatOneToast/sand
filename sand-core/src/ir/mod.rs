//! # Typed Command IR
//!
//! The IR is the internal bridge between Sand's public typed builders and final
//! `.mcfunction` text. It is not a second command language datapack authors are
//! expected to learn. Structured nodes remain typed through validation; strings
//! are created only at the rendering/export boundary.
//!
//! # Adding new command types to the IR
//!
//! 1. Add a variant to [`Cmd`] that captures all required fields as owned values.
//! 2. Add a render arm in [`Cmd::render`] that produces the exact Minecraft syntax.
//! 3. Add a parity test proving the IR output matches the existing
//!    string-builder output (copy-paste from the relevant `ScoreVar`/`Flag`/etc. test).
//! 4. Add a diagnostic test for invalid typed values.
//! 5. Update the public builder to construct the typed node internally while
//!    retaining its existing authoring API and output.

pub use sand_commands::{ConditionIr, ExecuteOp, ExecuteStoreTarget};
mod numeric;
#[doc(hidden)]
pub use numeric::NumericWrite;

/// An ordered sequence of authored operations awaiting compiler lowering.
///
/// Sand function macros collect these values at build time. They describe
/// Minecraft runtime work; constructing or combining them does not execute it.
/// Keep them intact when composing gameplay helpers so validation retains the
/// structured operations before the exporter emits command text.
#[derive(Debug, Clone, Default)]
#[must_use = "authored actions must be returned or collected into a Sand function"]
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::command::Actions",
    aliases = ["sand::cmd::Actions", "sand::prelude::Actions", "sand::prelude::cmd::Actions"],
    module = "sand::command",
    summary = "An ordered sequence of authored operations awaiting compiler lowering.",
    context = "Function macros collect these build-time values into Minecraft runtime bodies. Gameplay helpers can return Actions without exposing command strings or compiler IR variants.",
    minecraft = "The exporter validates and lowers each operation in authored order before emitting mcfunction resources.",
    use_when = ["Returning composed gameplay operations from a Rust helper"],
    avoid_when = ["Representing an operation that has already executed in Minecraft"],
    example = "use sand::command::Actions; fn empty() -> Actions { Actions::default() }",
)]
pub struct Actions(pub(crate) Vec<Cmd>);

impl sand_components::function::FunctionBody for Actions {}

impl<A: crate::IntoCommands> Extend<A> for Actions {
    fn extend<T: IntoIterator<Item = A>>(&mut self, actions: T) {
        for action in actions {
            self.0.extend(action.into_commands().0);
        }
    }
}

impl IntoIterator for Actions {
    type Item = Actions;
    type IntoIter = std::vec::IntoIter<Actions>;

    fn into_iter(self) -> Self::IntoIter {
        self.0
            .into_iter()
            .map(|node| Actions(vec![node]))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl FromIterator<Actions> for Actions {
    fn from_iter<T: IntoIterator<Item = Actions>>(actions: T) -> Self {
        let mut body = Self::default();
        body.extend(actions);
        body
    }
}

impl Actions {
    pub(crate) fn has_macro_lines(&self) -> bool {
        self.0.iter().any(Cmd::has_macro_line)
    }

    pub(crate) fn may_return_from_frame(&self) -> bool {
        self.0.iter().any(Cmd::may_return_from_frame)
    }

    pub(crate) fn identity(&self) -> String {
        // Retain variant distinctions: raw and typed nodes may render the same
        // text while requiring different validation. This identity contains no
        // process-local handles or source provenance.
        format!("{:?}", self.0)
    }

    // Preserve the canonical component error (including resource ownership) at
    // this export boundary, as DatapackComponent::try_content requires.
    #[allow(clippy::result_large_err)]
    pub(crate) fn lower(
        &self,
        location: &sand_components::ResourceLocation,
    ) -> sand_components::error::Result<Vec<String>> {
        let mut output = Vec::new();
        for (index, command) in self.0.iter().enumerate() {
            let rendered = command
                .lower_for_export(location)
                .and_then(|commands| {
                    commands
                        .into_iter()
                        .map(|command| command.try_render())
                        .collect::<sand_commands::CommandResult<Vec<_>>>()
                })
                .map_err(|error| sand_components::SandError::ComponentValidation {
                    location: location.clone(),
                    kind: "function".into(),
                    field: format!("actions[{index}].{}", error.field),
                    message: error.to_string(),
                })?;
            output.extend(rendered);
        }
        Ok(output)
    }
}

// ── ScoreOpKind ───────────────────────────────────────────────────────────────

/// Vanilla scoreboard player operation symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreOpKind {
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

impl ScoreOpKind {
    /// Render as the vanilla operator string used by `scoreboard players operation`.
    pub fn as_str(self) -> &'static str {
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

// ── ScorePlayersOp ────────────────────────────────────────────────────────────

/// The operation sub-variant of a `scoreboard players` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScorePlayersOp {
    /// `scoreboard players set <selector> <objective> <value>`
    Set {
        selector: String,
        objective: String,
        value: i32,
    },
    /// `scoreboard players add <selector> <objective> <amount>`
    Add {
        selector: String,
        objective: String,
        amount: i32,
    },
    /// `scoreboard players remove <selector> <objective> <amount>`
    Remove {
        selector: String,
        objective: String,
        amount: i32,
    },
    /// `scoreboard players reset <selector> <objective>`
    Reset { selector: String, objective: String },
    /// `scoreboard players operation <target> <target_obj> <op> <source> <source_obj>`
    Operation {
        target: String,
        target_obj: String,
        op: ScoreOpKind,
        source: String,
        source_obj: String,
    },
}

impl ScorePlayersOp {
    fn render(&self) -> String {
        match self {
            Self::Set {
                selector,
                objective,
                value,
            } => {
                format!("scoreboard players set {selector} {objective} {value}")
            }
            Self::Add {
                selector,
                objective,
                amount,
            } => {
                format!("scoreboard players add {selector} {objective} {amount}")
            }
            Self::Remove {
                selector,
                objective,
                amount,
            } => {
                format!("scoreboard players remove {selector} {objective} {amount}")
            }
            Self::Reset {
                selector,
                objective,
            } => {
                format!("scoreboard players reset {selector} {objective}")
            }
            Self::Operation {
                target,
                target_obj,
                op,
                source,
                source_obj,
            } => {
                format!(
                    "scoreboard players operation {target} {target_obj} {} {source} {source_obj}",
                    op.as_str()
                )
            }
        }
    }
}

// ── Cmd ───────────────────────────────────────────────────────────────────────

/// A typed Minecraft command IR node.
///
/// Use [`Cmd::render`] to produce the final command string.
/// Use `String::from(cmd)` or `cmd.into::<String>()` to render with the latest version context.
#[derive(Debug, Clone)]
pub enum Cmd {
    /// Opaque passthrough. Sand does not inspect, rewrite, or version-check it.
    Raw(String),

    /// `function <id>`
    Function(String),

    /// An owned anonymous function whose resource is registered during export.
    /// Keeping the body here makes cached actions independent of registry lifetime.
    AnonymousFunction { prefix: String, body: Actions },

    /// Retain compiler-managed scoreboard requirements alongside a command.
    /// Operands replay their owned setup during each export, including cached bodies.
    WithScoreOperands {
        operands: Vec<crate::state::score::ScoreOperand>,
        run: Box<Cmd>,
    },

    /// Return immediately with the result of one nested command.
    ReturnRun(Box<Cmd>),

    /// `scoreboard objectives add <objective> <criterion>`
    ScoreDefine {
        objective: String,
        criterion: String,
    },

    /// A `scoreboard players` sub-command.
    ScorePlayers(ScorePlayersOp),

    /// Standalone typed `data get|remove|modify|merge` command IR.
    Data(sand_commands::DataCommand),

    /// `execute <operations…> run <cmd>`, retaining ordered typed operations.
    Execute {
        operations: Vec<ExecuteOp>,
        run: Box<Cmd>,
    },

    /// A gameplay numeric assignment awaiting destination-aware lowering.
    #[doc(hidden)]
    NumericWrite(Box<NumericWrite>),

    /// `# <text>` — a comment line (not a real Minecraft command, but emitted in .mcfunction files).
    Comment(String),
}

fn nested_run_error(mut error: sand_commands::CommandError) -> sand_commands::CommandError {
    error.field = format!("run.{}", error.field);
    error
}

fn raw_command_may_return(line: &str) -> bool {
    let line = line.trim_start();
    let mut command = line.strip_prefix('$').unwrap_or(line);
    loop {
        command = command.strip_prefix("minecraft:").unwrap_or(command);
        let verb = command.split_whitespace().next().unwrap_or("");
        match verb {
            "return" => return true,
            "execute" => {
                let Some(inner) = sand_commands::render::collected_execute_command(command) else {
                    // Unknown or macro-generated operation grammar cannot prove
                    // that the scoped body continues to its cleanup command.
                    return true;
                };
                command = inner;
            }
            _ => return verb.contains("$("),
        }
    }
}

impl Cmd {
    fn lower_for_export(
        &self,
        owner: &sand_components::ResourceLocation,
    ) -> sand_commands::CommandResult<Vec<Self>> {
        Ok(match self {
            Self::NumericWrite(write) => return write.lower(owner),
            Self::WithScoreOperands { operands, run } => {
                for operand in operands {
                    operand.register_owned_setup();
                }
                return run.lower_for_export(owner);
            }
            Self::Execute { operations, run } => vec![Self::Execute {
                operations: operations.clone(),
                run: Box::new(
                    run.lower_single_for_export(owner)
                        .map_err(nested_run_error)?,
                ),
            }],
            Self::ReturnRun(run) => vec![Self::ReturnRun(Box::new(
                run.lower_single_for_export(owner)
                    .map_err(nested_run_error)?,
            ))],
            command => vec![command.clone()],
        })
    }

    fn lower_single_for_export(
        &self,
        owner: &sand_components::ResourceLocation,
    ) -> sand_commands::CommandResult<Self> {
        let mut commands = self.lower_for_export(owner)?;
        if commands.len() == 1 {
            Ok(commands.remove(0))
        } else {
            Ok(Self::AnonymousFunction {
                prefix: "sand/action_sequence".into(),
                body: Actions(commands),
            })
        }
    }

    fn has_macro_line(&self) -> bool {
        match self {
            Self::Raw(text) => text.lines().any(|line| line.trim_start().starts_with('$')),
            Self::Execute { run, .. }
            | Self::ReturnRun(run)
            | Self::WithScoreOperands { run, .. } => run.has_macro_line(),
            _ => false,
        }
    }

    fn may_return_from_frame(&self) -> bool {
        match self {
            Self::ReturnRun(_) => true,
            Self::Execute { run, .. } | Self::WithScoreOperands { run, .. } => {
                run.may_return_from_frame()
            }
            // Inspect command positions through the canonical execute parser;
            // quoted arguments and ordinary words such as `say return` are data.
            Self::Raw(text) => text.lines().any(raw_command_may_return),
            _ => false,
        }
    }

    /// Render this command after typed validation against Sand's 26+ command baseline.
    pub fn try_render(&self) -> sand_commands::CommandResult<String> {
        let rendered = match self {
            Self::Raw(s) => s.clone(),

            Self::Function(id) => format!("function {id}"),
            Self::AnonymousFunction { prefix, body } => {
                if body.has_macro_lines() {
                    return Err(sand_commands::CommandError::new(
                        "anonymous function",
                        "macro arguments",
                        "macro lines cannot move into a helper without an argument source; keep them in the argument-bearing function or call an explicit function with FunctionMacroArgs::call_with; scoped macro bodies cannot return early",
                    ));
                }
                let path = crate::function::register_dyn_fn_dedup(prefix, body.clone());
                format!("function {}:{path}", crate::function::SAND_LOCAL_NS)
            }
            Self::WithScoreOperands { operands, run } => {
                for operand in operands {
                    operand.register_owned_setup();
                }
                run.try_render()?
            }
            Self::ReturnRun(command) => {
                let rendered = command.try_render().map_err(|mut error| {
                    error.field = format!("run.{}", error.field);
                    error
                })?;
                format!("return run {rendered}")
            }

            Self::ScoreDefine {
                objective,
                criterion,
            } => {
                format!("scoreboard objectives add {objective} {criterion}")
            }

            Self::ScorePlayers(op) => op.render(),

            Self::Data(command) => {
                command.try_render(&sand_commands::CommandProfile::unprofiled())?
            }

            Self::Execute { operations, run } => {
                if operations.is_empty() {
                    return Err(sand_commands::CommandError::new(
                        "Execute",
                        "operations",
                        "execute chains require at least one operation",
                    )
                    .with_code("SAND-COMMAND-EXECUTE-EMPTY"));
                }
                let operation_text = operations
                    .iter()
                    .map(ExecuteOp::render)
                    .collect::<Vec<_>>()
                    .join(" ");
                let run_text = run.try_render().map_err(|mut error| {
                    error.field = format!("run.{}", error.field);
                    error
                })?;
                format!("execute {operation_text} run {run_text}")
            }

            Self::NumericWrite(_) => {
                return Err(sand_commands::CommandError::new(
                    "numeric assignment",
                    "value",
                    "numeric actions require function export lowering",
                ));
            }
            Self::Comment(text) => format!("# {text}"),
        };
        Ok(rendered)
    }

    /// Infallible convenience renderer for already-valid typed IR.
    pub fn render(&self) -> String {
        self.try_render()
            .expect("typed command IR must validate before infallible rendering")
    }
}

impl From<Cmd> for String {
    fn from(cmd: Cmd) -> String {
        cmd.render()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ScoreVar;

    fn render(cmd: Cmd) -> String {
        cmd.render()
    }

    #[test]
    fn raw_return_detection_uses_command_positions() {
        for line in [
            "$say return $(name)",
            "$minecraft:say return $(name)",
            "$execute if score @s return matches 1 run say return $(name)",
            r#"$tellraw @s {"text":"return $(name)"}"#,
            "say execute run return 0",
        ] {
            assert!(!raw_command_may_return(line), "{line}");
        }
        for line in [
            "$minecraft:return $(result)",
            "$minecraft:execute as @s run return $(result)",
            "$execute as @s run minecraft:return $(result)",
            "return fail",
            "$return $(result)",
            "$execute as @s run return $(result)",
            "execute as @s run execute at @s run return 0",
            "$execute $(operations) run say unknown",
            "$$(command)",
        ] {
            assert!(raw_command_may_return(line), "{line}");
        }
    }

    #[test]
    fn score_define_render() {
        let cmd = Cmd::ScoreDefine {
            objective: "mana".into(),
            criterion: "dummy".into(),
        };
        assert_eq!(render(cmd), "scoreboard objectives add mana dummy");
    }

    #[test]
    fn score_players_set_render() {
        let cmd = Cmd::ScorePlayers(ScorePlayersOp::Set {
            selector: "@s".into(),
            objective: "mana".into(),
            value: 100,
        });
        assert_eq!(render(cmd), "scoreboard players set @s mana 100");
    }

    #[test]
    fn score_players_add_render() {
        let cmd = Cmd::ScorePlayers(ScorePlayersOp::Add {
            selector: "@s".into(),
            objective: "mana".into(),
            amount: 5,
        });
        assert_eq!(render(cmd), "scoreboard players add @s mana 5");
    }

    #[test]
    fn score_players_remove_render() {
        let cmd = Cmd::ScorePlayers(ScorePlayersOp::Remove {
            selector: "@s".into(),
            objective: "mana".into(),
            amount: 10,
        });
        assert_eq!(render(cmd), "scoreboard players remove @s mana 10");
    }

    #[test]
    fn score_players_reset_render() {
        let cmd = Cmd::ScorePlayers(ScorePlayersOp::Reset {
            selector: "@s".into(),
            objective: "mana".into(),
        });
        assert_eq!(render(cmd), "scoreboard players reset @s mana");
    }

    #[test]
    fn score_players_operation_render() {
        let cmd = Cmd::ScorePlayers(ScorePlayersOp::Operation {
            target: "@s".into(),
            target_obj: "mana".into(),
            op: ScoreOpKind::Assign,
            source: "@p".into(),
            source_obj: "other".into(),
        });
        assert_eq!(
            render(cmd),
            "scoreboard players operation @s mana = @p other"
        );
    }

    #[test]
    fn execute_render() {
        let cmd = Cmd::Execute {
            operations: vec![ExecuteOp::If(ConditionIr::ScoreMatches {
                holder: sand_commands::ScoreHolder::self_(),
                objective: "mana".into(),
                range: "25..".into(),
            })],
            run: Box::new(Cmd::Raw("say ok".into())),
        };
        assert_eq!(
            render(cmd),
            "execute if score @s mana matches 25.. run say ok"
        );
    }

    #[test]
    fn nested_execute_keeps_both_chains_typed() {
        let command = Cmd::Execute {
            operations: vec![ExecuteOp::As(sand_commands::Selector::all_players())],
            run: Box::new(Cmd::Execute {
                operations: vec![ExecuteOp::At(sand_commands::Selector::self_())],
                run: Box::new(Cmd::Function("demo:tick".into())),
            }),
        };
        assert_eq!(
            render(command),
            "execute as @a run execute at @s run function demo:tick"
        );
    }

    #[test]
    fn empty_execute_chain_has_structured_diagnostic() {
        let error = Cmd::Execute {
            operations: vec![],
            run: Box::new(Cmd::Raw("say no".into())),
        }
        .try_render()
        .unwrap_err();
        assert_eq!(error.code, "SAND-COMMAND-EXECUTE-EMPTY");
        assert_eq!(error.field, "operations");
    }

    #[test]
    fn comment_render() {
        let cmd = Cmd::Comment("my comment".into());
        assert_eq!(render(cmd), "# my comment");
    }

    #[test]
    fn function_render() {
        let cmd = Cmd::Function("ns:path".into());
        assert_eq!(render(cmd), "function ns:path");
    }

    #[test]
    fn raw_render() {
        let cmd = Cmd::Raw("say hello".into());
        assert_eq!(render(cmd), "say hello");
    }

    #[test]
    fn from_cmd_for_string() {
        let cmd = Cmd::Raw("say hello".into());
        let s: String = cmd.into();
        assert_eq!(s, "say hello");
    }

    #[test]
    fn ir_matches_scorevar_builder() {
        static MANA: ScoreVar<i32> = ScoreVar::new("mana");

        // define
        let ir_define = render(Cmd::ScoreDefine {
            objective: "mana".into(),
            criterion: "dummy".into(),
        });
        assert_eq!(ir_define, MANA.define());

        // set
        let ir_set = render(Cmd::ScorePlayers(ScorePlayersOp::Set {
            selector: "@s".into(),
            objective: "mana".into(),
            value: 100,
        }));
        assert_eq!(ir_set, MANA.set("@s", 100));

        // add
        let ir_add = render(Cmd::ScorePlayers(ScorePlayersOp::Add {
            selector: "@s".into(),
            objective: "mana".into(),
            amount: 5,
        }));
        assert_eq!(ir_add, MANA.add("@s", 5));

        // remove
        let ir_remove = render(Cmd::ScorePlayers(ScorePlayersOp::Remove {
            selector: "@s".into(),
            objective: "mana".into(),
            amount: 10,
        }));
        assert_eq!(ir_remove, MANA.remove("@s", 10));

        // reset
        let ir_reset = render(Cmd::ScorePlayers(ScorePlayersOp::Reset {
            selector: "@s".into(),
            objective: "mana".into(),
        }));
        assert_eq!(ir_reset, MANA.reset("@s"));
    }

    #[test]
    fn all_score_op_kinds_render() {
        let ops = [
            (ScoreOpKind::Assign, "="),
            (ScoreOpKind::Add, "+="),
            (ScoreOpKind::Sub, "-="),
            (ScoreOpKind::Mul, "*="),
            (ScoreOpKind::Div, "/="),
            (ScoreOpKind::Mod, "%="),
            (ScoreOpKind::Min, "<"),
            (ScoreOpKind::Max, ">"),
            (ScoreOpKind::Swap, "><"),
        ];
        for (kind, expected) in ops {
            assert_eq!(kind.as_str(), expected);
        }
    }
}

#[cfg(test)]
mod nested_validation_tests {
    use super::*;

    #[test]
    fn validation_retains_every_nested_run_path() {
        let body = Actions(vec![Cmd::ReturnRun(Box::new(Cmd::Execute {
            operations: vec![ExecuteOp::As(sand_commands::Selector::self_())],
            run: Box::new(Cmd::Execute {
                operations: vec![],
                run: Box::new(Cmd::Raw("say invalid".into())),
            }),
        }))]);
        let error = body.lower(&"test:nested".parse().unwrap()).unwrap_err();
        assert!(error.to_string().contains("actions[0].run.run.operations"));
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    pub(crate) fn emitted(actions: impl crate::IntoCommands) -> Vec<String> {
        actions
            .into_commands()
            .lower(&"test:body".parse().unwrap())
            .unwrap()
    }

    pub(crate) fn drain_emitted() -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        loop {
            let pending = crate::function::drain_dyn_fns();
            if pending.is_empty() {
                break;
            }
            result.extend(
                pending
                    .into_iter()
                    .map(|(path, body)| (path, emitted(body))),
            );
        }
        result
    }
}
