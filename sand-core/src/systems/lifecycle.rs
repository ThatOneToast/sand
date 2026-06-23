//! Join/death/respawn lifecycle helpers (`systems-lifecycle` feature).
//!
//! Provides command generators for the three key player lifecycle events:
//! - **Join** — player connects (or reconnects) to the server
//! - **Death** — player dies
//! - **Respawn** — player respawns after death
//!
//! These complement the typed events in `sand_core::events` (e.g. `OnJoinEvent`,
//! `OnDeathEvent`, `OnRespawnEvent`) by exposing reusable command fragments
//! that can be called from those event handlers.

// ── Join helpers ───────────────────────────────────────────────────────────────

/// Commands to run when a player joins for the first time.
///
/// Checks a flag objective to distinguish first-ever joins from reconnects.
pub struct FirstJoinCommands {
    flag_obj: String,
}

impl FirstJoinCommands {
    /// Create a new first-join helper backed by the given flag objective name.
    pub fn new(flag_objective: impl Into<String>) -> Self {
        Self {
            flag_obj: flag_objective.into(),
        }
    }

    /// Define the first-join flag objective.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.flag_obj)
    }

    /// Guard: skip if this is not the player's first join.
    ///
    /// Returns early if `flag_obj` is already set to 1 for `@s`.
    pub fn guard_not_first(&self) -> String {
        format!(
            "execute if score @s {} matches 1 run return 0",
            self.flag_obj
        )
    }

    /// Mark the player as having joined before (set flag to 1).
    pub fn mark_joined(&self) -> String {
        format!("scoreboard players set @s {} 1", self.flag_obj)
    }
}

// ── Respawn helpers ────────────────────────────────────────────────────────────

/// Commands to run when a player respawns.
///
/// Provides a guard to avoid double-running if the respawn event fires
/// while the player is still in the death screen.
pub struct RespawnCommands {
    dead_flag_obj: String,
}

impl RespawnCommands {
    /// Create a new respawn helper backed by the given "is dead" flag objective.
    pub fn new(dead_flag_objective: impl Into<String>) -> Self {
        Self {
            dead_flag_obj: dead_flag_objective.into(),
        }
    }

    /// Define the death flag objective.
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.dead_flag_obj)
    }

    /// Set the "player is dead" flag.  Call from your death handler.
    pub fn mark_dead(&self) -> String {
        format!("scoreboard players set @s {} 1", self.dead_flag_obj)
    }

    /// Clear the "player is dead" flag.  Call from your respawn handler.
    pub fn clear_dead(&self) -> String {
        format!("scoreboard players set @s {} 0", self.dead_flag_obj)
    }

    /// Guard: skip if the player is not marked as dead.
    pub fn guard_not_dead(&self) -> String {
        format!(
            "execute unless score @s {} matches 1 run return 0",
            self.dead_flag_obj
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_join_define() {
        let h = FirstJoinCommands::new("sl_first_join");
        assert_eq!(h.define(), "scoreboard objectives add sl_first_join dummy");
    }

    #[test]
    fn first_join_guard() {
        let h = FirstJoinCommands::new("sl_first_join");
        let cmd = h.guard_not_first();
        assert!(
            cmd.contains("if score @s sl_first_join matches 1 run return 0"),
            "got: {cmd}"
        );
    }

    #[test]
    fn first_join_mark() {
        let h = FirstJoinCommands::new("sl_first_join");
        assert_eq!(h.mark_joined(), "scoreboard players set @s sl_first_join 1");
    }

    #[test]
    fn respawn_define() {
        let r = RespawnCommands::new("sl_is_dead");
        assert_eq!(r.define(), "scoreboard objectives add sl_is_dead dummy");
    }

    #[test]
    fn respawn_mark_and_clear() {
        let r = RespawnCommands::new("sl_is_dead");
        assert_eq!(r.mark_dead(), "scoreboard players set @s sl_is_dead 1");
        assert_eq!(r.clear_dead(), "scoreboard players set @s sl_is_dead 0");
    }
}
