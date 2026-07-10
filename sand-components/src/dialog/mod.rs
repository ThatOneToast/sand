//! Typed dialog datapack component builders.
//!
//! Dialogs are a Minecraft 1.21.6+ / 26.x feature for displaying data-driven
//! UI panels to players. They live at `data/<namespace>/dialog/<path>.json`.
//!
//! Always gate dialog usage with `VersionProfile::supports_dialogs()`:
//! ```rust,ignore
//! if profile.supports_dialogs() {
//!     let d = Dialog::notice_local("welcome")
//!         .title(Text::new("Welcome!").gold())
//!         .body(DialogBody::text(Text::new("Choose what to do next.")))
//!         .button(DialogButton::new(Text::new("Start").green()));
//! }
//! ```

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::{DatapackComponent, ResourceLocation};
use sand_commands::{Text, TextComponent};
use serde_json::{Value, json};

const SAND_LOCAL_NS: &str = "__sand_local";

// ── Dialog callback registry ──────────────────────────────────────────────────

/// The scoreboard trigger objective Sand uses for dialog callbacks.
pub const SAND_DIALOG_TRIGGER: &str = "sand.dialog";

/// Counter for assigning stable IDs to dialog callbacks.
/// Starts at 1 (0 = not triggered / not set).
static CALLBACK_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn callback_registry() -> &'static Mutex<Vec<(u32, String)>> {
    static REG: OnceLock<Mutex<Vec<(u32, String)>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register a dialog callback function and return its stable trigger ID.
///
/// The path must be a full `namespace:path` or a `__sand_local:path` sentinel.
/// IDs start at 1 (0 means "trigger not yet set").
pub fn register_dialog_callback(path: String) -> u32 {
    let id = CALLBACK_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    callback_registry().lock().unwrap().push((id, path));
    id
}

/// Drain all registered dialog callbacks.
///
/// Returns `(trigger_id, function_path)` pairs. Called by `export_components_json`
/// to generate the dialog dispatch tick/load functions.
pub fn drain_dialog_callbacks() -> Vec<(u32, String)> {
    std::mem::take(&mut *callback_registry().lock().unwrap())
}

/// Identifier accepted by dialog constructors.
#[derive(Debug, Clone)]
pub struct DialogId(ResourceLocation);

impl DialogId {
    pub fn local(path: impl AsRef<str>) -> Self {
        Self(
            ResourceLocation::new(SAND_LOCAL_NS, path).expect("invalid local dialog resource path"),
        )
    }

    pub fn external(location: impl AsRef<str>) -> Self {
        Self::from(location.as_ref())
    }

    fn into_location(self) -> ResourceLocation {
        self.0
    }
}

impl From<ResourceLocation> for DialogId {
    fn from(value: ResourceLocation) -> Self {
        Self(value)
    }
}

impl From<&ResourceLocation> for DialogId {
    fn from(value: &ResourceLocation) -> Self {
        Self(value.clone())
    }
}

impl From<&str> for DialogId {
    fn from(value: &str) -> Self {
        if value.contains(':') {
            Self(value.parse().expect("invalid dialog resource location"))
        } else {
            Self::local(value)
        }
    }
}

impl From<String> for DialogId {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct DialogText(TextComponent);

impl DialogText {
    fn to_json(&self) -> Value {
        serde_json::from_str(&self.0.to_string()).expect("TextComponent must serialize to JSON")
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

pub struct DialogFunctionPointerEntry {
    pub ptr: fn() -> Vec<String>,
    pub path: &'static str,
}
inventory::collect!(DialogFunctionPointerEntry);

pub struct DialogFunctionPointerTypeEntry {
    pub type_id: fn() -> std::any::TypeId,
    pub path: &'static str,
}
inventory::collect!(DialogFunctionPointerTypeEntry);

fn local_id_for_path(path: &str) -> String {
    if path.contains(':') {
        path.to_string()
    } else {
        format!("{SAND_LOCAL_NS}:{path}")
    }
}

fn registered_path_for_function_value<F>(value: F) -> Option<&'static str>
where
    F: Copy + 'static,
{
    let type_id = std::any::TypeId::of::<F>();
    for entry in inventory::iter::<DialogFunctionPointerTypeEntry>() {
        if (entry.type_id)() == type_id {
            return Some(entry.path);
        }
    }

    if std::mem::size_of::<F>() == std::mem::size_of::<fn() -> Vec<String>>() {
        let ptr = unsafe { *(&value as *const F).cast::<fn() -> Vec<String>>() };
        for entry in inventory::iter::<DialogFunctionPointerEntry>() {
            if entry.ptr as usize == ptr as usize {
                return Some(entry.path);
            }
        }
    }

    None
}

pub trait IntoDialogFunctionRef {
    fn into_dialog_function_command(self) -> String;
    fn into_dialog_function_path(self) -> String;
}

impl IntoDialogFunctionRef for ResourceLocation {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
    fn into_dialog_function_path(self) -> String {
        self.to_string()
    }
}

impl IntoDialogFunctionRef for &ResourceLocation {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
    fn into_dialog_function_path(self) -> String {
        self.to_string()
    }
}

impl IntoDialogFunctionRef for &str {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
    fn into_dialog_function_path(self) -> String {
        self.to_string()
    }
}

impl IntoDialogFunctionRef for String {
    fn into_dialog_function_command(self) -> String {
        format!("/function {}", self)
    }
    fn into_dialog_function_path(self) -> String {
        self
    }
}

impl<F> IntoDialogFunctionRef for F
where
    F: Fn() -> Vec<String> + Copy + 'static,
{
    fn into_dialog_function_command(self) -> String {
        if let Some(path) = registered_path_for_function_value(self) {
            return format!("/function {}", local_id_for_path(path));
        }
        panic!(
            "unregistered function pointer: the function must be annotated with \
             #[function] or #[function(\"path\")] to be used in DialogAction::run_function()"
        )
    }
    fn into_dialog_function_path(self) -> String {
        if let Some(path) = registered_path_for_function_value(self) {
            return local_id_for_path(path);
        }
        panic!(
            "unregistered function pointer: the function must be annotated with \
             #[function] or #[function(\"path\")] to be used in DialogAction::callback()"
        )
    }
}

pub trait IntoDialogRef {
    fn into_dialog_ref(self) -> String;
}

impl IntoDialogRef for ResourceLocation {
    fn into_dialog_ref(self) -> String {
        self.to_string()
    }
}

impl IntoDialogRef for &ResourceLocation {
    fn into_dialog_ref(self) -> String {
        self.to_string()
    }
}

impl IntoDialogRef for DialogId {
    fn into_dialog_ref(self) -> String {
        self.into_location().to_string()
    }
}

impl IntoDialogRef for &str {
    fn into_dialog_ref(self) -> String {
        DialogId::from(self).into_location().to_string()
    }
}

impl IntoDialogRef for String {
    fn into_dialog_ref(self) -> String {
        DialogId::from(self).into_location().to_string()
    }
}

// ── DialogTag ────────────────────────────────────────────────────────────────

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
    pub fn pause_screen_additions() -> Self {
        Self::well_known("pause_screen_additions")
    }

    /// Tag dialogs shown by the Quick Actions key.
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
    pub fn dialog(mut self, dialog: impl IntoDialogRef) -> Self {
        self.values.push(dialog.into_dialog_ref());
        self
    }

    /// Add multiple dialog entries to this tag.
    pub fn dialogs<I, D>(mut self, dialogs: I) -> Self
    where
        I: IntoIterator<Item = D>,
        D: IntoDialogRef,
    {
        self.values
            .extend(dialogs.into_iter().map(IntoDialogRef::into_dialog_ref));
        self
    }

    /// Set whether this tag replaces lower-priority definitions.
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }
}

// ── DialogBody ────────────────────────────────────────────────────────────────

/// A dialog body element (text, item display, etc.).
#[derive(Debug, Clone)]
pub enum DialogBody {
    /// Plain text body element.
    Text {
        text: Box<DialogText>,
        width: Option<u32>,
    },
    /// Item display body element.
    Item {
        item: String,
        width: Option<u32>,
        height: Option<u32>,
    },
}

impl DialogBody {
    /// Plain text body.
    pub fn text(content: impl Into<DialogText>) -> Self {
        Self::Text {
            text: Box::new(content.into()),
            width: None,
        }
    }

    /// Plain text body with explicit width.
    pub fn text_with_width(content: impl Into<DialogText>, width: u32) -> Self {
        Self::Text {
            text: Box::new(content.into()),
            width: Some(width),
        }
    }

    /// Item display body.
    pub fn item(item: impl Into<String>) -> Self {
        Self::Item {
            item: item.into(),
            width: None,
            height: None,
        }
    }

    /// Item display body with explicit dimensions.
    pub fn item_sized(item: impl Into<String>, width: u32, height: u32) -> Self {
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

/// An action associated with a dialog button.
#[derive(Debug, Clone)]
pub enum DialogAction {
    /// Run a command when the button is pressed.
    RunCommand(String),
    /// Fill the chat bar with a command suggestion.
    SuggestCommand(String),
    /// Open a URL (where server-controlled links are permitted).
    OpenUrl(String),
    /// Open another dialog.
    OpenDialog(String),
    /// Close the current dialog.
    Close,
}

impl DialogAction {
    pub fn run_command(cmd: impl Into<String>) -> Self {
        Self::RunCommand(cmd.into())
    }

    /// Run a datapack function when the button is pressed.
    ///
    /// Prefer this over [`run_command`](DialogAction::run_command) for datapack
    /// functions. It accepts registered function pointers and typed external
    /// resource locations.
    ///
    /// ```
    /// use sand_components::dialog::DialogAction;
    /// use sand_components::ResourceLocation;
    ///
    /// let action = DialogAction::run_function(
    ///     ResourceLocation::new("example", "start").unwrap()
    /// );
    /// ```
    pub fn run_function(id: impl IntoDialogFunctionRef) -> Self {
        Self::RunCommand(id.into_dialog_function_command())
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
    pub fn callback(id: impl IntoDialogFunctionRef) -> Self {
        let path = id.into_dialog_function_path();
        let trigger_id = register_dialog_callback(path);
        Self::RunCommand(format!("/trigger {SAND_DIALOG_TRIGGER} set {trigger_id}"))
    }

    pub fn suggest_command(cmd: impl Into<String>) -> Self {
        Self::SuggestCommand(cmd.into())
    }
    pub fn open_url(url: impl Into<String>) -> Self {
        Self::OpenUrl(url.into())
    }
    pub fn open_dialog(dialog: impl IntoDialogRef) -> Self {
        Self::OpenDialog(dialog.into_dialog_ref())
    }
    pub fn close() -> Self {
        Self::Close
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::RunCommand(c) => json!({"type": "minecraft:run_command", "command": c}),
            Self::SuggestCommand(c) => json!({"type": "minecraft:suggest_command", "command": c}),
            Self::OpenUrl(u) => json!({"type": "minecraft:open_url", "url": u}),
            Self::OpenDialog(d) => json!({"type": "minecraft:open_dialog", "dialog": d}),
            Self::Close => json!({"type": "minecraft:close"}),
        }
    }
}

// ── DialogButton ──────────────────────────────────────────────────────────────

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
    pub fn new(label: impl Into<DialogText>) -> Self {
        Self {
            label: label.into(),
            action: None,
            tooltip: None,
            width: None,
        }
    }

    /// Attach an action to this button.
    pub fn action(mut self, action: DialogAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Attach a tooltip shown when hovering over the button.
    pub fn tooltip(mut self, tip: impl Into<DialogText>) -> Self {
        self.tooltip = Some(tip.into());
        self
    }

    /// Set the button width in pixels.
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

/// A typed dialog datapack component builder.
///
/// Dialogs live at `data/<namespace>/dialog/<path>.json` and require
/// Minecraft 1.21.6+ / 26.x. Always check `VersionProfile::supports_dialogs()`
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
    /// The resource location for this dialog (e.g. `"example:welcome"`).
    pub id: ResourceLocation,
    kind: DialogKind,
    title: Option<DialogText>,
    body: Vec<DialogBody>,
    buttons: Vec<DialogButton>,
    pause: bool,
    external_title: bool,
}

impl Dialog {
    /// Create a notice dialog — informational, dismissible.
    pub fn notice(id: impl Into<DialogId>) -> Self {
        Self::new_with_kind(id, DialogKind::Notice)
    }

    /// Create a local notice dialog whose namespace is resolved during export.
    pub fn notice_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::Notice)
    }

    /// Create a confirmation dialog — confirm / cancel.
    pub fn confirmation(id: impl Into<DialogId>) -> Self {
        Self::new_with_kind(id, DialogKind::Confirmation)
    }

    /// Create a local confirmation dialog whose namespace is resolved during export.
    pub fn confirmation_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::Confirmation)
    }

    /// Create a multi-action dialog — multiple custom buttons.
    pub fn multi_action(id: impl Into<DialogId>) -> Self {
        Self::new_with_kind(id, DialogKind::MultiAction)
    }

    /// Create a local multi-action dialog whose namespace is resolved during export.
    pub fn multi_action_local(path: impl AsRef<str>) -> Self {
        Self::new_with_kind(DialogId::local(path), DialogKind::MultiAction)
    }

    fn new_with_kind(id: impl Into<DialogId>, kind: DialogKind) -> Self {
        Self {
            id: id.into().into_location(),
            kind,
            title: None,
            body: vec![],
            buttons: vec![],
            pause: false,
            external_title: false,
        }
    }

    /// Set the dialog title.
    pub fn title(mut self, text: impl Into<DialogText>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Append a body element.
    pub fn body(mut self, body: DialogBody) -> Self {
        self.body.push(body);
        self
    }

    /// Append a button.
    pub fn button(mut self, btn: DialogButton) -> Self {
        self.buttons.push(btn);
        self
    }

    /// Whether this dialog pauses the game in single-player.
    pub fn pause(mut self, v: bool) -> Self {
        self.pause = v;
        self
    }

    /// Whether the title is rendered outside the dialog frame.
    pub fn external_title(mut self, v: bool) -> Self {
        self.external_title = v;
        self
    }

    /// Serialize to the datapack JSON format.
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
    pub fn resource_path(&self) -> String {
        if self.id.namespace() == SAND_LOCAL_NS {
            format!("dialog/{}.json", self.id.path())
        } else {
            format!("{}/dialog/{}.json", self.id.namespace(), self.id.path())
        }
    }
}

impl DatapackComponent for Dialog {
    fn resource_location(&self) -> &ResourceLocation {
        &self.id
    }

    fn to_json(&self) -> Value {
        Dialog::to_json(self)
    }

    fn component_dir(&self) -> &'static str {
        "dialog"
    }
}

impl IntoDialogRef for &Dialog {
    fn into_dialog_ref(self) -> String {
        self.id.to_string()
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

    fn component_dir(&self) -> &'static str {
        "tags"
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_dialog_json() {
        let d = Dialog::notice("example:welcome")
            .title("Welcome!")
            .body(DialogBody::text("Choose an option."))
            .button(
                DialogButton::new("Start").action(DialogAction::run_function(
                    ResourceLocation::new("example", "start").unwrap(),
                )),
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
        let d = Dialog::notice("example:welcome");
        assert_eq!(d.resource_path(), "example/dialog/welcome.json");
    }

    #[test]
    fn resource_path_no_namespace() {
        let d = Dialog::notice("welcome");
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
        let pause = DialogTag::pause_screen_additions().dialog("example:welcome");
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
            .dialog(ResourceLocation::new("example", "settings").unwrap())
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
        let d = Dialog::confirmation("example:confirm");
        assert!(
            d.to_json()["type"]
                .as_str()
                .unwrap()
                .contains("confirmation")
        );
    }

    #[test]
    fn multi_action_type() {
        let d = Dialog::multi_action("example:menu");
        let json = d.to_json();
        assert!(json["type"].as_str().unwrap().contains("multi_action"));
    }

    #[test]
    fn multi_action_local_actions_key() {
        let d = Dialog::multi_action_local("menu")
            .title("Power Selector")
            .button(
                DialogButton::new("Enhanced Cells").action(DialogAction::run_function(
                    ResourceLocation::new("example", "power/1").unwrap(),
                )),
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
        let d = Dialog::multi_action("example:select")
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
                    DialogAction::run_function(
                        ResourceLocation::new("example", "power/enhanced_cells").unwrap(),
                    ),
                ),
            )
            .button(DialogButton::new(Text::new("Regeneration").aqua()).action(
                DialogAction::run_function(
                    ResourceLocation::new("example", "power/regeneration").unwrap(),
                ),
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
        let btn = DialogButton::new("Rules").action(DialogAction::open_dialog("example:rules"));
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
        let d = Dialog::notice("ex:test").pause(true).external_title(true);
        let json = d.to_json();
        assert!(json["pause"].as_bool().unwrap());
        assert!(json["external_title"].as_bool().unwrap());
    }

    #[test]
    fn item_body() {
        let body = DialogBody::item_sized("minecraft:diamond", 32, 32);
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
            .button(DialogButton::new(Text::new("Start").green()).action(
                DialogAction::run_function(ResourceLocation::new("example", "start").unwrap()),
            ))
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
}
