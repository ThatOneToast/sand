use serde_json::Value;

/// The content a resource pack component contributes to the output.
pub enum AssetContent {
    /// Serialized JSON — written directly to the output file.
    Json(Value),

    /// Raw bytes for a programmatically generated binary asset (e.g. a PNG
    /// produced at build time). This variant is a placeholder; no built-in
    /// component currently produces it, but the API is reserved for future
    /// image-generation utilities.
    Bytes(Vec<u8>),

    /// A path (relative to the project root, i.e. the directory containing
    /// `sand.toml`) of a source file to copy verbatim into the resource pack.
    ///
    /// Example: `"src/assets/health_bar.png"` → the CLI copies that file to
    /// the appropriate location inside `dist/<namespace>-resources/`.
    CopyFrom(String),
}

/// One file that a component contributes to the resource pack.
pub struct AssetOutput {
    /// Full path from the pack root, e.g.
    /// `"assets/my_pack/font/hud.json"` or
    /// `"assets/my_pack/textures/font/health_bar.png"`.
    pub path: String,
    /// The content to write.
    pub content: AssetContent,
}

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
    fn assets(&self, namespace: &str) -> Vec<AssetOutput>;
}

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
