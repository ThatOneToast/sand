//! Builder for `data/<namespace>/predicate/<id>.json`.
//!
//! [`Predicate`] wraps a [`PredicateRoot`], a dedicated typed model for the
//! common vanilla predicate condition shapes — boolean composition
//! (`all_of`/`any_of`/`inverted`), entity/location/weather/time checks,
//! random chance, and references to other predicate files. It reuses the
//! shared typed predicate model from [`crate::predicates`]
//! ([`crate::predicates::EntityPredicate`], [`crate::predicates::LocationPredicate`],
//! [`crate::predicates::WeatherPredicate`], …) rather than duplicating those
//! shapes, and keeps an explicit [`crate::raw::RawJson`] escape hatch
//! (`PredicateRoot::Raw`) for predicate condition types Sand does not yet
//! model.
//!
//! A standalone predicate authored this way is exactly the same JSON shape
//! Minecraft loot tables use for conditions; [`PredicateRoot::from_loot_condition`]
//! converts a legacy [`LootCondition`] tree into a [`PredicateRoot`] where the
//! shapes overlap, so existing `LootCondition`-based authoring keeps working.
//!
//! ```rust
//! use sand_components::predicate::{EntityPredicateTarget, Predicate, PredicateRoot};
//! use sand_components::predicates::EntityPredicate;
//! use sand_components::{DatapackComponent, ResourceLocation};
//!
//! let is_zombie = Predicate::new(
//!     ResourceLocation::new("example", "is_zombie").unwrap(),
//!     PredicateRoot::entity_properties(
//!         EntityPredicateTarget::This,
//!         EntityPredicate::type_("minecraft:zombie"),
//!     ),
//! );
//! assert_eq!(is_zombie.component_dir(), "predicate");
//! assert_eq!(
//!     is_zombie.to_json()["condition"],
//!     "minecraft:entity_properties"
//! );
//! ```

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result, SandError};
use crate::loot_table::LootCondition;
use crate::predicates::{EntityPredicate, IntRange, LocationPredicate, WeatherPredicate};
use crate::raw::RawJson;
use crate::registry::PredicateId;
use crate::resource_location::ResourceLocation;

// ── EntityPredicateTarget ───────────────────────────────────────────────────

/// The entity selector target for an `entity_properties` predicate condition.
///
/// Corresponds to the vanilla `entity` field: which loot-context entity the
/// nested [`EntityPredicate`] is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityPredicateTarget {
    /// The entity the loot/predicate context is centered on.
    This,
    /// The entity that killed `this`.
    Killer,
    /// The player that killed `this`, if any.
    KillerPlayer,
    /// The entity directly responsible for the damage (e.g. an arrow, not its shooter).
    DirectKiller,
    /// A custom/future target string, kept as an explicit typed extension point.
    Custom(String),
}

impl EntityPredicateTarget {
    fn as_str(&self) -> &str {
        match self {
            Self::This => "this",
            Self::Killer => "killer",
            Self::KillerPlayer => "killer_player",
            Self::DirectKiller => "direct_killer",
            Self::Custom(s) => s,
        }
    }
}

impl Serialize for EntityPredicateTarget {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

// ── PredicateRoot ────────────────────────────────────────────────────────────

/// The typed condition tree of a standalone predicate file.
///
/// Mirrors the subset of vanilla predicate/loot-condition shapes most
/// commonly reused across advancements, loot tables, selectors, and
/// `/execute if predicate`: boolean composition, entity/location/weather/time
/// checks, random chance, and references to other predicate files. Use
/// [`PredicateRoot::raw`] for condition types not yet modeled.
#[derive(Debug, Clone)]
pub enum PredicateRoot {
    /// All nested conditions must be true (AND). Must not be empty.
    AllOf(Vec<PredicateRoot>),
    /// At least one nested condition must be true (OR). Must not be empty.
    AnyOf(Vec<PredicateRoot>),
    /// Inverted logic (NOT).
    Inverted(Box<PredicateRoot>),
    /// Checks properties of a loot/predicate-context entity.
    EntityProperties(EntityPredicateTarget, Box<EntityPredicate>),
    /// Checks the current location (block, biome, dimension, position ranges).
    LocationCheck(Box<LocationPredicate>),
    /// Checks current weather state.
    WeatherCheck(WeatherPredicate),
    /// Checks the current game time. Uses [`IntRange`] for the day-time value.
    TimeCheck(IntRange),
    /// Random probability check (`0.0..=1.0`).
    RandomChance(f64),
    /// A reference to another standalone predicate file.
    Reference(PredicateId),
    /// Raw escape hatch for predicate condition types not yet modeled.
    Raw(RawJson),
}

impl PredicateRoot {
    /// `minecraft:entity_properties` — checks properties of a loot/predicate-context entity.
    pub fn entity_properties(target: EntityPredicateTarget, predicate: EntityPredicate) -> Self {
        Self::EntityProperties(target, Box::new(predicate))
    }

    /// `minecraft:location_check` — checks the current location.
    pub fn location(predicate: LocationPredicate) -> Self {
        Self::LocationCheck(Box::new(predicate))
    }

    /// `minecraft:weather_check` — checks current weather state.
    pub fn weather(predicate: WeatherPredicate) -> Self {
        Self::WeatherCheck(predicate)
    }

    /// `minecraft:time_check` — checks the current game time against a range.
    pub fn time(range: IntRange) -> Self {
        Self::TimeCheck(range)
    }

    /// `minecraft:random_chance` — random probability check.
    pub fn random_chance(chance: f64) -> Self {
        Self::RandomChance(chance)
    }

    /// `minecraft:reference` — reference to another standalone predicate file.
    pub fn reference(id: impl Into<PredicateId>) -> Self {
        Self::Reference(id.into())
    }

    /// `minecraft:inverted` — negates a nested condition.
    pub fn inverted(term: PredicateRoot) -> Self {
        Self::Inverted(Box::new(term))
    }

    /// Explicit raw escape hatch for predicate condition types not yet modeled.
    pub fn raw(json: RawJson) -> Self {
        Self::Raw(json)
    }

    /// Convert a [`LootCondition`] into a [`PredicateRoot`] where the shapes
    /// overlap (boolean composition, weather, and reference). Loot-only
    /// condition variants (`KilledByPlayer`, `MatchTool`, `SurvivesExplosion`,
    /// `TableBonus`, `EntityScores`, `BlockStateProperty`, `EntityProperties`,
    /// `TimeCheck`, `Custom`, or an unparsable `Reference` name) fall back to
    /// [`PredicateRoot::Raw`] carrying the re-serialized loot-condition JSON,
    /// so the conversion never fails or silently drops data.
    pub fn from_loot_condition(condition: &LootCondition) -> Self {
        match condition {
            LootCondition::AllOf { terms } => {
                Self::AllOf(terms.iter().map(Self::from_loot_condition).collect())
            }
            LootCondition::AnyOf { terms } => {
                Self::AnyOf(terms.iter().map(Self::from_loot_condition).collect())
            }
            LootCondition::Inverted { term } => {
                Self::Inverted(Box::new(Self::from_loot_condition(term)))
            }
            LootCondition::RandomChance { chance } => Self::RandomChance(*chance),
            LootCondition::WeatherCheck {
                raining,
                thundering,
            } => {
                let mut predicate = WeatherPredicate::new();
                if let Some(v) = raining {
                    predicate = predicate.raining(*v);
                }
                if let Some(v) = thundering {
                    predicate = predicate.thundering(*v);
                }
                Self::WeatherCheck(predicate)
            }
            LootCondition::Reference { name } => match name.parse::<ResourceLocation>() {
                Ok(rl) => Self::Reference(PredicateId::custom(rl)),
                Err(_) => Self::Raw(RawJson::new(loot_condition_to_value(condition))),
            },
            other => Self::Raw(RawJson::new(loot_condition_to_value(other))),
        }
    }

    fn validate_at(&self, path: &str) -> std::result::Result<(), (String, String)> {
        match self {
            Self::AllOf(terms) | Self::AnyOf(terms) => {
                if terms.is_empty() {
                    return Err((format!("{path}.terms"), "terms must not be empty".into()));
                }
                for (index, term) in terms.iter().enumerate() {
                    term.validate_at(&format!("{path}.terms[{index}]"))?;
                }
                Ok(())
            }
            Self::Inverted(term) => term.validate_at(&format!("{path}.term")),
            Self::EntityProperties(_, predicate) => predicate
                .validate_at(&format!("{path}.predicate"))
                .map_err(|message| (format!("{path}.predicate"), message)),
            Self::LocationCheck(predicate) => predicate
                .validate_at(path)
                .map_err(|message| (path.to_string(), message)),
            Self::WeatherCheck(predicate) => predicate
                .validate_at(path)
                .map_err(|message| (path.to_string(), message)),
            Self::TimeCheck(range) => range
                .validate_at(&format!("{path}.value"))
                .map_err(|message| (format!("{path}.value"), message)),
            Self::RandomChance(chance) => {
                if !chance.is_finite() || !(0.0..=1.0).contains(chance) {
                    return Err((
                        format!("{path}.chance"),
                        format!("chance must be finite and in 0.0..=1.0; received {chance}"),
                    ));
                }
                Ok(())
            }
            Self::Reference(_) | Self::Raw(_) => Ok(()),
        }
    }
}

fn loot_condition_to_value(condition: &LootCondition) -> Value {
    serde_json::to_value(condition)
        .unwrap_or_else(|error| panic!("loot condition serialization failed: {error}"))
}

impl Serialize for PredicateRoot {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::AllOf(terms) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:all_of")?;
                map.serialize_entry("terms", terms)?;
                map.end()
            }
            Self::AnyOf(terms) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:any_of")?;
                map.serialize_entry("terms", terms)?;
                map.end()
            }
            Self::Inverted(term) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:inverted")?;
                map.serialize_entry("term", term)?;
                map.end()
            }
            Self::EntityProperties(target, predicate) => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("condition", "minecraft:entity_properties")?;
                map.serialize_entry("entity", target)?;
                map.serialize_entry("predicate", predicate)?;
                map.end()
            }
            Self::LocationCheck(predicate) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:location_check")?;
                map.serialize_entry("predicate", predicate)?;
                map.end()
            }
            Self::WeatherCheck(predicate) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("condition", "minecraft:weather_check")?;
                // Weather fields are flattened directly onto the wrapper object,
                // matching the vanilla `minecraft:weather_check` shape.
                let value = serde_json::to_value(predicate).map_err(serde::ser::Error::custom)?;
                if let Value::Object(object) = value {
                    for (key, value) in object {
                        map.serialize_entry(&key, &value)?;
                    }
                }
                map.end()
            }
            Self::TimeCheck(range) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:time_check")?;
                map.serialize_entry("value", range)?;
                map.end()
            }
            Self::RandomChance(chance) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:random_chance")?;
                map.serialize_entry("chance", chance)?;
                map.end()
            }
            Self::Reference(id) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("condition", "minecraft:reference")?;
                map.serialize_entry("name", id)?;
                map.end()
            }
            Self::Raw(json) => {
                let mut map = serializer.serialize_map(None)?;
                if let Value::Object(object) = json.as_value() {
                    for (key, value) in object {
                        map.serialize_entry(key, value)?;
                    }
                } else {
                    return Err(serde::ser::Error::custom(
                        "PredicateRoot::Raw must be a JSON object",
                    ));
                }
                map.end()
            }
        }
    }
}

// ── Predicate ─────────────────────────────────────────────────────────────────

/// A Minecraft predicate that defines a condition that can be evaluated in commands,
/// advancements, selectors, or loot tables.
///
/// Emits to `data/<namespace>/predicate/<id>.json`.
pub struct Predicate {
    /// The resource location for this predicate.
    pub location: ResourceLocation,
    /// The typed condition tree for this predicate.
    pub root: PredicateRoot,
}

impl Predicate {
    /// Create a new predicate with the given resource location and typed condition tree.
    pub fn new(location: ResourceLocation, root: PredicateRoot) -> Self {
        Self { location, root }
    }

    /// Convenience constructor for `minecraft:all_of`.
    pub fn all_of(
        location: ResourceLocation,
        terms: impl IntoIterator<Item = PredicateRoot>,
    ) -> Self {
        Self::new(location, PredicateRoot::AllOf(terms.into_iter().collect()))
    }

    /// Convenience constructor for `minecraft:any_of`.
    pub fn any_of(
        location: ResourceLocation,
        terms: impl IntoIterator<Item = PredicateRoot>,
    ) -> Self {
        Self::new(location, PredicateRoot::AnyOf(terms.into_iter().collect()))
    }

    /// Construct a predicate directly from a legacy [`LootCondition`], converting
    /// it to a [`PredicateRoot`] via [`PredicateRoot::from_loot_condition`].
    ///
    /// Kept for compatibility with existing `LootCondition`-based authoring;
    /// prefer [`Predicate::new`] with a [`PredicateRoot`] for new code.
    pub fn from_loot_condition(location: ResourceLocation, condition: LootCondition) -> Self {
        Self::new(location, PredicateRoot::from_loot_condition(&condition))
    }
}

impl DatapackComponent for Predicate {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> Result<()> {
        self.root
            .validate_at("predicate")
            .map_err(|(field, message)| SandError::ComponentValidation {
                location: self.location.clone(),
                kind: "predicate".to_string(),
                field,
                message,
            })
    }

    fn to_json(&self) -> Value {
        serde_json::to_value(&self.root)
            .unwrap_or_else(|error| panic!("predicate serialization failed: {error}"))
    }

    fn try_content(&self) -> Result<ComponentContent> {
        self.validate()?;
        serde_json::to_value(&self.root)
            .map(ComponentContent::Json)
            .map_err(SandError::Serialization)
    }

    fn component_dir(&self) -> &'static str {
        "predicate"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::DatapackComponent;
    use crate::predicates::{EntityFlags, FloatRange};

    fn loc(path: &str) -> ResourceLocation {
        format!("test:{path}").parse().unwrap()
    }

    #[test]
    fn invalid_standalone_predicate_returns_contextual_error() {
        let predicate = Predicate::new(loc("bad_chance"), PredicateRoot::random_chance(1.5));
        let error = predicate.try_content().unwrap_err().to_string();
        assert!(error.contains("test:bad_chance"));
        assert!(error.contains("predicate"));
        assert!(error.contains("predicate.chance"));
    }

    #[test]
    fn valid_standalone_predicate_output_is_unchanged() {
        let predicate = Predicate::new(loc("valid_chance"), PredicateRoot::random_chance(0.5));
        assert_eq!(predicate.try_content().unwrap(), predicate.content());
    }

    #[test]
    fn random_chance_json_shape() {
        let predicate = Predicate::new(loc("rare_drop"), PredicateRoot::random_chance(0.1));
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:random_chance");
        assert!((json["chance"].as_f64().unwrap() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn entity_properties_json_shape() {
        let predicate = Predicate::new(
            loc("is_baby_zombie"),
            PredicateRoot::entity_properties(
                EntityPredicateTarget::This,
                EntityPredicate::type_("minecraft:zombie").flags(EntityFlags::new().baby(true)),
            ),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:entity_properties");
        assert_eq!(json["entity"], "this");
        assert_eq!(json["predicate"]["type"], "minecraft:zombie");
        assert_eq!(json["predicate"]["flags"]["is_baby"], true);
    }

    #[test]
    fn location_check_json_shape() {
        let predicate = Predicate::new(
            loc("in_overworld"),
            PredicateRoot::location(LocationPredicate::new().dimension("minecraft:overworld")),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:location_check");
        assert_eq!(json["predicate"]["dimension"], "minecraft:overworld");
    }

    #[test]
    fn weather_check_json_shape() {
        let predicate = Predicate::new(
            loc("is_raining"),
            PredicateRoot::weather(WeatherPredicate::new().raining(true)),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:weather_check");
        assert_eq!(json["raining"], true);
        assert!(json.get("thundering").is_none());
    }

    #[test]
    fn time_check_json_shape() {
        let predicate = Predicate::new(
            loc("is_day"),
            PredicateRoot::time(IntRange::between(0, 12000)),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:time_check");
        assert_eq!(json["value"]["min"], 0);
        assert_eq!(json["value"]["max"], 12000);
    }

    #[test]
    fn reference_json_shape() {
        let predicate = Predicate::new(
            loc("wraps_other"),
            PredicateRoot::reference(PredicateId::minecraft("some_condition").unwrap()),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:reference");
        assert_eq!(json["name"], "minecraft:some_condition");
    }

    #[test]
    fn raw_escape_hatch_round_trips_arbitrary_object() {
        let predicate = Predicate::new(
            loc("mod_condition"),
            PredicateRoot::raw(RawJson::new(serde_json::json!({
                "condition": "mymod:custom_condition",
                "level": 10
            }))),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "mymod:custom_condition");
        assert_eq!(json["level"], 10);
    }

    #[test]
    fn raw_escape_hatch_rejects_non_object() {
        let predicate = Predicate::new(
            loc("bad_raw"),
            PredicateRoot::raw(RawJson::new(serde_json::json!(5))),
        );
        assert!(predicate.try_content().is_err());
    }

    #[test]
    fn all_of_composition_round_trips() {
        let predicate = Predicate::all_of(
            loc("composed"),
            [
                PredicateRoot::random_chance(0.25),
                PredicateRoot::weather(WeatherPredicate::new().raining(true)),
                PredicateRoot::inverted(PredicateRoot::random_chance(0.5)),
            ],
        );
        let expected = serde_json::json!({
            "condition": "minecraft:all_of",
            "terms": [
                {"condition": "minecraft:random_chance", "chance": 0.25},
                {"condition": "minecraft:weather_check", "raining": true},
                {"condition": "minecraft:inverted", "term": {"condition": "minecraft:random_chance", "chance": 0.5}}
            ]
        });
        assert_eq!(predicate.to_json(), expected);
        assert_eq!(predicate.try_content().unwrap(), predicate.content());
    }

    #[test]
    fn any_of_empty_terms_rejected() {
        let predicate = Predicate::any_of(loc("empty"), []);
        let error = predicate.try_content().unwrap_err().to_string();
        assert!(error.contains("terms"));
    }

    #[test]
    fn location_x_range_validation_propagates() {
        let predicate = Predicate::new(
            loc("bad_location"),
            PredicateRoot::location(LocationPredicate::new().x(FloatRange::between(10.0, 5.0))),
        );
        assert!(predicate.try_content().is_err());
    }

    #[test]
    fn from_loot_condition_converts_overlapping_shapes() {
        let predicate = Predicate::from_loot_condition(
            loc("converted"),
            LootCondition::AllOf {
                terms: vec![
                    LootCondition::RandomChance { chance: 0.25 },
                    LootCondition::WeatherCheck {
                        raining: Some(true),
                        thundering: None,
                    },
                ],
            },
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:all_of");
        let terms = json["terms"].as_array().unwrap();
        assert_eq!(terms[0]["condition"], "minecraft:random_chance");
        assert_eq!(terms[1]["condition"], "minecraft:weather_check");
    }

    #[test]
    fn from_loot_condition_falls_back_to_raw_for_loot_only_shapes() {
        let predicate =
            Predicate::from_loot_condition(loc("killed_by_player"), LootCondition::KilledByPlayer);
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:killed_by_player");
    }
}
