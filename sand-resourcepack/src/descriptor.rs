use crate::ResourcePackComponent;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::ResourcePackDescriptor",
    module = "sand::resourcepack",
    summary = "Registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).",
    context = "Registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`). Submitted at link time via [`inventory::submit!`] — no manual collection or wiring is needed. - `name` — a human-readable identifier for the component, used in diagnostics and duplicate-detection warnings. - `make` — a zero-argument factory function that constructs the component and boxes it as a trait object.",
    minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::ResourcePackDescriptor;",
    availability = ["Cargo feature: resourcepack"],
    fields(make = "`make` provides the make when registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).", name = "`name` provides the name when registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`)."),
)]
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
    pub name: &'static str,
    /// `make` provides the make when registry entry for a resource pack component registered via one of the Sand resource pack macros (`hud_bar!`, `hud_element!`, `texture!`).
    pub make: fn() -> Box<dyn ResourcePackComponent>,
}

inventory::collect!(ResourcePackDescriptor);
