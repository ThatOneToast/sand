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

use crate::{DatapackComponent, ResourceLocation};
use sand_commands::{Text, TextComponent};
use serde_json::{Value, json};

const SAND_LOCAL_NS: &str = "__sand_local";

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
}

impl IntoDialogFunctionRef for ResourceLocation {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
}

impl IntoDialogFunctionRef for &ResourceLocation {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
}

impl IntoDialogFunctionRef for &str {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
    }
}

impl IntoDialogFunctionRef for String {
    fn into_dialog_function_command(self) -> String {
        format!("/function {self}")
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
            v["buttons"] = json!(self.buttons.iter().map(|b| b.to_json()).collect::<Vec<_>>());
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
                DialogButton::new("Start")
                    .action(DialogAction::run_command("/function example:start")),
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
        assert!(
            d.to_json()["type"]
                .as_str()
                .unwrap()
                .contains("multi_action")
        );
    }

    #[test]
    fn button_action_run_command() {
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
        let d = Dialog::notice_local("welcome")
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
        let buttons = json["buttons"].as_array().unwrap();
        assert_eq!(buttons.len(), 2);
        assert_eq!(json["title"]["color"].as_str().unwrap(), "gold");
        assert_eq!(buttons[0]["label"]["text"].as_str().unwrap(), "Start");
        assert_eq!(buttons[0]["label"]["color"].as_str().unwrap(), "green");
        assert_eq!(
            buttons[0]["action"]["command"].as_str().unwrap(),
            "/function example:start"
        );
        assert_eq!(buttons[1]["label"]["text"].as_str().unwrap(), "Rules");
        assert_eq!(
            buttons[1]["action"]["dialog"].as_str().unwrap(),
            "__sand_local:rules"
        );
    }
}
