use commands::{gamemode::GameMode, PlayerCommands};


pub mod advancements;
pub mod commands;
pub mod entities;
pub mod selector;

fn main() {
    let creative_command = PlayerCommands::Gamemode(GameMode::Creative, None);
    println!("{}", creative_command.to_string());
}
