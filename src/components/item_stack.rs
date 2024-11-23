use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::DataComponent;


#[derive(DataComponent, Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub slot: Option<u8>,
    pub id: Option<String>, // != minecraft:air
    pub count: u8,
    pub components: Option<HashMap<String, String>>
}