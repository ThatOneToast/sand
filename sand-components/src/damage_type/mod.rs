//! Builders for `data/<namespace>/damage_type/` JSON files in Sand's 26+ baseline.
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::component::DamageScaling", module = "sand::component",
 summary = "Selects how native damage scales with difficulty.",
 context = "A typed option for a native damage type resource.",
 minecraft = "Serializes to the corresponding built-in value in damage_type JSON.",
 use_when = ["Defining a native damage type"],
 avoid_when = ["Changing the raw amount dealt by a command"],
 example = "let option = sand::component::DamageScaling::Never;",
 variants(Never = "Do not scale.", WhenCausedByLivingNonPlayer = "Scale damage from living non-player attackers.", Always = "Always scale with difficulty."),
)]
pub enum DamageScaling {
    /// Damage does not scale with difficulty.
    Never,
    /// Damage scales when caused by a living non-player entity.
    WhenCausedByLivingNonPlayer,
    /// Damage always scales with difficulty.
    Always,
}

impl DamageScaling {
    /// Returns the exact identifier used in native damage type JSON.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageScaling::as_str", module = "sand::component", kind = "method",
        summary = "Returns the native damage option identifier.", context = "The typed resource builder uses this same serialization.",
        minecraft = "Returns the value accepted by the damage type codec.", use_when = ["Inspecting a native option"],
        avoid_when = ["Supplying this enum directly to a typed builder"], returns = "Static native option identifier.",
        example = "let name = sand::component::DamageScaling::Never.as_str();",
    )]
    pub fn as_str(self) -> &'static str {
        match self {
            DamageScaling::Never => "never",
            DamageScaling::WhenCausedByLivingNonPlayer => "when_caused_by_living_non_player",
            DamageScaling::Always => "always",
        }
    }
}

/// The visual/sound effect played when this damage type is dealt.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::component::DamageEffects", module = "sand::component",
 summary = "Selects the native effect when an entity takes damage.",
 context = "A typed option for a native damage type resource.",
 minecraft = "Serializes to the corresponding built-in value in damage_type JSON.",
 use_when = ["Defining a native damage type"],
 avoid_when = ["Changing the raw amount dealt by a command"],
 example = "let option = sand::component::DamageEffects::Hurt;",
 variants(Hurt = "Normal hurt effect.", Thorns = "Thorns effect.", Drowning = "Drowning effect.", Burning = "Burning effect.", Poking = "Poking effect.", Freezing = "Freezing effect."),
)]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

impl DamageEffects {
    /// Returns the exact identifier used in native damage type JSON.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageEffects::as_str", module = "sand::component", kind = "method",
        summary = "Returns the native damage option identifier.", context = "The typed resource builder uses this same serialization.",
        minecraft = "Returns the value accepted by the damage type codec.", use_when = ["Inspecting a native option"],
        avoid_when = ["Supplying this enum directly to a typed builder"], returns = "Static native option identifier.",
        example = "let name = sand::component::DamageEffects::Hurt.as_str();",
    )]
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::component::DeathMessageType", module = "sand::component",
 summary = "Selects the native death message format.",
 context = "A typed option for a native damage type resource.",
 minecraft = "Serializes to the corresponding built-in value in damage_type JSON.",
 use_when = ["Defining a native damage type"],
 avoid_when = ["Changing the raw amount dealt by a command"],
 example = "let option = sand::component::DeathMessageType::Default;",
 variants(Default = "Default death message.", FallVariants = "Fall-specific variants.", IntentionalGameDesign = "Intentional-game-design suffix."),
)]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

impl DeathMessageType {
    /// Returns the exact identifier used in native damage type JSON.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DeathMessageType::as_str", module = "sand::component", kind = "method",
        summary = "Returns the native damage option identifier.", context = "The typed resource builder uses this same serialization.",
        minecraft = "Returns the value accepted by the damage type codec.", use_when = ["Inspecting a native option"],
        avoid_when = ["Supplying this enum directly to a typed builder"], returns = "Static native option identifier.",
        example = "let name = sand::component::DeathMessageType::Default.as_str();",
    )]
    pub fn as_str(self) -> &'static str {
        match self {
            DeathMessageType::Default => "default",
            DeathMessageType::FallVariants => "fall_variants",
            DeathMessageType::IntentionalGameDesign => "intentional_game_design",
        }
    }
}

fn default_scaling() -> DamageScaling {
    DamageScaling::WhenCausedByLivingNonPlayer
}

/// A Minecraft damage type definition. The portable representation includes
/// the resource identity. Resource export emits the native JSON body.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[sand_macros::api(
 registry = sand_api_contract, path = "sand::component::DamageType", module = "sand::component",
 summary = "Defines a native damage type resource.",
 context = "Use the resource's identity with the damage command to select its difficulty, hurt effects, and death message.",
 minecraft = "Exports a validated damage_type JSON resource.",
 use_when = ["Adding a native source of gameplay damage"],
 avoid_when = ["Changing custom health or shield values"],
 example = r#"let device = sand::component::DamageType::new("demo:spike".parse().unwrap()).message_id("spike");"#,
)]
pub struct DamageType {
    #[serde(rename = "id")]
    #[schemars(with = "String")]
    location: ResourceLocation,
    /// Translation key suffix for death messages (e.g. `"laser"` → `"death.attack.laser"`).
    message_id: String,
    /// Hunger exhaustion applied when this damage is taken (0.0–0.4 typical).
    /// Must be finite and non-negative. No upper bound is enforced.
    #[serde(default)]
    exhaustion: f32,
    /// Whether the damage scales with difficulty.
    #[serde(default = "default_scaling")]
    scaling: DamageScaling,
    /// Optional visual/sound effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effects: Option<DamageEffects>,
    /// Optional death message format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    death_message_type: Option<DeathMessageType>,
}

impl DamageType {
    /// Begins a native damage type definition. Set a message ID before export.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::new", module = "sand::component", kind = "method",
        summary = "Begins a native damage type definition.", context = "Set a message ID before export.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(location = "The resource identity."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap());"#,
    )]
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

    /// Sets the suffix of the death message key. The suffix must contain text and no control characters.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::message_id", module = "sand::component", kind = "method",
        summary = "Sets the suffix of the death message key.", context = "The suffix must contain text and no control characters.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(id = "Suffix of the death.attack translation key."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap()).message_id("spike");"#,
    )]
    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = id.into();
        self
    }

    /// Sets how much hunger exhaustion this damage causes. The value is stored in the native exhaustion field.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::exhaustion", module = "sand::component", kind = "method",
        summary = "Sets how much hunger exhaustion this damage causes.", context = "The value is stored in the native exhaustion field.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(exhaustion = "Finite, non-negative hunger exhaustion."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap()).exhaustion(0.1);"#,
    )]
    pub fn exhaustion(mut self, exhaustion: f32) -> Self {
        self.exhaustion = exhaustion;
        self
    }

    /// Controls whether difficulty changes this damage. The native damage type codec reads the chosen scaling mode.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::scaling", module = "sand::component", kind = "method",
        summary = "Controls whether difficulty changes this damage.", context = "The native damage type codec reads the chosen scaling mode.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(scaling = "Difficulty scaling policy."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap()).scaling(sand::component::DamageScaling::Never);"#,
    )]
    pub fn scaling(mut self, scaling: DamageScaling) -> Self {
        self.scaling = scaling;
        self
    }

    /// Sets the effect played after a hit. The native damage type codec reads this effect.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::effects", module = "sand::component", kind = "method",
        summary = "Sets the effect played after a hit.", context = "The native damage type codec reads this effect.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(effects = "Native damage effect."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap()).effects(sand::component::DamageEffects::Thorns);"#,
    )]
    pub fn effects(mut self, effects: DamageEffects) -> Self {
        self.effects = Some(effects);
        self
    }

    /// Sets which native format handles the death message. The selected format controls the message suffix and parameters.
    #[sand_macros::api(
        registry = sand_api_contract, path = "sand::component::DamageType::death_message_type", module = "sand::component", kind = "method",
        summary = "Sets which native format handles the death message.", context = "The selected format controls the message suffix and parameters.",
        minecraft = "Defines the corresponding part of the native damage type resource.",
        use_when = ["Authoring a native damage subtype"], avoid_when = ["Configuring an individual damage command"],
        params(t = "Native death message format."), returns = "The damage type definition.",
        example = r#"let definition = sand::component::DamageType::new("demo:spike".parse().unwrap()).death_message_type(sand::component::DeathMessageType::Default);"#,
    )]
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
