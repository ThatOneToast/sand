mod add_cmd;
mod build_cmd;
mod config;
mod join_cmd;
mod pack_format;
mod run_cmd;
mod scaffold;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use colored::Colorize;

use scaffold::{ScaffoldOptions, name_to_namespace, validate_name};

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
        /// Also build the resource pack and write output to dist/<namespace>-resources/
        ///
        /// Requires a [resourcepack] section in sand.toml and a
        /// src/bin/sand_resource_export.rs binary in your project.
        /// Run `sand add resourcepack` to add these automatically.
        #[arg(long)]
        resourcepack: bool,
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
    /// **Requires Prism Launcher**
    /// Either join the local dev server started by `sand run` or join the sand-dev world with the datapack + optional resource pack
    Join {
        /// Join the local dev server started by `sand run`
        #[arg(long)]
        local: bool,
        /// Join the sand-dev world with the datapack + optional resource pack
        #[arg(long)]
        singleplayer: bool,
    },
    /// Remove build artifacts (dist/ and optionally Cargo target/)
    Clean {
        /// Also run `cargo clean` to remove the Cargo target directory
        #[arg(long)]
        cargo: bool,
        /// Also remove the dist/server/ directory created by `sand run`
        #[arg(long)]
        server: bool,
    },
    /// Add features to an existing Sand project
    Add(AddArgs),
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

    /// Scaffold with resource pack support enabled from the start
    ///
    /// Adds sand-resourcepack dependency, a sand_resource_export binary,
    /// a [resourcepack] section in sand.toml, and the __sand_resource_export
    /// hook in src/lib.rs.
    #[arg(long)]
    resourcepack: bool,
}

#[derive(clap::Args)]
struct InitArgs {
    /// Target Minecraft version [default: latest release]
    #[arg(long)]
    mc_version: Option<String>,

    /// Short description of the datapack
    #[arg(long, default_value = "A Minecraft datapack built with Sand")]
    description: String,

    /// Scaffold with resource pack support enabled from the start
    ///
    /// Adds sand-resourcepack dependency, a sand_resource_export binary,
    /// a [resourcepack] section in sand.toml, and the __sand_resource_export
    /// hook in src/lib.rs.
    #[arg(long)]
    resourcepack: bool,
}

#[derive(clap::Args)]
struct AddArgs {
    #[command(subcommand)]
    feature: AddFeature,
}

#[derive(Subcommand)]
enum AddFeature {
    /// Add resource pack support to an existing Sand project
    ///
    /// Modifies the project in-place:
    ///   - Cargo.toml: adds sand-resourcepack dep, resourcepack feature on
    ///     sand-macros, and a [[bin]] sand_resource_export target
    ///   - sand.toml: adds a [resourcepack] section
    ///   - src/bin/sand_resource_export.rs: created if absent
    ///   - src/lib.rs: appends __sand_resource_export hook if absent
    ///   - src/assets/: created if absent
    Resourcepack,
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
        Commands::Build {
            release,
            resourcepack,
        } => build_cmd::run(release, resourcepack),
        Commands::Run {
            ram,
            offline,
            no_build,
        } => run_cmd::run(run_cmd::RunArgs {
            ram,
            offline,
            no_build,
        }),
        Commands::Join {
            local,
            singleplayer,
        } => join_cmd::run(join_cmd::JoinArgs {
            local,
            singleplayer,
        }),
        Commands::Clean { cargo, server } => cmd_clean(cargo, server),
        Commands::Add(args) => match args.feature {
            AddFeature::Resourcepack => add_cmd::run_resourcepack(),
        },
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
        "{} {} (Minecraft {}{})...",
        "Creating".cyan().bold(),
        args.name.white().bold(),
        mc_version.yellow(),
        if args.resourcepack {
            " + resourcepack".cyan().to_string()
        } else {
            String::new()
        }
    );

    scaffold::scaffold(&ScaffoldOptions {
        name: args.name.clone(),
        namespace,
        description: args.description,
        mc_version,
        dir,
        resourcepack: args.resourcepack,
    })?;

    println!();
    println!("{} Your datapack project is ready.", "Done!".green().bold());
    println!();
    println!("  cd {}", args.name.white().bold());
    if args.resourcepack {
        println!(
            "  {} edit src/lib.rs, add assets to src/assets/, then run `sand build --resourcepack`",
            "#".dimmed()
        );
    } else {
        println!("  {} edit src/lib.rs, then run `sand build`", "#".dimmed());
    }
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
        "{} {} (Minecraft {}{})...",
        "Initializing".cyan().bold(),
        name.white().bold(),
        mc_version.yellow(),
        if args.resourcepack {
            " + resourcepack".cyan().to_string()
        } else {
            String::new()
        }
    );

    scaffold::scaffold(&ScaffoldOptions {
        name,
        namespace,
        description: args.description,
        mc_version,
        dir,
        resourcepack: args.resourcepack,
    })?;

    println!();
    println!(
        "{} Your datapack project is initialized.",
        "Done!".green().bold()
    );
    Ok(())
}

// ── `sand clean` ──────────────────────────────────────────────────────────────

fn cmd_clean(also_cargo: bool, also_server: bool) -> Result<()> {
    let dist = PathBuf::from("dist");
    let server_dir = dist.join("server");

    if dist.exists() {
        if also_server {
            // Remove everything including dist/server/.
            std::fs::remove_dir_all(&dist)
                .with_context(|| format!("failed to remove '{}'", dist.display()))?;
            println!(
                "{} {}",
                "Removed".cyan().bold(),
                dist.display().to_string().white().bold()
            );
        } else if server_dir.exists() {
            // Remove everything in dist/ except server/.
            for entry in std::fs::read_dir(&dist).context("failed to read dist/")? {
                let entry = entry?;
                let path = entry.path();
                if path == server_dir {
                    continue;
                }
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)
                        .with_context(|| format!("failed to remove '{}'", path.display()))?;
                } else {
                    std::fs::remove_file(&path)
                        .with_context(|| format!("failed to remove '{}'", path.display()))?;
                }
            }
            println!(
                "{} {} (kept dist/server/; use {} to remove it)",
                "Removed".cyan().bold(),
                "dist/*".white().bold(),
                "--server".yellow()
            );
        } else {
            // No server dir, just remove everything.
            std::fs::remove_dir_all(&dist)
                .with_context(|| format!("failed to remove '{}'", dist.display()))?;
            println!(
                "{} {}",
                "Removed".cyan().bold(),
                dist.display().to_string().white().bold()
            );
        }
    } else {
        println!(
            "{} dist/ does not exist, nothing to remove",
            "Note:".dimmed()
        );
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
            println!(
                "{}",
                "Fetching latest Minecraft version from Mojang...".dimmed()
            );
            Ok(sand_build::latest_release_version())
        }
    }
}
