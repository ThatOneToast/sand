//! # Level-up Reward Fixture
//!
//! Canonical datapack recipe for issue #71. Demonstrates composition of five
//! Sand subsystems in a single coherent pattern:
//!
//! - [`PlayerLevelsUp`] event — fires each time a player gains an XP level
//! - [`ScoreVar<i32>`][ScoreVar] — accumulates a "soul" reward currency
//! - [`Cooldown`] — collapses rapid multi-level bursts into one reward pulse
//! - `#[component(Load)]` — registers scoreboards once on datapack load
//! - `#[component(Tick)]` — drives cooldown decay every game tick
//!
//! ## System wiring
//!
//! ```text
//! load  →  lvl_load:   SOULS.define(), REWARD_CD.define()
//! tick  →  lvl_tick:   REWARD_CD.tick(@a)
//! event →  on_level_up: [REWARD_CD ready?] → lvl_grant_reward
//! fn    →  lvl_grant_reward: SOULS += 1, REWARD_CD.start, actionbar "+1 soul"
//! ```
//!
//! ## Vanilla limitation note
//!
//! Minecraft has no `minecraft:leveled_up` advancement trigger. Sand emulates it
//! via a tick-polled scoreboard comparison (`__sand_xp_lvl` vs `__sand_xp_prev`).
//! There is a one-tick dispatch latency — an inherent vanilla scoreboard constraint.
//! `REWARD_CD` handles back-to-back level gains (e.g. `/xp add @s 100 levels`)
//! by collapsing the burst into a single reward pulse.

use sand_core::event::vanilla::PlayerLevelsUp;
use sand_core::prelude::*;
use sand_macros::{component, event, function};

// ── State ─────────────────────────────────────────────────────────────────────

/// Accumulated "soul" reward currency — one per XP level gained.
static SOULS: ScoreVar<i32> = ScoreVar::new("lvl_souls");

/// 2-tick burst-collapse cooldown — prevents double rewards when a player gains
/// multiple levels in a single game tick (e.g. via `/xp add @s 100 levels`).
static REWARD_CD: Cooldown = Cooldown::new("lvl_reward_cd", Ticks::new(2));

// ── Load ──────────────────────────────────────────────────────────────────────

/// Registers scoreboard objectives on datapack load.
///
/// `scoreboard objectives add` is idempotent — Minecraft ignores it when the
/// objective already exists, so reloading the datapack never resets player data.
#[component(Load)]
pub fn lvl_load() {
    SOULS.define();
    REWARD_CD.define();
}

// ── Tick ──────────────────────────────────────────────────────────────────────

/// Decrements the burst-collapse cooldown every game tick for all online players.
#[component(Tick)]
pub fn lvl_tick() {
    REWARD_CD.tick(Selector::all_players());
}

// ── Event: level-up reward ────────────────────────────────────────────────────

fn ensure_reward_cooldown_score(selector: &str) -> String {
    let objective = REWARD_CD.objective_name();
    format!(
        "execute unless score {selector} {objective} = {selector} {objective} run scoreboard players set {selector} {objective} 0"
    )
}

/// Fires whenever a player's XP level increases.
///
/// Guards on `REWARD_CD` so that back-to-back level gains within 2 ticks produce
/// exactly one soul reward. The cooldown is started inside `lvl_grant_reward`
/// so the first tick in a burst still fires the reward.
#[event]
pub fn on_level_up(event: Event<PlayerLevelsUp>) {
    let _ = event;
    // In an event handler the executing entity is always @s — the player whose
    // level just increased. `event.player()` returns a Selector, but Cooldown
    // guards take &str, so we use the "@s" literal directly.
    cmd::raw(ensure_reward_cooldown_score("@s"));
    when(REWARD_CD.ready("@s")).then_all([cmd::function(
        ResourceLocation::new("hello_world", "lvl_grant_reward").unwrap(),
    )]);
}

// ── Reward function ───────────────────────────────────────────────────────────

/// Grants one soul, starts the burst-collapse cooldown, and notifies the player.
///
/// Exposed as a named function so other datapacks can hook into the reward
/// pipeline directly via `function hello_world:lvl_grant_reward`.
#[function("hello_world:lvl_grant_reward")]
pub fn lvl_grant_reward() {
    SOULS.add(Selector::self_(), 1);
    REWARD_CD.start(Selector::self_());
    Actionbar::show(Selector::self_(), Text::new("+1 soul").gold().bold(true));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::dyn_fn_test_lock;
    use sand_core::{FunctionDescriptor, FunctionTagDescriptor, drain_dyn_fns, inventory};

    #[test]
    fn lvl_load_defines_souls_and_cooldown_objectives() {
        let cmds = lvl_load();
        assert!(
            cmds.iter().any(|c| c.contains("lvl_souls")),
            "load should define souls objective: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("lvl_reward_cd")),
            "load should define burst-collapse cooldown objective: {cmds:?}"
        );
    }

    #[test]
    fn lvl_tick_decrements_cooldown() {
        let cmds = lvl_tick();
        assert!(
            cmds.iter().any(|c| c.contains("lvl_reward_cd")),
            "tick should decrement burst-collapse cooldown: {cmds:?}"
        );
    }

    #[test]
    fn lvl_load_registered_in_load_tag() {
        let tag = inventory::iter::<FunctionTagDescriptor>()
            .find(|d| d.function_path == "lvl_load")
            .expect("lvl_load not registered in FunctionTagDescriptor");
        assert_eq!(tag.tag, "minecraft:load");
    }

    #[test]
    fn lvl_tick_registered_in_tick_tag() {
        let tag = inventory::iter::<FunctionTagDescriptor>()
            .find(|d| d.function_path == "lvl_tick")
            .expect("lvl_tick not registered in FunctionTagDescriptor");
        assert_eq!(tag.tag, "minecraft:tick");
    }

    #[test]
    fn lvl_grant_reward_registered_as_function() {
        let paths: Vec<&str> = inventory::iter::<FunctionDescriptor>()
            .map(|d| d.path)
            .collect();
        assert!(
            paths.contains(&"lvl_grant_reward"),
            "lvl_grant_reward not in FunctionDescriptor inventory: {paths:?}"
        );
    }

    #[test]
    fn lvl_grant_reward_commands_add_soul_and_start_cooldown() {
        let cmds = lvl_grant_reward();
        assert!(
            cmds.iter().any(|c| c.contains("lvl_souls")),
            "reward should add to souls scoreboard: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("lvl_reward_cd")),
            "reward should start burst-collapse cooldown: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("+1 soul")),
            "reward should show actionbar notification: {cmds:?}"
        );
    }

    #[test]
    fn on_level_up_creates_cooldown_gated_branch() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();

        let cmds = on_level_up();

        // when(...).then_all([...]) emits a branch reference, not inline commands.
        let branch_cmds: Vec<_> = cmds
            .iter()
            .filter(|c| c.contains("function __sand_local:sand/branches/"))
            .collect();
        assert!(
            !branch_cmds.is_empty(),
            "on_level_up should produce a branch call via when().then_all: {cmds:?}"
        );

        let branches = drain_dyn_fns();
        assert!(
            branches
                .iter()
                .any(|(_, cmds)| cmds.iter().any(|c| c.contains("lvl_grant_reward"))),
            "branch should call lvl_grant_reward: {branches:?}"
        );
    }

    #[test]
    fn on_level_up_initializes_missing_cooldown_before_ready_guard() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();

        let cmds = on_level_up();
        let init_idx = cmds
            .iter()
            .position(|c| {
                c.contains("execute unless score @s lvl_reward_cd = @s lvl_reward_cd")
                    && c.contains("scoreboard players set @s lvl_reward_cd 0")
            })
            .expect("event should initialize missing cooldown score");
        let ready_idx = cmds
            .iter()
            .position(|c| c.contains("if score @s lvl_reward_cd matches 0"))
            .expect("event should check cooldown readiness");

        assert!(
            init_idx < ready_idx,
            "missing-score initialization must run before readiness check: {cmds:?}"
        );
    }
}
