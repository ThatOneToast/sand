use serde::{de::DeserializeOwned, Serialize};

// Implement all of the components from:  https://minecraft.wiki/w/Data_component_format
pub mod item_stack;

pub trait DataComponent: Serialize + DeserializeOwned {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    
    fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap()
    }
}