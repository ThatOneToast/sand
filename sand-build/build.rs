//! Embeds an automatic, deterministic fingerprint of this crate's own
//! generator implementation source tree into the compiled `sand-build`
//! crate.
//!
//! `sand-build`'s generated-Rust cache (`src/codegen/cache.rs`) must
//! invalidate whenever the *generator logic itself* changes -- editing
//! `codegen/registries.rs`, or adding a brand new `codegen/foo.rs` module,
//! must never leave a stale cache entry silently in place (issue #347 PR
//! #348 review). Recursively hashing `src/codegen/`'s entire source tree at
//! *compile time* (see `source_tree_fingerprint.rs`) and baking the result
//! into a `pub const` removes the human step entirely: no manually
//! maintained file list to forget to update when a new generator module is
//! added, and no version constant to remember to bump.
//!
//! `source_tree_fingerprint.rs` is included here via `#[path]` rather than
//! depended on normally, because `build.rs` cannot link against
//! `sand-build`'s own not-yet-built library artifact (the classic
//! build-script chicken-and-egg problem) -- but it *can* pull in an extra
//! sibling source file as its own separate compilation. That file is also a
//! normal module of the `sand-build` library (see `lib.rs`), so `cargo
//! test` exercises the *exact* code this build script runs, not a parallel
//! reimplementation that could drift from it.

#[path = "src/source_tree_fingerprint.rs"]
mod source_tree_fingerprint;

use std::path::Path;

/// The generator implementation source tree, relative to this crate's
/// manifest directory. Every `.rs` file recursively under here participates
/// in `CODEGEN_IMPL_FINGERPRINT` -- adding a new file, removing one, moving
/// one, or editing one all change the fingerprint automatically.
const CODEGEN_IMPL_DIR: &str = "src/codegen";

fn main() {
    let codegen_dir = Path::new(CODEGEN_IMPL_DIR);

    // Watching the directory itself (not just today's files) is what makes
    // Cargo rerun this build script when a file is *added to* or *removed
    // from* the generator tree, not only when an existing file's content
    // changes.
    println!("cargo:rerun-if-changed={}", codegen_dir.display());

    let fingerprint = source_tree_fingerprint::hash_source_tree(codegen_dir)
        .unwrap_or_else(|error| panic!("failed to fingerprint {}: {error}", codegen_dir.display()));

    // Explicit files are also individually watched so editors/tools that
    // only see per-file rerun-if-changed directives (rather than resolving
    // the directory watch) still behave -- redundant with the directory
    // watch above for Cargo itself, but harmless.
    let files = source_tree_fingerprint::discover_rust_files(codegen_dir)
        .unwrap_or_else(|error| panic!("failed to discover {}: {error}", codegen_dir.display()));
    for (_, path) in &files {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("codegen_impl_fingerprint.rs");
    std::fs::write(
        &dest,
        format!(
            "/// Automatic fingerprint of `sand-build`'s codegen generator \
             implementation source tree ({CODEGEN_IMPL_DIR}/**/*.rs), computed \
             at compile time by `build.rs`. Changes whenever any file under \
             that tree is added, removed, renamed, or edited -- never \
             requires a manual version bump or file-list update.\n\
             pub const CODEGEN_IMPL_FINGERPRINT: &str = {fingerprint:?};\n"
        ),
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", dest.display()));
}
