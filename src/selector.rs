use crate::{commands::{EntityName, TargetFilter}, entities::MinecraftEntity};

#[derive(Debug, Clone)]
pub struct TargetSelector {
    pub selector: EntityTargets,
    pub filter: TargetFilter,
}

impl Default for TargetSelector {
    fn default() -> Self {
        TargetSelector {
            selector: EntityTargets::Current,
            filter: TargetFilter::default(),
        }
    }
}

impl ToString for TargetSelector {
    fn to_string(&self) -> String {
        let entity_target = self.selector.to_string();
        let filter = self.filter.to_string();

        format!("{}[{}]", entity_target, filter)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityTargets {
    AllPlayers,                                          // @a
    AllEntities,                                         // @e
    Current,                                             // @s
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
            EntityTargets::Current => "@s".to_string(),
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