//! Typed dialog datapack component builders.
//!
//! Dialogs are part of Sand's Minecraft 26+ baseline for displaying data-driven
//! UI panels to players. They live at `data/<namespace>/dialog/<path>.json`.
//!
//! Always gate dialog usage with `VersionProfile::supports(VersionFeature::Dialogs)`:
//! ```rust,ignore
//! if profile.supports(VersionFeature::Dialogs) {
//!     let d = Dialog::notice_local("welcome")
//!         .title(Text::new("Welcome!").gold())
//!         .body(DialogBody::text(Text::new("Choose what to do next.")))
//!         .button(DialogButton::new(Text::new("Start").green()));
//! }
//! ```

use std::sync::{Mutex, OnceLock};

use crate::function::FunctionRef;
use crate::registry::DialogId;
use crate::resource_location::SAND_LOCAL_NS;
use crate::{DatapackComponent, ResourceLocation};
use sand_commands::{CommandProfile, Text, TextComponent};
use serde_json::{Value, json};

// ── Dialog callback registry ──────────────────────────────────────────────────

/// The scoreboard trigger objective Sand uses for dialog callbacks.
pub const SAND_DIALOG_TRIGGER: &str = "sand.dialog";

/// `next_id` and `callbacks` are reset together (see
/// [`reset_dialog_callbacks_for_export`]) so a fresh export's first
/// registration always gets ID 1, regardless of how many callbacks earlier
/// exports in the same process registered.
struct CallbackRegistry {
    next_id: u32,
    callbacks: Vec<(u32, String)>,
}

fn callback_registry() -> &'static Mutex<CallbackRegistry> {
    static REG: OnceLock<Mutex<CallbackRegistry>> = OnceLock::new();
    REG.get_or_init(|| {
        Mutex::new(CallbackRegistry {
            next_id: 1,
            callbacks: Vec::new(),
        })
    })
}

/// Register a dialog callback function and return its stable trigger ID.
///
/// The path must be a full `namespace:path` or a `__sand_local:path` sentinel.
/// IDs start at 1 (0 means "trigger not yet set").
///
/// Called from `DialogAction::to_json`, i.e. at serialization time rather
/// than at `DialogAction::callback` construction time — this is what lets a
/// cached/prebuilt `Dialog` (e.g. behind a `LazyLock`, constructed once but
/// exported many times) re-register its callback on every export even though
/// it is only *built* once.
pub fn register_dialog_callback(path: String) -> u32 {
    let mut registry = callback_registry().lock().unwrap();
    let id = registry.next_id;
    registry.next_id = registry
        .next_id
        .checked_add(1)
        .expect("dialog callback trigger ID space exhausted");
    registry.callbacks.push((id, path));
    id
}

/// Start a fresh dialog callback registration lifecycle for one export.
///
/// This is an exporter implementation detail. Safe to call before component
/// factories run (unlike a hypothetical reset that clears the *registry* of
/// already-embedded IDs): registration itself happens at JSON-serialization
/// time, not at `DialogAction::callback` construction time, so a reset here
/// never discards a prebuilt dialog's callback — it just re-registers fresh
/// when that dialog is serialized during this export.
#[doc(hidden)]
pub fn reset_dialog_callbacks_for_export() {
    let mut registry = callback_registry().lock().unwrap();
    registry.next_id = 1;
    registry.callbacks.clear();
}

/// Drain all registered dialog callbacks.
///
/// Returns `(trigger_id, function_path)` pairs. Called by the exporter to
/// generate the dialog dispatch tick/load functions.
pub fn drain_dialog_callbacks() -> Vec<(u32, String)> {
    let mut registry = callback_registry().lock().unwrap();
    registry.next_id = 1;
    std::mem::take(&mut registry.callbacks)
}

/// Validate a raw `namespace:path` resource reference string.
///
/// Used to validate dialog IDs, `open_dialog` targets, and function/callback
/// paths at [`Dialog::validate`] time rather than at construction time.
fn validate_resource_ref(raw: &str) -> std::result::Result<(), String> {
    raw.parse::<ResourceLocation>()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// A validated text component accepted by dialog labels and body content.
#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogText",
    module = "sand::component",
    summary = "A validated text component accepted by dialog labels and body content.",
    context = "A validated text component accepted by dialog labels and body content. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogText;",
)]
#[derive(Debug, Clone)]
pub struct DialogText(TextComponent);

impl DialogText {
    fn to_json(&self) -> Value {
        serde_json::from_str(&self.0.to_string()).expect("TextComponent must serialize to JSON")
    }

    fn validate(&self, path: &str) -> crate::error::Result<()> {
        self.0
            .validate_at_path(&CommandProfile::unprofiled(), path)
            .map_err(|error| crate::error::SandError::ComponentValidation {
                location: ResourceLocation::new("sand", "text").expect("static resource location"),
                kind: "text".to_string(),
                field: error.field,
                message: format!("error[{}] {}", error.code, error.message),
            })
    }
}

impl From<TextComponent> for DialogText {
    fn from(value: TextComponent) -> Self {
        Self(value)
    }
}

impl From<&TextComponent> for DialogText {
    fn from(value: &TextComponent) -> Self {
        Self(value.clone())
    }
}

impl From<&str> for DialogText {
    fn from(value: &str) -> Self {
        Self(Text::new(value))
    }
}

impl From<String> for DialogText {
    fn from(value: String) -> Self {
        Self(Text::new(value))
    }
}

// ── DialogTag ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogTag",
    aliases = ["sand::prelude::DialogTag"],
    module = "sand::component",
    summary = "A well-known vanilla dialog tag. Dialog tags expose dialogs through Minecraft UI entry points such as the pause screen and Quick Actions. These helpers emit the vanilla tag files:.",
    context = "A well-known vanilla dialog tag. Dialog tags expose dialogs through Minecraft UI entry points such as the pause screen and Quick Actions. These helpers emit the vanilla tag files: - `data/minecraft/tags/dialog/pause_screen_additions.json` - `data/minecraft/tags/dialog/quick_actions.json`",
    minecraft = "Dialog tags expose dialogs through Minecraft UI entry points such as the pause screen and Quick Actions. These helpers emit the vanilla tag files:",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogTag;",
)]
/// A well-known vanilla dialog tag.
///
/// Dialog tags expose dialogs through Minecraft UI entry points such as the
/// pause screen and Quick Actions. These helpers emit the vanilla tag files:
///
/// - `data/minecraft/tags/dialog/pause_screen_additions.json`
/// - `data/minecraft/tags/dialog/quick_actions.json`
#[derive(Debug, Clone)]
pub struct DialogTag {
    location: ResourceLocation,
    replace: bool,
    values: Vec<String>,
}

impl DialogTag {
    /// Tag dialogs shown in the pause screen additions menu.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogTag::pause_screen_additions",
        aliases = ["sand::prelude::DialogTag::pause_screen_additions"],
        module = "sand::component",
        kind = "method",
        summary = "Tag dialogs shown in the pause screen additions menu.",
        context = "Tag dialogs shown in the pause screen additions menu. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "A `DialogTag` identifying dialogs shown in the pause screen additions menu.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let dialog_tag = sand::component::DialogTag::pause_screen_additions();\n}",
    )]
    pub fn pause_screen_additions() -> Self {
        Self::well_known("pause_screen_additions")
    }

    /// Tag dialogs shown by the Quick Actions key.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogTag::quick_actions",
        aliases = ["sand::prelude::DialogTag::quick_actions"],
        module = "sand::component",
        kind = "method",
        summary = "Tag dialogs shown by the Quick Actions key.",
        context = "Tag dialogs shown by the Quick Actions key. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "A `DialogTag` identifying dialogs shown by the Quick Actions key.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let dialog_tag = sand::component::DialogTag::quick_actions();\n}",
    )]
    pub fn quick_actions() -> Self {
        Self::well_known("quick_actions")
    }

    fn well_known(path: &str) -> Self {
        Self {
            location: ResourceLocation::minecraft(format!("dialog/{path}"))
                .expect("well-known dialog tag path must be valid"),
            replace: false,
            values: Vec::new(),
        }
    }

    /// Add a dialog entry to this tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogTag::dialog",
        aliases = ["sand::prelude::DialogTag::dialog"],
        module = "sand::component",
        kind = "method",
        summary = "Add a dialog entry to this tag.",
        context = "Add a dialog entry to this tag. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(dialog = "`dialog` provides the dialog added when building a dialog entry to this tag."),
        returns = "The `DialogTag` value with the documented change applied to add a dialog entry to this tag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_tag_value: DialogTag, dialog: DialogId) {\n    let updated_dialog_tag = dialog_tag_value.dialog(dialog);\n}",
    )]
    pub fn dialog(mut self, dialog: DialogId) -> Self {
        self.values.push(dialog.to_string());
        self
    }

    /// Adds an explicitly raw dialog reference to this tag.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogTag::dialog_raw", aliases = ["sand::prelude::DialogTag::dialog_raw"], module = "sand::component", kind = "method", summary = "Adds an explicitly raw dialog reference to this tag.", context = "Advanced escape hatch for unsupported dialog reference syntax.", minecraft = "Serializes the supplied dialog token verbatim in the function tag.", use_when = ["Using future or modded dialog syntax"], avoid_when = ["A validated DialogId is available"], params(dialog = "The unchecked dialog token."), returns = "This dialog tag with the raw entry added.", example = "let tag = DialogTag::new(id).dialog_raw(\"mod:menu\");")]
    pub fn dialog_raw(mut self, dialog: impl Into<String>) -> Self {
        self.values.push(dialog.into());
        self
    }

    /// Add multiple dialog entries to this tag.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogTag::dialogs",
        aliases = ["sand::prelude::DialogTag::dialogs"],
        module = "sand::component",
        kind = "method",
        summary = "Add multiple dialog entries to this tag.",
        context = "Add multiple dialog entries to this tag. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(dialogs = "`dialogs` provides the dialogs added when building multiple dialog entries to this tag."),
        returns = "The `DialogTag` value with the documented change applied to add multiple dialog entries to this tag.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_tag_value: DialogTag, dialogs: Vec<DialogId>) {\n    let updated_dialog_tag = dialog_tag_value.dialogs(dialogs);\n}",
    )]
    pub fn dialogs(mut self, dialogs: impl IntoIterator<Item = DialogId>) -> Self {
        self.values
            .extend(dialogs.into_iter().map(|dialog| dialog.to_string()));
        self
    }

    /// Set whether this tag replaces lower-priority definitions.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogTag::replace",
        aliases = ["sand::prelude::DialogTag::replace"],
        module = "sand::component",
        kind = "method",
        summary = "Set whether this tag replaces lower-priority definitions.",
        context = "Set whether this tag replaces lower-priority definitions. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(replace = "`replace` provides the switch that enables or disables the behavior used to set whether this tag replaces lower-priority definitions."),
        returns = "The `DialogTag` value with the documented change applied to set whether this tag replaces lower-priority definitions.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_tag_value: sand::component::DialogTag, replace: bool)  {\n    let updated_dialog_tag = dialog_tag_value.replace(replace);\n}",
    )]
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
}

// ── DialogBody ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogBody",
    aliases = ["sand::prelude::DialogBody"],
    module = "sand::component",
    summary = "A dialog body element (text, item display, etc.).",
    context = "A dialog body element (text, item display, etc.). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogBody;",
    variants(Item = "Item display body element.", Text = "Plain text body element."),
    variant_fields(Item(height = "`height` optionally provides the height when item display body element.", item = "`item` provides the item when item display body element.", width = "`width` optionally provides the width when item display body element."), Text(text = "`text` provides the text when plain text body element.", width = "`width` optionally provides the width when plain text body element.")),
)]
/// A dialog body element (text, item display, etc.).
#[derive(Debug, Clone)]
pub enum DialogBody {
    /// Plain text body element.
    Text {
        /// `text` provides the text when plain text body element.
        text: Box<DialogText>,
        /// `width` optionally provides the width when plain text body element.
        width: Option<u32>,
    },
    /// Item display body element.
    Item {
        /// `item` provides the item when item display body element.
        item: String,
        /// `width` optionally provides the width when item display body element.
        width: Option<u32>,
        /// `height` optionally provides the height when item display body element.
        height: Option<u32>,
    },
}

impl DialogBody {
    /// Plain text body.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogBody::text",
        aliases = ["sand::prelude::DialogBody::text"],
        module = "sand::component",
        kind = "method",
        summary = "Plain text body.",
        context = "Plain text body. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(content = "`content` sets the player-visible text for plain text body."),
        returns = "A `DialogBody` configured for plain text body.",
        example = "use sand::prelude::*;\n\nfn demonstrate(content: impl Into < sand::component::DialogText >)  {\n    let dialog_body = sand::component::DialogBody::text(content);\n}",
    )]
    pub fn text(content: impl Into<DialogText>) -> Self {
        Self::Text {
            text: Box::new(content.into()),
            width: None,
        }
    }

    /// Plain text body with explicit width.
    ///
    /// `width` must be non-zero — a `0` width is rejected by
    /// [`Dialog::validate`]. There is no vanilla-documented upper bound, so
    /// large values are accepted (raw escape-hatch semantics).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogBody::text_with_width",
        aliases = ["sand::prelude::DialogBody::text_with_width"],
        module = "sand::component",
        kind = "method",
        summary = "Plain text body with explicit width. `width` must be non-zero — a `0` width is rejected by [`Dialog::validate`]. There is no vanilla-documented upper bound, so large values are accepted (raw escape-hatch semantics).",
        context = "Plain text body with explicit width. `width` must be non-zero — a `0` width is rejected by [`Dialog::validate`]. There is no vanilla-documented upper bound, so large values are accepted (raw escape-hatch semantics). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(content = "`content` sets the player-visible text for plain text body with explicit width. `width` must be non-zero — a `0` width is rejected by [`Dialog::validate`]. There is no vanilla-documented upper bound, so large values are accepted (raw escape-hatch semantics).", width = "`width` must be non-zero — a `0` width is rejected by [`Dialog::validate`]. There is no vanilla-documented upper bound, so large values are accepted (raw escape-hatch semantics)."),
        returns = "A `DialogBody` configured for plain text body with explicit width. `width` must be non-zero — a `0` width is rejected by [`Dialog::validate`]. There is no vanilla-documented upper bound, so large values are accepted (raw escape-hatch semantics).",
        example = "use sand::prelude::*;\n\nfn demonstrate(content: impl Into < sand::component::DialogText >, width: u32)  {\n    let dialog_body = sand::component::DialogBody::text_with_width(content, width);\n}",
    )]
    pub fn text_with_width(content: impl Into<DialogText>, width: u32) -> Self {
        Self::Text {
            text: Box::new(content.into()),
            width: Some(width),
        }
    }

    /// Item display body.
    ///
    /// Accepts the canonical typed [`crate::registry::ItemId`].
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogBody::item",
        aliases = ["sand::prelude::DialogBody::item"],
        module = "sand::component",
        kind = "method",
        summary = "Builds an item display body from the canonical typed ItemId.",
        context = "Dialog item references use the same ItemId accepted by other item-facing APIs; unsupported syntax belongs in item_raw.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(item = "The validated item registry ID to display."),
        returns = "A `DialogBody` displaying the selected item.",
        example = "use sand::prelude::*;\nlet body = DialogBody::item(ItemId::minecraft(\"diamond\").unwrap());",
    )]
    pub fn item(item: crate::registry::ItemId) -> Self {
        Self::Item {
            item: item.to_string(),
            width: None,
            height: None,
        }
    }

    /// Builds an item display body from explicit raw item syntax.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogBody::item_raw", aliases = ["sand::prelude::DialogBody::item_raw"], module = "sand::component", kind = "method", summary = "Builds an item display body from explicit raw item syntax.", context = "This escape hatch is for future or modded item syntax that ItemId cannot represent.", minecraft = "Stores the supplied token for validation and serialization as the dialog body's item reference.", use_when = ["Using unsupported future or modded item syntax"], avoid_when = ["A validated ItemId is available"], params(item = "The raw item reference token."), returns = "A dialog item body containing the raw reference.", example = "let body = DialogBody::item_raw(\"mod:future_item\");")]
    pub fn item_raw(item: impl Into<String>) -> Self {
        Self::Item {
            item: item.into(),
            width: None,
            height: None,
        }
    }

    /// Item display body with explicit dimensions.
    ///
    /// `width`/`height` must be non-zero — a `0` dimension is rejected by
    /// [`Dialog::validate`]. There is no vanilla-documented upper bound, so
    /// large values are accepted (raw escape-hatch semantics).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogBody::item_sized",
        aliases = ["sand::prelude::DialogBody::item_sized"],
        module = "sand::component",
        kind = "method",
        summary = "Builds a typed item display body with explicit non-zero dimensions.",
        context = "The item uses the canonical ItemId; dialog validation rejects zero dimensions.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(item = "The validated item registry ID to display.", width = "The requested non-zero display width.", height = "The requested non-zero display height."),
        returns = "A typed item display body with explicit dimensions.",
        example = "use sand::prelude::*;\nlet body = DialogBody::item_sized(ItemId::minecraft(\"diamond\").unwrap(), 32, 32);",
    )]
    pub fn item_sized(item: crate::registry::ItemId, width: u32, height: u32) -> Self {
        Self::Item {
            item: item.to_string(),
            width: Some(width),
            height: Some(height),
        }
    }

    /// Builds a sized item display body from explicit raw item syntax.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogBody::item_sized_raw", aliases = ["sand::prelude::DialogBody::item_sized_raw"], module = "sand::component", kind = "method", summary = "Builds a sized item display body from explicit raw item syntax.", context = "This escape hatch pairs unsupported item syntax with the ordinary dialog dimension validation.", minecraft = "Stores the raw item token and dimensions for dialog validation and serialization.", use_when = ["Using unsupported future or modded item syntax in a sized body"], avoid_when = ["A validated ItemId is available"], params(item = "The raw item reference token.", width = "The requested display width.", height = "The requested display height."), returns = "A sized dialog item body containing the raw reference.", example = "let body = DialogBody::item_sized_raw(\"mod:future_item\", 32, 32);")]
    pub fn item_sized_raw(item: impl Into<String>, width: u32, height: u32) -> Self {
        Self::Item {
            item: item.into(),
            width: Some(width),
            height: Some(height),
        }
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Text { text, width } => {
                let mut v = json!({"type": "minecraft:plain_message", "contents": text.to_json()});
                if let Some(w) = width {
                    v["width"] = json!(w);
                }
                v
            }
            Self::Item {
                item,
                width,
                height,
            } => {
                let mut v = json!({"type": "minecraft:item", "item": item});
                if let Some(w) = width {
                    v["width"] = json!(w);
                }
                if let Some(h) = height {
                    v["height"] = json!(h);
                }
                v
            }
        }
    }
}

// ── DialogAction ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogAction",
    aliases = ["sand::prelude::DialogAction"],
    module = "sand::component",
    summary = "An action associated with a dialog button.",
    context = "An action associated with a dialog button. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogAction;",
    variants(Callback = "Run a datapack function through Sand's survival-friendly callback dispatcher.", Close = "Close the current dialog.", OpenDialog = "Open another dialog.", OpenUrl = "Open a URL (where server-controlled links are permitted).", RunCommand = "Run a raw command when the button is pressed. This is the explicit raw escape hatch — the command string is never validated. Prefer [`DialogAction::run_function`] for datapack function calls.", RunFunction = "Run a datapack function when the button is pressed.", SuggestCommand = "Fill the chat bar with a command suggestion."),
    variant_fields(Callback = ["Run a datapack function through Sand's survival-friendly callback dispatcher."], OpenDialog = ["Open another dialog."], OpenUrl = ["Open a URL (where server-controlled links are permitted)."], RunCommand = ["Run a raw command when the button is pressed. This is the explicit raw escape hatch — the command string is never validated. Prefer [`DialogAction::run_function`] for datapack function calls."], RunFunction = ["Run a datapack function when the button is pressed."], SuggestCommand = ["Fill the chat bar with a command suggestion."]),
)]
/// An action associated with a dialog button.
#[derive(Debug, Clone)]
pub enum DialogAction {
    /// Run a raw command when the button is pressed.
    ///
    /// This is the explicit raw escape hatch — the command string is never
    /// validated. Prefer [`DialogAction::run_function`] for datapack
    /// function calls.
    RunCommand(
        #[doc = "Run a raw command when the button is pressed. This is the explicit raw escape hatch — the command string is never validated. Prefer [`DialogAction::run_function`] for datapack function calls."]
         String,
    ),
    /// Run a datapack function when the button is pressed.
    ///
    /// The raw function path (not yet formatted into a command) — validated
    /// by [`Dialog::validate`] before it can reach generated output.
    RunFunction(#[doc = "Run a datapack function when the button is pressed."] String),
    /// Fill the chat bar with a command suggestion.
    SuggestCommand(#[doc = "Fill the chat bar with a command suggestion."] String),
    /// Open a URL (where server-controlled links are permitted).
    OpenUrl(#[doc = "Open a URL (where server-controlled links are permitted)."] String),
    /// Open another dialog.
    OpenDialog(#[doc = "Open another dialog."] String),
    /// Close the current dialog.
    Close,
    /// Run a datapack function through Sand's survival-friendly callback
    /// dispatcher.
    ///
    /// Callback IDs are assigned when the containing dialog is serialized
    /// (see `DialogAction::to_json`) so cached or otherwise prebuilt
    /// dialogs participate in each export's callback lifecycle. The raw
    /// function path is validated by [`Dialog::validate`] before it can
    /// reach generated `__sand_dialog_tick` output.
    Callback(
        #[doc = "Run a datapack function through Sand's survival-friendly callback dispatcher."]
        String,
    ),
}

impl DialogAction {
    /// Sets the Minecraft run command property on this typed dialog action definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::run_command",
        aliases = ["sand::prelude::DialogAction::run_command"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft run command property on this typed dialog action definition and returns the updated builder.",
        context = "Sets the Minecraft run command property on this typed dialog action definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cmd = "`cmd` provides the cmd applied when setting the Minecraft run command property on this typed dialog action definition and returns the updated builder."),
        returns = "Sets the Minecraft run command property on this typed dialog action definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cmd: impl Into < String >)  {\n    let dialog_action = sand::component::DialogAction::run_command(cmd);\n}",
    )]
    pub fn run_command(cmd: impl Into<String>) -> Self {
        Self::RunCommand(cmd.into())
    }

    /// Run a datapack function when the button is pressed.
    ///
    /// Prefer this over [`run_command`](DialogAction::run_command) for datapack
    /// functions. It accepts registered function items and canonical
    /// [`FunctionId`](crate::FunctionId) handles.
    ///
    /// ```
    /// use sand_components::dialog::DialogAction;
    /// use sand_components::{FunctionId, ResourceLocation};
    ///
    /// let action = DialogAction::run_function(
    ///     FunctionId::custom(ResourceLocation::new("example", "start").unwrap())
    /// );
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::run_function",
        aliases = ["sand::prelude::DialogAction::run_function"],
        module = "sand::component",
        kind = "method",
        summary = "Run a datapack function when the button is pressed.",
        context = "Run a datapack function when the button is pressed. Prefer this over [`run_command`](DialogAction::run_command) for datapack functions. It accepts registered function items and canonical FunctionId handles.",
        minecraft = "Serializes the canonical function identity into a run-function dialog action.",
        use_when = ["Running a registered or typed external function from a dialog"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to run a datapack function when the button is pressed."),
        returns = "A `DialogAction` that runs a datapack function when the button is pressed.",
        example = "use sand::prelude::*;\nlet action = DialogAction::run_function(FunctionId::custom(\"example:start\".parse().unwrap()));",
    )]
    pub fn run_function(id: impl FunctionRef) -> Self {
        Self::RunFunction(id.function_id().to_string())
    }

    /// Runs an explicitly raw function identifier when Sand cannot model it.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogAction::run_function_raw", aliases = ["sand::prelude::DialogAction::run_function_raw"], module = "sand::component", kind = "method", summary = "Runs an explicitly raw function token from a dialog action.", context = "Advanced escape hatch for unsupported function syntax.", minecraft = "Serializes a run-function dialog action using the supplied token.", use_when = ["Using future or modded function syntax"], avoid_when = ["A FunctionId or registered #[function] item is available"], params(id = "The unchecked function token."), returns = "A raw run-function dialog action.", example = "let action = DialogAction::run_function_raw(\"mod:future\");")]
    pub fn run_function_raw(id: impl Into<String>) -> Self {
        Self::RunFunction(id.into())
    }

    /// Survival-friendly callback — runs a datapack function via a scoreboard trigger.
    ///
    /// Use this instead of [`run_function`](DialogAction::run_function) for player-facing
    /// dialog buttons. `/trigger` is available to all players in survival mode without
    /// requiring operator permissions.
    ///
    /// **How it works:**
    /// 1. Sand assigns the callback a stable integer ID.
    /// 2. The button action runs `/trigger sand.dialog set <id>`.
    /// 3. Sand generates a tick function that detects players with matching scores
    ///    and calls the target function as that player.
    /// 4. Load and tick infrastructure is generated automatically — no manual
    ///    `scoreboard objectives add` or tick wiring needed.
    ///
    /// ```rust,ignore
    /// DialogButton::new(Text::new("Enhanced Cells"))
    ///     .tooltip(Text::new("Gain an extra row of hearts"))
    ///     .action(DialogAction::callback(grant_enhanced_cells))
    /// ```
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::callback",
        aliases = ["sand::prelude::DialogAction::callback"],
        module = "sand::component",
        kind = "method",
        summary = "Survival-friendly callback — runs a datapack function via a scoreboard trigger.",
        context = "Survival-friendly callback — runs a datapack function via a scoreboard trigger. Use this instead of [`run_function`](DialogAction::run_function) for player-facing dialog buttons. `/trigger` is available to all players in survival mode without requiring operator permissions. How it works: 1. Sand assigns the callback a stable integer ID. 2. The button action runs `/trigger sand.dialog set <id>`. 3. Sand generates a tick function that detects players with matching scores and calls the target function as that player. 4. Load and tick infrastructure is generated automatically — no manual `scoreboard objectives add` or tick wiring needed.",
        minecraft = "How it works: 1. Sand assigns the callback a stable integer ID. 2. The button action runs `/trigger sand.dialog set <id>`. 3. Sand generates a tick function that detects players with matching scores and calls the target function as that player. 4. Load and tick infrastructure is generated automatically — no manual `scoreboard objectives add` or tick wiring needed.",
        use_when = ["Use this instead of [`run_function`](DialogAction::run_function) for player-facing dialog buttons. `/trigger` is available to all players in survival mode without requiring operator permissions."],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to survival-friendly callback — runs a datapack function via a scoreboard trigger."),
        returns = "A `DialogAction` for a survival-friendly callback — runs a datapack function via a scoreboard trigger.",
        example = "DialogButton::new(Text::new(\"Enhanced Cells\"))\n.tooltip(Text::new(\"Gain an extra row of hearts\"))\n.action(DialogAction::callback(grant_enhanced_cells))",
    )]
    pub fn callback(id: impl FunctionRef) -> Self {
        Self::Callback(id.function_id().to_string())
    }

    /// Registers an explicitly raw callback function identifier.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogAction::callback_raw", aliases = ["sand::prelude::DialogAction::callback_raw"], module = "sand::component", kind = "method", summary = "Registers a dialog callback through an explicitly raw function token.", context = "Advanced escape hatch for unsupported callback syntax.", minecraft = "Generates callback dispatch using the supplied function token.", use_when = ["Using future or modded function syntax"], avoid_when = ["A FunctionId or registered #[function] item is available"], params(id = "The unchecked function token."), returns = "A raw callback dialog action.", example = "let action = DialogAction::callback_raw(\"mod:future\");")]
    pub fn callback_raw(id: impl Into<String>) -> Self {
        Self::Callback(id.into())
    }

    /// Sets the Minecraft suggest command property on this typed dialog action definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::suggest_command",
        aliases = ["sand::prelude::DialogAction::suggest_command"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft suggest command property on this typed dialog action definition and returns the updated builder.",
        context = "Sets the Minecraft suggest command property on this typed dialog action definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(cmd = "`cmd` provides the cmd applied when setting the Minecraft suggest command property on this typed dialog action definition and returns the updated builder."),
        returns = "Sets the Minecraft suggest command property on this typed dialog action definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(cmd: impl Into < String >)  {\n    let dialog_action = sand::component::DialogAction::suggest_command(cmd);\n}",
    )]
    pub fn suggest_command(cmd: impl Into<String>) -> Self {
        Self::SuggestCommand(cmd.into())
    }
    /// Sets the Minecraft open url property on this typed dialog action definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::open_url",
        aliases = ["sand::prelude::DialogAction::open_url"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft open url property on this typed dialog action definition and returns the updated builder.",
        context = "Sets the Minecraft open url property on this typed dialog action definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(url = "`url` provides the url applied when setting the Minecraft open url property on this typed dialog action definition and returns the updated builder."),
        returns = "Sets the Minecraft open url property on this typed dialog action definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(url: impl Into < String >)  {\n    let dialog_action = sand::component::DialogAction::open_url(url);\n}",
    )]
    pub fn open_url(url: impl Into<String>) -> Self {
        Self::OpenUrl(url.into())
    }
    /// Sets the Minecraft open dialog property on this typed dialog action definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::open_dialog",
        aliases = ["sand::prelude::DialogAction::open_dialog"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft open dialog property on this typed dialog action definition and returns the updated builder.",
        context = "Sets the Minecraft open dialog property on this typed dialog action definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(dialog = "`dialog` provides the dialog applied when setting the Minecraft open dialog property on this typed dialog action definition and returns the updated builder."),
        returns = "Sets the Minecraft open dialog property on this typed dialog action definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog: DialogId) {\n    let dialog_action = DialogAction::open_dialog(dialog);\n}",
    )]
    pub fn open_dialog(dialog: DialogId) -> Self {
        Self::OpenDialog(dialog.to_string())
    }

    /// Opens an explicitly raw dialog reference.
    #[sand_macros::api(registry = sand_api_contract, path = "sand::component::DialogAction::open_dialog_raw", aliases = ["sand::prelude::DialogAction::open_dialog_raw"], module = "sand::component", kind = "method", summary = "Opens a dialog through an explicitly raw resource token.", context = "Advanced escape hatch for unsupported dialog syntax.", minecraft = "Serializes an open-dialog action using the supplied token.", use_when = ["Using future or modded dialog syntax"], avoid_when = ["A validated DialogId is available"], params(dialog = "The unchecked dialog token."), returns = "A raw open-dialog action.", example = "let action = DialogAction::open_dialog_raw(\"mod:menu\");")]
    pub fn open_dialog_raw(dialog: impl Into<String>) -> Self {
        Self::OpenDialog(dialog.into())
    }
    /// Sets the Minecraft close property on this typed dialog action definition and returns the updated builder.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogAction::close",
        aliases = ["sand::prelude::DialogAction::close"],
        module = "sand::component",
        kind = "method",
        summary = "Sets the Minecraft close property on this typed dialog action definition and returns the updated builder.",
        context = "Sets the Minecraft close property on this typed dialog action definition and returns the updated builder. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "Sets the Minecraft close property on this typed dialog action definition and returns the updated builder.",
        example = "use sand::prelude::*;\n\nfn demonstrate()  {\n    let dialog_action = sand::component::DialogAction::close();\n}",
    )]
    pub fn close() -> Self {
        Self::Close
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::RunCommand(c) => json!({"type": "minecraft:run_command", "command": c}),
            Self::RunFunction(path) => {
                json!({"type": "minecraft:run_command", "command": format!("/function {path}")})
            }
            Self::SuggestCommand(c) => json!({"type": "minecraft:suggest_command", "command": c}),
            Self::OpenUrl(u) => json!({"type": "minecraft:open_url", "url": u}),
            Self::OpenDialog(d) => json!({"type": "minecraft:open_dialog", "dialog": d}),
            Self::Close => json!({"type": "minecraft:close"}),
            Self::Callback(path) => {
                let trigger_id = register_dialog_callback(path.clone());
                json!({
                    "type": "minecraft:run_command",
                    "command": format!("/trigger {SAND_DIALOG_TRIGGER} set {trigger_id}"),
                })
            }
        }
    }

    /// Validate this action's raw resource references.
    ///
    /// `RunCommand` is the explicit raw escape hatch and is never validated.
    /// `RunFunction`/`Callback` targets and `OpenDialog` targets must be
    /// well-formed `namespace:path` references.
    fn validate(&self, path: &str) -> crate::error::Result<()> {
        let placeholder =
            ResourceLocation::new("sand", "dialog_action").expect("static placeholder is valid");
        let build_error = |message: String| crate::error::SandError::ComponentValidation {
            location: placeholder.clone(),
            kind: "dialog".to_string(),
            field: path.to_string(),
            message,
        };
        match self {
            Self::RunFunction(target) | Self::Callback(target) => validate_resource_ref(target)
                .map_err(|message| {
                    build_error(format!("invalid function reference `{target}`: {message}"))
                }),
            Self::OpenDialog(target) => validate_resource_ref(target).map_err(|message| {
                build_error(format!("invalid dialog reference `{target}`: {message}"))
            }),
            Self::RunCommand(_) | Self::SuggestCommand(_) | Self::OpenUrl(_) | Self::Close => {
                Ok(())
            }
        }
    }
}

// ── DialogButton ──────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogButton",
    aliases = ["sand::prelude::DialogButton"],
    module = "sand::component",
    summary = "A button displayed in a dialog.",
    context = "A button displayed in a dialog. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogButton;",
)]
/// A button displayed in a dialog.
#[derive(Debug, Clone)]
pub struct DialogButton {
    label: DialogText,
    action: Option<DialogAction>,
    tooltip: Option<DialogText>,
    width: Option<u32>,
}

impl DialogButton {
    /// Create a button with the given label text.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogButton::new",
        aliases = ["sand::prelude::DialogButton::new"],
        module = "sand::component",
        kind = "method",
        summary = "Create a button with the given label text.",
        context = "Create a button with the given label text. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(label = "`label` is used when creating a button with the given label text."),
        returns = "A `DialogButton` representing a button with the given label text.",
        example = "use sand::prelude::*;\n\nfn demonstrate(label: impl Into < sand::component::DialogText >)  {\n    let dialog_button = sand::component::DialogButton::new(label);\n}",
    )]
    pub fn new(label: impl Into<DialogText>) -> Self {
        Self {
            label: label.into(),
            action: None,
            tooltip: None,
            width: None,
        }
    }

    /// Attach an action to this button.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogButton::action",
        aliases = ["sand::prelude::DialogButton::action"],
        module = "sand::component",
        kind = "method",
        summary = "Attach an action to this button.",
        context = "Attach an action to this button. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(action = "`action` is used to attach an action to this button."),
        returns = "The `DialogButton` value with the documented change applied to attach an action to this button.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_button_value: sand::component::DialogButton, action: sand::component::DialogAction)  {\n    let updated_dialog_button = dialog_button_value.action(action);\n}",
    )]
    pub fn action(mut self, action: DialogAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Attach a tooltip shown when hovering over the button.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogButton::tooltip",
        aliases = ["sand::prelude::DialogButton::tooltip"],
        module = "sand::component",
        kind = "method",
        summary = "Attach a tooltip shown when hovering over the button.",
        context = "Attach a tooltip shown when hovering over the button. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(tip = "`tip` is used to attach a tooltip shown when hovering over the button."),
        returns = "The `DialogButton` value with the documented change applied to attach a tooltip shown when hovering over the button.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_button_value: sand::component::DialogButton, tip: impl Into < sand::component::DialogText >)  {\n    let updated_dialog_button = dialog_button_value.tooltip(tip);\n}",
    )]
    pub fn tooltip(mut self, tip: impl Into<DialogText>) -> Self {
        self.tooltip = Some(tip.into());
        self
    }

    /// Set the button width in pixels.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::DialogButton::width",
        aliases = ["sand::prelude::DialogButton::width"],
        module = "sand::component",
        kind = "method",
        summary = "Set the button width in pixels.",
        context = "Set the button width in pixels. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(w = "`w` provides the w applied when setting the button width in pixels."),
        returns = "The `DialogButton` value with the documented change applied to set the button width in pixels.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_button_value: sand::component::DialogButton, w: u32)  {\n    let updated_dialog_button = dialog_button_value.width(w);\n}",
    )]
    pub fn width(mut self, w: u32) -> Self {
        self.width = Some(w);
        self
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut v = json!({"label": self.label.to_json()});
        if let Some(a) = &self.action {
            v["action"] = a.to_json();
        }
        if let Some(t) = &self.tooltip {
            v["tooltip"] = t.to_json();
        }
        if let Some(w) = self.width {
            v["width"] = json!(w);
        }
        v
    }
}

// ── DialogKind ────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::DialogKind",
    aliases = ["sand::prelude::DialogKind"],
    module = "sand::component",
    summary = "The dialog variant (notice, confirmation, multi-action).",
    context = "The dialog variant (notice, confirmation, multi-action). This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
    minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::DialogKind;",
    variants(Confirmation = "A dialog with confirm / cancel buttons.", MultiAction = "A dialog with multiple custom action buttons.", Notice = "A simple informational dialog with one or more dismiss buttons."),
)]
/// The dialog variant (notice, confirmation, multi-action).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    /// A simple informational dialog with one or more dismiss buttons.
    Notice,
    /// A dialog with confirm / cancel buttons.
    Confirmation,
    /// A dialog with multiple custom action buttons.
    MultiAction,
}

impl DialogKind {
    fn type_str(&self) -> &'static str {
        match self {
            Self::Notice => "minecraft:notice",
            Self::Confirmation => "minecraft:confirmation",
            Self::MultiAction => "minecraft:multi_action",
        }
    }

    fn button_key(&self) -> &'static str {
        match self {
            Self::MultiAction => "actions",
            Self::Notice | Self::Confirmation => "buttons",
        }
    }
}

// ── Dialog ────────────────────────────────────────────────────────────────────

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::component::Dialog",
    aliases = ["sand::prelude::Dialog"],
    module = "sand::component",
    summary = "A typed dialog datapack component builder for Sand's Minecraft 26+ baseline.",
    context = "Dialogs live at `data/<namespace>/dialog/<path>.json`. Exact-profile export validation checks this resource against the selected verified 26.x schema.",
    minecraft = "Serializes a data-driven dialog resource under `data/<namespace>/dialog/<path>.json`.",
    use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
    avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
    example = "use sand::component::Dialog;",
    fields(id = "The validated resource location for this dialog (e.g. `\"example:welcome\"`)."),
)]
/// A typed dialog datapack component builder.
///
/// Dialogs live at `data/<namespace>/dialog/<path>.json` and require
/// Minecraft 26+. Export validation checks `VersionFeature::Dialogs`
/// before generating dialog output.
///
/// # Example
/// ```
/// use sand_components::dialog::{Dialog, DialogBody, DialogButton, DialogAction};
/// use sand_commands::Text;
///
/// let d = Dialog::notice_local("welcome")
///     .title(Text::new("Welcome!").gold())
///     .body(DialogBody::text(Text::new("Choose what to do next.")))
///     .button(
///         DialogButton::new(Text::new("Start").green())
///             .action(DialogAction::close())
///     );
///
/// let json = d.to_json();
/// assert!(json["type"].as_str().unwrap().contains("notice"));
/// assert!(json["title"]["text"].as_str().unwrap() == "Welcome!");
/// ```
#[derive(Debug, Clone)]
pub struct Dialog {
    /// The validated resource location for this dialog (e.g. `"example:welcome"`).
    pub id: DialogId,
    kind: DialogKind,
    title: Option<DialogText>,
    body: Vec<DialogBody>,
    buttons: Vec<DialogButton>,
    pause: bool,
    external_title: bool,
}

impl Dialog {
    /// Create a notice dialog — informational, dismissible.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::notice",
        aliases = ["sand::prelude::Dialog::notice"],
        module = "sand::component",
        kind = "method",
        summary = "Create a notice dialog — informational, dismissible.",
        context = "Create a notice dialog — informational, dismissible. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to create a notice dialog — informational, dismissible."),
        returns = "A `Dialog` representing a notice dialog — informational, dismissible.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::resource_ref::DialogId)  {\n    let dialog = sand::component::Dialog::notice(id);\n}",
    )]
    pub fn notice(id: DialogId) -> Self {
        Self::new_with_kind(id, DialogKind::Notice)
    }

    /// Create a local notice dialog whose namespace is resolved during export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::notice_local",
        aliases = ["sand::prelude::Dialog::notice_local"],
        module = "sand::component",
        kind = "method",
        summary = "Create a local notice dialog whose namespace is resolved during export.",
        context = "Create a local notice dialog whose namespace is resolved during export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(path = "`path` provides the typed resource identifier or location used to create a local notice dialog whose namespace is resolved during export."),
        returns = "A `Dialog` representing a local notice dialog whose namespace is resolved during export.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl AsRef < str >)  {\n    let dialog = sand::component::Dialog::notice_local(path);\n}",
    )]
    pub fn notice_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::Notice)
    }

    /// Create a confirmation dialog — confirm / cancel.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::confirmation",
        aliases = ["sand::prelude::Dialog::confirmation"],
        module = "sand::component",
        kind = "method",
        summary = "Create a confirmation dialog — confirm / cancel.",
        context = "Create a confirmation dialog — confirm / cancel. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to create a confirmation dialog — confirm / cancel."),
        returns = "A `Dialog` representing a confirmation dialog — confirm / cancel.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::resource_ref::DialogId)  {\n    let dialog = sand::component::Dialog::confirmation(id);\n}",
    )]
    pub fn confirmation(id: DialogId) -> Self {
        Self::new_with_kind(id, DialogKind::Confirmation)
    }

    /// Create a local confirmation dialog whose namespace is resolved during export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::confirmation_local",
        aliases = ["sand::prelude::Dialog::confirmation_local"],
        module = "sand::component",
        kind = "method",
        summary = "Create a local confirmation dialog whose namespace is resolved during export.",
        context = "Create a local confirmation dialog whose namespace is resolved during export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(path = "`path` provides the typed resource identifier or location used to create a local confirmation dialog whose namespace is resolved during export."),
        returns = "A `Dialog` representing a local confirmation dialog whose namespace is resolved during export.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl AsRef < str >)  {\n    let dialog = sand::component::Dialog::confirmation_local(path);\n}",
    )]
    pub fn confirmation_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::Confirmation)
    }

    /// Create a multi-action dialog — multiple custom buttons.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::multi_action",
        aliases = ["sand::prelude::Dialog::multi_action"],
        module = "sand::component",
        kind = "method",
        summary = "Create a multi-action dialog — multiple custom buttons.",
        context = "Create a multi-action dialog — multiple custom buttons. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(id = "`id` provides the typed resource identifier or location used to create a multi-action dialog — multiple custom buttons."),
        returns = "A `Dialog` representing a multi-action dialog — multiple custom buttons.",
        example = "use sand::prelude::*;\n\nfn demonstrate(id: sand::resource_ref::DialogId)  {\n    let dialog = sand::component::Dialog::multi_action(id);\n}",
    )]
    pub fn multi_action(id: DialogId) -> Self {
        Self::new_with_kind(id, DialogKind::MultiAction)
    }

    /// Create a local multi-action dialog whose namespace is resolved during export.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::multi_action_local",
        aliases = ["sand::prelude::Dialog::multi_action_local"],
        module = "sand::component",
        kind = "method",
        summary = "Create a local multi-action dialog whose namespace is resolved during export.",
        context = "Create a local multi-action dialog whose namespace is resolved during export. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(path = "`path` provides the typed resource identifier or location used to create a local multi-action dialog whose namespace is resolved during export."),
        returns = "A `Dialog` representing a local multi-action dialog whose namespace is resolved during export.",
        example = "use sand::prelude::*;\n\nfn demonstrate(path: impl AsRef < str >)  {\n    let dialog = sand::component::Dialog::multi_action_local(path);\n}",
    )]
    pub fn multi_action_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::MultiAction)
    }

    fn new_with_kind(id: DialogId, kind: DialogKind) -> Self {
        Self {
            id,
            kind,
            title: None,
            body: vec![],
            buttons: vec![],
            pause: false,
            external_title: false,
        }
    }

    /// Set the dialog title.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::title",
        aliases = ["sand::prelude::Dialog::title"],
        module = "sand::component",
        kind = "method",
        summary = "Set the dialog title.",
        context = "Set the dialog title. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(text = "`text` provides the author-visible text applied when setting the dialog title."),
        returns = "The `Dialog` value with the documented change applied to set the dialog title.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: sand::component::Dialog, text: impl Into < sand::component::DialogText >)  {\n    let updated_dialog = dialog_value.title(text);\n}",
    )]
    pub fn title(mut self, text: impl Into<DialogText>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Append a body element.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::body",
        aliases = ["sand::prelude::Dialog::body"],
        module = "sand::component",
        kind = "method",
        summary = "Append a body element.",
        context = "Append a body element. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(body = "`body` provides the body appended when building a body element."),
        returns = "The `Dialog` value with the documented change applied to append a body element.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: sand::component::Dialog, body: sand::component::DialogBody)  {\n    let updated_dialog = dialog_value.body(body);\n}",
    )]
    pub fn body(mut self, body: DialogBody) -> Self {
        self.body.push(body);
        self
    }

    /// Append a button.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::button",
        aliases = ["sand::prelude::Dialog::button"],
        module = "sand::component",
        kind = "method",
        summary = "Append a button.",
        context = "Append a button. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(btn = "`btn` provides the btn appended when building a button."),
        returns = "The `Dialog` value with the documented change applied to append a button.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: sand::component::Dialog, btn: sand::component::DialogButton)  {\n    let updated_dialog = dialog_value.button(btn);\n}",
    )]
    pub fn button(mut self, btn: DialogButton) -> Self {
        self.buttons.push(btn);
        self
    }

    /// Whether this dialog pauses the game in single-player.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::pause",
        aliases = ["sand::prelude::Dialog::pause"],
        module = "sand::component",
        kind = "method",
        summary = "Whether this dialog pauses the game in single-player.",
        context = "Whether this dialog pauses the game in single-player. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to determine whether this dialog pauses the game in single-player."),
        returns = "The `Dialog` value with the documented change applied to determine whether this dialog pauses the game in single-player.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: sand::component::Dialog, v: bool)  {\n    let updated_dialog = dialog_value.pause(v);\n}",
    )]
    pub fn pause(mut self, v: bool) -> Self {
        self.pause = v;
        self
    }

    /// Whether the title is rendered outside the dialog frame.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::external_title",
        aliases = ["sand::prelude::Dialog::external_title"],
        module = "sand::component",
        kind = "method",
        summary = "Whether the title is rendered outside the dialog frame.",
        context = "Whether the title is rendered outside the dialog frame. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        params(v = "`v` provides the switch that enables or disables the behavior used to determine whether the title is rendered outside the dialog frame."),
        returns = "The `Dialog` value with the documented change applied to determine whether the title is rendered outside the dialog frame.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: sand::component::Dialog, v: bool)  {\n    let updated_dialog = dialog_value.external_title(v);\n}",
    )]
    pub fn external_title(mut self, v: bool) -> Self {
        self.external_title = v;
        self
    }

    /// Serialize to the datapack JSON format.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::to_json",
        aliases = ["sand::prelude::Dialog::to_json"],
        module = "sand::component",
        kind = "method",
        summary = "Serialize to the datapack JSON format.",
        context = "Serialize to the datapack JSON format. This semantic component model describes a datapack resource or gameplay value; JSON serialization and exporter bookkeeping remain implementation details.",
        minecraft = "The value serializes to the matching version-aware Minecraft datapack JSON schema when the project is exported.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "The `Value` value produced to serialize to the datapack JSON format.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: &sand::component::Dialog)  {\n    let to_json = dialog_value.to_json();\n}",
    )]
    pub fn to_json(&self) -> Value {
        let mut v = json!({"type": self.kind.type_str()});
        if let Some(t) = &self.title {
            v["title"] = t.to_json();
        }
        if !self.body.is_empty() {
            v["body"] = json!(self.body.iter().map(|b| b.to_json()).collect::<Vec<_>>());
        }
        if !self.buttons.is_empty() {
            let key = self.kind.button_key();
            v[key] = json!(self.buttons.iter().map(|b| b.to_json()).collect::<Vec<_>>());
        }
        if self.pause {
            v["pause"] = json!(true);
        }
        if self.external_title {
            v["external_title"] = json!(true);
        }
        v
    }

    /// The resource path for this dialog within the datapack.
    ///
    /// For `"example:welcome"` returns `"example/dialog/welcome.json"`.
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::component::Dialog::resource_path",
        aliases = ["sand::prelude::Dialog::resource_path"],
        module = "sand::component",
        kind = "method",
        summary = "The resource path for this dialog within the datapack.",
        context = "The resource path for this dialog within the datapack. For `\"example:welcome\"` returns `\"example/dialog/welcome.json\"`.",
        minecraft = "For `\"example:welcome\"` returns `\"example/dialog/welcome.json\"`.",
        use_when = ["Defining a typed advancement, recipe, loot table, worldgen resource, item property, or related datapack component"],
        avoid_when = ["Injecting unchecked JSON when the typed schema can represent the resource"],
        returns = "For `\"example:welcome\"` returns `\"example/dialog/welcome.json\"`.",
        example = "use sand::prelude::*;\n\nfn demonstrate(dialog_value: &sand::component::Dialog)  {\n    let resource_path = dialog_value.resource_path();\n}",
    )]
    pub fn resource_path(&self) -> String {
        let location = self.id.as_resource_location();
        if location.namespace() == SAND_LOCAL_NS {
            format!("dialog/{}.json", location.path())
        } else {
            format!("{}/dialog/{}.json", location.namespace(), location.path())
        }
    }
}

impl DatapackComponent for Dialog {
    fn resource_location(&self) -> &ResourceLocation {
        self.id.as_resource_location()
    }

    fn to_json(&self) -> Value {
        Dialog::to_json(self)
    }

    fn validate(&self) -> crate::error::Result<()> {
        let map_error = |error: crate::error::SandError| match error {
            crate::error::SandError::ComponentValidation { field, message, .. } => {
                crate::error::SandError::ComponentValidation {
                    location: self.id.as_resource_location().clone(),
                    kind: "dialog".to_string(),
                    field,
                    message,
                }
            }
            other => other,
        };
        let field_error = |field: &str, message: String| {
            map_error(crate::error::SandError::ComponentValidation {
                location: self.id.as_resource_location().clone(),
                kind: "dialog".to_string(),
                field: field.to_string(),
                message,
            })
        };

        if let Some(title) = &self.title {
            title.validate("title").map_err(map_error)?;
        }

        for (index, body) in self.body.iter().enumerate() {
            match body {
                DialogBody::Text { text, width } => {
                    text.validate(&format!("body[{index}].text"))
                        .map_err(map_error)?;
                    if *width == Some(0) {
                        return Err(field_error(
                            &format!("body[{index}].width"),
                            "dimension must be greater than zero".to_string(),
                        ));
                    }
                }
                DialogBody::Item {
                    item,
                    width,
                    height,
                } => {
                    validate_resource_ref(item).map_err(|message| {
                        field_error(
                            &format!("body[{index}].item"),
                            format!("invalid item reference `{item}`: {message}"),
                        )
                    })?;
                    if *width == Some(0) {
                        return Err(field_error(
                            &format!("body[{index}].width"),
                            "dimension must be greater than zero".to_string(),
                        ));
                    }
                    if *height == Some(0) {
                        return Err(field_error(
                            &format!("body[{index}].height"),
                            "dimension must be greater than zero".to_string(),
                        ));
                    }
                }
            }
        }

        for (index, button) in self.buttons.iter().enumerate() {
            button
                .label
                .validate(&format!("buttons[{index}].label"))
                .map_err(map_error)?;
            if let Some(tooltip) = &button.tooltip {
                tooltip
                    .validate(&format!("buttons[{index}].tooltip"))
                    .map_err(map_error)?;
            }
            if button.width == Some(0) {
                return Err(field_error(
                    &format!("buttons[{index}].width"),
                    "dimension must be greater than zero".to_string(),
                ));
            }
            if let Some(action) = &button.action {
                action
                    .validate(&format!("buttons[{index}].action"))
                    .map_err(map_error)?;
            }
        }

        match self.kind {
            DialogKind::MultiAction => {
                if self.buttons.is_empty() {
                    return Err(field_error(
                        "actions",
                        "multi_action dialogs require at least one action".to_string(),
                    ));
                }
            }
            DialogKind::Notice | DialogKind::Confirmation => {
                if self.buttons.is_empty() {
                    return Err(field_error(
                        "buttons",
                        format!(
                            "{} dialogs require at least one button",
                            self.kind.type_str()
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    fn required_features(&self) -> &'static [sand_version::ComponentFeature] {
        &[sand_version::ComponentFeature::Dialogs]
    }

    fn component_dir(&self) -> &'static str {
        "dialog"
    }
}

impl DatapackComponent for DialogTag {
    fn resource_location(&self) -> &ResourceLocation {
        &self.location
    }

    fn to_json(&self) -> Value {
        json!({
            "replace": self.replace,
            "values": self.values,
        })
    }

    fn validate(&self) -> crate::error::Result<()> {
        for (index, value) in self.values.iter().enumerate() {
            validate_resource_ref(value).map_err(|message| {
                crate::error::SandError::ComponentValidation {
                    location: self.location.clone(),
                    kind: "dialog_tag".to_string(),
                    field: format!("values[{index}]"),
                    message: format!("invalid dialog reference `{value}`: {message}"),
                }
            })?;
        }
        Ok(())
    }

    fn component_dir(&self) -> &'static str {
        "tags"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FunctionId;

    fn dialog_id(value: &str) -> DialogId {
        value.parse().expect("test dialog resource location")
    }

    fn function_id(value: &str) -> FunctionId {
        value.parse().expect("test function resource location")
    }

    #[test]
    fn nested_text_validation_reports_dialog_path() {
        let dialog = Dialog::notice(dialog_id("example:invalid_text"))
            .title(Text::new("Bad").color_hex("#12FG00"));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("SAND-TEXT-COLOR"), "{error}");
        assert!(error.contains("title.style.color"), "{error}");
        assert!(error.contains("example:invalid_text"), "{error}");
    }

    #[test]
    fn notice_dialog_json() {
        let d = Dialog::notice(dialog_id("example:welcome"))
            .title("Welcome!")
            .body(DialogBody::text("Choose an option."))
            .button(
                DialogButton::new("Start")
                    .action(DialogAction::run_function(function_id("example:start"))),
            );
        let json = d.to_json();
        assert!(
            json["type"].as_str().unwrap().contains("notice"),
            "got: {json}"
        );
        assert_eq!(json["title"]["text"].as_str().unwrap(), "Welcome!");
        assert!(json["body"].is_array());
        assert!(json["buttons"].is_array());
    }

    /// Escape hatch: `DialogAction::run_command` accepts any raw command string,
    /// including non-function commands like `/say`. Use `run_function` for
    /// datapack function calls; use `run_command` only when there is no typed API.
    #[test]
    fn button_action_run_command_escape_hatch() {
        let btn = DialogButton::new("OK").action(DialogAction::run_command("/say hi"));
        let json = btn.to_json();
        assert_eq!(json["label"]["text"].as_str().unwrap(), "OK");
        assert!(
            json["action"]["command"]
                .as_str()
                .unwrap()
                .contains("/say hi")
        );
    }

    #[test]
    fn resource_path_namespaced() {
        let d = Dialog::notice(dialog_id("example:welcome"));
        assert_eq!(d.resource_path(), "example/dialog/welcome.json");
    }

    #[test]
    fn resource_path_no_namespace() {
        let d = Dialog::notice_local("welcome");
        assert_eq!(d.resource_path(), "dialog/welcome.json");
    }

    #[test]
    fn dialog_component_metadata() {
        let d = Dialog::notice_local("welcome");
        assert_eq!(d.resource_location().namespace(), SAND_LOCAL_NS);
        assert_eq!(d.resource_location().path(), "welcome");
        assert_eq!(d.component_dir(), "dialog");
        assert_eq!(d.file_extension(), "json");
    }

    #[test]
    fn dialog_tag_helpers_emit_well_known_vanilla_paths() {
        let pause = DialogTag::pause_screen_additions().dialog(dialog_id("example:welcome"));
        assert_eq!(pause.resource_location().namespace(), "minecraft");
        assert_eq!(
            pause.resource_location().path(),
            "dialog/pause_screen_additions"
        );
        assert_eq!(pause.component_dir(), "tags");
        assert_eq!(
            pause.to_json(),
            json!({
                "replace": false,
                "values": ["example:welcome"],
            })
        );

        let quick = DialogTag::quick_actions()
            .dialog(dialog_id("example:settings"))
            .replace(true);
        assert_eq!(quick.resource_location().namespace(), "minecraft");
        assert_eq!(quick.resource_location().path(), "dialog/quick_actions");
        assert_eq!(
            quick.to_json(),
            json!({
                "replace": true,
                "values": ["example:settings"],
            })
        );
    }

    #[test]
    fn confirmation_type() {
        let d = Dialog::confirmation(dialog_id("example:confirm"));
        assert!(
            d.to_json()["type"]
                .as_str()
                .unwrap()
                .contains("confirmation")
        );
    }

    #[test]
    fn multi_action_type() {
        let d = Dialog::multi_action(dialog_id("example:menu"));
        let json = d.to_json();
        assert!(json["type"].as_str().unwrap().contains("multi_action"));
    }

    #[test]
    fn multi_action_local_actions_key() {
        let d = Dialog::multi_action_local("menu")
            .title("Power Selector")
            .button(
                DialogButton::new("Enhanced Cells")
                    .action(DialogAction::run_function(function_id("example:power/1"))),
            );
        let json = d.to_json();
        assert!(
            json["actions"].is_array(),
            "multi_action must use \"actions\", got: {json}"
        );
        assert!(
            json.get("buttons").is_none(),
            "multi_action must not contain \"buttons\", got: {json}"
        );
        assert_eq!(
            json["actions"][0]["label"]["text"].as_str().unwrap(),
            "Enhanced Cells"
        );
    }

    #[test]
    fn multi_action_no_buttons_key() {
        let d = Dialog::multi_action(dialog_id("example:select"))
            .button(DialogButton::new("A"))
            .button(DialogButton::new("B"));
        let json = d.to_json();
        assert!(json["actions"].is_array());
        assert_eq!(json["actions"].as_array().unwrap().len(), 2);
        assert!(
            json.get("buttons").is_none(),
            "multi_action must not have \"buttons\""
        );
    }

    #[test]
    fn passive_power_selector_actions() {
        let d = Dialog::multi_action_local("power_selector")
            .title(Text::new("Select Passive Power").gold())
            .body(DialogBody::text(Text::new(
                "Choose a passive power to unlock.",
            )))
            .button(
                DialogButton::new(Text::new("Enhanced Cells").green()).action(
                    DialogAction::run_function(function_id("example:power/enhanced_cells")),
                ),
            )
            .button(DialogButton::new(Text::new("Regeneration").aqua()).action(
                DialogAction::run_function(function_id("example:power/regeneration")),
            ));
        let json = d.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:multi_action");
        assert!(json["actions"].is_array());
        assert_eq!(json["actions"].as_array().unwrap().len(), 2);
        assert_eq!(
            json["actions"][0]["label"]["text"].as_str().unwrap(),
            "Enhanced Cells"
        );
        assert_eq!(
            json["actions"][0]["action"]["command"].as_str().unwrap(),
            "/function example:power/enhanced_cells"
        );
        assert_eq!(
            json["actions"][1]["label"]["text"].as_str().unwrap(),
            "Regeneration"
        );
        assert!(json.get("buttons").is_none());
    }

    #[test]
    fn button_action_open_dialog() {
        let btn = DialogButton::new("Rules")
            .action(DialogAction::open_dialog(dialog_id("example:rules")));
        let json = btn.to_json();
        assert!(
            json["action"]["dialog"]
                .as_str()
                .unwrap()
                .contains("example:rules")
        );
    }

    #[test]
    fn dialog_close_action() {
        let btn = DialogButton::new("Close").action(DialogAction::close());
        let json = btn.to_json();
        assert_eq!(json["action"]["type"].as_str().unwrap(), "minecraft:close");
    }

    #[test]
    fn pause_and_external_title() {
        let d = Dialog::notice(dialog_id("ex:test"))
            .pause(true)
            .external_title(true);
        let json = d.to_json();
        assert!(json["pause"].as_bool().unwrap());
        assert!(json["external_title"].as_bool().unwrap());
    }

    #[test]
    fn item_body() {
        let body = DialogBody::item_sized(
            crate::registry::ItemId::minecraft("diamond").unwrap(),
            32,
            32,
        );
        let json = body.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:item");
        assert_eq!(json["item"].as_str().unwrap(), "minecraft:diamond");
        assert_eq!(json["width"].as_u64().unwrap(), 32);
    }

    #[test]
    fn golden_welcome_dialog() {
        let d = Dialog::multi_action_local("welcome")
            .title(Text::new("Welcome to the server!").gold())
            .body(DialogBody::text(Text::new(
                "Choose what you want to do next.",
            )))
            .button(
                DialogButton::new(Text::new("Start").green())
                    .action(DialogAction::run_function(function_id("example:start"))),
            )
            .button(
                DialogButton::new(Text::new("Rules").yellow())
                    .action(DialogAction::open_dialog(DialogId::local("rules"))),
            );
        let json = d.to_json();
        assert_eq!(json["type"].as_str().unwrap(), "minecraft:multi_action");
        let actions = json["actions"].as_array().unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(json["title"]["color"].as_str().unwrap(), "gold");
        assert_eq!(actions[0]["label"]["text"].as_str().unwrap(), "Start");
        assert_eq!(actions[0]["label"]["color"].as_str().unwrap(), "green");
        assert_eq!(
            actions[0]["action"]["command"].as_str().unwrap(),
            "/function example:start"
        );
        assert_eq!(actions[1]["label"]["text"].as_str().unwrap(), "Rules");
        assert_eq!(
            actions[1]["action"]["dialog"].as_str().unwrap(),
            "__sand_local:rules"
        );
    }

    // ── #150 validation tests ───────────────────────────────────────────────

    #[test]
    fn invalid_local_dialog_id_is_rejected_by_the_fallible_constructor() {
        assert!(DialogId::try_local("Not Valid! Path").is_err());
    }

    #[test]
    fn invalid_external_dialog_id_is_rejected_at_the_typed_boundary() {
        assert!("Not A Valid Location".parse::<DialogId>().is_err());
    }

    #[test]
    fn invalid_open_dialog_ref_rejected() {
        let dialog = Dialog::multi_action(dialog_id("example:menu")).button(
            DialogButton::new("Go").action(DialogAction::open_dialog_raw("not a valid ref")),
        );
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("buttons[0].action"), "{error}");
        assert!(error.contains("invalid dialog reference"), "{error}");
    }

    #[test]
    fn invalid_run_function_ref_rejected() {
        let dialog = Dialog::multi_action(dialog_id("example:menu")).button(
            DialogButton::new("Go").action(DialogAction::run_function_raw("not a valid ref")),
        );
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("buttons[0].action"), "{error}");
        assert!(error.contains("invalid function reference"), "{error}");
    }

    #[test]
    fn invalid_callback_ref_rejected_before_export() {
        let dialog = Dialog::multi_action(dialog_id("example:menu"))
            .button(DialogButton::new("Go").action(DialogAction::callback_raw("not a valid ref")));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("buttons[0].action"), "{error}");
        assert!(error.contains("invalid function reference"), "{error}");
    }

    #[test]
    fn callback_validation_runs_before_try_content_registers_it() {
        // try_content() must call validate() first — an invalid callback
        // must never reach DialogAction::to_json (which is what registers
        // the callback for __sand_dialog_tick generation).
        use crate::component::DatapackComponent;
        let _lock_guard = ();
        let before = drain_dialog_callbacks();
        assert!(before.is_empty(), "test must start with an empty registry");

        let dialog = Dialog::multi_action(dialog_id("example:menu"))
            .button(DialogButton::new("Go").action(DialogAction::callback_raw("not a valid ref")));
        assert!(DatapackComponent::try_content(&dialog).is_err());

        let after = drain_dialog_callbacks();
        assert!(
            after.is_empty(),
            "an invalid callback must never be registered, got: {after:?}"
        );
    }

    #[test]
    fn run_command_escape_hatch_not_validated() {
        // run_command remains the explicit raw escape hatch: arbitrary
        // strings (even ones that look nothing like a resource ref) pass.
        let dialog = Dialog::multi_action(dialog_id("example:menu"))
            .button(DialogButton::new("Go").action(DialogAction::run_command("say hello there")));
        assert!(dialog.validate().is_ok());
    }

    #[test]
    fn invalid_typed_item_reference_rejected() {
        let dialog = Dialog::notice(dialog_id("example:shop"))
            .body(DialogBody::item_raw("Not A Valid Item!"))
            .button(DialogButton::new("OK").action(DialogAction::close()));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("body[0].item"), "{error}");
        assert!(error.contains("invalid item reference"), "{error}");
    }

    #[test]
    fn valid_typed_item_reference_via_item_id() {
        let item_id = crate::registry::ItemId::minecraft("diamond").unwrap();
        let dialog = Dialog::notice(dialog_id("example:shop"))
            .body(DialogBody::item(item_id))
            .button(DialogButton::new("OK").action(DialogAction::close()));
        assert!(dialog.validate().is_ok());
        assert_eq!(
            dialog.to_json()["body"][0]["item"].as_str().unwrap(),
            "minecraft:diamond"
        );
    }

    #[test]
    fn zero_width_text_body_rejected() {
        let dialog = Dialog::notice(dialog_id("example:welcome"))
            .body(DialogBody::text_with_width("hi", 0))
            .button(DialogButton::new("OK").action(DialogAction::close()));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("body[0].width"), "{error}");
    }

    #[test]
    fn zero_dimension_item_body_rejected() {
        let dialog = Dialog::notice(dialog_id("example:shop"))
            .body(DialogBody::item_sized(
                crate::registry::ItemId::minecraft("diamond").unwrap(),
                0,
                32,
            ))
            .button(DialogButton::new("OK").action(DialogAction::close()));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("body[0].width"), "{error}");
    }

    #[test]
    fn zero_width_button_rejected() {
        let dialog = Dialog::notice(dialog_id("example:welcome")).button(
            DialogButton::new("OK")
                .action(DialogAction::close())
                .width(0),
        );
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("buttons[0].width"), "{error}");
    }

    #[test]
    fn multi_action_with_no_actions_rejected() {
        let dialog = Dialog::multi_action(dialog_id("example:empty"));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("field: actions"), "{error}");
        assert!(error.contains("at least one action"), "{error}");
    }

    #[test]
    fn notice_with_no_buttons_rejected() {
        let dialog = Dialog::notice(dialog_id("example:empty"));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("field: buttons"), "{error}");
    }

    #[test]
    fn confirmation_with_no_buttons_rejected() {
        let dialog = Dialog::confirmation(dialog_id("example:empty"));
        let error = dialog.validate().unwrap_err().to_string();
        assert!(error.contains("field: buttons"), "{error}");
    }

    #[test]
    fn well_formed_dialogs_of_every_kind_validate_ok() {
        let notice = Dialog::notice(dialog_id("example:notice"))
            .button(DialogButton::new("OK").action(DialogAction::close()));
        assert!(notice.validate().is_ok());

        let confirmation = Dialog::confirmation(dialog_id("example:confirm"))
            .button(DialogButton::new("Yes").action(DialogAction::close()))
            .button(DialogButton::new("No").action(DialogAction::close()));
        assert!(confirmation.validate().is_ok());

        let multi = Dialog::multi_action(dialog_id("example:menu"))
            .button(DialogButton::new("Go").action(DialogAction::close()));
        assert!(multi.validate().is_ok());
    }

    #[test]
    fn invalid_dialog_tag_entry_rejected() {
        use crate::component::DatapackComponent;
        let tag = DialogTag::pause_screen_additions().dialog_raw("not a valid ref");
        let error = tag.validate().unwrap_err().to_string();
        assert!(error.contains("values[0]"), "{error}");
        let _ = DatapackComponent::try_content(&tag);
    }
}
