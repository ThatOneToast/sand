//! `sand lang` and `sand lsp`: the Sand language frontend.
//!
//! Sand source compiles through the same portable compiler as JSON programs,
//! so `sand lang build` needs neither Cargo nor `sand.toml`.

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use colored::Colorize;
use sand_lang::{Analysis, DiskFiles, Severity};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct LangArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Type-check a Sand language package without writing files
    Check(Input),
    /// Compile a Sand language package into a datapack directory
    Build {
        #[command(flatten)]
        input: Input,
        /// Output directory; defaults to dist/<namespace> next to the root file
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Compile the package and launch a local Minecraft server to test it
    Run {
        #[command(flatten)]
        input: Input,
        /// JVM heap size, e.g. "4G" or "2048M"
        #[arg(long, default_value = "4G")]
        ram: String,
        /// Set online-mode=false in server.properties (easier local testing)
        #[arg(long)]
        offline: bool,
        /// Skip compiling; launch whatever is already in dist/
        #[arg(long)]
        no_build: bool,
        /// How to present the Minecraft server's log
        #[arg(long, value_enum, default_value = "classified")]
        server_log: crate::console::OutputMode,
    },
    /// Print the portable program JSON the package lowers to
    Emit(Input),
    /// Create a new Sand language project in a new directory
    New {
        /// Project directory (its name becomes the namespace)
        path: PathBuf,
        /// Namespace for generated resources; defaults to the directory name
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Install Sand language support for the Zed editor
    Zed {
        /// Where to write the extension; defaults to ~/.sand/zed
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

#[derive(Args)]
struct Input {
    /// Package root file, any file in the package, or a directory containing pack.sand
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Run a `sand lang` subcommand.
pub fn run(args: LangArgs) -> Result<()> {
    match args.command {
        Command::Check(input) => {
            let analysis = analyze(&input.path)?;
            let diagnostics = report(&analysis)?;
            println!(
                "{} {} ({diagnostics})",
                "Checked".green().bold(),
                analysis.package.root.display()
            );
            Ok(())
        }
        Command::Build { input, output } => {
            let analysis = analyze(&input.path)?;
            report(&analysis)?;
            build(&analysis, output)?;
            Ok(())
        }
        Command::Run {
            input,
            ram,
            offline,
            no_build,
            server_log,
        } => run_project(
            &input.path,
            crate::run_cmd::RunArgs {
                ram,
                offline,
                no_build,
                server_log,
                profile: "dev".into(),
            },
        ),
        Command::New { path, namespace } => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .context("project path needs a final directory name")?;
            let namespace = namespace.unwrap_or_else(|| {
                name.to_ascii_lowercase()
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect()
            });
            let valid = namespace
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase())
                && namespace
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
            if !valid {
                bail!(
                    "`{namespace}` is not a valid namespace; pass --namespace with lowercase letters, digits, and `_`"
                );
            }
            let minecraft = sand::advanced::compiler::Compiler::capabilities()["targets"]
                .as_array()
                .and_then(|targets| targets.last())
                .and_then(|target| target.as_str())
                .unwrap_or("26.2")
                .to_string();
            crate::lang_setup::new_project(&path, &namespace, &minecraft)?;
            println!(
                "{} {} (namespace `{namespace}`, Minecraft {minecraft})",
                "Created".green().bold(),
                path.display()
            );
            println!("  cd {} && sand lang build", path.display());
            Ok(())
        }
        Command::Zed { dir } => {
            let dir = match dir {
                Some(dir) => dir,
                None => crate::lang_setup::default_zed_dir()?,
            };
            let install = crate::lang_setup::install_zed(&dir)?;
            let short = &install.revision[..install.revision.len().min(12)];
            println!(
                "{} Zed extension → {}",
                "Wrote".green().bold(),
                install.extension.display()
            );
            println!(
                "      grammar → {} (commit {short})",
                install.grammar.display()
            );
            println!();
            println!("Finish in Zed (one time):");
            println!("  1. Open the command palette (cmd-shift-p / ctrl-shift-p).");
            println!("  2. Run `zed: install dev extension`.");
            println!("  3. Choose {}", install.extension.display());
            println!();
            println!(
                "Zed builds the extension with Rust (rustup) and starts `sand lsp` for .sand files."
            );
            println!(
                "After upgrading Sand, run `sand lang zed` again and choose `Rebuild` on the Sand extension."
            );
            Ok(())
        }
        Command::Emit(input) => {
            let analysis = analyze(&input.path)?;
            report(&analysis)?;
            let program = analysis
                .program
                .as_ref()
                .context("the package did not lower to a program")?;
            println!("{}", serde_json::to_string_pretty(program)?);
            Ok(())
        }
    }
}

/// Run the Sand language server over stdio.
/// Compile a package and publish it; returns its namespace.
fn build(analysis: &Analysis, output: Option<PathBuf>) -> Result<String> {
    let pack = analysis.compile().map_err(|errors| {
        for error in &errors {
            eprint!("{}", error.render(&analysis.package));
        }
        anyhow::anyhow!("compilation failed with {} error(s)", errors.len())
    })?;
    let namespace = namespace_of(analysis);
    let default_output = output.is_none();
    let root = project_root(analysis);
    let output = output.unwrap_or_else(|| root.join("dist").join(&namespace));
    let published = crate::build::publication::publish_pack(&output, &pack.resources)?;
    println!(
        "{} {} for Minecraft {} → {} ({} written, {} unchanged, {} removed)",
        "Built".green().bold(),
        namespace,
        pack.target,
        output.display(),
        published.summary.written,
        published.summary.unchanged,
        published.summary.removed,
    );
    // Keep a local test world current, so an edit only needs `/reload`.
    let installed = root.join("dist/server/world/datapacks").join(&namespace);
    if default_output && installed.exists() {
        crate::run_cmd::sync_dir(&output, &installed)
            .with_context(|| format!("failed to update {}", installed.display()))?;
        println!(
            "  {} the test world's copy; run {} in-game to load it",
            "Updated".dimmed(),
            "/reload".white()
        );
    }
    Ok(namespace)
}

fn namespace_of(analysis: &Analysis) -> String {
    analysis
        .pack
        .as_ref()
        .map(|p| p.namespace.clone())
        .unwrap_or_else(|| "pack".into())
}

fn project_root(analysis: &Analysis) -> PathBuf {
    analysis
        .package
        .root
        .parent()
        .unwrap_or(Path::new("."))
        .to_path_buf()
}

/// Compile a package (unless `--no-build`) and launch the local test server.
/// Language projects need neither Cargo nor `sand.toml`.
pub(crate) fn run_project(path: &Path, args: crate::run_cmd::RunArgs) -> Result<()> {
    let analysis = analyze(path)?;
    report(&analysis)?;
    let namespace = if args.no_build {
        namespace_of(&analysis)
    } else {
        build(&analysis, None)?
    };
    let minecraft = analysis
        .program
        .as_ref()
        .map(|program| program.target.minecraft.to_string())
        .context("the package did not produce a program")?;
    let root = project_root(&analysis);
    std::env::set_current_dir(&root).with_context(|| format!("cannot enter {}", root.display()))?;
    crate::run_cmd::launch_built(&namespace, &minecraft, args)
}

pub fn run_lsp() -> Result<()> {
    sand_lsp::run_stdio().map_err(|error| anyhow::anyhow!("language server failed: {error}"))
}

fn analyze(path: &Path) -> Result<Analysis> {
    let path =
        std::fs::canonicalize(path).with_context(|| format!("cannot open {}", path.display()))?;
    let root = if path.is_dir() {
        let root = path.join(sand_lang::ROOT_FILE);
        if !root.is_file() {
            bail!(
                "{} has no {}; pass the package root file explicitly",
                path.display(),
                sand_lang::ROOT_FILE
            );
        }
        root
    } else {
        sand_lang::find_root(&path, &DiskFiles)
    };
    Ok(sand_lang::analyze_path(&root, &DiskFiles))
}

/// Print diagnostics; fail when any is an error. Returns a summary.
fn report(analysis: &Analysis) -> Result<String> {
    let mut errors = 0;
    let mut warnings = 0;
    for diagnostic in &analysis.diagnostics {
        match diagnostic.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        }
        eprintln!("{}", diagnostic.render(&analysis.package));
    }
    if errors > 0 {
        bail!("{errors} error(s), {warnings} warning(s)");
    }
    Ok(format!("{warnings} warning(s)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_updates_an_existing_test_world_only_when_there_is_one() {
        let dir = tempfile::tempdir().unwrap();
        // Publishing refuses symlinked paths, such as macOS's `/var`.
        let base = std::fs::canonicalize(dir.path()).unwrap();
        let project = base.join("arena");
        crate::lang_setup::new_project(&project, "arena", "26.2").unwrap();
        let analysis = analyze(&project).unwrap();
        assert_eq!(build(&analysis, None).unwrap(), "arena");
        let installed = project.join("dist/server/world/datapacks/arena");
        assert!(!installed.exists(), "no test world exists yet");

        std::fs::create_dir_all(&installed).unwrap();
        build(&analysis, None).unwrap();
        assert!(
            installed.join("pack.mcmeta").is_file(),
            "reload picks up the rebuilt pack"
        );

        // An explicit output is a separate destination, not the project's pack.
        std::fs::remove_dir_all(&installed).unwrap();
        std::fs::create_dir_all(&installed).unwrap();
        build(&analysis, Some(base.join("elsewhere"))).unwrap();
        assert!(!installed.join("pack.mcmeta").exists());
    }
}
