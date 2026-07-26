//! Builders for armor trim material and pattern definitions (Minecraft 1.20+).
//!
//! # API hierarchy
//!
//! Every reference-shaped field on [`TrimMaterial`] and [`TrimPattern`] follows
//! the same three-tier convention used elsewhere in `sand-components`:
//!
//! 1. **Typed normal path** — takes a validated Sand type
//!    ([`ItemId`], [`ResourceLocation`], [`TrimAssetName`],
//!    [`ArmorMaterialId`], [`TextComponent`]). This is what you should write.
//! 2. **Validated compatibility adapter** — [`IntoTrimItemId`] lets a
//!    [`ResourceLocation`] be used wherever an [`ItemId`] is expected, without
//!    reopening the raw-string path.
//! 3. **Explicit `*_raw` escape hatch** — clearly named, documented, and
//!    reserved for modded or otherwise unmodelled shapes.
//!
//! The `*_raw` setters bypass Sand's *typed construction* boundary. They are
//! **not** an unchecked write: ID-shaped raw fields are still shape-checked at
//! export time, because a value that is not `namespace:path` cannot be valid in
//! any Minecraft version. Raw `description` / `override_armor_materials` values
//! are structurally checked (text-component shape, JSON object of non-empty
//! string values) but their contents are user-owned.
//!
//! # Validation
//!
//! The export path calls [`DatapackComponent::validate`] before serialization.
//! All diagnostics carry a stable `SAND-TRIM-*` code (or a delegated
//! `SAND-TEXT-*` code for text components).
//!
//! `TrimMaterial`:
//! - `asset_name` must be non-empty and a plain (non-tag) resource-location-shaped
//!   identifier (e.g. `"quartz"`). — `SAND-TRIM-ASSET-NAME`
//! - `ingredient` must be non-empty and a plain resource location, never a
//!   `#tag` reference: vanilla resolves it to a single item.
//!   — `SAND-TRIM-INGREDIENT`
//! - `item_model_index` must be finite (non-`NaN`, non-infinite) — required
//!   for the value to serialize as valid JSON. Sand does **not** enforce a
//!   `0.0..=1.0` numeric range here: `item_model_index` is a pre-1.21.4
//!   legacy field (superseded by `override_armor_assets` in current
//!   Minecraft; Sand does not yet model that field, see the crate's known
//!   limitations) with no vanilla-documented bound, and real third-party
//!   tooling uses out-of-`0..1` values (e.g. `-1.0` as a sentinel) for valid
//!   purposes. A `0.0..=1.0` range was previously enforced here as an
//!   unverified opinionated guess and has been relaxed after failing to find
//!   vanilla evidence for it. — `SAND-TRIM-MODEL-INDEX`
//! - `description`, when set, must be a valid text component.
//!   — delegated `SAND-TEXT-*`
//! - `override_armor_materials`, when set through the typed path, must not
//!   repeat an armor-material key. — `SAND-TRIM-OVERRIDE-DUPLICATE`
//! - `override_armor_materials_raw`, when set, must be a JSON object whose
//!   values are all non-empty strings. — `SAND-TRIM-OVERRIDE-SHAPE`
//!
//! `TrimPattern`:
//! - `asset_id` must be non-empty and a plain resource location.
//!   — `SAND-TRIM-ASSET-ID`
//! - `template_item` must be non-empty and a plain resource location.
//!   — `SAND-TRIM-TEMPLATE-ITEM`
//! - `description`, when set, must be a valid text component.
//!   — delegated `SAND-TEXT-*`
//!
//! # Schema note (read before claiming 26.2 parity)
//!
//! Sand emits the 1.19.4–1.21.3 `trim_material` / `trim_pattern` schema
//! (`asset_name` + `item_model_index` + `override_armor_materials`). Minecraft
//! 1.21.4 replaced `asset_name`/`override_armor_materials` with `assets` /
//! `override_armor_assets`. Modelling that newer schema is **not** part of this
//! module yet; see the `minecraft:trim_material` row in
//! [`crate::registry_coverage`] for the tracked gap.
//!
//! # Example
//!
//! ```
//! use sand_components::registry::{ArmorMaterialId, ItemId};
//! use sand_components::trim::{TrimAssetName, TrimMaterial, TrimPattern};
//! use sand_components::{DatapackComponent, ResourceLocation};
//! use sand_commands::TextComponent;
//!
//! let material = TrimMaterial::new(ResourceLocation::new("mypack", "amethyst")?)
//!     .asset_name(TrimAssetName::new("amethyst")?)
//!     .ingredient(ItemId::minecraft("amethyst_shard")?)
//!     .item_model_index(0.1)
//!     .description(TextComponent::literal("Amethyst"))
//!     .override_armor_material(
//!         ArmorMaterialId::minecraft("iron")?,
//!         TrimAssetName::new("amethyst_darker")?,
//!     );
//! assert!(material.validate().is_ok());
//!
//! let pattern = TrimPattern::new(ResourceLocation::new("mypack", "bolt")?)
//!     .asset_id(ResourceLocation::minecraft("bolt")?)
//!     .template_item(ItemId::minecraft("bolt_armor_trim_smithing_template")?)
//!     .description(TextComponent::translate("trim_pattern.minecraft.bolt"));
//! assert!(pattern.validate().is_ok());
//! # Ok::<(), sand_components::error::SandError>(())
//! ```

use std::fmt;

use sand_commands::{CommandProfile, TextComponent};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::error::{Result as SandResult, SandError};
use crate::registry::{ArmorMaterialId, ItemId};
use crate::resource_location::ResourceLocation;

// ── Shared helpers ────────────────────────────────────────────────────────────

fn trim_error(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    code: &str,
    message: &str,
) -> SandError {
    SandError::ComponentValidation {
        location: location.clone(),
        kind: kind.to_string(),
        field: field.to_string(),
        message: format!("error[{code}] {message}"),
    }
}

/// Shape-check a value that must serialize as a single concrete registry entry.
///
/// Rejects empty values and `#tag` references (the tag-vs-plain distinction is
/// a common authoring bug: a `#`-prefixed value would be written unchanged into
/// the datapack and only fail at world load).
fn require_plain_resource_ref(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    code: &str,
    value: &str,
) -> SandResult<()> {
    if value.is_empty() {
        return Err(trim_error(
            location,
            kind,
            field,
            code,
            &format!("{field} must not be empty"),
        ));
    }
    if let Some(rest) = value.strip_prefix('#') {
        return Err(trim_error(
            location,
            kind,
            field,
            code,
            &format!(
                "{field} must be a plain resource location, not a tag reference; \
                 received `#{rest}` — vanilla resolves this field to a single entry"
            ),
        ));
    }
    validate_resource_ref_chars(value).map_err(|message| {
        trim_error(
            location,
            kind,
            field,
            code,
            &format!("{field} {message}; received `{value}`"),
        )
    })
}

/// Validate `namespace:path` (or bare `path`) character rules.
fn validate_resource_ref_chars(value: &str) -> std::result::Result<(), String> {
    let (namespace, path) = match value.split_once(':') {
        Some((namespace, path)) => (Some(namespace), path),
        None => (None, value),
    };
    if let Some(namespace) = namespace
        && (namespace.is_empty()
            || !namespace
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b)))
    {
        return Err(
            "must have a namespace matching [a-z0-9_.-]+ before the `:` separator".to_string(),
        );
    }
    if path.is_empty()
        || !path
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_./-".contains(&b))
    {
        return Err("must have a path matching [a-z0-9_./-]+".to_string());
    }
    Ok(())
}

fn validate_text(
    location: &ResourceLocation,
    kind: &str,
    field: &str,
    text: &TrimDescription,
) -> SandResult<()> {
    let result = match text {
        TrimDescription::Typed(component) => {
            component.validate_at_path(&CommandProfile::unprofiled(), field)
        }
        TrimDescription::Raw(value) => {
            sand_commands::text::validate_json_text(value, &CommandProfile::unprofiled(), field)
        }
    };
    result.map_err(|error| SandError::ComponentValidation {
        location: location.clone(),
        kind: kind.to_string(),
        field: error.field,
        message: format!("error[{}] {}", error.code, error.message),
    })
}

// ── TrimAssetName ─────────────────────────────────────────────────────────────

/// A validated trim texture asset name (e.g. `"quartz"`, `"amethyst_darker"`).
///
/// This is **not** a [`ResourceLocation`]: vanilla writes a bare asset-name
/// segment here and resolves it against `trims/materials/<asset_name>`. Sand
/// therefore validates it as a non-empty, plain (non-tag) resource-location-shaped
/// identifier rather than forcing a `namespace:path` pair — matching the
/// pre-existing accepted-value set exactly, so no previously valid pack becomes
/// invalid.
///
/// ```
/// use sand_components::trim::TrimAssetName;
///
/// assert_eq!(TrimAssetName::new("quartz")?.as_str(), "quartz");
/// assert!(TrimAssetName::new("").is_err());
/// assert!(TrimAssetName::new("Not Valid!").is_err());
/// assert!(TrimAssetName::new("#minecraft:quartz").is_err());
/// # Ok::<(), sand_components::error::SandError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrimAssetName(String);

impl TrimAssetName {
    /// Construct a validated asset name. Returns an error for empty values,
    /// `#tag` references, and characters outside `[a-z0-9_./-]` (plus an
    /// optional `[a-z0-9_.-]+:` namespace prefix).
    pub fn new(name: impl AsRef<str>) -> SandResult<Self> {
        let name = name.as_ref();
        if name.is_empty() {
            return Err(SandError::InvalidPath(name.to_string()));
        }
        if name.starts_with('#') {
            return Err(SandError::InvalidPath(name.to_string()));
        }
        validate_resource_ref_chars(name).map_err(|_| SandError::InvalidPath(name.to_string()))?;
        Ok(Self(name.to_string()))
    }

    /// The asset name as written into the datapack JSON.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrimAssetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for TrimAssetName {
    type Err = SandError;
    fn from_str(s: &str) -> SandResult<Self> {
        Self::new(s)
    }
}

impl From<&TrimAssetName> for TrimAssetName {
    fn from(value: &TrimAssetName) -> Self {
        value.clone()
    }
}

// ── IntoTrimItemId ────────────────────────────────────────────────────────────

/// Validated compatibility adapter for trim fields that take an item ID.
///
/// Mirrors `recipe::IntoRecipeItemId`: it accepts Sand's typed [`ItemId`] and a
/// already-validated [`ResourceLocation`], and deliberately does **not** accept
/// `&str` — raw strings must go through the explicitly named `*_raw` setters.
pub trait IntoTrimItemId {
    /// Convert into a typed [`ItemId`].
    fn into_trim_item_id(self) -> ItemId;
}

impl IntoTrimItemId for ItemId {
    fn into_trim_item_id(self) -> ItemId {
        self
    }
}

impl IntoTrimItemId for &ItemId {
    fn into_trim_item_id(self) -> ItemId {
        self.clone()
    }
}

impl IntoTrimItemId for ResourceLocation {
    fn into_trim_item_id(self) -> ItemId {
        self.into()
    }
}

impl IntoTrimItemId for &ResourceLocation {
    fn into_trim_item_id(self) -> ItemId {
        self.clone().into()
    }
}

// ── Internal field representations ────────────────────────────────────────────

#[derive(Debug, Clone)]
enum TrimDescription {
    Typed(Box<TextComponent>),
    Raw(Value),
}

impl TrimDescription {
    fn to_json(&self) -> Value {
        match self {
            Self::Typed(component) => serde_json::from_str(&component.to_string())
                .expect("TextComponent must serialize to JSON"),
            Self::Raw(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone)]
enum TrimOverrides {
    /// Ordered typed entries. A `Vec` (not a map) so duplicate keys are
    /// *detected and reported* rather than silently collapsing.
    Typed(Vec<(ArmorMaterialId, TrimAssetName)>),
    Raw(Value),
}

// ── TrimMaterial ──────────────────────────────────────────────────────────────

/// A trim material definition (`data/<namespace>/trim_material/<id>.json`).
#[derive(Debug, Clone)]
pub struct TrimMaterial {
    location: ResourceLocation,
    /// Asset name used to locate the trim material texture (e.g. `"quartz"`).
    asset_name: String,
    /// Item used to apply this trim (e.g. `"minecraft:quartz"`).
    ingredient: String,
    /// Model index for the trim overlay. Must be finite; Minecraft does not
    /// document a numeric range for this legacy (pre-1.21.4) field.
    item_model_index: f32,
    /// Text component for the trim tooltip description.
    description: Option<TrimDescription>,
    /// Per-armor-material overrides for the texture asset name.
    override_armor_materials: Option<TrimOverrides>,
}

impl TrimMaterial {
    /// Start a new trim material at `location`.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_name: String::new(),
            ingredient: String::new(),
            item_model_index: 0.0,
            description: None,
            override_armor_materials: None,
        }
    }

    /// Set the texture asset name from a validated [`TrimAssetName`].
    pub fn asset_name(mut self, name: impl Into<TrimAssetName>) -> Self {
        self.asset_name = name.into().0;
        self
    }

    /// Set the texture asset name from an unchecked compatibility string.
    ///
    /// Escape hatch — prefer [`TrimMaterial::asset_name`]. This bypasses the
    /// [`TrimAssetName`] construction boundary; the value is still shape-checked
    /// at export time (`SAND-TRIM-ASSET-NAME`).
    pub fn asset_name_raw(mut self, name: impl Into<String>) -> Self {
        self.asset_name = name.into();
        self
    }

    /// Set the item that applies this trim, through Sand's typed item-ID boundary.
    pub fn ingredient(mut self, item: impl IntoTrimItemId) -> Self {
        self.ingredient = item.into_trim_item_id().to_string();
        self
    }

    /// Set the ingredient from an unchecked compatibility string.
    ///
    /// Escape hatch — prefer [`TrimMaterial::ingredient`]. This bypasses the
    /// [`ItemId`] construction boundary; the value is still shape-checked at
    /// export time and `#tag` references are rejected (`SAND-TRIM-INGREDIENT`).
    pub fn ingredient_raw(mut self, item: impl Into<String>) -> Self {
        self.ingredient = item.into();
        self
    }

    /// Set the legacy (pre-1.21.4) trim overlay model index.
    ///
    /// Only finiteness is enforced — see the module-level `# Validation` docs
    /// for why no numeric range is asserted.
    pub fn item_model_index(mut self, index: f32) -> Self {
        self.item_model_index = index;
        self
    }

    /// Set the tooltip description from a typed [`TextComponent`].
    pub fn description(mut self, desc: impl Into<TextComponent>) -> Self {
        self.description = Some(TrimDescription::Typed(Box::new(desc.into())));
        self
    }

    /// Set the tooltip description from raw JSON.
    ///
    /// Escape hatch — prefer [`TrimMaterial::description`]. The value is still
    /// validated as a text-component shape at export time (delegated
    /// `SAND-TEXT-*` diagnostics).
    pub fn description_raw(mut self, desc: Value) -> Self {
        self.description = Some(TrimDescription::Raw(desc));
        self
    }

    /// Override the texture asset name for one armor material.
    ///
    /// Repeat the call to add more entries. Setting the same
    /// [`ArmorMaterialId`] twice is rejected at export
    /// (`SAND-TRIM-OVERRIDE-DUPLICATE`) rather than silently collapsing.
    /// Calling this after [`TrimMaterial::override_armor_materials_raw`]
    /// replaces the raw map.
    pub fn override_armor_material(
        mut self,
        material: ArmorMaterialId,
        asset: impl Into<TrimAssetName>,
    ) -> Self {
        let entry = (material, asset.into());
        match self.override_armor_materials {
            Some(TrimOverrides::Typed(ref mut entries)) => entries.push(entry),
            _ => self.override_armor_materials = Some(TrimOverrides::Typed(vec![entry])),
        }
        self
    }

    /// Replace the armor-material override map with typed entries.
    ///
    /// An empty iterator clears the field: Sand omits `override_armor_materials`
    /// entirely rather than emitting `{}` (there is no vanilla evidence that an
    /// empty object is rejected, but emitting one is never meaningful).
    pub fn override_armor_materials(
        mut self,
        entries: impl IntoIterator<Item = (ArmorMaterialId, TrimAssetName)>,
    ) -> Self {
        self.override_armor_materials = Some(TrimOverrides::Typed(entries.into_iter().collect()));
        self
    }

    /// Set the armor-material override map from raw JSON.
    ///
    /// Escape hatch — prefer [`TrimMaterial::override_armor_material`]. Needed
    /// for shapes the typed path cannot express, notably pre-1.20.2 packs that
    /// used bare, un-namespaced keys (`{"iron": "iron_darker"}`). Still checked
    /// at export time to be a JSON object of non-empty string values
    /// (`SAND-TRIM-OVERRIDE-SHAPE`).
    pub fn override_armor_materials_raw(mut self, overrides: Value) -> Self {
        self.override_armor_materials = Some(TrimOverrides::Raw(overrides));
        self
    }

    fn overrides_json(&self) -> Option<Value> {
        match self.override_armor_materials.as_ref()? {
            TrimOverrides::Raw(value) => Some(value.clone()),
            TrimOverrides::Typed(entries) if entries.is_empty() => None,
            TrimOverrides::Typed(entries) => {
                let mut map = serde_json::Map::new();
                for (material, asset) in entries {
                    map.insert(material.to_string(), Value::String(asset.0.clone()));
                }
                Some(Value::Object(map))
            }
        }
    }
}

impl DatapackComponent for TrimMaterial {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> SandResult<()> {
        let kind = "trim_material";
        require_plain_resource_ref(
            &self.location,
            kind,
            "asset_name",
            "SAND-TRIM-ASSET-NAME",
            &self.asset_name,
        )?;
        require_plain_resource_ref(
            &self.location,
            kind,
            "ingredient",
            "SAND-TRIM-INGREDIENT",
            &self.ingredient,
        )?;
        if !self.item_model_index.is_finite() {
            return Err(trim_error(
                &self.location,
                kind,
                "item_model_index",
                "SAND-TRIM-MODEL-INDEX",
                "item_model_index must be a finite number (NaN and infinity cannot \
                 be serialized as JSON); Sand enforces no other numeric range",
            ));
        }
        if let Some(ref description) = self.description {
            validate_text(&self.location, kind, "description", description)?;
        }
        match self.override_armor_materials {
            None => {}
            Some(TrimOverrides::Typed(ref entries)) => {
                let mut seen = std::collections::HashSet::new();
                for (index, (material, _)) in entries.iter().enumerate() {
                    let key = material.to_string();
                    if !seen.insert(key.clone()) {
                        return Err(trim_error(
                            &self.location,
                            kind,
                            &format!("override_armor_materials[{index}]"),
                            "SAND-TRIM-OVERRIDE-DUPLICATE",
                            &format!(
                                "duplicate armor-material override key `{key}`; each \
                                 armor material may be overridden at most once"
                            ),
                        ));
                    }
                }
            }
            Some(TrimOverrides::Raw(ref value)) => {
                let Some(map) = value.as_object() else {
                    return Err(trim_error(
                        &self.location,
                        kind,
                        "override_armor_materials",
                        "SAND-TRIM-OVERRIDE-SHAPE",
                        "override_armor_materials must be a JSON object mapping \
                         armor-material keys to texture asset names",
                    ));
                };
                for (key, entry) in map {
                    let Some(asset) = entry.as_str() else {
                        return Err(trim_error(
                            &self.location,
                            kind,
                            &format!("override_armor_materials.{key}"),
                            "SAND-TRIM-OVERRIDE-SHAPE",
                            "armor-material override values must be texture asset \
                             name strings",
                        ));
                    };
                    if asset.is_empty() {
                        return Err(trim_error(
                            &self.location,
                            kind,
                            &format!("override_armor_materials.{key}"),
                            "SAND-TRIM-OVERRIDE-SHAPE",
                            "armor-material override values must not be empty",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "asset_name".to_string(),
            Value::String(self.asset_name.clone()),
        );
        map.insert(
            "ingredient".to_string(),
            Value::String(self.ingredient.clone()),
        );
        map.insert(
            "item_model_index".to_string(),
            serde_json::json!(self.item_model_index),
        );
        if let Some(ref desc) = self.description {
            map.insert("description".to_string(), desc.to_json());
        }
        if let Some(overrides) = self.overrides_json() {
            map.insert("override_armor_materials".to_string(), overrides);
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

/// A trim pattern definition (`data/<namespace>/trim_pattern/<id>.json`).
#[derive(Debug, Clone)]
pub struct TrimPattern {
    location: ResourceLocation,
    /// Resource location of the pattern texture (e.g. `"minecraft:bolt"`).
    asset_id: String,
    /// Item that applies this pattern at a smithing table.
    template_item: String,
    /// Text component for the pattern tooltip.
    description: Option<TrimDescription>,
    /// Whether this pattern is rendered as a decal overlay.
    decal: bool,
}

impl TrimPattern {
    /// Start a new trim pattern at `location`.
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            asset_id: String::new(),
            template_item: String::new(),
            description: None,
            decal: false,
        }
    }

    /// Set the pattern texture ID from a validated [`ResourceLocation`].
    pub fn asset_id(mut self, id: impl Into<ResourceLocation>) -> Self {
        self.asset_id = id.into().to_string();
        self
    }

    /// Set the pattern texture ID from an unchecked compatibility string.
    ///
    /// Escape hatch — prefer [`TrimPattern::asset_id`]. This bypasses the
    /// [`ResourceLocation`] construction boundary; the value is still
    /// shape-checked at export time (`SAND-TRIM-ASSET-ID`).
    pub fn asset_id_raw(mut self, id: impl Into<String>) -> Self {
        self.asset_id = id.into();
        self
    }

    /// Set the smithing template item, through Sand's typed item-ID boundary.
    pub fn template_item(mut self, item: impl IntoTrimItemId) -> Self {
        self.template_item = item.into_trim_item_id().to_string();
        self
    }

    /// Set the smithing template item from an unchecked compatibility string.
    ///
    /// Escape hatch — prefer [`TrimPattern::template_item`]. This bypasses the
    /// [`ItemId`] construction boundary; the value is still shape-checked at
    /// export time and `#tag` references are rejected
    /// (`SAND-TRIM-TEMPLATE-ITEM`).
    pub fn template_item_raw(mut self, item: impl Into<String>) -> Self {
        self.template_item = item.into();
        self
    }

    /// Set the tooltip description from a typed [`TextComponent`].
    pub fn description(mut self, desc: impl Into<TextComponent>) -> Self {
        self.description = Some(TrimDescription::Typed(Box::new(desc.into())));
        self
    }

    /// Set the tooltip description from raw JSON.
    ///
    /// Escape hatch — prefer [`TrimPattern::description`]. The value is still
    /// validated as a text-component shape at export time (delegated
    /// `SAND-TEXT-*` diagnostics).
    pub fn description_raw(mut self, desc: Value) -> Self {
        self.description = Some(TrimDescription::Raw(desc));
        self
    }

    /// Whether this pattern renders as a decal overlay.
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
        require_plain_resource_ref(
            &self.location,
            kind,
            "asset_id",
            "SAND-TRIM-ASSET-ID",
            &self.asset_id,
        )?;
        require_plain_resource_ref(
            &self.location,
            kind,
            "template_item",
            "SAND-TRIM-TEMPLATE-ITEM",
            &self.template_item,
        )?;
        if let Some(ref description) = self.description {
            validate_text(&self.location, kind, "description", description)?;
        }
        Ok(())
    }

    fn try_content(&self) -> SandResult<ComponentContent> {
        self.validate()?;
        Ok(self.content())
    }

    fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("asset_id".to_string(), Value::String(self.asset_id.clone()));
        map.insert(
            "template_item".to_string(),
            Value::String(self.template_item.clone()),
        );
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

    fn asset(name: &str) -> TrimAssetName {
        TrimAssetName::new(name).unwrap()
    }

    fn item(path: &str) -> ItemId {
        ItemId::minecraft(path).unwrap()
    }

    fn valid_material() -> TrimMaterial {
        TrimMaterial::new(rl())
            .asset_name(asset("quartz"))
            .ingredient(item("quartz"))
            .item_model_index(0.1)
    }

    fn valid_pattern() -> TrimPattern {
        TrimPattern::new(rl())
            .asset_id(ResourceLocation::minecraft("bolt").unwrap())
            .template_item(item("bolt_armor_trim_smithing_template"))
    }

    // ── TrimAssetName ───────────────────────────────────────────────────────

    #[test]
    fn trim_asset_name_accepts_bare_and_namespaced_segments() {
        assert_eq!(asset("quartz").as_str(), "quartz");
        assert_eq!(asset("minecraft:quartz").as_str(), "minecraft:quartz");
        assert_eq!(asset("amethyst_darker").to_string(), "amethyst_darker");
    }

    #[test]
    fn trim_asset_name_rejects_invalid_segments() {
        assert!(TrimAssetName::new("").is_err());
        assert!(TrimAssetName::new("Quartz").is_err());
        assert!(TrimAssetName::new("not valid").is_err());
        assert!(TrimAssetName::new("#minecraft:quartz").is_err());
        assert!(TrimAssetName::new("minecraft:").is_err());
        assert!(TrimAssetName::new(":quartz").is_err());
        assert!("quartz".parse::<TrimAssetName>().is_ok());
    }

    // ── TrimMaterial ────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_material_passes_validation() {
        assert!(valid_material().validate().is_ok());
    }

    #[test]
    fn empty_asset_name_is_rejected() {
        let m = TrimMaterial::new(rl()).ingredient(item("quartz"));
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("asset_name"), "{err}");
        assert!(err.to_string().contains("SAND-TRIM-ASSET-NAME"), "{err}");
    }

    #[test]
    fn empty_ingredient_is_rejected() {
        let m = TrimMaterial::new(rl()).asset_name(asset("quartz"));
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("ingredient"), "{err}");
        assert!(err.to_string().contains("SAND-TRIM-INGREDIENT"), "{err}");
    }

    #[test]
    fn malformed_raw_ingredient_is_rejected() {
        let m = valid_material().ingredient_raw("Not Valid!");
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("SAND-TRIM-INGREDIENT"), "{err}");
    }

    /// The tag-vs-plain distinction: `#ns:path` is character-valid but
    /// semantically wrong here, and must produce its own diagnostic.
    #[test]
    fn tag_reference_ingredient_is_rejected_with_tag_specific_message() {
        let m = valid_material().ingredient_raw("#minecraft:quartz_items");
        let err = m.validate().unwrap_err().to_string();
        assert!(err.contains("SAND-TRIM-INGREDIENT"), "{err}");
        assert!(err.contains("not a tag reference"), "{err}");
    }

    #[test]
    fn typed_ingredient_cannot_express_a_tag() {
        // `ItemId` construction rejects the `#` prefix outright, so the typed
        // normal path structurally cannot produce a tag reference.
        assert!(ItemId::minecraft("#quartz").is_err());
        assert!("#minecraft:quartz".parse::<ItemId>().is_err());
    }

    #[test]
    fn nan_item_model_index_is_rejected() {
        let err = valid_material()
            .item_model_index(f32::NAN)
            .validate()
            .unwrap_err()
            .to_string();
        assert!(err.contains("SAND-TRIM-MODEL-INDEX"), "{err}");
    }

    #[test]
    fn infinite_item_model_index_is_rejected() {
        assert!(
            valid_material()
                .item_model_index(f32::INFINITY)
                .validate()
                .is_err()
        );
        assert!(
            valid_material()
                .item_model_index(f32::NEG_INFINITY)
                .validate()
                .is_err()
        );
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

    // ── TrimMaterial: description ───────────────────────────────────────────

    #[test]
    fn typed_description_serializes_as_a_text_component() {
        let m = valid_material().description(TextComponent::literal("Quartz"));
        assert_eq!(
            m.to_json()["description"],
            serde_json::json!({"text": "Quartz"})
        );
        assert!(m.validate().is_ok());
    }

    #[test]
    fn typed_translate_description_serializes() {
        let m = valid_material()
            .description(TextComponent::translate("trim_material.minecraft.quartz"));
        assert_eq!(
            m.to_json()["description"],
            serde_json::json!({"translate": "trim_material.minecraft.quartz"})
        );
    }

    #[test]
    fn typed_description_with_invalid_translation_key_is_rejected() {
        let m = valid_material().description(TextComponent::translate("  "));
        let err = m.validate().unwrap_err().to_string();
        assert!(err.contains("SAND-TEXT-TRANSLATE"), "{err}");
    }

    #[test]
    fn raw_description_escape_hatch_round_trips_and_is_shape_checked() {
        let m = valid_material()
            .description_raw(serde_json::json!({"translate": "trim_material.minecraft.quartz"}));
        assert_eq!(
            m.to_json()["description"],
            serde_json::json!({"translate": "trim_material.minecraft.quartz"})
        );
        assert!(m.validate().is_ok());

        let bad = valid_material().description_raw(serde_json::json!({"translate": ""}));
        assert!(bad.validate().is_err());
    }

    // ── TrimMaterial: override_armor_materials ──────────────────────────────

    #[test]
    fn typed_overrides_serialize_and_validate() {
        let m = valid_material()
            .override_armor_material(
                ArmorMaterialId::minecraft("iron").unwrap(),
                asset("quartz_darker"),
            )
            .override_armor_material(
                ArmorMaterialId::minecraft("gold").unwrap(),
                asset("quartz_gold"),
            );
        assert!(m.validate().is_ok());
        assert_eq!(
            m.to_json()["override_armor_materials"],
            serde_json::json!({
                "minecraft:iron": "quartz_darker",
                "minecraft:gold": "quartz_gold",
            })
        );
    }

    #[test]
    fn duplicate_override_key_is_rejected() {
        let m = valid_material()
            .override_armor_material(ArmorMaterialId::minecraft("iron").unwrap(), asset("a"))
            .override_armor_material(ArmorMaterialId::minecraft("iron").unwrap(), asset("b"));
        let err = m.validate().unwrap_err().to_string();
        assert!(err.contains("SAND-TRIM-OVERRIDE-DUPLICATE"), "{err}");
        assert!(err.contains("minecraft:iron"), "{err}");
    }

    #[test]
    fn empty_typed_override_map_omits_the_field() {
        let m = valid_material().override_armor_materials(std::iter::empty());
        assert!(m.validate().is_ok());
        assert!(m.to_json().get("override_armor_materials").is_none());
    }

    #[test]
    fn raw_override_escape_hatch_supports_bare_legacy_keys() {
        let m = valid_material()
            .override_armor_materials_raw(serde_json::json!({"iron": "quartz_darker"}));
        assert!(m.validate().is_ok());
        assert_eq!(
            m.to_json()["override_armor_materials"],
            serde_json::json!({"iron": "quartz_darker"})
        );
    }

    #[test]
    fn raw_override_rejects_non_object_and_non_string_values() {
        let non_object = valid_material().override_armor_materials_raw(serde_json::json!(["iron"]));
        let err = non_object.validate().unwrap_err().to_string();
        assert!(err.contains("SAND-TRIM-OVERRIDE-SHAPE"), "{err}");

        let non_string =
            valid_material().override_armor_materials_raw(serde_json::json!({"iron": 3}));
        assert!(non_string.validate().is_err());

        let empty_value =
            valid_material().override_armor_materials_raw(serde_json::json!({"iron": ""}));
        assert!(empty_value.validate().is_err());
    }

    #[test]
    fn valid_trim_material_json_is_stable() {
        let m = valid_material();
        let json = m.to_json();
        assert_eq!(json["asset_name"], "quartz");
        assert_eq!(json["ingredient"], "minecraft:quartz");
        assert_eq!(json["item_model_index"], serde_json::json!(0.1_f32));
        let a = serde_json::to_string_pretty(&m.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&m.to_json()).unwrap();
        assert_eq!(a, b);
    }

    /// The typed path and the raw escape hatch must produce byte-identical
    /// JSON for the same logical value — proof that #198's migration is
    /// output-stable for already-valid packs.
    #[test]
    fn typed_and_raw_material_paths_produce_identical_json() {
        let typed = valid_material()
            .description(TextComponent::literal("Quartz"))
            .override_armor_material(
                ArmorMaterialId::minecraft("iron").unwrap(),
                asset("quartz_darker"),
            );
        let raw = TrimMaterial::new(rl())
            .asset_name_raw("quartz")
            .ingredient_raw("minecraft:quartz")
            .item_model_index(0.1)
            .description_raw(serde_json::json!({"text": "Quartz"}))
            .override_armor_materials_raw(serde_json::json!({"minecraft:iron": "quartz_darker"}));
        assert_eq!(typed.to_json(), raw.to_json());
    }

    // ── TrimPattern ─────────────────────────────────────────────────────────

    #[test]
    fn valid_trim_pattern_passes_validation() {
        assert!(valid_pattern().validate().is_ok());
    }

    #[test]
    fn empty_asset_id_is_rejected() {
        let p = TrimPattern::new(rl()).template_item(item("bolt_armor_trim_smithing_template"));
        let err = p.validate().unwrap_err().to_string();
        assert!(err.contains("asset_id"), "{err}");
        assert!(err.contains("SAND-TRIM-ASSET-ID"), "{err}");
    }

    #[test]
    fn empty_template_item_is_rejected() {
        let p = TrimPattern::new(rl()).asset_id(ResourceLocation::minecraft("bolt").unwrap());
        let err = p.validate().unwrap_err().to_string();
        assert!(err.contains("template_item"), "{err}");
        assert!(err.contains("SAND-TRIM-TEMPLATE-ITEM"), "{err}");
    }

    #[test]
    fn malformed_raw_template_item_is_rejected() {
        let p = valid_pattern().template_item_raw("Not Valid!");
        assert!(p.validate().is_err());
    }

    #[test]
    fn tag_reference_template_item_is_rejected() {
        let p = valid_pattern().template_item_raw("#minecraft:trim_templates");
        let err = p.validate().unwrap_err().to_string();
        assert!(err.contains("not a tag reference"), "{err}");
    }

    #[test]
    fn tag_reference_asset_id_is_rejected() {
        let p = valid_pattern().asset_id_raw("#minecraft:bolt");
        let err = p.validate().unwrap_err().to_string();
        assert!(err.contains("SAND-TRIM-ASSET-ID"), "{err}");
    }

    #[test]
    fn invalid_trim_pattern_fails_export() {
        let p = TrimPattern::new(rl());
        assert!(p.try_content().is_err());
    }

    #[test]
    fn typed_pattern_description_serializes_and_validates() {
        let p =
            valid_pattern().description(TextComponent::translate("trim_pattern.minecraft.bolt"));
        assert_eq!(
            p.to_json()["description"],
            serde_json::json!({"translate": "trim_pattern.minecraft.bolt"})
        );
        assert!(p.validate().is_ok());

        let bad = valid_pattern().description(TextComponent::translate("\u{7}"));
        assert!(bad.validate().is_err());
    }

    #[test]
    fn valid_trim_pattern_json_is_stable() {
        let p = valid_pattern().decal(true);
        let json = p.to_json();
        assert_eq!(json["asset_id"], "minecraft:bolt");
        assert_eq!(
            json["template_item"],
            "minecraft:bolt_armor_trim_smithing_template"
        );
        assert_eq!(json["decal"], true);
        let a = serde_json::to_string_pretty(&p.to_json()).unwrap();
        let b = serde_json::to_string_pretty(&p.to_json()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn typed_and_raw_pattern_paths_produce_identical_json() {
        let typed = valid_pattern().description(TextComponent::literal("Bolt"));
        let raw = TrimPattern::new(rl())
            .asset_id_raw("minecraft:bolt")
            .template_item_raw("minecraft:bolt_armor_trim_smithing_template")
            .description_raw(serde_json::json!({"text": "Bolt"}));
        assert_eq!(typed.to_json(), raw.to_json());
    }

    #[test]
    fn resource_location_compatibility_adapter_is_accepted() {
        let m = valid_material().ingredient(ResourceLocation::new("mymod", "shard").unwrap());
        assert_eq!(m.to_json()["ingredient"], "mymod:shard");
    }
}
