#[derive(Debug, Clone)]
pub enum GameMode {
    Survival,
    Creative,
    Spectator,
    Adventure,
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            GameMode::Survival => "survival",
            GameMode::Creative => "creative",
            GameMode::Spectator => "spectator",
            GameMode::Adventure => "adventure",
        }
        .to_string()
    }
}