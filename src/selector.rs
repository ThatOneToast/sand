use crate::commands::TargetFilter;

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
    Entity,                                              // @e
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
            EntityTargets::Entity => "@e".to_string(),
            EntityTargets::Other(name) => format!("{name}"),
        }
    }
}