//! Architecture guard: teaching material must import only the `sand` façade.
//!
//! `examples/book_project` is the canonical source for book snippets, and
//! `book/src` is written against it — if either imported internal crates,
//! the book would teach the wrong dependency model.

use std::path::{Path, PathBuf};

/// Internal crates that must never appear in `use` statements of guarded trees.
const FORBIDDEN_IMPORTS: &[&str] = &[
    "use sand_core::",
    "use sand_macros::",
    "use sand_commands::",
    "use sand_components::",
    "use sand_version::",
];

/// Directories (relative to the repo root) and the file extension to scan
/// within each, whose sources are guarded.
const GUARDED_ROOTS: &[(&str, &str)] = &[("examples/book_project/src", "rs"), ("book/src", "md")];

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is <repo>/sand for this test crate.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("sand crate lives inside the repo")
        .to_path_buf()
}

fn sources_with_extension(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {dir:?}: {e}")) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            sources_with_extension(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

#[test]
fn guarded_sources_import_only_the_facade() {
    let root = repo_root();
    let mut violations = Vec::new();

    for (guarded, ext) in GUARDED_ROOTS {
        let dir = root.join(guarded);
        assert!(dir.is_dir(), "guarded root missing: {dir:?}");

        let mut files = Vec::new();
        sources_with_extension(&dir, ext, &mut files);
        assert!(!files.is_empty(), "no .{ext} files under {dir:?}");

        for file in files {
            let source =
                std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
            for (idx, line) in source.lines().enumerate() {
                for forbidden in FORBIDDEN_IMPORTS {
                    if line.trim_start().starts_with(forbidden) {
                        violations.push(format!("{}:{}: {}", file.display(), idx + 1, line.trim()));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "internal-crate imports found in façade-only sources:\n{}",
        violations.join("\n")
    );
}
