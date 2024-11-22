use commands::{EntityTargets, GamemodeMode, PlayerCommands};

pub mod commands;
pub mod entities;
pub mod advancements;

fn main() {
    let creative_command = PlayerCommands::Gamemode(
        GamemodeMode::Creative,
        Some(EntityTargets::Entity(
            None,
            Some("ThatOneToast".to_string()),
        )),
    );
    println!("{}", creative_command.to_string());
}
