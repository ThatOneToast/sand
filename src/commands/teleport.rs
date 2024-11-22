use crate::entities::MinecraftEntity;

use super::EntityName;

pub type X = f32;
pub type Y = f32;
pub type Z = f32;

pub enum Teleport {
    SelfTo(X, Y, Z),
    PlayerTo(EntityName, X, Y, Z),
    EntityTo(MinecraftEntity, X, Y, Z),
    AllPlayersTo(X, Y, Z),
    AllEntitiesTo(X, Y, Z),
    NearestTo(X, Y, Z),
    Other(String),
}

impl ToString for Teleport {
    fn to_string(&self) -> String {
        match self {
            Teleport::SelfTo(x, y, z) => format!("tp {} {} {}", x, y, z),
            Teleport::PlayerTo(name, x, y, z) => format!("tp {} {} {} {}", name, x, y, z),
            Teleport::EntityTo(entity, x, y, z) => {
                format!("tp @e[type={}] {} {} {}", entity.to_string(), x, y, z)
            }
            Teleport::AllPlayersTo(x, y, z) => format!("tp @a {} {} {}", x, y, z),
            Teleport::AllEntitiesTo(x, y, z) => format!("tp @e {} {} {}", x, y, z),
            Teleport::NearestTo(x, y, z) => format!("tp @p {} {} {}", x, y, z),
            Teleport::Other(other) => format!("{other}"),
        }
    }
}
