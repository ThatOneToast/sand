use crate::entities::MinecraftEntity;

use super::{EntityName, TargetFilter};

pub type X = f32;
pub type Y = f32;
pub type Z = f32;

#[derive(Debug, Clone)]
pub enum Teleport {
    SelfTo(X, Y, Z),
    PlayerTo(EntityName, X, Y, Z),
    EntityTo(MinecraftEntity, X, Y, Z),
    AllPlayersTo(X, Y, Z, Option<TargetFilter>),
    AllEntitiesTo(X, Y, Z, Option<TargetFilter>),
    NearestTo(X, Y, Z, Option<TargetFilter>),
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
            Teleport::AllPlayersTo(x, y, z, filter) => {
                let filter = filter.to_owned().unwrap_or(TargetFilter::default());
                format!("tp @a[{}] {} {} {}", filter.to_string(), x, y, z)
            }
            Teleport::AllEntitiesTo(x, y, z, filter) => {
                let filter = filter.to_owned().unwrap_or(TargetFilter::default());
                format!(
                    "tp @e[{}] {} {} {}",
                    filter.to_string(),
                    x,
                    y,
                    z
                )
            }
            Teleport::NearestTo(x, y, z, filter) => {
                let filter = filter.to_owned().unwrap_or(TargetFilter::default());
                format!(
                    "tp @p[{}] {} {} {}",
                    filter.to_string(),
                    x,
                    y,
                    z
                )
            }
            Teleport::Other(other) => format!("{other}"),
        }
    }
}
