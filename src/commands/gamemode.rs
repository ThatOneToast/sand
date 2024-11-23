use crate::selector::TargetSelector;

#[derive(Debug, Clone)]
pub enum GamemodeType {
    Survival,
    Creative,
    Spectator,
    Adventure,
}

impl ToString for GamemodeType {
    fn to_string(&self) -> String {
        match self {
            Self::Survival => "survival".to_string(),
            Self::Creative => "creative".to_string(),
            Self::Spectator => "spectator".to_string(),
            Self::Adventure => "adventure".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameMode {
    Creative(Option<TargetSelector>),
    Spectator(Option<TargetSelector>),
    Adventure(Option<TargetSelector>),
    Survival(Option<TargetSelector>),
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            GameMode::Creative(target) => {
                let target = target.as_ref();
                let mut command = String::from("/gamemode creative ");
                if target.is_some() {
                    command.push_str(target.unwrap().to_string().as_str());
                } else {
                    command.push_str(TargetSelector::default().to_string().as_str());
                }
                command 
            }
            GameMode::Spectator(target) => {
                let target = target.as_ref();
                let mut command = String::from("/gamemode spectator ");
                if target.is_some() {
                    command.push_str(target.unwrap().to_string().as_str());
                } else {
                    command.push_str(TargetSelector::default().to_string().as_str());
                }
                command 
            }
            GameMode::Adventure(target) => {
                let target = target.as_ref();
                let mut command = String::from("/gamemode adventure ");
                if target.is_some() {
                    command.push_str(target.unwrap().to_string().as_str());
                } else {
                    command.push_str(TargetSelector::default().to_string().as_str());
                }
                command 
            }
            GameMode::Survival(target) => {
                let target = target.as_ref();
                let mut command = String::from("/gamemode survival ");
                if target.is_some() {
                    command.push_str(target.unwrap().to_string().as_str());
                } else {
                    command.push_str(TargetSelector::default().to_string().as_str());
                }
                command
            }
        }
    }
}
