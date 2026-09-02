use serde_json::Value;

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::AssetContent",
    module = "sand::resourcepack",
    summary = "The content a resource pack component contributes to the output.",
    context = "The content a resource pack component contributes to the output. This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
    minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::AssetContent;",
    availability = ["Cargo feature: resourcepack"],
    variants(Bytes = "Raw bytes for a programmatically generated binary asset (e.g. a PNG produced at build time). This variant is a placeholder; no built-in component currently produces it, but the API is reserved for future image-generation utilities.", CopyFrom = "A path (relative to the project root, i.e. the directory containing `sand.toml`) of a source file to copy verbatim into the resource pack.", Json = "Serialized JSON — written directly to the output file."),
    variant_fields(Bytes = ["Raw bytes for a programmatically generated binary asset (e.g. a PNG produced at build time). This variant is a placeholder; no built-in component currently produces it, but the API is reserved for future image-generation utilities."], CopyFrom = ["A path (relative to the project root, i.e. the directory containing `sand.toml`) of a source file to copy verbatim into the resource pack."], Json = ["Serialized JSON — written directly to the output file."]),
)]
/// The content a resource pack component contributes to the output.
pub enum AssetContent {
    /// Serialized JSON — written directly to the output file.
    Json(#[doc = "Serialized JSON — written directly to the output file."] Value),

    /// Raw bytes for a programmatically generated binary asset (e.g. a PNG
    /// produced at build time). This variant is a placeholder; no built-in
    /// component currently produces it, but the API is reserved for future
    /// image-generation utilities.
    Bytes(
        #[doc = "Raw bytes for a programmatically generated binary asset (e.g. a PNG produced at build time). This variant is a placeholder; no built-in component currently produces it, but the API is reserved for future image-generation utilities."]
         Vec<u8>,
    ),

    /// A path (relative to the project root, i.e. the directory containing
    /// `sand.toml`) of a source file to copy verbatim into the resource pack.
    ///
    /// Example: `"src/assets/health_bar.png"` → the CLI copies that file to
    /// the appropriate location inside `dist/<namespace>-resources/`.
    CopyFrom(
        #[doc = "A path (relative to the project root, i.e. the directory containing `sand.toml`) of a source file to copy verbatim into the resource pack."]
         String,
    ),
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::AssetOutput",
    module = "sand::resourcepack",
    summary = "One file that a component contributes to the resource pack.",
    context = "One file that a component contributes to the resource pack. This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
    minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::AssetOutput;",
    availability = ["Cargo feature: resourcepack"],
    fields(content = "The content to write.", path = "Full path from the pack root, e.g. `\"assets/my_pack/font/hud.json\"` or `\"assets/my_pack/textures/font/health_bar.png\"`."),
)]
/// One file that a component contributes to the resource pack.
pub struct AssetOutput {
    /// Full path from the pack root, e.g.
    /// `"assets/my_pack/font/hud.json"` or
    /// `"assets/my_pack/textures/font/health_bar.png"`.
    pub path: String,
    /// The content to write.
    pub content: AssetContent,
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::ResourcePackComponent",
    module = "sand::resourcepack",
    summary = "A value that can be written as one or more files into a Minecraft resource pack.",
    context = "A value that can be written as one or more files into a Minecraft resource pack. Implementors represent resource pack elements such as bitmap fonts, HUD overlays, and raw textures. Each component knows its own asset paths and can produce the JSON (or binary) content Minecraft expects. A single component may produce multiple output files. For example, [`HudBar`](sand::resourcepack::HudBar) produces both a font JSON entry *and* a texture copy record for the source PNG. When multiple components target the same font file (same `path` ending in `.json` with a `\"providers\"` key), [`export_resourcepack_json`](sand::resourcepack::export_resourcepack_json) automatically merges their provider arrays into one file.",
    minecraft = "Implementors represent resource pack elements such as bitmap fonts, HUD overlays, and raw textures. Each component knows its own asset paths and can produce the JSON (or binary) content Minecraft expects.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::ResourcePackComponent;",
    availability = ["Cargo feature: resourcepack"],
)]
/// A value that can be written as one or more files into a Minecraft resource
/// pack.
///
/// Implementors represent resource pack elements such as bitmap fonts, HUD
/// overlays, and raw textures. Each component knows its own asset paths and
/// can produce the JSON (or binary) content Minecraft expects.
///
/// # Multiple outputs
///
/// A single component may produce multiple output files. For example,
/// [`HudBar`](crate::HudBar) produces both a font JSON entry *and* a texture
/// copy record for the source PNG.
///
/// # Font merging
///
/// When multiple components target the same font file (same `path` ending in
/// `.json` with a `"providers"` key), [`export_resourcepack_json`](crate::export_resourcepack_json)
/// automatically merges their provider arrays into one file.
pub trait ResourcePackComponent {
    /// All asset outputs this component contributes to the resource pack.
    ///
    /// `namespace` is the pack namespace from `sand.toml` (e.g. `"my_pack"`).
    #[sand_macros::api(
        registry = sand_api_contract,
        path = "sand::resourcepack::ResourcePackComponent::assets",
        module = "sand::resourcepack",
        summary = "All asset outputs this component contributes to the resource pack.",
        context = "All asset outputs this component contributes to the resource pack. `namespace` is the pack namespace from `sand.toml` (e.g. `\"my_pack\"`).",
        minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
        use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
        avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
        params(namespace = "`namespace` is the pack namespace from `sand.toml` (e.g. `\"my_pack\"`)."),
        returns = "The ordered values produced to all asset outputs this component contributes to the resource pack.",
        example = "use sand::prelude::*;\n\nfn demonstrate<T: sand::resourcepack::ResourcePackComponent>(resource_pack_component_value: &T, namespace: & str)  {\n    let values = resource_pack_component_value.assets(namespace);\n}",
        availability = ["Cargo feature: resourcepack"],
    )]
    fn assets(&self, namespace: &str) -> Vec<AssetOutput>;
}

#[sand_macros::api(
    registry = sand_api_contract,
    path = "sand::resourcepack::ResourcePackRecord",
    module = "sand::resourcepack",
    summary = "Wire record emitted by `sand_resource_export` and consumed by the `sand` CLI to write resource pack files.",
    context = "Wire record emitted by `sand_resource_export` and consumed by the `sand` CLI to write resource pack files. This API defines client-side HUD, font, texture, or resource-pack output while keeping asset registration and exporter inventory wiring private.",
    minecraft = "The resourcepack exporter writes version-appropriate assets, bitmap-font providers, and pack metadata for the selected Minecraft profile.",
    use_when = ["Building HUD bars, HUD elements, textures, or resource-pack output alongside a Sand datapack"],
    avoid_when = ["The project is datapack-only or needs unrelated resource-pack functionality not modeled by Sand"],
    example = "use sand::resourcepack::ResourcePackRecord;",
    availability = ["Cargo feature: resourcepack"],
    fields(content = "JSON string, source path, or base64 bytes depending on `content_type`.", content_type = "`\"json\"` — write `content` as UTF-8 text. `\"copy\"` — copy the file at `content` (project-root-relative path). `\"bytes\"` — write `content` as base-64-decoded bytes (reserved).", path = "Full path from the pack root, e.g. `\"assets/ns/font/hud.json\"`."),
)]
/// Wire record emitted by `sand_resource_export` and consumed by the `sand`
/// CLI to write resource pack files.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResourcePackRecord {
    /// Full path from the pack root, e.g. `"assets/ns/font/hud.json"`.
    pub path: String,
    /// `"json"` — write `content` as UTF-8 text.
    /// `"copy"` — copy the file at `content` (project-root-relative path).
    /// `"bytes"` — write `content` as base-64-decoded bytes (reserved).
    pub content_type: String,
    /// JSON string, source path, or base64 bytes depending on `content_type`.
    pub content: String,
}
