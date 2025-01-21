use crate::lang::Rule;

use super::var::Type;

#[derive(Debug, Clone)]
pub struct CollectionId {
    pub name: String,
    pub properties: Vec<CollectionProperty>,
}

#[derive(Debug, Clone)]
pub struct CollectionProperty {
    pub name: String,
    pub value: Option<Type>,
}

impl CollectionId {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let collection_type = inner.next().unwrap();

        let mut properties = Vec::new();

        // Parse any properties that follow
        while let Some(prop) = inner.next() {
            if prop.as_rule() == Rule::collection_prop {
                let prop_name = prop.as_str().to_string();
                let value = inner.next().map(|v| Type::from_pest(v));

                properties.push(CollectionProperty {
                    name: prop_name,
                    value,
                });
            }
        }

        CollectionId {
            name: collection_type.as_str().to_string(),
            properties,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.name.as_str() {
            "entities" => self.validate_entities(),
            "advancements" | "achievements" => self.validate_advancements(),
            _ => Err(format!("Unknown collection type: {}", self.name)),
        }
    }

    fn validate_entities(&self) -> Result<(), String> {
        // Add validation logic for entities
        // Check for valid properties and their values
        Ok(())
    }

    fn validate_advancements(&self) -> Result<(), String> {
        // Add validation logic for advancements
        // Check for valid properties and their values
        Ok(())
    }
}
