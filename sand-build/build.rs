//! Embeds an automatic, deterministic fingerprint of this crate's own
//! generator implementation source into the compiled `sand-build` crate.
//!
//! `sand-build`'s generated-Rust cache (`src/codegen/cache.rs`) must
//! invalidate whenever the *generator logic itself* changes — editing
//! `codegen/registries.rs` and forgetting to bump a manual version constant
//! must never leave a stale cache entry silently in place (issue #347 PR
//! #348 review). Hashing this crate's own generator source files at
//! *compile time* and baking the result into a `pub const` removes the
//! human step entirely: any byte changed in the generator's source changes
//! this constant, with no bump to remember.
//!
//! This uses the same SHA-1-based hashing algorithm as
//! `sand_build::fingerprint` (see `src/fingerprint.rs`), duplicated here
//! deliberately: `sand-build`'s own `lib.rs` isn't available yet while its
//! own `build.rs` runs, so this small, self-contained computation can't
//! call into the crate it is building. Keeping both algorithms byte-for-
//! byte identical (length-prefixed, SHA-1) means the embedded constant is
//! directly combinable with other `sand_build::fingerprint::combine` inputs
//! without a second hashing scheme.

use std::path::Path;

use sha1::{Digest, Sha1};

/// Every source file whose bytes can change what
/// [`sand_build::codegen::generate_all`] produces. Kept as one list so the
/// fingerprint and the actual generator logic can't silently drift apart --
/// if a new file is added to the generator, it must be added here too (a
/// unit test in `codegen::cache` guards this by asserting the fingerprint
/// changes whenever any of these files' content changes).
const CODEGEN_IMPL_FILES: &[&str] = &[
    "src/codegen/mod.rs",
    "src/codegen/registries.rs",
    "src/codegen/blocks.rs",
    "src/codegen/commands.rs",
];

fn main() {
    for file in CODEGEN_IMPL_FILES {
        println!("cargo:rerun-if-changed={file}");
    }

    let fingerprint = hash_files(CODEGEN_IMPL_FILES);

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("codegen_impl_fingerprint.rs");
    std::fs::write(
        &dest,
        format!(
            "/// Automatic fingerprint of `sand-build`'s codegen generator \
             implementation source, computed at compile time by `build.rs`. \
             Changes whenever any byte of {CODEGEN_IMPL_FILES:?} changes -- \
             never requires a manual version bump.\n\
             pub const CODEGEN_IMPL_FINGERPRINT: &str = {fingerprint:?};\n"
        ),
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", dest.display()));
}

/// Deterministic, order-sensitive, length-prefixed hash of the given files'
/// contents. Byte-for-byte the same algorithm as
/// `sand_build::fingerprint::combine` over each file's
/// `sand_build::fingerprint::hash_bytes` result, so the embedded constant
/// composes cleanly with other fingerprint inputs at runtime.
fn hash_files(files: &[&str]) -> String {
    let mut hasher = Sha1::new();
    for file in files {
        let bytes =
            std::fs::read(file).unwrap_or_else(|error| panic!("failed to read {file}: {error}"));
        let file_hash = {
            let mut file_hasher = Sha1::new();
            file_hasher.update(&bytes);
            hex::encode(file_hasher.finalize())
        };
        hasher.update(file_hash.len().to_le_bytes());
        hasher.update(file_hash.as_bytes());
    }
    hex::encode(hasher.finalize())
}
