use commands::{clear, effect, gamemode::GameMode, kill, teleport, PlayerCommands, TargetFilter};
use selector::TargetSelector;

pub mod advancements;
pub mod status_effects;
pub mod commands;
pub mod entities;
pub mod selector;
pub mod items;

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
        teleport::Teleport::PlayerToPlayer("test".to_string(), "toast".to_string())
    );
    println!("{}", teleport_command.to_string());
    let kill_command = PlayerCommands::Kill(kill::Kill(TargetSelector{
        selector: selector::EntityTargets::AllPlayers,
        filter: TargetFilter {
            name: Some("TheOneTrueToast".to_string()),
            level: Some(10),
            ..Default::default()
        },
    }));
    println!("{}", kill_command.to_string());
    let effect_command = PlayerCommands::Effect(effect::Effect::Give(
        TargetSelector {
            selector: selector::EntityTargets::AllPlayers,
            filter: TargetFilter {
                name: Some("TheOneTrueToast".to_string()),
                ..Default::default()
            },
        },
        status_effects::StatusEffects::Blindness,
        30,
        0,
    ));
    println!("{}", effect_command.to_string());
    let clear_command = PlayerCommands::Clear(clear::Clear(Some(TargetSelector {
        selector: selector::EntityTargets::AllPlayers,
        filter: TargetFilter {
            name: Some("TheOneTrueToast".to_string()),
            ..Default::default()
        },
    })));
    println!("{}", clear_command.to_string());
}
