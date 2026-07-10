use serde::Deserialize;

use crate::build::records::PackNamespace;

#[derive(Deserialize)]
pub struct SandConfig {
    pub pack: PackConfig,
    /// Optional resource pack configuration. Required when running
    /// `sand build --resourcepack`.
    pub resourcepack: Option<ResourcePackConfig>,
}

#[derive(Deserialize)]
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
#[derive(Deserialize)]
pub struct ResourcePackConfig {
    /// Short description shown in the resource pack menu.
    /// Defaults to the pack description if omitted.
    pub description: Option<String>,
    /// Asset namespace. Defaults to `[pack].namespace` if omitted.
    pub namespace: Option<PackNamespace>,
    /// Resource pack format number. If omitted, derived automatically from
    /// `[pack].mc_version` using the bundled version table.
    pub resource_pack_format: Option<u32>,
}
