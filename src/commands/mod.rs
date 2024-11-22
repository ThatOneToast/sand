pub mod gamemode;
pub mod teleport;
pub mod utils;

use crate::{advancements::Advancements, entities::MinecraftEntity};
use gamemode::GameMode;

pub type EntityName = String;

pub enum Distance {
    Exact(f32),      // A single exact distance
    Range(f32, f32), // A range with minimum and maximum distance
    Max(f32),        // Maximum distance (e.g., `..10`)
    Min(f32),        // Minimum distance (e.g., `3..`)
}

impl ToString for Distance {
    fn to_string(&self) -> String {
        match self {
            Distance::Exact(distance) => format!("{}", distance),
            Distance::Range(min, max) => format!("{}..{}", min, max),
            Distance::Max(max) => format!("..{}", max),
            Distance::Min(min) => format!("{}..", min),
        }
    }
}

pub struct TargetFilter {
    pub advancements: Option<Vec<Advancements>>,
    pub distance: Option<Distance>,
    pub dx: Option<f32>,
    pub dy: Option<f32>,
    pub dz: Option<f32>,
    pub gamemode: Option<GameMode>,
    pub level: Option<u32>,
    pub limit: Option<u32>,
    pub name: Option<String>,
    pub nbt: Option<String>,
    pub predicate: Option<String>,
    pub scores: Option<Vec<(String, i32, i32)>>, // Scoreboard criteria and range
    pub sort: Option<String>,                    // Sort by: nearest, furthest, random, etc.
    pub tag: Option<String>,                     // Entity tag
    pub team: Option<String>,                    // Team name
    pub x: Option<f32>,
    pub x_rotation: Option<Distance>,
    pub y: Option<f32>,
    pub y_rotation: Option<Distance>,
    pub z: Option<f32>,
}

impl ToString for TargetFilter {
}

impl Default for TargetFilter {
    fn default() -> Self {
        TargetFilter {
            advancements: None,
            distance: None,
            dx: None,
            dy: None,
            dz: None,
            gamemode: None,
            level: None,
            limit: None,
            name: None,
            nbt: None,
            predicate: None,
            scores: None,
            sort: None,
            tag: None,
            team: None,
            x: None,
            x_rotation: None,
            y: None,
            y_rotation: None,
            z: None,
        }
    }
}

pub struct TargetSelector {
    pub selector: EntityTargets,
    pub filter: TargetFilter,
}

impl Default for TargetSelector {
    fn default() -> Self {
        TargetSelector {
            selector: EntityTargets::Self,
            filter: TargetFilter::default(),
        }
    }
}



impl TargetSelector {
    pub fn target_self() -> TargetSelector {
}

#[derive(Debug, Clone)]
pub enum EntityTargets {
    AllPlayers,                                          // @a
    AllEntities,                                         // @e
    Selected,                                            // @s
    Random,                                              // @r
    Nearest,                                             // @p
    Entity(Option<MinecraftEntity>, Option<EntityName>), // @e[type=type,name=name]
    Other(String),
}

impl ToString for EntityTargets {
    fn to_string(&self) -> String {
        match self {
            EntityTargets::AllPlayers => "@a".to_string(),
            EntityTargets::AllEntities => "@e".to_string(),
            EntityTargets::Selected => "@s".to_string(),
            EntityTargets::Nearest => "@p".to_string(),
            EntityTargets::Random => "@r".to_string(),
            EntityTargets::Entity(entity, name) => {
                let etype = entity.as_ref();
                let name = name.as_ref();

                if etype.is_none() && name.is_none() {
                    return "@e".to_string();
                } else if etype.is_some() && name.is_none() {
                    return format!("@e[type={}]", etype.unwrap().to_string());
                } else if etype.is_none() && name.is_some() {
                    return format!("@e[name={}]", name.unwrap().to_string());
                } else {
                    return format!(
                        "@e[type={}, name={}]",
                        etype.unwrap().to_string(),
                        name.unwrap().to_string()
                    );
                }
            }
            EntityTargets::Other(name) => format!("{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerCommands {
    Gamemode(GameMode, Option<TargetSelector>),
}

impl ToString for PlayerCommands {
    fn to_string(&self) -> String {
        match self {
            PlayerCommands::Gamemode(mode, target) => {
                let mode_string = mode.as_ref().to_string();
                let entity_target = target.as_ref().unwrap_or();
                
                let mut command = String::from("/gamemode ");
            }
        }
    }
}
