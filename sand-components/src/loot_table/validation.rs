use std::str::FromStr;

use serde_json::Value;

use super::{
    LootCondition, LootEntry, LootFunction, LootPool, LootTable, LootTableType, NumberProvider,
};
use crate::resource_location::ResourceLocation;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ValidationFailure {
    pub(crate) path: String,
    pub(crate) message: String,
}

type Result<T = ()> = std::result::Result<T, ValidationFailure>;

fn fail(path: impl Into<String>, message: impl Into<String>) -> ValidationFailure {
    ValidationFailure {
        path: path.into(),
        message: message.into(),
    }
}

fn finite(value: f64, path: &str) -> Result {
    if value.is_finite() {
        Ok(())
    } else {
        Err(fail(path, "number must be finite"))
    }
}

fn probability(value: f64, path: &str) -> Result {
    finite(value, path)?;
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(fail(path, "probability must be between 0.0 and 1.0"))
    }
}

fn resource_id(value: &str, path: &str) -> Result {
    let id = value.strip_prefix('#').unwrap_or(value);
    ResourceLocation::from_str(id).map(|_| ()).map_err(|error| {
        fail(
            path,
            format!("invalid resource location `{value}`: {error}"),
        )
    })
}

fn raw_object(value: &Value, path: &str, reserved_key: &str) -> Result {
    if let Some(object) = value.as_object() {
        if object.contains_key(reserved_key) {
            return Err(fail(
                path,
                format!("custom raw data must not redefine reserved `{reserved_key}` field"),
            ));
        }
        Ok(())
    } else {
        Err(fail(
            path,
            "custom raw data must be a JSON object because its fields are merged into the wrapper",
        ))
    }
}

impl NumberProvider {
    pub(crate) fn validate_at(&self, path: &str) -> Result {
        match self {
            Self::Constant(value) => finite(*value, path),
            Self::Uniform { min, max } => {
                finite(*min, &format!("{path}.min"))?;
                finite(*max, &format!("{path}.max"))?;
                if min > max {
                    return Err(fail(
                        path,
                        format!("uniform minimum {min} must not exceed maximum {max}"),
                    ));
                }
                Ok(())
            }
            Self::Binomial { n, p } => {
                if *n < 0 {
                    return Err(fail(
                        format!("{path}.n"),
                        "binomial trial count must be non-negative",
                    ));
                }
                probability(*p, &format!("{path}.p"))
            }
            Self::Score { score, scale, .. } => {
                if score.is_empty() {
                    return Err(fail(
                        format!("{path}.score"),
                        "score objective must not be empty",
                    ));
                }
                if let Some(scale) = scale {
                    finite(*scale, &format!("{path}.scale"))?;
                }
                Ok(())
            }
        }
    }

    fn known_minimum(&self) -> Option<f64> {
        match self {
            Self::Constant(value) => Some(*value),
            Self::Uniform { min, .. } => Some(*min),
            Self::Binomial { .. } => Some(0.0),
            Self::Score { .. } => None,
        }
    }
}

impl LootCondition {
    /// Validate stable condition invariants before a component embeds this JSON.
    pub(crate) fn validate_at(&self, path: &str) -> Result {
        match self {
            Self::AllOf { terms } | Self::AnyOf { terms } => {
                if terms.is_empty() {
                    return Err(fail(format!("{path}.terms"), "terms must not be empty"));
                }
                for (index, term) in terms.iter().enumerate() {
                    term.validate_at(&format!("{path}.terms[{index}]"))?;
                }
            }
            Self::Inverted { term } => term.validate_at(&format!("{path}.term"))?,
            Self::RandomChance { chance } => probability(*chance, &format!("{path}.chance"))?,
            Self::TableBonus {
                enchantment,
                chances,
            } => {
                resource_id(enchantment, &format!("{path}.enchantment"))?;
                if chances.is_empty() {
                    return Err(fail(
                        format!("{path}.chances"),
                        "chance list must not be empty",
                    ));
                }
                for (index, chance) in chances.iter().enumerate() {
                    probability(*chance, &format!("{path}.chances[{index}]"))?;
                }
            }
            Self::BlockStateProperty { block, .. } => resource_id(block, &format!("{path}.block"))?,
            Self::EntityScores { scores, .. } if scores.is_empty() => {
                return Err(fail(
                    format!("{path}.scores"),
                    "entity score map must not be empty",
                ));
            }
            Self::WeatherCheck {
                raining: None,
                thundering: None,
            } => {
                return Err(fail(
                    path,
                    "weather check must specify raining or thundering",
                ));
            }
            Self::Reference { name } => resource_id(name, &format!("{path}.name"))?,
            Self::Custom { condition, data } => {
                resource_id(condition, &format!("{path}.condition"))?;
                raw_object(data.as_value(), &format!("{path}.data"), "condition")?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl LootFunction {
    pub(crate) fn validate_at(&self, path: &str) -> Result {
        match self {
            Self::SetCount { count, add } => {
                count.validate_at(&format!("{path}.count"))?;
                if !add && count.known_minimum().is_some_and(|minimum| minimum <= 0.0) {
                    return Err(fail(
                        format!("{path}.count"),
                        "non-additive item count must be greater than zero",
                    ));
                }
            }
            Self::SetDamage { damage, .. } => damage.validate_at(&format!("{path}.damage"))?,
            Self::EnchantWithLevels { levels, options } => {
                levels.validate_at(&format!("{path}.levels"))?;
                if let Some(options) = options {
                    resource_id(options, &format!("{path}.options"))?;
                }
            }
            Self::EnchantRandomly {
                options: Some(options),
                ..
            } => {
                if options.is_empty() {
                    return Err(fail(
                        format!("{path}.options"),
                        "enchantment options must not be empty when present",
                    ));
                }
                for (index, option) in options.iter().enumerate() {
                    resource_id(option, &format!("{path}.options[{index}]"))?;
                }
            }
            Self::LootingEnchant { count, .. } => count.validate_at(&format!("{path}.count"))?,
            Self::CopyComponents {
                include, exclude, ..
            } => {
                for (index, component) in include.iter().enumerate() {
                    resource_id(component, &format!("{path}.include[{index}]"))?;
                }
                for (index, component) in exclude.iter().enumerate() {
                    resource_id(component, &format!("{path}.exclude[{index}]"))?;
                }
            }
            Self::Reference { name } => resource_id(name, &format!("{path}.name"))?,
            Self::Custom { function, data } => {
                resource_id(function, &format!("{path}.function"))?;
                raw_object(data.as_value(), &format!("{path}.data"), "function")?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl LootEntry {
    pub(crate) fn validate_at(&self, path: &str) -> Result {
        let (conditions, functions, weight) = match self {
            Self::Item {
                name,
                weight,
                functions,
                conditions,
                ..
            } => {
                resource_id(name, &format!("{path}.name"))?;
                (conditions, Some(functions.as_slice()), *weight)
            }
            Self::Tag {
                name,
                weight,
                conditions,
                ..
            } => {
                resource_id(name, &format!("{path}.name"))?;
                (conditions, None, *weight)
            }
            Self::LootTable {
                value,
                weight,
                conditions,
                ..
            } => {
                resource_id(value, &format!("{path}.value"))?;
                (conditions, None, *weight)
            }
            Self::Dynamic { name, conditions } => {
                resource_id(name, &format!("{path}.name"))?;
                (conditions, None, None)
            }
            Self::Empty {
                weight, conditions, ..
            } => (conditions, None, *weight),
            Self::Group {
                children,
                conditions,
            }
            | Self::Alternatives {
                children,
                conditions,
            }
            | Self::Sequence {
                children,
                conditions,
            } => {
                if children.is_empty() {
                    return Err(fail(
                        format!("{path}.children"),
                        "entry children must not be empty",
                    ));
                }
                for (index, child) in children.iter().enumerate() {
                    child.validate_at(&format!("{path}.children[{index}]"))?;
                }
                (conditions, None, None)
            }
        };

        if weight.is_some_and(|weight| weight <= 0) {
            return Err(fail(
                format!("{path}.weight"),
                "entry weight must be greater than zero",
            ));
        }
        if let Some(functions) = functions {
            for (index, function) in functions.iter().enumerate() {
                function.validate_at(&format!("{path}.functions[{index}]"))?;
            }
        }
        for (index, condition) in conditions.iter().enumerate() {
            condition.validate_at(&format!("{path}.conditions[{index}]"))?;
        }
        Ok(())
    }
}

impl LootPool {
    fn validate_at(&self, path: &str) -> Result {
        if self.entries.is_empty() {
            return Err(fail(
                format!("{path}.entries"),
                "loot pool must contain at least one entry",
            ));
        }
        self.rolls.validate_at(&format!("{path}.rolls"))?;
        if self
            .rolls
            .known_minimum()
            .is_some_and(|minimum| minimum < 0.0)
        {
            return Err(fail(
                format!("{path}.rolls"),
                "loot pool rolls must not be negative",
            ));
        }
        if let Some(bonus_rolls) = &self.bonus_rolls {
            bonus_rolls.validate_at(&format!("{path}.bonus_rolls"))?;
            if bonus_rolls
                .known_minimum()
                .is_some_and(|minimum| minimum < 0.0)
            {
                return Err(fail(
                    format!("{path}.bonus_rolls"),
                    "loot pool bonus rolls must not be negative",
                ));
            }
        }
        for (index, condition) in self.conditions.iter().enumerate() {
            condition.validate_at(&format!("{path}.conditions[{index}]"))?;
        }
        for (index, function) in self.functions.iter().enumerate() {
            function.validate_at(&format!("{path}.functions[{index}]"))?;
        }
        for (index, entry) in self.entries.iter().enumerate() {
            entry.validate_at(&format!("{path}.entries[{index}]"))?;
        }
        Ok(())
    }
}

impl LootTable {
    pub(super) fn validate_table(&self) -> Result {
        if self.pools.is_empty()
            && self.functions.is_empty()
            && self.conditions.is_empty()
            && !matches!(self.loot_type, Some(LootTableType::Empty))
        {
            return Err(fail(
                "pools",
                "loot table must contain content or explicitly use LootTableType::Empty",
            ));
        }
        if let Some(LootTableType::Custom(kind)) = &self.loot_type {
            resource_id(kind, "type")?;
        }
        if let Some(sequence) = &self.random_sequence {
            resource_id(sequence, "random_sequence")?;
        }
        for (index, condition) in self.conditions.iter().enumerate() {
            condition.validate_at(&format!("conditions[{index}]"))?;
        }
        for (index, function) in self.functions.iter().enumerate() {
            function.validate_at(&format!("functions[{index}]"))?;
        }
        for (index, pool) in self.pools.iter().enumerate() {
            pool.validate_at(&format!("pools[{index}]"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::raw::RawJson;

    #[test]
    fn number_providers_reject_invalid_numeric_states() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                NumberProvider::Constant(value)
                    .validate_at("rolls")
                    .is_err()
            );
            assert!(
                NumberProvider::Uniform {
                    min: value,
                    max: 1.0
                }
                .validate_at("rolls")
                .is_err()
            );
            assert!(
                NumberProvider::Binomial { n: 1, p: value }
                    .validate_at("rolls")
                    .is_err()
            );
            assert!(
                NumberProvider::Score {
                    target: json!("this"),
                    score: "points".into(),
                    scale: Some(value),
                }
                .validate_at("rolls")
                .is_err()
            );
        }
        assert!(
            NumberProvider::Uniform { min: 2.0, max: 1.0 }
                .validate_at("rolls")
                .is_err()
        );
        assert!(
            NumberProvider::Binomial { n: -1, p: 0.5 }
                .validate_at("rolls")
                .is_err()
        );
    }

    #[test]
    fn empty_children_and_invalid_weight_are_rejected() {
        for entry in [
            LootEntry::group(Vec::new()),
            LootEntry::alternatives(Vec::new()),
            LootEntry::sequence(Vec::new()),
        ] {
            let empty = entry.validate_at("entries[0]").unwrap_err();
            assert_eq!(empty.path, "entries[0].children");
        }
        let weighted = LootEntry::Item {
            name: "minecraft:diamond".into(),
            weight: Some(0),
            quality: None,
            functions: Vec::new(),
            conditions: Vec::new(),
        }
        .validate_at("entries[1]")
        .unwrap_err();
        assert_eq!(weighted.path, "entries[1].weight");
    }

    #[test]
    fn custom_wrapper_requires_id_and_object_data() {
        let failure = LootCondition::Custom {
            condition: "bad id".into(),
            data: RawJson::new(json!({})),
        }
        .validate_at("conditions[0]")
        .unwrap_err();
        assert_eq!(failure.path, "conditions[0].condition");

        let failure = LootFunction::Custom {
            function: "mod:custom".into(),
            data: RawJson::new(json!(42)),
        }
        .validate_at("functions[0]")
        .unwrap_err();
        assert_eq!(failure.path, "functions[0].data");

        let failure = LootCondition::Custom {
            condition: "mod:custom".into(),
            data: RawJson::new(json!({ "condition": "mod:override" })),
        }
        .validate_at("conditions[0]")
        .unwrap_err();
        assert!(failure.message.contains("reserved"));
    }

    #[test]
    fn functions_validate_nested_number_providers_and_references() {
        let failure = LootFunction::SetDamage {
            damage: NumberProvider::Uniform {
                min: 1.0,
                max: f64::NAN,
            },
            add: false,
        }
        .validate_at("functions[2]")
        .unwrap_err();
        assert_eq!(failure.path, "functions[2].damage.max");

        let failure = LootFunction::Reference {
            name: "not namespaced".into(),
        }
        .validate_at("functions[0]")
        .unwrap_err();
        assert_eq!(failure.path, "functions[0].name");
    }

    #[test]
    fn table_and_pool_structural_invariants_are_explicit() {
        let location = ResourceLocation::new("test", "empty").unwrap();
        assert_eq!(
            LootTable::new(location.clone())
                .validate_table()
                .unwrap_err()
                .path,
            "pools"
        );
        assert!(
            LootTable::new(location)
                .loot_type(LootTableType::Empty)
                .validate_table()
                .is_ok()
        );

        let failure = LootTable::new(ResourceLocation::new("test", "pool").unwrap())
            .pool(LootPool::new())
            .validate_table()
            .unwrap_err();
        assert_eq!(failure.path, "pools[0].entries");
    }

    #[test]
    fn legacy_string_ids_are_checked_only_at_export_validation() {
        let table = LootTable::new(ResourceLocation::new("test", "ids").unwrap())
            .loot_type(LootTableType::Custom("bad type".into()))
            .random_sequence("test:sequence")
            .pool(LootPool::new().entry(LootEntry::item("minecraft:diamond")));
        assert_eq!(table.validate_table().unwrap_err().path, "type");

        let table = LootTable::new(ResourceLocation::new("test", "entry_id").unwrap())
            .pool(LootPool::new().entry(LootEntry::item("bad item")));
        assert_eq!(
            table.validate_table().unwrap_err().path,
            "pools[0].entries[0].name"
        );
    }
}
