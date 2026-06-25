//! # Typed Command IR
//!
//! A minimal typed intermediate representation for Minecraft commands.
//! Every variant renders to an identical string as the existing manual builders —
//! IR is the intermediate, String is always the final output.
//!
//! # Adding new command types to the IR
//!
//! 1. Add a variant to [`Cmd`] that captures all required fields as owned values.
//! 2. Add a render arm in [`Cmd::render`] that produces the exact Minecraft syntax.
//!    For version-specific rendering, match on `ctx.mc_version.minor` (or major for 26.x).
//! 3. Add a test in the `tests` module proving the IR output matches the existing
//!    string-builder output (copy-paste from the relevant `ScoreVar`/`Flag`/etc. test).
//! 4. Optionally update the relevant builder method to construct a `Cmd` internally
//!    and call `.into()` — this keeps the public `-> String` signature unchanged.

use crate::McVersion;

// ── RenderContext ─────────────────────────────────────────────────────────────

/// Context passed to [`Cmd::render`] to allow version-specific command generation.
///
/// Currently all variants render identically regardless of version.
/// When version-specific syntax diverges, match on `ctx.mc_version` inside render arms.
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

    /// Render context for the latest supported Minecraft version (1.21.4).
    pub fn latest() -> Self {
        Self::for_version(1, 21, 4)
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    /// Passthrough for any string not yet migrated to the typed IR.
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

    /// `execute <conditions…> run <cmd>`
    Execute {
        conditions: Vec<String>,
        run: Box<Cmd>,
    },

    /// `# <text>` — a comment line (not a real Minecraft command, but emitted in .mcfunction files).
    Comment(String),
}

impl Cmd {
    /// Render this command to its Minecraft string representation.
    ///
    /// Currently version-agnostic — all variants render identically regardless of `ctx`.
    /// Future: match on `ctx.mc_version.minor` (or `.major` for 1.26+) to produce
    /// version-specific syntax where Minecraft changed its command grammar.
    #[allow(clippy::only_used_in_recursion)] // ctx is intentionally forwarded for future version-specific rendering
    pub fn render(&self, ctx: &RenderContext) -> String {
        match self {
            Self::Raw(s) => s.clone(),

            Self::Function(id) => format!("function {id}"),

            Self::ScoreDefine {
                objective,
                criterion,
            } => {
                format!("scoreboard objectives add {objective} {criterion}")
            }

            Self::ScorePlayers(op) => op.render(),

            Self::Execute { conditions, run } => {
                let cond_str = conditions.join(" ");
                let run_str = run.render(ctx);
                format!("execute {cond_str} run {run_str}")
            }

            Self::Comment(text) => format!("# {text}"),
        }
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
            conditions: vec!["if score @s mana matches 25..".into()],
            run: Box::new(Cmd::Raw("say ok".into())),
        };
        assert_eq!(
            render(cmd),
            "execute if score @s mana matches 25.. run say ok"
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
