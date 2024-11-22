use commands::{gamemode::GameMode, teleport, PlayerCommands, TargetFilter};
use selector::TargetSelector;

pub mod advancements;
pub mod commands;
pub mod entities;
pub mod selector;

fn main() {
    let creative_command = PlayerCommands::Gamemode(
        GameMode::Creative,
        Some(TargetSelector {
            selector: selector::EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        }),
    );
    println!("{}", creative_command.to_string());
    let teleport_command = PlayerCommands::Teleport(
        teleport::Teleport::AllPlayersTo(10.3, 100.0, 10.0, Some(TargetFilter {
            name: Some("TheOneTrueToast".to_string()),
            ..Default::default()
        }))
    );
    println!("{}", teleport_command.to_string());
}
