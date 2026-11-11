//! # The Sand language
//!
//! Sand source (`.sand` files) is a Rust-flavoured language for Minecraft Java
//! datapacks. It is a second entry point into Sand, next to the Rust API:
//! instead of running Rust code at build time to assemble commands, authors
//! write gameplay directly, and every `if`, `for`, and assignment *is* runtime
//! behaviour in Minecraft.
//!
//! ```text
//! #![pack(namespace = "demo", minecraft = "26.2")]
//!
//! player state Hero {
//!     #[clamp(0..=100)]
//!     energy: i32 = 100,
//! }
//!
//! #[tick]
//! fn regenerate() {
//!     for player in players() {
//!         if Hero.energy < 100 {
//!             Hero.energy += 1;
//!         }
//!     }
//! }
//! ```
//!
//! The pipeline is:
//!
//! 1. [`parser`] — a [pest](https://pest.rs) grammar with item- and
//!    statement-level error recovery produces a span-carrying [`ast`].
//! 2. [`package`] — loads the root file and every `mod` it declares, using
//!    Rust's module file layout.
//! 3. [`analysis`] — resolves names and `use` paths, type checks constants and
//!    enums, infers whether each function needs a player, and lowers the
//!    package into Sand's portable
//!    [`Program`](sand::advanced::compiler::Program).
//! 4. The portable [`Compiler`](sand::advanced::compiler::Compiler) validates
//!    the program and emits a deterministic datapack, exactly as it does for
//!    Rust and JSON frontends.
//!
//! [`ide`] answers editor queries (hover, definition, references, rename,
//! completion, symbols, inlay hints) over the same [`Analysis`]; the
//! `sand-lsp` crate exposes them over the Language Server Protocol.

pub mod analysis;
pub mod ast;
pub mod builtins;
pub mod diagnostic;
pub mod ide;
pub mod package;
pub mod parser;
mod position;
pub mod source;
pub mod vanilla;

pub use analysis::{Analysis, Context, Def, DefId, DefInfo, DefKind, Reference, Ty, analyze};
pub use diagnostic::{Diagnostic, Severity};
pub use package::{Package, ROOT_FILE, find_root, load};
pub use source::{DiskFiles, FileId, FileProvider, LineIndex, OverlayFiles};

use std::path::Path;

/// Load and analyse the package rooted at `root`.
///
/// `root` is the package root file, conventionally [`ROOT_FILE`]. Use
/// [`find_root`] to discover it from any source file in the package.
pub fn analyze_path(root: &Path, files: &dyn FileProvider) -> Analysis {
    analyze(load(root, files))
}
