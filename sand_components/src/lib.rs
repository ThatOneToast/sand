use serde::{de::DeserializeOwned, Serialize};

pub mod item;


pub trait DataComponent: Serialize + DeserializeOwned + Sized {
    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
}
