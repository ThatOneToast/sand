//! Typed builders for `data/<namespace>/enchantment_provider/` JSON files.
//!
//! Enchantment providers were added with data-driven enchantments in
//! Minecraft 1.21. The normal API covers all three vanilla provider kinds:
//! [`EnchantmentProvider::single`], [`EnchantmentProvider::by_cost`], and
//! [`EnchantmentProvider::by_cost_with_difficulty`].
//!
//! Constant and uniform integer providers are typed through
//! [`EnchantmentProviderInt`]. Use [`EnchantmentProvider::raw`] for provider
//! shapes that need another vanilla integer-provider form or a modded provider
//! kind; raw JSON must still be an object with a valid resource-location
//! `type` field.

use serde_json::{Value, json};

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::raw::RawJson;
use crate::registry::{EnchantmentId, TagId};
use crate::resource_location::ResourceLocation;
use crate::validation;

/// A positive integer provider used for enchantment levels and enchanting costs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantmentProviderInt {
    /// A fixed positive integer.
    Constant(i32),
    /// A uniformly sampled inclusive positive range.
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
}

impl EnchantmentProviderInt {
    /// Create a fixed integer provider.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProviderInt::constant` for the canonical contract."]
    pub fn constant(value: i32) -> Self {
        Self::Constant(value)
    }

    /// Create a uniformly sampled inclusive integer provider.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProviderInt::uniform` for the canonical contract."]
    pub fn uniform(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self::Uniform {
            min_inclusive,
            max_inclusive,
        }
    }

    fn validate(&self, location: &ResourceLocation, field: &str) -> SandResult<()> {
        let (min, max) = match *self {
            Self::Constant(value) => (value, value),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => (min_inclusive, max_inclusive),
        };
        if min < 1 {
            return Err(validation::error(
                location,
                "enchantment_provider",
                field,
                "integer provider values must be positive",
            ));
        }
        if min > max {
            return Err(validation::error(
                location,
                "enchantment_provider",
                field,
                "uniform min_inclusive must not exceed max_inclusive",
            ));
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        match *self {
            Self::Constant(value) => json!(value),
            Self::Uniform {
                min_inclusive,
                max_inclusive,
            } => json!({
                "type": "minecraft:uniform",
                "min_inclusive": min_inclusive,
                "max_inclusive": max_inclusive,
            }),
        }
    }
}

impl From<i32> for EnchantmentProviderInt {
    fn from(value: i32) -> Self {
        Self::constant(value)
    }
}

/// A typed set of enchantments accepted by cost-based providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantmentSelection {
    /// One concrete enchantment.
    Single(EnchantmentId),
    /// A non-empty list of concrete enchantments.
    List(Vec<EnchantmentId>),
    /// Every enchantment in a typed enchantment tag.
    Tag(TagId<EnchantmentId>),
}

impl EnchantmentSelection {
    /// Select one concrete enchantment.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentSelection::one` for the canonical contract."]
    pub fn one(enchantment: EnchantmentId) -> Self {
        Self::Single(enchantment)
    }

    /// Select multiple concrete enchantments.
    ///
    /// Empty collections are rejected during component validation.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentSelection::many` for the canonical contract."]
    pub fn many(enchantments: impl IntoIterator<Item = EnchantmentId>) -> Self {
        Self::List(enchantments.into_iter().collect())
    }

    /// Select all enchantments in a tag.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentSelection::tag` for the canonical contract."]
    pub fn tag(tag: TagId<EnchantmentId>) -> Self {
        Self::Tag(tag)
    }

    /// Validate this selection's builder invariants (non-empty explicit
    /// lists). `kind`/`field` let callers outside `enchantment_provider`
    /// (e.g. `villager_trade`) report the error against their own owning
    /// component/field path instead of `"enchantment_provider"`.
    pub(crate) fn validate_with(
        &self,
        location: &ResourceLocation,
        kind: &str,
        field: &str,
    ) -> SandResult<()> {
        if let Self::List(enchantments) = self
            && enchantments.is_empty()
        {
            return Err(validation::error(
                location,
                kind,
                field,
                "enchantment list must not be empty",
            ));
        }
        Ok(())
    }

    fn validate(&self, location: &ResourceLocation) -> SandResult<()> {
        self.validate_with(location, "enchantment_provider", "enchantments")
    }

    /// Render this selection to its vanilla JSON shape (single ID string,
    /// array of ID strings, or a `#namespace:path` tag string).
    pub(crate) fn to_json_value(&self) -> Value {
        self.to_json()
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Single(enchantment) => Value::String(enchantment.to_string()),
            Self::List(enchantments) => Value::Array(
                enchantments
                    .iter()
                    .map(|enchantment| Value::String(enchantment.to_string()))
                    .collect(),
            ),
            Self::Tag(tag) => Value::String(tag.to_tag_string()),
        }
    }
}

impl From<EnchantmentId> for EnchantmentSelection {
    fn from(enchantment: EnchantmentId) -> Self {
        Self::one(enchantment)
    }
}

impl From<TagId<EnchantmentId>> for EnchantmentSelection {
    fn from(tag: TagId<EnchantmentId>) -> Self {
        Self::tag(tag)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum EnchantmentProviderKind {
    Single {
        enchantment: EnchantmentId,
        level: EnchantmentProviderInt,
    },
    ByCost {
        enchantments: EnchantmentSelection,
        cost: EnchantmentProviderInt,
    },
    ByCostWithDifficulty {
        enchantments: EnchantmentSelection,
        min_cost: u32,
        max_cost_span: u32,
    },
    Raw(RawJson),
}

/// A data-driven enchantment provider definition (Minecraft 1.21+).
#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentProvider {
    location: ResourceLocation,
    kind: EnchantmentProviderKind,
}

impl EnchantmentProvider {
    /// Always provide one enchantment at a fixed or randomized positive level.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProvider::single` for the canonical contract."]
    pub fn single(
        location: ResourceLocation,
        enchantment: EnchantmentId,
        level: impl Into<EnchantmentProviderInt>,
    ) -> Self {
        Self {
            location,
            kind: EnchantmentProviderKind::Single {
                enchantment,
                level: level.into(),
            },
        }
    }

    /// Choose compatible enchantments from a typed set using an enchanting cost.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProvider::by_cost` for the canonical contract."]
    pub fn by_cost(
        location: ResourceLocation,
        enchantments: impl Into<EnchantmentSelection>,
        cost: impl Into<EnchantmentProviderInt>,
    ) -> Self {
        Self {
            location,
            kind: EnchantmentProviderKind::ByCost {
                enchantments: enchantments.into(),
                cost: cost.into(),
            },
        }
    }

    /// Choose enchantments using a cost influenced by local difficulty.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProvider::by_cost_with_difficulty` for the canonical contract."]
    pub fn by_cost_with_difficulty(
        location: ResourceLocation,
        enchantments: impl Into<EnchantmentSelection>,
        min_cost: u32,
        max_cost_span: u32,
    ) -> Self {
        Self {
            location,
            kind: EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments: enchantments.into(),
                min_cost,
                max_cost_span,
            },
        }
    }

    /// Use an explicit raw provider object for unsupported or modded shapes.
    ///
    /// The export boundary still requires an object with a valid namespaced
    /// `type` string. Nested fields remain intentionally opaque.
    #[doc = "**API Contract:** Run `sand api show sand::component::EnchantmentProvider::raw` for the canonical contract."]
    pub fn raw(location: ResourceLocation, provider: RawJson) -> Self {
        Self {
            location,
            kind: EnchantmentProviderKind::Raw(provider),
        }
    }
}

impl DatapackComponent for EnchantmentProvider {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        match &self.kind {
            EnchantmentProviderKind::Single { level, .. } => {
                level.validate(&self.location, "level")
            }
            EnchantmentProviderKind::ByCost { enchantments, cost } => {
                enchantments.validate(&self.location)?;
                cost.validate(&self.location, "cost")
            }
            EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments,
                min_cost,
                ..
            } => {
                enchantments.validate(&self.location)?;
                if *min_cost == 0 {
                    return Err(validation::error(
                        &self.location,
                        "enchantment_provider",
                        "min_cost",
                        "min_cost must be positive",
                    ));
                }
                Ok(())
            }
            EnchantmentProviderKind::Raw(provider) => {
                let Some(object) = provider.as_value().as_object() else {
                    return Err(validation::error(
                        &self.location,
                        "enchantment_provider",
                        "<root>",
                        "raw provider must be a JSON object",
                    ));
                };
                let Some(provider_type) = object.get("type").and_then(Value::as_str) else {
                    return Err(validation::error(
                        &self.location,
                        "enchantment_provider",
                        "type",
                        "raw provider must contain a string type field",
                    ));
                };
                validation::validate_resource_location_str(
                    &self.location,
                    "enchantment_provider",
                    "type",
                    provider_type,
                )
            }
        }
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        match &self.kind {
            EnchantmentProviderKind::Single { enchantment, level } => json!({
                "type": "minecraft:single",
                "enchantment": enchantment,
                "level": level.to_json(),
            }),
            EnchantmentProviderKind::ByCost { enchantments, cost } => json!({
                "type": "minecraft:by_cost",
                "enchantments": enchantments.to_json(),
                "cost": cost.to_json(),
            }),
            EnchantmentProviderKind::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => json!({
                "type": "minecraft:by_cost_with_difficulty",
                "enchantments": enchantments.to_json(),
                "min_cost": min_cost,
                "max_cost_span": max_cost_span,
            }),
            EnchantmentProviderKind::Raw(provider) => provider.as_value().clone(),
        }
    }

    fn component_dir(&self) -> &'static str {
        "enchantment_provider"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::Enchantments]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(path: &str) -> ResourceLocation {
        ResourceLocation::new("test", path).unwrap()
    }

    #[test]
    fn single_provider_has_exact_json() {
        let provider = EnchantmentProvider::single(
            location("enderman_loot_drop"),
            EnchantmentId::minecraft("silk_touch").unwrap(),
            1,
        );
        assert_eq!(
            provider.to_json(),
            json!({
                "type": "minecraft:single",
                "enchantment": "minecraft:silk_touch",
                "level": 1,
            })
        );
        assert!(provider.validate().is_ok());
    }

    #[test]
    fn cost_provider_supports_typed_lists_and_uniform_costs() {
        let provider = EnchantmentProvider::by_cost(
            location("treasure"),
            EnchantmentSelection::many([
                EnchantmentId::minecraft("fortune").unwrap(),
                EnchantmentId::minecraft("silk_touch").unwrap(),
            ]),
            EnchantmentProviderInt::uniform(10, 30),
        );
        assert_eq!(
            provider.to_json(),
            json!({
                "type": "minecraft:by_cost",
                "enchantments": ["minecraft:fortune", "minecraft:silk_touch"],
                "cost": {
                    "type": "minecraft:uniform",
                    "min_inclusive": 10,
                    "max_inclusive": 30,
                },
            })
        );
        assert!(provider.validate().is_ok());
    }

    #[test]
    fn difficulty_provider_supports_typed_enchantment_tags() {
        let provider = EnchantmentProvider::by_cost_with_difficulty(
            location("mob_spawn_equipment"),
            TagId::<EnchantmentId>::minecraft("on_mob_spawn_equipment").unwrap(),
            5,
            17,
        );
        assert_eq!(
            provider.to_json(),
            json!({
                "type": "minecraft:by_cost_with_difficulty",
                "enchantments": "#minecraft:on_mob_spawn_equipment",
                "min_cost": 5,
                "max_cost_span": 17,
            })
        );
        assert_eq!(provider.component_dir(), "enchantment_provider");
    }

    #[test]
    fn invalid_typed_ranges_and_empty_lists_are_rejected() {
        let bad_level = EnchantmentProvider::single(
            location("bad_level"),
            EnchantmentId::minecraft("sharpness").unwrap(),
            0,
        );
        assert!(
            bad_level
                .validate()
                .unwrap_err()
                .to_string()
                .contains("level")
        );

        let bad_range = EnchantmentProvider::by_cost(
            location("bad_range"),
            EnchantmentId::minecraft("sharpness").unwrap(),
            EnchantmentProviderInt::uniform(30, 10),
        );
        assert!(
            bad_range
                .validate()
                .unwrap_err()
                .to_string()
                .contains("cost")
        );

        let empty =
            EnchantmentProvider::by_cost(location("empty"), EnchantmentSelection::many([]), 10);
        assert!(
            empty
                .validate()
                .unwrap_err()
                .to_string()
                .contains("enchantments")
        );
    }

    #[test]
    fn zero_difficulty_min_cost_is_rejected() {
        let provider = EnchantmentProvider::by_cost_with_difficulty(
            location("zero"),
            EnchantmentId::minecraft("sharpness").unwrap(),
            0,
            17,
        );
        assert!(
            provider
                .validate()
                .unwrap_err()
                .to_string()
                .contains("min_cost")
        );
    }

    #[test]
    fn raw_provider_preserves_modded_json_and_validates_its_wrapper() {
        let value = json!({
            "type": "mymod:weighted",
            "entries": [{"enchantment": "mymod:arcane", "weight": 2}],
        });
        let provider = EnchantmentProvider::raw(location("modded"), RawJson::new(value.clone()));
        assert!(provider.validate().is_ok());
        assert_eq!(provider.to_json(), value);

        let non_object =
            EnchantmentProvider::raw(location("bad"), RawJson::new(json!(["mymod:weighted"])));
        assert!(non_object.validate().is_err());
        let bad_type = EnchantmentProvider::raw(
            location("bad_type"),
            RawJson::new(json!({"type": "not namespaced"})),
        );
        assert!(bad_type.validate().is_err());
    }
}
