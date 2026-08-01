//! Shared typed worldgen value providers.
//!
//! Worldgen registries reuse a handful of small value shapes — block states and
//! block-state providers being the most common. They live here so every
//! worldgen builder references one typed model instead of re-deriving raw JSON.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::error::Result as SandResult;
use crate::registry::BlockId;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A concrete block state: a block identifier plus optional property values.
///
/// ```
/// use sand_components::worldgen::providers::BlockState;
/// use sand_components::BlockId;
///
/// let state = BlockState::new(BlockId::minecraft("oak_log").unwrap()).property("axis", "y");
/// assert_eq!(state.block().to_string(), "minecraft:oak_log");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    block: BlockId,
    properties: BTreeMap<String, String>,
}

impl BlockState {
    /// Create a block state with no property overrides.
    pub fn new(block: BlockId) -> Self {
        Self {
            block,
            properties: BTreeMap::new(),
        }
    }

    /// Set a block-state property (e.g. `axis = y`).
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// The block this state refers to.
    pub fn block(&self) -> &BlockId {
        &self.block
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("Name".into(), Value::String(self.block.to_string()));
        if !self.properties.is_empty() {
            let mut properties = Map::new();
            for (key, value) in &self.properties {
                properties.insert(key.clone(), Value::String(value.clone()));
            }
            map.insert("Properties".into(), Value::Object(properties));
        }
        Value::Object(map)
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        for (key, value) in &self.properties {
            let key_field = format!("{field}.Properties");
            validation::require_non_empty(location, kind, &key_field, key)?;
            validation::reject_whitespace_only(location, kind, &key_field, key)?;
            validation::reject_control_chars(location, kind, &key_field, key)?;
            let value_field = format!("{field}.Properties.{key}");
            validation::require_non_empty(location, kind, &value_field, value)?;
            validation::reject_whitespace_only(location, kind, &value_field, value)?;
            validation::reject_control_chars(location, kind, &value_field, value)?;
        }
        Ok(())
    }
}

/// A weighted entry of a [`BlockStateProvider::Weighted`] provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedBlockState {
    state: BlockState,
    weight: u32,
}

impl WeightedBlockState {
    /// Create a weighted block-state entry. `weight` must be at least 1.
    pub fn new(state: BlockState, weight: u32) -> Self {
        Self { state, weight }
    }
}

/// A typed block-state provider used by worldgen feature configs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStateProvider {
    /// `minecraft:simple_state_provider` — always the same block state.
    Simple(BlockState),
    /// `minecraft:weighted_state_provider` — a weighted random choice.
    Weighted(Vec<WeightedBlockState>),
}

impl BlockStateProvider {
    /// Convenience constructor for a `minecraft:simple_state_provider`.
    pub fn simple(state: BlockState) -> Self {
        Self::Simple(state)
    }

    /// Convenience constructor for a `minecraft:weighted_state_provider`.
    pub fn weighted(entries: impl IntoIterator<Item = WeightedBlockState>) -> Self {
        Self::Weighted(entries.into_iter().collect())
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Simple(state) => serde_json::json!({
                "type": "minecraft:simple_state_provider",
                "state": state.to_json(),
            }),
            Self::Weighted(entries) => {
                let entries: Vec<Value> = entries
                    .iter()
                    .map(|entry| {
                        serde_json::json!({
                            "weight": entry.weight,
                            "data": entry.state.to_json(),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "type": "minecraft:weighted_state_provider",
                    "entries": entries,
                })
            }
        }
    }

    pub(crate) fn validate(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        match self {
            Self::Simple(state) => state.validate(location, kind, &format!("{field}.state")),
            Self::Weighted(entries) => {
                validation::require_non_empty_collection(
                    location,
                    kind,
                    &format!("{field}.entries"),
                    entries.len(),
                )?;
                for (index, entry) in entries.iter().enumerate() {
                    let entry_field = format!("{field}.entries[{index}]");
                    if entry.weight == 0 {
                        return Err(validation::error(
                            location,
                            kind,
                            &format!("{entry_field}.weight"),
                            "weight must be at least 1; received 0",
                        ));
                    }
                    entry.state.validate(location, kind, &entry_field)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location() -> ResourceLocation {
        ResourceLocation::new("test", "providers").unwrap()
    }

    #[test]
    fn simple_provider_serializes_block_state() {
        let provider = BlockStateProvider::simple(
            BlockState::new(BlockId::minecraft("oak_log").unwrap()).property("axis", "y"),
        );
        assert_eq!(
            provider.to_json(),
            serde_json::json!({
                "type": "minecraft:simple_state_provider",
                "state": {"Name": "minecraft:oak_log", "Properties": {"axis": "y"}},
            })
        );
        provider.validate(&location(), "kind", "field").unwrap();
    }

    #[test]
    fn weighted_provider_requires_entries_with_positive_weights() {
        let empty = BlockStateProvider::weighted([]);
        assert!(empty.validate(&location(), "kind", "field").is_err());

        let zero_weight = BlockStateProvider::weighted([WeightedBlockState::new(
            BlockState::new(BlockId::minecraft("stone").unwrap()),
            0,
        )]);
        assert!(zero_weight.validate(&location(), "kind", "field").is_err());

        let valid = BlockStateProvider::weighted([WeightedBlockState::new(
            BlockState::new(BlockId::minecraft("stone").unwrap()),
            3,
        )]);
        valid.validate(&location(), "kind", "field").unwrap();
        assert_eq!(valid.to_json()["entries"][0]["weight"], 3);
    }

    #[test]
    fn malformed_block_state_properties_are_rejected() {
        let state = BlockState::new(BlockId::minecraft("stone").unwrap()).property("", "value");
        assert!(state.validate(&location(), "kind", "field").is_err());

        let state = BlockState::new(BlockId::minecraft("stone").unwrap()).property("axis", "  ");
        assert!(state.validate(&location(), "kind", "field").is_err());
    }
}
