//! Builder for `data/<namespace>/banner_pattern/` JSON files (Minecraft 1.21+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//! - `asset_id` must be non-empty and a valid plain resource location
//!   (e.g. `"minecraft:diagonal_left"`).
//! - `translation_key` must be non-empty and must not contain control
//!   characters.

use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::Result as SandResult;
use crate::resource_location::ResourceLocation;
use crate::validation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::BannerPattern",
    aliases = ["sand::prelude::BannerPattern"],
    module = "sand::component",
    summary = "A banner pattern definition (`data/<namespace>/banner_pattern/<id>.json`).",
    context = "A banner pattern definition (`data/<namespace>/banner_pattern/<id>.json`). Banner patterns define custom designs that can be applied to banners and shields using a loom. Each pattern requires a translation key for its display name.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::BannerPattern;",
)]
/// A banner pattern definition (`data/<namespace>/banner_pattern/<id>.json`).
///
/// Banner patterns define custom designs that can be applied to banners and shields
/// using a loom. Each pattern requires a translation key for its display name.
pub struct BannerPattern {
    location: ResourceLocation,
    /// The asset ID of the texture for this banner pattern
    /// (e.g. `"minecraft:diagonal_left"`).
    asset_id: String,
    /// Translation key used for the pattern's display name in the UI
    /// (e.g. `"block.minecraft.banner.diagonal_left"`).
    translation_key: String,
}

impl BannerPattern {
    /// Creates a new banner pattern with the given resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BannerPattern::new",
        aliases = ["sand::prelude::BannerPattern::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new banner pattern with the given resource location.",
        context = "Creates a new banner pattern with the given resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new banner pattern with the given resource location."),
        returns = "A newly constructed `BannerPattern` configured to create a new banner pattern with the given resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let banner_pattern = sand::component::BannerPattern::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            translation_key: String::new(),
        }
    }

    /// Sets the asset ID (texture reference) for this banner pattern.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BannerPattern::asset_id",
        aliases = ["sand::prelude::BannerPattern::asset_id"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the asset ID (texture reference) for this banner pattern.",
        context = "Sets the asset ID (texture reference) for this banner pattern. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to set the asset ID (texture reference) for this banner pattern."),
        returns = "The `BannerPattern` value with the documented change applied to set the asset ID (texture reference) for this banner pattern.",
        example = "use sand::prelude::*;\n\nfn demonstrate(banner_pattern_value: sand::component::BannerPattern, id: impl Into < String >)  {\n    let updated_banner_pattern = banner_pattern_value.asset_id(id);\n}",
    )]
    pub fn asset_id(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    /// Sets the translation key for the banner pattern's display name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::BannerPattern::translation_key",
        aliases = ["sand::prelude::BannerPattern::translation_key"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the translation key for the banner pattern's display name.",
        context = "Sets the translation key for the banner pattern's display name. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(key = "`key` provides the key that identifies the setting or entry used to set the translation key for the banner pattern's display name."),
        returns = "The `BannerPattern` value with the documented change applied to set the translation key for the banner pattern's display name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(banner_pattern_value: sand::component::BannerPattern, key: impl Into < String >)  {\n    let updated_banner_pattern = banner_pattern_value.translation_key(key);\n}",
    )]
    pub fn translation_key(mut self, key: impl Into<String>) -> Self {
        self.translation_key = key.into();
        self
    }
}

impl DatapackComponent for BannerPattern {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "banner_pattern";
        validation::require_non_empty(&self.location, kind, "asset_id", &self.asset_id)?;
        validation::validate_resource_location_str(
            &self.location,
            kind,
            "asset_id",
            &self.asset_id,
        )?;
        validation::require_non_empty(
            &self.location,
            kind,
            "translation_key",
            &self.translation_key,
        )?;
        validation::reject_control_chars(
            &self.location,
            kind,
            "translation_key",
            &self.translation_key,
        )?;
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "asset_id": self.asset_id,
            "translation_key": self.translation_key,
        })
    }

    fn component_dir(&self) -> &'static str {
        "banner_pattern"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "diagonal_left").unwrap()
    }

    fn valid() -> BannerPattern {
        BannerPattern::new(rl())
            .asset_id("minecraft:diagonal_left")
            .translation_key("block.minecraft.banner.diagonal_left")
    }

    #[test]
    fn valid_banner_pattern_passes_validation() {
        assert!(valid().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let bp = BannerPattern::new(rl()).translation_key("block.minecraft.banner.diagonal_left");
        let err = bp.validate().unwrap_err();
        assert!(err.to_string().contains("asset_id"), "{err}");
    }

    #[test]
    fn malformed_asset_id_is_rejected() {
        let bp = valid().asset_id("Not Valid!");
        assert!(bp.validate().is_err());
    }

    #[test]
    fn tag_asset_id_is_rejected() {
        let bp = valid().asset_id("#minecraft:diagonal_left");
        assert!(bp.validate().is_err());
    }

    #[test]
    fn empty_translation_key_is_rejected() {
        let bp = BannerPattern::new(rl()).asset_id("minecraft:diagonal_left");
        let err = bp.validate().unwrap_err();
        assert!(err.to_string().contains("translation_key"), "{err}");
    }

    #[test]
    fn control_char_translation_key_is_rejected() {
        let bp = valid().translation_key("bad\u{0007}key");
        assert!(bp.validate().is_err());
    }

    #[test]
    fn invalid_banner_pattern_fails_export() {
        let bp = BannerPattern::new(rl());
        assert!(bp.try_content().is_err());
    }

    #[test]
    fn valid_banner_pattern_json_is_stable() {
        let bp = valid();
        let json = bp.to_json();
        assert_eq!(json["asset_id"], "minecraft:diagonal_left");
        assert_eq!(
            json["translation_key"],
            "block.minecraft.banner.diagonal_left"
        );
        let a = serde_json::to_string_pretty(&bp.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&bp.to_json()).unwrap();
        assert_eq!(a, b);
    }
}
