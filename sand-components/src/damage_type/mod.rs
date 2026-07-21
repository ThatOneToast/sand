//! Builders for `data/<namespace>/damage_type/` JSON files (Minecraft 1.19.4+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `message_id` must be non-empty, must not be whitespace-only, and must not
//!   contain control characters.
//! - `exhaustion` must be finite and non-negative. No upper bound is enforced.
//!
//! # Example
//! ```rust,ignore
//! let laser = DamageType::new(ResourceLocation::new("my_pack", "laser").unwrap())
//!     .message_id("laser")
//!     .exhaustion(0.1)
//!     .scaling(DamageScaling::Never)
//!     .effects(DamageEffects::Hurt);
//! ```

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

/// Controls how damage scales with difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageScaling {
    /// Damage does not scale with difficulty.
    Never,
    /// Damage scales when caused by a living non-player entity.
    WhenCausedByLivingNonPlayer,
    /// Damage always scales with difficulty.
    Always,
}

impl DamageScaling {
    pub fn as_str(self) -> &'static str {
        match self {
            DamageScaling::Never => "never",
            DamageScaling::WhenCausedByLivingNonPlayer => "when_caused_by_living_non_player",
            DamageScaling::Always => "always",
        }
    }
}

/// The visual/sound effect played when this damage type is dealt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

impl DamageEffects {
    pub fn as_str(self) -> &'static str {
        match self {
            DamageEffects::Hurt => "hurt",
            DamageEffects::Thorns => "thorns",
            DamageEffects::Drowning => "drowning",
            DamageEffects::Burning => "burning",
            DamageEffects::Poking => "poking",
            DamageEffects::Freezing => "freezing",
        }
    }
}

/// Controls the format of the death message for this damage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

impl DeathMessageType {
    pub fn as_str(self) -> &'static str {
        match self {
            DeathMessageType::Default => "default",
            DeathMessageType::FallVariants => "fall_variants",
            DeathMessageType::IntentionalGameDesign => "intentional_game_design",
        }
    }
}

/// A Minecraft damage type definition (`data/<namespace>/damage_type/<id>.json`).
pub struct DamageType {
    location: ResourceLocation,
    /// Translation key suffix for death messages (e.g. `"laser"` → `"death.attack.laser"`).
    message_id: String,
    /// Hunger exhaustion applied when this damage is taken (0.0–0.4 typical).
    exhaustion: f32,
    /// Whether the damage scales with difficulty.
    scaling: DamageScaling,
    /// Optional visual/sound effect.
    effects: Option<DamageEffects>,
    /// Optional death message format.
    death_message_type: Option<DeathMessageType>,
}

impl DamageType {
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            message_id: String::new(),
            exhaustion: 0.0,
            scaling: DamageScaling::WhenCausedByLivingNonPlayer,
            effects: None,
            death_message_type: None,
        }
    }

    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = id.into();
        self
    }

    pub fn exhaustion(mut self, exhaustion: f32) -> Self {
        self.exhaustion = exhaustion;
        self
    }

    pub fn scaling(mut self, scaling: DamageScaling) -> Self {
        self.scaling = scaling;
        self
    }

    pub fn effects(mut self, effects: DamageEffects) -> Self {
        self.effects = Some(effects);
        self
    }

    pub fn death_message_type(mut self, t: DeathMessageType) -> Self {
        self.death_message_type = Some(t);
        self
    }
}

impl DatapackComponent for DamageType {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "damage_type";
        validation::require_non_empty(&self.location, kind, "message_id", &self.message_id)?;
        validation::reject_whitespace_only(&self.location, kind, "message_id", &self.message_id)?;
        validation::reject_control_chars(&self.location, kind, "message_id", &self.message_id)?;
        validation::require_non_negative_f32(&self.location, kind, "exhaustion", self.exhaustion)?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "message_id".to_string(),
            Value::String(self.message_id.clone()),
        );
        map.insert("exhaustion".to_string(), serde_json::json!(self.exhaustion));
        map.insert(
            "scaling".to_string(),
            Value::String(self.scaling.as_str().to_string()),
        );
        if let Some(e) = self.effects {
            map.insert("effects".to_string(), Value::String(e.as_str().to_string()));
        }
        if let Some(d) = self.death_message_type {
            map.insert(
                "death_message_type".to_string(),
                Value::String(d.as_str().to_string()),
            );
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "damage_type"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::DamageTypes]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "spike").unwrap()
    }

    fn valid() -> DamageType {
        DamageType::new(rl()).message_id("spike").exhaustion(0.1)
    }

    #[test]
    fn valid_minimal_damage_type_exports_deterministic_json() {
        let dt = valid();
        assert!(dt.validate().is_ok());
        let json = dt.to_json();
        assert_eq!(json["message_id"], "spike");
        assert!(json["exhaustion"].as_f64().unwrap() - 0.1 < 0.001);
    }

    #[test]
    fn default_message_id_is_rejected() {
        let dt = DamageType::new(rl());
        let err = dt.validate().unwrap_err();
        assert!(err.to_string().contains("message_id"), "{err}");
    }

    #[test]
    fn empty_message_id_is_rejected() {
        let dt = DamageType::new(rl()).message_id("");
        assert!(dt.validate().is_err());
    }

    #[test]
    fn whitespace_only_message_id_is_rejected() {
        let dt = DamageType::new(rl()).message_id("   ");
        let err = dt.validate().unwrap_err();
        assert!(err.to_string().contains("whitespace"), "{err}");
    }

    #[test]
    fn control_char_message_id_is_rejected() {
        let dt = DamageType::new(rl()).message_id("foo\nbar");
        let err = dt.validate().unwrap_err();
        assert!(err.to_string().contains("control"), "{err}");
    }

    #[test]
    fn negative_exhaustion_is_rejected() {
        let dt = valid().exhaustion(-1.0);
        let err = dt.validate().unwrap_err();
        assert!(err.to_string().contains("exhaustion"), "{err}");
    }

    #[test]
    fn nan_exhaustion_is_rejected() {
        let dt = valid().exhaustion(f32::NAN);
        assert!(dt.validate().is_err());
    }

    #[test]
    fn positive_infinity_exhaustion_is_rejected() {
        let dt = valid().exhaustion(f32::INFINITY);
        assert!(dt.validate().is_err());
    }

    #[test]
    fn negative_infinity_exhaustion_is_rejected() {
        let dt = valid().exhaustion(f32::NEG_INFINITY);
        assert!(dt.validate().is_err());
    }

    #[test]
    fn zero_exhaustion_is_accepted() {
        let dt = valid().exhaustion(0.0);
        assert!(dt.validate().is_ok());
    }

    #[test]
    fn large_finite_exhaustion_is_accepted() {
        let dt = valid().exhaustion(1000.0);
        assert!(dt.validate().is_ok());
    }

    #[test]
    fn valid_damage_type_json_is_stable() {
        let dt = valid();
        let a = serde_json::to_string_pretty(&dt.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&dt.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn invalid_damage_type_fails_export() {
        let dt = DamageType::new(rl());
        assert!(dt.try_content().is_err());
    }
}
