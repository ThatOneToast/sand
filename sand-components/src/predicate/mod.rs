use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use crate::loot_table::LootCondition;

/// A Minecraft predicate that defines a condition that can be evaluated in commands or loot tables.
pub struct Predicate {
    /// The resource location for this predicate.
    pub location: ResourceLocation,
    /// The condition logic for this predicate.
    pub condition: LootCondition,
}

impl Predicate {
    /// Create a new predicate with the given resource location and condition.
    pub fn new(location: ResourceLocation, condition: LootCondition) -> Self {
        Self {
            location,
            condition,
        }
    }
}

impl DatapackComponent for Predicate {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        serde_json::to_value(&self.condition).unwrap()
    }

    fn component_dir(&self) -> &'static str {
        "predicate"
    }
}
