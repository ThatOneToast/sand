use anyhow::{Context, bail, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::config::SandConfig;

/// Launch an existing, authenticated Prism instance without modifying its files.
pub struct JoinArgs {
    /// Connect to the port in the project's prepared local server properties.
    pub local: bool,
    /// Prism instance folder ID.
    pub instance: String,
    /// Existing singleplayer world folder; defaults to `sand-dev`.
    pub world: Option<String>,
    /// Executable name on PATH or an explicit executable path.
    pub launcher: PathBuf,
}

/// Ask Prism to launch. A missing executable or nonzero launcher exit is an error.
/// This does not install Minecraft, authenticate, copy packs, or start a server.
pub fn run(args: JoinArgs) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let mut command = launch_command(&root, &args)?;
    let status = command.status().with_context(|| {
        format!(
            "failed to launch Prism at {}; install Prism or use --launcher",
            args.launcher.display()
        )
    })?;
    ensure!(status.success(), "Prism launcher exited with {status}");
    Ok(())
}

fn launch_command(root: &Path, args: &JoinArgs) -> anyhow::Result<Command> {
    let config_path = root.join("sand.toml");
    if !config_path.exists() {
        bail!("sand.toml not found in current directory");
    }
    let _config: SandConfig = toml::from_str(&std::fs::read_to_string(&config_path)?)
        .context("failed to parse sand.toml")?;

    ensure!(
        !args.instance.trim().is_empty(),
        "--instance must identify an existing Prism instance"
    );
    ensure!(
        !(args.local && args.world.is_some()),
        "--local and --world cannot be combined"
    );
    let mut command = Command::new(&args.launcher);
    command.arg("--launch").arg(&args.instance);
    if args.local {
        let properties = std::fs::read_to_string(root.join("dist/server/server.properties"))
            .context("local server is not prepared; run sand run or sand dev first")?;
        let port = local_port(&properties)?;
        command.arg("--server").arg(format!("127.0.0.1:{port}"));
    } else {
        let world = args.world.as_deref().unwrap_or("sand-dev");
        ensure!(
            !world.trim().is_empty(),
            "--world must identify an existing world folder"
        );
        command.arg("--world").arg(world);
    }
    Ok(command)
}

fn local_port(properties: &str) -> anyhow::Result<u16> {
    let mut value = "25565";
    for line in properties.lines() {
        let line = line.trim();
        if line.starts_with(['#', '!']) {
            continue;
        }
        let (key, candidate) = line.split_once(['=', ':', ' ', '\t']).unwrap_or((line, ""));
        if key == "server-port" {
            value = candidate.trim_start_matches(['=', ':', ' ', '\t']).trim();
        }
    }
    let port: u16 = value
        .parse()
        .context("invalid server-port in dist/server/server.properties")?;
    ensure!(port != 0, "server-port must be in 1..=65535");
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launches_exact_instance_and_world_arguments_without_shell_interpretation() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("sand.toml"),
            "[pack]\nnamespace='test'\ndescription='Test'\nmc_version='26.3'\n",
        )
        .unwrap();
        let mut args = JoinArgs {
            local: false,
            instance: "My instance $(literal)".into(),
            world: Some("World with spaces".into()),
            launcher: "prism with spaces".into(),
        };
        let command = launch_command(root.path(), &args).unwrap();
        assert_eq!(command.get_program(), "prism with spaces");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "--launch",
                "My instance $(literal)",
                "--world",
                "World with spaces"
            ]
        );
        args.local = true;
        assert!(launch_command(root.path(), &args).is_err());
        args.world = None;
        assert!(launch_command(root.path(), &args).is_err());
        std::fs::create_dir_all(root.path().join("dist/server")).unwrap();
        std::fs::write(
            root.path().join("dist/server/server.properties"),
            "server-port=25566\n",
        )
        .unwrap();
        assert_eq!(
            launch_command(root.path(), &args)
                .unwrap()
                .get_args()
                .collect::<Vec<_>>(),
            [
                "--launch",
                "My instance $(literal)",
                "--server",
                "127.0.0.1:25566"
            ]
        );
    }

    #[test]
    fn local_server_port_is_validated_and_last_property_wins() {
        assert_eq!(local_port("#server-port=1\n").unwrap(), 25565);
        assert_eq!(
            local_port("server-port=2\n server-port : 25567\n").unwrap(),
            25567
        );
        assert_eq!(
            local_port("server-port=invalid\nserver-port=25567\n").unwrap(),
            25567
        );
        for value in ["0", "65536", "invalid", ""] {
            assert!(local_port(&format!("server-port={value}")).is_err());
        }
    }
}
