pub mod gamemode;
pub mod teleport;
pub mod utils;

use crate::{entities::MinecraftEntity, selector::TargetSelector};
use gamemode::GameMode;
pub use utils::{Distance, EntityName, TargetFilter};



#[derive(Debug, Clone)]
pub enum PlayerCommands {
    Gamemode(GameMode, Option<TargetSelector>),
}

impl ToString for PlayerCommands {
    fn to_string(&self) -> String {
        match self {
            PlayerCommands::Gamemode(mode, target) => {
                let mode_string = mode.to_string();
                let entity_target = target.as_ref();

                let mut command = String::from(format!("/gamemode {mode_string} "));
                if entity_target.is_some() {
                    command.push_str(entity_target.unwrap().to_string().as_str());
                } else {
                    command.push_str(TargetSelector::default().to_string().as_str());
                }

                command
            }
        }
    }
}
