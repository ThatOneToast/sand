use serde_json::Value;

use crate::component::DatapackComponent;
use crate::resource_location::ResourceLocation;

use super::loot_table::LootFunction;

/// An item modifier that applies loot functions to transform items in Minecraft.
pub struct ItemModifier {
    pub location: ResourceLocation,
    pub functions: Vec<LootFunction>,
}

impl ItemModifier {
    /// Creates a new ItemModifier with the given resource location.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            functions: Vec::new(),
        }
    }

    /// Adds a loot function to this item modifier.
    pub fn function(mut self, f: LootFunction) -> Self {
        self.functions.push(f);
        self
    }
}

impl DatapackComponent for ItemModifier {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        match self.functions.as_slice() {
            [] => Value::Array(vec![]),
            [single] => serde_json::to_value(single).unwrap(),
            many => serde_json::to_value(many).unwrap(),
        }
    }

    fn component_dir(&self) -> &'static str {
        "item_modifier"
    }
}
