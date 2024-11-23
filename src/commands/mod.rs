pub mod effect;
pub mod gamemode;
pub mod kill;
pub mod teleport;
pub mod utils;
pub mod clear;

pub use utils::{Distance, EntityName, TargetFilter};

#[derive(Debug, Clone)]
pub enum PlayerCommands {
    Gamemode(gamemode::GameMode),
    Teleport(teleport::Teleport),
    Kill(kill::Kill),
    Effect(effect::Effect),
    Clear(clear::Clear),
}

impl ToString for PlayerCommands {
    fn to_string(&self) -> String {
        match self {
            PlayerCommands::Gamemode(mode) => {
                format!("{}", mode.to_string())
            }
            PlayerCommands::Teleport(target) => {
                format!("{}", target.to_string())
            }
            PlayerCommands::Kill(selector) => {
                format!("{}", selector.to_string())
            }
            PlayerCommands::Effect(condition) => {
                format!("{}", condition.to_string())
            }
            PlayerCommands::Clear(selector) => {
                format!("{}", selector.to_string())
            }
        }
    }
}
