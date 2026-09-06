use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Stable output mode shared by agent-facing CLI commands.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// Human-readable terminal output.
    #[default]
    Human,
    /// Schema-versioned JSON written to stdout.
    Json,
}

impl OutputFormat {
    pub fn is_json(self) -> bool {
        self == Self::Json
    }
}
