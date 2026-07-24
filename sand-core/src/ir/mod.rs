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
//! 3. Declare any real version requirement in the node's capability check.
//! 4. Add a parity test proving the IR output matches the existing
//!    string-builder output (copy-paste from the relevant `ScoreVar`/`Flag`/etc. test).
//! 5. Add a diagnostic test for both the supported and unsupported profile.
//! 6. Update the public builder to construct the typed node internally while
//!    retaining its existing authoring API and output.

use crate::McVersion;

pub use sand_commands::{ConditionIr, ExecuteCapability, ExecuteOp, ExecuteStoreTarget};

// ── RenderContext ─────────────────────────────────────────────────────────────

/// Context passed to [`Cmd::render`] to allow version-specific command generation.
///
/// Execute and condition capability checks use this version before rendering.
pub struct RenderContext {
    pub mc_version: McVersion,
}

impl RenderContext {
    /// Construct a render context for a specific Minecraft version.
    pub fn for_version(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            mc_version: McVersion::new(major, minor, patch),
        }
    }

    /// Render context for the latest supported Minecraft version (26.2).
    pub fn latest() -> Self {
        Self::for_version(26, 2, 0)
    }

    fn command_profile(&self) -> sand_commands::CommandProfile {
        sand_commands::CommandProfile::new(self.mc_version.to_string(), false)
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

    /// `scoreboard objectives add <objective> <criterion>`
    ScoreDefine {
        objective: String,
        criterion: String,
    },

    /// A `scoreboard players` sub-command.
    ScorePlayers(ScorePlayersOp),

    /// `execute <operations…> run <cmd>`, retaining ordered typed operations.
    Execute {
        operations: Vec<ExecuteOp>,
        run: Box<Cmd>,
    },

    /// `# <text>` — a comment line (not a real Minecraft command, but emitted in .mcfunction files).
    Comment(String),
}

impl Cmd {
    /// Render this command after typed version validation.
    pub fn try_render(&self, ctx: &RenderContext) -> sand_commands::CommandResult<String> {
        let profile = ctx.command_profile();
        self.try_render_with_profile(&profile)
    }

    fn try_render_with_profile(
        &self,
        profile: &sand_commands::CommandProfile,
    ) -> sand_commands::CommandResult<String> {
        let rendered = match self {
            Self::Raw(s) => s.clone(),

            Self::Function(id) => format!("function {id}"),

            Self::ScoreDefine {
                objective,
                criterion,
            } => {
                format!("scoreboard objectives add {objective} {criterion}")
            }

            Self::ScorePlayers(op) => op.render(),

            Self::Execute { operations, run } => {
                if operations.is_empty() {
                    return Err(sand_commands::CommandError::new(
                        "Execute",
                        "operations",
                        "execute chains require at least one operation",
                    )
                    .with_code("SAND-COMMAND-EXECUTE-EMPTY"));
                }
                for (index, operation) in operations.iter().enumerate() {
                    operation.validate_version(index, profile)?;
                }
                let operation_text = operations
                    .iter()
                    .map(ExecuteOp::render)
                    .collect::<Vec<_>>()
                    .join(" ");
                let run_text = run.try_render_with_profile(profile)?;
                let line = format!("execute {operation_text} run {run_text}");
                sand_commands::execute_ir::register_line(&line, operations);
                line
            }

            Self::Comment(text) => format!("# {text}"),
        };
        Ok(rendered)
    }

    /// Compatibility renderer using the supplied version context.
    ///
    /// Invalid typed IR panics here; exporters and diagnostic-aware tooling
    /// should use [`Cmd::try_render`].
    pub fn render(&self, ctx: &RenderContext) -> String {
        self.try_render(ctx)
            .expect("typed command IR must validate before infallible rendering")
    }
}

impl From<Cmd> for String {
    fn from(cmd: Cmd) -> String {
        cmd.render(&RenderContext::latest())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ScoreVar;

    fn render(cmd: Cmd) -> String {
        cmd.render(&RenderContext::latest())
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
        .try_render(&RenderContext::latest())
        .unwrap_err();
        assert_eq!(error.code, "SAND-COMMAND-EXECUTE-EMPTY");
        assert_eq!(error.field, "operations");
    }

    #[test]
    fn item_condition_has_real_version_gate() {
        let command = Cmd::Execute {
            operations: vec![ExecuteOp::If(ConditionIr::ItemsEntity {
                target: sand_commands::Selector::self_(),
                slot: sand_commands::ItemSlot::MainHand,
                item: "minecraft:diamond".into(),
            })],
            run: Box::new(Cmd::Raw("say found".into())),
        };
        assert!(
            command
                .try_render(&RenderContext::for_version(1, 20, 5))
                .is_ok()
        );
        let error = command
            .try_render(&RenderContext::for_version(1, 20, 4))
            .unwrap_err();
        assert_eq!(error.code, "SAND-COMMAND-VERSION");
        assert!(error.message.contains("ExecuteItemCondition"), "{error}");
        assert!(error.context.contains("Execute operation 0"), "{error}");
    }

    #[test]
    fn raw_condition_is_opaque_to_version_validation() {
        let command = Cmd::Execute {
            operations: vec![ExecuteOp::If(ConditionIr::Raw(
                "items entity @s weapon.mainhand minecraft:diamond".into(),
            ))],
            run: Box::new(Cmd::Raw("say user-owned".into())),
        };
        assert_eq!(
            command
                .try_render(&RenderContext::for_version(1, 20, 4))
                .unwrap(),
            "execute if items entity @s weapon.mainhand minecraft:diamond run say user-owned"
        );
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
