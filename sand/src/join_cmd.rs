use anyhow::{Context, bail};

use crate::config::SandConfig;

pub struct JoinArgs {
    pub local: bool,
    pub singleplayer: bool,
}

pub fn run(args: JoinArgs) -> anyhow::Result<()> {
    let config_path = std::env::current_dir()?.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found in current directory");
    }
    let config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
        .context("failed to parse sand.toml")?;

    if args.local {
        if let Some(resourcepack) = &config.resourcepack {}
    }
    Ok(())
}
