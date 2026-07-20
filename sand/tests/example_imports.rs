//! Architecture guard: teaching material must import only the `sand` façade,
//! and canonical examples must author commands through typed builders
//! rather than handwritten Minecraft command strings.
//!
//! `examples/book_project` and `examples/participant_audit` are canonical,
//! façade-only teaching/validation packs — the book's own snippets are
//! written against `book_project`, and `participant_audit` is studied
//! directly by #265's runtime-validation procedure — so both are held to a
//! zero-unapproved-raw-usage standard here. `book/src` prose is checked for
//! internal-crate imports only (its raw-escape-hatch chapters are prose
//! *about* `cmd::raw`, not Rust source, and a naive text scan there would
//! flag legitimate teaching content).
//!
//! `sand-example` is intentionally excluded: its own module doc describes it
//! as "the primary integration test for the Sand workspace" and it imports
//! internal crates (`sand_core`, `sand_macros`, ...) directly by design —
//! it is a compiler/integration-test crate, not a façade-only teaching
//! example, so holding it to this guard would be a category error, not a
//! fix. If it is ever repurposed as user-facing teaching material, add it
//! here.

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
/// within each, whose sources are guarded for internal-crate imports.
const GUARDED_ROOTS: &[(&str, &str)] = &[
    ("examples/book_project/src", "rs"),
    ("examples/participant_audit/src", "rs"),
    ("book/src", "md"),
];

/// Rust source roots held to the stricter zero-raw-command standard: no
/// `cmd::raw`/`RawCommand`/`RawComponent`, and no handwritten Minecraft
/// command string literals. `sand::advanced` is not pattern-checked here —
/// both canonical examples' sole use of it is the standard
/// `__sand_export`-calls-`sand::advanced` scaffold pattern (see
/// `examples/book_project/src/lib.rs`'s `__sand_export`), which a
/// substring grep cannot distinguish from a hypothetical non-scaffold use
/// without false positives; review new `sand::advanced` call sites by hand.
const ZERO_RAW_USAGE_ROOTS: &[&str] = &[
    "examples/book_project/src",
    "examples/participant_audit/src",
];

/// Substrings that indicate a raw command escape hatch or a handwritten,
/// fully/near-fully rendered Minecraft command inside a Rust string
/// literal. Scoped to Rust authoring source under [`ZERO_RAW_USAGE_ROOTS`]
/// only — never run against prose/fixtures — to keep false positives out.
const FORBIDDEN_RAW_PATTERNS: &[&str] = &[
    "cmd::raw(",
    "RawCommand::",
    "RawComponent::",
    "\"scoreboard players",
    "\"scoreboard objectives",
    "\"data modify storage",
    "\"data modify entity",
    "\"data get storage",
    "\"data remove storage",
    "\"execute at ",
    "\"execute if ",
    "\"execute unless ",
    "\"execute store",
    "\"execute as ",
    "\"execute positioned",
];

/// Files allowed to contain [`FORBIDDEN_RAW_PATTERNS`] matches, with the
/// reason inline. Keep this list small — it exists for the narrow case of
/// an example *intentionally* teaching the raw escape hatch, or the one
/// sanctioned `sand::advanced` call inside the standard `__sand_export`
/// scaffold every canonical example's `src/bin/sand_export.rs` calls into.
const RAW_USAGE_ALLOWLIST: &[(&str, &str)] = &[(
    "examples/book_project/src/lib.rs",
    "`claim_striders` deliberately teaches `cmd::raw` as the escape hatch \
     for giving a fully-configured custom `ItemStack` — no typed \"give an \
     item stack\" command builder exists in Sand today.",
)];

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

#[test]
fn canonical_examples_use_typed_command_builders_not_raw_strings() {
    let root = repo_root();
    let mut violations = Vec::new();

    for guarded in ZERO_RAW_USAGE_ROOTS {
        let dir = root.join(guarded);
        assert!(dir.is_dir(), "guarded root missing: {dir:?}");

        let mut files = Vec::new();
        sources_with_extension(&dir, "rs", &mut files);
        assert!(!files.is_empty(), "no .rs files under {dir:?}");

        for file in files {
            let rel = file
                .strip_prefix(&root)
                .expect("file under repo root")
                .to_string_lossy()
                .replace('\\', "/");
            if let Some((_, reason)) = RAW_USAGE_ALLOWLIST.iter().find(|(f, _)| *f == rel) {
                let _ = reason; // documented above; nothing further to check for this file.
                continue;
            }

            let source =
                std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
            for (idx, line) in source.lines().enumerate() {
                for forbidden in FORBIDDEN_RAW_PATTERNS {
                    if line.contains(forbidden) {
                        violations.push(format!("{}:{}: {}", file.display(), idx + 1, line.trim()));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "raw command usage found in zero-raw-usage canonical examples \
         (add a narrowly-justified entry to RAW_USAGE_ALLOWLIST if this is \
         an intentional escape-hatch teaching example):\n{}",
        violations.join("\n")
    );
}
