pub mod trigger_coverage;

use std::collections::HashMap;

use sand_commands::{CommandProfile, TextComponent};
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use serde_json::Value;

use crate::component::{ComponentContent, DatapackComponent};
use crate::predicates::{
    DamagePredicate, DistancePredicate, EffectPredicate, EntityPredicate, FloatRange, IntRange,
    ItemPredicate, LocationPredicate,
};
use crate::raw::RawJson;
use crate::registry::{
    AdvancementId, BlockId, DimensionId, FunctionId, ItemId, LootTableId, PotionRegistryId,
    RecipeId, StatusEffectId,
};
use crate::resource_location::ResourceLocation;

fn validate_resource_id(value: &str, path: &str) -> Result<(), String> {
    value
        .parse::<ResourceLocation>()
        .map(|_| ())
        .map_err(|_| format!("{path}: `{value}` must be a valid namespaced resource location"))
}

fn json_value<T: Serialize, E: serde::ser::Error>(value: &T) -> Result<Value, E> {
    serde_json::to_value(value).map_err(E::custom)
}

// ── AdvancementFrame ──────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::AdvancementFrame",
    aliases = ["sand::prelude::AdvancementFrame"],
    module = "sand::component",
    summary = "The visual frame style for an advancement in the advancement screen.",
    context = "The visual frame style for an advancement in the advancement screen. Determines how the advancement appears to the player when completed.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::AdvancementFrame;",
    variants(Challenge = "Uses Minecraft's challenge advancement frame style.", Goal = "Uses Minecraft's goal advancement frame style.", Task = "Uses Minecraft's task advancement frame style."),
)]
/// The visual frame style for an advancement in the advancement screen.
///
/// Determines how the advancement appears to the player when completed.
pub enum AdvancementFrame {
    #[doc = "Uses Minecraft's task advancement frame style."]
    Task,
    #[doc = "Uses Minecraft's goal advancement frame style."]
    Goal,
    #[doc = "Uses Minecraft's challenge advancement frame style."]
    Challenge,
}

impl AdvancementFrame {
    fn as_str(&self) -> &'static str {
        match self {
            AdvancementFrame::Task => "task",
            AdvancementFrame::Goal => "goal",
            AdvancementFrame::Challenge => "challenge",
        }
    }
}

// ── AdvancementIcon ───────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::AdvancementIcon",
    aliases = ["sand::prelude::AdvancementIcon"],
    module = "sand::component",
    summary = "The icon displayed for an advancement, with optional item components.",
    context = "The icon displayed for an advancement, with optional item components. The normal constructor accepts only item-registry IDs, so a block tag or another registry kind cannot be passed accidentally:",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::AdvancementIcon;",
)]
/// The icon displayed for an advancement, with optional item components.
///
/// The normal constructor accepts only item-registry IDs, so a block tag or
/// another registry kind cannot be passed accidentally:
///
/// ```compile_fail
/// use sand_components::{AdvancementIcon, BlockId};
///
/// let block = BlockId::minecraft("stone").unwrap();
/// let _icon = AdvancementIcon::new(block);
/// ```
pub struct AdvancementIcon {
    id: String,
    components: Option<RawJson>,
}

impl AdvancementIcon {
    /// Creates a new advancement icon through the typed item-ID path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementIcon::new",
        aliases = ["sand::prelude::AdvancementIcon::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new advancement icon through the typed item-ID path.",
        context = "Creates a new advancement icon through the typed item-ID path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to create a new advancement icon through the typed item-ID path."),
        returns = "An `AdvancementIcon` representing a new advancement icon through the typed item-ID path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::registry::ItemId)  {\n    let advancement_icon = sand::component::AdvancementIcon::new(id);\n}",
    )]
    pub fn new(id: ItemId) -> Self {
        Self {
            id: id.to_string(),
            components: None,
        }
    }

    /// Creates an advancement icon through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementIcon::raw",
        aliases = ["sand::prelude::AdvancementIcon::raw"],
        module = "sand::component",
        kind = "method",
        summary = "Creates an advancement icon through the explicit raw compatibility path.",
        context = "Creates an advancement icon through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to create an advancement icon through the explicit raw compatibility path."),
        returns = "An `AdvancementIcon` representing an advancement icon through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: impl Into < String >)  {\n    let advancement_icon = sand::component::AdvancementIcon::raw(id);\n}",
    )]
    pub fn raw(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            components: None,
        }
    }

    /// Sets the item components for this icon using an explicit [`RawJson`] escape hatch.
    ///
    /// Use this for icon component overrides (e.g. enchantments, custom model data)
    /// that are not yet modelled by the typed item component API.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementIcon::components",
        aliases = ["sand::prelude::AdvancementIcon::components"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the item components for this icon using an explicit [`RawJson`] escape hatch.",
        context = "Sets the item components for this icon using an explicit [`RawJson`] escape hatch. Use this for icon component overrides (e.g. enchantments, custom model data) that are not yet modelled by the typed item component API.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Use this for icon component overrides (e.g. enchantments, custom model data) that are not yet modelled by the typed item component API."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(components = "`components` provides the components applied when setting the item components for this icon using an explicit [`RawJson`] escape hatch."),
        returns = "The `AdvancementIcon` value with the documented change applied to set the item components for this icon using an explicit [`RawJson`] escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_icon_value: sand::component::AdvancementIcon, components: sand::component::RawJson)  {\n    let updated_advancement_icon = advancement_icon_value.components(components);\n}",
    )]
    pub fn components(mut self, components: RawJson) -> Self {
        self.components = Some(components);
        self
    }
}

impl Serialize for AdvancementIcon {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id)?;
        if let Some(ref c) = self.components {
            map.serialize_entry("components", c)?;
        }
        map.end()
    }
}

// ── AdvancementDisplay ────────────────────────────────────────────────────────

enum AdvancementText {
    Typed(Box<TextComponent>),
    Raw(RawJson),
}

impl AdvancementText {
    fn validate(&self, location: &ResourceLocation, field: &str) -> crate::error::Result<()> {
        let error = match self {
            Self::Typed(text) => text.validate_at_path(&CommandProfile::unprofiled(), field),
            Self::Raw(text) => sand_commands::text::validate_json_text(
                text.as_value(),
                &CommandProfile::unprofiled(),
                field,
            ),
        };
        error.map_err(|error| crate::error::SandError::ComponentValidation {
            location: location.clone(),
            kind: "advancement".to_string(),
            field: error.field,
            message: format!("error[{}] {}", error.code, error.message),
        })
    }
}

impl Serialize for AdvancementText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Typed(text) => {
                let value = text
                    .try_to_json_value()
                    .map_err(serde::ser::Error::custom)?;
                value.serialize(serializer)
            }
            Self::Raw(text) => text.serialize(serializer),
        }
    }
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::AdvancementDisplay",
    aliases = ["sand::prelude::AdvancementDisplay"],
    module = "sand::component",
    summary = "The display information shown for an advancement in the advancement screen and toast.",
    context = "The display information shown for an advancement in the advancement screen and toast. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::AdvancementDisplay;",
)]
/// The display information shown for an advancement in the advancement screen and toast.
pub struct AdvancementDisplay {
    icon: AdvancementIcon,
    title: AdvancementText,
    description: AdvancementText,
    background: Option<String>,
    frame: AdvancementFrame,
    show_toast: bool,
    announce_to_chat: bool,
    hidden: bool,
}

impl AdvancementDisplay {
    /// Creates a display using typed Minecraft text components.
    ///
    /// ```
    /// use sand_commands::Text;
    /// use sand_components::{AdvancementDisplay, AdvancementIcon, ItemId};
    ///
    /// let display = AdvancementDisplay::new(
    ///     AdvancementIcon::new(ItemId::minecraft("diamond").unwrap()),
    ///     Text::new("Diamond Collector").aqua().bold(true),
    ///     Text::new("Find a diamond"),
    /// );
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::new",
        aliases = ["sand::prelude::AdvancementDisplay::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a display using typed Minecraft text components.",
        context = "Creates a display using typed Minecraft text components. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(icon = "`icon` is used when creating a display using typed Minecraft text components.", title = "`title` is used when creating a display using typed Minecraft text components.", description = "`description` is used when creating a display using typed Minecraft text components."),
        returns = "An `AdvancementDisplay` representing a display using typed Minecraft text components.",
        example = "use sand::text::Text;\nuse {sand::component::AdvancementDisplay, sand::component::AdvancementIcon, sand::registry::ItemId};\nlet display = AdvancementDisplay::new(\nAdvancementIcon::new(ItemId::minecraft(\"diamond\").unwrap()),\nText::new(\"Diamond Collector\").aqua().bold(true),\nText::new(\"Find a diamond\"),\n);",
    )]
    pub fn new(icon: AdvancementIcon, title: TextComponent, description: TextComponent) -> Self {
        Self {
            icon,
            title: AdvancementText::Typed(Box::new(title)),
            description: AdvancementText::Typed(Box::new(description)),
            background: None,
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_to_chat: true,
            hidden: false,
        }
    }

    /// Creates a display with explicitly raw Minecraft text JSON.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::raw_text",
        aliases = ["sand::prelude::AdvancementDisplay::raw_text"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a display with explicitly raw Minecraft text JSON.",
        context = "Creates a display with explicitly raw Minecraft text JSON. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(icon = "`icon` is used when creating a display with explicitly raw Minecraft text JSON.", title = "`title` is used when creating a display with explicitly raw Minecraft text JSON.", description = "`description` is used when creating a display with explicitly raw Minecraft text JSON."),
        returns = "An `AdvancementDisplay` representing a display with explicitly raw Minecraft text JSON.",
        example = "use sand::prelude::*;\n\nfn demonstrate(icon: sand::component::AdvancementIcon, title: sand::component::RawJson, description: sand::component::RawJson)  {\n    let advancement_display = sand::component::AdvancementDisplay::raw_text(icon, title, description);\n}",
    )]
    pub fn raw_text(icon: AdvancementIcon, title: RawJson, description: RawJson) -> Self {
        Self {
            icon,
            title: AdvancementText::Raw(title),
            description: AdvancementText::Raw(description),
            background: None,
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_to_chat: true,
            hidden: false,
        }
    }

    /// Replaces the title through the explicit raw text escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::raw_title",
        aliases = ["sand::prelude::AdvancementDisplay::raw_title"],
        module = "sand::component",
        kind = "method",
        summary = "Replaces the title through the explicit raw text escape hatch.",
        context = "Replaces the title through the explicit raw text escape hatch. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(title = "`title` provides the replacement title when the title through the explicit raw text escape hatch."),
        returns = "The `AdvancementDisplay` value with the documented change applied to replace the title through the explicit raw text escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, title: sand::component::RawJson)  {\n    let updated_advancement_display = advancement_display_value.raw_title(title);\n}",
    )]
    pub fn raw_title(mut self, title: RawJson) -> Self {
        self.title = AdvancementText::Raw(title);
        self
    }

    /// Replaces the description through the explicit raw text escape hatch.
    ///
    /// `description` is an unchecked raw JSON text payload, not a typed
    /// player-visible [`TextComponent`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::raw_description",
        aliases = ["sand::prelude::AdvancementDisplay::raw_description"],
        module = "sand::component",
        kind = "method",
        summary = "Replaces the description through the explicit raw text escape hatch.",
        context = "Replaces the description through the explicit raw text escape hatch. `description` is an unchecked raw JSON text payload, not a typed player-visible [`TextComponent`].",
        minecraft = "`description` is an unchecked raw JSON text payload, not a typed player-visible [`TextComponent`].",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(description = "`description` is an unchecked raw JSON text payload, not a typed player-visible [`TextComponent`]."),
        returns = "The `AdvancementDisplay` value with the documented change applied to replace the description through the explicit raw text escape hatch.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, description: sand::component::RawJson)  {\n    let updated_advancement_display = advancement_display_value.raw_description(description);\n}",
    )]
    pub fn raw_description(mut self, description: RawJson) -> Self {
        self.description = AdvancementText::Raw(description);
        self
    }

    /// Sets the typed background texture for the advancement tab.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::background",
        aliases = ["sand::prelude::AdvancementDisplay::background"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the typed background texture for the advancement tab.",
        context = "Sets the typed background texture for the advancement tab. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(bg = "`bg` provides the typed Minecraft resource identifier used to set the typed background texture for the advancement tab."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set the typed background texture for the advancement tab.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, bg: sand::ResourceLocation)  {\n    let updated_advancement_display = advancement_display_value.background(bg);\n}",
    )]
    pub fn background(mut self, bg: ResourceLocation) -> Self {
        self.background = Some(bg.to_string());
        self
    }

    /// Sets the background through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::raw_background",
        aliases = ["sand::prelude::AdvancementDisplay::raw_background"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the background through the explicit raw compatibility path.",
        context = "Sets the background through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(bg = "`bg` provides the bg applied when setting the background through the explicit raw compatibility path."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set the background through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, bg: impl Into < String >)  {\n    let updated_advancement_display = advancement_display_value.raw_background(bg);\n}",
    )]
    pub fn raw_background(mut self, bg: impl Into<String>) -> Self {
        self.background = Some(bg.into());
        self
    }

    /// Sets the frame style for this advancement display.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::frame",
        aliases = ["sand::prelude::AdvancementDisplay::frame"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the frame style for this advancement display.",
        context = "Sets the frame style for this advancement display. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(frame = "`frame` provides the frame applied when setting the frame style for this advancement display."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set the frame style for this advancement display.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, frame: sand::component::AdvancementFrame)  {\n    let updated_advancement_display = advancement_display_value.frame(frame);\n}",
    )]
    pub fn frame(mut self, frame: AdvancementFrame) -> Self {
        self.frame = frame;
        self
    }

    /// Sets whether a toast notification is shown when this advancement is completed.
    ///
    /// `v` enables or suppresses only the top-right completion toast; chat
    /// announcements and advancement-screen visibility remain independently
    /// controlled by their own display settings.
    ///
    /// # Example
    ///
    /// ```
    /// use sand_commands::Text;
    /// use sand_components::{AdvancementDisplay, AdvancementIcon, ItemId};
    ///
    /// let display = AdvancementDisplay::new(
    ///     AdvancementIcon::new(ItemId::minecraft("diamond").unwrap()),
    ///     Text::new("Hidden toast"),
    ///     Text::new("This advancement does not pop up a toast"),
    /// )
    /// .show_toast(false);
    /// let json = serde_json::to_value(display).unwrap();
    /// assert_eq!(json["show_toast"], false);
    /// ```
    ///
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::show_toast",
        aliases = ["sand::prelude::AdvancementDisplay::show_toast"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether a toast notification is shown when this advancement is completed.",
        context = "Sets whether a toast notification is shown when this advancement is completed. `v` enables or suppresses only the top-right completion toast; chat announcements and advancement-screen visibility remain independently controlled by their own display settings.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` enables or suppresses only the top-right completion toast; chat announcements and advancement-screen visibility remain independently controlled by their own display settings."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set whether a toast notification is shown when this advancement is completed.",
        example = "use sand::text::Text;\nuse {sand::component::AdvancementDisplay, sand::component::AdvancementIcon, sand::registry::ItemId};\nlet display = AdvancementDisplay::new(\nAdvancementIcon::new(ItemId::minecraft(\"diamond\").unwrap()),\nText::new(\"Hidden toast\"),\nText::new(\"This advancement does not pop up a toast\"),\n)\n.show_toast(false);\nlet json = serde_json::to_value(display).unwrap();\nassert_eq!(json[\"show_toast\"], false);",
    )]
    pub fn show_toast(mut self, v: bool) -> Self {
        self.show_toast = v;
        self
    }

    /// Sets whether this advancement completion is announced in chat.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::announce_to_chat",
        aliases = ["sand::prelude::AdvancementDisplay::announce_to_chat"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether this advancement completion is announced in chat.",
        context = "Sets whether this advancement completion is announced in chat. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether this advancement completion is announced in chat."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set whether this advancement completion is announced in chat.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, v: bool)  {\n    let updated_advancement_display = advancement_display_value.announce_to_chat(v);\n}",
    )]
    pub fn announce_to_chat(mut self, v: bool) -> Self {
        self.announce_to_chat = v;
        self
    }

    /// Sets whether this advancement is hidden until completed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementDisplay::hidden",
        aliases = ["sand::prelude::AdvancementDisplay::hidden"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether this advancement is hidden until completed.",
        context = "Sets whether this advancement is hidden until completed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether this advancement is hidden until completed."),
        returns = "The `AdvancementDisplay` value with the documented change applied to set whether this advancement is hidden until completed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_display_value: sand::component::AdvancementDisplay, v: bool)  {\n    let updated_advancement_display = advancement_display_value.hidden(v);\n}",
    )]
    pub fn hidden(mut self, v: bool) -> Self {
        self.hidden = v;
        self
    }
}

impl Serialize for AdvancementDisplay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("icon", &self.icon)?;
        map.serialize_entry("title", &self.title)?;
        map.serialize_entry("description", &self.description)?;
        if let Some(ref bg) = self.background {
            map.serialize_entry("background", bg)?;
        }
        map.serialize_entry("frame", self.frame.as_str())?;
        map.serialize_entry("show_toast", &self.show_toast)?;
        map.serialize_entry("announce_to_chat", &self.announce_to_chat)?;
        map.serialize_entry("hidden", &self.hidden)?;
        map.end()
    }
}

// ── AdvancementTrigger ────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::AdvancementTrigger",
    aliases = ["sand::prelude::AdvancementTrigger"],
    module = "sand::component",
    summary = "Represents a trigger condition for an advancement criterion.",
    context = "Represents a trigger condition for an advancement criterion. Each variant uses typed predicate structs from [`sand::predicate`] instead of raw `serde_json::Value`. Prefer the typed associated constructors for variants whose public fields remain strings for source compatibility. The [`Custom`](AdvancementTrigger::Custom) variant is the legacy raw shape; [`AdvancementTrigger::custom_trigger`] is the validated normal path for custom/modded triggers.",
    minecraft = "Each variant uses typed predicate structs from [`sand::predicate`] instead of raw `serde_json::Value`. Prefer the typed associated constructors for variants whose public fields remain strings for source compatibility. The [`Custom`](AdvancementTrigger::Custom) variant is the legacy raw shape; [`AdvancementTrigger::custom_trigger`] is the validated normal path for custom/modded triggers.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::AdvancementTrigger;",
    variants(AllayDropItemOnBlock = "Player causes an allay to drop an item on a block (1.19+).", AvoidVibration = "Player avoids triggering a sculk sensor vibration (1.19+).", BeeNestDestroyed = "Player destroys a bee nest or beehive.", BredAnimals = "Matches Minecraft's bred animals advancement trigger.", BrewedPotion = "Player brews a potion.", ChangedDimension = "Matches Minecraft's changed dimension advancement trigger.", ChanneledLightning = "A lightning bolt hits an entity the player summoned with a trident.", ConstructBeacon = "Matches Minecraft's construct beacon advancement trigger.", ConsumeItem = "Matches Minecraft's consume item advancement trigger.", CraftedItem = "Player crafts an item.", CuredZombieVillager = "Matches Minecraft's cured zombie villager advancement trigger.", Custom = "Any trigger not covered by the typed variants. Use this to target triggers that were added to or removed from Minecraft after a given version, or for modded triggers.", EffectsChanged = "Matches Minecraft's effects changed advancement trigger.", EmptiedBucket = "Player empties a bucket.", EnchantedItem = "Player enchants an item.", EnterBlock = "Matches Minecraft's enter block advancement trigger.", EntityHurtPlayer = "Entity deals damage to the player.", EntityKilledPlayer = "Matches Minecraft's entity killed player advancement trigger.", FallFromHeight = "Matches Minecraft's fall from height advancement trigger.", FilledBucket = "Player fills a bucket.", FishingRodHooked = "Player uses a fishing rod and it hooks something.", HeroOfTheVillage = "Matches Minecraft's hero of the village advancement trigger.", Impossible = "Matches Minecraft's impossible advancement trigger.", InventoryChanged = "Matches Minecraft's inventory changed advancement trigger.", ItemDurabilityChanged = "An item in the player's inventory loses durability.", ItemUsedOnBlock = "Player right-clicks on a block while holding an item (1.19.4+).", KillMobNearSculkCatalyst = "Player kills a mob near a sculk catalyst (1.19+).", KilledByArrow = "Player kills one or more entities with a projectile weapon.", KilledByCrossbow = "Player kills an entity using a crossbow.", LeveledUp = "Matches Minecraft's leveled up advancement trigger.", LightningStrike = "A lightning bolt strikes near the player.", Location = "Matches Minecraft's location advancement trigger.", NetherTravel = "Matches Minecraft's nether travel advancement trigger.", PlacedBlock = "Matches Minecraft's placed block advancement trigger.", PlayerGeneratesContainerLoot = "Matches Minecraft's player generates container loot advancement trigger.", PlayerHurtEntity = "Player deals damage to an entity.", PlayerInteractedWithEntity = "Matches Minecraft's player interacted with entity advancement trigger.", PlayerKilledEntity = "Matches Minecraft's player killed entity advancement trigger.", RecipeCrafted = "Player completes a recipe. Vanilla exposes recipe and ingredient predicates, not the crafted result item.", RecipeUnlocked = "Matches Minecraft's recipe unlocked advancement trigger.", RideEntityInLava = "Player rides an entity in lava (1.16+).", ShotCrossbow = "Player shoots a crossbow.", SleptInBed = "Matches Minecraft's slept in bed advancement trigger.", SlideDownBlock = "Matches Minecraft's slide down block advancement trigger.", StartedRiding = "Matches Minecraft's started riding advancement trigger.", SummonedEntity = "Matches Minecraft's summoned entity advancement trigger.", TamedAnimal = "Matches Minecraft's tamed animal advancement trigger.", TamedAnimalInteracted = "Matches Minecraft's tamed animal interacted advancement trigger.", TargetHit = "Matches Minecraft's target hit advancement trigger.", ThrownItemPickedUp = "A thrown item is picked up by an entity.", ThrownItemPickedUpByEntity = "A thrown item is picked up by a non-player entity.", ThrownItemPickedUpByPlayer = "A thrown item is picked up by the player.", Tick = "Matches Minecraft's tick advancement trigger.", UsedEnderEye = "Matches Minecraft's used ender eye advancement trigger.", UsedItem = "Matches Minecraft's used item advancement trigger.", UsedTotem = "Player activates a totem of undying.", UsingItem = "Matches Minecraft's using item advancement trigger.", VillagerTrade = "Matches Minecraft's villager trade advancement trigger."),
    variant_fields(AllayDropItemOnBlock(item = "`item` optionally restricts the dropped item that satisfies this trigger.", location = "`location` optionally narrows the location predicate matched when a player causes an allay to drop an item on a block."), BeeNestDestroyed(block = "`block` optionally narrows the block matched when a player destroys a bee nest or beehive.", item = "`item` optionally narrows the item predicate matched when a player destroys a bee nest or beehive.", num_bees_inside = "`num_bees_inside` optionally provides the num bees inside when a player destroys a bee nest or beehive."), BredAnimals(child = "`child` optionally narrows the child predicate matched for Minecraft's bred animals advancement trigger.", parent = "`parent` optionally narrows the parent predicate matched for Minecraft's bred animals advancement trigger.", partner = "`partner` optionally narrows the partner predicate matched for Minecraft's bred animals advancement trigger."), BrewedPotion(potion = "`potion` optionally provides the potion when a player brews a potion."), ChangedDimension(from = "`from` optionally narrows the source value matched for Minecraft's changed dimension advancement trigger.", to = "`to` optionally narrows the destination value matched for Minecraft's changed dimension advancement trigger."), ChanneledLightning(victims = "`victims` optionally narrows the victims predicate matched when a lightning bolt hits an entity the player summoned with a trident."), ConstructBeacon(level = "`level` optionally narrows the level range matched for Minecraft's construct beacon advancement trigger."), ConsumeItem(item = "`item` optionally narrows the item predicate matched for Minecraft's consume item advancement trigger."), CraftedItem(item = "`item` optionally narrows the item predicate matched when a player crafts an item."), CuredZombieVillager(villager = "`villager` optionally narrows the villager predicate matched for Minecraft's cured zombie villager advancement trigger.", zombie = "`zombie` optionally narrows the zombie predicate matched for Minecraft's cured zombie villager advancement trigger."), Custom(conditions = "Raw JSON conditions block.  Use [`RawJson`] to signal intentional opt-out of the typed predicate API.", trigger = "`trigger` provides the trigger when any trigger not covered by the typed variants. Use this to target triggers that were added to or removed from Minecraft after a given version, or for modded triggers."), EffectsChanged(effects = "`effects` optionally narrows the effects predicate matched for Minecraft's effects changed advancement trigger.", source = "`source` optionally narrows the source predicate matched for Minecraft's effects changed advancement trigger."), EmptiedBucket(item = "`item` optionally narrows the item predicate matched when a player empties a bucket.", location = "`location` optionally narrows the location predicate matched when a player empties a bucket."), EnchantedItem(item = "`item` optionally narrows the item predicate matched when a player enchants an item.", levels = "`levels` optionally narrows the level range matched when a player enchants an item."), EnterBlock(block = "`block` optionally narrows the block matched for Minecraft's enter block advancement trigger.", state = "`state` optionally provides the state for Minecraft's enter block advancement trigger."), EntityHurtPlayer(damage = "`damage` optionally narrows the damage predicate matched when entity deals damage to the player.", entity = "`entity` optionally narrows the entity predicate matched when entity deals damage to the player."), EntityKilledPlayer(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's entity killed player advancement trigger.", killing_blow = "`killing_blow` optionally narrows the killing blow predicate matched for Minecraft's entity killed player advancement trigger."), FallFromHeight(distance = "`distance` optionally narrows the distance predicate matched for Minecraft's fall from height advancement trigger.", start_position = "`start_position` optionally narrows the start position predicate matched for Minecraft's fall from height advancement trigger."), FilledBucket(item = "`item` optionally narrows the item predicate matched when a player fills a bucket."), FishingRodHooked(entity = "`entity` optionally narrows the entity predicate matched when a player uses a fishing rod and it hooks something.", item = "`item` optionally narrows the item predicate matched when a player uses a fishing rod and it hooks something.", rod = "`rod` optionally narrows the rod predicate matched when a player uses a fishing rod and it hooks something."), HeroOfTheVillage(location = "`location` optionally narrows the location predicate matched for Minecraft's hero of the village advancement trigger."), InventoryChanged(items = "`items` provides the items predicate for Minecraft's inventory changed advancement trigger.", slots = "`slots` optionally narrows the slots predicate matched for Minecraft's inventory changed advancement trigger."), ItemDurabilityChanged(delta = "`delta` optionally provides the delta when an item in the player's inventory loses durability.", durability = "`durability` optionally provides the durability when an item in the player's inventory loses durability.", item = "`item` optionally narrows the item predicate matched when an item in the player's inventory loses durability."), ItemUsedOnBlock(item = "`item` optionally narrows the item predicate matched when a player right-clicks on a block while holding an item.", location = "`location` optionally narrows the location predicate matched when a player right-clicks on a block while holding an item."), KillMobNearSculkCatalyst(entity = "`entity` optionally narrows the entity predicate matched when a player kills a mob near a sculk catalyst.", killing_blow = "`killing_blow` optionally narrows the killing blow predicate matched when a player kills a mob near a sculk catalyst."), KilledByArrow(fired_from_weapon = "`fired_from_weapon` optionally narrows the fired from weapon predicate matched when a player kills one or more entities with a projectile weapon.", unique_entity_types = "`unique_entity_types` optionally provides the unique entity types when a player kills one or more entities with a projectile weapon.", victims = "`victims` optionally narrows the victims predicate matched when a player kills one or more entities with a projectile weapon."), KilledByCrossbow(unique_entity_types = "`unique_entity_types` optionally provides the unique entity types when a player kills an entity using a crossbow.", victims = "`victims` optionally narrows the victims predicate matched when a player kills an entity using a crossbow."), LeveledUp(level = "`level` optionally narrows the level range matched for Minecraft's leveled up advancement trigger."), LightningStrike(bystander = "`bystander` optionally narrows the bystander predicate matched when a lightning bolt strikes near the player.", lightning = "`lightning` optionally narrows the lightning predicate matched when a lightning bolt strikes near the player."), Location(location = "`location` optionally narrows the location predicate matched for Minecraft's location advancement trigger."), NetherTravel(distance = "`distance` optionally narrows the distance predicate matched for Minecraft's nether travel advancement trigger.", entered = "`entered` optionally narrows the entered predicate matched for Minecraft's nether travel advancement trigger.", exited = "`exited` optionally narrows the exited predicate matched for Minecraft's nether travel advancement trigger."), PlacedBlock(block = "`block` optionally narrows the block matched for Minecraft's placed block advancement trigger.", item = "`item` optionally narrows the item predicate matched for Minecraft's placed block advancement trigger.", location = "`location` optionally narrows the location predicate matched for Minecraft's placed block advancement trigger.", state = "`state` optionally provides the state for Minecraft's placed block advancement trigger."), PlayerGeneratesContainerLoot(loot_table = "`loot_table` optionally provides the loot table for Minecraft's player generates container loot advancement trigger."), PlayerHurtEntity(damage = "`damage` optionally narrows the damage predicate matched when a player deals damage to an entity.", entity = "`entity` optionally narrows the entity predicate matched when a player deals damage to an entity."), PlayerInteractedWithEntity(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's player interacted with entity advancement trigger.", item = "`item` optionally narrows the item predicate matched for Minecraft's player interacted with entity advancement trigger."), PlayerKilledEntity(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's player killed entity advancement trigger.", killing_blow = "`killing_blow` optionally narrows the killing blow predicate matched for Minecraft's player killed entity advancement trigger."), RecipeCrafted(ingredients = "`ingredients` provides the ingredients predicate when a player completes a recipe. Vanilla exposes recipe and ingredient predicates, not the crafted result item.", recipe_id = "`recipe_id` provides the recipe id when a player completes a recipe. Vanilla exposes recipe and ingredient predicates, not the crafted result item."), RecipeUnlocked(recipe = "`recipe` provides the recipe for Minecraft's recipe unlocked advancement trigger."), RideEntityInLava(distance = "`distance` optionally narrows the distance predicate matched when a player rides an entity in lava.", start_position = "`start_position` optionally narrows the start position predicate matched when a player rides an entity in lava."), ShotCrossbow(item = "`item` optionally narrows the item predicate matched when a player shoots a crossbow."), SleptInBed(location = "`location` optionally narrows the location predicate matched for Minecraft's slept in bed advancement trigger."), SlideDownBlock(block = "`block` optionally narrows the block matched for Minecraft's slide down block advancement trigger."), SummonedEntity(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's summoned entity advancement trigger."), TamedAnimal(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's tamed animal advancement trigger."), TamedAnimalInteracted(entity = "`entity` optionally narrows the entity predicate matched for Minecraft's tamed animal interacted advancement trigger.", item = "`item` optionally narrows the item predicate matched for Minecraft's tamed animal interacted advancement trigger."), TargetHit(projectile = "`projectile` optionally narrows the projectile predicate matched for Minecraft's target hit advancement trigger.", signal_strength = "`signal_strength` optionally provides the signal strength for Minecraft's target hit advancement trigger."), ThrownItemPickedUp(entity = "`entity` optionally narrows the entity predicate matched when a thrown item is picked up by an entity.", item = "`item` optionally narrows the item predicate matched when a thrown item is picked up by an entity."), ThrownItemPickedUpByEntity(entity = "`entity` optionally narrows the entity predicate matched when a thrown item is picked up by a non-player entity.", item = "`item` optionally narrows the item predicate matched when a thrown item is picked up by a non-player entity."), ThrownItemPickedUpByPlayer(entity = "`entity` optionally narrows the entity predicate matched when a thrown item is picked up by the player.", item = "`item` optionally narrows the item predicate matched when a thrown item is picked up by the player."), UsedEnderEye(distance = "`distance` optionally narrows the distance matched for Minecraft's used ender eye advancement trigger."), UsedItem(item = "`item` optionally narrows the item predicate matched for Minecraft's used item advancement trigger."), UsedTotem(item = "`item` optionally narrows the item predicate matched when a player activates a totem of undying."), UsingItem(item = "`item` optionally narrows the item predicate matched for Minecraft's using item advancement trigger."), VillagerTrade(item = "`item` optionally narrows the item predicate matched for Minecraft's villager trade advancement trigger.", villager = "`villager` optionally narrows the villager predicate matched for Minecraft's villager trade advancement trigger.")),
)]
/// Represents a trigger condition for an advancement criterion.
///
/// Each variant uses typed predicate structs from [`crate::predicates`]
/// instead of raw `serde_json::Value`. Prefer the typed associated constructors
/// for variants whose public fields remain strings for source compatibility.
/// The [`Custom`](AdvancementTrigger::Custom) variant is the legacy raw shape;
/// [`AdvancementTrigger::custom_trigger`] is the validated normal path for
/// custom/modded triggers.
///
/// # Escape hatch
///
/// ```rust
/// use sand_components::{AdvancementTrigger, RawJson};
/// use serde_json::json;
///
/// let t = AdvancementTrigger::Custom {
///     trigger: "mymod:custom_trigger".into(),
///     conditions: Some(RawJson::new(json!({"level": 5}))),
/// };
/// ```
#[allow(clippy::large_enum_variant)]
pub enum AdvancementTrigger {
    #[doc = "Matches Minecraft's tick advancement trigger."]
    Tick,
    #[doc = "Matches Minecraft's impossible advancement trigger."]
    Impossible,

    // ── Kill / combat ─────────────────────────────────────────────────────────
    #[doc = "Matches Minecraft's player killed entity advancement trigger."]
    PlayerKilledEntity {
        /// `entity` optionally narrows the entity predicate matched for Minecraft's player killed entity advancement trigger.
        entity: Option<EntityPredicate>,
        /// `killing_blow` optionally narrows the killing blow predicate matched for Minecraft's player killed entity advancement trigger.
        killing_blow: Option<DamagePredicate>,
    },
    #[doc = "Matches Minecraft's entity killed player advancement trigger."]
    EntityKilledPlayer {
        /// `entity` optionally narrows the entity predicate matched for Minecraft's entity killed player advancement trigger.
        entity: Option<EntityPredicate>,
        /// `killing_blow` optionally narrows the killing blow predicate matched for Minecraft's entity killed player advancement trigger.
        killing_blow: Option<DamagePredicate>,
    },
    /// Player deals damage to an entity.
    PlayerHurtEntity {
        /// `entity` optionally narrows the entity predicate matched when a player deals damage to an entity.
        entity: Option<EntityPredicate>,
        /// `damage` optionally narrows the damage predicate matched when a player deals damage to an entity.
        damage: Option<DamagePredicate>,
    },
    /// Entity deals damage to the player.
    EntityHurtPlayer {
        /// `entity` optionally narrows the entity predicate matched when entity deals damage to the player.
        entity: Option<EntityPredicate>,
        /// `damage` optionally narrows the damage predicate matched when entity deals damage to the player.
        damage: Option<DamagePredicate>,
    },
    /// Player kills an entity using a crossbow.
    KilledByCrossbow {
        /// `unique_entity_types` optionally provides the unique entity types when a player kills an entity using a crossbow.
        unique_entity_types: Option<IntRange>,
        /// `victims` optionally narrows the victims predicate matched when a player kills an entity using a crossbow.
        victims: Option<Vec<EntityPredicate>>,
    },
    /// Player kills one or more entities with a projectile weapon.
    KilledByArrow {
        /// `unique_entity_types` optionally provides the unique entity types when a player kills one or more entities with a projectile weapon.
        unique_entity_types: Option<IntRange>,
        /// `fired_from_weapon` optionally narrows the fired from weapon predicate matched when a player kills one or more entities with a projectile weapon.
        fired_from_weapon: Option<ItemPredicate>,
        /// `victims` optionally narrows the victims predicate matched when a player kills one or more entities with a projectile weapon.
        victims: Option<Vec<EntityPredicate>>,
    },
    /// A lightning bolt hits an entity the player summoned with a trident.
    ChanneledLightning {
        /// `victims` optionally narrows the victims predicate matched when a lightning bolt hits an entity the player summoned with a trident.
        victims: Option<Vec<EntityPredicate>>,
    },
    /// A lightning bolt strikes near the player.
    LightningStrike {
        /// `lightning` optionally narrows the lightning predicate matched when a lightning bolt strikes near the player.
        lightning: Option<EntityPredicate>,
        /// `bystander` optionally narrows the bystander predicate matched when a lightning bolt strikes near the player.
        bystander: Option<EntityPredicate>,
    },

    // ── Inventory / items ─────────────────────────────────────────────────────
    #[doc = "Matches Minecraft's inventory changed advancement trigger."]
    InventoryChanged {
        /// `slots` optionally narrows the slots predicate matched for Minecraft's inventory changed advancement trigger.
        slots: Option<InventorySlotsPredicate>,
        /// `items` provides the items predicate for Minecraft's inventory changed advancement trigger.
        items: Vec<ItemPredicate>,
    },
    #[doc = "Matches Minecraft's recipe unlocked advancement trigger."]
    RecipeUnlocked {
        /// `recipe` provides the recipe for Minecraft's recipe unlocked advancement trigger.
        recipe: String,
    },
    #[doc = "Matches Minecraft's used item advancement trigger."]
    UsedItem {
        /// `item` optionally narrows the item predicate matched for Minecraft's used item advancement trigger.
        item: Option<ItemPredicate>,
    },
    #[doc = "Matches Minecraft's consume item advancement trigger."]
    ConsumeItem {
        /// `item` optionally narrows the item predicate matched for Minecraft's consume item advancement trigger.
        item: Option<ItemPredicate>,
    },
    #[doc = "Matches Minecraft's using item advancement trigger."]
    UsingItem {
        /// `item` optionally narrows the item predicate matched for Minecraft's using item advancement trigger.
        item: Option<ItemPredicate>,
    },
    /// Player crafts an item.
    CraftedItem {
        /// `item` optionally narrows the item predicate matched when a player crafts an item.
        item: Option<ItemPredicate>,
    },
    /// Player completes a recipe. Vanilla exposes recipe and ingredient
    /// predicates, not the crafted result item.
    RecipeCrafted {
        /// `recipe_id` provides the recipe id when a player completes a recipe. Vanilla exposes recipe and ingredient predicates, not the crafted result item.
        recipe_id: String,
        /// `ingredients` provides the ingredients predicate when a player completes a recipe. Vanilla exposes recipe and ingredient predicates, not the crafted result item.
        ingredients: Vec<ItemPredicate>,
    },
    /// Player fills a bucket.
    FilledBucket {
        /// `item` optionally narrows the item predicate matched when a player fills a bucket.
        item: Option<ItemPredicate>,
    },
    /// Player empties a bucket.
    EmptiedBucket {
        /// `item` optionally narrows the item predicate matched when a player empties a bucket.
        item: Option<ItemPredicate>,
        /// `location` optionally narrows the location predicate matched when a player empties a bucket.
        location: Option<LocationPredicate>,
    },
    /// Player shoots a crossbow.
    ShotCrossbow {
        /// `item` optionally narrows the item predicate matched when a player shoots a crossbow.
        item: Option<ItemPredicate>,
    },
    /// Player activates a totem of undying.
    UsedTotem {
        /// `item` optionally narrows the item predicate matched when a player activates a totem of undying.
        item: Option<ItemPredicate>,
    },
    /// A thrown item is picked up by an entity.
    ThrownItemPickedUp {
        /// `item` optionally narrows the item predicate matched when a thrown item is picked up by an entity.
        item: Option<ItemPredicate>,
        /// `entity` optionally narrows the entity predicate matched when a thrown item is picked up by an entity.
        entity: Option<EntityPredicate>,
    },
    /// A thrown item is picked up by a non-player entity.
    ThrownItemPickedUpByEntity {
        /// `item` optionally narrows the item predicate matched when a thrown item is picked up by a non-player entity.
        item: Option<ItemPredicate>,
        /// `entity` optionally narrows the entity predicate matched when a thrown item is picked up by a non-player entity.
        entity: Option<EntityPredicate>,
    },
    /// A thrown item is picked up by the player.
    ThrownItemPickedUpByPlayer {
        /// `item` optionally narrows the item predicate matched when a thrown item is picked up by the player.
        item: Option<ItemPredicate>,
        /// `entity` optionally narrows the entity predicate matched when a thrown item is picked up by the player.
        entity: Option<EntityPredicate>,
    },
    /// An item in the player's inventory loses durability.
    ItemDurabilityChanged {
        /// `item` optionally narrows the item predicate matched when an item in the player's inventory loses durability.
        item: Option<ItemPredicate>,
        /// `delta` optionally provides the delta when an item in the player's inventory loses durability.
        delta: Option<IntRange>,
        /// `durability` optionally provides the durability when an item in the player's inventory loses durability.
        durability: Option<IntRange>,
    },
    /// Player brews a potion.
    BrewedPotion {
        /// `potion` optionally provides the potion when a player brews a potion.
        potion: Option<String>,
    },
    /// Player destroys a bee nest or beehive.
    BeeNestDestroyed {
        /// `block` optionally narrows the block matched when a player destroys a bee nest or beehive.
        block: Option<String>,
        /// `item` optionally narrows the item predicate matched when a player destroys a bee nest or beehive.
        item: Option<ItemPredicate>,
        /// `num_bees_inside` optionally provides the num bees inside when a player destroys a bee nest or beehive.
        num_bees_inside: Option<IntRange>,
    },

    /// Player enchants an item.
    EnchantedItem {
        /// `item` optionally narrows the item predicate matched when a player enchants an item.
        item: Option<ItemPredicate>,
        /// `levels` optionally narrows the level range matched when a player enchants an item.
        levels: Option<IntRange>,
    },

    // ── Entities / interactions ───────────────────────────────────────────────
    #[doc = "Matches Minecraft's bred animals advancement trigger."]
    BredAnimals {
        /// `parent` optionally narrows the parent predicate matched for Minecraft's bred animals advancement trigger.
        parent: Option<EntityPredicate>,
        /// `partner` optionally narrows the partner predicate matched for Minecraft's bred animals advancement trigger.
        partner: Option<EntityPredicate>,
        /// `child` optionally narrows the child predicate matched for Minecraft's bred animals advancement trigger.
        child: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's tamed animal advancement trigger."]
    TamedAnimal {
        /// `entity` optionally narrows the entity predicate matched for Minecraft's tamed animal advancement trigger.
        entity: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's summoned entity advancement trigger."]
    SummonedEntity {
        /// `entity` optionally narrows the entity predicate matched for Minecraft's summoned entity advancement trigger.
        entity: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's player interacted with entity advancement trigger."]
    PlayerInteractedWithEntity {
        /// `item` optionally narrows the item predicate matched for Minecraft's player interacted with entity advancement trigger.
        item: Option<ItemPredicate>,
        /// `entity` optionally narrows the entity predicate matched for Minecraft's player interacted with entity advancement trigger.
        entity: Option<EntityPredicate>,
    },
    /// Player uses a fishing rod and it hooks something.
    FishingRodHooked {
        /// `rod` optionally narrows the rod predicate matched when a player uses a fishing rod and it hooks something.
        rod: Option<ItemPredicate>,
        /// `entity` optionally narrows the entity predicate matched when a player uses a fishing rod and it hooks something.
        entity: Option<EntityPredicate>,
        /// `item` optionally narrows the item predicate matched when a player uses a fishing rod and it hooks something.
        item: Option<ItemPredicate>,
    },
    #[doc = "Matches Minecraft's tamed animal interacted advancement trigger."]
    TamedAnimalInteracted {
        /// `entity` optionally narrows the entity predicate matched for Minecraft's tamed animal interacted advancement trigger.
        entity: Option<EntityPredicate>,
        /// `item` optionally narrows the item predicate matched for Minecraft's tamed animal interacted advancement trigger.
        item: Option<ItemPredicate>,
    },
    #[doc = "Matches Minecraft's villager trade advancement trigger."]
    VillagerTrade {
        /// `item` optionally narrows the item predicate matched for Minecraft's villager trade advancement trigger.
        item: Option<ItemPredicate>,
        /// `villager` optionally narrows the villager predicate matched for Minecraft's villager trade advancement trigger.
        villager: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's cured zombie villager advancement trigger."]
    CuredZombieVillager {
        /// `villager` optionally narrows the villager predicate matched for Minecraft's cured zombie villager advancement trigger.
        villager: Option<EntityPredicate>,
        /// `zombie` optionally narrows the zombie predicate matched for Minecraft's cured zombie villager advancement trigger.
        zombie: Option<EntityPredicate>,
    },

    // ── Location / world ──────────────────────────────────────────────────────
    #[doc = "Matches Minecraft's placed block advancement trigger."]
    PlacedBlock {
        /// `block` optionally narrows the block matched for Minecraft's placed block advancement trigger.
        block: Option<String>,
        /// `item` optionally narrows the item predicate matched for Minecraft's placed block advancement trigger.
        item: Option<ItemPredicate>,
        /// `location` optionally narrows the location predicate matched for Minecraft's placed block advancement trigger.
        location: Option<LocationPredicate>,
        /// `state` optionally provides the state for Minecraft's placed block advancement trigger.
        state: Option<HashMap<String, String>>,
    },
    #[doc = "Matches Minecraft's enter block advancement trigger."]
    EnterBlock {
        /// `block` optionally narrows the block matched for Minecraft's enter block advancement trigger.
        block: Option<String>,
        /// `state` optionally provides the state for Minecraft's enter block advancement trigger.
        state: Option<HashMap<String, String>>,
    },
    #[doc = "Matches Minecraft's location advancement trigger."]
    Location {
        /// `location` optionally narrows the location predicate matched for Minecraft's location advancement trigger.
        location: Option<LocationPredicate>,
    },
    #[doc = "Matches Minecraft's nether travel advancement trigger."]
    NetherTravel {
        /// `entered` optionally narrows the entered predicate matched for Minecraft's nether travel advancement trigger.
        entered: Option<LocationPredicate>,
        /// `exited` optionally narrows the exited predicate matched for Minecraft's nether travel advancement trigger.
        exited: Option<LocationPredicate>,
        /// `distance` optionally narrows the distance predicate matched for Minecraft's nether travel advancement trigger.
        distance: Option<DistancePredicate>,
    },
    #[doc = "Matches Minecraft's changed dimension advancement trigger."]
    ChangedDimension {
        /// `from` optionally narrows the source value matched for Minecraft's changed dimension advancement trigger.
        from: Option<String>,
        /// `to` optionally narrows the destination value matched for Minecraft's changed dimension advancement trigger.
        to: Option<String>,
    },
    #[doc = "Matches Minecraft's slept in bed advancement trigger."]
    SleptInBed {
        /// `location` optionally narrows the location predicate matched for Minecraft's slept in bed advancement trigger.
        location: Option<LocationPredicate>,
    },
    #[doc = "Matches Minecraft's fall from height advancement trigger."]
    FallFromHeight {
        /// `distance` optionally narrows the distance predicate matched for Minecraft's fall from height advancement trigger.
        distance: Option<DistancePredicate>,
        /// `start_position` optionally narrows the start position predicate matched for Minecraft's fall from height advancement trigger.
        start_position: Option<LocationPredicate>,
    },
    #[doc = "Matches Minecraft's slide down block advancement trigger."]
    SlideDownBlock {
        /// `block` optionally narrows the block matched for Minecraft's slide down block advancement trigger.
        block: Option<String>,
    },
    #[doc = "Matches Minecraft's target hit advancement trigger."]
    TargetHit {
        /// `signal_strength` optionally provides the signal strength for Minecraft's target hit advancement trigger.
        signal_strength: Option<IntRange>,
        /// `projectile` optionally narrows the projectile predicate matched for Minecraft's target hit advancement trigger.
        projectile: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's hero of the village advancement trigger."]
    HeroOfTheVillage {
        /// `location` optionally narrows the location predicate matched for Minecraft's hero of the village advancement trigger.
        location: Option<LocationPredicate>,
    },
    #[doc = "Matches Minecraft's player generates container loot advancement trigger."]
    PlayerGeneratesContainerLoot {
        /// `loot_table` optionally provides the loot table for Minecraft's player generates container loot advancement trigger.
        loot_table: Option<String>,
    },

    // ── Player state ──────────────────────────────────────────────────────────
    #[doc = "Matches Minecraft's leveled up advancement trigger."]
    LeveledUp {
        /// `level` optionally narrows the level range matched for Minecraft's leveled up advancement trigger.
        level: Option<IntRange>,
    },
    #[doc = "Matches Minecraft's effects changed advancement trigger."]
    EffectsChanged {
        /// `effects` optionally narrows the effects predicate matched for Minecraft's effects changed advancement trigger.
        effects: Option<HashMap<String, EffectPredicate>>,
        /// `source` optionally narrows the source predicate matched for Minecraft's effects changed advancement trigger.
        source: Option<EntityPredicate>,
    },
    #[doc = "Matches Minecraft's started riding advancement trigger."]
    StartedRiding,
    #[doc = "Matches Minecraft's construct beacon advancement trigger."]
    ConstructBeacon {
        /// `level` optionally narrows the level range matched for Minecraft's construct beacon advancement trigger.
        level: Option<IntRange>,
    },
    #[doc = "Matches Minecraft's used ender eye advancement trigger."]
    UsedEnderEye {
        /// `distance` optionally narrows the distance matched for Minecraft's used ender eye advancement trigger.
        distance: Option<FloatRange>,
    },

    // ── 1.19+ triggers ───────────────────────────────────────────────────────
    /// Player causes an allay to drop an item on a block (1.19+).
    AllayDropItemOnBlock {
        /// `item` optionally restricts the dropped item that satisfies this trigger.
        ///
        /// ```rust
        /// use sand_components::{AdvancementTrigger, ItemId, ItemPredicate};
        ///
        /// let trigger = AdvancementTrigger::AllayDropItemOnBlock {
        ///     item: Some(ItemPredicate::id(ItemId::minecraft("cake").unwrap())),
        ///     location: None,
        /// };
        /// ```
        item: Option<ItemPredicate>,
        /// `location` optionally narrows the location predicate matched when a player causes an allay to drop an item on a block.
        location: Option<LocationPredicate>,
    },
    /// Player avoids triggering a sculk sensor vibration (1.19+).
    AvoidVibration,
    /// Player kills a mob near a sculk catalyst (1.19+).
    KillMobNearSculkCatalyst {
        /// `entity` optionally narrows the entity predicate matched when a player kills a mob near a sculk catalyst.
        entity: Option<EntityPredicate>,
        /// `killing_blow` optionally narrows the killing blow predicate matched when a player kills a mob near a sculk catalyst.
        killing_blow: Option<DamagePredicate>,
    },
    /// Player right-clicks on a block while holding an item (1.19.4+).
    ItemUsedOnBlock {
        /// `item` optionally narrows the item predicate matched when a player right-clicks on a block while holding an item.
        item: Option<ItemPredicate>,
        /// `location` optionally narrows the location predicate matched when a player right-clicks on a block while holding an item.
        location: Option<LocationPredicate>,
    },

    // ── 1.16+ triggers ───────────────────────────────────────────────────────
    /// Player rides an entity in lava (1.16+).
    RideEntityInLava {
        /// `start_position` optionally narrows the start position predicate matched when a player rides an entity in lava.
        start_position: Option<LocationPredicate>,
        /// `distance` optionally narrows the distance predicate matched when a player rides an entity in lava.
        distance: Option<DistancePredicate>,
    },

    // ── Custom (escape hatch) ─────────────────────────────────────────────────
    /// Any trigger not covered by the typed variants.
    ///
    /// Use this to target triggers that were added to or removed from Minecraft
    /// after a given version, or for modded triggers.
    ///
    /// ```rust
    /// use sand_components::AdvancementTrigger;
    /// let t = AdvancementTrigger::Custom {
    ///     trigger: "minecraft:tick".into(),
    ///     conditions: None,
    /// };
    /// ```
    Custom {
        /// `trigger` provides the trigger when any trigger not covered by the typed variants. Use this to target triggers that were added to or removed from Minecraft after a given version, or for modded triggers.
        trigger: String,
        /// Raw JSON conditions block.  Use [`RawJson`] to signal intentional
        /// opt-out of the typed predicate API.
        conditions: Option<RawJson>,
    },
}

// ── Inventory slots predicate (used only by InventoryChanged) ─────────────────

/// Slot-count conditions for [`AdvancementTrigger::InventoryChanged`].
///
/// Controls how many inventory slots must be occupied, full, or empty.
/// This is a *count* predicate, not a slot-position selector.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::InventorySlotsPredicate",
    module = "sand::component",
    summary = "Slot-count conditions for [`AdvancementTrigger::InventoryChanged`].",
    context = "Slot-count conditions for [`AdvancementTrigger::InventoryChanged`]. Controls how many inventory slots must be occupied, full, or empty. This is a *count* predicate, not a slot-position selector.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::InventorySlotsPredicate;",
    fields(empty = "Limits how many inventory slots contain no items.", full = "Limits how many inventory slots contain a full stack.", occupied = "Limits how many inventory slots contain any item stack."),
)]
#[derive(Debug, Clone, Default, Serialize)]
pub struct InventorySlotsPredicate {
    /// Limits how many inventory slots contain any item stack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupied: Option<IntRange>,
    /// Limits how many inventory slots contain a full stack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<IntRange>,
    /// Limits how many inventory slots contain no items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty: Option<IntRange>,
}

impl InventorySlotsPredicate {
    fn validate_at(&self, path: &str) -> Result<(), String> {
        for (name, range) in [
            ("occupied", &self.occupied),
            ("full", &self.full),
            ("empty", &self.empty),
        ] {
            if let Some(range) = range {
                range.validate_at(&format!("{path}.{name}"))?;
            }
        }
        Ok(())
    }

    /// Creates a slot-count predicate with no occupied, full, or empty limit.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::InventorySlotsPredicate::new",
        module = "sand::component",
        kind = "method",
        summary = "Creates a slot-count predicate with no occupied, full, or empty limit.",
        context = "Creates a slot-count predicate with no occupied, full, or empty limit. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "An `InventorySlotsPredicate` representing a slot-count predicate with no occupied, full, or empty limit.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let inventory_slots_predicate = sand::component::InventorySlotsPredicate::new();\n}",
    )]
    pub fn new() -> Self {
        Self::default()
    }
    /// Requires at least `n` inventory slots to contain an item stack.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::InventorySlotsPredicate::occupied_min",
        module = "sand::component",
        kind = "method",
        summary = "Requires at least `n` inventory slots to contain an item stack.",
        context = "Requires at least `n` inventory slots to contain an item stack. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(n = "Requires at least `n` inventory slots to contain an item stack."),
        returns = "The `InventorySlotsPredicate` value with the documented change applied to require at least `n` inventory slots to contain an item stack.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_slots_predicate_value: sand::component::InventorySlotsPredicate, n: i64)  {\n    let updated_inventory_slots_predicate = inventory_slots_predicate_value.occupied_min(n);\n}",
    )]
    pub fn occupied_min(mut self, n: i64) -> Self {
        self.occupied = Some(IntRange::at_least(n));
        self
    }
    /// Allows at most `n` inventory slots to contain an item stack.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::InventorySlotsPredicate::occupied_max",
        module = "sand::component",
        kind = "method",
        summary = "Allows at most `n` inventory slots to contain an item stack.",
        context = "Allows at most `n` inventory slots to contain an item stack. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(n = "Allows at most `n` inventory slots to contain an item stack."),
        returns = "The `InventorySlotsPredicate` value with the documented change applied to allow at most `n` inventory slots to contain an item stack.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_slots_predicate_value: sand::component::InventorySlotsPredicate, n: i64)  {\n    let updated_inventory_slots_predicate = inventory_slots_predicate_value.occupied_max(n);\n}",
    )]
    pub fn occupied_max(mut self, n: i64) -> Self {
        self.occupied = Some(IntRange::at_most(n));
        self
    }
    /// Requires at least `n` inventory slots to be empty.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::InventorySlotsPredicate::empty_min",
        module = "sand::component",
        kind = "method",
        summary = "Requires at least `n` inventory slots to be empty.",
        context = "Requires at least `n` inventory slots to be empty. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(n = "Requires at least `n` inventory slots to be empty."),
        returns = "The `InventorySlotsPredicate` value with the documented change applied to require at least `n` inventory slots to be empty.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_slots_predicate_value: sand::component::InventorySlotsPredicate, n: i64)  {\n    let updated_inventory_slots_predicate = inventory_slots_predicate_value.empty_min(n);\n}",
    )]
    pub fn empty_min(mut self, n: i64) -> Self {
        self.empty = Some(IntRange::at_least(n));
        self
    }
    /// Requires at least `n` inventory slots to contain a full stack.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::InventorySlotsPredicate::full_min",
        module = "sand::component",
        kind = "method",
        summary = "Requires at least `n` inventory slots to contain a full stack.",
        context = "Requires at least `n` inventory slots to contain a full stack. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(n = "Requires at least `n` inventory slots to contain a full stack."),
        returns = "The `InventorySlotsPredicate` value with the documented change applied to require at least `n` inventory slots to contain a full stack.",
        example = "use sand::prelude::*;\n\nfn demonstrate(inventory_slots_predicate_value: sand::component::InventorySlotsPredicate, n: i64)  {\n    let updated_inventory_slots_predicate = inventory_slots_predicate_value.full_min(n);\n}",
    )]
    pub fn full_min(mut self, n: i64) -> Self {
        self.full = Some(IntRange::at_least(n));
        self
    }
}

// ── AdvancementTrigger::trigger_id helper ─────────────────────────────────────

impl AdvancementTrigger {
    /// Create a recipe-unlocked trigger from a validated recipe reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::recipe_unlocked",
        aliases = ["sand::prelude::AdvancementTrigger::recipe_unlocked"],
        module = "sand::component",
        kind = "method",
        summary = "Create a recipe-unlocked trigger from a validated recipe reference.",
        context = "Create a recipe-unlocked trigger from a validated recipe reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(recipe = "`recipe` provides the typed Minecraft resource identifier used to create a recipe-unlocked trigger from a validated recipe reference."),
        returns = "An `AdvancementTrigger` representing a recipe-unlocked trigger from a validated recipe reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate(recipe: sand::ResourceLocation)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::recipe_unlocked(recipe);\n}",
    )]
    pub fn recipe_unlocked(recipe: ResourceLocation) -> Self {
        Self::RecipeUnlocked {
            recipe: recipe.to_string(),
        }
    }

    /// Create a brewed-potion trigger using the shared potion registry ID.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::brewed_potion",
        aliases = ["sand::prelude::AdvancementTrigger::brewed_potion"],
        module = "sand::component",
        kind = "method",
        summary = "Create a brewed-potion trigger using the shared potion registry ID.",
        context = "Create a brewed-potion trigger using the shared potion registry ID. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(potion = "`potion` is used when creating a brewed-potion trigger using the shared potion registry ID."),
        returns = "An `AdvancementTrigger` representing a brewed-potion trigger using the shared potion registry ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(potion: impl Into < sand::registry::PotionRegistryId >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::brewed_potion(potion);\n}",
    )]
    pub fn brewed_potion(potion: impl Into<PotionRegistryId>) -> Self {
        Self::BrewedPotion {
            potion: Some(potion.into().to_string()),
        }
    }

    /// Create an unfiltered brewed-potion trigger.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::brewed_any_potion",
        aliases = ["sand::prelude::AdvancementTrigger::brewed_any_potion"],
        module = "sand::component",
        kind = "method",
        summary = "Create an unfiltered brewed-potion trigger.",
        context = "Create an unfiltered brewed-potion trigger. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "An `AdvancementTrigger` representing an unfiltered brewed-potion trigger.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let advancement_trigger = sand::component::AdvancementTrigger::brewed_any_potion();\n}",
    )]
    pub fn brewed_any_potion() -> Self {
        Self::BrewedPotion { potion: None }
    }

    /// Create a bee-nest-destroyed trigger with typed block identity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::bee_nest_destroyed",
        aliases = ["sand::prelude::AdvancementTrigger::bee_nest_destroyed"],
        module = "sand::component",
        kind = "method",
        summary = "Create a bee-nest-destroyed trigger with typed block identity.",
        context = "Create a bee-nest-destroyed trigger with typed block identity. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(block = "`block` provides the block value or block predicate used to create a bee-nest-destroyed trigger with typed block identity.", item = "`item` provides the item value or item predicate used to create a bee-nest-destroyed trigger with typed block identity.", num_bees_inside = "`num_bees_inside` provides the accepted numeric range used to create a bee-nest-destroyed trigger with typed block identity."),
        returns = "An `AdvancementTrigger` representing a bee-nest-destroyed trigger with typed block identity.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block: Option < sand::registry::BlockId >, item: Option < sand::predicate::ItemPredicate >, num_bees_inside: Option < sand::predicate::IntRange >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::bee_nest_destroyed(block, item, num_bees_inside);\n}",
    )]
    pub fn bee_nest_destroyed(
        block: Option<BlockId>,
        item: Option<ItemPredicate>,
        num_bees_inside: Option<IntRange>,
    ) -> Self {
        Self::BeeNestDestroyed {
            block: block.map(|id| id.to_string()),
            item,
            num_bees_inside,
        }
    }

    /// Create a placed-block trigger with typed block identity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::placed_block",
        aliases = ["sand::prelude::AdvancementTrigger::placed_block"],
        module = "sand::component",
        kind = "method",
        summary = "Create a placed-block trigger with typed block identity.",
        context = "Create a placed-block trigger with typed block identity. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(block = "`block` provides the block value or block predicate used to create a placed-block trigger with typed block identity.", item = "`item` provides the item value or item predicate used to create a placed-block trigger with typed block identity.", location = "`location` provides the typed resource identifier or location used to create a placed-block trigger with typed block identity.", state = "`state` is used when creating a placed-block trigger with typed block identity."),
        returns = "An `AdvancementTrigger` representing a placed-block trigger with typed block identity.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block: Option < sand::registry::BlockId >, item: Option < sand::predicate::ItemPredicate >, location: Option < sand::predicate::LocationPredicate >, state: Option < std::collections::HashMap < String , String > >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::placed_block(block, item, location, state);\n}",
    )]
    pub fn placed_block(
        block: Option<BlockId>,
        item: Option<ItemPredicate>,
        location: Option<LocationPredicate>,
        state: Option<HashMap<String, String>>,
    ) -> Self {
        Self::PlacedBlock {
            block: block.map(|id| id.to_string()),
            item,
            location,
            state,
        }
    }

    /// Create an enter-block trigger with typed block identity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::enter_block",
        aliases = ["sand::prelude::AdvancementTrigger::enter_block"],
        module = "sand::component",
        kind = "method",
        summary = "Create an enter-block trigger with typed block identity.",
        context = "Create an enter-block trigger with typed block identity. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(block = "`block` provides the block value or block predicate used to create an enter-block trigger with typed block identity.", state = "`state` is used when creating an enter-block trigger with typed block identity."),
        returns = "An `AdvancementTrigger` representing an enter-block trigger with typed block identity.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block: Option < sand::registry::BlockId >, state: Option < std::collections::HashMap < String , String > >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::enter_block(block, state);\n}",
    )]
    pub fn enter_block(block: Option<BlockId>, state: Option<HashMap<String, String>>) -> Self {
        Self::EnterBlock {
            block: block.map(|id| id.to_string()),
            state,
        }
    }

    /// Create a dimension-change trigger with typed dimension identities.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::changed_dimension",
        aliases = ["sand::prelude::AdvancementTrigger::changed_dimension"],
        module = "sand::component",
        kind = "method",
        summary = "Create a dimension-change trigger with typed dimension identities.",
        context = "Create a dimension-change trigger with typed dimension identities. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(from = "`from` is used when creating a dimension-change trigger with typed dimension identities.", to = "`to` is used when creating a dimension-change trigger with typed dimension identities."),
        returns = "An `AdvancementTrigger` representing a dimension-change trigger with typed dimension identities.",
        example = "use sand::prelude::*;\n\nfn demonstrate(from: Option < sand::registry::DimensionId >, to: Option < sand::registry::DimensionId >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::changed_dimension(from, to);\n}",
    )]
    pub fn changed_dimension(from: Option<DimensionId>, to: Option<DimensionId>) -> Self {
        Self::ChangedDimension {
            from: from.map(|id| id.to_string()),
            to: to.map(|id| id.to_string()),
        }
    }

    /// Create a slide-down-block trigger with typed block identity.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::slide_down_block",
        aliases = ["sand::prelude::AdvancementTrigger::slide_down_block"],
        module = "sand::component",
        kind = "method",
        summary = "Create a slide-down-block trigger with typed block identity.",
        context = "Create a slide-down-block trigger with typed block identity. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(block = "`block` provides the block value or block predicate used to create a slide-down-block trigger with typed block identity."),
        returns = "An `AdvancementTrigger` representing a slide-down-block trigger with typed block identity.",
        example = "use sand::prelude::*;\n\nfn demonstrate(block: Option < sand::registry::BlockId >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::slide_down_block(block);\n}",
    )]
    pub fn slide_down_block(block: Option<BlockId>) -> Self {
        Self::SlideDownBlock {
            block: block.map(|id| id.to_string()),
        }
    }

    /// Create a container-loot trigger from a validated loot-table reference.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::player_generates_container_loot",
        aliases = ["sand::prelude::AdvancementTrigger::player_generates_container_loot"],
        module = "sand::component",
        kind = "method",
        summary = "Create a container-loot trigger from a validated loot-table reference.",
        context = "Create a container-loot trigger from a validated loot-table reference. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(loot_table = "`loot_table` provides the typed Minecraft resource identifier used to create a container-loot trigger from a validated loot-table reference."),
        returns = "An `AdvancementTrigger` representing a container-loot trigger from a validated loot-table reference.",
        example = "use sand::prelude::*;\n\nfn demonstrate(loot_table: Option < sand::ResourceLocation >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::player_generates_container_loot(loot_table);\n}",
    )]
    pub fn player_generates_container_loot(loot_table: Option<ResourceLocation>) -> Self {
        Self::PlayerGeneratesContainerLoot {
            loot_table: loot_table.map(|id| id.to_string()),
        }
    }

    /// Create an effects-changed trigger with typed status-effect map keys.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::effects_changed",
        aliases = ["sand::prelude::AdvancementTrigger::effects_changed"],
        module = "sand::component",
        kind = "method",
        summary = "Create an effects-changed trigger with typed status-effect map keys.",
        context = "Create an effects-changed trigger with typed status-effect map keys. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(effects = "`effects` is used when creating an effects-changed trigger with typed status-effect map keys.", source = "`source` provides the typed predicate that must match used to create an effects-changed trigger with typed status-effect map keys."),
        returns = "An `AdvancementTrigger` representing an effects-changed trigger with typed status-effect map keys.",
        example = "use sand::prelude::*;\n\nfn demonstrate<I: 'static, E: 'static>(effects: I, source: Option < sand::predicate::EntityPredicate >) where I : IntoIterator < Item = (E , sand::predicate::EffectPredicate) > , E : Into < sand::registry::StatusEffectId > {\n    let advancement_trigger = sand::component::AdvancementTrigger::effects_changed::<I, E>(effects, source);\n}",
    )]
    pub fn effects_changed<I, E>(effects: I, source: Option<EntityPredicate>) -> Self
    where
        I: IntoIterator<Item = (E, EffectPredicate)>,
        E: Into<StatusEffectId>,
    {
        let effects = effects
            .into_iter()
            .map(|(id, predicate)| (id.into().to_string(), predicate))
            .collect::<HashMap<_, _>>();
        Self::EffectsChanged {
            effects: (!effects.is_empty()).then_some(effects),
            source,
        }
    }

    /// Create an unfiltered effects-changed trigger.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::effects_changed_any",
        aliases = ["sand::prelude::AdvancementTrigger::effects_changed_any"],
        module = "sand::component",
        kind = "method",
        summary = "Create an unfiltered effects-changed trigger.",
        context = "Create an unfiltered effects-changed trigger. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(source = "`source` provides the typed predicate that must match used to create an unfiltered effects-changed trigger."),
        returns = "An `AdvancementTrigger` representing an unfiltered effects-changed trigger.",
        example = "use sand::prelude::*;\n\nfn demonstrate(source: Option < sand::predicate::EntityPredicate >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::effects_changed_any(source);\n}",
    )]
    pub fn effects_changed_any(source: Option<EntityPredicate>) -> Self {
        Self::EffectsChanged {
            effects: None,
            source,
        }
    }

    /// Create a custom/modded trigger with a validated trigger ID.
    ///
    /// The conditions remain an explicit opaque [`RawJson`] escape hatch.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::custom_trigger",
        aliases = ["sand::prelude::AdvancementTrigger::custom_trigger"],
        module = "sand::component",
        kind = "method",
        summary = "Create a custom/modded trigger with a validated trigger ID.",
        context = "Create a custom/modded trigger with a validated trigger ID. The conditions remain an explicit opaque [`RawJson`] escape hatch.",
        minecraft = "The conditions remain an explicit opaque [`RawJson`] escape hatch.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(trigger = "`trigger` provides the typed Minecraft resource identifier used to create a custom/modded trigger with a validated trigger ID.", conditions = "`conditions` is used when creating a custom/modded trigger with a validated trigger ID."),
        returns = "An `AdvancementTrigger` representing a custom/modded trigger with a validated trigger ID.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trigger: sand::ResourceLocation, conditions: Option < sand::component::RawJson >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::custom_trigger(trigger, conditions);\n}",
    )]
    pub fn custom_trigger(trigger: ResourceLocation, conditions: Option<RawJson>) -> Self {
        Self::Custom {
            trigger: trigger.to_string(),
            conditions,
        }
    }

    /// Validate stable predicate/range invariants for typed trigger conditions.
    /// Raw/custom trigger conditions remain an explicit escape hatch.
    pub(crate) fn validate_at(&self, path: &str) -> Result<(), String> {
        let conditions = format!("{path}.conditions");
        match self {
            Self::RecipeUnlocked { recipe } => {
                validate_resource_id(recipe, &format!("{conditions}.recipe"))?;
            }
            Self::RecipeCrafted {
                recipe_id,
                ingredients,
            } => {
                validate_resource_id(recipe_id, &format!("{conditions}.recipe_id"))?;
                for (index, item) in ingredients.iter().enumerate() {
                    item.validate_at(&format!("{conditions}.ingredients[{index}]"))?;
                }
            }
            Self::BrewedPotion {
                potion: Some(potion),
            } => {
                validate_resource_id(potion, &format!("{conditions}.potion"))?;
            }
            Self::BeeNestDestroyed {
                block: Some(block), ..
            } => {
                validate_resource_id(block, &format!("{conditions}.block"))?;
            }
            Self::PlacedBlock {
                block: Some(block), ..
            }
            | Self::EnterBlock {
                block: Some(block), ..
            }
            | Self::SlideDownBlock { block: Some(block) } => {
                validate_resource_id(block, &format!("{conditions}.block"))?;
            }
            Self::ChangedDimension { from, to } => {
                if let Some(from) = from {
                    validate_resource_id(from, &format!("{conditions}.from"))?;
                }
                if let Some(to) = to {
                    validate_resource_id(to, &format!("{conditions}.to"))?;
                }
            }
            Self::PlayerGeneratesContainerLoot {
                loot_table: Some(loot_table),
            } => {
                validate_resource_id(loot_table, &format!("{conditions}.loot_table"))?;
            }
            Self::Custom { trigger, .. } => {
                validate_resource_id(trigger, &format!("{path}.trigger"))?;
            }
            Self::PlayerKilledEntity {
                entity,
                killing_blow,
            }
            | Self::EntityKilledPlayer {
                entity,
                killing_blow,
            }
            | Self::KillMobNearSculkCatalyst {
                entity,
                killing_blow,
            } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(damage) = killing_blow {
                    damage.validate_at(&format!("{conditions}.killing_blow"))?;
                }
            }
            Self::PlayerHurtEntity { entity, damage }
            | Self::EntityHurtPlayer { entity, damage } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(damage) = damage {
                    damage.validate_at(&format!("{conditions}.damage"))?;
                }
            }
            Self::KilledByCrossbow {
                unique_entity_types,
                victims,
            } => {
                if let Some(range) = unique_entity_types {
                    range.validate_at(&format!("{conditions}.unique_entity_types"))?;
                }
                if let Some(victims) = victims {
                    for (index, victim) in victims.iter().enumerate() {
                        victim.validate_at(&format!("{conditions}.victims[{index}]"))?;
                    }
                }
            }
            Self::KilledByArrow {
                unique_entity_types,
                fired_from_weapon,
                victims,
            } => {
                if let Some(range) = unique_entity_types {
                    range.validate_at(&format!("{conditions}.unique_entity_types"))?;
                }
                if let Some(item) = fired_from_weapon {
                    item.validate_at(&format!("{conditions}.fired_from_weapon"))?;
                }
                if let Some(victims) = victims {
                    for (index, victim) in victims.iter().enumerate() {
                        victim.validate_at(&format!("{conditions}.victims[{index}]"))?;
                    }
                }
            }
            Self::ChanneledLightning {
                victims: Some(victims),
            } => {
                for (index, victim) in victims.iter().enumerate() {
                    victim.validate_at(&format!("{conditions}.victims[{index}]"))?;
                }
            }
            Self::LightningStrike {
                lightning,
                bystander,
            } => {
                if let Some(entity) = lightning {
                    entity.validate_at(&format!("{conditions}.lightning"))?;
                }
                if let Some(entity) = bystander {
                    entity.validate_at(&format!("{conditions}.bystander"))?;
                }
            }
            Self::InventoryChanged { slots, items } => {
                if let Some(slots) = slots {
                    slots.validate_at(&format!("{conditions}.slots"))?;
                }
                for (index, item) in items.iter().enumerate() {
                    item.validate_at(&format!("{conditions}.items[{index}]"))?;
                }
            }
            Self::LeveledUp { level } | Self::ConstructBeacon { level } => {
                if let Some(level) = level {
                    level.validate_at(&format!("{conditions}.level"))?;
                }
            }
            Self::UsedEnderEye {
                distance: Some(distance),
            } => distance.validate_at(&format!("{conditions}.distance"))?,
            Self::Location { location }
            | Self::SleptInBed { location }
            | Self::HeroOfTheVillage { location } => {
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::UsedItem { item }
            | Self::ConsumeItem { item }
            | Self::UsingItem { item }
            | Self::CraftedItem { item }
            | Self::FilledBucket { item }
            | Self::ShotCrossbow { item }
            | Self::UsedTotem { item } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
            }
            Self::EmptiedBucket { item, location }
            | Self::AllayDropItemOnBlock { item, location }
            | Self::ItemUsedOnBlock { item, location } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::ThrownItemPickedUp { item, entity }
            | Self::ThrownItemPickedUpByEntity { item, entity }
            | Self::ThrownItemPickedUpByPlayer { item, entity }
            | Self::PlayerInteractedWithEntity { item, entity }
            | Self::TamedAnimalInteracted { item, entity } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
            }
            Self::ItemDurabilityChanged {
                item,
                delta,
                durability,
            } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = delta {
                    range.validate_at(&format!("{conditions}.delta"))?;
                }
                if let Some(range) = durability {
                    range.validate_at(&format!("{conditions}.durability"))?;
                }
            }
            Self::BeeNestDestroyed {
                item,
                num_bees_inside,
                ..
            } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = num_bees_inside {
                    range.validate_at(&format!("{conditions}.num_bees_inside"))?;
                }
            }
            Self::EnchantedItem { item, levels } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(range) = levels {
                    range.validate_at(&format!("{conditions}.levels"))?;
                }
            }
            Self::BredAnimals {
                parent,
                partner,
                child,
            } => {
                for (name, entity) in [("parent", parent), ("partner", partner), ("child", child)] {
                    if let Some(entity) = entity {
                        entity.validate_at(&format!("{conditions}.{name}"))?;
                    }
                }
            }
            Self::TamedAnimal { entity } | Self::SummonedEntity { entity } => {
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
            }
            Self::FishingRodHooked { rod, entity, item } => {
                if let Some(rod) = rod {
                    rod.validate_at(&format!("{conditions}.rod"))?;
                }
                if let Some(entity) = entity {
                    entity.validate_at(&format!("{conditions}.entity"))?;
                }
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
            }
            Self::VillagerTrade { item, villager } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(villager) = villager {
                    villager.validate_at(&format!("{conditions}.villager"))?;
                }
            }
            Self::CuredZombieVillager { villager, zombie } => {
                if let Some(entity) = villager {
                    entity.validate_at(&format!("{conditions}.villager"))?;
                }
                if let Some(entity) = zombie {
                    entity.validate_at(&format!("{conditions}.zombie"))?;
                }
            }
            Self::PlacedBlock { item, location, .. } => {
                if let Some(item) = item {
                    item.validate_at(&format!("{conditions}.item"))?;
                }
                if let Some(location) = location {
                    location.validate_at(&format!("{conditions}.location"))?;
                }
            }
            Self::NetherTravel {
                entered,
                exited,
                distance,
            } => {
                if let Some(location) = entered {
                    location.validate_at(&format!("{conditions}.entered"))?;
                }
                if let Some(location) = exited {
                    location.validate_at(&format!("{conditions}.exited"))?;
                }
                if let Some(distance) = distance {
                    distance.validate_at(&format!("{conditions}.distance"))?;
                }
            }
            Self::FallFromHeight {
                distance,
                start_position,
            }
            | Self::RideEntityInLava {
                distance,
                start_position,
            } => {
                if let Some(distance) = distance {
                    distance.validate_at(&format!("{conditions}.distance"))?;
                }
                if let Some(location) = start_position {
                    location.validate_at(&format!("{conditions}.start_position"))?;
                }
            }
            Self::TargetHit {
                signal_strength,
                projectile,
            } => {
                if let Some(range) = signal_strength {
                    range.validate_at(&format!("{conditions}.signal_strength"))?;
                }
                if let Some(entity) = projectile {
                    entity.validate_at(&format!("{conditions}.projectile"))?;
                }
            }
            Self::EffectsChanged { effects, source } => {
                if let Some(effects) = effects {
                    for (effect, predicate) in effects {
                        validate_resource_id(effect, &format!("{conditions}.effects.{effect}"))?;
                        predicate.validate_at(&format!("{conditions}.effects.{effect}"))?;
                    }
                }
                if let Some(entity) = source {
                    entity.validate_at(&format!("{conditions}.source"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Return the vanilla trigger ID selected by this typed trigger.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::trigger_id",
        aliases = ["sand::prelude::AdvancementTrigger::trigger_id"],
        module = "sand::component",
        kind = "method",
        summary = "Return the vanilla trigger ID selected by this typed trigger.",
        context = "Return the vanilla trigger ID selected by this typed trigger. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Return the vanilla trigger ID selected by this typed trigger.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_trigger_value: &sand::component::AdvancementTrigger)  {\n    let trigger_id = advancement_trigger_value.trigger_id();\n}",
    )]
    pub fn trigger_id(&self) -> &str {
        match self {
            AdvancementTrigger::Tick => "minecraft:tick",
            AdvancementTrigger::Impossible => "minecraft:impossible",
            AdvancementTrigger::PlayerKilledEntity { .. } => "minecraft:player_killed_entity",
            AdvancementTrigger::EntityKilledPlayer { .. } => "minecraft:entity_killed_player",
            AdvancementTrigger::InventoryChanged { .. } => "minecraft:inventory_changed",
            AdvancementTrigger::RecipeUnlocked { .. } => "minecraft:recipe_unlocked",
            AdvancementTrigger::UsedItem { .. } => "minecraft:used_item",
            AdvancementTrigger::PlacedBlock { .. } => "minecraft:placed_block",
            AdvancementTrigger::BredAnimals { .. } => "minecraft:bred_animals",
            AdvancementTrigger::ConsumeItem { .. } => "minecraft:consume_item",
            AdvancementTrigger::EnterBlock { .. } => "minecraft:enter_block",
            AdvancementTrigger::EnchantedItem { .. } => "minecraft:enchanted_item",
            AdvancementTrigger::TamedAnimal { .. } => "minecraft:tame_animal",
            AdvancementTrigger::SummonedEntity { .. } => "minecraft:summoned_entity",
            AdvancementTrigger::Location { .. } => "minecraft:location",
            AdvancementTrigger::NetherTravel { .. } => "minecraft:nether_travel",
            AdvancementTrigger::UsingItem { .. } => "minecraft:using_item",
            AdvancementTrigger::PlayerInteractedWithEntity { .. } => {
                "minecraft:player_interacted_with_entity"
            }
            AdvancementTrigger::PlayerHurtEntity { .. } => "minecraft:player_hurt_entity",
            AdvancementTrigger::EntityHurtPlayer { .. } => "minecraft:entity_hurt_player",
            AdvancementTrigger::KilledByCrossbow { .. } => "minecraft:killed_by_crossbow",
            AdvancementTrigger::KilledByArrow { .. } => "minecraft:killed_by_arrow",
            AdvancementTrigger::ChanneledLightning { .. } => "minecraft:channeled_lightning",
            AdvancementTrigger::LightningStrike { .. } => "minecraft:lightning_strike",
            AdvancementTrigger::CraftedItem { .. } => "minecraft:crafted_item",
            AdvancementTrigger::RecipeCrafted { .. } => "minecraft:recipe_crafted",
            AdvancementTrigger::FilledBucket { .. } => "minecraft:filled_bucket",
            AdvancementTrigger::EmptiedBucket { .. } => "minecraft:emptied_bucket",
            AdvancementTrigger::FishingRodHooked { .. } => "minecraft:fishing_rod_hooked",
            AdvancementTrigger::ShotCrossbow { .. } => "minecraft:shot_crossbow",
            AdvancementTrigger::UsedTotem { .. } => "minecraft:used_totem",
            AdvancementTrigger::ThrownItemPickedUp { .. } => "minecraft:thrown_item_picked_up",
            AdvancementTrigger::ThrownItemPickedUpByEntity { .. } => {
                "minecraft:thrown_item_picked_up_by_entity"
            }
            AdvancementTrigger::ThrownItemPickedUpByPlayer { .. } => {
                "minecraft:thrown_item_picked_up_by_player"
            }
            AdvancementTrigger::ItemDurabilityChanged { .. } => "minecraft:item_durability_changed",
            AdvancementTrigger::BrewedPotion { .. } => "minecraft:brewed_potion",
            AdvancementTrigger::BeeNestDestroyed { .. } => "minecraft:bee_nest_destroyed",
            AdvancementTrigger::ChangedDimension { .. } => "minecraft:changed_dimension",
            AdvancementTrigger::SleptInBed { .. } => "minecraft:slept_in_bed",
            AdvancementTrigger::FallFromHeight { .. } => "minecraft:fall_from_height",
            AdvancementTrigger::LeveledUp { .. } => "minecraft:leveled_up",
            AdvancementTrigger::EffectsChanged { .. } => "minecraft:effects_changed",
            AdvancementTrigger::StartedRiding => "minecraft:started_riding",
            AdvancementTrigger::SlideDownBlock { .. } => "minecraft:slide_down_block",
            AdvancementTrigger::TargetHit { .. } => "minecraft:target_hit",
            AdvancementTrigger::ConstructBeacon { .. } => "minecraft:construct_beacon",
            AdvancementTrigger::CuredZombieVillager { .. } => "minecraft:cured_zombie_villager",
            AdvancementTrigger::UsedEnderEye { .. } => "minecraft:used_ender_eye",
            AdvancementTrigger::HeroOfTheVillage { .. } => "minecraft:hero_of_the_village",
            AdvancementTrigger::PlayerGeneratesContainerLoot { .. } => {
                "minecraft:player_generates_container_loot"
            }
            AdvancementTrigger::VillagerTrade { .. } => "minecraft:villager_trade",
            AdvancementTrigger::TamedAnimalInteracted { .. } => {
                "minecraft:player_interacted_with_entity"
            }
            AdvancementTrigger::AllayDropItemOnBlock { .. } => "minecraft:allay_drop_item_on_block",
            AdvancementTrigger::AvoidVibration => "minecraft:avoid_vibration",
            AdvancementTrigger::KillMobNearSculkCatalyst { .. } => {
                "minecraft:kill_mob_near_sculk_catalyst"
            }
            AdvancementTrigger::ItemUsedOnBlock { .. } => "minecraft:item_used_on_block",
            AdvancementTrigger::RideEntityInLava { .. } => "minecraft:ride_entity_in_lava",
            AdvancementTrigger::Custom { trigger, .. } => trigger.as_str(),
        }
    }

    /// Validate this trigger against Sand's supported vanilla target profiles.
    ///
    /// This intentionally fails before an advancement JSON file is emitted for
    /// IDs known to be absent from the vanilla registry.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::validate_for_target",
        aliases = ["sand::prelude::AdvancementTrigger::validate_for_target"],
        module = "sand::component",
        kind = "method",
        summary = "Validate this trigger against Sand's supported vanilla target profiles.",
        context = "Validate this trigger against Sand's supported vanilla target profiles. This intentionally fails before an advancement JSON file is emitted for IDs known to be absent from the vanilla registry.",
        minecraft = "This intentionally fails before an advancement JSON file is emitted for IDs known to be absent from the vanilla registry.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "On success, the value produced to validate this trigger against Sand's supported vanilla target profiles; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_trigger_value: &sand::component::AdvancementTrigger)  {\n    let validate_for_target = advancement_trigger_value.validate_for_target();\n}",
    )]
    pub fn validate_for_target(&self) -> Result<(), String> {
        self.validate_for_caps(None)
    }

    /// Validate this typed trigger's ID and version range for a resolved
    /// target. Raw [`AdvancementTrigger::Custom`] values bypass Sand-owned
    /// compatibility claims and remain user-owned.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::validate_for_caps",
        aliases = ["sand::prelude::AdvancementTrigger::validate_for_caps"],
        module = "sand::component",
        kind = "method",
        summary = "Validate this typed trigger's ID and version range for a resolved target. Raw [`AdvancementTrigger::Custom`] values bypass Sand-owned compatibility claims and remain user-owned.",
        context = "Validate this typed trigger's ID and version range for a resolved target. Raw [`AdvancementTrigger::Custom`] values bypass Sand-owned compatibility claims and remain user-owned. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(caps = "`caps` is the caps checked when validating this typed trigger's ID and version range for a resolved target. Raw [`AdvancementTrigger::Custom`] values bypass Sand-owned compatibility claims and remain user-owned."),
        returns = "On success, the value produced to validate this typed trigger's ID and version range for a resolved target. Raw [`AdvancementTrigger::Custom`] values bypass Sand-owned compatibility claims and remain user-owned; otherwise, the documented validation or export diagnostic.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_trigger_value: &sand::component::AdvancementTrigger, caps: Option < & sand::version::VersionCaps >)  {\n    let validate_for_caps = advancement_trigger_value.validate_for_caps(caps);\n}",
    )]
    pub fn validate_for_caps(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> Result<(), String> {
        if matches!(self, Self::Custom { .. }) {
            return Ok(());
        }
        let metadata =
            crate::advancement::trigger_coverage::trigger_metadata_for(self.trigger_id(), caps);
        if !metadata.supported {
            return Err(format!(
                "advancement trigger `{}` is not available for Sand's supported Minecraft targets. {}",
                self.trigger_id(),
                metadata.diagnostic.unwrap_or("choose a supported trigger")
            ));
        }

        let Some(caps) = caps else {
            return Ok(());
        };
        if caps.is_fallback() {
            return Err(format!(
                "advancement trigger `{}` requires an exact known Minecraft profile; `{}` resolved to conservative fallback capabilities. Select an exact known version or `latest`, or use AdvancementTrigger::Custom with user-verified raw compatibility",
                self.trigger_id(),
                caps.requested_version()
            ));
        }
        let coverage = crate::advancement::trigger_coverage::find_coverage(self.trigger_id())
            .ok_or_else(|| {
                format!(
                    "typed advancement trigger `{}` has no trigger-coverage metadata; use AdvancementTrigger::Custom only for intentional raw/modded compatibility",
                    self.trigger_id()
                )
            })?;
        if matches!(
            coverage.api_status,
            crate::advancement::trigger_coverage::TriggerApiStatus::Missing
                | crate::advancement::trigger_coverage::TriggerApiStatus::RawOnly
                | crate::advancement::trigger_coverage::TriggerApiStatus::IntentionallyUnsupported
        ) {
            return Err(format!(
                "advancement trigger `{}` is not available through Sand's typed API for target {}; use AdvancementTrigger::Custom only with profile-verified raw conditions",
                self.trigger_id(),
                caps.requested_version()
            ));
        }
        if let Some((major, minor, patch)) = parse_trigger_version(coverage.since)
            && !caps.is_at_least(major, minor, patch)
        {
            return Err(format!(
                "advancement trigger `{}` is available since Minecraft {}, but the selected target is {}",
                self.trigger_id(),
                coverage.since,
                caps.requested_version()
            ));
        }
        if let Some(removed_in) = coverage.removed_in
            && let Some((major, minor, patch)) = parse_trigger_version(removed_in)
            && caps.is_at_least(major, minor, patch)
        {
            return Err(format!(
                "advancement trigger `{}` was removed in Minecraft {}, but the selected target is {}",
                self.trigger_id(),
                removed_in,
                caps.requested_version()
            ));
        }
        Ok(())
    }

    // ── Convenience constructors ──────────────────────────────────────────────

    /// Build an `InventoryChanged` trigger matching any of the given item IDs.
    ///
    /// Items are generated registry values implementing `Display`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::inventory_changed",
        aliases = ["sand::prelude::AdvancementTrigger::inventory_changed"],
        module = "sand::component",
        kind = "method",
        summary = "Build an `InventoryChanged` trigger matching any of the given item IDs.",
        context = "Build an `InventoryChanged` trigger matching any of the given item IDs. Items are generated registry values implementing `Display`.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(items = "`items` provides the items used to build an `InventoryChanged` trigger matching any of the given item IDs."),
        returns = "An `AdvancementTrigger` that builds an `InventoryChanged` trigger matching any of the given item IDs.",
        example = "use sand::prelude::*;\n\nfn demonstrate(items: Vec < impl Into < sand::registry::ItemId > >)  {\n    let advancement_trigger = sand::component::AdvancementTrigger::inventory_changed(items);\n}",
    )]
    pub fn inventory_changed(items: Vec<impl Into<ItemId>>) -> Self {
        AdvancementTrigger::InventoryChanged {
            slots: None,
            items: items.into_iter().map(ItemPredicate::id).collect(),
        }
    }
}

fn parse_trigger_version(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = value.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

// ── Serialize ─────────────────────────────────────────────────────────────────

impl Serialize for AdvancementTrigger {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // `PlacedBlock` and `ItemUsedOnBlock` render through the same modern
        // `location_check`/`match_tool` lowering used by `render_for(None)` —
        // see #232/#233. This compatibility `Serialize` impl (used directly by
        // tests, `Criterion`, and any caller that doesn't route through
        // `render_for`) must never fall back to the old unfiltered flat
        // `conditions.block`/`conditions.item` shape, or it would silently
        // reintroduce the bug those issues fixed. The pre-item-component
        // legacy shape remains reachable only through the explicit
        // `render_for(Some(&caps))` profile-gated path.
        match self {
            AdvancementTrigger::PlacedBlock {
                block,
                item,
                location,
                state,
            } => {
                let value = render_placed_block_modern(block, item, location, state, None)
                    .map_err(serde::ser::Error::custom)?;
                return value.serialize(serializer);
            }
            AdvancementTrigger::ItemUsedOnBlock { item, location } => {
                let value = render_item_used_on_block_modern(item, location, None)
                    .map_err(serde::ser::Error::custom)?;
                return value.serialize(serializer);
            }
            _ => {}
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("trigger", self.trigger_id())?;

        match self {
            AdvancementTrigger::Tick
            | AdvancementTrigger::Impossible
            | AdvancementTrigger::StartedRiding => {}

            AdvancementTrigger::PlayerKilledEntity {
                entity,
                killing_blow,
            }
            | AdvancementTrigger::EntityKilledPlayer {
                entity,
                killing_blow,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), json_value::<_, S::Error>(k)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlayerHurtEntity { entity, damage }
            | AdvancementTrigger::EntityHurtPlayer { entity, damage } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(d) = damage {
                    cond.insert("damage".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::KilledByCrossbow {
                unique_entity_types,
                victims,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(u) = unique_entity_types {
                    cond.insert("unique_entity_types".into(), json_value::<_, S::Error>(u)?);
                }
                if let Some(v) = victims {
                    cond.insert("victims".into(), json_value::<_, S::Error>(v)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::KilledByArrow {
                unique_entity_types,
                fired_from_weapon,
                victims,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(value) = unique_entity_types {
                    cond.insert(
                        "unique_entity_types".into(),
                        json_value::<_, S::Error>(value)?,
                    );
                }
                if let Some(item) = fired_from_weapon {
                    cond.insert("fired_from_weapon".into(), json_value::<_, S::Error>(item)?);
                }
                if let Some(victims) = victims {
                    cond.insert("victims".into(), json_value::<_, S::Error>(victims)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ChanneledLightning { victims } => {
                if let Some(v) = victims {
                    map.serialize_entry("conditions", &serde_json::json!({ "victims": v }))?;
                }
            }

            AdvancementTrigger::LightningStrike {
                lightning,
                bystander,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(l) = lightning {
                    cond.insert("lightning".into(), json_value::<_, S::Error>(l)?);
                }
                if let Some(b) = bystander {
                    cond.insert("bystander".into(), json_value::<_, S::Error>(b)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::InventoryChanged { slots, items } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = slots {
                    cond.insert("slots".into(), json_value::<_, S::Error>(s)?);
                }
                if !items.is_empty() {
                    cond.insert("items".into(), json_value::<_, S::Error>(items)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::RecipeUnlocked { recipe } => {
                map.serialize_entry("conditions", &serde_json::json!({ "recipe": recipe }))?;
            }

            AdvancementTrigger::RecipeCrafted {
                recipe_id,
                ingredients,
            } => {
                let mut cond = serde_json::Map::new();
                cond.insert("recipe_id".into(), Value::String(recipe_id.clone()));
                if !ingredients.is_empty() {
                    cond.insert(
                        "ingredients".into(),
                        json_value::<_, S::Error>(ingredients)?,
                    );
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::UsedItem { item }
            | AdvancementTrigger::ConsumeItem { item }
            | AdvancementTrigger::UsingItem { item }
            | AdvancementTrigger::CraftedItem { item }
            | AdvancementTrigger::FilledBucket { item }
            | AdvancementTrigger::ShotCrossbow { item }
            | AdvancementTrigger::UsedTotem { item } => {
                if let Some(i) = item {
                    map.serialize_entry("conditions", &serde_json::json!({ "item": i }))?;
                }
            }

            AdvancementTrigger::EmptiedBucket { item, location } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = location {
                    cond.insert("location".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::FishingRodHooked { rod, entity, item } => {
                let mut cond = serde_json::Map::new();
                if let Some(r) = rod {
                    cond.insert("rod".into(), json_value::<_, S::Error>(r)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ThrownItemPickedUp { item, entity }
            | AdvancementTrigger::ThrownItemPickedUpByEntity { item, entity }
            | AdvancementTrigger::ThrownItemPickedUpByPlayer { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ItemDurabilityChanged {
                item,
                delta,
                durability,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(d) = delta {
                    cond.insert("delta".into(), json_value::<_, S::Error>(d)?);
                }
                if let Some(d) = durability {
                    cond.insert("durability".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::BrewedPotion { potion } => {
                if let Some(p) = potion {
                    map.serialize_entry("conditions", &serde_json::json!({ "potion": p }))?;
                }
            }

            AdvancementTrigger::BeeNestDestroyed {
                block,
                item,
                num_bees_inside,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(n) = num_bees_inside {
                    cond.insert("num_bees_inside".into(), json_value::<_, S::Error>(n)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::EnchantedItem { item, levels } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = levels {
                    cond.insert("levels".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::BredAnimals {
                parent,
                partner,
                child,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(p) = parent {
                    cond.insert("parent".into(), json_value::<_, S::Error>(p)?);
                }
                if let Some(p) = partner {
                    cond.insert("partner".into(), json_value::<_, S::Error>(p)?);
                }
                if let Some(c) = child {
                    cond.insert("child".into(), json_value::<_, S::Error>(c)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::TamedAnimal { entity }
            | AdvancementTrigger::SummonedEntity { entity } => {
                if let Some(e) = entity {
                    map.serialize_entry("conditions", &serde_json::json!({ "entity": e }))?;
                }
            }

            AdvancementTrigger::PlayerInteractedWithEntity { item, entity }
            | AdvancementTrigger::TamedAnimalInteracted { item, entity } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::VillagerTrade { item, villager } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(v) = villager {
                    cond.insert("villager".into(), json_value::<_, S::Error>(v)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::CuredZombieVillager { villager, zombie } => {
                let mut cond = serde_json::Map::new();
                if let Some(v) = villager {
                    cond.insert("villager".into(), json_value::<_, S::Error>(v)?);
                }
                if let Some(z) = zombie {
                    cond.insert("zombie".into(), json_value::<_, S::Error>(z)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::PlacedBlock { .. } => {
                unreachable!("PlacedBlock is handled by the early return above")
            }

            AdvancementTrigger::EnterBlock { block, state } => {
                let mut cond = serde_json::Map::new();
                if let Some(b) = block {
                    cond.insert("block".into(), Value::String(b.clone()));
                }
                if let Some(s) = state {
                    cond.insert("state".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::Location { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::NetherTravel {
                entered,
                exited,
                distance,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entered {
                    cond.insert("entered".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(e) = exited {
                    cond.insert("exited".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ChangedDimension { from, to } => {
                let mut cond = serde_json::Map::new();
                if let Some(f) = from {
                    cond.insert("from".into(), Value::String(f.clone()));
                }
                if let Some(t) = to {
                    cond.insert("to".into(), Value::String(t.clone()));
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::SleptInBed { location }
            | AdvancementTrigger::HeroOfTheVillage { location } => {
                if let Some(l) = location {
                    map.serialize_entry("conditions", &serde_json::json!({ "location": l }))?;
                }
            }

            AdvancementTrigger::FallFromHeight {
                distance,
                start_position,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::LeveledUp { level } => {
                if let Some(l) = level {
                    map.serialize_entry("conditions", &serde_json::json!({ "level": l }))?;
                }
            }

            AdvancementTrigger::EffectsChanged { effects, source } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = effects {
                    cond.insert("effects".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(s) = source {
                    cond.insert("source".into(), json_value::<_, S::Error>(s)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::SlideDownBlock { block } => {
                if let Some(b) = block {
                    map.serialize_entry("conditions", &serde_json::json!({ "block": b }))?;
                }
            }

            AdvancementTrigger::TargetHit {
                signal_strength,
                projectile,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = signal_strength {
                    cond.insert("signal_strength".into(), json_value::<_, S::Error>(s)?);
                }
                if let Some(p) = projectile {
                    cond.insert("projectile".into(), json_value::<_, S::Error>(p)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ConstructBeacon { level } => {
                if let Some(l) = level {
                    map.serialize_entry("conditions", &serde_json::json!({ "level": l }))?;
                }
            }

            AdvancementTrigger::UsedEnderEye { distance } => {
                if let Some(d) = distance {
                    map.serialize_entry("conditions", &serde_json::json!({ "distance": d }))?;
                }
            }

            AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table } => {
                if let Some(lt) = loot_table {
                    map.serialize_entry("conditions", &serde_json::json!({ "loot_table": lt }))?;
                }
            }

            AdvancementTrigger::AllayDropItemOnBlock { item, location } => {
                let mut cond = serde_json::Map::new();
                if let Some(i) = item {
                    cond.insert("item".into(), json_value::<_, S::Error>(i)?);
                }
                if let Some(l) = location {
                    cond.insert("location".into(), json_value::<_, S::Error>(l)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::AvoidVibration => {}

            AdvancementTrigger::KillMobNearSculkCatalyst {
                entity,
                killing_blow,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(e) = entity {
                    cond.insert("entity".into(), json_value::<_, S::Error>(e)?);
                }
                if let Some(k) = killing_blow {
                    cond.insert("killing_blow".into(), json_value::<_, S::Error>(k)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::ItemUsedOnBlock { .. } => {
                unreachable!("ItemUsedOnBlock is handled by the early return above")
            }

            AdvancementTrigger::RideEntityInLava {
                start_position,
                distance,
            } => {
                let mut cond = serde_json::Map::new();
                if let Some(s) = start_position {
                    cond.insert("start_position".into(), json_value::<_, S::Error>(s)?);
                }
                if let Some(d) = distance {
                    cond.insert("distance".into(), json_value::<_, S::Error>(d)?);
                }
                if !cond.is_empty() {
                    map.serialize_entry("conditions", &Value::Object(cond))?;
                }
            }

            AdvancementTrigger::Custom { conditions, .. } => {
                if let Some(c) = conditions {
                    map.serialize_entry("conditions", c)?;
                }
            }
        }

        map.end()
    }
}

// ── Schema families (#232) ─────────────────────────────────────────────────────

/// Which vanilla advancement condition/predicate schema a target Minecraft
/// profile expects.
///
/// This is the single place that maps a [`sand_version::VersionCaps`] profile
/// to a rendering strategy — trigger rendering matches on this enum instead
/// of comparing capability flags or version strings inline. See
/// [`AdvancementTrigger::render_for`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementSchemaFamily {
    /// Pre item-component era (pre-1.20.5). [`AdvancementTrigger::PlacedBlock`]
    /// and [`AdvancementTrigger::ItemUsedOnBlock`] render through the
    /// historical flat `conditions.block`/`conditions.item` shape here.
    ///
    /// **Known limitation:** unlike the modern family below, this flat shape
    /// has *not* been verified against a real pre-1.20.5 vanilla server.
    /// Historical research for #231/#232 found no authoritative evidence
    /// that `placed_block`/`item_used_on_block` ever accepted flat
    /// `conditions.block`/`conditions.item` fields at any version — the
    /// `location`/`location_check`/`match_tool` composition these triggers
    /// use predates the 1.20.5 item-component overhaul by years. It is
    /// possible this family has the same "filter silently ignored" defect
    /// #231 fixed for the modern family. This PR does not change legacy
    /// output without verified proof (existing supported-profile output is
    /// preserved per project policy), and does not implement the
    /// pre-component item-predicate schema (`tag`/`nbt`-based matching) that
    /// would be needed to correctly filter `item` on this family — that is
    /// full item-model work owned by #229. Filed as a follow-up: verify
    /// `placed_block`/`item_used_on_block` semantics on a real pre-1.20.5
    /// server and, if broken, apply the same `location`/`match_tool` fix
    /// used for the modern family here.
    Legacy,
    /// 1.20.5+ item-component era (includes every currently-supported 26.x
    /// profile). `PlacedBlock`/`ItemUsedOnBlock` render through
    /// `conditions.location` wrapping `minecraft:location_check` (block) and
    /// `minecraft:match_tool` (item), with item predicates using the
    /// `components` (exact)/`predicates` (partial) keys. Verified against a
    /// real Minecraft 1.21.4 and 26.2 servers. A protocol-client fixture also
    /// verifies placement/item-use match and non-match semantics on 1.21.4.
    LocationConditionItemComponents,
    /// Minecraft 26.2+ retains the modern location-condition/item-component
    /// trigger shape and additionally namespaces entity sub-predicate keys.
    NamespacedEntityPredicates,
}

impl AdvancementSchemaFamily {
    /// Map a target profile's capabilities to its advancement schema family.
    ///
    /// `caps` is `None` on the unprofiled compatibility export path, treated
    /// the same as a fully item-component-capable modern profile (matching
    /// the `VersionCaps::all_enabled()` convention used elsewhere in Sand).
    pub fn for_caps(caps: Option<&sand_version::VersionCaps>) -> Self {
        let Some(caps) = caps else {
            return Self::NamespacedEntityPredicates;
        };
        if !caps.supports(sand_version::ComponentFeature::ItemComponents) {
            Self::Legacy
        } else if caps.is_at_least(26, 2, 0) {
            Self::NamespacedEntityPredicates
        } else {
            Self::LocationConditionItemComponents
        }
    }

    fn uses_modern_location_conditions(self) -> bool {
        !matches!(self, Self::Legacy)
    }
}

#[derive(Debug, Clone)]
struct AdvancementPredicateConsumer {
    trigger_id: String,
    field: &'static str,
}

impl AdvancementPredicateConsumer {
    fn new(trigger_id: impl Into<String>, field: &'static str) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            field,
        }
    }

    fn label(&self) -> String {
        format!("`{}.conditions.{}`", self.trigger_id, self.field)
    }
}

/// Which advancement trigger/field a rendered [`ItemPredicate`] is being
/// converted for.
///
/// This is a narrowly-scoped, advancement-rendering-internal analog of the
/// consumer-aware matcher conversion the full shared item model (#229) will
/// eventually own. It exists so diagnostics can name the exact trigger/field
/// an unsupported item-predicate conversion was requested for, and so #229
/// has a documented seam to integrate with rather than needing to redesign
/// advancement export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementItemConsumer {
    /// The tool/item filter for [`AdvancementTrigger::PlacedBlock`], rendered
    /// as a `minecraft:match_tool` condition in the modern schema family.
    PlacedBlockTool,
    /// The tool/item filter for [`AdvancementTrigger::ItemUsedOnBlock`],
    /// rendered as a `minecraft:match_tool` condition in the modern schema family.
    ItemUsedOnBlockTool,
    /// Item dropped by an allay onto a block; uses the same modern
    /// `location_check`/`match_tool` consumer shape.
    AllayDropItemOnBlockTool,
}

impl AdvancementItemConsumer {
    /// The vanilla trigger ID this consumer belongs to, for diagnostics.
    pub const fn trigger_id(self) -> &'static str {
        match self {
            Self::PlacedBlockTool => "minecraft:placed_block",
            Self::ItemUsedOnBlockTool => "minecraft:item_used_on_block",
            Self::AllayDropItemOnBlockTool => "minecraft:allay_drop_item_on_block",
        }
    }
}

// ── Version-aware rendering (#231, #232, #233) ─────────────────────────────────

impl AdvancementTrigger {
    /// Render this trigger's `{"trigger": ..., "conditions": ...}` JSON for a
    /// specific Minecraft version's predicate schema.
    ///
    /// Every typed trigger is validated and lowered through the selected
    /// [`AdvancementSchemaFamily`]. Variants that consume item, entity,
    /// location, or damage predicates use consumer-aware conversion so nested
    /// schemas follow the target profile too. In particular,
    /// [`AdvancementTrigger::PlacedBlock`] and
    /// [`AdvancementTrigger::ItemUsedOnBlock`] render differently across the
    /// legacy and modern families. Minecraft's modern
    /// (1.20.5+ item-component era) schema expresses that filter as a
    /// `conditions.location` array of `minecraft:location_check` /
    /// `minecraft:match_tool` loot conditions, not the direct `block`/`item`
    /// fields this crate used to emit. Emitting the direct fields makes the
    /// generated advancement fire unconditionally in-game — see #231/#233.
    ///
    /// This never silently drops a filter: if a caller supplies both the
    /// trigger-level `block`/`state` shorthand *and* a `location` predicate
    /// that already sets `block`, rendering fails with an actionable
    /// [`SandError`](crate::error::SandError) instead of picking one silently.
    /// Likewise, requesting an item filter on [`AdvancementSchemaFamily::Legacy`]
    /// fails with an actionable error instead of emitting an item-component-era
    /// JSON shape (`components`/`predicates`) that legacy profiles don't
    /// recognize — see [`AdvancementSchemaFamily::Legacy`]'s docs.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementTrigger::render_for",
        aliases = ["sand::prelude::AdvancementTrigger::render_for"],
        module = "sand::component",
        kind = "method",
        summary = "Render this trigger's `{\"trigger\": ..., \"conditions\": ...}` JSON for a specific Minecraft version's predicate schema.",
        context = "Render this trigger's `{\"trigger\": ..., \"conditions\": ...}` JSON for a specific Minecraft version's predicate schema. Every typed trigger is validated and lowered through the selected [`AdvancementSchemaFamily`]. Variants that consume item, entity, location, or damage predicates use consumer-aware conversion so nested schemas follow the target profile too. In particular, [`AdvancementTrigger::PlacedBlock`] and [`AdvancementTrigger::ItemUsedOnBlock`] render differently across the legacy and modern families. Minecraft's modern (1.20.5+ item-component era) schema expresses that filter as a `conditions.location` array of `minecraft:location_check` / `minecraft:match_tool` loot conditions, not the direct `block`/`item` fields this crate used to emit. Emitting the direct fields makes the generated advancement fire unconditionally in-game — see #231/#233. This never silently drops a filter: if a caller supplies both the trigger-level `block`/`state` shorthand *and* a `location` predicate that already sets `block`, rendering fails with an actionable [`SandError`](sand::component::SandError) instead of picking one silently. Likewise, requesting an item filter on [`AdvancementSchemaFamily::L...",
        minecraft = "Every typed trigger is validated and lowered through the selected [`AdvancementSchemaFamily`]. Variants that consume item, entity, location, or damage predicates use consumer-aware conversion so nested schemas follow the target profile too. In particular, [`AdvancementTrigger::PlacedBlock`] and [`AdvancementTrigger::ItemUsedOnBlock`] render differently across the legacy and modern families. Minecraft's modern (1.20.5+ item-component era) schema expresses that filter as a `conditions.location` array of `minecraft:location_check` / `minecraft:match_tool` loot conditions, not the direct `block`/`item` fields this crate used to emit. Emitting the direct fields makes the generated advancement fire unconditionally in-game — see #231/#233.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(caps = "`caps` provides the caps rendered when this trigger's `{\"trigger\": ..., \"conditions\": ...}` JSON for a specific Minecraft version's predicate schema."),
        returns = "The `sand :: component :: Result < Value >` value produced to render this trigger's `{\"trigger\": ..., \"conditions\": ...}` JSON for a specific Minecraft version's predicate schema.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_trigger_value: &sand::component::AdvancementTrigger, caps: Option < & sand::version::VersionCaps >)  {\n    let render_for = advancement_trigger_value.render_for(caps);\n}",
    )]
    pub fn render_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<Value> {
        self.validate_for_caps(caps).map_err(|message| {
            predicate_render_error(
                AdvancementPredicateConsumer::new(self.trigger_id(), "trigger"),
                message,
            )
        })?;
        if matches!(self, Self::Custom { .. }) {
            return serde_json::to_value(self).map_err(crate::error::SandError::Serialization);
        }
        let family = AdvancementSchemaFamily::for_caps(caps);
        match self {
            AdvancementTrigger::PlacedBlock {
                block,
                item,
                location,
                state,
            } if family.uses_modern_location_conditions() => {
                render_placed_block_modern(block, item, location, state, caps)
            }
            AdvancementTrigger::ItemUsedOnBlock { item, location }
                if family.uses_modern_location_conditions() =>
            {
                render_item_used_on_block_modern(item, location, caps)
            }
            AdvancementTrigger::PlacedBlock { item: Some(_), .. }
                if matches!(family, AdvancementSchemaFamily::Legacy) =>
            {
                Err(unsupported_legacy_item_filter(
                    AdvancementItemConsumer::PlacedBlockTool,
                ))
            }
            AdvancementTrigger::ItemUsedOnBlock { item: Some(_), .. }
                if matches!(family, AdvancementSchemaFamily::Legacy) =>
            {
                Err(unsupported_legacy_item_filter(
                    AdvancementItemConsumer::ItemUsedOnBlockTool,
                ))
            }
            AdvancementTrigger::PlacedBlock {
                block,
                location,
                state,
                ..
            } => {
                if location.as_ref().is_some_and(|location| !location.is_raw()) {
                    return Err(predicate_render_error(
                        AdvancementPredicateConsumer::new("minecraft:placed_block", "location"),
                        "typed location filters have no verified lowering for this legacy advancement schema; use direct block/state fields, target Minecraft 1.21.4+, or use LocationPredicate::raw(...) with profile-verified JSON",
                    ));
                }
                Ok(render_placed_block_legacy(block, location, state))
            }
            AdvancementTrigger::ItemUsedOnBlock { location, .. } => {
                if location.as_ref().is_some_and(|location| !location.is_raw()) {
                    return Err(predicate_render_error(
                        AdvancementPredicateConsumer::new(
                            "minecraft:item_used_on_block",
                            "location",
                        ),
                        "typed location filters have no verified lowering for this legacy advancement schema; target Minecraft 1.21.4+ or use LocationPredicate::raw(...) with profile-verified JSON",
                    ));
                }
                Ok(render_item_used_on_block_legacy(location))
            }
            _ => render_profiled_trigger(self, caps),
        }
    }
}

/// Build the actionable diagnostic for requesting an item filter on
/// [`AdvancementSchemaFamily::Legacy`], where this crate has no verified,
/// correct representation.
///
/// Delegates to the shared [`crate::item::matcher::ItemMatcher`] conversion
/// diagnostic (#229) rather than maintaining a second, advancement-only copy
/// of the same capability check and message — this is the seam
/// [`AdvancementItemConsumer`]'s doc comment describes #229 integrating with.
fn unsupported_legacy_item_filter(consumer: AdvancementItemConsumer) -> crate::error::SandError {
    crate::item::matcher::unsupported_legacy_item_filter(consumer.into())
}

fn predicate_render_error(
    consumer: AdvancementPredicateConsumer,
    message: impl Into<String>,
) -> crate::error::SandError {
    crate::error::SandError::ComponentValidation {
        location: ResourceLocation::new("sand", "advancement_predicate")
            .expect("static resource location is valid"),
        kind: "advancement trigger predicate".into(),
        field: consumer.label(),
        message: message.into(),
    }
}

fn render_advancement_item(
    item: &ItemPredicate,
    caps: Option<&sand_version::VersionCaps>,
    consumer: AdvancementPredicateConsumer,
) -> crate::error::Result<Value> {
    item.render_for_advancement(caps)
        .map_err(|message| predicate_render_error(consumer, message))
}

fn render_advancement_location(
    location: &LocationPredicate,
    caps: Option<&sand_version::VersionCaps>,
    consumer: AdvancementPredicateConsumer,
) -> crate::error::Result<Value> {
    location
        .render_for_advancement(caps)
        .map_err(|message| predicate_render_error(consumer, message))
}

fn render_advancement_entity_predicate(
    entity: &EntityPredicate,
    caps: Option<&sand_version::VersionCaps>,
    consumer: AdvancementPredicateConsumer,
) -> crate::error::Result<Value> {
    entity
        .render_for_advancement(caps)
        .map_err(|message| predicate_render_error(consumer, message))
}

fn render_advancement_entity_condition(
    entity: &EntityPredicate,
    caps: Option<&sand_version::VersionCaps>,
    consumer: AdvancementPredicateConsumer,
) -> crate::error::Result<Value> {
    Ok(serde_json::json!([{
        "condition": "minecraft:entity_properties",
        "entity": "this",
        "predicate": render_advancement_entity_predicate(entity, caps, consumer)?,
    }]))
}

fn render_advancement_damage(
    damage: &DamagePredicate,
    caps: Option<&sand_version::VersionCaps>,
    consumer: AdvancementPredicateConsumer,
) -> crate::error::Result<Value> {
    damage
        .render_for_advancement(caps)
        .map_err(|message| predicate_render_error(consumer, message))
}

fn conditions_object(value: &mut Value) -> &mut serde_json::Map<String, Value> {
    value
        .as_object_mut()
        .expect("typed advancement trigger serializes as an object")
        .entry("conditions")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .expect("typed advancement conditions serialize as an object")
}

fn render_profiled_trigger(
    trigger: &AdvancementTrigger,
    caps: Option<&sand_version::VersionCaps>,
) -> crate::error::Result<Value> {
    let mut value =
        serde_json::to_value(trigger).map_err(crate::error::SandError::Serialization)?;
    let id = trigger.trigger_id();

    macro_rules! replace_item {
        ($conditions:expr, $field:literal, $item:expr) => {
            if let Some(item) = $item {
                $conditions.insert(
                    $field.into(),
                    render_advancement_item(
                        item,
                        caps,
                        AdvancementPredicateConsumer::new(id, $field),
                    )?,
                );
            }
        };
    }
    macro_rules! replace_entity {
        ($conditions:expr, $field:literal, $entity:expr) => {
            if let Some(entity) = $entity {
                $conditions.insert(
                    $field.into(),
                    render_advancement_entity_condition(
                        entity,
                        caps,
                        AdvancementPredicateConsumer::new(id, $field),
                    )?,
                );
            }
        };
    }
    macro_rules! replace_location {
        ($conditions:expr, $field:literal, $location:expr) => {
            if let Some(location) = $location {
                $conditions.insert(
                    $field.into(),
                    render_advancement_location(
                        location,
                        caps,
                        AdvancementPredicateConsumer::new(id, $field),
                    )?,
                );
            }
        };
    }

    let conditions = conditions_object(&mut value);
    match trigger {
        AdvancementTrigger::PlayerKilledEntity {
            entity,
            killing_blow,
        }
        | AdvancementTrigger::EntityKilledPlayer {
            entity,
            killing_blow,
        }
        | AdvancementTrigger::KillMobNearSculkCatalyst {
            entity,
            killing_blow,
        } => {
            replace_entity!(conditions, "entity", entity);
            if let Some(damage) = killing_blow {
                conditions.insert(
                    "killing_blow".into(),
                    render_advancement_damage(
                        damage,
                        caps,
                        AdvancementPredicateConsumer::new(id, "killing_blow"),
                    )?,
                );
            }
        }
        AdvancementTrigger::PlayerHurtEntity { entity, damage }
        | AdvancementTrigger::EntityHurtPlayer { entity, damage } => {
            replace_entity!(conditions, "entity", entity);
            if let Some(damage) = damage {
                conditions.insert(
                    "damage".into(),
                    render_advancement_damage(
                        damage,
                        caps,
                        AdvancementPredicateConsumer::new(id, "damage"),
                    )?,
                );
            }
        }
        AdvancementTrigger::KilledByCrossbow { victims, .. }
        | AdvancementTrigger::ChanneledLightning { victims } => {
            if let Some(victims) = victims {
                conditions.insert(
                    "victims".into(),
                    Value::Array(
                        victims
                            .iter()
                            .map(|entity| {
                                render_advancement_entity_condition(
                                    entity,
                                    caps,
                                    AdvancementPredicateConsumer::new(id, "victims"),
                                )
                            })
                            .collect::<crate::error::Result<Vec<_>>>()?,
                    ),
                );
            }
        }
        AdvancementTrigger::LightningStrike {
            lightning,
            bystander,
        } => {
            replace_entity!(conditions, "lightning", lightning);
            replace_entity!(conditions, "bystander", bystander);
        }
        AdvancementTrigger::InventoryChanged { items, .. } => {
            if !items.is_empty() {
                conditions.insert(
                    "items".into(),
                    Value::Array(
                        items
                            .iter()
                            .map(|item| {
                                render_advancement_item(
                                    item,
                                    caps,
                                    AdvancementPredicateConsumer::new(id, "items"),
                                )
                            })
                            .collect::<crate::error::Result<Vec<_>>>()?,
                    ),
                );
            }
        }
        AdvancementTrigger::RecipeCrafted { ingredients, .. } => {
            if !ingredients.is_empty() {
                conditions.insert(
                    "ingredients".into(),
                    Value::Array(
                        ingredients
                            .iter()
                            .map(|item| {
                                render_advancement_item(
                                    item,
                                    caps,
                                    AdvancementPredicateConsumer::new(id, "ingredients"),
                                )
                            })
                            .collect::<crate::error::Result<Vec<_>>>()?,
                    ),
                );
            }
        }
        AdvancementTrigger::KilledByArrow {
            fired_from_weapon,
            victims,
            ..
        } => {
            replace_item!(conditions, "fired_from_weapon", fired_from_weapon);
            if let Some(victims) = victims {
                conditions.insert(
                    "victims".into(),
                    Value::Array(
                        victims
                            .iter()
                            .map(|entity| {
                                render_advancement_entity_condition(
                                    entity,
                                    caps,
                                    AdvancementPredicateConsumer::new(id, "victims"),
                                )
                            })
                            .collect::<crate::error::Result<Vec<_>>>()?,
                    ),
                );
            }
        }
        AdvancementTrigger::UsedItem { item }
        | AdvancementTrigger::ConsumeItem { item }
        | AdvancementTrigger::UsingItem { item }
        | AdvancementTrigger::CraftedItem { item }
        | AdvancementTrigger::FilledBucket { item }
        | AdvancementTrigger::ShotCrossbow { item }
        | AdvancementTrigger::UsedTotem { item } => replace_item!(conditions, "item", item),
        AdvancementTrigger::EmptiedBucket { item, location } => {
            replace_item!(conditions, "item", item);
            replace_location!(conditions, "location", location);
        }
        AdvancementTrigger::ThrownItemPickedUp { item, entity }
        | AdvancementTrigger::ThrownItemPickedUpByEntity { item, entity }
        | AdvancementTrigger::ThrownItemPickedUpByPlayer { item, entity }
        | AdvancementTrigger::PlayerInteractedWithEntity { item, entity }
        | AdvancementTrigger::TamedAnimalInteracted { item, entity } => {
            replace_item!(conditions, "item", item);
            replace_entity!(conditions, "entity", entity);
        }
        AdvancementTrigger::ItemDurabilityChanged { item, .. }
        | AdvancementTrigger::BeeNestDestroyed { item, .. }
        | AdvancementTrigger::EnchantedItem { item, .. } => {
            replace_item!(conditions, "item", item)
        }
        AdvancementTrigger::BredAnimals {
            parent,
            partner,
            child,
        } => {
            replace_entity!(conditions, "parent", parent);
            replace_entity!(conditions, "partner", partner);
            replace_entity!(conditions, "child", child);
        }
        AdvancementTrigger::TamedAnimal { entity }
        | AdvancementTrigger::SummonedEntity { entity } => {
            replace_entity!(conditions, "entity", entity)
        }
        AdvancementTrigger::FishingRodHooked { rod, entity, item } => {
            replace_item!(conditions, "rod", rod);
            replace_entity!(conditions, "entity", entity);
            replace_item!(conditions, "item", item);
        }
        AdvancementTrigger::VillagerTrade { item, villager } => {
            replace_item!(conditions, "item", item);
            replace_entity!(conditions, "villager", villager);
        }
        AdvancementTrigger::CuredZombieVillager { villager, zombie } => {
            replace_entity!(conditions, "villager", villager);
            replace_entity!(conditions, "zombie", zombie);
        }
        AdvancementTrigger::Location { location }
        | AdvancementTrigger::SleptInBed { location }
        | AdvancementTrigger::HeroOfTheVillage { location } => {
            if let Some(location) = location {
                let entity = EntityPredicate::new().location(location.clone());
                conditions.remove("location");
                conditions.insert(
                    "player".into(),
                    render_advancement_entity_condition(
                        &entity,
                        caps,
                        AdvancementPredicateConsumer::new(id, "player"),
                    )?,
                );
            }
        }
        AdvancementTrigger::NetherTravel {
            entered, exited, ..
        } => {
            replace_location!(conditions, "entered", entered);
            replace_location!(conditions, "exited", exited);
        }
        AdvancementTrigger::FallFromHeight { start_position, .. }
        | AdvancementTrigger::RideEntityInLava { start_position, .. } => {
            replace_location!(conditions, "start_position", start_position)
        }
        AdvancementTrigger::TargetHit {
            projectile: Some(projectile),
            ..
        } => {
            conditions.insert(
                "projectile".into(),
                render_advancement_entity_condition(
                    projectile,
                    caps,
                    AdvancementPredicateConsumer::new(id, "projectile"),
                )?,
            );
        }
        AdvancementTrigger::EffectsChanged {
            source: Some(source),
            ..
        } => {
            conditions.insert(
                "source".into(),
                render_advancement_entity_condition(
                    source,
                    caps,
                    AdvancementPredicateConsumer::new(id, "source"),
                )?,
            );
        }
        AdvancementTrigger::AllayDropItemOnBlock { item, location } => {
            return render_location_condition_trigger(
                "minecraft:allay_drop_item_on_block",
                item,
                location,
                caps,
            );
        }
        _ => {}
    }

    if conditions.is_empty() {
        value
            .as_object_mut()
            .expect("trigger object")
            .remove("conditions");
    }
    Ok(value)
}

/// Pre-item-component-era flat rendering for [`AdvancementTrigger::PlacedBlock`],
/// preserved only for targets where `render_for` determines the modern
/// `location_check`/`match_tool` schema is unsupported. Not used by the
/// compatibility `Serialize` impl, which always renders the modern (correct)
/// shape — see the `Serialize for AdvancementTrigger` impl's doc comment.
fn render_placed_block_legacy(
    block: &Option<String>,
    location: &Option<LocationPredicate>,
    state: &Option<HashMap<String, String>>,
) -> Value {
    let mut cond = serde_json::Map::new();
    if let Some(b) = block {
        cond.insert("block".to_string(), Value::String(b.clone()));
    }
    if let Some(l) = location {
        cond.insert(
            "location".to_string(),
            serde_json::to_value(l).unwrap_or(Value::Null),
        );
    }
    if let Some(s) = state {
        cond.insert(
            "state".to_string(),
            serde_json::to_value(s).unwrap_or(Value::Null),
        );
    }

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:placed_block".to_string()),
    );
    if !cond.is_empty() {
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Value::Object(map)
}

/// Pre-item-component-era flat rendering for [`AdvancementTrigger::ItemUsedOnBlock`].
/// See [`render_placed_block_legacy`] for when this is used.
fn render_item_used_on_block_legacy(location: &Option<LocationPredicate>) -> Value {
    let mut cond = serde_json::Map::new();
    if let Some(l) = location {
        cond.insert(
            "location".to_string(),
            serde_json::to_value(l).unwrap_or(Value::Null),
        );
    }

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:item_used_on_block".to_string()),
    );
    if !cond.is_empty() {
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Value::Object(map)
}

/// Build the `minecraft:location_check` / `minecraft:match_tool` condition
/// array shared by [`AdvancementTrigger::PlacedBlock`] and
/// [`AdvancementTrigger::ItemUsedOnBlock`]'s modern rendering.
fn render_location_and_item_conditions(
    consumer: AdvancementItemConsumer,
    location: &Option<LocationPredicate>,
    item: &Option<ItemPredicate>,
    block_shorthand: Option<&String>,
    state_shorthand: &Option<HashMap<String, String>>,
    caps: Option<&sand_version::VersionCaps>,
) -> crate::error::Result<Vec<Value>> {
    let mut loc = location.clone().unwrap_or_default();

    if block_shorthand.is_some() || state_shorthand.is_some() {
        if loc.has_block() {
            return Err(crate::error::SandError::ComponentValidation {
                location: ResourceLocation::new("sand", "advancement_trigger")
                    .expect("static resource location is always valid"),
                kind: consumer.trigger_id().to_string(),
                field: "conditions.block".to_string(),
                message: "both the direct `block`/`state` shorthand and an explicit \
                    `location` predicate that may already set `block` (a typed `block`, \
                    or a `LocationPredicate::raw(...)` escape hatch whose contents Sand \
                    cannot inspect) were set; specify the block filter in exactly one place"
                    .to_string(),
            });
        }
        let mut bp = crate::predicates::BlockPredicate::new();
        if let Some(block) = block_shorthand {
            bp = bp.blocks(vec![block.parse().map_err(|error| {
                crate::error::SandError::ComponentValidation {
                    location: ResourceLocation::new("sand", "advancement_trigger")
                        .expect("static resource location is always valid"),
                    kind: consumer.trigger_id().to_string(),
                    field: "conditions.block".to_string(),
                    message: format!("invalid block identifier `{block}`: {error}"),
                }
            })?]);
        }
        if let Some(state) = state_shorthand {
            bp = bp.state(
                state
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            );
        }
        loc = loc.block(bp);
    }

    let mut conditions = Vec::new();
    if !loc.is_empty() {
        let predicate = render_advancement_location(
            &loc,
            caps,
            AdvancementPredicateConsumer::new(consumer.trigger_id(), "location"),
        )?;
        conditions.push(serde_json::json!({
            "condition": "minecraft:location_check",
            "predicate": predicate,
        }));
    }
    if let Some(item) = item {
        let predicate = render_advancement_item(
            item,
            caps,
            AdvancementPredicateConsumer::new(consumer.trigger_id(), "item"),
        )?;
        conditions.push(serde_json::json!({
            "condition": "minecraft:match_tool",
            "predicate": predicate,
        }));
    }
    Ok(conditions)
}

fn render_placed_block_modern(
    block: &Option<String>,
    item: &Option<ItemPredicate>,
    location: &Option<LocationPredicate>,
    state: &Option<HashMap<String, String>>,
    caps: Option<&sand_version::VersionCaps>,
) -> crate::error::Result<Value> {
    let conditions = render_location_and_item_conditions(
        AdvancementItemConsumer::PlacedBlockTool,
        location,
        item,
        block.as_ref(),
        state,
        caps,
    )?;

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:placed_block".to_string()),
    );
    if !conditions.is_empty() {
        let mut cond = serde_json::Map::new();
        cond.insert("location".to_string(), Value::Array(conditions));
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Ok(Value::Object(map))
}

fn render_item_used_on_block_modern(
    item: &Option<ItemPredicate>,
    location: &Option<LocationPredicate>,
    caps: Option<&sand_version::VersionCaps>,
) -> crate::error::Result<Value> {
    let conditions = render_location_and_item_conditions(
        AdvancementItemConsumer::ItemUsedOnBlockTool,
        location,
        item,
        None,
        &None,
        caps,
    )?;

    let mut map = serde_json::Map::new();
    map.insert(
        "trigger".to_string(),
        Value::String("minecraft:item_used_on_block".to_string()),
    );
    if !conditions.is_empty() {
        let mut cond = serde_json::Map::new();
        cond.insert("location".to_string(), Value::Array(conditions));
        map.insert("conditions".to_string(), Value::Object(cond));
    }
    Ok(Value::Object(map))
}

fn render_location_condition_trigger(
    trigger_id: &'static str,
    item: &Option<ItemPredicate>,
    location: &Option<LocationPredicate>,
    caps: Option<&sand_version::VersionCaps>,
) -> crate::error::Result<Value> {
    let consumer = match trigger_id {
        "minecraft:allay_drop_item_on_block" => AdvancementItemConsumer::AllayDropItemOnBlockTool,
        _ => unreachable!("location-condition trigger consumer must be registered"),
    };
    let conditions =
        render_location_and_item_conditions(consumer, location, item, None, &None, caps)?;
    let mut value = serde_json::Map::new();
    value.insert("trigger".into(), Value::String(trigger_id.into()));
    if !conditions.is_empty() {
        value.insert(
            "conditions".into(),
            serde_json::json!({ "location": conditions }),
        );
    }
    Ok(Value::Object(value))
}

// ── Criterion ─────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Criterion",
    aliases = ["sand::prelude::Criterion"],
    module = "sand::component",
    summary = "A single criterion for an advancement that must be met for progress.",
    context = "A single criterion for an advancement that must be met for progress. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Criterion;",
    fields(trigger = "`trigger` provides the trigger when a single criterion for an advancement that must be met for progress."),
)]
/// A single criterion for an advancement that must be met for progress.
pub struct Criterion {
    /// `trigger` provides the trigger when a single criterion for an advancement that must be met for progress.
    pub trigger: AdvancementTrigger,
}

impl Criterion {
    /// Creates a new criterion with the specified trigger.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Criterion::new",
        aliases = ["sand::prelude::Criterion::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new criterion with the specified trigger.",
        context = "Creates a new criterion with the specified trigger. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(trigger = "`trigger` is used when creating a new criterion with the specified trigger."),
        returns = "A `Criterion` representing a new criterion with the specified trigger.",
        example = "use sand::prelude::*;\n\nfn demonstrate(trigger: sand::component::AdvancementTrigger)  {\n    let criterion = sand::component::Criterion::new(trigger);\n}",
    )]
    pub fn new(trigger: AdvancementTrigger) -> Self {
        Self { trigger }
    }
}

impl Serialize for Criterion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.trigger.serialize(serializer)
    }
}

// ── AdvancementRewards ────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::AdvancementRewards",
    aliases = ["sand::prelude::AdvancementRewards"],
    module = "sand::component",
    summary = "Rewards granted to the player when an advancement is completed.",
    context = "Rewards granted to the player when an advancement is completed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::AdvancementRewards;",
)]
/// Rewards granted to the player when an advancement is completed.
pub struct AdvancementRewards {
    recipes: Vec<String>,
    loot: Vec<String>,
    experience: i32,
    function: Option<String>,
}

impl AdvancementRewards {
    /// Creates a new advancement rewards container with no rewards set.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::new",
        aliases = ["sand::prelude::AdvancementRewards::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new advancement rewards container with no rewards set.",
        context = "Creates a new advancement rewards container with no rewards set. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "An `AdvancementRewards` representing a new advancement rewards container with no rewards set.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let advancement_rewards = sand::component::AdvancementRewards::new();\n}",
    )]
    pub fn new() -> Self {
        Self {
            recipes: Vec::new(),
            loot: Vec::new(),
            experience: 0,
            function: None,
        }
    }

    /// Adds a recipe unlock reward.
    ///
    /// Custom pack IDs remain typed: parse them as [`RecipeId`] or construct
    /// them from a validated [`ResourceLocation`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::recipe",
        aliases = ["sand::prelude::AdvancementRewards::recipe"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a recipe unlock reward. Custom pack IDs remain typed: parse them as [`RecipeId`] or construct them from a validated [`ResourceLocation`].",
        context = "Adds a recipe unlock reward. Custom pack IDs remain typed: parse them as [`RecipeId`] or construct them from a validated [`ResourceLocation`]. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(recipe = "`recipe` provides the typed Minecraft resource identifier used to add a recipe unlock reward. Custom pack IDs remain typed: parse them as [`RecipeId`] or construct them from a validated [`ResourceLocation`]."),
        returns = "The `AdvancementRewards` value with the documented change applied to add a recipe unlock reward. Custom pack IDs remain typed: parse them as [`RecipeId`] or construct them from a validated [`ResourceLocation`].",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, recipe: sand::resource_ref::RecipeId)  {\n    let updated_advancement_rewards = advancement_rewards_value.recipe(recipe);\n}",
    )]
    pub fn recipe(mut self, recipe: RecipeId) -> Self {
        self.recipes.push(recipe.to_string());
        self
    }

    /// Adds a recipe reward through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::raw_recipe",
        aliases = ["sand::prelude::AdvancementRewards::raw_recipe"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a recipe reward through the explicit raw compatibility path.",
        context = "Adds a recipe reward through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(recipe = "`recipe` provides the recipe added when building a recipe reward through the explicit raw compatibility path."),
        returns = "The `AdvancementRewards` value with the documented change applied to add a recipe reward through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, recipe: impl Into < String >)  {\n    let updated_advancement_rewards = advancement_rewards_value.raw_recipe(recipe);\n}",
    )]
    pub fn raw_recipe(mut self, recipe: impl Into<String>) -> Self {
        self.recipes.push(recipe.into());
        self
    }

    /// Adds a loot table reward.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::loot",
        aliases = ["sand::prelude::AdvancementRewards::loot"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a loot table reward.",
        context = "Adds a loot table reward. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(loot = "`loot` provides the typed Minecraft resource identifier used to add a loot table reward."),
        returns = "The `AdvancementRewards` value with the documented change applied to add a loot table reward.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, loot: sand::resource_ref::LootTableId)  {\n    let updated_advancement_rewards = advancement_rewards_value.loot(loot);\n}",
    )]
    pub fn loot(mut self, loot: LootTableId) -> Self {
        self.loot.push(loot.to_string());
        self
    }

    /// Adds a loot-table reward through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::raw_loot",
        aliases = ["sand::prelude::AdvancementRewards::raw_loot"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a loot-table reward through the explicit raw compatibility path.",
        context = "Adds a loot-table reward through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(loot = "`loot` provides the loot added when building a loot-table reward through the explicit raw compatibility path."),
        returns = "The `AdvancementRewards` value with the documented change applied to add a loot-table reward through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, loot: impl Into < String >)  {\n    let updated_advancement_rewards = advancement_rewards_value.raw_loot(loot);\n}",
    )]
    pub fn raw_loot(mut self, loot: impl Into<String>) -> Self {
        self.loot.push(loot.into());
        self
    }

    /// Sets the experience points awarded.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::experience",
        aliases = ["sand::prelude::AdvancementRewards::experience"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the experience points awarded.",
        context = "Sets the experience points awarded. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(xp = "`xp` provides the xp applied when setting the experience points awarded."),
        returns = "The `AdvancementRewards` value with the documented change applied to set the experience points awarded.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, xp: i32)  {\n    let updated_advancement_rewards = advancement_rewards_value.experience(xp);\n}",
    )]
    pub fn experience(mut self, xp: i32) -> Self {
        self.experience = xp;
        self
    }

    /// Sets a function to execute as a reward.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::function",
        aliases = ["sand::prelude::AdvancementRewards::function"],
        module = "sand::component",
        kind = "method",
        summary = "Sets a function to execute as a reward.",
        context = "Sets a function to execute as a reward. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(func = "`func` provides the typed Minecraft resource identifier used to set a function to execute as a reward."),
        returns = "The `AdvancementRewards` value with the documented change applied to set a function to execute as a reward.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, func: sand::resource_ref::FunctionId)  {\n    let updated_advancement_rewards = advancement_rewards_value.function(func);\n}",
    )]
    pub fn function(mut self, func: FunctionId) -> Self {
        self.function = Some(func.to_string());
        self
    }

    /// Sets a reward function through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::AdvancementRewards::raw_function",
        aliases = ["sand::prelude::AdvancementRewards::raw_function"],
        module = "sand::component",
        kind = "method",
        summary = "Sets a reward function through the explicit raw compatibility path.",
        context = "Sets a reward function through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(func = "`func` provides the func applied when setting a reward function through the explicit raw compatibility path."),
        returns = "The `AdvancementRewards` value with the documented change applied to set a reward function through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_rewards_value: sand::component::AdvancementRewards, func: impl Into < String >)  {\n    let updated_advancement_rewards = advancement_rewards_value.raw_function(func);\n}",
    )]
    pub fn raw_function(mut self, func: impl Into<String>) -> Self {
        self.function = Some(func.into());
        self
    }

    fn validate(&self) -> Result<(), (String, String)> {
        if self.experience < 0 {
            return Err((
                "rewards.experience".into(),
                "experience reward must be non-negative".into(),
            ));
        }
        for (index, recipe) in self.recipes.iter().enumerate() {
            validate_resource_id(recipe, &format!("rewards.recipes[{index}]"))
                .map_err(split_validation_message)?;
        }
        for (index, loot) in self.loot.iter().enumerate() {
            validate_resource_id(loot, &format!("rewards.loot[{index}]"))
                .map_err(split_validation_message)?;
        }
        if let Some(function) = &self.function {
            validate_resource_id(function, "rewards.function").map_err(split_validation_message)?;
        }
        Ok(())
    }
}

fn split_validation_message(message: String) -> (String, String) {
    message
        .split_once(": ")
        .map(|(path, detail)| (path.to_string(), detail.to_string()))
        .unwrap_or_else(|| ("advancement".into(), message))
}

impl Default for AdvancementRewards {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for AdvancementRewards {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        if !self.recipes.is_empty() {
            map.serialize_entry("recipes", &self.recipes)?;
        }
        if !self.loot.is_empty() {
            map.serialize_entry("loot", &self.loot)?;
        }
        if self.experience != 0 {
            map.serialize_entry("experience", &self.experience)?;
        }
        if let Some(ref f) = self.function {
            map.serialize_entry("function", f)?;
        }
        map.end()
    }
}

// ── Advancement ───────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Advancement",
    aliases = ["sand::prelude::Advancement"],
    module = "sand::component",
    summary = "A complete advancement definition for a Minecraft datapack.",
    context = "A complete advancement definition for a Minecraft datapack. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Advancement;",
)]
/// A complete advancement definition for a Minecraft datapack.
pub struct Advancement {
    location: ResourceLocation,
    parent: Option<String>,
    display: Option<AdvancementDisplay>,
    criteria: HashMap<String, Criterion>,
    requirements: Option<Vec<Vec<String>>>,
    rewards: Option<AdvancementRewards>,
    sends_telemetry_data: bool,
}

impl Advancement {
    /// Creates a new advancement with the specified resource location.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::new",
        aliases = ["sand::prelude::Advancement::new"],
        module = "sand::component",
        kind = "method",
        summary = "Creates a new advancement with the specified resource location.",
        context = "Creates a new advancement with the specified resource location. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(location = "`location` provides the typed resource identifier or location used to create a new advancement with the specified resource location."),
        returns = "An `Advancement` representing a new advancement with the specified resource location.",
        example = "use sand::prelude::*;\n\nfn demonstrate(location: sand::ResourceLocation)  {\n    let advancement = sand::component::Advancement::new(location);\n}",
    )]
    pub fn new(location: ResourceLocation) -> Self {
        Self {
            location,
            parent: None,
            display: None,
            criteria: HashMap::new(),
            requirements: None,
            rewards: None,
            sends_telemetry_data: false,
        }
    }

    /// Sets the parent advancement.
    ///
    /// ```
    /// use sand_components::{Advancement, AdvancementId, ResourceLocation};
    ///
    /// let advancement = Advancement::new(ResourceLocation::new("demo", "child").unwrap())
    ///     .parent("demo:root".parse::<AdvancementId>().unwrap());
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::parent",
        aliases = ["sand::prelude::Advancement::parent"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the parent advancement.",
        context = "Sets the parent advancement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(parent = "`parent` provides the typed Minecraft resource identifier used to set the parent advancement."),
        returns = "The `Advancement` value with the documented change applied to set the parent advancement.",
        example = "use {sand::component::Advancement, sand::resource_ref::AdvancementId, sand::ResourceLocation};\nlet advancement = Advancement::new(ResourceLocation::new(\"demo\", \"child\").unwrap())\n.parent(\"demo:root\".parse::<AdvancementId>().unwrap());",
    )]
    pub fn parent(mut self, parent: AdvancementId) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// Sets the parent through the explicit raw compatibility path.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::raw_parent",
        aliases = ["sand::prelude::Advancement::raw_parent"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the parent through the explicit raw compatibility path.",
        context = "Sets the parent through the explicit raw compatibility path. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(parent = "`parent` provides the parent applied when setting the parent through the explicit raw compatibility path."),
        returns = "The `Advancement` value with the documented change applied to set the parent through the explicit raw compatibility path.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, parent: impl Into < String >)  {\n    let updated_advancement = advancement_value.raw_parent(parent);\n}",
    )]
    pub fn raw_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Sets the display information for this advancement.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::display",
        aliases = ["sand::prelude::Advancement::display"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the display information for this advancement.",
        context = "Sets the display information for this advancement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(display = "`display` provides the display applied when setting the display information for this advancement."),
        returns = "The `Advancement` value with the documented change applied to set the display information for this advancement.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, display: sand::component::AdvancementDisplay)  {\n    let updated_advancement = advancement_value.display(display);\n}",
    )]
    pub fn display(mut self, display: AdvancementDisplay) -> Self {
        self.display = Some(display);
        self
    }

    /// Adds a criterion with the specified name.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::criterion",
        aliases = ["sand::prelude::Advancement::criterion"],
        module = "sand::component",
        kind = "method",
        summary = "Adds a criterion with the specified name.",
        context = "Adds a criterion with the specified name. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(name = "`name` provides the author-visible text added when building a criterion with the specified name.", criterion = "`criterion` provides the criterion added when building a criterion with the specified name."),
        returns = "The `Advancement` value with the documented change applied to add a criterion with the specified name.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, name: impl Into < String >, criterion: sand::component::Criterion)  {\n    let updated_advancement = advancement_value.criterion(name, criterion);\n}",
    )]
    pub fn criterion(mut self, name: impl Into<String>, criterion: Criterion) -> Self {
        self.criteria.insert(name.into(), criterion);
        self
    }

    /// Sets the requirements specifying how criteria must be completed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::requirements",
        aliases = ["sand::prelude::Advancement::requirements"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the requirements specifying how criteria must be completed.",
        context = "Sets the requirements specifying how criteria must be completed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(requirements = "`requirements` provides the requirements applied when setting the requirements specifying how criteria must be completed."),
        returns = "The `Advancement` value with the documented change applied to set the requirements specifying how criteria must be completed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, requirements: Vec < Vec < String > >)  {\n    let updated_advancement = advancement_value.requirements(requirements);\n}",
    )]
    pub fn requirements(mut self, requirements: Vec<Vec<String>>) -> Self {
        self.requirements = Some(requirements);
        self
    }

    /// Sets the rewards given when this advancement is completed.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::rewards",
        aliases = ["sand::prelude::Advancement::rewards"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the rewards given when this advancement is completed.",
        context = "Sets the rewards given when this advancement is completed. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(rewards = "`rewards` provides the rewards applied when setting the rewards given when this advancement is completed."),
        returns = "The `Advancement` value with the documented change applied to set the rewards given when this advancement is completed.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, rewards: sand::component::AdvancementRewards)  {\n    let updated_advancement = advancement_value.rewards(rewards);\n}",
    )]
    pub fn rewards(mut self, rewards: AdvancementRewards) -> Self {
        self.rewards = Some(rewards);
        self
    }

    /// Sets whether telemetry data is sent for this advancement.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Advancement::sends_telemetry_data",
        aliases = ["sand::prelude::Advancement::sends_telemetry_data"],
        module = "sand::component",
        kind = "method",
        summary = "Sets whether telemetry data is sent for this advancement.",
        context = "Sets whether telemetry data is sent for this advancement. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to set whether telemetry data is sent for this advancement."),
        returns = "The `Advancement` value with the documented change applied to set whether telemetry data is sent for this advancement.",
        example = "use sand::prelude::*;\n\nfn demonstrate(advancement_value: sand::component::Advancement, v: bool)  {\n    let updated_advancement = advancement_value.sends_telemetry_data(v);\n}",
    )]
    pub fn sends_telemetry_data(mut self, v: bool) -> Self {
        self.sends_telemetry_data = v;
        self
    }

    fn validation_error(
        &self,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> crate::error::SandError {
        crate::error::SandError::ComponentValidation {
            location: self.location.clone(),
            kind: "advancement".to_string(),
            field: field.into(),
            message: message.into(),
        }
    }
}

impl DatapackComponent for Advancement {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn validate(&self) -> crate::error::Result<()> {
        if self.criteria.is_empty() {
            return Err(self.validation_error("criteria", "at least one criterion is required"));
        }

        if let Some(parent) = &self.parent {
            validate_resource_id(parent, "parent")
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }
        if let Some(display) = &self.display {
            validate_resource_id(&display.icon.id, "display.icon.id")
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
            display.title.validate(&self.location, "display.title")?;
            display
                .description
                .validate(&self.location, "display.description")?;
            if let Some(background) = &display.background {
                validate_resource_id(background, "display.background")
                    .map_err(split_validation_message)
                    .map_err(|(field, message)| self.validation_error(field, message))?;
            }
        }
        if let Some(rewards) = &self.rewards {
            rewards
                .validate()
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }

        let mut criteria = self.criteria.iter().collect::<Vec<_>>();
        criteria.sort_by_key(|(name, _)| *name);
        for (name, criterion) in criteria {
            ResourceLocation::new("sand", name).map_err(|_| {
                self.validation_error(
                    format!("criteria.{name}"),
                    "criterion name must be non-empty and contain only [a-z0-9_./-]",
                )
            })?;
            let path = format!("criteria.{name}");
            criterion
                .trigger
                .validate_at(&path)
                .map_err(split_validation_message)
                .map_err(|(field, message)| self.validation_error(field, message))?;
        }

        if let Some(requirements) = &self.requirements {
            if requirements.is_empty() {
                return Err(self.validation_error(
                    "requirements",
                    "requirements must contain at least one group",
                ));
            }
            let mut referenced = std::collections::HashSet::new();
            for (group_index, group) in requirements.iter().enumerate() {
                if group.is_empty() {
                    return Err(self.validation_error(
                        format!("requirements[{group_index}]"),
                        "requirement group must contain at least one criterion",
                    ));
                }
                for (criterion_index, name) in group.iter().enumerate() {
                    if !self.criteria.contains_key(name) {
                        return Err(self.validation_error(
                            format!("requirements[{group_index}][{criterion_index}]"),
                            format!("references missing criterion `{name}`"),
                        ));
                    }
                    referenced.insert(name.as_str());
                }
            }
            if let Some(missing) = self
                .criteria
                .keys()
                .filter(|name| !referenced.contains(name.as_str()))
                .min()
            {
                return Err(self.validation_error(
                    "requirements",
                    format!("criterion `{missing}` is not referenced by any requirement group"),
                ));
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Value {
        self.try_to_json_for(None)
            .unwrap_or_else(|error| panic!("advancement serialization failed: {error}"))
    }

    fn try_content(&self) -> crate::error::Result<ComponentContent> {
        self.try_content_for(None)
    }

    fn try_content_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<ComponentContent> {
        self.validate()?;
        self.try_to_json_for(caps).map(ComponentContent::Json)
    }

    fn component_dir(&self) -> &'static str {
        "advancement"
    }
}

impl Advancement {
    /// Serialize this advancement's JSON, rendering each criterion's trigger
    /// through [`AdvancementTrigger::render_for`] for the given profile.
    ///
    /// `caps` is `None` on the compatibility path, treated the same as a
    /// fully-capable modern profile — see [`AdvancementTrigger::render_for`].
    fn try_to_json_for(
        &self,
        caps: Option<&sand_version::VersionCaps>,
    ) -> crate::error::Result<Value> {
        let mut map = serde_json::Map::new();

        if let Some(ref p) = self.parent {
            map.insert("parent".into(), Value::String(p.clone()));
        }
        if let Some(ref d) = self.display {
            map.insert(
                "display".into(),
                serde_json::to_value(d).map_err(crate::error::SandError::Serialization)?,
            );
        }

        let mut criteria_map = serde_json::Map::new();
        for (name, criterion) in &self.criteria {
            let trigger_json = criterion.trigger.render_for(caps).map_err(|error| {
                self.validation_error(format!("criteria.{name}"), error.to_string())
            })?;
            criteria_map.insert(name.clone(), trigger_json);
        }
        map.insert("criteria".into(), Value::Object(criteria_map));

        // Always emit `requirements`. Minecraft treats a missing/empty `requirements`
        // array as "no criteria required", which makes the advancement fire
        // unconditionally regardless of how restrictive the criteria conditions are
        // (see #233). When the caller hasn't supplied an explicit group layout, derive
        // a single AND-group covering every defined criterion — the correct default
        // for the common single- and multi-criterion "all must complete" case.
        let requirements: Vec<Vec<String>> = match &self.requirements {
            Some(reqs) => reqs.clone(),
            None => {
                let mut names: Vec<String> = self.criteria.keys().cloned().collect();
                names.sort();
                // `validate()` rejects zero-criteria advancements, but `to_json()`/
                // `content()` are documented infallible escape hatches that can be
                // called without validating first — don't synthesize a structurally
                // invalid single empty requirement group (`[[]]`) in that case.
                if names.is_empty() {
                    vec![]
                } else {
                    vec![names]
                }
            }
        };
        map.insert(
            "requirements".into(),
            serde_json::to_value(&requirements).map_err(crate::error::SandError::Serialization)?,
        );
        if let Some(ref r) = self.rewards {
            map.insert(
                "rewards".into(),
                serde_json::to_value(r).map_err(crate::error::SandError::Serialization)?,
            );
        }
        if self.sends_telemetry_data {
            map.insert("sends_telemetry_data".into(), Value::Bool(true));
        }

        Ok(Value::Object(map))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicates::{
        DamagePredicate, DamageSourcePredicate, EntityPredicate, FloatRange, IntRange,
        ItemPredicate, LocationPredicate,
    };
    use sand_commands::Text;

    fn advancement_id(value: &str) -> AdvancementId {
        value.parse().unwrap()
    }

    fn function_id(value: &str) -> FunctionId {
        value.parse().unwrap()
    }

    fn entity_type_id(path: &str) -> crate::registry::EntityTypeId {
        crate::registry::EntityTypeId::minecraft(path).unwrap()
    }

    fn biome_id(path: &str) -> crate::registry::BiomeId {
        crate::registry::BiomeId::minecraft(path).unwrap()
    }

    fn block_id(path: &str) -> BlockId {
        BlockId::minecraft(path).unwrap()
    }

    fn item_id(value: &str) -> ItemId {
        value.parse().unwrap()
    }

    fn loot_table_id(value: &str) -> LootTableId {
        value.parse().unwrap()
    }

    fn recipe_id(value: &str) -> RecipeId {
        value.parse().unwrap()
    }

    #[test]
    fn tick_trigger_serializes() {
        let t = AdvancementTrigger::Tick;
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:tick");
    }

    #[test]
    fn consume_item_typed() {
        let t = AdvancementTrigger::ConsumeItem {
            item: Some(ItemPredicate::id(item_id("minecraft:golden_apple"))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:consume_item");
        assert_eq!(
            v["conditions"]["item"]["items"],
            serde_json::json!(["minecraft:golden_apple"])
        );
    }

    #[test]
    fn player_killed_entity_typed() {
        let t = AdvancementTrigger::PlayerKilledEntity {
            entity: Some(EntityPredicate::type_(entity_type_id("ender_dragon"))),
            killing_blow: None,
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:player_killed_entity");
        assert_eq!(v["conditions"]["entity"]["type"], "minecraft:ender_dragon");
    }

    #[test]
    fn player_hurt_entity_with_damage() {
        let t = AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: Some(DamagePredicate::new().dealt(FloatRange::at_least(5.0))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:player_hurt_entity");
        assert_eq!(v["conditions"]["damage"]["dealt"]["min"], 5.0);
    }

    #[test]
    fn leveled_up_typed() {
        let t = AdvancementTrigger::LeveledUp {
            level: Some(IntRange::at_least(30)),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["conditions"]["level"]["min"], 30);
    }

    #[test]
    fn leveled_up_is_rejected_before_advancement_export() {
        let trigger = AdvancementTrigger::LeveledUp { level: None };
        let error = trigger.validate_for_target().unwrap_err();
        assert!(error.contains("minecraft:leveled_up"));
        assert!(error.contains("experience query"));
    }

    #[test]
    fn inventory_changed_items() {
        let t = AdvancementTrigger::InventoryChanged {
            slots: None,
            items: vec![ItemPredicate::id(item_id("minecraft:diamond"))],
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(
            v["conditions"]["items"][0]["items"],
            serde_json::json!(["minecraft:diamond"])
        );
    }

    #[test]
    fn location_trigger_typed() {
        let t = AdvancementTrigger::Location {
            location: Some(LocationPredicate::new().biome(biome_id("plains"))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["conditions"]["location"]["biome"], "minecraft:plains");
    }

    #[test]
    fn custom_trigger_escape_hatch() {
        use crate::raw::RawJson;
        let t = AdvancementTrigger::Custom {
            trigger: "mymod:do_thing".into(),
            conditions: Some(RawJson::new(serde_json::json!({"count": 5}))),
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "mymod:do_thing");
        assert_eq!(v["conditions"]["count"], 5);
    }

    #[test]
    fn custom_trigger_no_conditions() {
        let t = AdvancementTrigger::Custom {
            trigger: "minecraft:tick".into(),
            conditions: None,
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["trigger"], "minecraft:tick");
        assert!(v.get("conditions").is_none());
    }

    #[test]
    fn advancement_full_round_trip() {
        let adv = Advancement::new("test:adv".parse().unwrap())
            .criterion(
                "killed_dragon",
                Criterion::new(AdvancementTrigger::PlayerKilledEntity {
                    entity: Some(EntityPredicate::type_(entity_type_id("ender_dragon"))),
                    killing_blow: None,
                }),
            )
            .rewards(
                AdvancementRewards::new()
                    .experience(1000)
                    .function(function_id("test:reward")),
            );
        let json = adv.to_json();
        assert_eq!(
            json["criteria"]["killed_dragon"]["conditions"]["entity"][0]["predicate"]["minecraft:entity_type"],
            "minecraft:ender_dragon"
        );
        assert_eq!(json["rewards"]["experience"], 1000);
    }

    #[test]
    fn typed_display_icon_parent_and_rewards_emit_canonical_json() {
        let display = AdvancementDisplay::new(
            AdvancementIcon::new(item_id("minecraft:diamond")),
            Text::new("Shiny").aqua().bold(true),
            Text::new("Find a diamond"),
        )
        .background("minecraft:textures/block/stone.png".parse().unwrap());
        let advancement = tick_advancement("typed_surface")
            .parent(advancement_id("test:root"))
            .display(display)
            .rewards(
                AdvancementRewards::new()
                    .recipe(recipe_id("test:diamond_recipe"))
                    .loot(loot_table_id("test:diamond_reward"))
                    .function(function_id("test:complete")),
            );

        let json = advancement.to_json();
        assert_eq!(json["parent"], "test:root");
        assert_eq!(json["display"]["icon"]["id"], "minecraft:diamond");
        assert_eq!(json["display"]["title"]["text"], "Shiny");
        assert_eq!(json["display"]["title"]["color"], "aqua");
        assert_eq!(json["display"]["title"]["bold"], true);
        assert_eq!(json["display"]["description"]["text"], "Find a diamond");
        assert_eq!(json["rewards"]["recipes"][0], "test:diamond_recipe");
        assert_eq!(json["rewards"]["loot"][0], "test:diamond_reward");
        assert_eq!(json["rewards"]["function"], "test:complete");
    }

    #[test]
    fn explicit_raw_top_level_escape_hatches_preserve_valid_custom_shapes() {
        let display = AdvancementDisplay::raw_text(
            AdvancementIcon::raw("mymod:icon")
                .components(RawJson::new(serde_json::json!({"mymod:glow": true}))),
            RawJson::new(serde_json::json!(["Raw ", {"text": "title"}])),
            RawJson::new(serde_json::json!("Raw description")),
        )
        .raw_background("mymod:textures/gui/root.png");
        let advancement = tick_advancement("raw_surface")
            .raw_parent("mymod:root")
            .display(display)
            .rewards(
                AdvancementRewards::new()
                    .raw_recipe("mymod:recipe")
                    .raw_loot("mymod:loot")
                    .raw_function("mymod:complete"),
            );

        advancement.validate().unwrap();
        let json = advancement.to_json();
        assert_eq!(json["display"]["title"][0], "Raw ");
        assert_eq!(json["display"]["description"], "Raw description");
        assert_eq!(json["display"]["icon"]["components"]["mymod:glow"], true);
    }

    #[test]
    fn malformed_raw_display_text_fails_with_the_advancement_field_path() {
        let display = AdvancementDisplay::raw_text(
            AdvancementIcon::new(item_id("minecraft:stone")),
            RawJson::new(serde_json::json!(42)),
            RawJson::new(serde_json::json!("Description")),
        );
        let error = tick_advancement("bad_raw_text")
            .display(display)
            .try_content()
            .unwrap_err()
            .to_string();
        assert!(error.contains("field: display.title"), "{error}");
        assert!(error.contains("SAND-TEXT-SHAPE"), "{error}");
    }

    #[test]
    fn typed_top_level_ids_reject_malformed_resource_locations_at_construction() {
        for malformed in ["not namespaced", "minecraft:bad path", "Bad:id"] {
            assert!(malformed.parse::<AdvancementId>().is_err());
            assert!(malformed.parse::<ItemId>().is_err());
            assert!(malformed.parse::<RecipeId>().is_err());
            assert!(malformed.parse::<LootTableId>().is_err());
            assert!(malformed.parse::<FunctionId>().is_err());
        }
    }

    // ── Trigger ID golden tests ───────────────────────────────────────────────
    // One test per trigger variant asserting the exact vanilla trigger ID.

    fn trigger_id(t: &AdvancementTrigger) -> &str {
        t.trigger_id()
    }

    macro_rules! trigger_id_test {
        ($name:ident, $trigger:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(trigger_id(&$trigger), $expected);
            }
        };
    }

    trigger_id_test!(tick_id, AdvancementTrigger::Tick, "minecraft:tick");
    trigger_id_test!(
        impossible_id,
        AdvancementTrigger::Impossible,
        "minecraft:impossible"
    );
    trigger_id_test!(
        player_killed_entity_id,
        AdvancementTrigger::PlayerKilledEntity {
            entity: None,
            killing_blow: None
        },
        "minecraft:player_killed_entity"
    );
    trigger_id_test!(
        entity_killed_player_id,
        AdvancementTrigger::EntityKilledPlayer {
            entity: None,
            killing_blow: None
        },
        "minecraft:entity_killed_player"
    );
    trigger_id_test!(
        player_hurt_entity_id,
        AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: None
        },
        "minecraft:player_hurt_entity"
    );
    trigger_id_test!(
        entity_hurt_player_id,
        AdvancementTrigger::EntityHurtPlayer {
            entity: None,
            damage: None
        },
        "minecraft:entity_hurt_player"
    );
    trigger_id_test!(
        killed_by_crossbow_id,
        AdvancementTrigger::KilledByCrossbow {
            unique_entity_types: None,
            victims: None
        },
        "minecraft:killed_by_crossbow"
    );
    trigger_id_test!(
        channeled_lightning_id,
        AdvancementTrigger::ChanneledLightning { victims: None },
        "minecraft:channeled_lightning"
    );
    trigger_id_test!(
        lightning_strike_id,
        AdvancementTrigger::LightningStrike {
            lightning: None,
            bystander: None
        },
        "minecraft:lightning_strike"
    );
    trigger_id_test!(
        inventory_changed_id,
        AdvancementTrigger::InventoryChanged {
            slots: None,
            items: vec![]
        },
        "minecraft:inventory_changed"
    );
    trigger_id_test!(
        recipe_unlocked_id,
        AdvancementTrigger::RecipeUnlocked {
            recipe: "test:r".into()
        },
        "minecraft:recipe_unlocked"
    );
    trigger_id_test!(
        used_item_id,
        AdvancementTrigger::UsedItem { item: None },
        "minecraft:used_item"
    );
    trigger_id_test!(
        consume_item_id,
        AdvancementTrigger::ConsumeItem { item: None },
        "minecraft:consume_item"
    );
    trigger_id_test!(
        using_item_id,
        AdvancementTrigger::UsingItem { item: None },
        "minecraft:using_item"
    );
    trigger_id_test!(
        crafted_item_id,
        AdvancementTrigger::CraftedItem { item: None },
        "minecraft:crafted_item"
    );
    trigger_id_test!(
        filled_bucket_id,
        AdvancementTrigger::FilledBucket { item: None },
        "minecraft:filled_bucket"
    );
    trigger_id_test!(
        emptied_bucket_id,
        AdvancementTrigger::EmptiedBucket {
            item: None,
            location: None
        },
        "minecraft:emptied_bucket"
    );
    trigger_id_test!(
        shot_crossbow_id,
        AdvancementTrigger::ShotCrossbow { item: None },
        "minecraft:shot_crossbow"
    );
    trigger_id_test!(
        used_totem_id,
        AdvancementTrigger::UsedTotem { item: None },
        "minecraft:used_totem"
    );
    trigger_id_test!(
        thrown_item_picked_up_id,
        AdvancementTrigger::ThrownItemPickedUp {
            item: None,
            entity: None
        },
        "minecraft:thrown_item_picked_up"
    );
    trigger_id_test!(
        item_durability_changed_id,
        AdvancementTrigger::ItemDurabilityChanged {
            item: None,
            delta: None,
            durability: None
        },
        "minecraft:item_durability_changed"
    );
    trigger_id_test!(
        brewed_potion_id,
        AdvancementTrigger::BrewedPotion { potion: None },
        "minecraft:brewed_potion"
    );
    trigger_id_test!(
        bee_nest_destroyed_id,
        AdvancementTrigger::BeeNestDestroyed {
            block: None,
            item: None,
            num_bees_inside: None
        },
        "minecraft:bee_nest_destroyed"
    );
    trigger_id_test!(
        enchanted_item_id,
        AdvancementTrigger::EnchantedItem {
            item: None,
            levels: None
        },
        "minecraft:enchanted_item"
    );
    trigger_id_test!(
        bred_animals_id,
        AdvancementTrigger::BredAnimals {
            parent: None,
            partner: None,
            child: None
        },
        "minecraft:bred_animals"
    );
    trigger_id_test!(
        tamed_animal_id,
        AdvancementTrigger::TamedAnimal { entity: None },
        "minecraft:tame_animal"
    );
    trigger_id_test!(
        summoned_entity_id,
        AdvancementTrigger::SummonedEntity { entity: None },
        "minecraft:summoned_entity"
    );
    trigger_id_test!(
        player_interacted_with_entity_id,
        AdvancementTrigger::PlayerInteractedWithEntity {
            item: None,
            entity: None
        },
        "minecraft:player_interacted_with_entity"
    );
    trigger_id_test!(
        fishing_rod_hooked_id,
        AdvancementTrigger::FishingRodHooked {
            rod: None,
            entity: None,
            item: None
        },
        "minecraft:fishing_rod_hooked"
    );
    trigger_id_test!(
        villager_trade_id,
        AdvancementTrigger::VillagerTrade {
            item: None,
            villager: None
        },
        "minecraft:villager_trade"
    );
    trigger_id_test!(
        cured_zombie_villager_id,
        AdvancementTrigger::CuredZombieVillager {
            villager: None,
            zombie: None
        },
        "minecraft:cured_zombie_villager"
    );
    trigger_id_test!(
        placed_block_id,
        AdvancementTrigger::PlacedBlock {
            block: None,
            item: None,
            location: None,
            state: None
        },
        "minecraft:placed_block"
    );
    trigger_id_test!(
        enter_block_id,
        AdvancementTrigger::EnterBlock {
            block: None,
            state: None
        },
        "minecraft:enter_block"
    );
    trigger_id_test!(
        location_id,
        AdvancementTrigger::Location { location: None },
        "minecraft:location"
    );
    trigger_id_test!(
        nether_travel_id,
        AdvancementTrigger::NetherTravel {
            entered: None,
            exited: None,
            distance: None
        },
        "minecraft:nether_travel"
    );
    trigger_id_test!(
        changed_dimension_id,
        AdvancementTrigger::ChangedDimension {
            from: None,
            to: None
        },
        "minecraft:changed_dimension"
    );
    trigger_id_test!(
        slept_in_bed_id,
        AdvancementTrigger::SleptInBed { location: None },
        "minecraft:slept_in_bed"
    );
    trigger_id_test!(
        fall_from_height_id,
        AdvancementTrigger::FallFromHeight {
            distance: None,
            start_position: None
        },
        "minecraft:fall_from_height"
    );
    trigger_id_test!(
        slide_down_block_id,
        AdvancementTrigger::SlideDownBlock { block: None },
        "minecraft:slide_down_block"
    );
    trigger_id_test!(
        target_hit_id,
        AdvancementTrigger::TargetHit {
            signal_strength: None,
            projectile: None
        },
        "minecraft:target_hit"
    );
    trigger_id_test!(
        hero_of_the_village_id,
        AdvancementTrigger::HeroOfTheVillage { location: None },
        "minecraft:hero_of_the_village"
    );
    trigger_id_test!(
        player_generates_container_loot_id,
        AdvancementTrigger::PlayerGeneratesContainerLoot { loot_table: None },
        "minecraft:player_generates_container_loot"
    );
    trigger_id_test!(
        leveled_up_id,
        AdvancementTrigger::LeveledUp { level: None },
        "minecraft:leveled_up"
    );
    trigger_id_test!(
        effects_changed_id,
        AdvancementTrigger::EffectsChanged {
            effects: None,
            source: None
        },
        "minecraft:effects_changed"
    );
    trigger_id_test!(
        started_riding_id,
        AdvancementTrigger::StartedRiding,
        "minecraft:started_riding"
    );
    trigger_id_test!(
        construct_beacon_id,
        AdvancementTrigger::ConstructBeacon { level: None },
        "minecraft:construct_beacon"
    );
    trigger_id_test!(
        used_ender_eye_id,
        AdvancementTrigger::UsedEnderEye { distance: None },
        "minecraft:used_ender_eye"
    );
    // New 1.19+ triggers
    trigger_id_test!(
        allay_drop_item_on_block_id,
        AdvancementTrigger::AllayDropItemOnBlock {
            item: None,
            location: None
        },
        "minecraft:allay_drop_item_on_block"
    );
    trigger_id_test!(
        avoid_vibration_id,
        AdvancementTrigger::AvoidVibration,
        "minecraft:avoid_vibration"
    );
    trigger_id_test!(
        kill_mob_near_sculk_catalyst_id,
        AdvancementTrigger::KillMobNearSculkCatalyst {
            entity: None,
            killing_blow: None
        },
        "minecraft:kill_mob_near_sculk_catalyst"
    );
    trigger_id_test!(
        item_used_on_block_id,
        AdvancementTrigger::ItemUsedOnBlock {
            item: None,
            location: None
        },
        "minecraft:item_used_on_block"
    );
    trigger_id_test!(
        ride_entity_in_lava_id,
        AdvancementTrigger::RideEntityInLava {
            start_position: None,
            distance: None
        },
        "minecraft:ride_entity_in_lava"
    );

    #[test]
    fn advancement_range_validation_retains_owner_and_criterion_path() {
        let advancement = Advancement::new("test:bad_level".parse().unwrap()).criterion(
            "level_up",
            Criterion::new(AdvancementTrigger::LeveledUp {
                level: Some(IntRange::between(10, 2)),
            }),
        );
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("test:bad_level"));
        assert!(error.contains("criteria.level_up.conditions.level"));
    }

    #[test]
    fn advancement_non_finite_range_is_rejected_before_serialization() {
        let advancement = Advancement::new("test:bad_distance".parse().unwrap()).criterion(
            "eye",
            Criterion::new(AdvancementTrigger::UsedEnderEye {
                distance: Some(FloatRange::at_least(f64::NAN)),
            }),
        );
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("criteria.eye.conditions.distance.min"));
        assert!(error.contains("finite"));

        let nested = Advancement::new("test:bad_damage".parse().unwrap()).criterion(
            "hurt",
            Criterion::new(AdvancementTrigger::PlayerHurtEntity {
                entity: None,
                damage: Some(DamagePredicate::new().dealt(FloatRange::at_most(f64::INFINITY))),
            }),
        );
        let nested_error = nested.try_content().unwrap_err().to_string();
        assert!(nested_error.contains("criteria.hurt.conditions.damage.dealt.max"));
    }

    #[test]
    fn advancement_valid_and_custom_content_remain_compatible() {
        let valid = Advancement::new("test:valid_level".parse().unwrap()).criterion(
            "level",
            Criterion::new(AdvancementTrigger::ConstructBeacon {
                level: Some(IntRange::between(1, 4)),
            }),
        );
        assert_eq!(valid.try_content().unwrap(), valid.content());

        let custom = Advancement::new("test:custom".parse().unwrap()).criterion(
            "custom",
            Criterion::new(AdvancementTrigger::Custom {
                trigger: "mymod:trigger".to_string(),
                conditions: Some(RawJson::new(serde_json::json!({"anything": true}))),
            }),
        );
        assert_eq!(custom.try_content().unwrap(), custom.content());
    }

    fn tick_advancement(path: &str) -> Advancement {
        Advancement::new(format!("test:{path}").parse().unwrap())
            .criterion("tick", Criterion::new(AdvancementTrigger::Tick))
    }

    #[test]
    fn advancement_requires_criteria() {
        let advancement = Advancement::new("test:empty".parse().unwrap());
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("test:empty"));
        assert!(error.contains("field: criteria"));
        assert!(error.contains("at least one criterion"));
    }

    #[test]
    fn advancement_criterion_names_must_be_safe_and_nonempty() {
        for name in ["", "has space", "UPPER", "bad\nname"] {
            let advancement = Advancement::new("test:bad_name".parse().unwrap())
                .criterion(name, Criterion::new(AdvancementTrigger::Tick));
            let error = advancement.try_content().unwrap_err().to_string();
            assert!(error.contains("criterion name"), "{error}");
            assert!(error.contains("test:bad_name"), "{error}");
        }
    }

    #[test]
    fn advancement_requirements_must_be_nonempty_and_reference_criteria() {
        let empty = tick_advancement("empty_requirements").requirements(Vec::new());
        assert!(
            empty
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: requirements")
        );

        let empty_group = tick_advancement("empty_group").requirements(vec![Vec::new()]);
        assert!(
            empty_group
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: requirements[0]")
        );

        let missing =
            tick_advancement("missing_requirement").requirements(vec![vec!["missing".into()]]);
        let error = missing.try_content().unwrap_err().to_string();
        assert!(error.contains("field: requirements[0][0]"), "{error}");
        assert!(error.contains("missing criterion `missing`"), "{error}");

        let unreferenced = tick_advancement("unreferenced_requirement")
            .criterion("other", Criterion::new(AdvancementTrigger::Impossible))
            .requirements(vec![vec!["tick".into()]]);
        let error = unreferenced.try_content().unwrap_err().to_string();
        assert!(error.contains("field: requirements"), "{error}");
        assert!(
            error.contains("criterion `other` is not referenced"),
            "{error}"
        );
    }

    #[test]
    fn advancement_rejects_negative_experience_rewards() {
        let advancement =
            tick_advancement("negative_xp").rewards(AdvancementRewards::new().experience(-1));
        let error = advancement.try_content().unwrap_err().to_string();
        assert!(error.contains("field: rewards.experience"), "{error}");
        assert!(error.contains("non-negative"), "{error}");
    }

    #[test]
    fn advancement_validates_top_level_resource_references() {
        let invalid_parent = tick_advancement("bad_parent").raw_parent("not namespaced");
        assert!(
            invalid_parent
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: parent")
        );

        let display = AdvancementDisplay::new(
            AdvancementIcon::raw("bad icon"),
            Text::new("Title"),
            Text::new("Description"),
        );
        let invalid_icon = tick_advancement("bad_icon").display(display);
        assert!(
            invalid_icon
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: display.icon.id")
        );

        let display = AdvancementDisplay::new(
            AdvancementIcon::new(item_id("minecraft:stone")),
            Text::new("Title"),
            Text::new("Description"),
        )
        .raw_background("bad background");
        let invalid_background = tick_advancement("bad_background").display(display);
        assert!(
            invalid_background
                .try_content()
                .unwrap_err()
                .to_string()
                .contains("field: display.background")
        );
    }

    #[test]
    fn advancement_validates_reward_resource_references() {
        let rewards = [
            AdvancementRewards::new().raw_recipe("bad recipe"),
            AdvancementRewards::new().raw_loot("bad loot"),
            AdvancementRewards::new().raw_function("bad function"),
        ];
        let fields = ["rewards.recipes[0]", "rewards.loot[0]", "rewards.function"];
        for (rewards, field) in rewards.into_iter().zip(fields) {
            let error = tick_advancement("bad_reward")
                .rewards(rewards)
                .try_content()
                .unwrap_err()
                .to_string();
            assert!(error.contains(&format!("field: {field}")), "{error}");
        }
    }

    #[test]
    fn advancement_validates_trigger_resource_reference_strings() {
        let triggers = vec![
            AdvancementTrigger::RecipeUnlocked {
                recipe: "bad recipe".into(),
            },
            AdvancementTrigger::BrewedPotion {
                potion: Some("bad potion".into()),
            },
            AdvancementTrigger::BeeNestDestroyed {
                block: Some("bad block".into()),
                item: None,
                num_bees_inside: None,
            },
            AdvancementTrigger::PlacedBlock {
                block: Some("bad block".into()),
                item: None,
                location: None,
                state: None,
            },
            AdvancementTrigger::EnterBlock {
                block: Some("bad block".into()),
                state: None,
            },
            AdvancementTrigger::SlideDownBlock {
                block: Some("bad block".into()),
            },
            AdvancementTrigger::ChangedDimension {
                from: Some("bad dimension".into()),
                to: None,
            },
            AdvancementTrigger::PlayerGeneratesContainerLoot {
                loot_table: Some("bad loot".into()),
            },
            AdvancementTrigger::Custom {
                trigger: "bad trigger".into(),
                conditions: Some(RawJson::new(serde_json::json!({"opaque": true}))),
            },
        ];

        for (index, trigger) in triggers.into_iter().enumerate() {
            let error = Advancement::new(format!("test:bad_trigger_{index}").parse().unwrap())
                .criterion("event", Criterion::new(trigger))
                .try_content()
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("valid namespaced resource location"),
                "{error}"
            );
            assert!(error.contains("criteria.event"), "{error}");
        }
    }

    #[test]
    fn advancement_valid_resource_references_and_raw_conditions_are_preserved() {
        let advancement = Advancement::new("mymod:advancement".parse().unwrap())
            .parent(advancement_id("mymod:parent"))
            .display(
                AdvancementDisplay::new(
                    AdvancementIcon::new(item_id("mymod:icon")),
                    Text::new("Title"),
                    Text::new("Description"),
                )
                .background("mymod:textures/gui/background.png".parse().unwrap()),
            )
            .criterion(
                "custom/event",
                Criterion::new(AdvancementTrigger::Custom {
                    trigger: "mymod:custom_trigger".into(),
                    conditions: Some(RawJson::new(serde_json::json!({"future": {"x": 1}}))),
                }),
            )
            .requirements(vec![vec!["custom/event".into()]])
            .rewards(
                AdvancementRewards::new()
                    .recipe(recipe_id("mymod:recipe"))
                    .loot(loot_table_id("mymod:loot"))
                    .function(function_id("mymod:reward")),
            );

        assert_eq!(advancement.try_content().unwrap(), advancement.content());
    }

    #[test]
    fn typed_trigger_reference_constructors_preserve_vanilla_json() {
        let typed_and_legacy = [
            (
                AdvancementTrigger::recipe_unlocked("test:recipe".parse().unwrap()),
                AdvancementTrigger::RecipeUnlocked {
                    recipe: "test:recipe".into(),
                },
            ),
            (
                AdvancementTrigger::brewed_potion(crate::PotionId::Swiftness),
                AdvancementTrigger::BrewedPotion {
                    potion: Some("minecraft:swiftness".into()),
                },
            ),
            (
                AdvancementTrigger::bee_nest_destroyed(
                    Some(BlockId::minecraft("bee_nest").unwrap()),
                    None,
                    None,
                ),
                AdvancementTrigger::BeeNestDestroyed {
                    block: Some("minecraft:bee_nest".into()),
                    item: None,
                    num_bees_inside: None,
                },
            ),
            (
                AdvancementTrigger::placed_block(
                    Some(BlockId::minecraft("stone").unwrap()),
                    None,
                    None,
                    None,
                ),
                AdvancementTrigger::PlacedBlock {
                    block: Some("minecraft:stone".into()),
                    item: None,
                    location: None,
                    state: None,
                },
            ),
            (
                AdvancementTrigger::enter_block(Some(BlockId::minecraft("water").unwrap()), None),
                AdvancementTrigger::EnterBlock {
                    block: Some("minecraft:water".into()),
                    state: None,
                },
            ),
            (
                AdvancementTrigger::changed_dimension(
                    Some(DimensionId::minecraft("overworld").unwrap()),
                    Some(DimensionId::minecraft("the_nether").unwrap()),
                ),
                AdvancementTrigger::ChangedDimension {
                    from: Some("minecraft:overworld".into()),
                    to: Some("minecraft:the_nether".into()),
                },
            ),
            (
                AdvancementTrigger::slide_down_block(Some(
                    BlockId::minecraft("honey_block").unwrap(),
                )),
                AdvancementTrigger::SlideDownBlock {
                    block: Some("minecraft:honey_block".into()),
                },
            ),
            (
                AdvancementTrigger::player_generates_container_loot(Some(
                    "test:chests/reward".parse().unwrap(),
                )),
                AdvancementTrigger::PlayerGeneratesContainerLoot {
                    loot_table: Some("test:chests/reward".into()),
                },
            ),
            (
                AdvancementTrigger::custom_trigger(
                    "mymod:future_trigger".parse().unwrap(),
                    Some(RawJson::new(serde_json::json!({"future": true}))),
                ),
                AdvancementTrigger::Custom {
                    trigger: "mymod:future_trigger".into(),
                    conditions: Some(RawJson::new(serde_json::json!({"future": true}))),
                },
            ),
        ];

        for (typed, legacy) in typed_and_legacy {
            assert_eq!(
                serde_json::to_value(typed).unwrap(),
                serde_json::to_value(legacy).unwrap()
            );
        }
    }

    // ── Version-aware placed_block rendering golden tests (#232, #233) ────────

    fn elevator_wool_item_predicate() -> ItemPredicate {
        ItemPredicate::id(item_id("minecraft:white_wool")).custom_data_key("elevator")
    }

    fn caps_1_21_4() -> sand_version::VersionCaps {
        sand_version::VersionCaps::from_profile_flags(
            "1.21.4", false, false, true, true, true, true, true, true,
        )
    }

    fn caps_1_18_2() -> sand_version::VersionCaps {
        sand_version::VersionCaps::from_profile_flags(
            "1.18.2", false, false, false, false, false, false, false, false,
        )
    }

    fn caps_1_20_4() -> sand_version::VersionCaps {
        sand_version::VersionCaps::from_profile_flags(
            "1.20.4", false, false, false, true, true, false, true, false,
        )
    }

    #[test]
    fn placed_block_modern_render_matches_vanilla_location_check_and_match_tool() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );

        let v = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();

        assert_eq!(v["trigger"], "minecraft:placed_block");
        let location = v["conditions"]["location"]
            .as_array()
            .expect("conditions.location must be an array");
        assert_eq!(location.len(), 2);

        assert_eq!(location[0]["condition"], "minecraft:location_check");
        assert_eq!(
            location[0]["predicate"]["block"]["blocks"],
            serde_json::json!(["minecraft:white_wool"])
        );

        assert_eq!(location[1]["condition"], "minecraft:match_tool");
        assert_eq!(
            location[1]["predicate"]["items"],
            serde_json::json!(["minecraft:white_wool"])
        );
        assert_eq!(
            location[1]["predicate"]["predicates"]["minecraft:custom_data"],
            "{elevator:1b}"
        );

        // Regression guard for #233: the old flat shape must be gone.
        assert!(v["conditions"].get("block").is_none());
        assert!(v["conditions"].get("item").is_none());
    }

    // ── ItemMatcher integration (#229) ─────────────────────────────────────────

    #[test]
    fn item_matcher_renders_identical_predicate_to_hand_built_item_predicate() {
        use crate::item::matcher::ItemMatcher;

        let matcher = ItemMatcher::item(crate::registry::ItemId::minecraft("white_wool").unwrap())
            .custom_data_partial("elevator");
        let via_matcher = matcher
            .try_into_advancement_predicate(
                AdvancementItemConsumer::PlacedBlockTool,
                Some(&sand_version::VersionCaps::all_enabled()),
            )
            .unwrap();

        assert_eq!(
            serde_json::to_value(&via_matcher).unwrap(),
            serde_json::to_value(elevator_wool_item_predicate()).unwrap()
        );
    }

    #[test]
    fn item_matcher_predicate_drives_the_same_placed_block_modern_rendering() {
        use crate::item::matcher::ItemMatcher;

        let matcher = ItemMatcher::item(crate::registry::ItemId::minecraft("white_wool").unwrap())
            .custom_data_partial("elevator");
        let predicate = matcher
            .try_into_advancement_predicate(AdvancementItemConsumer::PlacedBlockTool, None)
            .unwrap();

        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(predicate),
            None,
            None,
        );
        let v = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();

        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location[1]["condition"], "minecraft:match_tool");
        assert_eq!(
            location[1]["predicate"]["predicates"]["minecraft:custom_data"],
            "{elevator:1b}"
        );
    }

    #[test]
    fn item_matcher_on_legacy_profile_fails_with_the_same_diagnostic_as_placed_block() {
        use crate::item::matcher::ItemMatcher;

        let matcher = ItemMatcher::item(crate::registry::ItemId::minecraft("white_wool").unwrap())
            .custom_data_partial("elevator");
        let matcher_err = matcher
            .try_into_advancement_predicate(
                AdvancementItemConsumer::PlacedBlockTool,
                Some(&caps_1_18_2()),
            )
            .unwrap_err()
            .to_string();

        let trigger_err = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        )
        .render_for(Some(&caps_1_18_2()))
        .unwrap_err()
        .to_string();

        assert!(matcher_err.contains("pre-item-component"));
        assert!(trigger_err.contains("pre-item-component"));
        assert!(trigger_err.contains("minecraft:placed_block"));
    }

    #[test]
    fn placed_block_modern_render_block_only_has_no_match_tool_condition() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 1);
        assert_eq!(location[0]["condition"], "minecraft:location_check");
    }

    #[test]
    fn placed_block_modern_render_item_only_has_no_location_check_condition() {
        let trigger = AdvancementTrigger::placed_block(
            None,
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 1);
        assert_eq!(location[0]["condition"], "minecraft:match_tool");
    }

    #[test]
    fn placed_block_unfiltered_emits_no_conditions() {
        let trigger = AdvancementTrigger::placed_block(None, None, None, None);
        let v = trigger.render_for(None).unwrap();
        assert!(v.get("conditions").is_none());
    }

    #[test]
    fn placed_block_render_for_no_profile_defaults_to_modern() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let no_profile = trigger.render_for(None).unwrap();
        let modern = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();
        assert_eq!(no_profile, modern);
    }

    #[test]
    fn schema_family_for_caps_maps_correctly() {
        assert_eq!(
            AdvancementSchemaFamily::for_caps(None),
            AdvancementSchemaFamily::NamespacedEntityPredicates,
            "no profile is treated as the fully-capable modern profile"
        );
        assert_eq!(
            AdvancementSchemaFamily::for_caps(Some(&sand_version::VersionCaps::all_enabled())),
            AdvancementSchemaFamily::NamespacedEntityPredicates,
        );
        assert_eq!(
            AdvancementSchemaFamily::for_caps(Some(&caps_1_21_4())),
            AdvancementSchemaFamily::LocationConditionItemComponents,
        );
        assert_eq!(
            AdvancementSchemaFamily::for_caps(Some(&caps_1_18_2())),
            AdvancementSchemaFamily::Legacy,
        );
    }

    #[test]
    fn placed_block_render_for_legacy_profile_keeps_flat_shape_for_block_only() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let v = trigger.render_for(Some(&caps_1_18_2())).unwrap();
        // Pre-item-component targets never had `location_check`/`match_tool`
        // wrapping for this trigger — output must keep the historical flat shape.
        // Note this intentionally diverges from `Serialize`/`render_for(None)`,
        // which always render the modern (correct) shape by default; the legacy
        // shape is reachable only by explicitly passing pre-item-component caps.
        assert_eq!(v["conditions"]["block"], "minecraft:white_wool");
        assert!(v["conditions"].get("item").is_none());
        assert!(v["conditions"].get("location").is_none());
    }

    #[test]
    fn placed_block_render_for_legacy_profile_rejects_item_filter() {
        // Sand has no verified pre-item-component item-predicate schema (#229
        // territory), so requesting an item filter on a legacy profile must fail
        // with an actionable diagnostic instead of emitting a modern-era
        // `components`/`predicates` shape the target version won't recognize.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let error = trigger
            .render_for(Some(&caps_1_20_4()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:placed_block"));
        assert!(error.contains("pre-item-component"));
    }

    #[test]
    fn item_used_on_block_render_for_legacy_profile_rejects_item_filter() {
        let trigger = AdvancementTrigger::ItemUsedOnBlock {
            item: Some(elevator_wool_item_predicate()),
            location: None,
        };
        let error = trigger
            .render_for(Some(&caps_1_20_4()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:item_used_on_block"));
        assert!(error.contains("pre-item-component"));
    }

    #[test]
    fn placed_block_serialize_never_uses_legacy_flat_shape() {
        // Regression guard for the "Criterion::Serialize latent trap" found in
        // review: the plain `Serialize` impl (used by `Criterion` and any
        // direct `serde_json::to_value` caller) must always render the modern,
        // correct schema — never silently fall back to the pre-#233 shape.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let via_serialize = serde_json::to_value(&trigger).unwrap();
        let via_render_for_none = trigger.render_for(None).unwrap();
        assert_eq!(via_serialize, via_render_for_none);
        assert!(via_serialize["conditions"]["location"].is_array());
        assert!(via_serialize["conditions"].get("block").is_none());
        assert!(via_serialize["conditions"].get("item").is_none());
    }

    #[test]
    fn criterion_serialize_uses_modern_placed_block_shape() {
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            None,
            None,
            None,
        );
        let criterion = Criterion::new(trigger);
        let v = serde_json::to_value(&criterion).unwrap();
        assert!(v["conditions"]["location"].is_array());
    }

    #[test]
    fn item_used_on_block_modern_render_uses_location_check_and_match_tool() {
        let trigger = AdvancementTrigger::ItemUsedOnBlock {
            item: Some(elevator_wool_item_predicate()),
            location: Some(LocationPredicate::new().biome(biome_id("plains"))),
        };
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        assert_eq!(location.len(), 2);
        assert_eq!(location[0]["condition"], "minecraft:location_check");
        assert_eq!(location[0]["predicate"]["biomes"], "minecraft:plains");
        assert_eq!(location[1]["condition"], "minecraft:match_tool");
    }

    #[test]
    fn entity_conditions_render_as_loot_conditions_for_each_schema_family() {
        let trigger = AdvancementTrigger::PlayerKilledEntity {
            entity: Some(EntityPredicate::type_(entity_type_id("ender_dragon"))),
            killing_blow: None,
        };
        let stable = trigger.render_for(Some(&caps_1_21_4())).unwrap();
        assert_eq!(
            stable["conditions"]["entity"][0]["predicate"]["type"],
            "minecraft:ender_dragon"
        );
        assert_eq!(
            stable["conditions"]["entity"][0]["condition"],
            "minecraft:entity_properties"
        );

        let latest = trigger
            .render_for(Some(&sand_version::VersionCaps::all_enabled()))
            .unwrap();
        assert_eq!(
            latest["conditions"]["entity"][0]["predicate"]["minecraft:entity_type"],
            "minecraft:ender_dragon"
        );
        assert!(
            latest["conditions"]["entity"][0]["predicate"]
                .get("type")
                .is_none()
        );
    }

    #[test]
    fn location_intent_uses_player_entity_context_and_current_location_shape() {
        let trigger = AdvancementTrigger::Location {
            location: Some(
                LocationPredicate::new()
                    .biome(biome_id("plains"))
                    .y(FloatRange::at_least(64.0)),
            ),
        };
        let stable = trigger.render_for(Some(&caps_1_21_4())).unwrap();
        let predicate = &stable["conditions"]["player"][0]["predicate"]["location"];
        assert_eq!(predicate["biomes"], "minecraft:plains");
        assert_eq!(predicate["position"]["y"]["min"], 64.0);
        assert!(stable["conditions"].get("location").is_none());

        let latest = trigger.render_for(None).unwrap();
        assert_eq!(
            latest["conditions"]["player"][0]["predicate"]["minecraft:location"]["biomes"],
            "minecraft:plains"
        );
    }

    #[test]
    fn allay_drop_uses_location_and_match_tool_consumers() {
        let trigger = AdvancementTrigger::AllayDropItemOnBlock {
            item: Some(ItemPredicate::id(item_id("minecraft:cake"))),
            location: Some(LocationPredicate::new().block(
                crate::predicates::BlockPredicate::new().blocks(vec![block_id("note_block")]),
            )),
        };
        let value = trigger.render_for(Some(&caps_1_21_4())).unwrap();
        let conditions = value["conditions"]["location"].as_array().unwrap();
        assert_eq!(conditions[0]["condition"], "minecraft:location_check");
        assert_eq!(conditions[1]["condition"], "minecraft:match_tool");
        assert!(value["conditions"].get("item").is_none());
    }

    #[test]
    fn non_placement_component_item_filter_rejects_legacy_profile() {
        let trigger = AdvancementTrigger::ConsumeItem {
            item: Some(elevator_wool_item_predicate()),
        };
        let error = trigger
            .render_for(Some(&caps_1_18_2()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:consume_item"), "{error}");
        assert!(error.contains("item-component"), "{error}");
    }

    #[test]
    fn nested_equipment_component_filter_rejects_legacy_profile() {
        let trigger = AdvancementTrigger::PlayerKilledEntity {
            entity: Some(EntityPredicate::type_(entity_type_id("zombie")).equipment(
                crate::predicates::EntityEquipment::new().head(elevator_wool_item_predicate()),
            )),
            killing_blow: None,
        };
        let error = trigger
            .render_for(Some(&caps_1_18_2()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("minecraft:player_killed_entity"), "{error}");
        assert!(error.contains("item-component"), "{error}");
    }

    #[test]
    fn typed_damage_source_tags_render_for_supported_profiles() {
        let trigger = AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: Some(
                DamagePredicate::new().type_(
                    DamageSourcePredicate::new().requires_tag(
                        crate::registry::TagId::<crate::registry::DamageTypeId>::minecraft(
                            "is_projectile",
                        )
                        .unwrap(),
                    ),
                ),
            ),
        };
        let rendered = trigger.render_for(Some(&caps_1_21_4())).unwrap();
        assert_eq!(
            rendered["conditions"]["damage"]["type"]["tags"][0],
            serde_json::json!({"id": "minecraft:is_projectile", "expected": true})
        );
    }

    #[test]
    fn raw_item_predicate_is_preserved_verbatim() {
        let raw_predicate = serde_json::json!({"legacy": {"user_owned": true}});
        let trigger = AdvancementTrigger::ConsumeItem {
            item: Some(ItemPredicate::raw(RawJson::new(raw_predicate.clone()))),
        };
        let rendered = trigger.render_for(Some(&caps_1_21_4())).unwrap();
        assert_eq!(rendered["conditions"]["item"], raw_predicate);
    }

    #[test]
    fn raw_custom_conditions_preserve_every_json_kind_on_fallback_profiles() {
        for conditions in [
            serde_json::json!([1, 2]),
            serde_json::json!("opaque"),
            serde_json::json!(null),
        ] {
            let trigger = AdvancementTrigger::Custom {
                trigger: "minecraft:tick".into(),
                conditions: Some(RawJson::new(conditions.clone())),
            };
            let rendered = trigger
                .render_for(Some(&sand_version::VersionCaps::all_disabled()))
                .unwrap();
            assert_eq!(rendered["conditions"], conditions);
        }
    }

    #[test]
    fn nested_raw_predicates_are_preserved_verbatim() {
        let raw_entity = serde_json::json!({"future:entity": {"value": 1}});
        let entity_trigger = AdvancementTrigger::PlayerKilledEntity {
            entity: Some(EntityPredicate::raw(RawJson::new(raw_entity.clone()))),
            killing_blow: None,
        };
        assert_eq!(
            entity_trigger.render_for(Some(&caps_1_21_4())).unwrap()["conditions"]["entity"][0]["predicate"],
            raw_entity
        );

        let raw_location = serde_json::json!({"future:location": true});
        let location_trigger = AdvancementTrigger::Location {
            location: Some(LocationPredicate::raw(RawJson::new(raw_location.clone()))),
        };
        assert_eq!(
            location_trigger.render_for(Some(&caps_1_21_4())).unwrap()["conditions"]["player"][0]["predicate"]
                ["location"],
            raw_location
        );

        let raw_damage = serde_json::json!({"future:damage": {"value": 2}});
        let damage_trigger = AdvancementTrigger::PlayerHurtEntity {
            entity: None,
            damage: Some(DamagePredicate::raw(RawJson::new(raw_damage.clone()))),
        };
        assert_eq!(
            damage_trigger.render_for(Some(&caps_1_21_4())).unwrap()["conditions"]["damage"],
            raw_damage
        );
    }

    #[test]
    fn legacy_typed_location_filters_fail_but_raw_remains_user_owned() {
        let typed = AdvancementTrigger::Location {
            location: Some(LocationPredicate::new().biome(biome_id("plains"))),
        };
        let error = typed
            .render_for(Some(&caps_1_18_2()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("no verified"), "{error}");

        let raw = AdvancementTrigger::Location {
            location: Some(LocationPredicate::raw(RawJson::new(
                serde_json::json!({"biome": "minecraft:plains"}),
            ))),
        };
        assert!(raw.render_for(Some(&caps_1_18_2())).is_ok());
    }

    #[test]
    fn unfiltered_location_condition_triggers_emit_no_conditions() {
        for trigger in [
            AdvancementTrigger::ItemUsedOnBlock {
                item: None,
                location: None,
            },
            AdvancementTrigger::AllayDropItemOnBlock {
                item: None,
                location: None,
            },
        ] {
            let rendered = trigger.render_for(Some(&caps_1_21_4())).unwrap();
            assert!(rendered.get("conditions").is_none(), "{rendered}");
        }
    }

    #[test]
    fn current_replacement_triggers_render_deterministically_for_both_profiles() {
        let triggers = [
            AdvancementTrigger::KilledByArrow {
                unique_entity_types: Some(IntRange::at_least(2)),
                fired_from_weapon: Some(ItemPredicate::id(item_id("minecraft:crossbow"))),
                victims: Some(vec![EntityPredicate::type_(entity_type_id("phantom"))]),
            },
            AdvancementTrigger::RecipeCrafted {
                recipe_id: "minecraft:decorated_pot".into(),
                ingredients: vec![ItemPredicate::id(item_id("minecraft:brick"))],
            },
            AdvancementTrigger::ThrownItemPickedUpByEntity {
                item: Some(ItemPredicate::id(item_id("minecraft:cookie"))),
                entity: Some(EntityPredicate::type_(entity_type_id("allay"))),
            },
            AdvancementTrigger::ThrownItemPickedUpByPlayer {
                item: Some(ItemPredicate::id(item_id("minecraft:cookie"))),
                entity: Some(EntityPredicate::type_(entity_type_id("allay"))),
            },
        ];
        for caps in [caps_1_21_4(), sand_version::VersionCaps::all_enabled()] {
            for trigger in &triggers {
                let first = trigger.render_for(Some(&caps)).unwrap();
                let second = trigger.render_for(Some(&caps)).unwrap();
                assert_eq!(first, second);
                assert_eq!(first["trigger"], trigger.trigger_id());
            }
        }
    }

    #[test]
    fn trigger_version_ranges_apply_to_components_and_preserve_custom_escape_hatch() {
        let too_new = AdvancementTrigger::AllayDropItemOnBlock {
            item: None,
            location: None,
        };
        assert!(too_new.render_for(Some(&caps_1_18_2())).is_err());

        let legacy_crossbow = AdvancementTrigger::KilledByCrossbow {
            unique_entity_types: None,
            victims: None,
        };
        assert!(legacy_crossbow.render_for(Some(&caps_1_18_2())).is_ok());
        assert!(legacy_crossbow.render_for(Some(&caps_1_21_4())).is_err());

        let custom_known_id = AdvancementTrigger::Custom {
            trigger: "minecraft:tick".into(),
            conditions: None,
        };
        assert!(
            custom_known_id
                .render_for(Some(&sand_version::VersionCaps::all_disabled()))
                .is_ok()
        );
    }

    #[test]
    fn invalid_typed_trigger_is_rejected_but_custom_escape_hatch_is_preserved() {
        let typed = AdvancementTrigger::CraftedItem { item: None };
        assert!(typed.render_for(Some(&caps_1_21_4())).is_err());

        let raw = AdvancementTrigger::Custom {
            trigger: "minecraft:crafted_item".into(),
            conditions: Some(RawJson::new(serde_json::json!({"future": true}))),
        };
        assert_eq!(
            raw.render_for(Some(&caps_1_21_4())).unwrap(),
            serde_json::json!({
                "trigger": "minecraft:crafted_item",
                "conditions": {"future": true}
            })
        );
    }

    #[test]
    fn placed_block_render_rejects_conflicting_block_shorthand_and_location_block() {
        let trigger = AdvancementTrigger::PlacedBlock {
            block: Some("minecraft:white_wool".into()),
            item: None,
            location: Some(
                LocationPredicate::new()
                    .block(crate::predicates::BlockPredicate::new().blocks(vec![block_id("dirt")])),
            ),
            state: None,
        };
        let error = trigger.render_for(None).unwrap_err().to_string();
        assert!(error.contains("block"), "{error}");
    }

    #[test]
    fn placed_block_regression_dirt_and_plain_wool_are_structurally_excluded() {
        // Reproduces the #233 scenario: the generated predicate must only match
        // the exact block id and carry the custom-data partial-match condition,
        // so unrelated placements (dirt) and the un-tagged base item (plain
        // white wool with no `elevator` custom_data) cannot satisfy it.
        let trigger = AdvancementTrigger::placed_block(
            Some(BlockId::minecraft("white_wool").unwrap()),
            Some(elevator_wool_item_predicate()),
            None,
            None,
        );
        let v = trigger.render_for(None).unwrap();
        let location = v["conditions"]["location"].as_array().unwrap();
        let blocks = location[0]["predicate"]["block"]["blocks"]
            .as_array()
            .unwrap();
        assert_eq!(blocks, &[Value::String("minecraft:white_wool".into())]);
        assert_ne!(blocks[0], "minecraft:dirt");
        // The match_tool predicate requires the `elevator` custom_data marker,
        // which plain (untagged) white wool does not carry.
        assert_eq!(
            location[1]["predicate"]["predicates"]["minecraft:custom_data"],
            "{elevator:1b}"
        );
    }

    // ── requirements auto-derivation (#233) ────────────────────────────────────

    #[test]
    fn advancement_requirements_auto_derived_single_criterion() {
        let advancement = Advancement::new("test:single".parse().unwrap())
            .criterion(
                "event",
                Criterion::new(AdvancementTrigger::placed_block(
                    Some(BlockId::minecraft("white_wool").unwrap()),
                    None,
                    None,
                    None,
                )),
            )
            .rewards(AdvancementRewards::new().function(function_id("test:reward")));
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["event"]]));
    }

    #[test]
    fn advancement_requirements_auto_derived_multi_criterion_is_one_and_group() {
        let advancement = Advancement::new("test:multi".parse().unwrap())
            .criterion("a", Criterion::new(AdvancementTrigger::Tick))
            .criterion("b", Criterion::new(AdvancementTrigger::Impossible));
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["a", "b"]]));
    }

    #[test]
    fn advancement_explicit_requirements_are_preserved_when_set() {
        let advancement = Advancement::new("test:explicit".parse().unwrap())
            .criterion("a", Criterion::new(AdvancementTrigger::Tick))
            .criterion("b", Criterion::new(AdvancementTrigger::Impossible))
            .requirements(vec![vec!["a".into()], vec!["b".into()]]);
        let json = advancement.to_json();
        assert_eq!(json["requirements"], serde_json::json!([["a"], ["b"]]));
    }

    #[test]
    fn effects_changed_constructor_uses_typed_status_effect_keys() {
        let typed = AdvancementTrigger::effects_changed(
            [(
                crate::EffectId::Speed,
                EffectPredicate::new().amplifier(IntRange::exact(1)),
            )],
            None,
        );
        assert_eq!(
            serde_json::to_value(typed).unwrap(),
            serde_json::json!({
                "trigger": "minecraft:effects_changed",
                "conditions": {
                    "effects": {
                        "minecraft:speed": {"amplifier": 1}
                    }
                }
            })
        );

        let unfiltered = AdvancementTrigger::effects_changed_any(None);
        assert_eq!(
            serde_json::to_value(unfiltered).unwrap(),
            serde_json::json!({"trigger": "minecraft:effects_changed"})
        );
    }

    #[test]
    fn typed_trigger_ids_reject_malformed_resource_locations_at_construction() {
        assert!("bad recipe".parse::<ResourceLocation>().is_err());
        assert!(BlockId::minecraft("bad block").is_err());
        assert!(DimensionId::minecraft("bad dimension").is_err());
        assert!(PotionRegistryId::minecraft("bad potion").is_err());
        assert!(StatusEffectId::minecraft("bad effect").is_err());
    }
}
