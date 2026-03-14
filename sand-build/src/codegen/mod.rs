mod blocks;
mod commands;
mod registries;

use std::path::Path;

use crate::error::Result;

/// Generate all source files from the data generator reports.
///
/// Writes to `out_dir` (typically `$OUT_DIR` from Cargo):
/// - `registries.rs` — enums for item, block, entity type, biome, etc.
/// - `block_states.rs` — per-block property structs and shared property enums.
/// - `commands.rs`    — builder structs for Minecraft commands.
pub fn generate_all(reports_dir: &Path, out_dir: &Path) -> Result<()> {
    registries::generate(reports_dir, out_dir)?;
    blocks::generate(reports_dir, out_dir)?;
    commands::generate(reports_dir, out_dir)?;
    Ok(())
}
