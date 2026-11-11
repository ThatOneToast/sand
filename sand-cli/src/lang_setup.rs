//! `sand lang new` and `sand lang zed`: project scaffolding and editor setup.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Files of the Zed extension, embedded so setup works from any install.
const ZED_FILES: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../../editors/zed/Cargo.toml")),
    ("src/lib.rs", include_str!("../../editors/zed/src/lib.rs")),
    (
        "languages/sand/config.toml",
        include_str!("../../editors/zed/languages/sand/config.toml"),
    ),
    (
        "languages/sand/highlights.scm",
        include_str!("../../editors/zed/languages/sand/highlights.scm"),
    ),
    (
        "languages/sand/brackets.scm",
        include_str!("../../editors/zed/languages/sand/brackets.scm"),
    ),
    (
        "languages/sand/indents.scm",
        include_str!("../../editors/zed/languages/sand/indents.scm"),
    ),
    (
        "languages/sand/outline.scm",
        include_str!("../../editors/zed/languages/sand/outline.scm"),
    ),
];

/// The generated tree-sitter grammar Zed compiles for highlighting.
const GRAMMAR_FILES: &[(&str, &str)] = &[
    (
        "grammar.js",
        include_str!("../../editors/tree-sitter-sand/grammar.js"),
    ),
    (
        "tree-sitter.json",
        include_str!("../../editors/tree-sitter-sand/tree-sitter.json"),
    ),
    (
        "src/parser.c",
        include_str!("../../editors/tree-sitter-sand/src/parser.c"),
    ),
    (
        "src/grammar.json",
        include_str!("../../editors/tree-sitter-sand/src/grammar.json"),
    ),
    (
        "src/node-types.json",
        include_str!("../../editors/tree-sitter-sand/src/node-types.json"),
    ),
    (
        "src/tree_sitter/parser.h",
        include_str!("../../editors/tree-sitter-sand/src/tree_sitter/parser.h"),
    ),
    (
        "src/tree_sitter/alloc.h",
        include_str!("../../editors/tree-sitter-sand/src/tree_sitter/alloc.h"),
    ),
    (
        "src/tree_sitter/array.h",
        include_str!("../../editors/tree-sitter-sand/src/tree_sitter/array.h"),
    ),
    (
        "queries/highlights.scm",
        include_str!("../../editors/tree-sitter-sand/queries/highlights.scm"),
    ),
];

/// Where `sand lang zed` installs by default: `~/.sand/zed`.
pub fn default_zed_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .context("cannot find your home directory; pass --dir")?;
    Ok(PathBuf::from(home).join(".sand").join("zed"))
}

fn write_files(root: &Path, files: &[(&str, &str)]) -> Result<()> {
    for (relative, contents) in files {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        // A fixed local identity keeps setup independent of user git config.
        .args(["-c", "user.name=Sand", "-c", "user.email=sand@localhost"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .context("run git (Zed fetches grammars with git, so git must be installed)")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// What `install_zed` produced.
pub struct ZedInstall {
    /// The extension directory to select in Zed.
    pub extension: PathBuf,
    /// The local grammar repository.
    pub grammar: PathBuf,
    /// The pinned grammar commit.
    pub revision: String,
}

/// Write the Zed extension and a local, committed grammar repository.
///
/// Zed fetches grammars with git at an exact revision, so the grammar is
/// committed into its own repository and that commit is pinned in
/// `extension.toml`. Re-running updates both in place.
pub fn install_zed(dir: &Path) -> Result<ZedInstall> {
    let grammar = dir.join("tree-sitter-sand");
    let extension = dir.join("zed-sand");
    write_files(&grammar, GRAMMAR_FILES)?;
    if !grammar.join(".git").exists() {
        git(&grammar, &["init", "--quiet"])?;
    }
    git(&grammar, &["add", "--all"])?;
    let changed = !git(&grammar, &["status", "--porcelain"])?.is_empty();
    let has_head = git(&grammar, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_ok();
    if changed || !has_head {
        git(
            &grammar,
            &["commit", "--quiet", "--message", "Sand tree-sitter grammar"],
        )?;
    }
    let revision = git(&grammar, &["rev-parse", "HEAD"])?;

    write_files(&extension, ZED_FILES)?;
    let grammar_url = format!("file://{}", std::fs::canonicalize(&grammar)?.display());
    let manifest = format!(
        r#"id = "sand"
name = "Sand"
version = "0.1.0"
schema_version = 1
authors = ["Sand contributors"]
description = "Sand datapack language: highlighting and the Sand language server."
repository = "https://github.com/ThatOneToast/sand"

[grammars.sand]
repository = "{grammar_url}"
rev = "{revision}"

[language_servers.sand]
name = "Sand Language Server"
languages = ["Sand"]
"#
    );
    std::fs::write(extension.join("extension.toml"), manifest)?;
    Ok(ZedInstall {
        extension,
        grammar,
        revision,
    })
}

/// Create a new Sand language project.
pub fn new_project(dir: &Path, namespace: &str, minecraft: &str) -> Result<()> {
    if dir.exists() && dir.read_dir()?.next().is_some() {
        bail!(
            "directory '{}' already exists and is not empty",
            dir.display()
        );
    }
    let title = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| namespace.to_string());
    let pack = PACK_TEMPLATE
        .replace("@TITLE@", &title)
        .replace("@NAMESPACE@", namespace)
        .replace("@MINECRAFT@", minecraft);
    let readme = README_TEMPLATE
        .replace("@TITLE@", &title)
        .replace("@NAMESPACE@", namespace);
    write_files(
        dir,
        &[
            ("pack.sand", &pack),
            ("rewards.sand", REWARDS_TEMPLATE),
            ("shop.sand", SHOP_TEMPLATE),
            ("README.md", &readme),
            (".gitignore", "/dist/\n"),
        ],
    )
}

const PACK_TEMPLATE: &str = r#"//! @TITLE@: a datapack written in the Sand language.
#![pack(namespace = "@NAMESPACE@", minecraft = "@MINECRAFT@")]

mod rewards;
mod shop;

/// Coins each player has collected.
pub player state Wallet {
    #[clamp(0..=9999)]
    coins: number,
}

/// Greet players the first time they join.
#[on(first_join)]
fn welcome() {
    chat!("{gold}Welcome, {player}!{reset} You start with {Wallet.coins} coins.");
    give_item(Item::Bread, 4);
}

/// Start the payday timer whenever the pack loads.
#[load]
fn start() {
    schedule(payday, seconds = 60);
}

/// Pay every online player, then wait for the next payday.
fn payday() {
    for player in players() {
        rewards::pay();
    }
    schedule(payday, seconds = 60);
}
"#;

const REWARDS_TEMPLATE: &str = r#"//! Coin rewards.

use crate::Wallet;

/// Give the executing player one coin and tell them about it.
pub fn pay() {
    Wallet.coins += 1;
    actionbar!("{yellow}+1 coin {gray}({Wallet.coins} total)");
    play_sound(Sound::EntityExperienceOrbPickup);
}
"#;

const SHOP_TEMPLATE: &str = r#"//! Spend coins on extra hearts.

use crate::Wallet;

/// Max health is measured in half-hearts: 20 is ten hearts.
const MAX_HEALTH: number = 40;

/// The price of the next heart, rising with each one bought.
fn heart_price(extra_hearts: number) -> number {
    return 50 + extra_hearts * 25;
}

/// Right-click a carrot on a stick to buy one heart of max health.
#[on(use_item = Item::CarrotOnAStick)]
fn buy_heart() {
    let health = get_attribute(Attribute::MaxHealth);
    if health >= MAX_HEALTH {
        actionbar!("{gold}You already have the most hearts possible.");
        return;
    }
    let price = heart_price((health - 20) / 2);
    if Wallet.coins < price {
        chat!("{red}A heart costs {price} coins; you have {Wallet.coins}.");
        return;
    }
    Wallet.coins -= price;
    set_attribute(Attribute::MaxHealth, health + 2);
    play_sound(Sound::EntityPlayerLevelup);
    chat!("{green}Bought a heart for {price} coins!");
}
"#;

const README_TEMPLATE: &str = r#"# @TITLE@

A Minecraft datapack written in the Sand language.

```sh
sand lang check     # report errors and warnings
sand lang build     # write the datapack to dist/@NAMESPACE@
sand lang run       # compile and launch a local test server
```

Copy `dist/@NAMESPACE@` into a world's `datapacks` folder and run `/reload`.

Editor support: run `sand lang zed` once, then install the printed folder
with Zed's `zed: install dev extension` command.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zed_queries_match_the_grammar_queries() {
        assert_eq!(
            include_str!("../../editors/zed/languages/sand/highlights.scm"),
            include_str!("../../editors/tree-sitter-sand/queries/highlights.scm"),
        );
    }

    #[test]
    fn zed_install_pins_a_committed_grammar_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let first = install_zed(dir.path()).unwrap();
        let manifest = std::fs::read_to_string(first.extension.join("extension.toml")).unwrap();
        assert!(manifest.contains(&format!("rev = \"{}\"", first.revision)));
        assert!(manifest.contains("[language_servers.sand]"));
        assert!(first.extension.join("languages/sand/config.toml").is_file());
        assert!(first.grammar.join("src/parser.c").is_file());
        assert_eq!(
            git(&first.grammar, &["status", "--porcelain"]).unwrap(),
            "",
            "grammar files are committed"
        );
        let second = install_zed(dir.path()).unwrap();
        assert_eq!(first.revision, second.revision);
    }

    #[test]
    fn new_projects_analyse_and_compile_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("arena");
        new_project(&project, "arena", "26.2").unwrap();
        let analysis = sand_lang::analyze_path(&project.join("pack.sand"), &sand_lang::DiskFiles);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:#?}",
            analysis.diagnostics
        );
        analysis.compile().unwrap();
        assert!(
            new_project(&project, "arena", "26.2").is_err(),
            "refuses non-empty dirs"
        );
    }
}
