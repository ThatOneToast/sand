//! Enhanced Cells dogfood example — exercises the framework reliability patches:
//!
//! - Event bodies with `then_all` / `unless(...).then_all` correctly export
//!   their branch functions (Part 1 fix).
//! - `DamageTracker` with heart-based units (Part 4).
//! - `#[derive(SandStorage)]` for player state (Part 6).
//! - `DamageAmount::hearts(...)` produces correct HP commands.

use sand_core::AdvancementTrigger;
use sand_core::prelude::*;
use sand_core::systems::damage::DamageTracker;
use sand_macros::{SandStorage, component, event};

// ── Player state schema ───────────────────────────────────────────────────────

#[derive(SandStorage)]
#[sand(storage = "ec:players", root = "player.cells")]
pub struct CellsState {
    pub level: i32,
    pub charges: i32,
    #[sand(path = "aoe_enabled")]
    pub aoe: i32,
}

// ── Cooldowns ─────────────────────────────────────────────────────────────────

static SATURATION_COOLDOWN: Cooldown = Cooldown::new("ec_sat", Ticks::seconds(20));
static REGEN_COOLDOWN: Cooldown = Cooldown::new("ec_reg", Ticks::seconds(5));
static AOE_DMG_COOLDOWN: Cooldown = Cooldown::new("ec_aoe_dmg", Ticks::seconds(2));
static HAS_CELLS: Flag = Flag::new("ec_has_cells");

// ── Enhanced Cells damage event ───────────────────────────────────────────────

pub struct EnhancedCellsDamagedEvent;

impl AdvancementEvent for EnhancedCellsDamagedEvent {
    type Trigger = AdvancementTrigger;

    fn trigger() -> Self::Trigger {
        AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None,
        }
    }

    fn guard() -> Option<Condition> {
        Some(HAS_CELLS.of("@s").is_true())
    }
}

impl DamageAdvancementEvent for EnhancedCellsDamagedEvent {}

// ── Load / Tick components ────────────────────────────────────────────────────

/// Initialize damage tracking and cooldown objectives on load.
#[component(Load)]
pub fn ec_load() {
    // Damage tracker defines 5 objectives
    DamageTracker::define();

    // Cooldown objectives
    SATURATION_COOLDOWN.define();
    REGEN_COOLDOWN.define();
    AOE_DMG_COOLDOWN.define();
    HAS_CELLS.define();
}

/// Tick all damage tracking and cooldowns every game tick.
#[component(Tick)]
pub fn ec_tick() {
    // Must run before any damage-dependent event logic
    DamageTracker::tick_players();

    SATURATION_COOLDOWN.tick(Selector::all_players());
    REGEN_COOLDOWN.tick(Selector::all_players());
    AOE_DMG_COOLDOWN.tick(Selector::all_players());
}

// ── Event: AOE damage reflect on hit ─────────────────────────────────────────

/// When a player with Enhanced Cells is damaged and the AOE cooldown is ready,
/// reflect damage to nearby enemies.
///
/// This exercises `then_all` inside an `#[event]` body — the branch function
/// must be exported even though event bodies run after the early drain in the
/// old (buggy) pipeline.
#[event]
pub fn ec_on_damaged(event: DamageEvent<EnhancedCellsDamagedEvent>) {
    // when(...).then_all(...) registers a branch function.
    // With the Part 1 fix, this branch is exported correctly.
    when(AOE_DMG_COOLDOWN.ready("@s")).then_all([
        event
            .reflect_damage()
            .to(EntityTargets::nearby(5.0)
                .excluding_self()
                .excluding_players())
            .amount(DamageAmount::hearts(2.0))
            .damage_type(DamageKind::Magic)
            .run()
            .join("\n"),
        AOE_DMG_COOLDOWN.start(Selector::self_()),
        cmd::tellraw(
            Selector::self_(),
            Text::new("Enhanced Cells: AOE burst!").red(),
        )
        .to_string(),
    ]);
}

// ── Event: Regen on low health ────────────────────────────────────────────────

/// Grant regeneration when the player has taken significant damage recently.
#[event]
pub fn ec_on_damaged_regen(event: DamageEvent<EnhancedCellsDamagedEvent>) {
    let _ = event;
    // unless(...).then_all(...) also registers a branch
    unless(REGEN_COOLDOWN.active("@s")).then_all([
        cmd::effect_give(Selector::self_(), EffectId::Regeneration)
            .seconds(5)
            .amplifier(1)
            .particles(false)
            .to_string(),
        REGEN_COOLDOWN.start(Selector::self_()),
    ]);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::dyn_fn_test_lock;
    use sand_core::drain_dyn_fns;

    #[test]
    fn cells_state_schema_paths() {
        assert_eq!(CellsState::level().field_name(), "level");
        assert_eq!(CellsState::charges().field_name(), "charges");
        // Custom path override
        assert_eq!(CellsState::aoe().field_name(), "aoe_enabled");
        assert_eq!(CellsState::level().storage(), "ec:players");
        assert_eq!(CellsState::level().root_path(), "player.cells");
    }

    #[test]
    fn cells_state_get_command() {
        let cmd = CellsState::level().get();
        assert!(cmd.contains("data get storage ec:players"), "got: {cmd}");
        assert!(cmd.contains("player.cells.level"), "got: {cmd}");
    }

    #[test]
    fn cells_state_set_command() {
        let cmd = CellsState::level().set(SnbtValue::Int(10));
        assert!(cmd.contains("data modify storage ec:players"), "got: {cmd}");
        assert!(cmd.contains("player.cells.level"), "got: {cmd}");
        assert!(cmd.contains("10"), "got: {cmd}");
    }

    #[test]
    fn ec_load_includes_damage_tracker_objectives() {
        let cmds = ec_load();
        assert!(
            cmds.iter()
                .any(|c| c.contains("minecraft.custom:minecraft.damage_taken")),
            "load should define damage stat objective: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("sd_dmg_hurt")),
            "load should define hurt-age objective: {cmds:?}"
        );
        assert!(
            cmds.iter().any(|c| c.contains("sd_dmg_last")),
            "load should define last-damage objective: {cmds:?}"
        );
    }

    #[test]
    fn ec_on_damaged_registers_branch_for_then_all() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();

        // Simulate what the export pipeline does: call the event make() body.
        let cmds = ec_on_damaged();

        // The event body must have used then_all, creating a branch reference.
        let branch_cmds: Vec<_> = cmds
            .iter()
            .filter(|c| c.contains("function __sand_local:sand/branches/"))
            .collect();
        assert!(
            !branch_cmds.is_empty(),
            "ec_on_damaged should call a branch function: {cmds:?}"
        );

        // The branch must exist in the registry (ready for the late drain).
        let branches = drain_dyn_fns();
        assert!(
            !branches.is_empty(),
            "branch should be registered after ec_on_damaged(): {branches:?}"
        );
        // Branch content should include the damage command with hearts.
        // DamageAmount::hearts(2.0) → 4.0 HP → command contains "4"
        assert!(
            branches.iter().any(|(_, cmds)| cmds
                .iter()
                .any(|c| c.contains("damage") && c.contains("4") && c.contains("minecraft:magic"))),
            "branch should contain reflected magic damage: {branches:?}"
        );
    }

    #[test]
    fn ec_on_damaged_regen_registers_unless_branch() {
        let _guard = dyn_fn_test_lock();
        let _ = drain_dyn_fns();

        let cmds = ec_on_damaged_regen();

        let branch_cmds: Vec<_> = cmds
            .iter()
            .filter(|c| {
                c.contains("execute unless") && c.contains("function __sand_local:sand/branches/")
            })
            .collect();
        assert!(
            !branch_cmds.is_empty(),
            "ec_on_damaged_regen should use unless+then_all branch: {cmds:?}"
        );

        let branches = drain_dyn_fns();
        assert!(
            branches
                .iter()
                .any(|(_, cmds)| cmds.iter().any(|c| c.contains("minecraft:regeneration"))),
            "branch should contain regeneration effect: {branches:?}"
        );
    }

    #[test]
    fn aoe_damage_uses_heart_units() {
        // DamageAmount::hearts(2.0) = 2 hearts = 4 HP
        let dmds = DamageAmount::hearts(2.0);
        let cmd = Damage::new()
            .amount(dmds)
            .damage_type(DamageKind::Magic)
            .run();
        assert_eq!(cmd, vec!["damage @s 4 minecraft:magic"]);
    }

    #[test]
    fn ec_tick_includes_damage_tracker() {
        let cmds = ec_tick();
        assert!(
            cmds.iter()
                .any(|c| c.contains("sd_dmg_delta") && c.contains("sd_dmg_stat")),
            "tick should include damage tracker operation: {cmds:?}"
        );
    }
}
