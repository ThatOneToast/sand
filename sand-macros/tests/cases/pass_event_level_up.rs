mod actions_support;
use sand_core::event::vanilla::PlayerLevelsUp;
use sand_core::events::PlayerLevelUpEvent;
use sand_core::prelude::*;
use sand_core::advanced::state::ScoreVar;
use sand_macros::on_event;

static MANA: ScoreVar<i32> = ScoreVar::new("mana");

#[on_event]
pub fn on_level_up(event: Event<PlayerLevelUpEvent>) {
    MANA.add(event.player(), 10);
}

#[on_event]
pub fn on_levels_up_alias(event: Event<PlayerLevelsUp>) {
    MANA.add(event.player(), 5);
}

fn main() {
    // Handler body should contain the scoreboard add command.
    let commands = actions_support::emitted(on_level_up());
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

    // Public snapshots compose through the canonical numeric/condition APIs.
    let delta: StatCurve = PlayerLevelUpEvent::level_delta(Target::current_player());
    let _: sand::command::Actions = TypedExecute::as_players()
        .when(delta.gte(5)).run(cmd::say("large level increase"));
}
