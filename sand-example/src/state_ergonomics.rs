//! Demonstrates the ergonomic state API additions — new helper methods on
//! `ScoreVar`, `Flag`, `Timer`, and `Cooldown`.

use sand_core::prelude::*;

// -- State declarations -------------------------------------------------------

static MANA: ScoreVar<i32> = ScoreVar::new("mana");
static MAX_MANA: ScoreVar<i32> = ScoreVar::new("max_mana");
static SHIELD: Flag = Flag::new("shield");
static BLINK_TIMER: Timer = Timer::new("blink_tm", Ticks::seconds(2));
static DASH: Cooldown = Cooldown::new("dash", Ticks::seconds(3));

// -- Load ---------------------------------------------------------------------

/// Register all objectives and initialize defaults.
pub fn load_state() -> Vec<String> {
    vec![
        MANA.define(),
        MAX_MANA.define(),
        SHIELD.define(),
        BLINK_TIMER.define(),
        DASH.define(),
    ]
}

// -- Init player on join ------------------------------------------------------

/// Initialize a new player's state without clobbering existing scores.
pub fn init_player() -> Vec<String> {
    vec![
        // ScoreVar::init — only sets if score is completely missing
        MANA.init("@s", 100),
        MAX_MANA.init("@s", 100),
        // Flag::init_false — same nil-check, sets to 0
        SHIELD.init_false("@s"),
    ]
}

// -- Tick logic ---------------------------------------------------------------

/// Per-tick: clamp mana, tick cooldown, tick timer.
pub fn tick_state() -> Vec<String> {
    let mut cmds = Vec::new();

    // Clamp mana to [0, max_mana] each tick
    cmds.extend(MANA.clamp("@a", 0, 100));

    // Tick blink timer and cooldown for all players
    cmds.push(BLINK_TIMER.tick("@a"));
    cmds.push(DASH.tick_all_players());

    cmds
}

// -- Ability logic ------------------------------------------------------------

/// Conditions using the new shorthand helpers.
pub fn ability_conditions() -> Vec<Condition> {
    vec![
        // ScoreVar shorthand conditions
        MANA.is_zero("@s"),
        MANA.is_nonzero("@s"),
        MANA.positive("@s"),
        MANA.negative("@s"),
        // ScoreRef conditions
        MANA.of("@s").is_zero(),
        MANA.of("@s").positive(),
        // Flag shorthand conditions
        SHIELD.when_true("@s"),
        SHIELD.when_false("@s"),
        SHIELD.unless_true("@s"),
        // Timer conditions
        BLINK_TIMER.expired("@s"),
        BLINK_TIMER.active("@s"),
        // Cooldown conditions
        DASH.ready("@s"),
        DASH.active("@s"),
        DASH.expired("@s"),
    ]
}

/// Score copy and operation commands.
pub fn score_ops() -> Vec<String> {
    vec![
        // Copy mana → max_mana for this player
        MANA.copy_to("@s", &MAX_MANA, "@s"),
        // Copy max_mana → mana from another player (hypothetical restore)
        MANA.copy_from("@s", &MAX_MANA, "@s"),
        // Clamp mana to max via min_op (mana ← min(mana, max_mana))
        MANA.min_op("@s", &MAX_MANA, "@s"),
        // Set score with explicit bool
        SHIELD.set("@s", true),
        // Clear flag
        SHIELD.clear("@s"),
        // Aliases
        DASH.start_for("@s"),
        DASH.reset_for("@s"),
    ]
}

/// Guard commands.
pub fn guard_cmds() -> Vec<String> {
    vec![
        // Return early if dash is still active
        DASH.guard("@s"),
        DASH.guard_active("@s"),
        // Return early if blink timer is still running
        BLINK_TIMER.guard_active("@s"),
        // Return early if dash is READY (useful in cooldown-active checks)
        DASH.guard_ready("@s"),
    ]
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_registers_all_objectives() {
        let cmds = load_state();
        assert_eq!(cmds.len(), 5);
        assert!(cmds[0].contains("mana"), "got: {}", cmds[0]);
    }

    #[test]
    fn init_player_uses_nil_checks() {
        let cmds = init_player();
        for cmd in &cmds {
            assert!(
                cmd.contains("unless score @s") && cmd.contains("matches -2147483648.."),
                "expected nil-check: {cmd}"
            );
        }
    }

    #[test]
    fn tick_state_includes_all_ticks() {
        let cmds = tick_state();
        assert!(cmds.iter().any(|c| c.contains("blink_tm")), "timer tick");
        assert!(cmds.iter().any(|c| c.contains("dash")), "cooldown tick");
    }

    #[test]
    fn ability_conditions_count() {
        let conds = ability_conditions();
        assert_eq!(conds.len(), 14);
    }

    #[test]
    fn score_ops_count() {
        let cmds = score_ops();
        assert_eq!(cmds.len(), 7);
    }

    #[test]
    fn guard_cmds_count() {
        let cmds = guard_cmds();
        assert_eq!(cmds.len(), 4);
    }
}
