//! `sand lang` and `sand lsp`: the Sand language frontend.
//!
//! Sand source compiles through the same portable compiler as JSON programs,
//! so `sand lang build` needs neither Cargo nor `sand.toml`.

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use colored::Colorize;
use sand_lang::library::Library;
use sand_lang::{Analysis, Diagnostic, DiskFiles, Severity};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct LangArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run Sand-authored tests in a disposable Minecraft world
    Test {
        #[command(flatten)]
        input: Input,
        /// Discover tests without launching Minecraft
        #[arg(long)]
        list: bool,
        /// Use a specific server JAR instead of the verified version cache
        #[arg(long)]
        jar: Option<PathBuf>,
        /// Accept the Minecraft EULA for this test server
        #[arg(long)]
        accept_eula: bool,
        /// Maximum seconds for startup, each test, and shutdown
        #[arg(long, default_value_t = 120, value_parser = clap::value_parser!(u64).range(1..=3600))]
        timeout: u64,
    },
    /// Vendor a bundled Sand library into an existing package
    Add {
        /// Library to install (collections, outcomes, spatial, or time)
        library: Library,
        #[command(flatten)]
        input: Input,
    },
    /// Format every source file in a Sand package
    Fmt {
        #[command(flatten)]
        input: Input,
        /// Report unformatted files without modifying them
        #[arg(long)]
        check: bool,
    },
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
    /// Print source locations for emitted commands and portable operations as JSON
    SourceMap(Input),
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
        Command::Test {
            input,
            list,
            jar,
            accept_eula,
            timeout,
        } => {
            let root = package_root(&input.path)?;
            let analysis = sand_lang::analyze_tests(sand_lang::load(&root, &DiskFiles));
            report(&analysis)?;
            if list || analysis.tests.is_empty() {
                for test in &analysis.tests {
                    println!("{}", test.name);
                }
                println!("{} test(s)", analysis.tests.len());
                return Ok(());
            }
            crate::lang_test::run(
                &analysis,
                jar,
                accept_eula,
                std::time::Duration::from_secs(timeout),
            )
        }
        Command::Add { library, input } => {
            let installed = install_library(&input.path, library)?;
            println!("{} {}", "Added".green().bold(), installed.display());
            let import = match library {
                Library::Collections => "use collections::{Map, Set};",
                Library::Outcomes => "use outcomes::{Option, Result};",
                Library::Spatial => "use spatial::Position;",
                Library::Time => "use time::Duration;",
            };
            println!("  Import with {import}");
            Ok(())
        }
        Command::Fmt { input, check } => format_package(&input.path, check),
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
        Command::SourceMap(input) => {
            let analysis = analyze(&input.path)?;
            report(&analysis)?;
            let map = analysis
                .source_map()
                .map_err(|errors| report_compilation_errors(&analysis, errors))?;
            println!("{}", serde_json::to_string_pretty(&map)?);
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

/// Format all declared source files, or check their formatting without writes.
///
/// Every file is parsed and formatted before any file is changed. Semantic
/// errors do not prevent formatting, but syntax and module-loading errors do.
pub fn format_package(path: &Path, check: bool) -> Result<()> {
    let analysis = analyze(path)?;
    if !analysis.package.diagnostics.is_empty() {
        for diagnostic in &analysis.package.diagnostics {
            eprint!("{}", diagnostic.render(&analysis.package));
        }
        bail!("cannot format a package with syntax or module-loading errors");
    }
    let mut changed = Vec::new();
    for file in &analysis.package.files {
        let formatted = sand_lang::format::format(&file.text)
            .map_err(|_| anyhow::anyhow!("cannot format {}: syntax errors", file.path.display()))?;
        if formatted != file.text {
            changed.push((file.path.clone(), formatted));
        }
    }
    if check {
        for (path, _) in &changed {
            println!("Would format {}", path.display());
        }
        if !changed.is_empty() {
            bail!("{} source file(s) need formatting", changed.len());
        }
    } else {
        for (path, text) in &changed {
            std::fs::write(path, text)
                .with_context(|| format!("cannot write {}", path.display()))?;
            println!("Formatted {}", path.display());
        }
    }
    Ok(())
}

/// Compile a package and publish it; returns its namespace.
fn build(analysis: &Analysis, output: Option<PathBuf>) -> Result<String> {
    let pack = analysis
        .compile()
        .map_err(|errors| report_compilation_errors(analysis, errors))?;
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

fn package_root(path: &Path) -> Result<PathBuf> {
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
    Ok(root)
}

fn analyze(path: &Path) -> Result<Analysis> {
    Ok(sand_lang::analyze_path(&package_root(path)?, &DiskFiles))
}

fn imported_name(tree: &sand_lang::ast::UseTree, name: &str) -> bool {
    if let Some(group) = &tree.group {
        return group.iter().any(|tree| imported_name(tree, name));
    }
    tree.alias.as_ref().unwrap_or_else(|| tree.path.last()).name == name
}

/// Install a normal, editable source module without replacing existing code.
fn install_library(path: &Path, library: Library) -> Result<PathBuf> {
    use sand_lang::ast::ItemKind;
    let root = package_root(path)?;
    let parent = root
        .parent()
        .context("package root has no parent directory")?;
    let original = std::fs::read_to_string(&root)?;
    let (source, errors) = sand_lang::parser::parse(&original);
    anyhow::ensure!(
        errors.is_empty(),
        "fix package-root syntax errors before adding a library"
    );
    let name = library.name();
    let target = parent.join(format!("{name}.sand"));
    anyhow::ensure!(
        !target.exists() && !parent.join(name).exists(),
        "{} already exists; no files were changed",
        target.display()
    );
    let mut declared = false;
    for item in &source.items {
        if item.name().is_some_and(|item| item.name == name) {
            match &item.kind {
                ItemKind::Mod(module) if module.inline.is_none() => declared = true,
                _ => bail!(
                    "`{name}` is already declared in {}; no files were changed",
                    root.display()
                ),
            }
        }
        if let ItemKind::Use(import) = &item.kind {
            anyhow::ensure!(
                !imported_name(&import.tree, name),
                "`{name}` is already imported in {}; no files were changed",
                root.display()
            );
        }
    }
    let mut contents = original.clone();
    if !declared {
        if !contents.ends_with('\n') {
            contents.push('\n');
        }
        contents.push_str(&format!("\nmod {name};\n"));
    }
    let mut module = tempfile::NamedTempFile::new_in(parent)?;
    module.write_all(library.source().as_bytes())?;
    module
        .persist_noclobber(&target)
        .with_context(|| format!("install {}", target.display()))?;
    let result = (|| -> Result<()> {
        anyhow::ensure!(
            std::fs::read_to_string(&root)? == original,
            "package root changed during library installation; retry with the latest source"
        );
        if !declared {
            let mut staged = tempfile::NamedTempFile::new_in(parent)?;
            staged.write_all(contents.as_bytes())?;
            staged
                .as_file()
                .set_permissions(std::fs::metadata(&root)?.permissions())?;
            staged
                .persist(&root)
                .context("install library module declaration")?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&target);
        return Err(error);
    }
    Ok(target)
}

fn report_compilation_errors(analysis: &Analysis, errors: Vec<Diagnostic>) -> anyhow::Error {
    for error in &errors {
        eprint!("{}", error.render(&analysis.package));
    }
    anyhow::anyhow!("compilation failed with {} error(s)", errors.len())
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

#[cfg(test)]
mod library_tests {
    use super::*;

    #[test]
    fn installing_collections_adds_a_normal_module_and_compiles_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pack.sand");
        std::fs::write(&root, "#![pack(namespace = \"maps\", minecraft = \"26.2\")]\nuse collections::{Map, Set};\nfn run() { let mut map = Map::<string, number>::new(); map.insert(\"a\", 1); let set = Set::<number>::new(); }\n").unwrap();
        let library = install_library(dir.path(), Library::Collections).unwrap();
        assert_eq!(
            std::fs::read_to_string(&library).unwrap(),
            Library::Collections.source()
        );
        let source = std::fs::read_to_string(&root).unwrap();
        assert_eq!(source.matches("mod collections;").count(), 1);
        let analysis = analyze(dir.path()).unwrap();
        assert!(!analysis.has_errors(), "{:?}", analysis.diagnostics);
        analysis.compile().unwrap();
        assert!(install_library(dir.path(), Library::Collections).is_err());
        assert_eq!(std::fs::read_to_string(&root).unwrap(), source);
    }

    #[test]
    fn installation_preserves_existing_source_and_accepts_a_missing_declared_module() {
        for source in [
            "#![pack(namespace = \"maps\")] struct collections { x: number }",
            "#![pack(namespace = \"maps\")] use other as collections;",
            "#![pack(namespace = \"maps\")] mod collections {}",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("pack.sand");
            std::fs::write(&root, source).unwrap();
            assert!(install_library(dir.path(), Library::Collections).is_err());
            assert_eq!(std::fs::read_to_string(&root).unwrap(), source);
            assert!(!dir.path().join("collections.sand").exists());
        }
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pack.sand");
        let source = "#![pack(namespace = \"maps\")] mod collections;\n";
        std::fs::write(&root, source).unwrap();
        install_library(&root, Library::Collections).unwrap();
        assert_eq!(std::fs::read_to_string(root).unwrap(), source);
    }
}
