use crate::ResourcePackComponent;

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
    pub name: &'static str,
    pub make: fn() -> Box<dyn ResourcePackComponent>,
}

inventory::collect!(ResourcePackDescriptor);
