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
//! (`PredicateRoot::raw`) for predicate condition types Sand does not yet
//! model.
//!
//! ```rust
//! use sand_components::predicate::{EntityPredicateTarget, Predicate, PredicateRoot};
//! use sand_components::predicates::EntityPredicate;
//! use sand_components::{DatapackComponent, EntityTypeId, PredicateId};
//!
//! let is_zombie = Predicate::new(
//!     PredicateId::minecraft("is_zombie").unwrap(),
//!     PredicateRoot::entity_properties(
//!         EntityPredicateTarget::This,
//!         EntityPredicate::type_(EntityTypeId::minecraft("zombie").unwrap()),
//!     ),
//! );
//! assert_eq!(is_zombie.component_dir(), "predicate");
//! assert_eq!(
//!     is_zombie.to_json()["condition"],
//!     "minecraft:entity_properties"
//! );
//! ```

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result, SandError};
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
}

impl EntityPredicateTarget {
    fn as_str(&self) -> &str {
        match self {
            Self::This => "this",
            Self::Killer => "killer",
            Self::KillerPlayer => "killer_player",
            Self::DirectKiller => "direct_killer",
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
pub struct PredicateRoot(PredicateRootKind);

#[derive(Debug, Clone)]
enum PredicateRootKind {
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
    /// `minecraft:all_of` — requires every nested condition to succeed.
    pub fn all_of(terms: impl IntoIterator<Item = PredicateRoot>) -> Self {
        Self(PredicateRootKind::AllOf(terms.into_iter().collect()))
    }

    /// `minecraft:any_of` — requires at least one nested condition to succeed.
    pub fn any_of(terms: impl IntoIterator<Item = PredicateRoot>) -> Self {
        Self(PredicateRootKind::AnyOf(terms.into_iter().collect()))
    }

    /// `minecraft:entity_properties` — checks properties of a loot/predicate-context entity.
    pub fn entity_properties(target: EntityPredicateTarget, predicate: EntityPredicate) -> Self {
        Self(PredicateRootKind::EntityProperties(
            target,
            Box::new(predicate),
        ))
    }

    /// `minecraft:location_check` — checks the current location.
    pub fn location(predicate: LocationPredicate) -> Self {
        Self(PredicateRootKind::LocationCheck(Box::new(predicate)))
    }

    /// `minecraft:weather_check` — checks current weather state.
    pub fn weather(predicate: WeatherPredicate) -> Self {
        Self(PredicateRootKind::WeatherCheck(predicate))
    }

    /// `minecraft:time_check` — checks the current game time against a range.
    pub fn time(range: IntRange) -> Self {
        Self(PredicateRootKind::TimeCheck(range))
    }

    /// `minecraft:random_chance` — random probability check.
    pub fn random_chance(chance: f64) -> Self {
        Self(PredicateRootKind::RandomChance(chance))
    }

    /// `minecraft:reference` — reference to another standalone predicate file.
    pub fn reference(id: PredicateId) -> Self {
        Self(PredicateRootKind::Reference(id))
    }

    /// `minecraft:inverted` — negates a nested condition.
    pub fn inverted(term: PredicateRoot) -> Self {
        Self(PredicateRootKind::Inverted(Box::new(term)))
    }

    /// Explicit raw escape hatch for predicate condition types not yet modeled.
    pub fn raw(json: RawJson) -> serde_json::Result<Self> {
        let object =
            serde_json::from_value::<serde_json::Map<String, Value>>(json.as_value().clone())?;
        Ok(Self(PredicateRootKind::Raw(RawJson::new(Value::Object(
            object,
        )))))
    }

    fn validate_at(&self, path: &str) -> std::result::Result<(), (String, String)> {
        match &self.0 {
            PredicateRootKind::AllOf(terms) | PredicateRootKind::AnyOf(terms) => {
                if terms.is_empty() {
                    return Err((format!("{path}.terms"), "terms must not be empty".into()));
                }
                for (index, term) in terms.iter().enumerate() {
                    term.validate_at(&format!("{path}.terms[{index}]"))?;
                }
                Ok(())
            }
            PredicateRootKind::Inverted(term) => term.validate_at(&format!("{path}.term")),
            PredicateRootKind::EntityProperties(_, predicate) => predicate
                .validate_at(&format!("{path}.predicate"))
                .map_err(|message| (format!("{path}.predicate"), message)),
            PredicateRootKind::LocationCheck(predicate) => predicate
                .validate_at(path)
                .map_err(|message| (path.to_string(), message)),
            PredicateRootKind::WeatherCheck(predicate) => predicate
                .validate_at(path)
                .map_err(|message| (path.to_string(), message)),
            PredicateRootKind::TimeCheck(range) => range
                .validate_at(&format!("{path}.value"))
                .map_err(|message| (format!("{path}.value"), message)),
            PredicateRootKind::RandomChance(chance) => {
                if !chance.is_finite() || !(0.0..=1.0).contains(chance) {
                    return Err((
                        format!("{path}.chance"),
                        format!("chance must be finite and in 0.0..=1.0; received {chance}"),
                    ));
                }
                Ok(())
            }
            PredicateRootKind::Reference(_) => Ok(()),
            PredicateRootKind::Raw(json) if json.as_value().is_object() => Ok(()),
            PredicateRootKind::Raw(_) => Err((
                path.to_string(),
                "raw predicate roots must be JSON objects".into(),
            )),
        }
    }
}

impl PredicateRoot {
    fn render_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> std::result::Result<Value, String> {
        match &self.0 {
            PredicateRootKind::AllOf(terms) => render_terms("minecraft:all_of", terms, caps),
            PredicateRootKind::AnyOf(terms) => render_terms("minecraft:any_of", terms, caps),
            PredicateRootKind::Inverted(term) => Ok(serde_json::json!({
                "condition": "minecraft:inverted",
                "term": term.render_for(caps)?,
            })),
            PredicateRootKind::EntityProperties(target, predicate) => Ok(serde_json::json!({
                "condition": "minecraft:entity_properties",
                "entity": target,
                "predicate": predicate.render_for_advancement(caps)?,
            })),
            PredicateRootKind::LocationCheck(predicate) => Ok(serde_json::json!({
                "condition": "minecraft:location_check",
                "predicate": predicate.render_for_advancement(caps)?,
            })),
            PredicateRootKind::WeatherCheck(predicate) => {
                let mut object = serde_json::to_value(predicate)
                    .map_err(|error| error.to_string())?
                    .as_object()
                    .cloned()
                    .ok_or_else(|| {
                        "weather predicates must serialize as JSON objects".to_string()
                    })?;
                object.insert(
                    "condition".into(),
                    Value::String("minecraft:weather_check".into()),
                );
                Ok(Value::Object(object))
            }
            PredicateRootKind::TimeCheck(range) => Ok(serde_json::json!({
                "condition": "minecraft:time_check",
                "value": range,
            })),
            PredicateRootKind::RandomChance(chance) => Ok(serde_json::json!({
                "condition": "minecraft:random_chance",
                "chance": chance,
            })),
            PredicateRootKind::Reference(id) => Ok(serde_json::json!({
                "condition": "minecraft:reference",
                "name": id,
            })),
            PredicateRootKind::Raw(json) if json.as_value().is_object() => {
                Ok(json.as_value().clone())
            }
            PredicateRootKind::Raw(_) => Err("raw predicate roots must be JSON objects".into()),
        }
    }
}

fn render_terms(
    condition: &str,
    terms: &[PredicateRoot],
    caps: Option<&sand_version::VersionCaps>,
) -> std::result::Result<Value, String> {
    let terms = terms
        .iter()
        .map(|term| term.render_for(caps))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(serde_json::json!({"condition": condition, "terms": terms}))
}

// ── Predicate ─────────────────────────────────────────────────────────────────

/// A Minecraft predicate that defines a condition that can be evaluated in commands,
/// advancements, selectors, or loot tables.
///
/// Emits to `data/<namespace>/predicate/<id>.json`.
pub struct Predicate {
    location: PredicateId,
    root: PredicateRoot,
}

impl Predicate {
    /// Create a new predicate with the given typed predicate ID and condition tree.
    pub fn new(location: PredicateId, root: PredicateRoot) -> Self {
        Self { location, root }
    }

    /// Convenience constructor for `minecraft:all_of`.
    pub fn all_of(location: PredicateId, terms: impl IntoIterator<Item = PredicateRoot>) -> Self {
        Self::new(location, PredicateRoot::all_of(terms))
    }

    /// Convenience constructor for `minecraft:any_of`.
    pub fn any_of(location: PredicateId, terms: impl IntoIterator<Item = PredicateRoot>) -> Self {
        Self::new(location, PredicateRoot::any_of(terms))
    }
}

impl DatapackComponent for Predicate {
    fn resource_location(&self) -> &ResourceLocation {
        self.location.as_resource_location()
    }

    fn validate(&self) -> Result<()> {
        self.root
            .validate_at("predicate")
            .map_err(|(field, message)| SandError::ComponentValidation {
                location: self.location.as_resource_location().clone(),
                kind: "predicate".to_string(),
                field,
                message,
            })
    }

    fn to_json(&self) -> Value {
        self.root
            .render_for(None)
            .unwrap_or_else(|error| panic!("predicate serialization failed: {error}"))
    }

    fn try_content(&self) -> Result<ComponentContent> {
        self.try_content_for(None)
    }

    fn try_content_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<ComponentContent> {
        self.validate()?;
        self.root
            .render_for(caps)
            .map(ComponentContent::Json)
            .map_err(|message| SandError::ComponentValidation {
                location: self.location.as_resource_location().clone(),
                kind: "predicate".into(),
                field: "predicate".into(),
                message,
            })
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
    use crate::registry::{BiomeId, DimensionId, EntityTypeId};

    fn loc(path: &str) -> PredicateId {
        PredicateId::custom(format!("test:{path}").parse().unwrap())
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
                EntityPredicate::type_(EntityTypeId::minecraft("zombie").unwrap())
                    .flags(EntityFlags::new().baby(true)),
            ),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "minecraft:entity_properties");
        assert_eq!(json["entity"], "this");
        assert_eq!(
            json["predicate"]["minecraft:entity_type"],
            "minecraft:zombie"
        );
        assert_eq!(json["predicate"]["minecraft:flags"]["is_baby"], true);
    }

    #[test]
    fn location_check_json_shape() {
        let predicate = Predicate::new(
            loc("in_overworld"),
            PredicateRoot::location(
                LocationPredicate::new().dimension(DimensionId::minecraft("overworld").unwrap()),
            ),
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
            })))
            .unwrap(),
        );
        let json = predicate.to_json();
        assert_eq!(json["condition"], "mymod:custom_condition");
        assert_eq!(json["level"], 10);
    }

    #[test]
    fn raw_escape_hatch_rejects_non_object() {
        let error = PredicateRoot::raw(RawJson::new(serde_json::json!(5))).unwrap_err();
        assert!(error.to_string().contains("map"));
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
    fn profiled_entity_predicate_uses_target_schema() {
        let predicate = Predicate::new(
            loc("profiled_entity"),
            PredicateRoot::entity_properties(
                EntityPredicateTarget::This,
                EntityPredicate::type_(EntityTypeId::minecraft("zombie").unwrap()),
            ),
        );
        let legacy = sand_version::VersionCaps::from_profile_flags(
            "1.21.4", false, false, true, true, true, true, true, true,
        );
        let modern = sand_version::VersionCaps::all_enabled();

        let ComponentContent::Json(legacy_json) = predicate.try_content_for(Some(&legacy)).unwrap()
        else {
            panic!("predicates are JSON components")
        };
        let ComponentContent::Json(modern_json) = predicate.try_content_for(Some(&modern)).unwrap()
        else {
            panic!("predicates are JSON components")
        };
        assert_eq!(legacy_json["predicate"]["type"], "minecraft:zombie");
        assert!(
            legacy_json["predicate"]
                .get("minecraft:entity_type")
                .is_none()
        );
        assert_eq!(
            modern_json["predicate"]["minecraft:entity_type"],
            "minecraft:zombie"
        );
        assert!(modern_json["predicate"].get("type").is_none());
    }

    #[test]
    fn profiled_location_predicate_uses_target_schema_and_rejects_unknown_legacy() {
        let predicate = Predicate::new(
            loc("profiled_location"),
            PredicateRoot::location(
                LocationPredicate::new().biome(BiomeId::minecraft("plains").unwrap()),
            ),
        );
        let supported = sand_version::VersionCaps::from_profile_flags(
            "1.21.4", false, false, true, true, true, true, true, true,
        );
        let unsupported = sand_version::VersionCaps::from_profile_flags(
            "1.18.2", false, false, false, false, false, false, false, false,
        );

        let ComponentContent::Json(json) = predicate.try_content_for(Some(&supported)).unwrap()
        else {
            panic!("predicates are JSON components")
        };
        assert_eq!(json["predicate"]["biomes"], "minecraft:plains");
        assert!(json["predicate"].get("biome").is_none());

        let error = predicate
            .try_content_for(Some(&unsupported))
            .unwrap_err()
            .to_string();
        assert!(error.contains("1.21.4+"));
    }
}
