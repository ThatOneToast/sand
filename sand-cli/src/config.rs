use serde::Deserialize;

use crate::build::records::{PackNamespace, PackOverlay, PackSupportedFormats};

#[derive(Debug, Deserialize)]
pub struct SandConfig {
    pub pack: PackConfig,
    /// Optional resource pack configuration. Required when running
    /// `sand build --resourcepack`.
    pub resourcepack: Option<ResourcePackConfig>,
}

/// `[pack]` section in `sand.toml`.
///
/// ## `pack_format` vs `supported_formats` / `overlays`
///
/// `pack_format` alone is the right choice for the common case: a pack built
/// and tested against one Minecraft version. Sand derives it automatically
/// from `mc_version`, and it's what most projects should use.
///
/// Set `supported_formats` when the *same* generated content is valid across
/// a range of pack-format numbers (for example, a datapack with no
/// version-specific components that happens to load unchanged on several
/// recent drops). It widens the format range Minecraft will accept without
/// changing what gets written.
///
/// Use `overlays` when different format ranges need *different* content —
/// vanilla loads the overlay directory instead of (layered on top of) the
/// base pack once the world's format falls in that overlay's range. Sand
/// only emits the overlay metadata; it does not currently generate or route
/// files into overlay directories, so the directory contents are the
/// project's responsibility.
#[derive(Debug, Deserialize)]
pub struct PackConfig {
    /// Validated Minecraft namespace — rejected at parse time if it contains
    /// uppercase letters, spaces, or other illegal characters.
    pub namespace: PackNamespace,
    pub description: String,
    /// Minecraft version string. Use `"latest"` to target Sand's bundled
    /// latest-known verified version.
    pub mc_version: String,
    /// Pack format number. If omitted, it is derived automatically from
    /// `mc_version` using the bundled version table.
    pub pack_format: Option<u32>,
    /// Optional `pack.mcmeta` `supported_formats`: a single format number or
    /// an inclusive `{ min, max }` range. Omit to keep the minimal
    /// `pack_format`-only output.
    pub supported_formats: Option<PackSupportedFormats>,
    /// Optional `pack.mcmeta` overlay entries (`overlays.entries`). Each
    /// entry pairs a validated relative directory with its own supported
    /// format range. Empty (the default) omits the `overlays` key entirely.
    #[serde(default)]
    pub overlays: Vec<PackOverlay>,
}

/// `[resourcepack]` section in `sand.toml`.
///
/// Example:
/// ```toml
/// [resourcepack]
/// description = "My resource pack"
/// # namespace defaults to [pack].namespace if omitted
/// # namespace = "my_pack"
/// # resource_pack_format = 46  # override the auto-detected format
/// ```
#[derive(Debug, Deserialize)]
pub struct ResourcePackConfig {
    /// Short description shown in the resource pack menu.
    /// Defaults to the pack description if omitted.
    pub description: Option<String>,
    /// Asset namespace. Defaults to `[pack].namespace` if omitted.
    pub namespace: Option<PackNamespace>,
    /// Resource pack format number. If omitted, derived automatically from
    /// `[pack].mc_version` using the bundled version table.
    pub resource_pack_format: Option<u32>,
    /// Optional `pack.mcmeta` `supported_formats` for the resource pack. Same
    /// shape and semantics as `[pack].supported_formats`.
    pub supported_formats: Option<PackSupportedFormats>,
    /// Optional `pack.mcmeta` overlay entries for the resource pack. Same
    /// shape and semantics as `[pack].overlays`.
    #[serde(default)]
    pub overlays: Vec<PackOverlay>,
}
