use crate::ResourcePackComponent;

#[doc = "**API Contract:** Run `sand api show sand::resourcepack::ResourcePackDescriptor` for the canonical contract."]
/// Registry entry for a resource pack component registered via one of the
/// Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).
///
/// Submitted at link time via [`inventory::submit!`] — no manual collection
/// or wiring is needed.
///
/// # Fields
/// - `name` — a human-readable identifier for the component, used in
///   diagnostics and duplicate-detection warnings.
/// - `make` — a zero-argument factory function that constructs the component
///   and boxes it as a trait object.
pub struct ResourcePackDescriptor {
    /// `name` provides the name when registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).
    #[doc = "**API Contract:** Run `sand api show sand::resourcepack::ResourcePackDescriptor::name` for the canonical contract."]
    pub name: &'static str,
    /// `make` provides the make when registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).
    #[doc = "**API Contract:** Run `sand api show sand::resourcepack::ResourcePackDescriptor::make` for the canonical contract."]
    pub make: fn() -> Box<dyn ResourcePackComponent>,
}

inventory::collect!(ResourcePackDescriptor);
