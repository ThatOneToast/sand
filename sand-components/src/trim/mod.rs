//! Builders for armor trim material and pattern definitions (Minecraft 1.20+).
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization:
//!
//! `TrimMaterial`:
//! - `asset_name` is a validated [`TrimAssetName`] (e.g. `"quartz"`).
//! - `ingredient` is a typed [`ItemId`].
//! - `item_model_index` must be finite (non-`NaN`, non-infinite) — required
//!   for the value to serialize as valid JSON. Sand does **not** enforce a
//!   `0.0..=1.0` numeric range here: `item_model_index` is a pre-1.21.4
//!   legacy field (superseded by `override_armor_assets` in current
//!   Minecraft; Sand does not yet model that field, see the crate's known
//!   limitations) with no vanilla-documented bound, and real third-party
//!   tooling uses out-of-`0..1` values (e.g. `-1.0` as a sentinel) for valid
//!   purposes. A `0.0..=1.0` range was previously enforced here as an
//!   unverified opinionated guess and has been relaxed after failing to find
//!   vanilla evidence for it.
//!
//! `TrimPattern` uses a typed [`ResourceLocation`] asset ID and [`ItemId`]
//! template item.
//!
//! # Example
//! ```rust,ignore
//! let material = TrimMaterial::new(rl)
//!     .asset_name(TrimAssetName::new("quartz")?)
//!     .ingredient(ItemId::minecraft("quartz")?)
//!     .item_model_index(0.1)
//!     .description(TextComponent::translate("trim_material.minecraft.quartz"));
//!
//! let pattern = TrimPattern::new(rl)
//!     .asset_id(ResourceLocation::minecraft("bolt")?)
//!     .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template")?)
//!     .description(TextComponent::translate("trim_pattern.minecraft.bolt"));
//! ```

use std::collections::BTreeMap;
use std::fmt;

use sand_commands::{CommandProfile, TextComponent};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::raw::RawJson;
use crate::registry::ItemId;
use crate::resource_location::ResourceLocation;
use crate::validation;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TrimAssetName",
    aliases = ["sand::prelude::TrimAssetName"],
    module = "sand::component",
    summary = "A validated, un-namespaced trim texture asset name.",
    context = "A validated, un-namespaced trim texture asset name. Minecraft uses values such as `quartz` and `redstone_darker` here rather than registry IDs. Paths may contain lowercase resource-path characters, including `/`, but not a namespace separator.",
    minecraft = "Minecraft uses values such as `quartz` and `redstone_darker` here rather than registry IDs. Paths may contain lowercase resource-path characters, including `/`, but not a namespace separator.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TrimAssetName;",
)]
/// A validated, un-namespaced trim texture asset name.
///
/// Minecraft uses values such as `quartz` and `redstone_darker` here rather
/// than registry IDs. Paths may contain lowercase resource-path characters,
/// including `/`, but not a namespace separator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrimAssetName(String);

impl TrimAssetName {
    /// Validates an un-namespaced trim texture path such as `quartz` or `redstone_darker`.
    ///
    /// `name` is the texture path stored in the trim-material JSON. A namespace
    /// separator is rejected because Minecraft expects a path rather than a
    /// registry identifier in this field.
    ///
    /// On success, returns the validated trim asset name; invalid resource-path
    /// characters or a namespace separator produce a [`SandError`].
    ///
    /// ```rust
    /// use sand_components::TrimAssetName;
    ///
    /// let asset = TrimAssetName::new("redstone_darker")?;
    /// assert_eq!(asset.as_str(), "redstone_darker");
    /// # Ok::<(), sand_components::SandError>(())
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimAssetName::new",
        aliases = ["sand::prelude::TrimAssetName::new"],
        module = "sand::component",
        kind = "method",
        summary = "Validates an un-namespaced trim texture path such as `quartz` or `redstone_darker`.",
        context = "Validates an un-namespaced trim texture path such as `quartz` or `redstone_darker`. `name` is the texture path stored in the trim-material JSON. A namespace separator is rejected because Minecraft expects a path rather than a registry identifier in this field. On success, returns the validated trim asset name; invalid resource-path characters or a namespace separator produce a [`SandError`].",
        minecraft = "`name` is the texture path stored in the trim-material JSON. A namespace separator is rejected because Minecraft expects a path rather than a registry identifier in this field.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(name = "`name` is the texture path stored in the trim-material JSON. A namespace separator is rejected because Minecraft expects a path rather than a registry identifier in this field."),
        returns = "On success, returns the validated trim asset name; invalid resource-path characters or a namespace separator produce a [`SandError`].",
        example = "use sand::component::TrimAssetName;\nlet asset = TrimAssetName::new(\"redstone_darker\")?;\nassert_eq!(asset.as_str(), \"redstone_darker\");",
    )]
    pub fn new(name: impl AsRef<str>) -> SandResult<Self> {
        let name = name.as_ref();
        if name.contains(':') {
            return Err(SandError::InvalidPath(name.to_string()));
        }
        ResourceLocation::new("sand", name)?;
        Ok(Self(name.to_string()))
    }

    /// Returns the canonical Minecraft representation of this component value.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimAssetName::as_str",
        aliases = ["sand::prelude::TrimAssetName::as_str"],
        module = "sand::component",
        kind = "method",
        summary = "Returns the canonical Minecraft representation of this component value.",
        context = "Returns the canonical Minecraft representation of this component value. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Returns the canonical Minecraft representation of this component value.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_asset_name_value: &sand::component::TrimAssetName)  {\n    let as_str = trim_asset_name_value.as_str();\n}",
    )]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrimAssetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
enum TrimDescription {
    Typed(Box<TextComponent>),
    Raw(RawJson),
}

impl TrimDescription {
    fn validate(&self, location: &ResourceLocation, kind: &str, field: &str) -> SandResult<()> {
        if let Self::Typed(text) = self {
            text.validate_at_path(&CommandProfile::unprofiled(), field)
                .map_err(|error| {
                    validation::error(
                        location,
                        kind,
                        &error.field,
                        &format!("error[{}] {}", error.code, error.message),
                    )
                })?;
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Typed(text) => serde_json::from_str(&text.to_string())
                .expect("TextComponent always renders structurally valid JSON"),
            Self::Raw(raw) => raw.as_value().clone(),
        }
    }
}

#[derive(Debug, Clone)]
enum ArmorMaterialOverrides {
    Typed(BTreeMap<String, TrimAssetName>),
    Raw(RawJson),
}

// ── TrimMaterial ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TrimMaterial",
    aliases = ["sand::prelude::TrimMaterial"],
    module = "sand::component",
    summary = "A trim material definition (`data/<namespace>/trim_material/<id>.json`).",
    context = "A trim material definition (`data/<namespace>/trim_material/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TrimMaterial;",
)]
/// A trim material definition (`data/<namespace>/trim_material/<id>.json`).
pub struct TrimMaterial {
    location: ResourceLocation,
    /// Asset name used to locate the trim material texture (e.g. `"quartz"`).
    asset_name: Option<TrimAssetName>,
    /// Item used to apply this trim (e.g. `"minecraft:quartz"`).
    ingredient: Option<ItemId>,
    /// Model index for the trim overlay. Must be finite; Minecraft does not
    /// document a numeric range for this legacy (pre-1.21.4) field.
    item_model_index: f32,
    /// Text component for the trim tooltip description.
    description: Option<TrimDescription>,
    /// Per-armor-material overrides for the texture asset name.
    /// Keys are armor material IDs (e.g. `"minecraft:iron"`).
    override_armor_materials: Option<ArmorMaterialOverrides>,
}

impl TrimMaterial {
    /// Starts a trim-material definition at `location`; required material fields are added with the builder methods.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::new",
        aliases = ["sand::prelude::TrimMaterial::new"],
        module = "sand::component",
        kind = "method",
        summary = "Starts a trim-material definition at `location`; required material fields are added with the builder methods.",
        context = "Starts a trim-material definition at `location`; required material fields are added with the builder methods. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "Starts a trim-material definition at `location`; required material fields are added with the builder methods."),
        returns = "A newly constructed `TrimMaterial` configured to start a trim-material definition at `location`; required material fields are added with the builder methods.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let trim_material = sand::component::TrimMaterial::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_name: None,
            ingredient: None,
            item_model_index: 0.0,
            description: None,
            override_armor_materials: None,
        }
    }

    /// Sets the Minecraft asset name property on this typed trim material definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::asset_name",
        aliases = ["sand::prelude::TrimMaterial::asset_name"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft asset name property on this typed trim material definition and returns the updated builder.",
        context = "Sets the Minecraft asset name property on this typed trim material definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(name = "`name` provides the author-visible text value used to set the Minecraft asset name property on this typed trim material definition and returns the updated builder."),
        returns = "Sets the Minecraft asset name property on this typed trim material definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, name: sand::component::TrimAssetName)  {\n    let updated_trim_material = trim_material_value.asset_name(name);\n}",
    )]
    pub fn asset_name(mut self, name: TrimAssetName) -> Self {
        self.asset_name = Some(name);
        self
    }

    /// Sets the Minecraft ingredient property on this typed trim material definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::ingredient",
        aliases = ["sand::prelude::TrimMaterial::ingredient"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft ingredient property on this typed trim material definition and returns the updated builder.",
        context = "Sets the Minecraft ingredient property on this typed trim material definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(item = "`item` provides the item value or item predicate used to set the Minecraft ingredient property on this typed trim material definition and returns the updated builder."),
        returns = "Sets the Minecraft ingredient property on this typed trim material definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, item: sand::registry::ItemId)  {\n    let updated_trim_material = trim_material_value.ingredient(item);\n}",
    )]
    pub fn ingredient(mut self, item: ItemId) -> Self {
        self.ingredient = Some(item);
        self
    }

    /// Sets the Minecraft item model index property on this typed trim material definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::item_model_index",
        aliases = ["sand::prelude::TrimMaterial::item_model_index"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft item model index property on this typed trim material definition and returns the updated builder.",
        context = "Sets the Minecraft item model index property on this typed trim material definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(index = "`index` supplies the index value used to set the Minecraft item model index property on this typed trim material definition and returns the updated builder."),
        returns = "Sets the Minecraft item model index property on this typed trim material definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, index: f32)  {\n    let updated_trim_material = trim_material_value.item_model_index(index);\n}",
    )]
    pub fn item_model_index(mut self, index: f32) -> Self {
        self.item_model_index = index;
        self
    }

    /// Sets the Minecraft description property on this typed trim material definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::description",
        aliases = ["sand::prelude::TrimMaterial::description"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft description property on this typed trim material definition and returns the updated builder.",
        context = "Sets the Minecraft description property on this typed trim material definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(desc = "`desc` provides the player-visible text value used to set the Minecraft description property on this typed trim material definition and returns the updated builder."),
        returns = "Sets the Minecraft description property on this typed trim material definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, desc: sand::text::TextComponent)  {\n    let updated_trim_material = trim_material_value.description(desc);\n}",
    )]
    pub fn description(mut self, desc: TextComponent) -> Self {
        self.description = Some(TrimDescription::Typed(Box::new(desc)));
        self
    }

    /// Use a raw JSON text component when the typed text API cannot represent it.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::raw_description",
        aliases = ["sand::prelude::TrimMaterial::raw_description"],
        module = "sand::component",
        kind = "method",
        summary = "Use a raw JSON text component when the typed text API cannot represent it.",
        context = "Use a raw JSON text component when the typed text API cannot represent it. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Use a raw JSON text component when the typed text API cannot represent it."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(desc = "`desc` supplies the desc value used to use a raw JSON text component when the typed text API cannot represent it."),
        returns = "The `TrimMaterial` value with the documented change applied to use a raw JSON text component when the typed text API cannot represent it.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, desc: sand::component::RawJson)  {\n    let updated_trim_material = trim_material_value.raw_description(desc);\n}",
    )]
    pub fn raw_description(mut self, desc: RawJson) -> Self {
        self.description = Some(TrimDescription::Raw(desc));
        self
    }

    /// Override asset names for specific armor-material resource IDs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::override_armor_materials",
        aliases = ["sand::prelude::TrimMaterial::override_armor_materials"],
        module = "sand::component",
        kind = "method",
        summary = "Override asset names for specific armor-material resource IDs.",
        context = "Override asset names for specific armor-material resource IDs. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(overrides = "`overrides` provides the typed Minecraft resource identifier used to override asset names for specific armor-material resource IDs."),
        returns = "The `TrimMaterial` value with the documented change applied to override asset names for specific armor-material resource IDs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, overrides: impl IntoIterator < Item = (sand::ResourceLocation , sand::component::TrimAssetName) >)  {\n    let updated_trim_material = trim_material_value.override_armor_materials(overrides);\n}",
    )]
    pub fn override_armor_materials(
        mut self,
        overrides: impl IntoIterator<Item = (ResourceLocation, TrimAssetName)>,
    ) -> Self {
        self.override_armor_materials = Some(ArmorMaterialOverrides::Typed(
            overrides
                .into_iter()
                .map(|(material, asset)| (material.to_string(), asset))
                .collect(),
        ));
        self
    }

    /// Use a raw override object for unsupported or modded armor-material shapes.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimMaterial::raw_override_armor_materials",
        aliases = ["sand::prelude::TrimMaterial::raw_override_armor_materials"],
        module = "sand::component",
        kind = "method",
        summary = "Use a raw override object for unsupported or modded armor-material shapes.",
        context = "Use a raw override object for unsupported or modded armor-material shapes. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Use a raw override object for unsupported or modded armor-material shapes."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(overrides = "`overrides` supplies the overrides value used to use a raw override object for unsupported or modded armor-material shapes."),
        returns = "The `TrimMaterial` value with the documented change applied to use a raw override object for unsupported or modded armor-material shapes.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_material_value: sand::component::TrimMaterial, overrides: sand::component::RawJson)  {\n    let updated_trim_material = trim_material_value.raw_override_armor_materials(overrides);\n}",
    )]
    pub fn raw_override_armor_materials(mut self, overrides: RawJson) -> Self {
        self.override_armor_materials = Some(ArmorMaterialOverrides::Raw(overrides));
        self
    }
}

impl DatapackComponent for TrimMaterial {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "trim_material";
        if self.asset_name.is_none() {
            return Err(validation::error(
                &self.location,
                kind,
                "asset_name",
                "asset_name must be set",
            ));
        }
        if self.ingredient.is_none() {
            return Err(validation::error(
                &self.location,
                kind,
                "ingredient",
                "ingredient must be set",
            ));
        }
        validation::require_finite_f32(
            &self.location,
            kind,
            "item_model_index",
            self.item_model_index,
        )?;
        if let Some(description) = &self.description {
            description.validate(&self.location, kind, "description")?;
        }
        if let Some(ArmorMaterialOverrides::Raw(raw)) = &self.override_armor_materials
            && !raw.as_value().is_object()
        {
            return Err(validation::error(
                &self.location,
                kind,
                "override_armor_materials",
                "raw armor-material overrides must be a JSON object",
            ));
        }
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(asset_name) = &self.asset_name {
            map.insert(
                "asset_name".to_string(),
                Value::String(asset_name.to_string()),
            );
        }
        if let Some(ingredient) = &self.ingredient {
            map.insert(
                "ingredient".to_string(),
                Value::String(ingredient.to_string()),
            );
        }
        map.insert(
            "item_model_index".to_string(),
            serde_json::json!(self.item_model_index),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.to_json());
        }
        if let Some(ref overrides) = self.override_armor_materials {
            let value = match overrides {
                ArmorMaterialOverrides::Typed(entries) => Value::Object(
                    entries
                        .iter()
                        .map(|(material, asset)| {
                            (material.clone(), Value::String(asset.to_string()))
                        })
                        .collect(),
                ),
                ArmorMaterialOverrides::Raw(raw) => raw.as_value().clone(),
            };
            map.insert("override_armor_materials".to_string(), value);
        }
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "trim_material"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::TrimAssets]
    }
}

// ── TrimPattern ───────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::TrimPattern",
    aliases = ["sand::prelude::TrimPattern"],
    module = "sand::component",
    summary = "A trim pattern definition (`data/<namespace>/trim_pattern/<id>.json`).",
    context = "A trim pattern definition (`data/<namespace>/trim_pattern/<id>.json`). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::TrimPattern;",
)]
/// A trim pattern definition (`data/<namespace>/trim_pattern/<id>.json`).
pub struct TrimPattern {
    location: ResourceLocation,
    /// Resource location of the pattern texture (e.g. `"minecraft:bolt"`).
    asset_id: Option<ResourceLocation>,
    /// Item that applies this pattern at a smithing table.
    template_item: Option<ItemId>,
    /// Text component for the pattern tooltip.
    description: Option<TrimDescription>,
    /// Whether this pattern is rendered as a decal overlay.
    decal: bool,
}

impl TrimPattern {
    /// Starts a trim-pattern definition at `location`; required pattern fields are added with the builder methods.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::new",
        aliases = ["sand::prelude::TrimPattern::new"],
        module = "sand::component",
        kind = "method",
        summary = "Starts a trim-pattern definition at `location`; required pattern fields are added with the builder methods.",
        context = "Starts a trim-pattern definition at `location`; required pattern fields are added with the builder methods. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "Starts a trim-pattern definition at `location`; required pattern fields are added with the builder methods."),
        returns = "A newly constructed `TrimPattern` configured to start a trim-pattern definition at `location`; required pattern fields are added with the builder methods.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let trim_pattern = sand::component::TrimPattern::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: None,
            template_item: None,
            description: None,
            decal: false,
        }
    }

    /// Sets the Minecraft asset id property on this typed trim pattern definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::asset_id",
        aliases = ["sand::prelude::TrimPattern::asset_id"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft asset id property on this typed trim pattern definition and returns the updated builder.",
        context = "Sets the Minecraft asset id property on this typed trim pattern definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to set the Minecraft asset id property on this typed trim pattern definition and returns the updated builder."),
        returns = "Sets the Minecraft asset id property on this typed trim pattern definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_pattern_value: sand::component::TrimPattern, id: sand::ResourceLocation)  {\n    let updated_trim_pattern = trim_pattern_value.asset_id(id);\n}",
    )]
    pub fn asset_id(mut self, id: ResourceLocation) -> Self {
        self.asset_id = Some(id);
        self
    }

    /// Sets the Minecraft template item property on this typed trim pattern definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::template_item",
        aliases = ["sand::prelude::TrimPattern::template_item"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft template item property on this typed trim pattern definition and returns the updated builder.",
        context = "Sets the Minecraft template item property on this typed trim pattern definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(item = "`item` provides the item value or item predicate used to set the Minecraft template item property on this typed trim pattern definition and returns the updated builder."),
        returns = "Sets the Minecraft template item property on this typed trim pattern definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_pattern_value: sand::component::TrimPattern, item: sand::registry::ItemId)  {\n    let updated_trim_pattern = trim_pattern_value.template_item(item);\n}",
    )]
    pub fn template_item(mut self, item: ItemId) -> Self {
        self.template_item = Some(item);
        self
    }

    /// Sets the Minecraft description property on this typed trim pattern definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::description",
        aliases = ["sand::prelude::TrimPattern::description"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft description property on this typed trim pattern definition and returns the updated builder.",
        context = "Sets the Minecraft description property on this typed trim pattern definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(desc = "`desc` provides the player-visible text value used to set the Minecraft description property on this typed trim pattern definition and returns the updated builder."),
        returns = "Sets the Minecraft description property on this typed trim pattern definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_pattern_value: sand::component::TrimPattern, desc: sand::text::TextComponent)  {\n    let updated_trim_pattern = trim_pattern_value.description(desc);\n}",
    )]
    pub fn description(mut self, desc: TextComponent) -> Self {
        self.description = Some(TrimDescription::Typed(Box::new(desc)));
        self
    }

    /// Use a raw JSON text component when the typed text API cannot represent it.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::raw_description",
        aliases = ["sand::prelude::TrimPattern::raw_description"],
        module = "sand::component",
        kind = "method",
        summary = "Use a raw JSON text component when the typed text API cannot represent it.",
        context = "Use a raw JSON text component when the typed text API cannot represent it. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Use a raw JSON text component when the typed text API cannot represent it."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(desc = "`desc` supplies the desc value used to use a raw JSON text component when the typed text API cannot represent it."),
        returns = "The `TrimPattern` value with the documented change applied to use a raw JSON text component when the typed text API cannot represent it.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_pattern_value: sand::component::TrimPattern, desc: sand::component::RawJson)  {\n    let updated_trim_pattern = trim_pattern_value.raw_description(desc);\n}",
    )]
    pub fn raw_description(mut self, desc: RawJson) -> Self {
        self.description = Some(TrimDescription::Raw(desc));
        self
    }

    /// Sets the Minecraft decal property on this typed trim pattern definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::TrimPattern::decal",
        aliases = ["sand::prelude::TrimPattern::decal"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft decal property on this typed trim pattern definition and returns the updated builder.",
        context = "Sets the Minecraft decal property on this typed trim pattern definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set the Minecraft decal property on this typed trim pattern definition and returns the updated builder."),
        returns = "Sets the Minecraft decal property on this typed trim pattern definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trim_pattern_value: sand::component::TrimPattern, v: bool)  {\n    let updated_trim_pattern = trim_pattern_value.decal(v);\n}",
    )]
    pub fn decal(mut self, v: bool) -> Self {
        self.decal = v;
        self
    }
}

impl DatapackComponent for TrimPattern {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "trim_pattern";
        if self.asset_id.is_none() {
            return Err(validation::error(
                &self.location,
                kind,
                "asset_id",
                "asset_id must be set",
            ));
        }
        if self.template_item.is_none() {
            return Err(validation::error(
                &self.location,
                kind,
                "template_item",
                "template_item must be set",
            ));
        }
        if let Some(description) = &self.description {
            description.validate(&self.location, kind, "description")?;
        }
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        if let Some(asset_id) = &self.asset_id {
            map.insert("asset_id".to_string(), Value::String(asset_id.to_string()));
        }
        if let Some(template_item) = &self.template_item {
            map.insert(
                "template_item".to_string(),
                Value::String(template_item.to_string()),
            );
        }
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.to_json());
        }
        map.insert("decal".to_string(), Value::Bool(self.decal));
        Value::Object(map)
    }

    fn component_dir(&self) -> &'static str {
        "trim_pattern"
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::TrimAssets]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rl() -> ResourceLocation {
        ResourceLocation::new("test", "quartz").unwrap()
    }

    fn valid_material() -> TrimMaterial {
        TrimMaterial::new(rl())
            .asset_name(TrimAssetName::new("quartz").unwrap())
            .ingredient(ItemId::minecraft("quartz").unwrap())
            .item_model_index(0.1)
    }

    fn valid_pattern() -> TrimPattern {
        TrimPattern::new(rl())
            .asset_id(ResourceLocation::minecraft("bolt").unwrap())
            .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template").unwrap())
    }

    // ── TrimMaterial ────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_material_passes_validation() {
        assert!(valid_material().validate().is_ok());
    }

    #[test]
    fn empty_asset_name_is_rejected() {
        assert!(TrimAssetName::new("").is_err());
        let m = TrimMaterial::new(rl()).ingredient(ItemId::minecraft("quartz").unwrap());
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("asset_name"), "{err}");
    }

    #[test]
    fn empty_ingredient_is_rejected() {
        let m = TrimMaterial::new(rl()).asset_name(TrimAssetName::new("quartz").unwrap());
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("ingredient"), "{err}");
    }

    #[test]
    fn malformed_typed_inputs_are_rejected_at_construction() {
        assert!("Not Valid!".parse::<ItemId>().is_err());
        assert!(TrimAssetName::new("minecraft:quartz").is_err());
        assert!(TrimAssetName::new("Not Valid!").is_err());
    }

    #[test]
    fn nan_item_model_index_is_rejected() {
        let m = valid_material().item_model_index(f32::NAN);
        assert!(m.validate().is_err());
    }

    #[test]
    fn infinite_item_model_index_is_rejected() {
        let m = valid_material().item_model_index(f32::INFINITY);
        assert!(m.validate().is_err());
    }

    /// `item_model_index` has no vanilla-documented numeric range (see the
    /// module-level `# Validation` docs) — negative and >1.0 finite values
    /// are legitimate and must be accepted, matching real third-party usage
    /// (e.g. `-1.0` as a sentinel).
    #[test]
    fn item_model_index_out_of_unit_range_is_accepted() {
        assert!(valid_material().item_model_index(-1.0).validate().is_ok());
        assert!(valid_material().item_model_index(2.5).validate().is_ok());
    }

    #[test]
    fn item_model_index_bounds_are_accepted() {
        assert!(valid_material().item_model_index(0.0).validate().is_ok());
        assert!(valid_material().item_model_index(1.0).validate().is_ok());
    }

    #[test]
    fn invalid_trim_material_fails_export() {
        let m = TrimMaterial::new(rl());
        assert!(m.try_content().is_err());
    }

    #[test]
    fn valid_trim_material_json_is_stable() {
        let m = valid_material()
            .description(TextComponent::translate("trim_material.minecraft.quartz"))
            .override_armor_materials([(
                ResourceLocation::minecraft("iron").unwrap(),
                TrimAssetName::new("quartz_darker").unwrap(),
            )]);
        let json = m.to_json();
        assert_eq!(json["asset_name"], "quartz");
        assert_eq!(json["ingredient"], "minecraft:quartz");
        assert_eq!(json["item_model_index"], serde_json::json!(0.1_f32));
        assert_eq!(
            json["description"],
            serde_json::json!({"translate": "trim_material.minecraft.quartz"})
        );
        assert_eq!(
            json["override_armor_materials"],
            serde_json::json!({"minecraft:iron": "quartz_darker"})
        );
        let a = serde_json::to_string_pretty(&m.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&m.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn raw_material_escape_hatches_are_explicit_and_validated() {
        let material = valid_material()
            .raw_description(RawJson::new(serde_json::json!({"mymod:text": true})))
            .raw_override_armor_materials(RawJson::new(serde_json::json!({
                "mymod:alloy": "custom"
            })));
        assert!(material.validate().is_ok());
        assert_eq!(material.to_json()["description"]["mymod:text"], true);

        let invalid =
            valid_material().raw_override_armor_materials(RawJson::new(serde_json::json!([])));
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn invalid_typed_material_description_fails_export() {
        let material =
            valid_material().description(TextComponent::translate(" \t ").color_hex("#12FG00"));
        let error = material.try_content().unwrap_err().to_string();
        assert!(error.contains("description"), "{error}");
    }

    // ── TrimPattern ─────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_pattern_passes_validation() {
        assert!(valid_pattern().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let p = TrimPattern::new(rl())
            .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template").unwrap());
        let err = p.validate().unwrap_err();
        assert!(err.to_string().contains("asset_id"), "{err}");
    }

    #[test]
    fn empty_template_item_is_rejected() {
        let p = TrimPattern::new(rl()).asset_id(ResourceLocation::minecraft("bolt").unwrap());
        let err = p.validate().unwrap_err();
        assert!(err.to_string().contains("template_item"), "{err}");
    }

    #[test]
    fn malformed_pattern_ids_are_rejected_at_construction() {
        assert!("Not Valid!".parse::<ResourceLocation>().is_err());
        assert!("Not Valid!".parse::<ItemId>().is_err());
    }

    #[test]
    fn invalid_trim_pattern_fails_export() {
        let p = TrimPattern::new(rl());
        assert!(p.try_content().is_err());
    }

    #[test]
    fn valid_trim_pattern_json_is_stable() {
        let p = valid_pattern()
            .description(TextComponent::translate("trim_pattern.minecraft.bolt"))
            .decal(true);
        let json = p.to_json();
        assert_eq!(json["asset_id"], "minecraft:bolt");
        assert_eq!(
            json["template_item"],
            "minecraft:bolt_armor_trim_smithing_template"
        );
        assert_eq!(
            json["description"],
            serde_json::json!({"translate": "trim_pattern.minecraft.bolt"})
        );
        assert_eq!(json["decal"], true);
        let a = serde_json::to_string_pretty(&p.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&p.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn raw_pattern_description_is_preserved() {
        let pattern = valid_pattern()
            .raw_description(RawJson::new(serde_json::json!({"mymod:text": "bolt"})));
        assert!(pattern.validate().is_ok());
        assert_eq!(pattern.to_json()["description"]["mymod:text"], "bolt");
    }
}
