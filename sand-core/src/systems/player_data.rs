//! Player-data schema bootstrap (`systems-player-data` feature).
//!
//! Provides a builder API for defining typed per-player data schemas backed
//! by scoreboard objectives and NBT storage.
//!
//! # Design
//!
//! A `PlayerSchema` groups logically related per-player variables under a
//! shared namespace prefix.  It generates bootstrap commands (define all
//! objectives, set defaults) that belong in your load/join functions.
//!
//! # Example
//!
//! ```rust,ignore
//! use sand_core::state::{ScoreVar, Flag, Cooldown, Ticks};
//! use sand_core::systems::player_data::PlayerSchema;
//!
//! static MANA:       ScoreVar<i32> = ScoreVar::new("mana");
//! static HAS_CELLS:  Flag          = Flag::new("has_cells");
//! static DASH_CD:    Cooldown      = Cooldown::new("dash", Ticks::seconds(3));
//!
//! static PLAYER_SCHEMA: PlayerSchema = PlayerSchema::new("my_pack")
//!     .score(&MANA,      100)   // default mana = 100
//!     .flag(&HAS_CELLS,  false) // has_cells starts false
//!     .cooldown(&DASH_CD);      // registers objective only (no default value)
//!
//! // In your load function:
//! // PLAYER_SCHEMA.define_all()  → define every objective
//!
//! // In your join handler:
//! // PLAYER_SCHEMA.init_player("@s")  → set defaults for new player
//! ```

use crate::state::{Cooldown, Flag, ScoreVar};

// ── FieldInit ─────────────────────────────────────────────────────────────────

enum FieldInit {
    Score { obj: String, default: i32 },
    Flag { obj: String, default: bool },
    CooldownObj { obj: String },
}

impl FieldInit {
    fn define_cmd(&self) -> String {
        match self {
            FieldInit::Score { obj, .. }
            | FieldInit::Flag { obj, .. }
            | FieldInit::CooldownObj { obj } => {
                format!("scoreboard objectives add {obj} dummy")
            }
        }
    }

    fn init_cmd(&self, selector: &str) -> Option<String> {
        match self {
            FieldInit::Score { obj, default } => Some(format!(
                "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} {default}"
            )),
            FieldInit::Flag { obj, default } => {
                let val = if *default { 1 } else { 0 };
                Some(format!(
                    "execute unless score {selector} {obj} matches -2147483648.. run scoreboard players set {selector} {obj} {val}"
                ))
            }
            FieldInit::CooldownObj { .. } => None,
        }
    }
}

// ── PlayerSchema ──────────────────────────────────────────────────────────────

/// A collection of per-player variables with defaults, grouped under a pack namespace.
///
/// Build with the chained [`score`](PlayerSchema::score), [`flag`](PlayerSchema::flag),
/// and [`cooldown`](PlayerSchema::cooldown) methods.  Call [`define_all`](PlayerSchema::define_all)
/// in your load function and [`init_player`](PlayerSchema::init_player) in join handlers.
pub struct PlayerSchema {
    #[allow(dead_code)]
    namespace: &'static str,
    fields: Vec<FieldInit>,
}

impl PlayerSchema {
    /// Create an empty schema for `namespace`.
    pub const fn new(namespace: &'static str) -> Self {
        Self {
            namespace,
            fields: Vec::new(),
        }
    }

    /// Register a `ScoreVar` with a default value for new players.
    pub fn score<T>(mut self, var: &ScoreVar<T>, default: i32) -> Self {
        self.fields.push(FieldInit::Score {
            obj: var.objective_name(),
            default,
        });
        self
    }

    /// Register a `Flag` with a default boolean value for new players.
    pub fn flag(mut self, flag: &Flag, default: bool) -> Self {
        self.fields.push(FieldInit::Flag {
            obj: flag.objective_name(),
            default,
        });
        self
    }

    /// Register a `Cooldown` objective (define only; no default value).
    pub fn cooldown(mut self, cd: &Cooldown) -> Self {
        self.fields.push(FieldInit::CooldownObj {
            obj: cd.objective_name(),
        });
        self
    }

    /// Commands to define all objectives (for your load function).
    ///
    /// Duplicates are not deduplicated here — ensure each objective name is unique
    /// across schemas in your pack.
    pub fn define_all(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.define_cmd()).collect()
    }

    /// Commands to initialize a new player's scores to their defaults.
    ///
    /// Each command uses `unless score … matches -2147483648..` so it is a no-op
    /// for players who already have scores (reconnects, etc.).
    pub fn init_player(&self, selector: &str) -> Vec<String> {
        self.fields
            .iter()
            .filter_map(|f| f.init_cmd(selector))
            .collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Cooldown, Flag, ScoreVar, Ticks};

    static MANA: ScoreVar<i32> = ScoreVar::new("mana");
    static HAS_CELLS: Flag = Flag::new("has_cells");
    static DASH: Cooldown = Cooldown::new("dash", Ticks::new(60));

    fn schema() -> PlayerSchema {
        PlayerSchema::new("test_pack")
            .score(&MANA, 100)
            .flag(&HAS_CELLS, false)
            .cooldown(&DASH)
    }

    #[test]
    fn define_all_generates_three_commands() {
        let cmds = schema().define_all();
        assert_eq!(cmds.len(), 3);
        for cmd in &cmds {
            assert!(cmd.starts_with("scoreboard objectives add "), "got: {cmd}");
        }
        assert!(cmds[0].contains("mana"), "score obj: {}", cmds[0]);
        assert!(cmds[1].contains("has_cells"), "flag obj: {}", cmds[1]);
        assert!(cmds[2].contains("dash"), "cd obj: {}", cmds[2]);
    }

    #[test]
    fn init_player_skips_cooldown() {
        let cmds = schema().init_player("@s");
        // Cooldown has no default init command
        assert_eq!(cmds.len(), 2, "only score and flag have defaults");
    }

    #[test]
    fn init_player_score_default() {
        let cmds = schema().init_player("@s");
        assert!(
            cmds[0].contains("unless score @s mana matches -2147483648.."),
            "got: {}",
            cmds[0]
        );
        assert!(cmds[0].contains("set @s mana 100"), "got: {}", cmds[0]);
    }

    #[test]
    fn init_player_flag_default_false() {
        let cmds = schema().init_player("@s");
        assert!(
            cmds[1].contains("unless score @s has_cells matches -2147483648.."),
            "got: {}",
            cmds[1]
        );
        assert!(cmds[1].contains("set @s has_cells 0"), "got: {}", cmds[1]);
    }
}
