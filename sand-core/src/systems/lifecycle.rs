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

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::lifecycle::FirstJoinCommands",
    module = "sand::systems",
    summary = "Commands to run when a player joins for the first time.",
    context = "Commands to run when a player joins for the first time. Checks a flag objective to distinguish first-ever joins from reconnects.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::lifecycle::FirstJoinCommands;",
    availability = ["Cargo feature: systems-lifecycle"],
)]
/// Commands to run when a player joins for the first time.
///
/// Checks a flag objective to distinguish first-ever joins from reconnects.
pub struct FirstJoinCommands {
    flag_obj: String,
}

impl FirstJoinCommands {
    /// Create a new first-join helper backed by the given flag objective name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::FirstJoinCommands::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new first-join helper backed by the given flag objective name.",
        context = "Create a new first-join helper backed by the given flag objective name. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(flag_objective = "`flag_objective` is used when creating a new first-join helper backed by the given flag objective name."),
        returns = "A `FirstJoinCommands` representing a new first-join helper backed by the given flag objective name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(flag_objective: impl Into < String >)  {\n    let first_join_commands = sand::systems::lifecycle::FirstJoinCommands::new(flag_objective);\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn new(flag_objective: impl Into<String>) -> Self {
        Self {
            flag_obj: flag_objective.into(),
        }
    }

    /// Define the first-join flag objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::FirstJoinCommands::define",
        module = "sand::systems",
        kind = "method",
        summary = "Define the first-join flag objective.",
        context = "Define the first-join flag objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to define the first-join flag objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(first_join_commands_value: &sand::systems::lifecycle::FirstJoinCommands)  {\n    let define = first_join_commands_value.define();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.flag_obj)
    }

    /// Guard: skip if this is not the player's first join.
    ///
    /// Returns early if `flag_obj` is already set to 1 for `@s`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::FirstJoinCommands::guard_not_first",
        module = "sand::systems",
        kind = "method",
        summary = "Guard: skip if this is not the player's first join.",
        context = "Guard: skip if this is not the player's first join. Returns early if `flag_obj` is already set to 1 for `@s`.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "Returns early if `flag_obj` is already set to 1 for `@s`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(first_join_commands_value: &sand::systems::lifecycle::FirstJoinCommands)  {\n    let guard_not_first = first_join_commands_value.guard_not_first();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn guard_not_first(&self) -> String {
        format!(
            "execute if score @s {} matches 1 run return 0",
            self.flag_obj
        )
    }

    /// Mark the player as having joined before (set flag to 1).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::FirstJoinCommands::mark_joined",
        module = "sand::systems",
        kind = "method",
        summary = "Mark the player as having joined before (set flag to 1).",
        context = "Mark the player as having joined before (set flag to 1). This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to mark the player as having joined before (set flag to 1).",
        example = "use sand::prelude::*;\n\nfn demonstrate(first_join_commands_value: &sand::systems::lifecycle::FirstJoinCommands)  {\n    let mark_joined = first_join_commands_value.mark_joined();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn mark_joined(&self) -> String {
        format!("scoreboard players set @s {} 1", self.flag_obj)
    }
}

// ── Respawn helpers ────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::systems::lifecycle::RespawnCommands",
    module = "sand::systems",
    summary = "Commands to run when a player respawns. Provides a guard to avoid double-running if the respawn event fires while the player is still in the death screen.",
    context = "Commands to run when a player respawns. Provides a guard to avoid double-running if the respawn event fires while the player is still in the death screen. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
    minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
    use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
    avoid_when = ["Using the API outside its documented system scope or feature configuration"],
    example = "use sand::systems::lifecycle::RespawnCommands;",
    availability = ["Cargo feature: systems-lifecycle"],
)]
/// Commands to run when a player respawns.
///
/// Provides a guard to avoid double-running if the respawn event fires
/// while the player is still in the death screen.
pub struct RespawnCommands {
    dead_flag_obj: String,
}

impl RespawnCommands {
    /// Create a new respawn helper backed by the given "is dead" flag objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::RespawnCommands::new",
        module = "sand::systems",
        kind = "method",
        summary = "Create a new respawn helper backed by the given \"is dead\" flag objective.",
        context = "Create a new respawn helper backed by the given \"is dead\" flag objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        params(dead_flag_objective = "`dead_flag_objective` is used when creating a new respawn helper backed by the given \"is dead\" flag objective."),
        returns = "A `RespawnCommands` representing a new respawn helper backed by the given \"is dead\" flag objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dead_flag_objective: impl Into < String >)  {\n    let respawn_commands = sand::systems::lifecycle::RespawnCommands::new(dead_flag_objective);\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn new(dead_flag_objective: impl Into<String>) -> Self {
        Self {
            dead_flag_obj: dead_flag_objective.into(),
        }
    }

    /// Define the death flag objective.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::RespawnCommands::define",
        module = "sand::systems",
        kind = "method",
        summary = "Define the death flag objective.",
        context = "Define the death flag objective. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to define the death flag objective.",
        example = "use sand::prelude::*;\n\nfn demonstrate(respawn_commands_value: &sand::systems::lifecycle::RespawnCommands)  {\n    let define = respawn_commands_value.define();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn define(&self) -> String {
        format!("scoreboard objectives add {} dummy", self.dead_flag_obj)
    }

    /// Set the "player is dead" flag.  Call from your death handler.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::RespawnCommands::mark_dead",
        module = "sand::systems",
        kind = "method",
        summary = "Set the \"player is dead\" flag.  Call from your death handler.",
        context = "Set the \"player is dead\" flag.  Call from your death handler. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to set the \"player is dead\" flag. Call from your death handler.",
        example = "use sand::prelude::*;\n\nfn demonstrate(respawn_commands_value: &sand::systems::lifecycle::RespawnCommands)  {\n    let mark_dead = respawn_commands_value.mark_dead();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn mark_dead(&self) -> String {
        format!("scoreboard players set @s {} 1", self.dead_flag_obj)
    }

    /// Clear the "player is dead" flag.  Call from your respawn handler.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::RespawnCommands::clear_dead",
        module = "sand::systems",
        kind = "method",
        summary = "Clear the \"player is dead\" flag.  Call from your respawn handler.",
        context = "Clear the \"player is dead\" flag.  Call from your respawn handler. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to clear the \"player is dead\" flag. Call from your respawn handler.",
        example = "use sand::prelude::*;\n\nfn demonstrate(respawn_commands_value: &sand::systems::lifecycle::RespawnCommands)  {\n    let clear_dead = respawn_commands_value.clear_dead();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
    pub fn clear_dead(&self) -> String {
        format!("scoreboard players set @s {} 0", self.dead_flag_obj)
    }

    /// Guard: skip if the player is not marked as dead.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::systems::lifecycle::RespawnCommands::guard_not_dead",
        module = "sand::systems",
        kind = "method",
        summary = "Guard: skip if the player is not marked as dead.",
        context = "Guard: skip if the player is not marked as dead. This opt-in system composes Sand's typed primitives into a higher-level gameplay behavior; exporter registries and generated tick bookkeeping are private.",
        minecraft = "The exact commands, resources, and lifecycle behavior are described by the defining item's source documentation for the selected feature and Minecraft profile.",
        use_when = ["Opting into the documented higher-level gameplay behavior instead of assembling its commands manually"],
        avoid_when = ["Using the API outside its documented system scope or feature configuration"],
        returns = "The string value produced to guard skip if the player is not marked as dead.",
        example = "use sand::prelude::*;\n\nfn demonstrate(respawn_commands_value: &sand::systems::lifecycle::RespawnCommands)  {\n    let guard_not_dead = respawn_commands_value.guard_not_dead();\n}",
        availability = ["Cargo feature: systems-lifecycle"],
    )]
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
