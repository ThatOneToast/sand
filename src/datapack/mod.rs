use advancements::Advancements;

pub mod builder;
pub mod advancements;
pub mod mc_entities;


#[derive(Debug, Clone)]
pub enum Distance {
    Exact(f32),      // A single exact distance
    Range(f32, f32), // A range with minimum and maximum distance
    Max(f32),        // Maximum distance (e.g., `..10`)
    Min(f32),        // Minimum distance (e.g., `3..`)
}

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


#[derive(Debug, Clone, PartialEq)]
pub enum EntitySelector {
    AllPlayers,                                          // @a
    AllEntities,                                         // @e
    Current,                                             // @s
    Random,                                              // @r
    Nearest,                                             // @p
    Entity,                                              // @e
    Other(String),
}

impl ToString for EntitySelector {
    fn to_string(&self) -> String {
        match self {
            EntitySelector::AllPlayers => "@a".to_string(),
            EntitySelector::AllEntities => "@e".to_string(),
            EntitySelector::Current => "@s".to_string(),
            EntitySelector::Nearest => "@p".to_string(),
            EntitySelector::Random => "@r".to_string(),
            EntitySelector::Entity => "@e".to_string(),
            EntitySelector::Other(name) => format!("{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityTargetFilter {
    pub advancements: Option<Vec<Advancements>>,
    pub distance: Option<Distance>,
    pub dx: Option<f32>,
    pub dy: Option<f32>,
    pub dz: Option<f32>,
    pub gamemode: Option<GamemodeType>,
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

impl Default for EntityTargetFilter {
    fn default() -> Self {
        EntityTargetFilter {
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


impl ToString for EntityTargetFilter {
    fn to_string(&self) -> String {
        let mut filters = Vec::new();

        if let Some(advancements) = &self.advancements {
            let advancements = advancements.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            filters.push(format!("advancements={{{}}}", advancements.join(",")));
        }
        if let Some(distance) = &self.distance {
            filters.push(format!("distance={}", distance.to_string()));
        }
        if let Some(dx) = &self.dx {
            filters.push(format!("dx={}", dx));
        }
        if let Some(dy) = &self.dy {
            filters.push(format!("dy={}", dy));
        }
        if let Some(dz) = &self.dz {
            filters.push(format!("dz={}", dz));
        }
        if let Some(gamemode) = &self.gamemode {
            filters.push(format!("gamemode={}", gamemode.to_string()));
        }
        if let Some(level) = &self.level {
            filters.push(format!("level={}", level));
        }
        if let Some(limit) = &self.limit {
            filters.push(format!("limit={}", limit));
        }
        if let Some(name) = &self.name {
            filters.push(format!("name={}", name));
        }
        if let Some(nbt) = &self.nbt {
            filters.push(format!("nbt={}", nbt));
        }
        if let Some(_predicate) = &self.predicate {
            unimplemented!("I'm not sure how to handle these yet.")
        }
        if let Some(_scores) = &self.scores {
            unimplemented!("I'm not sure how to handle these yet.")
        }
        if let Some(sort) = &self.sort {
            filters.push(format!("sort={}", sort));
        }
        if let Some(tag) = &self.tag {
            filters.push(format!("tag={}", tag));
        }
        if let Some(team) = &self.team {
            filters.push(format!("team={}", team));
        }
        if let Some(x) = &self.x {
            filters.push(format!("x={}", x));
        }
        if let Some(x_rotation) = &self.x_rotation {
            filters.push(format!("x_rotation={}", x_rotation.to_string()));
        }
        if let Some(y) = &self.y {
            filters.push(format!("y={}", y));
        }
        if let Some(y_rotation) = &self.y_rotation {
            filters.push(format!("y_rotation={}", y_rotation.to_string()));
        }
        if let Some(z) = &self.z {
            filters.push(format!("z={}", z));
        }

        filters.join(",")
    }
}