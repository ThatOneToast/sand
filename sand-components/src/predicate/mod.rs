use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
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

    fn validate(&self) -> crate::error::Result<()> {
        self.condition.validate_at("predicate").map_err(|message| {
            crate::error::SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "predicate".to_string(),
                field: "predicate".to_string(),
                message,
            }
        })
    }

    fn to_json(&self) -> Value {
        serde_json::to_value(&self.condition)
            .unwrap_or_else(|error| panic!("predicate serialization failed: {error}"))
    }

    fn try_content(&self) -> crate::error::Result<ComponentContent> {
        self.validate()?;
        serde_json::to_value(&self.condition)
            .map(ComponentContent::Json)
            .map_err(crate::error::SandError::Serialization)
    }

    fn component_dir(&self) -> &'static str {
        "predicate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::DatapackComponent;

    #[test]
    fn invalid_standalone_predicate_returns_contextual_error() {
        let predicate = Predicate::new(
            "test:bad_chance".parse().unwrap(),
            LootCondition::RandomChance { chance: 1.5 },
        );
        let error = predicate.try_content().unwrap_err().to_string();
        assert!(error.contains("test:bad_chance"));
        assert!(error.contains("predicate"));
        assert!(error.contains("predicate.chance"));
    }

    #[test]
    fn valid_standalone_predicate_output_is_unchanged() {
        let predicate = Predicate::new(
            "test:valid_chance".parse().unwrap(),
            LootCondition::RandomChance { chance: 0.5 },
        );
        assert_eq!(predicate.try_content().unwrap(), predicate.content());
    }
}
