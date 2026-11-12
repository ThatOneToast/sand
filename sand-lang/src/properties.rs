//! Entity properties: attributes and data read through a handle.
//!
//! `Player.max_health` and, inside `for zombie in entities(..)`,
//! `zombie.step_height` read or write an attribute's base value. Data such as
//! `Player.health`, `Player.food`, or `zombie.on_ground` is read-only. Every
//! attribute in the target version's registry is a property named after its
//! identity: `minecraft:step_height` is `step_height`.

use sand::advanced::compiler::EntityData;

/// What a property reads or writes.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKind {
    /// An attribute's base value, such as `minecraft:max_health`.
    Attribute(String),
    /// A read-only value from the entity's data.
    Data(EntityData),
}

/// The value type of a property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    /// A whole number.
    Number,
    /// A decimal with three places.
    Float,
    /// `true` or `false`.
    Bool,
}

impl PropertyType {
    /// The language name of the type.
    pub fn name(self) -> &'static str {
        match self {
            PropertyType::Number => "number",
            PropertyType::Float => "float",
            PropertyType::Bool => "bool",
        }
    }
}

/// One property of a player or entity handle.
#[derive(Debug, Clone)]
pub struct Property {
    /// Field name, such as `max_health`.
    pub name: String,
    /// What it reads or writes.
    pub kind: PropertyKind,
    /// Value type.
    pub ty: PropertyType,
    /// Whether only players have it.
    pub player_only: bool,
    /// One-sentence description.
    pub summary: String,
}

impl Property {
    /// Whether assignments can change it.
    pub fn writable(&self) -> bool {
        matches!(self.kind, PropertyKind::Attribute(_))
    }

    /// Hover text: signature, capabilities, and the command it becomes.
    pub fn describe(&self, handle: &str) -> String {
        let access = if self.writable() {
            "read and write"
        } else {
            "read-only"
        };
        let minecraft = match &self.kind {
            PropertyKind::Attribute(id) => {
                format!("`attribute @s {id} base get` / `base set`")
            }
            PropertyKind::Data(data) => format!("`data get entity @s {}`", data.path()),
        };
        let only = if self.player_only {
            " Players only."
        } else {
            ""
        };
        format!(
            "```sand\n{handle}.{}: {}\n```\n\n{}{only}\n\n*{access}* · Minecraft: {minecraft}",
            self.name,
            self.ty.name(),
            self.summary
        )
    }
}

/// (name, data, type, summary) for each data property.
const DATA: &[(&str, EntityData, PropertyType, &str)] = &[
    (
        "health",
        EntityData::Health,
        PropertyType::Float,
        "Current health in half-hearts: 20.0 is ten hearts.",
    ),
    (
        "absorption",
        EntityData::Absorption,
        PropertyType::Float,
        "Extra golden health from absorption.",
    ),
    (
        "air",
        EntityData::Air,
        PropertyType::Number,
        "Breath left in ticks; 300 is full.",
    ),
    (
        "fire",
        EntityData::Fire,
        PropertyType::Number,
        "Ticks left burning; zero or less when not on fire.",
    ),
    (
        "fall_distance",
        EntityData::FallDistance,
        PropertyType::Float,
        "Blocks fallen since last standing on the ground.",
    ),
    (
        "on_ground",
        EntityData::OnGround,
        PropertyType::Bool,
        "Whether it is standing on a block.",
    ),
    ("x", EntityData::X, PropertyType::Float, "East-west position."),
    ("y", EntityData::Y, PropertyType::Float, "Height."),
    ("z", EntityData::Z, PropertyType::Float, "North-south position."),
    (
        "food",
        EntityData::Food,
        PropertyType::Number,
        "Hunger from 0 to 20.",
    ),
    (
        "saturation",
        EntityData::Saturation,
        PropertyType::Float,
        "Hidden food saturation, used up before hunger drops.",
    ),
    (
        "xp_level",
        EntityData::XpLevel,
        PropertyType::Number,
        "Experience level.",
    ),
    (
        "xp_total",
        EntityData::XpTotal,
        PropertyType::Number,
        "Experience points collected in total.",
    ),
    (
        "xp_progress",
        EntityData::XpProgress,
        PropertyType::Float,
        "Progress to the next level, from 0.0 to 1.0.",
    ),
    (
        "selected_slot",
        EntityData::SelectedSlot,
        PropertyType::Number,
        "Selected hotbar slot, from 0 to 8.",
    ),
];

/// Every property, data first, then attributes in name order.
pub fn all() -> Vec<Property> {
    let mut out: Vec<Property> = DATA
        .iter()
        .map(|(name, data, ty, summary)| Property {
            name: (*name).into(),
            kind: PropertyKind::Data(*data),
            ty: *ty,
            player_only: data.player_only(),
            summary: (*summary).into(),
        })
        .collect();
    if let Some(registry) = crate::vanilla::registry("Attribute") {
        let mut attributes: Vec<(&str, &str)> = registry
            .variants
            .iter()
            .filter_map(|(variant, id)| {
                Some((id.strip_prefix("minecraft:")?, variant.as_str()))
            })
            .collect();
        attributes.sort();
        for (name, variant) in attributes {
            if DATA.iter().any(|(data, ..)| *data == name) {
                continue;
            }
            out.push(Property {
                name: name.into(),
                kind: PropertyKind::Attribute(format!("minecraft:{name}")),
                ty: PropertyType::Float,
                player_only: false,
                summary: format!(
                    "The `Attribute::{variant}` base value; assigning it changes the base."
                ),
            });
        }
    }
    out
}

/// The property called `name`.
pub fn find(name: &str) -> Option<Property> {
    all().into_iter().find(|p| p.name == name)
}

/// The closest property names to a misspelling, best first.
pub fn suggest(wrong: &str) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = all()
        .into_iter()
        .map(|p| (crate::vanilla::distance(wrong, &p.name), p.name))
        .filter(|(d, name)| *d <= 2.max(name.len() / 4))
        .collect();
    scored.sort();
    scored.into_iter().take(3).map(|(_, name)| name).collect()
}
