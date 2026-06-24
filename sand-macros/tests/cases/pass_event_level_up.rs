use sand_core::event::vanilla::PlayerLevelsUp;
use sand_core::events::PlayerLevelUpEvent;
use sand_core::prelude::*;
use sand_macros::event;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[event]
pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    MANA.add(event.player(), 10);
}

#[event]
pub fn on_levels_up_alias(event: Event<PlayerLevelsUp>) {
    MANA.add(event.player(), 5);
}

fn main() {
    // Handler body should contain the scoreboard add command.
    let commands = on_level_up();
    assert!(
        commands
            .iter()
            .any(|cmd| cmd.contains("scoreboard players add")),
        "expected scoreboard add in handler body"
    );

    // Both handlers must register with XpLevelUp dispatch — not Advancement.
    let mut found_level_up = false;
    let mut found_alias = false;
    for descriptor in inventory::iter::<sand_core::EventDescriptor>() {
        if descriptor.path == "on_level_up" {
            match descriptor.dispatch {
                sand_core::EventDispatch::XpLevelUp => {
                    found_level_up = true;
                }
                _ => panic!("PlayerLevelUpEvent handler must use XpLevelUp dispatch"),
            }
        }
        if descriptor.path == "on_levels_up_alias" {
            match descriptor.dispatch {
                sand_core::EventDispatch::XpLevelUp => {
                    found_alias = true;
                }
                _ => panic!("PlayerLevelsUp handler must use XpLevelUp dispatch"),
            }
        }
    }
    assert!(found_level_up, "PlayerLevelUpEvent handler not found");
    assert!(found_alias, "PlayerLevelsUp handler not found");

    // Helper methods should produce condition strings referencing the correct objectives.
    // We verify via the gte condition which serializes the objective name.
    let delta_ge_5 = PlayerLevelUpEvent::level_delta("@s").gte(5);
    let cond_str = format!("{:?}", delta_ge_5);
    // The condition captures the objective name __sand_xp_delta.
    assert!(
        cond_str.contains("__sand_xp_delta"),
        "level_delta score ref should reference __sand_xp_delta, got: {cond_str}"
    );
}
