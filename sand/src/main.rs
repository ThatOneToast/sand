mod build_cmd;
mod config;
mod pack_format;
mod run_cmd;
mod scaffold;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;

use scaffold::{name_to_namespace, validate_name, ScaffoldOptions};

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "sand",
    version,
    about = "A Minecraft datapack toolkit for Rust"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Sand datapack project in a new directory
    New(NewArgs),
    /// Initialize a Sand project in the current directory
    Init(InitArgs),
    /// Build the datapack and write output to dist/
    Build {
        /// Package the output as a zip file for distribution
        #[arg(long)]
        release: bool,
    },
    /// Build the datapack, download the server jar, and start a local server
    Run {
        /// JVM heap size, e.g. "4G" or "2048M" (default: 4G)
        #[arg(long, default_value = "4G")]
        ram: String,
        /// Set online-mode=false in server.properties (easier local testing)
        #[arg(long)]
        offline: bool,
        /// Skip `sand build`; use whatever is already in dist/
        #[arg(long)]
        no_build: bool,
    },
    /// Remove build artifacts (dist/ and optionally Cargo target/)
    Clean {
        /// Also run `cargo clean` to remove the Cargo target directory
        #[arg(long)]
        cargo: bool,
    },
    /// Print the Sand version
    Version,
}

#[derive(clap::Args)]
struct NewArgs {
    /// Project name (lowercase letters, digits, underscores, hyphens)
    name: String,

    /// Target Minecraft version [default: latest release]
    #[arg(long)]
    mc_version: Option<String>,

    /// Short description of the datapack
    #[arg(long, default_value = "A Minecraft datapack built with Sand")]
    description: String,
}

#[derive(clap::Args)]
struct InitArgs {
    /// Target Minecraft version [default: latest release]
    #[arg(long)]
    mc_version: Option<String>,

    /// Short description of the datapack
    #[arg(long, default_value = "A Minecraft datapack built with Sand")]
    description: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e:#}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New(args) => cmd_new(args),
        Commands::Init(args) => cmd_init(args),
        Commands::Build { release } => build_cmd::run(release),
        Commands::Run { ram, offline, no_build } => run_cmd::run(run_cmd::RunArgs {
            ram, offline, no_build,
        }),
        Commands::Clean { cargo } => cmd_clean(cargo),
        Commands::Version => {
            println!("sand {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

// ── `sand new` ────────────────────────────────────────────────────────────────

fn cmd_new(args: NewArgs) -> Result<()> {
    validate_name(&args.name)?;
    let namespace = name_to_namespace(&args.name);
    let mc_version = resolve_mc_version(args.mc_version)?;
    let dir = PathBuf::from(&args.name);

    if dir.exists() {
        bail!("directory '{}' already exists", dir.display());
    }

    println!(
        "{} {} (Minecraft {})...",
        "Creating".cyan().bold(),
        args.name.white().bold(),
        mc_version.yellow()
    );

    scaffold::scaffold(&ScaffoldOptions {
        name: args.name.clone(),
        namespace,
        description: args.description,
        mc_version,
        dir,
    })?;

    println!();
    println!("{} Your datapack project is ready.", "Done!".green().bold());
    println!();
    println!("  cd {}", args.name.white().bold());
    println!("  {} edit src/lib.rs, then run `sand build`", "#".dimmed());
    Ok(())
}

// ── `sand init` ───────────────────────────────────────────────────────────────

fn cmd_init(args: InitArgs) -> Result<()> {
    let dir = std::env::current_dir()?;

    // Derive the project name from the directory name.
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my_pack")
        .to_string();
    validate_name(&name)?;

    // Refuse to init if a sand.toml already exists.
    if dir.join("sand.toml").exists() {
        bail!(
            "sand.toml already exists in '{}'. \
             Remove it or use a different directory.",
            dir.display()
        );
    }

    let namespace = name_to_namespace(&name);
    let mc_version = resolve_mc_version(args.mc_version)?;

    println!(
        "{} {} (Minecraft {})...",
        "Initializing".cyan().bold(),
        name.white().bold(),
        mc_version.yellow()
    );

    scaffold::scaffold(&ScaffoldOptions {
        name,
        namespace,
        description: args.description,
        mc_version,
        dir,
    })?;

    println!();
    println!("{} Your datapack project is initialized.", "Done!".green().bold());
    Ok(())
}

// ── `sand clean` ──────────────────────────────────────────────────────────────

fn cmd_clean(also_cargo: bool) -> Result<()> {
    let dist = PathBuf::from("dist");
    if dist.exists() {
        std::fs::remove_dir_all(&dist)
            .with_context(|| format!("failed to remove '{}'", dist.display()))?;
        println!("{} {}", "Removed".cyan().bold(), dist.display().to_string().white().bold());
    } else {
        println!("{} dist/ does not exist, nothing to remove", "Note:".dimmed());
    }

    if also_cargo {
        let status = std::process::Command::new("cargo")
            .arg("clean")
            .status()
            .context("failed to invoke `cargo clean`")?;
        if !status.success() {
            bail!("`cargo clean` failed");
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve the MC version: use the user-supplied value, or fetch the latest
/// release from Mojang (falling back to the hardcoded default if offline).
fn resolve_mc_version(supplied: Option<String>) -> Result<String> {
    match supplied {
        Some(v) => Ok(v),
        None => {
            println!("{}", "Fetching latest Minecraft version from Mojang...".dimmed());
            Ok(sand_build::latest_release_version())
        }
    }
}
