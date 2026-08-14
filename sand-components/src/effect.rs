use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::error::Result;
use crate::registry::{PotionRegistryId, StatusEffectId};
use crate::resource_location::ResourceLocation;

/// A duration expressed in Minecraft game ticks (20 ticks = 1 second).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks(u32);

impl Ticks {
    pub const fn new(ticks: u32) -> Self {
        Self(ticks)
    }

    /// Convert seconds to ticks (`seconds * 20`).
    ///
    /// Saturates at `u32::MAX` instead of panicking (debug builds) or
    /// silently wrapping (release builds) on overflow — see #148. A duration
    /// large enough to saturate is already far beyond any meaningful game
    /// duration, so clamping is safe and avoids a build-profile-dependent
    /// footgun for a `const fn` used in `static` contexts.
    pub const fn seconds(seconds: u32) -> Self {
        Self(seconds.saturating_mul(20))
    }

    /// Convert minutes to ticks (`minutes * 1200`). See [`seconds`](Self::seconds)
    /// for the overflow-saturation policy.
    pub const fn minutes(minutes: u32) -> Self {
        Self(minutes.saturating_mul(1200))
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub const fn as_seconds(self) -> u32 {
        self.0 / 20
    }
}

impl fmt::Display for Ticks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Ticks {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0)
    }
}

macro_rules! vanilla_registry_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($variant:ident => $path:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                #[doc = concat!("The vanilla Minecraft `", $path, "` registry entry.")]
                $variant,
            )+
            Custom(ResourceLocation),
        }

        // This macro is the authoritative generator for these facade APIs.
        // Keep the installed catalog beside the generated Rust declarations so
        // a new enum variant cannot become public without contract metadata.
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name)),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Enum,
                signature: concat!("pub enum ", stringify!($name)),
                summary: "Names a typed vanilla Minecraft registry entry.",
                context: "This generated enum keeps supported vanilla identifiers discoverable and typo-resistant while retaining an explicit Custom escape hatch for modded data.",
                minecraft: "Serializes to the corresponding Minecraft namespace:path registry identifier.",
                use_when: &["Selecting a known vanilla registry entry", "Passing a typed identifier to a Sand API"],
                avoid_when: &["Representing an arbitrary unvalidated resource location"],
                parameters: &[],
                returns: None,
                example: concat!("let id = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?;"),
                availability: &[],
            }
        }

        $(
            ::sand_api_contract::inventory::submit! {
                ::sand_api_contract::ApiRegistration {
                    canonical_path: concat!("sand::prelude::", stringify!($name), "::", stringify!($variant)),
                    aliases: &[],
                    canonical_module: "sand::prelude",
                    kind: ::sand_api_contract::ApiKind::Variant,
                    signature: concat!(stringify!($variant)),
                    summary: concat!("Selects Minecraft's `", $path, "` registry entry."),
                    context: "This typed variant is the canonical Sand spelling for one built-in Minecraft registry identifier.",
                    minecraft: concat!("Serializes as minecraft:", $path, "."),
                    use_when: &["Using this exact vanilla registry entry"],
                    avoid_when: &["Selecting a different registry entry or a modded identifier"],
                    parameters: &[],
                    returns: None,
                    example: concat!("let id = sand::prelude::", stringify!($name), "::", stringify!($variant), ";"),
                    availability: &[],
                }
            }
        )+

        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::Custom"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Variant,
                signature: "Custom(ResourceLocation)",
                summary: "Carries a validated custom or modded registry identifier.",
                context: "The Custom variant preserves the typed API when an identifier is not one of Sand's known vanilla entries.",
                minecraft: "Serializes the contained namespace:path registry identifier unchanged.",
                use_when: &["Addressing a modded or otherwise non-vanilla registry entry"],
                avoid_when: &["A named vanilla variant is available"],
                parameters: &[],
                returns: None,
                example: concat!("let id = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?;"),
                availability: &[],
            }
        }
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::Custom::0"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Field,
                signature: "ResourceLocation",
                summary: "Stores the validated custom registry location.",
                context: "This tuple field is the exact resource location carried by the Custom variant.",
                minecraft: "Uses the stored namespace:path identifier when Sand renders the registry reference.",
                use_when: &["Inspecting a Custom variant's validated location"],
                avoid_when: &["Constructing a built-in variant"],
                parameters: &[],
                returns: None,
                example: concat!("let id = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?;"),
                availability: &[],
            }
        }

        impl $name {
            /// Parse and wrap a custom or modded `namespace:path` ID.
            pub fn custom(id: impl AsRef<str>) -> Result<Self> {
                Ok(Self::Custom(id.as_ref().parse()?))
            }

            /// Wrap an already validated resource location as a custom/modded ID.
            pub fn from_resource_location(location: ResourceLocation) -> Self {
                Self::Custom(location)
            }

            pub fn as_resource_location(&self) -> ResourceLocation {
                match self {
                    $(Self::$variant => ResourceLocation::minecraft($path).expect("valid vanilla registry path"),)+
                    Self::Custom(location) => location.clone(),
                }
            }

            pub fn as_str(&self) -> String {
                self.as_resource_location().to_string()
            }
        }

        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::custom"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Method,
                signature: concat!("pub fn ", stringify!($name), "::custom(id: impl AsRef<str>) -> Result<", stringify!($name), ">"),
                summary: "Validates and wraps a custom registry identifier.",
                context: "This constructor keeps modded identifiers in the same typed family as known vanilla variants.",
                minecraft: "Validates Minecraft namespace:path syntax before serializing it as a registry reference.",
                use_when: &["Accepting a modded or data-driven registry ID"],
                avoid_when: &["A named vanilla variant is available"],
                parameters: &[::sand_api_contract::StaticApiParameter { name: "id", description: "The custom namespace:path registry identifier." }],
                returns: Some("The typed custom registry identifier, or a validation error."),
                example: concat!("let id = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?;"),
                availability: &[],
            }
        }
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::from_resource_location"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Method,
                signature: concat!("pub fn ", stringify!($name), "::from_resource_location(location: ResourceLocation) -> ", stringify!($name)),
                summary: "Wraps an already validated location as a custom registry identifier.",
                context: "This avoids reparsing when an API already owns a ResourceLocation.",
                minecraft: "Uses the supplied namespace:path location as the registry reference.",
                use_when: &["Converting an existing ResourceLocation into this typed registry family"],
                avoid_when: &["Parsing an unchecked string; use custom instead"],
                parameters: &[::sand_api_contract::StaticApiParameter { name: "location", description: "The validated custom registry location." }],
                returns: Some("The matching Custom enum variant."),
                example: concat!("let id = sand::prelude::", stringify!($name), "::from_resource_location(sand::ResourceLocation::minecraft(\"entry\")?);"),
                availability: &[],
            }
        }
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::as_resource_location"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Method,
                signature: concat!("pub fn ", stringify!($name), "::as_resource_location(&self) -> ResourceLocation"),
                summary: "Returns this entry's validated resource location.",
                context: "This is the common typed identity representation used by Sand APIs that accept several registry kinds.",
                minecraft: "Returns the namespace:path value rendered in Minecraft data and commands.",
                use_when: &["Passing this registry entry to an API that accepts ResourceLocation"],
                avoid_when: &["Only display text is needed; use as_str or Display"],
                parameters: &[],
                returns: Some("The corresponding validated resource location."),
                example: concat!("let location = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?.as_resource_location();"),
                availability: &[],
            }
        }
        ::sand_api_contract::inventory::submit! {
            ::sand_api_contract::ApiRegistration {
                canonical_path: concat!("sand::prelude::", stringify!($name), "::as_str"),
                aliases: &[],
                canonical_module: "sand::prelude",
                kind: ::sand_api_contract::ApiKind::Method,
                signature: concat!("pub fn ", stringify!($name), "::as_str(&self) -> String"),
                summary: "Returns this entry as namespace:path text.",
                context: "Use this at text boundaries after keeping the identifier typed through the rest of the API.",
                minecraft: "Returns the exact registry identifier Minecraft reads.",
                use_when: &["Writing the identifier into a text-only integration"],
                avoid_when: &["Passing a typed registry ID to another Sand API"],
                parameters: &[],
                returns: Some("The namespace:path registry identifier text."),
                example: concat!("let name = sand::prelude::", stringify!($name), "::custom(\"example:entry\")?.as_str();"),
                availability: &[],
            }
        }

        impl From<ResourceLocation> for $name {
            fn from(location: ResourceLocation) -> Self {
                Self::Custom(location)
            }
        }

        impl From<$name> for ResourceLocation {
            fn from(id: $name) -> Self {
                id.as_resource_location()
            }
        }

        impl FromStr for $name {
            type Err = crate::error::SandError;

            fn from_str(s: &str) -> Result<Self> {
                match s {
                    $(concat!("minecraft:", $path) => Ok(Self::$variant),)+
                    _ => Ok(Self::Custom(s.parse()?)),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.as_resource_location())
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }
    };
}

vanilla_registry_enum! {
    /// Typed Minecraft status effect identifier.
    EffectId {
        Absorption => "absorption",
        BadOmen => "bad_omen",
        Blindness => "blindness",
        ConduitPower => "conduit_power",
        Darkness => "darkness",
        DolphinGrace => "dolphins_grace",
        FireResistance => "fire_resistance",
        Glowing => "glowing",
        Haste => "haste",
        HealthBoost => "health_boost",
        HeroOfTheVillage => "hero_of_the_village",
        Hunger => "hunger",
        Infested => "infested",
        InstantDamage => "instant_damage",
        InstantHealth => "instant_health",
        Invisibility => "invisibility",
        JumpBoost => "jump_boost",
        Levitation => "levitation",
        Luck => "luck",
        MiningFatigue => "mining_fatigue",
        Nausea => "nausea",
        NightVision => "night_vision",
        Oozing => "oozing",
        Poison => "poison",
        RaidOmen => "raid_omen",
        Regeneration => "regeneration",
        Resistance => "resistance",
        Saturation => "saturation",
        SlowFalling => "slow_falling",
        Slowness => "slowness",
        Speed => "speed",
        Strength => "strength",
        TrialOmen => "trial_omen",
        Unluck => "unluck",
        WaterBreathing => "water_breathing",
        Weakness => "weakness",
        Weaving => "weaving",
        WindCharged => "wind_charged",
        Wither => "wither",
    }
}

impl From<EffectId> for StatusEffectId {
    fn from(id: EffectId) -> Self {
        id.as_resource_location().into()
    }
}

impl From<StatusEffectId> for EffectId {
    fn from(id: StatusEffectId) -> Self {
        Self::from_resource_location(id.into())
    }
}

vanilla_registry_enum! {
    /// Typed Minecraft potion identifier.
    PotionId {
        Water => "water",
        Mundane => "mundane",
        Thick => "thick",
        Awkward => "awkward",
        NightVision => "night_vision",
        LongNightVision => "long_night_vision",
        Invisibility => "invisibility",
        LongInvisibility => "long_invisibility",
        Leaping => "leaping",
        LongLeaping => "long_leaping",
        StrongLeaping => "strong_leaping",
        FireResistance => "fire_resistance",
        LongFireResistance => "long_fire_resistance",
        Swiftness => "swiftness",
        LongSwiftness => "long_swiftness",
        StrongSwiftness => "strong_swiftness",
        Slowness => "slowness",
        LongSlowness => "long_slowness",
        StrongSlowness => "strong_slowness",
        TurtleMaster => "turtle_master",
        LongTurtleMaster => "long_turtle_master",
        StrongTurtleMaster => "strong_turtle_master",
        WaterBreathing => "water_breathing",
        LongWaterBreathing => "long_water_breathing",
        Healing => "healing",
        StrongHealing => "strong_healing",
        Harming => "harming",
        StrongHarming => "strong_harming",
        Poison => "poison",
        LongPoison => "long_poison",
        StrongPoison => "strong_poison",
        Regeneration => "regeneration",
        LongRegeneration => "long_regeneration",
        StrongRegeneration => "strong_regeneration",
        Strength => "strength",
        LongStrength => "long_strength",
        StrongStrength => "strong_strength",
        Weakness => "weakness",
        LongWeakness => "long_weakness",
        Luck => "luck",
        SlowFalling => "slow_falling",
        LongSlowFalling => "long_slow_falling",
    }
}

impl From<PotionId> for PotionRegistryId {
    fn from(id: PotionId) -> Self {
        id.as_resource_location().into()
    }
}

impl From<PotionRegistryId> for PotionId {
    fn from(id: PotionRegistryId) -> Self {
        Self::from_resource_location(id.into())
    }
}

/// Structured status effect instance for item components and datapack JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEffectInstance {
    pub effect: EffectId,
    pub(crate) duration: Option<Ticks>,
    pub(crate) amplifier: u8,
    pub(crate) ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
}

impl StatusEffectInstance {
    /// Create an effect instance from either the enum-style [`EffectId`] or
    /// shared resource-location-backed [`StatusEffectId`].
    pub fn new(effect: impl Into<EffectId>) -> Self {
        Self {
            effect: effect.into(),
            duration: None,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
        }
    }

    pub fn duration(mut self, duration: Ticks) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn seconds(self, seconds: u32) -> Self {
        self.duration(Ticks::seconds(seconds))
    }

    pub fn amplifier(mut self, amplifier: u8) -> Self {
        self.amplifier = amplifier;
        self
    }

    pub fn ambient(mut self, ambient: bool) -> Self {
        self.ambient = ambient;
        self
    }

    pub fn particles(mut self, show_particles: bool) -> Self {
        self.show_particles = show_particles;
        self
    }

    pub fn icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    pub fn to_snbt(&self) -> String {
        let mut parts = vec![format!("id:\"{}\"", self.effect)];
        if let Some(duration) = self.duration {
            parts.push(format!("duration:{}", duration.get()));
        }
        if self.amplifier != 0 {
            parts.push(format!("amplifier:{}", self.amplifier));
        }
        if self.ambient {
            parts.push("ambient:true".to_string());
        }
        if !self.show_particles {
            parts.push("show_particles:false".to_string());
        }
        if !self.show_icon {
            parts.push("show_icon:false".to_string());
        }
        format!("{{{}}}", parts.join(","))
    }
}

impl Serialize for StatusEffectInstance {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut entries = 2;
        entries += usize::from(self.duration.is_some());
        entries += usize::from(self.ambient);
        entries += usize::from(!self.show_particles);
        entries += usize::from(!self.show_icon);

        let mut map = serializer.serialize_map(Some(entries))?;
        map.serialize_entry("id", &self.effect)?;
        if let Some(duration) = self.duration {
            map.serialize_entry("duration", &duration)?;
        }
        map.serialize_entry("amplifier", &self.amplifier)?;
        if self.ambient {
            map.serialize_entry("ambient", &self.ambient)?;
        }
        if !self.show_particles {
            map.serialize_entry("show_particles", &self.show_particles)?;
        }
        if !self.show_icon {
            map.serialize_entry("show_icon", &self.show_icon)?;
        }
        map.end()
    }
}

/// `minecraft:potion_contents` item component data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PotionContents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) potion: Option<PotionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) custom_color: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_effects: Vec<StatusEffectInstance>,
}

impl PotionContents {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base potion using either the enum-style [`PotionId`] or shared
    /// resource-location-backed [`PotionRegistryId`].
    pub fn potion(mut self, potion: impl Into<PotionId>) -> Self {
        self.potion = Some(potion.into());
        self
    }

    pub fn custom_color(mut self, color: u32) -> Self {
        self.custom_color = Some(color);
        self
    }

    pub fn effect(mut self, effect: StatusEffectInstance) -> Self {
        self.custom_effects.push(effect);
        self
    }

    pub fn custom_effect(self, effect: StatusEffectInstance) -> Self {
        self.effect(effect)
    }

    pub fn to_snbt(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref potion) = self.potion {
            parts.push(format!("potion:\"{potion}\""));
        }
        if let Some(color) = self.custom_color {
            parts.push(format!("custom_color:{color}"));
        }
        if !self.custom_effects.is_empty() {
            let effects = self
                .custom_effects
                .iter()
                .map(StatusEffectInstance::to_snbt)
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("custom_effects:[{effects}]"));
        }
        format!("{{{}}}", parts.join(","))
    }
}

/// Suspicious stew effect entry.
///
/// Vanilla's `minecraft:suspicious_stew_effects` component uses the field
/// name `id` for the effect identifier (matching `to_snbt()` below); the JSON
/// (`Serialize`) form is renamed to match so both representations agree — see #148.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuspiciousStewEffect {
    #[serde(rename = "id")]
    pub effect: EffectId,
    pub duration: Ticks,
}

impl SuspiciousStewEffect {
    pub fn new(effect: impl Into<EffectId>, duration: Ticks) -> Self {
        Self {
            effect: effect.into(),
            duration,
        }
    }

    pub fn seconds(effect: impl Into<EffectId>, seconds: u32) -> Self {
        Self::new(effect, Ticks::seconds(seconds))
    }

    pub fn to_snbt(&self) -> String {
        format!(
            "{{id:\"{}\",duration:{}}}",
            self.effect,
            self.duration.get()
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn effect_id_display_and_json() {
        assert_eq!(EffectId::Speed.to_string(), "minecraft:speed");
        assert_eq!(
            serde_json::to_value(EffectId::Regeneration).unwrap(),
            json!("minecraft:regeneration")
        );
    }

    #[test]
    fn effect_id_custom() {
        let id = EffectId::custom("mymod:arcane_burn").unwrap();
        assert_eq!(id.to_string(), "mymod:arcane_burn");
        assert_eq!(
            serde_json::to_value(id).unwrap(),
            json!("mymod:arcane_burn")
        );
    }

    #[test]
    fn compatibility_ids_convert_to_shared_registry_ids() {
        let effect: StatusEffectId = EffectId::Speed.into();
        let potion: PotionRegistryId = PotionId::LongSwiftness.into();
        assert_eq!(effect.to_string(), "minecraft:speed");
        assert_eq!(potion.to_string(), "minecraft:long_swiftness");

        let effect_compat: EffectId = effect.into();
        let potion_compat: PotionId = potion.into();
        assert_eq!(effect_compat.to_string(), "minecraft:speed");
        assert_eq!(potion_compat.to_string(), "minecraft:long_swiftness");
    }

    #[test]
    fn shared_registry_ids_work_in_existing_effect_builders() {
        let effect = StatusEffectInstance::new(StatusEffectId::minecraft("speed").unwrap());
        let potion = PotionContents::new()
            .potion(PotionRegistryId::minecraft("swiftness").unwrap())
            .effect(effect);
        assert_eq!(
            serde_json::to_value(potion).unwrap(),
            json!({
                "potion": "minecraft:swiftness",
                "custom_effects": [{
                    "id": "minecraft:speed",
                    "amplifier": 0
                }]
            })
        );
    }

    #[test]
    fn status_effect_instance_json_and_snbt() {
        let effect = StatusEffectInstance::new(EffectId::Speed)
            .duration(Ticks::seconds(10))
            .amplifier(1)
            .particles(false);
        assert_eq!(
            serde_json::to_value(&effect).unwrap(),
            json!({
                "id": "minecraft:speed",
                "duration": 200,
                "amplifier": 1,
                "show_particles": false
            })
        );
        assert_eq!(
            effect.to_snbt(),
            "{id:\"minecraft:speed\",duration:200,amplifier:1,show_particles:false}"
        );
    }

    #[test]
    fn potion_contents_json_and_snbt() {
        let contents = PotionContents::new()
            .potion(PotionId::LongSwiftness)
            .custom_color(0x55AAFF)
            .effect(StatusEffectInstance::new(EffectId::Haste).seconds(5));
        assert_eq!(
            serde_json::to_value(&contents).unwrap(),
            json!({
                "potion": "minecraft:long_swiftness",
                "custom_color": 5614335u32,
                "custom_effects": [{
                    "id": "minecraft:haste",
                    "duration": 100,
                    "amplifier": 0
                }]
            })
        );
        assert_eq!(
            contents.to_snbt(),
            "{potion:\"minecraft:long_swiftness\",custom_color:5614335,custom_effects:[{id:\"minecraft:haste\",duration:100}]}"
        );
    }

    #[test]
    fn potion_contents_empty_is_intentionally_permitted() {
        // An empty `minecraft:potion_contents` component (`{}`) is a valid
        // vanilla payload Sand does not reject — locking in current behavior
        // per #148's empty-PotionContents policy note.
        assert_eq!(PotionContents::new().to_snbt(), "{}");
    }

    #[test]
    fn ticks_seconds_and_minutes_saturate_instead_of_overflowing() {
        // #148: previously plain `u32` multiplication, which panics in debug
        // builds and silently wraps in release builds on overflow.
        assert_eq!(Ticks::seconds(u32::MAX).get(), u32::MAX);
        assert_eq!(Ticks::minutes(u32::MAX).get(), u32::MAX);
        // Normal values are unaffected.
        assert_eq!(Ticks::seconds(5).get(), 100);
        assert_eq!(Ticks::minutes(2).get(), 2400);
    }

    #[test]
    fn suspicious_stew_effect_json_and_snbt() {
        let effect = SuspiciousStewEffect::seconds(EffectId::NightVision, 7);
        assert_eq!(
            serde_json::to_value(&effect).unwrap(),
            json!({"id": "minecraft:night_vision", "duration": 140})
        );
        assert_eq!(
            effect.to_snbt(),
            "{id:\"minecraft:night_vision\",duration:140}"
        );
    }
}
