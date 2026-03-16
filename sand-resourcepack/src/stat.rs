//! High-level stat tracking for HUD bars.
//!
//! [`BarStat`] wraps a [`BarHandle`] and auto-generates the scoreboard
//! objectives and math commands needed to drive it — no manual `/scoreboard`
//! boilerplate required.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use sand_resourcepack::{BarStat, HudLayout};
//!
//! // In your Load function:
//! BarStat::health(HEALTH).setup()   // creates objectives
//!
//! // In your Tick function:
//! let health = BarStat::health(HEALTH);
//! health.update("@a")               // read → scale → clamp
//!
//! HudLayout::new("my_pack", 1000)
//!     .tracked_bar(&health, 250)    // uses health.frame_obj automatically
//!     .broadcast("@a");
//! ```
//!
//! # Sources
//!
//! | Constructor | What it tracks |
//! |---|---|
//! | [`BarStat::health`] | Player's `Health` attribute, scaled to bar steps |
//! | [`BarStat::score`] | A scoreboard objective, mapped from `0..max_val` to `0..steps` |

use crate::handle::BarHandle;

// ── Source enum ────────────────────────────────────────────────────────────────

enum StatSource {
    /// Tracks player health: reads `Health` NBT and `max_health` attribute.
    Health {
        /// Temporary objective for the raw current HP (×100).
        curr_obj: String,
        /// Temporary objective for the raw max HP (×100).
        max_obj: String,
    },
    /// Tracks an arbitrary scoreboard objective mapped from `0..max_val`.
    Score {
        /// Source scoreboard objective to read.
        source_obj: String,
        /// The maximum value of the source objective (maps to `steps - 1`).
        max_val: i32,
    },
}

// ── BarStat ────────────────────────────────────────────────────────────────────

/// Combines a [`BarHandle`] with auto-managed scoreboard objectives and
/// per-tick update math.
///
/// # Objective naming
///
/// All auto-created objectives use the bar's `name` as a prefix so they are
/// easy to identify in `/scoreboard` output:
///
/// | Objective | Purpose |
/// |---|---|
/// | `<name>_frame` | Current frame index fed to [`HudLayout`](crate::HudLayout) |
/// | `_<name>_hp` | Raw current HP ×100 (Health source only) |
/// | `_<name>_maxhp` | Raw max HP ×100 (Health source only) |
///
/// # Example — two bars in one layout
///
/// ```rust,ignore
/// hud_bar!(name = "health", texture = create!(...), steps = 20, height = 14, ascent = 14);
/// hud_bar!(name = "mana",   texture = create!(...), steps = 20, height = 14, ascent = 0);
///
/// #[component(Load)]
/// pub fn load() -> Vec<String> {
///     mcfunction! {
///         BarStat::health(HEALTH).setup();
///         BarStat::score(MANA, "player_mana", 100).setup();
///     }
/// }
///
/// #[component(Tick)]
/// pub fn tick() -> Vec<String> {
///     let health = BarStat::health(HEALTH);
///     let mana   = BarStat::score(MANA, "player_mana", 100);
///     mcfunction! {
///         health.update("@a");
///         mana.update("@a");
///         HudLayout::new("my_pack", 1000)
///             .tracked_bar(&health, 250)
///             .tracked_bar(&mana,   750)
///             .broadcast("@a");
///     }
/// }
/// ```
pub struct BarStat {
    /// The bar this stat drives.
    pub handle: BarHandle,
    /// Scoreboard objective that stores the current frame index.
    ///
    /// Pass this directly to [`BarHandle::broadcast_commands`] if you are not
    /// using [`HudLayout::tracked_bar`].
    pub frame_obj: String,
    source: StatSource,
}

impl BarStat {
    /// Create a `BarStat` that tracks player health.
    ///
    /// The generated `update` commands read `Health` NBT and the
    /// `max_health` generic attribute, then scale the result into
    /// `0..handle.steps`.
    ///
    /// Objectives created by `setup()`:
    /// - `<name>_frame` — frame index
    /// - `_<name>_hp`   — current HP ×100 (hidden; prefixed with `_`)
    /// - `_<name>_maxhp` — max HP ×100 (hidden)
    pub fn health(handle: BarHandle) -> Self {
        let n = handle.name;
        Self {
            handle,
            frame_obj: format!("{n}_frame"),
            source: StatSource::Health {
                curr_obj: format!("_{n}_hp"),
                max_obj: format!("_{n}_maxhp"),
            },
        }
    }

    /// Create a `BarStat` that tracks an arbitrary scoreboard objective.
    ///
    /// The value of `source_objective` is mapped linearly from `0..=max_val`
    /// to `0..=steps-1`. Values outside `[0, max_val]` are clamped.
    ///
    /// Objectives created by `setup()`:
    /// - `<name>_frame` — frame index
    ///
    /// (The source objective is **not** created automatically — it is assumed
    /// to already exist and be populated by your own commands.)
    pub fn score(handle: BarHandle, source_objective: impl Into<String>, max_val: i32) -> Self {
        let n = handle.name;
        Self {
            handle,
            frame_obj: format!("{n}_frame"),
            source: StatSource::Score {
                source_obj: source_objective.into(),
                max_val,
            },
        }
    }

    /// Return the commands that create all required scoreboard objectives.
    ///
    /// Call this from your `#[component(Load)]` function.
    pub fn setup(&self) -> Vec<String> {
        let mut cmds = Vec::new();
        let frame = &self.frame_obj;

        match &self.source {
            StatSource::Health { curr_obj, max_obj } => {
                cmds.push(format!("scoreboard objectives add {frame} dummy"));
                cmds.push(format!("scoreboard objectives add {curr_obj} dummy"));
                cmds.push(format!("scoreboard objectives add {max_obj} dummy"));
            }
            StatSource::Score { .. } => {
                cmds.push(format!("scoreboard objectives add {frame} dummy"));
            }
        }

        cmds
    }

    /// Return the commands that read the stat, scale it, and clamp the frame
    /// objective to `0..=steps-1`.
    ///
    /// Call this from your `#[component(Tick)]` function **before** emitting
    /// [`HudLayout`](crate::HudLayout) commands.
    ///
    /// `executor` is typically `"@a"`.
    pub fn update(&self, executor: &str) -> Vec<String> {
        let mut cmds = Vec::new();
        let frame = &self.frame_obj;
        let steps = self.handle.steps;
        let name = self.handle.name;
        let max_frame = steps - 1;

        match &self.source {
            StatSource::Health { curr_obj, max_obj } => {
                // Read current and max health (× 100 to get integer precision).
                cmds.push(format!(
                    "execute as {executor} store result score @s {curr_obj} run data get entity @s Health 100"
                ));
                cmds.push(format!(
                    "execute as {executor} store result score @s {max_obj} run attribute @s minecraft:max_health get 100"
                ));
                // frame = curr * (steps-1) / max
                // Using integer scoreboard math:
                //   frame = curr
                //   frame *= (steps-1)   (constant stored in a fake-player score)
                //   frame /= max
                let const_obj = format!("#const_{name}");
                cmds.push(format!(
                    "execute as {executor} run scoreboard players operation @s {frame} = @s {curr_obj}"
                ));
                cmds.push(format!(
                    "scoreboard players set {const_obj} {frame} {max_frame}"
                ));
                cmds.push(format!(
                    "execute as {executor} run scoreboard players operation @s {frame} *= {const_obj} {frame}"
                ));
                cmds.push(format!(
                    "execute as {executor} if score @s {max_obj} matches 1.. run scoreboard players operation @s {frame} /= @s {max_obj}"
                ));
            }
            StatSource::Score {
                source_obj,
                max_val,
            } => {
                // frame = source * (steps-1) / max_val
                let const_steps = format!("#steps_{name}");
                let const_max = format!("#max_{name}");
                cmds.push(format!(
                    "execute as {executor} run scoreboard players operation @s {frame} = @s {source_obj}"
                ));
                cmds.push(format!(
                    "scoreboard players set {const_steps} {frame} {max_frame}"
                ));
                cmds.push(format!(
                    "execute as {executor} run scoreboard players operation @s {frame} *= {const_steps} {frame}"
                ));
                cmds.push(format!(
                    "scoreboard players set {const_max} {frame} {max_val}"
                ));
                cmds.push(format!(
                    "execute as {executor} if score {const_max} {frame} matches 1.. run scoreboard players operation @s {frame} /= {const_max} {frame}"
                ));
            }
        }

        // Clamp to [0, steps-1].
        cmds.push(format!(
            "execute as {executor} if score @s {frame} matches {steps}.. run scoreboard players set @s {frame} {max_frame}"
        ));
        cmds.push(format!(
            "execute as {executor} if score @s {frame} matches ..-1 run scoreboard players set @s {frame} 0"
        ));

        cmds
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle::BarHandle;

    const HEALTH: BarHandle = BarHandle {
        name: "health",
        steps: 20,
        font: "hud",
        frame_width: 28,
    };
    const MANA: BarHandle = BarHandle {
        name: "mana",
        steps: 10,
        font: "hud",
        frame_width: 28,
    };

    #[test]
    fn health_setup_creates_three_objectives() {
        let stat = BarStat::health(HEALTH);
        let cmds = stat.setup();
        assert_eq!(cmds.len(), 3);
        assert!(cmds[0].contains("health_frame"));
        assert!(cmds[1].contains("_health_hp"));
        assert!(cmds[2].contains("_health_maxhp"));
    }

    #[test]
    fn score_setup_creates_one_objective() {
        let stat = BarStat::score(MANA, "player_mana", 100);
        let cmds = stat.setup();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("mana_frame"));
    }

    #[test]
    fn health_update_contains_nbt_read() {
        let stat = BarStat::health(HEALTH);
        let cmds = stat.update("@a");
        assert!(cmds.iter().any(|c| c.contains("data get entity @s Health")));
        assert!(cmds.iter().any(|c| c.contains("max_health")));
    }

    #[test]
    fn health_update_clamps() {
        let stat = BarStat::health(HEALTH);
        let cmds = stat.update("@a");
        // Upper clamp: matches 20..
        assert!(cmds.iter().any(|c| c.contains("matches 20..")));
        // Lower clamp: matches ..-1
        assert!(cmds.iter().any(|c| c.contains("matches ..-1")));
    }

    #[test]
    fn score_update_reads_source_objective() {
        let stat = BarStat::score(MANA, "player_mana", 100);
        let cmds = stat.update("@a");
        assert!(cmds.iter().any(|c| c.contains("player_mana")));
    }

    #[test]
    fn frame_obj_name() {
        assert_eq!(BarStat::health(HEALTH).frame_obj, "health_frame");
        assert_eq!(BarStat::score(MANA, "pm", 100).frame_obj, "mana_frame");
    }
}
