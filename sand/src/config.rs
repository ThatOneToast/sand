use serde::Deserialize;

#[derive(Deserialize)]
pub struct SandConfig {
    pub pack: PackConfig,
}

#[derive(Deserialize)]
pub struct PackConfig {
    pub namespace: String,
    pub description: String,
    /// Minecraft version string. Use `"latest"` to always resolve to the
    /// current latest release from Mojang's version manifest.
    pub mc_version: String,
    /// Pack format number. If omitted, it is derived automatically from
    /// `mc_version` using the bundled version table.
    pub pack_format: Option<u32>,
}
